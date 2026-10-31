//! Bounded, read-only health checks and factual operations aggregates. No content state is changed.
mod network;
pub mod schema;
use crate::{
    collectors,
    error::{AppError, AppResult},
    handlers::admin::{audit, AuthAdmin},
    state::AppState,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use ring::digest;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::PgPool;
use std::{collections::BTreeMap, time::Duration};
use tokio::task::JoinSet;
use uuid::Uuid;
const MAX_LINKS: usize = 60;
const CONCURRENCY: usize = 4;
const RUN_TIMEOUT: Duration = Duration::from_secs(120);
const CANDIDATE_SCOPE: &str = "最多读取按栏目和标识排序的 1000 条已发布导航、文档、工具，以及最近维护的 1000 条来源记录；同一链接合并，每条最多展示 50 处引用。";
#[derive(Clone, Debug, Serialize)]
struct Reference {
    kind: String,
    key: String,
    field: String,
    scope: String,
}
#[derive(Clone, Debug)]
struct Target {
    id: String,
    raw: String,
    url: String,
    host: String,
    refs: Vec<Reference>,
}
#[derive(Clone, sqlx::FromRow, Serialize)]
struct ResultRow {
    id: String,
    url: String,
    host: String,
    refs: Value,
    status: String,
    http_status: Option<i32>,
    final_url: Option<String>,
    error: Option<String>,
    duration_ms: i64,
    last_checked_at: DateTime<Utc>,
    last_success_at: Option<DateTime<Utc>>,
    failures: i32,
}
#[derive(sqlx::FromRow, Serialize)]
struct Settings {
    enabled: bool,
    interval_hours: i32,
    next_run_at: DateTime<Utc>,
    active_run: Option<Uuid>,
    lease_until: Option<DateTime<Utc>>,
}
#[derive(sqlx::FromRow, Serialize)]
struct Run {
    id: Uuid,
    status: String,
    started_at: DateTime<Utc>,
    finished_at: Option<DateTime<Utc>>,
    total_candidates: i32,
    selected_count: i32,
    checked_count: i32,
    healthy_count: i32,
    attention_count: i32,
    blocked_count: i32,
    error: Option<String>,
}
fn insert_target(map: &mut BTreeMap<String, Target>, raw: &str, reference: Reference) {
    if raw.trim().is_empty() {
        return;
    }
    let parsed = network::normalized(raw);
    let normalized = parsed.as_ref().map(|u| u.as_str()).unwrap_or(raw.trim());
    let id = digest::digest(&digest::SHA256, normalized.as_bytes())
        .as_ref()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let target = map.entry(id.clone()).or_insert_with(|| Target {
        id,
        raw: raw.trim().to_string(),
        url: parsed
            .as_ref()
            .map(|u| u.to_string())
            .unwrap_or_else(|_| network::display_url(raw)),
        host: parsed
            .as_ref()
            .ok()
            .and_then(|u| u.host_str())
            .unwrap_or("已阻止来源")
            .into(),
        refs: Vec::new(),
    });
    if target.refs.len() < 50 {
        target.refs.push(reference)
    }
}
async fn targets(pool: &PgPool) -> AppResult<Vec<Target>> {
    let rows:Vec<(String,String,Value)>=sqlx::query_as("SELECT kind,key,data FROM discover_items WHERE kind IN('nav','doc','tool') AND status='published' AND published_at<=now() ORDER BY kind,key LIMIT 1000").fetch_all(pool).await?;
    let mut map = BTreeMap::new();
    for (kind, key, data) in rows {
        let field = if kind == "tool" {
            "homepage_url"
        } else {
            "url"
        };
        if let Some(url) = data[field].as_str() {
            insert_target(
                &mut map,
                url,
                Reference {
                    kind: kind.clone(),
                    key: key.clone(),
                    field: field.into(),
                    scope: "published".into(),
                },
            )
        }
        if let Some(url) = data["source_url"].as_str() {
            insert_target(
                &mut map,
                url,
                Reference {
                    kind,
                    key,
                    field: "source_url".into(),
                    scope: "published".into(),
                },
            )
        }
    }
    // This is a fixed, stored source field; the run API accepts no supplied URLs or domains.
    let sources:Vec<(String,String,String)>=sqlx::query_as("SELECT kind,key,source_url FROM discover_drafts WHERE source_url<>'' AND kind IN('nav','doc','tool','news','model_board','model_usage','repo_issue') ORDER BY updated_at DESC,kind,key LIMIT 1000").fetch_all(pool).await?;
    for (kind, key, url) in sources {
        insert_target(
            &mut map,
            &url,
            Reference {
                kind,
                key,
                field: "source_url".into(),
                scope: "editor_source".into(),
            },
        )
    }
    Ok(map.into_values().collect())
}
async fn results(pool: &PgPool, targets: &[Target]) -> AppResult<Vec<ResultRow>> {
    let ids = targets.iter().map(|t| t.id.clone()).collect::<Vec<_>>();
    Ok(sqlx::query_as("SELECT id,url,host,refs,status,http_status,final_url,error,duration_ms,last_checked_at,last_success_at,failures FROM link_health_results WHERE id=ANY($1)").bind(ids).fetch_all(pool).await?)
}
pub fn admin_routes() -> Router<AppState> {
    Router::new()
        .route("/community-operations", get(operations))
        .route("/link-health", get(list))
        .route("/link-health/run", post(run))
        .route("/link-health/settings", axum::routing::patch(settings))
}
#[derive(Deserialize)]
struct Filter {
    page: Option<usize>,
    status: Option<String>,
}
async fn list(
    State(s): State<AppState>,
    _: AuthAdmin,
    Query(f): Query<Filter>,
) -> AppResult<Json<Value>> {
    let current = targets(&s.pool).await?;
    let recorded = results(&s.pool, &current).await?;
    let mut items=current.iter().map(|t|{
        let mut value=recorded.iter().find(|r|r.id==t.id).map(|r|serde_json::to_value(r).unwrap()).unwrap_or_else(||json!({"id":t.id,"status":"unchecked","last_checked_at":null,"last_success_at":null,"http_status":null,"error":null,"duration_ms":null,"failures":0}));
        value["url"]=json!(t.url);value["host"]=json!(t.host);value["refs"]=json!(t.refs);value
    }).filter(|v|f.status.as_ref().is_none_or(|status|status.is_empty()||v["status"].as_str()==Some(status))).collect::<Vec<_>>();
    items.sort_by(|a, b| {
        a["last_checked_at"]
            .as_str()
            .cmp(&b["last_checked_at"].as_str())
            .then_with(|| a["id"].as_str().cmp(&b["id"].as_str()))
    });
    let page = f.page.unwrap_or(1).clamp(1, 100_000);
    let total = items.len();
    let items = items
        .into_iter()
        .skip((page - 1) * 30)
        .take(30)
        .collect::<Vec<_>>();
    let settings:Settings=sqlx::query_as("SELECT enabled,interval_hours,next_run_at,active_run,lease_until FROM link_health_settings WHERE singleton").fetch_one(&s.pool).await?;
    let runs:Vec<Run>=sqlx::query_as("SELECT id,status,started_at,finished_at,total_candidates,selected_count,checked_count,healthy_count,attention_count,blocked_count,error FROM link_health_runs ORDER BY started_at DESC LIMIT 10").fetch_all(&s.pool).await?;
    Ok(Json(
        json!({"items":items,"total":total,"page":page,"page_size":30,"settings":settings,"runs":runs,"max_links_per_run":MAX_LINKS,"max_run_seconds":RUN_TIMEOUT.as_secs(),"candidate_scope":CANDIDATE_SCOPE}),
    ))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SettingsInput {
    enabled: bool,
    interval_hours: i32,
}
async fn settings(
    State(s): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Json(i): Json<SettingsInput>,
) -> AppResult<Json<Value>> {
    if !(24..=720).contains(&i.interval_hours) {
        return Err(AppError::BadRequest("自动检查周期须为 24–720 小时".into()));
    }
    let mut tx = s.pool.begin().await?;
    sqlx::query("UPDATE link_health_settings SET enabled=$1,interval_hours=$2,next_run_at=now()+make_interval(hours=>$2) WHERE singleton").bind(i.enabled).bind(i.interval_hours).execute(&mut *tx).await?;
    audit(
        &mut tx,
        &admin,
        "link_health.settings",
        "link_health",
        "settings",
        &format!("enabled={} interval_hours={}", i.enabled, i.interval_hours),
    )
    .await?;
    tx.commit().await?;
    Ok(Json(json!({"ok":true})))
}
// A typed empty object rejects attempts to supply a URL, host or credentials to the checker.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RunInput {}
async fn run(
    State(s): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Json(_): Json<RunInput>,
) -> AppResult<(StatusCode, Json<Value>)> {
    let (id, total, selected) = start(&s.pool, false).await?;
    let mut conn = s.pool.acquire().await?;
    audit(
        &mut conn,
        &admin,
        "link_health.run",
        "link_health",
        &id.to_string(),
        &format!("selected={selected} total_candidates={total}"),
    )
    .await?;
    Ok((
        StatusCode::ACCEPTED,
        Json(json!({"run_id":id,"total_candidates":total,"selected_count":selected})),
    ))
}
pub async fn start(pool: &PgPool, scheduled: bool) -> AppResult<(Uuid, usize, usize)> {
    let mut current = targets(pool).await?;
    let existing = results(pool, &current).await?;
    current.sort_by_key(|t| {
        existing
            .iter()
            .find(|r| r.id == t.id)
            .map(|r| r.last_checked_at)
    });
    let total = current.len();
    current.truncate(MAX_LINKS);
    let selected = current.len();
    let mut tx = pool.begin().await?;
    let settings:Settings=sqlx::query_as("SELECT enabled,interval_hours,next_run_at,active_run,lease_until FROM link_health_settings WHERE singleton FOR UPDATE").fetch_one(&mut *tx).await?;
    if settings.lease_until.is_some_and(|t| t > Utc::now()) {
        return Err(AppError::Conflict(
            "已有外链检查正在执行，请查看运行记录".into(),
        ));
    }
    if scheduled && (!settings.enabled || settings.next_run_at > Utc::now()) {
        return Err(AppError::Conflict("自动外链检查未启用或尚未到期".into()));
    }
    if let Some(old) = settings.active_run {
        sqlx::query("UPDATE link_health_runs SET status='failed',finished_at=now(),error='进程中断，租约已到期' WHERE id=$1 AND status='running'").bind(old).execute(&mut *tx).await?;
    }
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO link_health_runs(id,status,total_candidates,selected_count)VALUES($1,'running',$2,$3)").bind(id).bind(total as i32).bind(selected as i32).execute(&mut *tx).await?;
    sqlx::query("UPDATE link_health_settings SET active_run=$1,lease_until=now()+interval '5 minutes',next_run_at=now()+make_interval(hours=>interval_hours) WHERE singleton").bind(id).execute(&mut *tx).await?;
    tx.commit().await?;
    let pool = pool.clone();
    tokio::spawn(async move {
        let (done, partial) = check_targets(current).await;
        if finish(&pool, id, done, partial).await.is_err() {
            tracing::warn!(run_id=%id,"外链检查结果未保存，上一版状态保留");
        }
    });
    Ok((id, total, selected))
}
async fn check_targets(targets: Vec<Target>) -> (Vec<(Target, network::Outcome)>, bool) {
    let mut remaining = targets.into_iter();
    let mut tasks = JoinSet::new();
    let mut done = Vec::new();
    let launch = |tasks: &mut JoinSet<(Target, network::Outcome)>, t: Target| {
        tasks.spawn(async move {
            let result = network::probe(t.raw.clone()).await;
            (t, result)
        });
    };
    for _ in 0..CONCURRENCY {
        if let Some(t) = remaining.next() {
            launch(&mut tasks, t)
        }
    }
    let deadline = tokio::time::Instant::now() + RUN_TIMEOUT;
    while !tasks.is_empty() {
        match tokio::time::timeout_at(deadline, tasks.join_next()).await {
            Ok(Some(Ok(result))) => {
                done.push(result);
                if let Some(t) = remaining.next() {
                    launch(&mut tasks, t)
                }
            }
            Ok(Some(Err(_))) => {
                tasks.abort_all();
                return (done, true);
            }
            Ok(None) => break,
            Err(_) => {
                tasks.abort_all();
                return (done, true);
            }
        }
    }
    (done, false)
}
async fn finish(
    pool: &PgPool,
    id: Uuid,
    done: Vec<(Target, network::Outcome)>,
    partial: bool,
) -> AppResult<()> {
    let mut tx = pool.begin().await?;
    let valid:Option<(bool,)>=sqlx::query_as("SELECT coalesce(active_run=$1 AND lease_until>now(),false) FROM link_health_settings WHERE singleton FOR UPDATE").bind(id).fetch_optional(&mut *tx).await?;
    if !valid.is_some_and(|v| v.0) {
        return Err(AppError::Conflict(
            "此运行租约已过期，未覆盖新的检查结果".into(),
        ));
    }
    let checked = done.len() as i32;
    let healthy = done.iter().filter(|(_, r)| r.status == "healthy").count() as i32;
    let blocked = done.iter().filter(|(_, r)| r.status == "blocked").count() as i32;
    for (t, r) in done {
        sqlx::query("INSERT INTO link_health_results(id,url,host,refs,status,http_status,final_url,error,duration_ms,last_checked_at,last_success_at,failures)VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,now(),CASE WHEN $5='healthy' THEN now() ELSE NULL END,CASE WHEN $5='healthy' THEN 0 ELSE 1 END)ON CONFLICT(id)DO UPDATE SET url=$2,host=$3,refs=$4,status=$5,http_status=$6,final_url=$7,error=$8,duration_ms=$9,last_checked_at=now(),last_success_at=CASE WHEN $5='healthy' THEN now() ELSE link_health_results.last_success_at END,failures=CASE WHEN $5='healthy' THEN 0 ELSE link_health_results.failures+1 END")
            .bind(t.id).bind(t.url).bind(t.host).bind(json!(t.refs)).bind(r.status).bind(r.http_status).bind(r.final_url).bind(r.error).bind(r.duration_ms).execute(&mut *tx).await?;
    }
    sqlx::query("UPDATE link_health_runs SET status=$2,finished_at=now(),checked_count=$3,healthy_count=$4,attention_count=$5,blocked_count=$6,error=$7 WHERE id=$1").bind(id).bind(if partial{"partial"}else{"succeeded"}).bind(checked).bind(healthy).bind(checked-healthy-blocked).bind(blocked).bind(if partial{Some("本轮达到总时限或中断，未检查链接保留原状态")}else{None}).execute(&mut *tx).await?;
    sqlx::query("UPDATE link_health_settings SET active_run=NULL,lease_until=NULL WHERE singleton AND active_run=$1").bind(id).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(())
}
#[derive(sqlx::FromRow, Serialize)]
struct Column {
    kind: String,
    published: i64,
    future: i64,
    unlisted: i64,
    pending: i64,
    scheduled: i64,
    sample: i64,
    last_live_update: Option<DateTime<Utc>>,
}
#[derive(sqlx::FromRow, Serialize)]
struct Source {
    id: String,
    enabled: bool,
    interval_minutes: i32,
    last_success_at: Option<DateTime<Utc>>,
    last_error: Option<String>,
    next_run_at: DateTime<Utc>,
    active: bool,
    failed_24h: i64,
    succeeded_24h: i64,
    pending: i64,
}
async fn operations(State(s): State<AppState>, _: AuthAdmin) -> AppResult<Json<Value>> {
    Ok(Json(summary(&s.pool).await?))
}
pub async fn summary(pool: &PgPool) -> AppResult<Value> {
    let columns:Vec<Column>=sqlx::query_as("WITH kinds AS(SELECT kind FROM discover_items UNION SELECT kind FROM discover_drafts),live AS(SELECT kind,count(*)FILTER(WHERE status='published'AND published_at<=now())AS published,count(*)FILTER(WHERE status='published'AND published_at>now())AS future,count(*)FILTER(WHERE status<>'published')AS unlisted,count(*)FILTER(WHERE status='published'AND published_at<=now()AND data->>'sample'='true')AS sample,max(updated_at)FILTER(WHERE status='published'AND published_at<=now())AS last_live_update FROM discover_items GROUP BY kind),drafts AS(SELECT kind,count(*)FILTER(WHERE status='draft')AS pending,count(*)FILTER(WHERE status='scheduled')AS scheduled FROM discover_drafts GROUP BY kind)SELECT k.kind,coalesce(l.published,0)AS published,coalesce(l.future,0)AS future,coalesce(l.unlisted,0)AS unlisted,coalesce(d.pending,0)AS pending,coalesce(d.scheduled,0)AS scheduled,coalesce(l.sample,0)AS sample,l.last_live_update FROM kinds k LEFT JOIN live l USING(kind)LEFT JOIN drafts d USING(kind)ORDER BY k.kind").fetch_all(pool).await?;
    let sources:Vec<Source>=sqlx::query_as("SELECT s.id,s.enabled,s.interval_minutes,s.last_success_at,s.last_error,s.next_run_at,coalesce(s.active_run IS NOT NULL AND s.lease_until>now(),false)AS active,(SELECT count(*)FROM collection_runs r WHERE r.source=s.id AND r.status='failed'AND r.started_at>=now()-interval '24 hours')AS failed_24h,(SELECT count(*)FROM collection_runs r WHERE r.source=s.id AND r.status='succeeded'AND r.started_at>=now()-interval '24 hours')AS succeeded_24h,(SELECT count(*)FROM collection_owned_items o JOIN discover_drafts d USING(kind,key)WHERE o.source=s.id AND d.status='draft')AS pending FROM collection_sources s ORDER BY s.id").fetch_all(pool).await?;
    let sources = sources
        .into_iter()
        .map(|source| {
            let title = collectors::title(&source.id).unwrap_or("未知来源");
            let freshness = if !source.enabled {
                "disabled"
            } else if let Some(last) = source.last_success_at {
                if last
                    < Utc::now() - chrono::Duration::minutes(i64::from(source.interval_minutes) * 2)
                {
                    "stale"
                } else {
                    "fresh"
                }
            } else {
                "never"
            };
            let mut value = serde_json::to_value(source).unwrap();
            value["title"] = json!(title);
            value["freshness"] = json!(freshness);
            value
        })
        .collect::<Vec<_>>();
    let current = targets(pool).await?;
    let checks = results(pool, &current).await?;
    let mut health = BTreeMap::<String, usize>::new();
    for target in &current {
        let status = checks
            .iter()
            .find(|r| r.id == target.id)
            .map(|r| r.status.as_str())
            .unwrap_or("unchecked");
        *health.entry(status.into()).or_default() += 1;
    }
    Ok(
        json!({"as_of":Utc::now(),"columns":columns,"sources":sources,"links":{"total":current.len(),"statuses":health},"candidate_scope":CANDIDATE_SCOPE,"freshness_rule":"启用来源最近成功时间超过采集周期的两倍标为过期；停用来源单列。"}),
    )
}
pub fn spawn(state: AppState) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(300));
        loop {
            interval.tick().await;
            let due:Result<Option<(bool,)>,_>=sqlx::query_as("SELECT enabled AND next_run_at<=now()AND(lease_until IS NULL OR lease_until<=now())FROM link_health_settings WHERE singleton").fetch_optional(&state.pool).await;
            match due {
                Ok(Some((true,))) => {
                    if let Err(error) = start(&state.pool, true).await {
                        if !matches!(error, AppError::Conflict(_)) {
                            tracing::warn!("自动外链检查未启动，原状态保留")
                        }
                    }
                }
                Ok(_) => {}
                Err(_) => tracing::warn!("无法读取外链检查计划"),
            }
        }
    });
}
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
