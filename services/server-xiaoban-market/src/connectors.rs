//! 连接器插件静态校验和公开展示摘要，不连接 MCP 或返回凭据值。
use crate::plugin::{self, Issue, Manifest, McpServer};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

type Files = BTreeMap<String, Vec<u8>>;

pub fn servers(files: &Files) -> Result<BTreeMap<String, McpServer>, String> {
    let bytes = files.get("mcp.json").or_else(|| files.get(".mcp.json")).ok_or("缺少 mcp.json / .mcp.json")?;
    let servers = plugin::parse_mcp(bytes)?;
    if servers.is_empty() { return Err("至少需要一个 MCP 服务".into()); }
    Ok(servers)
}

pub fn kind(m: &Manifest, files: &Files) -> Result<&'static str, String> {
    if !m.expert_type.is_empty() && files.keys().any(|p| p.starts_with("agents/")) { return Ok("expert"); }
    if files.contains_key("mcp.json") || files.contains_key(".mcp.json") {
        servers(files)?;
        return Ok("connector");
    }
    if files.keys().any(|p| p.starts_with("skills/")) {
        return Err("暂不支持只有 skills/ 的技能插件包".into());
    }
    Err("无法判断插件类型：需要专家 agents/ + expertType 或 MCP 服务".into())
}

fn placeholders(value: &str) -> Result<Vec<String>, String> {
    let mut keys = Vec::new();
    let mut rest = value;
    while let Some((_, tail)) = rest.split_once("${") {
        let (key, tail) = tail.split_once('}').ok_or("未闭合的 ${KEY} 占位符")?;
        if key.is_empty() { return Err("占位符 key 不能为空".into()); }
        keys.push(key.into());
        rest = tail;
    }
    Ok(keys)
}

fn credential_key(key: &str) -> bool {
    let k = key.to_ascii_lowercase();
    ["token", "secret", "password", "authorization", "api_key", "apikey", "api-key", "cookie", "credential"].iter().any(|v| k.contains(v))
}

pub fn validate(m: &Manifest, files: &Files) -> Vec<Issue> {
    let mut out = Vec::new();
    let mut issue = |error, message: String| out.push(Issue { error, message });
    if m.name.trim().is_empty() { issue(true, "缺少 name".into()); }
    if m.version.trim().is_empty() { issue(true, "缺少 version".into()); }
    if m.hooks.as_ref().is_some_and(|h| !h.is_null()) { issue(false, "声明了 hooks：小伴不会执行插件钩子".into()); }
    let servers = match servers(files) { Ok(v) => v, Err(e) => { issue(true, e); return out; } };
    let setup = &m.extensions["ai.workbuddy"]["setup"]["mcpServers"];
    for (name, s) in &servers {
        if name.trim().is_empty() { issue(true, "MCP 服务名不能为空".into()); }
        if !matches!(s.transport.as_str(), "streamable-http" | "streamableHttp" | "sse" | "stdio") {
            issue(true, format!("服务 {name} 的 type 不支持"));
        }
        if (s.transport == "stdio" && s.command.trim().is_empty()) || (s.transport != "stdio" && s.url.trim().is_empty()) {
            issue(true, format!("服务 {name} 缺少 command / url"));
        }
        let auth = &setup[name]["auth"];
        let mut referenced = BTreeSet::new();
        for (key, value) in s.headers.iter().chain(s.env.iter()) {
            match placeholders(value) {
                Ok(keys) => { referenced.extend(keys); }
                Err(e) => issue(true, format!("服务 {name}：{e}")),
            }
            if credential_key(key) && !value.trim().is_empty() && !value.contains("${") {
                // 警告里只包含字段名，不记录值。
                issue(false, format!("服务 {name} 的 {key} 疑似包含真实密钥，请改为占位符"));
            }
        }
        // URL / args 中也可能使用占位符。
        for value in std::iter::once(&s.url).chain(s.args.iter()) {
            match placeholders(value) { Ok(keys) => referenced.extend(keys), Err(e) => issue(true, format!("服务 {name}：{e}")) }
        }
        if auth["type"] == "token" {
            let path = auth["schema"].as_str().unwrap_or("").trim_start_matches("./");
            let schema = match files.get(path).ok_or_else(|| format!("服务 {name} 找不到 token schema"))
                .and_then(|b| plugin::parse_token_schema(b)) {
                Ok(v) => v, Err(e) => { issue(true, e); continue; }
            };
            let mut keys = BTreeSet::new();
            for f in &schema.fields {
                if f.key.trim().is_empty() || !keys.insert(f.key.clone()) { issue(true, format!("服务 {name} 的 schema 字段 key 为空或重复")); }
            }
            if keys.is_empty() { issue(true, format!("服务 {name} 的 token schema 没有字段")); }
            if let Some(headers) = auth["headers"].as_object() {
                for mapping in headers.values() {
                    match mapping["field"].as_str().filter(|s| !s.is_empty()) {
                        Some(key) => { referenced.insert(key.into()); }
                        None => issue(true, format!("服务 {name} 的 headers 映射缺少 field")),
                    }
                }
            }
            for key in referenced.difference(&keys) { issue(true, format!("服务 {name} 引用了 schema 中不存在的字段 {key}")); }
            for key in keys.difference(&referenced) { issue(true, format!("服务 {name} 的 schema 字段 {key} 没有对应 headers / 占位符")); }
        }
    }
    if let Some(declarations) = setup.as_object() {
        for (name, spec) in declarations {
            if spec["auth"]["type"] == "token" && !servers.contains_key(name) { issue(true, format!("token setup 引用了不存在的服务 {name}")); }
        }
    }
    out
}

pub fn presentation(m: &Manifest, files: &Files) -> Value {
    let ext = &m.extensions["ai.workbuddy"];
    let d = &ext["display"];
    let string = |key: &str, fallback: &str| d[key].as_str().filter(|s| !s.trim().is_empty()).unwrap_or(fallback).to_string();
    let name_en = string("name_en", &m.name);
    let name = string("name", &name_en);
    let name_zh = string("name_zh", &name);
    let examples = |key: &str| d[key].as_array().map(|v| v.iter().filter_map(Value::as_str).collect::<Vec<_>>()).unwrap_or_default();
    let mut auth_types = BTreeSet::new();
    let mut transports = BTreeSet::new();
    let mut fields = Vec::new();
    let mut disabled = 0;
    for (name, s) in servers(files).unwrap_or_default() {
        transports.insert(if s.transport == "streamableHttp" { "streamable-http".into() } else { s.transport.clone() });
        let auth = &ext["setup"]["mcpServers"][&name]["auth"];
        let auth_type = if auth["type"] == "token" {
            let path = auth["schema"].as_str().unwrap_or("").trim_start_matches("./");
            if let Some(schema) = files.get(path).and_then(|b| plugin::parse_token_schema(b).ok()) {
                fields.extend(schema.fields.iter().map(|f| json!({"server": name, "key": f.key, "label": f.label, "type": f.kind, "required": f.required})));
            }
            "token"
        } else if auth["type"] == "oauth" || (!s.url.is_empty() && s.headers.is_empty()) {
            // 静态包无法确认远端是否要求授权，保留自动发现语义。
            "oauth"
        } else if ext["setup"]["mcpServers"][&name]["preAuth"] == "cli" || !s.env.is_empty() { "env" } else { "none" };
        auth_types.insert(auth_type);
        disabled += ext["policy"]["mcpServers"][&name]["disabledTools"].as_array().map_or(0, Vec::len);
    }
    let skills: Vec<_> = files.keys().filter_map(|p| p.strip_prefix("skills/")?.strip_suffix("/SKILL.md")).collect();
    json!({
        "displayName": {"zh": name_zh, "en": name_en},
        "displayDescription": {"zh": string("description_zh", &m.description), "en": string("description_en", &m.description)},
        "examples": {"zh": examples("examples_zh"), "en": examples("examples_en")},
        "authTypes": auth_types, "tokenFields": fields, "transports": transports,
        "skills": skills, "skillCount": skills.len(), "disabledToolCount": disabled,
    })
}

pub fn icon<'a>(files: &'a Files) -> Option<(&'a str, &'a [u8])> {
    ["png", "svg", "jpg", "jpeg", "webp"].iter().find_map(|ext| {
        let path = format!("ai.workbuddy/assets/icon.{ext}");
        files.get_key_value(&path).map(|(p, b)| (p.as_str(), b.as_slice()))
    })
}

/// 只查询固定分类；未知内容归入“其他”。
pub fn category(m: &Manifest) -> &'static str {
    let text = format!("{} {} {} {}", m.name, m.description, m.keywords.join(" "), m.extensions["ai.workbuddy"]["display"]).to_lowercase();
    for (slug, terms) in [
        ("database", &["database", "postgres", "mysql", "数据库"][..]),
        ("dev", &["github", "gitlab", "repository", "开发", "代码", "jira"]),
        ("docs", &["notion", "document", "office", "文档", "办公"]),
        ("web", &["search", "browser", "crawl", "搜索", "网页"]),
        ("chat", &["slack", "discord", "chat", "通信", "消息"]),
        ("design", &["figma", "design", "设计"]),
        ("files", &["filesystem", "file system", "本地文件"]),
    ] { if terms.iter().any(|t| text.contains(t)) { return slug; } }
    "other"
}

/// 拒绝活动 SVG 内容与外部资源；验证通过的文件保持原始字节。
pub fn safe_svg(bytes: &[u8]) -> bool {
    use quick_xml::{events::Event, Reader};
    let Ok(text) = std::str::from_utf8(bytes) else { return false };
    if bytes.len() > plugin::AVATAR_MAX_BYTES || text.contains('\\') { return false; }
    let mut reader = Reader::from_str(text);
    let mut root = false;
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                let n = e.local_name();
                let Ok(n) = std::str::from_utf8(n.as_ref()) else { return false };
                if !matches!(n, "svg" | "g" | "path" | "rect" | "circle" | "ellipse" | "line" | "polyline" | "polygon" | "defs" | "linearGradient" | "radialGradient" | "stop" | "clipPath" | "mask" | "use" | "symbol" | "title" | "desc" | "text" | "tspan") { return false; }
                if !root { if n != "svg" { return false; } root = true; }
                for a in e.attributes() {
                    let Ok(a) = a else { return false };
                    let Ok(k) = std::str::from_utf8(a.key.as_ref()) else { return false };
                    let k = k.to_ascii_lowercase();
                    let Ok(v) = a.decode_and_unescape_value(reader.decoder()) else { return false };
                    let v = v.trim().to_ascii_lowercase();
                    if k.starts_with("on") || k == "xml:base" || (k.ends_with("href") && !v.starts_with('#')) || v.contains('\\') || v.contains('@') { return false; }
                    let mut rest = v.as_str();
                    while let Some((_, tail)) = rest.split_once("url(") {
                        let Some((target, tail)) = tail.split_once(')') else { return false };
                        if !target.trim().trim_matches(['\'', '"']).starts_with('#') { return false; }
                        rest = tail;
                    }
                }
            }
            Ok(Event::DocType(_)) | Ok(Event::PI(_)) => return false,
            Ok(Event::Eof) => return root,
            Err(_) => return false,
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (Manifest, Files) {
        let m = plugin::parse_manifest(br#"{"name":"example","version":"1.0.0","extensions":{"ai.workbuddy":{"setup":{"mcpServers":{"a":{"auth":{"type":"token","schema":"./ai.workbuddy/token-schema.json","headers":{"Authorization":{"field":"KEY","prefix":"Bearer "}}}}}},"policy":{"mcpServers":{"a":{"disabledTools":["delete"]}}}}}}"#).unwrap();
        let f = BTreeMap::from([
            ("mcp.json".into(), br#"{"mcpServers":{"a":{"type":"streamableHttp","url":"https://example.com/mcp"}}}"#.to_vec()),
            ("ai.workbuddy/token-schema.json".into(), br#"{"fields":[{"key":"KEY","label":"Key","type":"password","required":true,"default":"NEVER_PUBLIC"}]}"#.to_vec()),
            ("skills/helper/SKILL.md".into(), b"# helper".to_vec()),
        ]);
        (m, f)
    }
    #[test]
    fn validates_token_mapping_and_presents_only_field_summaries() {
        let (m, mut f) = fixture();
        assert!(validate(&m, &f).is_empty());
        let p = presentation(&m, &f);
        assert_eq!(p["authTypes"], json!(["token"]));
        assert_eq!(p["transports"], json!(["streamable-http"]));
        assert_eq!(p["skillCount"], 1);
        assert_eq!(p["disabledToolCount"], 1);
        assert!(!p.to_string().contains("NEVER_PUBLIC"));
        f.remove("ai.workbuddy/token-schema.json");
        assert!(validate(&m, &f).iter().any(|i| i.error));
        let (_, mut f) = fixture();
        f.insert("ai.workbuddy/token-schema.json".into(), br#"{"fields":[{"key":"OTHER"}]}"#.to_vec());
        assert!(validate(&m, &f).iter().any(|i| i.error && i.message.contains("KEY")));
    }
    #[test]
    fn transport_placeholders_and_secret_warnings() {
        let (mut m, mut f) = fixture();
        m.extensions = json!({});
        for transport in ["streamable-http", "streamableHttp", "stdio", "sse"] {
            f.insert("mcp.json".into(), json!({"mcpServers":{"a":{"type":transport,"url":"https://example.com","command":"npx","headers":{"Authorization":"REAL_SECRET"}}}}).to_string().into_bytes());
            let issues = validate(&m, &f);
            assert!(issues.iter().all(|i| !i.error), "{issues:?}");
            assert!(issues.iter().any(|i| i.message.contains("Authorization")));
            assert!(!format!("{issues:?}").contains("REAL_SECRET"));
        }
        f.insert("mcp.json".into(), br#"{"mcpServers":{"a":{"type":"bogus","url":"${BROKEN"}}}"#.to_vec());
        assert!(validate(&m, &f).iter().any(|i| i.error));
        f.insert("mcp.json".into(), br#"{"mcpServers":{}}"#.to_vec());
        assert!(kind(&m, &f).is_err());
        f.remove("mcp.json");
        assert!(kind(&m, &f).unwrap_err().contains("暂不支持"));
    }
    #[test]
    fn svg_rejects_active_content_and_external_references() {
        assert!(safe_svg(br##"<svg xmlns="http://www.w3.org/2000/svg"><defs><path id="x" d="M0 0"/></defs><use href="#x"/><path fill="url(#x)"/></svg>"##));
        for bad in ["<svg><script/></svg>", "<svg onload = 'alert(1)'/>", "<svg><foreignObject/></svg>", "<svg><use href='https://evil/x'/></svg>", "<svg><use xlink:href='data:text/html,x'/></svg>", "<svg><path fill='url(https://evil)'/></svg>", "<!DOCTYPE svg [<!ENTITY x 'xx'>]><svg/>", "<svg><style>@import 'https://evil';</style></svg>", "<svg><animate attributeName='href'/></svg>", "<svg><use href='&#104;ttps://evil'/></svg>"] {
            assert!(!safe_svg(bad.as_bytes()), "{bad}");
        }
    }
    #[test]
    fn workbuddy_connector_oracle() {
        let Some(home) = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")) else { return };
        let root = std::path::Path::new(&home).join(".workbuddy/plugins/marketplaces/workbuddy-connector-plugins-official/connectors");
        let Ok(dirs) = std::fs::read_dir(root) else { return };
        fn read(base: &std::path::Path, dir: &std::path::Path, files: &mut Files) {
            for e in std::fs::read_dir(dir).unwrap() {
                let e = e.unwrap();
                if e.file_type().unwrap().is_dir() { read(base, &e.path(), files); }
                else { files.insert(e.path().strip_prefix(base).unwrap().to_string_lossy().replace('\\', "/"), std::fs::read(e.path()).unwrap()); }
            }
        }
        let (mut connectors, mut skills) = (0, 0);
        for d in dirs {
            let dir = d.unwrap().path();
            if !dir.is_dir() { continue; }
            let mut f = Files::new();
            read(&dir, &dir, &mut f);
            let m = plugin::parse_manifest(&f[".codebuddy-plugin/plugin.json"]).unwrap();
            if !f.contains_key("mcp.json") && !f.contains_key(".mcp.json") { skills += 1; continue; }
            let errors: Vec<_> = validate(&m, &f).into_iter().filter(|i| i.error).collect();
            assert!(errors.is_empty(), "{}: {errors:?}", m.name);
            assert_eq!(kind(&m, &f).unwrap(), "connector");
            connectors += 1;
        }
        eprintln!("WorkBuddy oracle: {connectors} connectors, {skills} skill-only plugins");
        assert!(connectors > 0);
    }
}
