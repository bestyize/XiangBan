//! Authenticated storage of forum images and attachments on the resource host.
use crate::{content, error::{AppError, AppResult}, state::AppState};
use axum::http::header;
use serde_json::Value;
use std::time::Duration;
use uuid::Uuid;

pub(super) async fn store(state: &AppState, kind: &str, id: Uuid, mime: &str, data: &[u8], file_ext: Option<&str>) -> AppResult<()> {
    let Some(base) = &state.config.resource_base_url else { return Ok(()) };
    let expected_ext = match (kind, mime, file_ext) {
        ("attachments", "application/octet-stream", Some(ext)) if !ext.is_empty() && ext.len() <= 8 && ext.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit()) => ext,
        ("images" | "discover/frontier-assets" | "discover/article-assets" | "open-source/assets", "image/png", None) => "png",
        ("images" | "discover/frontier-assets" | "discover/article-assets" | "open-source/assets", "image/jpeg", None) => "jpg",
        ("images" | "discover/frontier-assets" | "discover/article-assets" | "open-source/assets", "image/webp", None) => "webp",
        ("images" | "discover/frontier-assets" | "discover/article-assets" | "open-source/assets", "image/gif", None) => "gif",
        _ => return Err(AppError::Internal("资源类型无效".into())),
    };
    let hash = ring::digest::digest(&ring::digest::SHA256, data).as_ref().iter().map(|b| format!("{b:02x}")).collect::<String>();
    let expires = chrono::Utc::now().timestamp() + 120;
    let mut message = format!("POST\n{kind}/{id}\n{hash}\n{}\n{expires}", data.len());
    let path = if let Some(ext) = file_ext {
        message.push_str(&format!("\n{ext}"));
        format!("/internal/forum-file/{id}")
    } else { format!("/internal/forum-image/{kind}/{id}") };
    let signature = content::sign_callback(message.as_bytes(), &state.config.resource_secret);
    let mut request = state.http.post(format!("{base}{path}"))
        .timeout(Duration::from_secs(120))
        .header(header::CONTENT_TYPE, mime)
        .header("X-Content-Sha256", &hash)
        .header("X-Expiration", expires)
        .header("X-Signature", signature);
    if let Some(ext) = file_ext { request = request.header("X-File-Ext", ext); }
    let response = request.body(data.to_vec()).send().await
        .map_err(|e| AppError::Internal(format!("资源机上传失败: {e}")))?;
    if !response.status().is_success() {
        return Err(AppError::Internal(format!("资源机上传失败: HTTP {}", response.status())));
    }
    let receipt: Value = response.json().await.map_err(|e| AppError::Internal(format!("资源机回执无效: {e}")))?;
    let name = receipt["object_key"].as_str().and_then(|key| key.strip_prefix("forum-content/objects/")).unwrap_or("");
    let valid_name = name.split_once('.').is_some_and(|(digest, ext)| digest.len() == 32 &&
        digest.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) && ext == expected_ext);
    if !valid_name || receipt["sha256"].as_str() != Some(hash.as_str()) || receipt["size"].as_u64() != Some(data.len() as u64) {
        return Err(AppError::Internal("资源机回执校验失败".into()));
    }
    Ok(())
}
