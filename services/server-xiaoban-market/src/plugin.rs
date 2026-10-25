//! WorkBuddy 插件格式：`.codebuddy-plugin/plugin.json`、`agents/*.md`、`mcp.json`、`ai.workbuddy/token-schema.json`。
//!
//! 规范来自本机 WorkBuddy 自带的 skill-expert-manager（references/plugin-json-spec.md、agent-md-spec.md、
//! team-spec.md、avatar-spec.md）。两边不一致时以 WorkBuddy 为准，见 docs/architecture/workbuddy-alignment.md。
//! 客户端 `libs/agent-tools/skill-discovery/src/plugin.rs` 有同样的解析，允许重复实现，改动时两边一起改。
use crate::package::{self, Unpacked};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// WorkBuddy 的 12 个行业分类：(id, 中文名, 英文名)。
pub const CATEGORIES: [(&str, &str, &str); 12] = [
    ("01-ProductDesign", "产品设计", "Product Design"),
    ("02-Engineering", "技术工程", "Engineering"),
    ("03-GameSpatial", "游戏空间", "Game & Spatial"),
    ("04-DataAI", "数据智能", "Data & AI"),
    ("05-MarketingGrowth", "营销增长", "Marketing & Growth"),
    ("06-ContentCreative", "内容创作", "Content & Creative"),
    ("07-SalesCommerce", "销售商务", "Sales & Commerce"),
    ("08-FinanceInvestment", "金融投资", "Finance & Investment"),
    ("09-OperationsHR", "运营人力", "Operations & HR"),
    ("10-ProjectQuality", "项目质量", "Project & Quality"),
    ("11-SecurityCompliance", "法务安全", "Security & Compliance"),
    ("12-IndustryConsultant", "行业顾问", "Industry Consultant"),
];

pub const AVATAR_MAX_BYTES: usize = 500 * 1024;

/// 中英文成对的文本。有的插件只写一种语言，或直接写成字符串，这里都接受并互相补齐。
#[derive(Debug, Clone, Default, Serialize, PartialEq)]
pub struct Bilingual {
    pub en: String,
    pub zh: String,
}

impl<'de> Deserialize<'de> for Bilingual {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Raw {
            Text(String),
            Pair {
                #[serde(default)]
                en: Option<String>,
                #[serde(default)]
                zh: Option<String>,
            },
        }
        Ok(match Raw::deserialize(d)? {
            Raw::Text(s) => Bilingual { en: s.clone(), zh: s },
            Raw::Pair { en, zh } => Bilingual { en: en.unwrap_or_default(), zh: zh.unwrap_or_default() },
        })
    }
}

impl Bilingual {
    pub fn is_complete(&self) -> bool {
        !self.en.trim().is_empty() && !self.zh.trim().is_empty()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TeamInfo {
    pub lead_agent: String,
    pub member_agents: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Member {
    pub id: String,
    pub name: Bilingual,
    pub profession: Bilingual,
    pub avatar: String,
    pub role: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Manifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: Option<Value>,
    pub license: Option<String>,
    pub keywords: Vec<String>,
    pub homepage: Option<Value>,
    pub repository: Option<Value>,
    /// 生命周期钩子。只解析不执行。
    pub hooks: Option<Value>,
    pub agents: Vec<String>,
    pub skills: Vec<String>,
    pub expert_type: String,
    pub agent_name: String,
    pub team_info: Option<TeamInfo>,
    pub display_name: Option<Bilingual>,
    pub profession: Option<Bilingual>,
    pub display_description: Option<Bilingual>,
    pub avatar: String,
    pub category_id: String,
    pub default_init_prompt: Option<Bilingual>,
    pub plugin: String,
    pub tags: Vec<Bilingual>,
    pub quick_prompts: Vec<Bilingual>,
    pub members: Vec<Member>,
    /// `extensions["ai.workbuddy"]` 里有连接器的展示、认证和策略，第 4 期使用。
    pub extensions: Value,
}

pub fn parse_manifest(bytes: &[u8]) -> Result<Manifest, String> {
    serde_json::from_slice(bytes).map_err(|e| format!("plugin.json 无效：{e}"))
}

/// agent md 头部（`---` 之间）加正文。
#[derive(Debug, Clone, Default, Serialize)]
pub struct AgentDoc {
    pub name: String,
    pub description: String,
    pub display_name: Option<Bilingual>,
    pub profession: Option<Bilingual>,
    pub max_turns: Option<u32>,
    pub skills: Vec<String>,
    /// 头部是否写了 `tools`：规范禁止，工具权限由系统统一分配。
    pub declares_tools: bool,
    pub body: String,
}

fn unquote(v: &str) -> String {
    v.trim().trim_matches(|c| c == '"' || c == '\'').to_string()
}

/// 只处理规范里用到的 YAML 子集：标量、一层 `en:`/`zh:` 嵌套、`[a, b]` 与 `- a` 列表、`>-` / `|` 多行文本。
pub fn parse_agent_md(text: &str) -> Result<AgentDoc, String> {
    let text = text.trim_start_matches('\u{feff}');
    let rest = text.strip_prefix("---\n").or_else(|| text.strip_prefix("---\r\n")).ok_or("agent md 缺少头部（---）")?;
    let mut header = String::new();
    let mut body = None;
    let mut offset = 0;
    for line in rest.split_inclusive('\n') {
        offset += line.len();
        if line.trim_end() == "---" {
            body = Some(rest[offset..].to_string());
            break;
        }
        header.push_str(line);
    }
    let body = body.ok_or("agent md 的头部没有结束（---）")?;
    let lines: Vec<&str> = header.lines().collect();
    let mut doc = AgentDoc { body, ..Default::default() };
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        i += 1;
        if line.trim().is_empty() || line.trim_start().starts_with('#') || line.starts_with(char::is_whitespace) {
            continue;
        }
        let Some((key, value)) = line.split_once(':') else { continue };
        let (key, value) = (key.trim(), value.trim());
        // 收集缩进的后续行。
        let mut block = Vec::new();
        while i < lines.len() && (lines[i].starts_with(char::is_whitespace) || lines[i].trim().is_empty()) {
            block.push(lines[i]);
            i += 1;
        }
        let nested = |name: &str| -> String {
            block.iter().find_map(|l| l.trim().strip_prefix(&format!("{name}:")).map(unquote)).unwrap_or_default()
        };
        let folded = || block.iter().map(|l| l.trim()).filter(|l| !l.is_empty()).collect::<Vec<_>>().join(" ");
        let scalar = || if matches!(value, ">" | ">-" | "|" | "|-") { folded() } else { unquote(value) };
        match key {
            "name" => doc.name = scalar(),
            "description" => doc.description = scalar(),
            "displayName" => doc.display_name = Some(Bilingual { en: nested("en"), zh: nested("zh") }),
            "profession" => doc.profession = Some(Bilingual { en: nested("en"), zh: nested("zh") }),
            "maxTurns" => doc.max_turns = value.parse().ok(),
            "tools" => doc.declares_tools = true,
            "skills" => {
                let inline = value.strip_prefix('[').and_then(|v| v.strip_suffix(']'));
                doc.skills = match inline {
                    Some(list) => list.split(',').map(unquote).filter(|s| !s.is_empty()).collect(),
                    None => block.iter().filter_map(|l| l.trim().strip_prefix("- ").map(unquote)).collect(),
                };
            }
            _ => {}
        }
    }
    if doc.name.is_empty() {
        return Err("agent md 头部缺少 name".into());
    }
    Ok(doc)
}

/// 连接器的 MCP 服务配置（`mcp.json`）。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct McpServer {
    /// `streamable-http`、`sse` 或 `stdio`；没写时有 command 按 stdio，否则按 streamable-http。
    #[serde(rename = "type")]
    pub transport: String,
    pub url: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: BTreeMap<String, String>,
    pub headers: BTreeMap<String, String>,
}

pub fn parse_mcp(bytes: &[u8]) -> Result<BTreeMap<String, McpServer>, String> {
    let v: Value = serde_json::from_slice(bytes).map_err(|e| format!("mcp.json 无效：{e}"))?;
    let servers = v.get("mcpServers").cloned().unwrap_or(v);
    let mut out: BTreeMap<String, McpServer> = serde_json::from_value(servers).map_err(|e| format!("mcp.json 无效：{e}"))?;
    for server in out.values_mut() {
        if server.transport.is_empty() {
            server.transport = if server.command.is_empty() { "streamable-http".into() } else { "stdio".into() };
        }
    }
    Ok(out)
}

/// `ai.workbuddy/token-schema.json`：令牌填写表单。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TokenSchema {
    pub title: String,
    pub title_en: String,
    pub description: String,
    pub description_en: String,
    pub doc_url: String,
    pub doc_label: String,
    pub doc_label_en: String,
    pub fields: Vec<TokenField>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct TokenField {
    pub key: String,
    pub label: String,
    pub placeholder: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub required: bool,
    pub description: String,
}

pub fn parse_token_schema(bytes: &[u8]) -> Result<TokenSchema, String> {
    serde_json::from_slice(bytes).map_err(|e| format!("token-schema.json 无效：{e}"))
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Issue {
    pub error: bool,
    pub message: String,
}

fn kebab(s: &str) -> bool {
    !s.is_empty()
        && s.split('-').all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit()))
}

fn semver(s: &str) -> bool {
    let parts: Vec<&str> = s.split('.').collect();
    parts.len() == 3 && parts.iter().all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
}

fn rel(path: &str) -> &str {
    path.strip_prefix("./").unwrap_or(path)
}

/// 按 WorkBuddy 规范检查专家（agent / team）插件。结构性问题一律是错误；
/// 展示规范（3 个标签、3 条提问、40–50 字描述、默认引导语等于第一条提问）在 `lenient` 时降为警告，
/// 用于导入别处发布的插件。
pub fn validate_expert(m: &Manifest, files: &BTreeMap<String, Vec<u8>>, lenient: bool) -> Vec<Issue> {
    let mut issues = Vec::new();
    let mut err = |message: String| issues.push(Issue { error: true, message });
    let mut spec_issues: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    if !kebab(&m.name) {
        err("name 必须是小写字母、数字和连字符".into());
    }
    if m.version.trim().is_empty() {
        err("缺少 version".into());
    } else if !semver(&m.version) {
        // 实测 WorkBuddy 上架的专家有 `2026.10.23+novice.12` 这样的版本号，不强求三段式。
        spec_issues.push("version 规范是 MAJOR.MINOR.PATCH".into());
    }
    if m.hooks.as_ref().is_some_and(|h| !h.is_null()) {
        // 钩子会执行插件自带的命令，小伴不运行，只在审查时提示。
        warnings.push("plugin.json 声明了 hooks：小伴不会执行插件钩子".into());
    }
    if m.description.trim().is_empty() {
        err("缺少 description".into());
    }
    if m.plugin != m.name {
        err("plugin 必须与 name 一致".into());
    }
    if !matches!(m.expert_type.as_str(), "agent" | "team") {
        err("expertType 只能是 agent 或 team".into());
    }
    if m.agent_name.is_empty() || m.agent_name == "team-lead" {
        err("agentName 必须有业务语义，不能为空或 team-lead".into());
    }
    if m.agents.is_empty() {
        err("agents 不能为空".into());
    }

    let mut agent_names = Vec::new();
    for path in &m.agents {
        let key = rel(path);
        let stem = key.rsplit('/').next().unwrap_or(key).trim_end_matches(".md").to_string();
        let Some(raw) = files.get(key) else {
            err(format!("找不到 agent 文件：{path}"));
            continue;
        };
        match std::str::from_utf8(raw).map_err(|_| "不是 UTF-8".to_string()).and_then(parse_agent_md) {
            Ok(doc) => {
                if doc.name != stem {
                    err(format!("{path} 头部的 name「{}」必须与文件名一致", doc.name));
                }
                if doc.declares_tools {
                    err(format!("{path} 的头部不能声明 tools，工具权限由系统统一分配"));
                }
                if doc.description.is_empty() {
                    err(format!("{path} 头部缺少 description"));
                }
                if doc.body.trim().is_empty() {
                    err(format!("{path} 没有正文"));
                }
            }
            Err(e) => err(format!("{path}：{e}")),
        }
        agent_names.push(stem);
    }
    if !m.agent_name.is_empty() && !agent_names.contains(&m.agent_name) {
        err(format!("agentName「{}」在 agents 里没有对应文件", m.agent_name));
    }

    for path in &m.skills {
        let key = rel(path).trim_end_matches('/');
        if !files.contains_key(&format!("{key}/SKILL.md")) {
            err(format!("找不到技能：{path}（需要 {key}/SKILL.md）"));
        }
    }

    match m.avatar.as_str() {
        "" => err("缺少 avatar（头像）".into()),
        path => match files.get(rel(path)) {
            None => err(format!("找不到头像：{path}")),
            // 实测 WorkBuddy 自己发布的专家头像也有超过 500 KB 的，所以这条只是展示规范，不是硬限制。
            Some(bytes) if bytes.len() > AVATAR_MAX_BYTES => spec_issues.push(format!("头像 {} KB，规范建议不超过 500 KB", bytes.len() / 1024)),
            Some(_) if !matches!(rel(path).rsplit('.').next(), Some("png" | "jpg" | "jpeg")) => err("头像必须是 PNG 或 JPG".into()),
            _ => {}
        },
    }

    if !CATEGORIES.iter().any(|(id, _, _)| *id == m.category_id) {
        err(format!("categoryId「{}」不在 WorkBuddy 的 12 个行业分类里", m.category_id));
    }
    for (label, value) in [("displayName", &m.display_name), ("profession", &m.profession), ("displayDescription", &m.display_description)] {
        match value {
            Some(v) if v.is_complete() => {}
            _ => err(format!("{label} 需要中英文都填写")),
        }
    }

    if m.tags.len() != 3 || !m.tags.iter().all(Bilingual::is_complete) {
        spec_issues.push("tags 必须是固定 3 个，且中英文齐全".into());
    }
    if m.quick_prompts.len() != 3 || !m.quick_prompts.iter().all(Bilingual::is_complete) {
        spec_issues.push("quickPrompts 必须是固定 3 条，且中英文齐全".into());
    }
    match (&m.default_init_prompt, m.quick_prompts.first()) {
        (Some(d), Some(q)) if d.zh == q.zh && (d.en.is_empty() || q.en.is_empty() || d.en == q.en) => {}
        _ => spec_issues.push("defaultInitPrompt 必须与第一条 quickPrompts 一致".into()),
    }
    if let Some(d) = &m.display_description {
        let n = d.zh.chars().count();
        if !(40..=50).contains(&n) {
            spec_issues.push(format!("displayDescription 的中文应为 40–50 字，现在是 {n} 字"));
        }
    }

    if m.expert_type == "team" {
        let Some(info) = &m.team_info else {
            err("team 型必须填写 teamInfo".into());
            return finish(issues, spec_issues, warnings, lenient);
        };
        if info.lead_agent.is_empty() || info.lead_agent == "team-lead" || !agent_names.contains(&info.lead_agent) {
            err("teamInfo.leadAgent 必须是 agents 里的一个，且不能叫 team-lead".into());
        }
        if info.member_agents.is_empty() {
            err("teamInfo.memberAgents 不能为空".into());
        }
        for member in &info.member_agents {
            if !agent_names.contains(member) {
                err(format!("成员 {member} 在 agents 里没有对应文件"));
            }
            if *member == info.lead_agent {
                err("teamInfo.memberAgents 不能包含主理人".into());
            }
        }
        let leads: Vec<&Member> = m.members.iter().filter(|x| x.role == "lead").collect();
        if leads.len() != 1 || leads[0].id != info.lead_agent {
            err("members 里必须有且只有一个 role 为 lead 的主理人，且 id 等于 leadAgent".into());
        }
        for member in &m.members {
            if !matches!(member.role.as_str(), "lead" | "member") {
                err(format!("成员 {} 的 role 只能是 lead 或 member", member.id));
            }
            if !member.avatar.is_empty() && !files.contains_key(rel(&member.avatar)) {
                err(format!("找不到成员 {} 的头像：{}", member.id, member.avatar));
            }
        }
    }
    finish(issues, spec_issues, warnings, lenient)
}

fn finish(mut issues: Vec<Issue>, spec: Vec<String>, warnings: Vec<String>, lenient: bool) -> Vec<Issue> {
    issues.extend(spec.into_iter().map(|message| Issue { error: !lenient, message }));
    issues.extend(warnings.into_iter().map(|message| Issue { error: false, message }));
    issues
}

/// 解包后的 WorkBuddy 插件包。
#[derive(Debug)]
pub struct PluginPackage {
    pub unpacked: Unpacked,
    pub manifest: Manifest,
    pub agents: BTreeMap<String, AgentDoc>,
}

/// 通用校验（路径、大小、数量）之后读出清单和 agent md；不检查专家规范，需要的话再调 `validate_expert`。
pub fn inspect_plugin_zip(bytes: &[u8]) -> Result<PluginPackage, String> {
    let unpacked = package::unpack_zip_limited(bytes, package::MAX_PLUGIN_FILES)?;
    let raw = package::MANIFEST_DIRS
        .iter()
        .find_map(|dir| unpacked.contents.get(&format!("{dir}/plugin.json")))
        .ok_or("插件包缺少 .codebuddy-plugin/plugin.json")?;
    let manifest = parse_manifest(raw)?;
    let mut agents = BTreeMap::new();
    for path in &manifest.agents {
        let key = rel(path).to_string();
        if let Some(text) = unpacked.contents.get(&key).and_then(|b| std::str::from_utf8(b).ok()) {
            if let Ok(doc) = parse_agent_md(text) {
                agents.insert(key, doc);
            }
        }
    }
    Ok(PluginPackage { unpacked, manifest, agents })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    const AGENT: &str = "---\nname: accounting-supervisor\ndescription: \"Senior accountant.\"\ndisplayName:\n  en: \"AI Accounting Supervisor\"\n  zh: \"AI 会计主管\"\nprofession:\n  en: \"Accounting Supervisor\"\n  zh: \"会计主管\"\nmaxTurns: 50\n---\n\n# AI 会计主管\n\n你是会计主管。\n";

    fn manifest_json() -> String {
        serde_json::json!({
            "name": "accounting-supervisor", "version": "1.0.0", "description": "AI accounting supervisor.",
            "agents": ["./agents/accounting-supervisor.md"], "expertType": "agent", "agentName": "accounting-supervisor",
            "displayName": {"en": "AI Accounting Supervisor", "zh": "AI 会计主管"},
            "profession": {"en": "Accounting Supervisor", "zh": "会计主管"},
            "displayDescription": {"en": "Senior accountant.", "zh": "聚焦会计核算（凭证/账簿/报表/税务）与财务分析（比率/趋势/预算/经营诊断），输出严谨可落地成果。"},
            "avatar": "avatars/expert.png", "categoryId": "08-FinanceInvestment",
            "defaultInitPrompt": {"zh": "请审核这笔分录", "en": "Review this entry"},
            "plugin": "accounting-supervisor",
            "tags": [{"en": "A", "zh": "甲"}, {"en": "B", "zh": "乙"}, {"en": "C", "zh": "丙"}],
            "quickPrompts": [{"en": "Review this entry", "zh": "请审核这笔分录"}, {"en": "Two", "zh": "二"}, {"en": "Three", "zh": "三"}]
        })
        .to_string()
    }

    fn files() -> BTreeMap<String, Vec<u8>> {
        BTreeMap::from([
            ("agents/accounting-supervisor.md".to_string(), AGENT.as_bytes().to_vec()),
            ("avatars/expert.png".to_string(), vec![0x89, b'P', b'N', b'G']),
        ])
    }

    fn zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut w = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        let opts = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
        for (name, data) in entries {
            w.start_file(*name, opts).unwrap();
            w.write_all(data).unwrap();
        }
        w.finish().unwrap().into_inner()
    }

    #[test]
    fn parses_agent_headers_with_nested_names_and_block_text() {
        let doc = parse_agent_md(AGENT).unwrap();
        assert_eq!((doc.name.as_str(), doc.max_turns), ("accounting-supervisor", Some(50)));
        assert_eq!(doc.display_name.unwrap().zh, "AI 会计主管");
        assert_eq!(doc.profession.unwrap().en, "Accounting Supervisor");
        assert!(doc.body.contains("你是会计主管"));
        let folded = parse_agent_md("---\nname: x\ndescription: >-\n  first line\n  second line\nskills: [a, \"b\"]\ntools: [Bash]\n---\nbody").unwrap();
        assert_eq!(folded.description, "first line second line");
        assert_eq!(folded.skills, ["a", "b"]);
        assert!(folded.declares_tools);
        assert!(parse_agent_md("no header").is_err());
        assert!(parse_agent_md("---\ndescription: x\n---\nbody").is_err());
    }

    #[test]
    fn a_conforming_expert_has_no_issues() {
        let m = parse_manifest(manifest_json().as_bytes()).unwrap();
        assert_eq!(validate_expert(&m, &files(), false), vec![]);
    }

    #[test]
    fn structural_problems_are_errors_and_spec_problems_soften_when_lenient() {
        let mut m = parse_manifest(manifest_json().as_bytes()).unwrap();
        m.tags.pop();
        m.category_id = "99-Made".into();
        m.agent_name = "team-lead".into();
        let strict = validate_expert(&m, &files(), false);
        assert!(strict.iter().any(|i| i.error && i.message.contains("tags")));
        assert!(strict.iter().any(|i| i.error && i.message.contains("categoryId")));
        assert!(strict.iter().any(|i| i.error && i.message.contains("agentName")));
        let lenient = validate_expert(&m, &files(), true);
        let tags = lenient.iter().find(|i| i.message.contains("tags")).unwrap();
        assert!(!tags.error, "展示规范在宽松模式下只是警告");
        assert!(lenient.iter().any(|i| i.error && i.message.contains("categoryId")), "分类仍是错误");
        // 头部声明 tools、缺头像、缺技能。
        let mut bad = files();
        bad.insert("agents/accounting-supervisor.md".into(), AGENT.replace("maxTurns: 50", "tools: [Bash]").into_bytes());
        bad.remove("avatars/expert.png");
        let m = parse_manifest(manifest_json().as_bytes()).unwrap();
        let issues = validate_expert(&m, &bad, false);
        assert!(issues.iter().any(|i| i.message.contains("tools")));
        assert!(issues.iter().any(|i| i.message.contains("头像")));
    }

    #[test]
    fn teams_need_a_lead_members_and_matching_roles() {
        let mut m = parse_manifest(manifest_json().as_bytes()).unwrap();
        m.expert_type = "team".into();
        assert!(validate_expert(&m, &files(), false).iter().any(|i| i.message.contains("teamInfo")));
        m.team_info = Some(TeamInfo { lead_agent: "accounting-supervisor".into(), member_agents: vec!["accounting-supervisor".into()] });
        m.members = vec![Member { id: "accounting-supervisor".into(), role: "lead".into(), ..Default::default() }];
        let issues = validate_expert(&m, &files(), false);
        assert!(issues.iter().any(|i| i.message.contains("不能包含主理人")));
    }

    #[test]
    fn plugin_zips_keep_their_manifest_directory_and_reject_other_hidden_paths() {
        let ok = zip(&[(".codebuddy-plugin/plugin.json", manifest_json().as_bytes()), ("agents/accounting-supervisor.md", AGENT.as_bytes()), ("avatars/expert.png", b"png")]);
        let pkg = inspect_plugin_zip(&ok).unwrap();
        assert_eq!(pkg.manifest.name, "accounting-supervisor");
        assert!(pkg.agents.contains_key("agents/accounting-supervisor.md"));
        assert!(pkg.unpacked.files.iter().any(|f| f.path == ".codebuddy-plugin/plugin.json"));
        // 包着一层目录也行。
        let wrapped = zip(&[("pkg/.codebuddy-plugin/plugin.json", manifest_json().as_bytes()), ("pkg/agents/accounting-supervisor.md", AGENT.as_bytes())]);
        assert!(inspect_plugin_zip(&wrapped).is_ok());
        for bad in [".hidden/plugin.json", "agents/.codebuddy-plugin/x", ".codebuddy-plugin/.env", ".codebuddy-plugin/../x"] {
            assert!(package::unpack_zip(&zip(&[("a.md", b"x"), (bad, b"x")])).is_err(), "{bad} 应被拒绝");
        }
        assert!(inspect_plugin_zip(&zip(&[("agents/a.md", b"x")])).is_err(), "没有 plugin.json");
    }

    #[test]
    fn mcp_and_token_schema_files_parse_like_workbuddys() {
        let remote = parse_mcp(br#"{"mcpServers":{"a":{"type":"streamable-http","url":"https://mcp.example.com/mcp","headers":{"client":"WorkBuddy"}}}}"#).unwrap();
        assert_eq!(remote["a"].transport, "streamable-http");
        assert_eq!(remote["a"].headers["client"], "WorkBuddy");
        let stdio = parse_mcp(br#"{"mcpServers":{"b":{"command":"npx","args":["-y","pkg"],"env":{"NO_PROXY":"localhost"}}}}"#).unwrap();
        assert_eq!((stdio["b"].transport.as_str(), stdio["b"].args.len()), ("stdio", 2));
        let schema = parse_token_schema(r#"{"title":"连接","docUrl":"https://x","fields":[{"key":"API_KEY","label":"密钥","type":"password","required":true}]}"#.as_bytes()).unwrap();
        assert_eq!((schema.fields[0].key.as_str(), schema.fields[0].required), ("API_KEY", true));
    }

    /// 参照物：本机安装的 WorkBuddy 自带的专家和连接器，按它们自己的目录结构读进来再校验。
    /// 没装 WorkBuddy 的机器上直接跳过；测试只读，不把对方的文件放进仓库。
    #[test]
    fn real_workbuddy_plugins_pass_our_parser_and_validator() {
        let Some(home) = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")) else { return };
        let root = std::path::Path::new(&home).join(".workbuddy/plugins");
        fn read_dir(base: &std::path::Path, dir: &std::path::Path, out: &mut BTreeMap<String, Vec<u8>>) {
            let Ok(entries) = std::fs::read_dir(dir) else { return };
            for entry in entries.flatten() {
                let path = entry.path();
                let name = path.strip_prefix(base).unwrap().to_string_lossy().replace('\\', "/");
                if name.starts_with(".in_use") || name.starts_with(".downloaded") || name.starts_with(".orphaned") {
                    continue;
                }
                if path.is_dir() {
                    read_dir(base, &path, out);
                } else if let Ok(bytes) = std::fs::read(&path) {
                    out.insert(name, bytes);
                }
            }
        }
        // 已安装的专家是 cache/experts/<名>/<版本>/，市场里的是 marketplaces/experts/plugins/<名>/。
        let mut dirs = Vec::new();
        if let Ok(names) = std::fs::read_dir(root.join("cache/experts")) {
            for name in names.flatten() {
                if let Ok(versions) = std::fs::read_dir(name.path()) {
                    dirs.extend(versions.flatten().map(|v| v.path()));
                }
            }
        }
        if let Ok(names) = std::fs::read_dir(root.join("marketplaces/experts/plugins")) {
            dirs.extend(names.flatten().map(|n| n.path()));
        }
        let mut seen = 0;
        for dir in dirs {
            {
                let mut files = BTreeMap::new();
                read_dir(&dir, &dir, &mut files);
                let manifest = parse_manifest(files.get(".codebuddy-plugin/plugin.json").expect("plugin.json")).unwrap();
                // 宽松模式：展示规范（例如头像大小）WorkBuddy 自己发布的也不一定遵守，结构问题必须没有。
                let issues = validate_expert(&manifest, &files, true);
                let errors: Vec<_> = issues.iter().filter(|i| i.error).collect();
                assert!(errors.is_empty(), "{} 不该有结构错误：{errors:?}", manifest.name);
                for warning in issues.iter().filter(|i| !i.error) {
                    eprintln!("{}: {}", manifest.name, warning.message);
                }
                seen += 1;
            }
        }
        eprintln!("checked {seen} installed WorkBuddy experts");
    }
}
