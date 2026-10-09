use crate::{
    error::AppError,
    models::{AppItem, AppQuery, Category},
};
use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    response::Redirect,
    Json,
};
use serde_json::json;
use sqlx::{PgPool, Row};

pub async fn list_apps(
    State(pool): State<PgPool>,
    Query(query): Query<AppQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut sql = String::from(
        r#"
        SELECT
            a.id, a.slug, a.name, a.icon_url, a.category_id,
            c.name as category_name, c.slug as category_slug,
            a.summary, a.comment, a.referrer, a.referrer_avatar,
            a.pricing_type, a.platforms, a.badges, a.status,
            a.is_featured, a.sort_order, a.download_url, a.download_alt_url,
            a.official_url, a.appstore_url, a.github_url, a.github_stars,
            a.brew_cask, a.winget_id, a.mac_download_url, a.win_download_url,
            a.linux_download_url, a.file_size, a.release_notes, a.auto_sync_source, a.last_synced_at,
            a.pros, a.cons, a.alternatives,
            a.version, a.downloads_count, a.created_at, a.updated_at
        FROM appstore_apps a
        JOIN appstore_categories c ON a.category_id = c.id
        WHERE a.status = 'published'
        "#,
    );

    if let Some(cat) = &query.category {
        if !cat.is_empty() && cat != "all" {
            sql.push_str(&format!(" AND (c.slug = '{cat}' OR c.name = '{cat}')"));
        }
    }

    if let Some(s) = &query.search {
        let clean = s.replace('\'', "''");
        if !clean.is_empty() {
            sql.push_str(&format!(
                " AND (a.name ILIKE '%{clean}%' OR a.summary ILIKE '%{clean}%' OR a.comment ILIKE '%{clean}%')"
            ));
        }
    }

    if let Some(tag) = &query.tag {
        let clean = tag.replace('\'', "''");
        if !clean.is_empty() && clean != "all" {
            sql.push_str(&format!(" AND a.badges @> '[\"{clean}\"]'::jsonb"));
        }
    }

    if let Some(platform) = &query.platform {
        let clean = platform.replace('\'', "''");
        if !clean.is_empty() && clean != "all" {
            sql.push_str(&format!(" AND a.platforms @> '[\"{clean}\"]'::jsonb"));
        }
    }

    sql.push_str(" ORDER BY a.sort_order DESC, a.created_at DESC");

    let apps: Vec<AppItem> = sqlx::query_as(&sql).fetch_all(&pool).await?;

    Ok(Json(json!({
        "apps": apps,
        "total": apps.len(),
    })))
}

pub async fn get_app_by_slug(
    State(pool): State<PgPool>,
    Path(slug): Path<String>,
) -> Result<Json<AppItem>, AppError> {
    let app = sqlx::query_as::<_, AppItem>(
        r#"
        SELECT
            a.id, a.slug, a.name, a.icon_url, a.category_id,
            c.name as category_name, c.slug as category_slug,
            a.summary, a.comment, a.referrer, a.referrer_avatar,
            a.pricing_type, a.platforms, a.badges, a.status,
            a.is_featured, a.sort_order, a.download_url, a.download_alt_url,
            a.official_url, a.appstore_url, a.github_url, a.github_stars,
            a.brew_cask, a.winget_id, a.mac_download_url, a.win_download_url,
            a.linux_download_url, a.file_size, a.release_notes, a.auto_sync_source, a.last_synced_at,
            a.pros, a.cons, a.alternatives,
            a.version, a.downloads_count, a.created_at, a.updated_at
        FROM appstore_apps a
        JOIN appstore_categories c ON a.category_id = c.id
        WHERE a.slug = $1 AND a.status = 'published'
        "#,
    )
    .bind(slug)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound("应用未找到或已下架".to_string()))?;

    Ok(Json(app))
}

pub async fn list_categories(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<Category>>, AppError> {
    let categories = sqlx::query_as::<_, Category>(
        r#"
        SELECT
            c.id, c.slug, c.name, c.icon, c.description, c.sort_order,
            COUNT(CASE WHEN a.status = 'published' THEN a.id ELSE NULL END) as app_count
        FROM appstore_categories c
        LEFT JOIN appstore_apps a ON c.id = a.category_id
        GROUP BY c.id
        ORDER BY c.sort_order DESC, c.id ASC
        "#,
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(categories))
}

pub async fn get_featured_apps(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<AppItem>>, AppError> {
    let apps = sqlx::query_as::<_, AppItem>(
        r#"
        SELECT
            a.id, a.slug, a.name, a.icon_url, a.category_id,
            c.name as category_name, c.slug as category_slug,
            a.summary, a.comment, a.referrer, a.referrer_avatar,
            a.pricing_type, a.platforms, a.badges, a.status,
            a.is_featured, a.sort_order, a.download_url, a.download_alt_url,
            a.official_url, a.appstore_url, a.github_url, a.github_stars,
            a.brew_cask, a.winget_id, a.mac_download_url, a.win_download_url,
            a.linux_download_url, a.file_size, a.release_notes, a.auto_sync_source, a.last_synced_at,
            a.pros, a.cons, a.alternatives,
            a.version, a.downloads_count, a.created_at, a.updated_at
        FROM appstore_apps a
        JOIN appstore_categories c ON a.category_id = c.id
        WHERE a.status = 'published' AND a.is_featured = true
        ORDER BY a.sort_order DESC, a.created_at DESC
        LIMIT 6
        "#,
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(apps))
}

pub async fn track_download(
    State(pool): State<PgPool>,
    Path(slug): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("UPDATE appstore_apps SET downloads_count = downloads_count + 1 WHERE slug = $1")
        .bind(slug)
        .execute(&pool)
        .await?;

    Ok(Json(json!({ "success": true })))
}

// 核心端点：直接下载安装包，根据访客系统智能重定向到对应直接安装包 (.dmg / .exe / .AppImage)
pub async fn direct_download(
    State(pool): State<PgPool>,
    Path(slug): Path<String>,
    headers: HeaderMap,
) -> Result<Redirect, AppError> {
    let row = sqlx::query(
        "SELECT download_url, mac_download_url, win_download_url, linux_download_url FROM appstore_apps WHERE slug = $1 AND status = 'published'"
    )
    .bind(&slug)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound("应用未找到或已下架".to_string()))?;

    // 累加下载量
    let _ = sqlx::query("UPDATE appstore_apps SET downloads_count = downloads_count + 1 WHERE slug = $1")
        .bind(&slug)
        .execute(&pool)
        .await;

    let user_agent = headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_lowercase();

    let mac_url: Option<String> = row.get("mac_download_url");
    let win_url: Option<String> = row.get("win_download_url");
    let linux_url: Option<String> = row.get("linux_download_url");
    let default_url: String = row.get("download_url");

    let target = if (user_agent.contains("macintosh") || user_agent.contains("mac os")) && mac_url.is_some() {
        mac_url.unwrap()
    } else if user_agent.contains("windows") && win_url.is_some() {
        win_url.unwrap()
    } else if user_agent.contains("linux") && linux_url.is_some() {
        linux_url.unwrap()
    } else if let Some(m) = mac_url {
        m
    } else if let Some(w) = win_url {
        w
    } else {
        default_url
    };

    Ok(Redirect::temporary(&target))
}
