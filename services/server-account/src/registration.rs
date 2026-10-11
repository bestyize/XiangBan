use crate::{
    error::AppError,
    identifier::{Identifier, Kind},
};
use axum::http::StatusCode;
use chrono::Utc;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

/// 注册凭证的有效期：验证码通过后，用户在这段时间内设置用户名和密码。
pub const TTL_SECS: i64 = 15 * 60;
const PURPOSE: &str = "register";

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    purpose: String,
    kind: String,
    value: String,
    exp: i64,
    iat: i64,
}

/// 凭证只证明「这个邮箱 / 手机号刚刚通过了验证码」。真正建号发生在 `/api/auth/register`。
pub fn issue(secret: &[u8], ident: &Identifier) -> Result<String, AppError> {
    let now = Utc::now().timestamp();
    let claims = Claims {
        purpose: PURPOSE.into(),
        kind: ident.kind.as_str().into(),
        value: ident.value.clone(),
        exp: now + TTL_SECS,
        iat: now,
    };
    encode(&Header::new(Algorithm::HS256), &claims, &EncodingKey::from_secret(secret))
        .map_err(|e| AppError::Internal(format!("sign registration token: {e}")))
}

pub fn verify(secret: &[u8], token: &str) -> Result<Identifier, AppError> {
    let expired = || AppError::Coded {
        status: StatusCode::UNAUTHORIZED,
        code: "registration_expired",
        message: "验证已过期，请重新获取验证码".into(),
    };
    let mut validation = Validation::new(Algorithm::HS256);
    validation.leeway = 5;
    let claims = decode::<Claims>(token, &DecodingKey::from_secret(secret), &validation)
        .map(|data| data.claims)
        .map_err(|_| expired())?;
    let kind = Kind::from_db(&claims.kind).filter(|_| claims.purpose == PURPOSE).ok_or_else(expired)?;
    Ok(Identifier { kind, value: claims.value })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn email() -> Identifier {
        Identifier { kind: Kind::Email, value: "a@example.com".into() }
    }

    #[test]
    fn roundtrip_and_wrong_secret() {
        let token = issue(b"secret-one-secret-one-secret-one", &email()).unwrap();
        assert_eq!(verify(b"secret-one-secret-one-secret-one", &token).unwrap(), email());
        assert!(verify(b"secret-two-secret-two-secret-two", &token).is_err());
        assert!(verify(b"secret-one-secret-one-secret-one", "not.a.token").is_err());
    }

    #[test]
    fn other_token_kinds_are_not_registration_tokens() {
        // 同一密钥签出的、purpose 不对的令牌（例如将来别处复用了密钥）也不能当注册凭证。
        let secret = b"secret-one-secret-one-secret-one";
        let now = Utc::now().timestamp();
        let claims = Claims { purpose: "other".into(), kind: "email".into(), value: "a@example.com".into(), exp: now + 60, iat: now };
        let token = encode(&Header::new(Algorithm::HS256), &claims, &EncodingKey::from_secret(secret)).unwrap();
        assert!(verify(secret, &token).is_err());
    }

    #[test]
    fn expired_token_is_rejected_with_the_stable_code() {
        let secret = b"secret-one-secret-one-secret-one";
        let now = Utc::now().timestamp();
        let claims = Claims { purpose: PURPOSE.into(), kind: "email".into(), value: "a@example.com".into(), exp: now - 60, iat: now - 120 };
        let token = encode(&Header::new(Algorithm::HS256), &claims, &EncodingKey::from_secret(secret)).unwrap();
        match verify(secret, &token) {
            Err(AppError::Coded { code, .. }) => assert_eq!(code, "registration_expired"),
            other => panic!("unexpected: {other:?}"),
        }
    }
}
