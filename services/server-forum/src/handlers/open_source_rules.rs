//! 精选栏目的封面协议。不能用于新闻、作品、讨论等其他板块。
use crate::error::{AppError, AppResult};
use serde_json::{json, Value};
use std::{collections::HashSet, sync::OnceLock};

pub fn manifest() -> &'static Value {
    static RULES: OnceLock<Value> = OnceLock::new();
    RULES.get_or_init(|| {
        serde_json::from_str(include_str!("../../editorial/open-source-cover-rules.json"))
            .expect("valid curated cover rules")
    })
}
fn bad(message: impl Into<String>) -> AppError {
    AppError::BadRequest(message.into())
}
pub fn hash(name: &str) -> u32 {
    name.to_ascii_lowercase()
        .bytes()
        .fold(0u32, |n, byte| n.wrapping_mul(31).wrapping_add(byte as u32))
}
pub fn https(raw: &str) -> bool {
    if raw.len() > 2048 || raw.chars().any(char::is_control) {
        return false;
    }
    reqwest::Url::parse(raw).is_ok_and(|u| {
        u.scheme() == "https"
            && u.host_str().is_some()
            && u.username().is_empty()
            && u.password().is_none()
            && u.port().is_none()
    })
}
pub fn asset_url(raw: &str) -> bool {
    if raw.is_empty() || https(raw) {
        return true;
    }
    if let Some(id) = raw.strip_prefix("/api/open-source/assets/") {
        return uuid::Uuid::parse_str(id).is_ok();
    }
    raw.strip_prefix("/open-source-media/").is_some_and(|name| {
        !name.is_empty()
            && !name.contains("..")
            && name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"-_ .".contains(&b))
            && !name.contains(' ')
    })
}
fn text(project: &mut Value, key: &str, max: usize, required: bool) -> AppResult<()> {
    let raw = project[key].as_str().unwrap_or_default().trim();
    if raw.chars().count() > max || raw.contains('\0') || required && raw.is_empty() {
        return Err(bad(format!("{key} 必填或超过 {max} 字限制")));
    }
    project[key] = json!(raw);
    Ok(())
}
fn strings(project: &mut Value, key: &str, count: usize, length: usize) -> AppResult<()> {
    let values = project[key].as_array().cloned().unwrap_or_default();
    if values.len() > count {
        return Err(bad(format!("{key} 最多 {count} 项")));
    }
    let mut out = Vec::new();
    for value in values {
        let raw = value
            .as_str()
            .ok_or_else(|| bad(format!("{key} 必须是文字列表")))?
            .trim();
        if raw.chars().count() > length || raw.contains('\0') {
            return Err(bad(format!("{key} 内容过长")));
        }
        if !raw.is_empty() && !out.iter().any(|v: &String| v == raw) {
            out.push(raw.to_string());
        }
    }
    project[key] = json!(out);
    Ok(())
}

pub fn resolve(project: &mut Value) -> AppResult<()> {
    let rules = manifest();
    let templates = rules["templates"].as_array().unwrap();
    let legacy = match project["cover_style"].as_str().unwrap_or("") {
        "screen" => "screenshot",
        "photo" => "photo",
        _ => project["cover_layout"].as_str().unwrap_or("auto"),
    };
    let requested = project["cover_template"]
        .as_str()
        .filter(|s| !s.is_empty())
        .unwrap_or(legacy)
        .to_string();
    if requested != "auto" && !templates.iter().any(|t| t["id"] == requested) {
        return Err(bad("封面模板不属于精选板块"));
    }
    let has_media = project["cover_url"].as_str().is_some_and(|v| !v.is_empty());
    let picked = if requested == "auto" {
        rules["rules"]
            .as_array()
            .unwrap()
            .iter()
            .find(|rule| {
                (!rule["requires_media"].as_bool().unwrap_or(false) || has_media)
                    && rule["categories"]
                        .as_array()
                        .is_none_or(|items| items.iter().any(|c| c == &project["category"]))
                    && rule["tags"].as_array().is_none_or(|items| {
                        items.iter().any(|tag| {
                            project["tags"]
                                .as_array()
                                .is_some_and(|tags| tags.contains(tag))
                        })
                    })
            })
            .and_then(|r| r["template"].as_str())
            .unwrap_or("flow")
    } else {
        requested.as_str()
    };
    let preset = templates.iter().find(|t| t["id"] == picked).unwrap();
    let render = if preset["requires_media"].as_bool().unwrap_or(false) && !has_media {
        templates.iter().find(|t| t["id"] == "flow").unwrap()
    } else {
        preset
    };
    project["cover_template"] = json!(requested);
    project["cover_style"] = render["style"].clone();
    project["cover_layout"] = render["layout"].clone();
    let hash = hash(project["full_name"].as_str().unwrap_or_default()) as usize;
    let variants = rules["variants"].as_array().unwrap();
    if !variants.iter().any(|v| v["id"] == project["cover_variant"]) {
        project["cover_variant"] = variants[hash % variants.len()]["id"].clone();
    }
    let color = project["cover_accent"].as_str().unwrap_or_default();
    if color.is_empty() {
        let palette = rules["palettes"].as_array().unwrap();
        project["cover_accent"] = palette[hash % palette.len()].clone();
    } else if color.len() != 7
        || !color.starts_with('#')
        || !color[1..].bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err(bad("封面颜色需要 #RRGGBB 格式"));
    }
    if project["cover_lines"]
        .as_array()
        .is_none_or(|v| v.is_empty())
    {
        project["cover_lines"] = json!(project["tags"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .take(3)
            .collect::<Vec<_>>());
    }
    for (key, default) in [("cover_focus_x", 50.0), ("cover_focus_y", 46.0)] {
        let number = project[key].as_f64().unwrap_or(default);
        if !(0.0..=100.0).contains(&number) {
            return Err(bad("图片焦点需要在 0–100 之间"));
        }
        project[key] = json!(number);
    }
    project["icon_fit"] = json!(if project["icon_fit"] == "mark-left" {
        "mark-left"
    } else {
        "contain"
    });
    Ok(())
}

pub fn normalize(mut catalog: Value, publishing: bool) -> AppResult<Value> {
    if !catalog.is_object() {
        return Err(bad("精选目录格式无效"));
    }
    if catalog["scope"]
        .as_str()
        .is_some_and(|v| v != "open-source")
    {
        return Err(bad("这里只接受精选板块规则"));
    }
    catalog.as_object_mut().unwrap().remove("revision");
    catalog.as_object_mut().unwrap().remove("published_at");
    let categories = super::open_source::catalog()["categories"].clone();
    let category_keys: HashSet<_> = categories
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|c| c["key"].as_str())
        .collect();
    let projects = catalog["projects"]
        .as_array_mut()
        .ok_or_else(|| bad("缺少项目列表"))?;
    if projects.len() > 1000 {
        return Err(bad("精选目录最多 1000 个项目"));
    }
    let mut names = HashSet::new();
    for (index, project) in projects.iter_mut().enumerate() {
        if !project.is_object() {
            return Err(bad("项目格式无效"));
        }
        for (key, max, required) in [
            ("full_name", 140, true),
            ("name", 100, true),
            ("title", 200, publishing),
            ("summary", 1000, false),
            ("effort", 200, false),
            ("cover_caption", 200, false),
            ("audience", 4000, false),
            ("getting_started", 4000, false),
        ] {
            text(project, key, max, required)?;
        }
        let name = project["full_name"].as_str().unwrap();
        if !crate::collectors::repo_sources::valid_name(name)
            || !names.insert(name.to_ascii_lowercase())
        {
            return Err(bad("仓库名称无效或重复"));
        }
        if !category_keys.contains(project["category"].as_str().unwrap_or_default()) {
            return Err(bad(format!("{name} 缺少有效分类")));
        }
        for (key, max, length) in [
            ("related_categories", 2, 40),
            ("tags", 12, 40),
            ("highlights", 8, 500),
            ("notes", 12, 1000),
            ("cover_lines", 3, 100),
        ] {
            strings(project, key, max, length)?;
        }
        if project["related_categories"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| !category_keys.contains(c.as_str().unwrap_or_default()))
        {
            return Err(bad("关联分类无效"));
        }
        for key in ["homepage_url", "source_url", "icon_source_url"] {
            text(project, key, 2048, publishing && key == "source_url")?;
            if project[key]
                .as_str()
                .is_some_and(|v| !v.is_empty() && !https(v))
            {
                return Err(bad(format!("{key} 需要公开 HTTPS 地址")));
            }
        }
        for key in ["icon_url", "cover_url"] {
            text(project, key, 2048, false)?;
            if !asset_url(project[key].as_str().unwrap_or_default()) {
                return Err(bad(format!("{key} 需要站内素材或公开 HTTPS 图片")));
            }
        }
        project["featured"] = json!(project["featured"].as_bool().unwrap_or(true));
        project["order"] = json!(project["order"]
            .as_i64()
            .unwrap_or(index as i64 + 1)
            .clamp(0, 100000));
        resolve(project)?;
    }
    let collections = catalog["collections"]
        .as_array_mut()
        .ok_or_else(|| bad("缺少专题列表"))?;
    if collections.len() > 100 {
        return Err(bad("最多 100 个专题"));
    }
    let mut keys = HashSet::new();
    for collection in collections {
        for (key, max, required) in [
            ("key", 80, true),
            ("title", 200, true),
            ("summary", 1000, false),
            ("description", 4000, false),
        ] {
            text(collection, key, max, required)?;
        }
        let key = collection["key"].as_str().unwrap();
        if !key
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            || !keys.insert(key.to_string())
        {
            return Err(bad("专题标识需要唯一的小写字母、数字和连字符"));
        }
        strings(collection, "projects", 50, 140)?;
        let count = collection["projects"].as_array().unwrap().len();
        let notes = collection["notes"].as_array().cloned().unwrap_or_default();
        if notes.len() > count
            || notes
                .iter()
                .any(|v| v.as_str().is_none_or(|s| s.chars().count() > 1000))
        {
            return Err(bad("专题点评与项目数量不匹配，或点评内容过长"));
        }
        collection["notes"] = json!((0..count)
            .map(|index| notes
                .get(index)
                .and_then(Value::as_str)
                .unwrap_or_default()
                .trim()
                .to_string())
            .collect::<Vec<_>>());
        if collection["projects"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| !names.contains(&v.as_str().unwrap_or_default().to_ascii_lowercase()))
        {
            return Err(bad("专题引用了不在目录中的项目，请先调整专题"));
        }
    }
    catalog["categories"] = categories;
    catalog["scope"] = json!("open-source");
    catalog["version"] = json!(2);
    catalog["cover_rules_version"] = manifest()["version"].clone();
    catalog["notice"] =
        json!("根据项目公开资料整理，未经本站实测；具体功能与许可请以官方说明为准。");
    Ok(catalog)
}
