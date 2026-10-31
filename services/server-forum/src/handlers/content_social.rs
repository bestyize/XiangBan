use super::admin::AuthAdmin;
use crate::{auth::AuthUser, content_social as social, error::AppResult, state::AppState};
use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/content-social/{kind}/{id}", get(state))
        .route(
            "/content-social/{kind}/{id}/bookmark",
            axum::routing::put(bookmark).delete(unbookmark),
        )
        .route("/content-social/{kind}/{id}/report", post(report))
        .route("/me/content-bookmarks", get(bookmarks))
}
/// 由宿主嵌入 /api/admin/content-social，全部要求管理员凭据。
pub fn admin_routes() -> Router<AppState> {
    Router::new()
        .route("/reports", get(reports))
        .route("/reports/{id}/handle", post(handle))
}
async fn state(
    State(s): State<AppState>,
    auth: AuthUser,
    Path((kind, id)): Path<(String, i64)>,
) -> AppResult<Json<Value>> {
    Ok(Json(
        social::state(&s.pool, auth.user.uid, &kind, id).await?,
    ))
}
async fn bookmark(
    State(s): State<AppState>,
    auth: AuthUser,
    Path((kind, id)): Path<(String, i64)>,
) -> AppResult<Json<Value>> {
    Ok(Json(
        social::bookmark(&s.pool, auth.user.uid, &kind, id, true).await?,
    ))
}
async fn unbookmark(
    State(s): State<AppState>,
    auth: AuthUser,
    Path((kind, id)): Path<(String, i64)>,
) -> AppResult<Json<Value>> {
    Ok(Json(
        social::bookmark(&s.pool, auth.user.uid, &kind, id, false).await?,
    ))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Report {
    reason: String,
    #[serde(default)]
    detail: String,
}
async fn report(
    State(s): State<AppState>,
    auth: AuthUser,
    Path((kind, id)): Path<(String, i64)>,
    Json(input): Json<Report>,
) -> AppResult<Json<Value>> {
    social::report(
        &s.pool,
        auth.user.uid,
        &kind,
        id,
        &input.reason,
        &input.detail,
    )
    .await?;
    Ok(Json(json!({"ok":true})))
}
#[derive(Deserialize)]
struct Filter {
    kind: Option<String>,
    page: Option<i64>,
}
async fn bookmarks(
    State(s): State<AppState>,
    auth: AuthUser,
    Query(input): Query<Filter>,
) -> AppResult<Json<Value>> {
    Ok(Json(
        social::bookmarks(
            &s,
            auth.user.uid,
            input.kind.as_deref(),
            input.page.unwrap_or(1),
        )
        .await?,
    ))
}
#[derive(Deserialize)]
struct ReportFilter {
    status: Option<String>,
    page: Option<i64>,
}
async fn reports(
    State(s): State<AppState>,
    _: AuthAdmin,
    Query(input): Query<ReportFilter>,
) -> AppResult<Json<Value>> {
    Ok(Json(
        social::reports(
            &s.pool,
            input.status.as_deref().unwrap_or("open"),
            input.page.unwrap_or(1),
        )
        .await?,
    ))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Handle {
    expected_revision: i64,
    action: String,
    note: String,
}
async fn handle(
    State(s): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Path(id): Path<i64>,
    Json(input): Json<Handle>,
) -> AppResult<Json<Value>> {
    Ok(Json(
        social::handle_report(
            &s.pool,
            &admin,
            id,
            input.expected_revision,
            &input.action,
            &input.note,
        )
        .await?,
    ))
}
#[cfg(test)]
#[path = "content_social_tests.rs"]
mod tests;
