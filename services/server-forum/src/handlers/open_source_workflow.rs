use super::{
    admin::{audit, AdminIdentity, AuthAdmin},
    open_source, open_source_rules as rules,
};
use crate::{
    error::{AppError, AppResult},
    state::AppState,
};
use axum::{
    extract::{DefaultBodyLimit, State},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::{PgConnection, PgPool};

pub const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS open_source_catalog (
    singleton BOOLEAN PRIMARY KEY DEFAULT true CHECK(singleton),
    revision BIGINT NOT NULL DEFAULT 0,
    draft JSONB NOT NULL,
    published JSONB,
    published_revision BIGINT NOT NULL DEFAULT 0,
    updated_by TEXT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    published_at TIMESTAMPTZ
);
CREATE TABLE IF NOT EXISTS open_source_assets (
    id UUID PRIMARY KEY,
    project TEXT NOT NULL,
    mime TEXT NOT NULL,
    data BYTEA NOT NULL,
    source_url TEXT NOT NULL DEFAULT '',
    created_by TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS open_source_assets_project_idx ON open_source_assets(project,created_at DESC);
"#;
pub fn admin_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(read).put(save))
        .route("/preview", post(preview))
        .route("/publish", post(publish))
        .route("/import-github", post(super::open_source_import::import))
        .merge(super::open_source_assets::admin_routes())
        .layer(DefaultBodyLimit::max(8 * 1024 * 1024))
}
pub async fn public_catalog(pool: &PgPool) -> AppResult<Value> {
    let row: Option<(Value,i64,Option<DateTime<Utc>>)> = sqlx::query_as("SELECT published,published_revision,published_at FROM open_source_catalog WHERE singleton AND published IS NOT NULL").fetch_optional(pool).await?;
    let (mut value, revision, published_at) = if let Some(row) = row {
        row
    } else {
        (
            rules::normalize(open_source::catalog().clone(), true)?,
            0,
            None,
        )
    };
    value["revision"] = json!(revision);
    value["published_at"] = json!(published_at);
    Ok(value)
}
pub async fn contains(pool: &PgPool, name: &str) -> AppResult<bool> {
    Ok(public_catalog(pool).await?["projects"]
        .as_array()
        .is_some_and(|items| {
            items.iter().any(|p| {
                p["full_name"]
                    .as_str()
                    .is_some_and(|v| v.eq_ignore_ascii_case(name))
            })
        }))
}
async fn admin_data(pool: &PgPool) -> AppResult<Value> {
    let row: Option<(i64, Value, i64)> = sqlx::query_as(
        "SELECT revision,draft,published_revision FROM open_source_catalog WHERE singleton",
    )
    .fetch_optional(pool)
    .await?;
    let (revision, draft, published_revision) = row.unwrap_or((
        0,
        rules::normalize(open_source::catalog().clone(), false)?,
        0,
    ));
    Ok(
        json!({"scope":"open-source","revision":revision,"published_revision":published_revision,"draft":draft,"live":public_catalog(pool).await?,"cover_rules":rules::manifest()}),
    )
}
async fn read(State(s): State<AppState>, _: AuthAdmin) -> AppResult<Json<Value>> {
    Ok(Json(admin_data(&s.pool).await?))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Save {
    expected_revision: i64,
    data: Value,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Publish {
    expected_revision: i64,
}
async fn lock(conn: &mut PgConnection) -> AppResult<()> {
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended('open-source-catalog',0))")
        .execute(conn)
        .await?;
    Ok(())
}
async fn save(
    State(s): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Json(input): Json<Save>,
) -> AppResult<Json<Value>> {
    let value = rules::normalize(input.data, false)?;
    let mut tx = s.pool.begin().await?;
    lock(&mut tx).await?;
    let old: Option<(i64,)> =
        sqlx::query_as("SELECT revision FROM open_source_catalog WHERE singleton FOR UPDATE")
            .fetch_optional(&mut *tx)
            .await?;
    if old.map_or(0, |v| v.0) != input.expected_revision {
        return Err(AppError::Conflict(
            "精选草稿已被其他编辑修改，请重新读取后合并".into(),
        ));
    }
    sqlx::query("INSERT INTO open_source_catalog(singleton,revision,draft,updated_by) VALUES(true,$1,$2,$3) ON CONFLICT(singleton) DO UPDATE SET revision=$1,draft=$2,updated_by=$3,updated_at=clock_timestamp()")
        .bind(input.expected_revision+1).bind(value).bind(&admin.username).execute(&mut *tx).await?;
    audit(
        &mut tx,
        &admin,
        "open-source.draft.save",
        "open-source",
        "catalog",
        format!("revision={}", input.expected_revision + 1),
    )
    .await?;
    tx.commit().await?;
    Ok(Json(admin_data(&s.pool).await?))
}
async fn preview(
    State(s): State<AppState>,
    _: AuthAdmin,
    Json(input): Json<Value>,
) -> AppResult<Json<Value>> {
    let value = rules::normalize(input, true)?;
    let mut conn = s.pool.acquire().await?;
    super::open_source_assets::validate_refs(&mut conn, &value).await?;
    Ok(Json(value))
}
async fn publish(
    State(s): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Json(input): Json<Publish>,
) -> AppResult<Json<Value>> {
    publish_revision(&s.pool, &admin, input.expected_revision).await?;
    Ok(Json(admin_data(&s.pool).await?))
}
pub async fn publish_revision(
    pool: &PgPool,
    admin: &AdminIdentity,
    revision: i64,
) -> AppResult<()> {
    let mut tx = pool.begin().await?;
    lock(&mut tx).await?;
    let old: Option<(i64, Value)> =
        sqlx::query_as("SELECT revision,draft FROM open_source_catalog WHERE singleton FOR UPDATE")
            .fetch_optional(&mut *tx)
            .await?;
    let (current, draft) = old.ok_or_else(|| AppError::BadRequest("请先保存精选草稿".into()))?;
    if revision != current {
        return Err(AppError::Conflict(
            "草稿版本已变化，请重新预览后发布".into(),
        ));
    }
    let mut value = rules::normalize(draft, true)?;
    super::open_source_assets::validate_refs(&mut tx, &value).await?;
    value["reviewed_at"] = json!(Utc::now()
        .with_timezone(&chrono_tz::Asia::Shanghai)
        .date_naive()
        .to_string());
    sqlx::query("UPDATE open_source_catalog SET draft=$1,published=$1,published_revision=$2,published_at=clock_timestamp() WHERE singleton").bind(value).bind(revision).execute(&mut *tx).await?;
    audit(
        &mut tx,
        admin,
        "open-source.publish",
        "open-source",
        "catalog",
        format!("revision={revision}"),
    )
    .await?;
    tx.commit().await?;
    Ok(())
}
