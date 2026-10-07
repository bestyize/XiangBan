use crate::error::{AppError, AppResult};
use crate::models::{ShareContentResponse, ShareInfoResponse, VerifyShareRequest, VerifyShareResponse};
use crate::services::ShareService;
use crate::storage::StorageManager;
use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use tokio_util::io::ReaderStream;

#[derive(Clone)]
pub struct PublicHandlerState {
    pub share_service: ShareService,
    pub storage: StorageManager,
}

#[derive(Deserialize)]
pub struct ShareContentQuery {
    pub sub_folder_id: Option<String>,
    pub token: Option<String>,
}

pub async fn get_share_info(
    State(state): State<PublicHandlerState>,
    Path(code): Path<String>,
) -> AppResult<Json<ShareInfoResponse>> {
    let info = state.share_service.get_public_share_info(&code).await?;
    Ok(Json(info))
}

pub async fn verify_share(
    State(state): State<PublicHandlerState>,
    Path(code): Path<String>,
    Json(payload): Json<VerifyShareRequest>,
) -> AppResult<Json<VerifyShareResponse>> {
    let res = state
        .share_service
        .verify_share_password(&code, payload.password.as_deref())
        .await?;
    Ok(Json(res))
}

pub async fn get_share_content(
    State(state): State<PublicHandlerState>,
    Path(code): Path<String>,
    Query(query): Query<ShareContentQuery>,
    headers: HeaderMap,
) -> AppResult<Json<ShareContentResponse>> {
    // 优先从 Header 获取 X-Share-Token，回退到 Query token
    let token = headers
        .get("X-Share-Token")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
        .or(query.token);

    let content = state
        .share_service
        .get_share_content(&code, query.sub_folder_id, token.as_deref())
        .await?;
    Ok(Json(content))
}

#[derive(Deserialize)]
pub struct ShareTokenQuery {
    pub token: Option<String>,
}

pub async fn download_share_file(
    State(state): State<PublicHandlerState>,
    Path((code, file_id)): Path<(String, String)>,
    Query(query): Query<ShareTokenQuery>,
    headers: HeaderMap,
) -> AppResult<Response> {
    let token = headers
        .get("X-Share-Token")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
        .or(query.token);

    let file_item = state
        .share_service
        .get_share_file_for_access(&code, &file_id, token.as_deref(), true)
        .await?;

    let storage_path = file_item
        .storage_path
        .as_deref()
        .ok_or_else(|| AppError::NotFound("文件数据缺失".to_string()))?;

    let abs_path = state.storage.get_absolute_path(storage_path)?;
    let file = tokio::fs::File::open(&abs_path).await?;
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let mut resp_headers = HeaderMap::new();
    resp_headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&file_item.mime_type)
            .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream")),
    );
    resp_headers.insert(
        header::CONTENT_LENGTH,
        HeaderValue::from_str(&file_item.size.to_string()).unwrap(),
    );

    let encoded_name = percent_encoding::utf8_percent_encode(
        &file_item.name,
        percent_encoding::NON_ALPHANUMERIC,
    )
    .to_string();
    let disposition = format!(
        "attachment; filename=\"{}\"; filename*=UTF-8''{}",
        file_item.name.replace('"', ""),
        encoded_name
    );
    if let Ok(val) = HeaderValue::from_str(&disposition) {
        resp_headers.insert(header::CONTENT_DISPOSITION, val);
    }

    Ok((StatusCode::OK, resp_headers, body).into_response())
}

pub async fn preview_share_file(
    State(state): State<PublicHandlerState>,
    Path((code, file_id)): Path<(String, String)>,
    Query(query): Query<ShareTokenQuery>,
    headers: HeaderMap,
) -> AppResult<Response> {
    let token = headers
        .get("X-Share-Token")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
        .or(query.token);

    let file_item = state
        .share_service
        .get_share_file_for_access(&code, &file_id, token.as_deref(), false)
        .await?;

    let storage_path = file_item
        .storage_path
        .as_deref()
        .ok_or_else(|| AppError::NotFound("文件数据缺失".to_string()))?;

    let abs_path = state.storage.get_absolute_path(storage_path)?;
    let file = tokio::fs::File::open(&abs_path).await?;
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let mut resp_headers = HeaderMap::new();
    resp_headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&file_item.mime_type)
            .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream")),
    );
    resp_headers.insert(
        header::CONTENT_LENGTH,
        HeaderValue::from_str(&file_item.size.to_string()).unwrap(),
    );
    resp_headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static("inline"),
    );

    Ok((StatusCode::OK, resp_headers, body).into_response())
}
