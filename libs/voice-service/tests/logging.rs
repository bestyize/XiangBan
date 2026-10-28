mod common;
use common::*;
use std::{
    io::{self, Write},
    sync::{Arc, Mutex},
};
#[derive(Clone)]
struct Capture(Arc<Mutex<Vec<u8>>>);
impl Write for Capture {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for Capture {
    type Writer = Self;
    fn make_writer(&'a self) -> Self {
        self.clone()
    }
}
#[tokio::test]
async fn request_logs_exclude_tokens_text_headers_bodies_and_raw_paths() {
    let bytes = Arc::new(Mutex::new(Vec::new()));
    tracing::subscriber::set_global_default(
        tracing_subscriber::fmt()
            .without_time()
            .with_ansi(false)
            .with_writer(Capture(bytes.clone()))
            .finish(),
    )
    .unwrap();
    let server = TestServer::new().await;
    let response = server
        .post("/v1/audio/speech")
        .header("x-client-name", "secret-client")
        .json(&serde_json::json!({"model":"x","input":"secret-spoken-text"}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(
        server
            .transcribe(tone(), "json", None)
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    assert_eq!(
        server
            .get("/secret-path?secret-query")
            .send()
            .await
            .unwrap()
            .status(),
        404
    );
    server.running.shutdown().await.unwrap();
    let logs = String::from_utf8(bytes.lock().unwrap().clone()).unwrap();
    assert!(logs.contains("voice request"));
    assert!(logs.contains("/v1/audio/speech"));
    for secret in [
        TOKEN,
        "secret-spoken-text",
        "你好",
        "secret-client",
        "secret-path",
        "secret-query",
    ] {
        assert!(!logs.contains(secret), "log leaked {secret}");
    }
}
