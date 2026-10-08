use crate::error::{ApiError, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashSet};

pub const MAX_RESOURCE: u64 = 256 * 1024 * 1024;
pub const MAX_CATALOG: usize = 1024 * 1024;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Resource {
    pub url: String,
    #[serde(default)]
    pub sha256: String,
    #[serde(default)]
    pub bytes: u64,
    pub mime: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Control {
    pub key: String,
    pub label: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub default: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step: Option<f64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub options: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover: Option<Resource>,
    #[serde(default)]
    pub assets: BTreeMap<String, Resource>,
    #[serde(default)]
    pub controls: Vec<Control>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Release {
    pub schema_version: u32,
    pub version: String,
    #[serde(default)]
    pub notes: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub published_at: Option<String>,
    #[serde(default)]
    pub platforms: BTreeMap<String, Resource>,
}
pub fn identifier(s: &str) -> bool {
    !s.is_empty()
        && ![".", ".."].contains(&s)
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
}
pub fn safe_key(s: &str) -> bool {
    identifier(s) && !["__proto__", "constructor", "prototype"].contains(&s)
}
pub fn hash(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}
pub fn color(s: &str) -> bool {
    s.len() == 7 && s.starts_with('#') && s.as_bytes()[1..].iter().all(u8::is_ascii_hexdigit)
}
pub fn resource_url(value: &str, allow_loopback: bool) -> Result<url::Url> {
    let url = url::Url::parse(value).map_err(|_| ApiError::bad("资源地址格式不正确"))?;
    let localhost = url
        .host_str()
        .is_some_and(|h| h == "localhost" || h == "127.0.0.1" || h == "[::1]");
    if value.len() > 4096
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
        || url.host_str().is_none()
        || !(url.scheme() == "https" || (allow_loopback && localhost && url.scheme() == "http"))
    {
        return Err(ApiError::bad(
            "资源使用 HTTPS 外部地址，不接受文件路径、账号密码或片段",
        ));
    }
    Ok(url)
}
fn validate_resource(r: &Resource, published: bool, limit: u64, loopback: bool) -> Result<()> {
    if !r.url.is_empty() {
        resource_url(&r.url, loopback)?;
    }
    if (!r.sha256.is_empty() && !hash(&r.sha256)) || r.bytes > limit {
        return Err(ApiError::bad("SHA-256 或资源大小不合法"));
    }
    if published && (r.url.is_empty() || !hash(&r.sha256) || r.bytes == 0) {
        return Err(ApiError::bad(
            "发布前需要下载地址、SHA-256 与准确文件字节数，可先探测资源信息",
        ));
    }
    Ok(())
}
pub fn validate_wallpaper(w: &Wallpaper, published: bool, loopback: bool) -> Result<()> {
    if !identifier(&w.id)
        || w.id == "companion"
        || w.id.starts_with("builtin-")
        || w.id.starts_with("local-")
    {
        return Err(ApiError::bad("ID 不合法或为桌面客户端保留名称"));
    }
    semver::Version::parse(&w.version).map_err(|_| ApiError::bad("版本需为 SemVer，如 1.0.0"))?;
    if w.title.trim().is_empty()
        || w.title.len() > 160
        || w.description.len() > 2048
        || w.author.len() > 160
    {
        return Err(ApiError::bad("标题、介绍或作者长度不合法"));
    }
    let valid_mime = match w.kind.as_str() {
        "image" => ["image/png", "image/jpeg", "image/webp", "image/avif"]
            .contains(&w.resource.mime.as_str()),
        "video" => w.resource.mime == "video/mp4",
        "canvas" => w.resource.mime == "application/javascript",
        _ => false,
    };
    if !valid_mime {
        return Err(ApiError::bad("壁纸类型与 MIME 不匹配"));
    }
    validate_resource(
        &w.resource,
        published,
        if w.kind == "canvas" {
            2 * 1024 * 1024
        } else {
            MAX_RESOURCE
        },
        loopback,
    )?;
    if let Some(cover) = &w.cover {
        if !["image/png", "image/jpeg", "image/webp", "image/avif"].contains(&cover.mime.as_str()) {
            return Err(ApiError::bad("封面需要图片 MIME"));
        }
        validate_resource(cover, published, 2 * 1024 * 1024, loopback)?;
    }
    if w.assets.len() > 32
        || w.controls.len() > 16
        || (w.kind != "canvas" && (!w.assets.is_empty() || !w.controls.is_empty()))
    {
        return Err(ApiError::bad("附属资源或动画参数数量不合法"));
    }
    let mut total = w.resource.bytes;
    for (key, r) in &w.assets {
        if !safe_key(key)
            || ![
                "image/png",
                "image/jpeg",
                "image/webp",
                "image/avif",
                "video/mp4",
                "application/javascript",
                "application/json",
                "application/octet-stream",
            ]
            .contains(&r.mime.as_str())
        {
            return Err(ApiError::bad("附属资源名称或 MIME 不合法"));
        }
        validate_resource(
            r,
            published,
            if r.mime == "application/javascript" {
                2 * 1024 * 1024
            } else {
                MAX_RESOURCE
            },
            loopback,
        )?;
        total = total
            .checked_add(r.bytes)
            .ok_or_else(|| ApiError::bad("资源大小溢出"))?;
    }
    if total > MAX_RESOURCE {
        return Err(ApiError::bad("单张壁纸入口与附属资源总量超过 256 MiB"));
    }
    let mut keys = HashSet::new();
    for c in &w.controls {
        let key = c.key.as_str();
        let label = c.label.as_str();
        if !safe_key(key) || !keys.insert(key) || label.is_empty() || label.len() > 100 {
            return Err(ApiError::bad("动画参数名称或标签不合法"));
        }
        let default = &c.default;
        let valid = match c.kind.as_str() {
            "range" => match (c.min, c.max, c.step, default.as_f64()) {
                (Some(min), Some(max), Some(step), Some(v)) => {
                    min < max && step > 0. && step <= max - min && v >= min && v <= max
                }
                _ => false,
            },
            "color" => default.as_str().is_some_and(color),
            "boolean" => default.is_boolean(),
            "select" => {
                !c.options.is_empty()
                    && c.options.len() <= 16
                    && c.options.iter().all(|s| !s.is_empty() && s.len() <= 160)
                    && default
                        .as_str()
                        .is_some_and(|s| c.options.iter().any(|v| v == s))
            }
            _ => false,
        };
        if !valid {
            return Err(ApiError::bad("动画参数类型、范围或默认值不合法"));
        }
    }
    Ok(())
}
pub fn validate_release(r: &Release, published: bool, loopback: bool) -> Result<()> {
    if r.schema_version != 1 || r.notes.len() > 16000 || (published && r.platforms.is_empty()) {
        return Err(ApiError::bad(
            "发布清单不合法，发布时至少提供一个平台安装包",
        ));
    }
    semver::Version::parse(&r.version).map_err(|_| ApiError::bad("应用版本需为 SemVer"))?;
    if let Some(date) = &r.published_at {
        chrono::DateTime::parse_from_rfc3339(date)
            .map_err(|_| ApiError::bad("发布日期需为 RFC3339"))?;
    }
    for (key, resource) in &r.platforms {
        if !["windows-x86_64", "darwin-aarch64", "darwin-x86_64"].contains(&key.as_str())
            || ![
                "application/zip",
                "application/octet-stream",
                "application/x-apple-diskimage",
            ]
            .contains(&resource.mime.as_str())
        {
            return Err(ApiError::bad("安装包平台或 MIME 不支持"));
        }
        validate_resource(resource, published, 60 * 1024 * 1024, loopback)?;
    }
    Ok(())
}
pub fn parse_import(value: Value) -> Result<Vec<Wallpaper>> {
    let entries = match value {
        Value::Array(a) => a,
        Value::Object(mut o) if o.contains_key("items") => {
            if o.get("schemaVersion").is_some_and(|v| v != &json!(1)) {
                return Err(ApiError::bad("清单版本不支持"));
            }
            o.remove("items")
                .and_then(|v| v.as_array().cloned())
                .ok_or_else(|| ApiError::bad("items 应为数组"))?
        }
        v => vec![v],
    };
    if entries.is_empty() || entries.len() > 100 {
        return Err(ApiError::bad("一次导入 1–100 张壁纸"));
    }
    let mut result = Vec::new();
    let mut ids = HashSet::new();
    for mut value in entries {
        if value.get("resource").is_none() {
            let o = value
                .as_object_mut()
                .ok_or_else(|| ApiError::bad("壁纸应为 JSON 对象"))?;
            let url = o
                .remove("downloadUrl")
                .or_else(|| o.remove("url"))
                .unwrap_or(json!(""));
            let kind = o
                .remove("kind")
                .or_else(|| o.remove("type"))
                .unwrap_or(json!("image"));
            let mime = o.remove("mime").unwrap_or_else(|| {
                json!(if kind == "video" {
                    "video/mp4"
                } else if kind == "canvas" {
                    "application/javascript"
                } else if url.as_str().is_some_and(|u| u
                    .to_lowercase()
                    .split('?')
                    .next()
                    .unwrap_or("")
                    .ends_with(".png"))
                {
                    "image/png"
                } else {
                    "image/jpeg"
                })
            });
            let resource = json!({"url":url,"mime":mime,"sha256":o.remove("sha256").unwrap_or(json!("")),"bytes":o.remove("bytes").unwrap_or(json!(0))});
            o.insert("resource".into(), resource);
            o.insert("kind".into(), kind);
            if let Some(url) = o
                .remove("coverUrl")
                .filter(|v| v.as_str().is_some_and(|s| !s.is_empty()))
            {
                let cover = json!({"url":url,"sha256":o.remove("coverSha256").unwrap_or(json!("")),"bytes":o.remove("coverBytes").unwrap_or(json!(0)),"mime":o.remove("coverMime").unwrap_or(json!("image/jpeg"))});
                o.insert("cover".into(), cover);
            }
        }
        let o = value
            .as_object_mut()
            .ok_or_else(|| ApiError::bad("壁纸应为对象"))?;
        o.entry("id")
            .or_insert_with(|| json!(format!("wall-{}", uuid::Uuid::new_v4())));
        o.entry("version").or_insert(json!("1.0.0"));
        let wallpaper: Wallpaper = serde_json::from_value(value)
            .map_err(|e| ApiError::bad(format!("JSON 格式不正确：{e}")))?;
        if !ids.insert(wallpaper.id.clone()) {
            return Err(ApiError::bad("导入中存在重复 ID"));
        }
        result.push(wallpaper);
    }
    Ok(result)
}
