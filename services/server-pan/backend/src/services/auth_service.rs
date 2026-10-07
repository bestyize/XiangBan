use crate::config::AppConfig;
use crate::db::schema::hash_password_with_salt;
use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::{Claims, LoginRequest, LoginResponse, User, UserDto};
use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};

#[derive(Clone)]
pub struct AuthService {
    db: DbPool,
    config: AppConfig,
}

impl AuthService {
    pub fn new(db: DbPool, config: AppConfig) -> Self {
        Self { db, config }
    }

    pub async fn login(&self, req: LoginRequest) -> AppResult<LoginResponse> {
        let conn = self.db.lock().await;

        let user = conn
            .query_row(
                "SELECT id, username, password_hash, salt, is_admin, created_at FROM users WHERE username = ?1",
                [&req.username],
                |row| {
                    Ok(User {
                        id: row.get(0)?,
                        username: row.get(1)?,
                        password_hash: row.get(2)?,
                        salt: row.get(3)?,
                        is_admin: row.get::<_, i64>(4)? == 1,
                        created_at: row.get(5)?,
                    })
                },
            )
            .map_err(|_| AppError::Unauthorized("用户名或密码错误".to_string()))?;

        // 校验加盐密码
        let incoming_hash = hash_password_with_salt(&req.password, &user.salt);
        if incoming_hash != user.password_hash {
            return Err(AppError::Unauthorized("用户名或密码错误".to_string()));
        }

        // 签发 JWT
        let exp = Utc::now()
            .checked_add_signed(chrono::Duration::hours(self.config.jwt_expiration_hours))
            .ok_or_else(|| AppError::Internal("时间计算溢出".to_string()))?
            .timestamp() as usize;

        let claims = Claims {
            sub: user.username.clone(),
            user_id: user.id,
            is_admin: user.is_admin,
            exp,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.jwt_secret.as_bytes()),
        )
        .map_err(|e| AppError::Internal(format!("JWT 签发失败: {}", e)))?;

        Ok(LoginResponse {
            token,
            user: UserDto::from(user),
        })
    }

    pub async fn get_user_by_id(&self, user_id: i64) -> AppResult<UserDto> {
        let conn = self.db.lock().await;
        let user = conn
            .query_row(
                "SELECT id, username, password_hash, salt, is_admin, created_at FROM users WHERE id = ?1",
                [user_id],
                |row| {
                    Ok(User {
                        id: row.get(0)?,
                        username: row.get(1)?,
                        password_hash: row.get(2)?,
                        salt: row.get(3)?,
                        is_admin: row.get::<_, i64>(4)? == 1,
                        created_at: row.get(5)?,
                    })
                },
            )
            .map_err(|_| AppError::NotFound("用户不存在".to_string()))?;

        Ok(UserDto::from(user))
    }

    pub async fn change_password(&self, user_id: i64, old_pwd: &str, new_pwd: &str) -> AppResult<()> {
        if new_pwd.len() < 6 {
            return Err(AppError::BadRequest("新密码长度不能少于 6 位".to_string()));
        }

        let conn = self.db.lock().await;
        let (current_hash, salt): (String, String) = conn
            .query_row(
                "SELECT password_hash, salt FROM users WHERE id = ?1",
                [user_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|_| AppError::NotFound("用户不存在".to_string()))?;

        let old_hash = hash_password_with_salt(old_pwd, &salt);
        if old_hash != current_hash {
            return Err(AppError::BadRequest("原密码不正确".to_string()));
        }

        let new_salt = uuid::Uuid::new_v4().simple().to_string();
        let new_hash = hash_password_with_salt(new_pwd, &new_salt);

        conn.execute(
            "UPDATE users SET password_hash = ?1, salt = ?2 WHERE id = ?3",
            (&new_hash, &new_salt, user_id),
        )?;

        Ok(())
    }
}
