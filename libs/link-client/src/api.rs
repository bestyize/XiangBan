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
    pub public_key: String,
}

#[derive(Debug, Deserialize)]
pub struct PairCode {
    pub code: String,
    pub expires_at: String,
}

#[derive(Debug, Deserialize)]
pub struct Confirmed {
    pub pairing_id: String,
    pub phone: PeerInfo,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PendingRequest {
    pub pairing_id: String,
    pub peer: PeerInfo,
    pub proof: String,
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

    pub async fn register_device(&self, kind: &str, name: &str, public_key: &str) -> Result<String, Error> {
        let v: Value = self.call(reqwest::Method::POST, "/devices", Some(json!({"kind": kind, "name": name, "public_key": public_key}))).await?;
        v["device_id"].as_str().map(str::to_string).ok_or_else(|| Error::Network("响应缺少 device_id".into()))
    }

    pub async fn create_pair_code(&self, device_id: &str) -> Result<PairCode, Error> {
        self.call(reqwest::Method::POST, "/pair-codes", Some(json!({"device_id": device_id}))).await
    }

    pub async fn confirm(&self, pairing_id: &str) -> Result<Confirmed, Error> {
        self.call(reqwest::Method::POST, &format!("/pairings/{pairing_id}/confirm"), None).await
    }

    pub async fn reject(&self, pairing_id: &str) -> Result<(), Error> {
        self.call::<Value>(reqwest::Method::POST, &format!("/pairings/{pairing_id}/reject"), None).await.map(|_| ())
    }

    pub async fn remove_pairing(&self, pairing_id: &str) -> Result<(), Error> {
        self.call::<Value>(reqwest::Method::DELETE, &format!("/pairings/{pairing_id}"), None).await.map(|_| ())
    }

    pub async fn pending(&self, device_id: &str) -> Result<Vec<PendingRequest>, Error> {
        let v: Value = self.call(reqwest::Method::GET, &format!("/pairings?device_id={device_id}&status=pending"), None).await?;
        Ok(v["items"]
            .as_array()
            .map(|items| items.iter().filter_map(|i| Some(PendingRequest { pairing_id: i["pairing_id"].as_str()?.into(), peer: serde_json::from_value(i["peer"].clone()).ok()?, proof: i["proof"].as_str()?.into() })).collect())
            .unwrap_or_default())
    }
}
