use super::admin::AuthAdmin;
use crate::{
    auth::AuthUser,
    community_editor as editorial,
    error::{AppError, AppResult},
    state::AppState,
};
use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
pub fn admin_routes() -> Router<AppState> {
    Router::new()
        .route("/featured", get(featured).put(save_featured))
        .route("/featured/candidates", get(candidates))
        .route("/challenges", get(admin_challenges).post(create_challenge))
        .route(
            "/challenges/{key}",
            get(admin_challenge).put(save_challenge),
        )
        .route("/challenges/{key}/publish", post(publish_challenge))
        .route("/challenges/{key}/close", post(close_challenge))
        .route("/challenges/{key}/unpublish", post(unpublish_challenge))
        .route("/challenges/{key}/entries", get(admin_challenge))
        .route(
            "/challenges/{key}/entries/{pack_id}",
            axum::routing::patch(feature_entry),
        )
}
pub fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/challenges", get(challenges))
        .route("/challenges/{key}", get(challenge))
        .route("/challenges/{key}/entries", post(submit_entry))
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

#[derive(Deserialize)]
struct Page {
    page: Option<i64>,
}
async fn admin_challenges(
    State(s): State<AppState>,
    _: AuthAdmin,
    Query(page): Query<Page>,
) -> AppResult<Json<Value>> {
    Ok(Json(
        editorial::list_challenges(&s.pool, true, page.page.unwrap_or(1)).await?,
    ))
}
async fn challenges(State(s): State<AppState>, Query(page): Query<Page>) -> AppResult<Json<Value>> {
    Ok(Json(
        editorial::list_challenges(&s.pool, false, page.page.unwrap_or(1)).await?,
    ))
}
async fn admin_challenge(
    State(s): State<AppState>,
    _: AuthAdmin,
    Path(key): Path<String>,
    Query(page): Query<Page>,
) -> AppResult<Json<Value>> {
    Ok(Json(
        editorial::challenge_detail(&s.pool, &key, true, page.page.unwrap_or(1)).await?,
    ))
}
async fn challenge(
    State(s): State<AppState>,
    Path(key): Path<String>,
    Query(page): Query<Page>,
) -> AppResult<Json<Value>> {
    let mut detail =
        editorial::challenge_detail(&s.pool, &key, false, page.page.unwrap_or(1)).await?;
    if let Some(base) = s.config.resource_base_url.as_deref() {
        if let Some(items) = detail["entries"]["items"].as_array_mut() {
            for item in items {
                if let Some(key) = item["pack"]["cover_key"].as_str() {
                    item["pack"]["cover_url"] =
                        json!(format!("{}/{}", base.trim_end_matches('/'), key));
                }
            }
        }
    }
    Ok(Json(detail))
}
#[derive(Deserialize)]
struct CreateChallenge {
    key: String,
    #[serde(flatten)]
    input: editorial::ChallengeInput,
}
async fn create_challenge(
    State(s): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Json(input): Json<CreateChallenge>,
) -> AppResult<Json<Value>> {
    if input.input.expected_revision != 0 {
        return Err(AppError::BadRequest(
            "新建挑战的 expected_revision 必须为 0".into(),
        ));
    }
    editorial::save_challenge(&s.pool, &admin.username, &input.key, input.input).await?;
    Ok(Json(
        editorial::challenge_detail(&s.pool, &input.key, true, 1).await?,
    ))
}
async fn save_challenge(
    State(s): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Path(key): Path<String>,
    Json(input): Json<editorial::ChallengeInput>,
) -> AppResult<Json<Value>> {
    editorial::save_challenge(&s.pool, &admin.username, &key, input).await?;
    Ok(Json(
        editorial::challenge_detail(&s.pool, &key, true, 1).await?,
    ))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Revision {
    expected_revision: i64,
}
async fn publish_challenge(
    State(s): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Path(key): Path<String>,
    Json(input): Json<Revision>,
) -> AppResult<Json<Value>> {
    editorial::transition_challenge(
        &s.pool,
        &admin.username,
        &key,
        input.expected_revision,
        "publish",
    )
    .await?;
    Ok(Json(
        editorial::challenge_detail(&s.pool, &key, true, 1).await?,
    ))
}
async fn close_challenge(
    State(s): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Path(key): Path<String>,
    Json(input): Json<Revision>,
) -> AppResult<Json<Value>> {
    editorial::transition_challenge(
        &s.pool,
        &admin.username,
        &key,
        input.expected_revision,
        "close",
    )
    .await?;
    Ok(Json(
        editorial::challenge_detail(&s.pool, &key, true, 1).await?,
    ))
}
async fn unpublish_challenge(
    State(s): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Path(key): Path<String>,
    Json(input): Json<Revision>,
) -> AppResult<Json<Value>> {
    editorial::transition_challenge(
        &s.pool,
        &admin.username,
        &key,
        input.expected_revision,
        "unpublish",
    )
    .await?;
    Ok(Json(
        editorial::challenge_detail(&s.pool, &key, true, 1).await?,
    ))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EntryInput {
    pack_id: i64,
}
async fn submit_entry(
    State(s): State<AppState>,
    user: AuthUser,
    Path(key): Path<String>,
    Json(input): Json<EntryInput>,
) -> AppResult<Json<Value>> {
    editorial::submit_entry(&s.pool, &key, input.pack_id, &user.user).await?;
    Ok(Json(json!({"status":"submitted","pack_id":input.pack_id})))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FeatureEntry {
    expected_revision: i64,
    featured: bool,
    sort_order: i32,
}
async fn feature_entry(
    State(s): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Path((key, pack_id)): Path<(String, i64)>,
    Json(input): Json<FeatureEntry>,
) -> AppResult<Json<Value>> {
    editorial::feature_entry(
        &s.pool,
        &admin.username,
        &key,
        pack_id,
        input.expected_revision,
        input.featured,
        input.sort_order,
    )
    .await?;
    Ok(Json(json!({"ok":true})))
}
#[cfg(test)]
#[path = "community_editor_tests.rs"]
mod tests;
