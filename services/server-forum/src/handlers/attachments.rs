//! 帖子附件：先上传（得到 id），发帖或编辑时用 `attachment_ids` 挂到帖子上。
//!
//! 文件放在 `FORUM_UPLOAD_DIR/{id}`，数据库只记文件名和大小。下载一律按
//! `application/octet-stream` + `attachment` 输出，浏览器不会在站内直接打开用户上传的 HTML 或 SVG。

use super::topics::GUARDIAN_LEVEL;
use crate::{
    auth::{AuthUser, MaybeUser},
    config::AppConfig,
    error::{AppError, AppResult},
    models::Reader,
    state::AppState,
    users,
};
use axum::{
    body::Body,
    extract::{multipart::Field, multipart::MultipartError, Multipart, Path, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::Response,
    Json,
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::{json, Value};
use sqlx::{PgPool, Postgres, Transaction};
use std::path::Path as FsPath;
use tokio::{fs::File, io::AsyncWriteExt};
use tokio_util::io::ReaderStream;
use uuid::Uuid;

/// 一个帖子最多挂几个附件。
pub const MAX_PER_TOPIC: usize = 5;
/// 已上传但还没挂到帖子上的附件上限，防止只传不发占满磁盘。
const MAX_PENDING: i64 = 10;
/// multipart 头部和边界的余量，用于按 Content-Length 提前拒绝超大请求。
const MULTIPART_OVERHEAD: u64 = 64 * 1024;

#[derive(Serialize, sqlx::FromRow)]
pub struct AttachmentItem {
    pub id: Uuid,
    pub name: String,
    pub size: i64,
    pub created_at: DateTime<Utc>,
}

/// 普通用户和守护者（站务人员，信任等级 3）的单文件上限不同。
pub fn upload_limit(config: &AppConfig, trust_level: i16) -> u64 {
    pick_limit(trust_level, config.upload_limit_user, config.upload_limit_admin)
}

fn pick_limit(trust_level: i16, user: u64, admin: u64) -> u64 {
    if trust_level >= GUARDIAN_LEVEL {
        admin
    } else {
        user
    }
}

fn too_large(limit: u64) -> AppError {
    AppError::BadRequest(format!("文件太大，你最多可以上传 {} MB 以内的文件", limit / (1024 * 1024)))
}

fn io_err(e: std::io::Error) -> AppError {
    AppError::Internal(format!("附件文件读写失败: {e}"))
}

fn multipart_err(e: MultipartError, limit: u64) -> AppError {
    if e.status() == StatusCode::PAYLOAD_TOO_LARGE {
        too_large(limit)
    } else {
        AppError::BadRequest("上传失败，请重试".into())
    }
}

/// 只留文件名本身：去掉路径、控制字符，过长时保留扩展名截断。
pub fn clean_filename(raw: &str) -> String {
    let base = raw.rsplit(['/', '\\']).next().unwrap_or("");
    let name: String = base.chars().filter(|c| !c.is_control()).collect();
    let name = name.trim();
    if name.is_empty() {
        return "file".into();
    }
    if name.chars().count() <= 120 {
        return name.to_string();
    }
    let (stem, ext) = match name.rfind('.') {
        Some(i) if name[i..].chars().count() <= 16 => name.split_at(i),
        _ => (name, ""),
    };
    let keep = 120 - ext.chars().count();
    format!("{}{ext}", stem.chars().take(keep).collect::<String>())
}

pub async fn upload(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> AppResult<Json<Value>> {
    let user = &auth.user;
    users::ensure_can_write(user)?;
    if user.trust_level < 1 {
        return Err(AppError::Forbidden("新人暂时不能上传附件，升级为成员后就可以了".into()));
    }
    let limit = upload_limit(&state.config, user.trust_level);
    let declared = headers.get(header::CONTENT_LENGTH).and_then(|v| v.to_str().ok()).and_then(|v| v.parse::<u64>().ok());
    if declared.is_some_and(|n| n > limit + MULTIPART_OVERHEAD) {
        return Err(too_large(limit));
    }
    let (pending,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM attachments WHERE uploader_uid = $1 AND topic_id IS NULL")
            .bind(user.uid)
            .fetch_one(&state.pool)
            .await?;
    if pending >= MAX_PENDING {
        return Err(AppError::BadRequest("有太多还没挂到帖子上的附件，请先发布帖子或删掉一些".into()));
    }

    let mut field = loop {
        match multipart.next_field().await.map_err(|e| multipart_err(e, limit))? {
            Some(f) if f.name() == Some("file") => break f,
            Some(_) => continue,
            None => return Err(AppError::BadRequest("没有收到文件".into())),
        }
    };
    let filename = clean_filename(field.file_name().unwrap_or(""));

    let id = Uuid::new_v4();
    let dir = &state.config.upload_dir;
    tokio::fs::create_dir_all(dir).await.map_err(io_err)?;
    let final_path = dir.join(id.to_string());
    let tmp_path = dir.join(format!("{id}.part"));

    let size = match write_field(&mut field, &tmp_path, limit).await {
        Ok(n) => n,
        Err(e) => {
            let _ = tokio::fs::remove_file(&tmp_path).await;
            return Err(e);
        }
    };
    if size == 0 {
        let _ = tokio::fs::remove_file(&tmp_path).await;
        return Err(AppError::BadRequest("文件是空的".into()));
    }
    tokio::fs::rename(&tmp_path, &final_path).await.map_err(io_err)?;

    let inserted = sqlx::query("INSERT INTO attachments (id, uploader_uid, filename, size) VALUES ($1, $2, $3, $4)")
        .bind(id)
        .bind(user.uid)
        .bind(&filename)
        .bind(size as i64)
        .execute(&state.pool)
        .await;
    if let Err(e) = inserted {
        let _ = tokio::fs::remove_file(&final_path).await;
        return Err(e.into());
    }
    Ok(Json(json!({ "id": id, "name": filename, "size": size })))
}

async fn write_field(field: &mut Field<'_>, path: &FsPath, limit: u64) -> AppResult<u64> {
    let mut file = File::create(path).await.map_err(io_err)?;
    let mut size = 0u64;
    while let Some(chunk) = field.chunk().await.map_err(|e| multipart_err(e, limit))? {
        size += chunk.len() as u64;
        if size > limit {
            return Err(too_large(limit));
        }
        file.write_all(&chunk).await.map_err(io_err)?;
    }
    file.flush().await.map_err(io_err)?;
    Ok(size)
}

/// 撤销一个还没挂到帖子上的上传。已经挂在帖子上的附件通过编辑帖子移除。
pub async fn delete_pending(State(state): State<AppState>, auth: AuthUser, Path(id): Path<Uuid>) -> AppResult<Json<Value>> {
    let done = sqlx::query("DELETE FROM attachments WHERE id = $1 AND uploader_uid = $2 AND topic_id IS NULL")
        .bind(id)
        .bind(auth.user.uid)
        .execute(&state.pool)
        .await?
        .rows_affected();
    if done == 0 {
        return Err(AppError::not_found("附件"));
    }
    remove_files(&state.config.upload_dir, &[id]).await;
    Ok(Json(json!({ "ok": true })))
}

#[derive(sqlx::FromRow)]
struct DownloadRow {
    filename: String,
    size: i64,
    uploader_uid: Uuid,
    topic_id: Option<i64>,
    topic_status: Option<String>,
    min_read_level: Option<i16>,
    author_uid: Option<Uuid>,
}

pub async fn download(
    State(state): State<AppState>,
    MaybeUser(user): MaybeUser,
    Path(id): Path<Uuid>,
) -> AppResult<Response> {
    let row: DownloadRow = sqlx::query_as(
        "SELECT a.filename, a.size, a.uploader_uid, a.topic_id, t.status AS topic_status,
                t.min_read_level, t.author_uid
         FROM attachments a LEFT JOIN topics t ON t.id = a.topic_id WHERE a.id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("附件"))?;

    match (row.topic_id, row.author_uid) {
        (Some(_), Some(author)) => {
            // 和读帖子同一套规则：不可见的帖子对外是 404，等级不够是 403。
            let is_author = user.as_ref().is_some_and(|u| u.uid == author);
            if row.topic_status.as_deref() != Some("normal") && !is_author {
                return Err(AppError::not_found("附件"));
            }
            Reader::from_user(user.as_ref()).check(row.min_read_level.unwrap_or(0), author)?;
        }
        // 还没挂到帖子上的，只有上传的人自己能取。
        _ => {
            if !user.as_ref().is_some_and(|u| u.uid == row.uploader_uid) {
                return Err(AppError::not_found("附件"));
            }
        }
    }

    let file = File::open(state.config.upload_dir.join(id.to_string()))
        .await
        .map_err(|_| AppError::not_found("附件"))?;
    let mut response = Response::new(Body::from_stream(ReaderStream::new(file)));
    let h = response.headers_mut();
    h.insert(header::CONTENT_TYPE, HeaderValue::from_static("application/octet-stream"));
    h.insert(header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    h.insert(header::CACHE_CONTROL, HeaderValue::from_static("private, max-age=0"));
    // 带上 identity 让压缩中间件跳过，这样浏览器能拿到 Content-Length 显示下载进度。
    h.insert(header::CONTENT_ENCODING, HeaderValue::from_static("identity"));
    h.insert(header::CONTENT_LENGTH, HeaderValue::from(row.size.max(0) as u64));
    if let Ok(v) = HeaderValue::from_str(&content_disposition(&row.filename)) {
        h.insert(header::CONTENT_DISPOSITION, v);
    }
    Ok(response)
}

pub fn content_disposition(filename: &str) -> String {
    let fallback: String =
        filename.chars().map(|c| if c.is_ascii_graphic() && !matches!(c, '"' | '\\' | '%') || c == ' ' { c } else { '_' }).collect();
    let mut encoded = String::new();
    for b in filename.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~') {
            encoded.push(b as char);
        } else {
            encoded.push_str(&format!("%{b:02X}"));
        }
    }
    format!("attachment; filename=\"{fallback}\"; filename*=UTF-8''{encoded}")
}

pub async fn list_for_topic(pool: &PgPool, topic_id: i64) -> AppResult<Vec<AttachmentItem>> {
    Ok(sqlx::query_as(
        "SELECT id, filename AS name, size, created_at FROM attachments WHERE topic_id = $1 ORDER BY created_at, id",
    )
    .bind(topic_id)
    .fetch_all(pool)
    .await?)
}

/// 把帖子的附件设成 `ids`：不在列表里的删除（返回它们的 id，提交后再清文件），
/// 列表里属于自己且还没挂过的挂上来。列表里有别人的或已失效的附件就整体失败。
pub async fn set_for_topic(
    tx: &mut Transaction<'_, Postgres>,
    topic_id: i64,
    uid: Uuid,
    ids: &[Uuid],
) -> AppResult<Vec<Uuid>> {
    let mut ids = ids.to_vec();
    ids.sort();
    ids.dedup();
    if ids.len() > MAX_PER_TOPIC {
        return Err(AppError::BadRequest(format!("一个帖子最多 {MAX_PER_TOPIC} 个附件")));
    }
    let removed: Vec<(Uuid,)> =
        sqlx::query_as("DELETE FROM attachments WHERE topic_id = $1 AND NOT (id = ANY($2)) RETURNING id")
            .bind(topic_id)
            .bind(&ids)
            .fetch_all(&mut **tx)
            .await?;
    sqlx::query("UPDATE attachments SET topic_id = $1 WHERE id = ANY($2) AND topic_id IS NULL AND uploader_uid = $3")
        .bind(topic_id)
        .bind(&ids)
        .bind(uid)
        .execute(&mut **tx)
        .await?;
    let (attached,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM attachments WHERE topic_id = $1 AND id = ANY($2)")
        .bind(topic_id)
        .bind(&ids)
        .fetch_one(&mut **tx)
        .await?;
    if attached as usize != ids.len() {
        return Err(AppError::BadRequest("有附件已失效，请重新上传".into()));
    }
    Ok(removed.into_iter().map(|(id,)| id).collect())
}

pub async fn remove_files(dir: &FsPath, ids: &[Uuid]) {
    for id in ids {
        if let Err(e) = tokio::fs::remove_file(dir.join(id.to_string())).await {
            if e.kind() != std::io::ErrorKind::NotFound {
                tracing::warn!("删除附件文件失败 {id}: {e}");
            }
        }
    }
}

/// 清掉超过一天还没挂到帖子上的上传。
pub async fn purge_orphans(pool: &PgPool, dir: &FsPath) -> AppResult<usize> {
    let rows: Vec<(Uuid,)> = sqlx::query_as(
        "DELETE FROM attachments WHERE topic_id IS NULL AND created_at < now() - interval '1 day' RETURNING id",
    )
    .fetch_all(pool)
    .await?;
    let ids: Vec<Uuid> = rows.into_iter().map(|(id,)| id).collect();
    remove_files(dir, &ids).await;
    Ok(ids.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guardians_get_the_larger_limit() {
        const MB: u64 = 1024 * 1024;
        for level in 0..=2 {
            assert_eq!(pick_limit(level, 5 * MB, 30 * MB), 5 * MB);
        }
        assert_eq!(pick_limit(3, 5 * MB, 30 * MB), 30 * MB);
    }

    #[test]
    fn filenames_lose_paths_and_control_chars() {
        assert_eq!(clean_filename("../../etc/passwd"), "passwd");
        assert_eq!(clean_filename(r"C:\Users\me\报告.pdf"), "报告.pdf");
        assert_eq!(clean_filename("a\r\nb.txt"), "ab.txt");
        assert_eq!(clean_filename("  "), "file");
        assert_eq!(clean_filename(""), "file");
        let long = format!("{}.tar.gz", "名".repeat(300));
        let cleaned = clean_filename(&long);
        assert_eq!(cleaned.chars().count(), 120);
        assert!(cleaned.ends_with(".gz"));
    }

    #[test]
    fn content_disposition_is_header_safe() {
        let v = content_disposition("报告 \"终稿\".pdf");
        assert!(v.starts_with("attachment; filename=\""));
        assert!(v.contains("filename*=UTF-8''%E6%8A%A5%E5%91%8A%20%22%E7%BB%88%E7%A8%BF%22.pdf"));
        assert!(HeaderValue::from_str(&v).is_ok());
    }
}
