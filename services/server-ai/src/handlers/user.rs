use crate::{
    binding,
    error::AppError,
    extract::{AuthUser, ClientIp},
    products, quota, redeem,
    state::AppState,
};
use axum::{
    extract::{Query, State},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::FromRow;

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

/// 给用户看的额度桶：不含来源（里面有补发的管理员用户名）和内部关闭原因。
#[derive(Serialize)]
struct BucketView {
    id: i64,
    kind: String,
    total: i64,
    remaining: i64,
    expires_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
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
        let views: Vec<BucketView> = buckets
            .into_iter()
            .map(|b| BucketView { id: b.id, kind: b.kind, total: b.total, remaining: b.remaining, expires_at: b.expires_at, created_at: b.created_at })
            .collect();
        body["total"] = json!(total);
        body["paid"] = json!(paid);
        body["buckets"] = json!(views);
        body["trial"]["available"] = json!(!trial_used && product.trial_quota > 0);
    }
    Ok(Json(body))
}

#[derive(Deserialize)]
pub struct LedgerQuery {
    before: Option<i64>,
    limit: Option<i64>,
}

#[derive(FromRow)]
struct LedgerSource {
    id: i64,
    kind: String,
    delta: i64,
    balance_after: i64,
    created_at: DateTime<Utc>,
    source: Option<String>,
}

#[derive(Serialize)]
struct LedgerView {
    id: i64,
    kind: String,
    title: String,
    delta: i64,
    balance_after: i64,
    created_at: DateTime<Utc>,
}

/// 流水的标题由类型和来源推出，不使用管理员写的备注（里面可能有订单号等内部信息）。
fn ledger_title(kind: &str, source: Option<&str>) -> &'static str {
    match (kind, source) {
        ("grant", Some("trial")) => "新用户试用额度",
        ("grant", Some(s)) if s.starts_with("code:") => "兑换码入账",
        ("grant", _) => "额度补发",
        ("consume" | "consume_overdraft", _) => "AI 对话消耗",
        ("expire", _) => "额度到期作废",
        ("void", _) => "额度调整",
        _ => "额度变动",
    }
}

pub async fn ledger(State(state): State<AppState>, auth: AuthUser, Query(query): Query<LedgerQuery>) -> Result<Json<Value>, AppError> {
    let Some(active) = binding::find_active(&state, auth.uid, &auth.product.product_id).await? else {
        return Ok(Json(json!({ "items": [] })));
    };
    let rows: Vec<LedgerSource> = sqlx::query_as(
        "SELECT l.id, l.kind, l.delta, l.balance_after, l.created_at, b.source
         FROM quota_ledger l LEFT JOIN quota_buckets b ON b.id = l.bucket_id
         WHERE l.binding_id = $1 AND ($2::BIGINT IS NULL OR l.id < $2) ORDER BY l.id DESC LIMIT $3",
    )
    .bind(active.id)
    .bind(query.before)
    .bind(query.limit.unwrap_or(50).clamp(1, 200))
    .fetch_all(&state.pool)
    .await?;
    let items: Vec<LedgerView> = rows
        .into_iter()
        .map(|r| LedgerView {
            title: ledger_title(&r.kind, r.source.as_deref()).to_string(),
            id: r.id,
            kind: r.kind,
            delta: r.delta,
            balance_after: r.balance_after,
            created_at: r.created_at,
        })
        .collect();
    Ok(Json(json!({ "items": items })))
}

#[derive(Deserialize)]
pub struct RedeemReq {
    code: String,
}

pub async fn redeem(State(state): State<AppState>, auth: AuthUser, ClientIp(ip): ClientIp, Json(req): Json<RedeemReq>) -> Result<Json<Value>, AppError> {
    let done = redeem::redeem(&state, auth.uid, &auth.product, &req.code, &ip).await?;
    Ok(Json(json!({
        "ok": true,
        "kind": done.kind,
        "quota": done.quota,
        "expires_at": done.expires_at,
        "balance": done.balance,
    })))
}
