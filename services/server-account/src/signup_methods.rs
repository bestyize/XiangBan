use crate::{error::AppError, identifier::Kind, state::AppState};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};

const KEY: &str = "signup_methods";

/// 控制「新用户能用哪种方式注册 / 新绑定」。已有账号的登录、重置密码不受影响。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignupMethods {
    pub email: bool,
    pub phone: bool,
}

impl Default for SignupMethods {
    fn default() -> Self {
        Self { email: true, phone: false }
    }
}

impl SignupMethods {
    pub fn allows(&self, kind: Kind) -> bool {
        match kind {
            Kind::Email => self.email,
            Kind::Phone => self.phone,
        }
    }
}

pub async fn load(state: &AppState) -> Result<SignupMethods, AppError> {
    let row: Option<(Vec<u8>,)> = sqlx::query_as("SELECT value_enc FROM system_settings WHERE key = $1")
        .bind(KEY)
        .fetch_optional(&state.pool)
        .await?;
    let Some((enc,)) = row else { return Ok(SignupMethods::default()) };
    let json = state.keys.sub.decrypt(KEY, &enc)?;
    serde_json::from_str(&json).map_err(|e| AppError::Internal(format!("signup_methods 配置损坏: {e}")))
}

pub async fn save(state: &AppState, methods: SignupMethods) -> Result<(), AppError> {
    let json = serde_json::to_string(&methods).map_err(|e| AppError::Internal(e.to_string()))?;
    sqlx::query(
        "INSERT INTO system_settings (key, value_enc, updated_at) VALUES ($1, $2, now())
         ON CONFLICT (key) DO UPDATE SET value_enc = EXCLUDED.value_enc, updated_at = now()",
    )
    .bind(KEY)
    .bind(state.keys.sub.encrypt(KEY, &json)?)
    .execute(&state.pool)
    .await?;
    Ok(())
}

pub fn disabled(kind: Kind) -> AppError {
    let what = match kind {
        Kind::Email => "邮箱",
        Kind::Phone => "手机号",
    };
    AppError::Coded {
        status: StatusCode::FORBIDDEN,
        code: "signup_method_disabled",
        message: format!("暂未开放{what}注册，请使用其他方式"),
    }
}

/// 新注册 / 新绑定某类标识前调用。
pub async fn require(state: &AppState, kind: Kind) -> Result<(), AppError> {
    if load(state).await?.allows(kind) {
        Ok(())
    } else {
        Err(disabled(kind))
    }
}

/// 后台保存前的校验：至少开放一种；开放的方式必须有能发验证码的通道。
pub fn validate(state: &AppState, methods: SignupMethods) -> Result<(), AppError> {
    if !methods.email && !methods.phone {
        return Err(AppError::BadRequest("至少要开放一种注册方式".into()));
    }
    for (on, kind, name) in [(methods.email, Kind::Email, "邮箱"), (methods.phone, Kind::Phone, "手机号")] {
        if on && !state.sender.can_send(kind) {
            return Err(AppError::BadRequest(format!("当前发信通道无法发送{name}验证码，请先在「发信配置」里配置好再开放")));
        }
    }
    Ok(())
}
