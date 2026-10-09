//! Managed installer uploads and per-version publication for Xiaoban Desktop.
use crate::{
    api::AppState,
    desktop::{self, DesktopConfig, Package, Release},
    error::AppError,
};
use axum::{
    extract::{Multipart, Path, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;
use tower::ServiceExt;

pub const MAX_PACKAGE: u64 = 512 * 1024 * 1024;
const PREFIX: &str = "/api/v1/desktop/downloads/";
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoredPackage {
    id: String,
    target: String,
    file_name: String,
    extension: String,
    sha256: String,
    bytes: u64,
}
struct RemoveTemporary(PathBuf);
impl Drop for RemoveTemporary {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
fn bad(message: &str) -> AppError {
    AppError::BadRequest(message.into())
}
fn valid_id(id: &str) -> bool {
    id.len() == 32 && id.bytes().all(|c| c.is_ascii_hexdigit())
}
fn key(id: &str) -> String {
    format!("desktop.package.{id}")
}
fn path(package: &StoredPackage) -> PathBuf {
    PathBuf::from("app-packages").join(format!("{}.{}", package.id, package.extension))
}
pub fn package_id(url: &str) -> Option<&str> {
    if let Some(id) = url.strip_prefix(PREFIX) {
        return Some(id).filter(|id| valid_id(id));
    }
    if desktop::network_url(url).is_err() {
        return None;
    }
    let (_, rest) = url.split_once("://")?;
    let (_, relative) = rest.split_once('/')?;
    relative
        .strip_prefix(PREFIX.trim_start_matches('/'))
        .filter(|id| valid_id(id))
}
pub fn is_owned_url(url: &str) -> bool {
    package_id(url).is_some()
}
fn extension(target: &str, name: &str) -> Result<String, AppError> {
    if name.is_empty()
        || name.len() > 240
        || name
            .chars()
            .any(|c| c.is_control() || c == '/' || c == '\\')
    {
        return Err(bad("安装包文件名无效"));
    }
    let ext = name.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    let allowed = match target {
        "windows-x86_64" => ["exe", "msi", "zip"].contains(&ext.as_str()),
        "darwin-aarch64" | "darwin-x86_64" => ["dmg", "pkg", "zip"].contains(&ext.as_str()),
        _ => false,
    };
    if !allowed {
        return Err(bad(
            "Windows 支持 EXE/MSI/ZIP，macOS 支持 DMG/PKG/ZIP；请检查平台与文件格式",
        ));
    }
    Ok(ext)
}
async fn stored(state: &AppState, id: &str) -> Result<StoredPackage, AppError> {
    if !valid_id(id) {
        return Err(bad("安装包 ID 无效"));
    }
    let value = state
        .admin
        .store()?
        .get_setting(&key(id))
        .await?
        .ok_or_else(|| AppError::NotFound("安装包不存在".into()))?;
    serde_json::from_str(&value).map_err(|e| AppError::Internal(e.to_string()))
}
pub async fn normalize_release(state: &AppState, release: &mut Release) -> Result<(), AppError> {
    for (target, package) in &mut release.platforms {
        if let Some(id) = package_id(&package.url) {
            let uploaded = stored(state, id).await?;
            if uploaded.target != *target {
                return Err(bad("上传的安装包与发布平台不匹配"));
            }
            if !tokio::fs::metadata(path(&uploaded))
                .await
                .is_ok_and(|m| m.is_file() && m.len() == uploaded.bytes)
            {
                return Err(bad("安装包文件丢失或大小已改变，请重新上传"));
            }
            package.sha256 = Some(uploaded.sha256);
            package.bytes = Some(uploaded.bytes);
            package.url = format!("{PREFIX}{}", uploaded.id);
        }
    }
    Ok(())
}
pub async fn upload(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let id = uuid::Uuid::new_v4().simple().to_string();
    tokio::fs::create_dir_all("app-packages")
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let temporary = PathBuf::from("app-packages").join(format!("{id}.part"));
    let _cleanup = RemoveTemporary(temporary.clone());
    let mut target: Option<String> = None;
    let mut result: Option<StoredPackage> = None;
    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|_| bad("安装包上传数据不完整"))?
    {
        match field.name().unwrap_or("") {
            "target" if target.is_none() && result.is_none() => {
                let mut bytes = Vec::new();
                while let Some(chunk) = field.chunk().await.map_err(|_| bad("平台字段无效"))?
                {
                    if bytes.len() + chunk.len() > 64 {
                        return Err(bad("平台字段过长"));
                    }
                    bytes.extend_from_slice(&chunk);
                }
                let value = String::from_utf8(bytes).map_err(|_| bad("平台字段无效"))?;
                if !["windows-x86_64", "darwin-aarch64", "darwin-x86_64"].contains(&value.as_str())
                {
                    return Err(bad("不支持的平台"));
                }
                target = Some(value);
            }
            "file" if result.is_none() => {
                let target = target.as_ref().ok_or_else(|| bad("请先指定安装包平台"))?;
                let name = field
                    .file_name()
                    .ok_or_else(|| bad("请选择安装包文件"))?
                    .to_string();
                let ext = extension(target, &name)?;
                let mut file = tokio::fs::File::create(&temporary)
                    .await
                    .map_err(|e| AppError::Internal(e.to_string()))?;
                let mut digest = Sha256::new();
                let mut bytes = 0u64;
                while let Some(chunk) = field.chunk().await.map_err(|_| bad("安装包上传中断"))?
                {
                    bytes += chunk.len() as u64;
                    if bytes > MAX_PACKAGE {
                        return Err(bad("安装包不能超过 512 MiB"));
                    }
                    digest.update(&chunk);
                    file.write_all(&chunk)
                        .await
                        .map_err(|e| AppError::Internal(e.to_string()))?;
                }
                if bytes == 0 {
                    return Err(bad("安装包不能为空"));
                }
                file.flush()
                    .await
                    .map_err(|e| AppError::Internal(e.to_string()))?;
                drop(file);
                result = Some(StoredPackage {
                    id: id.clone(),
                    target: target.clone(),
                    file_name: name,
                    extension: ext,
                    sha256: format!("{:x}", digest.finalize()),
                    bytes,
                });
            }
            _ => return Err(bad("每次只能上传一个安装包，并指定一个平台")),
        }
    }
    let uploaded = result.ok_or_else(|| bad("未收到安装包"))?;
    let destination = path(&uploaded);
    tokio::fs::rename(&temporary, &destination)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    if let Err(error) = state
        .admin
        .store()?
        .set_setting(&key(&id), &serde_json::to_string(&uploaded).unwrap())
        .await
    {
        let _ = tokio::fs::remove_file(&destination).await;
        return Err(error);
    }
    Ok((
        StatusCode::CREATED,
        Json(
            json!({"id":id,"url":format!("{PREFIX}{id}"),"fileName":uploaded.file_name,"sha256":uploaded.sha256,"bytes":uploaded.bytes}),
        ),
    ))
}
pub async fn list(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    let mut releases = desktop::load_config(state.admin.store()?).await?.releases;
    releases.sort_by(|a, b| b.published_at.cmp(&a.published_at));
    Ok(Json(json!({"releases":releases})))
}
pub async fn save(
    State(state): State<AppState>,
    Path(version): Path<String>,
    Json(mut release): Json<Release>,
) -> Result<Json<Release>, AppError> {
    if release.version != version {
        return Err(bad("版本号与请求地址不一致"));
    }
    normalize_release(&state, &mut release).await?;
    state
        .admin
        .store()?
        .update_desktop_config(|config| {
            config.releases.retain(|r| r.version != version);
            config.releases.push(release.clone());
            Ok(())
        })
        .await?;
    Ok(Json(release))
}
pub async fn remove(
    State(state): State<AppState>,
    Path(version): Path<String>,
) -> Result<Json<Value>, AppError> {
    state
        .admin
        .store()?
        .update_desktop_config(|config| {
            let before = config.releases.len();
            config.releases.retain(|r| r.version != version);
            if before == config.releases.len() {
                return Err(AppError::NotFound("版本不存在".into()));
            }
            Ok(())
        })
        .await?;
    Ok(Json(json!({"success":true})))
}
fn encoded_name(value: &str) -> String {
    value
        .as_bytes()
        .iter()
        .map(|b| {
            if b.is_ascii_alphanumeric() || matches!(*b, b'-' | b'.' | b'_') {
                (*b as char).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect()
}
pub fn origin(headers: &HeaderMap, fallback: &str) -> String {
    if let Some(host) = headers.get(header::HOST).and_then(|h| h.to_str().ok()) {
        if let Ok(mut url) = desktop::network_url(&format!("https://{host}")) {
            if url.path() == "/" && url.query().is_none() {
                if matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]")) {
                    let _ = url.set_scheme("http");
                }
                return url.as_str().trim_end_matches('/').to_string();
            }
        }
    }
    fallback.to_string()
}
pub fn public_platforms(
    release: &Release,
    origin: &str,
) -> std::collections::BTreeMap<String, Package> {
    release
        .platforms
        .iter()
        .map(|(target, package)| {
            let mut package = package.clone();
            if let Some(id) = package_id(&package.url) {
                package.url = format!("{origin}{PREFIX}{id}");
            }
            (target.clone(), package)
        })
        .collect()
}
pub async fn download(
    State(state): State<AppState>,
    Path(id): Path<String>,
    request: axum::extract::Request,
) -> Result<Response, AppError> {
    let uploaded = stored(&state, &id).await?;
    let config: DesktopConfig = desktop::load_config(state.admin.store()?).await?;
    if !config.releases.iter().any(|r| {
        r.published
            && r.published_at <= chrono::Utc::now()
            && r.platforms
                .values()
                .any(|p| package_id(&p.url) == Some(id.as_str()))
    }) {
        return Err(AppError::NotFound("安装包尚未上架或已下架".into()));
    }
    if !tokio::fs::metadata(path(&uploaded))
        .await
        .is_ok_and(|m| m.is_file() && m.len() == uploaded.bytes)
    {
        return Err(AppError::NotFound("安装包文件不可用".into()));
    }
    let response = tower_http::services::ServeFile::new(path(&uploaded))
        .oneshot(request)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut response = response.into_response();
    let disposition = format!(
        "attachment; filename=\"XiaoBan-{}.{}\"; filename*=UTF-8''{}",
        uploaded.target,
        uploaded.extension,
        encoded_name(&uploaded.file_name)
    );
    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&disposition).map_err(|_| bad("安装包文件名无效"))?,
    );
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/octet-stream"),
    );
    response.headers_mut().insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    Ok(response)
}
