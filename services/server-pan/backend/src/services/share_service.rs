use crate::config::AppConfig;
use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::{
    BreadcrumbItem, CreateShareRequest, CreateShareResponse, FileItem, ShareContentResponse,
    ShareDto, ShareInfoResponse, ShareItem, ShareVisitorClaims, UpdateShareRequest,
    VerifyShareResponse,
};
use crate::services::file_service::FileService;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rand::distr::Alphanumeric;
use rand::Rng;
use rusqlite::params;

#[derive(Clone)]
pub struct ShareService {
    db: DbPool,
    config: AppConfig,
}

impl ShareService {
    pub fn new(db: DbPool, config: AppConfig) -> Self {
        Self { db, config }
    }

    /// 创建分享
    pub async fn create_share(
        &self,
        user_id: i64,
        req: CreateShareRequest,
    ) -> AppResult<CreateShareResponse> {
        let conn = self.db.lock().await;

        // 验证文件存在且属于当前用户
        let file_exists: bool = conn
            .query_row(
                "SELECT 1 FROM files WHERE id = ?1 AND user_id = ?2",
                params![req.file_id, user_id],
                |_| Ok(true),
            )
            .unwrap_or(false);

        if !file_exists {
            return Err(AppError::NotFound("分享的目标文件不存在".to_string()));
        }

        // 生成 6 位随机短码
        let code: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(6)
            .map(char::from)
            .collect();

        let password = req
            .password
            .map(|p| p.trim().to_string())
            .filter(|p| !p.is_empty());

        let note = req
            .note
            .map(|n| n.trim().to_string())
            .filter(|n| !n.is_empty());

        let expires_at = req.expire_days.and_then(|days| {
            if days > 0 {
                Utc::now()
                    .checked_add_signed(Duration::days(days))
                    .map(|dt| dt.timestamp())
            } else {
                None
            }
        });

        let now = Utc::now().timestamp();

        conn.execute(
            "INSERT INTO shares (code, user_id, file_id, password, note, expires_at, view_count, download_count, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, 0, ?7)",
            params![code, user_id, req.file_id, password, note, expires_at, now],
        )?;

        let has_password = password.is_some();

        Ok(CreateShareResponse {
            code: code.clone(),
            share_url: format!("/share/{}", code),
            has_password,
            password,
            note,
            expires_at,
        })
    }

    /// 列出当前用户的分享
    pub async fn list_my_shares(&self, user_id: i64) -> AppResult<Vec<ShareDto>> {
        let conn = self.db.lock().await;
        let mut stmt = conn.prepare(
            "SELECT s.code, s.file_id, f.name, f.is_dir, f.size, s.password, s.note, s.expires_at, s.view_count, s.download_count, s.created_at
             FROM shares s
             JOIN files f ON s.file_id = f.id
             WHERE s.user_id = ?1
             ORDER BY s.created_at DESC",
        )?;

        let now = Utc::now().timestamp();
        let rows = stmt.query_map(params![user_id], |r| {
            let pwd: Option<String> = r.get(5)?;
            let exp: Option<i64> = r.get(7)?;
            let is_expired = exp.map(|e| e < now).unwrap_or(false);

            Ok(ShareDto {
                code: r.get(0)?,
                file_id: r.get(1)?,
                file_name: r.get(2)?,
                is_dir: r.get::<_, i64>(3)? == 1,
                file_size: r.get(4)?,
                has_password: pwd.is_some(),
                password: pwd,
                note: r.get(6)?,
                expires_at: exp,
                is_expired,
                view_count: r.get(8)?,
                download_count: r.get(9)?,
                created_at: r.get(10)?,
            })
        })?;

        let mut list = Vec::new();
        for row in rows {
            list.push(row?);
        }
        Ok(list)
    }

    /// 取消分享
    pub async fn delete_share(&self, user_id: i64, code: &str) -> AppResult<()> {
        let conn = self.db.lock().await;
        let affected = conn.execute(
            "DELETE FROM shares WHERE code = ?1 AND user_id = ?2",
            params![code, user_id],
        )?;

        if affected == 0 {
            return Err(AppError::NotFound("分享不存在或无权操作".to_string()));
        }
        Ok(())
    }

    /// 编辑分享信息（部分更新：只更新请求中显式提供的字段）
    ///
    /// 字段语义：
    /// - `note`：`None` 不修改；`Some("")` 或纯空白清空说明；否则写入新说明
    /// - `password`：`None` 不修改；`Some("")` 取消密码保护；否则设置新提取码
    /// - `expire_days`：`None` 不修改；`Some(0)` 改为永久有效；`Some(n>0)` 从现在起 n 天后过期
    pub async fn update_share(
        &self,
        user_id: i64,
        code: &str,
        req: UpdateShareRequest,
    ) -> AppResult<()> {
        let conn = self.db.lock().await;

        // 确认分享存在且归属当前用户
        let exists: bool = conn
            .query_row(
                "SELECT 1 FROM shares WHERE code = ?1 AND user_id = ?2",
                params![code, user_id],
                |_| Ok(true),
            )
            .unwrap_or(false);
        if !exists {
            return Err(AppError::NotFound("分享不存在或无权操作".to_string()));
        }

        // 动态构造 SET 子句，仅更新显式提供的字段
        let mut sets: Vec<&str> = Vec::new();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(note) = req.note {
            let note_val = {
                let trimmed = note.trim().to_string();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed)
                }
            };
            sets.push("note = ?");
            params_vec.push(Box::new(note_val));
        }

        if let Some(password) = req.password {
            let pwd_val = {
                let trimmed = password.trim().to_string();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed)
                }
            };
            sets.push("password = ?");
            params_vec.push(Box::new(pwd_val));
        }

        if let Some(days) = req.expire_days {
            let expires_at: Option<i64> = if days > 0 {
                Utc::now()
                    .checked_add_signed(Duration::days(days))
                    .map(|dt| dt.timestamp())
            } else {
                None // 0 或负数表示永久有效
            };
            sets.push("expires_at = ?");
            params_vec.push(Box::new(expires_at));
        }

        if sets.is_empty() {
            return Err(AppError::BadRequest("没有需要更新的字段".to_string()));
        }

        let sql = format!(
            "UPDATE shares SET {} WHERE code = ? AND user_id = ?",
            sets.join(", ")
        );
        params_vec.push(Box::new(code.to_string()));
        params_vec.push(Box::new(user_id));

        let params_ref: Vec<&dyn rusqlite::ToSql> =
            params_vec.iter().map(|b| b.as_ref()).collect();
        conn.execute(&sql, params_ref.as_slice())?;

        Ok(())
    }

    /// 公开获取分享基础信息
    pub async fn get_public_share_info(&self, code: &str) -> AppResult<ShareInfoResponse> {
        let conn = self.db.lock().await;
        let share_info = conn.query_row(
            "SELECT s.code, f.name, f.is_dir, f.size, s.password, s.note, s.expires_at, u.username, s.created_at
             FROM shares s
             JOIN files f ON s.file_id = f.id
             JOIN users u ON s.user_id = u.id
             WHERE s.code = ?1",
            params![code],
            |r| {
                let pwd: Option<String> = r.get(4)?;
                let exp: Option<i64> = r.get(6)?;
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, i64>(2)? == 1,
                    r.get::<_, i64>(3)?,
                    pwd.is_some(),
                    r.get::<_, Option<String>>(5)?,
                    exp,
                    r.get::<_, String>(7)?,
                    r.get::<_, i64>(8)?,
                ))
            },
        ).map_err(|_| AppError::NotFound("分享链接不存在或已被删除".to_string()))?;

        let now = Utc::now().timestamp();
        let is_expired = share_info.6.map(|e| e < now).unwrap_or(false);

        Ok(ShareInfoResponse {
            code: share_info.0,
            target_name: share_info.1,
            is_dir: share_info.2,
            file_size: share_info.3,
            has_password: share_info.4,
            note: share_info.5,
            is_expired,
            expires_at: share_info.6,
            creator_name: share_info.7,
            created_at: share_info.8,
        })
    }

    /// 校验提取码并下发临时凭证
    pub async fn verify_share_password(
        &self,
        code: &str,
        input_password: Option<&str>,
    ) -> AppResult<VerifyShareResponse> {
        let conn = self.db.lock().await;

        let (stored_password, expires_at): (Option<String>, Option<i64>) = conn
            .query_row(
                "SELECT password, expires_at FROM shares WHERE code = ?1",
                params![code],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .map_err(|_| AppError::NotFound("分享链接不存在".to_string()))?;

        let now = Utc::now().timestamp();
        if let Some(exp) = expires_at {
            if exp < now {
                return Err(AppError::BadRequest("该分享链接已过期".to_string()));
            }
        }

        if let Some(ref pwd) = stored_password {
            match input_password {
                Some(inp) if inp.trim() == pwd => {}
                _ => return Err(AppError::Unauthorized("提取码错误".to_string())),
            }
        }

        // 签发 24 小时访客 token
        let exp = Utc::now()
            .checked_add_signed(Duration::hours(24))
            .ok_or_else(|| AppError::Internal("时间计算溢出".to_string()))?
            .timestamp() as usize;

        let claims = ShareVisitorClaims {
            share_code: code.to_string(),
            exp,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.jwt_secret.as_bytes()),
        )
        .map_err(|e| AppError::Internal(format!("签发访客凭证失败: {}", e)))?;

        Ok(VerifyShareResponse { share_token: token })
    }

    /// 校验访客是否有权访问该分享
    fn validate_visitor_access(
        &self,
        share: &ShareItem,
        share_token: Option<&str>,
    ) -> AppResult<()> {
        let now = Utc::now().timestamp();
        if let Some(exp) = share.expires_at {
            if exp < now {
                return Err(AppError::Forbidden("分享链接已过期".to_string()));
            }
        }

        if share.password.is_some() {
            let token = share_token
                .ok_or_else(|| AppError::Unauthorized("此分享受提取码保护，请先输入密码".to_string()))?;

            let decoded = decode::<ShareVisitorClaims>(
                token,
                &DecodingKey::from_secret(self.config.jwt_secret.as_bytes()),
                &Validation::default(),
            )
            .map_err(|_| AppError::Unauthorized("提取码验证已过期，请重新输入".to_string()))?;

            if decoded.claims.share_code != share.code {
                return Err(AppError::Unauthorized("无效的提取码凭据".to_string()));
            }
        }

        Ok(())
    }

    /// 获取分享内容（支持文件夹内穿透浏览）
    pub async fn get_share_content(
        &self,
        code: &str,
        sub_folder_id: Option<String>,
        share_token: Option<&str>,
    ) -> AppResult<ShareContentResponse> {
        let conn = self.db.lock().await;

        let share = conn
            .query_row(
                "SELECT code, user_id, file_id, password, note, expires_at, view_count, download_count, created_at
                 FROM shares WHERE code = ?1",
                params![code],
                |r| {
                    Ok(ShareItem {
                        code: r.get(0)?,
                        user_id: r.get(1)?,
                        file_id: r.get(2)?,
                        password: r.get(3)?,
                        note: r.get(4)?,
                        expires_at: r.get(5)?,
                        view_count: r.get(6)?,
                        download_count: r.get(7)?,
                        created_at: r.get(8)?,
                    })
                },
            )
            .map_err(|_| AppError::NotFound("分享链接不存在".to_string()))?;

        self.validate_visitor_access(&share, share_token)?;

        // 增加浏览量计数
        let _ = conn.execute(
            "UPDATE shares SET view_count = view_count + 1 WHERE code = ?1",
            params![code],
        );

        // 获取分享的目标根节点
        let target_item = conn
            .query_row(
                "SELECT id, user_id, parent_id, name, size, mime_type, is_dir, storage_path, created_at, updated_at
                 FROM files WHERE id = ?1",
                params![share.file_id],
                |r| FileService::row_to_file_item(r),
            )
            .map_err(|_| AppError::NotFound("分享的源文件已被删除".to_string()))?;

        if !target_item.is_dir {
            // 单文件分享
            return Ok(ShareContentResponse {
                target_item: target_item.clone(),
                current_folder: None,
                breadcrumbs: vec![BreadcrumbItem {
                    id: Some(target_item.id.clone()),
                    name: target_item.name.clone(),
                }],
                items: vec![target_item],
            });
        }

        // 文件夹分享
        let active_folder_id = sub_folder_id.unwrap_or_else(|| target_item.id.clone());

        // 安全检查：确认 active_folder_id 确实是 target_item 本身或其子孙文件夹
        let is_valid_descendant = if active_folder_id == target_item.id {
            true
        } else {
            Self::check_is_descendant(&conn, &target_item.id, &active_folder_id)?
        };

        if !is_valid_descendant {
            return Err(AppError::Forbidden("无权访问该子目录".to_string()));
        }

        let current_folder = conn
            .query_row(
                "SELECT id, user_id, parent_id, name, size, mime_type, is_dir, storage_path, created_at, updated_at
                 FROM files WHERE id = ?1",
                params![active_folder_id],
                |r| FileService::row_to_file_item(r),
            )
            .map_err(|_| AppError::NotFound("目录不存在".to_string()))?;

        // 构造面包屑（以分享根目录为起点）
        let mut breadcrumbs = Vec::new();
        let mut curr = current_folder.clone();
        breadcrumbs.push(BreadcrumbItem {
            id: Some(curr.id.clone()),
            name: curr.name.clone(),
        });

        while curr.id != target_item.id {
            if let Some(pid) = curr.parent_id {
                if let Ok(p) = conn.query_row(
                    "SELECT id, user_id, parent_id, name, size, mime_type, is_dir, storage_path, created_at, updated_at
                     FROM files WHERE id = ?1",
                    params![pid],
                    |r| FileService::row_to_file_item(r),
                ) {
                    curr = p;
                    breadcrumbs.push(BreadcrumbItem {
                        id: Some(curr.id.clone()),
                        name: curr.name.clone(),
                    });
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        breadcrumbs.reverse();

        // 查询子项
        let mut stmt = conn.prepare(
            "SELECT id, user_id, parent_id, name, size, mime_type, is_dir, storage_path, created_at, updated_at
             FROM files WHERE parent_id = ?1
             ORDER BY is_dir DESC, name ASC",
        )?;

        let rows = stmt.query_map(params![active_folder_id], |r| {
            FileService::row_to_file_item(r)
        })?;

        let mut items = Vec::new();
        for r in rows {
            items.push(r?);
        }

        Ok(ShareContentResponse {
            target_item,
            current_folder: Some(current_folder),
            breadcrumbs,
            items,
        })
    }

    /// 访客下载/预览校验
    pub async fn get_share_file_for_access(
        &self,
        code: &str,
        file_id: &str,
        share_token: Option<&str>,
        is_download: bool,
    ) -> AppResult<FileItem> {
        let conn = self.db.lock().await;

        let share = conn
            .query_row(
                "SELECT code, user_id, file_id, password, note, expires_at, view_count, download_count, created_at
                 FROM shares WHERE code = ?1",
                params![code],
                |r| {
                    Ok(ShareItem {
                        code: r.get(0)?,
                        user_id: r.get(1)?,
                        file_id: r.get(2)?,
                        password: r.get(3)?,
                        note: r.get(4)?,
                        expires_at: r.get(5)?,
                        view_count: r.get(6)?,
                        download_count: r.get(7)?,
                        created_at: r.get(8)?,
                    })
                },
            )
            .map_err(|_| AppError::NotFound("分享链接不存在".to_string()))?;

        self.validate_visitor_access(&share, share_token)?;

        // 校验目标文件是否合法（必须是根文件本身，或者是根文件夹的子孙文件）
        let target_file = conn
            .query_row(
                "SELECT id, user_id, parent_id, name, size, mime_type, is_dir, storage_path, created_at, updated_at
                 FROM files WHERE id = ?1",
                params![file_id],
                |r| FileService::row_to_file_item(r),
            )
            .map_err(|_| AppError::NotFound("文件不存在".to_string()))?;

        if target_file.is_dir {
            return Err(AppError::BadRequest("目录无法直接下载".to_string()));
        }

        let is_valid = if target_file.id == share.file_id {
            true
        } else {
            Self::check_is_descendant(&conn, &share.file_id, &target_file.id)?
        };

        if !is_valid {
            return Err(AppError::Forbidden("无权访问该文件".to_string()));
        }

        if is_download {
            let _ = conn.execute(
                "UPDATE shares SET download_count = download_count + 1 WHERE code = ?1",
                params![code],
            );
        }

        Ok(target_file)
    }

    // 辅助检查：child_id 是否是 ancestor_id 的子孙
    fn check_is_descendant(
        conn: &rusqlite::Connection,
        ancestor_id: &str,
        child_id: &str,
    ) -> AppResult<bool> {
        let mut curr_id = Some(child_id.to_string());
        while let Some(cid) = curr_id {
            let parent_id: Option<String> = conn
                .query_row(
                    "SELECT parent_id FROM files WHERE id = ?1",
                    params![cid],
                    |r| r.get(0),
                )
                .map_err(|_| AppError::NotFound("节点未找到".to_string()))?;

            if let Some(ref pid) = parent_id {
                if pid == ancestor_id {
                    return Ok(true);
                }
            }
            curr_id = parent_id;
        }
        Ok(false)
    }
}
