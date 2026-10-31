//! Manual tool mirrors are transport metadata, never an alternate content publishing channel.
//! Registry rows become public only when the exact asset ID/hash/size is in the approved live tool.
use crate::{
    collectors::tool_sources,
    error::{AppError, AppResult},
    handlers::{
        admin::{audit, AuthAdmin},
        discover_editor,
    },
    state::AppState,
};
use axum::{
    extract::{Path, State},
    response::Redirect,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use reqwest::{Client, Url};
use ring::hmac;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::PgPool;
use std::time::Duration;
const REQUEST_DOMAIN: &[u8] = b"POST\n/internal/tool-mirrors\n";
const RECEIPT_DOMAIN: &[u8] = b"TOOL-MIRROR-RECEIPT\n";
const MAX_ASSET: u64 = 128 * 1024 * 1024;
#[derive(Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
struct Claims {
    nonce: String,
    repository: String,
    version: String,
    filename: String,
    repo_id: u64,
    asset_id: u64,
    source_url: String,
    size: u64,
    sha256: String,
    exp: i64,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    nonce: String,
    repo_id: u64,
    asset_id: u64,
    filename: String,
    sha256: String,
    size: u64,
    file_key: String,
    readback_verified: bool,
    verified_at: i64,
}
#[derive(Clone)]
struct Candidate {
    claims: Claims,
    revision: i64,
}
fn signature(domain: &[u8], raw: &[u8], secret: &[u8]) -> String {
    let mut msg = domain.to_vec();
    msg.extend_from_slice(raw);
    hmac::sign(&hmac::Key::new(hmac::HMAC_SHA256, secret), &msg)
        .as_ref()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
fn verify(domain: &[u8], raw: &[u8], sig: &str, secret: &[u8]) -> bool {
    if sig.len() != 64 || !sig.bytes().all(|b| b.is_ascii_hexdigit()) {
        return false;
    }
    let bytes = (0..32)
        .map(|i| u8::from_str_radix(&sig[i * 2..i * 2 + 2], 16).unwrap())
        .collect::<Vec<_>>();
    let mut msg = domain.to_vec();
    msg.extend_from_slice(raw);
    hmac::verify(&hmac::Key::new(hmac::HMAC_SHA256, secret), &msg, &bytes).is_ok()
}
fn unavailable() -> AppError {
    AppError::ServiceUnavailable("工具镜像资源服务尚未配置或暂时不可用".into())
}
fn invalid() -> AppError {
    AppError::BadRequest("工具安装包或镜像回执未通过校验".into())
}
fn base_url(raw: &str) -> AppResult<Url> {
    let url = Url::parse(raw).map_err(|_| unavailable())?;
    if !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || !(url.scheme() == "https"
            || (url.scheme() == "http"
                && matches!(url.host_str(), Some("127.0.0.1" | "localhost" | "[::1]"))))
    {
        return Err(unavailable());
    }
    Ok(url)
}
fn endpoint(base: &str, path: &str) -> AppResult<Url> {
    let mut base = base_url(base)?;
    let prefix = base.path().trim_end_matches('/').to_string();
    base.set_path(&format!("{prefix}/{path}"));
    Ok(base)
}
fn candidate(data: Value, revision: i64, asset_id: u64) -> AppResult<Candidate> {
    let data = tool_sources::validate_tool(data)?;
    let asset = data["artifacts"]
        .as_array()
        .ok_or_else(invalid)?
        .iter()
        .find(|a| a["asset_id"].as_u64() == Some(asset_id))
        .ok_or_else(|| AppError::not_found("工具安装包"))?;
    Ok(Candidate {
        revision,
        claims: Claims {
            nonce: uuid::Uuid::new_v4().to_string(),
            repository: data["github_full_name"]
                .as_str()
                .ok_or_else(invalid)?
                .into(),
            version: data["releases"][0]["version"]
                .as_str()
                .ok_or_else(invalid)?
                .into(),
            filename: asset["filename"].as_str().ok_or_else(invalid)?.into(),
            repo_id: data["github_repo_id"].as_u64().ok_or_else(invalid)?,
            asset_id,
            source_url: asset["github_url"].as_str().ok_or_else(invalid)?.into(),
            size: asset["size"].as_u64().ok_or_else(invalid)?,
            sha256: asset["sha256"].as_str().ok_or_else(invalid)?.into(),
            exp: Utc::now().timestamp() + 240,
        },
    })
}
fn receipt_valid(r: &Receipt, c: &Claims, now: i64) -> bool {
    r.nonce == c.nonce
        && r.repo_id == c.repo_id
        && r.asset_id == c.asset_id
        && r.filename == c.filename
        && r.sha256 == c.sha256
        && r.size == c.size
        && r.file_key == format!("tools/{}.bin", c.sha256)
        && r.readback_verified
        && r.verified_at >= now - 300
        && r.verified_at <= now + 30
}
async fn request_mirror(
    http: &Client,
    base: &str,
    c: &Claims,
    secret: &[u8],
) -> AppResult<Receipt> {
    let target = endpoint(base, "internal/tool-mirrors")?;
    let raw = serde_json::to_vec(c).map_err(|_| invalid())?;
    let mut response = http
        .post(target.clone())
        .header("content-type", "application/json")
        .header("x-tool-signature", signature(REQUEST_DOMAIN, &raw, secret))
        .body(raw)
        .timeout(Duration::from_secs(195))
        .send()
        .await
        .map_err(|_| unavailable())?;
    if response.url() != &target {
        return Err(unavailable());
    }
    if !response.status().is_success() {
        return Err(AppError::ServiceUnavailable(
            match response.status().as_u16() {
                507 => "资源机工具镜像配额已满（1 GiB / 128 对象），请人工维护",
                503 => "资源机已有镜像写入或磁盘暂不可用，请稍后重试",
                422 => "官方安装包或镜像回读大小 / SHA-256 不匹配，本次结果未登记",
                403 => "资源机拒绝镜像签名，请核对双方资源签名密钥",
                504 => "工具镜像执行超过三分钟，本次结果未登记",
                _ => "工具镜像资源服务暂时不可用，本次结果未登记",
            }
            .into(),
        ));
    }
    let sig = response
        .headers()
        .get("x-tool-receipt-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(invalid)?
        .to_string();
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|_| unavailable())? {
        if bytes.len().saturating_add(chunk.len()) > 8192 {
            return Err(invalid());
        }
        bytes.extend_from_slice(&chunk)
    }
    if !verify(RECEIPT_DOMAIN, &bytes, &sig, secret) {
        return Err(invalid());
    }
    let r: Receipt = serde_json::from_slice(&bytes).map_err(|_| invalid())?;
    if !receipt_valid(&r, c, Utc::now().timestamp()) {
        return Err(invalid());
    }
    Ok(r)
}
pub async fn init(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("CREATE TABLE IF NOT EXISTS tool_mirror_registry(tool_key TEXT NOT NULL,asset_id BIGINT NOT NULL,sha256 TEXT NOT NULL CHECK(sha256~'^[a-f0-9]{64}$'),size BIGINT NOT NULL CHECK(size>0 AND size<=134217728),filename TEXT NOT NULL,file_key TEXT NOT NULL,source_url TEXT NOT NULL,verified_at TIMESTAMPTZ NOT NULL,PRIMARY KEY(tool_key,asset_id,sha256))").execute(pool).await?;
    Ok(())
}
pub fn admin_routes() -> Router<AppState> {
    Router::new()
        .route("/tool-mirrors", get(list))
        .route("/tool-mirrors/{key}/{asset_id}", post(create))
}
pub fn public_routes() -> Router<AppState> {
    Router::new().route(
        "/discover/tool-mirrors/{key}/{asset_id}/{sha}",
        get(download),
    )
}
async fn list(State(s): State<AppState>, _: AuthAdmin) -> AppResult<Json<Value>> {
    let rows:Vec<(String,i64,String,Value)>=sqlx::query_as("SELECT key,revision,status,data FROM discover_drafts WHERE kind='tool' AND data ? 'github_repo_id' ORDER BY key LIMIT 50").fetch_all(&s.pool).await?;
    let mut items = Vec::new();
    let mut invalid_items = 0;
    for (key, revision, status, data) in rows {
        let data = match tool_sources::validate_tool(data) {
            Ok(data) => data,
            Err(_) => {
                invalid_items += 1;
                continue;
            }
        };
        let records: Vec<(i64, String, DateTime<Utc>, i64)> = sqlx::query_as(
            "SELECT asset_id,sha256,verified_at,size FROM tool_mirror_registry WHERE tool_key=$1",
        )
        .bind(&key)
        .fetch_all(&s.pool)
        .await?;
        let artifacts=data["artifacts"].as_array().unwrap_or(&Vec::new()).iter().map(|a|{let verified=records.iter().find(|r|a["asset_id"].as_i64()==Some(r.0)&&a["sha256"].as_str()==Some(&r.1)&&a["size"].as_i64()==Some(r.3));json!({"asset_id":a["asset_id"],"filename":a["filename"],"os":a["os"],"arch":a["arch"],"size":a["size"],"sha256":a["sha256"],"github_url":a["github_url"],"verified_at":verified.map(|r|r.2)})}).collect::<Vec<_>>();
        items.push(json!({"key":key,"revision":revision,"status":status,"name":data["name"],"version":data["releases"][0]["version"],"artifacts":artifacts}));
    }
    Ok(Json(
        json!({"items":items,"invalid_items":invalid_items,"resource_configured":s.config.resource_base_url.is_some(),"max_asset_bytes":MAX_ASSET,"max_storage_bytes":1024_u64*1024*1024,"max_storage_objects":128}),
    ))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    expected_revision: i64,
}
async fn create(
    State(s): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Path((key, asset_id)): Path<(String, u64)>,
    Json(i): Json<Input>,
) -> AppResult<Json<Value>> {
    if !crate::discover::valid_key(&key) || asset_id > i64::MAX as u64 || i.expected_revision <= 0 {
        return Err(invalid());
    }
    let base = s
        .config
        .resource_base_url
        .as_deref()
        .ok_or_else(unavailable)?;
    let row: Option<(i64, Value)> =
        sqlx::query_as("SELECT revision,data FROM discover_drafts WHERE kind='tool' AND key=$1")
            .bind(&key)
            .fetch_optional(&s.pool)
            .await?;
    let (revision, data) = row.ok_or_else(|| AppError::not_found("工具草稿"))?;
    if revision != i.expected_revision {
        return Err(AppError::Conflict("工具草稿已更新，请刷新后重试".into()));
    }
    let expected = candidate(data, revision, asset_id)?;
    let http = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| unavailable())?;
    let receipt = request_mirror(&http, base, &expected.claims, &s.config.resource_secret).await?;
    register(&s.pool, &key, &expected, &receipt, Some(&admin)).await?;
    Ok(Json(
        json!({"status":"verified","verified_at":DateTime::from_timestamp(receipt.verified_at,0),"published":false}),
    ))
}
async fn register(
    pool: &PgPool,
    key: &str,
    expected: &Candidate,
    r: &Receipt,
    admin: Option<&crate::handlers::admin::AdminIdentity>,
) -> AppResult<()> {
    if !receipt_valid(r, &expected.claims, Utc::now().timestamp()) {
        return Err(invalid());
    }
    let mut tx = pool.begin().await?;
    discover_editor::lock(&mut tx, "tool", key).await?;
    let row: Option<(i64, Value)> = sqlx::query_as(
        "SELECT revision,data FROM discover_drafts WHERE kind='tool' AND key=$1 FOR UPDATE",
    )
    .bind(key)
    .fetch_optional(&mut *tx)
    .await?;
    let (revision, data) = row.ok_or_else(|| AppError::not_found("工具草稿"))?;
    if revision != expected.revision {
        return Err(AppError::Conflict(
            "镜像完成期间草稿已更新；未登记此结果，请刷新后重试".into(),
        ));
    }
    let current = candidate(data, revision, expected.claims.asset_id)?;
    if current.claims.sha256 != expected.claims.sha256
        || current.claims.size != expected.claims.size
        || current.claims.source_url != expected.claims.source_url
        || current.claims.filename != expected.claims.filename
    {
        return Err(AppError::Conflict(
            "工具安装包已变化，镜像结果未登记".into(),
        ));
    }
    sqlx::query("INSERT INTO tool_mirror_registry(tool_key,asset_id,sha256,size,filename,file_key,source_url,verified_at)VALUES($1,$2,$3,$4,$5,$6,$7,$8)ON CONFLICT(tool_key,asset_id,sha256)DO UPDATE SET verified_at=EXCLUDED.verified_at,size=EXCLUDED.size,filename=EXCLUDED.filename,file_key=EXCLUDED.file_key,source_url=EXCLUDED.source_url")
        .bind(key).bind(r.asset_id as i64).bind(&r.sha256).bind(r.size as i64).bind(&r.filename).bind(&r.file_key).bind(&expected.claims.source_url).bind(DateTime::from_timestamp(r.verified_at,0)).execute(&mut *tx).await?;
    if let Some(admin) = admin {
        audit(
            &mut tx,
            admin,
            "tool.mirror.verify",
            "tool",
            key,
            &format!("asset_id={} sha256={}", r.asset_id, r.sha256),
        )
        .await?;
    }
    tx.commit().await?;
    Ok(())
}
/// Call only on public read responses. This helper independently checks the currently published
/// asset reference, so accidentally passing an unpublished candidate cannot expose a staged mirror.
pub async fn decorate_tool(
    pool: &PgPool,
    public_base: &str,
    key: &str,
    data: &mut Value,
) -> AppResult<()> {
    if data["source"] != "GitHub Releases" || data["github_repo_id"].as_u64().is_none() {
        return Ok(());
    }
    let live: Option<(Value,)> =
        sqlx::query_as("SELECT data FROM discover_items WHERE kind='tool' AND key=$1 AND status='published' AND published_at<=now()")
            .bind(key)
            .fetch_optional(pool)
            .await?;
    let Some((live,)) = live else { return Ok(()) };
    let rows: Vec<(i64, String, i64)> =
        sqlx::query_as("SELECT asset_id,sha256,size FROM tool_mirror_registry WHERE tool_key=$1")
            .bind(key)
            .fetch_all(pool)
            .await?;
    if let Some(artifacts) = data["artifacts"].as_array_mut() {
        for a in artifacts {
            let same = live["artifacts"].as_array().is_some_and(|items| {
                items.iter().any(|v| {
                    v["asset_id"] == a["asset_id"]
                        && v["sha256"] == a["sha256"]
                        && v["size"] == a["size"]
                })
            });
            if same
                && rows.iter().any(|r| {
                    a["asset_id"].as_i64() == Some(r.0)
                        && a["sha256"].as_str() == Some(&r.1)
                        && a["size"].as_i64() == Some(r.2)
                })
            {
                a["mirror_url"] = json!(endpoint(
                    public_base,
                    &format!(
                        "api/discover/tool-mirrors/{key}/{}/{}",
                        a["asset_id"].as_i64().unwrap(),
                        a["sha256"].as_str().unwrap()
                    )
                )?
                .to_string())
            }
        }
    }
    Ok(())
}
fn resource_download_url(
    base: &str,
    sha: &str,
    name: &str,
    exp: i64,
    secret: &[u8],
) -> AppResult<Url> {
    let logical = format!("tool-mirrors/{sha}/{name}");
    let sig = signature(b"GET\n", format!("{logical}\n{exp}").as_bytes(), secret);
    let mut url = endpoint(base, "tool-mirrors")?;
    url.path_segments_mut()
        .map_err(|_| unavailable())?
        .push(sha)
        .push(name);
    url.query_pairs_mut()
        .append_pair("exp", &exp.to_string())
        .append_pair("sig", &sig);
    Ok(url)
}
async fn download(
    State(s): State<AppState>,
    Path((key, asset_id, sha)): Path<(String, i64, String)>,
) -> AppResult<Redirect> {
    if !crate::discover::valid_key(&key)
        || asset_id <= 0
        || sha.len() != 64
        || !sha.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err(AppError::not_found("工具镜像"));
    }
    let live: Option<(Value,)> =
        sqlx::query_as("SELECT data FROM discover_items WHERE kind='tool' AND key=$1 AND status='published' AND published_at<=now()")
            .bind(&key)
            .fetch_optional(&s.pool)
            .await?;
    let data = live.ok_or_else(|| AppError::not_found("工具镜像"))?.0;
    let c = candidate(data, 0, asset_id as u64)?;
    if c.claims.sha256 != sha {
        return Err(AppError::not_found("工具镜像"));
    }
    let stored:Option<(i64,String,String)>=sqlx::query_as("SELECT size,filename,file_key FROM tool_mirror_registry WHERE tool_key=$1 AND asset_id=$2 AND sha256=$3").bind(&key).bind(asset_id).bind(&sha).fetch_optional(&s.pool).await?;
    let (size, _name, file_key) = stored.ok_or_else(|| AppError::not_found("工具镜像"))?;
    if size != c.claims.size as i64 || file_key != format!("tools/{sha}.bin") {
        return Err(invalid());
    }
    let Some(base) = s.config.resource_base_url.as_deref() else {
        return Ok(Redirect::temporary(&c.claims.source_url));
    };
    let exp = Utc::now().timestamp() + 600;
    let parsed = resource_download_url(
        base,
        &sha,
        &c.claims.filename,
        exp,
        &s.config.resource_secret,
    )?;
    let healthy = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(3))
        .build()
        .map_err(|_| unavailable())?
        .head(parsed.clone())
        .send()
        .await
        .is_ok_and(|r| r.url() == &parsed && r.status().is_success());
    Ok(Redirect::temporary(if healthy {
        parsed.as_str()
    } else {
        &c.claims.source_url
    }))
}
#[cfg(test)]
#[path = "tool_mirrors_tests.rs"]
mod tests;
