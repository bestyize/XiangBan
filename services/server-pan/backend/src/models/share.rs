use serde::{Deserialize, Serialize};
use super::file::{BreadcrumbItem, FileItem};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareItem {
    pub code: String,
    pub user_id: i64,
    pub file_id: String,
    pub password: Option<String>,
    pub note: Option<String>,
    pub expires_at: Option<i64>,
    pub view_count: i64,
    pub download_count: i64,
    pub created_at: i64,
}

#[derive(Debug, Serialize)]
pub struct ShareDto {
    pub code: String,
    pub file_id: String,
    pub file_name: String,
    pub is_dir: bool,
    pub file_size: i64,
    pub has_password: bool,
    pub password: Option<String>, // 创建者自己看可以看到明文提取码
    pub note: Option<String>,     // 分享说明（可选）
    pub expires_at: Option<i64>,
    pub is_expired: bool,
    pub view_count: i64,
    pub download_count: i64,
    pub created_at: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateShareRequest {
    pub file_id: String,
    pub password: Option<String>,
    pub note: Option<String>,     // 分享说明（可选）
    pub expire_days: Option<i64>, // None 表示永久有效，也可以是 1, 7, 30 等
}

#[derive(Debug, Deserialize)]
pub struct UpdateShareRequest {
    pub password: Option<String>, // None 表示不修改；空串表示取消密码保护
    pub note: Option<String>,     // None 表示不修改；空串表示清空说明
    pub expire_days: Option<i64>, // None 表示不修改；0 表示改为永久有效
}

#[derive(Debug, Serialize)]
pub struct CreateShareResponse {
    pub code: String,
    pub share_url: String,
    pub has_password: bool,
    pub password: Option<String>,
    pub note: Option<String>,
    pub expires_at: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct ShareInfoResponse {
    pub code: String,
    pub target_name: String,
    pub is_dir: bool,
    pub file_size: i64,
    pub has_password: bool,
    pub note: Option<String>, // 分享说明（可选）
    pub is_expired: bool,
    pub expires_at: Option<i64>,
    pub creator_name: String,
    pub created_at: i64,
}

#[derive(Debug, Deserialize)]
pub struct VerifyShareRequest {
    pub password: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct VerifyShareResponse {
    pub share_token: String,
}

#[derive(Debug, Serialize)]
pub struct ShareContentResponse {
    pub target_item: FileItem,
    pub current_folder: Option<FileItem>,
    pub breadcrumbs: Vec<BreadcrumbItem>,
    pub items: Vec<FileItem>, // 如果目标是文件，则 items 只有一项；如果是文件夹，则是当前目录子项
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareVisitorClaims {
    pub share_code: String,
    pub exp: usize,
}
