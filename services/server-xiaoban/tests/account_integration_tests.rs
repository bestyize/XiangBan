use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use jsonwebtoken::{encode, Algorithm, DecodingKey, EncodingKey, Header};
use ring::signature::{Ed25519KeyPair, KeyPair};
use serde_json::{json, Value};
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;
use wallpaper_collect::{
    admin::AdminService,
    api::{create_api_router, AppState},
    auth::{AccessClaims, JwksVerifier},
    config::AppConfig,
    db::Database,
    models::{
        Author, DownloadUrls, MediaType, Orientation, Resolution, Stats, WallpaperItem,
    },
    providers::ProviderRegistry,
    runtime::Runtime,
    services::{SearchCache, WallpaperAggregator, WallpaperCollector},
};

fn sample_item(id: &str) -> WallpaperItem {
    WallpaperItem {
        id: id.to_string(),
        provider: "pexels".into(),
        source_id: "s1".into(),
        media_type: MediaType::Image,
        title: "Test Wallpaper".into(),
        description: None,
        tags: vec!["nature".into()],
        resolution: Resolution::new(1920, 1080),
        orientation: Orientation::Landscape,
        dominant_color: None,
        preview_url: "https://example.com/p.jpg".into(),
        regular_url: "https://example.com/r.jpg".into(),
        downloads: DownloadUrls {
            original: "https://example.com/o.jpg".into(),
            large: None,
            medium: None,
            small: None,
            thumb: "https://example.com/p.jpg".into(),
        },
        video: None,
        author: Author {
            name: "Artist".into(),
            username: "artist".into(),
            avatar_url: None,
            profile_url: None,
        },
        stats: Stats::default(),
        source_page_url: "https://example.com/item".into(),
        license: "free".into(),
        status: "published".into(),
        source_type: "collected".into(),
        category: Some("nature".into()),
        local_path: None,
        local_copy: None,
        collected_at: None,
    }
}

struct TestHarness {
    app: axum::Router,
    encoding: EncodingKey,
    issuer: String,
    client_id: String,
}

impl TestHarness {
    async fn new() -> Self {
        let key_pair_der = Ed25519KeyPair::generate_pkcs8(&ring::rand::SystemRandom::new()).unwrap();
        let pair = Ed25519KeyPair::from_pkcs8(key_pair_der.as_ref()).unwrap();
        let pub_x = URL_SAFE_NO_PAD.encode(pair.public_key().as_ref());

        let encoding = EncodingKey::from_ed_der(key_pair_der.as_ref());
        let decoding = DecodingKey::from_ed_components(&pub_x).unwrap();

        let issuer = "https://account.xiangban.xyz".to_string();
        let client_id = "xiangban-wallpaper-web".to_string();

        let cfg = AppConfig {
            server_host: "127.0.0.1".into(),
            server_port: 0,
            database_url: None,
            unsplash_access_key: None,
            pixabay_api_key: None,
            pexels_api_key: None,
            okslop_api_key: None,
            bing_api_key: None,
            enable_demo_data: true,
            cache_ttl_secs: 3600,
            auto_sync_interval_mins: 0,
            account_issuer: issuer.clone(),
            account_jwks_url: None,
            account_allowed_clients: vec![client_id.clone(), "xiaoban-desktop-wallpaper".into()],
            account_public_key: Some(pub_x),
        };

        let verifier = Arc::new(JwksVerifier::with_static_key(
            issuer.clone(),
            vec![client_id.clone(), "xiaoban-desktop-wallpaper".into()],
            decoding,
        ));

        let runtime = Runtime::from_env(&cfg);
        let registry = ProviderRegistry::new(&runtime, true);
        let db = Database::init(None).await;
        let cache = SearchCache::new(None, runtime.clone());

        let state = AppState {
            aggregator: Arc::new(WallpaperAggregator::new(registry.clone(), db.clone(), cache.clone())),
            collector: Arc::new(WallpaperCollector::new(registry.clone(), db.clone(), 0)),
            registry,
            db,
            runtime,
            cache,
            admin: AdminService::bootstrap(None).await.unwrap(),
            verifier,
            config: cfg,
        };

        let app = create_api_router(state);
        Self {
            app,
            encoding,
            issuer,
            client_id,
        }
    }

    fn make_token(&self, uid: Uuid, ttl_secs: i64) -> String {
        let now = chrono::Utc::now().timestamp();
        let claims = AccessClaims {
            iss: self.issuer.clone(),
            sub: uid.to_string(),
            aud: self.client_id.clone(),
            exp: now + ttl_secs,
            iat: now,
            jti: Some("test-jti".into()),
        };
        let mut header = Header::new(Algorithm::EdDSA);
        header.kid = Some("test-kid".into());
        encode(&header, &claims, &self.encoding).unwrap()
    }

    async fn send(&self, req: Request<Body>) -> (StatusCode, Value) {
        let resp = self.app.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
        let body: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
        (status, body)
    }
}

#[tokio::test]
async fn test_auth_config_endpoint() {
    let h = TestHarness::new().await;
    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/auth/config")
        .body(Body::empty())
        .unwrap();
    let (status, body) = h.send(req).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);
    assert_eq!(body["issuer"], "https://account.xiangban.xyz");
    assert_eq!(body["default_client"], "xiangban-wallpaper-web");
}

#[tokio::test]
async fn test_user_me_endpoint() {
    let h = TestHarness::new().await;
    let uid = Uuid::new_v4();
    let token = h.make_token(uid, 3600);

    // 1. 无凭证应返回 401
    let unauth_req = Request::builder()
        .method("GET")
        .uri("/api/v1/user/me")
        .body(Body::empty())
        .unwrap();
    let (status, _) = h.send(unauth_req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // 2. 携带有效凭证返回正确 uid
    let auth_req = Request::builder()
        .method("GET")
        .uri("/api/v1/user/me")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let (status, body) = h.send(auth_req).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);
    assert_eq!(body["user"]["uid"], uid.to_string());
    assert_eq!(body["user"]["client_id"], "xiangban-wallpaper-web");

    // 3. 携带过期凭证返回 401
    let expired_token = h.make_token(uid, -60);
    let expired_req = Request::builder()
        .method("GET")
        .uri("/api/v1/user/me")
        .header(header::AUTHORIZATION, format!("Bearer {expired_token}"))
        .body(Body::empty())
        .unwrap();
    let (status, _) = h.send(expired_req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_user_favorites_isolation_and_status() {
    let h = TestHarness::new().await;
    let user_a = Uuid::new_v4();
    let user_b = Uuid::new_v4();
    let token_a = h.make_token(user_a, 3600);
    let token_b = h.make_token(user_b, 3600);

    let item1 = sample_item("pexels_101");
    let item2 = sample_item("pexels_102");

    // User A 收藏 item1
    let post_req = Request::builder()
        .method("POST")
        .uri("/api/v1/favorites")
        .header(header::AUTHORIZATION, format!("Bearer {token_a}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&item1).unwrap()))
        .unwrap();
    let (status, body) = h.send(post_req).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);
    assert_eq!(body["authenticated"], true);
    assert_eq!(body["user_id"], user_a.to_string());

    // User A 检查单项收藏状态
    let check_a = Request::builder()
        .method("GET")
        .uri("/api/v1/favorites/pexels_101/status")
        .header(header::AUTHORIZATION, format!("Bearer {token_a}"))
        .body(Body::empty())
        .unwrap();
    let (status, body) = h.send(check_a).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["is_favorite"], true);

    // User B 检查单项收藏状态（应当为 false）
    let check_b = Request::builder()
        .method("GET")
        .uri("/api/v1/favorites/pexels_101/status")
        .header(header::AUTHORIZATION, format!("Bearer {token_b}"))
        .body(Body::empty())
        .unwrap();
    let (status, body) = h.send(check_b).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["is_favorite"], false);

    // User B 列表为空
    let list_b = Request::builder()
        .method("GET")
        .uri("/api/v1/favorites")
        .header(header::AUTHORIZATION, format!("Bearer {token_b}"))
        .body(Body::empty())
        .unwrap();
    let (status, body) = h.send(list_b).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total"], 0);

    // User A 批量检查收藏
    let batch_a = Request::builder()
        .method("GET")
        .uri("/api/v1/favorites/check?ids=pexels_101,pexels_102")
        .header(header::AUTHORIZATION, format!("Bearer {token_a}"))
        .body(Body::empty())
        .unwrap();
    let (status, body) = h.send(batch_a).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["favorite_ids"], json!(["pexels_101"]));

    // 游客模式：无 Authorization 头也可使用访客收藏
    let guest_post = Request::builder()
        .method("POST")
        .uri("/api/v1/favorites")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&item2).unwrap()))
        .unwrap();
    let (status, body) = h.send(guest_post).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["authenticated"], false);
    assert!(body["user_id"].is_null());

    // 游客列表包含 item2 但不包含 item1
    let guest_list = Request::builder()
        .method("GET")
        .uri("/api/v1/favorites")
        .body(Body::empty())
        .unwrap();
    let (status, body) = h.send(guest_list).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total"], 1);
    assert_eq!(body["items"][0]["id"], "pexels_102");

    // User A 删除 item1 收藏
    let del_a = Request::builder()
        .method("DELETE")
        .uri("/api/v1/favorites/pexels_101")
        .header(header::AUTHORIZATION, format!("Bearer {token_a}"))
        .body(Body::empty())
        .unwrap();
    let (status, body) = h.send(del_a).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["removed"], true);

    // User A 列表变为空
    let list_a_after = Request::builder()
        .method("GET")
        .uri("/api/v1/favorites")
        .header(header::AUTHORIZATION, format!("Bearer {token_a}"))
        .body(Body::empty())
        .unwrap();
    let (status, body) = h.send(list_a_after).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total"], 0);
}
