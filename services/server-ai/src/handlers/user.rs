use crate::{binding, error::AppError, extract::AuthUser, products, quota, state::AppState};
use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};

/// 当前产品的可用档位。只返回档位名和展示信息，永远不返回真实模型名。
pub async fn tiers(State(state): State<AppState>, auth: AuthUser) -> Result<Json<Value>, AppError> {
    let tiers = products::tiers(&state.pool, &auth.product.product_id, true).await?;
    let paid = match binding::find_active(&state, auth.uid, &auth.product.product_id).await? {
        Some(active) => quota::paid_total(&state.pool, active.id).await? > 0,
        None => false,
    };
    let items: Vec<Value> = tiers
        .iter()
        .map(|t| {
            json!({
                "tier": t.tier,
                "display_name": t.display_name,
                "description": t.description,
                "is_default": t.is_default,
                "requires_paid": t.requires_paid,
                "available": !t.requires_paid || paid,
            })
        })
        .collect();
    Ok(Json(json!({ "tiers": items })))
}

/// 余额只读不开通：没用过 AI 的账号不会因为打开页面而在 New API 里多出一个用户。
pub async fn balance(State(state): State<AppState>, auth: AuthUser) -> Result<Json<Value>, AppError> {
    let product = &auth.product;
    let mut body = json!({
        "product": { "product_id": product.product_id, "name": product.name },
        "quota_per_unit": state.config.quota_per_unit,
        "total": 0,
        "paid": 0,
        "buckets": [],
        "trial": { "available": product.trial_quota > 0, "quota": product.trial_quota, "days": product.trial_days },
    });
    if let Some(active) = binding::find_active(&state, auth.uid, &product.product_id).await? {
        let buckets = quota::buckets(&state.pool, active.id, true).await?;
        let total: i64 = buckets.iter().map(|b| b.remaining).sum();
        let paid: i64 = buckets.iter().filter(|b| b.kind != "trial").map(|b| b.remaining).sum();
        let (trial_used,): (bool,) = sqlx::query_as("SELECT EXISTS (SELECT 1 FROM quota_buckets WHERE binding_id = $1 AND kind = 'trial')")
            .bind(active.id)
            .fetch_one(&state.pool)
            .await?;
        body["total"] = json!(total);
        body["paid"] = json!(paid);
        body["buckets"] = json!(buckets);
        body["trial"]["available"] = json!(!trial_used && product.trial_quota > 0);
    }
    Ok(Json(body))
}

#[derive(Deserialize)]
pub struct LedgerQuery {
    before: Option<i64>,
    limit: Option<i64>,
}

pub async fn ledger(State(state): State<AppState>, auth: AuthUser, Query(query): Query<LedgerQuery>) -> Result<Json<Value>, AppError> {
    let items = match binding::find_active(&state, auth.uid, &auth.product.product_id).await? {
        Some(active) => quota::ledger(&state.pool, active.id, query.before, query.limit.unwrap_or(50)).await?,
        None => Vec::new(),
    };
    Ok(Json(json!({ "items": items })))
}
