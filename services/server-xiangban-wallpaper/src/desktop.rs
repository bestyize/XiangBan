//! The website owns desktop releases, notices and the published wallpaper catalogue.
use crate::{
    api::AppState,
    error::AppError,
    hotlink::{Overrides, Policy},
    models::{CopyFile, LocalCopy, MediaType, WallpaperItem},
    public_item::apply_copy,
    store::Store,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, HashSet},
    net::IpAddr,
    path::PathBuf,
    time::Duration,
};
use tokio::io::AsyncWriteExt;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Package {
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bytes: Option<u64>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Release {
    pub version: String,
    pub notes: String,
    pub published_at: DateTime<Utc>,
    pub published: bool,
    pub platforms: BTreeMap<String, Package>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Announcement {
    pub id: String,
    pub title: String,
    pub body: String,
    pub published: bool,
    pub audience: Vec<String>,
    pub starts_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct DesktopConfig {
    pub site_url: String,
    pub introduction: String,
    pub releases: Vec<Release>,
    pub announcements: Vec<Announcement>,
}
impl Default for DesktopConfig {
    fn default() -> Self {
        Self {
            site_url: "https://wallpaper.xiangban.xyz".into(),
            introduction: "会回应的桌面小伙伴，搭配喜欢的图片与动态壁纸，让工作和休息多一点温暖。"
                .into(),
            releases: vec![],
            announcements: vec![],
        }
    }
}
const TARGETS: &[&str] = &["windows-x86_64", "darwin-aarch64", "darwin-x86_64"];
pub async fn no_store(mut response: Response) -> Response {
    response.headers_mut().insert(
        axum::http::header::CACHE_CONTROL,
        axum::http::HeaderValue::from_static("no-store"),
    );
    response
}
fn bad(message: &str) -> AppError {
    AppError::BadRequest(message.into())
}
pub fn network_url(value: &str) -> Result<reqwest::Url, AppError> {
    let url = reqwest::Url::parse(value).map_err(|_| bad("地址格式无效"))?;
    let local = matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"));
    if !(url.scheme() == "https" || (url.scheme() == "http" && local))
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
    {
        return Err(bad(
            "地址必须使用 HTTPS（本机开发可使用 HTTP），不能包含账户或片段",
        ));
    }
    Ok(url)
}
fn hash(value: &[u8]) -> String {
    format!("{:x}", Sha256::digest(value))
}
fn valid_hash(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
}
impl DesktopConfig {
    pub fn validate(&self) -> Result<(), AppError> {
        let url = network_url(&self.site_url)?;
        if url.path() != "/" || url.query().is_some() {
            return Err(bad(
                "网站地址只填写源站地址，例如 https://wallpaper.example.com",
            ));
        }
        if self.introduction.len() > 4000
            || self.releases.len() > 100
            || self.announcements.len() > 100
            || serde_json::to_vec(self).map_or(true, |v| v.len() > 900 * 1024)
        {
            return Err(bad("介绍或配置数量超过限制（配置总量最多 900 KiB）"));
        }
        let mut versions = HashSet::new();
        for release in &self.releases {
            semver::Version::parse(&release.version)
                .map_err(|_| bad("版本号必须使用 SemVer，例如 1.2.0"))?;
            if !versions.insert(&release.version)
                || release.notes.len() > 16000
                || release.platforms.is_empty()
            {
                return Err(bad("版本重复、说明过长或缺少平台下载链接"));
            }
            for (target, package) in &release.platforms {
                if !TARGETS.contains(&target.as_str()) {
                    return Err(bad("不支持的平台"));
                }
                if !crate::app_packages::is_owned_url(&package.url) {
                    network_url(&package.url)?;
                }
                if package.sha256.as_deref().is_some_and(|h| !valid_hash(h))
                    || package.bytes.is_some_and(|n| n == 0)
                {
                    return Err(bad("安装包大小或 SHA-256 无效"));
                }
            }
        }
        let mut ids = HashSet::new();
        for notice in &self.announcements {
            if notice.id == "update"
                || notice.id.is_empty()
                || notice.id.len() > 64
                || !notice
                    .id
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-')
                || !ids.insert(&notice.id)
                || notice.title.trim().is_empty()
                || notice.title.len() > 160
                || notice.body.len() > 16000
                || notice.audience.is_empty()
                || notice
                    .audience
                    .iter()
                    .any(|v| !["website", "desktop"].contains(&v.as_str()))
                || matches!((notice.starts_at, notice.ends_at), (Some(start), Some(end)) if end <= start)
            {
                return Err(bad("公告 ID、内容、展示位置或有效时间无效"));
            }
        }
        Ok(())
    }
    pub fn latest(&self, target: Option<&str>, prerelease: bool) -> Option<&Release> {
        self.releases
            .iter()
            .filter(|r| r.published && r.published_at <= Utc::now())
            .filter(|r| target.is_none_or(|t| r.platforms.contains_key(t)))
            .filter(|r| {
                prerelease || semver::Version::parse(&r.version).is_ok_and(|v| v.pre.is_empty())
            })
            .max_by_key(|r| semver::Version::parse(&r.version).ok())
    }
    pub fn notices(&self, audience: &str) -> Vec<Value> {
        let now = Utc::now();
        self.announcements.iter().filter(|a| a.published && a.audience.iter().any(|v| v == audience)
            && a.starts_at.is_none_or(|s| s <= now) && a.ends_at.is_none_or(|e| e > now))
            .map(|a| json!({"id":a.id,"title":a.title,"body":a.body,"revision":hash(serde_json::to_string(a).unwrap().as_bytes())})).collect()
    }
}
pub(crate) async fn load_config(store: &Store) -> Result<DesktopConfig, AppError> {
    match store.get_setting("desktop.config").await? {
        Some(value) => serde_json::from_str(&value).map_err(|e| AppError::Internal(e.to_string())),
        None => Ok(DesktopConfig::default()),
    }
}
pub async fn admin_get(State(state): State<AppState>) -> Result<Json<DesktopConfig>, AppError> {
    Ok(Json(load_config(state.admin.store()?).await?))
}
pub async fn admin_save(
    State(state): State<AppState>,
    Json(mut value): Json<DesktopConfig>,
) -> Result<Json<DesktopConfig>, AppError> {
    value.site_url = value.site_url.trim().trim_end_matches('/').to_string();
    for release in &mut value.releases {
        crate::app_packages::normalize_release(&state, release).await?;
    }
    value.validate()?;
    state
        .admin
        .store()?
        .update_desktop_config(|config| {
            *config = value.clone();
            Ok(())
        })
        .await?;
    Ok(Json(value))
}
pub async fn product(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Value>, AppError> {
    let config = load_config(state.admin.store()?).await?;
    let origin = crate::app_packages::origin(&headers, &config.site_url);
    let downloads: BTreeMap<_,_> = TARGETS.iter().filter_map(|target| config.latest(Some(target), false).map(|r| {let platforms=crate::app_packages::public_platforms(r,&origin);(*target, json!({"version":r.version,"notes":r.notes,"publishedAt":r.published_at,"url":platforms[*target].url}))})).collect();
    Ok(Json(
        json!({"schemaVersion":1,"introduction":config.introduction,"downloads":downloads,"announcements":config.notices("website")}),
    ))
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateQuery {
    current_version: String,
    target: String,
}
pub async fn updates(
    State(state): State<AppState>,
    Query(query): Query<UpdateQuery>,
    headers: axum::http::HeaderMap,
) -> Result<Response, AppError> {
    let current =
        semver::Version::parse(&query.current_version).map_err(|_| bad("当前版本号无效"))?;
    if !TARGETS.contains(&query.target.as_str()) {
        return Err(bad("不支持的平台"));
    }
    let config = load_config(state.admin.store()?).await?;
    let origin = crate::app_packages::origin(&headers, &config.site_url);
    match config.latest(Some(&query.target), !current.pre.is_empty()) {
        Some(r) if semver::Version::parse(&r.version).unwrap() > current => Ok(Json(json!({"schemaVersion":1,"version":r.version,"notes":r.notes,"publishedAt":r.published_at,"platforms":crate::app_packages::public_platforms(r,&origin)})).into_response()),
        _ => Ok(StatusCode::NO_CONTENT.into_response()),
    }
}
pub async fn announcements(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    Ok(Json(
        json!({"schemaVersion":1,"items":load_config(state.admin.store()?).await?.notices("desktop")}),
    ))
}

// Materialise selected published media once. Files are immutable, content-addressed and streamed;
// public catalogue requests never proxy arbitrary caller-supplied URLs.
#[derive(Clone, Serialize, Deserialize)]
pub struct Resource {
    url: String,
    sha256: String,
    bytes: u64,
    mime: String,
}
fn public_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            !ip.is_private()
                && !ip.is_loopback()
                && !ip.is_link_local()
                && !ip.is_unspecified()
                && !ip.is_multicast()
                && ip.octets()[0] != 0
                && ip.octets()[0] < 240
                && !(ip.octets()[0] == 100 && (64..=127).contains(&ip.octets()[1]))
                && !ip.is_broadcast()
        }
        IpAddr::V6(ip) => ip
            .to_ipv4_mapped()
            .map(|v| public_ip(IpAddr::V4(v)))
            .unwrap_or_else(|| {
                !ip.is_loopback()
                    && !ip.is_unspecified()
                    && !ip.is_multicast()
                    && (ip.segments()[0] & 0xe000) == 0x2000
            }),
    }
}
async fn remote(value: &str) -> Result<reqwest::Response, AppError> {
    let mut url = network_url(value)?;
    for _ in 0..5 {
        if url.scheme() != "https" {
            return Err(bad("外部媒体必须使用 HTTPS"));
        }
        let host = url.host_str().ok_or_else(|| bad("媒体缺少域名"))?;
        let addresses: Vec<_> =
            tokio::net::lookup_host((host, url.port_or_known_default().unwrap_or(443)))
                .await
                .map_err(|_| bad(&format!("媒体域名 {host} 无法解析")))?
                .collect();
        if addresses.is_empty() || addresses.iter().any(|a| !public_ip(a.ip())) {
            return Err(bad("媒体地址不能指向内部网络"));
        }
        let client = reqwest::Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(120))
            .resolve_to_addrs(host, &addresses)
            .build()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let response = client
            .get(url.clone())
            .header("Accept-Encoding", "identity")
            .send()
            .await
            .map_err(|e| bad(&fetch_failure(host, &e)))?;
        if response.status().is_redirection() {
            url = url
                .join(
                    response
                        .headers()
                        .get("location")
                        .and_then(|h| h.to_str().ok())
                        .ok_or_else(|| bad("媒体重定向无效"))?,
                )
                .map_err(|_| bad("媒体重定向无效"))?;
            network_url(url.as_str())?;
            continue;
        }
        if !response.status().is_success() {
            // Still a 400 for clients, but the message says which side failed and how.
            return Err(bad(&format!(
                "媒体源 {host} 返回 HTTP {}",
                response.status().as_u16()
            )));
        }
        return Ok(response);
    }
    Err(bad("媒体重定向过多"))
}
fn fetch_failure(host: &str, error: &reqwest::Error) -> String {
    if error.is_timeout() {
        format!("媒体源 {host} 响应超时（超过 120 秒）")
    } else if error.is_connect() {
        format!("无法连接媒体源 {host}: {error}")
    } else {
        format!("读取媒体源 {host} 失败: {error}")
    }
}
/// The stored copy of `source`, if one was made earlier and its file is still intact. Never
/// downloads. The returned URL is site-relative.
async fn cached_resource(store: &Store, source: &str) -> Result<Option<Resource>, AppError> {
    let key = format!("desktop.media.{}", hash(source.as_bytes()));
    let Some(value) = store.get_setting(&key).await? else {
        return Ok(None);
    };
    let resource: Resource =
        serde_json::from_str(&value).map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(file_matches(&resource).await.then_some(resource))
}
async fn file_matches(resource: &Resource) -> bool {
    tokio::fs::metadata(resource.url.trim_start_matches('/'))
        .await
        .is_ok_and(|m| m.len() == resource.bytes)
}
fn with_origin(mut resource: Resource, origin: &str) -> Resource {
    resource.url = format!("{origin}{}", resource.url);
    resource
}
fn copy_file(resource: &Resource) -> CopyFile {
    CopyFile {
        path: resource.url.clone(),
        sha256: resource.sha256.clone(),
        bytes: resource.bytes,
        mime: resource.mime.clone(),
    }
}
/// A copy recorded in `local_copy`, as a resource, if the file is still there.
async fn recorded(file: Option<&CopyFile>) -> Option<Resource> {
    let file = file.filter(|f| f.is_safe())?;
    let resource = Resource {
        url: file.path.clone(),
        sha256: file.sha256.clone(),
        bytes: file.bytes,
        mime: file.mime.clone(),
    };
    file_matches(&resource).await.then_some(resource)
}
async fn resource(
    store: &Store,
    source: &str,
    kind: &str,
    origin: &str,
) -> Result<Resource, AppError> {
    let key = format!("desktop.media.{}", hash(source.as_bytes()));
    if let Some(resource) = cached_resource(store, source).await? {
        return Ok(with_origin(resource, origin));
    }
    let max = 256 * 1024 * 1024;
    tokio::fs::create_dir_all("uploads")
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let temporary = PathBuf::from(format!("uploads/desktop-{}.part", uuid::Uuid::new_v4()));
    let result = async {
        let mut file = tokio::fs::File::create(&temporary)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let mut digest = Sha256::new();
        let mut bytes = 0u64;
        let mime;
        if let Some(name) = source.strip_prefix("/uploads/") {
            if name.is_empty()
                || !name
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_' | b'.'))
                || name.contains("..")
            {
                return Err(bad("本地媒体路径无效"));
            }
            let root = tokio::fs::canonicalize("uploads")
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;
            let path = tokio::fs::canonicalize(root.join(name))
                .await
                .map_err(|_| bad("媒体文件不存在"))?;
            if !path.starts_with(&root) {
                return Err(bad("媒体路径越界"));
            }
            mime = mime_guess::from_path(&path)
                .first_or_octet_stream()
                .essence_str()
                .to_string();
            let mut input = tokio::fs::File::open(path)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;
            let mut buffer = vec![0u8; 64 * 1024];
            loop {
                use tokio::io::AsyncReadExt;
                let n = input
                    .read(&mut buffer)
                    .await
                    .map_err(|e| AppError::Internal(e.to_string()))?;
                if n == 0 {
                    break;
                }
                bytes += n as u64;
                if bytes > max {
                    return Err(bad("桌面壁纸不能超过 256 MiB"));
                }
                digest.update(&buffer[..n]);
                file.write_all(&buffer[..n])
                    .await
                    .map_err(|e| AppError::Internal(e.to_string()))?;
            }
        } else {
            let mut response = remote(source).await?;
            mime = response
                .headers()
                .get("content-type")
                .and_then(|h| h.to_str().ok())
                .unwrap_or("")
                .split(';')
                .next()
                .unwrap_or("")
                .to_string();
            if response.content_length().is_some_and(|n| n > max) {
                return Err(bad("桌面壁纸不能超过 256 MiB"));
            }
            while let Some(chunk) = response.chunk().await.map_err(|e| {
                bad(if e.is_timeout() {
                    "媒体下载超时（超过 120 秒）"
                } else {
                    "媒体下载中断"
                })
            })? {
                bytes += chunk.len() as u64;
                if bytes > max {
                    return Err(bad("桌面壁纸不能超过 256 MiB"));
                }
                digest.update(&chunk);
                file.write_all(&chunk)
                    .await
                    .map_err(|e| AppError::Internal(e.to_string()))?;
            }
        }
        if bytes == 0
            || !(if kind == "video" {
                ["video/mp4", "video/webm"].contains(&mime.as_str())
            } else {
                [
                    "image/png",
                    "image/jpeg",
                    "image/webp",
                    "image/avif",
                    "image/gif",
                    "image/bmp",
                ]
                .contains(&mime.as_str())
            })
        {
            return Err(bad(&format!(
                "媒体格式不适用于桌面壁纸（{}，{bytes} 字节）",
                if mime.is_empty() { "未知类型" } else { mime.as_str() }
            )));
        }
        file.flush()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        drop(file);
        let sha256 = format!("{:x}", digest.finalize());
        let filename = format!(
            "desktop-{sha256}.{}",
            mime_guess::get_mime_extensions_str(&mime)
                .and_then(|e| e.first())
                .unwrap_or(&"bin")
        );
        let path = PathBuf::from("uploads").join(&filename);
        // Concurrent requests for the same immutable file are harmless; Windows cannot rename over it.
        if tokio::fs::metadata(&path).await.is_err() {
            if let Err(error) = tokio::fs::rename(&temporary, &path).await {
                if tokio::fs::metadata(&path).await.is_err() {
                    return Err(AppError::Internal(error.to_string()));
                }
            }
        }
        let mut resource = Resource {
            url: format!("/uploads/{filename}"),
            sha256,
            bytes,
            mime,
        };
        store
            .set_setting(&key, &serde_json::to_string(&resource).unwrap())
            .await?;
        resource.url = format!("{origin}{}", resource.url);
        Ok(resource)
    }
    .await;
    let _ = tokio::fs::remove_file(&temporary).await;
    result
}
fn source(item: &WallpaperItem) -> Result<(&str, &str), AppError> {
    match item.media_type {
        MediaType::Image => Ok((&item.downloads.original, "image")),
        MediaType::Video => item
            .video
            .as_ref()
            .and_then(|v| {
                v.qualities
                    .iter()
                    .filter(|q| {
                        q.size_bytes.is_none_or(|bytes| bytes <= 256 * 1024 * 1024)
                            && q.url
                                .split('?')
                                .next()
                                .is_some_and(|p| p.ends_with(".mp4") || p.ends_with(".webm"))
                    })
                    .max_by_key(|q| q.width)
            })
            .map(|q| (q.url.as_str(), "video"))
            .ok_or_else(|| bad("该视频没有可供桌面播放的 MP4/WebM 资源")),
        _ => Err(bad("该媒体类型暂不支持桌面播放")),
    }
}
fn ffmpeg() -> std::path::PathBuf {
    if let Some(value) = std::env::var_os("FFMPEG_BIN") {
        return value.into();
    }
    let local = std::path::PathBuf::from(if cfg!(windows) {
        "target/media-tools/node_modules/ffmpeg-static/ffmpeg.exe"
    } else {
        "target/media-tools/node_modules/ffmpeg-static/ffmpeg"
    });
    if local.is_file() {
        local
    } else {
        "ffmpeg".into()
    }
}
async fn first_frame(media: &Resource) -> Result<Resource, AppError> {
    let temporary = std::path::PathBuf::from(format!("uploads/frame-{}.jpg", uuid::Uuid::new_v4()));
    let mut command = tokio::process::Command::new(ffmpeg());
    command
        .args([
            "-nostdin",
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-i",
            media.url.trim_start_matches('/'),
            "-frames:v",
            "1",
            "-vf",
            "scale=640:-2",
            "-q:v",
            "3",
        ])
        .arg(&temporary)
        .kill_on_drop(true);
    #[cfg(windows)]
    command.creation_flags(0x08000000);
    let result = async {
        let output = tokio::time::timeout(Duration::from_secs(30), command.output())
            .await
            .map_err(|_| bad("视频首帧处理超时"))?
            .map_err(|_| {
                AppError::ServiceUnavailable(
                    "生成视频首帧需要 FFmpeg，请安装或设置 FFMPEG_BIN".into(),
                )
            })?;
        if !output.status.success() {
            return Err(bad(&format!(
                "无法读取视频首帧：{}",
                String::from_utf8_lossy(&output.stderr)
                    .chars()
                    .take(500)
                    .collect::<String>()
            )));
        }
        let bytes = tokio::fs::read(&temporary)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        if bytes.is_empty() || bytes.len() > 2 * 1024 * 1024 {
            return Err(bad("视频首帧大小无效"));
        }
        let sha256 = hash(&bytes);
        let name = format!("desktop-{sha256}.jpg");
        let target = std::path::PathBuf::from("uploads").join(&name);
        if tokio::fs::metadata(&target).await.is_err() {
            tokio::fs::rename(&temporary, &target)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;
        }
        Ok(Resource {
            url: format!("/uploads/{name}"),
            sha256,
            bytes: bytes.len() as u64,
            mime: "image/jpeg".into(),
        })
    }
    .await;
    let _ = tokio::fs::remove_file(&temporary).await;
    result
}
/// An absolute https link a client can fetch directly. Local `/uploads/` paths, plain http and
/// links with credentials or fragments are not usable as a source.
pub fn source_link(value: &str) -> Option<String> {
    let url = reqwest::Url::parse(value).ok()?;
    (url.scheme() == "https"
        && url.host_str().is_some()
        && url.username().is_empty()
        && url.password().is_none()
        && url.fragment().is_none())
    .then(|| url.to_string())
}
/// The file type implied by the link's extension. Only a hint for clients, which still decide by
/// the real bytes.
pub fn guess_mime(value: &str) -> Option<&'static str> {
    let path = value.split(['?', '#']).next()?.to_ascii_lowercase();
    match path.rsplit_once('.')?.1 {
        "mp4" => Some("video/mp4"),
        "webm" => Some("video/webm"),
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "webp" => Some("image/webp"),
        "avif" => Some("image/avif"),
        "gif" => Some("image/gif"),
        "bmp" => Some("image/bmp"),
        _ => None,
    }
}
/// Finds or makes our copy of the item's media (and, for videos, its cover). This downloads.
async fn ensure_copy(
    store: &Store,
    item: &WallpaperItem,
) -> Result<(Resource, Option<Resource>), AppError> {
    let (url, kind) = source(item)?;
    let media = match recorded(item.local_copy.as_ref().and_then(|c| c.media.as_ref())).await {
        Some(media) => media,
        None => resource(store, url, kind, "").await?,
    };
    if item.media_type != MediaType::Video {
        return Ok((media, None));
    }
    let cover = match recorded(item.local_copy.as_ref().and_then(|c| c.cover.as_ref())).await {
        Some(cover) => cover,
        None => video_cover(store, item, &media).await?,
    };
    store
        .set_setting(
            &format!("desktop.cover.{}", media.sha256),
            &serde_json::to_string(&cover).unwrap(),
        )
        .await?;
    Ok((media, Some(cover)))
}
async fn video_cover(
    store: &Store,
    item: &WallpaperItem,
    media: &Resource,
) -> Result<Resource, AppError> {
    let has_cover = !item.preview_url.is_empty()
        && !item.preview_url.split('?').next().is_some_and(|v| {
            v.to_ascii_lowercase().ends_with(".mp4") || v.to_ascii_lowercase().ends_with(".webm")
        });
    if has_cover {
        return resource(store, &item.preview_url, "image", "").await;
    }
    match cover_for(store, &media.sha256).await? {
        Some(cover) => Ok(cover),
        None => first_frame(media).await,
    }
}
/// The cover generated earlier for a stored video, if its file is still there.
async fn cover_for(store: &Store, media_sha256: &str) -> Result<Option<Resource>, AppError> {
    let Some(value) = store.get_setting(&format!("desktop.cover.{media_sha256}")).await? else {
        return Ok(None);
    };
    let cover: Resource =
        serde_json::from_str(&value).map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(tokio::fs::metadata(cover.url.trim_start_matches('/'))
        .await
        .is_ok()
        .then_some(cover))
}
fn local_copy_of(media: &Resource, cover: Option<&Resource>) -> LocalCopy {
    LocalCopy {
        media: Some(copy_file(media)),
        cover: cover.map(copy_file),
    }
}
/// Collection-time step. Under the `source` policy nothing is downloaded: the provider's links
/// stay as they are and our copy is made later, only if a client needs it. Under `copy` the
/// media is stored now and recorded in `local_copy`; the provider's links are kept as well.
pub async fn prepare_wallpaper(store: &Store, item: &mut WallpaperItem) -> Result<(), AppError> {
    if item.media_type != MediaType::Video {
        return Ok(());
    }
    let policy = Overrides::load(store).await?.policy(item);
    prepare_with_policy(store, item, policy).await
}
/// `prepare_wallpaper` with the policy already decided (batch callers load the overrides once).
pub async fn prepare_with_policy(
    store: &Store,
    item: &mut WallpaperItem,
    policy: Policy,
) -> Result<(), AppError> {
    if item.media_type != MediaType::Video {
        return Ok(());
    }
    // Still checks that there is a playable MP4/WebM, without any network access.
    let (url, _) = source(item)?;
    if policy == Policy::Source && source_link(url).is_some() {
        return Ok(());
    }
    let (media, cover) = ensure_copy(store, item).await?;
    let copy = local_copy_of(&media, cover.as_ref());
    item.local_path = Some(media.url.clone());
    // Only fields that hold no remote link are rewritten (custom uploads, rows from before
    // `local_copy`); a provider's link is never replaced by a path into uploads/.
    apply_copy(item, &copy, true);
    item.local_copy = Some(copy);
    Ok(())
}
/// The manifest entry without any media: who, what and which kind.
fn definition(item: &WallpaperItem, kind: &str) -> Value {
    json!({"id":format!("site-{}", &hash(item.id.as_bytes())[..40]),"version":"1.0.0","title":clip(&item.title,160),"description":clip(item.description.as_deref().unwrap_or(""),2048),"author":clip(&item.author.name,160),"kind":kind})
}
/// When a client may use the provider's link directly: the item's policy is `source` and the
/// media has an absolute https link. Returns that link and the media kind.
pub fn direct_source(item: &WallpaperItem, policy: Policy) -> Option<(String, &str)> {
    if policy != Policy::Source {
        return None;
    }
    let (url, kind) = source(item).ok()?;
    Some((source_link(url)?, kind))
}
/// The `prefer=source` manifest. Nothing is downloaded: `resource` and `cover` describe our copy
/// only if it already exists. `revision` identifies the source link, so it does not change when a
/// copy is made later.
pub fn source_manifest(
    item: &WallpaperItem,
    kind: &str,
    link: &str,
    media: Option<Resource>,
    cover: Option<Resource>,
    origin: &str,
) -> Value {
    let mut source = json!({ "url": link });
    if let Some(mime) = guess_mime(link).map(str::to_string).or(media.as_ref().map(|m| m.mime.clone())) {
        source["mime"] = json!(mime);
    }
    let mut definition = definition(item, kind);
    definition["source"] = source;
    if let Some(media) = media {
        definition["resource"] = serde_json::to_value(with_origin(media, origin)).unwrap();
    }
    if let Some(cover) = cover {
        definition["cover"] = serde_json::to_value(with_origin(cover, origin)).unwrap();
    }
    json!({"schemaVersion":1,"revision":hash(link.as_bytes()),"items":[definition]})
}
/// Our copy of the media and cover if they already exist; never downloads.
async fn existing_copy(
    store: &Store,
    item: &WallpaperItem,
    media_url: &str,
) -> Result<(Option<Resource>, Option<Resource>), AppError> {
    let copy = item.local_copy.as_ref();
    let media = match recorded(copy.and_then(|c| c.media.as_ref())).await {
        Some(media) => Some(media),
        None => cached_resource(store, media_url).await?,
    };
    if item.media_type != MediaType::Video {
        return Ok((media, None));
    }
    let cover = match recorded(copy.and_then(|c| c.cover.as_ref())).await {
        Some(cover) => Some(cover),
        None if item.preview_url.is_empty() => None,
        None => match cached_resource(store, &item.preview_url).await? {
            Some(cover) => Some(cover),
            None => match &media {
                Some(media) => cover_for(store, &media.sha256).await?,
                None => None,
            },
        },
    };
    Ok((media, cover))
}
#[derive(Deserialize, Default)]
pub struct ManifestQuery {
    /// `source`: let the client fetch the provider's link itself. Other values mean no preference.
    prefer: Option<String>,
}
pub async fn wallpaper(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<ManifestQuery>,
) -> Result<Json<Value>, AppError> {
    let item = state
        .db
        .get_wallpaper(&id)
        .await?
        .filter(|w| w.status == "published")
        .ok_or_else(|| AppError::NotFound("壁纸未上架或不存在".into()))?;
    let store = state.admin.store()?;
    let config = load_config(store).await?;
    if query.prefer.as_deref() == Some("source") {
        let policy = Overrides::load(store).await?.policy(&item);
        if let Some((link, kind)) = direct_source(&item, policy) {
            let (url, _) = source(&item)?;
            let (media, cover) = existing_copy(store, &item, url).await?;
            return Ok(Json(source_manifest(&item, kind, &link, media, cover, &config.site_url)));
        }
    }
    let (_, kind) = source(&item)?;
    let (media, cover) = ensure_copy(store, &item).await?;
    // Remember the copy so the public catalogue can offer it as the fallback to the source link.
    let copy = local_copy_of(&media, cover.as_ref());
    if item.local_copy.as_ref() != Some(&copy) {
        if let Err(error) = state.db.update_local_copy(&item.id, &copy).await {
            tracing::warn!("Could not record the stored copy of {}: {error}", item.id);
        }
    }
    // Remote thumbnails are fetched by the client only for gallery display; the manifest retains
    // the strict resource format and does not turn video URLs into image covers.
    let media = with_origin(media, &config.site_url);
    let mut definition = definition(&item, kind);
    definition["resource"] = serde_json::to_value(&media).unwrap();
    if let Some(cover) = cover {
        definition["cover"] = serde_json::to_value(with_origin(cover, &config.site_url)).unwrap();
    }
    Ok(Json(
        json!({"schemaVersion":1,"revision":media.sha256,"items":[definition]}),
    ))
}
fn clip(value: &str, max: usize) -> &str {
    let mut end = value.len().min(max);
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    &value[..end]
}

#[cfg(test)]
mod tests {
    use super::*;
    fn release(version: &str, target: &str) -> Release {
        Release {
            version: version.into(),
            notes: "".into(),
            published: true,
            published_at: Utc::now() - chrono::Duration::seconds(10),
            platforms: BTreeMap::from([(
                target.into(),
                Package {
                    url: "https://example.com/app.zip".into(),
                    sha256: None,
                    bytes: None,
                },
            )]),
        }
    }
    #[test]
    fn selects_semver_per_platform_and_ignores_drafts_future_and_prereleases() {
        let mut config = DesktopConfig::default();
        config.releases = vec![
            release("1.9.0", "windows-x86_64"),
            release("1.10.0", "windows-x86_64"),
            release("2.0.0", "darwin-aarch64"),
            release("3.0.0-beta.1", "windows-x86_64"),
        ];
        let mut future = release("4.0.0", "windows-x86_64");
        future.published_at = Utc::now() + chrono::Duration::days(1);
        config.releases.push(future);
        let mut draft = release("5.0.0", "windows-x86_64");
        draft.published = false;
        config.releases.push(draft);
        assert_eq!(
            config
                .latest(Some("windows-x86_64"), false)
                .unwrap()
                .version,
            "1.10.0"
        );
        assert_eq!(
            config
                .latest(Some("darwin-aarch64"), false)
                .unwrap()
                .version,
            "2.0.0"
        );
        assert_eq!(
            config
                .latest(Some("darwin-x86_64"), false)
                .map(|r| &r.version),
            None
        );
        config.validate().unwrap();
    }
    #[test]
    fn notice_visibility_and_revision_follow_content() {
        let mut config = DesktopConfig::default();
        config.announcements.push(Announcement {
            id: "hello".into(),
            title: "你好".into(),
            body: "公告".into(),
            published: true,
            audience: vec!["desktop".into()],
            starts_at: None,
            ends_at: None,
        });
        assert!(config.notices("website").is_empty());
        let before = config.notices("desktop")[0]["revision"].clone();
        config.announcements[0].body = "新公告".into();
        assert_ne!(before, config.notices("desktop")[0]["revision"]);
        config.announcements[0].ends_at = Some(Utc::now() - chrono::Duration::seconds(1));
        assert!(config.notices("desktop").is_empty());
    }
    #[test]
    fn rejects_bad_configuration_and_private_media_addresses() {
        assert!(!public_ip("127.0.0.1".parse().unwrap()));
        assert!(!public_ip("10.0.0.1".parse().unwrap()));
        assert!(!public_ip("::ffff:192.168.0.1".parse().unwrap()));
        assert!(network_url("javascript:alert(1)").is_err());
        let mut config = DesktopConfig::default();
        config.releases = vec![release("bad", "windows-x86_64")];
        assert!(config.validate().is_err());
    }

    use crate::public_item::tests::sample;

    fn stored(name: &str, mime: &str) -> Resource {
        Resource {
            url: format!("/uploads/{name}"),
            sha256: "c".repeat(64),
            bytes: 4321,
            mime: mime.into(),
        }
    }

    #[test]
    fn direct_source_needs_the_source_policy_and_an_https_link() {
        let image = sample("pexels", MediaType::Image);
        let (link, kind) = direct_source(&image, Policy::Source).unwrap();
        assert_eq!((link.as_str(), kind), ("https://cdn.example.com/p/original.jpg", "image"));
        assert!(direct_source(&image, Policy::Copy).is_none(), "policy copy: plain behaviour");

        let mut local = sample("pexels", MediaType::Image);
        local.downloads.original = "/uploads/desktop-x.jpg".into();
        assert!(direct_source(&local, Policy::Source).is_none(), "already rewritten: no source link");
        let mut insecure = sample("pexels", MediaType::Image);
        insecure.downloads.original = "http://cdn.example.com/x.jpg".into();
        assert!(direct_source(&insecure, Policy::Source).is_none());
        let mut credentials = sample("pexels", MediaType::Image);
        credentials.downloads.original = "https://user:pass@cdn.example.com/x.jpg".into();
        assert!(direct_source(&credentials, Policy::Source).is_none());

        let video = sample("pixabay", MediaType::Video);
        assert_eq!(direct_source(&video, Policy::Source).unwrap().1, "video");
    }

    #[test]
    fn source_manifest_without_a_copy_has_a_source_and_no_resource() {
        let item = sample("pexels", MediaType::Image);
        let (link, kind) = direct_source(&item, Policy::Source).unwrap();
        let manifest = source_manifest(&item, kind, &link, None, None, "https://site.example");
        let entry = &manifest["items"][0];
        assert_eq!(manifest["schemaVersion"], 1);
        assert_eq!(entry["kind"], "image");
        assert_eq!(
            entry["source"],
            json!({"url": "https://cdn.example.com/p/original.jpg", "mime": "image/jpeg"})
        );
        assert!(entry.get("resource").is_none());
        assert!(entry.get("cover").is_none());
        assert_eq!(manifest["revision"], hash(link.as_bytes()));
    }

    #[test]
    fn source_manifest_for_a_video_without_copy_omits_resource_and_cover() {
        let item = sample("pixabay", MediaType::Video);
        let (link, kind) = direct_source(&item, Policy::Source).unwrap();
        let manifest = source_manifest(&item, kind, &link, None, None, "https://site.example");
        let entry = &manifest["items"][0];
        assert_eq!(entry["kind"], "video");
        assert_eq!(entry["source"], json!({"url": "https://cdn.example.com/v/large.mp4", "mime": "video/mp4"}));
        assert!(entry.get("resource").is_none() && entry.get("cover").is_none());
    }

    #[test]
    fn source_manifest_carries_our_copy_when_it_already_exists() {
        let item = sample("pixabay", MediaType::Video);
        let (link, kind) = direct_source(&item, Policy::Source).unwrap();
        let media = stored("desktop-m.mp4", "video/mp4");
        let cover = stored("desktop-c.jpg", "image/jpeg");
        let with_copy = source_manifest(&item, kind, &link, Some(media), Some(cover), "https://site.example");
        let entry = &with_copy["items"][0];
        assert_eq!(entry["source"]["url"], "https://cdn.example.com/v/large.mp4");
        assert_eq!(entry["resource"]["url"], "https://site.example/uploads/desktop-m.mp4");
        assert_eq!(entry["resource"]["sha256"], "c".repeat(64));
        assert_eq!(entry["resource"]["bytes"], 4321);
        assert_eq!(entry["resource"]["mime"], "video/mp4");
        assert_eq!(entry["cover"]["url"], "https://site.example/uploads/desktop-c.jpg");
        let without = source_manifest(&item, kind, &link, None, None, "https://site.example");
        assert_eq!(with_copy["revision"], without["revision"], "a later copy does not change the revision");
    }

    #[test]
    fn source_mime_falls_back_to_the_copys_when_the_link_has_no_extension() {
        let item = sample("pexels", MediaType::Image);
        let link = "https://cdn.example.com/photo?id=7";
        let bare = source_manifest(&item, "image", link, None, None, "");
        assert!(bare["items"][0]["source"].get("mime").is_none(), "unknown type is omitted");
        let copy = stored("desktop-m.png", "image/png");
        let known = source_manifest(&item, "image", link, Some(copy), None, "");
        assert_eq!(known["items"][0]["source"]["mime"], "image/png");
    }

    #[test]
    fn mime_hints_and_source_links() {
        assert_eq!(guess_mime("https://a.example/x/clip.MP4?token=1"), Some("video/mp4"));
        assert_eq!(guess_mime("https://a.example/x/photo.jpeg#frag"), Some("image/jpeg"));
        assert_eq!(guess_mime("https://a.example/x/photo"), None);
        assert_eq!(source_link("https://a.example/x.png").as_deref(), Some("https://a.example/x.png"));
        assert!(source_link("/uploads/x.png").is_none());
        assert!(source_link("javascript:alert(1)").is_none());
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteConfig {
    site_url: String,
    introduction: String,
    announcements: Vec<Announcement>,
}
pub async fn admin_save_site(
    State(state): State<AppState>,
    Json(value): Json<SiteConfig>,
) -> Result<Json<DesktopConfig>, AppError> {
    let config = state
        .admin
        .store()?
        .update_desktop_config(|config| {
            config.site_url = value.site_url.trim().trim_end_matches('/').to_string();
            config.introduction = value.introduction;
            config.announcements = value.announcements;
            Ok(())
        })
        .await?;
    Ok(Json(config))
}
