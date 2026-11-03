use agent_kernel::builtin::{SearchBackend, SearchHit, WebSearch};
use agent_kernel::Tool;
use serde::Deserialize;
use serde_json::json;
use std::{sync::Arc, time::Duration};

struct GatewaySearch(tauri::AppHandle);

#[derive(Deserialize)]
struct SearchReply {
    results: Vec<GatewayHit>,
}

#[derive(Deserialize)]
struct GatewayHit {
    title: String,
    url: String,
    snippet: String,
}

pub(super) fn tool(app: tauri::AppHandle) -> Arc<dyn Tool> {
    Arc::new(WebSearch::new(GatewaySearch(app)))
}

impl SearchBackend for GatewaySearch {
    fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchHit>, String> {
        if query.chars().count() > 128 {
            return Err("搜索关键词最多 128 字".into());
        }
        tauri::async_runtime::block_on(async {
            tokio::time::timeout(Duration::from_secs(28), self.request(query, limit))
                .await.map_err(|_| "小伴 AI 搜索超时".to_string())?
        })
    }
}

impl GatewaySearch {
    async fn request(&self, query: &str, limit: usize) -> Result<Vec<SearchHit>, String> {
        let token = crate::account::fresh_access_token(&self.0).await
            .map_err(|_| "请登录小伴账号后使用网页搜索".to_string())?;
        let client = crate::network::async_builder(reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(8))
            .redirect(reqwest::redirect::Policy::none()))?
            .build().map_err(|_| "无法创建搜索连接".to_string())?;
        let mut response = client.post(format!("{}/v1/search", crate::ai_gateway::ai_base_url()))
            .bearer_auth(token)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(json!({"query": query, "count": limit, "need_content": false}).to_string())
            .send().await.map_err(|_| "无法连接小伴 AI 搜索".to_string())?;
        if !response.status().is_success() {
            return Err(match response.status().as_u16() {
                401 => "请重新登录小伴账号".into(),
                402 => "小伴 AI 额度不足".into(),
                429 => "搜索请求过于频繁".into(),
                503 => "小伴 AI 搜索暂不可用".into(),
                status => format!("小伴 AI 搜索失败（HTTP {status}）"),
            });
        }
        let mut body = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(|_| "读取搜索结果失败".to_string())? {
            if body.len().saturating_add(chunk.len()) > 1024 * 1024 {
                return Err("搜索结果过大".into());
            }
            body.extend_from_slice(&chunk);
        }
        let reply: SearchReply = serde_json::from_slice(&body)
            .map_err(|_| "搜索结果格式无效".to_string())?;
        Ok(reply.results.into_iter().take(limit).map(|hit| SearchHit {
            title: hit.title.chars().take(300).collect(),
            url: hit.url,
            snippet: hit.snippet.chars().take(1200).collect(),
        }).collect())
    }
}
