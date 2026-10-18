use crate::{crypto, error::AppError, state::AppState, tokens};
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: &'static str,
    pub expires_in: i64,
}

const SESSION_EXPIRED: &str = "登录已失效，请重新登录";

fn access_ttl_secs(client_id: &str, default_secs: i64) -> i64 {
    match client_id {
        "xiaoban-desktop-wallpaper" => 30 * 24 * 60 * 60,
        _ => default_secs,
    }
}

fn hash_token(token: &str) -> Vec<u8> {
    crypto::sha256(token.as_bytes())
}

pub async fn issue(
    state: &AppState,
    uid: Uuid,
    client_id: &str,
    family: Option<Uuid>,
    device_name: Option<&str>,
    ip: &str,
) -> Result<TokenPair, AppError> {
    let refresh_token = crypto::random_token();
    sqlx::query(
        "INSERT INTO refresh_tokens (id, uid, client_id, family_id, token_hash, device_name, ip, expires_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, now() + make_interval(days => $8))",
    )
    .bind(Uuid::new_v4())
    .bind(uid)
    .bind(client_id)
    .bind(family.unwrap_or_else(Uuid::new_v4))
    .bind(hash_token(&refresh_token))
    .bind(device_name.map(|d| d.chars().take(64).collect::<String>()))
    .bind(ip)
    .bind(state.config.refresh_ttl_days as i32)
    .execute(&state.pool)
    .await?;

    let ttl_secs = access_ttl_secs(client_id, state.config.access_ttl_secs);
    let access_token = tokens::issue_access(&state.keys, &state.config.issuer, uid, client_id, ttl_secs)?;
    Ok(TokenPair {
        access_token,
        refresh_token,
        token_type: "Bearer",
        expires_in: ttl_secs,
    })
}

/// 刷新令牌轮换：旧令牌一次性使用；已吊销的令牌再次出现视为泄露，整个令牌族作废。
pub async fn rotate(state: &AppState, refresh_token: &str, client_id: &str, ip: &str) -> Result<TokenPair, AppError> {
    let row: Option<(Uuid, Uuid, String, Uuid, Option<String>, bool, bool)> = sqlx::query_as(
        "SELECT id, uid, client_id, family_id, device_name, revoked_at IS NOT NULL, expires_at <= now()
         FROM refresh_tokens WHERE token_hash = $1",
    )
    .bind(hash_token(refresh_token))
    .fetch_optional(&state.pool)
    .await?;
    let Some((id, uid, token_client, family, device, revoked, expired)) = row else {
        return Err(AppError::Unauthorized(SESSION_EXPIRED.into()));
    };

    if revoked {
        sqlx::query("UPDATE refresh_tokens SET revoked_at = now() WHERE family_id = $1 AND revoked_at IS NULL")
            .bind(family)
            .execute(&state.pool)
            .await?;
        tracing::warn!("refresh token reuse detected for uid {uid}, family {family} revoked");
        return Err(AppError::Unauthorized(SESSION_EXPIRED.into()));
    }
    if expired || token_client != client_id {
        return Err(AppError::Unauthorized(SESSION_EXPIRED.into()));
    }
    let client_enabled: Option<(bool,)> = sqlx::query_as("SELECT enabled FROM clients WHERE client_id = $1")
        .bind(client_id)
        .fetch_optional(&state.pool)
        .await?;
    if !matches!(client_enabled, Some((true,))) {
        return Err(AppError::Unauthorized(SESSION_EXPIRED.into()));
    }

    let claimed = sqlx::query("UPDATE refresh_tokens SET revoked_at = now() WHERE id = $1 AND revoked_at IS NULL")
        .bind(id)
        .execute(&state.pool)
        .await?;
    if claimed.rows_affected() == 0 {
        return Err(AppError::Unauthorized(SESSION_EXPIRED.into()));
    }

    let (status,): (String,) = sqlx::query_as("SELECT status FROM users WHERE uid = $1")
        .bind(uid)
        .fetch_one(&state.pool)
        .await?;
    if status != "active" {
        return Err(AppError::Forbidden("账号已被停用".into()));
    }
    issue(state, uid, client_id, Some(family), device.as_deref(), ip).await
}

pub async fn revoke(state: &AppState, refresh_token: &str) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE refresh_tokens SET revoked_at = now()
         WHERE family_id = (SELECT family_id FROM refresh_tokens WHERE token_hash = $1) AND revoked_at IS NULL",
    )
    .bind(hash_token(refresh_token))
    .execute(&state.pool)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_access_tokens_last_thirty_days() {
        let ttl = access_ttl_secs("xiaoban-desktop-wallpaper", 1800);
        assert_eq!(ttl, 2_592_000);
        let keys = crate::keys::Keys::ephemeral();
        let token = tokens::issue_access(&keys, "account", Uuid::new_v4(), "xiaoban-desktop-wallpaper", ttl).unwrap();
        let claims = tokens::verify_access(&keys, "account", &token).unwrap();
        assert_eq!(claims.exp - claims.iat, ttl);
    }

    #[test]
    fn other_clients_keep_the_configured_lifetime() {
        for client in ["xiaban-web", "xiaoban-mobile-wallpaper", "server-forum", "other"] {
            for configured in [1800, 3600] {
                assert_eq!(access_ttl_secs(client, configured), configured);
            }
        }
    }
}

pub async fn revoke_all(state: &AppState, uid: Uuid) -> Result<(), AppError> {
    sqlx::query("UPDATE refresh_tokens SET revoked_at = now() WHERE uid = $1 AND revoked_at IS NULL")
        .bind(uid)
        .execute(&state.pool)
        .await?;
    Ok(())
}
