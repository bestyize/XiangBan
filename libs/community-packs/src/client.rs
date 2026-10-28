//! 向论坛取下载信息，并从主地址或镜像下载内容包。每个来源都要过同一道大小与 SHA-256 校验，
//! 任何一个来源被篡改或截断都不会被采用，客户端换下一个来源。

use crate::pack::size_limit;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{io::Read, time::Duration};
use url::Url;

#[derive(Debug, Clone, Deserialize)]
pub struct Mirror {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DownloadInfo {
    pub pack_id: i64,
    pub kind: String,
    pub version: i32,
    pub label: String,
    pub sha256: String,
    pub size: u64,
    pub url: String,
    #[serde(default)]
    pub mirrors: Vec<Mirror>,
    pub trust_level: i64,
    pub one_click: bool,
}

pub struct Community {
    origin: String,
    http: reqwest::blocking::Client,
}

fn is_loopback(url: &Url) -> bool {
    matches!(url.host_str(), Some("127.0.0.1" | "localhost" | "[::1]"))
}

/// 下载来源必须是 HTTPS（本机回环仅用于开发和测试）。
fn check_source(text: &str) -> Result<Url, String> {
    let url = Url::parse(text).map_err(|_| "下载地址无效".to_string())?;
    match url.scheme() {
        "https" => Ok(url),
        "http" if is_loopback(&url) => Ok(url),
        _ => Err("下载地址必须使用 HTTPS".into()),
    }
}

impl Community {
    pub fn new(origin: &str) -> Result<Self, String> {
        let origin = origin.trim_end_matches('/').to_string();
        check_source(&origin)?;
        let http = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(60))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| e.to_string())?;
        Ok(Self { origin, http })
    }

    pub fn origin(&self) -> &str {
        &self.origin
    }

    /// 取某个内容包某个版本的下载信息。`pinned` 是链接里带的哈希：与服务器当前版本不一致就拒绝，
    /// 避免用户在网页上看到的是一版、点开后装上的是另一版。
    pub fn download_info(&self, pack_id: i64, version: Option<i32>, pinned: Option<&str>) -> Result<DownloadInfo, String> {
        let mut url = format!("{}/api/packs/{pack_id}/download", self.origin);
        if let Some(v) = version {
            url.push_str(&format!("?version={v}"));
        }
        let response = self.http.get(&url).send().map_err(|_| "无法连接社区服务器".to_string())?;
        if !response.status().is_success() {
            let status = response.status();
            let message = response.json::<serde_json::Value>().ok().and_then(|v| v["message"].as_str().map(str::to_string));
            return Err(message.unwrap_or_else(|| format!("社区服务器返回 {status}")));
        }
        let info: DownloadInfo = response.json().map_err(|_| "社区服务器返回了无法识别的内容".to_string())?;
        if info.pack_id != pack_id || info.sha256.len() != 64 {
            return Err("社区服务器返回的下载信息不完整".into());
        }
        if let Some(pinned) = pinned {
            if !pinned.eq_ignore_ascii_case(&info.sha256) {
                return Err("这个链接指向的版本已经更新或与服务器记录不一致，请回到网页重新打开".into());
            }
        }
        let limit = size_limit(&info.kind).ok_or("不支持的内容类型")?;
        if info.size == 0 || info.size > limit {
            return Err("内容包大小超出该类型的上限".into());
        }
        Ok(info)
    }

    /// 社区页的只读浏览：只放行周刊和内容包列表，调用方给的路径不能借此访问别的接口。
    pub fn browse(&self, path: &str) -> Result<serde_json::Value, String> {
        let allowed = path == "/api/weekly"
            || path.starts_with("/api/weekly?")
            || path.strip_prefix("/api/weekly/").is_some_and(|w| w.len() <= 12 && w.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-'))
            || path == "/api/packs"
            || path.starts_with("/api/packs?")
            || path.strip_prefix("/api/packs/").is_some_and(|id| !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit()));
        if !allowed || path.contains("..") || path.contains('#') || path.len() > 200 {
            return Err("不支持的社区页请求".into());
        }
        let response = self.http.get(format!("{}{path}", self.origin)).send().map_err(|_| "无法连接社区服务器".to_string())?;
        if !response.status().is_success() {
            let status = response.status();
            let message = response.json::<serde_json::Value>().ok().and_then(|v| v["message"].as_str().map(str::to_string));
            return Err(message.unwrap_or_else(|| format!("社区服务器返回 {status}")));
        }
        response.json().map_err(|_| "社区服务器返回了无法识别的内容".to_string())
    }

    /// 应用成功后告诉论坛一声（只做聚合计数）。失败不影响已经完成的应用，所以不返回错误。
    pub fn report_applied(&self, pack_id: i64, version: i32) {
        let _ = self
            .http
            .post(format!("{}/api/packs/{pack_id}/applied", self.origin))
            .json(&serde_json::json!({ "version": version }))
            .timeout(Duration::from_secs(10))
            .send();
    }

    /// 匿名埋点（PRD §8.3）：只报白名单事件名和一个内容类型维度，带随机设备标识，不带账号、内容和文件名。
    /// 尽力而为：失败静默忽略，不能影响用户正在做的事。
    pub fn track(&self, device: &str, events: &[(&str, Option<&str>)]) {
        let events: Vec<serde_json::Value> = events.iter().map(|(name, dim)| serde_json::json!({ "name": name, "dim": dim })).collect();
        let _ = self
            .http
            .post(format!("{}/api/events", self.origin))
            .json(&serde_json::json!({ "device": device, "events": events }))
            .timeout(Duration::from_secs(5))
            .send();
    }

    /// 依次尝试主地址和官方镜像，返回第一份通过校验的字节。
    pub fn fetch(&self, info: &DownloadInfo) -> Result<Vec<u8>, String> {
        let mut failures = Vec::new();
        let sources = std::iter::once(("社区服务器".to_string(), info.url.clone()))
            .chain(info.mirrors.iter().map(|m| (m.name.clone(), m.url.clone())));
        for (name, url) in sources {
            match self.fetch_one(&url, info) {
                Ok(bytes) => return Ok(bytes),
                Err(reason) => failures.push(format!("{name}：{reason}")),
            }
        }
        Err(format!("下载失败（{}）", failures.join("；")))
    }

    fn fetch_one(&self, url: &str, info: &DownloadInfo) -> Result<Vec<u8>, String> {
        let url = check_source(url)?;
        let response = self.http.get(url).send().map_err(|_| "连接失败".to_string())?;
        if !response.status().is_success() {
            return Err(format!("返回 {}", response.status()));
        }
        // 最多读到声明大小 + 1 字节，多出来就是不对；不把无限长的响应读进内存。
        let mut bytes = Vec::with_capacity(info.size as usize);
        response.take(info.size + 1).read_to_end(&mut bytes).map_err(|_| "下载中断".to_string())?;
        if bytes.len() as u64 != info.size {
            return Err("文件大小与记录不一致".into());
        }
        let digest = Sha256::digest(&bytes);
        let actual: String = digest.iter().map(|b| format!("{b:02x}")).collect();
        if actual != info.sha256.to_ascii_lowercase() {
            return Err("文件校验失败（SHA-256 不一致）".into());
        }
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::{
        io::{BufRead, BufReader, Write},
        net::TcpListener,
        sync::{Arc, Mutex},
        thread,
    };

    type Routes = Arc<Mutex<Vec<(String, u16, Vec<u8>)>>>;

    /// 极简 HTTP 服务：按路径返回预设的状态码与正文。
    fn serve(routes: Routes) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let origin = format!("http://{}", listener.local_addr().unwrap());
        thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { break };
                let mut line = String::new();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                reader.read_line(&mut line).unwrap();
                loop {
                    let mut header = String::new();
                    reader.read_line(&mut header).unwrap();
                    if header == "\r\n" || header.is_empty() {
                        break;
                    }
                }
                let path = line.split_whitespace().nth(1).unwrap_or("/").to_string();
                let found = routes.lock().unwrap().iter().find(|(p, _, _)| *p == path).cloned();
                let (status, body) = found.map(|(_, s, b)| (s, b)).unwrap_or((404, b"{}".to_vec()));
                let _ = write!(stream, "HTTP/1.1 {status} X\r\ncontent-length: {}\r\nconnection: close\r\n\r\n", body.len());
                let _ = stream.write_all(&body);
            }
        });
        origin
    }

    fn sha(bytes: &[u8]) -> String {
        Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect()
    }

    fn info_json(origin: &str, data: &[u8], mirrors: serde_json::Value) -> Vec<u8> {
        json!({
            "pack_id": 5, "kind": "effect", "version": 2, "label": "1.1.0", "sha256": sha(data), "size": data.len(),
            "url": format!("{origin}/files/main.zip"), "mirrors": mirrors, "trust_level": 0, "one_click": true
        })
        .to_string()
        .into_bytes()
    }

    #[test]
    fn download_info_pinning_and_limits() {
        let routes: Routes = Arc::default();
        let origin = serve(routes.clone());
        let data = b"payload".to_vec();
        routes.lock().unwrap().push(("/api/packs/5/download".into(), 200, info_json(&origin, &data, json!([]))));
        routes.lock().unwrap().push(("/api/packs/9/download".into(), 404, r#"{"message":"内容包不存在"}"#.as_bytes().to_vec()));
        let client = Community::new(&origin).unwrap();

        let info = client.download_info(5, None, None).unwrap();
        assert_eq!((info.version, info.kind.as_str()), (2, "effect"));
        assert!(client.download_info(5, None, Some(&sha(&data).to_uppercase())).is_ok(), "哈希大小写不敏感");
        assert!(client.download_info(5, None, Some(&"0".repeat(64))).unwrap_err().contains("版本已经更新"));
        assert_eq!(client.download_info(9, None, None).unwrap_err(), "内容包不存在");

        // 声明的大小超过该类型上限的信息被拒绝。
        let mut big: serde_json::Value = serde_json::from_slice(&info_json(&origin, &data, json!([]))).unwrap();
        big["size"] = json!(2 * 1024 * 1024);
        routes.lock().unwrap().push(("/api/packs/6/download".into(), 200, big.to_string().into_bytes()));
        let mut wrong_id = big.clone();
        wrong_id["pack_id"] = json!(6);
        routes.lock().unwrap()[2].2 = wrong_id.to_string().into_bytes();
        assert!(client.download_info(6, None, None).unwrap_err().contains("上限"));
    }

    #[test]
    fn fetch_verifies_every_source_and_falls_back() {
        let routes: Routes = Arc::default();
        let origin = serve(routes.clone());
        let data = b"the real pack bytes".to_vec();
        let mut tampered = data.clone();
        tampered[0] ^= 1;
        {
            let mut r = routes.lock().unwrap();
            r.push(("/files/main.zip".into(), 200, tampered.clone()));
            r.push(("/files/short.zip".into(), 200, data[..5].to_vec()));
            r.push(("/files/long.zip".into(), 200, [data.clone(), b"x".to_vec()].concat()));
            r.push(("/files/missing.zip".into(), 404, vec![]));
            r.push(("/files/mirror.zip".into(), 200, data.clone()));
        }
        let mirrors = json!([
            {"name": "短", "url": format!("{origin}/files/short.zip")},
            {"name": "长", "url": format!("{origin}/files/long.zip")},
            {"name": "缺", "url": format!("{origin}/files/missing.zip")},
            {"name": "魔搭", "url": format!("{origin}/files/mirror.zip")},
        ]);
        let info: DownloadInfo = serde_json::from_slice(&info_json(&origin, &data, mirrors)).unwrap();
        let client = Community::new(&origin).unwrap();
        // 主地址被篡改，前三个镜像分别是截断、超长、404，最后一个才对。
        assert_eq!(client.fetch(&info).unwrap(), data);

        let only_bad: DownloadInfo = serde_json::from_slice(&info_json(&origin, &data, json!([]))).unwrap();
        let err = client.fetch(&only_bad).unwrap_err();
        assert!(err.contains("SHA-256"), "{err}");
    }

    #[test]
    fn browse_only_allows_read_only_community_paths() {
        let routes: Routes = Arc::default();
        let origin = serve(routes.clone());
        routes.lock().unwrap().push(("/api/weekly".into(), 200, br#"{"items":[]}"#.to_vec()));
        routes.lock().unwrap().push(("/api/packs?kind=effect&sort=hot".into(), 200, br#"{"items":[1]}"#.to_vec()));
        let client = Community::new(&origin).unwrap();
        assert_eq!(client.browse("/api/weekly").unwrap()["items"].as_array().unwrap().len(), 0);
        assert!(client.browse("/api/packs?kind=effect&sort=hot").is_ok());
        for bad in [
            "/api/admin/packs", "/api/me/points", "/api/weekly/../admin", "/api/packs/abc", "/api/weekly/2026-W44/x",
            "/api/internal/pack-uploaded", "http://evil.test/api/weekly", "/api/weeklyx", "/api/packs#x",
        ] {
            assert!(client.browse(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn track_sends_only_the_named_events_and_device() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let origin = format!("http://{}", listener.local_addr().unwrap());
        let (tx, rx) = std::sync::mpsc::channel();
        thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            let mut length = 0usize;
            loop {
                let mut header = String::new();
                reader.read_line(&mut header).unwrap();
                if header == "\r\n" {
                    break;
                }
                if let Some(v) = header.to_ascii_lowercase().strip_prefix("content-length:") {
                    length = v.trim().parse().unwrap();
                }
            }
            let mut body = vec![0u8; length];
            std::io::Read::read_exact(&mut reader, &mut body).unwrap();
            let _ = write!(stream, "HTTP/1.1 204 No Content\r\nconnection: close\r\n\r\n");
            tx.send((line, String::from_utf8(body).unwrap())).unwrap();
        });
        Community::new(&origin).unwrap().track("dev-0123456789abcdef", &[("desktop_apply_success", Some("effect")), ("desktop_community_open", None)]);
        let (line, body) = rx.recv_timeout(Duration::from_secs(5)).unwrap();
        assert!(line.starts_with("POST /api/events "), "{line}");
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(v["device"], "dev-0123456789abcdef");
        assert_eq!(v["events"][0], json!({"name": "desktop_apply_success", "dim": "effect"}));
        assert_eq!(v["events"][1]["dim"], serde_json::Value::Null);
        assert_eq!(v.as_object().unwrap().len(), 2, "除设备标识和事件外不带任何东西");
        // 连不上也不报错。
        Community::new("http://127.0.0.1:9").unwrap().track("dev-0123456789abcdef", &[("search", None)]);
    }

    #[test]
    fn sources_must_be_https_outside_loopback() {
        assert!(Community::new("http://example.com").is_err());
        assert!(Community::new("https://chat.xiangban.xyz").is_ok());
        assert!(check_source("http://127.0.0.1:1").is_ok());
        assert!(check_source("http://evil.test/x.zip").is_err());
        assert!(check_source("ftp://x/y").is_err());
        assert!(check_source("not a url").is_err());
    }
}
