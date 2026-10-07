use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileItem {
    pub id: String,
    pub user_id: i64,
    pub parent_id: Option<String>,
    pub name: String,
    pub size: i64,
    pub mime_type: String,
    pub is_dir: bool,
    pub storage_path: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreadcrumbItem {
    pub id: Option<String>,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct FileListResponse {
    pub current_folder: Option<FileItem>,
    pub breadcrumbs: Vec<BreadcrumbItem>,
    pub items: Vec<FileItem>,
}

#[derive(Debug, Deserialize)]
pub struct CreateFolderRequest {
    pub name: String,
    pub parent_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RenameRequest {
    pub new_name: String,
}

#[derive(Debug, Serialize)]
pub struct StorageStats {
    pub total_files: i64,
    pub total_folders: i64,
    pub used_bytes: i64,
    pub total_shares: i64,
}
