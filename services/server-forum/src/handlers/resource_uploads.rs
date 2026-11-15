//! Direct user uploads: the forum authorizes metadata; bytes go to the resource host.
use super::attachments::{clean_filename, resource_extension, upload_limit};
use crate::{auth::AuthUser, content, error::{AppError, AppResult}, state::AppState, users};
use axum::{extract::{Path, State}, Json};
use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct TicketInput { kind: String, name: String, mime: String, size: i64, sha256: String }
#[derive(sqlx::FromRow)]
struct TicketRow { kind: String, filename: String, mime: String, size: i64, sha256: String, expires_at: DateTime<Utc>, used: bool }

fn extension(kind: &str, mime: &str, filename: &str) -> Option<String> {
    match (kind, mime) {
        ("image", "image/png") => Some("png".into()),
        ("image", "image/jpeg") => Some("jpg".into()),
        ("image", "image/webp") => Some("webp".into()),
        ("image", "image/gif") => Some("gif".into()),
        ("attachment", "application/octet-stream") => Some(resource_extension(filename)),
        _ => None,
    }
}

pub async fn ticket(State(state): State<AppState>, auth: AuthUser, Json(input): Json<TicketInput>) -> AppResult<Json<Value>> {
    users::ensure_can_write(&auth.user)?;
    let base = state.config.resource_base_url.as_ref().ok_or_else(|| AppError::ServiceUnavailable("资源机暂不可用".into()))?;
    let name = clean_filename(&input.name);
    let mime = if input.kind == "attachment" { "application/octet-stream" } else { input.mime.as_str() };
    let ext = extension(&input.kind, mime, &name).ok_or_else(|| AppError::BadRequest("资源类型不支持".into()))?;
    let limit = match input.kind.as_str() {
        "image" => state.config.upload_limit_image.min(10 * 1024 * 1024),
        "attachment" => upload_limit(&state.config, auth.user.trust_level).min(32 * 1024 * 1024),
        _ => return Err(AppError::BadRequest("资源类型不支持".into())),
    };
    if input.size <= 0 || input.size as u64 > limit || !content::valid_sha256(&input.sha256) {
        return Err(AppError::BadRequest("文件大小或校验值无效".into()));
    }
    let mut tx = state.pool.begin().await?;
    sqlx::query("SELECT uid FROM users WHERE uid=$1 FOR UPDATE").bind(auth.user.uid).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM forum_direct_uploads WHERE expires_at < now() - interval '1 day'").execute(&mut *tx).await?;
    let (count,): (i64,) = if input.kind == "image" {
        sqlx::query_as("SELECT (SELECT count(*) FROM images WHERE uploader_uid=$1 AND created_at > now() - interval '1 day') + (SELECT count(*) FROM forum_direct_uploads WHERE uid=$1 AND kind='image' AND NOT used AND created_at > now() - interval '1 day')")
            .bind(auth.user.uid).fetch_one(&mut *tx).await?
    } else {
        sqlx::query_as("SELECT (SELECT count(*) FROM attachments WHERE uploader_uid=$1 AND topic_id IS NULL) + (SELECT count(*) FROM forum_direct_uploads WHERE uid=$1 AND kind='attachment' AND NOT used AND created_at > now() - interval '1 day')")
            .bind(auth.user.uid).fetch_one(&mut *tx).await?
    };
    if input.kind == "image" && count >= 50 {
        return Err(AppError::TooManyRequests { message:"今天上传的图片已达上限（50 张）".into(), retry_after:3600 });
    }
    if input.kind == "attachment" && count >= 10 {
        return Err(AppError::BadRequest("有太多还没挂到帖子上的附件，请先发布帖子或删掉一些".into()));
    }
    let id = Uuid::new_v4();
    let expires_at = Utc::now() + Duration::seconds(240);
    sqlx::query("INSERT INTO forum_direct_uploads(id,uid,kind,filename,mime,size,sha256,expires_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8)")
        .bind(id).bind(auth.user.uid).bind(&input.kind).bind(&name).bind(mime).bind(input.size).bind(&input.sha256).bind(expires_at)
        .execute(&mut *tx).await?;
    tx.commit().await?;
    let resource_kind = if input.kind == "image" { "images" } else { "attachments" };
    let mut message = format!("POST\n{resource_kind}/{id}\n{}\n{}\n{}", input.sha256, input.size, expires_at.timestamp());
    let path = if input.kind == "image" { format!("/internal/forum-image/images/{id}") } else {
        message.push_str(&format!("\n{ext}")); format!("/internal/forum-file/{id}")
    };
    let signature = content::sign_callback(message.as_bytes(), &state.config.resource_secret);
    let mut headers = json!({"Content-Type":mime,"X-Content-Sha256":input.sha256,"X-Expiration":expires_at.timestamp().to_string(),"X-Signature":signature});
    if input.kind == "attachment" { headers["X-File-Ext"] = json!(ext); }
    Ok(Json(json!({"id":id,"upload_url":format!("{base}{path}"),"headers":headers,"expires_at":expires_at})))
}

#[derive(Deserialize)]
pub struct Receipt { object_key: String, sha256: String, size: i64, signature: String }

pub async fn complete(State(state): State<AppState>, auth: AuthUser, Path(id): Path<Uuid>, Json(receipt): Json<Receipt>) -> AppResult<Json<Value>> {
    users::ensure_can_write(&auth.user)?;
    let mut tx = state.pool.begin().await?;
    let row: TicketRow = sqlx::query_as("SELECT kind,filename,mime,size,sha256,expires_at,used FROM forum_direct_uploads WHERE id=$1 AND uid=$2 FOR UPDATE")
        .bind(id).bind(auth.user.uid).fetch_optional(&mut *tx).await?.ok_or_else(|| AppError::not_found("上传凭证"))?;
    if row.expires_at + Duration::minutes(10) < Utc::now() { return Err(AppError::Conflict("上传凭证已过期，请重新上传".into())); }
    let ext = extension(&row.kind, &row.mime, &row.filename).ok_or_else(|| AppError::BadRequest("资源类型无效".into()))?;
    let object = receipt.object_key.strip_prefix("forum-content/objects/").unwrap_or("");
    let valid_object = object.split_once('.').is_some_and(|(md5, suffix)| md5.len()==32 &&
        md5.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) && suffix==ext);
    let resource_kind = if row.kind == "image" { "images" } else { "attachments" };
    let message = format!("RECEIPT\n{resource_kind}/{id}\n{}\n{}\n{}", row.sha256, row.size, receipt.object_key);
    if !valid_object || receipt.sha256 != row.sha256 || receipt.size != row.size ||
        !content::verify_callback(message.as_bytes(), &receipt.signature, &state.config.resource_secret) {
        return Err(AppError::Forbidden("资源机回执无效".into()));
    }
    if !row.used {
        if row.kind == "image" {
            sqlx::query("INSERT INTO images(id,uploader_uid,mime,size) VALUES($1,$2,$3,$4)")
                .bind(id).bind(auth.user.uid).bind(&row.mime).bind(row.size).execute(&mut *tx).await?;
        } else {
            sqlx::query("INSERT INTO attachments(id,uploader_uid,filename,size) VALUES($1,$2,$3,$4)")
                .bind(id).bind(auth.user.uid).bind(&row.filename).bind(row.size).execute(&mut *tx).await?;
        }
        sqlx::query("UPDATE forum_direct_uploads SET used=true WHERE id=$1").bind(id).execute(&mut *tx).await?;
    } else {
        let exists: (bool,) = if row.kind == "image" {
            sqlx::query_as("SELECT EXISTS(SELECT 1 FROM images WHERE id=$1 AND uploader_uid=$2)").bind(id).bind(auth.user.uid).fetch_one(&mut *tx).await?
        } else {
            sqlx::query_as("SELECT EXISTS(SELECT 1 FROM attachments WHERE id=$1 AND uploader_uid=$2)").bind(id).bind(auth.user.uid).fetch_one(&mut *tx).await?
        };
        if !exists.0 { return Err(AppError::not_found("上传文件")); }
    }
    tx.commit().await?;
    if row.kind == "image" {
        Ok(Json(json!({"id":id,"url":format!("/api/images/{id}"),"size":row.size})))
    } else {
        Ok(Json(json!({"id":id,"name":row.filename,"size":row.size,"price":0,"purchased":true})))
    }
}
