//! Model Context Protocol (MCP) client over stdio.
//!
//! Spawns an external MCP server (such as `open-computer-use mcp` or custom tools),
//! completes the JSON-RPC 2.0 handshake, queries `tools/list`, and maps each tool into
//! an `Arc<dyn Tool>`. Output and errors go through the kernel's normal permission,
//! taint tracking and timeout mechanisms.

use crate::error::HarnessError;
use crate::tool::{Risk, Tool, ToolCtx, ToolSpec};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub const DEFAULT_PROTOCOL_VERSION: &str = "2024-11-05";

#[derive(Clone, Debug)]
pub struct McpConfig {
    pub command: String,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
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

    pub fn cwd(mut self, cwd: impl Into<PathBuf>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }
}

fn kill_tree(pid: u32) {
    #[cfg(unix)]
    let _ = Command::new("sh").args(["-c", &format!("kill -KILL -- -{pid}")]).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).status();
    #[cfg(windows)]
    let _ = Command::new("taskkill").args(["/PID", &pid.to_string(), "/T", "/F"]).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).status();
}

struct McpToolInfo {
    original_name: String,
    spec: ToolSpec,
}

pub struct McpClient {
    stdin: Mutex<ChildStdin>,
    pending: Arc<Mutex<HashMap<u64, Sender<Result<Value, String>>>>>,
    next_id: AtomicU64,
    tools: Vec<McpToolInfo>,
    child: Mutex<Option<Child>>,
    reader: Mutex<Option<JoinHandle<()>>>,
    closed: Arc<AtomicBool>,
}

impl Drop for McpClient {
    fn drop(&mut self) {
        self.closed.store(true, Ordering::SeqCst);
        if let Ok(mut child_lock) = self.child.lock() {
            if let Some(mut child) = child_lock.take() {
                kill_tree(child.id());
                let _ = child.kill();
                let _ = child.wait();
            }
        }
        if let Ok(mut r) = self.reader.lock() {
            if let Some(h) = r.take() {
                let _ = h.join();
            }
        }
    }
}

impl McpClient {
    pub fn spawn(config: McpConfig) -> Result<Arc<Self>, HarnessError> {
        let mut cmd = Command::new(&config.command);
        cmd.args(&config.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        for (k, v) in &config.env {
            cmd.env(k, v);
        }
        if let Some(cwd) = &config.cwd {
            cmd.current_dir(cwd);
        }
        #[cfg(unix)]
        std::os::unix::process::CommandExt::process_group(&mut cmd, 0);

        let mut child = cmd.spawn().map_err(|e| {
            HarnessError::Config(format!("cannot start MCP server `{}`: {e}", config.command))
        })?;

        let stdin = child.stdin.take().ok_or_else(|| {
            HarnessError::Config("failed to capture MCP server stdin".into())
        })?;
        let stdout = child.stdout.take().ok_or_else(|| {
            HarnessError::Config("failed to capture MCP server stdout".into())
        })?;

        let pending: Arc<Mutex<HashMap<u64, Sender<Result<Value, String>>>>> = Arc::default();
        let closed = Arc::new(AtomicBool::new(false));

        let reader_pending = pending.clone();
        let reader_closed = closed.clone();
        let reader_thread = thread::Builder::new()
            .name(format!("mcp-{}", config.command))
            .spawn(move || {
                let mut reader = BufReader::new(stdout);
                let mut line = String::new();
                loop {
                    line.clear();
                    match reader.read_line(&mut line) {
                        Ok(0) => break,
                        Ok(_) => {
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
                                    if let Some(tx) = tx {
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
                // Drop all remaining pending requests with an error so they do not hang
                let mut lock = reader_pending.lock().unwrap_or_else(|e| e.into_inner());
                for (_, tx) in lock.drain() {
                    let _ = tx.send(Err("MCP server closed".into()));
                }
            })
            .map_err(|e| HarnessError::Config(format!("failed to start MCP reader thread: {e}")))?;

        let client = Arc::new(Self {
            stdin: Mutex::new(stdin),
            pending,
            next_id: AtomicU64::new(1),
            tools: Vec::new(),
            child: Mutex::new(Some(child)),
            reader: Mutex::new(Some(reader_thread)),
            closed,
        });

        // 1. Handshake: initialize
        let init_params = json!({
            "protocolVersion": DEFAULT_PROTOCOL_VERSION,
            "capabilities": {},
            "clientInfo": {
                "name": "agent-kernel",
                "version": "0.1.0"
            }
        });
        let init_res = client.send_request("initialize", init_params, Duration::from_secs(15))
            .map_err(|e| HarnessError::Config(format!("MCP initialize handshake failed: {e}")))?;
        let _ = init_res;

        // 2. Notification: initialized
        client.send_notification("notifications/initialized", json!({}))
            .map_err(|e| HarnessError::Config(format!("MCP initialized notification failed: {e}")))?;

        // 3. Query tools/list
        let tools_res = client.send_request("tools/list", json!({}), Duration::from_secs(15))
            .map_err(|e| HarnessError::Config(format!("MCP tools/list failed: {e}")))?;

        let mut tool_infos = Vec::new();
        if let Some(tools_arr) = tools_res.get("tools").and_then(Value::as_array) {
            for t in tools_arr {
                if let Some(orig_name) = t.get("name").and_then(Value::as_str) {
                    let desc = t.get("description").and_then(Value::as_str).unwrap_or("").to_string();
                    let schema = t.get("inputSchema").cloned().unwrap_or_else(|| json!({"type": "object"}));
                    let exposed_name = if config.tool_prefix.is_empty() {
                        orig_name.to_string()
                    } else {
                        format!("{}{}", config.tool_prefix, orig_name)
                    };
                    let spec = ToolSpec::new(&exposed_name, &desc, schema)
                        .risk(config.default_risk)
                        .serial()
                        .timeout(config.timeout);
                    tool_infos.push(McpToolInfo {
                        original_name: orig_name.to_string(),
                        spec,
                    });
                }
            }
        }

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
            pending.insert(id, tx);
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

    pub fn call_tool(&self, original_name: &str, args: Value, ctx: &ToolCtx) -> Result<String, String> {
        if ctx.is_cancelled() {
            return Err("cancelled".into());
        }
        let id = self.next_request_id();
        let (tx, rx) = mpsc::channel();
        {
            let mut pending = self.pending.lock().unwrap_or_else(|e| e.into_inner());
            pending.insert(id, tx);
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
                self.pending.lock().unwrap_or_else(|p| p.into_inner()).remove(&id);
                return Err(format!("failed to send to MCP stdin: {e}"));
            }
        }

        let res = match ctx.wait_for(&rx) {
            Some(res) => res?,
            None => {
                self.pending.lock().unwrap_or_else(|p| p.into_inner()).remove(&id);
                return Err("cancelled or timed out".into());
            }
        };

        let is_error = res.get("isError").and_then(Value::as_bool).unwrap_or(false);
        let mut out = String::new();
        if let Some(content) = res.get("content").and_then(Value::as_array) {
            for part in content {
                match part.get("type").and_then(Value::as_str) {
                    Some("text") => {
                        if let Some(t) = part.get("text").and_then(Value::as_str) {
                            if !out.is_empty() {
                                out.push('\n');
                            }
                            out.push_str(t);
                        }
                    }
                    Some("image") => {
                        let mime = part.get("mimeType").and_then(Value::as_str).unwrap_or("image");
                        let len = part.get("data").and_then(Value::as_str).map_or(0, |s| s.len());
                        if !out.is_empty() {
                            out.push('\n');
                        }
                        out.push_str(&format!("[image: {mime} ({len} bytes)]"));
                    }
                    _ => {}
                }
            }
        }

        if is_error {
            Err(if out.is_empty() { "MCP tool reported an error".into() } else { out })
        } else {
            Ok(if out.is_empty() { "ok".into() } else { out })
        }
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
}
