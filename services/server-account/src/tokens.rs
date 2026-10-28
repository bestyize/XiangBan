use crate::{crypto, error::AppError, keys::Keys};
use chrono::Utc;
use jsonwebtoken::{decode, encode, Algorithm, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessClaims {
    pub iss: String,
    pub sub: String,
    pub aud: String,
    pub exp: i64,
    pub iat: i64,
    pub jti: String,
}

pub fn issue_access(keys: &Keys, issuer: &str, uid: Uuid, client_id: &str, ttl_secs: i64) -> Result<String, AppError> {
    let now = Utc::now().timestamp();
    let claims = AccessClaims {
        iss: issuer.to_string(),
        sub: uid.to_string(),
        aud: client_id.to_string(),
        exp: now + ttl_secs,
        iat: now,
        jti: crypto::hex(&crypto::random_bytes::<16>()),
    };
    let mut header = Header::new(Algorithm::EdDSA);
    header.kid = Some(keys.kid.clone());
    encode(&header, &claims, &keys.encoding).map_err(|e| AppError::Internal(format!("sign access token: {e}")))
}

pub fn verify_access(keys: &Keys, issuer: &str, token: &str) -> Result<AccessClaims, AppError> {
    let mut validation = Validation::new(Algorithm::EdDSA);
    validation.set_issuer(&[issuer]);
    validation.validate_aud = false;
    validation.leeway = 5;
    decode::<AccessClaims>(token, &keys.decoding, &validation)
        .map(|data| data.claims)
        .map_err(|_| AppError::Unauthorized("登录已过期，请重新登录".into()))
}

pub fn jwks(keys: &Keys) -> Value {
    json!({
        "keys": [{
            "kty": "OKP",
            "crv": "Ed25519",
            "alg": "EdDSA",
            "use": "sig",
            "kid": keys.kid,
            "x": keys.public_x,
        }]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn issued_token_verifies_and_rejects_foreign_issuer_or_key() {
        let keys = Keys::ephemeral();
        let uid = Uuid::new_v4();
        let token = issue_access(&keys, "iss-a", uid, "client-1", 60).unwrap();
        let claims = verify_access(&keys, "iss-a", &token).unwrap();
        assert_eq!(claims.sub, uid.to_string());
        assert_eq!(claims.aud, "client-1");
        assert!(verify_access(&keys, "iss-b", &token).is_err());
        assert!(verify_access(&Keys::ephemeral(), "iss-a", &token).is_err());
    }

    #[test]
    fn expired_token_is_rejected() {
        let keys = Keys::ephemeral();
        let token = issue_access(&keys, "iss", Uuid::new_v4(), "c", -60).unwrap();
        assert!(verify_access(&keys, "iss", &token).is_err());
    }

    #[test]
    fn issued_token_for_xiaoban_android_has_matching_audience() {
        let keys = Keys::ephemeral();
        let uid = Uuid::new_v4();
        let token = issue_access(&keys, "https://account.xiangban.xyz", uid, "xiaoban-android", 1800).unwrap();
        let claims = verify_access(&keys, "https://account.xiangban.xyz", &token).unwrap();
        assert_eq!(claims.aud, "xiaoban-android");
        assert_eq!(claims.sub, uid.to_string());
    }
}
