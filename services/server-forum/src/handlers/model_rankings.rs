//! 已由管理员审核发布的完整模型榜快照；旧模型榜接口保持兼容。
use crate::{error::{AppError, AppResult}, state::AppState};
use axum::{extract::{Query, State}, routing::get, Json, Router};
use chrono::{DateTime, NaiveDate, Utc};
use serde::Deserialize;
use serde_json::{json, Value};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/discover/models/v2", get(capability))
        .route("/discover/model-usage", get(usage))
        .route("/discover/model-usage/dates", get(dates))
}

async fn snapshot(s: &AppState, kind: &str, key: Option<&str>) -> AppResult<Value> {
    let row: Option<(Value, DateTime<Utc>)> = sqlx::query_as(
        "SELECT data,updated_at FROM discover_items WHERE kind=$1 AND status='published'
         AND published_at<=now() AND ($2::text IS NULL OR key=$2)
         ORDER BY key DESC LIMIT 1"
    ).bind(kind).bind(key).fetch_optional(&s.pool).await?;
    let (mut data, updated_at) = row.ok_or_else(|| AppError::not_found("已发布榜单"))?;
    data["updated_at"] = json!(updated_at);
    Ok(data)
}

async fn capability(State(s): State<AppState>) -> AppResult<Json<Value>> {
    snapshot(&s, "model_board", Some("artificial-analysis")).await.map(Json)
}

#[derive(Deserialize)]
struct UsageFilter { date: Option<String> }

async fn usage(State(s): State<AppState>, Query(f): Query<UsageFilter>) -> AppResult<Json<Value>> {
    if let Some(date) = &f.date {
        let parsed = NaiveDate::parse_from_str(date, "%Y-%m-%d")
            .map_err(|_| AppError::BadRequest("日期格式应为 YYYY-MM-DD".into()))?;
        if parsed.format("%Y-%m-%d").to_string() != *date {
            return Err(AppError::BadRequest("日期格式应为 YYYY-MM-DD".into()));
        }
    }
    let mut data = snapshot(&s, "model_usage", f.date.as_deref()).await?;
    let date = data["date"].as_str().and_then(|v| NaiveDate::parse_from_str(v, "%Y-%m-%d").ok());
    let previous = match date.and_then(|d| d.pred_opt()) {
        Some(day) => {
            let row: Option<(Value,)> = sqlx::query_as("SELECT data FROM discover_items WHERE kind='model_usage' AND key=$1 AND status='published' AND published_at<=now()")
                .bind(day.to_string()).fetch_optional(&s.pool).await?;
            row.map(|r| r.0)
        }
        None => None,
    };
    data["previous_date_available"] = json!(previous.is_some());
    if let Some(models) = data["models"].as_array_mut() {
        for model in models {
            let prior = previous.as_ref().and_then(|p| p["models"].as_array()).and_then(|rows| rows.iter().find(|r| r["id"] == model["id"]));
            let rank = prior.and_then(|r| r["rank"].as_i64());
            model["previous_rank"] = json!(rank);
            model["rank_change"] = json!(rank.zip(model["rank"].as_i64()).map(|(old,new)| old-new));
            model["new_entry"] = json!(previous.is_some() && prior.is_none());
        }
    }
    Ok(Json(data))
}

async fn dates(State(s): State<AppState>) -> AppResult<Json<Value>> {
    let rows: Vec<(String, DateTime<Utc>)> = sqlx::query_as(
        "SELECT key,updated_at FROM discover_items WHERE kind='model_usage' AND status='published'
         AND published_at<=now() ORDER BY key DESC LIMIT 366"
    ).fetch_all(&s.pool).await?;
    Ok(Json(json!({"items": rows.into_iter().map(|(date,updated_at)| json!({"date":date,"updated_at":updated_at})).collect::<Vec<_>>()})))
}
