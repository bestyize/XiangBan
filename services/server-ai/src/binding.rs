//! 绑定：一个 (账号, 产品) 在 New API 里对应的用户与令牌。首次使用某产品的 AI 时懒创建，并发放该产品的试用额度。

use crate::{error::AppError, keys, products::Product, quota, state::AppState, sync};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{Duration, Utc};
use uuid::Uuid;

/// New API 里这类令牌统一用这个名字，重复申请会拿到同一个 Key。
const TOKEN_NAME: &str = "server-ai";
const KEY_AAD: &str = "newapi_key";

pub struct ActiveBinding {
    pub id: i64,
    pub key: String,
}

/// New API 的用户名最长 20 位，序号至多 18 位（头尾各占一位固定字符）。
const MAX_BINDING_ID: i64 = 999_999_999_999_999_999;

/// New API 的用户搜索是子串匹配，头尾各加固定字符，保证任何序号都不会是另一个序号的子串。
pub fn newapi_username(binding_id: i64) -> String {
    debug_assert!((1..=MAX_BINDING_ID).contains(&binding_id), "binding id out of range: {binding_id}");
    format!("a{binding_id}z")
}

pub async fn find_active(state: &AppState, uid: Uuid, product_id: &str) -> Result<Option<ActiveBinding>, AppError> {
    let row: Option<(i64, Vec<u8>)> = sqlx::query_as(
        "SELECT id, newapi_key_enc FROM bindings WHERE uid = $1 AND product_id = $2 AND status = 'active' AND newapi_key_enc IS NOT NULL",
    )
    .bind(uid)
    .bind(product_id)
    .fetch_optional(&state.pool)
    .await?;
    row.map(|(id, enc)| Ok(ActiveBinding { id, key: state.keys.decrypt(KEY_AAD, &enc)? })).transpose()
}

pub async fn ensure_active(state: &AppState, uid: Uuid, product: &Product) -> Result<ActiveBinding, AppError> {
    if let Some(active) = find_active(state, uid, &product.product_id).await? {
        return Ok(active);
    }
    let (binding_id,): (i64,) = sqlx::query_as(
        "INSERT INTO bindings (uid, product_id) VALUES ($1, $2)
         ON CONFLICT (uid, product_id) DO UPDATE SET uid = EXCLUDED.uid RETURNING id",
    )
    .bind(uid)
    .bind(&product.product_id)
    .fetch_one(&state.pool)
    .await?;

    provision(state, binding_id, product).await?;
    find_active(state, uid, &product.product_id)
        .await?
        .ok_or_else(|| AppError::Internal(format!("binding {binding_id} 开通后仍不可用")))
}

async fn provision(state: &AppState, binding_id: i64, product: &Product) -> Result<(), AppError> {
    let mut tx = state.pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock($1)").bind(binding_id).execute(&mut *tx).await?;

    let (status, existing_user): (String, Option<i64>) =
        sqlx::query_as("SELECT status, newapi_user_id FROM bindings WHERE id = $1").bind(binding_id).fetch_one(&mut *tx).await?;
    if status == "active" {
        return Ok(());
    }

    let username = newapi_username(binding_id);
    let user_id = match existing_user {
        Some(id) => id,
        None => {
            let password = URL_SAFE_NO_PAD.encode(keys::random_bytes::<24>());
            // 用户可能在上一次中途失败时已经建好了，所以建失败不致命，以随后的精确查找为准。
            if let Err(e) = state.newapi.create_user(&username, &password).await {
                tracing::info!("在 New API 创建用户 {username} 未成功，尝试查找已有用户: {e}");
            }
            state
                .newapi
                .find_user(&username)
                .await?
                .ok_or_else(|| AppError::BadGateway("无法在 AI 网关创建用户".into()))?
                .id
        }
    };
    let key = state.newapi.provision_token(user_id, TOKEN_NAME).await?;
    let encrypted = state.keys.encrypt(KEY_AAD, &key)?;

    sqlx::query(
        "UPDATE bindings SET newapi_user_id = $2, newapi_username = $3, newapi_key_enc = $4, status = 'active' WHERE id = $1",
    )
    .bind(binding_id)
    .bind(user_id)
    .bind(&username)
    .bind(encrypted)
    .execute(&mut *tx)
    .await?;

    if product.trial_quota > 0 {
        let expires_at = Utc::now() + Duration::days(i64::from(product.trial_days.max(1)));
        quota::grant(&mut tx, binding_id, "trial", product.trial_quota, Some(expires_at), "trial", "新用户试用").await?;
    }
    tx.commit().await?;

    if let Err(e) = sync::sync_binding(state, binding_id).await {
        tracing::warn!("binding {binding_id} 首次对账失败，稍后由后台任务重试: {e}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usernames_never_contain_one_another() {
        let names: Vec<String> = [1, 12, 112, 121, 1200, 21].iter().map(|id| newapi_username(*id)).collect();
        for (i, a) in names.iter().enumerate() {
            for (j, b) in names.iter().enumerate() {
                assert!(i == j || !b.contains(a.as_str()), "{b} contains {a}");
            }
        }
        assert_eq!(newapi_username(MAX_BINDING_ID).len(), 20, "the largest valid id still fits New API's username limit");
    }
}
