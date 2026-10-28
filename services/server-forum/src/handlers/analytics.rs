//! 匿名埋点上报与后台汇总。

use crate::{
    analytics::{clean_event, hash_device, valid_device},
    error::{AppError, AppResult},
    handlers::admin::AuthAdmin,
    state::AppState,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use chrono::NaiveDate;
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
pub struct Event {
    name: String,
    dim: Option<String>,
    /// 同一事件在这一批里发生的次数，默认 1，最多 50。
    count: Option<i32>,
}

#[derive(Deserialize)]
pub struct Batch {
    device: String,
    events: Vec<Event>,
}

/// 上报是尽力而为：不合格的事件静默丢掉，成功一律 204，客户端不需要处理错误。
pub async fn ingest(State(state): State<AppState>, Json(batch): Json<Batch>) -> AppResult<StatusCode> {
    if !valid_device(&batch.device) || batch.events.len() > 50 {
        return Err(AppError::BadRequest("上报内容无效".into()));
    }
    let device = hash_device(&batch.device, &state.config.analytics_salt);
    let mut tx = state.pool.begin().await?;
    for e in &batch.events {
        let Some((name, dim)) = clean_event(&e.name, e.dim.as_deref()) else { continue };
        sqlx::query(
            "INSERT INTO events_daily (day, name, dim, device, count) VALUES (CURRENT_DATE, $1, $2, $3, $4)
             ON CONFLICT (day, name, dim, device) DO UPDATE SET count = events_daily.count + EXCLUDED.count",
        )
        .bind(name)
        .bind(dim)
        .bind(&device)
        .bind(e.count.unwrap_or(1).clamp(1, 50))
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct MetricsQuery {
    days: Option<i64>,
}

/// 后台汇总：每天的活跃设备、每个事件的次数与独立设备数、次日留存。
pub async fn metrics(State(state): State<AppState>, _admin: AuthAdmin, Query(q): Query<MetricsQuery>) -> AppResult<Json<Value>> {
    let days = q.days.unwrap_or(14).clamp(1, 90);
    let daily: Vec<(NaiveDate, i64)> = sqlx::query_as(
        "SELECT day, COUNT(DISTINCT device) FROM events_daily WHERE day > CURRENT_DATE - $1::int GROUP BY day ORDER BY day DESC",
    )
    .bind(days as i32)
    .fetch_all(&state.pool)
    .await?;
    let events: Vec<(NaiveDate, String, String, i64, i64)> = sqlx::query_as(
        "SELECT day, name, dim, SUM(count)::bigint, COUNT(DISTINCT device) FROM events_daily
         WHERE day > CURRENT_DATE - $1::int GROUP BY day, name, dim ORDER BY day DESC, name, dim",
    )
    .bind(days as i32)
    .fetch_all(&state.pool)
    .await?;
    // 次日留存：某天出现过的设备，第二天也出现的比例。最近一天还没有“次日”，不统计。
    let retention: Vec<(NaiveDate, i64, i64)> = sqlx::query_as(
        "SELECT a.day, COUNT(DISTINCT a.device),
                COUNT(DISTINCT a.device) FILTER (WHERE EXISTS (SELECT 1 FROM events_daily b WHERE b.device = a.device AND b.day = a.day + 1))
         FROM events_daily a WHERE a.day > CURRENT_DATE - $1::int AND a.day < CURRENT_DATE GROUP BY a.day ORDER BY a.day DESC",
    )
    .bind(days as i32)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(json!({
        "daily_devices": daily.into_iter().map(|(d, n)| json!({ "day": d, "devices": n })).collect::<Vec<_>>(),
        "events": events.into_iter().map(|(d, name, dim, count, devices)| json!({ "day": d, "name": name, "dim": dim, "count": count, "devices": devices })).collect::<Vec<_>>(),
        "retention": retention.into_iter().map(|(d, total, back)| json!({ "day": d, "devices": total, "returned": back })).collect::<Vec<_>>(),
    })))
}

pub fn admin_routes() -> axum::Router<AppState> {
    axum::Router::new().route("/metrics", axum::routing::get(metrics))
}
