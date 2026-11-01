//! 后台翻译服务设置：接口地址、模型、只写不读的密钥和翻译范围。保存前先用示例文本验证，通过才加密入库。
use super::admin::{audit, AdminIdentity, AuthAdmin};
use crate::{
    collectors::credentials,
    error::{AppError, AppResult},
    state::AppState,
    translation::{self, Config, Options, OptionsInput},
};
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::time::{Duration, Instant};

pub fn routes() -> Router<AppState> {
    Router::new().route("/translation", get(show).put(save)).route("/translation/test", post(test))
}

async fn show(State(s): State<AppState>, _: AuthAdmin) -> AppResult<Json<Value>> {
    let recent = translation::recent(&s.pool, 20).await?;
    Ok(Json(match translation::stored(&s.pool).await? {
        Some(c) => json!({"configured":true,"base_url":c.base_url,"model":c.model,"hint":c.hint,"options":c.options.to_json(),"updated_at":c.updated_at,"recent":recent}),
        None => json!({"configured":false,"base_url":"","model":"","hint":null,"options":Options::default().to_json(),"recent":recent}),
    }))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    base_url: String,
    model: String,
    key: Option<String>,
    options: Option<OptionsInput>,
}

async fn save(State(s): State<AppState>, AuthAdmin(admin): AuthAdmin, Json(input): Json<Input>) -> AppResult<Json<Value>> {
    apply(&s, &admin, input, false).await
}

/// `relaxed` 只给测试用，允许连本机模拟服务；接口路由始终传 false。
pub(super) async fn apply(s: &AppState, admin: &AdminIdentity, input: Input, relaxed: bool) -> AppResult<Json<Value>> {
    let base_url = translation::clean_base_url(&input.base_url, relaxed)?;
    let model = translation::clean_model(&input.model)?;
    let options = translation::stored(&s.pool).await?.map(|c| c.options).unwrap_or_default().merged(input.options.unwrap_or_default())?;
    let new_key = match input.key.as_deref().map(str::trim).filter(|k| !k.is_empty()) {
        Some(k) => Some(translation::clean_key(k)?),
        None => None,
    };
    let candidate = match &new_key {
        Some(key) => Config::new(base_url.clone(), model.clone(), key.clone(), options.clone()),
        None => translation::load(&s.pool)
            .await?
            .ok_or_else(|| AppError::BadRequest("请先填写 API key".into()))?
            .with(base_url.clone(), model.clone(), options.clone()),
    };
    let http = credentials::http_client()?;
    translation::complete(&http, &candidate, translation::SUMMARY_PROMPT, translation::SAMPLE, 512, Duration::from_secs(30))
        .await
        .map_err(|m| AppError::BadRequest(format!("翻译服务验证失败：{m}")))?;
    let mut tx = s.pool.begin().await?;
    let hint = translation::save(&mut tx, &base_url, &model, new_key.as_deref(), &options).await?;
    audit(&mut tx, admin, "translation.configure", "translation", "settings", format!("model={model}; key_updated={}", hint.is_some())).await?;
    tx.commit().await?;
    Ok(Json(json!({"ok":true,"hint":hint})))
}

async fn test(State(s): State<AppState>, _: AuthAdmin) -> AppResult<Json<Value>> {
    let cfg = translation::load(&s.pool).await?.ok_or_else(|| AppError::BadRequest("请先配置翻译服务".into()))?;
    let http = credentials::http_client()?;
    let begin = Instant::now();
    let result = translation::complete(&http, &cfg, translation::SUMMARY_PROMPT, translation::SAMPLE, 512, Duration::from_secs(30)).await;
    let millis = begin.elapsed().as_millis();
    translation::log(&s.pool, "", "test", result.is_ok(), result.as_ref().err().map_or("", |e| e.as_str()), millis).await;
    match result {
        Ok(translated) => Ok(Json(json!({"source":translation::SAMPLE,"translated":translated,"millis":millis}))),
        Err(message) => Err(AppError::BadRequest(format!("翻译测试失败：{message}"))),
    }
}
