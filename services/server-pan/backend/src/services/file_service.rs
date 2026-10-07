use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::{BreadcrumbItem, FileItem, FileListResponse, StorageStats};
use crate::storage::StorageManager;
use chrono::Utc;
use rusqlite::params;
use uuid::Uuid;

#[derive(Clone)]
pub struct FileService {
    db: DbPool,
    storage: StorageManager,
}

impl FileService {
    pub fn new(db: DbPool, storage: StorageManager) -> Self {
        Self { db, storage }
    }

    /// 列出目录内容及面包屑导航
    pub async fn list_files(
        &self,
        user_id: i64,
        folder_id: Option<String>,
    ) -> AppResult<FileListResponse> {
        let folder_id = folder_id.filter(|s| !s.trim().is_empty());
        let conn = self.db.lock().await;

        let current_folder = match &folder_id {
            Some(fid) => {
                let item = conn
                    .query_row(
                        "SELECT id, user_id, parent_id, name, size, mime_type, is_dir, storage_path, created_at, updated_at
                         FROM files WHERE id = ?1 AND user_id = ?2 AND is_dir = 1",
                        params![fid, user_id],
                        |row| Self::row_to_file_item(row),
                    )
                    .map_err(|_| AppError::NotFound("目标文件夹不存在".to_string()))?;
                Some(item)
            }
            None => None,
        };

        // 计算面包屑导航
        let mut breadcrumbs = vec![BreadcrumbItem {
            id: None,
            name: "全部文件".to_string(),
        }];

        if let Some(mut curr) = current_folder.clone() {
            let mut ancestors = Vec::new();
            ancestors.push(BreadcrumbItem {
                id: Some(curr.id.clone()),
                name: curr.name.clone(),
            });

            while let Some(parent_id) = curr.parent_id {
                let parent_item = conn.query_row(
                    "SELECT id, user_id, parent_id, name, size, mime_type, is_dir, storage_path, created_at, updated_at
                     FROM files WHERE id = ?1 AND user_id = ?2",
                    params![parent_id, user_id],
                    |row| Self::row_to_file_item(row),
                );

                if let Ok(p) = parent_item {
                    ancestors.push(BreadcrumbItem {
                        id: Some(p.id.clone()),
                        name: p.name.clone(),
                    });
                    curr = p;
                } else {
                    break;
                }
            }

            ancestors.reverse();
            breadcrumbs.extend(ancestors);
        }

        // 获取当前层级的所有子文件和子文件夹（文件夹排在前面，按名称升序）
        let mut items = Vec::new();
        if let Some(ref fid) = folder_id {
            let mut stmt = conn.prepare(
                "SELECT id, user_id, parent_id, name, size, mime_type, is_dir, storage_path, created_at, updated_at
                 FROM files WHERE user_id = ?1 AND parent_id = ?2
                 ORDER BY is_dir DESC, name ASC",
            )?;
            let iter = stmt.query_map(params![user_id, fid], Self::row_to_file_item)?;
            for item in iter {
                items.push(item?);
            }
        } else {
            let mut stmt = conn.prepare(
                "SELECT id, user_id, parent_id, name, size, mime_type, is_dir, storage_path, created_at, updated_at
                 FROM files WHERE user_id = ?1 AND parent_id IS NULL
                 ORDER BY is_dir DESC, name ASC",
            )?;
            let iter = stmt.query_map(params![user_id], Self::row_to_file_item)?;
            for item in iter {
                items.push(item?);
            }
        }

        Ok(FileListResponse {
            current_folder,
            breadcrumbs,
            items,
        })
    }

    /// 创建新文件夹
    pub async fn create_folder(
        &self,
        user_id: i64,
        name: &str,
        parent_id: Option<String>,
    ) -> AppResult<FileItem> {
        let parent_id = parent_id.filter(|s| !s.trim().is_empty());
        let trimmed_name = name.trim();
        if trimmed_name.is_empty() {
            return Err(AppError::BadRequest("文件夹名称不能为空".to_string()));
        }
        if trimmed_name.contains('/') || trimmed_name.contains('\\') {
            return Err(AppError::BadRequest("文件夹名称不能包含斜杠等特殊字符".to_string()));
        }

        let conn = self.db.lock().await;

        // 验证父文件夹是否存在
        if let Some(ref pid) = parent_id {
            let parent_exists: bool = conn
                .query_row(
                    "SELECT 1 FROM files WHERE id = ?1 AND user_id = ?2 AND is_dir = 1",
                    params![pid, user_id],
                    |_| Ok(true),
                )
                .unwrap_or(false);
            if !parent_exists {
                return Err(AppError::NotFound("父文件夹不存在".to_string()));
            }
        }

        // 检查同名冲突
        let count: i64 = if let Some(ref pid) = parent_id {
            conn.query_row(
                "SELECT COUNT(*) FROM files WHERE user_id = ?1 AND parent_id = ?2 AND name = ?3",
                params![user_id, pid, trimmed_name],
                |r| r.get(0),
            )?
        } else {
            conn.query_row(
                "SELECT COUNT(*) FROM files WHERE user_id = ?1 AND parent_id IS NULL AND name = ?2",
                params![user_id, trimmed_name],
                |r| r.get(0),
            )?
        };

        if count > 0 {
            return Err(AppError::BadRequest("同级目录下已存在同名文件或文件夹".to_string()));
        }

        let new_id = Uuid::new_v4().to_string();
        let now = Utc::now().timestamp();

        conn.execute(
            "INSERT INTO files (id, user_id, parent_id, name, size, mime_type, is_dir, storage_path, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, 0, 'inode/directory', 1, NULL, ?5, ?5)",
            params![new_id, user_id, parent_id, trimmed_name, now],
        )?;

        Ok(FileItem {
            id: new_id,
            user_id,
            parent_id,
            name: trimmed_name.to_string(),
            size: 0,
            mime_type: "inode/directory".to_string(),
            is_dir: true,
            storage_path: None,
            created_at: now,
            updated_at: now,
        })
    }

    /// 保存上传文件并登记元数据
    pub async fn record_uploaded_file(
        &self,
        user_id: i64,
        parent_id: Option<String>,
        original_name: &str,
        size: i64,
        mime_type: &str,
        storage_path: &str,
    ) -> AppResult<FileItem> {
        let parent_id = parent_id.filter(|s| !s.trim().is_empty());
        let conn = self.db.lock().await;

        // 处理同名冲突（如 file.txt 重名，递增为 file (1).txt）
        let final_name = Self::generate_unique_name(&conn, user_id, parent_id.as_deref(), original_name)?;

        let new_id = Uuid::new_v4().to_string();
        let now = Utc::now().timestamp();

        conn.execute(
            "INSERT INTO files (id, user_id, parent_id, name, size, mime_type, is_dir, storage_path, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, ?7, ?8, ?8)",
            params![new_id, user_id, parent_id, final_name, size, mime_type, storage_path, now],
        )?;

        Ok(FileItem {
            id: new_id,
            user_id,
            parent_id,
            name: final_name,
            size,
            mime_type: mime_type.to_string(),
            is_dir: false,
            storage_path: Some(storage_path.to_string()),
            created_at: now,
            updated_at: now,
        })
    }

    /// 重命名
    pub async fn rename(&self, user_id: i64, file_id: &str, new_name: &str) -> AppResult<FileItem> {
        let trimmed_name = new_name.trim();
        if trimmed_name.is_empty() {
            return Err(AppError::BadRequest("新名称不能为空".to_string()));
        }
        if trimmed_name.contains('/') || trimmed_name.contains('\\') {
            return Err(AppError::BadRequest("名称不能包含斜杠".to_string()));
        }

        let conn = self.db.lock().await;

        let item = conn
            .query_row(
                "SELECT id, user_id, parent_id, name, size, mime_type, is_dir, storage_path, created_at, updated_at
                 FROM files WHERE id = ?1 AND user_id = ?2",
                params![file_id, user_id],
                |row| Self::row_to_file_item(row),
            )
            .map_err(|_| AppError::NotFound("文件不存在".to_string()))?;

        // 检查同级同名
        let conflict: bool = if let Some(ref pid) = item.parent_id {
            conn.query_row(
                "SELECT 1 FROM files WHERE user_id = ?1 AND parent_id = ?2 AND name = ?3 AND id != ?4",
                params![user_id, pid, trimmed_name, file_id],
                |_| Ok(true),
            )
            .unwrap_or(false)
        } else {
            conn.query_row(
                "SELECT 1 FROM files WHERE user_id = ?1 AND parent_id IS NULL AND name = ?2 AND id != ?3",
                params![user_id, trimmed_name, file_id],
                |_| Ok(true),
            )
            .unwrap_or(false)
        };

        if conflict {
            return Err(AppError::BadRequest("同级目录下已存在同名项".to_string()));
        }

        let now = Utc::now().timestamp();
        conn.execute(
            "UPDATE files SET name = ?1, updated_at = ?2 WHERE id = ?3 AND user_id = ?4",
            params![trimmed_name, now, file_id, user_id],
        )?;

        let mut updated = item;
        updated.name = trimmed_name.to_string();
        updated.updated_at = now;
        Ok(updated)
    }

    /// 递归删除（如果是文件夹，删除全部子孙文件和物理磁盘内容）
    pub async fn delete(&self, user_id: i64, file_id: &str) -> AppResult<()> {
        let conn = self.db.lock().await;

        let root_item = conn
            .query_row(
                "SELECT id, user_id, parent_id, name, size, mime_type, is_dir, storage_path, created_at, updated_at
                 FROM files WHERE id = ?1 AND user_id = ?2",
                params![file_id, user_id],
                |row| Self::row_to_file_item(row),
            )
            .map_err(|_| AppError::NotFound("文件不存在".to_string()))?;

        // 收集所有需要删除的物理路径
        let mut storage_paths_to_delete = Vec::new();
        if !root_item.is_dir {
            if let Some(path) = root_item.storage_path {
                storage_paths_to_delete.push(path);
            }
        } else {
            // 广度优先搜索所有子孙节点
            let mut queue = vec![root_item.id.clone()];
            while let Some(current_fid) = queue.pop() {
                let mut stmt = conn.prepare(
                    "SELECT id, is_dir, storage_path FROM files WHERE user_id = ?1 AND parent_id = ?2",
                )?;
                let rows = stmt.query_map(params![user_id, current_fid], |r| {
                    Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)? == 1, r.get::<_, Option<String>>(2)?))
                })?;

                for row in rows {
                    let (id, is_dir, s_path) = row?;
                    if is_dir {
                        queue.push(id);
                    } else if let Some(sp) = s_path {
                        storage_paths_to_delete.push(sp);
                    }
                }
            }
        }

        // 从数据库删除（外键约束级联删除其子文件夹和分享）
        conn.execute(
            "DELETE FROM files WHERE id = ?1 AND user_id = ?2",
            params![file_id, user_id],
        )?;

        drop(conn); // 释放锁后再执行磁盘异步 I/O

        // 异步删除物理文件
        for sp in storage_paths_to_delete {
            let _ = self.storage.delete_file(&sp).await;
        }

        Ok(())
    }

    /// 关键词搜索
    pub async fn search(&self, user_id: i64, query: &str) -> AppResult<Vec<FileItem>> {
        let pattern = format!("%{}%", query.trim());
        let conn = self.db.lock().await;

        let mut stmt = conn.prepare(
            "SELECT id, user_id, parent_id, name, size, mime_type, is_dir, storage_path, created_at, updated_at
             FROM files WHERE user_id = ?1 AND name LIKE ?2
             ORDER BY is_dir DESC, name ASC LIMIT 100",
        )?;

        let rows = stmt.query_map(params![user_id, pattern], |row| Self::row_to_file_item(row))?;
        let mut result = Vec::new();
        for r in rows {
            result.push(r?);
        }

        Ok(result)
    }

    /// 获取单个文件信息（校验归属）
    pub async fn get_file(&self, user_id: i64, file_id: &str) -> AppResult<FileItem> {
        let conn = self.db.lock().await;
        conn.query_row(
            "SELECT id, user_id, parent_id, name, size, mime_type, is_dir, storage_path, created_at, updated_at
             FROM files WHERE id = ?1 AND user_id = ?2",
            params![file_id, user_id],
            |row| Self::row_to_file_item(row),
        )
        .map_err(|_| AppError::NotFound("文件不存在或无权访问".to_string()))
    }

    /// 获取存储空间统计信息
    pub async fn get_stats(&self, user_id: i64) -> AppResult<StorageStats> {
        let conn = self.db.lock().await;

        let (total_files, used_bytes): (i64, i64) = conn.query_row(
            "SELECT COUNT(*), COALESCE(SUM(size), 0) FROM files WHERE user_id = ?1 AND is_dir = 0",
            params![user_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;

        let total_folders: i64 = conn.query_row(
            "SELECT COUNT(*) FROM files WHERE user_id = ?1 AND is_dir = 1",
            params![user_id],
            |r| r.get(0),
        )?;

        let total_shares: i64 = conn.query_row(
            "SELECT COUNT(*) FROM shares WHERE user_id = ?1",
            params![user_id],
            |r| r.get(0),
        )?;

        Ok(StorageStats {
            total_files,
            total_folders,
            used_bytes,
            total_shares,
        })
    }

    // Helper: 转换数据库 Row 为 FileItem
    pub fn row_to_file_item(row: &rusqlite::Row) -> rusqlite::Result<FileItem> {
        Ok(FileItem {
            id: row.get(0)?,
            user_id: row.get(1)?,
            parent_id: row.get(2)?,
            name: row.get(3)?,
            size: row.get(4)?,
            mime_type: row.get(5)?,
            is_dir: row.get::<_, i64>(6)? == 1,
            storage_path: row.get(7)?,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
        })
    }

    // Helper: 生成唯一重命名文件名
    fn generate_unique_name(
        conn: &rusqlite::Connection,
        user_id: i64,
        parent_id: Option<&str>,
        original_name: &str,
    ) -> AppResult<String> {
        let (stem, ext) = match original_name.rfind('.') {
            Some(idx) if idx > 0 => (&original_name[..idx], &original_name[idx..]),
            _ => (original_name, ""),
        };

        let mut candidate = original_name.to_string();
        let mut counter = 1;

        loop {
            let exists: bool = if let Some(pid) = parent_id {
                conn.query_row(
                    "SELECT 1 FROM files WHERE user_id = ?1 AND parent_id = ?2 AND name = ?3",
                    params![user_id, pid, candidate],
                    |_| Ok(true),
                )
                .unwrap_or(false)
            } else {
                conn.query_row(
                    "SELECT 1 FROM files WHERE user_id = ?1 AND parent_id IS NULL AND name = ?2",
                    params![user_id, candidate],
                    |_| Ok(true),
                )
                .unwrap_or(false)
            };

            if !exists {
                return Ok(candidate);
            }

            candidate = format!("{} ({}){}", stem, counter, ext);
            counter += 1;
        }
    }
}
