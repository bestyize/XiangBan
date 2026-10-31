mod scan;
mod sign;
mod tool_mirrors;

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderValue, Method, Request, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, put},
    Json, Router,
};
use futures_util::StreamExt;
use ring::digest;
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    net::SocketAddr,
    path::{Path as FsPath, PathBuf},
    sync::Arc,
    time::Duration,
};
use tokio::{fs, io::AsyncWriteExt, sync::Semaphore};
use tower::ServiceExt;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeFile,
    trace::TraceLayer,
};

/// 不用签名就能读的目录：封面图是公开的。
const PUBLIC_PREFIX: &str = "covers/";

#[derive(Clone)]
pub struct AppState {
    pub data_dir: Arc<PathBuf>,
    pub secret: Arc<Vec<u8>>,
    pub callback_url: Arc<String>,
    pub http: reqwest::Client,
    pub uploads: Arc<Semaphore>,
}

type ApiResult<T> = Result<T, (StatusCode, Json<Value>)>;

fn fail<T>(status: StatusCode, message: impl Into<String>) -> ApiResult<T> {
    Err((status, Json(json!({ "error": true, "message": message.into() }))))
}

/// 与论坛一致的对象键规则：相对路径里只含安全字符，防止目录穿越。
fn valid_file_key(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 200
        && !value.starts_with('/')
        && !value.contains("..")
        && value.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'/' | b'-' | b'_' | b'.'))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn now() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

pub fn build_router(state: AppState, cors_origins: &[String]) -> Router {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::HEAD, Method::PUT, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::RANGE, header::HeaderName::from_static("x-upload-ticket")]);
    let origins: Vec<HeaderValue> = cors_origins.iter().filter_map(|o| o.parse().ok()).collect();
    let cors = if origins.is_empty() { cors.allow_origin(Any) } else { cors.allow_origin(origins) };
    Router::new()
        .route("/health", get(|| async { Json(json!({ "ok": true })) }))
        .route("/upload", put(upload))
        .merge(tool_mirrors::routes())
        .route("/{*key}", get(download))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

// ───────────────────────── 上传 ─────────────────────────

async fn upload(State(state): State<AppState>, headers: HeaderMap, body: Body) -> ApiResult<Json<Value>> {
    let ticket = headers.get("x-upload-ticket").and_then(|v| v.to_str().ok()).unwrap_or("");
    let claims = sign::verify_upload_ticket(ticket, now(), &state.secret).map_err(|m| fail::<()>(StatusCode::FORBIDDEN, m).unwrap_err())?;
    if !valid_file_key(&claims.file_key) || claims.size == 0 {
        return fail(StatusCode::BAD_REQUEST, "上传凭证内容无效");
    }
    let declared = headers.get(header::CONTENT_LENGTH).and_then(|v| v.to_str().ok()).and_then(|v| v.parse::<u64>().ok());
    if declared.is_some_and(|len| len != claims.size) {
        return fail(StatusCode::BAD_REQUEST, "文件大小与凭证不一致");
    }
    let target = state.data_dir.join(&claims.file_key);
    if fs::try_exists(&target).await.unwrap_or(false) {
        return fail(StatusCode::CONFLICT, "这个文件已经上传过");
    }
    let Ok(_permit) = state.uploads.clone().try_acquire_owned() else {
        return fail(StatusCode::SERVICE_UNAVAILABLE, "当前上传的人太多，请稍后再试");
    };

    let tmp_dir = state.data_dir.join("tmp");
    fs::create_dir_all(&tmp_dir).await.map_err(internal)?;
    let tmp = tmp_dir.join(uuid::Uuid::new_v4().simple().to_string());
    let outcome = receive_and_check(&state, &claims, body, &tmp, &target).await;
    if outcome.is_err() {
        let _ = fs::remove_file(&tmp).await;
        let _ = fs::remove_file(&target).await;
    }
    outcome.map(|()| Json(json!({ "ok": true, "file_key": claims.file_key })))
}

fn internal(err: impl std::fmt::Display) -> (StatusCode, Json<Value>) {
    tracing::error!("internal error: {err}");
    (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": true, "message": "服务暂时不可用" })))
}

async fn receive_and_check(state: &AppState, claims: &sign::UploadClaims, body: Body, tmp: &FsPath, target: &FsPath) -> ApiResult<()> {
    // 边收边写边算哈希，超过凭证声明的大小立即中止，不把整个文件放进内存。
    let mut file = fs::File::create(tmp).await.map_err(internal)?;
    let mut ctx = digest::Context::new(&digest::SHA256);
    let mut received: u64 = 0;
    let mut stream = body.into_data_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({ "error": true, "message": "上传中断" }))))?;
        received += chunk.len() as u64;
        if received > claims.size {
            return fail(StatusCode::PAYLOAD_TOO_LARGE, "文件比凭证声明的大");
        }
        ctx.update(&chunk);
        file.write_all(&chunk).await.map_err(internal)?;
    }
    file.flush().await.map_err(internal)?;
    drop(file);
    if received != claims.size {
        return fail(StatusCode::BAD_REQUEST, "文件大小与凭证不一致");
    }
    if hex(ctx.finish().as_ref()) != claims.sha256 {
        return fail(StatusCode::UNPROCESSABLE_ENTITY, "文件哈希与凭证不一致");
    }

    let manifest = if claims.kind == "cover" {
        let mut head = [0u8; 16];
        let mut f = fs::File::open(tmp).await.map_err(internal)?;
        let n = tokio::io::AsyncReadExt::read(&mut f, &mut head).await.map_err(internal)?;
        if scan::sniff_image(&head[..n]).is_none() {
            return fail(StatusCode::UNSUPPORTED_MEDIA_TYPE, "封面只支持 PNG、JPEG、WebP");
        }
        None
    } else {
        let path = tmp.to_path_buf();
        let scanned = tokio::task::spawn_blocking(move || scan::scan_pack(&path)).await.map_err(internal)?;
        match scanned {
            Ok(manifest) => Some(manifest),
            Err(message) => return fail(StatusCode::UNPROCESSABLE_ENTITY, message),
        }
    };

    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).await.map_err(internal)?;
    }
    fs::rename(tmp, target).await.map_err(internal)?;

    // 回调论坛，论坛确认后这个上传才算有效；回调失败就撤销文件，让客户端重传。
    let payload = json!({ "file_key": claims.file_key, "size": claims.size, "sha256": claims.sha256, "manifest": manifest });
    let raw = serde_json::to_vec(&payload).map_err(internal)?;
    let response = state
        .http
        .post(state.callback_url.as_str())
        .header(header::CONTENT_TYPE, "application/json")
        .header("X-Signature", sign::sign_callback(&raw, &state.secret))
        .body(raw)
        .send()
        .await;
    match response {
        Ok(r) if r.status().is_success() => Ok(()),
        Ok(r) if r.status().is_client_error() => {
            let message = r.json::<Value>().await.ok().and_then(|v| v.get("message").and_then(Value::as_str).map(str::to_string));
            fail(StatusCode::UNPROCESSABLE_ENTITY, message.unwrap_or_else(|| "论坛没有接受这个文件".into()))
        }
        Ok(r) => {
            tracing::warn!("callback failed: {}", r.status());
            fail(StatusCode::BAD_GATEWAY, "通知论坛失败，请稍后重试")
        }
        Err(err) => {
            tracing::warn!("callback error: {err}");
            fail(StatusCode::BAD_GATEWAY, "通知论坛失败，请稍后重试")
        }
    }
}

// ───────────────────────── 下载 ─────────────────────────

#[derive(Deserialize)]
struct SignedQuery {
    exp: Option<i64>,
    sig: Option<String>,
}

async fn download(State(state): State<AppState>, Path(key): Path<String>, Query(q): Query<SignedQuery>, request: Request<Body>) -> Response {
    if !valid_file_key(&key) || key.starts_with("tmp/") {
        return StatusCode::NOT_FOUND.into_response();
    }
    let public = key.starts_with(PUBLIC_PREFIX);
    if !public {
        let ok = matches!((q.exp, q.sig.as_deref()), (Some(exp), Some(sig)) if sign::verify_download(&key, exp, sig, now(), &state.secret));
        if !ok {
            return (StatusCode::FORBIDDEN, "下载地址无效或已过期").into_response();
        }
    }
    let path = state.data_dir.join(&key);
    if public {
        return serve_cover(&path).await;
    }
    // ServeFile 负责 Range 与条件请求，客户端可以断点续传。
    match ServeFile::new(&path).oneshot(request).await {
        Ok(response) => {
            let mut response = response.map(Body::new);
            let h = response.headers_mut();
            h.insert(header::CONTENT_TYPE, HeaderValue::from_static("application/zip"));
            h.insert(header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
            h.insert(header::CONTENT_DISPOSITION, HeaderValue::from_static("attachment"));
            h.insert(header::CACHE_CONTROL, HeaderValue::from_static("private, max-age=0"));
            response
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn serve_cover(path: &FsPath) -> Response {
    let Ok(bytes) = fs::read(path).await else { return StatusCode::NOT_FOUND.into_response() };
    let Some(content_type) = scan::sniff_image(&bytes) else { return StatusCode::NOT_FOUND.into_response() };
    (
        [
            (header::CONTENT_TYPE, content_type),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
            (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
        ],
        bytes,
    )
        .into_response()
}

// ───────────────────────── 启动 ─────────────────────────

fn env(key: &str) -> Option<String> {
    std::env::var(key).ok().map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = dotenvy::from_filename("services/server-resource/.env");
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "server_resource=info,tower_http=info".into()))
        .init();

    let secret = env("RESOURCE_SECRET").filter(|s| s.len() >= 32).ok_or("RESOURCE_SECRET 必须设置，且不少于 32 位，并与论坛的 FORUM_RESOURCE_SECRET 一致")?;
    let data_dir = PathBuf::from(env("RESOURCE_DATA_DIR").unwrap_or_else(|| "services/server-resource/data".into()));
    fs::create_dir_all(&data_dir).await?;
    let max_uploads = env("RESOURCE_MAX_UPLOADS").and_then(|v| v.parse().ok()).unwrap_or(8).max(1);
    let cors: Vec<String> = env("RESOURCE_CORS_ORIGINS")
        .map(|raw| raw.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
        .unwrap_or_default();
    let state = AppState {
        data_dir: Arc::new(data_dir),
        secret: Arc::new(secret.into_bytes()),
        callback_url: Arc::new(env("RESOURCE_CALLBACK_URL").unwrap_or_else(|| "https://chat.xiangban.xyz/api/internal/pack-uploaded".into())),
        http: reqwest::Client::builder().timeout(Duration::from_secs(15)).build()?,
        uploads: Arc::new(Semaphore::new(max_uploads)),
    };
    // 启动时清掉上次中断留下的临时文件。
    let _ = fs::remove_dir_all(state.data_dir.join("tmp")).await;

    let addr: SocketAddr = format!("{}:{}", env("RESOURCE_HOST").unwrap_or_else(|| "127.0.0.1".into()), env("RESOURCE_PORT").unwrap_or_else(|| "55020".into())).parse()?;
    tracing::info!("server-resource listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, build_router(state, &cors)).await?;
    Ok(())
}

#[cfg(test)]
mod tests;
