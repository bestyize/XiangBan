use super::{
    admin::{audit, AdminIdentity, AuthAdmin},
    open_source_rules as rules,
};
use crate::{
    error::{AppError, AppResult},
    state::AppState,
};
use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, HeaderValue},
    response::Response,
    routing::{get, post},
    Json, Router,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::PgConnection;
use uuid::Uuid;
const LIMIT: usize = 5 * 1024 * 1024;
pub fn admin_routes() -> Router<AppState> {
    Router::new()
        .route("/assets", post(upload))
        .route("/assets/fetch", post(fetch))
        .route("/assets/{id}", get(preview))
}
fn image(data: &[u8]) -> AppResult<&'static str> {
    if data.is_empty() || data.len() > LIMIT {
        return Err(AppError::BadRequest("素材需要在 5 MiB 以内".into()));
    }
    match super::images::sniff(data) {
        Some(mime @ ("image/png" | "image/jpeg" | "image/webp")) => Ok(mime),
        _ => Err(AppError::BadRequest(
            "素材仅接受 PNG、JPEG、WebP；SVG 可在上传时转为 PNG".into(),
        )),
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Upload {
    project: String,
    source_url: String,
    data_base64: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fetch {
    project: String,
    url: String,
}
async fn store(
    state: &AppState,
    admin: &AdminIdentity,
    project: &str,
    source: &str,
    data: Vec<u8>,
) -> AppResult<Value> {
    if !crate::collectors::repo_sources::valid_name(project)
        || !source.is_empty() && !rules::https(source)
    {
        return Err(AppError::BadRequest(
            "素材需要有效项目和公开来源地址".into(),
        ));
    }
    let mime = image(&data)?;
    let id = Uuid::new_v4();
    let size = data.len();
    let mut tx = state.pool.begin().await?;
    sqlx::query("INSERT INTO open_source_assets(id,project,mime,data,source_url,created_by) VALUES($1,$2,$3,$4,$5,$6)").bind(id).bind(project.to_ascii_lowercase()).bind(mime).bind(data).bind(source).bind(&admin.username).execute(&mut *tx).await?;
    audit(
        &mut tx,
        admin,
        "open-source.asset.upload",
        "open-source",
        id,
        format!("project={project},size={size}"),
    )
    .await?;
    tx.commit().await?;
    Ok(
        json!({"id":id,"url":format!("/api/open-source/assets/{id}"),"mime":mime,"size":size,"source_url":source}),
    )
}
async fn upload(
    State(state): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Json(input): Json<Upload>,
) -> AppResult<Json<Value>> {
    if input.data_base64.len() > 7 * 1024 * 1024 {
        return Err(AppError::BadRequest("素材过大".into()));
    }
    let bytes = STANDARD
        .decode(input.data_base64)
        .map_err(|_| AppError::BadRequest("素材编码无效".into()))?;
    Ok(Json(
        store(&state, &admin, &input.project, &input.source_url, bytes).await?,
    ))
}
async fn fetch(
    State(state): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Json(input): Json<Fetch>,
) -> AppResult<Json<Value>> {
    let data = download(&input.url).await?;
    Ok(Json(
        store(&state, &admin, &input.project, &input.url, data).await?,
    ))
}
async fn download(raw: &str) -> AppResult<Vec<u8>> {
    let mut url =
        reqwest::Url::parse(raw).map_err(|_| AppError::BadRequest("图片地址无效".into()))?;
    for _ in 0..4 {
        if !rules::https(url.as_str()) {
            return Err(AppError::BadRequest("只采集公开 HTTPS 图片".into()));
        }
        let host = url.host_str().unwrap().to_string();
        let addresses: Vec<_> = tokio::net::lookup_host((host.as_str(), 443))
            .await
            .map_err(|_| AppError::BadRequest("素材主机无法解析".into()))?
            .collect();
        if addresses.is_empty()
            || addresses
                .iter()
                .any(|a| !crate::link_health::network::public_ip(a.ip()))
        {
            return Err(AppError::BadRequest("素材地址不是公开网络地址".into()));
        }
        let client = reqwest::Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .resolve_to_addrs(&host, &addresses)
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(|_| AppError::Internal("素材客户端初始化失败".into()))?;
        let mut response = client
            .get(url.clone())
            .header("User-Agent", "Xiaoban-Curated-Covers/1")
            .send()
            .await
            .map_err(|_| AppError::BadRequest("素材下载失败，可改为本地上传".into()))?;
        if response.status().is_redirection() {
            let target = response
                .headers()
                .get(header::LOCATION)
                .and_then(|h| h.to_str().ok())
                .ok_or_else(|| AppError::BadRequest("素材重定向无效".into()))?;
            url = url
                .join(target)
                .map_err(|_| AppError::BadRequest("素材重定向无效".into()))?;
            continue;
        }
        if !response.status().is_success() {
            return Err(AppError::BadRequest(format!(
                "素材来源返回 HTTP {}",
                response.status().as_u16()
            )));
        }
        if response.content_length().is_some_and(|n| n > LIMIT as u64) {
            return Err(AppError::BadRequest("素材超过 5 MiB".into()));
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|_| AppError::BadRequest("素材读取失败".into()))?
        {
            if bytes.len() + chunk.len() > LIMIT {
                return Err(AppError::BadRequest("素材超过 5 MiB".into()));
            }
            bytes.extend_from_slice(&chunk);
        }
        image(&bytes)?;
        return Ok(bytes);
    }
    Err(AppError::BadRequest("素材重定向次数过多".into()))
}
fn response(mime: String, bytes: Vec<u8>, public: bool) -> Response {
    let mut response = Response::new(Body::from(bytes));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&mime)
            .unwrap_or(HeaderValue::from_static("application/octet-stream")),
    );
    response.headers_mut().insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static(if public {
            "public,max-age=300"
        } else {
            "private,no-store"
        }),
    );
    response
}
pub async fn public(State(s): State<AppState>, Path(id): Path<Uuid>) -> AppResult<Response> {
    let url = format!("/api/open-source/assets/{id}");
    let row:Option<(String,Vec<u8>)>=sqlx::query_as("SELECT mime,data FROM open_source_assets a WHERE a.id=$1 AND EXISTS(SELECT 1 FROM open_source_catalog c,jsonb_array_elements(c.published->'projects') p WHERE c.singleton AND (p->>'icon_url'=$2 OR p->>'cover_url'=$2))").bind(id).bind(url).fetch_optional(&s.pool).await?;
    let (mime, data) = row.ok_or_else(|| AppError::not_found("已发布素材"))?;
    Ok(response(mime, data, true))
}
async fn preview(
    State(s): State<AppState>,
    _: AuthAdmin,
    Path(id): Path<Uuid>,
) -> AppResult<Response> {
    let row: Option<(String, Vec<u8>)> =
        sqlx::query_as("SELECT mime,data FROM open_source_assets WHERE id=$1")
            .bind(id)
            .fetch_optional(&s.pool)
            .await?;
    let (mime, data) = row.ok_or_else(|| AppError::not_found("素材"))?;
    Ok(response(mime, data, false))
}
pub async fn validate_refs(conn: &mut PgConnection, catalog: &Value) -> AppResult<()> {
    for project in catalog["projects"].as_array().unwrap() {
        for key in ["cover_url", "icon_url"] {
            if let Some(raw) = project[key]
                .as_str()
                .and_then(|v| v.strip_prefix("/api/open-source/assets/"))
            {
                let id = Uuid::parse_str(raw)
                    .map_err(|_| AppError::BadRequest("素材标识无效".into()))?;
                let (valid,): (bool,) = sqlx::query_as(
                    "SELECT EXISTS(SELECT 1 FROM open_source_assets WHERE id=$1 AND project=$2)",
                )
                .bind(id)
                .bind(project["full_name"].as_str().unwrap().to_ascii_lowercase())
                .fetch_one(&mut *conn)
                .await?;
                if !valid {
                    return Err(AppError::BadRequest(format!(
                        "{} 引用了缺失或属于其他项目的素材",
                        project["full_name"].as_str().unwrap()
                    )));
                }
            }
        }
    }
    Ok(())
}
