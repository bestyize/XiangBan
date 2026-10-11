use crate::{error::AppResult, state::AppState};
use axum::{extract::State, Json};
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
    /// 内置标签：发帖时按这个分类给出的可一键选用的标签。
    pub preset_tags: Vec<String>,
    pub topic_count: i64,
}

pub async fn categories(State(state): State<AppState>) -> AppResult<Json<Vec<Category>>> {
    let rows = sqlx::query_as(
        "SELECT c.id, c.parent_id, c.slug, c.name, c.description, c.kind, c.sort_order, c.min_post_level,
                c.on_home, c.award_points, c.admin_only, c.preset_tags,
                (SELECT COUNT(*) FROM topics t WHERE t.category_id = c.id AND t.status = 'normal' AND t.min_read_level = 0) AS topic_count
         FROM categories c ORDER BY c.sort_order, c.id",
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}

pub async fn popular_tags(State(state): State<AppState>) -> AppResult<Json<Vec<Value>>> {
    let rows: Vec<(String, i64)> = sqlx::query_as(
        "SELECT g.name, COUNT(*) AS n FROM topic_tags tt JOIN tags g ON g.id = tt.tag_id
         JOIN topics t ON t.id = tt.topic_id AND t.status = 'normal' AND t.min_read_level = 0
         GROUP BY g.name ORDER BY n DESC, g.name LIMIT 30",
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows.into_iter().map(|(name, count)| json!({ "name": name, "count": count })).collect()))
}
