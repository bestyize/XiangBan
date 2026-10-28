use crate::{audio, mcp, openai, security, Backends, Capability, ClientInfo, Pcm, VoiceError};
use axum::{
    body::{to_bytes, Body},
    extract::{DefaultBodyLimit, Request, State},
    http::{header, Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use std::{
    io,
    net::SocketAddr,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{
    net::TcpListener,
    sync::{oneshot, OwnedSemaphorePermit, Semaphore},
    task::{JoinHandle, JoinSet},
};

/// Local HTTP listener and resource limits. Use a fresh cryptographically random token.
pub struct ServerConfig {
    /// Loopback bind address, default `127.0.0.1:0`.
    pub bind: SocketAddr,
    /// ASCII Bearer secret (at least 24 characters). Empty default must be replaced.
    pub token: String,
    /// Exact browser Origins allowed; empty by default.
    pub allowed_origins: Vec<String>,
    /// Maximum complete HTTP body, including multipart framing (default 25 MiB).
    pub max_body_bytes: usize,
}
impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind: SocketAddr::from(([127, 0, 0, 1], 0)),
            token: String::new(),
            allowed_origins: Vec::new(),
            max_body_bytes: 25 * 1024 * 1024,
        }
    }
}
/// Running server. Drop signals graceful shutdown; `shutdown` also awaits completion.
pub struct RunningServer {
    /// Actual bound address including the allocated port.
    pub addr: SocketAddr,
    shutdown: Option<oneshot::Sender<()>>,
    task: Option<JoinHandle<io::Result<()>>>,
}
impl RunningServer {
    /// Stop accepting requests, cancel background notifications, and drain active requests.
    pub async fn shutdown(mut self) -> io::Result<()> {
        if let Some(tx) = self.shutdown.take() {
            let _ = tx.send(());
        }
        if let Some(task) = self.task.take() {
            task.await.map_err(io::Error::other)??;
        }
        Ok(())
    }
}
impl Drop for RunningServer {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown.take() {
            let _ = tx.send(());
        }
    }
}

pub(crate) struct App {
    pub(crate) backends: Backends,
    pub(crate) consent: security::ConsentCache,
    pub(crate) transcription: Arc<Semaphore>,
    pub(crate) listening: Arc<Semaphore>,
    pub(crate) speaking: Arc<Semaphore>,
    pub(crate) background: std::sync::Mutex<JoinSet<()>>,
    digest: [u8; 32],
    origins: Vec<String>,
    max_body: usize,
}
pub(crate) const OPERATION_TIMEOUT: Duration = Duration::from_secs(120);

/// 整个请求的处理上限。转写最长要处理 10 分钟的音频（分段识别、每段另有 [`OPERATION_TIMEOUT`]），
/// 所以单独放宽；其余接口都是短操作。
pub(crate) fn request_timeout(path: &str) -> Duration {
    if path == "/v1/audio/transcriptions" {
        Duration::from_secs(660)
    } else {
        Duration::from_secs(180)
    }
}

/// Start a loopback-only server. Rejects non-loopback binds and short/invalid tokens.
/// Library operations use only memory, the listening socket, and injected backends.
pub async fn start(config: ServerConfig, backends: Backends) -> io::Result<RunningServer> {
    if !config.bind.ip().is_loopback()
        || config.token.len() < 24
        || config.token.len() > 4096
        || !config.token.bytes().all(|b| (33..=126).contains(&b))
        || config.max_body_bytes == 0
    {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "loopback bind, 24..4096 printable ASCII token characters and positive body limit required"));
    }
    let app = Arc::new(App {
        consent: security::ConsentCache::new(backends.consent.clone()),
        backends,
        transcription: Arc::new(Semaphore::new(2)),
        listening: Arc::new(Semaphore::new(1)),
        speaking: Arc::new(Semaphore::new(1)),
        background: std::sync::Mutex::new(JoinSet::new()),
        digest: security::token_digest(&config.token),
        origins: config.allowed_origins,
        max_body: config.max_body_bytes,
    });
    let routes = Router::new()
        .route("/health", get(|| async { Json(json!({"ok": true})) }))
        .route("/v1/audio/transcriptions", post(openai::transcriptions))
        .route("/v1/audio/speech", post(openai::speech))
        .route("/v1/audio/voices", get(openai::voices))
        .route(
            "/mcp",
            post(mcp::handle)
                .get(|| async { (StatusCode::METHOD_NOT_ALLOWED, [(header::ALLOW, "POST")]) }),
        )
        .fallback(|| async {
            ApiError::new(StatusCode::NOT_FOUND, "not_found", "unknown endpoint")
        })
        .method_not_allowed_fallback(|| async {
            ApiError::new(
                StatusCode::METHOD_NOT_ALLOWED,
                "method_not_allowed",
                "method not allowed",
            )
        })
        .layer(DefaultBodyLimit::disable())
        .layer(middleware::from_fn_with_state(app.clone(), guard))
        .with_state(app.clone());
    let listener = TcpListener::bind(config.bind).await?;
    let addr = listener.local_addr()?;
    let (tx, rx) = oneshot::channel();
    let task = tokio::spawn(async move {
        let closing = app.clone();
        let result = axum::serve(listener, routes)
            .with_graceful_shutdown(async move {
                let _ = rx.await;
                closing.background.lock().unwrap().abort_all();
            })
            .await;
        let mut background = std::mem::take(&mut *app.background.lock().unwrap());
        background.abort_all();
        while background.join_next().await.is_some() {}
        result
    });
    Ok(RunningServer {
        addr,
        shutdown: Some(tx),
        task: Some(task),
    })
}

async fn guard(State(app): State<Arc<App>>, request: Request, next: Next) -> Response {
    let began = Instant::now();
    // Never log a raw path/query/header: a malicious client can put secrets in them.
    let route = match request.uri().path() {
        "/health" => "/health",
        "/v1/audio/transcriptions" => "/v1/audio/transcriptions",
        "/v1/audio/speech" => "/v1/audio/speech",
        "/v1/audio/voices" => "/v1/audio/voices",
        "/mcp" => "/mcp",
        _ => "other",
    };
    let (response, size) = guarded(app, request, next).await;
    tracing::info!(
        path = route,
        status = response.status().as_u16(),
        elapsed_ms = began.elapsed().as_millis() as u64,
        size,
        "voice request"
    );
    response
}
async fn guarded(app: Arc<App>, request: Request, next: Next) -> (Response, usize) {
    let headers = request.headers();
    if headers.get_all(header::HOST).iter().count() != 1
        || !headers
            .get(header::HOST)
            .and_then(|v| v.to_str().ok())
            .is_some_and(security::valid_host)
    {
        return (
            ApiError::new(
                StatusCode::FORBIDDEN,
                "forbidden_host",
                "Host is not allowed",
            )
            .into_response(),
            0,
        );
    }
    let origin = headers.get(header::ORIGIN).cloned();
    if headers.get_all(header::ORIGIN).iter().count() > 1
        || origin
            .as_ref()
            .is_some_and(|v| !v.to_str().is_ok_and(|s| app.origins.iter().any(|o| o == s)))
    {
        return (
            ApiError::new(
                StatusCode::FORBIDDEN,
                "forbidden_origin",
                "Origin is not allowed",
            )
            .into_response(),
            0,
        );
    }
    if request.method() == Method::OPTIONS {
        let Some(origin) = origin else {
            return (
                ApiError::new(
                    StatusCode::FORBIDDEN,
                    "forbidden_origin",
                    "preflight requires an allowed Origin",
                )
                .into_response(),
                0,
            );
        };
        let mut response = StatusCode::NO_CONTENT.into_response();
        response
            .headers_mut()
            .insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin);
        response.headers_mut().insert(
            header::ACCESS_CONTROL_ALLOW_METHODS,
            "GET, POST, OPTIONS".parse().unwrap(),
        );
        response.headers_mut().insert(header::ACCESS_CONTROL_ALLOW_HEADERS, "Authorization, Content-Type, X-Client-Name, MCP-Protocol-Version, Mcp-Method, Mcp-Name".parse().unwrap());
        response
            .headers_mut()
            .insert(header::VARY, "Origin".parse().unwrap());
        return (response, 0);
    }
    if request.uri().path() != "/health" && !security::authenticated(headers, &app.digest) {
        return (
            ApiError::new(StatusCode::UNAUTHORIZED, "unauthorized", "unauthorized").into_response(),
            0,
        );
    }
    let (parts, body) = request.into_parts();
    let parts_path = parts.uri.path().to_owned();
    let bytes =
        match tokio::time::timeout(Duration::from_secs(30), to_bytes(body, app.max_body)).await {
            Ok(Ok(bytes)) => bytes,
            Ok(Err(_)) => {
                return (
                    ApiError::new(
                        StatusCode::PAYLOAD_TOO_LARGE,
                        "body_too_large",
                        "request body exceeds limit or is unreadable",
                    )
                    .into_response(),
                    0,
                )
            }
            Err(_) => {
                return (
                    ApiError::new(
                        StatusCode::REQUEST_TIMEOUT,
                        "request_timeout",
                        "request body timed out",
                    )
                    .into_response(),
                    0,
                )
            }
        };
    let size = bytes.len();
    let request = Request::from_parts(parts, Body::from(bytes));
    let limit = request_timeout(parts_path.as_str());
    let mut response = match tokio::time::timeout(limit, next.run(request)).await {
        Ok(response) => response,
        Err(_) => ApiError::new(StatusCode::GATEWAY_TIMEOUT, "timeout", "request timed out")
            .into_response(),
    };
    if let Some(origin) = origin {
        response
            .headers_mut()
            .insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin);
        response
            .headers_mut()
            .insert(header::VARY, "Origin".parse().unwrap());
    }
    (response, size)
}

pub(crate) struct ApiError {
    pub(crate) status: StatusCode,
    pub(crate) code: &'static str,
    message: String,
}
impl ApiError {
    pub(crate) fn new(status: StatusCode, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
        }
    }
    pub(crate) fn bad(code: &'static str, message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, code, message)
    }
}
impl From<VoiceError> for ApiError {
    fn from(error: VoiceError) -> Self {
        let (status, code) = match &error {
            VoiceError::Busy => (StatusCode::TOO_MANY_REQUESTS, "busy"),
            VoiceError::Unavailable(_) => (StatusCode::SERVICE_UNAVAILABLE, "unavailable"),
            VoiceError::Unsupported(_) => (
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "unsupported_audio_format",
            ),
            VoiceError::TooLong => (StatusCode::PAYLOAD_TOO_LARGE, "audio_too_long"),
            VoiceError::InvalidInput(_) => (StatusCode::BAD_REQUEST, "invalid_input"),
            VoiceError::Cancelled => (StatusCode::REQUEST_TIMEOUT, "cancelled"),
            VoiceError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
        };
        Self::new(status, code, error.to_string())
    }
}
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let mut response = (self.status, Json(json!({"error": {"message": self.message, "type": "invalid_request_error", "code": self.code}}))).into_response();
        if self.status == StatusCode::TOO_MANY_REQUESTS {
            response
                .headers_mut()
                .insert(header::RETRY_AFTER, "2".parse().unwrap());
        }
        response
    }
}
pub(crate) fn permit(semaphore: &Arc<Semaphore>) -> Result<OwnedSemaphorePermit, ApiError> {
    semaphore
        .clone()
        .try_acquire_owned()
        .map_err(|_| VoiceError::Busy.into())
}
impl App {
    pub(crate) async fn approve(
        &self,
        client: &ClientInfo,
        capabilities: &[Capability],
    ) -> Result<(), ApiError> {
        for capability in capabilities {
            if !self.consent.allow(client, *capability).await {
                return Err(ApiError::new(
                    StatusCode::FORBIDDEN,
                    "consent_denied",
                    "user consent denied or timed out",
                ));
            }
        }
        Ok(())
    }
    pub(crate) async fn transcribe(
        &self,
        pcm: Pcm,
        language: Option<&str>,
    ) -> Result<String, ApiError> {
        audio::validate_pcm(&pcm)?;
        let ranges = audio::segments(&pcm.samples, self.backends.transcriber.max_samples())?;
        let mut texts = Vec::new();
        for range in ranges {
            let segment = Pcm {
                samples: pcm.samples[range].to_vec(),
            };
            let text = tokio::time::timeout(
                OPERATION_TIMEOUT,
                self.backends.transcriber.transcribe(&segment, language),
            )
            .await
            .map_err(|_| {
                ApiError::new(
                    StatusCode::GATEWAY_TIMEOUT,
                    "timeout",
                    "recognition timed out",
                )
            })??;
            if !text.trim().is_empty() {
                texts.push(text.trim().to_owned());
            }
        }
        Ok(texts.join(
            if language.is_some_and(|s| s.to_ascii_lowercase().starts_with("zh")) {
                ""
            } else {
                " "
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_transcription_gets_the_long_deadline() {
        assert_eq!(
            request_timeout("/v1/audio/transcriptions"),
            Duration::from_secs(660)
        );
        for path in [
            "/v1/audio/speech",
            "/v1/audio/voices",
            "/mcp",
            "/health",
            "/other",
        ] {
            assert_eq!(request_timeout(path), Duration::from_secs(180), "{path}");
        }
    }
}
