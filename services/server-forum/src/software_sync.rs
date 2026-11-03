//! Refresh the software directory independently of page traffic.
use crate::{
    collectors::credentials,
    error::{AppError, AppResult},
    handlers::admin::{audit, AuthAdmin},
    state::AppState,
};
use axum::{
    extract::{Path, State},
    routing::{delete, get},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::PgPool;
use std::{collections::BTreeMap, sync::LazyLock, time::Duration};

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS open_source_software_tokens (
    id uuid PRIMARY KEY,
    label text NOT NULL,
    credential bytea NOT NULL,
    hint text NOT NULL,
    last_used_at timestamptz
);
CREATE TABLE IF NOT EXISTS open_source_software_sync (
    project text PRIMARY KEY,
    next_at timestamptz NOT NULL,
    synced_at timestamptz,
    attempted_at timestamptz,
    snapshot jsonb
);
ALTER TABLE open_source_software_sync ADD COLUMN IF NOT EXISTS attempted_at timestamptz;
CREATE TABLE IF NOT EXISTS open_source_software_stars (
    project text NOT NULL REFERENCES open_source_software_sync(project) ON DELETE CASCADE,
    day date NOT NULL,
    stars bigint NOT NULL,
    PRIMARY KEY (project, day)
);
";

const TOKEN_SOURCE: &str = "software_github";

pub fn admin_routes() -> Router<AppState> {
    Router::new()
        .route("/software-tokens", get(list_tokens).post(add_token))
        .route("/software-tokens/{id}", delete(remove_token))
}

#[derive(sqlx::FromRow, Serialize)]
struct TokenInfo {
    id: uuid::Uuid,
    label: String,
    hint: String,
    last_used_at: Option<chrono::DateTime<chrono::Utc>>,
}

async fn list_tokens(State(s): State<AppState>, _: AuthAdmin) -> AppResult<Json<Vec<TokenInfo>>> {
    Ok(Json(
        sqlx::query_as(
            "SELECT id,label,hint,last_used_at FROM open_source_software_tokens ORDER BY label,id",
        )
        .fetch_all(&s.pool)
        .await?,
    ))
}

#[derive(Deserialize)]
struct NewToken {
    label: String,
    token: String,
}

async fn add_token(
    State(s): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Json(input): Json<NewToken>,
) -> AppResult<Json<Value>> {
    let label = input.label.trim();
    let token = input.token.trim();
    if label.is_empty()
        || label.chars().count() > 80
        || !(16..=256).contains(&token.len())
        || !token
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_')
    {
        return Err(AppError::BadRequest(
            "请输入名称及有效格式的 GitHub 令牌".into(),
        ));
    }
    let sealed = credentials::encrypt(TOKEN_SOURCE, token)?;
    let response = credentials::http_client()?
        .get("https://api.github.com/rate_limit")
        .bearer_auth(token)
        .send()
        .await
        .map_err(|_| AppError::BadRequest("暂时无法连接 GitHub 验证令牌".into()))?;
    if !response.status().is_success() {
        return Err(AppError::BadRequest("GitHub 令牌验证失败".into()));
    }
    let id = uuid::Uuid::new_v4();
    let mut tx = s.pool.begin().await?;
    sqlx::query(
        "INSERT INTO open_source_software_tokens(id,label,credential,hint) VALUES ($1,$2,$3,$4)",
    )
    .bind(id)
    .bind(label)
    .bind(sealed)
    .bind(credentials::hint(token))
    .execute(&mut *tx)
    .await?;
    audit(
        &mut tx,
        &admin,
        "software.token.add",
        "software_token",
        id,
        "",
    )
    .await?;
    tx.commit().await?;
    Ok(Json(json!({"ok":true})))
}

async fn remove_token(
    State(s): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Path(id): Path<uuid::Uuid>,
) -> AppResult<Json<Value>> {
    let mut tx = s.pool.begin().await?;
    sqlx::query("DELETE FROM open_source_software_tokens WHERE id=$1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    audit(
        &mut tx,
        &admin,
        "software.token.remove",
        "software_token",
        id,
        "",
    )
    .await?;
    tx.commit().await?;
    Ok(Json(json!({"ok":true})))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Source {
    release_repo: String,
    project: Option<Value>,
}

static SOURCES: LazyLock<BTreeMap<String, Source>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../editorial/open-source-software.json"))
        .expect("valid software manifest")
});

pub fn directory_project(name: &str) -> Option<&'static Value> {
    SOURCES.get(&name.to_ascii_lowercase())?.project.as_ref()
}

pub async fn init(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::raw_sql(SCHEMA).execute(pool).await?;
    let projects: Vec<&str> = SOURCES.keys().map(String::as_str).collect();
    sqlx::query(
        "INSERT INTO open_source_software_sync(project, next_at)
        SELECT project, now() + ($2::bigint * (position - 1) / cardinality($1::text[])) * interval '1 second'
        FROM unnest($1::text[]) WITH ORDINALITY AS sources(project, position)
        ON CONFLICT DO NOTHING",
    )
    .bind(&projects)
    .bind(sync_policy().1)
    .execute(pool)
    .await?;
    Ok(())
}

fn github_token() -> Option<String> {
    std::env::var("FORUM_SOFTWARE_GITHUB_TOKEN")
        .ok()
        .filter(|token| !token.trim().is_empty())
}

fn sync_policy() -> (i64, i64) {
    if SOURCES.len() <= 400 {
        (240, 86_400)
    } else if github_token().is_some() {
        (90, 86_400)
    } else {
        (240, 259_200)
    }
}

#[derive(Deserialize)]
struct Repository {
    stargazers_count: i64,
    pushed_at: Option<String>,
}

#[derive(Deserialize, Serialize)]
struct Asset {
    name: String,
    browser_download_url: String,
    size: u64,
}

#[derive(Deserialize, Serialize)]
struct Release {
    id: u64,
    tag_name: String,
    published_at: Option<String>,
    prerelease: bool,
    assets: Vec<Asset>,
}

#[derive(Debug, thiserror::Error)]
enum FetchError {
    #[error("GitHub HTTP {0}")]
    Status(reqwest::StatusCode, Option<i64>),
    #[error("{0}")]
    Request(#[from] reqwest::Error),
}

async fn github<T: serde::de::DeserializeOwned>(
    client: &reqwest::Client,
    path: &str,
    token: Option<&str>,
) -> Result<T, FetchError> {
    let mut request = client
        .get(format!("https://api.github.com/repos/{path}"))
        .header("User-Agent", "Xiaoban-Software-Directory")
        .header("Accept", "application/vnd.github+json");
    if let Some(token) = token {
        request = request.bearer_auth(token);
    }
    let response = request.send().await?;
    if !response.status().is_success() {
        let status = response.status();
        let headers = response.headers();
        let retry = headers
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<i64>().ok());
        let reset = headers
            .get("x-ratelimit-reset")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<i64>().ok());
        let limited = status.as_u16() == 429 || status.as_u16() == 403;
        let delay = limited.then(|| {
            retry
                .or_else(|| reset.map(|ts| ts - chrono::Utc::now().timestamp()))
                .unwrap_or(900)
                .max(60)
                .saturating_add(60)
        });
        return Err(FetchError::Status(status, delay));
    }
    Ok(response.json().await?)
}

async fn fetch(
    client: &reqwest::Client,
    project: &str,
    source: &Source,
    token: Option<&str>,
) -> Result<(i64, Value), FetchError> {
    let repo: Repository = github(client, project, token).await?;
    let history: Vec<Release> = github(
        client,
        &format!("{}/releases?per_page=100", source.release_repo),
        token,
    )
    .await?;
    let latest: Option<Release> = match github(
        client,
        &format!("{}/releases/latest", source.release_repo),
        token,
    )
    .await
    {
        Ok(release) => Some(release),
        Err(FetchError::Status(reqwest::StatusCode::NOT_FOUND, _)) => None,
        Err(err) => return Err(err),
    };
    Ok((
        repo.stargazers_count,
        json!({
            "stars": repo.stargazers_count, "updatedAt": repo.pushed_at,
            "version": latest.as_ref().map(|release| &release.tag_name),
            "latest": latest, "history": history
        }),
    ))
}

fn refresh_interval(stars: i64, snapshot: &Value) -> i64 {
    const DAY: i64 = 86_400;
    if stars < 1_000 {
        return 14 * DAY;
    }
    let recent_push = snapshot["updatedAt"]
        .as_str()
        .and_then(|date| chrono::DateTime::parse_from_rfc3339(date).ok())
        .is_some_and(|date| date >= chrono::Utc::now() - chrono::Duration::days(30));
    if recent_push {
        2 * DAY
    } else {
        7 * DAY
    }
}

pub fn spawn(state: AppState) {
    let (spacing, _) = sync_policy();
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_secs(10));
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            tick.tick().await;
            if let Err(err) = sync_one(&state, spacing).await {
                tracing::warn!("软件目录同步数据库错误: {err}");
            }
        }
    });
}

async fn sync_one(state: &AppState, spacing: i64) -> Result<(), sqlx::Error> {
    let mut tx = state.pool.begin().await?;
    let locked: bool = sqlx::query_scalar("SELECT pg_try_advisory_xact_lock(763420168)")
        .fetch_one(&mut *tx)
        .await?;
    if !locked {
        return Ok(());
    }
    let active: Vec<String> = SOURCES.keys().cloned().collect();
    let project: Option<String> = sqlx::query_scalar(
        "SELECT project FROM open_source_software_sync
        WHERE project = ANY($1) AND next_at <= now()
        AND NOT EXISTS (SELECT 1 FROM open_source_software_sync
            WHERE attempted_at > now() - $2::bigint * interval '1 second')
        ORDER BY next_at, project LIMIT 1",
    )
    .bind(&active)
    .bind(spacing)
    .fetch_optional(&mut *tx)
    .await?;
    let Some(project) = project else {
        return Ok(());
    };
    sqlx::query("UPDATE open_source_software_sync SET attempted_at = now() WHERE project = $1")
        .bind(&project)
        .execute(&mut *tx)
        .await?;
    let saved: Option<(uuid::Uuid, Vec<u8>)> = sqlx::query_as(
        "SELECT id,credential FROM open_source_software_tokens ORDER BY last_used_at NULLS FIRST,id LIMIT 1",
    ).fetch_optional(&mut *tx).await?;
    let token = if let Some((id, sealed)) = saved {
        sqlx::query("UPDATE open_source_software_tokens SET last_used_at=now() WHERE id=$1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        match credentials::decrypt(TOKEN_SOURCE, &sealed) {
            Ok(token) => Some(token),
            Err(_) => {
                tracing::warn!("无法解密软件采集令牌，请检查采集主密钥");
                github_token()
            }
        }
    } else {
        github_token()
    };
    match fetch(&state.http, &project, &SOURCES[&project], token.as_deref()).await {
        Ok((stars, snapshot)) => {
            sqlx::query(
                "UPDATE open_source_software_sync
                SET snapshot = $2, synced_at = now(), next_at = now() + $3::bigint * interval '1 second'
                WHERE project = $1",
            )
            .bind(&project)
            .bind(&snapshot)
            .bind(refresh_interval(stars, &snapshot))
            .execute(&mut *tx)
            .await?;
            sqlx::query(
                "INSERT INTO open_source_software_stars(project, day, stars)
                VALUES ($1, (now() AT TIME ZONE 'UTC')::date, $2)
                ON CONFLICT (project, day) DO UPDATE SET stars = excluded.stars",
            )
            .bind(&project)
            .bind(stars)
            .execute(&mut *tx)
            .await?;
            sqlx::query(
                "DELETE FROM open_source_software_stars
                WHERE day < (now() AT TIME ZONE 'UTC')::date - 30",
            )
            .execute(&mut *tx)
            .await?;
        }
        Err(err) => {
            tracing::warn!("软件目录同步失败 {project}: {err}");
            if let FetchError::Status(_, Some(delay)) = err {
                // Delay due work and preserve the spacing of future jobs.
                sqlx::query(
                    "UPDATE open_source_software_sync
                    SET next_at = greatest(next_at, now()) + $1::bigint * interval '1 second'
                    WHERE project = ANY($2)",
                )
                .bind(delay)
                .bind(&active)
                .execute(&mut *tx)
                .await?;
            } else {
                sqlx::query("UPDATE open_source_software_sync SET next_at = now() + interval '1 hour' WHERE project = $1")
                    .bind(&project).execute(&mut *tx).await?;
            }
        }
    }
    tx.commit().await
}

pub async fn summaries(State(state): State<AppState>) -> AppResult<Json<Value>> {
    let active: Vec<String> = SOURCES.keys().cloned().collect();
    let rows: Vec<(String, Value)> = sqlx::query_as(
        "SELECT s.project, jsonb_build_object(
        'stars', s.snapshot->'stars', 'updatedAt', s.snapshot->'updatedAt',
        'version', s.snapshot->'version', 'syncedAt', s.synced_at,
        'starGain', current_stars.stars - old_stars.stars)
        FROM open_source_software_sync s
        LEFT JOIN open_source_software_stars current_stars ON current_stars.project = s.project
            AND current_stars.day = (s.synced_at AT TIME ZONE 'UTC')::date
        LEFT JOIN LATERAL (
            SELECT stars FROM open_source_software_stars
            WHERE project = s.project AND day BETWEEN current_stars.day - 8 AND current_stars.day - 6
            ORDER BY abs(day - (current_stars.day - 7)), day DESC LIMIT 1
        ) old_stars ON true
        WHERE s.project = ANY($1) AND s.snapshot IS NOT NULL",
    )
    .bind(active)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(json!(rows.into_iter().collect::<BTreeMap<_, _>>())))
}

pub async fn detail(
    State(state): State<AppState>,
    Path((owner, repo)): Path<(String, String)>,
) -> AppResult<Json<Value>> {
    let project = format!("{owner}/{repo}").to_ascii_lowercase();
    if !SOURCES.contains_key(&project) {
        return Err(crate::error::AppError::not_found("开源软件"));
    }
    let snapshot: Option<Value> = sqlx::query_scalar(
        "SELECT snapshot || jsonb_build_object('syncedAt', synced_at)
        FROM open_source_software_sync WHERE project = $1 AND snapshot IS NOT NULL",
    )
    .bind(project)
    .fetch_optional(&state.pool)
    .await?;
    Ok(Json(snapshot.unwrap_or(Value::Null)))
}
