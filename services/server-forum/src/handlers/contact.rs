use super::{
    admin::{audit, AuthAdmin},
    images::sniff,
};
use crate::{
    error::{AppError, AppResult},
    state::AppState,
};
use axum::{
    body::{Body, Bytes},
    extract::{DefaultBodyLimit, Multipart, Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use chrono::{NaiveDate, Utc};
use chrono_tz::Asia::Shanghai;
use serde::{Deserialize, Serialize};
use sqlx::{types::Json as DbJson, PgPool};
use std::collections::BTreeMap;

const MAX_QR_BYTES: usize = 5 * 1024 * 1024;
const MAX_REQUEST_BYTES: usize = 3 * MAX_QR_BYTES + 16 * 1024;

#[derive(Clone, Copy, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "kebab-case")]
enum Channel {
    WechatOfficial,
    QqGroup,
    WechatGroup,
}

impl Channel {
    fn key(self) -> &'static str {
        match self {
            Self::WechatOfficial => "wechat-official",
            Self::QqGroup => "qq-group",
            Self::WechatGroup => "wechat-group",
        }
    }

    fn from_key(field: &str) -> AppResult<Self> {
        match field {
            "wechat-official" => Ok(Self::WechatOfficial),
            "qq-group" => Ok(Self::QqGroup),
            "wechat-group" => Ok(Self::WechatGroup),
            _ => Err(AppError::BadRequest("联系方式字段无效".into())),
        }
    }

    fn default_image(self) -> &'static [u8] {
        match self {
            Self::WechatOfficial => include_bytes!("../../assets/contact/wechat-official.webp"),
            Self::QqGroup => include_bytes!("../../assets/contact/qq-group.webp"),
            Self::WechatGroup => include_bytes!("../../assets/contact/wechat-group.webp"),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ContactSettings {
    wechat_official_name: String,
    wechat_official_qr: bool,
    qq_group_name: String,
    qq_group_number: String,
    qq_group_invite_url: String,
    qq_group_qr: bool,
    wechat_group_name: String,
    wechat_group_qr: bool,
    wechat_group_expires_on: Option<NaiveDate>,
}

impl Default for ContactSettings {
    fn default() -> Self {
        Self {
            wechat_official_name: "从来不想".into(),
            wechat_official_qr: true,
            qq_group_name: "小伴产品交流群".into(),
            qq_group_number: "1136181862".into(),
            qq_group_invite_url: String::new(),
            qq_group_qr: true,
            wechat_group_name: "从来不想".into(),
            wechat_group_qr: true,
            wechat_group_expires_on: NaiveDate::from_ymd_opt(2026, 11, 21),
        }
    }
}

impl ContactSettings {
    fn qr(&self, channel: Channel) -> bool {
        match channel {
            Channel::WechatOfficial => self.wechat_official_qr,
            Channel::QqGroup => self.qq_group_qr,
            Channel::WechatGroup => self.wechat_group_qr,
        }
    }

    fn hide_expired(&mut self, today: NaiveDate) {
        if self
            .wechat_group_expires_on
            .is_some_and(|date| date <= today)
        {
            self.wechat_group_qr = false;
        }
    }

    fn normalize(&mut self) -> AppResult<()> {
        for name in [
            &mut self.wechat_official_name,
            &mut self.qq_group_name,
            &mut self.wechat_group_name,
        ] {
            *name = name.trim().into();
            if name.chars().count() > 100 {
                return Err(AppError::BadRequest("公众号和群名称最多 100 个字符".into()));
            }
        }
        self.qq_group_number = self.qq_group_number.trim().into();
        if self.qq_group_number.len() > 32
            || !self.qq_group_number.bytes().all(|b| b.is_ascii_digit())
        {
            return Err(AppError::BadRequest("QQ 群号最多填写 32 位数字".into()));
        }
        self.qq_group_invite_url = self.qq_group_invite_url.trim().into();
        if !self.qq_group_invite_url.is_empty() {
            let invalid = || AppError::BadRequest("请填写有效的 HTTPS 加群链接".into());
            let url = reqwest::Url::parse(&self.qq_group_invite_url).map_err(|_| invalid())?;
            if url.scheme() != "https"
                || url.host_str().is_none()
                || !url.username().is_empty()
                || url.password().is_some()
                || url.as_str().len() > 2048
            {
                return Err(invalid());
            }
            self.qq_group_invite_url = url.into();
        }
        if !self.wechat_group_qr {
            self.wechat_group_expires_on = None;
        }
        Ok(())
    }
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/contact", get(public_settings))
        .route("/contact/qr/{channel}", get(public_qr))
}

pub fn admin_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/contact",
            get(admin_settings)
                .put(save)
                .layer(DefaultBodyLimit::max(MAX_REQUEST_BYTES)),
        )
        .route("/contact/qr/{channel}", get(admin_qr))
}

async fn load(pool: &PgPool) -> AppResult<ContactSettings> {
    Ok(sqlx::query_scalar::<_, DbJson<ContactSettings>>(
        "SELECT config FROM contact_settings WHERE id",
    )
    .fetch_optional(pool)
    .await?
    .map(|config| config.0)
    .unwrap_or_default())
}

fn today() -> NaiveDate {
    Utc::now().with_timezone(&Shanghai).date_naive()
}

async fn public_settings(State(state): State<AppState>) -> AppResult<impl IntoResponse> {
    let mut config = load(&state.pool).await?;
    config.hide_expired(today());
    Ok(([(header::CACHE_CONTROL, "no-store")], Json(config)))
}

async fn public_qr(
    State(state): State<AppState>,
    Path(channel): Path<Channel>,
) -> AppResult<Response> {
    qr_response(&state.pool, channel, true).await
}

async fn admin_qr(
    State(state): State<AppState>,
    AuthAdmin(_): AuthAdmin,
    Path(channel): Path<Channel>,
) -> AppResult<Response> {
    qr_response(&state.pool, channel, false).await
}

async fn qr_response(pool: &PgPool, channel: Channel, public: bool) -> AppResult<Response> {
    // Read the settings and the image from the same database snapshot.
    let row: (
        Option<DbJson<ContactSettings>>,
        Option<String>,
        Option<Vec<u8>>,
    ) = sqlx::query_as(
        "SELECT s.config, q.mime, q.data FROM (SELECT true AS id) seed
         LEFT JOIN contact_settings s ON s.id = seed.id
         LEFT JOIN contact_qr_images q ON q.channel = $1",
    )
    .bind(channel.key())
    .fetch_one(pool)
    .await?;
    let (config, mime, data) = row;
    let mut config = config.map(|value| value.0).unwrap_or_default();
    if public {
        config.hide_expired(today());
    }
    if !config.qr(channel) {
        return Ok(([(header::CACHE_CONTROL, "no-store")], StatusCode::NOT_FOUND).into_response());
    }
    let (mime, data) = match (mime, data) {
        (Some(mime), Some(data)) => (mime, data),
        _ => ("image/webp".into(), channel.default_image().to_vec()),
    };
    // The resource host can detect an updated QR code with a HEAD request,
    // and fetch the bytes only when the content actually changes.
    let digest = ring::digest::digest(&ring::digest::SHA256, &data)
        .as_ref().iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    Ok((
        [
            (header::CACHE_CONTROL, "no-store"),
            (header::CONTENT_TYPE, mime.as_str()),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
            (header::HeaderName::from_static("x-resource-sha256"), digest.as_str()),
        ],
        Body::from(data),
    )
        .into_response())
}

async fn admin_settings(
    State(state): State<AppState>,
    AuthAdmin(_): AuthAdmin,
) -> AppResult<impl IntoResponse> {
    Ok((
        [(header::CACHE_CONTROL, "no-store")],
        Json(load(&state.pool).await?),
    ))
}

async fn save(
    State(state): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    mut form: Multipart,
) -> AppResult<impl IntoResponse> {
    let mut config: Option<ContactSettings> = None;
    let mut images = BTreeMap::<Channel, (&str, Bytes)>::new();
    let bad_form = || AppError::BadRequest("无法读取联系方式表单".into());
    while let Some(field) = form.next_field().await.map_err(|_| bad_form())? {
        let name = field.name().ok_or_else(bad_form)?.to_owned();
        let bytes = field.bytes().await.map_err(|_| bad_form())?;
        if name == "config" && config.is_none() {
            config = Some(serde_json::from_slice(&bytes).map_err(|_| bad_form())?);
            continue;
        }
        let channel = Channel::from_key(&name)?;
        if bytes.len() > MAX_QR_BYTES {
            return Err(AppError::BadRequest("每张二维码最多 5 MiB".into()));
        }
        let mime = match sniff(&bytes) {
            Some(mime @ ("image/png" | "image/jpeg" | "image/webp")) => mime,
            _ => {
                return Err(AppError::BadRequest(
                    "二维码仅接受 PNG、JPG 或 WebP 图片".into(),
                ))
            }
        };
        if images.insert(channel, (mime, bytes)).is_some() {
            return Err(bad_form());
        }
    }
    let mut config = config.ok_or_else(bad_form)?;
    config.normalize()?;
    let mut tx = state.pool.begin().await?;
    sqlx::query("INSERT INTO contact_settings (id, config) VALUES (true, $1) ON CONFLICT (id) DO UPDATE SET config = EXCLUDED.config")
        .bind(DbJson(&config)).execute(&mut *tx).await?;
    for (channel, (mime, bytes)) in images {
        if config.qr(channel) {
            sqlx::query("INSERT INTO contact_qr_images (channel, mime, data) VALUES ($1, $2, $3)
                         ON CONFLICT (channel) DO UPDATE SET mime = EXCLUDED.mime, data = EXCLUDED.data")
                .bind(channel.key()).bind(mime).bind(bytes.as_ref()).execute(&mut *tx).await?;
        }
    }
    audit(&mut tx, &admin, "contact.update", "contact", "site", "").await?;
    tx.commit().await?;
    Ok(([(header::CACHE_CONTROL, "no-store")], Json(config)))
}
