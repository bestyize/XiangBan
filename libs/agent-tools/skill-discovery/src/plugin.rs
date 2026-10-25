//! WorkBuddy 插件包：专家、专家团、技能包和连接器共用的安装格式。
//!
//! 格式见 docs/architecture/workbuddy-alignment.md；两边不一致时以 WorkBuddy 为准。
//! 服务端 `services/server-xiaoban-market/src/plugin.rs` 有同样的解析，允许重复实现，改动时两边一起改。
//! 安装只写文件，不执行插件里的任何代码（包括 plugin.json 的 hooks 和 mcp.json 的命令）。
use crate::package::{valid_path, MANIFEST_DIRS, MAX_ARCHIVE, MAX_FILE, MAX_PLUGIN_FILES, MAX_TOTAL};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, HashSet},
    fs,
    io::{Cursor, Read},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use zip::ZipArchive;

/// 安装回执：写在插件目录里，记录来源、版本和每个文件的摘要。
pub const RECEIPT: &str = ".xiaoban-plugin.json";

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn kebab(s: &str) -> bool {
    !s.is_empty() && s.len() <= 64 && s.split('-').all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit()))
}

fn rel(path: &str) -> &str {
    path.strip_prefix("./").unwrap_or(path)
}

fn unquote(v: &str) -> String {
    v.trim().trim_matches(|c| c == '"' || c == '\'').to_string()
}

/// 中英文成对的文本。有的插件只写一种语言，或直接写成字符串。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Bilingual {
    pub en: String,
    pub zh: String,
}

impl Bilingual {
    fn from_value(v: &Value) -> Option<Self> {
        match v {
            Value::String(s) => Some(Self { en: s.clone(), zh: s.clone() }),
            Value::Object(m) => Some(Self {
                en: m.get("en").and_then(Value::as_str).unwrap_or("").to_string(),
                zh: m.get("zh").and_then(Value::as_str).unwrap_or("").to_string(),
            }),
            _ => None,
        }
    }

    /// 界面语言优先，缺了就用另一种。
    pub fn pick(&self, zh: bool) -> &str {
        match (zh, self.zh.is_empty(), self.en.is_empty()) {
            (true, false, _) | (false, _, true) => &self.zh,
            _ => &self.en,
        }
    }
}

/// agent md：头部加正文（人设提示词）。
#[derive(Debug, Clone, Default)]
pub struct AgentDoc {
    pub name: String,
    pub description: String,
    pub display_name: Option<Bilingual>,
    pub profession: Option<Bilingual>,
    pub max_turns: Option<u32>,
    pub skills: Vec<String>,
    /// 头部写了 `tools` 也不会授予任何工具：工具权限由小伴统一管理。
    pub declares_tools: bool,
    pub body: String,
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
        let mut block = Vec::new();
        while i < lines.len() && (lines[i].starts_with(char::is_whitespace) || lines[i].trim().is_empty()) {
            block.push(lines[i]);
            i += 1;
        }
        let nested = |name: &str| -> String { block.iter().find_map(|l| l.trim().strip_prefix(&format!("{name}:")).map(unquote)).unwrap_or_default() };
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
                doc.skills = match value.strip_prefix('[').and_then(|v| v.strip_suffix(']')) {
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

/// 插件的类型，决定装到哪里、怎么进对话。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// 单个专家（`expertType: agent`）。
    Agent,
    /// 专家团（`expertType: team`）。
    Team,
    /// 带 `mcp.json` 的连接器。
    Connector,
    /// 只有技能的插件。
    Skills,
}

impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Agent => "agent",
            Kind::Team => "team",
            Kind::Connector => "connector",
            Kind::Skills => "skills",
        }
    }
}

#[derive(Debug)]
pub struct FileInfo {
    pub path: String,
    pub size: u64,
    pub sha256: String,
}

/// 通过路径、大小和结构校验后的插件包，内容都在内存里，还没有写盘。
#[derive(Debug)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub description: String,
    pub kind: Kind,
    /// plugin.json 原文解析结果，后续阶段按需取展示字段、认证声明等。
    pub manifest: Value,
    pub files: Vec<FileInfo>,
    pub contents: BTreeMap<String, Vec<u8>>,
    /// 与小伴市场相同的清单摘要：对排序后的 `"<sha256>  <path>\n"` 求 SHA-256。
    pub manifest_sha256: String,
    /// 审查时要给用户看的提示（例如声明了钩子）。
    pub warnings: Vec<String>,
}

impl Package {
    /// 给审查对话框看的摘要：清单摘要用来锁定版本，安装时必须一致。
    pub fn summary(&self) -> Value {
        let list = |key: &str| -> Vec<String> { self.manifest[key].as_array().into_iter().flatten().filter_map(Value::as_str).map(|p| rel(p).to_string()).collect() };
        json!({
            "name": self.name, "version": self.version, "description": self.description, "kind": self.kind.as_str(),
            "manifestSha256": self.manifest_sha256, "warnings": self.warnings,
            "fileCount": self.files.len(), "totalBytes": self.files.iter().map(|f| f.size).sum::<u64>(),
            "files": self.files.iter().map(|f| json!({"path": f.path, "size": f.size, "sha256": f.sha256})).collect::<Vec<_>>(),
            "agents": list("agents"), "skills": list("skills"),
            "hasMcp": self.contents.contains_key("mcp.json") || self.contents.contains_key(".mcp.json"),
            "connectorCount": self.mcp_servers(),
        })
    }

    /// 清单里声明的 MCP 服务个数（mcp.json 或根目录的 .mcp.json），读不出来按 0。
    pub fn mcp_servers(&self) -> usize {
        ["mcp.json", ".mcp.json"]
            .iter()
            .find_map(|key| self.contents.get(*key))
            .and_then(|bytes| serde_json::from_slice::<Value>(bytes).ok())
            .map(|v| v.get("mcpServers").and_then(Value::as_object).map_or_else(|| v.as_object().map_or(0, |m| m.len()), |m| m.len()))
            .unwrap_or(0)
    }

    pub fn agent_docs(&self) -> BTreeMap<String, AgentDoc> {
        let mut out = BTreeMap::new();
        for path in self.manifest["agents"].as_array().into_iter().flatten().filter_map(Value::as_str) {
            let key = rel(path).to_string();
            if let Some(doc) = self.contents.get(&key).and_then(|b| std::str::from_utf8(b).ok()).and_then(|t| parse_agent_md(t).ok()) {
                out.insert(key, doc);
            }
        }
        out
    }
}

/// 解包并校验 ZIP：路径规则、大小和数量限制、符号链接、大小写冲突；允许一层顶层包装目录。
pub fn inspect_zip(bytes: &[u8]) -> Result<Package, String> {
    if bytes.len() > MAX_ARCHIVE {
        return Err("插件包超过 12 MiB".into());
    }
    let mut zip = ZipArchive::new(Cursor::new(bytes)).map_err(|e| format!("不是有效的 ZIP：{e}"))?;
    if zip.len() > MAX_PLUGIN_FILES * 3 {
        return Err("插件包条目过多".into());
    }
    // 先收集所有条目（含目录）；路径规则要在去掉顶层包装目录之后再检查，
    // 否则 `pkg/.codebuddy-plugin/plugin.json` 里的清单目录不再是最外层。
    let mut entries = Vec::new();
    for n in 0..zip.len() {
        let entry = zip.by_index(n).map_err(|e| e.to_string())?;
        let name = entry.name().map_err(|e| format!("ZIP 条目名无效：{e}"))?.into_owned();
        if let Some(mode) = entry.unix_mode() {
            let kind = mode & 0o170000;
            if kind != 0 && kind != 0o100000 && !(entry.is_dir() && kind == 0o040000) {
                return Err(format!("不允许链接和特殊文件：{name}"));
            }
        }
        entries.push((n, name, entry.is_dir()));
    }
    let names: Vec<(usize, String)> = entries.iter().filter(|(_, _, dir)| !dir).map(|(n, name, _)| (*n, name.clone())).collect();
    if names.is_empty() {
        return Err("插件包为空".into());
    }
    if names.len() > MAX_PLUGIN_FILES {
        return Err(format!("插件包文件数超过 {MAX_PLUGIN_FILES}"));
    }
    let first_dir = |n: &str| n.split_once('/').map(|(d, _)| d.to_string());
    let wrapper = first_dir(&names[0].1).filter(|d| !MANIFEST_DIRS.contains(&d.as_str()) && names.iter().all(|(_, n)| first_dir(n).as_deref() == Some(d.as_str())));
    for (_, name, _) in &entries {
        let stripped = match &wrapper {
            Some(d) => name.strip_prefix(&format!("{d}/")).unwrap_or(name.as_str()),
            None => name.as_str(),
        };
        let bare = stripped.trim_end_matches('/');
        if wrapper.as_deref().is_some_and(|d| bare == d) || bare.is_empty() {
            continue;
        }
        if !valid_path(bare) {
            return Err(format!("不安全的文件路径：{name}"));
        }
    }
    let mut seen = HashSet::new();
    let mut total = 0u64;
    let mut files = Vec::new();
    let mut contents = BTreeMap::new();
    for (n, raw) in &names {
        let path = match &wrapper {
            Some(d) => raw[d.len() + 1..].to_string(),
            None => raw.clone(),
        };
        if !valid_path(&path) || !seen.insert(path.to_lowercase()) {
            return Err(format!("不安全或重复的文件路径：{path}"));
        }
        let limit = if path == "SKILL.md" { 100 * 1024 } else { MAX_FILE };
        let mut data = Vec::new();
        zip.by_index(*n).map_err(|e| e.to_string())?.take(limit + 1).read_to_end(&mut data).map_err(|e| format!("无法解压 {path}：{e}"))?;
        if data.len() as u64 > limit {
            return Err(format!("文件过大：{path}"));
        }
        total += data.len() as u64;
        if total > MAX_TOTAL {
            return Err("解压后总大小超过 12 MiB".into());
        }
        files.push(FileInfo { path: path.clone(), size: data.len() as u64, sha256: sha256_hex(&data) });
        contents.insert(path, data);
    }
    files.sort_by(|a, b| a.path.cmp(&b.path));
    let mut digest = Sha256::new();
    for f in &files {
        digest.update(format!("{}  {}\n", f.sha256, f.path).as_bytes());
    }
    let manifest_sha256 = format!("{:x}", digest.finalize());

    let raw = MANIFEST_DIRS.iter().find_map(|d| contents.get(&format!("{d}/plugin.json"))).ok_or("插件包缺少 .codebuddy-plugin/plugin.json")?;
    let manifest: Value = serde_json::from_slice(raw).map_err(|e| format!("plugin.json 无效：{e}"))?;
    let name = manifest["name"].as_str().unwrap_or("").to_string();
    if !kebab(&name) {
        return Err("plugin.json 的 name 必须是小写字母、数字和连字符".into());
    }
    let version = manifest["version"].as_str().unwrap_or("").trim().to_string();
    if version.is_empty() || version.len() > 44 || !version.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'+')) {
        return Err("plugin.json 的 version 无效".into());
    }
    let mut warnings = Vec::new();
    // agents / skills 声明的文件必须在包里；agent md 必须能解析，且 name 与文件名一致。
    for path in manifest["agents"].as_array().into_iter().flatten().filter_map(Value::as_str) {
        let key = rel(path);
        let text = contents.get(key).ok_or_else(|| format!("找不到 agent 文件：{path}"))?;
        let doc = std::str::from_utf8(text).map_err(|_| format!("{path} 不是 UTF-8")).and_then(|t| parse_agent_md(t).map_err(|e| format!("{path}：{e}")))?;
        let stem = key.rsplit('/').next().unwrap_or(key).trim_end_matches(".md");
        if doc.name != stem {
            return Err(format!("{path} 头部的 name「{}」必须与文件名一致", doc.name));
        }
        if doc.declares_tools {
            warnings.push(format!("{path} 的头部声明了 tools：小伴忽略它，工具权限由小伴统一管理"));
        }
    }
    for path in manifest["skills"].as_array().into_iter().flatten().filter_map(Value::as_str) {
        let key = rel(path).trim_end_matches('/');
        if !contents.contains_key(&format!("{key}/SKILL.md")) {
            return Err(format!("找不到技能：{path}（需要 {key}/SKILL.md）"));
        }
    }
    if manifest.get("hooks").is_some_and(|h| !h.is_null()) {
        warnings.push("plugin.json 声明了 hooks：小伴不会执行插件钩子".into());
    }
    let kind = match manifest["expertType"].as_str() {
        Some("agent") => Kind::Agent,
        Some("team") => Kind::Team,
        _ if contents.contains_key("mcp.json") || contents.contains_key(".mcp.json") => Kind::Connector,
        _ => Kind::Skills,
    };
    if matches!(kind, Kind::Agent | Kind::Team) && manifest["agents"].as_array().is_none_or(Vec::is_empty) {
        return Err("专家插件的 agents 不能为空".into());
    }
    Ok(Package { name, version, description: manifest["description"].as_str().unwrap_or("").to_string(), kind, manifest, files, contents, manifest_sha256, warnings })
}

/// 已安装插件的回执。
pub fn read_receipt(root: &Path, name: &str) -> Result<Value, String> {
    if !kebab(name) {
        return Err("无效的插件名".into());
    }
    let path = root.join(name).join(RECEIPT);
    if fs::symlink_metadata(&path).map_err(|e| e.to_string())?.file_type().is_symlink() {
        return Err("不安全的插件回执".into());
    }
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    let v: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    if v["name"] != name {
        return Err("插件回执与目录名不一致".into());
    }
    Ok(v)
}

/// Windows 上刚写完文件的目录常被杀毒软件或索引服务短暂占用，改名会瞬时返回“拒绝访问”；
/// 只对这一种错误重试，其他错误立即返回。
fn rename_retry(from: &Path, to: &Path) -> std::io::Result<()> {
    let mut attempt = 0;
    loop {
        match fs::rename(from, to) {
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied && attempt < 20 => {
                attempt += 1;
                std::thread::sleep(std::time::Duration::from_millis(50 * attempt));
            }
            other => return other,
        }
    }
}

struct Stage(PathBuf);
impl Drop for Stage {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// 安装到 `<root>/<name>/`。`source` 记录来源（市场、本地文件）；`marketplace` 用来拼 `pluginId`（`名称@市场`）。
/// 已装同名插件时：只有本应用装的才会被替换，旧目录移到 `backups`，失败会还原。返回回执。
pub fn install(root: &Path, backups: &Path, pkg: &Package, marketplace: &str, source: Value) -> Result<Value, String> {
    fs::create_dir_all(root).map_err(|e| e.to_string())?;
    let destination = root.join(&pkg.name);
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e| e.to_string())?.as_nanos();
    let stage = Stage(root.join(format!(".plugin-install-{}-{nonce}", std::process::id())));
    fs::create_dir(&stage.0).map_err(|e| e.to_string())?;
    for (path, data) in &pkg.contents {
        let target = stage.0.join(path);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        fs::write(target, data).map_err(|e| e.to_string())?;
    }
    let installed_at = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0);
    let receipt = json!({
        "name": pkg.name, "version": pkg.version, "kind": pkg.kind.as_str(),
        "pluginId": format!("{}@{}", pkg.name, marketplace), "marketplace": marketplace,
        "source": source, "manifestSha256": pkg.manifest_sha256, "installedAt": installed_at,
        "files": pkg.files.iter().map(|f| json!({"path": f.path, "sha256": f.sha256})).collect::<Vec<_>>(),
    });
    fs::write(stage.0.join(RECEIPT), receipt.to_string()).map_err(|e| e.to_string())?;
    if destination.exists() {
        // 只替换本应用装的；别人放进来的目录不动。
        let old = read_receipt(root, &pkg.name).map_err(|_| "同名插件目录不是由小伴安装的，不会被替换".to_string())?;
        fs::create_dir_all(backups).map_err(|e| e.to_string())?;
        let backup = backups.join(format!("{}-{}-{nonce}", pkg.name, old["version"].as_str().unwrap_or("old")));
        rename_retry(&destination, &backup).map_err(|e| e.to_string())?;
        if let Err(e) = rename_retry(&stage.0, &destination) {
            let _ = rename_retry(&backup, &destination);
            return Err(e.to_string());
        }
    } else {
        rename_retry(&stage.0, &destination).map_err(|e| e.to_string())?;
    }
    Ok(receipt)
}

/// 所有已安装插件的回执，按名称排序；读不出回执的目录跳过。
pub fn installed(root: &Path) -> Vec<Value> {
    let Ok(entries) = fs::read_dir(root) else { return Vec::new() };
    let mut out: Vec<Value> = entries
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .filter_map(|e| e.file_name().to_str().and_then(|n| read_receipt(root, n).ok()))
        .collect();
    out.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
    out
}

/// 卸载：只删本应用装的插件目录。
pub fn remove(root: &Path, name: &str) -> Result<(), String> {
    read_receipt(root, name).map_err(|_| "没有找到由小伴安装的这个插件".to_string())?;
    fs::remove_dir_all(root.join(name)).map_err(|e| e.to_string())
}

/// 插件自带的技能目录（`<root>/<name>/skills`），交给技能加载器作为只读根。
pub fn skill_roots(root: &Path) -> Vec<PathBuf> {
    installed(root).iter().filter_map(|r| r["name"].as_str()).map(|n| root.join(n).join("skills")).filter(|p| p.is_dir()).collect()
}

/// 读取已安装专家的某个 agent（`agents/<名>.md`）。
pub fn read_agent(root: &Path, name: &str, agent: &str) -> Result<AgentDoc, String> {
    read_receipt(root, name)?;
    if !kebab(agent) {
        return Err("无效的 agent 名".into());
    }
    let text = fs::read_to_string(root.join(name).join("agents").join(format!("{agent}.md"))).map_err(|e| e.to_string())?;
    parse_agent_md(&text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use zip::write::SimpleFileOptions;

    const AGENT: &str = "---\nname: accounting-supervisor\ndescription: \"Senior accountant.\"\ndisplayName:\n  en: \"AI Accounting Supervisor\"\n  zh: \"AI 会计主管\"\nprofession:\n  en: \"Accounting Supervisor\"\n  zh: \"会计主管\"\nmaxTurns: 50\n---\n\n# AI 会计主管\n\n你是会计主管。\n";
    const SKILL: &str = "---\nname: ledger-check\ndescription: 对账\n---\n\n步骤\n";

    fn manifest() -> String {
        manifest_named("accounting-supervisor")
    }

    fn manifest_named(name: &str) -> String {
        json!({
            "name": name, "version": "1.0.0", "description": "AI accounting supervisor.",
            "agents": ["./agents/accounting-supervisor.md"], "skills": ["./skills/ledger-check"],
            "expertType": "agent", "agentName": "accounting-supervisor",
            "displayName": {"en": "AI Accounting Supervisor", "zh": "AI 会计主管"}
        })
        .to_string()
    }

    fn zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut w = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
        for (name, data) in entries {
            w.start_file(*name, opts).unwrap();
            w.write_all(data).unwrap();
        }
        w.finish().unwrap().into_inner()
    }

    fn expert_zip() -> Vec<u8> {
        zip(&[
            (".codebuddy-plugin/plugin.json", manifest().as_bytes()),
            ("agents/accounting-supervisor.md", AGENT.as_bytes()),
            ("skills/ledger-check/SKILL.md", SKILL.as_bytes()),
            ("README.md", b"readme"),
        ])
    }

    fn temp(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("xb-plugin-{tag}-{}-{}", std::process::id(), SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn agent_headers_parse_like_the_server() {
        let doc = parse_agent_md(AGENT).unwrap();
        assert_eq!((doc.name.as_str(), doc.max_turns), ("accounting-supervisor", Some(50)));
        assert_eq!(doc.display_name.unwrap().pick(true), "AI 会计主管");
        assert!(doc.body.contains("你是会计主管"));
        let folded = parse_agent_md("---\nname: x\ndescription: >-\n  a\n  b\nskills: [p, \"q\"]\ntools: [Bash]\n---\nbody").unwrap();
        assert_eq!((folded.description.as_str(), folded.skills.len(), folded.declares_tools), ("a b", 2, true));
        assert!(parse_agent_md("no header").is_err() && parse_agent_md("---\ndescription: x\n---\n").is_err());
    }

    #[test]
    fn an_expert_zip_is_inspected_with_its_kind_digest_and_agents() {
        let pkg = inspect_zip(&expert_zip()).unwrap();
        assert_eq!((pkg.name.as_str(), pkg.version.as_str(), pkg.kind), ("accounting-supervisor", "1.0.0", Kind::Agent));
        assert!(pkg.files.iter().any(|f| f.path == ".codebuddy-plugin/plugin.json"));
        assert_eq!(pkg.agent_docs()["agents/accounting-supervisor.md"].name, "accounting-supervisor");
        let summary = pkg.summary();
        assert_eq!((summary["kind"].as_str(), summary["fileCount"].as_u64(), summary["hasMcp"].as_bool()), (Some("agent"), Some(4), Some(false)));
        assert_eq!(summary["agents"][0], "agents/accounting-supervisor.md");
        assert_eq!(summary["skills"][0], "skills/ledger-check");
        // 市场的清单摘要算法：排序后的 "<sha>  <path>\n"。
        let mut digest = Sha256::new();
        for f in &pkg.files {
            digest.update(format!("{}  {}\n", f.sha256, f.path).as_bytes());
        }
        assert_eq!(pkg.manifest_sha256, format!("{:x}", digest.finalize()));
        // 一层包装目录。
        let wrapped = zip(&[("pkg/.codebuddy-plugin/plugin.json", manifest().as_bytes()), ("pkg/agents/accounting-supervisor.md", AGENT.as_bytes()), ("pkg/skills/ledger-check/SKILL.md", SKILL.as_bytes())]);
        assert_eq!(inspect_zip(&wrapped).unwrap().name, "accounting-supervisor");
    }

    #[test]
    fn unsafe_or_inconsistent_packages_are_refused() {
        for bad in ["../x", ".hidden/a", "agents/.codebuddy-plugin/x", ".codebuddy-plugin/.env", "C:/x", "a\\b"] {
            let z = zip(&[(".codebuddy-plugin/plugin.json", manifest().as_bytes()), (bad, b"x")]);
            assert!(inspect_zip(&z).is_err(), "{bad}");
        }
        assert!(inspect_zip(&zip(&[("agents/a.md", b"x")])).is_err(), "没有 plugin.json");
        assert!(inspect_zip(&zip(&[(".codebuddy-plugin/plugin.json", manifest().as_bytes()), ("skills/ledger-check/SKILL.md", SKILL.as_bytes())])).is_err(), "agent 文件缺失");
        let renamed = AGENT.replace("name: accounting-supervisor", "name: someone-else");
        assert!(inspect_zip(&zip(&[(".codebuddy-plugin/plugin.json", manifest().as_bytes()), ("agents/accounting-supervisor.md", renamed.as_bytes()), ("skills/ledger-check/SKILL.md", SKILL.as_bytes())])).is_err());
        assert!(inspect_zip(b"not a zip").is_err());
    }

    #[test]
    fn hooks_and_declared_tools_are_surfaced_but_never_run() {
        let with_hooks = manifest().replace("\"expertType\"", "\"hooks\":{\"PostToolUse\":[]},\"expertType\"");
        let agent = AGENT.replace("maxTurns: 50", "tools: [Bash]");
        let z = zip(&[(".codebuddy-plugin/plugin.json", with_hooks.as_bytes()), ("agents/accounting-supervisor.md", agent.as_bytes()), ("skills/ledger-check/SKILL.md", SKILL.as_bytes())]);
        let pkg = inspect_zip(&z).unwrap();
        assert!(pkg.warnings.iter().any(|w| w.contains("hooks")) && pkg.warnings.iter().any(|w| w.contains("tools")));
    }

    #[test]
    fn install_list_read_agent_replace_and_remove() {
        let (root, backups) = (temp("root"), temp("backups"));
        let pkg = inspect_zip(&expert_zip()).unwrap();
        let receipt = install(&root, &backups, &pkg, "xiaoban-market", json!({"service": "xiaoban-market"})).unwrap();
        assert_eq!(receipt["pluginId"], "accounting-supervisor@xiaoban-market");
        assert!(root.join("accounting-supervisor/agents/accounting-supervisor.md").is_file());
        assert!(root.join("accounting-supervisor/.codebuddy-plugin/plugin.json").is_file());
        let list = installed(&root);
        assert_eq!((list.len(), list[0]["kind"].as_str()), (1, Some("agent")));
        assert_eq!(skill_roots(&root), vec![root.join("accounting-supervisor/skills")]);
        // 内核按只读根读取插件自带的技能，技能保持原名。
        let skills = skill_roots(&root).into_iter().fold(agent_kernel::skills::Skills::new(), |s, r| s.read_only_root(r));
        let names: Vec<String> = skills.list().into_iter().map(|i| i.name).collect();
        assert_eq!(names, ["ledger-check"]);
        assert!(skills.load("ledger-check").is_ok());
        assert_eq!(read_agent(&root, "accounting-supervisor", "accounting-supervisor").unwrap().max_turns, Some(50));
        assert!(read_agent(&root, "accounting-supervisor", "../x").is_err());
        // 升级：旧目录进备份。
        let newer = manifest().replace("1.0.0", "1.1.0");
        let z = zip(&[(".codebuddy-plugin/plugin.json", newer.as_bytes()), ("agents/accounting-supervisor.md", AGENT.as_bytes()), ("skills/ledger-check/SKILL.md", SKILL.as_bytes())]);
        install(&root, &backups, &inspect_zip(&z).unwrap(), "xiaoban-market", json!({})).unwrap();
        assert_eq!(installed(&root)[0]["version"], "1.1.0");
        assert_eq!(fs::read_dir(&backups).unwrap().count(), 1);
        // 不是小伴装的同名目录不会被覆盖，也不会被卸载。
        fs::create_dir_all(root.join("foreign-tool")).unwrap();
        let foreign = manifest_named("foreign-tool");
        let z = zip(&[(".codebuddy-plugin/plugin.json", foreign.as_bytes()), ("agents/accounting-supervisor.md", AGENT.as_bytes()), ("skills/ledger-check/SKILL.md", SKILL.as_bytes())]);
        assert!(install(&root, &backups, &inspect_zip(&z).unwrap(), "xiaoban-market", json!({})).is_err());
        assert!(remove(&root, "foreign-tool").is_err() && root.join("foreign-tool").is_dir());
        remove(&root, "accounting-supervisor").unwrap();
        assert!(installed(&root).is_empty() && !root.join("accounting-supervisor").exists());
        let _ = fs::remove_dir_all(root);
        let _ = fs::remove_dir_all(backups);
    }

    /// 参照物：把本机 WorkBuddy 里的专家和连接器按原目录打成 ZIP，再走我们的校验。没装 WorkBuddy 就跳过。
    #[test]
    fn real_workbuddy_plugins_install_through_our_pipeline() {
        let Some(home) = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")) else { return };
        let plugins = Path::new(&home).join(".workbuddy/plugins");
        fn collect(base: &Path, dir: &Path, out: &mut Vec<(String, Vec<u8>)>) {
            let Ok(entries) = fs::read_dir(dir) else { return };
            for e in entries.flatten() {
                let p = e.path();
                let name = p.strip_prefix(base).unwrap().to_string_lossy().replace('\\', "/");
                if name.starts_with(".in_use") || name.starts_with(".downloaded") || name.starts_with(".orphaned") {
                    continue;
                }
                if p.is_dir() {
                    collect(base, &p, out);
                } else if let Ok(b) = fs::read(&p) {
                    out.push((name, b));
                }
            }
        }
        let mut dirs = Vec::new();
        if let Ok(names) = fs::read_dir(plugins.join("marketplaces/experts/plugins")) {
            dirs.extend(names.flatten().map(|n| n.path()));
        }
        if let Ok(names) = fs::read_dir(plugins.join("marketplaces/workbuddy-connector-plugins-official/connectors")) {
            dirs.extend(names.flatten().map(|n| n.path()).take(400));
        }
        let (root, backups) = (temp("real-root"), temp("real-backups"));
        let (mut ok, mut skipped, mut kinds) = (0, Vec::new(), BTreeMap::<&str, usize>::new());
        for dir in dirs {
            let mut entries = Vec::new();
            collect(&dir, &dir, &mut entries);
            let refs: Vec<(&str, &[u8])> = entries.iter().map(|(n, b)| (n.as_str(), b.as_slice())).collect();
            match inspect_zip(&zip(&refs)) {
                Ok(pkg) => {
                    install(&root, &backups, &pkg, "workbuddy-oracle", json!({"test": true})).unwrap_or_else(|e| panic!("{}: {e}", pkg.name));
                    *kinds.entry(pkg.kind.as_str()).or_default() += 1;
                    ok += 1;
                }
                Err(e) => skipped.push(format!("{}: {e}", dir.file_name().unwrap().to_string_lossy())),
            }
        }
        eprintln!("installed {ok} real WorkBuddy plugins {kinds:?}; refused {}: {:?}", skipped.len(), &skipped[..skipped.len().min(8)]);
        assert!(skipped.iter().all(|s| !s.contains("缺少 .codebuddy-plugin")), "每个插件都应有 plugin.json：{skipped:?}");
        let _ = fs::remove_dir_all(root);
        let _ = fs::remove_dir_all(backups);
    }
}
