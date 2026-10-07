use crate::error::{AppError, AppResult};
use crate::middleware::AuthUser;
use crate::models::{CreateFolderRequest, FileItem, FileListResponse, RenameRequest};
use crate::services::FileService;
use crate::storage::StorageManager;
use axum::{
    body::Body,
    extract::{Multipart, Path, Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::io::AsyncWriteExt;
use tokio_util::io::ReaderStream;

#[derive(Clone)]
pub struct FileHandlerState {
    pub file_service: FileService,
    pub storage: StorageManager,
}

#[derive(Deserialize)]
pub struct ListQuery {
    pub folder_id: Option<String>,
}

pub async fn list_files(
    auth: AuthUser,
    State(state): State<FileHandlerState>,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<FileListResponse>> {
    let res = state
        .file_service
        .list_files(auth.0.user_id, query.folder_id)
        .await?;
    Ok(Json(res))
}

pub async fn create_folder(
    auth: AuthUser,
    State(state): State<FileHandlerState>,
    Json(payload): Json<CreateFolderRequest>,
) -> AppResult<Json<FileItem>> {
    let item = state
        .file_service
        .create_folder(auth.0.user_id, &payload.name, payload.parent_id)
        .await?;
    Ok(Json(item))
}

#[derive(Deserialize)]
pub struct UploadQuery {
    pub folder_id: Option<String>,
}

pub async fn upload_file(
    auth: AuthUser,
    State(state): State<FileHandlerState>,
    Query(query): Query<UploadQuery>,
    mut multipart: Multipart,
) -> AppResult<Json<Vec<FileItem>>> {
    let mut uploaded_items = Vec::new();

    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("表单解析错误: {}", e)))?
    {
        let original_name = field
            .file_name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unnamed_file".to_string());

        let extension = std::path::Path::new(&original_name)
            .extension()
            .and_then(|s| s.to_str());

        let mime_type = mime_guess::from_path(&original_name)
            .first_or_octet_stream()
            .to_string();

        // 创建异步磁盘文件，流式分块写入（支持几兆到数吉字节大文件，且不占用堆内存）
        let (storage_path, mut target_file) = state.storage.create_file(extension).await?;
        let mut total_size: i64 = 0;

        loop {
            match field.chunk().await {
                Ok(Some(chunk)) => {
                    total_size += chunk.len() as i64;
                    if let Err(e) = target_file.write_all(&chunk).await {
                        let _ = state.storage.delete_file(&storage_path).await;
                        return Err(AppError::Io(e));
                    }
                }
                Ok(None) => break, // 当前文件流式传输完成
                Err(e) => {
                    let _ = state.storage.delete_file(&storage_path).await;
                    return Err(AppError::BadRequest(format!("读取文件流错误: {}", e)));
                }
            }
        }

        if let Err(e) = target_file.flush().await {
            let _ = state.storage.delete_file(&storage_path).await;
            return Err(AppError::Io(e));
        }

        // 登记元数据到数据库
        let item = state
            .file_service
            .record_uploaded_file(
                auth.0.user_id,
                query.folder_id.clone(),
                &original_name,
                total_size,
                &mime_type,
                &storage_path,
            )
            .await?;

        uploaded_items.push(item);
    }

    if uploaded_items.is_empty() {
        return Err(AppError::BadRequest("未接收到任何文件数据".to_string()));
    }

    Ok(Json(uploaded_items))
}

pub async fn rename_file(
    auth: AuthUser,
    State(state): State<FileHandlerState>,
    Path(id): Path<String>,
    Json(payload): Json<RenameRequest>,
) -> AppResult<Json<FileItem>> {
    let item = state
        .file_service
        .rename(auth.0.user_id, &id, &payload.new_name)
        .await?;
    Ok(Json(item))
}

pub async fn delete_file(
    auth: AuthUser,
    State(state): State<FileHandlerState>,
    Path(id): Path<String>,
) -> AppResult<Json<Value>> {
    state.file_service.delete(auth.0.user_id, &id).await?;
    Ok(Json(json!({ "message": "删除成功" })))
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

pub async fn search_files(
    auth: AuthUser,
    State(state): State<FileHandlerState>,
    Query(query): Query<SearchQuery>,
) -> AppResult<Json<Vec<FileItem>>> {
    let items = state
        .file_service
        .search(auth.0.user_id, &query.q)
        .await?;
    Ok(Json(items))
}

pub async fn download_file(
    auth: AuthUser,
    State(state): State<FileHandlerState>,
    Path(id): Path<String>,
) -> AppResult<Response> {
    let item = state.file_service.get_file(auth.0.user_id, &id).await?;
    if item.is_dir {
        return Err(AppError::BadRequest("文件夹无法直接单文件下载".to_string()));
    }

    let storage_path = item
        .storage_path
        .as_deref()
        .ok_or_else(|| AppError::NotFound("文件物理数据丢失".to_string()))?;

    let abs_path = state.storage.get_absolute_path(storage_path)?;
    let file = tokio::fs::File::open(&abs_path).await?;
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&item.mime_type)
            .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream")),
    );
    headers.insert(
        header::CONTENT_LENGTH,
        HeaderValue::from_str(&item.size.to_string()).unwrap(),
    );

    // 标准 RFC 5987 编码文件名
    let encoded_name = percent_encoding::utf8_percent_encode(
        &item.name,
        percent_encoding::NON_ALPHANUMERIC,
    )
    .to_string();
    let disposition = format!(
        "attachment; filename=\"{}\"; filename*=UTF-8''{}",
        item.name.replace('"', ""),
        encoded_name
    );
    if let Ok(val) = HeaderValue::from_str(&disposition) {
        headers.insert(header::CONTENT_DISPOSITION, val);
    }

    Ok((StatusCode::OK, headers, body).into_response())
}

pub async fn preview_file(
    auth: AuthUser,
    State(state): State<FileHandlerState>,
    Path(id): Path<String>,
) -> AppResult<Response> {
    let item = state.file_service.get_file(auth.0.user_id, &id).await?;
    if item.is_dir {
        return Err(AppError::BadRequest("文件夹不支持内联预览".to_string()));
    }

    let storage_path = item
        .storage_path
        .as_deref()
        .ok_or_else(|| AppError::NotFound("文件物理数据丢失".to_string()))?;

    let abs_path = state.storage.get_absolute_path(storage_path)?;
    let file = tokio::fs::File::open(&abs_path).await?;
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&item.mime_type)
            .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream")),
    );
    headers.insert(
        header::CONTENT_LENGTH,
        HeaderValue::from_str(&item.size.to_string()).unwrap(),
    );
    // inline 显示
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static("inline"),
    );

    Ok((StatusCode::OK, headers, body).into_response())
}
