mod common;
use common::*;
use serde_json::json;
use std::sync::{atomic::Ordering, Arc};
use xiaoban_voice_service::*;

#[tokio::test]
async fn authentication_health_host_origin_and_preflight() {
    let server = TestServer::new().await;
    let response = server
        .client
        .get(server.url("/health"))
        .send()
        .await
        .unwrap();
    assert_eq!(
        response.json::<serde_json::Value>().await.unwrap(),
        json!({"ok":true})
    );
    for token in [None, Some("incorrect")] {
        let mut request = server.client.get(server.url("/v1/audio/voices"));
        if let Some(token) = token {
            request = request.bearer_auth(token);
        }
        assert_error(request.send().await.unwrap(), 401, "unauthorized").await;
    }
    assert_eq!(
        server
            .get("/v1/audio/voices")
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    for host in [
        "evil.com",
        "127.0.0.1.evil.com",
        "localhost@evil.com",
        "[::1]:evil",
        "127.0.0.1:99999",
    ] {
        assert_error(
            server
                .get("/v1/audio/voices")
                .header("host", host)
                .send()
                .await
                .unwrap(),
            403,
            "forbidden_host",
        )
        .await;
    }
    for host in [
        "localhost",
        "localhost:123",
        "[::1]",
        "[::1]:123",
        "127.0.0.1",
    ] {
        assert_eq!(
            server
                .get("/v1/audio/voices")
                .header("host", host)
                .send()
                .await
                .unwrap()
                .status(),
            200
        );
    }
    assert_error(
        server
            .get("/health")
            .header("origin", "https://evil.com")
            .send()
            .await
            .unwrap(),
        403,
        "forbidden_origin",
    )
    .await;
    assert_error(
        server
            .client
            .request(reqwest::Method::OPTIONS, server.url("/v1/audio/speech"))
            .send()
            .await
            .unwrap(),
        403,
        "forbidden_origin",
    )
    .await;
    let allowed = TestServer::with(
        Arc::new(Fake::default()),
        ServerConfig {
            allowed_origins: vec!["https://trusted.example".into()],
            ..Default::default()
        },
        false,
    )
    .await;
    let response = allowed
        .client
        .request(reqwest::Method::OPTIONS, allowed.url("/v1/audio/speech"))
        .header("origin", "https://trusted.example")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 204);
    assert_eq!(
        response.headers()["access-control-allow-origin"],
        "https://trusted.example"
    );
    let response = allowed
        .get("/v1/audio/voices")
        .header("origin", "https://trusted.example")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(
        response.headers()["access-control-allow-origin"],
        "https://trusted.example"
    );
    server.running.shutdown().await.unwrap();
    allowed.running.shutdown().await.unwrap();
}
#[tokio::test]
async fn rejects_bad_start_config() {
    let fake = Arc::new(Fake::default());
    for config in [
        ServerConfig::default(),
        ServerConfig {
            token: TOKEN.into(),
            bind: "0.0.0.0:0".parse().unwrap(),
            ..Default::default()
        },
        ServerConfig {
            token: TOKEN.into(),
            bind: "[::]:0".parse().unwrap(),
            ..Default::default()
        },
    ] {
        assert!(start(config, backends(&fake, false)).await.is_err());
    }
}
#[tokio::test]
async fn transcriptions_decode_all_formats_and_return_text_or_json() {
    let server = TestServer::new().await;
    let mut inputs = vec![
        tone(),
        wav(&vec![0.25; 3200], 16_000, 1, 32, true),
        wav(&vec![0.25; 3200], 16_000, 1, 24, false),
    ];
    inputs.extend(["mp3", "flac", "ogg"].map(fixtures::fixture));
    for bytes in inputs {
        let response = server
            .transcribe(bytes, "json", Some("zh"))
            .send()
            .await
            .unwrap();
        assert_eq!(
            response.status(),
            200,
            "{}",
            response.text().await.unwrap_or_default()
        );
    }
    for pcm in server.fake.seen.lock().unwrap().iter() {
        assert!((3000..5500).contains(&pcm.samples.len()));
        assert!(pcm
            .samples
            .iter()
            .all(|s| s.is_finite() && (-1.0..=1.0).contains(s)));
    }
    let response = server
        .transcribe(tone(), "text", None)
        .send()
        .await
        .unwrap();
    assert_eq!(
        response.headers()["content-type"],
        "text/plain; charset=utf-8"
    );
    assert_eq!(response.text().await.unwrap(), "你好");
    let response = server
        .transcribe(tone(), "json", None)
        .send()
        .await
        .unwrap();
    assert_eq!(
        response.json::<serde_json::Value>().await.unwrap(),
        json!({"text":"你好"})
    );
}
#[tokio::test]
async fn transcription_validation() {
    let server = TestServer::new().await;
    assert_error(
        server
            .transcribe(Vec::new(), "json", None)
            .send()
            .await
            .unwrap(),
        400,
        "audio_too_short",
    )
    .await;
    for format in ["verbose_json", "srt", "vtt", "bogus"] {
        assert_error(
            server
                .transcribe(tone(), format, None)
                .send()
                .await
                .unwrap(),
            400,
            "unsupported_response_format",
        )
        .await;
    }
    for bytes in [
        b"WebM Opus".to_vec(),
        b"\x00\x00\x00\x18ftypM4A".to_vec(),
        b"OggS".iter().copied().chain(vec![0; 40]).collect(),
    ] {
        let v = assert_error(
            server.transcribe(bytes, "json", None).send().await.unwrap(),
            415,
            "unsupported_audio_format",
        )
        .await;
        assert!(v["error"]["message"]
            .as_str()
            .unwrap()
            .contains("OGG Vorbis"));
    }
    for count in [0, 1599] {
        assert_error(
            server
                .transcribe(wav(&vec![0.1; count], 16_000, 1, 16, false), "json", None)
                .send()
                .await
                .unwrap(),
            400,
            "audio_too_short",
        )
        .await;
    }
    assert_error(
        server
            .transcribe(
                wav(&vec![0.0; audio::MAX_SAMPLES + 1], 16_000, 1, 16, false),
                "json",
                None,
            )
            .send()
            .await
            .unwrap(),
        413,
        "audio_too_long",
    )
    .await;
    let response = server
        .post("/v1/audio/transcriptions")
        .multipart(reqwest::multipart::Form::new().text("model", ""))
        .send()
        .await
        .unwrap();
    assert_error(response, 400, "missing_file").await;
    let response = server
        .post("/v1/audio/transcriptions")
        .multipart(
            reqwest::multipart::Form::new().part("file", reqwest::multipart::Part::bytes(tone())),
        )
        .send()
        .await
        .unwrap();
    assert_error(response, 400, "missing_model").await;
}
#[test]
fn resampling_preserves_sine_frequency_duration_and_averages_stereo() {
    let source: Vec<f32> = (0..44100)
        .flat_map(|i| {
            let s = (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 44100.0).sin();
            [s * 0.6, s * 0.2]
        })
        .collect();
    let pcm = audio::decode(&wav(&source, 44100, 2, 32, true)).unwrap();
    assert_eq!(pcm.samples.len(), 16000);
    let crossings = pcm
        .samples
        .windows(2)
        .filter(|w| w[0] <= 0.0 && w[1] > 0.0)
        .count();
    assert!((439..=441).contains(&crossings));
    let mse = pcm
        .samples
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let ideal = 0.4 * (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 16000.0).sin();
            (s - ideal).powi(2)
        })
        .sum::<f32>()
        / 16000.0;
    assert!(mse < 0.000001, "mse {mse}");
    let pcm = audio::decode(&wav(&[0.8, -0.8].repeat(3200), 16000, 2, 32, true)).unwrap();
    assert!(pcm.samples.iter().all(|s| s.abs() < 1e-6));
}
#[tokio::test]
async fn low_energy_segmentation_is_lossless_bounded_and_chinese_joined() {
    let fake = Arc::new(Fake {
        max: 64000,
        ..Default::default()
    });
    let server = TestServer::with(fake.clone(), ServerConfig::default(), false).await;
    let mut samples = vec![0.5; 150000];
    samples[45000..49000].fill(0.0);
    samples[92000..96000].fill(0.0);
    let ranges = audio::segments(&samples, fake.max).unwrap();
    assert!((45000..49000).contains(&ranges[0].end));
    assert!(ranges.iter().all(|r| r.len() <= fake.max));
    assert_eq!(ranges.iter().map(|r| r.len()).sum::<usize>(), samples.len());
    let response = server
        .transcribe(wav(&samples, 16000, 1, 32, true), "json", Some("zh-CN"))
        .send()
        .await
        .unwrap();
    let text = response.json::<serde_json::Value>().await.unwrap();
    assert_eq!(text["text"], "你好".repeat(ranges.len()));
    let received: Vec<f32> = fake
        .seen
        .lock()
        .unwrap()
        .iter()
        .flat_map(|p| p.samples.clone())
        .collect();
    assert_eq!(received, samples);
    fake.seen.lock().unwrap().clear();
    let response = server
        .transcribe(wav(&samples, 16000, 1, 32, true), "json", Some("en"))
        .send()
        .await
        .unwrap();
    assert_eq!(
        response.json::<serde_json::Value>().await.unwrap()["text"],
        vec!["你好"; ranges.len()].join(" ")
    );
    assert!(audio::segments(&samples, 0).is_err());
}
#[tokio::test]
async fn speech_voice_mapping_validation_mp3_and_wav() {
    let server = TestServer::new().await;
    for voice in [None, Some("second"), Some("alloy"), Some("nova")] {
        let mut input = json!({"model":"ignored", "input":"中文", "speed":1.5});
        if let Some(voice) = voice {
            input["voice"] = json!(voice);
        }
        let response = server
            .post("/v1/audio/speech")
            .json(&input)
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), 200);
        assert_eq!(response.headers()["content-type"], "audio/mpeg");
        assert_eq!(
            response.bytes().await.unwrap().as_ref(),
            fixtures::fixture("mp3")
        );
        let calls = server.fake.synthesis.lock().unwrap();
        assert_eq!(
            calls.last().unwrap().1.as_deref(),
            Some(if voice == Some("second") {
                "second"
            } else {
                "zh-CN-XiaoxiaoNeural"
            })
        );
    }
    for (input, code) in [
        (json!({"input":"", "model":"x"}), "empty_input"),
        (
            json!({"input":"你".repeat(4001),"model":"x"}),
            "input_too_long",
        ),
        (
            json!({"input":"a","model":"x","voice":"missing"}),
            "unknown_voice",
        ),
        (
            json!({"input":"a","model":"x","speed":0.1}),
            "invalid_speed",
        ),
        (
            json!({"input":"a","model":"x","speed":4.1}),
            "invalid_speed",
        ),
        (
            json!({"input":"a","model":"x","response_format":"flac"}),
            "unsupported_response_format",
        ),
    ] {
        assert_error(
            server
                .post("/v1/audio/speech")
                .json(&input)
                .send()
                .await
                .unwrap(),
            400,
            code,
        )
        .await;
    }
    let response = server
        .post("/v1/audio/speech")
        .json(&json!({"model":true,"input":"hello","response_format":"wav"}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.headers()["content-type"], "audio/wav");
    let bytes = response.bytes().await.unwrap();
    let pcm = audio::decode(&bytes).unwrap();
    assert!(pcm.samples.len() > 1600);
    assert!(pcm.samples.iter().any(|s| s.abs() > 0.01));
}
#[tokio::test]
async fn consent_denial_retry_cache_and_identity_capability_separation() {
    let server = TestServer::new().await;
    server.fake.allow.store(false, Ordering::SeqCst);
    assert_error(
        server
            .transcribe(tone(), "json", None)
            .send()
            .await
            .unwrap(),
        403,
        "consent_denied",
    )
    .await;
    server.fake.allow.store(true, Ordering::SeqCst);
    for _ in 0..2 {
        assert_eq!(
            server
                .transcribe(tone(), "json", None)
                .send()
                .await
                .unwrap()
                .status(),
            200
        );
    }
    assert_eq!(server.fake.approvals.lock().unwrap().len(), 2);
    assert_eq!(
        server
            .get("/v1/audio/voices")
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    assert_eq!(
        server
            .get("/v1/audio/voices")
            .header("x-client-name", "another")
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    assert_eq!(server.fake.approvals.lock().unwrap().len(), 4);
}
#[tokio::test]
async fn third_concurrent_transcription_is_busy_and_slots_recover() {
    let server = TestServer::new().await;
    server.fake.block_asr.store(true, Ordering::SeqCst);
    let first = tokio::spawn(server.transcribe(tone(), "json", None).send());
    let second = tokio::spawn(server.transcribe(tone(), "json", None).send());
    entered(&server.fake.asr_entered, 2).await;
    assert_error(
        server
            .transcribe(tone(), "json", None)
            .send()
            .await
            .unwrap(),
        429,
        "busy",
    )
    .await;
    server.fake.gate.add_permits(2);
    assert_eq!(first.await.unwrap().unwrap().status(), 200);
    assert_eq!(second.await.unwrap().unwrap().status(), 200);
    server.fake.block_asr.store(false, Ordering::SeqCst);
    assert_eq!(
        server
            .transcribe(tone(), "json", None)
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
}
#[tokio::test]
async fn all_backend_error_variants_have_openai_envelopes() {
    let server = TestServer::new().await;
    for (error, status, code) in [
        (VoiceError::Busy, 429, "busy"),
        (VoiceError::Unavailable("secret".into()), 503, "unavailable"),
        (
            VoiceError::Unsupported("secret".into()),
            415,
            "unsupported_audio_format",
        ),
        (VoiceError::TooLong, 413, "audio_too_long"),
        (
            VoiceError::InvalidInput("secret".into()),
            400,
            "invalid_input",
        ),
        (VoiceError::Cancelled, 408, "cancelled"),
        (VoiceError::Internal("secret".into()), 500, "internal_error"),
    ] {
        *server.fake.error.lock().unwrap() = Some(error);
        let value = assert_error(
            server
                .transcribe(tone(), "json", None)
                .send()
                .await
                .unwrap(),
            status,
            code,
        )
        .await;
        assert!(!value.to_string().contains("secret"));
    }
}
#[tokio::test]
async fn limits_json_and_chunked_bodies() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let server = TestServer::with(
        Arc::new(Fake::default()),
        ServerConfig {
            max_body_bytes: 128,
            ..Default::default()
        },
        false,
    )
    .await;
    assert_error(
        server
            .post("/v1/audio/speech")
            .body("x".repeat(129))
            .send()
            .await
            .unwrap(),
        413,
        "body_too_large",
    )
    .await;
    assert_error(
        server
            .post("/v1/audio/speech")
            .header("content-type", "application/json")
            .body("{")
            .send()
            .await
            .unwrap(),
        400,
        "invalid_json",
    )
    .await;
    let mut socket = tokio::net::TcpStream::connect(server.running.addr)
        .await
        .unwrap();
    let body = "x".repeat(129);
    socket.write_all(format!("POST /v1/audio/speech HTTP/1.1\r\nHost: localhost\r\nAuthorization: Bearer {TOKEN}\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n81\r\n{body}\r\n0\r\n\r\n").as_bytes()).await.unwrap();
    let mut response = String::new();
    socket.read_to_string(&mut response).await.unwrap();
    assert!(response.starts_with("HTTP/1.1 413"));
    assert!(response.contains("body_too_large"));
}
