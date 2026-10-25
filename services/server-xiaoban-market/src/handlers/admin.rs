//! 管理后台 API：`/api/v1/admin/*`，除登录外都需要 `Authorization: Bearer <token>`。
use crate::{
    auth::{self, Admin},
    crawler,
    error::{AppError, Result},
    importer::{self, ImportOpts},
    items, package,
    state::AppState,
    storage,
    upstream::{self, Params, SourceRow},
};
use axum::{
    extract::{DefaultBodyLimit, Multipart, Path, Query, State},
    routing::{get, post, put},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::types::Json as PgJson;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/config", get(public_config))
        .route("/login", post(login))
        .route("/login/account", post(login_with_account))
        .route("/account/bindings", get(list_bindings).post(bind_account))
        .route("/account/bindings/{id}", axum::routing::delete(unbind_account))
        .route("/me", get(me))
        .route("/account", put(change_password))
        .route("/stats", get(stats))
        .route("/sources", get(list_sources))
        .route("/sources/{id}", put(update_source))
        .route("/sources/{id}/probe", post(probe))
        .route("/sources/{id}/jobs", post(start_job))
        .route("/jobs", get(list_jobs))
        .route("/jobs/{id}", get(get_job))
        .route("/jobs/{id}/cancel", post(cancel_job))
        .route("/staged", get(list_staged))
        .route("/staged/import", post(import_many))
        .route("/staged/ignore", post(ignore_many))
        .route("/staged/restore", post(restore_many))
        .route("/staged/{id}", get(get_staged))
        .route("/items", get(list_items).post(create_item))
        .route("/items/{id}", get(get_item).put(update_item).delete(delete_item))
        .route("/items/{id}/status", post(set_status))
        .route("/items/{id}/versions", post(upload_version).layer(DefaultBodyLimit::max(14 * 1024 * 1024)))
        .route("/versions/{id}", put(update_version))
        .route("/categories", get(list_categories).post(create_category))
        .route("/categories/{id}", put(update_category).delete(delete_category))
}

type Rows = Vec<PgJson<Value>>;

fn unwrap(rows: Rows) -> Vec<Value> {
    rows.into_iter().map(|j| j.0).collect()
}

fn total_of(rows: &[Value]) -> i64 {
    rows.first().and_then(|r| r["total"].as_i64()).unwrap_or(0)
}

fn page_args(page: Option<i64>, size: Option<i64>, default: i64, max: i64) -> (i64, i64, i64) {
    let page = page.unwrap_or(1).max(1);
    let size = size.filter(|s| *s > 0).unwrap_or(default).min(max);
    (page, size, (page - 1) * size)
}

// ───────────── 登录与账号 ─────────────

#[derive(Deserialize)]
struct LoginBody {
    username: String,
    password: String,
}

async fn login(State(state): State<AppState>, Json(body): Json<LoginBody>) -> Result<Json<Value>> {
    let row: Option<(String,)> = sqlx::query_as("SELECT password_hash FROM admin_users WHERE username = $1")
        .bind(&body.username)
        .fetch_optional(&state.pool)
        .await?;
    match row {
        Some((hash,)) if auth::verify_password(&body.password, &hash) => {
            Ok(Json(json!({ "token": auth::issue_token(&body.username, &state.config.jwt_secret)?, "username": body.username })))
        }
        _ => Err(AppError::Unauthorized("用户名或密码错误".into())),
    }
}

/// 登录页需要的公开配置：是否启用相伴账号登录，以及浏览器该访问哪个账号中心。
async fn public_config(State(state): State<AppState>) -> Json<Value> {
    let c = &state.config;
    Json(json!({ "account": { "enabled": state.account.is_some(), "url": c.account_url, "clientId": c.account_client_id } }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AccountTokenBody {
    access_token: String,
    #[serde(default)]
    label: String,
}

fn verifier(state: &AppState) -> Result<&crate::account::AccountVerifier> {
    state.account.as_deref().ok_or_else(|| AppError::BadRequest("未启用相伴账号登录".into()))
}

/// 账号中心登录：令牌验签通过，且该账号已被绑定为管理员才签发后台令牌（8 小时）。
async fn login_with_account(State(state): State<AppState>, Json(body): Json<AccountTokenBody>) -> Result<Json<Value>> {
    let account_id = verifier(&state)?.verify(&body.access_token).await?;
    let label: Option<String> = sqlx::query_scalar("SELECT label FROM admin_accounts WHERE account_id = $1").bind(account_id).fetch_optional(&state.pool).await?;
    let label = label.ok_or_else(|| AppError::Unauthorized("这个相伴账号还不是后台管理员，请先用本地管理员登录并在「设置」里绑定".into()))?;
    let token = auth::issue_token_for(&label, &state.config.jwt_secret, 8 * 3600, Some(account_id.to_string()))?;
    Ok(Json(json!({ "token": token, "username": label })))
}

async fn list_bindings(State(state): State<AppState>, _admin: Admin) -> Result<Json<Value>> {
    let rows = unwrap(
        sqlx::query_scalar::<_, PgJson<Value>>(
            "SELECT to_jsonb(x) FROM (SELECT id, account_id AS \"accountId\", label, bound_by AS \"boundBy\", created_at AS \"createdAt\" FROM admin_accounts ORDER BY created_at) x",
        )
        .fetch_all(&state.pool)
        .await?,
    );
    Ok(Json(json!({ "enabled": state.account.is_some(), "results": rows })))
}

/// 绑定：调用者提交要绑定的账号的访问令牌（在账号中心登录得到），服务端验签后登记。
async fn bind_account(State(state): State<AppState>, Admin(claims): Admin, Json(body): Json<AccountTokenBody>) -> Result<Json<Value>> {
    let account_id = verifier(&state)?.verify(&body.access_token).await?;
    let label: String = body.label.trim().chars().take(40).collect();
    if label.is_empty() {
        return Err(AppError::BadRequest("请填写一个便于识别的名称".into()));
    }
    sqlx::query(
        "INSERT INTO admin_accounts (account_id, label, bound_by) VALUES ($1, $2, $3)
         ON CONFLICT (account_id) DO UPDATE SET label = EXCLUDED.label",
    )
    .bind(account_id)
    .bind(&label)
    .bind(&claims.sub)
    .execute(&state.pool)
    .await?;
    Ok(Json(json!({ "ok": true })))
}

/// 解绑后，该账号已有的后台会话在下一次请求就会失效（见 `Admin` 提取器）。本地管理员不受影响。
async fn unbind_account(State(state): State<AppState>, _admin: Admin, Path(id): Path<Uuid>) -> Result<Json<Value>> {
    let n = sqlx::query("DELETE FROM admin_accounts WHERE id = $1").bind(id).execute(&state.pool).await?.rows_affected();
    if n == 0 {
        return Err(AppError::NotFound("绑定不存在".into()));
    }
    Ok(Json(json!({ "ok": true })))
}

async fn me(Admin(claims): Admin) -> Json<Value> {
    Json(json!({ "username": claims.sub }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PasswordBody {
    current_password: String,
    new_password: String,
}

async fn change_password(State(state): State<AppState>, Admin(claims): Admin, Json(body): Json<PasswordBody>) -> Result<Json<Value>> {
    if body.new_password.chars().count() < 8 {
        return Err(AppError::BadRequest("新密码至少 8 位".into()));
    }
    let hash: String = sqlx::query_scalar("SELECT password_hash FROM admin_users WHERE username = $1").bind(&claims.sub).fetch_one(&state.pool).await?;
    if !auth::verify_password(&body.current_password, &hash) {
        return Err(AppError::Unauthorized("当前密码不正确".into()));
    }
    sqlx::query("UPDATE admin_users SET password_hash = $2 WHERE username = $1")
        .bind(&claims.sub)
        .bind(auth::hash_password(&body.new_password)?)
        .execute(&state.pool)
        .await?;
    Ok(Json(json!({ "ok": true })))
}

// ───────────── 总览 ─────────────

async fn stats(State(state): State<AppState>, _admin: Admin) -> Result<Json<Value>> {
    let value: PgJson<Value> = sqlx::query_scalar(
        "SELECT jsonb_build_object(
           'pendingStaged', (SELECT count(*) FROM staged_items WHERE status = 'pending'),
           'upstreamChanged', (SELECT count(*) FROM staged_items WHERE status = 'pending' AND item_id IS NOT NULL),
           'importFailed', (SELECT count(*) FROM staged_items WHERE import_error IS NOT NULL AND status = 'pending'),
           'failedJobs24h', (SELECT count(*) FROM crawl_jobs WHERE status = 'failed' AND created_at > now() - interval '24 hours'),
           'publishedThisWeek', (SELECT COALESCE(jsonb_object_agg(kind, n), '{}'::jsonb)
               FROM (SELECT kind, count(*) AS n FROM items WHERE published_at > now() - interval '7 days' GROUP BY kind) t),
           'itemsByStatus', (SELECT COALESCE(jsonb_object_agg(status, n), '{}'::jsonb)
               FROM (SELECT status, count(*) AS n FROM items GROUP BY status) t))",
    )
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(value.0))
}

// ───────────── 采集来源 ─────────────

/// 回显配置时隐藏密钥；保存时把 `***` 还原成旧值。
fn restore_secrets(old: &Value, new: &mut Value) {
    let (Some(old), Some(new)) = (old.as_object(), new.as_object_mut()) else { return };
    if new.get("api_key").and_then(|v| v.as_str()) == Some("***") {
        match old.get("api_key") {
            Some(v) => new.insert("api_key".into(), v.clone()),
            None => new.remove("api_key"),
        };
    }
    if let (Some(old_h), Some(new_h)) = (old.get("headers").and_then(|h| h.as_object()), new.get_mut("headers").and_then(|h| h.as_object_mut())) {
        for (k, v) in new_h.iter_mut() {
            if v.as_str() == Some("***") {
                if let Some(o) = old_h.get(k) {
                    *v = o.clone();
                }
            }
        }
    }
}

fn source_json(s: &SourceRow, last_job: Option<Value>, pending: i64) -> Value {
    let configured = s.adapter == "skillhub" || s.config["url"].as_str().is_some_and(|u| !u.is_empty());
    json!({
        "id": s.id, "key": s.key, "name": s.name, "kind": s.kind, "adapter": s.adapter, "enabled": s.enabled,
        "config": upstream::masked_config(&s.config.0), "intervalMinutes": s.interval_minutes, "autoImport": s.auto_import,
        "lastJobAt": s.last_job_at, "configured": configured, "pending": pending, "lastJob": last_job,
    })
}

async fn list_sources(State(state): State<AppState>, _admin: Admin) -> Result<Json<Value>> {
    let sources: Vec<SourceRow> = sqlx::query_as("SELECT * FROM sources ORDER BY id").fetch_all(&state.pool).await?;
    let mut out = Vec::new();
    for s in &sources {
        let last: Option<PgJson<Value>> = sqlx::query_scalar(
            "SELECT jsonb_build_object('id', id, 'status', status, 'stats', stats, 'error', error, 'createdAt', created_at, 'finishedAt', finished_at)
             FROM crawl_jobs WHERE source_id = $1 ORDER BY created_at DESC LIMIT 1",
        )
        .bind(s.id)
        .fetch_optional(&state.pool)
        .await?;
        let pending: i64 = sqlx::query_scalar("SELECT count(*) FROM staged_items WHERE source_id = $1 AND status = 'pending'").bind(s.id).fetch_one(&state.pool).await?;
        out.push(source_json(s, last.map(|j| j.0), pending));
    }
    Ok(Json(json!({ "results": out })))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SourceUpdate {
    name: Option<String>,
    enabled: Option<bool>,
    interval_minutes: Option<i32>,
    auto_import: Option<String>,
    config: Option<Value>,
}

async fn load_source(state: &AppState, id: i32) -> Result<SourceRow> {
    sqlx::query_as("SELECT * FROM sources WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("采集来源不存在".into()))
}

async fn update_source(State(state): State<AppState>, _admin: Admin, Path(id): Path<i32>, Json(body): Json<SourceUpdate>) -> Result<Json<Value>> {
    let current = load_source(&state, id).await?;
    if let Some(m) = body.interval_minutes {
        if m != 0 && !(15..=10080).contains(&m) {
            return Err(AppError::BadRequest("定时间隔应为 0（仅手动）或 15–10080 分钟".into()));
        }
    }
    if let Some(a) = &body.auto_import {
        if !matches!(a.as_str(), "none" | "draft" | "published") {
            return Err(AppError::BadRequest("autoImport 只能是 none / draft / published".into()));
        }
    }
    let config = match body.config {
        Some(mut c) => {
            if !c.is_object() {
                return Err(AppError::BadRequest("config 必须是 JSON 对象".into()));
            }
            restore_secrets(&current.config.0, &mut c);
            for key in ["url", "base_url"] {
                if let Some(u) = c[key].as_str().filter(|u| !u.is_empty()) {
                    upstream::validate_url(u, state.config.allow_private_upstream).map_err(AppError::BadRequest)?;
                }
            }
            Some(c)
        }
        None => None,
    };
    sqlx::query(
        "UPDATE sources SET name = COALESCE($2, name), enabled = COALESCE($3, enabled), interval_minutes = COALESCE($4, interval_minutes),
           auto_import = COALESCE($5, auto_import), config = COALESCE($6, config) WHERE id = $1",
    )
    .bind(id)
    .bind(body.name)
    .bind(body.enabled)
    .bind(body.interval_minutes)
    .bind(body.auto_import)
    .bind(config.map(PgJson))
    .execute(&state.pool)
    .await?;
    let updated = load_source(&state, id).await?;
    Ok(Json(source_json(&updated, None, 0)))
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct ProbeBody {
    params: Params,
}

/// 试采：只抓第一页并规范化，不写库。
async fn probe(State(state): State<AppState>, _admin: Admin, Path(id): Path<i32>, body: Option<Json<ProbeBody>>) -> Result<Json<Value>> {
    let source = load_source(&state, id).await?;
    let params = body.map(|b| b.0.params).unwrap_or_default();
    match upstream::fetch_page(&state.ctx(), &source, &params, 1).await {
        Err(e) => Ok(Json(json!({ "ok": false, "error": e.message }))),
        Ok(page) => {
            let mapping = if source.adapter == "generic_json" { page.entries.first().map(|e| upstream::generic::mapping_report(&source, &e.raw)).unwrap_or_default() } else { vec![] };
            let preview = page.response.to_string();
            let response = if preview.len() > 100_000 { json!({ "truncated": true, "preview": preview.chars().take(100_000).collect::<String>() }) } else { page.response };
            Ok(Json(json!({
                "ok": true, "count": page.entries.len(), "total": page.total, "hasMore": page.has_more, "skipped": page.skipped,
                "entries": page.entries.iter().take(3).map(|e| json!({ "raw": e.raw, "normalized": e.normalized })).collect::<Vec<_>>(),
                "mapping": mapping, "response": response,
            })))
        }
    }
}

async fn start_job(State(state): State<AppState>, admin: Admin, Path(id): Path<i32>, body: Option<Json<Params>>) -> Result<Json<Value>> {
    let params = body.map(|b| b.0).unwrap_or_default();
    let job = crawler::start(&state, id, params, "manual", Some(&admin.0.sub)).await?;
    Ok(Json(json!({ "id": job })))
}

// ───────────── 采集任务 ─────────────

const JOB_COLUMNS: &str = "j.id, j.source_id AS \"sourceId\", s.name AS \"sourceName\", j.status, j.trigger, j.params, j.stats, j.error,
  j.created_by AS \"createdBy\", j.created_at AS \"createdAt\", j.started_at AS \"startedAt\", j.finished_at AS \"finishedAt\"";

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct JobQuery {
    source_id: Option<i32>,
    status: Option<String>,
    page: Option<i64>,
    page_size: Option<i64>,
}

async fn list_jobs(State(state): State<AppState>, _admin: Admin, Query(q): Query<JobQuery>) -> Result<Json<Value>> {
    let (page, size, offset) = page_args(q.page, q.page_size, 30, 100);
    let sql = format!(
        "SELECT to_jsonb(x) FROM (SELECT {JOB_COLUMNS}, count(*) OVER() AS total FROM crawl_jobs j JOIN sources s ON s.id = j.source_id
         WHERE ($1::int IS NULL OR j.source_id = $1) AND ($2::text IS NULL OR j.status = $2) ORDER BY j.created_at DESC LIMIT $3 OFFSET $4) x"
    );
    let rows = unwrap(
        sqlx::query_scalar::<_, PgJson<Value>>(&sql)
            .bind(q.source_id)
            .bind(q.status.filter(|s| !s.is_empty()))
            .bind(size)
            .bind(offset)
            .fetch_all(&state.pool)
            .await?,
    );
    Ok(Json(json!({ "total": total_of(&rows), "page": page, "pageSize": size, "results": rows })))
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct LogQuery {
    after: i64,
}

async fn get_job(State(state): State<AppState>, _admin: Admin, Path(id): Path<Uuid>, Query(q): Query<LogQuery>) -> Result<Json<Value>> {
    let sql = format!("SELECT to_jsonb(x) FROM (SELECT {JOB_COLUMNS} FROM crawl_jobs j JOIN sources s ON s.id = j.source_id WHERE j.id = $1) x");
    let job: PgJson<Value> = sqlx::query_scalar(&sql).bind(id).fetch_optional(&state.pool).await?.ok_or_else(|| AppError::NotFound("任务不存在".into()))?;
    let logs = unwrap(
        sqlx::query_scalar::<_, PgJson<Value>>("SELECT to_jsonb(x) FROM (SELECT id, ts, level, message FROM crawl_logs WHERE job_id = $1 AND id > $2 ORDER BY id LIMIT 500) x")
            .bind(id)
            .bind(q.after)
            .fetch_all(&state.pool)
            .await?,
    );
    Ok(Json(json!({ "job": job.0, "logs": logs })))
}

async fn cancel_job(State(state): State<AppState>, _admin: Admin, Path(id): Path<Uuid>) -> Result<Json<Value>> {
    if state.runner.cancel(id) {
        return Ok(Json(json!({ "ok": true })));
    }
    Err(AppError::Conflict("任务不在运行中".into()))
}

// ───────────── 暂存区 ─────────────

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct StagedQuery {
    source_id: Option<i32>,
    status: Option<String>,
    change: Option<String>,
    keyword: String,
    page: Option<i64>,
    page_size: Option<i64>,
}

async fn list_staged(State(state): State<AppState>, _admin: Admin, Query(q): Query<StagedQuery>) -> Result<Json<Value>> {
    let (page, size, offset) = page_args(q.page, q.page_size, 50, 100);
    let status = q.status.filter(|s| matches!(s.as_str(), "pending" | "imported" | "ignored"));
    let change = q.change.filter(|s| matches!(s.as_str(), "new" | "changed" | "failed"));
    let rows = unwrap(
        sqlx::query_scalar::<_, PgJson<Value>>(
            "SELECT to_jsonb(y) FROM (
               SELECT x.*, count(*) OVER() AS total FROM (
                 SELECT s.id, s.source_id AS \"sourceId\", src.name AS \"sourceName\", s.upstream_id AS \"upstreamId\", s.kind, s.name, s.summary,
                   s.category, s.status, s.item_id AS \"itemId\", s.import_error AS \"importError\", s.first_seen_at AS \"firstSeenAt\",
                   s.last_seen_at AS \"lastSeenAt\", s.normalized->>'slug' AS slug, s.normalized->>'version' AS version,
                   CASE WHEN s.import_error IS NOT NULL THEN 'failed' WHEN s.status = 'pending' AND s.item_id IS NULL THEN 'new'
                        WHEN s.status = 'pending' THEN 'changed' END AS change
                 FROM staged_items s JOIN sources src ON src.id = s.source_id
                 WHERE ($1::int IS NULL OR s.source_id = $1) AND ($2::text IS NULL OR s.status = $2) AND s.search_text ILIKE ALL($4::text[])
               ) x WHERE ($3::text IS NULL OR x.change = $3)
               ORDER BY x.\"lastSeenAt\" DESC, x.id LIMIT $5 OFFSET $6) y",
        )
        .bind(q.source_id)
        .bind(status)
        .bind(change)
        .bind(super::public::patterns(&q.keyword))
        .bind(size)
        .bind(offset)
        .fetch_all(&state.pool)
        .await?,
    );
    let counts: PgJson<Value> = sqlx::query_scalar(
        "SELECT COALESCE(jsonb_object_agg(status, n), '{}'::jsonb) FROM (SELECT status, count(*) AS n FROM staged_items GROUP BY status) t",
    )
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(json!({ "total": total_of(&rows), "page": page, "pageSize": size, "results": rows, "counts": counts.0 })))
}

async fn get_staged(State(state): State<AppState>, _admin: Admin, Path(id): Path<Uuid>) -> Result<Json<Value>> {
    let row: PgJson<Value> = sqlx::query_scalar(
        "SELECT to_jsonb(x) FROM (SELECT s.id, s.source_id AS \"sourceId\", src.name AS \"sourceName\", s.kind, s.status, s.raw, s.normalized,
           s.import_error AS \"importError\", s.item_id AS \"itemId\", s.last_seen_at AS \"lastSeenAt\",
           (SELECT to_jsonb(i) FROM (SELECT id, name, summary, tags, latest_version AS version, status, category_id AS \"categoryId\"
              FROM items WHERE id = s.item_id) i) AS existing
         FROM staged_items s JOIN sources src ON src.id = s.source_id WHERE s.id = $1) x",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("暂存项不存在".into()))?;
    Ok(Json(row.0))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImportBody {
    ids: Vec<Uuid>,
    status: Option<String>,
    category_id: Option<i32>,
}

fn check_status(s: &str) -> Result<()> {
    if matches!(s, "draft" | "review" | "published" | "offline") {
        Ok(())
    } else {
        Err(AppError::BadRequest("状态只能是 draft / review / published / offline".into()))
    }
}

async fn import_many(State(state): State<AppState>, _admin: Admin, Json(body): Json<ImportBody>) -> Result<Json<Value>> {
    if body.ids.is_empty() || body.ids.len() > 100 {
        return Err(AppError::BadRequest("一次入库 1–100 项".into()));
    }
    if let Some(s) = &body.status {
        check_status(s)?;
    }
    let opts = ImportOpts { status: body.status, category_id: body.category_id };
    let mut results = Vec::new();
    for id in body.ids {
        match importer::import_staged(&state, id, &opts).await {
            Ok(item) => results.push(json!({ "id": id, "ok": true, "itemId": item })),
            Err(e) => results.push(json!({ "id": id, "ok": false, "error": e.to_string() })),
        }
    }
    Ok(Json(json!({ "results": results })))
}

#[derive(Deserialize)]
struct IdsBody {
    ids: Vec<Uuid>,
}

async fn ignore_many(State(state): State<AppState>, _admin: Admin, Json(body): Json<IdsBody>) -> Result<Json<Value>> {
    let n = sqlx::query("UPDATE staged_items SET status = 'ignored' WHERE id = ANY($1) AND status = 'pending'").bind(&body.ids).execute(&state.pool).await?.rows_affected();
    Ok(Json(json!({ "updated": n })))
}

async fn restore_many(State(state): State<AppState>, _admin: Admin, Json(body): Json<IdsBody>) -> Result<Json<Value>> {
    let n = sqlx::query("UPDATE staged_items SET status = 'pending' WHERE id = ANY($1) AND status = 'ignored'").bind(&body.ids).execute(&state.pool).await?.rows_affected();
    Ok(Json(json!({ "updated": n })))
}

// ───────────── 内容库 ─────────────

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct ItemQuery {
    kind: Option<String>,
    status: Option<String>,
    variant: Option<String>,
    category_id: Option<i32>,
    keyword: String,
    page: Option<i64>,
    page_size: Option<i64>,
}

const ITEM_COLUMNS: &str = "i.id, i.kind, i.namespace, i.slug, i.name, i.summary, i.status, i.source, i.origin, i.variant, i.featured, i.latest_version AS version,
  i.download_count AS downloads, i.favorite_count AS favorites, i.tags, i.icon_url AS \"iconUrl\", i.category_id AS \"categoryId\",
  c.name AS \"categoryName\", i.updated_at AS \"updatedAt\", i.published_at AS \"publishedAt\"";

async fn list_items(State(state): State<AppState>, _admin: Admin, Query(q): Query<ItemQuery>) -> Result<Json<Value>> {
    let (page, size, offset) = page_args(q.page, q.page_size, 50, 100);
    let sql = format!(
        "SELECT to_jsonb(x) FROM (SELECT {ITEM_COLUMNS}, count(*) OVER() AS total FROM items i LEFT JOIN categories c ON c.id = i.category_id
         WHERE ($1::text IS NULL OR i.kind = $1) AND ($2::text IS NULL OR i.status = $2) AND ($3::int IS NULL OR i.category_id = $3)
           AND i.search_text ILIKE ALL($4::text[]) AND ($7::text IS NULL OR i.variant = $7) ORDER BY i.updated_at DESC, i.id LIMIT $5 OFFSET $6) x"
    );
    let rows = unwrap(
        sqlx::query_scalar::<_, PgJson<Value>>(&sql)
            .bind(q.kind.filter(|s| !s.is_empty()))
            .bind(q.status.filter(|s| !s.is_empty()))
            .bind(q.category_id)
            .bind(super::public::patterns(&q.keyword))
            .bind(size)
            .bind(offset)
            .bind(q.variant.filter(|v| matches!(v.as_str(), "single" | "team" | "bundle")))
            .fetch_all(&state.pool)
            .await?,
    );
    Ok(Json(json!({ "total": total_of(&rows), "page": page, "pageSize": size, "results": rows })))
}

#[derive(Deserialize, Clone)]
struct MemberRef {
    namespace: String,
    slug: String,
}

const MAX_MEMBERS: usize = 24;

fn member_kind(variant: &str) -> &'static str {
    if variant == "team" { "expert" } else { "skill" }
}

/// 团队 / 套件的成员必须已存在、是单个条目（不嵌套）、不是自己，且不重复。返回规范化后的引用列表。
async fn validate_members(state: &AppState, variant: &str, members: &[MemberRef], own: Option<Uuid>) -> Result<Vec<Value>> {
    if variant == "single" {
        return if members.is_empty() { Ok(vec![]) } else { Err(AppError::BadRequest("只有专家团和套件可以设置成员".into())) };
    }
    if members.is_empty() || members.len() > MAX_MEMBERS {
        return Err(AppError::BadRequest(format!("{}需要 1–{MAX_MEMBERS} 个成员", if variant == "team" { "专家团" } else { "套件" })));
    }
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for m in members {
        let key = (m.namespace.trim().to_string(), m.slug.trim().to_string());
        if !seen.insert(key.clone()) {
            return Err(AppError::BadRequest(format!("成员 {}/{} 重复", key.0, key.1)));
        }
        let row: Option<(Uuid, String)> = sqlx::query_as("SELECT id, variant FROM items WHERE kind = $1 AND namespace = $2 AND slug = $3")
            .bind(member_kind(variant))
            .bind(&key.0)
            .bind(&key.1)
            .fetch_optional(&state.pool)
            .await?;
        let (id, v) = row.ok_or_else(|| AppError::BadRequest(format!("成员 {}/{} 不存在（需要是{}）", key.0, key.1, if variant == "team" { "专家" } else { "技能" })))?;
        if Some(id) == own {
            return Err(AppError::BadRequest("成员不能包含自己".into()));
        }
        if v != "single" {
            return Err(AppError::BadRequest(format!("成员 {}/{} 本身是{}，不能嵌套", key.0, key.1, if v == "team" { "专家团" } else { "套件" })));
        }
        out.push(json!({ "namespace": key.0, "slug": key.1 }));
    }
    Ok(out)
}

async fn get_item(State(state): State<AppState>, _admin: Admin, Path(id): Path<Uuid>) -> Result<Json<Value>> {
    let sql = format!(
        "SELECT to_jsonb(x) FROM (SELECT {ITEM_COLUMNS}, i.description, i.license, i.origin_url AS \"originUrl\", i.prompts, i.metadata
         FROM items i LEFT JOIN categories c ON c.id = i.category_id WHERE i.id = $1) x"
    );
    let item: PgJson<Value> = sqlx::query_scalar(&sql).bind(id).fetch_optional(&state.pool).await?.ok_or_else(|| AppError::NotFound("条目不存在".into()))?;
    let versions = unwrap(
        sqlx::query_scalar::<_, PgJson<Value>>(
            "SELECT to_jsonb(x) FROM (SELECT id, version, changelog, package_size AS size, manifest_sha256 AS \"manifestSha256\",
               package_sha256 AS \"packageSha256\", yanked, created_at AS \"createdAt\", jsonb_array_length(manifest->'files') AS \"fileCount\"
             FROM versions WHERE item_id = $1 ORDER BY created_at DESC, version DESC) x",
        )
        .bind(id)
        .fetch_all(&state.pool)
        .await?,
    );
    let members = unwrap(
        sqlx::query_scalar::<_, PgJson<Value>>(
            "SELECT to_jsonb(x) FROM (
               SELECT m.v->>'namespace' AS namespace, m.v->>'slug' AS slug, i.id, i.name, i.status
               FROM items me CROSS JOIN LATERAL jsonb_array_elements(COALESCE(me.metadata->'members', '[]'::jsonb)) WITH ORDINALITY AS m(v, n)
               LEFT JOIN items i ON i.namespace = m.v->>'namespace' AND i.slug = m.v->>'slug' AND i.kind = CASE WHEN me.variant = 'team' THEN 'expert' ELSE 'skill' END
               WHERE me.id = $1 ORDER BY m.n) x",
        )
        .bind(id)
        .fetch_all(&state.pool)
        .await?,
    );
    Ok(Json(json!({ "item": item.0, "versions": versions, "members": members })))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct NewItem {
    kind: String,
    variant: Option<String>,
    #[serde(default)]
    members: Vec<MemberRef>,
    namespace: Option<String>,
    slug: String,
    name: String,
    #[serde(default)]
    summary: String,
    #[serde(default)]
    description: String,
    category_id: Option<i32>,
    #[serde(default)]
    tags: Vec<String>,
    icon_url: Option<String>,
    license: Option<String>,
    source: Option<String>,
    #[serde(default)]
    prompts: Vec<String>,
    status: Option<String>,
    #[serde(default)]
    metadata: Value,
}

fn clean_tags(tags: Vec<String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for t in tags {
        let t: String = t.trim().chars().take(32).collect();
        if !t.is_empty() && !out.contains(&t) && out.len() < 12 {
            out.push(t);
        }
    }
    out
}

fn check_source(s: &str) -> Result<()> {
    if matches!(s, "official" | "community" | "enterprise") {
        Ok(())
    } else {
        Err(AppError::BadRequest("来源只能是 official / community / enterprise".into()))
    }
}

fn check_icon(url: &Option<String>) -> Result<()> {
    match url.as_deref().filter(|u| !u.is_empty()) {
        Some(u) if !u.starts_with("https://") => Err(AppError::BadRequest("图标地址必须是 HTTPS".into())),
        _ => Ok(()),
    }
}

async fn category_name(state: &AppState, id: Option<i32>) -> Result<String> {
    Ok(match id {
        Some(id) => sqlx::query_scalar("SELECT name FROM categories WHERE id = $1").bind(id).fetch_optional(&state.pool).await?.ok_or_else(|| AppError::BadRequest("分类不存在".into()))?,
        None => String::new(),
    })
}

async fn create_item(State(state): State<AppState>, _admin: Admin, Json(b): Json<NewItem>) -> Result<Json<Value>> {
    let kind = super::public::kind_of(&b.kind)?;
    let namespace = b.namespace.filter(|n| !n.is_empty()).unwrap_or_else(|| "xiaoban".into());
    if !upstream::valid_slug(&b.slug) || !upstream::valid_namespace(&namespace) {
        return Err(AppError::BadRequest("标识只能含小写字母、数字、连字符（2–64 位）；命名空间只能含字母、数字、下划线、连字符".into()));
    }
    if b.name.trim().is_empty() {
        return Err(AppError::BadRequest("名称不能为空".into()));
    }
    let source = b.source.unwrap_or_else(|| "official".into());
    check_source(&source)?;
    let status = b.status.unwrap_or_else(|| "draft".into());
    check_status(&status)?;
    check_icon(&b.icon_url)?;
    let variant = b.variant.unwrap_or_else(|| "single".into());
    let allowed = matches!((kind, variant.as_str()), (_, "single") | ("expert", "team") | ("skill", "bundle"));
    if !allowed {
        return Err(AppError::BadRequest("专家团只能是专家类型，套件只能是技能类型".into()));
    }
    let members = validate_members(&state, &variant, &b.members, None).await?;
    let mut metadata = if b.metadata.is_object() { b.metadata } else { json!({}) };
    if variant != "single" {
        metadata["members"] = json!(members);
        if status == "published" {
            check_members_published(&state, &variant, &members).await?;
        }
    }
    let tags = clean_tags(b.tags);
    let search = items::search_text(&b.name, &b.slug, &namespace, &b.summary, &tags, &category_name(&state, b.category_id).await?);
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO items (kind, namespace, slug, name, summary, description, category_id, tags, icon_url, source, license, status, prompts, metadata, search_text, published_at, variant)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15, CASE WHEN $12='published' THEN now() END, $16) RETURNING id",
    )
    .bind(kind)
    .bind(&namespace)
    .bind(&b.slug)
    .bind(b.name.trim())
    .bind(&b.summary)
    .bind(&b.description)
    .bind(b.category_id)
    .bind(&tags)
    .bind(b.icon_url.filter(|u| !u.is_empty()))
    .bind(&source)
    .bind(b.license)
    .bind(&status)
    .bind(PgJson(json!(b.prompts)))
    .bind(PgJson(metadata))
    .bind(&search)
    .bind(&variant)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(d) if d.is_unique_violation() => AppError::Conflict("该标识已存在".into()),
        other => other.into(),
    })?;
    Ok(Json(json!({ "id": id })))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ItemUpdate {
    name: Option<String>,
    summary: Option<String>,
    description: Option<String>,
    category_id: Option<i32>,
    tags: Option<Vec<String>>,
    icon_url: Option<String>,
    license: Option<String>,
    source: Option<String>,
    featured: Option<bool>,
    prompts: Option<Vec<String>>,
    members: Option<Vec<MemberRef>>,
}

async fn update_item(State(state): State<AppState>, _admin: Admin, Path(id): Path<Uuid>, Json(b): Json<ItemUpdate>) -> Result<Json<Value>> {
    #[derive(sqlx::FromRow)]
    struct Cur {
        namespace: String,
        slug: String,
        name: String,
        summary: String,
        tags: Vec<String>,
        category_id: Option<i32>,
        variant: String,
        status: String,
    }
    let cur: Cur = sqlx::query_as("SELECT namespace, slug, name, summary, tags, category_id, variant, status FROM items WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("条目不存在".into()))?;
    if let Some(s) = &b.source {
        check_source(s)?;
    }
    check_icon(&b.icon_url)?;
    if b.name.as_deref().is_some_and(|n| n.trim().is_empty()) {
        return Err(AppError::BadRequest("名称不能为空".into()));
    }
    let members = match &b.members {
        Some(list) => {
            let checked = validate_members(&state, &cur.variant, list, Some(id)).await?;
            if cur.status == "published" {
                check_members_published(&state, &cur.variant, &checked).await?;
            }
            Some(checked)
        }
        None => None,
    };
    let name = b.name.map(|n| n.trim().to_string()).unwrap_or(cur.name);
    let summary = b.summary.unwrap_or(cur.summary);
    let tags = b.tags.map(clean_tags).unwrap_or(cur.tags);
    let category = b.category_id.or(cur.category_id);
    let search = items::search_text(&name, &cur.slug, &cur.namespace, &summary, &tags, &category_name(&state, category).await?);
    sqlx::query(
        "UPDATE items SET name=$2, summary=$3, tags=$4, category_id=$5, search_text=$6, description=COALESCE($7, description),
           icon_url=COALESCE($8, icon_url), license=COALESCE($9, license), source=COALESCE($10, source), featured=COALESCE($11, featured),
           prompts=COALESCE($12, prompts), metadata = CASE WHEN $13::jsonb IS NULL THEN metadata ELSE jsonb_set(metadata, '{members}', $13::jsonb) END,
           updated_at=now() WHERE id=$1",
    )
    .bind(id)
    .bind(&name)
    .bind(&summary)
    .bind(&tags)
    .bind(category)
    .bind(&search)
    .bind(b.description)
    .bind(b.icon_url)
    .bind(b.license)
    .bind(b.source)
    .bind(b.featured)
    .bind(b.prompts.map(|p| PgJson(json!(p.into_iter().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).take(8).collect::<Vec<_>>()))))
    .bind(members.map(|m| PgJson(json!(m))))
    .execute(&state.pool)
    .await?;
    Ok(Json(json!({ "ok": true })))
}

#[derive(Deserialize)]
struct StatusBody {
    status: String,
}

/// 团队 / 套件上架前，所有成员都必须已上架，否则前台会出现打不开的成员。
async fn check_members_published(state: &AppState, variant: &str, members: &[Value]) -> Result<()> {
    let missing: Vec<String> = sqlx::query_scalar(
        "SELECT (m.v->>'namespace') || '/' || (m.v->>'slug') FROM jsonb_array_elements($1::jsonb) AS m(v)
         WHERE NOT EXISTS (SELECT 1 FROM items i WHERE i.kind = $2 AND i.namespace = m.v->>'namespace' AND i.slug = m.v->>'slug' AND i.status = 'published')",
    )
    .bind(PgJson(json!(members)))
    .bind(member_kind(variant))
    .fetch_all(&state.pool)
    .await?;
    if missing.is_empty() {
        Ok(())
    } else {
        Err(AppError::BadRequest(format!("这些成员还没有上架：{}", missing.join("、"))))
    }
}

async fn set_status(State(state): State<AppState>, _admin: Admin, Path(id): Path<Uuid>, Json(b): Json<StatusBody>) -> Result<Json<Value>> {
    check_status(&b.status)?;
    let (kind, variant, members): (String, String, Option<PgJson<Value>>) = sqlx::query_as("SELECT kind, variant, metadata->'members' FROM items WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("条目不存在".into()))?;
    if b.status == "published" && variant != "single" {
        let list: Vec<Value> = members.and_then(|m| m.0.as_array().cloned()).unwrap_or_default();
        check_members_published(&state, &variant, &list).await?;
    }
    // 套件没有自己的安装包，由成员各自安装；单个技能才要求有可用版本。
    if b.status == "published" && kind == "skill" && variant == "single" {
        let usable: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM versions WHERE item_id = $1 AND NOT yanked)").bind(id).fetch_one(&state.pool).await?;
        if !usable {
            return Err(AppError::BadRequest("技能需要至少一个未撤回的版本才能上架".into()));
        }
    }
    sqlx::query("UPDATE items SET status=$2, updated_at=now(), published_at = CASE WHEN $2='published' THEN COALESCE(published_at, now()) ELSE published_at END WHERE id=$1")
        .bind(id)
        .bind(&b.status)
        .execute(&state.pool)
        .await?;
    Ok(Json(json!({ "ok": true })))
}

async fn delete_item(State(state): State<AppState>, _admin: Admin, Path(id): Path<Uuid>) -> Result<Json<Value>> {
    let n = sqlx::query("DELETE FROM items WHERE id = $1").bind(id).execute(&state.pool).await?.rows_affected();
    if n == 0 {
        return Err(AppError::NotFound("条目不存在".into()));
    }
    Ok(Json(json!({ "ok": true })))
}

async fn upload_version(State(state): State<AppState>, _admin: Admin, Path(id): Path<Uuid>, mut multipart: Multipart) -> Result<Json<Value>> {
    let (kind, item_slug): (String, String) = sqlx::query_as("SELECT kind, slug FROM items WHERE id = $1").bind(id).fetch_optional(&state.pool).await?.ok_or_else(|| AppError::NotFound("条目不存在".into()))?;
    if kind != "skill" {
        return Err(AppError::BadRequest("只有技能可以上传安装包".into()));
    }
    let (mut version, mut changelog, mut file) = (String::new(), String::new(), None);
    while let Some(field) = multipart.next_field().await.map_err(|e| AppError::BadRequest(format!("上传内容无效：{e}")))? {
        match field.name().unwrap_or("") {
            "version" => version = field.text().await.map_err(|e| AppError::BadRequest(e.to_string()))?,
            "changelog" => changelog = field.text().await.map_err(|e| AppError::BadRequest(e.to_string()))?,
            "file" => file = Some(field.bytes().await.map_err(|e| AppError::BadRequest(format!("读取文件失败：{e}")))?),
            _ => {}
        }
    }
    let version = version.trim().to_string();
    if version.is_empty() || version.len() > 44 || !version.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'+')) {
        return Err(AppError::BadRequest("版本号只能含字母、数字、. - +，最长 44 位".into()));
    }
    let bytes = file.ok_or_else(|| AppError::BadRequest("缺少 file 字段（ZIP 安装包）".into()))?;
    let bytes = package::prepare_package(&bytes, &item_slug).map_err(|e| AppError::BadRequest(format!("安装包未通过校验：{e}")))?;
    let info = package::inspect_zip(&bytes).map_err(|e| AppError::BadRequest(format!("安装包未通过校验：{e}")))?;
    let sha = package::sha256_hex(&bytes);
    storage::store_package(&state.config.data_dir, &sha, &bytes).await.map_err(|e| AppError::Internal(format!("保存安装包失败：{e}")))?;
    let inserted = sqlx::query(
        "INSERT INTO versions (item_id, version, changelog, manifest, manifest_sha256, package_sha256, package_size) VALUES ($1,$2,$3,$4,$5,$6,$7)",
    )
    .bind(id)
    .bind(&version)
    .bind(changelog.trim())
    .bind(PgJson(json!({ "files": info.files })))
    .bind(&info.manifest_sha256)
    .bind(&sha)
    .bind(bytes.len() as i64)
    .execute(&state.pool)
    .await;
    match inserted {
        Err(sqlx::Error::Database(d)) if d.is_unique_violation() => return Err(AppError::Conflict("该版本号已存在，请使用新的版本号".into())),
        Err(e) => return Err(e.into()),
        Ok(_) => {}
    }
    refresh_latest(&state, id).await?;
    let (name, description) = package::frontmatter(&info.skill_md);
    Ok(Json(json!({ "version": version, "manifestSha256": info.manifest_sha256, "packageSha256": sha, "files": info.files.len(), "skillName": name, "skillDescription": description })))
}

async fn refresh_latest(state: &AppState, item_id: Uuid) -> Result<()> {
    sqlx::query(
        "UPDATE items SET updated_at = now(),
           latest_version = (SELECT version FROM versions WHERE item_id = $1 AND NOT yanked ORDER BY created_at DESC, version DESC LIMIT 1)
         WHERE id = $1",
    )
    .bind(item_id)
    .execute(&state.pool)
    .await?;
    Ok(())
}

#[derive(Deserialize)]
struct YankBody {
    yanked: bool,
}

async fn update_version(State(state): State<AppState>, _admin: Admin, Path(id): Path<Uuid>, Json(b): Json<YankBody>) -> Result<Json<Value>> {
    let item_id: Uuid = sqlx::query_scalar("UPDATE versions SET yanked = $2 WHERE id = $1 RETURNING item_id")
        .bind(id)
        .bind(b.yanked)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("版本不存在".into()))?;
    refresh_latest(&state, item_id).await?;
    Ok(Json(json!({ "ok": true })))
}

// ───────────── 分类 ─────────────

#[derive(Deserialize, Default)]
struct CategoryQuery {
    kind: Option<String>,
}

async fn list_categories(State(state): State<AppState>, _admin: Admin, Query(q): Query<CategoryQuery>) -> Result<Json<Value>> {
    let kind = match q.kind.filter(|k| !k.is_empty()) {
        Some(k) => Some(super::public::kind_of(&k)?),
        None => None,
    };
    let rows = unwrap(
        sqlx::query_scalar::<_, PgJson<Value>>(
            "SELECT to_jsonb(x) FROM (SELECT c.id, c.kind, c.slug, c.name, c.name_en AS \"nameEn\", c.sort_order AS \"sortOrder\", c.active, count(i.id) AS items
             FROM categories c LEFT JOIN items i ON i.category_id = c.id WHERE ($1::text IS NULL OR c.kind = $1)
             GROUP BY c.id ORDER BY c.kind, c.sort_order, c.id) x",
        )
        .bind(kind)
        .fetch_all(&state.pool)
        .await?,
    );
    Ok(Json(json!({ "results": rows })))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct NewCategory {
    kind: String,
    slug: Option<String>,
    name: String,
    #[serde(default)]
    name_en: String,
    #[serde(default)]
    sort_order: i32,
}

async fn create_category(State(state): State<AppState>, _admin: Admin, Json(b): Json<NewCategory>) -> Result<Json<Value>> {
    let kind = super::public::kind_of(&b.kind)?;
    if b.name.trim().is_empty() {
        return Err(AppError::BadRequest("分类名称不能为空".into()));
    }
    let slug = b.slug.filter(|s| !s.is_empty()).unwrap_or_else(|| upstream::slugify(&b.name, &b.name));
    let id: i32 = sqlx::query_scalar("INSERT INTO categories (kind, slug, name, name_en, sort_order) VALUES ($1,$2,$3,$4,$5) RETURNING id")
        .bind(kind)
        .bind(&slug)
        .bind(b.name.trim())
        .bind(&b.name_en)
        .bind(b.sort_order)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(d) if d.is_unique_violation() => AppError::Conflict("该分类标识已存在".into()),
            other => other.into(),
        })?;
    Ok(Json(json!({ "id": id, "slug": slug })))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CategoryUpdate {
    name: Option<String>,
    name_en: Option<String>,
    sort_order: Option<i32>,
    active: Option<bool>,
}

async fn update_category(State(state): State<AppState>, _admin: Admin, Path(id): Path<i32>, Json(b): Json<CategoryUpdate>) -> Result<Json<Value>> {
    let n = sqlx::query(
        "UPDATE categories SET name=COALESCE($2,name), name_en=COALESCE($3,name_en), sort_order=COALESCE($4,sort_order), active=COALESCE($5,active) WHERE id=$1",
    )
    .bind(id)
    .bind(b.name.map(|n| n.trim().to_string()).filter(|n| !n.is_empty()))
    .bind(b.name_en)
    .bind(b.sort_order)
    .bind(b.active)
    .execute(&state.pool)
    .await?
    .rows_affected();
    if n == 0 {
        return Err(AppError::NotFound("分类不存在".into()));
    }
    Ok(Json(json!({ "ok": true })))
}

async fn delete_category(State(state): State<AppState>, _admin: Admin, Path(id): Path<i32>) -> Result<Json<Value>> {
    // 条目的 category_id 会被置空，不会删除条目。
    let n = sqlx::query("DELETE FROM categories WHERE id = $1").bind(id).execute(&state.pool).await?.rows_affected();
    if n == 0 {
        return Err(AppError::NotFound("分类不存在".into()));
    }
    Ok(Json(json!({ "ok": true })))
}
