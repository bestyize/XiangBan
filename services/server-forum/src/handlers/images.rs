//! 正文插图：编辑器里上传一张图，得到站内地址 `/api/images/{id}`，写进 Markdown 的 `![](…)`。
//!
//! 只收 PNG、JPEG、GIF、WebP，按文件头判断真实类型（不信客户端给的类型和文件名），不收 SVG。
//! 文件放在 `FORUM_UPLOAD_DIR/img-{id}`，地址靠不可猜的 id 保护，和帖子的阅读等级无关。

use super::attachments::{io_err, multipart_err, too_large, write_field, MULTIPART_OVERHEAD};
use crate::{
    auth::AuthUser,
    error::{AppError, AppResult},
    state::AppState,
    users,
};
use axum::{
    body::Body,
    extract::{Multipart, Path, State},
    http::{header, HeaderMap, HeaderValue},
    response::Response,
    Json,
};
use serde_json::{json, Value};
use sqlx::{PgPool, Postgres, Transaction};
use std::collections::HashSet;
use std::path::{Path as FsPath, PathBuf};
use tokio::{fs::File, io::AsyncReadExt};
use tokio_util::io::ReaderStream;
use uuid::Uuid;

/// 每人每天最多传多少张，防止被拿来当图床。
const MAX_PER_DAY: i64 = 50;

/// 按文件头判断图片类型，返回 MIME。
pub fn sniff(head: &[u8]) -> Option<&'static str> {
    if head.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) {
        Some("image/png")
    } else if head.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Some("image/jpeg")
    } else if head.starts_with(b"GIF87a") || head.starts_with(b"GIF89a") {
        Some("image/gif")
    } else if head.len() >= 12 && &head[0..4] == b"RIFF" && &head[8..12] == b"WEBP" {
        Some("image/webp")
    } else {
        None
    }
}

fn path_for(dir: &FsPath, id: Uuid) -> PathBuf {
    dir.join(format!("img-{id}"))
}

/// Store a private copy on the resource host before making an upload visible.
/// The resource host checks the public forum endpoint before serving the image.
pub(super) async fn mirror_to_resource(state: &AppState, kind: &str, id: Uuid, mime: &str, data: &[u8]) -> AppResult<()> {
    super::resource_mirror::store(state, kind, id, mime, data, None).await
}

pub async fn upload(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> AppResult<Json<Value>> {
    let user = &auth.user;
    users::ensure_can_write(user)?;
    let limit = state.config.upload_limit_image;
    let declared = headers.get(header::CONTENT_LENGTH).and_then(|v| v.to_str().ok()).and_then(|v| v.parse::<u64>().ok());
    if declared.is_some_and(|n| n > limit + MULTIPART_OVERHEAD) {
        return Err(too_large(limit));
    }
    let (today,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM images WHERE uploader_uid = $1 AND created_at > now() - interval '1 day'",
    )
    .bind(user.uid)
    .fetch_one(&state.pool)
    .await?;
    if today >= MAX_PER_DAY {
        return Err(AppError::TooManyRequests { message: format!("今天上传的图片已达上限（{MAX_PER_DAY} 张）"), retry_after: 3600 });
    }

    let mut field = loop {
        match multipart.next_field().await.map_err(|e| multipart_err(e, limit))? {
            Some(f) if f.name() == Some("file") => break f,
            Some(_) => continue,
            None => return Err(AppError::BadRequest("没有收到图片".into())),
        }
    };

    let id = Uuid::new_v4();
    let dir = &state.config.upload_dir;
    tokio::fs::create_dir_all(dir).await.map_err(io_err)?;
    let tmp_path = dir.join(format!("{id}.part"));
    let final_path = path_for(dir, id);

    let checked = async {
        let size = write_field(&mut field, &tmp_path, limit).await?;
        let mut head = [0u8; 12];
        let mut file = File::open(&tmp_path).await.map_err(io_err)?;
        let mut got = 0;
        while got < head.len() {
            match file.read(&mut head[got..]).await.map_err(io_err)? {
                0 => break,
                n => got += n,
            }
        }
        let mime = sniff(&head[..got]).ok_or_else(|| AppError::BadRequest("只支持 PNG、JPEG、GIF、WebP 图片".into()))?;
        Ok::<_, AppError>((size, mime))
    }
    .await;
    let (size, mime) = match checked {
        Ok(v) => v,
        Err(e) => {
            let _ = tokio::fs::remove_file(&tmp_path).await;
            return Err(e);
        }
    };
    tokio::fs::rename(&tmp_path, &final_path).await.map_err(io_err)?;

    let mirrored = async {
        let bytes = tokio::fs::read(&final_path).await.map_err(io_err)?;
        mirror_to_resource(&state, "images", id, mime, &bytes).await
    }.await;
    if let Err(err) = mirrored {
        let _ = tokio::fs::remove_file(&final_path).await;
        return Err(err);
    }

    let inserted = sqlx::query("INSERT INTO images (id, uploader_uid, mime, size) VALUES ($1, $2, $3, $4)")
        .bind(id)
        .bind(user.uid)
        .bind(mime)
        .bind(size as i64)
        .execute(&state.pool)
        .await;
    if let Err(e) = inserted {
        let _ = tokio::fs::remove_file(&final_path).await;
        return Err(e.into());
    }
    Ok(Json(json!({ "id": id, "url": format!("/api/images/{id}"), "size": size })))
}

pub async fn show(State(state): State<AppState>, Path(id): Path<Uuid>) -> AppResult<Response> {
    let (mime, size): (String, i64) = sqlx::query_as("SELECT mime, size FROM images WHERE id = $1 AND NOT editor_pending")
        .bind(id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::not_found("图片"))?;
    let file = File::open(path_for(&state.config.upload_dir, id)).await.map_err(|_| AppError::not_found("图片"))?;
    let mut response = Response::new(Body::from_stream(ReaderStream::new(file)));
    let h = response.headers_mut();
    if let Ok(v) = HeaderValue::from_str(&mime) {
        h.insert(header::CONTENT_TYPE, v);
    }
    h.insert(header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    // id 不可变，浏览器和代理都可以长期缓存。
    h.insert(header::CACHE_CONTROL, HeaderValue::from_static("public, max-age=31536000, immutable"));
    h.insert(header::CONTENT_SECURITY_POLICY, HeaderValue::from_static("default-src 'none'; sandbox"));
    h.insert(header::CONTENT_LENGTH, HeaderValue::from(size.max(0) as u64));
    Ok(response)
}

/// Authorization-only HEAD for the resource host; do not read the image body.
pub async fn head(State(state): State<AppState>, Path(id): Path<Uuid>) -> AppResult<axum::http::StatusCode> {
    let (allowed,): (bool,) = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM images WHERE id=$1 AND NOT editor_pending)")
        .bind(id).fetch_one(&state.pool).await?;
    if allowed { Ok(axum::http::StatusCode::OK) } else { Err(AppError::not_found("图片")) }
}

pub enum CommentImageTarget {
    Daily(i64),
    Frontier(i64),
    Repo(i64),
}

pub fn validate_comment_content(body: &str, image_ids: &[Uuid]) -> AppResult<()> {
    let length = body.chars().count();
    if length == 0 && image_ids.is_empty() {
        return Err(AppError::BadRequest("请输入评论内容或添加图片".into()));
    }
    if length > 2000 {
        return Err(AppError::BadRequest("评论最多 2000 个字".into()));
    }
    if image_ids.iter().collect::<HashSet<_>>().len() != image_ids.len() {
        return Err(AppError::BadRequest("请移除重复图片".into()));
    }
    Ok(())
}

pub fn comment_urls(image_ids: Vec<Uuid>) -> Vec<String> {
    image_ids.into_iter().map(|id| format!("/api/images/{id}")).collect()
}

pub async fn attach_to_comment(
    tx: &mut Transaction<'_, Postgres>,
    target: CommentImageTarget,
    author: Uuid,
    image_ids: &[Uuid],
) -> AppResult<()> {
    if image_ids.is_empty() {
        return Ok(());
    }
    let (daily, frontier, repo) = match target {
        CommentImageTarget::Daily(id) => (Some(id), None, None),
        CommentImageTarget::Frontier(id) => (None, Some(id), None),
        CommentImageTarget::Repo(id) => (None, None, Some(id)),
    };
    let attached = sqlx::query(
        "INSERT INTO discover_comment_images(daily_comment_id,frontier_comment_id,repo_comment_id,image_id,position)
         SELECT $1,$2,$3,i.id,selected.position
         FROM unnest($4::uuid[]) WITH ORDINALITY AS selected(id,position)
         JOIN images i ON i.id=selected.id AND i.uploader_uid=$5 AND NOT i.editor_pending
         FOR KEY SHARE OF i",
    )
    .bind(daily)
    .bind(frontier)
    .bind(repo)
    .bind(image_ids)
    .bind(author)
    .execute(&mut **tx)
    .await?;
    if attached.rows_affected() != image_ids.len() as u64 {
        return Err(AppError::BadRequest("部分图片已失效，请重新选图".into()));
    }
    Ok(())
}

/// 账号中心可能先保存成功，论坛需要保留原图直到同步确认。
pub async fn hold_avatar(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    let updated = sqlx::query("UPDATE images SET avatar_pending=true WHERE id=$1 AND uploader_uid=$2 AND NOT editor_pending")
        .bind(id).bind(auth.user.uid).execute(&state.pool).await?;
    if updated.rows_affected() == 0 {
        return Err(AppError::not_found("图片"));
    }
    Ok(Json(json!({"ok": true})))
}

/// 清掉上传超过一天且未关联帖子、评论、待发布稿或用户头像的图片。
/// 头像存在账号中心，论坛 `users.avatar_url` 是登录时同步的副本，图片 id 出现在里面就保留。
pub async fn purge_unused(pool: &PgPool, dir: &FsPath) -> AppResult<usize> {
    let rows: Vec<(Uuid,)> = sqlx::query_as(
        "DELETE FROM images i
         WHERE i.created_at < now() - interval '1 day'
           AND NOT i.avatar_pending
           AND NOT EXISTS (SELECT 1 FROM topics t WHERE position(i.id::text in t.body_md) > 0)
           AND NOT EXISTS (SELECT 1 FROM posts p WHERE position(i.id::text in p.body_md) > 0)
           AND NOT EXISTS (SELECT 1 FROM editor_draft_assets a WHERE a.image_id = i.id)
           AND NOT EXISTS (SELECT 1 FROM discover_comment_images a WHERE a.image_id = i.id)
           AND NOT EXISTS (SELECT 1 FROM users u WHERE u.avatar_url IS NOT NULL AND position(i.id::text in u.avatar_url) > 0)
         RETURNING i.id",
    )
    .fetch_all(pool)
    .await?;
    for (id,) in &rows {
        if let Err(e) = tokio::fs::remove_file(path_for(dir, *id)).await {
            if e.kind() != std::io::ErrorKind::NotFound {
                tracing::warn!("删除图片文件失败 {id}: {e}");
            }
        }
    }
    Ok(rows.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sniff_recognizes_real_images_only() {
        assert_eq!(sniff(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 0]), Some("image/png"));
        assert_eq!(sniff(&[0xFF, 0xD8, 0xFF, 0xE0]), Some("image/jpeg"));
        assert_eq!(sniff(b"GIF89a\x01\x00\x01\x00"), Some("image/gif"));
        assert_eq!(sniff(b"RIFF\x24\x00\x00\x00WEBPVP8 "), Some("image/webp"));
        // 别的 RIFF（比如 WAV）、SVG、HTML、脚本都不行
        assert_eq!(sniff(b"RIFF\x24\x00\x00\x00WAVEfmt "), None);
        assert_eq!(sniff(b"<svg xmlns=\"http://www.w3.org/2000/svg\">"), None);
        assert_eq!(sniff(b"<!DOCTYPE html>"), None);
        assert_eq!(sniff(b""), None);
    }

    #[tokio::test]
    async fn mirror_signs_content_and_rejects_a_bad_receipt() {
        use axum::{body::Bytes, routing::post, Router};
        use crate::{auth::JwksVerifier, config::AppConfig};
        use std::sync::Arc;

        const SECRET: &[u8] = b"0123456789abcdef0123456789abcdef";
        let app = Router::new().route("/internal/forum-image/images/{id}", post(|Path(id): Path<Uuid>, headers: HeaderMap, body: Bytes| async move {
            let hash = headers["X-Content-Sha256"].to_str().unwrap();
            let expires = headers["X-Expiration"].to_str().unwrap();
            let message = format!("POST\nimages/{id}\n{hash}\n{}\n{expires}", body.len());
            assert!(crate::content::verify_callback(message.as_bytes(), headers["X-Signature"].to_str().unwrap(), SECRET));
            let receipt_hash = if body.ends_with(b"bad") { "bad" } else { hash };
            Json(json!({"object_key":format!("forum-content/objects/{}.png", "a".repeat(32)),"sha256":receipt_hash,"size":body.len()}))
        })).route("/internal/forum-file/{id}", post(|Path(id): Path<Uuid>, headers: HeaderMap, body: Bytes| async move {
            let hash = headers["X-Content-Sha256"].to_str().unwrap();
            let expires = headers["X-Expiration"].to_str().unwrap();
            let ext = headers["X-File-Ext"].to_str().unwrap();
            let message = format!("POST\nattachments/{id}\n{hash}\n{}\n{expires}\n{ext}", body.len());
            assert!(crate::content::verify_callback(message.as_bytes(), headers["X-Signature"].to_str().unwrap(), SECRET));
            Json(json!({"object_key":format!("forum-content/objects/{}.{ext}", "b".repeat(32)),"sha256":hash,"size":body.len()}))
        }));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let mut config = AppConfig::from_env();
        config.resource_base_url = Some(format!("http://{addr}"));
        config.resource_secret = SECRET.to_vec();
        let state = AppState {
            pool: sqlx::postgres::PgPoolOptions::new().connect_lazy("postgres://localhost/test").unwrap(),
            config: Arc::new(config),
            verifier: Arc::new(JwksVerifier::new("test".into(), "test".into(), "test".into())),
            http: reqwest::Client::new(),
        };
        assert!(mirror_to_resource(&state, "images", Uuid::new_v4(), "image/png", b"good").await.is_ok());
        assert!(mirror_to_resource(&state, "images", Uuid::new_v4(), "image/png", b"bad").await.is_err());
        assert!(mirror_to_resource(&state, "../bad", Uuid::new_v4(), "image/png", b"good").await.is_err());
        assert!(super::super::resource_mirror::store(&state, "attachments", Uuid::new_v4(), "application/octet-stream", b"file", Some("zip")).await.is_ok());
        assert!(super::super::resource_mirror::store(&state, "attachments", Uuid::new_v4(), "application/octet-stream", b"file", Some("../zip")).await.is_err());
        server.abort();
    }
}

