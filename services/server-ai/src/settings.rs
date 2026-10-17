//! 在后台填写、而不是写进 `.env` 的运行时设置。值用主密钥派生的密钥加密后存在 `system_settings` 表，
//! 同一个令牌换了用途（`aad` 不同）就解不开，数据库被复制走也看不到明文。

use crate::{error::AppError, keys::Keys};
use chrono::{DateTime, Utc};
use sqlx::PgPool;

const NEWAPI_TOKEN_KEY: &str = "newapi_admin_token";

fn aad() -> String {
    format!("setting:{NEWAPI_TOKEN_KEY}")
}

pub struct StoredToken {
    pub token: String,
    pub updated_at: DateTime<Utc>,
}

pub async fn load_newapi_token(pool: &PgPool, keys: &Keys) -> Result<Option<StoredToken>, AppError> {
    let row: Option<(Vec<u8>, DateTime<Utc>)> = sqlx::query_as("SELECT value_enc, updated_at FROM system_settings WHERE key = $1")
        .bind(NEWAPI_TOKEN_KEY)
        .fetch_optional(pool)
        .await?;
    row.map(|(encrypted, updated_at)| Ok(StoredToken { token: keys.decrypt(&aad(), &encrypted)?, updated_at })).transpose()
}

pub async fn save_newapi_token(pool: &PgPool, keys: &Keys, token: &str) -> Result<(), AppError> {
    let encrypted = keys.encrypt(&aad(), token)?;
    sqlx::query(
        "INSERT INTO system_settings (key, value_enc) VALUES ($1, $2)
         ON CONFLICT (key) DO UPDATE SET value_enc = $2, updated_at = now()",
    )
    .bind(NEWAPI_TOKEN_KEY)
    .bind(encrypted)
    .execute(pool)
    .await?;
    Ok(())
}

/// 只做明显的格式检查，真正对不对由网关说了算。去掉首尾空白，因为从网页复制很容易带上。
pub fn validate_token_format(raw: &str) -> Result<String, AppError> {
    let token = raw.trim();
    if token.is_empty() {
        return Err(AppError::BadRequest("请填写令牌".into()));
    }
    if token.len() > 256 || !token.is_ascii() || token.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return Err(AppError::BadRequest("令牌里不能有空格、换行或中文，请重新复制".into()));
    }
    if token.len() < 16 {
        return Err(AppError::BadRequest("令牌太短，像是没有复制完整".into()));
    }
    Ok(token.to_string())
}

/// 只露出尾号，让管理员能认出现在用的是哪一个，又不足以还原令牌。
pub fn hint(token: &str) -> String {
    let tail: Vec<char> = token.chars().rev().take(4).collect();
    format!("…{}", tail.into_iter().rev().collect::<String>())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_checks_catch_copy_mistakes_and_trim_the_edges() {
        assert_eq!(validate_token_format("  nap_0123456789abcdef\n").unwrap(), "nap_0123456789abcdef");
        for bad in ["", "   ", "nap_short", "nap_0123 456789abcdef", "nap_0123456789abcdé", "令牌令牌令牌令牌令牌令牌令牌令牌", &"x".repeat(257)] {
            assert!(validate_token_format(bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn the_hint_never_reveals_more_than_the_last_four_characters() {
        assert_eq!(hint("nap_0123456789abcdef"), "…cdef");
        assert_eq!(hint("ab"), "…ab");
    }
}
