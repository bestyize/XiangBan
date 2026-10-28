//! Thin newline-delimited JSON-RPC proxy for this crate's loopback JSON endpoint.
#![forbid(unsafe_code)]
use reqwest::{Client, Url};
use serde_json::{json, Value};
use std::{collections::HashMap, io, time::Duration};
use tokio::{
    io::{AsyncBufRead, AsyncBufReadExt, AsyncWriteExt, BufReader},
    task::JoinSet,
};

fn error(id: Value, code: i32, message: &str) -> Value {
    json!({"jsonrpc":"2.0","id":id,"error":{"code":code,"message":message}})
}
fn endpoint(value: &str) -> Result<Url, &'static str> {
    let mut url = Url::parse(value).map_err(|_| "invalid voice URL")?;
    if url.scheme() != "http"
        || !matches!(url.host_str(), Some("127.0.0.1" | "localhost" | "[::1]"))
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || !matches!(url.path(), "" | "/" | "/mcp")
    {
        return Err("voice URL must be http://127.0.0.1, localhost or [::1], with optional /mcp");
    }
    // Avoid any DNS lookup, including proxy configuration and localhost overrides.
    if url.host_str() == Some("localhost") {
        url.set_host(Some("127.0.0.1"))
            .map_err(|_| "invalid loopback host")?;
    }
    url.set_path("/mcp");
    Ok(url)
}
async fn line(
    reader: &mut (impl AsyncBufRead + Unpin),
    partial: &mut Vec<u8>,
) -> io::Result<Option<Vec<u8>>> {
    loop {
        let bytes = reader.fill_buf().await?;
        if bytes.is_empty() {
            return if partial.is_empty() {
                Ok(None)
            } else {
                Ok(Some(std::mem::take(partial)))
            };
        }
        let newline = bytes.iter().position(|b| *b == b'\n');
        let count = newline.map_or(bytes.len(), |i| i + 1);
        if partial.len() + count > 1024 * 1024 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "stdio line exceeds 1 MiB",
            ));
        }
        partial.extend_from_slice(&bytes[..count]);
        reader.consume(count);
        if newline.is_some() {
            return Ok(Some(std::mem::take(partial)));
        }
    }
}
async fn forward(
    client: Client,
    url: Url,
    token: String,
    body: Value,
    version: String,
) -> Option<Value> {
    let notification = body.is_object()
        && body["jsonrpc"] == "2.0"
        && body["method"].is_string()
        && body.get("id").is_none();
    let id = (!notification).then(|| body.get("id").cloned().unwrap_or(Value::Null));
    let version = body["params"]["_meta"]["io.modelcontextprotocol/protocolVersion"]
        .as_str()
        .unwrap_or(&version);
    let mut request = client
        .post(url)
        .bearer_auth(token)
        .header("accept", "application/json, text/event-stream")
        .header("x-client-name", "xiaoban-voice-mcp-stdio")
        .header("mcp-protocol-version", version);
    if body["params"]["_meta"]["io.modelcontextprotocol/protocolVersion"].is_string() {
        if let Some(method) = body["method"].as_str() {
            request = request.header("mcp-method", method);
        }
        if let Some(name) = body["params"]["name"].as_str() {
            use base64::Engine;
            let name = if name.is_ascii()
                && name.trim() == name
                && name.bytes().all(|b| (32..=126).contains(&b))
                && !(name.starts_with("=?base64?") && name.ends_with("?="))
            {
                name.to_owned()
            } else {
                format!(
                    "=?base64?{}?=",
                    base64::engine::general_purpose::STANDARD.encode(name)
                )
            };
            request = request.header("mcp-name", name);
        }
    }
    let response = match request.json(&body).send().await {
        Ok(response) => response,
        Err(_) => return id.map(|id| error(id, -1, "local voice HTTP request failed")),
    };
    if response.status() == 202 {
        return None;
    }
    let status = response.status();
    let id = id?;
    if response.content_length().is_some_and(|n| n > 1024 * 1024) {
        return Some(error(id, -1, "voice response too large"));
    }
    let mut response = response;
    let mut bytes = Vec::new();
    loop {
        match response.chunk().await {
            Ok(Some(chunk)) if bytes.len() + chunk.len() <= 1024 * 1024 => {
                bytes.extend_from_slice(&chunk)
            }
            Ok(None) => break,
            _ => return Some(error(id, -1, "invalid or oversized voice response")),
        }
    }
    match serde_json::from_slice::<Value>(&bytes) {
        Ok(value) if value["jsonrpc"] == "2.0" => Some(value),
        _ => Some(error(
            id,
            -1,
            if status == 401 {
                "voice authentication failed"
            } else if status == 403 {
                "voice permission denied"
            } else {
                "voice endpoint did not return JSON-RPC"
            },
        )),
    }
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut url = std::env::var("XIAOBAN_VOICE_URL").ok();
    let mut token = std::env::var("XIAOBAN_VOICE_TOKEN").ok();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--url" => url = args.next(),
            "--token" => token = args.next(),
            _ => return Err("usage: xiaoban-voice-mcp-stdio [--url URL] [--token TOKEN]".into()),
        }
    }
    let url = endpoint(&url.ok_or("XIAOBAN_VOICE_URL or --url required")?)?;
    let token = token.ok_or("XIAOBAN_VOICE_TOKEN or --token required")?;
    let client = Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(185))
        .build()?;
    let mut reader = BufReader::new(tokio::io::stdin());
    let mut stdout = tokio::io::stdout();
    let mut partial = Vec::new();
    let mut pending = JoinSet::new();
    let mut handles = HashMap::<String, tokio::task::AbortHandle>::new();
    let mut version = "2025-03-26".to_owned();
    let mut eof = false;
    loop {
        if eof && pending.is_empty() {
            break;
        }
        let mut output = None;
        tokio::select! {
            message = line(&mut reader,&mut partial), if !eof => {
                let Some(bytes) = message? else { eof = true; continue; };
                if bytes.iter().all(u8::is_ascii_whitespace) { continue; }
                match serde_json::from_slice::<Value>(&bytes) {
                    Err(_) => output = Some(error(Value::Null,-32700,"invalid JSON")),
                    Ok(body) => {
                        let key = body.get("id").map(Value::to_string).unwrap_or_default();
                        if body["method"] == "notifications/cancelled" {
                            if let Some(handle) = handles.remove(&body["params"]["requestId"].to_string()) { handle.abort(); }
                            continue;
                        }
                        if pending.len() >= 32 || (!key.is_empty() && handles.contains_key(&key)) {
                            if let Some(id) = body.get("id") { output = Some(error(id.clone(),-1,"too many pending requests or duplicate request ID")); }
                        } else {
                            let (client,url,token,version) = (client.clone(),url.clone(),token.clone(),version.clone());
                            let task_key = key.clone();
                            let handle = pending.spawn(async move { (task_key,forward(client,url,token,body,version).await) });
                            if !key.is_empty() { handles.insert(key,handle); }
                        }
                    }
                }
            }
            completed = pending.join_next(), if !pending.is_empty() => {
                if let Some(Ok((key,value))) = completed {
                    handles.remove(&key);
                    if let Some(value) = value {
                        if let Some(selected) = value["result"]["protocolVersion"].as_str() { version = selected.to_owned(); }
                        output = Some(value);
                    }
                }
            }
        }
        if let Some(value) = output {
            stdout
                .write_all(serde_json::to_string(&value)?.as_bytes())
                .await?;
            stdout.write_all(b"\n").await?;
            stdout.flush().await?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn proxy_rejects_remote_tls_credentials_and_paths() {
        for url in [
            "http://evil.com",
            "https://localhost",
            "http://localhost.evil",
            "http://user:pass@localhost",
            "http://localhost/other",
            "http://localhost/?x=1",
            "http://localhost/#fragment",
        ] {
            assert!(endpoint(url).is_err(), "{url}");
        }
        for url in [
            "http://localhost:8080",
            "http://127.0.0.1:8080/mcp",
            "http://[::1]:8080",
        ] {
            assert!(endpoint(url).is_ok(), "{url}");
        }
    }
}
