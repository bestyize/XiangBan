use agent_connector_tools::ConnectorManager;
use agent_kernel::ToolCtx;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use reqwest::{blocking::Client, Url};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};

#[derive(Default)]
struct State {
    redirect: String,
    challenge: String,
    refreshes: usize,
    token_calls: usize,
    expire: bool,
    short_lived: bool,
    wrong_resource: bool,
    refuse_refresh: bool,
    legacy: bool,
    post_auth: bool,
    device_polls: usize,
    unauthenticated_probes: usize,
    stream: Option<TcpStream>,
}
struct Server {
    base: String,
    state: Arc<Mutex<State>>,
    stop: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}
impl Server {
    fn new() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let state = Arc::new(Mutex::new(State::default()));
        let stop = Arc::new(AtomicBool::new(false));
        let data = state.clone();
        let quit = stop.clone();
        let origin = base.clone();
        let thread = std::thread::spawn(move || {
            while !quit.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((stream, _)) => handle(stream, &origin, &data),
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(5))
                    }
                    Err(_) => break,
                }
            }
        });
        Self {
            base,
            state,
            stop,
            thread: Some(thread),
        }
    }
    fn config(&self) -> Value {
        let mut config = json!({"url":format!("{}/mcp",self.base)});
        if self.state.lock().unwrap().legacy { config["transport"] = json!("sse"); }
        config
    }
    fn callback(&self, result: &Value, valid: bool) -> u16 {
        let url = Url::parse(result["authorizationUrl"].as_str().unwrap()).unwrap();
        let pairs = url
            .query_pairs()
            .collect::<std::collections::HashMap<_, _>>();
        assert_eq!(pairs["code_challenge_method"], "S256");
        assert_eq!(pairs["scope"], "read");
        assert_eq!(pairs["resource"], format!("{}/mcp", self.base));
        self.state.lock().unwrap().challenge = pairs["code_challenge"].to_string();
        let mut callback = Url::parse(&pairs["redirect_uri"]).unwrap();
        callback
            .query_pairs_mut()
            .append_pair(
                "state",
                if valid {
                    &pairs["state"]
                } else {
                    "wrong-state"
                },
            )
            .append_pair("code", "one-use-code");
        Client::new()
            .get(callback)
            .send()
            .unwrap()
            .status()
            .as_u16()
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        self.thread.take().unwrap().join().unwrap();
    }
}
fn send(stream: &mut TcpStream, status: &str, body: Value, extra: &str) {
    let body = body.to_string();
    write!(stream,"HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n{extra}\r\n{body}",body.len()).unwrap();
}
fn handle(mut stream: TcpStream, base: &str, state: &Mutex<State>) {
    // Windows may inherit the listener's nonblocking mode on accept.
    stream.set_nonblocking(false).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .unwrap();
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    let first = line.clone();
    let parts = first.split_whitespace().collect::<Vec<_>>();
    let (method, path) = (parts[0], parts[1]);
    let mut length = 0;
    let mut auth = String::new();
    loop {
        line.clear();
        reader.read_line(&mut line).unwrap();
        if line.trim().is_empty() {
            break;
        }
        if let Some((key, v)) = line.split_once(':') {
            if key.eq_ignore_ascii_case("content-length") {
                length = v.trim().parse().unwrap();
            }
            if key.eq_ignore_ascii_case("authorization") {
                auth = v.trim().into();
            }
        }
    }
    let mut bytes = vec![0; length];
    reader.read_exact(&mut bytes).unwrap();
    let mut state = state.lock().unwrap();
    match (method, path) {
        ("GET", "/mcp") => {
            assert!(state.legacy);
            if !state.post_auth && (auth.is_empty() || (state.expire && auth == "Bearer access-secret-1")) {
                send(&mut stream,"401 Unauthorized",json!({}),&format!("WWW-Authenticate: Bearer resource_metadata=\"{base}/resource\", scope=\"read\"\r\n"));
            } else {
                stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\n\r\nevent: endpoint\ndata: /messages\n\n").unwrap();
                state.stream = Some(stream);
            }
        }
        ("GET", "/resource") => {
            let resource = if state.wrong_resource {
                "https://other.invalid/mcp".into()
            } else {
                format!("{base}/mcp")
            };
            send(
                &mut stream,
                "200 OK",
                json!({"resource":resource,"authorization_servers":[base]}),
                "",
            );
        }
        ("GET", "/.well-known/oauth-authorization-server") => send(
            &mut stream,
            "200 OK",
            json!({"issuer":base,"authorization_endpoint":format!("{base}/login"),"token_endpoint":format!("{base}/token"),"registration_endpoint":format!("{base}/register"),"code_challenge_methods_supported":["S256"]}),
            "",
        ),
        ("POST", "/device/code") => {
            send(
                &mut stream,
                "200 OK",
                json!({
                    "device_code": "dev-code-1234",
                    "user_code": "XB-7788",
                    "verification_uri": format!("{base}/verify"),
                    "expires_in": 30,
                    "interval": 1
                }),
                "",
            );
        }
        ("POST", "/device/token") => {
            let body = Url::parse(&format!(
                "https://form.invalid/?{}",
                String::from_utf8(bytes).unwrap()
            ))
            .unwrap();
            let p = body
                .query_pairs()
                .collect::<std::collections::HashMap<_, _>>();
            state.token_calls += 1;
            if p.get("grant_type").map(|s| s.as_ref())
                == Some("urn:ietf:params:oauth:grant-type:device_code")
            {
                assert_eq!(p["device_code"], "dev-code-1234");
                if state.device_polls == 0 {
                    state.device_polls += 1;
                    send(
                        &mut stream,
                        "200 OK",
                        json!({"error": "authorization_pending"}),
                        "",
                    );
                } else {
                    send(
                        &mut stream,
                        "200 OK",
                        json!({
                            "access_token": "device-access-1",
                            "refresh_token": "device-refresh-1",
                            "token_type": "Bearer",
                            "expires_in": if state.short_lived { 1 } else { 3600 }
                        }),
                        "",
                    );
                }
            } else if p.get("grant_type").map(|s| s.as_ref()) == Some("refresh_token") {
                state.refreshes += 1;
                send(
                    &mut stream,
                    "200 OK",
                    json!({
                        "access_token": "device-access-2",
                        "refresh_token": "device-refresh-2",
                        "token_type": "Bearer",
                        "expires_in": 3600
                    }),
                    "",
                );
            } else {
                send(
                    &mut stream,
                    "400 Bad Request",
                    json!({"error": "unsupported_grant_type"}),
                    "",
                );
            }
        }
        ("POST", "/register") => {
            let body: Value = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(body["token_endpoint_auth_method"], "none");
            state.redirect = body["redirect_uris"][0].as_str().unwrap().into();
            send(
                &mut stream,
                "201 Created",
                json!({"client_id":"xiaoban-test","redirect_uris":[state.redirect],"token_endpoint_auth_method":"none"}),
                "",
            );
        }
        ("POST", "/token") => {
            let body = Url::parse(&format!(
                "https://form.invalid/?{}",
                String::from_utf8(bytes).unwrap()
            ))
            .unwrap();
            let p = body
                .query_pairs()
                .collect::<std::collections::HashMap<_, _>>();
            assert_eq!(p["client_id"], "xiaoban-test");
            assert_eq!(p["resource"], format!("{base}/mcp"));
            state.token_calls += 1;
            if p["grant_type"] == "authorization_code" {
                assert_eq!(p["code"], "one-use-code");
                assert_eq!(p["redirect_uri"], state.redirect);
                assert_eq!(
                    URL_SAFE_NO_PAD.encode(Sha256::digest(p["code_verifier"].as_bytes())),
                    state.challenge
                );
                send(
                    &mut stream,
                    "200 OK",
                    json!({"access_token":"access-secret-1","refresh_token":"refresh-secret-1","token_type":"Bearer","expires_in":if state.short_lived{1}else{3600}}),
                    "",
                );
            } else {
                state.refreshes += 1;
                if state.refuse_refresh {
                    send(
                        &mut stream,
                        "400 Bad Request",
                        json!({"error":"invalid_grant","error_description":"do-not-display-this-secret"}),
                        "",
                    );
                    return;
                }
                assert_eq!(
                    p["refresh_token"],
                    format!("refresh-secret-{}", state.refreshes)
                );
                let n = state.refreshes + 1;
                send(
                    &mut stream,
                    "200 OK",
                    json!({"access_token":format!("access-secret-{n}"),"refresh_token":format!("refresh-secret-{n}"),"token_type":"Bearer","expires_in":3600}),
                    "",
                );
            }
        }
        ("DELETE", "/mcp") => {
            assert!(auth.starts_with("Bearer access-secret-"));
            send(&mut stream, "200 OK", json!({}), "");
        }
        ("POST", "/mcp" | "/messages") => {
            if auth.is_empty() { state.unauthenticated_probes += 1; }
            assert_eq!(path == "/messages", state.legacy);
            if auth.is_empty()
                || (state.expire
                    && (auth == "Bearer access-secret-1"
                        || auth == "Bearer device-access-1"
                        || state.refuse_refresh))
            {
                send(&mut stream,"401 Unauthorized",json!({}),&format!("WWW-Authenticate: Bearer resource_metadata=\"{base}/resource\", scope=\"read\"\r\n"));
                return;
            }
            assert!(
                auth.starts_with("Bearer access-secret-")
                    || auth.starts_with("Bearer device-access-")
            );
            let body: Value = serde_json::from_slice(&bytes).unwrap();
            let result = match body["method"].as_str().unwrap() {
                "initialize" => {
                    json!({"protocolVersion":if state.legacy {"2024-11-05"} else {"2025-06-18"},"serverInfo":{"name":"oauth-test","version":"1"},"capabilities":{"tools":{}}})
                }
                "notifications/initialized" => {
                    send(&mut stream, "202 Accepted", json!({}), "");
                    return;
                }
                "tools/list" => json!({"tools":[{"name":"echo","inputSchema":{"type":"object"}}]}),
                "tools/call" => json!({"content":[{"type":"text","text":"authorized result"}]}),
                _ => panic!("unexpected RPC"),
            };
            if state.legacy {
                send(&mut stream, "202 Accepted", json!({}), "");
                write!(state.stream.as_mut().unwrap(), "event: message\ndata: {}\n\n", json!({"jsonrpc":"2.0","id":body["id"],"result":result})).unwrap();
            } else { send(
                &mut stream,
                "200 OK",
                json!({"jsonrpc":"2.0","id":body["id"],"result":result}),
                "",
            ); }
        }
        _ => send(&mut stream, "404 Not Found", json!({}), ""),
    }
}
fn root() -> PathBuf {
    std::env::temp_dir().join(format!(
        "xiaoban-oauth-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}
fn finish(manager: &ConnectorManager, flow: &Value) -> Value {
    let start = Instant::now();
    loop {
        let status = manager
            .authorization_status(flow["flowId"].as_str().unwrap())
            .unwrap();
        if status["status"] == "connected"
            || status["status"] == "error"
            || status["status"] == "cancelled"
        {
            return status;
        }
        assert!(start.elapsed() < Duration::from_secs(10));
        std::thread::sleep(Duration::from_millis(10));
    }
}
#[test]
fn legacy_sse_get_and_post_401_discover_authorize_refresh_and_restore() {
    for post_auth in [false, true] {
        let server = Server::new();
        { let mut state = server.state.lock().unwrap(); state.legacy = true; state.post_auth = post_auth; }
        let root = root();
        let manager = Arc::new(ConnectorManager::new(root.clone(), root.clone()).unwrap());
        let flow = manager.begin_connect("legacy-oauth", "Legacy OAuth", server.config(), true).unwrap();
        assert_eq!(flow["status"], "authorization_required");
        assert_eq!(server.callback(&flow, true), 200);
        let status = finish(&manager, &flow);
        assert_eq!(status["status"], "connected", "{status}");
        let tools = manager.tools();
        assert_eq!(tools[0].execute(json!({}), &ToolCtx::detached("legacy-oauth")).unwrap(), "authorized result");
        server.state.lock().unwrap().expire = true;
        if post_auth {
            assert_eq!(tools[0].execute(json!({}), &ToolCtx::detached("refresh-post")).unwrap(), "authorized result");
        } else {
            manager.disconnect("legacy-oauth").unwrap();
            manager.reconnect_with_authorization("legacy-oauth").unwrap();
        }
        assert_eq!(server.state.lock().unwrap().refreshes, 1);
        assert!(!manager.statuses().to_string().contains("secret"));
        manager.shutdown();
        let restored = ConnectorManager::new(root.clone(), root.clone()).unwrap();
        assert_eq!(restored.tools().len(), 1);
        assert_eq!(restored.tools()[0].execute(json!({}), &ToolCtx::detached("restore")).unwrap(), "authorized result");
        restored.shutdown();
        fs::remove_dir_all(root).unwrap();
    }
}
#[test]
fn browser_authorization_checks_state_binds_resource_refreshes_and_restores() {
    let server = Server::new();
    let root = root();
    let manager = Arc::new(ConnectorManager::new(root.clone(), root.clone()).unwrap());
    let flow = manager
        .begin_connect("oauth-test", "OAuth", server.config(), true)
        .unwrap();
    assert_eq!(flow["status"], "authorization_required");
    assert_eq!(server.callback(&flow, false), 400);
    assert!(manager.tools().is_empty());
    assert_eq!(server.callback(&flow, true), 200);
    let status = finish(&manager, &flow);
    assert_eq!(status["status"], "connected", "{status}");
    assert!(!manager.statuses().to_string().contains("secret"));
    let records = fs::read_to_string(root.join("connections.json")).unwrap();
    assert!(!records.contains("access-secret"));
    assert!(!records.contains("refresh-secret"));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for file in fs::read_dir(&root).unwrap() {
            assert_eq!(
                file.unwrap().metadata().unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
    }
    let tools = manager.tools();
    server.state.lock().unwrap().expire = true;
    let result = tools[0]
        .execute(json!({}), &ToolCtx::detached("oauth"))
        .unwrap();
    assert_eq!(result, "authorized result");
    assert_eq!(server.state.lock().unwrap().refreshes, 1);
    manager.disconnect("oauth-test").unwrap();
    assert!(tools[0]
        .execute(json!({}), &ToolCtx::detached("stale"))
        .is_err());
    manager.reconnect_with_authorization("oauth-test").unwrap();
    assert_eq!(manager.tools().len(), 1);
    manager.shutdown();
    let restored = ConnectorManager::new(root.clone(), root.clone()).unwrap();
    assert_eq!(restored.tools().len(), 1);
    restored.shutdown();
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn cancellation_does_not_save_credentials_or_report_connected() {
    let server = Server::new();
    let root = root();
    let manager = Arc::new(ConnectorManager::new(root.clone(), root.clone()).unwrap());
    let flow = manager
        .begin_connect("oauth-test", "OAuth", server.config(), true)
        .unwrap();
    manager
        .cancel_authorization(flow["flowId"].as_str().unwrap())
        .unwrap();
    assert_eq!(finish(&manager, &flow)["status"], "cancelled");
    assert!(manager.tools().is_empty());
    assert!(!root.join("connections.json").exists());
    manager.shutdown();
    std::thread::sleep(Duration::from_millis(100));
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn credentials_refresh_before_expiry_and_failed_refresh_requires_reauthorization() {
    let server = Server::new();
    server.state.lock().unwrap().short_lived = true;
    let root = root();
    let manager = Arc::new(ConnectorManager::new(root.clone(), root.clone()).unwrap());
    let flow = manager
        .begin_connect("oauth-test", "OAuth", server.config(), true)
        .unwrap();
    server.callback(&flow, true);
    let status = finish(&manager, &flow);
    assert_eq!(status["status"], "connected", "{status}");
    assert_eq!(server.state.lock().unwrap().refreshes, 1);
    {
        let mut state = server.state.lock().unwrap();
        state.expire = true;
        state.refuse_refresh = true;
    }
    let error = manager.tools()[0]
        .execute(json!({}), &ToolCtx::detached("expired"))
        .unwrap_err();
    assert!(error.contains("需要重新授权"));
    assert!(!error.contains("do-not-display-this-secret"));
    assert_eq!(manager.statuses()[0]["connected"], false);
    assert_eq!(server.state.lock().unwrap().refreshes, 2);
    let next = manager.reconnect_with_authorization("oauth-test").unwrap();
    assert_eq!(next["status"], "authorization_required");
    manager
        .cancel_authorization(next["flowId"].as_str().unwrap())
        .unwrap();
    manager.shutdown();
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn replacing_disconnected_authorization_retires_only_the_old_generation() {
    let server = Server::new();
    let root = root();
    let manager = Arc::new(ConnectorManager::new(root.clone(), root.clone()).unwrap());
    let first = manager
        .begin_connect("oauth-test", "OAuth", server.config(), true)
        .unwrap();
    server.callback(&first, true);
    assert_eq!(finish(&manager, &first)["status"], "connected");
    manager.disconnect("oauth-test").unwrap();
    let next = manager
        .begin_connect("oauth-test", "OAuth", server.config(), true)
        .unwrap();
    server.callback(&next, true);
    assert_eq!(finish(&manager, &next)["status"], "connected");
    let token_files = || {
        fs::read_dir(&root)
            .unwrap()
            .filter(|file| {
                file.as_ref()
                    .unwrap()
                    .file_name()
                    .to_string_lossy()
                    .starts_with("oauth-")
            })
            .count()
    };
    assert_eq!(token_files(), 1);
    manager
        .cancel_authorization(first["flowId"].as_str().unwrap())
        .unwrap();
    assert_eq!(manager.statuses()[0]["connected"], true);
    assert_eq!(token_files(), 1);
    manager
        .cancel_authorization(next["flowId"].as_str().unwrap())
        .unwrap();
    assert_eq!(finish(&manager, &next)["status"], "connected");
    assert_eq!(manager.tools()[0].execute(json!({}), &ToolCtx::detached("late-cancel")).unwrap(), "authorized result");
    assert_eq!(token_files(), 1);
    manager.shutdown();
    let restored = ConnectorManager::new(root.clone(), root.clone()).unwrap();
    assert_eq!(restored.tools().len(), 1);
    restored.disconnect("oauth-test").unwrap();
    assert!(restored.tools().is_empty());
    // Disconnect retains credentials for a deliberate reconnect.
    assert_eq!(token_files(), 1);
    restored.shutdown();
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn refuses_resource_metadata_for_another_service() {
    let server = Server::new();
    server.state.lock().unwrap().wrong_resource = true;
    let root = root();
    let manager = Arc::new(ConnectorManager::new(root.clone(), root.clone()).unwrap());
    let error = manager
        .begin_connect("oauth-test", "OAuth", server.config(), true)
        .unwrap_err();
    assert!(error.contains("不匹配"));
    assert!(manager.tools().is_empty());
    manager.shutdown();
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn plugin_oauth_encrypts_rotates_restores_and_unbinds_credentials() {
    use agent_connector_tools::plugin::{AuthSpec, ConnectorPlugin, Display, ServerSpec, Transport};
    let server = Server::new();
    let root = root();
    let plugin = ConnectorPlugin {
        name: "oauth-plugin".into(), version: "1".into(),
        display: Display { name: "OAuth plugin".into(), ..Default::default() },
        servers: vec![ServerSpec {
            name: "service".into(), transport: Transport::StreamableHttp,
            url: Some(format!("{}/mcp",server.base)), command: None, args: vec![],
            env: Default::default(), headers: Default::default(), auth: AuthSpec::None,
            disabled_tools: vec![], timeout_ms: Some(2000),
        }], skills_dir: None, icon: None,
    };
    let id = plugin.connection_id("service");
    let manager = Arc::new(ConnectorManager::new(root.clone(), root.clone()).unwrap());
    let flow = manager.begin_connect_plugin(&plugin, "service", &Default::default(), true).unwrap();
    assert_eq!(flow["status"], "authorization_required");
    assert_eq!(server.callback(&flow, true), 200);
    assert_eq!(finish(&manager, &flow)["status"], "connected");
    server.state.lock().unwrap().expire = true;
    assert_eq!(manager.tools()[0].execute(json!({}), &ToolCtx::detached("plugin")).unwrap(), "authorized result");
    assert_eq!(server.state.lock().unwrap().refreshes, 1);
    fn no_plaintext(dir: &std::path::Path) {
        for entry in fs::read_dir(dir).unwrap().map(Result::unwrap) {
            if entry.path().is_dir() { no_plaintext(&entry.path()); }
            else {
                let bytes = fs::read(entry.path()).unwrap();
                let text = String::from_utf8_lossy(&bytes);
                assert!(!text.contains("access-secret")); assert!(!text.contains("refresh-secret"));
            }
        }
    }
    no_plaintext(&root);
    manager.shutdown(); drop(manager);
    let restored = Arc::new(ConnectorManager::new(root.clone(), root.clone()).unwrap());
    assert_eq!(restored.tools().len(), 1);
    server.state.lock().unwrap().expire = false;
    let next = restored.begin_connect_plugin(&plugin, "service", &Default::default(), true).unwrap();
    assert_eq!(next["status"], "authorization_required");
    server.callback(&next, true);
    assert_eq!(finish(&restored, &next)["status"], "connected");
    assert_eq!(fs::read_dir(&root).unwrap().map(Result::unwrap).filter(|e| e.file_name().to_string_lossy().starts_with("oauth-")).count(), 1);
    no_plaintext(&root);
    restored.unbind_plugin(&id).unwrap();
    assert!(restored.tools().is_empty());
    assert_eq!(fs::read_dir(root.join("vault")).unwrap().map(Result::unwrap).filter(|e| e.path().extension().is_some_and(|s| s == "json")).count(), 0);
    restored.shutdown();
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn device_authorization_polls_returns_code_refreshes_and_restores() {
    let server = Server::new();
    let root = root();
    let manager = Arc::new(ConnectorManager::new(root.clone(), root.clone()).unwrap());
    let config = json!({
        "url": format!("{}/mcp", server.base),
        "authMode": "device",
        "deviceAuth": {
            "codeUrl": format!("{}/device/code", server.base),
            "tokenUrl": format!("{}/device/token", server.base),
            "clientId": "test-device-client",
            "scope": "repo"
        }
    });
    let flow = manager
        .begin_connect("dev-flow-test", "Device Flow", config, true)
        .unwrap();
    assert_eq!(flow["status"], "authorization_required");
    assert_eq!(flow["userCode"], "XB-7788");
    assert_eq!(flow["authorizationUrl"], format!("{}/verify", server.base));
    assert_eq!(server.state.lock().unwrap().unauthenticated_probes, 0, "device login must not wait for an MCP challenge");
    let status = finish(&manager, &flow);
    assert_eq!(status["status"], "connected", "{status}");
    // Device login has no callback into the view. Cleanup before the next UI poll
    // must not undo a connection that has already finished in the background.
    manager.cancel_authorization(flow["flowId"].as_str().unwrap()).unwrap();
    assert_eq!(finish(&manager, &flow)["status"], "connected");
    let tools = manager.tools();
    assert_eq!(tools.len(), 1);
    assert_eq!(
        tools[0]
            .execute(json!({}), &ToolCtx::detached("device-first"))
            .unwrap(),
        "authorized result"
    );
    server.state.lock().unwrap().expire = true;
    assert_eq!(
        tools[0]
            .execute(json!({}), &ToolCtx::detached("device-refresh"))
            .unwrap(),
        "authorized result"
    );
    assert_eq!(server.state.lock().unwrap().refreshes, 1);
    manager.shutdown();
    let restored = ConnectorManager::new(root.clone(), root.clone()).unwrap();
    assert_eq!(restored.tools().len(), 1);
    assert_eq!(
        restored.tools()[0]
            .execute(json!({}), &ToolCtx::detached("device-restored"))
            .unwrap(),
        "authorized result"
    );
    restored.shutdown();
    fs::remove_dir_all(root).unwrap();
}
