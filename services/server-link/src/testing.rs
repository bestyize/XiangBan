//! 测试支持：用固定密钥签发令牌、独立 schema 的真实 Postgres、监听本机随机端口的真实服务。
//! 只在测试或开启 `testing` 特性时编译，供其他 crate（如桌面链接层）做端到端集成测试。
use crate::{auth::JwksVerifier, build_router, config::Config, db, hub::Hub, state::AppState};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD as B64, Engine};
use jsonwebtoken::{encode, Algorithm, DecodingKey, EncodingKey, Header};
use ring::signature::{Ed25519KeyPair, KeyPair};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

pub const DESKTOP_CLIENT: &str = "xiaoban-desktop-wallpaper";
pub const ANDROID_CLIENT: &str = "xiaoban-android";

pub struct TestServer {
    /// 例如 `http://127.0.0.1:41234/api/link`
    pub base: String,
    /// 例如 `ws://127.0.0.1:41234/api/link`
    pub ws_base: String,
    pub pool: PgPool,
    enc: EncodingKey,
}

impl TestServer {
    /// 需要一个可用的 Postgres 地址；每次启动用独立 schema。
    pub async fn start(database_url: &str) -> Self {
        Self::start_with(database_url, |_| {}).await
    }

    pub async fn start_with(database_url: &str, tweak: impl FnOnce(&mut Config)) -> Self {
        let schema = format!("link_test_{}", Uuid::new_v4().simple());
        let admin = db::connect(database_url, None).await.unwrap();
        sqlx::query(&format!("CREATE SCHEMA {schema}")).execute(&admin).await.unwrap();
        let pool = db::connect(database_url, Some(&schema)).await.unwrap();
        db::migrate(&pool).await.unwrap();
        db::migrate(&pool).await.unwrap();

        let mut config = Config::from_env();
        config.client_ids = vec![DESKTOP_CLIENT.into(), ANDROID_CLIENT.into()];
        tweak(&mut config);
        let pkcs8 = Ed25519KeyPair::generate_pkcs8(&ring::rand::SystemRandom::new()).unwrap();
        let pair = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).unwrap();
        let x = B64.encode(pair.public_key().as_ref());
        let verifier = JwksVerifier::with_static_key("iss".into(), config.client_ids.clone(), DecodingKey::from_ed_components(&x).unwrap());
        let state = AppState {
            pool: pool.clone(),
            hub: Arc::new(Hub::new(config.notify_queue_cap, config.notify_ttl)),
            verifier: Arc::new(verifier),
            config: Arc::new(config),
        };
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(listener, build_router(state)).await.unwrap() });
        Self { base: format!("http://{addr}/api/link"), ws_base: format!("ws://{addr}/api/link"), pool, enc: EncodingKey::from_ed_der(pkcs8.as_ref()) }
    }

    pub fn token_aud(&self, uid: Uuid, aud: &str, ttl_secs: i64) -> String {
        #[derive(serde::Serialize)]
        struct C {
            iss: String,
            sub: String,
            aud: String,
            exp: i64,
        }
        let c = C { iss: "iss".into(), sub: uid.to_string(), aud: aud.into(), exp: chrono::Utc::now().timestamp() + ttl_secs };
        encode(&Header::new(Algorithm::EdDSA), &c, &self.enc).unwrap()
    }

    /// 手机端的令牌（受众 xiaoban-android）。
    pub fn phone_token(&self, uid: Uuid) -> String {
        self.token_aud(uid, ANDROID_CLIENT, 600)
    }

    /// 桌面端的令牌。
    pub fn desktop_token(&self, uid: Uuid) -> String {
        self.token_aud(uid, DESKTOP_CLIENT, 600)
    }
}
