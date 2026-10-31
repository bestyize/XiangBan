//! Owner-only, explicitly selected community references. No arbitrary settings or files are accepted.
use crate::{
    auth::AuthUser,
    error::{AppError, AppResult},
    state::AppState,
};
use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn init(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::raw_sql("CREATE TABLE IF NOT EXISTS scheme_backups(owner_uid UUID NOT NULL REFERENCES users(uid),id UUID NOT NULL,title TEXT NOT NULL,summary TEXT NOT NULL,items JSONB NOT NULL,revision BIGINT NOT NULL DEFAULT 1,status TEXT NOT NULL DEFAULT 'active' CHECK(status IN ('active','removed')),updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),PRIMARY KEY(owner_uid,id));CREATE INDEX IF NOT EXISTS scheme_backups_owner_updated ON scheme_backups(owner_uid,updated_at DESC);").execute(pool).await?;
    Ok(())
}
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/me/scheme-backups", get(list))
        .route("/me/scheme-backups/{id}", get(detail).put(save))
        .route("/me/scheme-backups/{id}/remove", post(remove))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Item {
    pack_id: i64,
    version: i32,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    schema_version: u32,
    title: String,
    #[serde(default)]
    summary: String,
    items: Vec<Item>,
    expected_revision: i64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Revision {
    expected_revision: i64,
}
#[derive(Deserialize)]
struct List {
    page: Option<i64>,
    page_size: Option<i64>,
    status: Option<String>,
}
type Row = (Uuid, String, String, Value, i64, String, DateTime<Utc>);
fn text(raw: &str, max: usize, required: bool) -> AppResult<String> {
    let v = raw.trim();
    if (required && v.is_empty())
        || v.chars().count() > max
        || v.chars().any(|c| c.is_control() && c != '\n')
    {
        return Err(AppError::BadRequest("方案名称或简介无效".into()));
    }
    Ok(v.to_string())
}
async fn render(s: &AppState, row: Row) -> AppResult<Value> {
    let (id, title, summary, refs, revision, status, updated_at) = row;
    let items = crate::handlers::content::scheme_items(s, &refs).await?;
    Ok(
        json!({"id":id,"schema_version":1,"title":title,"summary":summary,"items":items,"revision":revision,"status":status,"updated_at":updated_at}),
    )
}
async fn list(
    State(s): State<AppState>,
    auth: AuthUser,
    Query(q): Query<List>,
) -> AppResult<Json<Value>> {
    let page = q.page.unwrap_or(1).clamp(1, 100_000);
    let size = q.page_size.unwrap_or(20).clamp(1, 50);
    let status = q.status.unwrap_or_else(|| "active".into());
    if !["active", "removed"].contains(&status.as_str()) {
        return Err(AppError::BadRequest("未知备份状态".into()));
    }
    let (total,): (i64,) =
        sqlx::query_as("SELECT count(*) FROM scheme_backups WHERE owner_uid=$1 AND status=$2")
            .bind(auth.user.uid)
            .bind(&status)
            .fetch_one(&s.pool)
            .await?;
    let rows:Vec<Row>=sqlx::query_as("SELECT id,title,summary,items,revision,status,updated_at FROM scheme_backups WHERE owner_uid=$1 AND status=$2 ORDER BY updated_at DESC,id DESC LIMIT $3 OFFSET $4").bind(auth.user.uid).bind(&status).bind(size).bind((page-1)*size).fetch_all(&s.pool).await?;
    let mut items = Vec::new();
    for row in rows {
        items.push(render(&s, row).await?)
    }
    Ok(Json(
        json!({"items":items,"page":page,"page_size":size,"total":total}),
    ))
}
async fn detail(
    State(s): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    let row:Option<Row>=sqlx::query_as("SELECT id,title,summary,items,revision,status,updated_at FROM scheme_backups WHERE owner_uid=$1 AND id=$2").bind(auth.user.uid).bind(id).fetch_optional(&s.pool).await?;
    Ok(Json(
        render(&s, row.ok_or_else(|| AppError::not_found("方案备份"))?).await?,
    ))
}
async fn save(
    State(s): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(input): Json<Input>,
) -> AppResult<Json<Value>> {
    if input.schema_version != 1
        || input.expected_revision < 0
        || input.items.is_empty()
        || input.items.len() > 20
    {
        return Err(AppError::BadRequest(
            "方案需要1～20个作品引用及有效版本".into(),
        ));
    }
    let title = text(&input.title, 60, true)?;
    let summary = text(&input.summary, 500, false)?;
    let mut unique = std::collections::HashSet::new();
    let mut refs = Vec::new();
    for item in input.items {
        if item.pack_id <= 0 || item.version <= 0 || !unique.insert(item.pack_id) {
            return Err(AppError::BadRequest("作品引用无效或重复".into()));
        }
        // Same public policy as pack links; saving does not grant access to removed or restricted content.
        if crate::handlers::content::public_content_card(
            &s,
            "pack",
            item.pack_id,
            Some(item.version),
        )
        .await?
        .is_none()
        {
            return Err(AppError::BadRequest("选择的作品当前不可公开访问".into()));
        }
        refs.push(json!({"pack_id":item.pack_id,"version":item.version}));
    }
    let mut tx = s.pool.begin().await?;
    sqlx::query("SELECT uid FROM users WHERE uid=$1 FOR UPDATE")
        .bind(auth.user.uid)
        .execute(&mut *tx)
        .await?;
    let old: Option<(i64,)> = sqlx::query_as(
        "SELECT revision FROM scheme_backups WHERE owner_uid=$1 AND id=$2 FOR UPDATE",
    )
    .bind(auth.user.uid)
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?;
    if old.map_or(0, |v| v.0) != input.expected_revision {
        return Err(AppError::Conflict(
            "云端方案已变化，请读取最新版本后再保存".into(),
        ));
    }
    let (count,): (i64,) = sqlx::query_as(
        "SELECT count(*) FROM scheme_backups WHERE owner_uid=$1 AND status='active' AND id<>$2",
    )
    .bind(auth.user.uid)
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;
    let (stored,): (i64,) =
        sqlx::query_as("SELECT count(*) FROM scheme_backups WHERE owner_uid=$1 AND id<>$2")
            .bind(auth.user.uid)
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
    if stored >= 200 {
        return Err(AppError::BadRequest(
            "云端方案及回收站最多200份，请复用已有备份".into(),
        ));
    }
    if count >= 100 {
        return Err(AppError::BadRequest(
            "最多保存100份云端方案，请先移除不再需要的备份".into(),
        ));
    }
    let (revision,):(i64,)=sqlx::query_as("INSERT INTO scheme_backups(owner_uid,id,title,summary,items) VALUES($1,$2,$3,$4,$5) ON CONFLICT(owner_uid,id) DO UPDATE SET title=$3,summary=$4,items=$5,revision=scheme_backups.revision+1,status='active',updated_at=now() RETURNING revision").bind(auth.user.uid).bind(id).bind(title).bind(summary).bind(Value::Array(refs)).fetch_one(&mut *tx).await?;
    tx.commit().await?;
    Ok(Json(json!({"id":id,"revision":revision,"status":"active"})))
}
async fn remove(
    State(s): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(input): Json<Revision>,
) -> AppResult<Json<Value>> {
    let row:Option<(i64,)>=sqlx::query_as("UPDATE scheme_backups SET status='removed',revision=revision+1,updated_at=now() WHERE owner_uid=$1 AND id=$2 AND revision=$3 RETURNING revision").bind(auth.user.uid).bind(id).bind(input.expected_revision).fetch_optional(&s.pool).await?;
    let (revision,) =
        row.ok_or_else(|| AppError::Conflict("备份不存在或版本已变化，请刷新后重试".into()))?;
    Ok(Json(
        json!({"id":id,"revision":revision,"status":"removed"}),
    ))
}

#[cfg(test)]
#[path = "scheme_backups_tests.rs"]
mod tests;
