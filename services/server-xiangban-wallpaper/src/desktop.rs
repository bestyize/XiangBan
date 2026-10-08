//! The website owns desktop releases, notices and the published wallpaper catalogue.
use crate::{
    api::AppState,
    error::AppError,
    models::{MediaType, WallpaperItem},
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
                network_url(&package.url)?;
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
async fn config(store: &Store) -> Result<DesktopConfig, AppError> {
    match store.get_setting("desktop.config").await? {
        Some(value) => serde_json::from_str(&value).map_err(|e| AppError::Internal(e.to_string())),
        None => Ok(DesktopConfig::default()),
    }
}
pub async fn admin_get(State(state): State<AppState>) -> Result<Json<DesktopConfig>, AppError> {
    Ok(Json(config(state.admin.store()?).await?))
}
pub async fn admin_save(
    State(state): State<AppState>,
    Json(mut value): Json<DesktopConfig>,
) -> Result<Json<DesktopConfig>, AppError> {
    value.site_url = value.site_url.trim().trim_end_matches('/').to_string();
    value.validate()?;
    state
        .admin
        .store()?
        .set_setting(
            "desktop.config",
            &serde_json::to_string(&value).map_err(|e| AppError::Internal(e.to_string()))?,
        )
        .await?;
    Ok(Json(value))
}
pub async fn product(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    let config = config(state.admin.store()?).await?;
    let downloads: BTreeMap<_,_> = TARGETS.iter().filter_map(|target| config.latest(Some(target), false).map(|r| (*target, json!({"version":r.version,"notes":r.notes,"publishedAt":r.published_at,"url":r.platforms[*target].url})))).collect();
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
) -> Result<Response, AppError> {
    let current =
        semver::Version::parse(&query.current_version).map_err(|_| bad("当前版本号无效"))?;
    if !TARGETS.contains(&query.target.as_str()) {
        return Err(bad("不支持的平台"));
    }
    let config = config(state.admin.store()?).await?;
    match config.latest(Some(&query.target), !current.pre.is_empty()) {
        Some(r) if semver::Version::parse(&r.version).unwrap() > current => Ok(Json(json!({"schemaVersion":1,"version":r.version,"notes":r.notes,"publishedAt":r.published_at,"platforms":r.platforms})).into_response()),
        _ => Ok(StatusCode::NO_CONTENT.into_response()),
    }
}
pub async fn announcements(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    Ok(Json(
        json!({"schemaVersion":1,"items":config(state.admin.store()?).await?.notices("desktop")}),
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
                .map_err(|_| bad("媒体域名无法解析"))?
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
            .map_err(|e| bad(&format!("读取媒体失败: {e}")))?;
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
        return response
            .error_for_status()
            .map_err(|_| bad("媒体源返回错误"));
    }
    Err(bad("媒体重定向过多"))
}
async fn resource(
    store: &Store,
    source: &str,
    kind: &str,
    origin: &str,
) -> Result<Resource, AppError> {
    let key = format!("desktop.media.{}", hash(source.as_bytes()));
    if let Some(value) = store.get_setting(&key).await? {
        let mut resource: Resource =
            serde_json::from_str(&value).map_err(|e| AppError::Internal(e.to_string()))?;
        if tokio::fs::metadata(resource.url.trim_start_matches('/'))
            .await
            .is_ok_and(|m| m.len() == resource.bytes)
        {
            resource.url = format!("{origin}{}", resource.url);
            return Ok(resource);
        }
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
            while let Some(chunk) = response.chunk().await.map_err(|_| bad("媒体下载中断"))? {
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
            return Err(bad("媒体格式不适用于桌面壁纸"));
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
                        q.url
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
pub async fn wallpaper(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, AppError> {
    let item = state
        .db
        .get_wallpaper(&id)
        .await?
        .filter(|w| w.status == "published")
        .ok_or_else(|| AppError::NotFound("壁纸未上架或不存在".into()))?;
    let store = state.admin.store()?;
    let config = config(store).await?;
    let (url, kind) = source(&item)?;
    let resource = resource(store, url, kind, &config.site_url).await?;
    // Remote thumbnails are fetched by the client only for gallery display; the manifest retains
    // the strict resource format and does not turn video URLs into image covers.
    let definition = json!({"id":format!("site-{}", &hash(item.id.as_bytes())[..40]),"version":"1.0.0","title":clip(&item.title,160),"description":clip(item.description.as_deref().unwrap_or(""),2048),"author":clip(&item.author.name,160),"kind":kind,"resource":resource});
    Ok(Json(
        json!({"schemaVersion":1,"revision":resource.sha256,"items":[definition]}),
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
}
