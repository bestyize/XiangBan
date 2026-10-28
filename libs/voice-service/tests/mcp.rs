mod common;
use common::*;
use serde_json::{json, Value};
use std::sync::{atomic::Ordering, Arc};
use xiaoban_voice_service::*;

fn rpc(server: &TestServer, method: &str, params: Value) -> reqwest::RequestBuilder {
    server
        .post("/mcp")
        .header("accept", "application/json, text/event-stream")
        .header("mcp-protocol-version", "2025-11-25")
        .json(&json!({"jsonrpc":"2.0","id":1,"method":method,"params":params}))
}
fn modern_body(method: &str, mut params: Value) -> Value {
    params["_meta"] = json!({"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientInfo":{"name":"test-modern","version":"1"},"io.modelcontextprotocol/clientCapabilities":{}});
    json!({"jsonrpc":"2.0","id":"modern","method":method,"params":params})
}
fn modern(server: &TestServer, method: &str, params: Value) -> reqwest::RequestBuilder {
    let name = params["name"].as_str().map(str::to_owned);
    let mut request = server
        .post("/mcp")
        .header("accept", "application/json, text/event-stream")
        .header("mcp-protocol-version", "2026-07-28")
        .header("mcp-method", method);
    if let Some(name) = name {
        request = request.header("mcp-name", name);
    }
    request.json(&modern_body(method, params))
}
async fn devices() -> TestServer {
    TestServer::with(Arc::new(Fake::default()), Default::default(), true).await
}
async fn value(request: reqwest::RequestBuilder) -> Value {
    let response = request.send().await.unwrap();
    assert_eq!(response.status(), 200);
    response.json().await.unwrap()
}
async fn busy_value(request: reqwest::RequestBuilder) -> Value {
    let response = request.send().await.unwrap();
    // 忙是工具层的错误：HTTP 200，不带 Retry-After，错误在 JSON-RPC 结果的 isError 里。
    assert_eq!(response.status(), 200);
    assert!(response.headers().get("retry-after").is_none());
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["result"]["structuredContent"]["busy"], true);
    body
}
#[tokio::test]
async fn legacy_initialize_notification_ping_list_and_all_tools() {
    let server = devices().await;
    let initialized = value(rpc(&server,"initialize",json!({"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"test","version":"1"}}))).await;
    assert_eq!(initialized["result"]["protocolVersion"], "2025-11-25");
    let response = server
        .post("/mcp")
        .header("accept", "application/json, text/event-stream")
        .json(&json!({"jsonrpc":"2.0","method":"notifications/initialized"}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 202);
    assert!(response.bytes().await.unwrap().is_empty());
    assert_eq!(
        value(rpc(&server, "ping", json!({}))).await["result"],
        json!({})
    );
    let listed = value(rpc(&server, "tools/list", json!({}))).await;
    let names: Vec<_> = listed["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, vec!["speak", "listen", "converse", "notify"]);
    let spoken = value(rpc(
        &server,
        "tools/call",
        json!({"name":"speak","arguments":{"text":"hello"}}),
    ))
    .await;
    assert_eq!(
        spoken["result"]["structuredContent"],
        json!({"spoken":true})
    );
    assert_eq!(server.fake.played.load(Ordering::SeqCst), 1);
    let heard = value(rpc(
        &server,
        "tools/call",
        json!({"name":"listen","arguments":{}}),
    ))
    .await;
    assert_eq!(heard["result"]["structuredContent"], json!({"text":"你好"}));
    assert_eq!(server.fake.mic_seconds.load(Ordering::SeqCst), 15);
    let conversation = value(rpc(&server,"tools/call",json!({"name":"converse","arguments":{"text":"question","max_seconds":30,"voice":"second"}}))).await;
    assert_eq!(
        conversation["result"]["structuredContent"],
        json!({"spoken":true,"text":"你好"})
    );
    assert_eq!(server.fake.played.load(Ordering::SeqCst), 2);
    assert_eq!(server.fake.mic_seconds.load(Ordering::SeqCst), 30);
    let approvals = server.fake.approvals.lock().unwrap();
    assert_eq!(approvals.len(), 4);
    assert!(approvals.iter().any(|(_, cap)| *cap == Capability::Listen));
}
#[tokio::test]
async fn absent_devices_are_hidden_and_uncallable() {
    for (mic, speaker, expected) in [
        (false, false, vec![]),
        (true, false, vec!["listen"]),
        (false, true, vec!["speak", "notify"]),
    ] {
        let fake = Arc::new(Fake::default());
        let mut backends = backends(&fake, true);
        if !mic {
            backends.microphone = None;
        }
        if !speaker {
            backends.speaker = None;
        }
        let running = start(
            ServerConfig {
                token: TOKEN.into(),
                ..Default::default()
            },
            backends,
        )
        .await
        .unwrap();
        let server = TestServer {
            running,
            fake,
            client: reqwest::Client::builder().no_proxy().build().unwrap(),
        };
        let listed = value(rpc(&server, "tools/list", json!({}))).await;
        let names: Vec<_> = listed["result"]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v["name"].as_str().unwrap())
            .collect();
        assert_eq!(names, expected);
        if !mic {
            assert_eq!(
                value(rpc(&server, "tools/call", json!({"name":"listen"}))).await["error"]["code"],
                -32602
            );
        }
    }
}
#[tokio::test]
async fn modern_discovery_metadata_headers_and_results() {
    let server = devices().await;
    let discovered = value(modern(&server, "server/discover", json!({}))).await;
    assert_eq!(discovered["result"]["supportedVersions"][0], "2026-07-28");
    assert_eq!(discovered["result"]["resultType"], "complete");
    assert_eq!(
        value(modern(&server, "tools/list", json!({}))).await["result"]["tools"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
    let spoken = value(modern(
        &server,
        "tools/call",
        json!({"name":"speak","arguments":{"text":"hello"}}),
    ))
    .await;
    assert_eq!(spoken["result"]["resultType"], "complete");
    assert_eq!(spoken["result"]["isError"], false);
    let response = modern(&server, "tools/list", json!({}))
        .header("mcp-method", "tools/call")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 400);
    assert_eq!(
        response.json::<Value>().await.unwrap()["error"]["code"],
        -32020
    );
    let mut body = modern_body("tools/list", json!({}));
    body["params"]["_meta"]["io.modelcontextprotocol/protocolVersion"] = json!("1900-01-01");
    let response = server
        .post("/mcp")
        .header("accept", "application/json, text/event-stream")
        .header("mcp-protocol-version", "1900-01-01")
        .header("mcp-method", "tools/list")
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 400);
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["error"]["code"], -32022);
    assert_eq!(body["error"]["data"]["supported"][0], "2026-07-28");
    let response = modern(&server, "unknown-method", json!({}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 404);
    assert_eq!(
        response.json::<Value>().await.unwrap()["error"]["code"],
        -32601
    );
    let mut request = modern(
        &server,
        "tools/call",
        json!({"name":"speak","arguments":{"text":"x"}}),
    )
    .build()
    .unwrap();
    request
        .headers_mut()
        .insert("mcp-name", "=?base64?c3BlYWs=?=".parse().unwrap());
    let response = server.client.execute(request).await.unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(
        response.json::<Value>().await.unwrap()["result"]["isError"],
        false
    );
}
#[tokio::test]
async fn listen_empty_silent_busy_and_concurrent_limit() {
    let server = devices().await;
    *server.fake.captured.lock().unwrap() = Pcm::default();
    assert_eq!(
        value(rpc(&server, "tools/call", json!({"name":"listen"}))).await["result"]
            ["structuredContent"]["text"],
        ""
    );
    *server.fake.captured.lock().unwrap() = Pcm {
        samples: vec![0.0; 3200],
    };
    assert_eq!(
        value(rpc(&server, "tools/call", json!({"name":"listen"}))).await["result"]
            ["structuredContent"]["text"],
        ""
    );
    *server.fake.error.lock().unwrap() = Some(VoiceError::Busy);
    let busy = busy_value(rpc(&server, "tools/call", json!({"name":"listen"}))).await;
    assert_eq!(busy["result"]["isError"], true);
    assert!(busy["result"]["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("麦克风正被占用"));
    *server.fake.error.lock().unwrap() = None;
    server.fake.block_mic.store(true, Ordering::SeqCst);
    let first = tokio::spawn(rpc(&server, "tools/call", json!({"name":"listen"})).send());
    entered(&server.fake.mic_entered, 4).await;
    let busy = busy_value(rpc(&server, "tools/call", json!({"name":"listen"}))).await;
    assert_eq!(busy["result"]["isError"], true);
    assert_eq!(server.fake.mic_entered.load(Ordering::SeqCst), 4);
    server.fake.gate.add_permits(1);
    assert_eq!(first.await.unwrap().unwrap().status(), 200);
}
#[tokio::test]
async fn invalid_protocol_parameters_and_mcp_security() {
    let server = devices().await;
    for args in [
        json!({"max_seconds":0}),
        json!({"max_seconds":31}),
        json!({"max_seconds":1.5}),
        json!({"max_seconds":"15"}),
        json!({"other":1}),
    ] {
        assert_eq!(
            value(rpc(
                &server,
                "tools/call",
                json!({"name":"listen","arguments":args})
            ))
            .await["error"]["code"],
            -32602
        );
    }
    for args in [
        json!({}),
        json!({"text":""}),
        json!({"text":"x","voice":false}),
        json!({"text":"x".repeat(4001)}),
    ] {
        assert_eq!(
            value(rpc(
                &server,
                "tools/call",
                json!({"name":"speak","arguments":args})
            ))
            .await["error"]["code"],
            -32602
        );
    }
    let response = server
        .post("/mcp")
        .json(&json!({"jsonrpc":"2.0","id":1,"method":"ping"}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 406);
    let response = server
        .post("/mcp")
        .header("accept", "application/json, text/event-stream")
        .header("content-type", "application/json")
        .body("[")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 400);
    assert_eq!(
        response.json::<Value>().await.unwrap()["error"]["code"],
        -32700
    );
    let response = server
        .client
        .post(server.url("/mcp"))
        .json(&json!({"jsonrpc":"2.0","id":1,"method":"ping"}))
        .send()
        .await
        .unwrap();
    assert_error(response, 401, "unauthorized").await;
    assert_eq!(server.get("/mcp").send().await.unwrap().status(), 405);
    server.fake.allow.store(false, Ordering::SeqCst);
    assert_error(
        rpc(
            &server,
            "tools/call",
            json!({"name":"converse","arguments":{"text":"x"}}),
        )
        .send()
        .await
        .unwrap(),
        403,
        "consent_denied",
    )
    .await;
    assert_eq!(server.fake.played.load(Ordering::SeqCst), 0);
    assert_eq!(server.fake.mic_entered.load(Ordering::SeqCst), 0);
}
#[tokio::test]
async fn notify_returns_before_playback_and_shutdown_cancels_background_work() {
    let server = devices().await;
    server.fake.block_speaker.store(true, Ordering::SeqCst);
    let notification = value(rpc(
        &server,
        "tools/call",
        json!({"name":"notify","arguments":{"text":"notice"}}),
    ))
    .await;
    assert_eq!(
        notification["result"]["structuredContent"],
        json!({"queued":true})
    );
    entered(&server.fake.played, 1).await;
    let busy = busy_value(rpc(
        &server,
        "tools/call",
        json!({"name":"speak","arguments":{"text":"x"}}),
    ))
    .await;
    assert_eq!(busy["result"]["isError"], true);
    tokio::time::timeout(std::time::Duration::from_secs(2), server.running.shutdown())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(server.fake.gate.available_permits(), 0);
}

#[tokio::test]
async fn tool_deadline_is_bounded_and_releases_speaker() {
    let server = devices().await;
    server.fake.block_speaker.store(true, Ordering::SeqCst);
    let first = tokio::spawn(
        rpc(
            &server,
            "tools/call",
            json!({"name":"speak","arguments":{"text":"x"}}),
        )
        .send(),
    );
    entered(&server.fake.played, 1).await;
    tokio::time::pause();
    tokio::time::advance(std::time::Duration::from_secs(121)).await;
    tokio::time::resume();
    let response = first.await.unwrap().unwrap();
    assert_eq!(response.status(), 200);
    let result: Value = response.json().await.unwrap();
    assert_eq!(result["result"]["isError"], true);
    assert!(result["result"]["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("超时"));
    server.fake.block_speaker.store(false, Ordering::SeqCst);
    let result = value(rpc(
        &server,
        "tools/call",
        json!({"name":"speak","arguments":{"text":"y"}}),
    ))
    .await;
    assert_eq!(result["result"]["isError"], false);
}

#[cfg(feature = "stdio")]
#[tokio::test]
async fn stdio_binary_round_trip_legacy_and_modern_without_stdout_noise() {
    use std::process::Stdio;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    let server = devices().await;
    let mut child = tokio::process::Command::new(env!("CARGO_BIN_EXE_xiaoban-voice-mcp-stdio"))
        .env("XIAOBAN_VOICE_URL", server.url("/mcp"))
        .env("XIAOBAN_VOICE_TOKEN", TOKEN)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    let mut output = BufReader::new(child.stdout.take().unwrap()).lines();
    let requests = [
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"test","version":"1"}}}),
        json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
        json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}),
        json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"speak","arguments":{"text":"hello"}}}),
        modern_body("server/discover", json!({})),
    ];
    for request in requests {
        input
            .write_all(format!("{request}\n").as_bytes())
            .await
            .unwrap();
        input.flush().await.unwrap();
        if request.get("id").is_none() {
            continue;
        }
        let line = tokio::time::timeout(std::time::Duration::from_secs(5), output.next_line())
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        let response: Value = serde_json::from_str(&line).unwrap();
        assert_eq!(response["id"], request["id"]);
        assert!(response.get("result").is_some(), "{response}");
    }
    input.write_all(b"[]\n").await.unwrap();
    input.flush().await.unwrap();
    let line = tokio::time::timeout(std::time::Duration::from_secs(5), output.next_line())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    let invalid: Value = serde_json::from_str(&line).unwrap();
    assert_eq!(invalid["id"], Value::Null);
    assert_eq!(invalid["error"]["code"], -32600);
    drop(input);
    assert!(
        tokio::time::timeout(std::time::Duration::from_secs(5), child.wait())
            .await
            .unwrap()
            .unwrap()
            .success()
    );
    assert!(output.next_line().await.unwrap().is_none());
}
