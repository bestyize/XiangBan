//! MCP 2024-11-05 HTTP+SSE: one GET stream, same-origin POST endpoint,
//! and response dispatch by JSON-RPC id. A lost stream is never replayed.
use super::{http::rpc_result, HttpAuthorization, HttpMcpClient, HttpMcpConfig, MAX_FRAME_BYTES};
use crate::Tool;
use reqwest::{header::HeaderMap, Client, Url};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};

/// Uses the same handshake, tool policies, authorization and uncertainty tracking
/// as Streamable HTTP. Closing or cancelling any call closes the whole session.
pub struct SseMcpClient(Arc<HttpMcpClient>);
impl SseMcpClient {
    pub fn connect(config: HttpMcpConfig) -> Result<Arc<Self>, String> {
        Self::connect_inner(config, None, None)
    }
    pub fn connect_with_authorization(
        config: HttpMcpConfig,
        auth: Arc<dyn HttpAuthorization>,
    ) -> Result<Arc<Self>, String> {
        Self::connect_inner(config, Some(auth), None)
    }
    pub fn connect_cancellable(
        config: HttpMcpConfig,
        auth: Option<Arc<dyn HttpAuthorization>>,
        cancel: Arc<AtomicBool>,
    ) -> Result<Arc<Self>, String> {
        Self::connect_inner(config, auth, Some(cancel))
    }
    fn connect_inner(
        config: HttpMcpConfig,
        auth: Option<Arc<dyn HttpAuthorization>>,
        cancel: Option<Arc<AtomicBool>>,
    ) -> Result<Arc<Self>, String> {
        HttpMcpClient::connect_legacy(config, auth, cancel).map(|c| Arc::new(Self(c)))
    }
    pub fn is_connected(&self) -> bool {
        self.0.is_connected()
    }
    pub fn needs_review(&self) -> bool {
        self.0.needs_review()
    }
    pub fn close(&self) {
        self.0.close();
    }
    pub fn tools(self: &Arc<Self>) -> Vec<Arc<dyn Tool>> {
        self.0.tools()
    }
}

type Reply = Result<Value, String>;
pub(super) struct Stream {
    pub endpoint: Url,
    shared: Arc<Shared>,
}
struct Shared {
    closed: Arc<AtomicBool>,
    pending: Mutex<HashMap<u64, Sender<Reply>>>,
    error: Mutex<Option<String>>,
}
impl Shared {
    fn fail(&self, error: String) {
        *self.error.lock().unwrap_or_else(|e| e.into_inner()) = Some(error.clone());
        self.closed.store(true, Ordering::SeqCst);
        for (_, sender) in self
            .pending
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .drain()
        {
            let _ = sender.send(Err(error.clone()));
        }
    }
}
impl Stream {
    pub fn open(
        client: Client,
        url: Url,
        headers: HeaderMap,
        auth: Option<Arc<dyn HttpAuthorization>>,
        closed: Arc<AtomicBool>,
        timeout: Duration,
        cancel: impl Fn() -> bool,
    ) -> Result<Self, String> {
        let shared = Arc::new(Shared {
            closed,
            pending: Mutex::new(HashMap::new()),
            error: Mutex::new(None),
        });
        let data = shared.clone();
        let (tx, rx) = mpsc::channel();
        std::thread::Builder::new()
            .name("mcp-legacy-sse".into())
            .spawn(move || {
                let result = read_stream(&client, &url, headers, auth, &data, timeout, &tx);
                let error = result
                    .err()
                    .unwrap_or_else(|| "MCP SSE stream disconnected".into());
                let _ = tx.send(Err(error.clone()));
                data.fail(error);
            })
            .map_err(|_| "Cannot start MCP SSE reader")?;
        let deadline = Instant::now() + timeout;
        loop {
            if cancel() || Instant::now() >= deadline {
                shared.fail("MCP SSE endpoint cancelled or timed out".into());
                return Err("MCP SSE endpoint cancelled or timed out".into());
            }
            match rx.recv_timeout(Duration::from_millis(20)) {
                Ok(Ok(endpoint)) => return Ok(Self { endpoint, shared }),
                Ok(Err(e)) => return Err(e),
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    return Err("MCP SSE reader stopped".into())
                }
                Err(mpsc::RecvTimeoutError::Timeout) => (),
            }
        }
    }
    pub fn register(&self, id: u64) -> Result<Receiver<Reply>, String> {
        let (tx, rx) = mpsc::channel();
        let mut pending = self
            .shared
            .pending
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if self.shared.closed.load(Ordering::SeqCst) {
            return Err("MCP SSE disconnected".into());
        }
        pending.insert(id, tx);
        Ok(rx)
    }
    pub fn remove(&self, id: u64) {
        self.shared
            .pending
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&id);
    }
    pub fn wait(
        &self,
        rx: &Receiver<Reply>,
        cancel: impl Fn() -> bool,
        deadline: Instant,
    ) -> Reply {
        loop {
            if cancel() || Instant::now() >= deadline {
                let error = self
                    .shared
                    .error
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .clone()
                    .unwrap_or_else(|| "MCP SSE request cancelled or timed out".into());
                self.shared.fail(error.clone());
                return Err(error);
            }
            match rx.recv_timeout(Duration::from_millis(20)) {
                Ok(result) => return result,
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    return Err("MCP SSE disconnected".into())
                }
                Err(mpsc::RecvTimeoutError::Timeout) => (),
            }
        }
    }
}
impl Drop for Stream {
    fn drop(&mut self) {
        self.shared.fail("MCP SSE closed".into());
    }
}

fn read_stream(
    client: &Client,
    url: &Url,
    mut headers: HeaderMap,
    auth: Option<Arc<dyn HttpAuthorization>>,
    shared: &Shared,
    timeout: Duration,
    endpoint: &Sender<Result<Url, String>>,
) -> Result<(), String> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;
    let cancel = || shared.closed.load(Ordering::SeqCst);
    let mut response = None;
    for attempt in 0..2 {
        // OAuth refresh may use blocking reqwest and must stay outside the runtime.
        if let Some(auth) = &auth {
            let value = format!("Bearer {}", auth.access_token(attempt != 0)?);
            headers.insert(
                "authorization",
                value.parse().map_err(|_| "Invalid MCP authorization")?,
            );
        }
        let result = rt
            .block_on(crate::net::wait(
                client
                    .get(url.clone())
                    .headers(headers.clone())
                    .header("Accept", "text/event-stream")
                    .send(),
                cancel,
                timeout,
            ))
            .map_err(|e| e.to_string())?
            .map_err(|_| "MCP SSE GET failed or timed out")?;
        if result.status().as_u16() == 401 && auth.is_some() && attempt == 0 {
            continue;
        }
        if !result.status().is_success() {
            return Err(format!("MCP HTTP {} on SSE GET", result.status().as_u16()));
        }
        if !result
            .headers()
            .get("content-type")
            .and_then(|h| h.to_str().ok())
            .is_some_and(|h| {
                h.split(';')
                    .next()
                    .unwrap_or("")
                    .trim()
                    .eq_ignore_ascii_case("text/event-stream")
            })
        {
            return Err("MCP SSE GET must return text/event-stream".into());
        }
        response = Some(result);
        break;
    }
    let mut response = response.ok_or("MCP SSE authorization failed")?;
    let mut parser = EventParser::default();
    let mut target = None;
    loop {
        // No reqwest whole-response timeout: a healthy SSE connection is long lived.
        let chunk = rt
            .block_on(crate::net::wait(
                response.chunk(),
                cancel,
                Duration::from_secs(365 * 86400),
            ))
            .map_err(|e| e.to_string())?
            .map_err(|_| "MCP SSE read failed")?;
        let Some(chunk) = chunk else {
            return Err("MCP SSE stream disconnected".into());
        };
        for byte in chunk {
            let Some((event, text)) = parser.push(byte)? else {
                continue;
            };
            match event.as_str() {
                "endpoint" => {
                    if target.is_some() {
                        return Err("MCP SSE duplicate endpoint".into());
                    }
                    let resolved = url
                        .join(text.trim())
                        .map_err(|_| "Invalid MCP SSE endpoint")?;
                    if resolved.origin() != url.origin()
                        || !resolved.username().is_empty()
                        || resolved.password().is_some()
                        || resolved.fragment().is_some()
                    {
                        return Err("MCP SSE endpoint must be same-origin".into());
                    }
                    endpoint
                        .send(Ok(resolved.clone()))
                        .map_err(|_| "MCP SSE startup cancelled")?;
                    target = Some(resolved);
                }
                "message" => {
                    let value: Value =
                        serde_json::from_str(&text).map_err(|_| "Invalid MCP SSE JSON")?;
                    if let Some(method) = value["method"].as_str() {
                        if let Some(id) = value
                            .get("id")
                            .filter(|id| id.is_string() || id.is_number())
                        {
                            let reply = if method == "ping" {
                                json!({"jsonrpc":"2.0","id":id,"result":{}})
                            } else {
                                json!({"jsonrpc":"2.0","id":id,"error":{"code":-32601,"message":"Method not supported"}})
                            };
                            let target = target.as_ref().ok_or("MCP message before endpoint")?;
                            for attempt in 0..2 {
                                if let Some(auth) = &auth {
                                    headers.insert(
                                        "authorization",
                                        format!("Bearer {}", auth.access_token(attempt != 0)?)
                                            .parse()
                                            .map_err(|_| "Invalid MCP authorization")?,
                                    );
                                }
                                let result = rt
                                    .block_on(crate::net::wait(
                                        client
                                            .post(target.clone())
                                            .headers(headers.clone())
                                            .header("Content-Type", "application/json")
                                            .body(reply.to_string())
                                            .send(),
                                        cancel,
                                        timeout,
                                    ))
                                    .map_err(|e| e.to_string())?
                                    .map_err(|_| "MCP SSE server request reply failed")?;
                                if result.status().as_u16() == 401 && auth.is_some() && attempt == 0
                                {
                                    continue;
                                }
                                if !result.status().is_success() {
                                    return Err(format!(
                                        "MCP HTTP {} on SSE reply",
                                        result.status().as_u16()
                                    ));
                                }
                                break;
                            }
                        }
                    } else if let Some(id) = value["id"].as_u64() {
                        if let Some(sender) = shared
                            .pending
                            .lock()
                            .unwrap_or_else(|e| e.into_inner())
                            .remove(&id)
                        {
                            let _ = sender.send(rpc_result(value, id));
                        }
                    }
                }
                _ => (), // Comments, retry hints and extension events do not initiate reconnection.
            }
        }
    }
}

/// Limit raw bytes per event (including comments/unknown fields) and per line,
/// rather than cumulative stream bytes. Parsing works across arbitrary chunks.
#[derive(Default)]
struct EventParser {
    line: Vec<u8>,
    event: String,
    data: String,
    size: u64,
}
impl EventParser {
    fn push(&mut self, byte: u8) -> Result<Option<(String, String)>, String> {
        self.size += 1;
        if self.size > MAX_FRAME_BYTES {
            return Err("MCP SSE event exceeds 8 MiB".into());
        }
        if byte != b'\n' {
            self.line.push(byte);
            return Ok(None);
        }
        let line = std::mem::take(&mut self.line);
        let line = std::str::from_utf8(&line)
            .map_err(|_| "Invalid MCP SSE UTF-8")?
            .trim_end_matches('\r');
        if line.is_empty() {
            self.size = 0;
            let event = std::mem::take(&mut self.event);
            let data = std::mem::take(&mut self.data);
            return Ok((!data.is_empty()).then(|| {
                (
                    if event.is_empty() {
                        "message".into()
                    } else {
                        event
                    },
                    data,
                )
            }));
        }
        let (field, value) = line.split_once(':').unwrap_or((line, ""));
        let value = value.strip_prefix(' ').unwrap_or(value);
        match field {
            "event" => self.event = value.into(),
            "data" => {
                self.data.push_str(value);
                self.data.push('\n');
            }
            _ => (),
        }
        Ok(None)
    }
}

#[cfg(test)]
#[path = "../../tests/legacy_sse_support/mod.rs"]
mod test_support;
#[cfg(test)]
mod tests {
    use super::test_support as support;
    use super::*;

    #[test]
    fn cancelling_a_running_call_closes_stream_and_releases_other_waiters() {
        let server = support::Server::new(support::Options {
            stall_call: true,
            ..Default::default()
        });
        let client = SseMcpClient::connect(HttpMcpConfig {
            url: server.url(),
            headers: vec![],
            tool_prefix: String::new(),
            timeout: Duration::from_secs(5),
        })
        .unwrap();
        let ctx = crate::ToolCtx::detached("cancel");
        let cancel = ctx.run_cancel.clone();
        let tools = client.tools();
        let first = tools[0].clone();
        let second = tools[1].clone();
        let one = std::thread::spawn(move || first.execute_result(json!({}), &ctx));
        let two = std::thread::spawn(move || {
            second.execute_result(json!({}), &crate::ToolCtx::detached("other"))
        });
        let deadline = Instant::now() + Duration::from_secs(2);
        while server
            .requests
            .lock()
            .unwrap()
            .iter()
            .filter(|(_, v)| v["method"] == "tools/call")
            .count()
            < 2
        {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(5));
        }
        cancel.store(true, Ordering::SeqCst);
        assert!(one.join().unwrap().is_error && two.join().unwrap().is_error);
        assert!(!client.is_connected());
        assert!(client.needs_review());
        while !server.stream_closed.load(Ordering::SeqCst) {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}
