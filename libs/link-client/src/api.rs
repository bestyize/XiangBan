//! 中转服务的 REST 接口（见 services/server-link 的 README）。
use crate::{Error, TokenFn};
use serde::{de::DeserializeOwned, Deserialize};
use serde_json::{json, Value};

#[derive(Clone)]
pub struct Api {
    http: reqwest::Client,
    base: String,
    token: TokenFn,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PeerInfo {
    pub id: String,
    pub kind: String,
    pub name: String,
    #[serde(default)]
    pub online: bool,
}

impl Api {
    pub fn new(base: &str, token: TokenFn) -> Self {
        Self { http: reqwest::Client::builder().timeout(std::time::Duration::from_secs(15)).build().unwrap_or_default(), base: base.trim_end_matches('/').to_string(), token }
    }

    async fn call<T: DeserializeOwned>(&self, method: reqwest::Method, path: &str, body: Option<Value>) -> Result<T, Error> {
        let token = (self.token)().await.ok_or(Error::NoToken)?;
        let mut req = self.http.request(method, format!("{}{path}", self.base)).bearer_auth(token);
        if let Some(b) = body {
            req = req.json(&b);
        }
        let resp = req.send().await.map_err(|e| Error::Network(e.to_string()))?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| Error::Network(e.to_string()))?;
        if !status.is_success() {
            let v: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
            return Err(Error::Server { status: status.as_u16(), code: v["code"].as_str().unwrap_or("").into(), message: v["message"].as_str().unwrap_or("").into() });
        }
        // 204 没有正文。
        serde_json::from_str(if text.is_empty() { "null" } else { &text }).map_err(|e| Error::Network(format!("解析响应失败：{e}")))
    }

    pub async fn register_device(&self, kind: &str, name: &str, install_id: &str) -> Result<String, Error> {
        let v: Value = self.call(reqwest::Method::POST, "/devices", Some(json!({"kind": kind, "name": name, "install_id": install_id}))).await?;
        v["device_id"].as_str().map(str::to_string).ok_or_else(|| Error::Network("响应缺少 device_id".into()))
    }

    pub async fn remove_device(&self, device_id: &str) -> Result<(), Error> {
        self.call::<Value>(reqwest::Method::DELETE, &format!("/devices/{device_id}"), None).await.map(|_| ())
    }

    pub async fn peers(&self, device_id: &str) -> Result<Vec<PeerInfo>, Error> {
        let v: Value = self.call(reqwest::Method::GET, &format!("/peers?device_id={device_id}"), None).await?;
        Ok(serde_json::from_value(v["items"].clone()).unwrap_or_default())
    }
}
