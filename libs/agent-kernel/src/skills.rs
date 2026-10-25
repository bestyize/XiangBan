//! Skills: Markdown instruction files the model loads on demand.
//!
//! Layout: `<root>/<name>/SKILL.md` starting with a `---` block holding `name:` and `description:`.
//! Only the name and description go into the prompt; the body is read by `load_skill`.
//! Symlinked skill directories are ignored.

use crate::builtin::{opt_str_arg, str_arg};
use crate::tool::{Risk, Tool, ToolCtx, ToolSpec};
use crate::util::{atomic_write, now_millis, valid_slug};
use serde_json::{json, Value};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;

const MAX_SKILLS: usize = 200;
const MAX_FILE_BYTES: u64 = 100 * 1024;
const MAX_BODY_BYTES: usize = 20_000;
const MAX_DESCRIPTION_CHARS: usize = 500;
const INDEX_BUDGET: usize = 8000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillInfo {
    pub name: String,
    pub description: String,
    pub dir: PathBuf,
    /// Written by `skill_manage`; only these can be changed or deleted by the model.
    pub agent_created: bool,
    pub writable: bool,
    /// `user-invocable: false` hides the skill from the user's `/` menu (default true).
    pub user_invocable: bool,
    /// `disable-model-invocation: true` keeps the skill out of the model's index, search and `load_skill`;
    /// only the user can start it (default: the model may use it).
    pub model_invocable: bool,
    /// `allowed-tools`, as declared. Shown to the user; it never grants a permission by itself.
    pub allowed_tools: Vec<String>,
    pub display_name: Option<String>,
    pub display_name_en: Option<String>,
    pub description_zh: Option<String>,
    pub description_en: Option<String>,
    pub version: Option<String>,
    pub author: Option<String>,
    pub category: Option<String>,
    pub tags: Vec<String>,
}

struct Root {
    path: PathBuf,
    writable: bool,
}

/// Skill roots in priority order: when two roots hold the same name, the earlier root wins.
pub struct Skills {
    roots: Vec<Root>,
}

impl Skills {
    pub fn new() -> Self {
        Self { roots: Vec::new() }
    }

    /// Bundled or shared skills. Never modified by the kernel.
    pub fn read_only_root(mut self, path: impl Into<PathBuf>) -> Self {
        self.roots.push(Root { path: path.into(), writable: false });
        self
    }

    /// Where `skill_manage` saves new skills. Without one, the tool is not offered.
    pub fn writable_root(mut self, path: impl Into<PathBuf>) -> Self {
        self.roots.push(Root { path: path.into(), writable: true });
        self
    }

    pub fn list(&self) -> Vec<SkillInfo> {
        let mut found = self.scan();
        found.truncate(MAX_SKILLS);
        found
    }

    /// Search must include skills omitted from the bounded initial listing.
    fn scan(&self) -> Vec<SkillInfo> {
        let mut found: Vec<SkillInfo> = Vec::new();
        for root in &self.roots {
            let Ok(read) = fs::read_dir(&root.path) else { continue };
            let mut names: Vec<String> = read
                .flatten()
                .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
                .filter_map(|e| e.file_name().into_string().ok())
                .filter(|n| valid_slug(n))
                .collect();
            names.sort();
            for name in names {
                if found.iter().any(|s| s.name == name) {
                    continue;
                }
                if let Some(info) = read_info(root, &name) {
                    found.push(info);
                }
            }
        }
        found.sort_by(|a, b| a.name.cmp(&b.name));
        found
    }

    pub fn find(&self, name: &str) -> Option<SkillInfo> {
        if !valid_slug(name) {
            return None;
        }
        self.roots.iter().find_map(|root| read_info(root, name))
    }

    /// Skills the user may start with `/name`.
    pub fn invocable(&self) -> Vec<SkillInfo> {
        self.scan().into_iter().filter(|s| s.user_invocable).collect()
    }

    /// The skill body for the model's `load_skill`. A skill marked `disable-model-invocation` is refused:
    /// only the user can start it.
    pub fn load(&self, name: &str) -> Result<String, String> {
        if self.find(name).is_some_and(|i| !i.model_invocable) {
            return Err(format!("`{name}` can only be started by the user (they type /{name}); do not load it yourself."));
        }
        self.load_any(name)
    }

    /// The body of a skill the user started with `/name`. A skill marked `user-invocable: false` is refused.
    pub fn load_for_user(&self, name: &str) -> Result<String, String> {
        if self.find(name).is_some_and(|i| !i.user_invocable) {
            return Err(format!("`{name}` is not meant to be started by hand"));
        }
        self.load_any(name)
    }

    /// The skill body with a short header, whoever started it. The host uses it for skills a persona preloads.
    pub fn load_any(&self, name: &str) -> Result<String, String> {
        let Some(info) = self.find(name) else {
            // A folder that is there but cannot be listed: say why, rather than that it does not exist.
            let file = valid_slug(name).then(|| self.roots.iter().map(|r| r.path.join(name).join("SKILL.md")).find(|f| f.is_file())).flatten();
            if let Some(file) = file {
                read_limited(&file)?;
                return Err(format!("{} has no valid frontmatter (`name: {name}` and a description)", file.display()));
            }
            return Err(format!("no skill named `{name}`. Available: {}", self.names_hint()));
        };
        let text = read_limited(&info.dir.join("SKILL.md"))?;
        let body = split_frontmatter(&text).map_or(text.as_str(), |(_, body)| body);
        Ok(format!("Skill `{}` (directory: {})\n\n{}", info.name, info.dir.display(), body.trim()))
    }

    fn names_hint(&self) -> String {
        let names: Vec<String> = self.list().into_iter().filter(|s| s.model_invocable).map(|s| s.name).collect();
        if names.is_empty() { "none".to_string() } else { names.join(", ") }
    }

    /// For `AgentBuilder::prompt_section`: the skill index, rebuilt on every model call.
    pub fn prompt_section(self: &Arc<Self>) -> impl Fn() -> String + Send + Sync + 'static {
        let skills = self.clone();
        move || {
            let list: Vec<SkillInfo> = skills.scan().into_iter().filter(|s| s.model_invocable).collect();
            if list.is_empty() {
                return String::new();
            }
            let mut out = String::from(
                "## Skills\nSkills are instructions for specific kinds of tasks. When a task matches a skill, \
                 call `load_skill` with its name before starting, then follow it. Use `skill_find` \
                 to search all installed skills, including any omitted from this index. \
                 `skill_find` is local; `skill_search` searches the online store.",
            );
            let mut hidden = 0;
            for skill in &list {
                let line = format!("\n- {}: {}", skill.name, skill.description);
                if out.len() + line.len() > INDEX_BUDGET {
                    hidden += 1;
                } else {
                    out.push_str(&line);
                }
            }
            if hidden > 0 {
                out.push_str(&format!("\n({hidden} more skills not listed; use `skill_find` to discover them.)"));
            }
            out
        }
    }

    /// `load_skill`, plus `skill_manage` when a writable root was configured.
    pub fn tools(self: &Arc<Self>) -> Vec<Arc<dyn Tool>> {
        let mut tools: Vec<Arc<dyn Tool>> = vec![Arc::new(LoadSkill::new(self.clone())), Arc::new(FindSkill::new(self.clone()))];
        if self.roots.iter().any(|r| r.writable) {
            tools.push(Arc::new(SkillManage::new(self.clone())));
        }
        tools
    }

    fn save_root(&self) -> Option<&Root> {
        self.roots.iter().find(|r| r.writable)
    }
}

impl Default for Skills {
    fn default() -> Self {
        Self::new()
    }
}

fn read_limited(path: &Path) -> Result<String, String> {
    let file = File::open(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let mut bytes = Vec::new();
    file.take(MAX_FILE_BYTES + 1).read_to_end(&mut bytes).map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_FILE_BYTES {
        return Err(format!("{} is larger than {} KiB", path.display(), MAX_FILE_BYTES / 1024));
    }
    String::from_utf8(bytes).map_err(|_| format!("{} is not valid UTF-8", path.display()))
}

/// Metadata shared by installation validation and the installed-skill index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillMetadata {
    pub name: String,
    pub description: String,
    pub origin: Option<String>,
    pub user_invocable: bool,
    pub model_invocable: bool,
    pub allowed_tools: Vec<String>,
    pub display_name: Option<String>,
    pub display_name_en: Option<String>,
    pub description_zh: Option<String>,
    pub description_en: Option<String>,
    pub version: Option<String>,
    pub author: Option<String>,
    pub category: Option<String>,
    pub tags: Vec<String>,
}

impl SkillMetadata {
    /// Name and description only, with the defaults of every optional field.
    pub fn basic(name: impl Into<String>, description: impl Into<String>, origin: Option<String>) -> Self {
        Self {
            name: name.into(), description: description.into(), origin,
            user_invocable: true, model_invocable: true, allowed_tools: Vec::new(),
            display_name: None, display_name_en: None, description_zh: None, description_en: None,
            version: None, author: None, category: None, tags: Vec::new(),
        }
    }
}

/// WorkBuddy writes these as booleans, but a hand-edited file may say `"false"` or `no`.
fn yaml_flag(value: &Option<serde_yaml_ng::Value>) -> Option<bool> {
    match value.as_ref()? {
        serde_yaml_ng::Value::Bool(b) => Some(*b),
        serde_yaml_ng::Value::String(s) => match s.trim().to_ascii_lowercase().as_str() {
            "true" | "yes" | "on" => Some(true),
            "false" | "no" | "off" => Some(false),
            _ => None,
        },
        _ => None,
    }
}

/// Strings, numbers (`version: 1.0`) and booleans all read as text; blank means absent.
fn yaml_text(value: &Option<serde_yaml_ng::Value>) -> Option<String> {
    let text = match value.as_ref()? {
        serde_yaml_ng::Value::String(s) => s.clone(),
        serde_yaml_ng::Value::Number(n) => n.to_string(),
        serde_yaml_ng::Value::Bool(b) => b.to_string(),
        _ => return None,
    };
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    (!text.is_empty()).then_some(text)
}

/// A list, or one string separated by commas or spaces (`allowed-tools: Read, Grep`).
fn yaml_list(value: &Option<serde_yaml_ng::Value>) -> Vec<String> {
    match value.as_ref() {
        Some(serde_yaml_ng::Value::Sequence(items)) => items.iter().filter_map(|i| yaml_text(&Some(i.clone()))).collect(),
        Some(serde_yaml_ng::Value::String(s)) => s.split(|c: char| c == ',' || c.is_whitespace()).map(str::trim).filter(|t| !t.is_empty()).map(String::from).collect(),
        _ => Vec::new(),
    }
}

fn yaml_scalar_string<'de,D:serde::Deserializer<'de>>(deserializer:D)->Result<String,D::Error>{
    let value:serde_yaml_ng::Value=serde::Deserialize::deserialize(deserializer)?;
    match value {
        serde_yaml_ng::Value::String(value)=>Ok(value),
        _=>Err(serde::de::Error::custom("metadata field must be a string")),
    }
}

fn yaml_scalar_opt<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Option<String>, D::Error> {
    let value: Option<serde_yaml_ng::Value> = serde::Deserialize::deserialize(deserializer)?;
    match value {
        None | Some(serde_yaml_ng::Value::Null) => Ok(None),
        Some(serde_yaml_ng::Value::String(value)) => Ok(Some(value)),
        _ => Err(serde::de::Error::custom("metadata field must be a string")),
    }
}

/// Parse standard YAML metadata, preserving the original pinned document bytes.
pub fn parse_skill_metadata(text: &str) -> Result<SkillMetadata, String> {
    parse_skill_metadata_named(text, None)
}

/// Like `parse_skill_metadata`, but a header without `name` takes `fallback_name` (the directory name),
/// as WorkBuddy and Claude Code do for third-party skills that leave it out.
pub fn parse_skill_metadata_named(text: &str, fallback_name: Option<&str>) -> Result<SkillMetadata, String> {
    let (header, _) = split_frontmatter(text)
        .ok_or("SKILL.md requires a closed --- frontmatter block")?;
    #[derive(serde::Deserialize)]
    struct Metadata {
        #[serde(default, deserialize_with="yaml_scalar_opt")]
        name: Option<String>,
        #[serde(deserialize_with="yaml_scalar_string")]
        description: String,
        origin: Option<String>,
        // WorkBuddy / Claude Code fields. Read loosely: a malformed optional field never invalidates the skill.
        #[serde(default, rename = "user-invocable", alias = "user_invocable", alias = "userInvocable")]
        user_invocable: Option<serde_yaml_ng::Value>,
        #[serde(default, rename = "disable-model-invocation", alias = "disable_model_invocation", alias = "disableModelInvocation")]
        disable_model_invocation: Option<serde_yaml_ng::Value>,
        #[serde(default, rename = "allowed-tools", alias = "allowed_tools", alias = "allowedTools")]
        allowed_tools: Option<serde_yaml_ng::Value>,
        #[serde(default, alias = "displayName")]
        display_name: Option<serde_yaml_ng::Value>,
        #[serde(default, alias = "displayNameEn")]
        display_name_en: Option<serde_yaml_ng::Value>,
        #[serde(default)]
        description_zh: Option<serde_yaml_ng::Value>,
        #[serde(default)]
        description_en: Option<serde_yaml_ng::Value>,
        #[serde(default)]
        version: Option<serde_yaml_ng::Value>,
        #[serde(default)]
        author: Option<serde_yaml_ng::Value>,
        #[serde(default)]
        category: Option<serde_yaml_ng::Value>,
        #[serde(default)]
        tags: Option<serde_yaml_ng::Value>,
    }
    // A number of published skills use legacy unquoted descriptions containing ': '.
    // Quote only these scalar fields in the parsing buffer, never in the installed file.
    let mut normalized = String::new();
    let mut lines = header.lines().peekable();
    while let Some(line) = lines.next() {
        let field = if !line.starts_with(char::is_whitespace) { line.split_once(':') } else { None };
        if let Some((key, value)) = field {
            let value = value.trim();
            if matches!(key.trim(), "name" | "description" | "description_zh" | "description_en" | "display_name" | "display_name_en" | "title" | "author")
                && !value.starts_with(['\'', '"', '|', '>', '[', '{', '&', '*', '!'])
                && (value.contains(": ") || value.ends_with(':')) {
                let mut value = value.to_string();
                while lines.peek().is_some_and(|l| l.starts_with(' ')) {
                    value.push(' '); value.push_str(lines.next().unwrap().trim());
                }
                normalized.push_str(key); normalized.push_str(": ");
                normalized.push_str(&serde_json::to_string(&value).map_err(|e|e.to_string())?);
                normalized.push('\n');
                continue;
            }
        }
        normalized.push_str(line); normalized.push('\n');
    }
    let raw: Metadata = serde_yaml_ng::from_str(&normalized)
        .map_err(|e|format!("SKILL.md frontmatter YAML invalid: {e}"))?;
    let Some(name) = raw.name.filter(|n| !n.trim().is_empty()).or_else(|| fallback_name.map(String::from)) else {
        return Err("SKILL.md frontmatter name is missing or empty".into());
    };
    if raw.description.trim().is_empty() { return Err("SKILL.md frontmatter description is missing or empty".into()); }
    Ok(SkillMetadata {
        user_invocable: yaml_flag(&raw.user_invocable).unwrap_or(true),
        model_invocable: !yaml_flag(&raw.disable_model_invocation).unwrap_or(false),
        allowed_tools: yaml_list(&raw.allowed_tools),
        display_name: yaml_text(&raw.display_name),
        display_name_en: yaml_text(&raw.display_name_en),
        description_zh: yaml_text(&raw.description_zh),
        description_en: yaml_text(&raw.description_en),
        version: yaml_text(&raw.version),
        author: yaml_text(&raw.author),
        category: yaml_text(&raw.category),
        tags: yaml_list(&raw.tags),
        ..SkillMetadata::basic(name, raw.description.split_whitespace().collect::<Vec<_>>().join(" "), raw.origin)
    })
}

/// Locate the complete leading YAML block, without inspecting its nested values.
fn split_frontmatter(text: &str) -> Option<(&str, &str)> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let opening = text.split_inclusive('\n').next()?;
    if !opening.ends_with('\n') || opening.trim_end() != "---" { return None; }
    let rest = &text[opening.len()..];
    let mut offset = 0;
    for line in rest.split_inclusive('\n') {
        if line.trim_end() == "---" { return Some((&rest[..offset], &rest[offset + line.len()..])); }
        offset += line.len();
    }
    None
}

fn read_info(root: &Root, name: &str) -> Option<SkillInfo> {
    let dir = root.path.join(name);
    if !fs::symlink_metadata(&dir).ok()?.is_dir() {
        return None;
    }
    let text = read_limited(&dir.join("SKILL.md")).ok()?;
    let metadata = match parse_skill_metadata_named(&text, Some(name)) {
        Ok(metadata)=>metadata,
        Err(_) if !text.trim_start_matches('\u{feff}').starts_with("---")=>{
            let lock:Value=serde_json::from_str(&read_limited(&dir.join(".skillhub-lock.json")).ok()?).ok()?;
            if lock["provider"]!="skillhub" || lock["slug"]!=name || lock["metadata"]["name"]!=name {return None;}
            let description=lock["metadata"]["description"].as_str().filter(|s|!s.trim().is_empty())?;
            SkillMetadata::basic(name, description.split_whitespace().collect::<Vec<_>>().join(" "), None)
        },
        Err(_)=>return None,
    };
    if metadata.name != name {
        return None;
    }
    Some(SkillInfo {
        name: name.to_string(),
        description: metadata.description,
        dir,
        agent_created: metadata.origin.as_deref() == Some("agent"),
        writable: root.writable,
        user_invocable: metadata.user_invocable,
        model_invocable: metadata.model_invocable,
        allowed_tools: metadata.allowed_tools,
        display_name: metadata.display_name,
        display_name_en: metadata.display_name_en,
        description_zh: metadata.description_zh,
        description_en: metadata.description_en,
        version: metadata.version,
        author: metadata.author,
        category: metadata.category,
        tags: metadata.tags,
    })
}

struct LoadSkill {
    spec: ToolSpec,
    skills: Arc<Skills>,
}

impl LoadSkill {
    fn new(skills: Arc<Skills>) -> Self {
        let schema = json!({ "type": "object", "properties": { "name": { "type": "string" } }, "required": ["name"] });
        let description = "Load the full instructions of an installed skill from the Skills index or skill_find results.";
        Self { spec: ToolSpec::new("load_skill", description, schema).risk(Risk::Read).parallel(), skills }
    }
}

impl Tool for LoadSkill {
    fn spec(&self) -> &ToolSpec {
        &self.spec
    }

    fn execute(&self, args: Value, _ctx: &ToolCtx) -> Result<String, String> {
        self.skills.load(str_arg(&args, "name")?.trim())
    }
}

struct FindSkill {
    spec: ToolSpec,
    skills: Arc<Skills>,
}

impl FindSkill {
    fn new(skills: Arc<Skills>) -> Self {
        let schema = json!({"type":"object","properties":{
            "query":{"type":"string","minLength":1,"maxLength":200},
            "page":{"type":"integer","minimum":1},
            "limit":{"type":"integer","minimum":1,"maximum":20}
        },"required":["query"],"additionalProperties":false});
        Self { spec: ToolSpec::new("skill_find", "Search names and descriptions of all installed skills, including skills omitted from the prompt index. Local metadata only: no network, installation, body loading or script execution. Use load_skill on the selected name.", schema).risk(Risk::Read).parallel(), skills }
    }
}

impl Tool for FindSkill {
    fn spec(&self) -> &ToolSpec { &self.spec }

    fn execute(&self, args: Value, ctx: &ToolCtx) -> Result<String, String> {
        if ctx.is_cancelled() { return Err("cancelled".into()); }
        let query = str_arg(&args, "query")?.trim();
        if query.is_empty() { return Err("query must not be blank".into()); }
        let limit = args["limit"].as_u64().unwrap_or(10).clamp(1, 20) as usize;
        let page = args["page"].as_u64().unwrap_or(1).max(1);
        let offset = page.saturating_sub(1).saturating_mul(limit as u64).min(usize::MAX as u64) as usize;
        let mut matches: Vec<_> = self.skills.scan().into_iter().filter(|s| s.model_invocable).filter_map(|skill| {
            let score = crate::discovery::relevance(query, &skill.name, &skill.description);
            (score > 0).then_some((score, skill))
        }).collect();
        matches.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.name.cmp(&b.1.name)));
        let total = matches.len();
        let results: Vec<_> = matches.into_iter().skip(offset).take(limit).map(|(_, s)| json!({"name":s.name,"description":s.description.chars().take(160).collect::<String>()})).collect();
        Ok(json!({"skills":results,"total":total,"page":page,"hasMore":offset.saturating_add(limit)<total,"next_step":"Use load_skill with an exact returned name before following that skill."}).to_string())
    }
}

struct SkillManage {
    spec: ToolSpec,
    skills: Arc<Skills>,
}

impl SkillManage {
    fn new(skills: Arc<Skills>) -> Self {
        let schema = json!({
            "type": "object",
            "properties": {
                "action": { "type": "string", "enum": ["create", "patch", "delete"] },
                "name": { "type": "string", "description": "Lowercase letters, digits and hyphens, at most 64 characters." },
                "description": { "type": "string", "description": format!("One line saying what the skill does and when to use it, at most {MAX_DESCRIPTION_CHARS} characters. Required for create.") },
                "content": { "type": "string", "description": format!("Markdown instructions, 1 to {MAX_BODY_BYTES} bytes. Required for create.") },
                "old_string": { "type": "string", "description": "For patch: exact text in the body to replace; must occur once." },
                "new_string": { "type": "string", "description": "For patch: the replacement text." }
            },
            "required": ["action", "name"]
        });
        let description = "Save, improve or remove a reusable skill. The user must approve every call. Only create a skill \
                           for a procedure worth reusing, keep it short and specific, and never put secrets in it. You can \
                           only change or delete skills you created yourself.";
        let spec = ToolSpec::new("skill_manage", description, schema).risk(Risk::Write).serial().always_ask();
        Self { spec, skills }
    }

    fn require_own(&self, name: &str) -> Result<SkillInfo, String> {
        let info = self.skills.find(name).ok_or_else(|| format!("no skill named `{name}`"))?;
        if !info.writable || !info.agent_created {
            return Err(format!("`{name}` was not created by the assistant and cannot be changed here. Create a new skill with a different name instead."));
        }
        Ok(info)
    }

    fn create(&self, args: &Value, name: &str) -> Result<String, String> {
        let description = str_arg(args, "description")?.trim();
        let content = str_arg(args, "content")?.trim();
        if description.is_empty() || description.chars().count() > MAX_DESCRIPTION_CHARS || description.chars().any(char::is_control) {
            return Err(format!("`description` must be one line of 1 to {MAX_DESCRIPTION_CHARS} characters"));
        }
        if content.is_empty() || content.len() > MAX_BODY_BYTES {
            return Err(format!("`content` must be 1 to {MAX_BODY_BYTES} bytes"));
        }
        if self.skills.find(name).is_some() {
            return Err(format!("a skill named `{name}` already exists; choose another name"));
        }
        let root = self.skills.save_root().ok_or("no writable skill directory is configured")?;
        fs::create_dir_all(&root.path).map_err(|e| format!("cannot create skill directory: {e}"))?;
        let dir = root.path.join(name);
        fs::create_dir(&dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
        // Quote descriptions so colons, leading # and quotes remain scalar text.
        let description = serde_json::to_string(description).map_err(|e| e.to_string())?;
        let text = format!("---\nname: {name}\ndescription: {description}\norigin: agent\n---\n\n{content}\n");
        atomic_write(&dir.join("SKILL.md"), text.as_bytes()).map_err(|e| {
            let _ = fs::remove_dir(&dir);
            format!("cannot write skill: {e}")
        })?;
        Ok(format!("Created skill `{name}`."))
    }

    fn patch(&self, args: &Value, name: &str) -> Result<String, String> {
        let info = self.require_own(name)?;
        let old = str_arg(args, "old_string")?;
        let new = opt_str_arg(args, "new_string")?.unwrap_or("");
        if old.is_empty() {
            return Err("`old_string` must not be empty".to_string());
        }
        let path = info.dir.join("SKILL.md");
        let text = read_limited(&path)?;
        let (_, body) = split_frontmatter(&text).ok_or("the skill file has no frontmatter")?;
        let header = &text[..text.len() - body.len()];
        let count = body.matches(old).count();
        if count != 1 {
            return Err(if count == 0 { "`old_string` was not found in the skill body".to_string() } else { format!("`old_string` occurs {count} times; include more context so it is unique") });
        }
        let new_body = body.replacen(old, new, 1);
        if new_body.len() > MAX_BODY_BYTES {
            return Err(format!("the skill body would exceed {MAX_BODY_BYTES} bytes"));
        }
        self.backup(&info, &text)?;
        atomic_write(&path, format!("{header}{new_body}").as_bytes()).map_err(|e| format!("cannot write skill: {e}"))?;
        Ok(format!("Updated skill `{name}`."))
    }

    fn delete(&self, name: &str) -> Result<String, String> {
        let info = self.require_own(name)?;
        let path = info.dir.join("SKILL.md");
        let text = read_limited(&path)?;
        self.backup(&info, &text)?;
        fs::remove_file(&path).map_err(|e| format!("cannot delete skill: {e}"))?;
        let note = if fs::remove_dir(&info.dir).is_ok() { "" } else { " Other files in its directory were left in place." };
        Ok(format!("Deleted skill `{name}`.{note}"))
    }

    fn backup(&self, info: &SkillInfo, text: &str) -> Result<(), String> {
        let history = info.dir.parent().ok_or("invalid skill directory")?.join(".history");
        fs::create_dir_all(&history).map_err(|e| format!("cannot save a backup: {e}"))?;
        atomic_write(&history.join(format!("{}.{}.md", info.name, now_millis())), text.as_bytes()).map_err(|e| format!("cannot save a backup: {e}"))
    }
}

impl Tool for SkillManage {
    fn spec(&self) -> &ToolSpec {
        &self.spec
    }

    fn execute(&self, args: Value, _ctx: &ToolCtx) -> Result<String, String> {
        let name = str_arg(&args, "name")?.trim();
        if !valid_slug(name) {
            return Err("`name` must be 1 to 64 characters of lowercase letters, digits and hyphens, not starting or ending with a hyphen".to_string());
        }
        match str_arg(&args, "action")? {
            "create" => self.create(&args, name),
            "patch" => self.patch(&args, name),
            "delete" => self.delete(name),
            other => Err(format!("unknown action `{other}`; use create, patch or delete")),
        }
    }
}

#[cfg(test)]
mod frontmatter_tests {
    use super::*;
    #[test]
    fn headerless_skills_load_only_with_matching_registry_metadata() {
        let root=std::env::temp_dir().join(format!("headerless-skill-{}-{}",std::process::id(),now_millis()));
        let dir=root.join("legacy-skill");fs::create_dir_all(&dir).unwrap();
        let body="# Legacy skill\nOriginal body, unchanged.";
        fs::write(dir.join("SKILL.md"),body).unwrap();
        let skills=Skills::new().writable_root(&root);
        assert!(skills.find("legacy-skill").is_none());
        let mut lock=json!({"provider":"skillhub","slug":"legacy-skill","metadata":{"name":"legacy-skill","description":"Verified registry description"}});
        fs::write(dir.join(".skillhub-lock.json"),lock.to_string()).unwrap();
        let info=skills.find("legacy-skill").unwrap();
        assert!(!info.agent_created);
        assert!(skills.load("legacy-skill").unwrap().ends_with(body));
        lock["slug"]=json!("different");
        fs::write(dir.join(".skillhub-lock.json"),lock.to_string()).unwrap();
        assert!(skills.find("legacy-skill").is_none());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn parses_common_scalar_formats_without_reading_body_metadata() {
        for (field, expected) in [
            ("'Use it''s data # safely' # comment", "Use it's data # safely"),
            ("\"Read \\\"quoted\\\" text\" # comment", "Read \"quoted\" text"),
            ("Plain text # comment", "Plain text"),
            (">-\n  Read the report\n  and summarize it.", "Read the report and summarize it."),
            ("| # comment\n  Read the report\n  and summarize it.", "Read the report and summarize it."),
            ("\n  Read the report\n  and summarize it.", "Read the report and summarize it."),
        ] {
            let text = format!("---\nmetadata:\n  name: ignored\nname: 'demo-skill'\ndescription: {field}\n---\nname: body-only");
            let metadata = parse_skill_metadata(&text).unwrap();
            assert_eq!(metadata.name, "demo-skill");
            assert_eq!(metadata.description, expected);
        }
    }

    /// A header in the shape WorkBuddy's own `pptx` skill uses.
    const WORKBUDDY_HEADER: &str = "---\nname: pptx\ndisplay_name: \"PPT 演示文稿\"\ndisplay_name_en: \"PowerPoint\"\ndescription: >-\n  Use for anything touching .pptx.\n  凡是涉及 .pptx 的任务都用它。\ndescription_zh: \"创建、读取、编辑 .pptx 演示文稿。\"\ndescription_en: Creates decks: slides, layouts and notes.\nversion: \"1.0.0\"\nauthor: zxh\ncategory: document\ndisable-model-invocation: false\nuser-invocable: true\ntitle: \"PPT 演示文稿 / PowerPoint\"\ntags: [\"PPT\", \"pptx\", \"幻灯片\"]\nallowed-tools: Read, Grep\n---\nBody";

    #[test]
    fn reads_the_workbuddy_skill_fields() {
        let m = parse_skill_metadata(WORKBUDDY_HEADER).unwrap();
        assert_eq!((m.name.as_str(), m.display_name.as_deref(), m.display_name_en.as_deref()), ("pptx", Some("PPT 演示文稿"), Some("PowerPoint")));
        assert_eq!((m.description_zh.as_deref(), m.version.as_deref(), m.author.as_deref(), m.category.as_deref()), (Some("创建、读取、编辑 .pptx 演示文稿。"), Some("1.0.0"), Some("zxh"), Some("document")));
        assert_eq!((m.user_invocable, m.model_invocable), (true, true));
        assert_eq!(m.tags, ["PPT", "pptx", "幻灯片"]);
        assert_eq!(m.allowed_tools, ["Read", "Grep"]);
        // Unquoted colons in the optional fields, a numeric version, a list-form tools field and string flags.
        let loose = "---\nname: x\ndescription: d\ndescription_en: Reads files: fast and safe\nversion: 1.2\nallowed-tools:\n  - Read\n  - Bash(ls:*)\nuser-invocable: \"false\"\ndisable-model-invocation: yes\n---\n";
        let m = parse_skill_metadata(loose).unwrap();
        assert_eq!((m.description_en.as_deref(), m.version.as_deref()), (Some("Reads files: fast and safe"), Some("1.2")));
        assert_eq!(m.allowed_tools, ["Read", "Bash(ls:*)"]);
        assert_eq!((m.user_invocable, m.model_invocable), (false, false));
        // A skill with no optional fields keeps the defaults, and a malformed one never invalidates the skill.
        let plain = parse_skill_metadata("---\nname: x\ndescription: d\n---\n").unwrap();
        assert_eq!(plain, SkillMetadata::basic("x", "d", None));
        let odd = parse_skill_metadata("---\nname: x\ndescription: d\nuser-invocable: [1]\ntags: 5\nversion: {a: 1}\n---\n").unwrap();
        assert_eq!((odd.user_invocable, odd.version, odd.tags), (true, None, Vec::<String>::new()));
    }

    /// Reference: every SKILL.md of the WorkBuddy installed on this machine must read. Skipped when there is none;
    /// the files are only read, never copied into the repository.
    #[test]
    fn every_skill_of_the_local_workbuddy_reads() {
        let Some(home) = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")) else { return };
        let base = Path::new(&home).join(".workbuddy");
        fn walk(dir: &Path, depth: usize, out: &mut Vec<PathBuf>) {
            let Ok(read) = fs::read_dir(dir) else { return };
            for entry in read.flatten() {
                let path = entry.path();
                if path.is_dir() && depth < 9 {
                    walk(&path, depth + 1, out);
                } else if path.file_name().is_some_and(|n| n == "SKILL.md") {
                    out.push(path);
                }
            }
        }
        let mut files = Vec::new();
        walk(&base.join("skills"), 0, &mut files);
        walk(&base.join("plugins"), 0, &mut files);
        let (mut ok, mut failed, mut headerless) = (0, Vec::new(), 0);
        for file in &files {
            let Ok(text) = fs::read_to_string(file) else { continue };
            if !text.trim_start_matches('\u{feff}').starts_with("---") {
                headerless += 1;
                continue;
            }
            let dir_name = file.parent().and_then(|p| p.file_name()).and_then(|n| n.to_str());
            match parse_skill_metadata_named(&text, dir_name) {
                Ok(_) => ok += 1,
                Err(e) => failed.push(format!("{}: {e}", file.display())),
            }
        }
        eprintln!("local WorkBuddy skills: {ok} read, {headerless} without a header, {} failed", failed.len());
        for line in failed.iter().take(10) {
            eprintln!("  {line}");
        }
        // Third-party skills with broken YAML exist in WorkBuddy's own marketplace; a missing `name` must not be one of the reasons.
        assert!(failed.iter().all(|f| !f.contains("missing field `name`") && !f.contains("name is missing")), "{failed:?}");
    }

    #[test]
    fn a_header_without_a_name_takes_the_directory_name() {
        let text = "---\ndescription: Needs no name\n---\nBody";
        assert!(parse_skill_metadata(text).is_err(), "installation checks stay strict");
        let m = parse_skill_metadata_named(text, Some("from-dir")).unwrap();
        assert_eq!(m.name, "from-dir");
        let root = std::env::temp_dir().join(format!("nameless-skill-{}-{}", std::process::id(), now_millis()));
        let dir = root.join("from-dir");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("SKILL.md"), text).unwrap();
        let skills = Skills::new().read_only_root(&root);
        assert_eq!(skills.find("from-dir").unwrap().name, "from-dir");
        assert!(skills.load("from-dir").unwrap().ends_with("Body"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn user_only_and_model_only_skills_are_kept_apart() {
        let root = std::env::temp_dir().join(format!("invocation-skills-{}-{}", std::process::id(), now_millis()));
        for (name, flags) in [("both", ""), ("user-only", "disable-model-invocation: true\n"), ("model-only", "user-invocable: false\n")] {
            let dir = root.join(name);
            fs::create_dir_all(&dir).unwrap();
            fs::write(dir.join("SKILL.md"), format!("---\nname: {name}\ndescription: the {name} skill\n{flags}---\nBody of {name}")).unwrap();
        }
        let skills = Arc::new(Skills::new().read_only_root(&root));
        let index = skills.prompt_section()();
        assert!(index.contains("- both:") && index.contains("- model-only:") && !index.contains("user-only"), "{index}");
        let found = FindSkill::new(skills.clone()).execute(json!({"query": "skill"}), &ToolCtx::detached("t")).unwrap();
        assert!(found.contains("model-only") && !found.contains("user-only"), "{found}");
        // The model's load_skill refuses a user-only skill and says why; the user's own invocation works.
        let refused = skills.load("user-only").unwrap_err();
        assert!(refused.contains("/user-only"), "{refused}");
        assert!(skills.load_for_user("user-only").unwrap().ends_with("Body of user-only"));
        assert!(skills.load("model-only").unwrap().ends_with("Body of model-only"));
        assert!(skills.load_for_user("model-only").is_err());
        assert!(skills.load_any("user-only").is_ok() && skills.load_any("model-only").is_ok());
        let menu: Vec<String> = skills.invocable().into_iter().map(|s| s.name).collect();
        assert_eq!(menu, ["both", "user-only"]);
        // A skill the model cannot use is not offered as a hint when a name is wrong either.
        assert!(!skills.load("missing").unwrap_err().contains("user-only"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn parses_multiline_quotes_and_legacy_colons_without_rewriting_source() {
        let quoted = "---\nname: demo-skill\ndescription: \"First line\n  second line with \\\"quotes\\\"\"\nmetadata:\n  settings: {enabled: true, tags: [one, two]}\n---\nBody";
        assert_eq!(parse_skill_metadata(quoted).unwrap().description, "First line second line with \"quotes\"");
        let legacy = "---\nname: demo-skill\ndescription: Follow the workflow: read, analyze, deliver\n---\nBody";
        assert_eq!(parse_skill_metadata(legacy).unwrap().description, "Follow the workflow: read, analyze, deliver");
        assert_eq!(split_frontmatter(legacy).unwrap().1, "Body");
    }

    #[test]
    fn rejects_missing_duplicate_nested_and_empty_metadata() {
        for header in [
            "description: example",
            "name: demo-skill",
            "name: demo-skill\ndescription: ''",
            "name: demo-skill\ndescription: >-\n  ",
            "name: demo-skill\ndescription: null",
            "name: demo-skill\nname: other\ndescription: example",
            "metadata:\n  name: demo-skill\n  description: example",
            "name: demo-skill\ndescription: *alias",
            "name: demo-skill\ndescription: \"unterminated",
        ] {
            let text = format!("---\n{header}\n---\nname: demo-skill\ndescription: body cannot supply metadata");
            assert!(parse_skill_metadata(&text).is_err(), "{header}");
        }
        assert!(parse_skill_metadata("---\nname: demo-skill\ndescription: missing closing delimiter").is_err());
    }

    #[test]
    fn reads_long_bom_crlf_headers_and_loads_the_original_body() {
        let root = std::env::temp_dir().join(format!("skill-metadata-{}-{}", std::process::id(), now_millis()));
        let dir = root.join("demo-skill");
        fs::create_dir_all(&dir).unwrap();
        let padding = "# extra metadata\r\n".repeat(600);
        let text = format!("\u{feff}---\r\n{padding}name: 'demo-skill'\r\ndescription: >-\r\n  Read reports\r\n  and summarize.\r\n---\r\n\r\nOriginal body.\r\n");
        fs::write(dir.join("SKILL.md"), &text).unwrap();
        let skills = Skills::new().read_only_root(&root);
        let info = skills.find("demo-skill").unwrap();
        assert_eq!(info.description, "Read reports and summarize.");
        assert_eq!(skills.list().len(), 1);
        let loaded = skills.load("demo-skill").unwrap();
        assert!(loaded.ends_with("Original body."));
        assert!(!loaded.contains("# extra metadata"));
        assert_eq!(fs::read_to_string(dir.join("SKILL.md")).unwrap(), text);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn created_skills_with_special_descriptions_still_load_and_patch() {
        let root = std::env::temp_dir().join(format!("skill-create-metadata-{}-{}", std::process::id(), now_millis()));
        let skills = Arc::new(Skills::new().writable_root(&root));
        let manager = SkillManage::new(skills.clone());
        let description = "# Use \"reports\": it's useful";
        manager.create(&json!({"description": description, "content": "Original body"}), "demo-skill").unwrap();
        let info = skills.find("demo-skill").unwrap();
        assert_eq!(info.description, description);
        assert!(info.agent_created);
        manager.patch(&json!({"old_string": "Original body", "new_string": "Updated body"}), "demo-skill").unwrap();
        assert!(skills.load("demo-skill").unwrap().ends_with("Updated body"));
        assert_eq!(skills.find("demo-skill").unwrap().description, description);
        fs::remove_dir_all(root).unwrap();
    }
}
