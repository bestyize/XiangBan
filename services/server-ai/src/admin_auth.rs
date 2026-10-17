use crate::{config::AppConfig, error::AppError, password, state::AppState};
use chrono::Utc;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminClaims {
    pub sub: String,
    pub tv: i32,
    pub exp: i64,
    pub iat: i64,
}

pub fn issue(secret: &[u8], admin_id: Uuid, token_version: i32, ttl_secs: i64) -> Result<String, AppError> {
    let now = Utc::now().timestamp();
    let claims = AdminClaims { sub: admin_id.to_string(), tv: token_version, exp: now + ttl_secs, iat: now };
    encode(&Header::new(Algorithm::HS256), &claims, &EncodingKey::from_secret(secret))
        .map_err(|e| AppError::Internal(format!("sign admin token: {e}")))
}

pub fn verify(secret: &[u8], token: &str) -> Result<AdminClaims, AppError> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.leeway = 5;
    decode::<AdminClaims>(token, &DecodingKey::from_secret(secret), &validation)
        .map(|data| data.claims)
        .map_err(|_| AppError::Unauthorized("管理员登录已过期，请重新登录".into()))
}

pub fn normalize_username(raw: &str) -> Result<String, AppError> {
    let name = raw.trim().to_lowercase();
    let ok = (3..=32).contains(&name.len())
        && name.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'));
    if ok {
        Ok(name)
    } else {
        Err(AppError::BadRequest("管理员用户名需为 3-32 位字母、数字、下划线、点或连字符".into()))
    }
}

/// 没有任何管理员时，用环境变量创建初始管理员；已有管理员则忽略环境变量，不覆盖密码。
pub async fn seed(pool: &PgPool, config: &AppConfig) -> Result<(), AppError> {
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM admins").fetch_one(pool).await?;
    if count > 0 {
        return Ok(());
    }
    let (Some(user), Some(pass)) = (&config.admin_user, &config.admin_password) else {
        tracing::warn!("尚无管理员账号：设置 AI_ADMIN_USER 与 AI_ADMIN_PASSWORD 后重启即可创建初始管理员");
        return Ok(());
    };
    let username = normalize_username(user)?;
    password::validate(pass)?;
    let hash = password::hash(pass.clone()).await?;
    sqlx::query("INSERT INTO admins (id, username, password_hash) VALUES ($1, $2, $3)")
        .bind(Uuid::new_v4())
        .bind(&username)
        .bind(hash)
        .execute(pool)
        .await?;
    tracing::info!("已创建初始管理员 {username}，登录后请在后台修改密码并从环境变量中移除 AI_ADMIN_PASSWORD");
    Ok(())
}

pub struct AdminIdentity {
    pub id: Uuid,
    pub username: String,
}

pub async fn authenticate(state: &AppState, token: &str) -> Result<AdminIdentity, AppError> {
    let claims = verify(&state.keys.admin_jwt_secret, token)?;
    let expired = || AppError::Unauthorized("管理员登录已过期，请重新登录".into());
    let id = Uuid::parse_str(&claims.sub).map_err(|_| expired())?;
    let row: Option<(String, i32)> = sqlx::query_as("SELECT username, token_version FROM admins WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.pool)
        .await?;
    match row {
        Some((username, version)) if version == claims.tv => Ok(AdminIdentity { id, username }),
        _ => Err(expired()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_roundtrip_and_wrong_secret_rejected() {
        let id = Uuid::new_v4();
        let token = issue(b"secret-a", id, 3, 60).unwrap();
        let claims = verify(b"secret-a", &token).unwrap();
        assert_eq!((claims.sub, claims.tv), (id.to_string(), 3));
        assert!(verify(b"secret-b", &token).is_err());
        assert!(verify(b"secret-a", &issue(b"secret-a", id, 0, -60).unwrap()).is_err());
    }

    #[test]
    fn usernames_are_normalized_and_validated() {
        assert_eq!(normalize_username(" Root.Admin ").unwrap(), "root.admin");
        for bad in ["ab", "has space", "名字名字", &"x".repeat(33)] {
            assert!(normalize_username(bad).is_err(), "{bad}");
        }
    }
}
