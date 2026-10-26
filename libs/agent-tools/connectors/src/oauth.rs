//! Public-client MCP OAuth: discovery, loopback callback, PKCE and rotating refresh tokens.
use agent_kernel::mcp::HttpAuthorization;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use reqwest::{
    blocking::{Client, Response},
    redirect::Policy,
    Url,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    fs,
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, OnceLock, Weak,
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

pub const AUTH_REQUIRED: &str = "需要重新授权";
const MAX_JSON: u64 = 1024 * 1024;
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
pub fn random() -> Result<String, String> {
    let mut bytes = [0; 32];
    getrandom::fill(&mut bytes).map_err(|_| "无法生成授权随机数")?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}
pub fn valid_session(s: &str) -> bool {
    s.len() == 43
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}
pub fn token_path(root: &Path, id: &str, session: &str) -> Result<PathBuf, String> {
    if !super::valid_id(id) || !valid_session(session) {
        return Err("授权会话无效".into());
    }
    Ok(root.join(format!("oauth-{id}-{session}.json")))
}
pub fn private_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let temporary = path.with_extension(format!("{}.tmp", random()?));
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let result = (|| {
        let mut file = options.open(&temporary).map_err(|_| "无法保存授权凭据")?;
        file.write_all(bytes)
            .and_then(|_| file.sync_all())
            .map_err(|_| "无法保存授权凭据")?;
        drop(file);
        fs::rename(&temporary, path).map_err(|_| "无法保存授权凭据")
    })();
    if result.is_err() {
        let _ = fs::remove_file(temporary);
    }
    Ok(result?)
}
fn client() -> Result<Client, String> {
    agent_kernel::net::blocking_builder(Client::builder()
        .timeout(Duration::from_secs(15))
        .connect_timeout(Duration::from_secs(8))
        .redirect(Policy::none()))?
        .build()
        .map_err(|_| "无法创建授权客户端".into())
}
fn loopback(url: &Url) -> bool {
    matches!(
        url.host_str(),
        Some("127.0.0.1" | "localhost" | "[::1]" | "::1")
    )
}
fn endpoint(raw: &str, allow_local: bool) -> Result<Url, String> {
    if raw.len() > 8192 {
        return Err("授权服务地址过长".into());
    }
    let url = Url::parse(raw).map_err(|_| "授权服务地址无效")?;
    if (url.scheme() != "https" && !(allow_local && url.scheme() == "http" && loopback(&url)))
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
        || (!allow_local && loopback(&url))
    {
        return Err("授权服务必须使用 HTTPS，且不可指向本机地址".into());
    }
    if let Some(host) = url.host_str() {
        if let Ok(ip) = host.trim_matches(['[', ']']).parse::<std::net::IpAddr>() {
            let private = match ip {
                std::net::IpAddr::V4(ip) => {
                    ip.is_private() || ip.is_loopback() || ip.is_link_local() || ip.is_unspecified()
                }
                std::net::IpAddr::V6(ip) => {
                    ip.is_loopback()
                        || ip.is_unspecified()
                        || (ip.segments()[0] & 0xfe00) == 0xfc00
                        || (ip.segments()[0] & 0xffc0) == 0xfe80
                }
            };
            if private && !(allow_local && ip.is_loopback()) {
                return Err("授权服务地址不可使用内网 IP".into());
            }
        }
    }
    Ok(url)
}
fn read_json(response: Response) -> Result<Value, String> {
    if !response.status().is_success() {
        return Err(format!("授权服务返回 HTTP {}", response.status().as_u16()));
    }
    let mut bytes = Vec::new();
    response
        .take(MAX_JSON + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "授权响应读取失败")?;
    if bytes.len() as u64 > MAX_JSON {
        return Err("授权响应过大".into());
    }
    serde_json::from_slice(&bytes).map_err(|_| "授权响应格式无效".into())
}
fn get_json(client: &Client, url: Url) -> Result<Value, String> {
    read_json(client.get(url).send().map_err(|_| "无法读取授权服务信息")?)
}
fn form(client: &Client, url: &str, pairs: &[(&str, &str)]) -> Result<Value, String> {
    let encoded = Url::parse_with_params("https://form.invalid", pairs)
        .map_err(|_| "授权参数无效")?
        .query()
        .unwrap_or("")
        .to_string();
    let response = client
        .post(url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        // GitHub answers token requests with a urlencoded body unless JSON is requested.
        .header("Accept", "application/json")
        .body(encoded)
        .send()
        .map_err(|_| "授权请求失败或超时")?;
    if response.status().as_u16() == 400 || response.status().as_u16() == 401 {
        return Err(format!("{AUTH_REQUIRED}：登录已失效或授权请求被拒绝"));
    }
    read_json(response)
}
fn parameter(header: &str, key: &str) -> Option<String> {
    // Split commas only outside quoted strings; accept the Bearer challenge's parameters.
    let (scheme, header) = header.split_once(' ')?;
    if !scheme.eq_ignore_ascii_case("bearer") {
        return None;
    }
    let header = header.trim_start();
    let mut quoted = false;
    let mut escaped = false;
    let mut start = 0;
    for (i, c) in header
        .char_indices()
        .chain(std::iter::once((header.len(), ',')))
    {
        if escaped {
            escaped = false;
            continue;
        }
        if c == '\\' && quoted {
            escaped = true;
            continue;
        }
        if c == '"' {
            quoted = !quoted;
        }
        if c == ',' && !quoted {
            let part = header[start..i].trim();
            start = i + 1;
            if let Some((k, v)) = part.split_once('=') {
                if k.trim().eq_ignore_ascii_case(key) {
                    return Some(v.trim().trim_matches('"').to_string());
                }
            }
        }
    }
    None
}
fn well_known(issuer: &Url, name: &str) -> Url {
    let mut url = issuer.clone();
    url.set_path(&format!(
        "/.well-known/{name}{}",
        issuer.path().trim_end_matches('/')
    ));
    url.set_query(None);
    url
}

#[derive(Clone, Serialize, Deserialize)]
struct Credentials {
    server: String,
    resource: String,
    issuer: String,
    token_endpoint: String,
    client_id: String,
    access_token: String,
    refresh_token: Option<String>,
    expires_at: u64,
    scope: String,
}
fn token_values(
    value: &Value,
    old_refresh: Option<String>,
) -> Result<(String, Option<String>, u64), String> {
    // GitHub reports a rejected grant as HTTP 200 with an `error` field.
    if value["error"].is_string() {
        return Err(format!("{AUTH_REQUIRED}：登录已失效或授权请求被拒绝"));
    }
    if !value["token_type"]
        .as_str()
        .is_some_and(|s| s.eq_ignore_ascii_case("bearer"))
    {
        return Err("授权服务未返回 Bearer 凭据".into());
    }
    let access = value["access_token"]
        .as_str()
        .filter(|s| !s.is_empty() && s.len() <= 16384 && !s.chars().any(char::is_control))
        .ok_or("授权凭据无效")?
        .to_string();
    let refresh = match value.get("refresh_token") {
        None => old_refresh,
        Some(v) => Some(
            v.as_str()
                .filter(|s| !s.is_empty() && s.len() <= 32768 && !s.chars().any(char::is_control))
                .ok_or("刷新凭据无效")?
                .to_string(),
        ),
    };
    let duration = match value.get("expires_in") {
        None => 3600,
        Some(v) => v
            .as_u64()
            .filter(|v| *v > 0 && *v <= 366 * 86400)
            .ok_or("授权有效期无效")?,
    };
    Ok((access, refresh, now() + duration))
}
pub struct Session {
    path: PathBuf,
    client: Client,
    data: Mutex<Credentials>,
    retired: AtomicBool,
    encrypted: bool,
}
fn session_vault(path: &Path) -> Result<(super::vault::Vault, String), String> {
    let vault = super::vault::Vault::open(path.parent().ok_or("授权路径无效")?.join("vault"))?;
    Ok((vault, format!("oauth/{}", path.file_name().and_then(|s| s.to_str()).ok_or("授权路径无效")?)))
}
fn write_credentials(path: &Path, data: &Credentials, encrypted: bool) -> Result<(), String> {
    if encrypted {
        let (vault, key) = session_vault(path)?;
        vault.set(&key, "credentials", &serde_json::to_string(data).map_err(|_| "无法保存授权凭据")?)?;
        super::vault::atomic_write(path, br#"{"vault":true}"#)
    } else { private_write(path, &serde_json::to_vec(data).map_err(|_| "无法保存授权凭据")?) }
}
fn remove_credentials(path: &Path) {
    let encrypted = fs::read(path).ok().and_then(|b| serde_json::from_slice::<Value>(&b).ok()).is_some_and(|v| v["vault"] == true);
    if encrypted { if let Ok((vault, key)) = session_vault(path) { let _ = vault.remove_connector(&key); } }
    let _ = fs::remove_file(path);
}
pub(crate) fn retire_credentials(path: &Path) {
    let existing = cache().lock().unwrap_or_else(|e| e.into_inner()).get(path).and_then(Weak::upgrade);
    if let Some(session) = existing { session.retire(); } else { remove_credentials(path); }
}
static SESSIONS: OnceLock<Mutex<HashMap<PathBuf, Weak<Session>>>> = OnceLock::new();
fn cache() -> &'static Mutex<HashMap<PathBuf, Weak<Session>>> {
    SESSIONS.get_or_init(Default::default)
}
impl Session {
    pub fn load(path: PathBuf, server: &str) -> Result<Arc<Self>, String> {
        let mut sessions = cache().lock().unwrap_or_else(|e| e.into_inner());
        sessions.retain(|_, v| v.strong_count() > 0);
        if let Some(existing) = sessions.get(&path).and_then(Weak::upgrade) {
            if existing
                .data
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .server
                == server
            {
                return Ok(existing);
            }
        }
        let metadata = fs::symlink_metadata(&path).map_err(|_| AUTH_REQUIRED)?;
        if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > 128 * 1024 {
            return Err("授权凭据文件无效".into());
        }
        let stored: Value = serde_json::from_slice(&fs::read(&path).map_err(|_| AUTH_REQUIRED)?).map_err(|_| "授权凭据文件无效")?;
        let encrypted = stored["vault"] == true;
        let data: Credentials = if encrypted {
            let (vault, key) = session_vault(&path)?;
            serde_json::from_str(&vault.get(&key, "credentials")?.ok_or(AUTH_REQUIRED)?).map_err(|_| "授权凭据文件无效")?
        } else { serde_json::from_value(stored).map_err(|_| "授权凭据文件无效")? };
        let target = Url::parse(server).map_err(|_| "服务地址无效")?;
        let local = loopback(&target);
        endpoint(&data.token_endpoint, local)?;
        endpoint(&data.issuer, local)?;
        if data.server != server {
            return Err("授权凭据与当前服务不匹配".into());
        }
        let session = Arc::new(Self {
            path: path.clone(),
            client: client()?,
            data: Mutex::new(data),
            retired: AtomicBool::new(false),
            encrypted,
        });
        sessions.insert(path, Arc::downgrade(&session));
        Ok(session)
    }
    pub fn retire(&self) {
        self.retired.store(true, Ordering::SeqCst);
        let _guard = self.data.lock().unwrap_or_else(|e| e.into_inner());
        remove_credentials(&self.path);
    }
}
impl HttpAuthorization for Session {
    fn access_token(&self, refresh: bool) -> Result<String, String> {
        if self.retired.load(Ordering::SeqCst) {
            return Err(AUTH_REQUIRED.into());
        }
        let mut data = self.data.lock().unwrap_or_else(|e| e.into_inner());
        if refresh || data.expires_at <= now() + 30 {
            let token = data.refresh_token.as_deref().ok_or(AUTH_REQUIRED)?;
            let mut pairs = vec![
                ("grant_type", "refresh_token"),
                ("refresh_token", token),
                ("client_id", data.client_id.as_str()),
            ];
            // Device-flow services (GitHub) have no RFC 8707 resource binding.
            if !data.resource.is_empty() {
                pairs.push(("resource", data.resource.as_str()));
            }
            let value = form(&self.client, &data.token_endpoint, &pairs)?;
            let (access, refresh, expires) = token_values(&value, data.refresh_token.clone())?;
            let mut next = data.clone();
            next.access_token = access;
            next.refresh_token = refresh;
            next.expires_at = expires;
            if self.retired.load(Ordering::SeqCst) {
                return Err(AUTH_REQUIRED.into());
            }
            write_credentials(&self.path, &next, self.encrypted)?;
            *data = next;
        }
        Ok(data.access_token.clone())
    }
    fn cached_token(&self) -> Option<String> {
        self.data.try_lock().ok().map(|v| v.access_token.clone())
    }
}
/// RFC 8628 device authorization for services whose OAuth has no discovery or dynamic registration.
pub(crate) struct DeviceSpec {
    pub code_url: String,
    pub token_url: String,
    pub client_id: String,
    pub scope: String,
}
struct Device {
    code: String,
    interval: u64,
    wait: Duration,
}
/// The client ID is public (device flow needs no secret); GitHub OAuth App "小伴" with Device Flow enabled.
const GITHUB_DEVICE_CLIENT_ID: &str = "Ov23liVklr8MnYx5S5wI";
const GITHUB_DEVICE_SCOPE: &str = "repo read:org read:user";
/// Hosts known to need the device flow. A configured `clientId` replaces the built-in one.
pub(crate) fn device_spec(config: &Value, target: &Url) -> Option<DeviceSpec> {
    let custom = config.get("deviceAuth").and_then(Value::as_object);
    let is_github = target.scheme() == "https" && target.host_str() == Some("api.githubcopilot.com");
    let is_device_mode = config["authMode"] == "device";
    if !is_github && !is_device_mode && custom.is_none() {
        return None;
    }
    let text = |key: &str| config[key].as_str().filter(|s| !s.trim().is_empty());
    let (code_url, token_url, default_client, default_scope) = if let Some(m) = custom {
        (
            m.get("codeUrl").and_then(Value::as_str).unwrap_or(""),
            m.get("tokenUrl").and_then(Value::as_str).unwrap_or(""),
            m.get("clientId").and_then(Value::as_str).unwrap_or(GITHUB_DEVICE_CLIENT_ID),
            m.get("scope").and_then(Value::as_str).unwrap_or(GITHUB_DEVICE_SCOPE),
        )
    } else {
        (
            "https://github.com/login/device/code",
            "https://github.com/login/oauth/access_token",
            GITHUB_DEVICE_CLIENT_ID,
            GITHUB_DEVICE_SCOPE,
        )
    };
    if code_url.is_empty() || token_url.is_empty() {
        return None;
    }
    Some(DeviceSpec {
        code_url: code_url.into(),
        token_url: token_url.into(),
        client_id: text("clientId").unwrap_or(default_client).into(),
        scope: text("scope").unwrap_or(default_scope).into(),
    })
}
fn bounded(value: &Value, max: usize) -> Option<String> {
    value
        .as_str()
        .filter(|s| !s.is_empty() && s.len() <= max && !s.chars().any(char::is_control))
        .map(str::to_string)
}
pub struct Flow {
    pub url: String,
    pub session_id: String,
    /// Shown to the user to type into the verification page; only set for device flows.
    pub user_code: Option<String>,
    listener: Option<TcpListener>,
    device: Option<Device>,
    redirect: String,
    state: String,
    verifier: String,
    client: Client,
    credentials: Credentials,
    path: PathBuf,
    pub encrypt_credentials: bool,
}
impl Flow {
    pub(crate) fn begin_device(
        server: &str,
        spec: DeviceSpec,
        root: &Path,
        id: &str,
        local: bool,
    ) -> Result<Self, String> {
        let code_url = endpoint(&spec.code_url, local)?;
        let token_url = endpoint(&spec.token_url, local)?;
        let client = client()?;
        let grant = form(
            &client,
            code_url.as_str(),
            &[("client_id", spec.client_id.as_str()), ("scope", spec.scope.as_str())],
        )?;
        if grant["error"].is_string() {
            return Err("授权服务未开放设备授权，请联系服务商或改用密钥连接".into());
        }
        let code = bounded(&grant["device_code"], 1024).ok_or("设备授权响应无效")?;
        let user_code = bounded(&grant["user_code"], 64).ok_or("设备授权响应无效")?;
        let verification = endpoint(grant["verification_uri"].as_str().ok_or("设备授权响应无效")?, local)?;
        if verification.origin() != code_url.origin() {
            return Err("设备授权页面与授权服务不一致".into());
        }
        let session_id = random()?;
        let path = token_path(root, id, &session_id)?;
        Ok(Self {
            url: verification.to_string(),
            session_id,
            user_code: Some(user_code),
            listener: None,
            device: Some(Device {
                code,
                interval: grant["interval"].as_u64().unwrap_or(5).clamp(1, 30),
                wait: Duration::from_secs(grant["expires_in"].as_u64().unwrap_or(300).clamp(30, 300)),
            }),
            redirect: String::new(),
            state: String::new(),
            verifier: String::new(),
            client,
            credentials: Credentials {
                server: server.into(),
                resource: String::new(),
                issuer: code_url.origin().ascii_serialization(),
                token_endpoint: token_url.to_string(),
                client_id: spec.client_id,
                access_token: String::new(),
                refresh_token: None,
                expires_at: 0,
                scope: spec.scope,
            },
            path,
            encrypt_credentials: false,
        })
    }
    fn complete_device(mut self, cancel: &AtomicBool) -> Result<Arc<Session>, String> {
        let device = self.device.take().ok_or("设备授权会话无效")?;
        let start = Instant::now();
        let mut interval = device.interval;
        let mut next = Instant::now() + Duration::from_secs(interval);
        let value = loop {
            if cancel.load(Ordering::SeqCst) {
                return Err("授权已取消".into());
            }
            if start.elapsed() > device.wait {
                return Err("授权等待超时，请重新连接".into());
            }
            if Instant::now() < next {
                std::thread::sleep(Duration::from_millis(75));
                continue;
            }
            let value = form(
                &self.client,
                &self.credentials.token_endpoint,
                &[
                    ("client_id", self.credentials.client_id.as_str()),
                    ("device_code", device.code.as_str()),
                    ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ],
            )?;
            match value["error"].as_str() {
                None => break value,
                Some("authorization_pending") => {}
                Some("slow_down") => interval = (interval + 5).min(60),
                Some("access_denied") => return Err("用户未完成授权，请重新连接".into()),
                Some("expired_token") => return Err("授权等待超时，请重新连接".into()),
                Some(_) => return Err("授权失败，请重新连接".into()),
            }
            next = Instant::now() + Duration::from_secs(interval);
        };
        let (access, refresh, expires) = token_values(&value, None)?;
        let mut data = self.credentials;
        data.access_token = access;
        data.refresh_token = refresh;
        data.expires_at = expires;
        if cancel.load(Ordering::SeqCst) {
            return Err("授权已取消".into());
        }
        write_credentials(&self.path, &data, self.encrypt_credentials)?;
        Session::load(self.path, &data.server)
    }
    pub fn begin(config: &Value, root: &Path, id: &str) -> Result<Self, String> {
        let server = config["url"].as_str().ok_or("网页授权仅适用于远程 MCP")?;
        let target = Url::parse(server).map_err(|_| "服务地址无效")?;
        let local = loopback(&target);
        endpoint(server, local)?;
        if target.query().is_some() {
            return Err("网页授权的服务地址不可携带查询凭据，请使用不含密钥的 MCP 地址".into());
        }
        if let Some(spec) = device_spec(config, &target) {
            return Self::begin_device(server, spec, root, id, local);
        }
        let client = client()?;
        let legacy = config["transport"] == "sse" || config["type"] == "sse";
        let mut routing = reqwest::header::HeaderMap::new();
        // Tenant routing headers belong only to the resource probe, never to its authorization server.
        for (key, value) in super::string_map(config.get("headers"))? {
            if !matches!(
                key.to_ascii_lowercase().as_str(),
                "authorization"
                    | "host"
                    | "accept"
                    | "content-type"
                    | "content-length"
                    | "mcp-session-id"
                    | "mcp-protocol-version"
            ) {
                routing.insert(
                    reqwest::header::HeaderName::from_bytes(key.as_bytes()).map_err(|_| "请求头无效")?,
                    value.parse().map_err(|_| "请求头无效")?,
                );
            }
        }
        let initialize = json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":if legacy {"2024-11-05"} else {"2025-06-18"},"capabilities":{},"clientInfo":{"name":"xiaoban","version":env!("CARGO_PKG_VERSION")}}}).to_string();
        let probe = if legacy {
            let response = client.get(server).headers(routing.clone()).header("Accept", "text/event-stream")
                .send().map_err(|_| "无法检测服务授权方式")?;
            if response.status().is_success() {
                // GET may be public while the advertised POST endpoint requires OAuth.
                // Bound the endpoint probe and validate its origin before forwarding headers.
                let mut reader = BufReader::new(response.take(64 * 1024));
                let mut event = String::new();
                let mut data = String::new();
                let post = loop {
                    let mut line = String::new();
                    if reader.read_line(&mut line).map_err(|_| "无法读取 SSE 授权端点")? == 0 {
                        return Err("SSE 授权探测缺少 endpoint".into());
                    }
                    let line = line.trim_end_matches(['\r', '\n']);
                    if line.is_empty() {
                        if event == "endpoint" && !data.is_empty() {
                            let post = target.join(data.trim()).map_err(|_| "SSE endpoint 无效")?;
                            if post.origin() != target.origin() || !post.username().is_empty() || post.password().is_some() || post.fragment().is_some() {
                                return Err("SSE endpoint 必须同源".into());
                            }
                            break post;
                        }
                        event.clear(); data.clear();
                    } else if let Some(value) = line.strip_prefix("event:") {
                        event = value.strip_prefix(' ').unwrap_or(value).into();
                    } else if let Some(value) = line.strip_prefix("data:") {
                        data.push_str(value.strip_prefix(' ').unwrap_or(value)); data.push('\n');
                    }
                };
                // Keep the stream alive until the POST challenge has been read.
                client.post(post).headers(routing).header("Content-Type", "application/json")
                    .body(initialize).send().map_err(|_| "无法检测服务授权方式")?
            } else { response }
        } else {
            client.post(server).headers(routing).header("Accept", "application/json, text/event-stream")
                .header("Content-Type", "application/json").body(initialize).send().map_err(|_| "无法检测服务授权方式")?
        };
        let bearer_challenge = probe
            .headers()
            .get_all("www-authenticate")
            .iter()
            .filter_map(|v| v.to_str().ok())
            .find(|v| v.to_ascii_lowercase().starts_with("bearer "))
            .unwrap_or("")
            .to_string();
        let metadata = if let Some(raw) = parameter(&bearer_challenge, "resource_metadata") {
            get_json(&client, endpoint(&raw, local)?)?
        } else {
            get_json(&client, well_known(&target, "oauth-protected-resource"))
                .or_else(|_| {
                    let mut base = target.clone();
                    base.set_path("/");
                    get_json(&client, well_known(&base, "oauth-protected-resource"))
                })
                .map_err(|_| "该服务没有标准网页授权入口，请使用密钥连接或联系服务商开放小伴接入")?
        };
        let resource = endpoint(
            metadata["resource"].as_str().ok_or("授权资源信息缺失")?,
            local,
        )?;
        let base_path = resource.path().trim_end_matches('/');
        if resource.origin() != target.origin()
            || !(target.path() == base_path || target.path().starts_with(&format!("{base_path}/")))
        {
            return Err("授权资源与 MCP 服务不匹配".into());
        }
        let issuer = endpoint(
            metadata["authorization_servers"]
                .as_array()
                .and_then(|v| v.first())
                .and_then(Value::as_str)
                .ok_or("授权服务信息缺失")?,
            local,
        )?;
        let auth =
            get_json(&client, well_known(&issuer, "oauth-authorization-server")).or_else(|_| {
                let mut url = issuer.clone();
                url.set_path(&format!(
                    "{}/.well-known/openid-configuration",
                    issuer.path().trim_end_matches('/')
                ));
                get_json(&client, url)
            })?;
        if auth["issuer"]
            .as_str()
            .and_then(|s| Url::parse(s).ok())
            .is_none_or(|v| v != issuer)
        {
            return Err("授权服务身份不匹配".into());
        }
        if auth["code_challenge_methods_supported"]
            .as_array()
            .is_some_and(|v| !v.iter().any(|s| s == "S256"))
        {
            return Err("授权服务不支持 PKCE S256".into());
        }
        let authorization = endpoint(
            auth["authorization_endpoint"]
                .as_str()
                .ok_or("授权入口缺失")?,
            local,
        )?;
        if authorization.query_pairs().any(|(key, _)| {
            matches!(
                key.as_ref(),
                "client_id"
                    | "redirect_uri"
                    | "state"
                    | "response_type"
                    | "resource"
                    | "scope"
                    | "code_challenge"
                    | "code_challenge_method"
            )
        }) {
            return Err("授权入口包含冲突参数，请联系服务商修正".into());
        }
        let token = endpoint(
            auth["token_endpoint"].as_str().ok_or("令牌入口缺失")?,
            local,
        )?;
        let listener = TcpListener::bind("127.0.0.1:0").map_err(|_| "无法接收本机授权回调")?;
        listener
            .set_nonblocking(true)
            .map_err(|_| "无法接收本机授权回调")?;
        let redirect = format!(
            "http://127.0.0.1:{}/oauth/callback",
            listener.local_addr().map_err(|_| "本机回调不可用")?.port()
        );
        let client_id = if let Some(id) = config["clientId"].as_str().filter(|s| !s.is_empty()) {
            id.to_string()
        } else {
            let registration=endpoint(auth["registration_endpoint"].as_str().ok_or("该服务不支持自动注册，请填写为小伴注册的客户端 ID；WorkBuddy 专属身份不能直接复用")?,local)?;
            let value=read_json(client.post(registration).header("Content-Type","application/json").body(json!({"client_name":"小伴","redirect_uris":[redirect],"grant_types":["authorization_code","refresh_token"],"response_types":["code"],"token_endpoint_auth_method":"none"}).to_string()).send().map_err(|_|"客户端注册失败")?)?;
            if !value["redirect_uris"]
                .as_array()
                .is_some_and(|v| v.iter().any(|v| v == &redirect))
                || value["token_endpoint_auth_method"]
                    .as_str()
                    .is_some_and(|s| s != "none")
            {
                return Err("服务未接受小伴的公共客户端和回调地址".into());
            }
            value["client_id"]
                .as_str()
                .filter(|s| !s.is_empty() && s.len() <= 2048)
                .ok_or("客户端注册未返回有效 ID")?
                .to_string()
        };
        let state = random()?;
        let verifier = random()?;
        let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
        // Request only configured scopes or the resource server's explicit challenge.
        let scope = config["scope"]
            .as_str()
            .filter(|s| !s.trim().is_empty())
            .map(str::to_string)
            .or_else(|| parameter(&bearer_challenge, "scope"))
            .unwrap_or_default();
        if scope.len() > 2048 || scope.chars().any(char::is_control) {
            return Err("授权权限范围无效".into());
        }
        let mut url = authorization;
        url.query_pairs_mut().extend_pairs([
            ("response_type", "code"),
            ("client_id", client_id.as_str()),
            ("redirect_uri", redirect.as_str()),
            ("state", state.as_str()),
            ("code_challenge", challenge.as_str()),
            ("code_challenge_method", "S256"),
            ("resource", resource.as_str()),
        ]);
        if !scope.is_empty() {
            url.query_pairs_mut().append_pair("scope", &scope);
        }
        let session_id = random()?;
        let path = token_path(root, id, &session_id)?;
        Ok(Self {
            url: url.to_string(),
            session_id,
            user_code: None,
            listener: Some(listener),
            device: None,
            redirect,
            state,
            verifier,
            client,
            credentials: Credentials {
                server: server.into(),
                resource: resource.to_string(),
                issuer: issuer.to_string(),
                token_endpoint: token.to_string(),
                client_id,
                access_token: String::new(),
                refresh_token: None,
                expires_at: 0,
                scope,
            },
            path,
            encrypt_credentials: false,
        })
    }
    pub fn complete(self, cancel: &AtomicBool) -> Result<Arc<Session>, String> {
        if self.device.is_some() {
            return self.complete_device(cancel);
        }
        let listener = self.listener.as_ref().ok_or("本机回调不可用")?;
        let start = Instant::now();
        let code = loop {
            if cancel.load(Ordering::SeqCst) {
                return Err("授权已取消".into());
            }
            if start.elapsed() > Duration::from_secs(300) {
                return Err("授权等待超时，请重新连接".into());
            }
            match listener.accept() {
                Ok((mut stream, address)) => {
                    if !address.ip().is_loopback() {
                        continue;
                    }
                    if let Some(result) = callback(&mut stream, &self.redirect, &self.state) {
                        break result?;
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(75))
                }
                Err(_) => return Err("无法接收授权结果".into()),
            }
        };
        if cancel.load(Ordering::SeqCst) {
            return Err("授权已取消".into());
        }
        let value = form(
            &self.client,
            &self.credentials.token_endpoint,
            &[
                ("grant_type", "authorization_code"),
                ("code", &code),
                ("redirect_uri", &self.redirect),
                ("client_id", &self.credentials.client_id),
                ("code_verifier", &self.verifier),
                ("resource", &self.credentials.resource),
            ],
        )?;
        let (access, refresh, expires) = token_values(&value, None)?;
        let mut data = self.credentials;
        data.access_token = access;
        data.refresh_token = refresh;
        data.expires_at = expires;
        if cancel.load(Ordering::SeqCst) {
            return Err("授权已取消".into());
        }
        write_credentials(&self.path, &data, self.encrypt_credentials)?;
        Session::load(self.path, &data.server)
    }
}
fn constant_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.bytes().zip(b.bytes()).fold(0u8, |v, (a, b)| v | (a ^ b)) == 0
}
fn reply(stream: &mut TcpStream, ok: bool) {
    let body = if ok {
        "已收到授权结果，请返回小伴查看连接状态。"
    } else {
        "授权回调无效，请返回小伴重新连接。"
    };
    let status = if ok { "200 OK" } else { "400 Bad Request" };
    let _=write!(stream,"HTTP/1.1 {status}\r\nContent-Type: text/plain; charset=utf-8\r\nCache-Control: no-store\r\nContent-Security-Policy: default-src 'none'\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{body}",body.len());
}
fn callback(stream: &mut TcpStream, redirect: &str, state: &str) -> Option<Result<String, String>> {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));
    let mut reader = BufReader::new(&mut *stream);
    let mut bytes = 0usize;
    let mut line = String::new();
    if (&mut reader)
        .take(8193)
        .read_line(&mut line)
        .ok()
        .filter(|v| *v <= 8192)
        .is_none()
    {
        reply(stream, false);
        return None;
    }
    bytes += line.len();
    let first = line.clone();
    let parts = first.split_whitespace().collect::<Vec<_>>();
    if parts.len() != 3 || parts[0] != "GET" || !parts[1].starts_with('/') {
        reply(stream, false);
        return None;
    }
    let expected = Url::parse(redirect).ok()?;
    let mut host = String::new();
    loop {
        line.clear();
        let n = (&mut reader)
            .take((8193 - bytes) as u64)
            .read_line(&mut line)
            .ok()?;
        bytes += n;
        if bytes > 8192 || n == 0 {
            reply(stream, false);
            return None;
        }
        if line == "\r\n" || line == "\n" {
            break;
        }
        if let Some((key, value)) = line.split_once(':') {
            if key.eq_ignore_ascii_case("host") {
                host = value.trim().to_string();
            }
        }
    }
    let url = expected.join(parts[1]).ok()?;
    if host != format!("127.0.0.1:{}", expected.port()?)
        || url.origin() != expected.origin()
        || url.path() != expected.path()
    {
        reply(stream, false);
        return None;
    }
    let mut params = HashMap::new();
    for (key, value) in url.query_pairs() {
        if params.insert(key.to_string(), value.to_string()).is_some() {
            reply(stream, false);
            return None;
        }
    }
    if !params.get("state").is_some_and(|s| constant_eq(s, state)) {
        reply(stream, false);
        return None;
    }
    if params.contains_key("error") {
        reply(stream, true);
        return Some(Err("用户未完成授权，请重新连接".into()));
    }
    let code = params
        .get("code")
        .filter(|s| !s.is_empty() && s.len() <= 4096)
        .cloned();
    reply(stream, code.is_some());
    code.map(Ok)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bearer_parameters_preserve_quoted_commas_and_ignore_scheme_case() {
        let header =
            "bEaReR scope=\"read,write\", resource_metadata=\"https://auth.example/metadata\"";
        assert_eq!(parameter(header, "scope").as_deref(), Some("read,write"));
        assert_eq!(
            parameter(header, "resource_metadata").as_deref(),
            Some("https://auth.example/metadata")
        );
        assert!(parameter("Basic scope=\"read\"", "scope").is_none());
    }

    #[test]
    fn authorization_endpoints_and_session_paths_are_bounded() {
        for url in [
            "http://auth.example/token",
            "https://user:password@auth.example/token",
            "https://auth.example/token#fragment",
            "https://127.0.0.1/token",
            "https://10.0.0.1/token",
            "https://[fc00::1]/token",
        ] {
            assert!(endpoint(url, false).is_err(), "{url}");
        }
        assert!(endpoint("http://127.0.0.1:1234/token", true).is_ok());
        assert!(endpoint(&format!("https://auth.example/{}", "x".repeat(8192)), false).is_err());
        assert!(token_path(Path::new("/tmp"), "../escape", &random().unwrap()).is_err());
        assert!(token_path(Path::new("/tmp"), "valid", "../escape").is_err());
    }
}
