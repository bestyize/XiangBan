//! Wallpaper delivery v1. Files are content-addressed, never arbitrary local paths.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, HashSet},
    fs,
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    time::Duration,
};
use tauri::Emitter;

pub const MAX_JSON: usize = 1024 * 1024;
pub const MAX_FILE: u64 = 256 * 1024 * 1024;
/// Error text of a download the user stopped; callers match on it, so it must not fall back.
pub const CANCELLED: &str = "已取消下载";
static NEVER: AtomicBool = AtomicBool::new(false);

fn check_cancel(cancel: &AtomicBool) -> Result<(), String> {
    if cancel.load(Ordering::Relaxed) {
        Err(CANCELLED.into())
    } else {
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Resource {
    pub url: String,
    pub sha256: String,
    pub bytes: u64,
    pub mime: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Control {
    pub key: String,
    pub label: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub default: Value,
    #[serde(default)]
    pub min: Option<f64>,
    #[serde(default)]
    pub max: Option<f64>,
    #[serde(default)]
    pub step: Option<f64>,
    #[serde(default)]
    pub options: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Wallpaper {
    pub id: String,
    pub version: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub author: String,
    pub kind: String,
    pub resource: Resource,
    #[serde(default)]
    pub cover: Option<Resource>,
    #[serde(default)]
    pub assets: BTreeMap<String, Resource>,
    #[serde(default)]
    pub controls: Vec<Control>,
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Catalog {
    pub schema_version: u32,
    #[serde(default)]
    pub revision: String,
    pub items: Vec<Wallpaper>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct Options {
    pub fit: String,
    pub playback_rate: f64,
    pub background: String,
    pub values: BTreeMap<String, Value>,
    /// Old preferences kept particle effects per wallpaper. The value is only read, so `Settings::normalize`
    /// can move it to the desktop-wide setting; it is never written back.
    #[serde(skip_serializing)]
    pub effects: Value,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            fit: "cover".into(),
            playback_rate: 1.0,
            background: "#edf5f0".into(),
            values: BTreeMap::new(),
            effects: Value::Null,
        }
    }
}

pub fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_' | b'.'))
}
pub fn safe_key(value: &str) -> bool {
    identifier(value) && !["__proto__", "prototype", "constructor"].contains(&value)
}
pub fn hash_name(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
}
pub fn color(value: &str) -> bool {
    value.len() == 7
        && value.starts_with('#')
        && value.as_bytes()[1..].iter().all(u8::is_ascii_hexdigit)
}

pub fn network_url(value: &str) -> Result<url::Url, String> {
    let url = url::Url::parse(value).map_err(|_| "地址格式不正确")?;
    let loopback = match url.host() {
        Some(url::Host::Domain(host)) => host == "localhost",
        Some(url::Host::Ipv4(ip)) => ip.is_loopback(),
        Some(url::Host::Ipv6(ip)) => ip.is_loopback(),
        _ => false,
    };
    if value.len() > 4096
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
        || !(url.scheme() == "https" || (url.scheme() == "http" && loopback))
    {
        return Err(
            "地址需使用 HTTPS；本机调试可使用 localhost HTTP，不接受文件地址或账户密码。".into(),
        );
    }
    Ok(url)
}

fn validate_resource(resource: &Resource, builtin: bool) -> Result<(), String> {
    if !hash_name(&resource.sha256) || resource.bytes == 0 || resource.bytes > MAX_FILE {
        return Err("资源校验值或大小不合法".into());
    }
    if builtin && resource.url.starts_with("builtin://") {
        let filename = &resource.url[10..];
        if !identifier(filename) {
            return Err("内置资源路径不合法".into());
        }
    } else {
        network_url(&resource.url)?;
    }
    if ![
        "image/png",
        "image/jpeg",
        "image/webp",
        "image/avif",
        "image/gif",
        "image/bmp",
        "video/mp4",
        "video/webm",
        "application/javascript",
        "application/json",
        "application/octet-stream",
        "text/html",
    ]
    .contains(&resource.mime.as_str())
        && !(builtin && resource.mime == "image/svg+xml")
    {
        return Err("不支持的资源类型".into());
    }
    if resource.mime == "application/javascript" && resource.bytes > 2 * 1024 * 1024 {
        return Err("动画脚本不能超过 2 MB，请预先打包压缩。".into());
    }
    if resource.mime == "text/html" && resource.bytes > 4 * 1024 * 1024 {
        return Err("自包含网页壁纸不能超过 4 MB。".into());
    }
    Ok(())
}

pub fn validate_item(item: &Wallpaper, builtin: bool) -> Result<(), String> {
    if !identifier(&item.id)
        || item.id == "companion"
        || (!builtin && (item.id.starts_with("builtin-") || item.id.starts_with("local-")))
    {
        return Err("壁纸 ID 不合法或使用了保留名称".into());
    }
    semver::Version::parse(&item.version).map_err(|_| "壁纸版本需使用 SemVer，如 1.0.0")?;
    if item.title.trim().is_empty()
        || item.title.len() > 160
        || item.description.len() > 2048
        || item.author.len() > 160
    {
        return Err("壁纸名称或介绍长度不合法".into());
    }
    validate_resource(&item.resource, builtin)?;
    let mime = &item.resource.mime;
    if !match item.kind.as_str() {
        "image" => mime.starts_with("image/"),
        "video" => ["video/mp4", "video/webm"].contains(&mime.as_str()),
        "canvas" => mime == "application/javascript",
        "web" => mime == "text/html",
        _ => false,
    } {
        return Err("壁纸类型与资源 MIME 不匹配".into());
    }
    if let Some(cover) = &item.cover {
        validate_resource(cover, builtin)?;
        if !cover.mime.starts_with("image/") || cover.bytes > 2 * 1024 * 1024 {
            return Err("封面需为不超过 2 MB 的图片".into());
        }
    }
    if item.assets.len() > 32
        || item.controls.len() > 16
        || (item.kind != "canvas" && (!item.assets.is_empty() || !item.controls.is_empty()))
    {
        return Err("附属资源或参数数量不合法".into());
    }
    let mut total = item.resource.bytes;
    for (key, resource) in &item.assets {
        if !safe_key(key) {
            return Err("动画资源名称不合法".into());
        }
        validate_resource(resource, builtin)?;
        total = total.checked_add(resource.bytes).ok_or("资源大小溢出")?;
    }
    if total > MAX_FILE {
        return Err("单张壁纸资源总量不能超过 256 MB".into());
    }
    let mut keys = HashSet::new();
    for control in &item.controls {
        if !safe_key(&control.key)
            || !keys.insert(&control.key)
            || control.label.is_empty()
            || control.label.len() > 100
        {
            return Err("动画参数名称不合法或重复".into());
        }
        match control.kind.as_str() {
            "range" => {
                let (min, max, step) = (
                    control.min.ok_or("滑块缺少最小值")?,
                    control.max.ok_or("滑块缺少最大值")?,
                    control.step.ok_or("滑块缺少步长")?,
                );
                if !min.is_finite()
                    || !max.is_finite()
                    || !step.is_finite()
                    || min >= max
                    || step <= 0.0
                    || step > max - min
                {
                    return Err("滑块范围不合法".into());
                }
            }
            "color" | "boolean" => {}
            "select"
                if !control.options.is_empty()
                    && control.options.len() <= 16
                    && control
                        .options
                        .iter()
                        .all(|s| !s.is_empty() && s.len() <= 160) => {}
            _ => return Err("不支持的动画参数类型".into()),
        }
        validate_value(control, &control.default)?;
    }
    Ok(())
}

fn validate_value(control: &Control, value: &Value) -> Result<(), String> {
    let valid = match control.kind.as_str() {
        "range" => value.as_f64().is_some_and(|v| {
            v.is_finite()
                && v >= control.min.unwrap_or(f64::INFINITY)
                && v <= control.max.unwrap_or(f64::NEG_INFINITY)
        }),
        "color" => value.as_str().is_some_and(color),
        "boolean" => value.is_boolean(),
        "select" => value
            .as_str()
            .is_some_and(|v| control.options.iter().any(|s| s == v)),
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        Err(format!("参数“{}”的值不合法", control.label))
    }
}

pub fn validate_options(item: &Wallpaper, mut options: Options) -> Result<Options, String> {
    // Effects belong to the desktop, not to a wallpaper; nothing a caller sends here is kept.
    options.effects = Value::Null;
    if !["cover", "contain", "fill"].contains(&options.fit.as_str())
        || !options.playback_rate.is_finite()
        || !(0.25..=2.0).contains(&options.playback_rate)
        || !color(&options.background)
    {
        return Err("壁纸显示设置不合法".into());
    }
    for key in options.values.keys() {
        if !item.controls.iter().any(|c| &c.key == key) {
            return Err("壁纸含未知参数".into());
        }
    }
    for control in &item.controls {
        let value = options
            .values
            .entry(control.key.clone())
            .or_insert_with(|| control.default.clone());
        validate_value(control, value)?;
    }
    Ok(options)
}

pub fn parse_catalog(text: &str) -> Result<Catalog, String> {
    if text.len() > MAX_JSON {
        return Err("壁纸清单不能超过 1 MB".into());
    }
    let catalog: Catalog =
        serde_json::from_str(text).map_err(|e| format!("清单格式不正确：{e}"))?;
    if catalog.schema_version != 1 || catalog.items.len() > 100 || catalog.revision.len() > 160 {
        return Err("不支持的清单版本或数量".into());
    }
    let mut ids = HashSet::new();
    for item in &catalog.items {
        validate_item(item, false)?;
        if !ids.insert(&item.id) {
            return Err("清单壁纸 ID 重复".into());
        }
    }
    Ok(catalog)
}

pub fn builtins() -> Vec<Wallpaper> {
    let catalog: Catalog =
        serde_json::from_str(include_str!("../../public/wallpapers/builtin/catalog.json"))
            .expect("内置目录必须有效");
    catalog.items
}
pub fn assets(item: &Wallpaper) -> Vec<&Resource> {
    let mut result = vec![&item.resource];
    result.extend(item.assets.values());
    result
}
pub fn asset_url(resource: &Resource) -> String {
    if let Some(filename) = resource.url.strip_prefix("builtin://") {
        return format!("/wallpapers/builtin/{filename}");
    }
    if cfg!(windows) {
        format!("http://wallpaper-asset.localhost/{}", resource.sha256)
    } else {
        format!("wallpaper-asset://localhost/{}", resource.sha256)
    }
}
pub fn cached(root: &Path, resource: &Resource) -> bool {
    if resource.url.starts_with("builtin://") {
        return true;
    }
    let path = crate::local_media::resource_path(root, resource);
    fs::metadata(path).is_ok_and(|m| m.is_file() && m.len() == resource.bytes)
}
pub fn read_catalog(root: &Path) -> Catalog {
    fs::read(root.join("catalog.json"))
        .ok()
        .and_then(|data| serde_json::from_slice(&data).ok())
        .unwrap_or_default()
}
pub fn all_items(root: &Path) -> Vec<Wallpaper> {
    let mut items = builtins();
    items.extend(
        read_catalog(root)
            .items
            .into_iter()
            .filter(|item| validate_item(item, false).is_ok()),
    );
    items.extend(crate::local_media::read(root).map(|c| c.items).unwrap_or_default());
    items
}
pub fn validate_stored(item: &Wallpaper) -> Result<(), String> {
    if crate::local_media::is_local(item) { crate::local_media::validate(item) }
    else { validate_item(item, item.id.starts_with("builtin-")) }
}
pub fn save_json(path: &Path, value: &impl Serialize) -> Result<(), String> {
    fs::create_dir_all(path.parent().ok_or("无效保存路径")?).map_err(|e| e.to_string())?;
    let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
    let result = (|| {
        fs::write(
            &temporary,
            serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        fs::rename(&temporary, path).map_err(|e| e.to_string())
    })();
    if result.is_err() {
        let _ = fs::remove_file(temporary);
    }
    result
}

fn client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(120))
        .connect_timeout(Duration::from_secs(10))
        .user_agent(concat!("XiaoBanDesktop/", env!("CARGO_PKG_VERSION")))
        .redirect(reqwest::redirect::Policy::custom(|attempt| {
            if attempt.previous().len() > 4 || network_url(attempt.url().as_str()).is_err() {
                attempt.error("资源重定向不合法")
            } else {
                attempt.follow()
            }
        }))
        .build()
        .map_err(|e| e.to_string())
}
pub fn fetch_json(value: &str) -> Result<Option<String>, String> {
    fetch_json_with_backoff(value, [Duration::from_millis(250), Duration::from_millis(750)])
}

fn metadata_client() -> Result<reqwest::blocking::Client, String> {
    static CLIENT: std::sync::Mutex<Option<reqwest::blocking::Client>> = std::sync::Mutex::new(None);
    let mut shared = CLIENT.lock().map_err(|_| "网络客户端初始化失败")?;
    if let Some(client) = shared.as_ref() {
        return Ok(client.clone());
    }
    // Only successful initialization is cached. Media downloads retain their own client/timeouts.
    let client = client()?;
    *shared = Some(client.clone());
    Ok(client)
}

fn fetch_json_with_backoff(value: &str, backoff: [Duration; 2]) -> Result<Option<String>, String> {
    network_url(value)?;
    let client = metadata_client()?;
    // Three 8-second attempts plus 1 second of backoff fit within 26 seconds.
    let deadline = std::time::Instant::now() + Duration::from_secs(26);
    let mut attempt = 0;
    let response = loop {
        let timeout = deadline.saturating_duration_since(std::time::Instant::now())
            .min(Duration::from_secs(8));
        let result = client.get(value)
            .timeout(timeout)
            .header("Accept-Encoding", "identity")
            .send();
        let retry = match &result {
            Ok(response) => matches!(response.status().as_u16(), 502 | 503 | 504),
            // Do not retry redirect failures, body reads or generic connection failures.
            Err(error) => error.is_connect() && error.is_timeout(),
        };
        if retry && attempt < backoff.len()
            && deadline.saturating_duration_since(std::time::Instant::now()) > backoff[attempt]
        {
            drop(result);
            std::thread::sleep(backoff[attempt]);
            attempt += 1;
            continue;
        }
        break result.map_err(|e| format!("网络连接失败：{e}"))?;
    };
    if response.status().as_u16() == 204 {
        return Ok(None);
    }
    let response = response.error_for_status().map_err(|e| format!("服务请求失败：{e}"))?;
    let mut data = Vec::new();
    response
        .take((MAX_JSON + 1) as u64)
        .read_to_end(&mut data)
        .map_err(|e| e.to_string())?;
    if data.len() > MAX_JSON {
        return Err("服务返回的清单超过 1 MB".into());
    }
    Ok(Some(
        String::from_utf8(data).map_err(|_| "清单需为 UTF-8 JSON")?,
    ))
}

pub fn verify_file(path: &Path, resource: &Resource) -> Result<bool, String> {
    let mut file = match fs::File::open(path) {
        Ok(file) => file,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(e.to_string()),
    };
    if file.metadata().map_err(|e| e.to_string())?.len() != resource.bytes {
        return Ok(false);
    }
    let mut digest = Sha256::new();
    let mut buffer = [0u8; 65536];
    loop {
        let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        digest.update(&buffer[..n]);
    }
    Ok(format!("{:x}", digest.finalize()) == resource.sha256)
}
pub fn download(root: &Path, resource: &Resource, progress: impl FnMut(u64)) -> Result<(), String> {
    download_cancellable(root, resource, &NEVER, progress)
}

fn download_cancellable(
    root: &Path,
    resource: &Resource,
    cancel: &AtomicBool,
    mut progress: impl FnMut(u64),
) -> Result<(), String> {
    if resource.url.starts_with("builtin://") {
        return Ok(());
    }
    if crate::local_media::resource_local(resource) {
        if resource.url != format!("local://{}", resource.sha256) || !hash_name(&resource.sha256) {
            return Err("本地资源标识不合法".into());
        }
        if !verify_file(&crate::local_media::resource_path(root, resource), resource)? {
            return Err("导入的壁纸副本不存在或已损坏，请重新导入原文件。".into());
        }
        progress(resource.bytes);
        return Ok(());
    }
    validate_resource(resource, false)?;
    let cache = root.join("cache");
    fs::create_dir_all(&cache).map_err(|e| e.to_string())?;
    let target = cache.join(&resource.sha256);
    if verify_file(&target, resource)? {
        progress(resource.bytes);
        return Ok(());
    }
    let temporary = cache.join(format!("{}.{}.part", resource.sha256, std::process::id()));
    let result = (|| {
        check_cancel(cancel)?;
        let mut response = client()?
            .get(&resource.url)
            .header("Accept-Encoding", "identity")
            .send()
            .map_err(|e| e.to_string())?
            .error_for_status()
            .map_err(|e| e.to_string())?;
        let mut file = fs::File::create(&temporary).map_err(|e| e.to_string())?;
        let mut digest = Sha256::new();
        let mut buffer = [0u8; 65536];
        let mut count = 0;
        loop {
            check_cancel(cancel)?;
            let n = response.read(&mut buffer).map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
            count += n as u64;
            if count > resource.bytes {
                return Err("下载资源超出声明大小".into());
            }
            digest.update(&buffer[..n]);
            file.write_all(&buffer[..n]).map_err(|e| e.to_string())?;
            progress(count);
        }
        if count != resource.bytes || format!("{:x}", digest.finalize()) != resource.sha256 {
            return Err("资源完整性校验失败，请联系壁纸提供方".into());
        }
        file.sync_all().map_err(|e| e.to_string())?;
        drop(file);
        if resource.mime == "application/javascript" {
            String::from_utf8(fs::read(&temporary).map_err(|e| e.to_string())?)
                .map_err(|_| "动画脚本需使用 UTF-8")?;
        }
        fs::rename(&temporary, target).map_err(|e| e.to_string())
    })();
    if result.is_err() {
        let _ = fs::remove_file(temporary);
    }
    result
}

static SOURCE_PARTS: AtomicU64 = AtomicU64::new(0);

/// Downloads a provider link the website handed out ("source first"). The manifest hash
/// describes the website's own copy, so nothing is compared against it: the real type is read
/// from the first bytes (Content-Type and the manifest mime are not trusted), the size is
/// capped, and sha256/bytes are computed locally. The file is then published as
/// `cache/<sha256>` like any verified download, so the library and offline use work the same.
#[cfg(test)]
pub fn download_source(
    root: &Path,
    url: &str,
    kind: &str,
    progress: impl FnMut(u64, Option<u64>),
) -> Result<Resource, String> {
    download_source_limited(root, url, kind, MAX_FILE, &NEVER, progress)
}

/// `download_source` that stops with `CANCELLED` once `cancel` is set; the partial file is removed.
pub fn download_source_cancellable(
    root: &Path,
    url: &str,
    kind: &str,
    cancel: &AtomicBool,
    progress: impl FnMut(u64, Option<u64>),
) -> Result<Resource, String> {
    download_source_limited(root, url, kind, MAX_FILE, cancel, progress)
}

fn download_source_limited(
    root: &Path,
    url: &str,
    kind: &str,
    limit: u64,
    cancel: &AtomicBool,
    mut progress: impl FnMut(u64, Option<u64>),
) -> Result<Resource, String> {
    network_url(url)?;
    if !["image", "video"].contains(&kind) {
        return Err("只有图片和视频可以直接使用来源链接".into());
    }
    let cache = root.join("cache");
    fs::create_dir_all(&cache).map_err(|e| e.to_string())?;
    let temporary = cache.join(format!(
        "source-{}-{}.part",
        std::process::id(),
        SOURCE_PARTS.fetch_add(1, Ordering::Relaxed)
    ));
    let result = (|| {
        check_cancel(cancel)?;
        let mut response = client()?
            .get(url)
            .header("Accept-Encoding", "identity")
            .send()
            .map_err(|e| format!("来源链接连接失败：{e}"))?
            .error_for_status()
            .map_err(|e| e.to_string())?;
        let total = response.content_length().filter(|n| *n > 0);
        if total.is_some_and(|n| n > limit) {
            return Err("来源文件超过大小上限".to_string());
        }
        // The type check needs the start of the file, and a network read may return fewer
        // bytes than the ftyp box, so fill the head buffer before looking at it.
        let mut buffer = vec![0u8; 65536];
        let mut head = 0;
        while head < buffer.len() {
            check_cancel(cancel)?;
            let n = response.read(&mut buffer[head..]).map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
            head += n;
        }
        let (found, mime) = crate::local_media::media_type(&buffer[..head])
            .map_err(|_| "来源文件不是受支持的图片或视频".to_string())?;
        if found != kind {
            return Err("来源文件类型与壁纸类型不一致".to_string());
        }
        let mut file = fs::File::create(&temporary).map_err(|e| e.to_string())?;
        let mut digest = Sha256::new();
        let mut count = 0u64;
        let mut n = head;
        while n > 0 {
            count += n as u64;
            if count > limit {
                return Err("来源文件超过大小上限".to_string());
            }
            digest.update(&buffer[..n]);
            file.write_all(&buffer[..n]).map_err(|e| e.to_string())?;
            progress(count, total);
            check_cancel(cancel)?;
            n = response.read(&mut buffer).map_err(|e| e.to_string())?;
        }
        file.sync_all().map_err(|e| e.to_string())?;
        drop(file);
        let resource = Resource {
            url: url.to_string(),
            sha256: format!("{:x}", digest.finalize()),
            bytes: count,
            mime: mime.into(),
        };
        validate_resource(&resource, false)?;
        // Cache files are immutable and content-addressed: keep an identical existing file.
        let target = cache.join(&resource.sha256);
        if verify_file(&target, &resource)? {
            fs::remove_file(&temporary).map_err(|e| e.to_string())?;
        } else {
            fs::rename(&temporary, target).map_err(|e| e.to_string())?;
        }
        Ok(resource)
    })();
    if result.is_err() {
        let _ = fs::remove_file(temporary);
    }
    result
}

pub fn prepare(app: &tauri::AppHandle, root: &Path, item: &Wallpaper) -> Result<(), String> {
    prepare_cancellable(app, root, item, &NEVER)
}

pub fn prepare_cancellable(
    app: &tauri::AppHandle,
    root: &Path,
    item: &Wallpaper,
    cancel: &AtomicBool,
) -> Result<(), String> {
    let list = assets(item);
    let total: u64 = list.iter().map(|r| r.bytes).sum();
    let mut completed = 0;
    for resource in list {
        let mut last = 0u64;
        download_cancellable(root, resource, cancel, |bytes| {
            if bytes.saturating_sub(last) >= 256 * 1024 || bytes == resource.bytes {
                last = bytes;
                let _ = app.emit_to(
                    "control",
                    "wallpaper-progress",
                    json!({"id":item.id,"received":completed+bytes,"total":total}),
                );
            }
        })?;
        completed += resource.bytes;
    }
    Ok(())
}

pub fn serve(
    app: &tauri::AppHandle,
    request: tauri::http::Request<Vec<u8>>,
) -> tauri::http::Response<Vec<u8>> {
    let error = |status| {
        tauri::http::Response::builder()
            .status(status)
            .body(Vec::new())
            .unwrap()
    };
    if request.uri().path() == "/sandbox" {
        return tauri::http::Response::builder().header("Content-Type","text/html; charset=utf-8")
            .header("Content-Security-Policy","default-src 'none'; script-src 'nonce-Y2FudmFzLXNhbmRib3g=' blob:; style-src 'unsafe-inline'; img-src blob: data:; media-src blob:; connect-src blob:; worker-src 'none'; child-src 'none'; object-src 'none'; base-uri 'none'; form-action 'none'")
            .body(include_bytes!("../../public/wallpapers/sandbox.html").to_vec()).unwrap();
    }
    let name = request.uri().path().trim_start_matches('/');
    if !hash_name(name) {
        return error(404);
    }
    let root = match crate::content::root(app) {
        Ok(root) => root,
        Err(_) => return error(404),
    };
    let mut items = all_items(&root);
    if let Ok(data) = fs::read(root.join("active.json")) {
        if let Ok(item) = serde_json::from_slice::<Wallpaper>(&data) {
            if validate_stored(&item).is_ok() { items.push(item); }
        }
    }
    let resource = items
        .iter()
        .flat_map(|item| assets(item).into_iter().chain(item.cover.iter()))
        .find(|r| r.sha256 == name);
    let Some(resource) = resource else {
        return error(404);
    };
    let mut file = match fs::File::open(crate::local_media::resource_path(&root, resource)) {
        Ok(f) => f,
        Err(_) => return error(404),
    };
    let length = match file.metadata() {
        Ok(m) if m.len() == resource.bytes => m.len(),
        _ => return error(404),
    };
    let range = request.headers().get("range").and_then(|v| v.to_str().ok());
    let (start, end, partial) = match playback_range(range, length, &resource.mime) {
        Ok(v) => v,
        Err(_) => {
            return tauri::http::Response::builder()
                .status(416)
                .header("Content-Range", format!("bytes */{length}"))
                .body(Vec::new())
                .unwrap()
        }
    };
    if file.seek(SeekFrom::Start(start)).is_err() {
        return error(500);
    }
    let mut body = Vec::new();
    if file.take(end - start + 1).read_to_end(&mut body).is_err() {
        return error(500);
    }
    let mut response = tauri::http::Response::builder()
        .status(if partial { 206 } else { 200 })
        .header("Content-Type", &resource.mime)
        .header("Content-Length", body.len())
        .header("Accept-Ranges", "bytes")
        .header("Access-Control-Allow-Origin", "*")
        .header("X-Content-Type-Options", "nosniff")
        .header("Cache-Control", "public, max-age=31536000, immutable");
    if partial {
        response = response.header("Content-Range", format!("bytes {start}-{end}/{length}"));
    }
    response.body(body).unwrap()
}

fn playback_range(header: Option<&str>, length: u64, mime: &str) -> Result<(u64, u64, bool), String> {
    let (start, end, partial) = byte_range(header, length)?;
    // Tauri custom-protocol responses own a Vec, so limit video chunks instead
    // of allocating a potentially 1 GB response for a bytes=0- request.
    if mime.starts_with("video/") && end - start >= 8 * 1024 * 1024 {
        Ok((start, start + 8 * 1024 * 1024 - 1, true))
    } else { Ok((start, end, partial)) }
}

pub fn byte_range(header: Option<&str>, length: u64) -> Result<(u64, u64, bool), String> {
    if length == 0 {
        return Err("空资源".into());
    }
    let Some(header) = header else {
        return Ok((0, length - 1, false));
    };
    let value = header.strip_prefix("bytes=").ok_or("不支持的范围")?;
    let (a, b) = value.split_once('-').ok_or("范围不合法")?;
    if a.is_empty() {
        let suffix: u64 = b.parse().map_err(|_| "范围不合法")?;
        if suffix == 0 {
            return Err("范围不合法".into());
        }
        return Ok((length.saturating_sub(suffix), length - 1, true));
    }
    let start: u64 = a.parse().map_err(|_| "范围不合法")?;
    let end = if b.is_empty() {
        length - 1
    } else {
        b.parse::<u64>().map_err(|_| "范围不合法")?.min(length - 1)
    };
    if start >= length || end < start {
        return Err("范围超出资源".into());
    }
    Ok((start, end, true))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct UpdateManifest {
    schema_version: u32,
    version: String,
    #[serde(default)]
    notes: String,
    #[serde(default)]
    published_at: String,
    platforms: BTreeMap<String, UpdatePackage>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UpdatePackage {
    url: String,
    #[serde(default)]
    sha256: Option<String>,
    #[serde(default)]
    bytes: Option<u64>,
    #[serde(default, rename = "mime")]
    _mime: Option<String>,
}
pub fn update_result(text: &str, current: &str, platform: &str) -> Result<Value, String> {
    let manifest: UpdateManifest =
        serde_json::from_str(text).map_err(|e| format!("更新清单不合法：{e}"))?;
    if manifest.schema_version != 1
        || manifest.notes.len() > 16000
        || manifest.published_at.len() > 64
    {
        return Err("不支持的更新清单版本".into());
    }
    let latest = semver::Version::parse(&manifest.version).map_err(|_| "服务版本号不合法")?;
    let running = semver::Version::parse(current).map_err(|_| "当前版本号不合法")?;
    let artifact = manifest.platforms.get(platform);
    if latest > running && artifact.is_none() {
        return Err("新版本尚未提供当前平台的安装包".into());
    }
    if let Some(artifact) = artifact {
        network_url(&artifact.url)?;
        if artifact.sha256.as_deref().is_some_and(|h| !hash_name(h)) || artifact.bytes.is_some_and(|n| n == 0)
        {
            return Err("更新包校验信息不合法".into());
        }
    }
    Ok(
        json!({"available":latest>running,"version":manifest.version,"notes":manifest.notes,"publishedAt":manifest.published_at,"downloadUrl":artifact.map(|r|&r.url)}),
    )
}

/// Loopback HTTP server for tests; no internet is involved.
#[cfg(test)]
pub(crate) mod test_http {
    use std::{
        collections::HashMap,
        io::{Read, Write},
        net::TcpListener,
        sync::{Arc, Mutex},
    };

    pub fn temp_root(name: &str) -> std::path::PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static N: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "xiaoban-source-test-{name}-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        root
    }

    pub enum Reply {
        /// status, content type, body; sends Content-Length.
        Body(u16, &'static str, Vec<u8>),
        /// Same, but without Content-Length (body ends when the connection closes).
        Streamed(u16, &'static str, Vec<u8>),
        /// Per-route responses consumed in order, then repeats the last response.
        Sequence(Mutex<std::collections::VecDeque<(u16, Vec<u8>)>>),
        /// Closes the connection without answering.
        Drop,
    }
    pub struct Server {
        pub base: String,
        pub hits: Arc<Mutex<Vec<String>>>,
    }
    pub fn serve(routes: Vec<(&'static str, Reply)>) -> Server {
        // reqwest reads the system proxy environment; loopback test traffic must never use it.
        static DIRECT: std::sync::Once = std::sync::Once::new();
        DIRECT.call_once(|| {
            std::env::set_var("NO_PROXY", "127.0.0.1,localhost");
            std::env::set_var("no_proxy", "127.0.0.1,localhost");
        });
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://127.0.0.1:{}", listener.local_addr().unwrap().port());
        let routes: Arc<HashMap<&'static str, Reply>> = Arc::new(routes.into_iter().collect());
        let hits = Arc::new(Mutex::new(Vec::new()));
        let log = hits.clone();
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                let (routes, log) = (routes.clone(), log.clone());
                std::thread::spawn(move || {
                    let mut request = Vec::new();
                    let mut chunk = [0u8; 1024];
                    while !request.windows(4).any(|w| w == b"\r\n\r\n") {
                        match stream.read(&mut chunk) {
                            Ok(0) | Err(_) => return,
                            Ok(n) => request.extend_from_slice(&chunk[..n]),
                        }
                    }
                    let text = String::from_utf8_lossy(&request).to_string();
                    let path = text
                        .split_whitespace()
                        .nth(1)
                        .unwrap_or("/")
                        .to_string();
                    log.lock().unwrap_or_else(|e| e.into_inner()).push(path.clone());
                    let route = path.split('?').next().unwrap_or("/");
                    let sequence_body;
                    let (status, kind, body, length) = match routes.get(route) {
                        Some(Reply::Body(s, k, b)) => (*s, *k, b, true),
                        Some(Reply::Streamed(s, k, b)) => (*s, *k, b, false),
                        Some(Reply::Sequence(replies)) => {
                            let mut replies = replies.lock().unwrap_or_else(|e| e.into_inner());
                            let (status, body) = if replies.len() > 1 {
                                replies.pop_front().unwrap()
                            } else {
                                replies.front().unwrap().clone()
                            };
                            sequence_body = body;
                            (status, "application/json", &sequence_body, true)
                        }
                        Some(Reply::Drop) => return,
                        None => (404, "text/plain", &Vec::new(), true),
                    };
                    let mut head =
                        format!("HTTP/1.1 {status} X\r\nContent-Type: {kind}\r\nConnection: close\r\n");
                    if length {
                        head += &format!("Content-Length: {}\r\n", body.len());
                    }
                    let _ = stream.write_all(format!("{head}\r\n").as_bytes());
                    let _ = stream.write_all(body);
                });
            }
        });
        Server { base, hits }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_http::{serve, temp_root, Reply};

    fn metadata_sequence(statuses: &[u16]) -> test_http::Server {
        serve(vec![("/json", Reply::Sequence(std::sync::Mutex::new(
            statuses.iter().map(|status| (*status, b"{\"ok\":true}".to_vec())).collect()
        )))])
    }

    fn metadata_get(server: &test_http::Server) -> Result<Option<String>, String> {
        fetch_json_with_backoff(&format!("{}/json", server.base), [Duration::ZERO; 2])
    }

    #[test]
    fn metadata_recovers_from_a_transient_gateway_response() {
        for status in [502, 503, 504] {
            let server = metadata_sequence(&[status, 200]);
            assert_eq!(metadata_get(&server).unwrap().as_deref(), Some("{\"ok\":true}"));
            assert_eq!(server.hits.lock().unwrap_or_else(|e| e.into_inner()).len(), 2);
        }
    }

    #[test]
    fn metadata_gateway_retries_are_bounded_and_preserve_failure_status() {
        let server = metadata_sequence(&[502]);
        let error = metadata_get(&server).unwrap_err();
        assert!(error.contains("服务请求失败") && error.contains("502"), "{error}");
        assert_eq!(server.hits.lock().unwrap_or_else(|e| e.into_inner()).len(), 3);
    }

    #[test]
    fn metadata_does_not_retry_client_errors_or_oversized_bodies() {
        let missing = metadata_sequence(&[404, 200]);
        assert!(metadata_get(&missing).unwrap_err().contains("404"));
        assert_eq!(missing.hits.lock().unwrap_or_else(|e| e.into_inner()).len(), 1);
        for reply in [
            Reply::Body(200, "application/json", vec![b' '; MAX_JSON + 1]),
            Reply::Streamed(200, "application/json", vec![b' '; MAX_JSON + 1]),
        ] {
            let server = serve(vec![("/json", reply)]);
            assert!(metadata_get(&server).unwrap_err().contains("超过 1 MB"));
            assert_eq!(server.hits.lock().unwrap_or_else(|e| e.into_inner()).len(), 1);
        }
    }

    #[test]
    fn metadata_requests_share_client_without_sharing_retry_state() {
        let servers: Vec<_> = (0..8).map(|_| metadata_sequence(&[502, 200])).collect();
        std::thread::scope(|scope| {
            let handles: Vec<_> = servers.iter().map(|server| scope.spawn(move || {
                assert!(metadata_get(server).unwrap().is_some());
            })).collect();
            for handle in handles { handle.join().unwrap(); }
        });
        for server in servers { assert_eq!(server.hits.lock().unwrap_or_else(|e| e.into_inner()).len(), 2); }
    }

    #[test]
    fn metadata_preserves_no_content_and_does_not_retry_invalid_utf8() {
        let empty = metadata_sequence(&[204]);
        assert_eq!(metadata_get(&empty).unwrap(), None);
        let invalid = serve(vec![("/json", Reply::Body(200, "application/json", vec![255]))]);
        assert!(metadata_get(&invalid).unwrap_err().contains("UTF-8"));
        assert_eq!(invalid.hits.lock().unwrap_or_else(|e| e.into_inner()).len(), 1);
    }

    #[test]
    fn a_wallpaper_still_reads_an_old_effects_key_but_never_writes_or_keeps_one() {
        let old: Options = serde_json::from_value(json!({"fit":"contain","effects":{"rain":{"enabled":true}}})).unwrap();
        assert_eq!(old.effects["rain"]["enabled"], true);
        let fresh: Options = serde_json::from_value(json!({"fit":"contain"})).unwrap();
        assert!(fresh.effects.is_null());
        assert!(serde_json::to_value(&old).unwrap().get("effects").is_none());
        for kind in ["image", "video", "canvas"] {
            let mut item = builtins().remove(0);
            item.kind = kind.into();
            let valid = validate_options(&item, old.clone()).unwrap();
            assert!(valid.effects.is_null(), "{kind}");
            assert!(!String::from_utf8(serde_json::to_vec(&valid).unwrap()).unwrap().contains("effects"));
        }
    }

    fn png(tag: &str) -> Vec<u8> {
        [b"\x89PNG\r\n\x1a\n".as_slice(), tag.as_bytes()].concat()
    }
    fn mp4() -> Vec<u8> {
        [b"\0\0\0\x18ftypisom\0\0\0\0isomiso2".as_slice(), &[7u8; 100]].concat()
    }
    fn hex(data: &[u8]) -> String {
        format!("{:x}", Sha256::digest(data))
    }
    fn leftovers(root: &Path) -> Vec<String> {
        fs::read_dir(root.join("cache"))
            .map(|d| d.map(|e| e.unwrap().file_name().to_string_lossy().to_string()).collect())
            .unwrap_or_default()
    }

    #[test]
    fn source_download_registers_the_file_under_its_own_hash() {
        let body = png("provider-file");
        // Content-Type is wrong on purpose: only the first bytes decide the type.
        let server = serve(vec![("/a.png", Reply::Body(200, "text/html", body.clone()))]);
        let root = temp_root("ok");
        let url = format!("{}/a.png", server.base);
        let mut seen = 0;
        let resource = download_source(&root, &url, "image", |n, total| {
            seen = n;
            assert_eq!(total, Some(body.len() as u64));
        })
        .unwrap();
        assert_eq!(seen, body.len() as u64);
        assert_eq!(resource.sha256, hex(&body));
        assert_eq!(resource.bytes, body.len() as u64);
        assert_eq!(resource.mime, "image/png");
        assert_eq!(resource.url, url);
        assert!(cached(&root, &resource));
        assert!(verify_file(&root.join("cache").join(&resource.sha256), &resource).unwrap());
        validate_resource(&resource, false).unwrap();
        // A second download keeps the immutable file and leaves no temporary files.
        let again = download_source(&root, &url, "image", |_, _| {}).unwrap();
        assert_eq!(again.sha256, resource.sha256);
        assert_eq!(leftovers(&root), vec![resource.sha256.clone()]);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn source_video_is_detected_from_bytes() {
        let server = serve(vec![("/v", Reply::Body(200, "application/octet-stream", mp4()))]);
        let root = temp_root("video");
        let url = format!("{}/v", server.base);
        assert_eq!(download_source(&root, &url, "video", |_, _| {}).unwrap().mime, "video/mp4");
        // A video must not be accepted as an image wallpaper, nor the other way around.
        assert!(download_source(&root, &url, "image", |_, _| {}).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn source_download_rejects_unusable_responses_without_leaving_files() {
        let server = serve(vec![
            ("/html", Reply::Body(200, "image/png", b"<html>blocked</html>".to_vec())),
            ("/svg", Reply::Body(200, "image/svg+xml", b"<svg onload='x'/>".to_vec())),
            ("/empty", Reply::Body(200, "image/png", Vec::new())),
            ("/missing", Reply::Body(404, "text/plain", b"no".to_vec())),
            ("/drop", Reply::Drop),
            ("/png", Reply::Body(200, "image/png", png("x"))),
        ]);
        let root = temp_root("bad");
        for path in ["html", "svg", "empty", "missing", "drop"] {
            let url = format!("{}/{path}", server.base);
            assert!(download_source(&root, &url, "image", |_, _| {}).is_err(), "{path}");
        }
        assert!(download_source(&root, &format!("{}/png", server.base), "canvas", |_, _| {}).is_err());
        assert!(leftovers(&root).is_empty());
        // Only https (or loopback http) links are accepted.
        assert!(download_source(&root, "http://example.test/a.png", "image", |_, _| {}).is_err());
        assert!(download_source(&root, "file:///etc/passwd", "image", |_, _| {}).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn oversized_source_is_rejected_by_header_and_by_stream() {
        let big = [png("big"), vec![1u8; 5000]].concat();
        let server = serve(vec![
            ("/declared", Reply::Body(200, "image/png", big.clone())),
            ("/streamed", Reply::Streamed(200, "image/png", big.clone())),
            ("/exact", Reply::Streamed(200, "image/png", big.clone())),
        ]);
        let root = temp_root("big");
        for path in ["declared", "streamed"] {
            let url = format!("{}/{path}", server.base);
            assert!(download_source_limited(&root, &url, "image", 1000, &NEVER, |_, _| {}).is_err());
        }
        assert!(leftovers(&root).is_empty());
        // The limit is inclusive.
        let url = format!("{}/exact", server.base);
        assert!(download_source_limited(&root, &url, "image", big.len() as u64, &NEVER, |_, _| {}).is_ok());
        assert_eq!(MAX_FILE, 256 * 1024 * 1024);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn a_cancelled_download_stops_with_the_cancel_error_and_leaves_no_files() {
        let big = [png("big"), vec![7u8; 400 * 1024]].concat();
        let server = serve(vec![("/big", Reply::Body(200, "image/png", big.clone()))]);
        let root = temp_root("cancel");
        let url = format!("{}/big", server.base);
        let cancel = AtomicBool::new(false);
        let error = download_source_cancellable(&root, &url, "image", &cancel, |received, _| {
            if received > 0 {
                cancel.store(true, Ordering::Relaxed);
            }
        })
        .unwrap_err();
        assert_eq!(error, CANCELLED);
        let resource = Resource { url: url.clone(), sha256: hex(&big), bytes: big.len() as u64, mime: "image/png".into() };
        let cancel = AtomicBool::new(false);
        let error = download_cancellable(&root, &resource, &cancel, |received| {
            if received > 0 {
                cancel.store(true, Ordering::Relaxed);
            }
        })
        .unwrap_err();
        assert_eq!(error, CANCELLED);
        assert!(leftovers(&root).is_empty(), "{:?}", leftovers(&root));
        // Set before the request: nothing is fetched at all.
        let hits = server.hits.lock().unwrap_or_else(|e| e.into_inner()).len();
        assert_eq!(download_source_cancellable(&root, &url, "image", &cancel, |_, _| {}).unwrap_err(), CANCELLED);
        assert_eq!(server.hits.lock().unwrap_or_else(|e| e.into_inner()).len(), hits);
        // Without cancellation the same file still downloads.
        download_cancellable(&root, &resource, &NEVER, |_| {}).unwrap();
        assert!(cached(&root, &resource));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn large_video_requests_have_bounded_response_memory() {
        let chunk = 8 * 1024 * 1024;
        assert_eq!(playback_range(Some("bytes=0-"), 1024 * 1024 * 1024, "video/mp4").unwrap(), (0, chunk-1, true));
        assert_eq!(playback_range(None, chunk*3, "video/webm").unwrap(), (0, chunk-1, true));
        assert_eq!(playback_range(Some("bytes=100-199"), chunk*3, "video/mp4").unwrap(), (100,199,true));
        assert_eq!(playback_range(None, 1024, "video/mp4").unwrap(), (0,1023,false));
    }
    #[test]
    fn scheme_and_resource_paths_are_constrained() {
        assert!(network_url("https://example.test/catalog.json").is_ok());
        assert!(network_url("http://127.0.0.1:8123/a").is_ok());
        for value in [
            "file:///etc/passwd",
            "javascript:alert(1)",
            "https://user:pass@host.test/a",
            "http://example.test/a",
        ] {
            assert!(network_url(value).is_err());
        }
        assert!(!hash_name("../preferences.json"));
        assert!(!safe_key("__proto__"));
    }
    #[test]
    fn all_builtin_wallpapers_are_valid() {
        for item in builtins() {
            validate_item(&item, true).unwrap();
            validate_options(&item, Options::default()).unwrap();
        }
    }
    #[test]
    fn video_ranges_support_seeking_and_reject_invalid_ranges() {
        assert_eq!(
            byte_range(Some("bytes=10-19"), 100).unwrap(),
            (10, 19, true)
        );
        assert_eq!(byte_range(Some("bytes=-10"), 100).unwrap(), (90, 99, true));
        assert_eq!(byte_range(Some("bytes=80-"), 100).unwrap(), (80, 99, true));
        assert!(byte_range(Some("bytes=200-"), 100).is_err());
        assert!(byte_range(Some("bytes=1-2,3-4"), 100).is_err());
    }
    #[test]
    fn update_versions_are_compared_semantically() {
        let text=json!({"schemaVersion":1,"version":"0.1.10","platforms":{"windows-x86_64":{"url":"https://example.test/app.zip","sha256":"a".repeat(64),"bytes":20,"mime":"application/zip"}}}).to_string();
        assert_eq!(
            update_result(&text, "0.1.9", "windows-x86_64").unwrap()["available"],
            true
        );
        assert_eq!(
            update_result(&text, "0.2.0", "windows-x86_64").unwrap()["available"],
            false
        );
        assert!(update_result(&text, "0.1.9", "darwin-aarch64").is_err());
    }
    #[test]
    fn website_download_links_need_no_fake_package_hash() {
        let text=json!({"schemaVersion":1,"version":"1.10.0","platforms":{"windows-x86_64":{"url":"https://wallpaper.xiangban.xyz/download/app.zip"}}}).to_string();
        assert_eq!(update_result(&text,"1.9.0","windows-x86_64").unwrap()["available"],true);
        let invalid=text.replace("https://wallpaper.xiangban.xyz/download/app.zip","javascript:alert(1)");
        assert!(update_result(&invalid,"1.9.0","windows-x86_64").is_err());
    }
}
