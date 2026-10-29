use crate::models::category::CATEGORIES;
use axum::Json;
use serde_json::{json, Value};

/// GET /api/v1/categories
pub async fn list_categories() -> Json<Value> {
    Json(json!({
        "success": true,
        "total": CATEGORIES.len(),
        "categories": CATEGORIES
    }))
}
