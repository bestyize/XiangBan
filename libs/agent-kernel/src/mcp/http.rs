//! Streamable HTTP transport (JSON and response SSE). Legacy HTTP+SSE is explicit unsupported.
use super::{decode_result, MAX_FRAME_BYTES};
use crate::{Risk, Tool, ToolCtx, ToolResult, ToolSpec};
use reqwest::{
    Client, Response,
    header::{HeaderMap, HeaderName, HeaderValue},
    redirect::Policy,
};
use serde_json::{json, Value};
use std::{
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

pub struct HttpMcpConfig {
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub tool_prefix: String,
    pub timeout: Duration,
}
/// Host-owned credentials. Tokens never enter tool schemas or the model context.
pub trait HttpAuthorization: Send + Sync {
    fn access_token(&self, refresh: bool) -> Result<String, String>;
    fn cached_token(&self) -> Option<String> {
        None
    }
}
pub struct HttpMcpClient {
    client: Client,
    url: String,
    headers: HeaderMap,
    authorization: Option<Arc<dyn HttpAuthorization>>,
    session: Mutex<Option<String>>,
    protocol: Mutex<String>,
    next: AtomicU64,
    closed: AtomicBool,
    specs: Vec<(String, ToolSpec)>,
    startup_cancel:Option<Arc<AtomicBool>>,
    startup_deadline:Option<std::time::Instant>,
    // Closing a socket does not prove that a remote write stopped. Keep its run's
    // resource ownership until the user explicitly closes/reconnects this service.
    uncertain: Mutex<Option<crate::RunControl>>,
    revoked: AtomicBool,
}
impl HttpMcpClient {
    pub fn connect(config: HttpMcpConfig) -> Result<Arc<Self>, String> {
        Self::connect_inner(config, None, None)
    }
    pub fn connect_with_authorization(
        config: HttpMcpConfig,
        authorization: Arc<dyn HttpAuthorization>,
    ) -> Result<Arc<Self>, String> {
        Self::connect_inner(config, Some(authorization), None)
    }
    pub fn connect_cancellable(config:HttpMcpConfig,auth:Option<Arc<dyn HttpAuthorization>>,cancel:Arc<AtomicBool>)->Result<Arc<Self>,String>{Self::connect_inner(config,auth,Some(cancel))}
    fn connect_inner(
        config: HttpMcpConfig,
        authorization: Option<Arc<dyn HttpAuthorization>>,
        startup_cancel:Option<Arc<AtomicBool>>,
    ) -> Result<Arc<Self>, String> {
        let url = reqwest::Url::parse(&config.url).map_err(|_| "MCP 地址无效")?;
        if !(url.scheme() == "https"
            || (url.scheme() == "http"
                && matches!(
                    url.host_str(),
                    Some("localhost" | "127.0.0.1" | "[::1]" | "::1")
                )))
            || !url.username().is_empty()
            || url.password().is_some()
            || url.fragment().is_some()
        {
            return Err("远程 MCP 必须使用 HTTPS；本机服务可使用 HTTP".into());
        }
        let mut headers = HeaderMap::new();
        for (key, value) in config.headers {
            let key = HeaderName::from_bytes(key.as_bytes()).map_err(|_| "MCP 请求头名称无效")?;
            if matches!(
                key.as_str(),
                "host"
                    | "content-length"
                    | "connection"
                    | "transfer-encoding"
                    | "mcp-session-id"
                    | "mcp-protocol-version"
                    | "accept"
                    | "content-type"
            ) {
                return Err("不可覆盖 MCP 协议请求头".into());
            }
            headers.insert(
                key,
                HeaderValue::from_str(&value).map_err(|_| "MCP 请求头值无效")?,
            );
        }
        let client = crate::net::async_builder(Client::builder()
            .redirect(Policy::none())
            .connect_timeout(Duration::from_secs(10))
            .timeout(config.timeout))?
            .build()
            .map_err(|_| "无法创建 MCP HTTP 客户端")?;
        if authorization.is_some() {
            headers.remove("authorization");
        }
        let mut this = Self {
            client,
            url: config.url,
            headers,
            authorization,
            session: Mutex::new(None),
            protocol: Mutex::new("2025-06-18".into()),
            next: AtomicU64::new(1),
            closed: AtomicBool::new(false),
            specs: Vec::new(),
            startup_cancel,
            startup_deadline:Some(std::time::Instant::now()+Duration::from_secs(45)),
            uncertain: Mutex::new(None),
            revoked: AtomicBool::new(false),
        };
        let result = this.request(
            "initialize",
            json!({"protocolVersion":"2025-06-18","capabilities":{},
            "clientInfo":{"name":"xiaoban","version":env!("CARGO_PKG_VERSION")}}),
            false, None,
        )?;
        let version = result["protocolVersion"]
            .as_str()
            .ok_or("MCP 初始化缺少协议版本")?;
        if !matches!(version, "2025-06-18" | "2025-03-26") {
            return Err("此 MCP 协议版本暂不支持，请选择 Streamable HTTP 或 stdio 配置".into());
        }
        *this.protocol.lock().unwrap_or_else(|e| e.into_inner()) = version.into();
        this.request("notifications/initialized", json!({}), true, None)?;
        let mut cursor = None;
        let mut seen = std::collections::HashSet::new();
        for _ in 0..20 {
            let params = cursor
                .as_ref()
                .map(|v| json!({"cursor":v}))
                .unwrap_or(json!({}));
            let result = this.request("tools/list", params, false, None)?;
            for tool in result["tools"]
                .as_array()
                .ok_or("MCP tools/list 格式无效")?
            {
                let name = tool["name"].as_str().ok_or("MCP 工具缺少名称")?;
                if !seen.insert(name.to_string()) || this.specs.len() >= 500 {
                    return Err("MCP 工具名称重复或数量超限".into());
                }
                let spec = ToolSpec::new(
                    format!("{}{name}", config.tool_prefix),
                    tool["description"].as_str().unwrap_or(""),
                    tool.get("inputSchema")
                        .cloned()
                        .unwrap_or(json!({"type":"object"})),
                )
                .risk(Risk::Exec)
                .untrusted_output()
                .always_ask()
                .timeout(config.timeout);
                this.specs.push((name.into(), spec));
            }
            cursor = result["nextCursor"].as_str().map(str::to_string);
            if cursor.is_none() {
                this.startup_cancel=None;
                this.startup_deadline=None;
                return Ok(Arc::new(this));
            }
        }
        Err("MCP 工具分页超限".into())
    }
    fn request(&self, method: &str, params: Value, notification: bool, ctx:Option<&ToolCtx>) -> Result<Value, String> {
        if self.closed.load(Ordering::SeqCst) {
            return Err("MCP 已断开".into());
        }
        let id = self.next.fetch_add(1, Ordering::Relaxed);
        let mut request = json!({"jsonrpc":"2.0","method":method,"params":params});
        if !notification {
            request["id"] = json!(id);
        }
        let body = serde_json::to_vec(&request).map_err(|_| "MCP 请求序列化失败")?;
        let runtime=tokio::runtime::Builder::new_current_thread().enable_all().build().map_err(|e|e.to_string())?;
        let cancelled=||self.startup_deadline.is_some_and(|d|std::time::Instant::now()>=d)||self.closed.load(Ordering::SeqCst)||self.startup_cancel.as_ref().is_some_and(|c|c.load(Ordering::SeqCst))||ctx.is_some_and(ToolCtx::is_cancelled);
        // Authorization may use a blocking HTTP client. Resolve/refresh credentials
        // outside the async runtime; reqwest's blocking runtime cannot be dropped there.
        for attempt in 0..2 {
            let mut builder = self
                .client
                .post(&self.url)
                .headers(self.headers.clone())
                .header("Accept", "application/json, text/event-stream")
                .header("Content-Type", "application/json");
            if let Some(auth) = &self.authorization {
                let token = auth.access_token(attempt != 0).map_err(|e| {
                    self.closed.store(true, Ordering::SeqCst);
                    e
                })?;
                builder = builder.bearer_auth(token);
            }
            if method != "initialize" {
                builder = builder.header(
                    "MCP-Protocol-Version",
                    self.protocol
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .as_str(),
                );
            }
            if let Some(session) = self
                .session
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .as_ref()
            {
                builder = builder.header("Mcp-Session-Id", session);
            }
            if matches!(method, "initialize" | "tools/list") {
                builder = builder.timeout(Duration::from_secs(15));
            }
            let response = runtime.block_on(async {
                crate::net::wait(builder.body(body.clone()).send(), &cancelled, Duration::from_secs(45)).await
            }).map_err(|e|e.to_string())?.map_err(|_| {
                self.closed.store(true, Ordering::SeqCst);
                "MCP 网络请求失败或超时，请检查地址和凭据"
            })?;
            if response.status().as_u16() == 401 && self.authorization.is_some() && attempt == 0 {
                continue;
            }
            if !response.status().is_success() {
                self.closed.store(true, Ordering::SeqCst);
                return Err(format!(
                    "MCP HTTP {}；请检查鉴权与传输方式，旧式 SSE 暂不支持",
                    response.status().as_u16()
                ));
            }
            if method == "initialize" {
                if let Some(session) = response.headers().get("Mcp-Session-Id") {
                    *self.session.lock().unwrap_or_else(|e| e.into_inner()) =
                        Some(session.to_str().map_err(|_| "MCP 会话头无效")?.to_string());
                }
            }
            if notification {
                return Ok(Value::Null);
            }
            return runtime.block_on(read_response(response, id, &cancelled));
        }
        Err("MCP 授权失效，请重新连接并授权".into())
    }

    pub fn is_connected(&self) -> bool {
        !self.closed.load(Ordering::SeqCst)
    }
    pub fn needs_review(&self) -> bool {
        self.uncertain.lock().unwrap_or_else(|e| e.into_inner()).is_some()
    }
    pub fn close(&self) {
        self.revoked.store(true, Ordering::SeqCst);
        self.closed.store(true,Ordering::SeqCst);
        self.uncertain.lock().unwrap_or_else(|e| e.into_inner()).take();
        let session = self
            .session
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .take();
        if let Some(session) = session {
            let mut request = self
                .client
                .delete(&self.url)
                .headers(self.headers.clone())
                .header("Mcp-Session-Id", session)
                .header(
                    "MCP-Protocol-Version",
                    self.protocol
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .as_str(),
                )
                .timeout(Duration::from_secs(2));
            if let Some(auth) = &self.authorization {
                let Some(token) = auth.cached_token() else {
                    return;
                };
                request = request.bearer_auth(token);
            }
            let _=std::thread::Builder::new().name("mcp-session-close".into()).spawn(move||{if let Ok(rt)=tokio::runtime::Builder::new_current_thread().enable_all().build(){let _=rt.block_on(request.send());}});
        }
    }
    pub fn tools(self: &Arc<Self>) -> Vec<Arc<dyn Tool>> {
        self.specs
            .iter()
            .map(|(name, spec)| {
                Arc::new(HttpTool {
                    client: self.clone(),
                    name: name.clone(),
                    spec: spec.clone(),
                }) as Arc<dyn Tool>
            })
            .collect()
    }
}
impl Drop for HttpMcpClient{fn drop(&mut self){self.close();}}

fn rpc_result(value: Value, id: u64) -> Result<Value, String> {
    if value["id"].as_u64() != Some(id) {
        return Err("MCP 响应请求编号不匹配".into());
    }
    if value.get("error").is_some() {
        return Err("MCP 服务返回执行错误".into());
    }
    value
        .get("result")
        .cloned()
        .ok_or("MCP 响应缺少结果".into())
}
async fn read_response(mut response:Response,id:u64,cancel:impl Fn()->bool)->Result<Value,String>{
 let sse=response.headers().get("content-type").and_then(|v|v.to_str().ok()).is_some_and(|v|v.starts_with("text/event-stream"));
 let mut bytes=Vec::new();let mut total=0u64;let mut event=String::new();
 loop{
  let chunk=crate::net::wait(response.chunk(),&cancel,Duration::from_secs(30)).await.map_err(|e|e.to_string())?.map_err(|_|"MCP response read failed")?;
  let Some(chunk)=chunk else{break;};total+=chunk.len() as u64;if total>MAX_FRAME_BYTES{return Err("MCP response exceeds 8 MiB".into());}bytes.extend_from_slice(&chunk);
  if sse{while let Some(end)=bytes.iter().position(|b|*b==b'\n'){
   let line:Vec<_>=bytes.drain(..=end).collect();let line=std::str::from_utf8(&line).map_err(|_|"invalid MCP UTF-8")?;
   if line.trim().is_empty(){if !event.is_empty(){let value:Value=serde_json::from_str(&event).map_err(|_|"invalid MCP SSE JSON")?;event.clear();if value["id"].as_u64()==Some(id)&&value.get("method").is_none(){return rpc_result(value,id);}}}
   else if let Some(data)=line.strip_prefix("data:"){event.push_str(data.strip_prefix(' ').unwrap_or(data).trim_end_matches(['\r','\n']));event.push('\n');}
  }}
 }
 if sse{return Err("MCP SSE closed before complete result".into());}
 rpc_result(serde_json::from_slice(&bytes).map_err(|_|"invalid MCP JSON")?,id)
}

struct HttpTool {
    client: Arc<HttpMcpClient>,
    name: String,
    spec: ToolSpec,
}
impl Tool for HttpTool {
    fn spec(&self) -> &ToolSpec {
        &self.spec
    }
    fn execute(&self, args: Value, ctx: &ToolCtx) -> Result<String, String> {
        let result = self.execute_result(args, ctx);
        if result.is_error {
            Err(result.output)
        } else {
            Ok(result.output)
        }
    }
    fn execute_result(&self, args: Value, ctx: &ToolCtx) -> ToolResult {
        if ctx.is_cancelled() || !self.client.is_connected() {
            return ToolResult::error("cancelled");
        }
        self.client
            .request(
                "tools/call",
                json!({"name":self.name,"arguments":args}),
                false, Some(ctx),
            )
            .and_then(|v| decode_result(&v))
            .unwrap_or_else(|error| {
                let mut uncertain = self.client.uncertain.lock().unwrap_or_else(|e| e.into_inner());
                if !self.client.revoked.load(Ordering::SeqCst) {
                    *uncertain = Some(ctx.control().clone());
                }
                ToolResult::unknown(&format!("{error}; inspect remote state, then disconnect/reconnect this MCP service before resuming"))
            })
    }
}
