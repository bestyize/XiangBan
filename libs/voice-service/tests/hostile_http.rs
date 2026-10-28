mod common;
use common::{TestServer, TOKEN};
use serde_json::Value;
use std::time::Duration;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream, time::timeout};

#[tokio::test]
async fn malformed_http_inputs_leave_the_server_usable() {
    let server = TestServer::new().await;
    let deep = format!("{}0{}", "[".repeat(200), "]".repeat(200));
    let huge = format!("{{\"input\":{}}}", "9".repeat(1000));
    let huge_id = format!("{{\"jsonrpc\":\"2.0\",\"id\":{},\"method\":\"ping\"}}", "9".repeat(1000));
    let deep_params = format!("{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"ping\",\"params\":{deep}}}");
    let speech: Vec<Vec<u8>> = vec![b"{".to_vec(), vec![0xff, 0xfe], deep.as_bytes().to_vec(), huge.into_bytes(), b"{\"input\":4}".to_vec(), b"{\"input\":\"a\",\"input\":\"b\"}".to_vec()];
    let long_name = format!("--edge\r\nContent-Disposition: form-data; name=\"{}\"\r\n\r\nx\r\n--edge--\r\n", "x".repeat(8192));
    let multipart = ["garbage", "--edge\r\nContent-Disposition: form-data; name=\"model\"\r\n\r\nx\r\n--edge--\r\n", "--edge\r\nContent-Disposition: form-data; name=\"file\"\r\n", "--edge\r\nBad: \r\n nested\r\n\r\na\r\n--edge--\r\n", "--edge\r\nContent-Disposition: form-data; name=\"file\"\r\n\r\na\r\n--edge\r\nContent-Disposition: form-data; name=\"file\"\r\n\r\nb\r\n--edge--\r\n", long_name.as_str()];
    for body in speech {
        let response = timeout(Duration::from_secs(5), server.post("/v1/audio/speech").header("content-type", "application/json").body(body).send()).await.unwrap().unwrap();
        assert!(response.status().is_client_error() || response.status().is_server_error());
        let json: Value = response.json().await.unwrap(); assert!(json["error"]["code"].is_string());
    }
    for body in multipart {
        let response = timeout(Duration::from_secs(5), server.post("/v1/audio/transcriptions").header("content-type", "multipart/form-data; boundary=edge").body(body.to_owned()).send()).await.unwrap().unwrap();
        assert!(response.status().is_client_error() || response.status().is_server_error());
        let json: Value = response.json().await.unwrap(); assert!(json["error"]["code"].is_string());
    }
    for body in ["{", "[]", "{}", "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":123}", "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"ping\",\"params\":[]}", deep.as_str(), huge_id.as_str(), deep_params.as_str()] {
        let response = timeout(Duration::from_secs(5), server.post("/mcp").header("content-type", "application/json").header("accept", "application/json, text/event-stream").body(body.to_owned()).send()).await.unwrap().unwrap();
        assert!(response.status().is_client_error() || response.status().is_server_error());
        let json: Value = response.json().await.unwrap(); assert!(json["error"].is_object());
    }
    let mut random = 0x9e37_79b9_7f4a_7c15u64;
    for path in ["/v1/audio/speech", "/v1/audio/transcriptions", "/mcp"] {
        for _ in 0..30 {
            let mut body = vec![0xff];
            for _ in 0..100 { random ^= random << 13; random ^= random >> 7; random ^= random << 17; body.push(random as u8); }
            let content_type = if path.ends_with("transcriptions") { "multipart/form-data; boundary=edge" } else { "application/json" };
            let response = timeout(Duration::from_secs(5), server.post(path).header("content-type", content_type).header("accept", "application/json, text/event-stream").body(body).send()).await.unwrap().unwrap();
            assert!(response.status().is_client_error() || response.status().is_server_error());
            let json: Value = response.json().await.unwrap(); assert!(json["error"].is_object());
        }
    }
    assert_eq!(server.get("/health").send().await.unwrap().status(), 200);
}

#[tokio::test]
async fn stalled_request_bodies_timeout_without_blocking_other_requests() {
    let server = TestServer::new().await;
    let mut sockets = Vec::new();
    for _ in 0..3 {
        let mut socket = TcpStream::connect(server.running.addr).await.unwrap();
        let headers = format!("POST /v1/audio/speech HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {TOKEN}\r\nContent-Type: application/json\r\nContent-Length: 100\r\n\r\n{{");
        socket.write_all(headers.as_bytes()).await.unwrap();
        sockets.push(socket);
    }
    assert_eq!(timeout(Duration::from_secs(5), server.get("/health").send()).await.unwrap().unwrap().status(), 200);
    let (mut reader, mut writer) = sockets.remove(0).into_split();
    let mut data = vec![0; 4096];
    let n = timeout(Duration::from_secs(35), async {
        let mut ticks = tokio::time::interval(Duration::from_secs(1));
        loop {
            tokio::select! {
                result = reader.read(&mut data) => break result.unwrap(),
                _ = ticks.tick() => writer.write_all(b" ").await.unwrap(),
            }
        }
    }).await.unwrap();
    let response = String::from_utf8_lossy(&data[..n]);
    assert!(response.contains("408") && response.contains("request_timeout"), "{response}");
    assert_eq!(server.get("/health").send().await.unwrap().status(), 200);
}

#[tokio::test]
#[ignore = "known limitation: axum::serve does not configure a header-read timer"]
async fn stalled_request_headers_are_closed_within_35_seconds() {
    let server = TestServer::new().await;
    let mut socket = TcpStream::connect(server.running.addr).await.unwrap();
    socket.write_all(b"GET /health HTTP/1.1\r\nHost: 127.0.0.1\r\nX-Slow: ").await.unwrap();
    assert_eq!(server.get("/health").send().await.unwrap().status(), 200);
    let (mut reader, mut writer) = socket.into_split();
    let mut data = [0; 1024];
    let read = timeout(Duration::from_secs(35), async {
        let mut ticks = tokio::time::interval(Duration::from_secs(1));
        loop {
            tokio::select! {
                result = reader.read(&mut data) => break result,
                _ = ticks.tick() => writer.write_all(b"a").await.unwrap(),
            }
        }
    }).await;
    assert!(read.is_ok(), "incomplete HTTP headers remained open for more than 35 seconds");
}

#[tokio::test]
async fn contradictory_and_truncated_http_framing_cannot_break_the_server() {
    let server = TestServer::new().await;
    for headers_and_body in [
        format!("POST /v1/audio/speech HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {TOKEN}\r\nContent-Type: application/json\r\nContent-Length: 100\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n0\r\n\r\n"),
        format!("POST /v1/audio/speech HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {TOKEN}\r\nContent-Type: application/json\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\nZZ\r\n{{}}\r\n0\r\n\r\n"),
        format!("POST /v1/audio/speech HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {TOKEN}\r\nContent-Type: application/json\r\nContent-Length: 100\r\nConnection: close\r\n\r\n{{"),
    ] {
        let mut stream = TcpStream::connect(server.running.addr).await.unwrap();
        stream.write_all(headers_and_body.as_bytes()).await.unwrap();
        stream.shutdown().await.unwrap();
        let mut bytes = Vec::new();
        timeout(Duration::from_secs(5), stream.read_to_end(&mut bytes)).await.unwrap().unwrap();
        let response = String::from_utf8_lossy(&bytes);
        assert!(response.is_empty() || response.starts_with("HTTP/1.1 4") || response.starts_with("HTTP/1.1 5"), "{response}");
    }
    assert_eq!(server.get("/health").send().await.unwrap().status(), 200);
}
