use agent_connector_tools::{
    plugin::{AuthSpec, ConnectorPlugin, Transport},
    vault::Vault,
    ConnectorManager,
};
use agent_kernel::ToolCtx;
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let mut nonce = [0; 12];
        getrandom::fill(&mut nonce).unwrap();
        let path =
            std::env::temp_dir().join(format!("connector-lib-{}-{:x?}", std::process::id(), nonce));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn values() -> BTreeMap<String, String> {
    BTreeMap::from([("TOKEN".into(), "unique-private-token-91234".into())])
}
fn fixture(root: &Path, mcp: Value, policy: Value) -> ConnectorPlugin {
    fs::create_dir_all(root.join(".codebuddy-plugin")).unwrap();
    fs::create_dir_all(root.join("ai.workbuddy/assets")).unwrap();
    fs::create_dir_all(root.join("skills/helper")).unwrap();
    fs::write(root.join("skills/helper/SKILL.md"), "# helper").unwrap();
    fs::write(root.join("ai.workbuddy/assets/icon.svg"), "<svg/>").unwrap();
    fs::write(root.join(".codebuddy-plugin/plugin.json"), json!({
        "name":"fixture", "version":"1.0", "extensions":{"ai.workbuddy":{
            "display":{"name":"测试","name_en":"Test","examples_zh":["试一试"]},
            "setup":{"mcpServers":{"service":{"auth":{"type":"token","schema":"./ai.workbuddy/token-schema.json","headers":{}}}}},
            "policy":{"mcpServers":{"service":policy}}
        }}
    }).to_string()).unwrap();
    fs::write(
        root.join("ai.workbuddy/token-schema.json"),
        json!({"title":"Token","fields":[
            {"key":"TOKEN","type":"password","required":true}, {"key":"OPTIONAL","type":"text"}
        ]})
        .to_string(),
    )
    .unwrap();
    fs::write(
        root.join(".mcp.json"),
        json!({"mcpServers":{"service":mcp}}).to_string(),
    )
    .unwrap();
    ConnectorPlugin::load(root).unwrap()
}
#[test]
fn templates_schema_auth_and_safe_paths() {
    let tmp = Temp::new();
    let plugin_dir = tmp.0.join("plugin");
    let mut plugin = fixture(
        &plugin_dir,
        json!({"type":"streamableHttp","url":"https://example.com/${TOKEN}","headers":{"Authorization":"wrong","X-Optional":"${OPTIONAL}"}}),
        json!({"disabledTools":["write"],"timeout":1234}),
    );
    let spec = &mut plugin.servers[0];
    assert_eq!(spec.transport, Transport::StreamableHttp);
    if let AuthSpec::Token { headers, .. } = &mut spec.auth {
        headers.push(("authorization".into(), "TOKEN".into(), "Bearer ".into()));
    }
    assert_eq!(spec.disabled_tools, vec!["write"]);
    assert_eq!(spec.timeout_ms, Some(1234));
    assert!(plugin.icon.as_ref().unwrap().is_file());
    assert!(plugin.skills_dir.as_ref().unwrap().is_dir());
    let built = plugin.build_config("service", &values()).unwrap();
    assert_eq!(
        built["headers"]["authorization"],
        format!("Bearer {}", values()["TOKEN"])
    );
    assert!(built["headers"].get("Authorization").is_none());
    assert_eq!(built["headers"]["X-Optional"], "");
    assert!(built["url"].as_str().unwrap().ends_with(&values()["TOKEN"]));
    assert!(plugin.build_config("missing", &values()).is_err());
    assert!(plugin.build_config("service", &BTreeMap::new()).is_err());
    for bad in ["\r\ninjected", "nul\0byte", "\u{7f}"] {
        assert!(plugin
            .build_config("service", &BTreeMap::from([("TOKEN".into(), bad.into())]))
            .is_err());
    }
    let mut unknown = values();
    unknown.insert("UNKNOWN".into(), "value".into());
    assert!(plugin.build_config("service", &unknown).is_err());
    plugin.servers[0].url = Some("https://example.com/${UNKNOWN}".into());
    assert!(plugin.build_config("service", &values()).is_err());
    plugin.servers[0].transport = Transport::Sse;
    assert!(plugin.servers[0]
        .unsupported()
        .unwrap()
        .contains("暂不支持"));
    assert!(plugin
        .build_config("service", &values())
        .unwrap_err()
        .contains("暂不支持"));
    let path = plugin_dir.join(".codebuddy-plugin/plugin.json");
    let mut manifest: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    manifest["extensions"]["ai.workbuddy"]["setup"]["mcpServers"]["service"]["auth"]["schema"] =
        json!("../outside.json");
    fs::write(tmp.0.join("outside.json"), "{}").unwrap();
    fs::write(&path, manifest.to_string()).unwrap();
    assert!(ConnectorPlugin::load(&plugin_dir).is_err());
}
#[test]
fn vault_authenticates_identity_nonce_ciphertext_and_master() {
    let tmp = Temp::new();
    let vault = Vault::open(tmp.0.join("vault")).unwrap();
    vault.set("one", "token", "secret").unwrap();
    vault.set("one", "other", "second").unwrap();
    vault.set("two", "token", "different").unwrap();
    assert_eq!(
        vault.get("one", "token").unwrap().as_deref(),
        Some("secret")
    );
    assert_eq!(vault.has("one").unwrap(), vec!["other", "token"]);
    assert_eq!(vault.get("none", "missing").unwrap(), None);
    let files: Vec<_> = fs::read_dir(tmp.0.join("vault"))
        .unwrap()
        .map(|f| f.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect();
    let path = files
        .iter()
        .find(|p| {
            let v: Value = serde_json::from_slice(&fs::read(p).unwrap()).unwrap();
            v.get("other").is_some()
        })
        .unwrap();
    let original = fs::read(path).unwrap();
    assert!(!String::from_utf8_lossy(&original).contains("secret"));
    vault.set("one", "token", "secret").unwrap();
    let renewed = fs::read(path).unwrap();
    assert_ne!(renewed, original);
    let original: Value = serde_json::from_slice(&renewed).unwrap();
    for bad in [json!("invalid base64"), json!("")] {
        let mut corrupt = original.clone();
        corrupt["token"]["nonce"] = bad;
        fs::write(path, corrupt.to_string()).unwrap();
        assert!(vault.get("one", "token").is_err());
        assert!(vault.has("one").is_err());
    }
    let mut corrupt = original.clone();
    corrupt["token"]["data"] = json!("AAAA");
    fs::write(path, corrupt.to_string()).unwrap();
    assert!(vault.get("one", "token").is_err());
    let mut swapped = original.clone();
    swapped["token"] = original["other"].clone();
    fs::write(path, swapped.to_string()).unwrap();
    assert!(vault.get("one", "token").is_err());
    fs::write(path, original.to_string()).unwrap();
    let other = files.iter().find(|p| *p != path).unwrap();
    let saved_other = fs::read(other).unwrap();
    fs::copy(path, other).unwrap();
    assert!(vault.get("two", "token").is_err());
    fs::write(other, saved_other).unwrap();
    drop(vault);
    let replacement = Vault::open(tmp.0.join("replacement")).unwrap();
    drop(replacement);
    fs::copy(
        tmp.0.join("replacement/master.key"),
        tmp.0.join("vault/master.key"),
    )
    .unwrap();
    let vault = Vault::open(tmp.0.join("vault")).unwrap();
    assert!(vault.get("one", "token").is_err());
    vault.remove_connector("one").unwrap();
    assert!(vault.has("one").unwrap().is_empty());
    vault.remove_connector("one").unwrap();
    fs::remove_file(tmp.0.join("vault/master.key")).unwrap();
    assert!(Vault::open(tmp.0.join("vault")).is_err());
}
#[test]
fn plugin_stdio_policy_restore_overrides_and_unbind() {
    let tmp = Temp::new();
    let script = tmp.0.join("server.py");
    fs::write(&script, r#"import sys,json,os
for line in sys.stdin:
 r=json.loads(line)
 if 'id' not in r: continue
 if r['method']=='initialize': v={'protocolVersion':'2024-11-05','capabilities':{'tools':{}},'serverInfo':{'name':'test','version':'1'}}
 elif r['method']=='tools/list': v={'tools':[{'name':n,'inputSchema':{'type':'object'}} for n in ['read','write']]}
 else: v={'content':[{'type':'text','text':'original' if os.environ.get('TOKEN')=='unique-private-token-91234' else 'override'}]}
 print(json.dumps({'jsonrpc':'2.0','id':r['id'],'result':v}),flush=True)
"#).unwrap();
    let plugin = fixture(
        &tmp.0.join("plugin"),
        json!({"type":"stdio","command":if cfg!(windows) {"python"} else {"python3"},"args":[script],"env":{"TOKEN":"${TOKEN}"}}),
        json!({"disabledTools":["write"],"timeout":1500}),
    );
    let root = tmp.0.join("config");
    let id = plugin.connection_id("service");
    let manager = ConnectorManager::new(root.clone(), tmp.0.clone()).unwrap();
    assert!(manager
        .connect_plugin(&plugin, "service", &values(), false)
        .is_err());
    assert!(!root.join("vault").exists());
    manager
        .connect_plugin(&plugin, "service", &values(), true)
        .unwrap();
    assert_eq!(
        manager.configured_plugin_fields(&id).unwrap(),
        vec!["TOKEN"]
    );
    let tools = manager.tools();
    assert_eq!(tools.len(), 1);
    assert_eq!(manager.statuses()[0]["toolCount"], 1);
    assert_eq!(tools[0].spec().timeout, Duration::from_millis(1500));
    assert_eq!(
        tools[0]
            .execute(json!({}), &ToolCtx::detached("test"))
            .unwrap(),
        "original"
    );
    assert!(!fs::read_to_string(root.join("connections.json"))
        .unwrap()
        .contains(&values()["TOKEN"]));
    assert!(!manager.statuses().to_string().contains(&values()["TOKEN"]));
    manager.shutdown();
    drop(manager);
    let manager = ConnectorManager::new(root.clone(), tmp.0.clone()).unwrap();
    assert_eq!(manager.tools().len(), 1);
    assert!(tools[0]
        .execute(json!({}), &ToolCtx::detached("old"))
        .is_err());
    manager
        .set_plugin_overrides(
            &id,
            &BTreeMap::new(),
            &BTreeMap::from([("TOKEN".into(), "private-override-1234".into())]),
        )
        .unwrap();
    assert!(manager.tools().is_empty());
    assert!(!fs::read_to_string(root.join("connector-states.json"))
        .unwrap()
        .contains("private-override-1234"));
    manager.reconnect(&id).unwrap();
    let tools = manager.tools();
    assert_eq!(
        tools[0]
            .execute(json!({}), &ToolCtx::detached("override"))
            .unwrap(),
        "override"
    );
    manager.set_plugin_enabled(&id, false).unwrap();
    assert!(manager.plugin_state(&id).user_disabled);
    drop(manager);
    let manager = ConnectorManager::new(root.clone(), tmp.0.clone()).unwrap();
    assert!(manager.tools().is_empty());
    manager.set_plugin_enabled(&id, true).unwrap();
    assert_eq!(manager.tools().len(), 1);
    manager.unbind_plugin(&id).unwrap();
    assert!(manager.configured_plugin_fields(&id).unwrap().is_empty());
    assert!(manager.reconnect(&id).is_err());
    manager.shutdown();
}
#[test]
fn real_workbuddy_marketplace_oracle() {
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .unwrap_or_default();
    let root = PathBuf::from(home)
        .join(".workbuddy/plugins/marketplaces/workbuddy-connector-plugins-official/connectors");
    if !root.is_dir() {
        eprintln!("WorkBuddy marketplace absent: oracle skipped");
        return;
    }
    let mut loaded = 0;
    let mut token = 0;
    let mut built = 0;
    let mut sse = 0;
    let mut skills_only = 0;
    for dir in fs::read_dir(root)
        .unwrap()
        .map(|d| d.unwrap().path())
        .filter(|d| d.is_dir())
    {
        let plugin =
            ConnectorPlugin::load(&dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
        loaded += 1;
        if plugin.servers.is_empty() {
            skills_only += 1;
        }
        for spec in &plugin.servers {
            if spec.transport == Transport::Sse {
                sse += 1;
            }
            if let AuthSpec::Token { schema, .. } = &spec.auth {
                token += 1;
                let values = schema
                    .fields
                    .iter()
                    .map(|f| (f.key.clone(), "fixture-value-1234".into()))
                    .collect();
                let result = plugin.build_config(&spec.name, &values);
                if spec.unsupported().is_some() {
                    assert!(result.unwrap_err().contains("暂不支持"));
                } else {
                    let config =
                        result.unwrap_or_else(|e| panic!("{} / {}: {e}", plugin.name, spec.name));
                    assert!(config.get("url").is_some() || config.get("command").is_some());
                    built += 1;
                }
            }
        }
    }
    eprintln!("WorkBuddy oracle: loaded={loaded}, skill-only={skills_only}, token={token}, built={built}, SSE unsupported={sse}");
    assert!(loaded > 0);
    assert!(token > 0);
}

#[test]
fn http_plugin_uses_token_headers_and_call_timeout() {
    use std::io::{BufRead, BufReader, Read, Write};
    let tmp = Temp::new();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let thread = std::thread::spawn(move || {
        for step in 0..4 {
            let (mut socket, _) = listener.accept().unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            let mut reader = BufReader::new(socket.try_clone().unwrap());
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            let mut length = 0;
            let mut authenticated = false;
            loop {
                line.clear();
                reader.read_line(&mut line).unwrap();
                if line.trim().is_empty() {
                    break;
                }
                if let Some((key, value)) = line.split_once(':') {
                    if key.eq_ignore_ascii_case("content-length") {
                        length = value.trim().parse().unwrap();
                    }
                    if key.eq_ignore_ascii_case("authorization") {
                        assert_eq!(value.trim(), "Bearer unique-private-token-91234");
                        authenticated = true;
                    }
                }
            }
            assert!(authenticated);
            let mut bytes = vec![0; length];
            reader.read_exact(&mut bytes).unwrap();
            let request: Value = serde_json::from_slice(&bytes).unwrap();
            let result = match step {
                0 => {
                    json!({"protocolVersion":"2025-06-18","capabilities":{"tools":{}},"serverInfo":{"name":"test","version":"1"}})
                }
                1 => {
                    socket.write_all(b"HTTP/1.1 202 Accepted\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
                    continue;
                }
                2 => {
                    json!({"tools":[{"name":"read","inputSchema":{"type":"object"}},{"name":"write","inputSchema":{"type":"object"}}]})
                }
                _ => {
                    std::thread::sleep(Duration::from_millis(600));
                    json!({"content":[{"type":"text","text":"late"}]})
                }
            };
            let body = json!({"jsonrpc":"2.0","id":request["id"],"result":result}).to_string();
            let _ = write!(socket,"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len());
        }
    });
    let mut plugin = fixture(
        &tmp.0.join("plugin"),
        json!({"type":"streamable-http","url":format!("http://{address}/mcp")}),
        json!({"disabledTools":["write"],"timeout":120}),
    );
    if let AuthSpec::Token { headers, .. } = &mut plugin.servers[0].auth {
        headers.push(("Authorization".into(), "TOKEN".into(), "Bearer ".into()));
    }
    let manager = ConnectorManager::new(tmp.0.join("config"), tmp.0.clone()).unwrap();
    manager
        .connect_plugin(&plugin, "service", &values(), true)
        .unwrap();
    let tools = manager.tools();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].spec().timeout, Duration::from_millis(120));
    let start = std::time::Instant::now();
    assert!(
        tools[0]
            .execute_result(json!({}), &ToolCtx::detached("timeout"))
            .is_error
    );
    assert!(start.elapsed() < Duration::from_millis(500));
    manager.shutdown();
    thread.join().unwrap();
}

#[test]
fn legacy_sse_is_explicitly_unsupported_without_network_or_credential_writes() {
    let tmp = Temp::new();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let plugin = fixture(
        &tmp.0.join("plugin"),
        json!({"type":"sse","url":format!("http://{}/sse",listener.local_addr().unwrap())}),
        json!({}),
    );
    let root = tmp.0.join("config");
    let manager = ConnectorManager::new(root.clone(), tmp.0.clone()).unwrap();
    let error = manager
        .connect_plugin(&plugin, "service", &values(), true)
        .unwrap_err();
    assert!(error.contains("暂不支持"));
    assert!(!root.join("vault").exists());
    assert!(manager.statuses().as_array().unwrap().is_empty());
    assert_eq!(
        listener.accept().unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock
    );
}
