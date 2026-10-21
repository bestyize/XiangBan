//! Connection lifecycle and consent belong to the host; tools share kernel permissions.
use agent_kernel::mcp::{HttpMcpClient, HttpMcpConfig, McpClient, McpConfig};
use agent_kernel::{Risk, Tool, ToolCtx, ToolResult, ToolSpec};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Write,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

#[derive(Clone, Serialize, Deserialize)]
struct Record {
    id: String,
    name: String,
    config: Value,
    enabled: bool,
}
enum Client {
    Stdio(Arc<McpClient>),
    Http(Arc<HttpMcpClient>),
}
impl Client {
    fn close(&self) {
        match self {
            Self::Stdio(c) => c.close(),
            Self::Http(c) => c.close(),
        }
    }
    fn is_connected(&self) -> bool {
        match self {
            Self::Stdio(c) => c.is_connected(),
            Self::Http(c) => c.is_connected(),
        }
    }
    fn tools(&self) -> Vec<Arc<dyn Tool>> {
        match self {
            Self::Stdio(c) => c.tools(),
            Self::Http(c) => c.tools(),
        }
    }
}
struct Active {
    client: Client,
    valid: Arc<AtomicBool>,
}
impl Active {
    fn close(&self) {
        self.valid.store(false, Ordering::SeqCst);
        self.client.close();
    }
}
impl Drop for Active {
    fn drop(&mut self) {
        self.close();
    }
}
#[derive(Default)]
struct State {
    records: Vec<Record>,
    active: std::collections::HashMap<String, Active>,
    errors: std::collections::HashMap<String, String>,
}
pub struct ConnectorManager {
    root: PathBuf,
    workspace: PathBuf,
    state: Mutex<State>,
}
fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
fn normalized_config(value: &Value) -> Result<Value, String> {
    let config = if let Some(map) = value["mcpServers"].as_object() {
        if map.len() != 1 {
            return Err("每次请选择或导入一个 MCP 服务".into());
        }
        map.values().next().unwrap().clone()
    } else {
        value.clone()
    };
    let obj = config.as_object().ok_or("MCP 配置必须是 JSON 对象")?;
    if serde_json::to_vec(obj).map_err(|e| e.to_string())?.len() > 32 * 1024 {
        return Err("MCP 配置过大".into());
    }
    let serialized = serde_json::to_string(obj).map_err(|e| e.to_string())?;
    if serialized.contains("YOUR_")
        || serialized.contains("<从")
        || serialized.contains("<your")
        || serialized.contains("\u{24}{")
    {
        return Err("请先填写配置中的密钥、地址或环境变量占位值".into());
    }
    let transport = obj
        .get("type")
        .or(obj.get("transport"))
        .and_then(Value::as_str)
        .unwrap_or("");
    if transport == "sse" {
        return Err("旧式 SSE 暂不支持，请选择 stdio 或 Streamable HTTP 配置".into());
    }
    let url = obj
        .get("url")
        .or(obj.get("baseUrl"))
        .and_then(Value::as_str);
    if let Some(url) = url {
        if obj.get("command").is_some() {
            return Err("不能同时配置远程地址和本地命令".into());
        }
        return Ok(json!({"url":url,"headers":string_map(obj.get("headers"))?}));
    }
    let command = obj
        .get("command")
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
        .ok_or("请提供 MCP 地址或启动命令")?;
    if command.chars().any(char::is_control) {
        return Err("MCP 启动命令无效".into());
    }
    let args = match obj.get("args") {
        None => Vec::new(),
        Some(Value::Array(a)) if a.len() <= 100 => a
            .iter()
            .map(|v| {
                v.as_str()
                    .map(str::to_string)
                    .ok_or("MCP 参数必须是字符串".to_string())
            })
            .collect::<Result<Vec<_>, _>>()?,
        _ => return Err("MCP args 必须是字符串数组".into()),
    };
    Ok(json!({"command":command,"args":args,"env":string_map(obj.get("env"))?}))
}
fn string_map(value: Option<&Value>) -> Result<std::collections::BTreeMap<String, String>, String> {
    match value {
        None => Ok(Default::default()),
        Some(Value::Object(map)) if map.len() <= 64 => map
            .iter()
            .map(|(k, v)| {
                if k.is_empty() || k.contains('=') || k.chars().any(char::is_control) {
                    return Err("环境变量或请求头名称无效".into());
                }
                Ok((
                    k.clone(),
                    v.as_str()
                        .ok_or("环境变量或请求头值必须是字符串")?
                        .to_string(),
                ))
            })
            .collect(),
        _ => Err("env / headers 必须是字符串映射".into()),
    }
}
impl ConnectorManager {
    pub fn new(root: PathBuf, workspace: PathBuf) -> Result<Self, String> {
        fs::create_dir_all(&root).map_err(|e| e.to_string())?;
        if fs::symlink_metadata(&root)
            .map_err(|e| e.to_string())?
            .file_type()
            .is_symlink()
        {
            return Err("连接器配置目录不可为符号链接".into());
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&root, fs::Permissions::from_mode(0o700))
                .map_err(|e| e.to_string())?;
        }
        let path = root.join("connections.json");
        let records = if path.exists() {
            if fs::symlink_metadata(&path)
                .map_err(|e| e.to_string())?
                .file_type()
                .is_symlink()
            {
                return Err("连接器配置不可为符号链接".into());
            }
            let bytes = fs::read(&path).map_err(|e| e.to_string())?;
            if bytes.len() > 512 * 1024 {
                return Err("连接器配置文件过大".into());
            }
            let records: Vec<Record> =
                serde_json::from_slice(&bytes).map_err(|_| "连接器配置文件无效")?;
            if records.len() > 16 || records.iter().any(|r| !valid_id(&r.id)) {
                return Err("连接器配置无效或超过 16 个".into());
            }
            records
        } else {
            Vec::new()
        };
        let this = Self {
            root,
            workspace,
            state: Mutex::new(State {
                records,
                ..State::default()
            }),
        };
        {
            let mut state = this.state.lock().unwrap_or_else(|e| e.into_inner());
            for record in state.records.clone().into_iter().filter(|r| r.enabled) {
                match this.start(&record.id, &record.config) {
                    Ok(active) => {
                        state.active.insert(record.id, active);
                    }
                    Err(error) => {
                        state.errors.insert(record.id, error);
                    }
                }
            }
        }
        Ok(this)
    }
    fn start(&self, id: &str, config: &Value) -> Result<Active, String> {
        let config = normalized_config(config)?;
        let prefix = format!("mcp_ext_{}_", id.replace('-', "_"));
        let client = if let Some(url) = config["url"].as_str() {
            Client::Http(HttpMcpClient::connect(HttpMcpConfig {
                url: url.into(),
                headers: string_map(config.get("headers"))?.into_iter().collect(),
                tool_prefix: prefix,
                timeout: Duration::from_secs(45),
            })?)
        } else {
            let mut mcp = McpConfig::new(config["command"].as_str().unwrap(), &[])
                .tool_prefix(prefix)
                .default_risk(Risk::Exec)
                .timeout(Duration::from_secs(45))
                .cwd(&self.workspace);
            mcp.args = config["args"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap().to_string())
                .collect();
            mcp.env = string_map(config.get("env"))?.into_iter().collect();
            Client::Stdio(
                McpClient::spawn(mcp)
                    .map_err(|_| "MCP 启动或初始化失败，请检查本机依赖、命令及凭据")?,
            )
        };
        if client.tools().is_empty() {
            client.close();
            return Err("MCP 未返回可用工具".into());
        }
        Ok(Active {
            client,
            valid: Arc::new(AtomicBool::new(true)),
        })
    }
    fn save(&self, records: &[Record]) -> Result<(), String> {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos();
        let path = self
            .root
            .join(format!(".connections-{}-{nonce}.tmp", std::process::id()));
        let mut options = fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options
            .open(&path)
            .map_err(|_| "无法写入连接器配置（可能存在未完成的保存）")?;
        let result = (|| {
            file.write_all(&serde_json::to_vec(records).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
            file.sync_all().map_err(|e| e.to_string())?;
            fs::rename(&path, self.root.join("connections.json")).map_err(|e| e.to_string())
        })();
        if result.is_err() {
            let _ = fs::remove_file(&path);
        }
        result
    }
    pub fn connect(
        &self,
        id: &str,
        name: &str,
        config: Value,
        confirmed: bool,
    ) -> Result<Value, String> {
        if !confirmed {
            return Err("连接 MCP 前需要用户明确确认配置".into());
        }
        if !valid_id(id) || name.chars().count() > 100 {
            return Err("连接器标识或名称无效".into());
        }
        let config = normalized_config(&config)?;
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if state.records.len() >= 16 && !state.records.iter().any(|r| r.id == id) {
            return Err("最多配置 16 个连接器".into());
        }
        let active = self.start(id, &config)?;
        let record = Record {
            id: id.into(),
            name: name.into(),
            config,
            enabled: true,
        };
        let mut records = state.records.clone();
        if let Some(existing) = records.iter_mut().find(|r| r.id == id) {
            *existing = record;
        } else {
            records.push(record);
        }
        self.save(&records)?;
        if let Some(old) = state.active.remove(id) {
            old.close();
        }
        state.records = records;
        state.errors.remove(id);
        state.active.insert(id.into(), active);
        Ok(status(&state, id))
    }
    pub fn reconnect(&self, id: &str) -> Result<Value, String> {
        let record = self
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .records
            .iter()
            .find(|r| r.id == id)
            .cloned()
            .ok_or("连接器未配置")?;
        self.connect(id, &record.name, record.config, true)
    }
    pub fn disconnect(&self, id: &str) -> Result<(), String> {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let mut records = state.records.clone();
        let record = records
            .iter_mut()
            .find(|r| r.id == id)
            .ok_or("连接器未配置")?;
        record.enabled = false;
        self.save(&records)?;
        if let Some(active) = state.active.remove(id) {
            active.close();
        }
        state.records = records;
        state.errors.remove(id);
        Ok(())
    }
    /// No credentials or connection configuration are returned to list views or model prompts.
    pub fn statuses(&self) -> Value {
        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        json!(state
            .records
            .iter()
            .map(|r| status(&state, &r.id))
            .collect::<Vec<_>>())
    }
    pub fn tools(&self) -> Vec<Arc<dyn Tool>> {
        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state
            .active
            .iter()
            .filter(|(_, a)| a.client.is_connected())
            .flat_map(|(id, a)| {
                a.client.tools().into_iter().map(move |tool| {
                    let mut spec = tool.spec().clone().always_ask().untrusted_output();
                    let hash = format!(
                        "{:x}",
                        Sha256::digest(format!("{id}\0{}", spec.name).as_bytes())
                    );
                    let safe = spec
                        .name
                        .chars()
                        .map(|c| {
                            if c.is_ascii_alphanumeric() || c == '_' {
                                c
                            } else {
                                '_'
                            }
                        })
                        .take(30)
                        .collect::<String>();
                    spec.name = format!("mcp_ext_{}_{safe}", &hash[..12]);
                    Arc::new(GatedTool {
                        tool,
                        spec,
                        valid: a.valid.clone(),
                    }) as Arc<dyn Tool>
                })
            })
            .collect()
    }
    pub fn shutdown(&self) {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        for (_, active) in state.active.drain() {
            active.close();
        }
    }
}
fn status(state: &State, id: &str) -> Value {
    let record = state.records.iter().find(|r| r.id == id);
    let active = state.active.get(id);
    json!({"id":id,"name":record.map(|r|r.name.as_str()).unwrap_or(id),"connected":active.is_some_and(|a|a.client.is_connected()),
        "enabled":record.is_some_and(|r|r.enabled),"toolCount":active.map(|a|a.client.tools().len()).unwrap_or(0),"error":state.errors.get(id)})
}
struct GatedTool {
    tool: Arc<dyn Tool>,
    spec: ToolSpec,
    valid: Arc<AtomicBool>,
}
impl Tool for GatedTool {
    fn spec(&self) -> &ToolSpec {
        &self.spec
    }
    fn execute(&self, args: Value, ctx: &ToolCtx) -> Result<String, String> {
        if !self.valid.load(Ordering::SeqCst) {
            return Err("连接器已断开，请重新连接后重试".into());
        }
        self.tool.execute(args, ctx)
    }
    fn execute_result(&self, args: Value, ctx: &ToolCtx) -> ToolResult {
        if !self.valid.load(Ordering::SeqCst) {
            return ToolResult::error("连接器已断开，请重新连接后重试");
        }
        self.tool.execute_result(args, ctx)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn config_import_and_placeholders() {
        assert_eq!(normalized_config(&json!({"mcpServers":{"test":{"command":"python3","args":["server.py"],"env":{"TOKEN":"abc"}}}})).unwrap()["env"]["TOKEN"],"abc");
        assert!(normalized_config(&json!({"url":"https://example.com/mcp?key=YOUR_KEY"})).is_err());
        assert!(normalized_config(&json!({"type":"sse","url":"https://example.com/sse"})).is_err());
        assert!(normalized_config(&json!({"command":"echo","args":"bad"})).is_err());
        assert!(normalized_config(&json!({"mcpServers":{"one":{},"two":{}}})).is_err());
    }
    fn test_root(label: &str) -> PathBuf {
        let n = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("xiaoban-mcp-{label}-{}-{n}", std::process::id()))
    }
    #[test]
    fn stdio_tools_are_persisted_and_revoked_after_disconnect() {
        let root = test_root("stdio");
        fs::create_dir_all(&root).unwrap();
        let script = root.join("server.py");
        fs::write(&script,r#"import sys,json
for line in sys.stdin:
 r=json.loads(line)
 if 'id' not in r: continue
 m=r['method']
 if m=='initialize': v={'protocolVersion':'2024-11-05','capabilities':{'tools':{}},'serverInfo':{'name':'test','version':'1'}}
 elif m=='tools/list': v={'tools':[{'name':'echo.with.dot','description':'test','inputSchema':{'type':'object'}}]}
 else: v={'content':[{'type':'text','text':'called'}],'structuredContent':{'ok':True}}
 print(json.dumps({'jsonrpc':'2.0','id':r['id'],'result':v}),flush=True)
"#).unwrap();
        let manager = ConnectorManager::new(root.join("config"), root.clone()).unwrap();
        let config = json!({"command":"python3","args":[script.to_str().unwrap()],"env":{"TEST_TOKEN":"must-not-be-in-status"}});
        assert!(manager
            .connect("test", "Test", config.clone(), false)
            .is_err());
        manager.connect("test", "Test", config, true).unwrap();
        let tools = manager.tools();
        assert_eq!(tools.len(), 1);
        assert!(tools[0].spec().always_ask && tools[0].spec().untrusted_output);
        assert!(tools[0].spec().name.starts_with("mcp_ext_"));
        assert!(tools[0].spec().name.len() <= 64);
        assert!(!tools[0].spec().name.contains('.'));
        let result = tools[0].execute_result(json!({}), &ToolCtx::detached("test"));
        assert!(!result.is_error);
        assert_eq!(result.structured_content.unwrap()["ok"], true);
        assert!(!manager
            .statuses()
            .to_string()
            .contains("must-not-be-in-status"));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(root.join("config/connections.json"))
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o777,
                0o600
            );
        }
        manager.disconnect("test").unwrap();
        assert!(manager.tools().is_empty());
        assert!(tools[0]
            .execute(json!({}), &ToolCtx::detached("test"))
            .is_err());
        drop(manager);
        let restored = ConnectorManager::new(root.join("config"), root.clone()).unwrap();
        assert!(restored.tools().is_empty());
        restored.reconnect("test").unwrap();
        assert_eq!(restored.tools().len(), 1);
        restored.shutdown();
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn http_negotiates_session_paginates_tools_and_reads_sse_result() {
        use std::io::{BufRead, BufReader, Read};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let thread = std::thread::spawn(move || {
            for step in 0..5 {
                let (mut socket, _) = listener.accept().unwrap();
                socket
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                let mut reader = BufReader::new(socket.try_clone().unwrap());
                let mut first = String::new();
                reader.read_line(&mut first).unwrap();
                let mut length = 0usize;
                let mut headers = String::new();
                loop {
                    let mut line = String::new();
                    reader.read_line(&mut line).unwrap();
                    if line.trim().is_empty() {
                        break;
                    }
                    if let Some(v) = line.to_lowercase().strip_prefix("content-length:") {
                        length = v.trim().parse().unwrap();
                    }
                    headers.push_str(&line.to_lowercase());
                }
                let mut bytes = vec![0; length];
                reader.read_exact(&mut bytes).unwrap();
                let request: Value = serde_json::from_slice(&bytes).unwrap();
                assert!(headers.contains("authorization: bearer secret"));
                if step > 0 {
                    assert!(headers.contains("mcp-session-id: session-test"));
                    assert!(headers.contains("mcp-protocol-version: 2025-06-18"));
                }
                let result = match step {
                    0 => {
                        json!({"protocolVersion":"2025-06-18","capabilities":{"tools":{}},"serverInfo":{"name":"test","version":"1"}})
                    }
                    1 => {
                        socket.write_all(b"HTTP/1.1 202 Accepted\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
                        continue;
                    }
                    2 => {
                        json!({"tools":[{"name":"first","inputSchema":{"type":"object"}}],"nextCursor":"page-two"})
                    }
                    3 => {
                        assert_eq!(request["params"]["cursor"], "page-two");
                        json!({"tools":[{"name":"second","inputSchema":{"type":"object"}}]})
                    }
                    _ => {
                        json!({"content":[{"type":"text","text":"real result"}],"structuredContent":{"answer":42}})
                    }
                };
                let response =
                    json!({"jsonrpc":"2.0","id":request["id"],"result":result}).to_string();
                let (mime, body) = if step == 4 {
                    ("text/event-stream",format!("data: {{\"jsonrpc\":\"2.0\",\"method\":\"notifications/progress\"}}\n\ndata: {response}\n\n"))
                } else {
                    ("application/json", response)
                };
                let session = if step == 0 {
                    "Mcp-Session-Id: session-test\r\n"
                } else {
                    ""
                };
                write!(socket,"HTTP/1.1 200 OK\r\nContent-Type: {mime}\r\n{session}Content-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).unwrap();
            }
        });
        let root = test_root("http");
        let manager = ConnectorManager::new(root.join("config"), root.clone()).unwrap();
        manager.connect("http-test","HTTP",json!({"url":format!("http://{address}/mcp"),"headers":{"Authorization":"Bearer secret"}}),true).unwrap();
        let tools = manager.tools();
        assert_eq!(tools.len(), 2);
        let result = tools[0].execute_result(json!({}), &ToolCtx::detached("http"));
        assert!(!result.is_error, "{}", result.output);
        assert_eq!(result.output, "real result");
        assert_eq!(result.structured_content.unwrap()["answer"], 42);
        assert!(!manager.statuses().to_string().contains("Bearer secret"));
        manager.disconnect("http-test").unwrap();
        assert!(
            tools[0]
                .execute_result(json!({}), &ToolCtx::detached("http"))
                .is_error
        );
        thread.join().unwrap();
        fs::remove_dir_all(root).unwrap();
    }
}
