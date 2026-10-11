use crate::{error::{AppError, AppResult}, models::{self, ListFilter}, state::AppState};
use axum::{
    extract::{Path, State},
    Json,
};
use serde::Serialize;
use serde_json::{json, Value};

pub async fn health() -> Json<Value> {
    Json(json!({ "ok": true }))
}

/// 前端启动时读取：站点名、账号中心地址和本站 client_id。
pub async fn site_config(State(state): State<AppState>) -> Json<Value> {
    let c = &state.config;
    Json(json!({
        "site_name": c.site_name,
        "account_base": c.account_issuer,
        "client_id": c.account_client_id,
        "conditions": models::CONDITIONS.iter().map(|(k, b)| json!({ "key": k, "bit": b })).collect::<Vec<_>>(),
    }))
}

#[derive(Serialize, sqlx::FromRow)]
pub struct Category {
    pub id: i32,
    pub parent_id: Option<i32>,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub kind: String,
    pub sort_order: i32,
    pub min_post_level: i16,
    pub on_home: bool,
    pub award_points: bool,
    pub admin_only: bool,
    pub topic_count: i64,
}

pub async fn categories(State(state): State<AppState>) -> AppResult<Json<Vec<Category>>> {
    let rows = sqlx::query_as(
        "SELECT c.id, c.parent_id, c.slug, c.name, c.description, c.kind, c.sort_order, c.min_post_level,
                c.on_home, c.award_points, c.admin_only,
                (SELECT COUNT(*) FROM topics t WHERE t.category_id = c.id AND t.status = 'normal') AS topic_count
         FROM categories c ORDER BY c.sort_order, c.id",
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}

#[derive(Serialize, sqlx::FromRow)]
pub struct Product {
    pub id: i32,
    pub slug: String,
    pub name: String,
    pub vendor: String,
    pub website: String,
    pub description: String,
    pub deal_count: i64,
    pub active_deal_count: i64,
}

const PRODUCT_SELECT: &str = "SELECT p.id, p.slug, p.name, p.vendor, p.website, p.description,
        COUNT(t.id) AS deal_count,
        COUNT(t.id) FILTER (WHERE d.deal_status <> 'expired') AS active_deal_count
    FROM products p
    LEFT JOIN deals d ON d.product_id = p.id
    LEFT JOIN topics t ON t.id = d.topic_id AND t.status = 'normal'";

pub async fn products(State(state): State<AppState>) -> AppResult<Json<Vec<Product>>> {
    let rows = sqlx::query_as(&format!(
        "{PRODUCT_SELECT} GROUP BY p.id ORDER BY active_deal_count DESC, deal_count DESC, p.name"
    ))
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}

pub async fn product(State(state): State<AppState>, Path(slug): Path<String>) -> AppResult<Json<Value>> {
    let product: Product = sqlx::query_as(&format!("{PRODUCT_SELECT} WHERE p.slug = $1 GROUP BY p.id"))
        .bind(&slug)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::not_found("产品"))?;
    // 产品页：在架的羊毛 + 提到这个产品的讨论（按标签匹配产品名）。
    let deals = models::list_topics(
        &state.pool,
        &ListFilter { product: Some(slug.clone()), kind: Some("deal".into()), deal_status: Some("all".into()), sort: Some("new".into()), page_size: Some(50), ..Default::default() },
    )
    .await?;
    let discussions = models::list_topics(
        &state.pool,
        &ListFilter { tag: Some(product.name.to_lowercase()), kind: Some("normal".into()), page_size: Some(20), ..Default::default() },
    )
    .await?;
    Ok(Json(json!({ "product": product, "deals": deals.items, "discussions": discussions.items })))
}

pub async fn popular_tags(State(state): State<AppState>) -> AppResult<Json<Vec<Value>>> {
    let rows: Vec<(String, i64)> = sqlx::query_as(
        "SELECT g.name, COUNT(*) AS n FROM topic_tags tt JOIN tags g ON g.id = tt.tag_id
         JOIN topics t ON t.id = tt.topic_id AND t.status = 'normal'
         GROUP BY g.name ORDER BY n DESC, g.name LIMIT 30",
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows.into_iter().map(|(name, count)| json!({ "name": name, "count": count })).collect()))
}
