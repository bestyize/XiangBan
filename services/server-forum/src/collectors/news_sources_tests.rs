use super::*;
use axum::{http::StatusCode, response::IntoResponse, routing::get, Router};

fn rss(items: &str) -> Vec<u8> {
    format!("<?xml version=\"1.0\"?><rss version=\"2.0\"><channel><title>Official</title>{items}</channel></rss>").into_bytes()
}
fn item(title: &str, url: &str, date: &str) -> String {
    format!("<item><title>{title}</title><link>{url}</link>{date}<description>Private full text deliberately ignored.</description></item>")
}
#[test]
fn config_is_only_unique_known_ids_and_never_urls() {
    assert_eq!(sanitize_config(json!({})).unwrap(), json!({"feed_ids":[]}));
    assert_eq!(
        sanitize_config(json!({"feed_ids":["google_deepmind","openai_news"]})).unwrap(),
        json!({"feed_ids":["google_deepmind","openai_news"]})
    );
    for bad in [
        Value::Null,
        json!([]),
        json!({"url":"http://127.0.0.1"}),
        json!({"feed_ids":["openai_news","openai_news"]}),
        json!({"feed_ids":["https://openai.com/news/rss.xml"]}),
        json!({"feed_ids":[5]}),
        json!({"feed_ids":"openai_news"}),
        json!({"feed_ids":[],"other":true}),
    ] {
        assert!(sanitize_config(bad).is_err());
    }
}
#[test]
fn rss_decodes_entities_and_preserves_only_news_metadata() {
    let xml = rss(&item(
        "AI &amp; Research <b>launch</b>",
        "https://openai.com/index/launch/?utm_source=rss&amp;a=1#part",
        "<pubDate>Fri, 30 Oct 2026 09:30:00 GMT</pubDate>",
    ));
    let items = parse_feed(FEEDS[0], &xml).unwrap();
    assert_eq!(items[0].data["title"], "AI & Research launch");
    assert_eq!(items[0].source_url, "https://openai.com/index/launch?a=1");
    assert_eq!(
        items[0].published_at.unwrap().to_rfc3339(),
        "2026-10-30T09:30:00+00:00"
    );
    assert_eq!(items[0].data["time_basis"], "source_published");
    assert_eq!(items[0].key.len(), 53);
    assert!(!items[0].data.to_string().contains("Private full text"));
    assert!(!items[0].data.to_string().contains("description"));
}
#[test]
fn atom_supports_xhtml_titles_and_uses_published_instead_of_updated() {
    let xml=br#"<feed xmlns="http://www.w3.org/2005/Atom"><entry><title type="xhtml"><div xmlns="http://www.w3.org/1999/xhtml">Models <b>arrive</b></div></title><link rel="self" href="https://openai.com/data.atom"/><link rel="alternate" type="text/html" href="https://openai.com/index/atom"/><updated>2026-10-30T03:00:00Z</updated></entry></feed>"#;
    let items = parse_feed(FEEDS[0], xml).unwrap();
    assert_eq!(items[0].data["title"], "Models arrive");
    assert!(items[0].published_at.is_none());
    assert_eq!(items[0].data["source_published_at"], Value::Null);
    assert_eq!(items[0].data["time_basis"], "collected");
}
#[test]
fn malformed_dates_remain_unknown_not_fake_publication_time() {
    for date in ["", "<pubDate>not a date</pubDate>"] {
        let rows = parse_feed(
            FEEDS[0],
            &rss(&item("Unknown time", "https://openai.com/news/time", date)),
        )
        .unwrap();
        assert!(rows[0].published_at.is_none());
        assert_eq!(rows[0].data["source_published_at"], Value::Null);
    }
}
#[test]
fn no_xxe_dtd_deep_nesting_big_documents_or_invalid_roots() {
    let xxe=br#"<!DOCTYPE rss [<!ENTITY secret SYSTEM "file:///etc/passwd">]><rss><channel><item><title>&secret;</title><link>https://openai.com/index/x</link></item></channel></rss>"#;
    assert!(parse_feed(FEEDS[0], xxe).is_err());
    let bomb=br#"<!DOCTYPE rss [<!ENTITY a "123"><!ENTITY b "&a;&a;">]><rss><channel><item><title>&b;</title><link>https://openai.com/index/x</link></item></channel></rss>"#;
    assert!(parse_feed(FEEDS[0], bomb).is_err());
    let nested = format!(
        "<rss><channel>{}x{}</channel></rss>",
        "<a>".repeat(60),
        "</a>".repeat(60)
    );
    assert!(parse_feed(FEEDS[0], nested.as_bytes()).is_err());
    let nodes = format!(
        "<rss><channel>{}</channel></rss>",
        "<a/>".repeat(MAX_NODES as usize)
    );
    assert!(parse_feed(FEEDS[0], nodes.as_bytes()).is_err());
    assert!(parse_feed(FEEDS[0], &vec![b'x'; MAX_BODY + 1]).is_err());
    for raw in [
        b"<html>not RSS</html>".as_slice(),
        b"<rss><channel/></rss>",
        b"<rss><channel>",
    ] {
        assert!(parse_feed(FEEDS[0], raw).is_err());
    }
}
#[test]
fn foreign_or_unsafe_links_cannot_enter_candidates() {
    for url in [
        "https://openai.com.evil.test/index/x",
        "https://example.com/x",
        "http://openai.com/index/x",
        "https://user@openai.com/index/x",
        "https://openai.com:444/index/x",
        "https://127.0.0.1/index/x",
        "javascript:alert(1)",
    ] {
        assert!(parse_feed(FEEDS[0], &rss(&item("Title", url, ""))).is_err());
    }
}
#[test]
fn canonical_dedup_stable_hash_and_top_ten_selection() {
    let mut xml = String::new();
    for day in 1..=15 {
        xml += &item(
            &format!("Day {day}"),
            &format!("https://openai.com/index/day-{day}"),
            &format!("<pubDate>2026-10-{day:02}T00:00:00Z</pubDate>"),
        );
    }
    xml += &item(
        "Day 15 duplicate",
        "https://openai.com/index/day-15/?utm_campaign=rss#title",
        "<pubDate>2026-10-15T00:00:00Z</pubDate>",
    );
    let chosen = select_candidates(parse_feed(FEEDS[0], &rss(&xml)).unwrap());
    assert_eq!(chosen.len(), 10);
    assert_eq!(chosen[0].data["title"], "Day 15");
    assert_eq!(chosen[9].data["title"], "Day 6");
    let a = parse_feed(
        FEEDS[0],
        &rss(&item("New title", "https://openai.com/index/day-15/", "")),
    )
    .unwrap();
    assert_eq!(chosen[0].key, a[0].key);
}

async fn serve(app: Router) -> (String, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (format!("http://{addr}"), server)
}
fn client() -> Client {
    Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
}
#[tokio::test]
async fn network_failures_are_fixed_categories_without_echoing_body() {
    for (status, label) in [
        (StatusCode::TOO_MANY_REQUESTS, "429"),
        (StatusCode::BAD_GATEWAY, "5xx"),
        (StatusCode::UNAUTHORIZED, "非成功"),
    ] {
        let app = Router::new().route(
            "/feed",
            get(move || async move { (status, "NEVER_ECHO_UPSTREAM_BODY") }),
        );
        let (base, server) = serve(app).await;
        let err = fetch_xml(&client(), FEEDS[0], &format!("{base}/feed"), TIMEOUT)
            .await
            .unwrap_err();
        assert!(err.contains(label));
        assert!(err.contains("OpenAI"));
        assert!(!err.contains("NEVER_ECHO"));
        server.abort();
    }
    let app = Router::new().route(
        "/feed",
        get(|| async {
            tokio::time::sleep(Duration::from_millis(100)).await;
            "late"
        }),
    );
    let (base, server) = serve(app).await;
    assert!(fetch_xml(
        &client(),
        FEEDS[0],
        &format!("{base}/feed"),
        Duration::from_millis(10)
    )
    .await
    .unwrap_err()
    .contains("超时"));
    server.abort();
}
#[tokio::test]
async fn redirect_is_not_followed_and_response_body_is_bounded_after_gzip() {
    let app = Router::new()
        .route(
            "/redirect",
            get(|| async {
                (
                    StatusCode::FOUND,
                    [("location", "http://127.0.0.1:9/private")],
                    "redirect",
                )
            }),
        )
        .route("/large", get(|| async { "x".repeat(MAX_BODY + 1) }))
        .route(
            "/gzip",
            get(|| async { "x".repeat(MAX_BODY + 1).into_response() })
                .layer(tower_http::compression::CompressionLayer::new()),
        );
    let (base, server) = serve(app).await;
    for endpoint in ["/redirect", "/large", "/gzip"] {
        let err = fetch_xml(&client(), FEEDS[1], &format!("{base}{endpoint}"), TIMEOUT)
            .await
            .unwrap_err();
        assert!(
            err.contains(if endpoint == "/redirect" {
                "重定向"
            } else {
                "2 MiB"
            }),
            "{err}"
        );
    }
    server.abort();
}

#[tokio::test]
#[ignore = "requires access to the official public RSS endpoints; run explicitly"]
async fn live_official_feeds_produce_metadata_without_publishing() {
    let http = client();
    for feed in FEEDS {
        let items = collect(&http, &json!({"feed_ids":[feed.id]}))
            .await
            .unwrap();
        assert!(!items.is_empty() && items.len() <= MAX_ITEMS);
        assert!(items
            .iter()
            .all(|i| i.source_name == feed.title && i.kind == "news"));
    }
}
