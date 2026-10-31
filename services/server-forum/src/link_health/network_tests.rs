use super::*;
use std::sync::{Arc, Mutex};
fn public() -> Vec<SocketAddr> {
    vec!["93.184.216.34:443".parse().unwrap()]
}
#[test]
fn public_address_filter_rejects_ipv4_ipv6_private_mapped_and_reserved_ranges() {
    for raw in [
        "0.0.0.0",
        "10.1.2.3",
        "100.64.0.1",
        "127.0.0.1",
        "169.254.169.254",
        "172.16.0.1",
        "192.168.1.2",
        "192.0.0.1",
        "192.0.2.1",
        "192.88.99.1",
        "198.18.0.1",
        "198.51.100.1",
        "203.0.113.1",
        "224.0.0.1",
        "255.255.255.255",
        "::",
        "::1",
        "::ffff:127.0.0.1",
        "fc00::1",
        "fe80::1",
        "ff02::1",
        "2001:db8::1",
        "2002:7f00:1::1",
        "64:ff9b::7f00:1",
        "3fff::1",
    ] {
        assert!(!public_ip(raw.parse().unwrap()), "{raw}")
    }
    for raw in [
        "8.8.8.8",
        "1.1.1.1",
        "93.184.216.34",
        "2606:4700:4700::1111",
        "2001:4860:4860::8888",
    ] {
        assert!(public_ip(raw.parse().unwrap()), "{raw}")
    }
    let mut mixed = public();
    mixed.push("127.0.0.1:443".parse().unwrap());
    assert!(!valid_addresses(&mixed));
    assert!(!valid_addresses(&[]));
}
#[test]
fn urls_are_fixed_hosts_https_and_have_no_credential_escape_hatches() {
    for raw in [
        "http://openai.com/",
        "https://openai.com:444/",
        "https://user:pass@openai.com/",
        "https://127.0.0.1/",
        "https://2130706433/",
        "https://openai.com.evil.example/",
        "https://evil.example/",
        "https://openai.com/?%61pi_key=secret",
        "https://openai.com/?access_token=secret",
    ] {
        assert!(normalized(raw).is_err(), "{raw}")
    }
    assert_eq!(
        normalized("https://openai.com/news/#section")
            .unwrap()
            .as_str(),
        "https://openai.com/news/"
    );
    assert!(!display_url("https://user:secret@openai.com/news/?token=secret").contains("secret"));
}
#[tokio::test]
async fn dns_rebinding_and_redirect_hosts_are_checked_before_every_hop() {
    let calls = Arc::new(Mutex::new(0));
    let next = calls.clone();
    let result = probe_with(
        "https://openai.com/news".into(),
        move |_| {
            let next = next.clone();
            async move {
                let mut n = next.lock().unwrap();
                *n += 1;
                if *n == 1 {
                    Ok(public())
                } else {
                    Ok(vec!["127.0.0.1:443".parse().unwrap()])
                }
            }
        },
        |_, _, _| async {
            Ok(Headers {
                status: 302,
                location: Some("https://github.com/openai/codex".into()),
            })
        },
    )
    .await;
    assert_eq!(result.status, "blocked");
    assert_eq!(*calls.lock().unwrap(), 2);
    let result = probe_with(
        "https://openai.com/".into(),
        |_| async { Ok(public()) },
        |_, _, _| async {
            Ok(Headers {
                status: 302,
                location: Some("http://169.254.169.254/private".into()),
            })
        },
    )
    .await;
    assert_eq!(result.status, "blocked");
}
#[tokio::test]
async fn head_405_falls_back_once_to_small_get_and_restricted_is_not_called_broken() {
    let methods = Arc::new(Mutex::new(Vec::new()));
    let recorded = methods.clone();
    let result = probe_with(
        "https://openai.com/news".into(),
        |_| async { Ok(public()) },
        move |_, method, _| {
            recorded.lock().unwrap().push(method.clone());
            async move {
                Ok(Headers {
                    status: if method == Method::HEAD { 405 } else { 403 },
                    location: None,
                })
            }
        },
    )
    .await;
    assert_eq!(result.status, "restricted");
    assert_eq!(*methods.lock().unwrap(), vec![Method::HEAD, Method::GET]);
}
#[tokio::test]
async fn redirects_and_http_failures_have_bounded_non_destructive_states() {
    let result = probe_with(
        "https://openai.com/".into(),
        |_| async { Ok(public()) },
        |_, _, _| async {
            Ok(Headers {
                status: 302,
                location: Some("/loop".into()),
            })
        },
    )
    .await;
    assert_eq!(result.status, "blocked");
    for (status, expected) in [
        (200, "healthy"),
        (404, "broken"),
        (410, "broken"),
        (429, "rate_limited"),
        (503, "unavailable"),
    ] {
        let r = probe_with(
            "https://openai.com/".into(),
            |_| async { Ok(public()) },
            move |_, _, _| async move {
                Ok(Headers {
                    status,
                    location: None,
                })
            },
        )
        .await;
        assert_eq!(r.status, expected);
    }
}

#[tokio::test]
async fn actual_transport_has_no_credentials_and_stops_at_headers_without_reading_a_body() {
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
    };
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (send, receive) = tokio::sync::oneshot::channel();
    let server = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut request = Vec::new();
        loop {
            let mut byte = [0];
            if socket.read(&mut byte).await.unwrap() == 0 {
                break;
            }
            request.push(byte[0]);
            if request.ends_with(b"\r\n\r\n") {
                break;
            }
            assert!(request.len() < 8192);
        }
        send.send(String::from_utf8(request).unwrap().to_lowercase())
            .unwrap();
        // A body is promised but never sent. Reading the body would hang this probe.
        socket
            .write_all(b"HTTP/1.1 206 Partial Content\r\nContent-Length: 999999999\r\n\r\n")
            .await
            .unwrap();
        std::future::pending::<()>().await;
    });
    // Only the private raw transport test uses localhost. Production probe_with blocks it.
    let response = tokio::time::timeout(
        Duration::from_secs(1),
        headers(
            Url::parse(&format!("http://localhost:{}/test", addr.port())).unwrap(),
            Method::GET,
            vec![addr],
        ),
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(response.status, 206);
    let request = receive.await.unwrap();
    assert!(request.starts_with("get /test http/1.1"));
    assert!(request.contains("range: bytes=0-0"));
    assert!(!request.contains("authorization:"));
    assert!(!request.contains("cookie:"));
    assert!(!request.contains("referer:"));
    server.abort();
}

#[tokio::test]
async fn oversized_response_headers_are_bounded_by_the_http_parser() {
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
    };
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut buf = [0; 8192];
        let _ = socket.read(&mut buf).await;
        let header = format!(
            "HTTP/1.1 200 OK\r\nX-Oversized: {}\r\nContent-Length: 0\r\n\r\n",
            "a".repeat(1024 * 1024)
        );
        let _ = socket.write_all(header.as_bytes()).await;
    });
    let result = tokio::time::timeout(
        Duration::from_secs(2),
        headers(
            Url::parse(&format!("http://localhost:{}/", addr.port())).unwrap(),
            Method::HEAD,
            vec![addr],
        ),
    )
    .await
    .unwrap();
    assert!(matches!(result, Err(Failure::Network)));
    server.abort();
}
