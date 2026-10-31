//! 手机端社区的发现页接口：首页推荐、日报、开源热榜、模型榜、追踪、导航、资料导读、AI 工具、求助统计和投票。
//!
//! 条目都是后台手录的（见 `crate::discover`），这里只读取和组装；公开接口只返回已发布的条目。

use crate::{
    auth::AuthUser,
    discover::{self, TIERS},
    error::{AppError, AppResult},
    handlers::{
        admin::{audit, AuthAdmin},
        content,
    },
    models::{self, Page},
    state::AppState,
};
use axum::{
    extract::{Path, Query, State},
    routing::{get, put},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::{json, Map, Value};
use sqlx::PgPool;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/discover/home", get(home))
        .route("/discover/daily", get(daily))
        .route("/discover/repos", get(repos))
        .route("/discover/repos/issues", get(repo_issues))
        .route("/discover/models", get(models_board))
        .route("/discover/tracker", get(tracker))
        .route("/discover/nav", get(nav))
        .route("/discover/docs", get(docs))
        .route("/tools", get(list_tools))
        .route("/tools/{slug}", get(get_tool))
        .route("/help/stats", get(help_stats))
        .route("/topics/{id}/poll/vote", put(vote))
}

pub fn admin_routes() -> Router<AppState> {
    Router::new()
        .route("/discover/{kind}", get(admin_list))
        .route("/discover/{kind}/{key}", put(admin_put).delete(admin_delete))
}

type Row = (String, i32, DateTime<Utc>, Value);

/// 读出某类已发布条目，把 key 和发布时间并进 data。`order` 是安全的常量。
async fn fetch(pool: &PgPool, kind: &str, order: &str, limit: i64, offset: i64) -> AppResult<Vec<Value>> {
    let rows: Vec<Row> = sqlx::query_as(&format!(
        "SELECT key, sort_order, published_at, data FROM discover_items
         WHERE kind = $1 AND status = 'published' ORDER BY {order} LIMIT $2 OFFSET $3"
    ))
    .bind(kind)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(item_json).collect())
}

fn item_json((key, _order, published_at, data): Row) -> Value {
    let mut out = match data {
        Value::Object(m) => m,
        _ => Map::new(),
    };
    out.insert("key".into(), json!(key));
    out.insert("published_at".into(), json!(published_at));
    Value::Object(out)
}

async fn count(pool: &PgPool, kind: &str) -> AppResult<i64> {
    let (n,): (i64,) = sqlx::query_as("SELECT count(*) FROM discover_items WHERE kind = $1 AND status = 'published'")
        .bind(kind)
        .fetch_one(pool)
        .await?;
    Ok(n)
}

const NEWEST: &str = "published_at DESC, key DESC";
const BY_ORDER: &str = "sort_order, published_at DESC, key";

fn any_sample(items: &[Value]) -> bool {
    items.iter().any(|i| i["sample"].as_bool().unwrap_or(false))
}

async fn latest_repo_issue(pool: &PgPool, key: Option<&str>) -> AppResult<Option<Value>> {
    let row: Option<Row> = sqlx::query_as(
        "SELECT key, sort_order, published_at, data FROM discover_items
         WHERE kind = 'repo_issue' AND status = 'published' AND ($1::text IS NULL OR key = $1)
         ORDER BY (data->>'issue_no')::int DESC LIMIT 1",
    )
    .bind(key)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| {
        let mut v = item_json(r);
        if let Some(ranks) = v["ranks"].as_array_mut() {
            for (i, r) in ranks.iter_mut().enumerate() {
                r["rank"] = json!(i + 1);
            }
        }
        v
    }))
}

/// 首页「推荐」：头图、当天日报、精选创作和本周热榜前三，一次返回。
async fn home(State(state): State<AppState>) -> AppResult<Json<Value>> {
    let pool = &state.pool;
    let banners = fetch(pool, "banner", BY_ORDER, 5, 0).await?;
    let news = fetch(pool, "news", NEWEST, 5, 0).await?;
    let (day, total): (Option<chrono::NaiveDate>, i64) = sqlx::query_as(
        "SELECT max((published_at AT TIME ZONE 'Asia/Shanghai')::date),
                count(*) FILTER (WHERE (published_at AT TIME ZONE 'Asia/Shanghai')::date =
                    (SELECT max((published_at AT TIME ZONE 'Asia/Shanghai')::date) FROM discover_items WHERE kind = 'news' AND status = 'published'))
         FROM discover_items WHERE kind = 'news' AND status = 'published'",
    )
    .fetch_one(pool)
    .await?;
    let featured = content::top_packs(&state, 8).await?;
    let issue = latest_repo_issue(pool, None).await?;
    let repos = issue.map(|mut i| {
        if let Some(r) = i["ranks"].as_array_mut() {
            r.truncate(3);
        }
        i
    });
    let sample = any_sample(&banners) || any_sample(&news) || repos.as_ref().is_some_and(|r| r["sample"].as_bool().unwrap_or(false));
    Ok(Json(json!({
        "banners": banners,
        "news": { "date": day, "total": total, "items": news },
        "featured": featured,
        "repos": repos,
        "sample": sample,
    })))
}

#[derive(Deserialize)]
pub struct PageQuery {
    page: Option<i64>,
    page_size: Option<i64>,
}

async fn daily(State(state): State<AppState>, Query(q): Query<PageQuery>) -> AppResult<Json<Page<Value>>> {
    let page = q.page.unwrap_or(1).max(1);
    let size = q.page_size.unwrap_or(20).clamp(1, 50);
    let items = fetch(&state.pool, "news", NEWEST, size, (page - 1) * size).await?;
    let total = count(&state.pool, "news").await?;
    Ok(Json(Page { items, page, page_size: size, total }))
}

#[derive(Deserialize)]
pub struct IssueQuery {
    issue: Option<String>,
}

async fn repos(State(state): State<AppState>, Query(q): Query<IssueQuery>) -> AppResult<Json<Value>> {
    match latest_repo_issue(&state.pool, q.issue.as_deref()).await? {
        Some(v) => Ok(Json(v)),
        None => Err(AppError::not_found("榜单")),
    }
}

async fn repo_issues(State(state): State<AppState>) -> AppResult<Json<Value>> {
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT key, sort_order, published_at, data FROM discover_items
         WHERE kind = 'repo_issue' AND status = 'published' ORDER BY (data->>'issue_no')::int DESC LIMIT 60",
    )
    .fetch_all(&state.pool)
    .await?;
    let items: Vec<Value> = rows
        .into_iter()
        .map(|(key, _, _, d)| json!({ "key": key, "issue_no": d["issue_no"], "week_start": d["week_start"], "week_end": d["week_end"] }))
        .collect();
    Ok(Json(json!({ "items": items })))
}

async fn models_board(State(state): State<AppState>) -> AppResult<Json<Value>> {
    let mut vendors = fetch(&state.pool, "vendor", BY_ORDER, 100, 0).await?;
    let mut models = fetch(&state.pool, "model", BY_ORDER, 200, 0).await?;
    let updated_at = models.iter().filter_map(|m| m["published_at"].as_str()).max().map(str::to_string);
    for m in &mut models {
        let score = m["iq"].as_f64().unwrap_or(0.0);
        m["tier"] = json!(discover::tier_for(score));
    }
    let sample = any_sample(&models);
    for v in &mut vendors {
        if let Some(o) = v.as_object_mut() {
            o.remove("published_at");
        }
    }
    let tiers: Vec<Value> = TIERS
        .iter()
        .map(|(id, label, min)| json!({ "id": id, "label": label, "min": if min.is_finite() { json!(min) } else { Value::Null } }))
        .collect();
    Ok(Json(json!({
        "vendors": vendors,
        "models": models,
        "tiers": tiers,
        "source": "Artificial Analysis",
        "updated_at": updated_at,
        "sample": sample,
    })))
}

async fn tracker(State(state): State<AppState>) -> AppResult<Json<Value>> {
    let quota = fetch(&state.pool, "quota", BY_ORDER, 1, 0).await?.into_iter().next();
    let launches = fetch(&state.pool, "launch", NEWEST, 10, 0).await?;
    let now = Utc::now();
    let mut promos = Vec::new();
    for mut p in fetch(&state.pool, "promo", "(data->>'ends_at')::timestamptz", 50, 0).await? {
        let (Some(start), Some(end)) = (parse_time(&p["started_at"]), parse_time(&p["ends_at"])) else { continue };
        if end <= now {
            continue;
        }
        p["pct"] = json!(discover::promo_progress(start, end, now));
        p["days_left"] = json!(((end - now).num_seconds() + 86_399) / 86_400);
        promos.push(p);
    }
    let sample = quota.as_ref().is_some_and(|q| q["sample"].as_bool().unwrap_or(false)) || any_sample(&launches) || any_sample(&promos);
    Ok(Json(json!({ "quota": quota, "launches": launches, "promos": promos, "now": now, "sample": sample })))
}

fn parse_time(v: &Value) -> Option<DateTime<Utc>> {
    v.as_str()?.parse().ok()
}

/// 把条目按 `field` 的值分组，保持首次出现的顺序。
fn group_by(items: Vec<Value>, field: &str) -> Vec<Value> {
    let mut groups: Vec<(String, Vec<Value>)> = Vec::new();
    for item in items {
        let name = item[field].as_str().unwrap_or("").to_string();
        match groups.iter_mut().find(|(n, _)| *n == name) {
            Some((_, list)) => list.push(item),
            None => groups.push((name, vec![item])),
        }
    }
    groups.into_iter().map(|(name, items)| json!({ "name": name, "items": items })).collect()
}

async fn nav(State(state): State<AppState>) -> AppResult<Json<Value>> {
    let items = fetch(&state.pool, "nav", BY_ORDER, 200, 0).await?;
    Ok(Json(json!({ "groups": group_by(items, "group") })))
}

async fn docs(State(state): State<AppState>) -> AppResult<Json<Value>> {
    let items = fetch(&state.pool, "doc", BY_ORDER, 200, 0).await?;
    Ok(Json(json!({ "groups": group_by(items, "product") })))
}

fn latest_release(data: &Value) -> (Option<String>, Option<String>) {
    let first = data["releases"].as_array().and_then(|r| r.first());
    (
        first.and_then(|r| r["version"].as_str()).map(str::to_string),
        first.and_then(|r| r["date"].as_str()).map(str::to_string),
    )
}

async fn list_tools(State(state): State<AppState>) -> AppResult<Json<Value>> {
    let items: Vec<Value> = fetch(&state.pool, "tool", BY_ORDER, 100, 0)
        .await?
        .into_iter()
        .map(|t| {
            let (version, date) = latest_release(&t);
            let size = t["artifacts"].as_array().and_then(|a| a.first()).and_then(|a| a["size"].as_i64());
            json!({
                "key": t["key"], "name": t["name"], "developer": t["developer"], "type_label": t["type_label"],
                "license": t["license"], "platforms": t["platforms"], "icon_url": t["icon_url"],
                "latest_version": version, "latest_date": date, "size": size, "sample": t["sample"],
            })
        })
        .collect();
    Ok(Json(json!({ "items": items })))
}

async fn get_tool(State(state): State<AppState>, Path(slug): Path<String>) -> AppResult<Json<Value>> {
    let row: Option<Row> = sqlx::query_as(
        "SELECT key, sort_order, published_at, data FROM discover_items WHERE kind = 'tool' AND key = $1 AND status = 'published'",
    )
    .bind(&slug)
    .fetch_optional(&state.pool)
    .await?;
    let mut tool = item_json(row.ok_or_else(|| AppError::not_found("工具"))?);
    let (version, date) = latest_release(&tool);
    tool["latest_version"] = json!(version);
    tool["latest_date"] = json!(date);
    let tag = tool["discussion_tag"].as_str().unwrap_or("").to_string();
    let discussions: i64 = if tag.is_empty() {
        0
    } else {
        let (n,): (i64,) = sqlx::query_as(
            "SELECT count(*) FROM topics t JOIN topic_tags tt ON tt.topic_id = t.id JOIN tags g ON g.id = tt.tag_id
             WHERE g.name = $1 AND t.status = 'normal'",
        )
        .bind(&tag)
        .fetch_one(&state.pool)
        .await?;
        n
    };
    tool["discussion_count"] = json!(discussions);
    Ok(Json(tool))
}

/// 求助悬赏页顶部三个数：进行中的求助、本周已解决、进行中的悬赏积分合计（全站）。
async fn help_stats(State(state): State<AppState>) -> AppResult<Json<Value>> {
    let (open, bounty): (i64, i64) = sqlx::query_as(
        "SELECT count(*), COALESCE(sum(h.bounty), 0)::bigint FROM help_requests h JOIN topics t ON t.id = h.topic_id
         WHERE h.status = 'open' AND h.expires_at > now() AND t.status = 'normal'",
    )
    .fetch_one(&state.pool)
    .await?;
    let (solved,): (i64,) = sqlx::query_as(
        "SELECT count(*) FROM help_requests
         WHERE status = 'solved' AND settled_at >= date_trunc('week', now() AT TIME ZONE 'Asia/Shanghai') AT TIME ZONE 'Asia/Shanghai'",
    )
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(json!({ "open": open, "solved_this_week": solved, "bounty_points": bounty })))
}

#[derive(Deserialize)]
pub struct VoteInput {
    option: i16,
}

async fn vote(State(state): State<AppState>, auth: AuthUser, Path(id): Path<i64>, Json(input): Json<VoteInput>) -> AppResult<Json<Value>> {
    crate::users::ensure_can_write(&auth.user)?;
    let poll: Option<(Vec<String>, Option<DateTime<Utc>>, String)> = sqlx::query_as(
        "SELECT p.options, p.closes_at, t.status FROM polls p JOIN topics t ON t.id = p.topic_id WHERE p.topic_id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?;
    let (options, closes_at, status) = poll.ok_or_else(|| AppError::not_found("投票"))?;
    if status != "normal" {
        return Err(AppError::not_found("投票"));
    }
    if closes_at.is_some_and(|c| c <= Utc::now()) {
        return Err(AppError::Conflict("投票已经结束".into()));
    }
    if input.option < 0 || input.option as usize >= options.len() {
        return Err(AppError::BadRequest("没有这个选项".into()));
    }
    let inserted = sqlx::query("INSERT INTO poll_votes (topic_id, uid, option_idx) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING")
        .bind(id)
        .bind(auth.user.uid)
        .bind(input.option)
        .execute(&state.pool)
        .await?
        .rows_affected();
    if inserted == 0 {
        return Err(AppError::Conflict("你已经投过票了".into()));
    }
    let poll = models::load_polls(&state.pool, &[id], Some(auth.user.uid)).await?.remove(&id);
    Ok(Json(json!({ "poll": poll })))
}

async fn admin_list(State(state): State<AppState>, _admin: AuthAdmin, Path(kind): Path<String>) -> AppResult<Json<Value>> {
    if !discover::is_kind(&kind) {
        return Err(AppError::BadRequest("未知的类型".into()));
    }
    let rows: Vec<(String, i32, String, DateTime<Utc>, Value)> = sqlx::query_as(
        "SELECT key, sort_order, status, published_at, data FROM discover_items WHERE kind = $1 ORDER BY sort_order, published_at DESC, key LIMIT 500",
    )
    .bind(&kind)
    .fetch_all(&state.pool)
    .await?;
    let items: Vec<Value> = rows
        .into_iter()
        .map(|(key, sort_order, status, published_at, data)| {
            json!({ "key": key, "sort_order": sort_order, "status": status, "published_at": published_at, "data": data })
        })
        .collect();
    Ok(Json(json!({ "items": items })))
}

#[derive(Deserialize)]
pub struct PutInput {
    data: Value,
    #[serde(default)]
    sort_order: i32,
    status: Option<String>,
    /// 日报、上新这类按时间排序的条目可以指定发布时间，不传按当前时间（改已有条目时保持原样）。
    published_at: Option<DateTime<Utc>>,
    /// 标记为示例内容，客户端会显示「示例数据」。
    #[serde(default)]
    sample: bool,
}

async fn admin_put(
    State(state): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Path((kind, key)): Path<(String, String)>,
    Json(input): Json<PutInput>,
) -> AppResult<Json<Value>> {
    if !discover::is_kind(&kind) {
        return Err(AppError::BadRequest("未知的类型".into()));
    }
    if !discover::valid_key(&key) {
        return Err(AppError::BadRequest("key 只能用小写字母、数字、连字符和下划线".into()));
    }
    let status = input.status.as_deref().unwrap_or("published");
    if !["draft", "published"].contains(&status) {
        return Err(AppError::BadRequest("状态只能是 draft 或 published".into()));
    }
    let mut data = discover::validate(&kind, input.data)?;
    data["sample"] = Value::Bool(input.sample);
    let mut tx = state.pool.begin().await?;
    sqlx::query(
        "INSERT INTO discover_items (kind, key, sort_order, status, published_at, data)
         VALUES ($1, $2, $3, $4, COALESCE($5, now()), $6)
         ON CONFLICT (kind, key) DO UPDATE SET sort_order = $3, status = $4,
             published_at = COALESCE($5, discover_items.published_at), data = $6, updated_at = now()",
    )
    .bind(&kind)
    .bind(&key)
    .bind(input.sort_order)
    .bind(status)
    .bind(input.published_at)
    .bind(data)
    .execute(&mut *tx)
    .await?;
    audit(&mut tx, &admin, "discover.put", &kind, &key, status).await?;
    tx.commit().await?;
    Ok(Json(json!({ "kind": kind, "key": key })))
}

async fn admin_delete(
    State(state): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Path((kind, key)): Path<(String, String)>,
) -> AppResult<Json<Value>> {
    let mut tx = state.pool.begin().await?;
    let gone = sqlx::query("DELETE FROM discover_items WHERE kind = $1 AND key = $2")
        .bind(&kind)
        .bind(&key)
        .execute(&mut *tx)
        .await?
        .rows_affected();
    if gone == 0 {
        return Err(AppError::not_found("条目"));
    }
    audit(&mut tx, &admin, "discover.delete", &kind, &key, "").await?;
    tx.commit().await?;
    Ok(Json(json!({ "deleted": true })))
}

#[cfg(test)]
#[path = "discover_tests.rs"]
mod discover_tests;
