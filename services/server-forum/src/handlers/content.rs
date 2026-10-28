//! 社区内容层接口：内容包（浏览、下载、发布、版本）、方案、项目、周刊、分享，以及后台审核与资源机回调。
//!
//! 文件本体不经过本服务：下载返回资源机的签名地址，上传由客户端拿凭证直传资源机，
//! 资源机校验哈希后回调 `/api/internal/pack-uploaded`，发布内容包时只认已确认的上传。

use crate::{
    auth::{AuthUser, MaybeUser},
    content::{self, UploadClaims},
    error::{AppError, AppResult},
    handlers::admin::{audit, AuthAdmin},
    markdown,
    models::Page,
    state::AppState,
};
use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::HeaderMap,
    routing::{get, post, put},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/packs", get(list_packs).post(create_pack))
        .route("/packs/upload-ticket", post(upload_ticket))
        .route("/packs/{id}", get(get_pack))
        .route("/packs/{id}/download", get(download))
        .route("/packs/{id}/applied", post(applied))
        .route("/packs/{id}/versions", post(add_version))
        .route("/packs/{id}/remove", post(remove_own_pack))
        .route("/me/packs", get(my_packs))
        .route("/schemes", get(list_schemes).post(create_scheme))
        .route("/projects", get(list_projects))
        .route("/projects/{slug}", get(get_project))
        .route("/weekly", get(list_weekly))
        .route("/weekly/{week}", get(get_weekly))
        .route("/share/{kind}/{id}", get(share))
        .route("/internal/pack-uploaded", post(pack_uploaded))
}

pub fn admin_routes() -> Router<AppState> {
    Router::new()
        .route("/packs", get(admin_list_packs).post(admin_create_pack))
        .route("/packs/{id}/review", post(admin_review_pack))
        .route("/packs/{id}/mirrors", post(admin_add_mirror))
        .route("/moderation", get(admin_moderation))
        .route("/projects", get(admin_list_projects))
        .route("/projects/{slug}", put(admin_put_project))
        .route("/weekly", get(admin_list_weekly))
        .route("/weekly/{week}", put(admin_put_weekly))
}

fn require_resource(state: &AppState) -> AppResult<&str> {
    state
        .config
        .resource_base_url
        .as_deref()
        .ok_or_else(|| AppError::ServiceUnavailable("资源服务器尚未接入".into()))
}

#[derive(Deserialize)]
pub struct ListQuery {
    kind: Option<String>,
    q: Option<String>,
    tag: Option<String>,
    author: Option<Uuid>,
    /// `new`（默认）或 `hot`。
    sort: Option<String>,
    page: Option<i64>,
    page_size: Option<i64>,
}

fn paging(page: Option<i64>, size: Option<i64>) -> (i64, i64) {
    (page.unwrap_or(1).max(1), size.unwrap_or(20).clamp(1, 50))
}

/// 搜索词里的通配符按字面匹配。
fn like_pattern(q: &str) -> String {
    let escaped: String = q.chars().flat_map(|c| if matches!(c, '%' | '_' | '\\') { vec!['\\', c] } else { vec![c] }).collect();
    format!("%{escaped}%")
}

const PACK_COLUMNS: &str = "p.id, p.kind, p.slug, p.title, p.summary, p.tags, p.author_uid, u.nickname AS author_name,
    p.trust_level, p.status, p.current_version, p.cover_key, p.apply_count, p.topic_id, p.created_at, p.updated_at";

#[derive(sqlx::FromRow)]
struct PackRow {
    id: i64,
    kind: String,
    slug: String,
    title: String,
    summary: String,
    tags: Vec<String>,
    author_uid: Option<Uuid>,
    author_name: Option<String>,
    trust_level: i16,
    status: String,
    current_version: i32,
    cover_key: Option<String>,
    apply_count: i64,
    topic_id: Option<i64>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl PackRow {
    fn json(&self, state: &AppState) -> Value {
        json!({
            "id": self.id,
            "kind": self.kind,
            "slug": self.slug,
            "title": self.title,
            "summary": self.summary,
            "tags": self.tags,
            "author": self.author_uid.map(|uid| json!({ "uid": uid, "nickname": self.author_name })),
            "trust_level": self.trust_level,
            "status": self.status,
            "version": self.current_version,
            "cover_url": cover_url(state, self.cover_key.as_deref()),
            "apply_count": self.apply_count,
            "topic_id": self.topic_id,
            "one_click": content::one_click_allowed(&self.kind, self.trust_level, &self.status),
            "pure_data": content::is_pure_data(&self.kind),
            "created_at": self.created_at,
            "updated_at": self.updated_at,
        })
    }
}

/// 封面是公开的静态图，不签名；资源机未接入时不返回地址。
fn cover_url(state: &AppState, key: Option<&str>) -> Option<String> {
    let base = state.config.resource_base_url.as_deref()?;
    Some(format!("{base}/{}", key?))
}

async fn list_packs(State(state): State<AppState>, Query(q): Query<ListQuery>) -> AppResult<Json<Page<Value>>> {
    let (page, size) = paging(q.page, q.page_size);
    if let Some(kind) = &q.kind {
        if !content::is_known_kind(kind) {
            return Err(AppError::BadRequest("未知的内容类型".into()));
        }
    }
    let pattern = q.q.as_deref().map(str::trim).filter(|s| !s.is_empty()).map(like_pattern);
    let order = if q.sort.as_deref() == Some("hot") { "p.apply_count DESC, p.id DESC" } else { "p.updated_at DESC, p.id DESC" };
    let filter = "p.status = 'published'
        AND ($1::text IS NULL OR p.kind = $1)
        AND ($2::text IS NULL OR p.title ILIKE $2 OR p.summary ILIKE $2)
        AND ($3::text IS NULL OR $3 = ANY(p.tags))
        AND ($4::uuid IS NULL OR p.author_uid = $4)";
    let (total,): (i64,) = sqlx::query_as(&format!("SELECT count(*) FROM packs p WHERE {filter}"))
        .bind(&q.kind)
        .bind(&pattern)
        .bind(&q.tag)
        .bind(q.author)
        .fetch_one(&state.pool)
        .await?;
    let rows: Vec<PackRow> = sqlx::query_as(&format!(
        "SELECT {PACK_COLUMNS} FROM packs p LEFT JOIN users u ON u.uid = p.author_uid
         WHERE {filter} ORDER BY {order} LIMIT $5 OFFSET $6"
    ))
    .bind(&q.kind)
    .bind(&pattern)
    .bind(&q.tag)
    .bind(q.author)
    .bind(size)
    .bind((page - 1) * size)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(Page { items: rows.iter().map(|r| r.json(&state)).collect(), page, page_size: size, total }))
}

async fn find_pack(state: &AppState, id: i64) -> AppResult<PackRow> {
    sqlx::query_as(&format!("SELECT {PACK_COLUMNS} FROM packs p LEFT JOIN users u ON u.uid = p.author_uid WHERE p.id = $1"))
        .bind(id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::not_found("内容包"))
}

/// 已下架或被拒的内容包只有作者本人能看到。
fn visible(pack: &PackRow, viewer: Option<Uuid>) -> bool {
    pack.status == "published" || (viewer.is_some() && viewer == pack.author_uid)
}

async fn get_pack(State(state): State<AppState>, MaybeUser(user): MaybeUser, Path(id): Path<i64>) -> AppResult<Json<Value>> {
    let pack = find_pack(&state, id).await?;
    if !visible(&pack, user.as_ref().map(|u| u.uid)) {
        return Err(AppError::not_found("内容包"));
    }
    let versions: Vec<(i32, String, String, i64, Value, String, DateTime<Utc>)> = sqlx::query_as(
        "SELECT version, label, sha256, size, manifest, changelog, created_at FROM pack_versions WHERE pack_id = $1 ORDER BY version DESC",
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await?;
    let versions: Vec<Value> = versions
        .into_iter()
        .map(|(version, label, sha256, size, manifest, changelog, created_at)| {
            json!({ "version": version, "label": label, "sha256": sha256, "size": size, "manifest": manifest, "changelog": changelog, "created_at": created_at })
        })
        .collect();
    let mut body = pack.json(&state);
    body["versions"] = json!(versions);
    Ok(Json(body))
}

#[derive(Deserialize)]
pub struct DownloadQuery {
    version: Option<i32>,
}

async fn download(
    State(state): State<AppState>,
    MaybeUser(user): MaybeUser,
    Path(id): Path<i64>,
    Query(q): Query<DownloadQuery>,
) -> AppResult<Json<Value>> {
    let base = require_resource(&state)?;
    let pack = find_pack(&state, id).await?;
    if !visible(&pack, user.as_ref().map(|u| u.uid)) {
        return Err(AppError::not_found("内容包"));
    }
    let version = q.version.unwrap_or(pack.current_version);
    let (label, sha256, size, storage, file_key, mirrors): (String, String, i64, String, String, Value) = sqlx::query_as(
        "SELECT label, sha256, size, storage, file_key, mirrors FROM pack_versions WHERE pack_id = $1 AND version = $2",
    )
    .bind(id)
    .bind(version)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("版本"))?;
    let exp = Utc::now().timestamp() + state.config.download_url_ttl_secs;
    // 官方镜像（如魔搭）是公开地址，由客户端在主地址失败时按顺序回退；每个来源都要过同一个 sha256 校验。
    Ok(Json(json!({
        "pack_id": id,
        "kind": pack.kind,
        "version": version,
        "label": label,
        "sha256": sha256,
        "size": size,
        "storage": storage,
        "url": content::signed_download_url(base, &file_key, exp, &state.config.resource_secret),
        "expires_at": exp,
        "mirrors": mirrors,
        "trust_level": pack.trust_level,
        "one_click": content::one_click_allowed(&pack.kind, pack.trust_level, &pack.status),
    })))
}

#[derive(Deserialize)]
pub struct AppliedInput {
    version: Option<i32>,
}

async fn applied(State(state): State<AppState>, Path(id): Path<i64>, body: Option<Json<AppliedInput>>) -> AppResult<Json<Value>> {
    let _ = body.and_then(|Json(b)| b.version);
    let mut tx = state.pool.begin().await?;
    let updated = sqlx::query("UPDATE packs SET apply_count = apply_count + 1 WHERE id = $1 AND status = 'published'")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    if updated.rows_affected() == 0 {
        return Err(AppError::not_found("内容包"));
    }
    sqlx::query(
        "INSERT INTO pack_applies (pack_id, day, count) VALUES ($1, CURRENT_DATE, 1)
         ON CONFLICT (pack_id, day) DO UPDATE SET count = pack_applies.count + 1",
    )
    .bind(id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Json(json!({ "ok": true })))
}

// ───────────────────────── 上传 ─────────────────────────

#[derive(Deserialize)]
pub struct TicketInput {
    kind: String,
    size: u64,
    sha256: String,
}

async fn upload_ticket(State(state): State<AppState>, auth: AuthUser, Json(input): Json<TicketInput>) -> AppResult<Json<Value>> {
    let base = require_resource(&state)?;
    let limit = content::upload_limit(&input.kind).ok_or_else(|| AppError::BadRequest("未知的内容类型".into()))?;
    if input.size == 0 || input.size > limit {
        return Err(AppError::BadRequest(format!("该类型的内容包不能超过 {} MB", limit / 1024 / 1024)));
    }
    if !content::valid_sha256(&input.sha256) {
        return Err(AppError::BadRequest("sha256 需要是 64 位小写十六进制".into()));
    }
    if input.kind == content::COVER {
        check_cover_quota(&state, &auth).await?;
    } else {
        check_pack_quota(&state, &auth).await?;
    }

    let file_key = if input.kind == content::COVER {
        format!("covers/{}", Uuid::new_v4().simple())
    } else {
        format!("packs/{}/{}.zip", input.kind, Uuid::new_v4().simple())
    };
    let exp = Utc::now().timestamp() + state.config.upload_ticket_ttl_secs;
    sqlx::query("INSERT INTO pack_uploads (file_key, uid, kind, size, sha256) VALUES ($1, $2, $3, $4, $5)")
        .bind(&file_key)
        .bind(auth.user.uid)
        .bind(&input.kind)
        .bind(input.size as i64)
        .bind(&input.sha256)
        .execute(&state.pool)
        .await?;
    let ticket = content::issue_upload_ticket(
        &UploadClaims {
            uid: auth.user.uid.to_string(),
            kind: input.kind,
            size: input.size,
            sha256: input.sha256,
            file_key: file_key.clone(),
            exp,
        },
        &state.config.resource_secret,
    );
    Ok(Json(json!({
        "ticket": ticket,
        "upload_url": format!("{base}/upload"),
        "file_key": file_key,
        "expires_at": exp,
    })))
}

/// 普通用户每天的发布数量上限（官方与管理员走后台，不受限制）。
async fn check_pack_quota(state: &AppState, auth: &AuthUser) -> AppResult<()> {
    let (count,): (i64,) = sqlx::query_as(
        "SELECT (SELECT count(*) FROM packs WHERE author_uid = $1 AND created_at > now() - interval '1 day')
              + (SELECT count(*) FROM pack_uploads WHERE uid = $1 AND kind <> 'cover' AND used = false AND created_at > now() - interval '1 day')",
    )
    .bind(auth.user.uid)
    .fetch_one(&state.pool)
    .await?;
    if count >= state.config.daily_pack_publishes {
        return Err(AppError::TooManyRequests { message: "今天发布得有点多，明天再来".into(), retry_after: 3600 });
    }
    Ok(())
}

async fn check_cover_quota(state: &AppState, auth: &AuthUser) -> AppResult<()> {
    let (count,): (i64,) =
        sqlx::query_as("SELECT count(*) FROM pack_uploads WHERE uid = $1 AND kind = 'cover' AND created_at > now() - interval '1 day'")
            .bind(auth.user.uid)
            .fetch_one(&state.pool)
            .await?;
    if count >= state.config.daily_pack_publishes * 4 {
        return Err(AppError::TooManyRequests { message: "今天上传的图片有点多，明天再来".into(), retry_after: 3600 });
    }
    Ok(())
}

#[derive(Deserialize)]
struct UploadedCallback {
    file_key: String,
    size: u64,
    sha256: String,
    /// 资源机从压缩包根目录读出的 `manifest.json`；内容包必带，封面没有。发布时以这里为准，不信客户端自己报的。
    manifest: Option<Value>,
}

/// 资源机在文件落盘并重算哈希后回调；请求体由 `X-Signature`（HMAC-SHA256）保护。
async fn pack_uploaded(State(state): State<AppState>, headers: HeaderMap, body: Bytes) -> AppResult<Json<Value>> {
    let signature = headers.get("x-signature").and_then(|v| v.to_str().ok()).unwrap_or("");
    if !content::verify_callback(&body, signature, &state.config.resource_secret) {
        return Err(AppError::Unauthorized("签名无效".into()));
    }
    let cb: UploadedCallback = serde_json::from_slice(&body).map_err(|_| AppError::BadRequest("回调内容无效".into()))?;
    let kind: Option<(String,)> = sqlx::query_as("SELECT kind FROM pack_uploads WHERE file_key = $1")
        .bind(&cb.file_key)
        .fetch_optional(&state.pool)
        .await?;
    let (kind,) = kind.ok_or_else(|| AppError::Conflict("没有匹配的上传记录".into()))?;
    if kind != content::COVER {
        let manifest = cb.manifest.as_ref().ok_or_else(|| AppError::BadRequest("缺少 manifest".into()))?;
        content::validate_manifest(manifest, &kind).map_err(AppError::BadRequest)?;
    }
    let updated = sqlx::query(
        "UPDATE pack_uploads SET verified = true, manifest = $4 WHERE file_key = $1 AND size = $2 AND sha256 = $3",
    )
    .bind(&cb.file_key)
    .bind(cb.size as i64)
    .bind(&cb.sha256)
    .bind(&cb.manifest)
    .execute(&state.pool)
    .await?;
    if updated.rows_affected() == 0 {
        return Err(AppError::Conflict("没有匹配的上传记录，或文件大小、哈希与凭证不一致".into()));
    }
    Ok(Json(json!({ "ok": true })))
}

// ───────────────────────── 发布与版本 ─────────────────────────

#[derive(Deserialize)]
pub struct CreatePackInput {
    kind: String,
    title: String,
    #[serde(default)]
    summary: String,
    #[serde(default)]
    tags: Vec<String>,
    file_key: String,
    cover_key: Option<String>,
    /// 发布者确认自己有权发布这份内容（原创或已获授权），不勾选不能发布。
    #[serde(default)]
    rights_ack: bool,
}

fn clean_tags(tags: &[String]) -> AppResult<Vec<String>> {
    let mut out: Vec<String> = Vec::new();
    for tag in tags.iter().map(|t| t.trim()).filter(|t| !t.is_empty()) {
        if tag.chars().count() > 20 {
            return Err(AppError::BadRequest("单个标签最长 20 个字".into()));
        }
        if !out.iter().any(|t| t == tag) {
            out.push(tag.to_string());
        }
    }
    if out.len() > 8 {
        return Err(AppError::BadRequest("最多 8 个标签".into()));
    }
    Ok(out)
}

fn clean_text(value: &str, what: &str, max: usize, required: bool) -> AppResult<String> {
    let value = value.trim();
    if (required && value.is_empty()) || value.chars().count() > max {
        return Err(AppError::BadRequest(format!("{what}{}，最长 {max} 个字", if required { "必填" } else { "" })));
    }
    Ok(value.to_string())
}

/// 取走一条属于该用户、已被资源机确认且没用过的上传记录，返回大小、哈希和资源机读出的清单。
async fn claim_upload(
    tx: &mut sqlx::PgConnection,
    uid: Uuid,
    kind: &str,
    file_key: &str,
) -> AppResult<(i64, String, Value)> {
    let row: Option<(i64, String, Option<Value>, bool, bool)> = sqlx::query_as(
        "SELECT size, sha256, manifest, verified, used FROM pack_uploads WHERE file_key = $1 AND uid = $2 AND kind = $3 FOR UPDATE",
    )
    .bind(file_key)
    .bind(uid)
    .bind(kind)
    .fetch_optional(&mut *tx)
    .await?;
    let (size, sha256, manifest, verified, used) = row.ok_or_else(|| AppError::BadRequest("没有找到这次上传，请重新上传".into()))?;
    if !verified {
        return Err(AppError::Conflict("文件还没有通过资源服务器校验，请稍后再试或重新上传".into()));
    }
    if used {
        return Err(AppError::Conflict("这个文件已经用于发布".into()));
    }
    sqlx::query("UPDATE pack_uploads SET used = true WHERE file_key = $1").bind(file_key).execute(&mut *tx).await?;
    Ok((size, sha256, manifest.unwrap_or(Value::Null)))
}

/// 封面必须是自己上传并已被资源机确认的图片。
async fn check_cover(tx: &mut sqlx::PgConnection, uid: Uuid, cover_key: &str) -> AppResult<()> {
    let ok: Option<(bool,)> = sqlx::query_as("SELECT verified FROM pack_uploads WHERE file_key = $1 AND uid = $2 AND kind = 'cover'")
        .bind(cover_key)
        .bind(uid)
        .fetch_optional(&mut *tx)
        .await?;
    if ok != Some((true,)) {
        return Err(AppError::BadRequest("封面还没有上传完成，请重新上传".into()));
    }
    Ok(())
}

async fn create_pack(State(state): State<AppState>, auth: AuthUser, Json(input): Json<CreatePackInput>) -> AppResult<Json<Value>> {
    if !input.rights_ack {
        return Err(AppError::BadRequest("请确认你拥有这份内容的发布权利".into()));
    }
    check_pack_quota_for_publish(&state, &auth).await?;
    if !content::is_known_kind(&input.kind) {
        return Err(AppError::BadRequest("未知的内容类型".into()));
    }
    let title = clean_text(&input.title, "标题", 60, true)?;
    let summary = clean_text(&input.summary, "简介", 500, false)?;
    let tags = clean_tags(&input.tags)?;
    let mut tx = state.pool.begin().await?;
    if let Some(cover) = &input.cover_key {
        check_cover(&mut tx, auth.user.uid, cover).await?;
    }
    let (size, sha256, manifest) = claim_upload(&mut tx, auth.user.uid, &input.kind, &input.file_key).await?;
    let info = content::validate_manifest(&manifest, &input.kind).map_err(AppError::BadRequest)?;
    let (id,): (i64,) = sqlx::query_as(
        "INSERT INTO packs (kind, slug, title, summary, tags, author_uid, trust_level, status, cover_key)
         VALUES ($1, $2, $3, $4, $5, $6, 2, 'published', $7) RETURNING id",
    )
    .bind(&input.kind)
    .bind(&info.slug)
    .bind(&title)
    .bind(&summary)
    .bind(&tags)
    .bind(auth.user.uid)
    .bind(&input.cover_key)
    .fetch_one(&mut *tx)
    .await
    .map_err(slug_conflict)?;
    insert_version(&mut tx, id, 1, &info.version_label, &sha256, size, &manifest, &input.file_key, &json!([]), "").await?;
    // 用户发布的内容先上架（未审标记），同时进审核队列，由人工看过后升级为“已审核”。
    sqlx::query("INSERT INTO moderation_queue (target_type, target_id) VALUES ('pack', $1)").bind(id).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(Json(json!({ "id": id, "slug": info.slug, "version": 1, "trust_level": content::T2, "status": "published" })))
}

/// 发布前再确认一次配额：凭证签发时已经计过一次占位，这里只拦“已发布数”超限。
async fn check_pack_quota_for_publish(state: &AppState, auth: &AuthUser) -> AppResult<()> {
    let (count,): (i64,) =
        sqlx::query_as("SELECT count(*) FROM packs WHERE author_uid = $1 AND created_at > now() - interval '1 day'")
            .bind(auth.user.uid)
            .fetch_one(&state.pool)
            .await?;
    if count >= state.config.daily_pack_publishes {
        return Err(AppError::TooManyRequests { message: "今天发布得有点多，明天再来".into(), retry_after: 3600 });
    }
    Ok(())
}

fn slug_conflict(err: sqlx::Error) -> AppError {
    match &err {
        sqlx::Error::Database(db) if db.is_unique_violation() => AppError::Conflict("这个 id 已被占用，请换一个".into()),
        _ => AppError::Database(err),
    }
}

#[allow(clippy::too_many_arguments)]
async fn insert_version(
    tx: &mut sqlx::PgConnection,
    pack_id: i64,
    version: i32,
    label: &str,
    sha256: &str,
    size: i64,
    manifest: &Value,
    file_key: &str,
    mirrors: &Value,
    changelog: &str,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO pack_versions (pack_id, version, label, sha256, size, manifest, file_key, mirrors, changelog)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    )
    .bind(pack_id)
    .bind(version)
    .bind(label)
    .bind(sha256)
    .bind(size)
    .bind(manifest)
    .bind(file_key)
    .bind(mirrors)
    .bind(changelog)
    .execute(&mut *tx)
    .await?;
    Ok(())
}

#[derive(Deserialize)]
pub struct AddVersionInput {
    file_key: String,
    #[serde(default)]
    changelog: String,
}

async fn add_version(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i64>,
    Json(input): Json<AddVersionInput>,
) -> AppResult<Json<Value>> {
    let changelog = clean_text(&input.changelog, "更新说明", 500, false)?;
    let mut tx = state.pool.begin().await?;
    let (kind, slug, author, current): (String, String, Option<Uuid>, i32) =
        sqlx::query_as("SELECT kind, slug, author_uid, current_version FROM packs WHERE id = $1 AND status <> 'removed' FOR UPDATE")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| AppError::not_found("内容包"))?;
    if author != Some(auth.user.uid) {
        return Err(AppError::Forbidden("只有作者可以更新自己的内容包".into()));
    }
    let (size, sha256, manifest) = claim_upload(&mut tx, auth.user.uid, &kind, &input.file_key).await?;
    let info = content::validate_manifest(&manifest, &kind).map_err(AppError::BadRequest)?;
    if info.slug != slug {
        return Err(AppError::BadRequest("新版本的 id 必须与原来一致".into()));
    }
    let next = current + 1;
    insert_version(&mut tx, id, next, &info.version_label, &sha256, size, &manifest, &input.file_key, &json!([]), &changelog)
        .await
        .map_err(|e| match e {
            AppError::Database(sqlx::Error::Database(db)) if db.is_unique_violation() => AppError::Conflict("版本冲突，请重试".into()),
            other => other,
        })?;
    // 新版本内容变了：已审核的降回未审并重新入队，避免“审过旧版再换内容”。
    sqlx::query("UPDATE packs SET current_version = $2, trust_level = GREATEST(trust_level, 2), updated_at = now() WHERE id = $1")
        .bind(id)
        .bind(next)
        .execute(&mut *tx)
        .await?;
    sqlx::query("INSERT INTO moderation_queue (target_type, target_id) VALUES ('pack', $1) ON CONFLICT DO NOTHING")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(Json(json!({ "id": id, "version": next, "trust_level": content::T2 })))
}

/// 我的作品：自己发布的全部内容包，含已下架和被拒的，附未处理的审核状态。
async fn my_packs(State(state): State<AppState>, auth: AuthUser) -> AppResult<Json<Value>> {
    let rows: Vec<PackRow> = sqlx::query_as(&format!(
        "SELECT {PACK_COLUMNS} FROM packs p LEFT JOIN users u ON u.uid = p.author_uid
         WHERE p.author_uid = $1 ORDER BY p.updated_at DESC, p.id DESC LIMIT 200"
    ))
    .bind(auth.user.uid)
    .fetch_all(&state.pool)
    .await?;
    let pending: Vec<(i64,)> =
        sqlx::query_as("SELECT target_id FROM moderation_queue WHERE target_type = 'pack' AND status = 'pending'")
            .fetch_all(&state.pool)
            .await?;
    let pending: std::collections::HashSet<i64> = pending.into_iter().map(|r| r.0).collect();
    let items: Vec<Value> = rows
        .iter()
        .map(|r| {
            let mut v = r.json(&state);
            v["in_review"] = json!(pending.contains(&r.id));
            v
        })
        .collect();
    Ok(Json(json!({ "items": items })))
}

/// 作者自己下架：只改状态，不删数据，已应用的人不受影响。
async fn remove_own_pack(State(state): State<AppState>, auth: AuthUser, Path(id): Path<i64>) -> AppResult<Json<Value>> {
    let updated = sqlx::query(
        "UPDATE packs SET status = 'removed', updated_at = now() WHERE id = $1 AND author_uid = $2 AND status <> 'removed'",
    )
    .bind(id)
    .bind(auth.user.uid)
    .execute(&state.pool)
    .await?;
    if updated.rows_affected() == 0 {
        return Err(AppError::not_found("内容包"));
    }
    Ok(Json(json!({ "id": id, "status": "removed" })))
}

// ───────────────────────── 方案 ─────────────────────────

async fn list_schemes(State(state): State<AppState>, Query(q): Query<ListQuery>) -> AppResult<Json<Page<Value>>> {
    let (page, size) = paging(q.page, q.page_size);
    let (total,): (i64,) = sqlx::query_as("SELECT count(*) FROM schemes WHERE status = 'published'")
        .fetch_one(&state.pool)
        .await?;
    let rows: Vec<(i64, Uuid, String, String, Value, i64, DateTime<Utc>)> = sqlx::query_as(
        "SELECT s.id, s.author_uid, u.nickname, s.title, s.items, s.apply_count, s.created_at
         FROM schemes s JOIN users u ON u.uid = s.author_uid
         WHERE s.status = 'published' ORDER BY s.created_at DESC, s.id DESC LIMIT $1 OFFSET $2",
    )
    .bind(size)
    .bind((page - 1) * size)
    .fetch_all(&state.pool)
    .await
    .map(|rows: Vec<(i64, Uuid, String, String, Value, i64, DateTime<Utc>)>| rows)?;
    let items = rows
        .into_iter()
        .map(|(id, uid, nickname, title, items, apply_count, created_at)| {
            json!({ "id": id, "author": { "uid": uid, "nickname": nickname }, "title": title, "items": items,
                    "apply_count": apply_count, "created_at": created_at })
        })
        .collect();
    Ok(Json(Page { items, page, page_size: size, total }))
}

#[derive(Deserialize)]
pub struct SchemeItem {
    pack_id: i64,
    version: Option<i32>,
}

#[derive(Deserialize)]
pub struct CreateSchemeInput {
    title: String,
    #[serde(default)]
    summary: String,
    items: Vec<SchemeItem>,
}

/// 方案是一组内容包的组合，只存引用（包 id 与版本），应用时客户端逐个按各自的信任级别处理。
async fn create_scheme(State(state): State<AppState>, auth: AuthUser, Json(input): Json<CreateSchemeInput>) -> AppResult<Json<Value>> {
    let title = clean_text(&input.title, "标题", 60, true)?;
    let summary = clean_text(&input.summary, "简介", 500, false)?;
    if input.items.is_empty() || input.items.len() > 20 {
        return Err(AppError::BadRequest("方案需要包含 1～20 个内容".into()));
    }
    let (today,): (i64,) = sqlx::query_as("SELECT count(*) FROM schemes WHERE author_uid = $1 AND created_at > now() - interval '1 day'")
        .bind(auth.user.uid)
        .fetch_one(&state.pool)
        .await?;
    if today >= state.config.daily_pack_publishes * 2 {
        return Err(AppError::TooManyRequests { message: "今天分享得有点多，明天再来".into(), retry_after: 3600 });
    }
    let mut resolved = Vec::with_capacity(input.items.len());
    for item in &input.items {
        let row: Option<(String, String, i32)> =
            sqlx::query_as("SELECT kind, slug, current_version FROM packs WHERE id = $1 AND status = 'published'")
                .bind(item.pack_id)
                .fetch_optional(&state.pool)
                .await?;
        let (kind, slug, current) = row.ok_or_else(|| AppError::BadRequest(format!("内容 {} 不存在或已下架", item.pack_id)))?;
        resolved.push(json!({ "pack_id": item.pack_id, "kind": kind, "slug": slug, "version": item.version.unwrap_or(current) }));
    }
    let (id,): (i64,) = sqlx::query_as("INSERT INTO schemes (author_uid, title, summary, items) VALUES ($1, $2, $3, $4) RETURNING id")
        .bind(auth.user.uid)
        .bind(&title)
        .bind(&summary)
        .bind(Value::Array(resolved))
        .fetch_one(&state.pool)
        .await?;
    Ok(Json(json!({ "id": id })))
}

// ───────────────────────── 项目与周刊 ─────────────────────────

async fn list_projects(State(state): State<AppState>) -> AppResult<Json<Value>> {
    let rows: Vec<(String, String, String, Value, DateTime<Utc>)> = sqlx::query_as(
        "SELECT slug, name, summary, links, updated_at FROM projects WHERE status = 'published' ORDER BY sort_order, updated_at DESC",
    )
    .fetch_all(&state.pool)
    .await?;
    let items: Vec<Value> = rows
        .into_iter()
        .map(|(slug, name, summary, links, updated_at)| json!({ "slug": slug, "name": name, "summary": summary, "links": links, "updated_at": updated_at }))
        .collect();
    Ok(Json(json!({ "items": items })))
}

async fn get_project(State(state): State<AppState>, Path(slug): Path<String>) -> AppResult<Json<Value>> {
    let row: Option<(String, String, String, String, Value, DateTime<Utc>)> = sqlx::query_as(
        "SELECT slug, name, summary, body_html, links, updated_at FROM projects WHERE slug = $1 AND status = 'published'",
    )
    .bind(&slug)
    .fetch_optional(&state.pool)
    .await?;
    let (slug, name, summary, body_html, links, updated_at) = row.ok_or_else(|| AppError::not_found("项目"))?;
    Ok(Json(json!({ "slug": slug, "name": name, "summary": summary, "body_html": body_html, "links": links, "updated_at": updated_at })))
}

async fn list_weekly(State(state): State<AppState>, Query(q): Query<ListQuery>) -> AppResult<Json<Page<Value>>> {
    let (page, size) = paging(q.page, q.page_size);
    let (total,): (i64,) = sqlx::query_as("SELECT count(*) FROM weekly_issues WHERE status = 'published'")
        .fetch_one(&state.pool)
        .await?;
    let rows: Vec<(String, String, String, Option<DateTime<Utc>>)> = sqlx::query_as(
        "SELECT week, title, summary, published_at FROM weekly_issues WHERE status = 'published'
         ORDER BY week DESC LIMIT $1 OFFSET $2",
    )
    .bind(size)
    .bind((page - 1) * size)
    .fetch_all(&state.pool)
    .await?;
    let items = rows
        .into_iter()
        .map(|(week, title, summary, published_at)| json!({ "week": week, "title": title, "summary": summary, "published_at": published_at }))
        .collect();
    Ok(Json(Page { items, page, page_size: size, total }))
}

/// 周刊条目只存引用，读取时把内容包补成当前的标题与封面，条目下架后自动隐去。
async fn get_weekly(State(state): State<AppState>, Path(week): Path<String>) -> AppResult<Json<Value>> {
    if !content::valid_week(&week) {
        return Err(AppError::BadRequest("周标识形如 2026-W44".into()));
    }
    let row: Option<(String, String, String, Value, Option<DateTime<Utc>>)> = sqlx::query_as(
        "SELECT week, title, summary, items, published_at FROM weekly_issues WHERE week = $1 AND status = 'published'",
    )
    .bind(&week)
    .fetch_optional(&state.pool)
    .await?;
    let (week, title, summary, items, published_at) = row.ok_or_else(|| AppError::not_found("周刊"))?;
    let mut resolved = Vec::new();
    for item in items.as_array().cloned().unwrap_or_default() {
        let note = item.get("note").and_then(Value::as_str).unwrap_or("").to_string();
        match (item.get("type").and_then(Value::as_str), item.get("ref").and_then(Value::as_i64)) {
            (Some("pack"), Some(id)) => {
                if let Ok(pack) = find_pack(&state, id).await {
                    if pack.status == "published" {
                        let mut value = pack.json(&state);
                        value["type"] = json!("pack");
                        value["note"] = json!(note);
                        resolved.push(value);
                    }
                }
            }
            (Some("topic"), Some(id)) => {
                let topic: Option<(String, String)> =
                    sqlx::query_as("SELECT title, excerpt FROM topics WHERE id = $1 AND status = 'normal' AND min_read_level = 0")
                        .bind(id)
                        .fetch_optional(&state.pool)
                        .await?;
                if let Some((t, excerpt)) = topic {
                    resolved.push(json!({ "type": "topic", "id": id, "title": t, "summary": excerpt, "note": note }));
                }
            }
            (Some("text"), _) => resolved.push(json!({ "type": "text", "title": item.get("title"), "note": note })),
            _ => {}
        }
    }
    Ok(Json(json!({ "week": week, "title": title, "summary": summary, "items": resolved, "published_at": published_at })))
}

// ───────────────────────── 分享 ─────────────────────────

/// 分享落地页与海报需要的最小信息：标题、摘要、封面、网页地址与唤起 App 的深链接。
async fn share(State(state): State<AppState>, Path((kind, id)): Path<(String, String)>) -> AppResult<Json<Value>> {
    let site = &state.config.public_url;
    let number = || id.parse::<i64>().map_err(|_| AppError::BadRequest("编号无效".into()));
    let body = match kind.as_str() {
        "pack" => {
            let pack = find_pack(&state, number()?).await?;
            if pack.status != "published" {
                return Err(AppError::not_found("内容包"));
            }
            json!({
                "kind": "pack", "title": pack.title, "summary": pack.summary, "cover_url": cover_url(&state, pack.cover_key.as_deref()),
                "url": format!("{site}/packs/{}", pack.id),
                "app_link": format!("xiaoban://community/apply/{}/{}?version={}", pack.kind, pack.id, pack.current_version),
                "pack_kind": pack.kind, "one_click": content::one_click_allowed(&pack.kind, pack.trust_level, &pack.status),
            })
        }
        "topic" => {
            let t: Option<(String, String)> = sqlx::query_as("SELECT title, excerpt FROM topics WHERE id = $1 AND status = 'normal' AND min_read_level = 0")
                .bind(number()?)
                .fetch_optional(&state.pool)
                .await?;
            let (title, summary) = t.ok_or_else(|| AppError::not_found("帖子"))?;
            json!({ "kind": "topic", "title": title, "summary": summary, "url": format!("{site}/t/{id}"), "app_link": format!("xiaoban://community/open/topic/{id}") })
        }
        "scheme" => {
            let s: Option<(String, String)> = sqlx::query_as("SELECT title, summary FROM schemes WHERE id = $1 AND status = 'published'")
                .bind(number()?)
                .fetch_optional(&state.pool)
                .await?;
            let (title, summary) = s.ok_or_else(|| AppError::not_found("方案"))?;
            json!({ "kind": "scheme", "title": title, "summary": summary, "url": format!("{site}/schemes/{id}"), "app_link": format!("xiaoban://community/open/scheme/{id}") })
        }
        "project" => {
            let p: Option<(String, String)> = sqlx::query_as("SELECT name, summary FROM projects WHERE slug = $1 AND status = 'published'")
                .bind(&id)
                .fetch_optional(&state.pool)
                .await?;
            let (title, summary) = p.ok_or_else(|| AppError::not_found("项目"))?;
            json!({ "kind": "project", "title": title, "summary": summary, "url": format!("{site}/projects/{id}"), "app_link": format!("xiaoban://community/open/project/{id}") })
        }
        "weekly" => {
            let w: Option<(String, String)> = sqlx::query_as("SELECT title, summary FROM weekly_issues WHERE week = $1 AND status = 'published'")
                .bind(&id)
                .fetch_optional(&state.pool)
                .await?;
            let (title, summary) = w.ok_or_else(|| AppError::not_found("周刊"))?;
            json!({ "kind": "weekly", "title": title, "summary": summary, "url": format!("{site}/weekly/{id}"), "app_link": format!("xiaoban://community/open/weekly/{id}") })
        }
        _ => return Err(AppError::BadRequest("不支持分享这种内容".into())),
    };
    Ok(Json(body))
}

// ───────────────────────── 后台 ─────────────────────────

#[derive(Deserialize)]
pub struct AdminListQuery {
    status: Option<String>,
    kind: Option<String>,
    page: Option<i64>,
    page_size: Option<i64>,
}

async fn admin_list_packs(State(state): State<AppState>, _admin: AuthAdmin, Query(q): Query<AdminListQuery>) -> AppResult<Json<Page<Value>>> {
    let (page, size) = paging(q.page, q.page_size);
    let (total,): (i64,) = sqlx::query_as(
        "SELECT count(*) FROM packs p WHERE ($1::text IS NULL OR p.status = $1) AND ($2::text IS NULL OR p.kind = $2)",
    )
    .bind(&q.status)
    .bind(&q.kind)
    .fetch_one(&state.pool)
    .await?;
    let rows: Vec<PackRow> = sqlx::query_as(&format!(
        "SELECT {PACK_COLUMNS} FROM packs p LEFT JOIN users u ON u.uid = p.author_uid
         WHERE ($1::text IS NULL OR p.status = $1) AND ($2::text IS NULL OR p.kind = $2)
         ORDER BY p.updated_at DESC, p.id DESC LIMIT $3 OFFSET $4"
    ))
    .bind(&q.status)
    .bind(&q.kind)
    .bind(size)
    .bind((page - 1) * size)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(Page { items: rows.iter().map(|r| r.json(&state)).collect(), page, page_size: size, total }))
}

#[derive(Deserialize)]
pub struct AdminCreatePackInput {
    kind: String,
    title: String,
    #[serde(default)]
    summary: String,
    #[serde(default)]
    tags: Vec<String>,
    manifest: Value,
    file_key: String,
    sha256: String,
    size: i64,
    cover_key: Option<String>,
    /// 官方镜像（魔搭等），每项 `{ "name": "...", "url": "https://..." }`，客户端回退用。
    #[serde(default)]
    mirrors: Vec<Value>,
    /// 0 官方（默认）或 1 已审核的第三方作品。
    trust_level: Option<i16>,
}

/// 官方直接登记内容包：文件已由运维放进资源机，这里只录入元数据，不经过上传凭证。
async fn admin_create_pack(
    State(state): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Json(input): Json<AdminCreatePackInput>,
) -> AppResult<Json<Value>> {
    let limit = content::kind_limit(&input.kind).ok_or_else(|| AppError::BadRequest("未知的内容类型".into()))?;
    if input.size <= 0 || input.size as u64 > limit {
        return Err(AppError::BadRequest(format!("该类型的内容包不能超过 {} MB", limit / 1024 / 1024)));
    }
    if !content::valid_sha256(&input.sha256) || !content::valid_file_key(&input.file_key) {
        return Err(AppError::BadRequest("sha256 或文件键无效".into()));
    }
    let trust = input.trust_level.unwrap_or(content::T0);
    if !(content::T0..=1).contains(&trust) {
        return Err(AppError::BadRequest("官方登记只能是官方（0）或已审核（1）".into()));
    }
    let mirrors: Vec<&Value> = input.mirrors.iter().collect();
    for m in &mirrors {
        let url = m.get("url").and_then(Value::as_str).unwrap_or("");
        if !url.starts_with("https://") || m.get("name").and_then(Value::as_str).is_none() {
            return Err(AppError::BadRequest("镜像需要 name 和 https 地址".into()));
        }
    }
    if let Some(cover) = &input.cover_key {
        if !content::valid_file_key(cover) {
            return Err(AppError::BadRequest("封面键无效".into()));
        }
    }
    let title = clean_text(&input.title, "标题", 60, true)?;
    let summary = clean_text(&input.summary, "简介", 500, false)?;
    let tags = clean_tags(&input.tags)?;
    let info = content::validate_manifest(&input.manifest, &input.kind).map_err(AppError::BadRequest)?;

    let mut tx = state.pool.begin().await?;
    let (id,): (i64,) = sqlx::query_as(
        "INSERT INTO packs (kind, slug, title, summary, tags, trust_level, status, cover_key)
         VALUES ($1, $2, $3, $4, $5, $6, 'published', $7) RETURNING id",
    )
    .bind(&input.kind)
    .bind(&info.slug)
    .bind(&title)
    .bind(&summary)
    .bind(&tags)
    .bind(trust)
    .bind(&input.cover_key)
    .fetch_one(&mut *tx)
    .await
    .map_err(slug_conflict)?;
    insert_version(&mut tx, id, 1, &info.version_label, &input.sha256, input.size, &input.manifest, &input.file_key, &json!(input.mirrors), "")
        .await?;
    audit(&mut tx, &admin, "pack.create", "pack", id, format!("{} {}", input.kind, info.slug)).await?;
    tx.commit().await?;
    Ok(Json(json!({ "id": id, "slug": info.slug })))
}

#[derive(Deserialize)]
pub struct ReviewInput {
    /// `approve`（升为已审核）、`reject`（拒绝并下架）、`remove`（下架）、`restore`（恢复上架）。
    action: String,
    #[serde(default)]
    note: String,
}

async fn admin_review_pack(
    State(state): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Path(id): Path<i64>,
    Json(input): Json<ReviewInput>,
) -> AppResult<Json<Value>> {
    let note = clean_text(&input.note, "备注", 200, false)?;
    let mut tx = state.pool.begin().await?;
    let (trust, status): (i16, String) = sqlx::query_as("SELECT trust_level, status FROM packs WHERE id = $1 FOR UPDATE")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::not_found("内容包"))?;
    let (new_trust, new_status, queue) = match input.action.as_str() {
        "approve" => (trust.min(1), "published", Some("approved")),
        "reject" => (trust, "rejected", Some("rejected")),
        "remove" => (trust, "removed", None),
        "restore" => (trust, "published", None),
        _ => return Err(AppError::BadRequest("action 只能是 approve、reject、remove 或 restore".into())),
    };
    sqlx::query("UPDATE packs SET trust_level = $2, status = $3, updated_at = now() WHERE id = $1")
        .bind(id)
        .bind(new_trust)
        .bind(new_status)
        .execute(&mut *tx)
        .await?;
    if let Some(queue) = queue {
        sqlx::query(
            "UPDATE moderation_queue SET status = $2, note = $3, handled_by = $4, handled_at = now()
             WHERE target_type = 'pack' AND target_id = $1 AND status = 'pending'",
        )
        .bind(id)
        .bind(queue)
        .bind(&note)
        .bind(&admin.username)
        .execute(&mut *tx)
        .await?;
    }
    audit(&mut tx, &admin, &format!("pack.{}", input.action), "pack", id, format!("{status} -> {new_status}; {note}")).await?;
    tx.commit().await?;
    Ok(Json(json!({ "id": id, "trust_level": new_trust, "status": new_status })))
}

#[derive(Deserialize)]
pub struct MirrorInput {
    /// 不传就是当前版本。镜像按版本锁定：每个版本各有自己的镜像地址。
    version: Option<i32>,
    name: String,
    url: String,
}

/// 给某个版本补一个官方镜像（如魔搭）。同名镜像会被替换。客户端对每个来源都校验 sha256，所以镜像地址本身不需要被信任。
async fn admin_add_mirror(
    State(state): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Path(id): Path<i64>,
    Json(input): Json<MirrorInput>,
) -> AppResult<Json<Value>> {
    let name = clean_text(&input.name, "镜像名称", 30, true)?;
    if !input.url.starts_with("https://") || input.url.len() > 500 || input.url.contains(char::is_whitespace) {
        return Err(AppError::BadRequest("镜像需要 https 地址".into()));
    }
    let mut tx = state.pool.begin().await?;
    let (current,): (i32,) = sqlx::query_as("SELECT current_version FROM packs WHERE id = $1 FOR UPDATE")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::not_found("内容包"))?;
    let version = input.version.unwrap_or(current);
    let (mirrors,): (Value,) = sqlx::query_as("SELECT mirrors FROM pack_versions WHERE pack_id = $1 AND version = $2 FOR UPDATE")
        .bind(id)
        .bind(version)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::not_found("版本"))?;
    let mut list: Vec<Value> = mirrors.as_array().cloned().unwrap_or_default();
    list.retain(|m| m.get("name").and_then(Value::as_str) != Some(name.as_str()));
    list.push(json!({ "name": name, "url": input.url }));
    if list.len() > 5 {
        return Err(AppError::BadRequest("每个版本最多 5 个镜像".into()));
    }
    sqlx::query("UPDATE pack_versions SET mirrors = $3 WHERE pack_id = $1 AND version = $2")
        .bind(id)
        .bind(version)
        .bind(Value::Array(list.clone()))
        .execute(&mut *tx)
        .await?;
    audit(&mut tx, &admin, "pack.mirror", "pack", id, format!("v{version} {name}")).await?;
    tx.commit().await?;
    Ok(Json(json!({ "id": id, "version": version, "mirrors": list })))
}

async fn admin_moderation(State(state): State<AppState>, _admin: AuthAdmin) -> AppResult<Json<Value>> {
    let rows: Vec<(i64, String, i64, DateTime<Utc>, Option<String>, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT q.id, q.target_type, q.target_id, q.created_at, p.title, p.kind, u.nickname
         FROM moderation_queue q
         LEFT JOIN packs p ON q.target_type = 'pack' AND p.id = q.target_id
         LEFT JOIN users u ON u.uid = p.author_uid
         WHERE q.status = 'pending' ORDER BY q.created_at LIMIT 200",
    )
    .fetch_all(&state.pool)
    .await?;
    let items: Vec<Value> = rows
        .into_iter()
        .map(|(id, target_type, target_id, created_at, title, kind, author)| {
            json!({ "id": id, "target_type": target_type, "target_id": target_id, "created_at": created_at,
                    "title": title, "kind": kind, "author": author })
        })
        .collect();
    Ok(Json(json!({ "items": items })))
}

/// 后台列表包含草稿，并返回可编辑的 Markdown 原文。
async fn admin_list_projects(State(state): State<AppState>, _admin: AuthAdmin) -> AppResult<Json<Value>> {
    let rows: Vec<(String, String, String, String, Value, String, i32)> = sqlx::query_as(
        "SELECT slug, name, summary, body_md, links, status, sort_order FROM projects ORDER BY sort_order, updated_at DESC",
    )
    .fetch_all(&state.pool)
    .await?;
    let items: Vec<Value> = rows
        .into_iter()
        .map(|(slug, name, summary, body_md, links, status, sort_order)| {
            json!({ "slug": slug, "name": name, "summary": summary, "body_md": body_md, "links": links, "status": status, "sort_order": sort_order })
        })
        .collect();
    Ok(Json(json!({ "items": items })))
}

async fn admin_list_weekly(State(state): State<AppState>, _admin: AuthAdmin) -> AppResult<Json<Value>> {
    let rows: Vec<(String, String, String, Value, String)> =
        sqlx::query_as("SELECT week, title, summary, items, status FROM weekly_issues ORDER BY week DESC LIMIT 100")
            .fetch_all(&state.pool)
            .await?;
    let items: Vec<Value> = rows
        .into_iter()
        .map(|(week, title, summary, items, status)| json!({ "week": week, "title": title, "summary": summary, "items": items, "status": status }))
        .collect();
    Ok(Json(json!({ "items": items })))
}

#[derive(Deserialize)]
pub struct ProjectInput {
    name: String,
    #[serde(default)]
    summary: String,
    #[serde(default)]
    body_md: String,
    #[serde(default)]
    links: Vec<Value>,
    status: Option<String>,
    sort_order: Option<i32>,
}

async fn admin_put_project(
    State(state): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Path(slug): Path<String>,
    Json(input): Json<ProjectInput>,
) -> AppResult<Json<Value>> {
    if !content::valid_slug(&slug) {
        return Err(AppError::BadRequest("项目 id 只能用小写字母、数字、连字符和下划线".into()));
    }
    let name = clean_text(&input.name, "名称", 60, true)?;
    let summary = clean_text(&input.summary, "简介", 300, false)?;
    let status = input.status.as_deref().unwrap_or("published");
    if !["draft", "published"].contains(&status) {
        return Err(AppError::BadRequest("状态只能是 draft 或 published".into()));
    }
    let body_html = markdown::render(&input.body_md, true);
    let mut tx = state.pool.begin().await?;
    sqlx::query(
        "INSERT INTO projects (slug, name, summary, body_md, body_html, links, status, sort_order)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         ON CONFLICT (slug) DO UPDATE SET name = $2, summary = $3, body_md = $4, body_html = $5, links = $6,
             status = $7, sort_order = $8, updated_at = now()",
    )
    .bind(&slug)
    .bind(&name)
    .bind(&summary)
    .bind(&input.body_md)
    .bind(&body_html)
    .bind(Value::Array(input.links))
    .bind(status)
    .bind(input.sort_order.unwrap_or(0))
    .execute(&mut *tx)
    .await?;
    audit(&mut tx, &admin, "project.put", "project", &slug, status).await?;
    tx.commit().await?;
    Ok(Json(json!({ "slug": slug })))
}

#[derive(Deserialize)]
pub struct WeeklyInput {
    title: String,
    #[serde(default)]
    summary: String,
    #[serde(default)]
    items: Vec<Value>,
    status: Option<String>,
}

async fn admin_put_weekly(
    State(state): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Path(week): Path<String>,
    Json(input): Json<WeeklyInput>,
) -> AppResult<Json<Value>> {
    if !content::valid_week(&week) {
        return Err(AppError::BadRequest("周标识形如 2026-W44".into()));
    }
    let title = clean_text(&input.title, "标题", 60, true)?;
    let summary = clean_text(&input.summary, "导语", 500, false)?;
    let status = input.status.as_deref().unwrap_or("draft");
    if !["draft", "published"].contains(&status) {
        return Err(AppError::BadRequest("状态只能是 draft 或 published".into()));
    }
    if input.items.len() > 30 {
        return Err(AppError::BadRequest("每期最多 30 条".into()));
    }
    let mut tx = state.pool.begin().await?;
    sqlx::query(
        "INSERT INTO weekly_issues (week, title, summary, items, status, published_at)
         VALUES ($1, $2, $3, $4, $5, CASE WHEN $5 = 'published' THEN now() END)
         ON CONFLICT (week) DO UPDATE SET title = $2, summary = $3, items = $4, status = $5,
             published_at = CASE WHEN $5 = 'published' THEN COALESCE(weekly_issues.published_at, now()) END,
             updated_at = now()",
    )
    .bind(&week)
    .bind(&title)
    .bind(&summary)
    .bind(Value::Array(input.items))
    .bind(status)
    .execute(&mut *tx)
    .await?;
    audit(&mut tx, &admin, "weekly.put", "weekly", &week, status).await?;
    tx.commit().await?;
    Ok(Json(json!({ "week": week })))
}

#[cfg(test)]
#[path = "content_tests.rs"]
mod content_tests;
