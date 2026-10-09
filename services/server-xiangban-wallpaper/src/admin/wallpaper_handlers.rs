use crate::{
    api::state::AppState,
    error::AppError,
    models::{
        query::SearchQuery,
        wallpaper::{Author, DownloadUrls, MediaType, Orientation, Resolution, Stats, VideoMeta, VideoQuality},
        WallpaperItem,
    },
};
use axum::{
    extract::{Multipart, Path, Query, State},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::PathBuf;

type ApiResult = Result<Json<Value>, AppError>;

/* ---------------- 采集工作台接口 ---------------- */

/// GET /api/v1/admin/collector/search
pub async fn collector_search(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> ApiResult {
    let result = state.aggregator.search(&query).await?;
    Ok(Json(json!(result)))
}

#[derive(Debug, Deserialize)]
pub struct ImportWallpaperBody {
    pub item: WallpaperItem,
    #[serde(default = "default_status")]
    pub status: String,
    pub category: Option<String>,
}

fn default_status() -> String {
    "published".to_string()
}

/// POST /api/v1/admin/collector/import
pub async fn collector_import(
    State(state): State<AppState>,
    Json(body): Json<ImportWallpaperBody>,
) -> ApiResult {
    let mut item = body.item;
    item.status = body.status;
    if body.category.is_some() {
        item.category = body.category;
    }
    item.source_type = "collected".to_string();
    // The stored copy is recorded by the server only, never taken from the browser.
    item.local_copy = None;
    item.collected_at = Some(chrono::Utc::now());
    crate::desktop::prepare_wallpaper(state.admin.store()?,&mut item).await?;

    state.db.upsert_wallpapers(&[item.clone()]).await?;
    Ok(Json(json!({
        "success": true,
        "message": "壁纸已成功采集入库",
        "item": item
    })))
}

#[derive(Debug, Deserialize)]
pub struct CheckStatusBody {
    pub ids: Vec<String>,
}

/// POST /api/v1/admin/collector/check-status
pub async fn collector_check_status(
    State(state): State<AppState>,
    Json(body): Json<CheckStatusBody>,
) -> ApiResult {
    let collected_ids = state.db.check_collected_ids(&body.ids).await?;
    Ok(Json(json!({
        "collected_ids": collected_ids
    })))
}

/* ---------------- 图库管理接口 ---------------- */

#[derive(Debug, Deserialize)]
pub struct AdminWallpaperQuery {
    pub status: Option<String>,
    pub category: Option<String>,
    pub media_type: Option<String>,
    pub q: Option<String>,
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

fn default_page() -> u32 {
    1
}

fn default_per_page() -> u32 {
    24
}

/// GET /api/v1/admin/wallpapers
pub async fn admin_list_wallpapers(
    State(state): State<AppState>,
    Query(params): Query<AdminWallpaperQuery>,
) -> ApiResult {
    let (items, total) = state
        .db
        .admin_list_wallpapers(
            params.status.as_deref(),
            params.category.as_deref(),
            params.media_type.as_deref(),
            params.q.as_deref(),
            params.page,
            params.per_page,
        )
        .await?;

    Ok(Json(json!({
        "items": items,
        "total": total,
        "page": params.page,
        "per_page": params.per_page
    })))
}

#[derive(Debug, Deserialize)]
pub struct UpdateStatusBody {
    pub status: String,
}

/// PUT /api/v1/admin/wallpapers/{id}/status
pub async fn admin_update_wallpaper_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateStatusBody>,
) -> ApiResult {
    if body.status=="published" {
        let mut item=state.db.get_wallpaper(&id).await?.ok_or_else(||AppError::NotFound("壁纸不存在".into()))?;
        crate::desktop::prepare_wallpaper(state.admin.store()?,&mut item).await?;
        item.status=body.status.clone();state.db.upsert_wallpapers(&[item]).await?;
        return Ok(Json(json!({"success":true})));
    }
    let ok = state.db.update_status(&id, &body.status).await?;
    if !ok {
        return Err(AppError::NotFound("壁纸不存在".to_string()));
    }
    Ok(Json(json!({ "success": true })))
}

#[derive(Debug, Deserialize)]
pub struct UpdateWallpaperMetaBody {
    pub title: String,
    pub category: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default = "default_status")]
    pub status: String,
}

/// PUT /api/v1/admin/wallpapers/{id}
pub async fn admin_update_wallpaper(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateWallpaperMetaBody>,
) -> ApiResult {
    if body.status=="published" {
        let mut item=state.db.get_wallpaper(&id).await?.ok_or_else(||AppError::NotFound("壁纸不存在".into()))?;
        crate::desktop::prepare_wallpaper(state.admin.store()?,&mut item).await?;
        item.title=body.title;item.category=body.category;item.tags=body.tags;item.status=body.status;
        state.db.upsert_wallpapers(&[item]).await?;return Ok(Json(json!({"success":true})));
    }
    let ok = state
        .db
        .update_wallpaper_meta(
            &id,
            &body.title,
            body.category.as_deref(),
            &body.tags,
            &body.status,
        )
        .await?;
    if !ok {
        return Err(AppError::NotFound("壁纸不存在".to_string()));
    }
    Ok(Json(json!({ "success": true })))
}

/// DELETE /api/v1/admin/wallpapers/{id}
pub async fn admin_delete_wallpaper(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult {
    let ok = state.db.delete_wallpaper(&id).await?;
    if !ok {
        return Err(AppError::NotFound("壁纸不存在".to_string()));
    }
    Ok(Json(json!({ "success": true })))
}

/* ---------------- 文件上传与自定义上架 ---------------- */

/// POST /api/v1/admin/wallpapers/upload
pub async fn admin_upload_file(mut multipart: Multipart) -> ApiResult {
    tokio::fs::create_dir_all("uploads").await.map_err(|e| {
        AppError::Internal(format!("创建上传目录失败: {e}"))
    })?;

    let mut saved_files = Vec::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        let original_name = field.file_name().unwrap_or("file").to_string();
        let ext = PathBuf::from(&original_name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let ext_str = if ext.is_empty() { String::new() } else { format!(".{ext}") };
        let unique_name = format!("{}_{}{ext_str}", chrono::Utc::now().timestamp_millis(), uuid::Uuid::new_v4().simple());
        let file_path = format!("uploads/{unique_name}");

        let data = field.bytes().await.map_err(|e| {
            AppError::BadRequest(format!("读取上传数据失败: {e}"))
        })?;
        if data.len() > 256 * 1024 * 1024 {
            return Err(AppError::BadRequest("单个壁纸文件不能超过 256 MiB".into()));
        }

        tokio::fs::write(&file_path, &data).await.map_err(|e| {
            AppError::Internal(format!("保存文件失败: {e}"))
        })?;

        let url = format!("/uploads/{unique_name}");
        saved_files.push(json!({
            "original_name": original_name,
            "filename": unique_name,
            "url": url,
            "size": data.len()
        }));
    }

    if saved_files.is_empty() {
        return Err(AppError::BadRequest("未接收到任何文件".to_string()));
    }

    Ok(Json(json!({
        "success": true,
        "files": saved_files,
        "file": saved_files.first().unwrap()
    })))
}

#[derive(Debug, Deserialize)]
pub struct CreateCustomWallpaperBody {
    pub title: String,
    pub description: Option<String>,
    pub media_type: String, // "image" | "video"
    pub preview_url: String,
    pub regular_url: String,
    pub video_url: Option<String>,
    pub category: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub orientation: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    #[serde(default = "default_status")]
    pub status: String,
}

/// POST /api/v1/admin/wallpapers/custom
pub async fn admin_create_custom_wallpaper(
    State(state): State<AppState>,
    Json(body): Json<CreateCustomWallpaperBody>,
) -> ApiResult {
    let width = body.width.unwrap_or(1920);
    let height = body.height.unwrap_or(1080);
    let orientation = match body.orientation.as_deref() {
        Some("portrait") => Orientation::Portrait,
        Some("squarish") => Orientation::Squarish,
        Some("landscape") => Orientation::Landscape,
        _ => Orientation::from_dimensions(width, height),
    };

    let media_type = match body.media_type.as_str() {
        "video" => MediaType::Video,
        "live_photo" => MediaType::LivePhoto,
        _ => MediaType::Image,
    };

    let uid = uuid::Uuid::new_v4().simple().to_string();
    let id = format!("custom_{uid}");

    let downloads = DownloadUrls {
        original: body.regular_url.clone(),
        large: Some(body.regular_url.clone()),
        medium: Some(body.regular_url.clone()),
        small: Some(body.preview_url.clone()),
        thumb: body.preview_url.clone(),
    };

    let video = if media_type == MediaType::Video {
        let vurl = body.video_url.clone().unwrap_or_else(|| body.regular_url.clone());
        Some(VideoMeta {
            duration_secs: 15,
            preview_url: vurl.clone(),
            qualities: vec![VideoQuality {
                quality: "1080p".to_string(),
                url: vurl,
                width,
                height,
                size_bytes: None,
            }],
        })
    } else {
        None
    };

    let mut item = WallpaperItem {
        id,
        provider: "custom".to_string(),
        source_id: uid,
        media_type,
        title: body.title,
        description: body.description,
        tags: body.tags,
        resolution: Resolution::new(width, height),
        orientation,
        dominant_color: None,
        preview_url: body.preview_url.clone(),
        regular_url: body.regular_url.clone(),
        downloads,
        video,
        author: Author {
            name: "管理员".to_string(),
            username: "admin".to_string(),
            avatar_url: None,
            profile_url: None,
        },
        stats: Stats::default(),
        source_page_url: "".to_string(),
        license: "自有版权 / 自定义上架".to_string(),
        status: body.status,
        source_type: "custom".to_string(),
        category: body.category,
        local_path: None,
        local_copy: None,
        collected_at: Some(chrono::Utc::now()),
    };
    crate::desktop::prepare_wallpaper(state.admin.store()?,&mut item).await?;
    state.db.upsert_wallpapers(&[item.clone()]).await?;

    Ok(Json(json!({
        "success": true,
        "message": "自定义壁纸上架成功",
        "item": item
    })))
}
