//! Credential-free official installer mirror storage. Only a forum-signed fixed GitHub release
//! asset can be pulled. Byte, object, time and cross-process filesystem lock quotas are fixed.
use crate::{ApiResult, AppState};
use axum::{
    body::{Body, Bytes},
    extract::{DefaultBodyLimit, Path, Query, State},
    http::{header, HeaderMap, HeaderValue, Request, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use ring::{digest, hmac};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    path::{Path as FsPath, PathBuf},
    time::Duration,
};
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
};
use tower::ServiceExt;
use tower_http::services::ServeFile;
const MAX_ASSET: u64 = 128 * 1024 * 1024;
const MAX_STORAGE: u64 = 1024 * 1024 * 1024;
const MAX_OBJECTS: usize = 128;
const TIMEOUT: Duration = Duration::from_secs(180);
const REQUEST_DOMAIN: &[u8] = b"POST\n/internal/tool-mirrors\n";
const RECEIPT_DOMAIN: &[u8] = b"TOOL-MIRROR-RECEIPT\n";
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
struct Claims {
    nonce: String,
    repository: String,
    version: String,
    filename: String,
    repo_id: u64,
    asset_id: u64,
    source_url: String,
    size: u64,
    sha256: String,
    exp: i64,
}
#[derive(Serialize, Deserialize, Debug)]
struct Receipt {
    nonce: String,
    repo_id: u64,
    asset_id: u64,
    filename: String,
    sha256: String,
    size: u64,
    file_key: String,
    readback_verified: bool,
    verified_at: i64,
}
fn signature(domain: &[u8], raw: &[u8], secret: &[u8]) -> String {
    let mut message = domain.to_vec();
    message.extend_from_slice(raw);
    crate::hex(hmac::sign(&hmac::Key::new(hmac::HMAC_SHA256, secret), &message).as_ref())
}
fn verify(domain: &[u8], raw: &[u8], sig: &str, secret: &[u8]) -> bool {
    let Some(bytes) = decode_hex(sig) else {
        return false;
    };
    let mut msg = domain.to_vec();
    msg.extend_from_slice(raw);
    hmac::verify(&hmac::Key::new(hmac::HMAC_SHA256, secret), &msg, &bytes).is_ok()
}
fn decode_hex(raw: &str) -> Option<Vec<u8>> {
    if raw.len() != 64 || !raw.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    (0..32)
        .map(|i| u8::from_str_radix(&raw[i * 2..i * 2 + 2], 16).ok())
        .collect()
}
fn hash_name(raw: &str) -> bool {
    raw.len() == 64
        && raw
            .bytes()
            .all(|b| b.is_ascii_digit() || matches!(b, b'a'..=b'f'))
}
fn filename(raw: &str) -> bool {
    !raw.is_empty()
        && raw.len() <= 255
        && !matches!(raw, "." | "..")
        && !raw
            .chars()
            .any(|c| c.is_control() || matches!(c, '/' | '\\'))
}
fn repo(raw: &str) -> bool {
    let Some((owner, name)) = raw.split_once('/') else {
        return false;
    };
    (1..=39).contains(&owner.len())
        && !owner.starts_with('-')
        && !owner.ends_with('-')
        && owner
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'-')
        && (1..=100).contains(&name.len())
        && !matches!(name, "." | "..")
        && name
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_' | b'.'))
}
fn claims_valid(c: &Claims, now: i64) -> bool {
    if uuid::Uuid::parse_str(&c.nonce).is_err()
        || !repo(&c.repository)
        || !filename(&c.filename)
        || c.version.is_empty()
        || c.version.len() > 128
        || c.version.chars().any(char::is_control)
        || c.repo_id == 0
        || c.asset_id == 0
        || c.size == 0
        || c.size > MAX_ASSET
        || !hash_name(&c.sha256)
        || c.exp < now
        || c.exp > now + 300
    {
        return false;
    }
    let Ok(mut expected) = reqwest::Url::parse(&format!(
        "https://github.com/{}/releases/download/",
        c.repository
    )) else {
        return false;
    };
    expected
        .path_segments_mut()
        .unwrap()
        .pop_if_empty()
        .push(&c.version)
        .push(&c.filename);
    expected.as_str() == c.source_url
}
fn allowed(url: &reqwest::Url) -> bool {
    url.scheme() == "https"
        && url.username().is_empty()
        && url.password().is_none()
        && url.port().is_none()
        && url.fragment().is_none()
        && url.as_str().len() <= 2000
        && match url.host_str() {
            Some("github.com") => url.path().contains("/releases/download/"),
            Some("release-assets.githubusercontent.com" | "objects.githubusercontent.com") => true,
            _ => false,
        }
}
fn err(status: StatusCode, msg: &str) -> (StatusCode, axum::Json<Value>) {
    (status, axum::Json(json!({"error":true,"message":msg})))
}
fn io_error(_: impl std::fmt::Display) -> (StatusCode, axum::Json<Value>) {
    err(StatusCode::INTERNAL_SERVER_ERROR, "镜像磁盘操作失败")
}
pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/internal/tool-mirrors",
            post(create).layer(DefaultBodyLimit::max(8192)),
        )
        .route("/tool-mirrors/{sha}/{filename}", get(download))
}
async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<Response> {
    let sig = headers
        .get("x-tool-signature")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if !verify(REQUEST_DOMAIN, &body, sig, &state.secret) {
        return Err(err(StatusCode::FORBIDDEN, "工具镜像签名无效"));
    }
    let claims: Claims = serde_json::from_slice(&body)
        .map_err(|_| err(StatusCode::BAD_REQUEST, "工具镜像请求格式无效"))?;
    if !claims_valid(&claims, crate::now()) {
        return Err(err(
            StatusCode::BAD_REQUEST,
            "工具镜像来源、大小或有效期无效",
        ));
    }
    let http = reqwest::Client::builder()
        .no_gzip()
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_secs(10))
        .user_agent("XiaobanToolMirror/1.0")
        .build()
        .map_err(io_error)?;
    let receipt = tokio::time::timeout(
        TIMEOUT,
        store(
            &state.data_dir,
            &http,
            &claims,
            allowed,
            MAX_STORAGE,
            MAX_OBJECTS,
        ),
    )
    .await
    .map_err(|_| err(StatusCode::GATEWAY_TIMEOUT, "工具镜像执行超过三分钟"))??;
    let raw = serde_json::to_vec(&receipt).map_err(io_error)?;
    let mut response = Response::new(Body::from(raw.clone()));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    response.headers_mut().insert(
        "x-tool-receipt-signature",
        HeaderValue::from_str(&signature(RECEIPT_DOMAIN, &raw, &state.secret)).map_err(io_error)?,
    );
    Ok(response)
}
struct Part(PathBuf);
impl Drop for Part {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
async fn hash_file(path: &FsPath, size: u64, expected: &str) -> ApiResult<()> {
    let meta = fs::symlink_metadata(path).await.map_err(io_error)?;
    if !meta.file_type().is_file() || meta.len() != size {
        return Err(err(
            StatusCode::UNPROCESSABLE_ENTITY,
            "镜像回读大小或文件类型不匹配",
        ));
    }
    let mut file = fs::File::open(path).await.map_err(io_error)?;
    let mut hash = digest::Context::new(&digest::SHA256);
    let mut read = 0;
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        let n = file.read(&mut buf).await.map_err(io_error)?;
        if n == 0 {
            break;
        }
        read += n as u64;
        if read > size {
            return Err(err(StatusCode::UNPROCESSABLE_ENTITY, "镜像回读大小不匹配"));
        }
        hash.update(&buf[..n]);
    }
    if read != size || crate::hex(hash.finish().as_ref()) != expected {
        return Err(err(
            StatusCode::UNPROCESSABLE_ENTITY,
            "镜像回读 SHA-256 不匹配",
        ));
    }
    Ok(())
}
async fn store(
    root: &FsPath,
    http: &reqwest::Client,
    c: &Claims,
    allowed: fn(&reqwest::Url) -> bool,
    max_bytes: u64,
    max_objects: usize,
) -> ApiResult<Receipt> {
    let dir = root.join("tools");
    fs::create_dir_all(&dir).await.map_err(io_error)?;
    // OS advisory locks are released on process exit, unlike stale create-new lock files.
    let lock_path = dir.join(".quota.lock");
    let lock = tokio::task::spawn_blocking(move || {
        let f = std::fs::File::options()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(lock_path)?;
        f.try_lock().map_err(std::io::Error::other)?;
        Ok::<_, std::io::Error>(f)
    })
    .await
    .map_err(io_error)?
    .map_err(|_| {
        err(
            StatusCode::SERVICE_UNAVAILABLE,
            "已有工具镜像写入，请稍后重试",
        )
    })?;
    let _lock = lock;
    let target = dir.join(format!("{}.bin", c.sha256));
    if fs::try_exists(&target).await.map_err(io_error)? {
        if let Err(error) = hash_file(&target, c.size, &c.sha256).await {
            let _ = fs::remove_file(&target).await;
            return Err(error);
        }
    } else {
        let (mut total, mut count) = (0u64, 0usize);
        let mut entries = fs::read_dir(&dir).await.map_err(io_error)?;
        while let Some(entry) = entries.next_entry().await.map_err(io_error)? {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name == ".quota.lock" {
                continue;
            }
            let ty = entry.file_type().await.map_err(io_error)?;
            if !ty.is_file() {
                return Err(err(StatusCode::CONFLICT, "镜像目录包含未知目录或符号链接"));
            }
            if name.ends_with(".part") {
                fs::remove_file(entry.path()).await.map_err(io_error)?;
                continue;
            }
            total = total.saturating_add(entry.metadata().await.map_err(io_error)?.len());
            count += 1;
        }
        if count >= max_objects || total.saturating_add(c.size) > max_bytes {
            return Err(err(
                StatusCode::INSUFFICIENT_STORAGE,
                "工具镜像配额已满（1 GiB / 128 对象），请人工维护",
            ));
        }
        let part = Part(dir.join(format!("{}.part", uuid::Uuid::new_v4())));
        let mut file = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&part.0)
            .await
            .map_err(io_error)?;
        let mut url = reqwest::Url::parse(&c.source_url).map_err(io_error)?;
        let mut response = None;
        for hop in 0..=3 {
            if !allowed(&url) {
                return Err(err(
                    StatusCode::BAD_REQUEST,
                    "安装包重定向不属于官方 GitHub 主机",
                ));
            }
            let r = http
                .get(url.clone())
                .header("accept-encoding", "identity")
                .timeout(TIMEOUT)
                .send()
                .await
                .map_err(|_| err(StatusCode::BAD_GATEWAY, "官方安装包下载失败或超时"))?;
            if r.url() != &url {
                return Err(err(StatusCode::BAD_GATEWAY, "镜像客户端禁止自动重定向"));
            }
            if r.status().is_redirection() {
                if hop == 3 {
                    return Err(err(StatusCode::BAD_GATEWAY, "官方安装包重定向超过三次"));
                }
                let location = r
                    .headers()
                    .get(header::LOCATION)
                    .and_then(|v| v.to_str().ok())
                    .filter(|v| v.len() <= 2000)
                    .ok_or_else(|| err(StatusCode::BAD_GATEWAY, "官方安装包重定向无效"))?;
                url = url.join(location).map_err(io_error)?;
                continue;
            }
            if !r.status().is_success() {
                return Err(err(StatusCode::BAD_GATEWAY, "官方安装包返回失败状态"));
            }
            if r.headers()
                .get(header::CONTENT_ENCODING)
                .is_some_and(|v| v != "identity")
                || r.content_length().is_some_and(|n| n != c.size)
            {
                return Err(err(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "官方安装包大小或编码不匹配",
                ));
            }
            response = Some(r);
            break;
        }
        let mut response =
            response.ok_or_else(|| err(StatusCode::BAD_GATEWAY, "官方安装包下载失败"))?;
        let mut hash = digest::Context::new(&digest::SHA256);
        let mut received = 0u64;
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|_| err(StatusCode::BAD_GATEWAY, "官方安装包下载中断"))?
        {
            received = received.saturating_add(chunk.len() as u64);
            if received > c.size {
                return Err(err(StatusCode::PAYLOAD_TOO_LARGE, "安装包实际大小超过预期"));
            }
            hash.update(&chunk);
            file.write_all(&chunk).await.map_err(io_error)?;
        }
        if received != c.size || crate::hex(hash.finish().as_ref()) != c.sha256 {
            return Err(err(
                StatusCode::UNPROCESSABLE_ENTITY,
                "官方安装包大小或 SHA-256 不匹配",
            ));
        }
        file.sync_all().await.map_err(io_error)?;
        drop(file);
        fs::hard_link(&part.0, &target).await.map_err(io_error)?;
        if let Err(error) = hash_file(&target, c.size, &c.sha256).await {
            let _ = fs::remove_file(&target).await;
            return Err(error);
        }
    }
    Ok(Receipt {
        nonce: c.nonce.clone(),
        repo_id: c.repo_id,
        asset_id: c.asset_id,
        filename: c.filename.clone(),
        sha256: c.sha256.clone(),
        size: c.size,
        file_key: format!("tools/{}.bin", c.sha256),
        readback_verified: true,
        verified_at: crate::now(),
    })
}
#[derive(Deserialize)]
struct Signed {
    exp: i64,
    sig: String,
}
async fn download(
    State(s): State<AppState>,
    Path((sha, name)): Path<(String, String)>,
    Query(q): Query<Signed>,
    request: Request<Body>,
) -> Response {
    if !hash_name(&sha)
        || !filename(&name)
        || q.exp > crate::now() + 3600
        || !crate::sign::verify_download(
            &format!("tool-mirrors/{sha}/{name}"),
            q.exp,
            &q.sig,
            crate::now(),
            &s.secret,
        )
    {
        return StatusCode::FORBIDDEN.into_response();
    }
    let path = s.data_dir.join("tools").join(format!("{sha}.bin"));
    if !fs::symlink_metadata(&path)
        .await
        .is_ok_and(|m| m.file_type().is_file())
    {
        return StatusCode::NOT_FOUND.into_response();
    }
    let mut response = match ServeFile::new(path).oneshot(request).await {
        Ok(r) => r.map(Body::new),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    let encoded = name
        .as_bytes()
        .iter()
        .map(|b| {
            if b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'_') {
                (*b as char).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect::<String>();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/octet-stream"),
    );
    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename*=UTF-8''{encoded}")).unwrap(),
    );
    response.headers_mut().insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, max-age=0"),
    );
    response
}
#[cfg(test)]
#[path = "tool_mirrors_tests.rs"]
mod tests;
