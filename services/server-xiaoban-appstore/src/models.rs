use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Category {
    pub id: i32,
    pub slug: String,
    pub name: String,
    pub icon: String,
    pub description: String,
    pub sort_order: i32,
    #[sqlx(default)]
    pub app_count: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AppItem {
    pub id: i32,
    pub slug: String,
    pub name: String,
    pub icon_url: String,
    pub category_id: i32,
    #[sqlx(default)]
    pub category_name: Option<String>,
    #[sqlx(default)]
    pub category_slug: Option<String>,
    pub summary: String,
    pub comment: String,
    pub referrer: String,
    pub referrer_avatar: Option<String>,
    pub pricing_type: String,
    pub platforms: serde_json::Value,
    pub badges: serde_json::Value,
    pub status: String, // "published", "unlisted", "draft"
    pub is_featured: bool,
    pub sort_order: i32,
    pub download_url: String,
    pub download_alt_url: Option<String>,
    pub official_url: Option<String>,
    pub appstore_url: Option<String>,
    pub github_url: Option<String>,
    pub github_stars: Option<i32>,
    pub brew_cask: Option<String>,
    pub winget_id: Option<String>,
    pub mac_download_url: Option<String>,
    pub win_download_url: Option<String>,
    pub linux_download_url: Option<String>,
    pub file_size: Option<String>,
    pub release_notes: Option<String>,
    pub auto_sync_source: Option<String>,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub pros: serde_json::Value,
    pub cons: serde_json::Value,
    pub alternatives: serde_json::Value,
    pub version: Option<String>,
    pub downloads_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct AppQuery {
    pub category: Option<String>,
    pub search: Option<String>,
    pub tag: Option<String>,
    pub platform: Option<String>,
    pub status: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAppRequest {
    pub slug: String,
    pub name: String,
    pub icon_url: String,
    pub category_id: i32,
    pub summary: String,
    pub comment: String,
    pub referrer: Option<String>,
    pub referrer_avatar: Option<String>,
    pub pricing_type: String,
    pub platforms: serde_json::Value,
    pub badges: serde_json::Value,
    pub status: Option<String>,
    pub is_featured: Option<bool>,
    pub sort_order: Option<i32>,
    pub download_url: String,
    pub download_alt_url: Option<String>,
    pub official_url: Option<String>,
    pub appstore_url: Option<String>,
    pub github_url: Option<String>,
    pub github_stars: Option<i32>,
    pub brew_cask: Option<String>,
    pub winget_id: Option<String>,
    pub mac_download_url: Option<String>,
    pub win_download_url: Option<String>,
    pub linux_download_url: Option<String>,
    pub file_size: Option<String>,
    pub release_notes: Option<String>,
    pub auto_sync_source: Option<String>,
    pub pros: Option<serde_json::Value>,
    pub cons: Option<serde_json::Value>,
    pub alternatives: Option<serde_json::Value>,
    pub version: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAppRequest {
    pub slug: Option<String>,
    pub name: Option<String>,
    pub icon_url: Option<String>,
    pub category_id: Option<i32>,
    pub summary: Option<String>,
    pub comment: Option<String>,
    pub referrer: Option<String>,
    pub referrer_avatar: Option<String>,
    pub pricing_type: Option<String>,
    pub platforms: Option<serde_json::Value>,
    pub badges: Option<serde_json::Value>,
    pub status: Option<String>,
    pub is_featured: Option<bool>,
    pub sort_order: Option<i32>,
    pub download_url: Option<String>,
    pub download_alt_url: Option<String>,
    pub official_url: Option<String>,
    pub appstore_url: Option<String>,
    pub github_url: Option<String>,
    pub github_stars: Option<i32>,
    pub brew_cask: Option<String>,
    pub winget_id: Option<String>,
    pub mac_download_url: Option<String>,
    pub win_download_url: Option<String>,
    pub linux_download_url: Option<String>,
    pub file_size: Option<String>,
    pub release_notes: Option<String>,
    pub auto_sync_source: Option<String>,
    pub pros: Option<serde_json::Value>,
    pub cons: Option<serde_json::Value>,
    pub alternatives: Option<serde_json::Value>,
    pub version: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StatusUpdateRequest {
    pub status: String, // "published" 或 "unlisted"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminClaims {
    pub sub: String, // admin username
    pub exp: usize,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct ChangeAdminCredentialsRequest {
    pub current_password: String,
    pub new_username: Option<String>,
    pub new_password: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub username: String,
}

#[derive(Debug, Serialize)]
pub struct AdminStats {
    pub total_apps: i64,
    pub published_apps: i64,
    pub unlisted_apps: i64,
    pub total_categories: i64,
    pub total_downloads: i64,
}
