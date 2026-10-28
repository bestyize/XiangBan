#![cfg(feature = "testing")]
use base64::{engine::general_purpose::URL_SAFE_NO_PAD as B64, Engine};
use futures_util::{SinkExt, StreamExt};
use server_link::testing::TestServer;
use serde_json::{json, Value};
use std::time::Duration;
use tokio_tungstenite::{connect_async, tungstenite::{client::IntoClientRequest, Message}};
use uuid::Uuid;
use xiaoban_link_protocol::keys::DeviceKey;

#[cfg(windows)]
fn resident_bytes() -> Option<usize> {
    use std::{ffi::c_void, mem::size_of};
    #[repr(C)]
    struct Counters { size: u32, faults: u32, peak: usize, resident: usize, peak_paged: usize, paged: usize, peak_nonpaged: usize, nonpaged: usize, pagefile: usize, peak_pagefile: usize }
    #[link(name = "kernel32")]
    unsafe extern "system" { fn GetCurrentProcess() -> *mut c_void; }
    #[link(name = "psapi")]
    unsafe extern "system" { fn GetProcessMemoryInfo(process: *mut c_void, counters: *mut Counters, size: u32) -> i32; }
    let mut c = Counters { size: size_of::<Counters>() as u32, faults: 0, peak: 0, resident: 0, peak_paged: 0, paged: 0, peak_nonpaged: 0, nonpaged: 0, pagefile: 0, peak_pagefile: 0 };
    (unsafe { GetProcessMemoryInfo(GetCurrentProcess(), &mut c, size_of::<Counters>() as u32) } != 0).then_some(c.resident)
}

#[cfg(target_os = "linux")]
fn resident_bytes() -> Option<usize> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    let kb = status.lines().find_map(|line| line.strip_prefix("VmRSS:")?.split_whitespace().next()?.parse::<usize>().ok())?;
    Some(kb * 1024)
}

#[cfg(not(any(windows, target_os = "linux")))]
fn resident_bytes() -> Option<usize> { None }

#[tokio::test]
async fn axum_default_accepts_two_mib_but_configured_limit_rejects_it() {
    use axum::{extract::ws::{Message as AxumMessage, WebSocketUpgrade}, routing::get, Router};
    let router = Router::new()
        .route("/default", get(|ws: WebSocketUpgrade| async move {
            ws.on_upgrade(|mut socket| async move {
                if let Some(Ok(AxumMessage::Text(text))) = socket.next().await {
                    let _ = socket.send(AxumMessage::Text(text.len().to_string().into())).await;
                }
            })
        }))
        .route("/limited", get(|ws: WebSocketUpgrade| async move {
            ws.max_message_size(512 * 1024).max_frame_size(512 * 1024).on_upgrade(|mut socket| async move {
                let _ = socket.next().await;
            })
        }));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    let (mut default, _) = connect_async(format!("ws://{addr}/default")).await.unwrap();
    default.send(Message::Text("x".repeat(2 * 1024 * 1024).into())).await.unwrap();
    let reply = tokio::time::timeout(Duration::from_secs(5), default.next()).await.unwrap().unwrap().unwrap();
    assert_eq!(reply.to_text().unwrap(), "2097152");
    let (mut limited, _) = connect_async(format!("ws://{addr}/limited")).await.unwrap();
    let _ = limited.send(Message::Text("x".repeat(2 * 1024 * 1024).into())).await;
    let closed = tokio::time::timeout(Duration::from_secs(5), limited.next()).await.unwrap();
    assert!(matches!(closed, None | Some(Err(_)) | Some(Ok(Message::Close(_)))));
}

#[tokio::test]
async fn axum_default_accepts_fragmented_32_mib_message() {
    use axum::{extract::ws::{Message as AxumMessage, WebSocketUpgrade}, routing::get, Router};
    use std::sync::{atomic::{AtomicUsize, Ordering}, Arc};
    use tokio_tungstenite::tungstenite::protocol::frame::{coding::{Data, OpCode}, Frame};
    let measured = Arc::new(AtomicUsize::new(0));
    let server_measured = measured.clone();
    let router = Router::new().route("/", get(move |ws: WebSocketUpgrade| {
        let measured = server_measured.clone();
        async move { ws.on_upgrade(move |mut socket| async move {
            if let Some(Ok(AxumMessage::Text(text))) = socket.next().await {
                measured.store(resident_bytes().unwrap_or(0), Ordering::SeqCst);
                let _ = socket.send(AxumMessage::Text(text.len().to_string().into())).await;
            }
        }) }
    }));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    let (mut ws, _) = connect_async(format!("ws://{addr}/")).await.unwrap();
    let before = resident_bytes();
    for i in 0..4 {
        let opcode = if i == 0 { Data::Text } else { Data::Continue };
        ws.send(Message::Frame(Frame::message(vec![b'x'; 8 * 1024 * 1024], OpCode::Data(opcode), i == 3))).await.unwrap();
    }
    let reply = tokio::time::timeout(Duration::from_secs(15), ws.next()).await.unwrap().unwrap().unwrap();
    assert_eq!(reply.to_text().unwrap(), "33554432");
    println!("whole test process RSS before sending: {before:?}, during server handling: {} bytes (includes client)", measured.load(Ordering::SeqCst));
}

#[tokio::test]
async fn oversized_and_hostile_frames_do_not_affect_other_connections() {
    let Ok(url) = std::env::var("TEST_DATABASE_URL") else { eprintln!("skipped: TEST_DATABASE_URL is not set"); return; };
    let server = TestServer::start_with(&url, |config| { config.msgs_per_sec = 10000.0; config.bytes_per_sec = 32_000_000.0; }).await;
    let token = server.phone_token(Uuid::new_v4());
    let http = reqwest::Client::new();
    let mut ids = Vec::new();
    for name in ["one", "two", "half-open"] {
        let key = DeviceKey::generate();
        let response: Value = http.post(format!("{}/devices", server.base)).bearer_auth(&token)
            .json(&json!({"kind":"phone", "name":name, "public_key":B64.encode(key.public_bytes())}))
            .send().await.unwrap().error_for_status().unwrap().json().await.unwrap();
        ids.push(response["device_id"].as_str().unwrap().to_owned());
    }
    let connect = |id: String| {
        let mut request = format!("{}/ws?device_id={id}", server.ws_base).into_client_request().unwrap();
        request.headers_mut().insert("Authorization", format!("Bearer {token}").parse().unwrap());
        async move { connect_async(request).await.unwrap().0 }
    };
    let mut hostile = connect(ids[0].clone()).await;
    let mut healthy = connect(ids[1].clone()).await;
    for ws in [&mut hostile, &mut healthy] {
        let ready = tokio::time::timeout(Duration::from_secs(5), ws.next()).await.unwrap().unwrap().unwrap();
        assert!(ready.to_text().unwrap().contains("ready"));
    }
    let mut half_open = connect(ids[2].clone()).await;
    let ready = tokio::time::timeout(Duration::from_secs(5), half_open.next()).await.unwrap().unwrap().unwrap();
    assert!(ready.to_text().unwrap().contains("ready"));
    use tokio::io::AsyncWriteExt;
    half_open.get_mut().write_all(&[0x81, 0xfe, 0x01]).await.unwrap();
    healthy.send(Message::Text(r#"{"op":"ping"}"#.into())).await.unwrap();
    assert!(tokio::time::timeout(Duration::from_secs(5), healthy.next()).await.unwrap().unwrap().unwrap().to_text().unwrap().contains("pong"));
    for text in ["not json", "[]", "{\"op\":42}", "{\"op\":\"send\"}", "{\"op\":\"unknown\"}"] {
        hostile.send(Message::Text(text.into())).await.unwrap();
        let frame = tokio::time::timeout(Duration::from_secs(5), hostile.next()).await.unwrap().unwrap().unwrap();
        assert!(frame.to_text().unwrap().contains("error"));
    }
    for text in [format!("{{\"op\":\"send\",\"env\":{}}}", "[".repeat(200)), format!("{{\"op\":{}}}", "9".repeat(1000))] {
        hostile.send(Message::Text(text.into())).await.unwrap();
        let frame = tokio::time::timeout(Duration::from_secs(5), hostile.next()).await.unwrap().unwrap().unwrap();
        assert!(frame.to_text().unwrap().contains("error"));
    }
    let before = resident_bytes();
    for i in 0..300 {
        hostile.send(Message::Text(format!("{{\"op\":\"invalid-{i}\"}}").into())).await.unwrap();
        let frame = tokio::time::timeout(Duration::from_secs(5), hostile.next()).await.unwrap().unwrap().unwrap();
        assert!(frame.to_text().unwrap().contains("error"));
    }
    if let (Some(before), Some(after)) = (before, resident_bytes()) {
        assert!(after.saturating_sub(before) < 64 * 1024 * 1024, "RSS increased by {} bytes", after.saturating_sub(before));
    }
    hostile.send(Message::Binary(vec![0; 4].into())).await.unwrap();
    let frame = tokio::time::timeout(Duration::from_secs(5), hostile.next()).await.unwrap().unwrap().unwrap();
    assert!(frame.to_text().unwrap().contains("text_only"));
    // Before the size limit a single 2 MiB frame fits axum's default 16 MiB frame limit
    // and is delivered to application code. It must now close the connection instead.
    let _ = hostile.send(Message::Text("x".repeat(2 * 1024 * 1024).into())).await;
    let closed = tokio::time::timeout(Duration::from_secs(5), hostile.next()).await.unwrap();
    assert!(matches!(closed, None | Some(Err(_)) | Some(Ok(Message::Close(_)))));
    healthy.send(Message::Text(r#"{"op":"ping"}"#.into())).await.unwrap();
    let pong = tokio::time::timeout(Duration::from_secs(5), healthy.next()).await.unwrap().unwrap().unwrap();
    assert!(pong.to_text().unwrap().contains("pong"));
    drop(half_open);
    // Each fragment is below 512 KiB; their combined message must still be rejected.
    use tokio_tungstenite::tungstenite::protocol::frame::{coding::{Data, OpCode}, Frame};
    let mut fragmented = connect(ids[0].clone()).await;
    assert!(tokio::time::timeout(Duration::from_secs(5), fragmented.next()).await.unwrap().unwrap().unwrap().to_text().unwrap().contains("ready"));
    fragmented.send(Message::Frame(Frame::message(vec![b'x'; 400 * 1024], OpCode::Data(Data::Text), false))).await.unwrap();
    let _ = fragmented.send(Message::Frame(Frame::message(vec![b'x'; 400 * 1024], OpCode::Data(Data::Continue), true))).await;
    let closed = tokio::time::timeout(Duration::from_secs(5), fragmented.next()).await.unwrap();
    assert!(matches!(closed, None | Some(Err(_)) | Some(Ok(Message::Close(_)))));
    healthy.send(Message::Text(r#"{"op":"ping"}"#.into())).await.unwrap();
    assert!(tokio::time::timeout(Duration::from_secs(5), healthy.next()).await.unwrap().unwrap().unwrap().to_text().unwrap().contains("pong"));
}
