//! 管理员采集配置与整批终审，采集令牌不可访问。
use super::{
    admin::{audit, AuthAdmin},
    discover_editor,
};
use crate::{
    collectors::{self, credentials},
    error::{AppError, AppResult},
    state::AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/sources", get(sources))
        .route("/sources/{id}", axum::routing::patch(configure))
        .route("/sources/{id}/credential", axum::routing::put(credential))
        .route("/sources/{id}/run", post(run))
        .route("/runs", get(runs))
        .route("/runs/{id}/drafts", get(drafts))
        .route("/runs/{id}/review", post(review))
}
#[derive(sqlx::FromRow, Serialize)]
struct Source {
    id: String,
    configured: bool,
    config: Value,
    hint: Option<String>,
    enabled: bool,
    interval_minutes: i32,
    last_success_at: Option<DateTime<Utc>>,
    last_error: Option<String>,
    next_run_at: DateTime<Utc>,
}
async fn sources(State(s): State<AppState>, _: AuthAdmin) -> AppResult<Json<Value>> {
    let items:Vec<Source>=sqlx::query_as("SELECT id,credential IS NOT NULL AS configured,config,hint,enabled,interval_minutes,last_success_at,last_error,next_run_at FROM collection_sources ORDER BY id").fetch_all(&s.pool).await?;
    Ok(Json(json!({"items":items.into_iter().map(|source| {
        let title=collectors::title(&source.id).unwrap_or("未知来源");
        let mut value=serde_json::to_value(source).unwrap(); value["title"]=json!(title); value
    }).collect::<Vec<_>>()})))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Credential {
    key: String,
}
async fn credential(
    State(s): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Path(id): Path<String>,
    Json(input): Json<Credential>,
) -> AppResult<Json<Value>> {
    collectors::check_source(&id)?;
    let secret = input.key.trim();
    if !(8..=4096).contains(&secret.len()) || !secret.bytes().all(|b| b.is_ascii_graphic()) {
        return Err(AppError::BadRequest(
            "密钥应为 8–4096 个非空白 ASCII 字符".into(),
        ));
    }
    // 先检查主密钥并加密，再调用官方验证，任何失败均不覆盖现有凭据。
    let sealed = credentials::encrypt(&id, secret)?;
    let http = credentials::http_client()?;
    credentials::validate(&id, secret, &http).await?;
    let hint = credentials::hint(secret);
    let mut tx = s.pool.begin().await?;
    sqlx::query("UPDATE collection_sources SET credential=$2,hint=$3,last_error=NULL,updated_at=now() WHERE id=$1")
        .bind(&id).bind(sealed).bind(&hint).execute(&mut *tx).await?;
    audit(
        &mut tx,
        &admin,
        "collection.credential.update",
        "collection_source",
        &id,
        "凭据验证成功并加密保存",
    )
    .await?;
    tx.commit().await?;
    Ok(Json(json!({"configured":true,"hint":hint})))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Configuration {
    enabled: Option<bool>,
    interval_minutes: Option<i32>,
    config: Option<Value>,
}
async fn configure(
    State(s): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Path(id): Path<String>,
    Json(input): Json<Configuration>,
) -> AppResult<Json<Value>> {
    collectors::check_source(&id)?;
    if input.enabled.is_none() && input.interval_minutes.is_none() && input.config.is_none() {
        return Err(AppError::BadRequest("请提供采集配置".into()));
    }
    if input
        .interval_minutes
        .is_some_and(|m| !(60..=10080).contains(&m))
    {
        return Err(AppError::BadRequest("采集间隔应为 60–10080 分钟".into()));
    }
    let mut tx = s.pool.begin().await?;
    let old: Option<(Option<Vec<u8>>, Value, bool)> = sqlx::query_as(
        "SELECT credential,config,enabled FROM collection_sources WHERE id=$1 FOR UPDATE",
    )
    .bind(&id)
    .fetch_optional(&mut *tx)
    .await?;
    let (sealed, old_config, old_enabled) = old.ok_or_else(|| AppError::not_found("采集来源"))?;
    let config = match input.config {
        Some(config) if id == "github_repos" => collectors::repo_sources::sanitize_config(config)?,
        Some(config) if config == json!({}) => config,
        Some(_) => return Err(AppError::BadRequest("此来源不支持额外配置".into())),
        None => old_config,
    };
    if id == "github_repos" && input.enabled.unwrap_or(old_enabled) {
        collectors::repo_sources::configured_names(&config)?;
    }
    if input.enabled == Some(true) {
        let sealed = sealed.ok_or_else(|| AppError::BadRequest("请先配置来源凭据".into()))?;
        credentials::decrypt(&id, &sealed)?;
    }
    sqlx::query("UPDATE collection_sources SET enabled=COALESCE($2,enabled),interval_minutes=COALESCE($3,interval_minutes),config=$4,next_run_at=CASE WHEN $3::int=1440 THEN date_trunc('day',(now()-interval '4 hours') AT TIME ZONE 'UTC') AT TIME ZONE 'UTC'+interval '1 day 4 hours' WHEN $3::int IS NOT NULL THEN now()+make_interval(mins=>$3) ELSE next_run_at END,updated_at=now() WHERE id=$1")
        .bind(&id).bind(input.enabled).bind(input.interval_minutes).bind(config).execute(&mut *tx).await?;
    audit(
        &mut tx,
        &admin,
        "collection.configure",
        "collection_source",
        &id,
        format!(
            "enabled={:?}; interval_minutes={:?}",
            input.enabled, input.interval_minutes
        ),
    )
    .await?;
    tx.commit().await?;
    Ok(Json(json!({"ok":true})))
}
async fn run(
    State(s): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Path(id): Path<String>,
) -> AppResult<(StatusCode, Json<Value>)> {
    let run_id = collectors::start(s.clone(), id.clone(), false).await?;
    let mut tx = s.pool.begin().await?;
    audit(
        &mut tx,
        &admin,
        "collection.run",
        "collection_source",
        &id,
        format!("run={run_id}"),
    )
    .await?;
    tx.commit().await?;
    Ok((StatusCode::ACCEPTED, Json(json!({"run_id":run_id}))))
}
#[derive(Deserialize)]
struct Filter {
    source: Option<String>,
    page: Option<i64>,
}
#[derive(sqlx::FromRow, Serialize)]
struct Run {
    id: Uuid,
    source: String,
    status: String,
    started_at: DateTime<Utc>,
    finished_at: Option<DateTime<Utc>>,
    item_count: i32,
    error: Option<String>,
    reviewed_at: Option<DateTime<Utc>>,
}
async fn runs(
    State(s): State<AppState>,
    _: AuthAdmin,
    Query(input): Query<Filter>,
) -> AppResult<Json<Value>> {
    if let Some(source) = &input.source {
        collectors::check_source(source)?;
    }
    let page = input.page.unwrap_or(1).clamp(1, 100000);
    let items:Vec<Run>=sqlx::query_as("SELECT id,source,status,started_at,finished_at,item_count,error,reviewed_at FROM collection_runs WHERE ($1::text IS NULL OR source=$1) ORDER BY started_at DESC,id DESC LIMIT 30 OFFSET $2")
        .bind(&input.source).bind((page-1)*30).fetch_all(&s.pool).await?;
    let (total,): (i64,) = sqlx::query_as(
        "SELECT count(*) FROM collection_runs WHERE ($1::text IS NULL OR source=$1)",
    )
    .bind(&input.source)
    .fetch_one(&s.pool)
    .await?;
    Ok(Json(
        json!({"items":items,"total":total,"page":page,"page_size":30}),
    ))
}
async fn drafts(
    State(s): State<AppState>,
    _: AuthAdmin,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    Ok(Json(
        discover_editor::collected_batch_preview(&s.pool, id).await?,
    ))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Review {
    items: Vec<discover_editor::BatchReviewItem>,
}
async fn review(
    State(s): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Path(id): Path<Uuid>,
    Json(input): Json<Review>,
) -> AppResult<Json<Value>> {
    if input.items.len() > 1000 {
        return Err(AppError::BadRequest("每批最多 1000 项".into()));
    }
    let count =
        discover_editor::review_collected_batch(&s.pool, id, &admin.username, input.items).await?;
    Ok(Json(json!({"status":"published","item_count":count})))
}

#[cfg(test)]
#[path = "collection_tests.rs"]
mod tests;
