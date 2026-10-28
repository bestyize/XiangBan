//! Authenticated loopback HTTP receiver.
use crate::{
    endpoint::{self, Endpoint},
    events::{EventId, HookDecision, HookEvent},
};
use axum::{
    body::{to_bytes, Body},
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::post,
    Router,
};
use rand::{rngs::OsRng, RngCore};
use serde_json::{json, Value};
use std::{
    collections::{HashMap, VecDeque},
    io,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};
use tokio::{
    net::TcpListener,
    sync::{broadcast, oneshot},
    task::JoinHandle,
};
const MAX_BODY: usize = 1024 * 1024;
/// Receiver timeouts and broadcast capacity.
#[derive(Clone, Debug)]
pub struct ReceiverConfig {
    /// Maximum permission decision wait, normally 120 seconds.
    pub permission_timeout: Duration,
    /// Maximum duration for reading a request body.
    pub read_timeout: Duration,
    /// Event broadcast buffer size.
    pub broadcast_capacity: usize,
}
impl Default for ReceiverConfig {
    fn default() -> Self {
        Self {
            permission_timeout: Duration::from_secs(120),
            read_timeout: Duration::from_secs(10),
            broadcast_capacity: 256,
        }
    }
}
struct Shared {
    endpoint: Endpoint,
    config: ReceiverConfig,
    next: AtomicU64,
    events: broadcast::Sender<HookEvent>,
    pending: Mutex<HashMap<EventId, oneshot::Sender<HookDecision>>>,
    instructions: Mutex<HashMap<String, VecDeque<String>>>,
}
/// Running loopback server. Call shutdown to remove its endpoint file.
pub struct HookServer {
    state: Arc<Shared>,
    task: JoinHandle<()>,
}
impl HookServer {
    /// Bind an ephemeral loopback port and publish a private endpoint file.
    pub async fn start(config: ReceiverConfig) -> io::Result<Self> {
        if config.permission_timeout >= Duration::from_secs(125)
            || config.broadcast_capacity == 0
            || config.read_timeout.is_zero()
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid timeout or broadcast capacity",
            ));
        }
        let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).await?;
        let mut random = [0u8; 32];
        OsRng.fill_bytes(&mut random);
        let token: String = random.iter().map(|b| format!("{b:02x}")).collect();
        let endpoint = Endpoint {
            port: listener.local_addr()?.port(),
            token,
            pid: std::process::id(),
        };
        endpoint::write(&endpoint)?;
        let (events, _) = broadcast::channel(config.broadcast_capacity);
        let state = Arc::new(Shared {
            endpoint,
            config,
            next: AtomicU64::new(1),
            events,
            pending: Mutex::new(HashMap::new()),
            instructions: Mutex::new(HashMap::new()),
        });
        let app = Router::new()
            .route("/h/{token}/{event}", post(handle))
            .with_state(state.clone());
        let header_timeout = state.config.read_timeout;
        let task = tokio::spawn(async move {
            let mut connections = tokio::task::JoinSet::new();
            loop {
                tokio::select! {
                    accepted = listener.accept() => {
                        let Ok((stream, _)) = accepted else { break; };
                        let service = hyper_util::service::TowerToHyperService::new(app.clone());
                        connections.spawn(async move {
                            let io = hyper_util::rt::TokioIo::new(stream);
                            let mut builder = hyper::server::conn::http1::Builder::new();
                            builder.timer(hyper_util::rt::TokioTimer::new()).header_read_timeout(header_timeout).keep_alive(false);
                            let _ = builder.serve_connection(io, service).await;
                        });
                    }
                    Some(_) = connections.join_next(), if !connections.is_empty() => {}
                }
            }
        });
        Ok(Self { state, task })
    }
    /// Return the current endpoint and secret token.
    pub fn endpoint(&self) -> Endpoint {
        self.state.endpoint.clone()
    }
    /// Subscribe to subsequent events.
    pub fn events(&self) -> broadcast::Receiver<HookEvent> {
        self.state.events.subscribe()
    }
    /// Resolve a pending permission event, if still pending.
    pub fn respond(&self, event_id: EventId, decision: HookDecision) {
        if let Some(tx) = self.state.pending.lock().unwrap().remove(&event_id) {
            let _ = tx.send(decision);
        }
    }
    /// Queue one instruction for the next eligible Stop of this session.
    pub fn queue_instruction(&self, session_id: &str, text: String) {
        self.state
            .instructions
            .lock()
            .unwrap()
            .entry(session_id.to_owned())
            .or_default()
            .push_back(text);
    }
    /// Stop the server and remove its endpoint if it is still ours.
    pub async fn shutdown(self) {
        self.task.abort();
        let _ = self.task.await;
        endpoint::remove_if_owned(&self.state.endpoint);
    }
}
async fn handle(
    State(s): State<Arc<Shared>>,
    Path((token, event)): Path<(String, String)>,
    headers: HeaderMap,
    body: Body,
) -> impl IntoResponse {
    if token != s.endpoint.token {
        return (StatusCode::NOT_FOUND, "{}").into_response();
    }
    let host = headers
        .get(axum::http::header::HOST)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    let valid_host = host == "localhost"
        || host == "127.0.0.1"
        || host == format!("localhost:{}", s.endpoint.port)
        || host == format!("127.0.0.1:{}", s.endpoint.port);
    if !valid_host {
        return (StatusCode::FORBIDDEN, "{}").into_response();
    }
    let bytes = match tokio::time::timeout(s.config.read_timeout, to_bytes(body, MAX_BODY)).await {
        Ok(Ok(b)) => b,
        Ok(Err(_)) => return (StatusCode::PAYLOAD_TOO_LARGE, "{}").into_response(),
        Err(_) => return (StatusCode::REQUEST_TIMEOUT, "{}").into_response(),
    };
    let mut data: HookEvent = match serde_json::from_slice(&bytes) {
        Ok(v) => v,
        Err(_) => return (StatusCode::BAD_REQUEST, "{}").into_response(),
    };
    if data.hook_event_name != event {
        return (StatusCode::BAD_REQUEST, "{}").into_response();
    }
    data.id = s.next.fetch_add(1, Ordering::Relaxed);
    let rx = if event == "PermissionRequest" {
        let (tx, rx) = oneshot::channel();
        s.pending.lock().unwrap().insert(data.id, tx);
        Some(rx)
    } else {
        None
    };
    let _ = s.events.send(data.clone());
    let response: Value = if let Some(rx) = rx {
        let decision = tokio::time::timeout(s.config.permission_timeout, rx)
            .await
            .ok()
            .and_then(Result::ok)
            .unwrap_or(HookDecision::NoOpinion);
        s.pending.lock().unwrap().remove(&data.id);
        match decision {
            HookDecision::Allow => {
                json!({"hookSpecificOutput":{"hookEventName":"PermissionRequest","decision":{"behavior":"allow"}}})
            }
            HookDecision::Deny { message } => {
                let mut d = json!({"behavior":"deny"});
                if let Some(m) = message {
                    d["message"] = json!(m);
                }
                json!({"hookSpecificOutput":{"hookEventName":"PermissionRequest","decision":d}})
            }
            HookDecision::NoOpinion => json!({}),
        }
    } else if event == "Stop" && !data.stop_hook_active && !data.session_id.is_empty() {
        let mut map = s.instructions.lock().unwrap();
        let item = map.get_mut(&data.session_id).and_then(VecDeque::pop_front);
        if map.get(&data.session_id).is_some_and(VecDeque::is_empty) {
            map.remove(&data.session_id);
        }
        item.map(|reason| json!({"decision":"block","reason":reason}))
            .unwrap_or_else(|| json!({}))
    } else {
        json!({})
    };
    (StatusCode::OK, axum::Json(response)).into_response()
}
