use super::admin::AuthAdmin;
use crate::{community_editor as editorial, error::AppResult, state::AppState};
use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
pub fn admin_routes() -> Router<AppState> {
    Router::new()
        .route("/featured", get(featured).put(save_featured))
        .route("/featured/candidates", get(candidates))
}
pub fn public_routes() -> Router<AppState> {
    Router::new()
}
async fn featured(State(s): State<AppState>, _: AuthAdmin) -> AppResult<Json<Value>> {
    Ok(Json(editorial::featured_admin(&s.pool).await?))
}
#[derive(Deserialize)]
struct Search {
    #[serde(default)]
    q: String,
}
async fn candidates(
    State(s): State<AppState>,
    _: AuthAdmin,
    Query(input): Query<Search>,
) -> AppResult<Json<Value>> {
    Ok(Json(
        json!({"items":editorial::candidates(&s.pool,&input.q).await?}),
    ))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FeaturedInput {
    expected_revision: i64,
    items: Vec<editorial::FeaturedItem>,
}
async fn save_featured(
    State(s): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Json(input): Json<FeaturedInput>,
) -> AppResult<Json<Value>> {
    editorial::replace_featured(
        &s.pool,
        &admin.username,
        input.expected_revision,
        input.items,
    )
    .await?;
    Ok(Json(editorial::featured_admin(&s.pool).await?))
}
#[cfg(test)]
#[path = "community_editor_tests.rs"]
mod tests;
