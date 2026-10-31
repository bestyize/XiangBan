//! Public GitHub Releases metadata and bounded official asset hashing. No executable is run.
//! API contract: https://docs.github.com/en/rest/releases/releases#get-the-latest-release
//! Assets: https://docs.github.com/en/rest/releases/assets#get-a-release-asset
//! Metadata client must disable redirects. Downloads use a separate client without credentials.
use super::CollectedItem;
use crate::error::{AppError, AppResult};
use chrono::{DateTime, Utc};
use reqwest::{
    header::{HeaderValue, AUTHORIZATION},
    Client, Url,
};
use ring::digest;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::BTreeSet,
    time::{Duration, Instant},
};

const API: &str = "https://api.github.com";
const MAX_METADATA: usize = 1024 * 1024;
const MAX_ASSET: u64 = 128 * 1024 * 1024;
const TOTAL_DOWNLOAD: u64 = 256 * 1024 * 1024;
const RUN_TIMEOUT: Duration = Duration::from_secs(180);
const MAX_ARTIFACTS: usize = 12;
const OPEN_LICENSES: &[&str] = &[
    "MIT",
    "Apache-2.0",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "MPL-2.0",
    "GPL-2.0",
    "GPL-3.0",
    "LGPL-2.1",
    "LGPL-3.0",
    "AGPL-3.0",
    "Unlicense",
    "0BSD",
    "CC0-1.0",
    "BSL-1.0",
    "Zlib",
];
type ToolResult<T> = Result<T, String>;
fn bad() -> AppError {
    AppError::BadRequest("GitHub 工具快照格式或来源校验失败".into())
}
fn valid_repo(repo: &str) -> bool {
    let Some((owner, name)) = repo.split_once('/') else {
        return false;
    };
    (1..=39).contains(&owner.len())
        && !owner.starts_with('-')
        && !owner.ends_with('-')
        && owner
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
        && (1..=100).contains(&name.len())
        && !matches!(name, "." | "..")
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'))
}
pub fn sanitize_config(config: Value) -> AppResult<Value> {
    let obj = config.as_object().ok_or_else(bad)?;
    if obj.keys().any(|k| k != "repositories") {
        return Err(bad());
    }
    let empty = Vec::new();
    let repos = match obj.get("repositories") {
        Some(v) => v.as_array().ok_or_else(bad)?,
        None => &empty,
    };
    if repos.len() > 10 {
        return Err(AppError::BadRequest(
            "AI 工具最多配置 10 个 GitHub 仓库".into(),
        ));
    }
    let mut clean = BTreeSet::new();
    for repo in repos {
        let repo = repo.as_str().ok_or_else(bad)?.trim().to_ascii_lowercase();
        if !valid_repo(&repo) || !clean.insert(repo) {
            return Err(AppError::BadRequest(
                "仓库须为不重复的 owner/repo，不能填写 URL".into(),
            ));
        }
    }
    Ok(json!({"repositories": clean.into_iter().collect::<Vec<_>>()}))
}
fn text(value: &Value, max: usize) -> ToolResult<String> {
    value
        .as_str()
        .filter(|s| {
            !s.trim().is_empty() && s.chars().count() <= max && !s.chars().any(char::is_control)
        })
        .map(str::to_string)
        .ok_or_else(|| "GitHub 工具字段缺失或超长".into())
}
fn iso(raw: &str) -> bool {
    DateTime::parse_from_rfc3339(raw).is_ok()
}
fn sha(raw: &str) -> bool {
    raw.len() == 64 && raw.bytes().all(|b| b.is_ascii_hexdigit())
}
fn safe_url(raw: &str, hosts: &[&str]) -> bool {
    Url::parse(raw).is_ok_and(|u| {
        u.scheme() == "https"
            && u.host_str().is_some_and(|h| hosts.contains(&h))
            && u.username().is_empty()
            && u.password().is_none()
            && u.port().is_none()
            && raw.len() <= 2000
            && u.fragment().is_none()
    })
}
fn release_url(raw: &str, repo: &str) -> bool {
    safe_url(raw, &["github.com"])
        && Url::parse(raw).is_ok_and(|u| {
            u.query().is_none()
                && u.path()
                    .to_ascii_lowercase()
                    .starts_with(&format!("/{}/releases/tag/", repo.to_ascii_lowercase()))
        })
}
fn source_path(repo: &str, group: &str, tag: &str, filename: Option<&str>) -> Option<Url> {
    let mut expected = Url::parse(&format!("https://github.com/{repo}/releases/{group}/")).ok()?;
    {
        let mut path = expected.path_segments_mut().ok()?;
        path.pop_if_empty().push(tag);
        if let Some(name) = filename {
            path.push(name);
        }
    }
    Some(expected)
}
fn public_asset_url(raw: &str, repo: &str) -> bool {
    safe_url(raw, &["github.com"])
        && Url::parse(raw).is_ok_and(|u| {
            u.query().is_none()
                && u.path().to_ascii_lowercase().starts_with(&format!(
                    "/{}/releases/download/",
                    repo.to_ascii_lowercase()
                ))
        })
}
fn download_url(url: &Url) -> bool {
    safe_url(
        url.as_str(),
        &[
            "github.com",
            "release-assets.githubusercontent.com",
            "objects.githubusercontent.com",
        ],
    ) && (url.host_str() != Some("github.com") || url.path().contains("/releases/download/"))
}
async fn metadata(http: &Client, endpoint: &str, key: &str) -> ToolResult<Value> {
    let mut request = http
        .get(endpoint)
        .header("accept", "application/vnd.github+json")
        .header("x-github-api-version", "2022-11-28")
        .timeout(Duration::from_secs(20));
    if !key.is_empty() {
        let value = HeaderValue::from_str(&format!("Bearer {key}"))
            .map_err(|_| "GitHub 凭据格式无效".to_string())?;
        request = request.header(AUTHORIZATION, value);
    }
    let mut response = request
        .send()
        .await
        .map_err(|_| "GitHub 工具接口连接失败或超时".to_string())?;
    if response.url().as_str() != endpoint || response.status().is_redirection() {
        return Err("GitHub 工具仓库发生重定向，请确认仓库更名后更新配置".into());
    }
    if !response.status().is_success() {
        return Err(match response.status().as_u16() {
            404 => "GitHub 仓库或正式版本不存在（HTTP 404）",
            403 | 429 => "GitHub API 权限不足或限流，请稍后重试",
            _ => "GitHub 工具接口暂时不可用",
        }
        .into());
    }
    if response
        .content_length()
        .is_some_and(|v| v > MAX_METADATA as u64)
    {
        return Err("GitHub 工具元数据超过 1 MiB".into());
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| "GitHub 工具响应中断".to_string())?
    {
        if bytes.len().saturating_add(chunk.len()) > MAX_METADATA {
            return Err("GitHub 工具元数据超过 1 MiB".into());
        }
        bytes.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&bytes).map_err(|_| "GitHub 工具响应不是有效 JSON".into())
}
struct Budget {
    remaining: u64,
    deadline: Instant,
}
/// URL validation happens before every request, including redirects. This client has no auth.
async fn hash_download(
    http: &Client,
    start: &str,
    size: u64,
    expected: Option<&str>,
    budget: &mut Budget,
    allowed: fn(&Url) -> bool,
) -> ToolResult<String> {
    if size == 0 || size > MAX_ASSET || size > budget.remaining {
        return Err("安装包超过下载限制（单文件 128 MiB、单次总计 256 MiB）".into());
    }
    let mut url = Url::parse(start).map_err(|_| "安装包地址无效".to_string())?;
    for hop in 0..=3 {
        if !allowed(&url) {
            return Err("安装包地址或重定向不属于允许的 GitHub 官方主机".into());
        }
        let remaining = budget.deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err("工具采集超过三分钟预算".into());
        }
        let mut response = http
            .get(url.clone())
            .header("accept-encoding", "identity")
            .timeout(remaining)
            .send()
            .await
            .map_err(|_| "安装包下载失败或超时".to_string())?;
        // Automatic redirects must be disabled even for the separate download client.
        if response.url() != &url {
            return Err("下载客户端不允许自动重定向".into());
        }
        if response.status().is_redirection() {
            if hop == 3 {
                return Err("安装包重定向超过三次".into());
            }
            let target = response
                .headers()
                .get(reqwest::header::LOCATION)
                .and_then(|v| v.to_str().ok())
                .filter(|v| v.len() <= 2000)
                .ok_or_else(|| "安装包重定向无效".to_string())?;
            url = url
                .join(target)
                .map_err(|_| "安装包重定向地址无效".to_string())?;
            continue;
        }
        if !response.status().is_success() {
            return Err("安装包来源返回失败状态".into());
        }
        if response
            .headers()
            .get(reqwest::header::CONTENT_ENCODING)
            .is_some_and(|v| v != "identity")
        {
            return Err("安装包来源未遵守 identity 编码，无法保证字节校验".into());
        }
        if response
            .content_length()
            .is_some_and(|v| v != size || v > MAX_ASSET || v > budget.remaining)
        {
            return Err("安装包大小与 GitHub 官方元数据不一致或超限".into());
        }
        let mut hash = digest::Context::new(&digest::SHA256);
        let mut downloaded = 0u64;
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|_| "安装包下载中断".to_string())?
        {
            let length = chunk.len() as u64;
            if downloaded.saturating_add(length) > size || length > budget.remaining {
                return Err("安装包实际大小超过元数据或下载预算".into());
            }
            budget.remaining -= length;
            downloaded += length;
            hash.update(&chunk);
        }
        if downloaded != size {
            return Err("安装包下载不完整，大小与元数据不一致".into());
        }
        let actual = hash
            .finish()
            .as_ref()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        if expected.is_some_and(|sha| !sha.eq_ignore_ascii_case(&actual)) {
            return Err("安装包 SHA-256 与官方摘要不一致".into());
        }
        return Ok(actual);
    }
    Err("安装包下载失败".into())
}
// A release may also contain unrelated helper programs. Keep only conservative primary-name
// matches; unfamiliar naming conventions require manual mapping rather than a guessed installer.
fn main_asset(filename: &str, repo_name: &str) -> bool {
    let lower = filename.to_ascii_lowercase();
    let repo = repo_name.to_ascii_lowercase();
    let Some(suffix) = lower.strip_prefix(&repo) else {
        return false;
    };
    if !suffix.starts_with(['-', '_', '.']) {
        return false;
    }
    let token = suffix
        .trim_start_matches(['-', '_', '.'])
        .split(|c: char| !c.is_ascii_alphanumeric())
        .next()
        .unwrap_or("");
    matches!(
        token,
        "aarch64"
            | "arm64"
            | "armv7"
            | "armhf"
            | "x86"
            | "amd64"
            | "x64"
            | "i686"
            | "i386"
            | "universal"
            | "windows"
            | "win"
            | "win32"
            | "win64"
            | "macos"
            | "mac"
            | "osx"
            | "darwin"
            | "linux"
            | "setup"
            | "installer"
            | "portable"
            | "exe"
            | "msi"
            | "dmg"
            | "pkg"
            | "appimage"
            | "deb"
            | "rpm"
    ) || token.as_bytes().first().is_some_and(u8::is_ascii_digit)
        || token
            .strip_prefix('v')
            .is_some_and(|v| v.as_bytes().first().is_some_and(u8::is_ascii_digit))
}
fn classify(name: &str) -> Option<(&'static str, &'static str)> {
    let lower = name.to_ascii_lowercase();
    let tokens = lower
        .split(|c: char| !c.is_ascii_alphanumeric())
        .collect::<Vec<_>>();
    if tokens.iter().any(|t| {
        matches!(
            *t,
            "source" | "sources" | "src" | "debug" | "symbols" | "checksum" | "checksums" | "sbom"
        )
    }) {
        return None;
    }
    let installer_os = if lower.ends_with(".exe") || lower.ends_with(".msi") {
        Some("windows")
    } else if lower.ends_with(".dmg") || lower.ends_with(".pkg") {
        Some("macos")
    } else if lower.ends_with(".appimage") || lower.ends_with(".deb") || lower.ends_with(".rpm") {
        Some("linux")
    } else {
        None
    };
    let archive = [".zip", ".tar.gz", ".tgz", ".tar.xz", ".tar.zst"]
        .iter()
        .any(|ext| lower.ends_with(ext));
    if installer_os.is_none() && !archive {
        return None;
    }
    let mut platforms = BTreeSet::new();
    if tokens
        .iter()
        .any(|t| matches!(*t, "windows" | "win32" | "win64" | "win"))
    {
        platforms.insert("windows");
    }
    if tokens
        .iter()
        .any(|t| matches!(*t, "macos" | "mac" | "osx" | "darwin"))
    {
        platforms.insert("macos");
    }
    if tokens.contains(&"linux") {
        platforms.insert("linux");
    }
    if let Some(os) = installer_os {
        platforms.insert(os);
    }
    if platforms.len() != 1 {
        return None;
    }
    let os = *platforms.first()?;
    let mut arches = BTreeSet::new();
    if lower.contains("x86_64")
        || lower.contains("x86-64")
        || tokens
            .iter()
            .any(|t| matches!(*t, "amd64" | "x64" | "win64"))
    {
        arches.insert("x86_64");
    }
    if tokens.iter().any(|t| matches!(*t, "arm64" | "aarch64")) {
        arches.insert("aarch64");
    }
    if tokens
        .iter()
        .any(|t| matches!(*t, "i686" | "i386" | "win32"))
        || (tokens.contains(&"x86") && !arches.contains("x86_64"))
    {
        arches.insert("x86");
    }
    if tokens.iter().any(|t| matches!(*t, "armv7" | "armhf")) {
        arches.insert("armv7");
    }
    if tokens.contains(&"universal") {
        arches.insert("universal");
    }
    let arch = if arches.len() == 1 {
        *arches.first()?
    } else {
        "unknown"
    };
    if archive && arch == "unknown" {
        return None;
    }
    Some((os, arch))
}
#[derive(Serialize, Deserialize)]
struct ReleaseItem {
    kind: String,
    text: String,
}
#[derive(Serialize, Deserialize)]
struct Release {
    version: String,
    date: String,
    url: String,
    items: Vec<ReleaseItem>,
}
#[derive(Serialize, Deserialize)]
struct Artifact {
    os: String,
    arch: String,
    desc: String,
    filename: String,
    size: u64,
    sha256: String,
    hash_source: String,
    asset_id: u64,
    source_updated_at: Option<String>,
    mirror_url: String,
    github_url: String,
}
#[derive(Serialize, Deserialize)]
struct Snapshot {
    source: String,
    as_of: String,
    github_repo_id: u64,
    github_full_name: String,
    release_id: u64,
    release_url: String,
    source_published_at: String,
    name: String,
    developer: String,
    type_label: String,
    license: String,
    license_spdx: Option<String>,
    license_source_url: String,
    platforms: Vec<String>,
    description: String,
    homepage_url: String,
    icon_url: String,
    discussion_tag: String,
    releases: Vec<Release>,
    artifacts: Vec<Artifact>,
    collection_notes: Vec<String>,
}
/// Dedicated validator retains the explicit snapshot metadata; legacy manual Tool remains separate.
pub fn validate_tool(value: Value) -> AppResult<Value> {
    let s: Snapshot = serde_json::from_value(value).map_err(|_| bad())?;
    let bounded = |v: &str, n| v.chars().count() <= n && !v.chars().any(char::is_control);
    if s.source != "GitHub Releases"
        || !iso(&s.as_of)
        || !iso(&s.source_published_at)
        || s.github_repo_id == 0
        || s.release_id == 0
        || !valid_repo(&s.github_full_name)
        || !release_url(&s.release_url, &s.github_full_name)
        || s.homepage_url != format!("https://github.com/{}", s.github_full_name)
        || s.license_source_url != s.homepage_url
        || !s.icon_url.is_empty()
        || !s.discussion_tag.is_empty()
        || s.name.is_empty()
        || !bounded(&s.name, 100)
        || s.developer.is_empty()
        || !bounded(&s.developer, 100)
        || s.type_label != "桌面 / 命令行工具"
        || !bounded(&s.description, 500)
        || s.collection_notes.len() > 20
        || s.collection_notes.iter().any(|v| !bounded(v, 300))
        || s.license_spdx.as_ref().is_some_and(|v| !bounded(v, 100))
    {
        return Err(bad());
    }
    if s.github_full_name.to_ascii_lowercase()
        != format!("{}/{}", s.developer, s.name).to_ascii_lowercase()
    {
        return Err(bad());
    }
    let expected_license = if s
        .license_spdx
        .as_deref()
        .is_some_and(|v| OPEN_LICENSES.contains(&v))
    {
        "open"
    } else {
        "unknown"
    };
    if s.license != expected_license
        || s.releases.len() != 1
        || s.artifacts.is_empty()
        || s.artifacts.len() > MAX_ARTIFACTS
    {
        return Err(bad());
    }
    let r = &s.releases[0];
    let date = DateTime::parse_from_rfc3339(&s.source_published_at)
        .map_err(|_| bad())?
        .with_timezone(&Utc)
        .date_naive()
        .to_string();
    if r.version.is_empty()
        || !bounded(&r.version, 128)
        || r.url != s.release_url
        || source_path(&s.github_full_name, "tag", &r.version, None)
            .is_none_or(|u| u.as_str() != r.url)
        || r.date != date
        || r.items.len() != 1
        || r.items[0].kind != "new"
        || !bounded(&r.items[0].text, 300)
    {
        return Err(bad());
    }
    let mut ids = BTreeSet::new();
    let mut platforms = BTreeSet::new();
    for a in &s.artifacts {
        if a.asset_id == 0
            || !ids.insert(a.asset_id)
            || a.size == 0
            || a.size > MAX_ASSET
            || !sha(&a.sha256)
            || !matches!(
                a.hash_source.as_str(),
                "github_digest" | "downloaded_sha256"
            )
            || !a.mirror_url.is_empty()
            || !public_asset_url(&a.github_url, &s.github_full_name)
            || source_path(
                &s.github_full_name,
                "download",
                &r.version,
                Some(&a.filename),
            )
            .is_none_or(|u| u.as_str() != a.github_url)
            || !bounded(&a.filename, 255)
            || a.desc != a.filename
            || !main_asset(&a.filename, &s.name)
            || classify(&a.filename) != Some((a.os.as_str(), a.arch.as_str()))
            || a.source_updated_at.as_ref().is_some_and(|v| !iso(v))
        {
            return Err(bad());
        }
        platforms.insert(match a.os.as_str() {
            "windows" => "Windows",
            "macos" => "macOS",
            "linux" => "Linux",
            _ => return Err(bad()),
        });
    }
    if s.platforms
        != platforms
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>()
    {
        return Err(bad());
    }
    serde_json::to_value(s).map_err(|_| bad())
}
async fn build_tool(
    repo: Value,
    release: Value,
    downloads: &Client,
    budget: &mut Budget,
) -> ToolResult<CollectedItem> {
    let repo_id = repo["id"]
        .as_u64()
        .filter(|v| *v > 0)
        .ok_or("GitHub 仓库 ID 无效")?;
    if repo["private"].as_bool() != Some(false) {
        return Err("工具采集仅支持公开仓库".into());
    }
    let full_name = text(&repo["full_name"], 140)?;
    if !valid_repo(&full_name) {
        return Err("GitHub 仓库名称无效".into());
    }
    let name = text(&repo["name"], 100)?;
    let developer = text(&repo["owner"]["login"], 100)?;
    if full_name.to_ascii_lowercase() != format!("{developer}/{name}").to_ascii_lowercase() {
        return Err("GitHub 仓库名称与所有者不一致".into());
    }
    if release["draft"].as_bool() != Some(false) || release["prerelease"].as_bool() != Some(false) {
        return Err("仅采集正式非草稿版本".into());
    }
    let release_id = release["id"]
        .as_u64()
        .filter(|v| *v > 0)
        .ok_or("正式版本 ID 无效")?;
    let tag = text(&release["tag_name"], 128)?;
    let release_link = text(&release["html_url"], 2000)?;
    if !release_url(&release_link, &full_name) {
        return Err("正式版本链接不属于仓库官方 GitHub 页面".into());
    }
    let published = text(&release["published_at"], 64)?;
    let date = DateTime::parse_from_rfc3339(&published)
        .map_err(|_| "正式版本发布时间无效".to_string())?
        .with_timezone(&Utc);
    let assets = release["assets"]
        .as_array()
        .filter(|v| v.len() <= 500)
        .ok_or("安装包清单无效或超过500项")?;
    let mut artifacts = Vec::new();
    let mut notes = Vec::new();
    let mut size_skips = 0;
    for asset in assets {
        let filename = text(&asset["name"], 255)?;
        if !main_asset(&filename, &name)
            || filename
                .to_ascii_lowercase()
                .split(|c: char| !c.is_ascii_alphanumeric())
                .any(|v| v == "bazel")
        {
            if notes.len() < 16 {
                notes.push(format!(
                    "未纳入：{filename}（辅助程序或非默认构建，需人工确认）"
                ));
            }
            continue;
        }
        let Some((os, arch)) = classify(&filename) else {
            if notes.len() < 16 {
                notes.push(format!(
                    "未纳入：{filename}（类型或平台/架构不明确，需人工确认）"
                ));
            }
            continue;
        };
        if asset["state"].as_str() != Some("uploaded") {
            return Err("安装包尚未上传完成".into());
        }
        if artifacts.len() == MAX_ARTIFACTS {
            if !notes
                .iter()
                .any(|s| s == "可识别安装包超过12项，其余安装包请人工编排")
            {
                notes.push("可识别安装包超过12项，其余安装包请人工编排".into());
            }
            continue;
        }
        let size = asset["size"]
            .as_u64()
            .filter(|v| *v > 0)
            .ok_or("安装包大小无效")?;
        if size > MAX_ASSET {
            size_skips += 1;
            if notes.len() < 16 {
                notes.push(format!(
                    "未纳入：{filename}（超过单文件128 MiB上限，请人工维护）"
                ));
            }
            continue;
        }
        let asset_id = asset["id"]
            .as_u64()
            .filter(|v| *v > 0)
            .ok_or("安装包 ID 无效")?;
        let link = text(&asset["browser_download_url"], 2000)?;
        if !public_asset_url(&link, &full_name) {
            return Err("安装包不属于仓库的官方 GitHub Release".into());
        }
        let (hash, hash_source) = match asset["digest"].as_str().filter(|v| !v.is_empty()) {
            Some(v) if v.starts_with("sha256:") => {
                let hash = &v[7..];
                if !sha(hash) {
                    return Err("官方 SHA-256 摘要格式无效".into());
                }
                (hash.to_ascii_lowercase(), "github_digest")
            }
            _ => (
                hash_download(downloads, &link, size, None, budget, download_url).await?,
                "downloaded_sha256",
            ),
        };
        if arch == "unknown" && notes.len() < 16 {
            notes.push(format!(
                "{filename} 未注明架构，已标为 unknown，发布前请人工核对"
            ));
        }
        let updated = asset["updated_at"].as_str().map(str::to_string);
        artifacts.push(Artifact {
            os: os.into(),
            arch: arch.into(),
            desc: filename.clone(),
            filename,
            size,
            sha256: hash,
            hash_source: hash_source.into(),
            asset_id,
            source_updated_at: updated,
            mirror_url: String::new(),
            github_url: link,
        });
    }
    if artifacts.is_empty() && size_skips > 0 {
        return Err("可识别安装包超过单文件128 MiB限制，其余资产不能可靠识别；旧版本保留".into());
    }
    if artifacts.is_empty() {
        return Err("没有可识别的正式安装包，请人工确认平台和架构；旧版本保留".into());
    }
    if size_skips > 0 {
        notes.push(format!(
            "共 {size_skips} 个主程序安装包超过单文件128 MiB上限，未纳入；请人工维护"
        ));
    }
    let platforms = artifacts
        .iter()
        .map(|a| match a.os.as_str() {
            "windows" => "Windows",
            "macos" => "macOS",
            _ => "Linux",
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(str::to_string)
        .collect();
    let spdx = repo["license"]["spdx_id"]
        .as_str()
        .filter(|v| !v.is_empty() && *v != "NOASSERTION")
        .map(str::to_string);
    let license = if spdx.as_deref().is_some_and(|v| OPEN_LICENSES.contains(&v)) {
        "open"
    } else {
        "unknown"
    };
    let homepage = format!("https://github.com/{full_name}");
    let description = match repo["description"].as_str() {
        Some(v) if v.chars().count() <= 500 => v.to_string(),
        None => String::new(),
        _ => return Err("仓库简介超过500字".into()),
    };
    let snapshot = Snapshot {
        source: "GitHub Releases".into(),
        as_of: Utc::now().to_rfc3339(),
        github_repo_id: repo_id,
        github_full_name: full_name,
        release_id,
        release_url: release_link.clone(),
        source_published_at: published,
        name,
        developer,
        type_label: "桌面 / 命令行工具".into(),
        license: license.into(),
        license_spdx: spdx,
        license_source_url: homepage.clone(),
        platforms,
        description,
        homepage_url: homepage,
        icon_url: String::new(),
        discussion_tag: String::new(),
        releases: vec![Release {
            version: tag.clone(),
            date: date.date_naive().to_string(),
            url: release_link.clone(),
            items: vec![ReleaseItem {
                kind: "new".into(),
                text: format!("开发者已正式发布 {tag}；完整更新记录见官方版本链接。"),
            }],
        }],
        artifacts,
        collection_notes: notes,
    };
    let data =
        validate_tool(serde_json::to_value(snapshot).map_err(|_| "无法整理工具快照".to_string())?)
            .map_err(|_| "工具快照未通过格式或来源校验".to_string())?;
    Ok(CollectedItem {
        kind: "tool".into(),
        key: format!("github-tool-{repo_id}"),
        data,
        sort_order: 0,
        published_at: Some(date),
        source_name: "GitHub Releases".into(),
        source_url: release_link,
    })
}
pub async fn collect(http: &Client, config: &Value, key: &str) -> ToolResult<Vec<CollectedItem>> {
    let config =
        sanitize_config(config.clone()).map_err(|_| "GitHub 工具来源配置无效".to_string())?;
    let repos = config["repositories"]
        .as_array()
        .expect("sanitized repositories");
    if repos.is_empty() {
        return Err("请先配置至少一个工具仓库".into());
    }
    let downloads = Client::builder()
        .no_gzip()
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_secs(10))
        .user_agent("XiaobanCommunityCollector/1.0")
        .build()
        .map_err(|_| "无法初始化安装包下载客户端".to_string())?;
    let future = async {
        let mut budget = Budget {
            remaining: TOTAL_DOWNLOAD,
            deadline: Instant::now() + RUN_TIMEOUT,
        };
        let mut items = Vec::new();
        let mut ids = BTreeSet::new();
        for repo in repos {
            let name = repo.as_str().expect("sanitized repo");
            let repo_data = metadata(http, &format!("{API}/repos/{name}"), key).await?;
            let release =
                metadata(http, &format!("{API}/repos/{name}/releases/latest"), key).await?;
            let item = build_tool(repo_data, release, &downloads, &mut budget).await?;
            if !ids.insert(item.key.clone()) {
                return Err("工具仓库 ID 重复，请检查重命名配置".into());
            }
            items.push(item);
        }
        Ok(items)
    };
    tokio::time::timeout(RUN_TIMEOUT, future)
        .await
        .map_err(|_| "工具采集超过三分钟预算，旧版本保留".to_string())?
}
#[cfg(test)]
#[path = "tool_sources_tests.rs"]
mod tests;
