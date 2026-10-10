use crate::{
    crypto,
    error::AppError,
    identifier::Identifier,
    state::AppState,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Purpose {
    Login,
    ResetPassword,
    Bind,
}

impl Purpose {
    pub fn as_str(self) -> &'static str {
        match self {
            Purpose::Login => "login",
            Purpose::ResetPassword => "reset_password",
            Purpose::Bind => "bind",
        }
    }
}

const INVALID_CODE: &str = "验证码错误或已过期";

fn too_many(message: &str, retry_after: i64) -> AppError {
    AppError::TooManyRequests { message: message.to_string(), retry_after }
}

pub async fn issue(state: &AppState, ident: &Identifier, purpose: Purpose, ip: &str) -> Result<(), AppError> {
    let cfg = &state.config;
    let kind = ident.kind.as_str();
    let id_hash = state.keys.sub.identifier_hash(kind, &ident.value);

    let last: Option<(i64,)> = sqlx::query_as(
        "SELECT GREATEST(1, CEIL(EXTRACT(EPOCH FROM (created_at + make_interval(secs => $3) - now()))))::BIGINT
         FROM verification_codes
         WHERE kind = $1 AND identifier_hash = $2 AND created_at > now() - make_interval(secs => $3)
         ORDER BY created_at DESC LIMIT 1",
    )
    .bind(kind)
    .bind(&id_hash)
    .bind(cfg.resend_cooldown_secs as f64)
    .fetch_optional(&state.pool)
    .await?;
    if let Some((wait,)) = last {
        return Err(too_many("发送过于频繁，请稍后再试", wait));
    }

    let (by_ident,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM verification_codes WHERE kind = $1 AND identifier_hash = $2 AND created_at > now() - interval '1 day'",
    )
    .bind(kind)
    .bind(&id_hash)
    .fetch_one(&state.pool)
    .await?;
    if by_ident >= cfg.daily_per_identifier {
        return Err(too_many("今日验证码发送次数已达上限", 3600));
    }

    let (by_ip,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM verification_codes WHERE ip = $1 AND created_at > now() - interval '1 day'",
    )
    .bind(ip)
    .fetch_one(&state.pool)
    .await?;
    if by_ip >= cfg.daily_per_ip {
        return Err(too_many("当前网络今日验证码发送次数已达上限", 3600));
    }

    let code = crypto::random_code();
    let code_hash = state.keys.sub.code_hash(kind, &ident.value, purpose.as_str(), &code);
    let (row_id,): (i64,) = sqlx::query_as(
        "INSERT INTO verification_codes (kind, identifier_hash, purpose, code_hash, ip, expires_at)
         VALUES ($1, $2, $3, $4, $5, now() + make_interval(secs => $6)) RETURNING id",
    )
    .bind(kind)
    .bind(&id_hash)
    .bind(purpose.as_str())
    .bind(&code_hash)
    .bind(ip)
    .bind(cfg.code_ttl_secs as f64)
    .fetch_one(&state.pool)
    .await?;

    if let Err(err) = state.sender.send_code(ident, &code, cfg.code_ttl_secs / 60).await {
        sqlx::query("DELETE FROM verification_codes WHERE id = $1").bind(row_id).execute(&state.pool).await?;
        return Err(err);
    }
    Ok(())
}

/// 校验并消耗验证码：只看该标识符 + 用途下最新的一条未消耗、未过期记录。
pub async fn verify(state: &AppState, ident: &Identifier, purpose: Purpose, code: &str) -> Result<(), AppError> {
    let kind = ident.kind.as_str();
    let id_hash = state.keys.sub.identifier_hash(kind, &ident.value);
    let row: Option<(i64, Vec<u8>, i32)> = sqlx::query_as(
        "SELECT id, code_hash, attempts FROM verification_codes
         WHERE kind = $1 AND identifier_hash = $2 AND purpose = $3 AND consumed_at IS NULL AND expires_at > now()
         ORDER BY created_at DESC LIMIT 1",
    )
    .bind(kind)
    .bind(&id_hash)
    .bind(purpose.as_str())
    .fetch_optional(&state.pool)
    .await?;
    let Some((id, stored_hash, attempts)) = row else {
        return Err(AppError::BadRequest(INVALID_CODE.into()));
    };
    if attempts >= state.config.max_code_attempts {
        return Err(AppError::BadRequest(INVALID_CODE.into()));
    }

    if !state.keys.sub.code_matches(kind, &ident.value, purpose.as_str(), code.trim(), &stored_hash) {
        sqlx::query("UPDATE verification_codes SET attempts = attempts + 1 WHERE id = $1")
            .bind(id)
            .execute(&state.pool)
            .await?;
        return Err(AppError::BadRequest(INVALID_CODE.into()));
    }

    let consumed = sqlx::query("UPDATE verification_codes SET consumed_at = now() WHERE id = $1 AND consumed_at IS NULL")
        .bind(id)
        .execute(&state.pool)
        .await?;
    if consumed.rows_affected() == 0 {
        return Err(AppError::BadRequest(INVALID_CODE.into()));
    }
    Ok(())
}
