use crate::error::AppError;
use argon2::{
    password_hash::{phc::PasswordHash, PasswordHasher, PasswordVerifier},
    Argon2,
};

pub const MIN_LEN: usize = 8;
pub const MAX_LEN: usize = 128;

pub fn validate(password: &str) -> Result<(), AppError> {
    let len = password.chars().count();
    if !(MIN_LEN..=MAX_LEN).contains(&len) {
        return Err(AppError::BadRequest(format!("密码长度需为 {MIN_LEN}-{MAX_LEN} 位")));
    }
    Ok(())
}

pub async fn hash(password: String) -> Result<String, AppError> {
    tokio::task::spawn_blocking(move || {
        Argon2::default()
            .hash_password(password.as_bytes())
            .map(|h: PasswordHash| h.to_string())
            .map_err(|e| AppError::Internal(format!("password hashing failed: {e}")))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
}

pub async fn verify(password: String, hash: String) -> bool {
    tokio::task::spawn_blocking(move || Argon2::default().verify_password(password.as_bytes(), hash.as_str()).is_ok())
        .await
        .unwrap_or(false)
}
