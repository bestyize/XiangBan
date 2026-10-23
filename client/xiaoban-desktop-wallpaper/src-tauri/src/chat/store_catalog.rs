//! Backward compatibility for websites that do not yet expose the product catalog API.
use serde_json::Value;

/// `None` means this server returned HTTP 404 and the caller may use legacy discovery.
/// Transport, authorization, rate-limit and malformed-response errors remain errors.
pub(super) async fn fetch(origin: &str, path: &str, pairs: &[(&str, String)]) -> Result<Option<Value>, String> {
    let mut response = reqwest::Client::builder().timeout(std::time::Duration::from_secs(20))
        .build().map_err(|e| e.to_string())?
        .get(format!("{origin}{path}")).query(pairs).send().await
        .map_err(|_| "暂时无法连接小伴插件市场，请检查网站连接设置".to_string())?;
    let status = response.status().as_u16();
    // Old backends and reverse proxies may send plain text or HTML for missing routes.
    // Decide before reading/parsing their body; never use an error-message string as a signal.
    if status == 404 { return Ok(None); }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|_| "插件目录读取失败")? {
        if bytes.len() + chunk.len() > 2 * 1024 * 1024 { return Err("插件目录响应过大".into()); }
        bytes.extend_from_slice(&chunk);
    }
    let body: Value = serde_json::from_slice(&bytes).map_err(|_| "插件市场返回了无效数据")?;
    if !(200..300).contains(&status) {
        return Err(body["error"].as_str().unwrap_or("插件市场暂不可用，请稍后重试").to_string());
    }
    if !body["results"].is_array() { return Err("插件目录格式发生变化，请更新客户端".into()); }
    Ok(Some(body))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{io::{Read, Write}, net::TcpListener, thread};

    async fn reply(status: u16, body: &str) -> Result<Option<Value>, String> {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let origin = format!("http://{}", listener.local_addr().unwrap());
        let body = body.to_string();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream.set_read_timeout(Some(std::time::Duration::from_secs(5))).unwrap();
            let mut request = [0; 4096];
            stream.read(&mut request).unwrap();
            let response = format!("HTTP/1.1 {status} Test\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
            stream.write_all(response.as_bytes()).unwrap();
        });
        let result = fetch(&origin, "/api/v1/ai/plugins", &[]).await;
        server.join().unwrap();
        result
    }

    #[tokio::test]
    async fn missing_routes_allow_fallback_regardless_of_error_body_format() {
        for body in ["404 Not Found", "<html>Not found</html>", r#"{"error":"接口不存在"}"#] {
            assert!(reply(404, body).await.unwrap().is_none());
        }
    }

    #[tokio::test]
    async fn available_catalogs_including_empty_results_do_not_fall_back() {
        for body in [r#"{"results":[],"total":0,"hasMore":false}"#,
            r#"{"results":[{"slug":"demo-skill","namespace":"author_one"}],"total":1}"#] {
            let result = reply(200, body).await.unwrap().unwrap();
            assert_eq!(result, serde_json::from_str::<Value>(body).unwrap());
        }
    }

    #[tokio::test]
    async fn authorization_limits_and_failures_cannot_bypass_the_product_server() {
        for status in [400, 401, 403, 429, 500, 502, 503] {
            assert_eq!(reply(status, r#"{"error":"服务暂不可用"}"#).await.unwrap_err(), "服务暂不可用");
        }
        for body in ["<html>Unavailable</html>", r#"{"results":null}"#, r#"{"error":"404"}"#] {
            assert!(reply(200, body).await.is_err());
        }
    }

    #[tokio::test]
    async fn connection_failures_remain_errors() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let origin = format!("http://{}", listener.local_addr().unwrap());
        drop(listener);
        assert!(fetch(&origin, "/api/v1/ai/plugins", &[]).await.is_err());
    }
}
