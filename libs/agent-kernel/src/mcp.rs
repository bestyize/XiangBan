//! Model Context Protocol (MCP) client over stdio.
//!
//! Spawns an external MCP server (such as `open-computer-use mcp` or custom tools),
//! completes the JSON-RPC 2.0 handshake, queries `tools/list`, and maps each tool into
//! an `Arc<dyn Tool>`. Output and errors go through the kernel's normal permission,
//! taint tracking and timeout mechanisms.

use crate::error::HarnessError;
use crate::tool::{Risk, Tool, ToolCtx, ToolResult, ToolSpec};
use crate::Part;
use base64::Engine;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::PathBuf;
use std::process::{ChildStdin, Command, Stdio};
use crate::process_tree::ChildTree;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

#[cfg(any(feature = "openai", feature = "anthropic", feature = "web"))]
mod http;
#[cfg(any(feature = "openai", feature = "anthropic", feature = "web"))]
pub use http::{HttpAuthorization, HttpMcpClient, HttpMcpConfig};

pub const DEFAULT_PROTOCOL_VERSION: &str = "2024-11-05";
const MAX_FRAME_BYTES: u64 = 8 * 1024 * 1024;
const MAX_IMAGE_BYTES: usize = 4 * 1024 * 1024;


#[derive(Clone, Debug)]
pub struct McpConfig {
    pub command: String,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    /// Extra host environment keys to pass through. Credentials are never inherited by default.
    pub inherited_env: Vec<String>,
    pub cwd: Option<PathBuf>,
    /// Optional prefix for tool names (e.g. `ocu_` or empty).
    pub tool_prefix: String,
    /// Risk level assigned to tools (defaults to Risk::Exec).
    pub default_risk: Risk,
    /// Timeout per tool call (defaults to 120s).
    pub timeout: Duration,
}

impl McpConfig {
    pub fn new(command: impl Into<String>, args: &[&str]) -> Self {
        Self {
            command: command.into(),
            args: args.iter().map(|s| s.to_string()).collect(),
            env: Vec::new(),
            inherited_env: Vec::new(),
            cwd: None,
            tool_prefix: String::new(),
            default_risk: Risk::Exec,
            timeout: Duration::from_secs(120),
        }
    }

    pub fn tool_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.tool_prefix = prefix.into();
        self
    }

    pub fn default_risk(mut self, risk: Risk) -> Self {
        self.default_risk = risk;
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn env(mut self, key: &str, value: &str) -> Self {
        self.env.push((key.to_string(), value.to_string()));
        self
    }

    pub fn inherit_env(mut self, key: &str) -> Self {
        self.inherited_env.push(key.to_string());
        self
    }

    pub fn cwd(mut self, cwd: impl Into<PathBuf>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }
}


struct McpToolInfo {
    original_name: String,
    spec: ToolSpec,
}

struct PendingRequest {
    tx: Sender<Result<Value, String>>,
    // Cancellation ends the caller's wait, not the server's operation. Retain ownership
    // until a response arrives or the local process is explicitly closed.
    _control: Option<crate::RunControl>,
}

pub struct McpClient {
    stdin: Mutex<ChildStdin>,
    pending: Arc<Mutex<HashMap<u64, PendingRequest>>>,
    next_id: AtomicU64,
    tools: Vec<McpToolInfo>,
    child: Arc<Mutex<Option<ChildTree>>>,
    reader: Mutex<Option<JoinHandle<()>>>,
    closed: Arc<AtomicBool>,
}

impl Drop for McpClient {
    fn drop(&mut self) {
        self.closed.store(true, Ordering::SeqCst);
        if let Ok(mut child_lock) = self.child.lock() {
            if let Some(mut child) = child_lock.take() {
                child.terminate();
                let _ = child.kill();
                let _ = child.wait();
            }
        }
        if let Ok(mut r) = self.reader.lock() {
            if let Some(h) = r.take() {
                let deadline = std::time::Instant::now()+Duration::from_secs(1);
                while !h.is_finished() && std::time::Instant::now()<deadline { thread::sleep(Duration::from_millis(10)); }
                if h.is_finished() { let _=h.join(); }
            }
        }
    }
}

impl McpClient {
    pub fn is_connected(&self) -> bool {
        !self.closed.load(Ordering::SeqCst)
    }

    /// Revoke a connection even when an in-flight Agent still owns tool wrappers.
    pub fn close(&self) {
        self.closed.store(true, Ordering::SeqCst);
        if let Some(mut child) = self.child.lock().unwrap_or_else(|e|e.into_inner()).take() {
            child.terminate();
            let _ = child.kill();
            let _ = child.wait();
        }
        for (_,request) in self.pending.lock().unwrap_or_else(|e|e.into_inner()).drain(){let _=request.tx.send(Err("MCP closed".into()));}
    }

    pub fn spawn(config: McpConfig) -> Result<Arc<Self>, HarnessError> {Self::spawn_cancellable(config,Arc::new(AtomicBool::new(false)))}
    pub fn spawn_cancellable(config:McpConfig,cancel:Arc<AtomicBool>)->Result<Arc<Self>,HarnessError>{
        if cancel.load(Ordering::SeqCst){return Err(HarnessError::Cancelled);}
        let mut cmd = Command::new(crate::child_env::program(&config.command));
        cmd.args(&config.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        crate::child_env::apply(&mut cmd, &config.inherited_env);
        for (k, v) in &config.env {
            cmd.env(k, v);
        }
        if let Some(cwd) = &config.cwd {
            cmd.current_dir(cwd);
        }
        #[cfg(unix)]
        std::os::unix::process::CommandExt::process_group(&mut cmd, 0);
        // Native console MCP servers should not open a console window in desktop hosts.
        #[cfg(windows)]
        std::os::windows::process::CommandExt::creation_flags(&mut cmd, 0x08000000); // CREATE_NO_WINDOW

        let mut child = ChildTree::spawn(&mut cmd).map_err(|e| {
            HarnessError::Config(format!("cannot start MCP server `{}`: {e}", config.command))
        })?;

        let stdin = child.stdin.take().ok_or_else(|| {
            HarnessError::Config("failed to capture MCP server stdin".into())
        })?;
        let stdout = child.stdout.take().ok_or_else(|| {
            HarnessError::Config("failed to capture MCP server stdout".into())
        })?;

        let pending: Arc<Mutex<HashMap<u64, PendingRequest>>> = Arc::default();
        let closed = Arc::new(AtomicBool::new(false));
        let child = Arc::new(Mutex::new(Some(child)));

        let reader_pending = pending.clone();
        let reader_closed = closed.clone();
        let reader_child = child.clone();
        let reader_thread = thread::Builder::new()
            .name(format!("mcp-{}", config.command))
            .spawn(move || {
                let mut reader = BufReader::new(stdout);
                let mut line = String::new();
                loop {
                    line.clear();
                    match (&mut reader).take(MAX_FRAME_BYTES + 1).read_line(&mut line) {
                        Ok(0) => break,
                        Ok(_) => {
                            if line.len() as u64 > MAX_FRAME_BYTES { break; }
                            let text = line.trim();
                            if text.is_empty() {
                                continue;
                            }
                            if let Ok(val) = serde_json::from_str::<Value>(text) {
                                if let Some(id) = val.get("id").and_then(Value::as_u64) {
                                    let tx = {
                                        let mut lock = reader_pending.lock().unwrap_or_else(|e| e.into_inner());
                                        lock.remove(&id)
                                    };
                                    if let Some(request) = tx {
                                        let tx = request.tx;
                                        if let Some(err) = val.get("error") {
                                            let msg = err.get("message")
                                                .and_then(Value::as_str)
                                                .unwrap_or("MCP error");
                                            let _ = tx.send(Err(msg.to_string()));
                                        } else if let Some(result) = val.get("result") {
                                            let _ = tx.send(Ok(result.clone()));
                                        } else {
                                            let _ = tx.send(Ok(val.clone()));
                                        }
                                    }
                                }
                            }
                        }
                        Err(_) => break,
                    }
                }
                reader_closed.store(true, Ordering::SeqCst);
                // EOF is not proof of process termination: a server can close stdout
                // and keep changing state. Stop its process tree before dropping leases.
                if let Some(mut child) = reader_child.lock().unwrap_or_else(|e| e.into_inner()).take() {
                    child.terminate();
                    let _ = child.wait();
                }
                // Drop all remaining pending requests with an error so they do not hang
                let mut lock = reader_pending.lock().unwrap_or_else(|e| e.into_inner());
                for (_, request) in lock.drain() {
                    let _ = request.tx.send(Err("MCP server closed".into()));
                }
            })
            .map_err(|e| HarnessError::Config(format!("failed to start MCP reader thread: {e}")))?;

        let client = Arc::new(Self {
            stdin: Mutex::new(stdin),
            pending,
            next_id: AtomicU64::new(1),
            tools: Vec::new(),
            child,
            reader: Mutex::new(Some(reader_thread)),
            closed,
        });

        let c=client.clone();let deadline=std::time::Instant::now()+Duration::from_secs(45);
        let watch=crate::StopWatch::new(move||cancel.load(Ordering::SeqCst)||std::time::Instant::now()>=deadline,move||c.close());
        // 1. Handshake: initialize
        let init_params = json!({
            "protocolVersion": DEFAULT_PROTOCOL_VERSION,
            "capabilities": {},
            "clientInfo": {
                "name": "agent-kernel",
                "version": env!("CARGO_PKG_VERSION")
            }
        });
        let init_res = client.send_request("initialize", init_params, Duration::from_secs(15))
            .map_err(|e| HarnessError::Config(format!("MCP initialize handshake failed: {e}")))?;
        let _ = init_res;

        // 2. Notification: initialized
        client.send_notification("notifications/initialized", json!({}))
            .map_err(|e| HarnessError::Config(format!("MCP initialized notification failed: {e}")))?;

        let mut tool_infos = Vec::new();
        let mut cursor: Option<String> = None;
        let mut cursors = std::collections::HashSet::new();
        let mut names = std::collections::HashSet::new();
        for page in 0..20 {
            let params=cursor.as_ref().map_or_else(||json!({}),|c|json!({"cursor":c}));
            let result=client.send_request("tools/list",params,Duration::from_secs(15)).map_err(HarnessError::Config)?;
            let tools=result["tools"].as_array().ok_or_else(||HarnessError::Config("invalid MCP catalog".into()))?;
            for t in tools {
                let name=t["name"].as_str().filter(|n|!n.is_empty()).ok_or_else(||HarnessError::Config("missing MCP name".into()))?;
                if !names.insert(name.to_string()) || tool_infos.len()>=500 {return Err(HarnessError::Config("duplicate/oversized MCP catalog".into()));}
                let spec=ToolSpec::new(format!("{}{name}",config.tool_prefix),t["description"].as_str().unwrap_or(""),t.get("inputSchema").cloned().unwrap_or_else(||json!({"type":"object"}))).risk(config.default_risk).untrusted_output().serial().timeout(config.timeout);
                tool_infos.push(McpToolInfo{original_name:name.into(),spec});
            }
            cursor=match result.get("nextCursor") {None|Some(Value::Null)=>None,Some(Value::String(c)) if !c.is_empty()=>Some(c.clone()),_=>return Err(HarnessError::Config("invalid MCP cursor".into()))};
            let Some(c)=&cursor else {break;};
            if page==19 || !cursors.insert(c.clone()){return Err(HarnessError::Config("MCP pagination did not terminate".into()));}
        }

        drop(watch);
        // Drop the temporary ref and place tool_infos inside client
        let mut final_client = match Arc::try_unwrap(client) {
            Ok(c) => c,
            Err(_) => return Err(HarnessError::Config("unexpected refcount in MCP initialization".into())),
        };
        final_client.tools = tool_infos;
        Ok(Arc::new(final_client))
    }

    fn next_request_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }

    fn send_notification(&self, method: &str, params: Value) -> Result<(), String> {
        let notif = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        });
        let mut data = serde_json::to_vec(&notif).map_err(|e| e.to_string())?;
        data.push(b'\n');
        let mut stdin = self.stdin.lock().unwrap_or_else(|e| e.into_inner());
        stdin.write_all(&data).map_err(|e| e.to_string())?;
        stdin.flush().map_err(|e| e.to_string())
    }

    fn send_request(&self, method: &str, params: Value, timeout: Duration) -> Result<Value, String> {
        if self.closed.load(Ordering::SeqCst) {
            return Err("MCP server is closed".into());
        }
        let id = self.next_request_id();
        let (tx, rx) = mpsc::channel();
        {
            let mut pending = self.pending.lock().unwrap_or_else(|e| e.into_inner());
            pending.insert(id, PendingRequest { tx, _control: None });
        }

        let req = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });
        let mut data = serde_json::to_vec(&req).map_err(|e| e.to_string())?;
        data.push(b'\n');

        {
            let mut stdin = self.stdin.lock().unwrap_or_else(|e| e.into_inner());
            if let Err(e) = stdin.write_all(&data).and_then(|_| stdin.flush()) {
                self.pending.lock().unwrap_or_else(|p| p.into_inner()).remove(&id);
                return Err(format!("failed to send to MCP stdin: {e}"));
            }
        }

        match rx.recv_timeout(timeout) {
            Ok(res) => res,
            Err(_) => {
                self.pending.lock().unwrap_or_else(|p| p.into_inner()).remove(&id);
                Err("MCP request timed out".into())
            }
        }
    }

    /// Legacy text-only projection. The harness uses `call_tool_result` to preserve media.
    pub fn call_tool(&self, original_name: &str, args: Value, ctx: &ToolCtx) -> Result<String, String> {
        let result = self.call_tool_result(original_name, args, ctx)?;
        let text = result.message("legacy").tool_text();
        if result.is_error { Err(text) } else { Ok(text) }
    }

    pub fn call_tool_result(&self, original_name: &str, args: Value, ctx: &ToolCtx) -> Result<ToolResult, String> {
        if ctx.is_cancelled() || !self.is_connected() {
            return Err("cancelled".into());
        }
        let id = self.next_request_id();
        let (tx, rx) = mpsc::channel();
        {
            let mut pending = self.pending.lock().unwrap_or_else(|e| e.into_inner());
            pending.insert(id, PendingRequest { tx, _control: Some(ctx.control().clone()) });
        }

        let req = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "tools/call",
            "params": {
                "name": original_name,
                "arguments": args,
            }
        });
        let mut data = serde_json::to_vec(&req).map_err(|e| e.to_string())?;
        data.push(b'\n');

        {
            let mut stdin = self.stdin.lock().unwrap_or_else(|e| e.into_inner());
            if let Err(e) = stdin.write_all(&data).and_then(|_| stdin.flush()) {
                return Ok(ToolResult::unknown(&format!("failed to send to MCP stdin: {e}")));
            }
        }

        let res = match ctx.wait_for(&rx) {
            Some(Ok(res)) => res,
            Some(Err(e))=>return Ok(ToolResult::unknown(&e)),
            None => {
                // MCP cancellation is advisory. Acknowledgement is the eventual execution
                // response, so do not remove the request or its resource lease here.
                let _ = self.send_notification("notifications/cancelled", json!({
                    "requestId": id, "reason": "caller cancelled or tool deadline expired"
                }));
                return Ok(ToolResult::unknown("MCP call cancelled without an execution result"));
            }
        };

        decode_result(&res)
    }

    pub fn tools(self: &Arc<Self>) -> Vec<Arc<dyn Tool>> {
        self.tools.iter().map(|info| {
            Arc::new(McpToolWrapper {
                client: self.clone(),
                original_name: info.original_name.clone(),
                spec: info.spec.clone(),
            }) as Arc<dyn Tool>
        }).collect()
    }
}

struct McpToolWrapper {
    client: Arc<McpClient>,
    original_name: String,
    spec: ToolSpec,
}

impl Tool for McpToolWrapper {
    fn spec(&self) -> &ToolSpec {
        &self.spec
    }

    fn execute(&self, args: Value, ctx: &ToolCtx) -> Result<String, String> {
        self.client.call_tool(&self.original_name, args, ctx)
    }

    fn execute_result(&self, args: Value, ctx: &ToolCtx) -> ToolResult {
        if ctx.is_cancelled() || !self.client.is_connected() {
            return ToolResult::error("MCP call was not dispatched: cancelled or disconnected");
        }
        self.client.call_tool_result(&self.original_name, args, ctx)
            .unwrap_or_else(|error| ToolResult::unknown(&error))
    }
}

/// Decode supported MCP content without turning image bytes into a text placeholder.
fn decode_result(res: &Value) -> Result<ToolResult, String> {
    let mut result = ToolResult::ok("");
    result.is_error = res.get("isError").and_then(Value::as_bool).unwrap_or(false);
    result.structured_content = res.get("structuredContent").cloned();
    if let Some(content) = res.get("content") {
        let content = content.as_array().ok_or("MCP result content must be an array")?;
        for part in content {
            let text = match part.get("type").and_then(Value::as_str) {
                Some("text") => part.get("text").and_then(Value::as_str).ok_or("MCP text block is missing text")?.to_string(),
                Some("image") => {
                    let mime = part.get("mimeType").and_then(Value::as_str).ok_or("MCP image is missing mimeType")?;
                    if !matches!(mime, "image/png" | "image/jpeg" | "image/gif" | "image/webp") {
                        return Err(format!("unsupported MCP image MIME type: {mime}"));
                    }
                    let data = part.get("data").and_then(Value::as_str).ok_or("MCP image is missing data")?;
                    if data.len() > MAX_IMAGE_BYTES.div_ceil(3) * 4 { return Err("MCP image exceeds 4 MiB".into()); }
                    let bytes = base64::engine::general_purpose::STANDARD.decode(data).map_err(|_| "MCP image data is not valid base64")?;
                    if bytes.len() > MAX_IMAGE_BYTES { return Err("MCP image exceeds 4 MiB".into()); }
                    result.parts.push(Part::Image { url: format!("data:{mime};base64,{data}") });
                    format!("[image: {mime}]")
                }
                Some("resource") => {
                    let resource = part.get("resource").ok_or("MCP resource block is missing resource")?;
                    resource.get("text").and_then(Value::as_str).ok_or("binary MCP resources are not supported")?.to_string()
                }
                Some("resource_link") => format!("[resource: {}]", part.get("uri").and_then(Value::as_str).ok_or("MCP resource link is missing uri")?),
                other => return Err(format!("unsupported MCP content block: {}", other.unwrap_or("missing type"))),
            };
            if !result.output.is_empty() { result.output.push('\n'); }
            result.output.push_str(&text);
        }
    }
    if result.output.is_empty() && result.structured_content.is_none() {
        result.output = if result.is_error { "MCP tool reported an error" } else { "ok" }.into();
    }
    Ok(result)
}
