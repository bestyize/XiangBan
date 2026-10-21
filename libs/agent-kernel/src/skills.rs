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
        found.truncate(MAX_SKILLS);
        found
    }

    pub fn find(&self, name: &str) -> Option<SkillInfo> {
        if !valid_slug(name) {
            return None;
        }
        self.roots.iter().find_map(|root| read_info(root, name))
    }

    /// The skill body with a short header, ready to hand to the model.
    pub fn load(&self, name: &str) -> Result<String, String> {
        let info = self.find(name).ok_or_else(|| format!("no skill named `{name}`. Available: {}", self.names_hint()))?;
        let text = read_limited(&info.dir.join("SKILL.md"))?;
        let body = split_frontmatter(&text).map_or(text.as_str(), |(_, body)| body);
        Ok(format!("Skill `{}` (directory: {})\n\n{}", info.name, info.dir.display(), body.trim()))
    }

    fn names_hint(&self) -> String {
        let names: Vec<String> = self.list().into_iter().map(|s| s.name).collect();
        if names.is_empty() { "none".to_string() } else { names.join(", ") }
    }

    /// For `AgentBuilder::prompt_section`: the skill index, rebuilt on every model call.
    pub fn prompt_section(self: &Arc<Self>) -> impl Fn() -> String + Send + Sync + 'static {
        let skills = self.clone();
        move || {
            let list = skills.list();
            if list.is_empty() {
                return String::new();
            }
            let mut out = String::from(
                "## Skills\nSkills are instructions for specific kinds of tasks. When a task matches a skill, \
                 call `load_skill` with its name before starting, then follow it.",
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
                out.push_str(&format!("\n({hidden} more skills not listed; `load_skill` still works for them.)"));
            }
            out
        }
    }

    /// `load_skill`, plus `skill_manage` when a writable root was configured.
    pub fn tools(self: &Arc<Self>) -> Vec<Arc<dyn Tool>> {
        let mut tools: Vec<Arc<dyn Tool>> = vec![Arc::new(LoadSkill::new(self.clone()))];
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
}

/// Parse the scalar metadata used by Skills, without evaluating YAML tags or aliases.
/// The original document is never rewritten: its bytes may be pinned by a manifest.
pub fn parse_skill_metadata(text: &str) -> Result<SkillMetadata, String> {
    let (pairs, _) = split_frontmatter(text)
        .ok_or("SKILL.md requires a closed --- frontmatter block with unique scalar metadata fields")?;
    let get = |key: &str| pairs.iter().find(|(k, _)| *k == key).map(|(_, v)| v.as_str());
    let name = get("name").filter(|v| !v.trim().is_empty())
        .ok_or("SKILL.md frontmatter name is missing or empty")?;
    let description = get("description").filter(|v| !v.trim().is_empty())
        .ok_or("SKILL.md frontmatter description is missing or empty")?;
    Ok(SkillMetadata {
        name: name.to_string(),
        description: description.split_whitespace().collect::<Vec<_>>().join(" "),
        origin: get("origin").map(str::to_string),
    })
}

/// The supported scalar subset includes quoted values, comments and indented descriptions.
/// Ignore unrelated/nested metadata; only top-level identity fields are authoritative.
fn split_frontmatter(text: &str) -> Option<(Vec<(&str, String)>, &str)> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let opening = text.split_inclusive('\n').next()?;
    if !opening.ends_with('\n') || opening.trim_end() != "---" {
        return None;
    }
    let rest = &text[opening.len()..];
    let mut offset = 0;
    let mut header_end = None;
    for line in rest.split_inclusive('\n') {
        if line.trim_end() == "---" {
            header_end = Some((offset, offset + line.len()));
            break;
        }
        offset += line.len();
    }
    let (end, body_start) = header_end?;
    let mut lines = rest[..end].lines().peekable();
    let mut pairs = Vec::new();
    while let Some(line) = lines.next() {
        if line.starts_with(char::is_whitespace) || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once(':') else { continue };
        let key = key.trim();
        if !matches!(key, "name" | "description" | "origin") {
            continue;
        }
        if pairs.iter().any(|(k, _)| *k == key) {
            return None;
        }
        let value = value.trim();
        let indicator = value.split_whitespace().next().unwrap_or("");
        let scalar = if matches!(indicator, "|" | "|-" | "|+" | ">" | ">-" | ">+")
            || value.is_empty() || value.starts_with('#')
        {
            if !indicator.is_empty() && !indicator.starts_with('#')
                && !value[indicator.len()..].trim_start().is_empty()
                && !value[indicator.len()..].trim_start().starts_with('#') {
                return None;
            }
            let mut block = Vec::new();
            while lines.peek().is_some_and(|l| l.trim().is_empty() || l.starts_with(' ')) {
                block.push(lines.next()?.trim());
            }
            block.join(if indicator.starts_with('|') { "\n" } else { " " })
        } else {
            scalar_value(value)?
        };
        pairs.push((key, scalar));
    }
    Some((pairs, &rest[body_start..]))
}

fn scalar_value(value: &str) -> Option<String> {
    if value.starts_with('"') {
        let mut escaped = false;
        for (index, c) in value.char_indices().skip(1) {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                let suffix = value[index + 1..].trim_start();
                if !suffix.is_empty() && !suffix.starts_with('#') {
                    return None;
                }
                return serde_json::from_str(&value[..index + 1]).ok();
            }
        }
        return None;
    }
    if value.starts_with('\'') {
        let mut chars = value.char_indices().skip(1).peekable();
        while let Some((index, c)) = chars.next() {
            if c != '\'' { continue; }
            if chars.peek().is_some_and(|(_, next)| *next == '\'') {
                chars.next();
                continue;
            }
            let suffix = value[index + 1..].trim_start();
            if !suffix.is_empty() && !suffix.starts_with('#') {
                return None;
            }
            return Some(value[1..index].replace("''", "'"));
        }
        return None;
    }
    if value.starts_with(['[', '{', '&', '*', '!']) {
        return None;
    }
    let end = value.char_indices().find_map(|(index, c)| {
        (c == '#' && (index == 0 || value[..index].ends_with(char::is_whitespace)))
            .then_some(index)
    }).unwrap_or(value.len());
    let scalar = value[..end].trim_end();
    if matches!(scalar, "~" | "null" | "Null" | "NULL") {
        return Some(String::new());
    }
    Some(scalar.to_string())
}

fn read_info(root: &Root, name: &str) -> Option<SkillInfo> {
    let dir = root.path.join(name);
    if !fs::symlink_metadata(&dir).ok()?.is_dir() {
        return None;
    }
    let text = read_limited(&dir.join("SKILL.md")).ok()?;
    let metadata = parse_skill_metadata(&text).ok()?;
    if metadata.name != name {
        return None;
    }
    Some(SkillInfo {
        name: name.to_string(),
        description: metadata.description,
        dir,
        agent_created: metadata.origin.as_deref() == Some("agent"),
        writable: root.writable,
    })
}

struct LoadSkill {
    spec: ToolSpec,
    skills: Arc<Skills>,
}

impl LoadSkill {
    fn new(skills: Arc<Skills>) -> Self {
        let schema = json!({ "type": "object", "properties": { "name": { "type": "string" } }, "required": ["name"] });
        let description = "Load the full instructions of a skill listed under Skills in the system prompt.";
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
                "description": { "type": "string", "description": "One line saying what the skill does and when to use it. Required for create." },
                "content": { "type": "string", "description": "Markdown instructions. Required for create." },
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
