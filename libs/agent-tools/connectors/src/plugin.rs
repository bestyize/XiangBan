//! WorkBuddy connector directories. Loading never starts a process or contacts a service.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Display {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub name_en: String,
    #[serde(default)]
    pub name_zh: String,
    #[serde(default)]
    pub description_en: String,
    #[serde(default)]
    pub description_zh: String,
    #[serde(default)]
    pub examples_en: Vec<String>,
    #[serde(default)]
    pub examples_zh: Vec<String>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TokenSchema {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub title_en: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub description_en: String,
    #[serde(default)]
    pub fields: Vec<TokenField>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenField {
    pub key: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub label_en: String,
    #[serde(rename = "type", default)]
    pub field_type: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub placeholder: String,
    #[serde(default)]
    pub placeholder_en: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub description_en: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AuthSpec {
    None,
    Token {
        schema: TokenSchema,
        headers: Vec<(String, String, String)>,
    },
    OAuth,
    Cli,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Transport {
    #[serde(rename = "streamable-http")]
    StreamableHttp,
    #[serde(rename = "stdio")]
    Stdio,
    #[serde(rename = "sse")]
    Sse,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerSpec {
    pub name: String,
    pub transport: Transport,
    pub url: Option<String>,
    pub command: Option<String>,
    pub args: Vec<String>,
    pub env: BTreeMap<String, String>,
    pub headers: BTreeMap<String, String>,
    pub auth: AuthSpec,
    pub disabled_tools: Vec<String>,
    pub timeout_ms: Option<u64>,
}
impl ServerSpec {
    /// Old HTTP+SSE must never silently fall back to Streamable HTTP POST.
    pub fn unsupported(&self) -> Option<&'static str> {
        (self.transport == Transport::Sse).then_some("旧式 SSE 暂不支持")
    }
    pub fn schema(&self) -> Option<&TokenSchema> {
        match &self.auth {
            AuthSpec::Token { schema, .. } => Some(schema),
            _ => None,
        }
    }
    pub(crate) fn validate_values(
        &self,
        values: &BTreeMap<String, String>,
        required: bool,
    ) -> Result<(), String> {
        for (key, value) in values {
            if !self
                .schema()
                .is_some_and(|s| s.fields.iter().any(|f| f.key == *key))
            {
                return Err("令牌字段不在 schema 中".into());
            }
            if value.chars().any(char::is_control) {
                return Err("令牌值不可含控制字符".into());
            }
        }
        if required
            && self.schema().is_some_and(|s| {
                s.fields
                    .iter()
                    .any(|f| f.required && values.get(&f.key).is_none_or(|v| v.trim().is_empty()))
            })
        {
            return Err("缺少必填令牌字段".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConnectorPlugin {
    pub name: String,
    pub version: String,
    pub display: Display,
    pub servers: Vec<ServerSpec>,
    pub skills_dir: Option<PathBuf>,
    pub icon: Option<PathBuf>,
}
fn read_json(path: &Path) -> Result<Value, String> {
    let bytes = fs::read(path).map_err(|_| "无法读取连接器插件文件")?;
    if bytes.len() > 1024 * 1024 {
        return Err("连接器插件文件过大".into());
    }
    serde_json::from_slice(bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(&bytes))
        .map_err(|_| "连接器插件 JSON 无效".into())
}
fn local_path(root: &Path, relative: &str) -> Result<PathBuf, String> {
    let path = root
        .join(relative)
        .canonicalize()
        .map_err(|_| "连接器资源文件不存在")?;
    if !path.starts_with(root) {
        return Err("连接器资源不可越出插件目录".into());
    }
    Ok(path)
}
fn text(value: &Value, key: &str) -> Option<String> {
    value[key].as_str().map(str::to_owned)
}
fn strings(value: Option<&Value>) -> Result<Vec<String>, String> {
    match value {
        None => Ok(vec![]),
        Some(v) => {
            serde_json::from_value(v.clone()).map_err(|_| "连接器字段必须为字符串数组".into())
        }
    }
}
impl ConnectorPlugin {
    pub fn load(dir: &Path) -> Result<Self, String> {
        let root = dir.canonicalize().map_err(|_| "连接器目录不存在")?;
        let manifest = read_json(&local_path(&root, ".codebuddy-plugin/plugin.json")?)?;
        let name = text(&manifest, "name")
            .filter(|s| !s.trim().is_empty())
            .ok_or("连接器缺少名称")?;
        let version = text(&manifest, "version")
            .filter(|s| !s.trim().is_empty())
            .ok_or("连接器缺少版本")?;
        let wb = &manifest["extensions"]["ai.workbuddy"];
        let mut display: Display = match wb.get("display") {
            Some(v) => serde_json::from_value(v.clone()).map_err(|_| "连接器 display 无效")?,
            None => Display::default(),
        };
        if display.name.is_empty() {
            display.name = name.clone();
        }
        if display.description_en.is_empty() {
            display.description_en = text(&manifest, "description").unwrap_or_default();
        }
        let mcp_path = ["mcp.json", ".mcp.json"]
            .into_iter()
            .find(|p| root.join(p).is_file());
        let mcp = mcp_path
            .map(|p| local_path(&root, p).and_then(|p| read_json(&p)))
            .transpose()?;
        let mut servers = vec![];
        if let Some(mcp) = mcp {
            let map = mcp["mcpServers"]
                .as_object()
                .ok_or("连接器缺少 mcpServers")?;
            for (server, config) in map {
                let transport =
                    match config["type"]
                        .as_str()
                        .unwrap_or(if config.get("command").is_some() {
                            "stdio"
                        } else {
                            "streamable-http"
                        }) {
                        "streamable-http" | "streamableHttp" => Transport::StreamableHttp,
                        "stdio" => Transport::Stdio,
                        "sse" => Transport::Sse,
                        _ => return Err("连接器传输类型无效".into()),
                    };
                let setup = &wb["setup"]["mcpServers"][server];
                let policy = &wb["policy"]["mcpServers"][server];
                let auth = match setup["auth"]["type"].as_str() {
                    Some("token") => {
                        let path = setup["auth"]["schema"]
                            .as_str()
                            .ok_or("令牌缺少 schema 路径")?;
                        let schema: TokenSchema =
                            serde_json::from_value(read_json(&local_path(&root, path)?)?)
                                .map_err(|_| "令牌 schema 无效")?;
                        let mut keys = std::collections::BTreeSet::new();
                        for field in &schema.fields {
                            if field.key.is_empty()
                                || field.key.chars().any(char::is_control)
                                || !keys.insert(&field.key)
                            {
                                return Err("令牌 schema key 无效或重复".into());
                            }
                        }
                        let mut headers = vec![];
                        if let Some(map) = setup["auth"].get("headers") {
                            for (header, spec) in map.as_object().ok_or("认证 headers 无效")? {
                                let field =
                                    spec["field"].as_str().ok_or("认证 header 缺少 field")?;
                                if !keys.contains(&field.to_owned()) {
                                    return Err("认证 header 引用了未知字段".into());
                                }
                                headers.push((
                                    header.clone(),
                                    field.into(),
                                    spec["prefix"].as_str().unwrap_or("").into(),
                                ));
                            }
                        }
                        AuthSpec::Token { schema, headers }
                    }
                    Some("oauth") => AuthSpec::OAuth,
                    Some("none") => AuthSpec::None,
                    Some(_) => return Err("连接器认证类型暂不支持".into()),
                    None if setup["preAuth"] == "cli" => AuthSpec::Cli,
                    None => AuthSpec::None, // Remote None uses authMode:auto to discover OAuth on 401.
                };
                let spec = ServerSpec {
                    name: server.clone(),
                    transport,
                    url: text(config, "url"),
                    command: text(config, "command"),
                    args: strings(config.get("args"))?,
                    env: super::string_map(config.get("env"))?,
                    headers: super::string_map(config.get("headers"))?,
                    auth,
                    disabled_tools: strings(policy.get("disabledTools"))?,
                    timeout_ms: policy
                        .get("timeout")
                        .or(config.get("timeout"))
                        .map(|v| v.as_u64().filter(|v| *v > 0).ok_or("连接器 timeout 无效"))
                        .transpose()?,
                };
                if (spec.transport == Transport::Stdio
                    && (spec.command.as_deref().is_none_or(str::is_empty) || spec.url.is_some()))
                    || (spec.transport != Transport::Stdio
                        && (spec.url.as_deref().is_none_or(str::is_empty)
                            || spec.command.is_some()))
                {
                    return Err("连接器地址或命令无效".into());
                }
                servers.push(spec);
            }
        }
        // Official marketplace includes 58 skill-only directories; those legitimately have no servers.
        let skills_dir = root
            .join("skills")
            .is_dir()
            .then(|| local_path(&root, "skills"))
            .transpose()?;
        let icon = [
            "ai.workbuddy/assets/icon.png",
            "ai.workbuddy/assets/icon.svg",
        ]
        .into_iter()
        .find(|p| root.join(p).is_file())
        .map(|p| local_path(&root, p))
        .transpose()?;
        Ok(Self {
            name,
            version,
            display,
            servers,
            skills_dir,
            icon,
        })
    }
    pub fn server(&self, name: &str) -> Result<&ServerSpec, String> {
        self.servers
            .iter()
            .find(|s| s.name == name)
            .ok_or("连接器没有该 MCP 服务".into())
    }
    pub fn build_config(
        &self,
        server: &str,
        values: &BTreeMap<String, String>,
    ) -> Result<Value, String> {
        let spec = self.server(server)?;
        if let Some(reason) = spec.unsupported() {
            return Err(reason.into());
        }
        spec.validate_values(values, true)?;
        let mut config = if spec.transport == Transport::Stdio {
            json!({"command":spec.command,"args":spec.args,"env":spec.env})
        } else {
            json!({"url":spec.url,"headers":spec.headers,"authMode":match spec.auth { AuthSpec::Token {..} => "token", AuthSpec::OAuth => "oauth", _ => "auto" }})
        };
        substitute(&mut config, spec, values)?;
        if let AuthSpec::Token { headers, .. } = &spec.auth {
            for (header, field, prefix) in headers {
                if let Some(value) = values.get(field).filter(|s| !s.is_empty()) {
                    // Auth mapping takes precedence even if a template uses different header casing.
                    let map = config["headers"]
                        .as_object_mut()
                        .ok_or("stdio 不支持认证 headers")?;
                    map.retain(|k, _| !k.eq_ignore_ascii_case(header));
                    map.insert(header.clone(), json!(format!("{prefix}{value}")));
                }
            }
        }
        let config = super::normalized_config(&config)?;
        for (header, value) in &super::string_map(config.get("headers"))? {
            reqwest::header::HeaderName::from_bytes(header.as_bytes())
                .map_err(|_| "连接器请求头名称无效")?;
            reqwest::header::HeaderValue::from_str(value).map_err(|_| "连接器请求头值无效")?;
        }
        Ok(config)
    }
    /// Stable per-plugin/server ID; no collision between plugins with the same MCP server name.
    pub fn connection_id(&self, server: &str) -> String {
        use sha2::{Digest, Sha256};
        format!(
            "plugin-{:x}",
            Sha256::digest(format!("{}\0{server}", self.name))
        )[..55]
            .to_string()
    }
}
fn substitute(
    value: &mut Value,
    spec: &ServerSpec,
    values: &BTreeMap<String, String>,
) -> Result<(), String> {
    match value {
        Value::String(s) => {
            let mut output = String::new();
            let mut rest = s.as_str();
            while let Some(start) = rest.find("${") {
                output.push_str(&rest[..start]);
                let tail = &rest[start + 2..];
                let end = tail.find('}').ok_or("令牌占位符未闭合")?;
                let key = &tail[..end];
                if !spec
                    .schema()
                    .is_some_and(|s| s.fields.iter().any(|f| f.key == key))
                {
                    return Err("占位符引用了未知令牌字段".into());
                }
                output.push_str(values.get(key).map(String::as_str).unwrap_or(""));
                rest = &tail[end + 1..];
            }
            output.push_str(rest);
            if output.chars().any(char::is_control) {
                return Err("连接器配置不可含控制字符".into());
            }
            *s = output;
        }
        Value::Array(a) => {
            for v in a {
                substitute(v, spec, values)?;
            }
        }
        Value::Object(o) => {
            for v in o.values_mut() {
                substitute(v, spec, values)?;
            }
        }
        _ => (),
    }
    Ok(())
}
