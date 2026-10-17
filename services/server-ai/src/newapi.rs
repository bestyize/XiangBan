//! New API 管理接口与调用转发的客户端。管理接口返回 `{success, message, data}`，失败也可能是 HTTP 200，所以两层都要判断。

use crate::error::AppError;
use reqwest::{Method, Response};
use serde::Deserialize;
use serde_json::{json, Value};
use std::time::Duration;

#[derive(Debug, Clone, Deserialize)]
pub struct NewApiUser {
    pub id: i64,
    pub username: String,
    #[serde(default)]
    pub quota: i64,
    #[serde(default)]
    pub used_quota: i64,
    #[serde(default)]
    pub status: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelPricing {
    pub model_name: String,
    #[serde(default)]
    pub model_ratio: f64,
    #[serde(default)]
    pub completion_ratio: f64,
    #[serde(default)]
    pub model_price: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuotaMode {
    Add,
    Subtract,
}

impl QuotaMode {
    fn as_str(self) -> &'static str {
        match self {
            QuotaMode::Add => "add",
            QuotaMode::Subtract => "subtract",
        }
    }
}

pub struct NewApi {
    http: reqwest::Client,
    /// 转发调用不设整体超时（流式响应可能持续很久），只限制建立连接。
    stream_http: reqwest::Client,
    base: String,
    admin_token: Option<String>,
}

impl NewApi {
    pub fn new(base: String, admin_token: Option<String>) -> Self {
        // New API 与本服务同机（回环地址），环境里的 HTTP(S)_PROXY 不应该影响到它，否则请求会被代理截走。
        let http = reqwest::Client::builder().no_proxy().timeout(Duration::from_secs(15)).build().unwrap_or_default();
        let stream_http = reqwest::Client::builder()
            .no_proxy()
            .connect_timeout(Duration::from_secs(10))
            .read_timeout(Duration::from_secs(300))
            .build()
            .unwrap_or_default();
        Self { http, stream_http, base, admin_token }
    }

    pub fn is_configured(&self) -> bool {
        self.admin_token.is_some()
    }

    async fn admin_call(&self, method: Method, path: &str, body: Option<Value>) -> Result<Value, AppError> {
        let token = self
            .admin_token
            .as_deref()
            .ok_or_else(|| AppError::ServiceUnavailable("尚未配置 New API 管理令牌".into()))?;
        let mut request = self.http.request(method, format!("{}{}", self.base, path)).bearer_auth(token);
        if let Some(body) = body {
            request = request.json(&body);
        }
        let response = request.send().await.map_err(|e| {
            tracing::warn!("New API 请求失败 {path}: {e}");
            AppError::BadGateway("AI 网关暂时不可用".into())
        })?;
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        let value: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
        if !status.is_success() {
            tracing::warn!("New API 返回 {status} {path}: {}", truncate(&text));
            return Err(AppError::BadGateway("AI 网关返回异常".into()));
        }
        if value.get("success").and_then(Value::as_bool) == Some(false) {
            let message = value.get("message").and_then(Value::as_str).unwrap_or("").to_string();
            tracing::warn!("New API 拒绝 {path}: {message}");
            return Err(AppError::BadGateway(format!("AI 网关拒绝请求: {message}")));
        }
        Ok(value.get("data").cloned().unwrap_or(Value::Null))
    }

    /// 创建一个普通用户。用户名已存在也会失败，调用方随后用 `find_user` 兜底，因此失败可以忽略。
    pub async fn create_user(&self, username: &str, password: &str) -> Result<(), AppError> {
        let body = json!({ "username": username, "password": password, "display_name": username, "role": 1 });
        self.admin_call(Method::POST, "/api/user/", Some(body)).await.map(|_| ())
    }

    /// 搜索是子串匹配，所以必须再做一次精确比较，避免 `a12z` 命中 `a112z`。
    pub async fn find_user(&self, username: &str) -> Result<Option<NewApiUser>, AppError> {
        let path = format!("/api/user/search?keyword={}&p=1&page_size=20", urlencode(username));
        let data = self.admin_call(Method::GET, &path, None).await?;
        let items = data.get("items").and_then(Value::as_array).cloned().unwrap_or_default();
        Ok(items
            .into_iter()
            .filter_map(|v| serde_json::from_value::<NewApiUser>(v).ok())
            .find(|u| u.username == username))
    }

    pub async fn get_user(&self, id: i64) -> Result<NewApiUser, AppError> {
        let data = self.admin_call(Method::GET, &format!("/api/user/{id}"), None).await?;
        serde_json::from_value(data).map_err(|e| AppError::BadGateway(format!("无法解析 New API 用户: {e}")))
    }

    pub async fn adjust_quota(&self, id: i64, mode: QuotaMode, value: i64) -> Result<(), AppError> {
        if value <= 0 {
            return Ok(());
        }
        let body = json!({ "id": id, "action": "add_quota", "mode": mode.as_str(), "value": value });
        self.admin_call(Method::POST, "/api/user/manage", Some(body)).await.map(|_| ())
    }

    /// 同名令牌只会有一个：重复调用返回已有的 Key，所以中途崩溃后可以安全重试。
    pub async fn provision_token(&self, user_id: i64, name: &str) -> Result<String, AppError> {
        let data = self
            .admin_call(Method::POST, &format!("/api/user/{user_id}/token"), Some(json!({ "name": name })))
            .await?;
        data.get("key")
            .and_then(Value::as_str)
            .filter(|k| !k.is_empty())
            .map(str::to_string)
            .ok_or_else(|| AppError::BadGateway("New API 没有返回令牌".into()))
    }

    /// 公开的价格接口：列出已启用的模型及其倍率，不需要管理令牌。
    pub async fn list_models(&self) -> Result<Vec<ModelPricing>, AppError> {
        let response = self.http.get(format!("{}/api/pricing", self.base)).send().await.map_err(|e| {
            tracing::warn!("读取 New API 模型列表失败: {e}");
            AppError::BadGateway("AI 网关暂时不可用".into())
        })?;
        let value: Value = response.json().await.map_err(|_| AppError::BadGateway("AI 网关返回异常".into()))?;
        let items = value.get("data").and_then(Value::as_array).cloned().unwrap_or_default();
        Ok(items.into_iter().filter_map(|v| serde_json::from_value(v).ok()).collect())
    }

    /// 以某个绑定自己的 Key 转发一次对话补全，原样返回上游响应供调用方处理。
    pub async fn chat_completions(&self, key: &str, body: Vec<u8>) -> Result<Response, AppError> {
        self.stream_http
            .post(format!("{}/v1/chat/completions", self.base))
            .bearer_auth(format!("sk-{key}"))
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(body)
            .send()
            .await
            .map_err(|e| {
                tracing::warn!("转发到 New API 失败: {e}");
                AppError::BadGateway("AI 服务暂时不可用".into())
            })
    }
}

fn truncate(text: &str) -> String {
    text.chars().take(300).collect()
}

fn urlencode(raw: &str) -> String {
    raw.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
            other => format!("%{other:02X}"),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urlencode_keeps_unreserved_and_escapes_the_rest() {
        assert_eq!(urlencode("a12z"), "a12z");
        assert_eq!(urlencode("a b&c"), "a%20b%26c");
    }
}
