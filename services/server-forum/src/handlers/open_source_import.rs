use super::{admin::AuthAdmin, open_source_rules as rules};
use crate::{
    error::{AppError, AppResult},
    state::AppState,
};
use axum::{extract::State, Json};
use base64::{engine::general_purpose::STANDARD, Engine};
use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use serde::Deserialize;
use serde_json::{json, Value};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    repository: String,
}
pub fn repository(raw: &str) -> AppResult<String> {
    let name = if raw.starts_with("https://") {
        let u =
            reqwest::Url::parse(raw).map_err(|_| AppError::BadRequest("仓库地址无效".into()))?;
        if u.host_str() != Some("github.com") || !rules::https(raw) || u.query().is_some() {
            return Err(AppError::BadRequest("请填写 GitHub 仓库首页地址".into()));
        }
        u.path()
            .trim_matches('/')
            .trim_end_matches(".git")
            .to_string()
    } else {
        raw.trim().trim_end_matches(".git").to_string()
    };
    if !crate::collectors::repo_sources::valid_name(&name) {
        return Err(AppError::BadRequest(
            "请填写 owner/repo 或 GitHub 仓库首页地址".into(),
        ));
    }
    Ok(name)
}
async fn get(http: &reqwest::Client, path: &str) -> AppResult<Value> {
    let mut response = http
        .get(format!("https://api.github.com{path}"))
        .header("User-Agent", "Xiaoban-Curated-Covers/1")
        .header("Accept", "application/vnd.github+json")
        .timeout(std::time::Duration::from_secs(20))
        .send()
        .await
        .map_err(|_| AppError::BadRequest("GitHub 暂时无法访问".into()))?;
    if response.status().as_u16() == 404 {
        return Ok(Value::Null);
    }
    if !response.status().is_success() {
        return Err(AppError::BadRequest(format!(
            "GitHub 返回 HTTP {}，请稍后重试或手工填写",
            response.status().as_u16()
        )));
    }
    let mut data = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| AppError::BadRequest("GitHub 读取失败".into()))?
    {
        if data.len() + chunk.len() > 2 * 1024 * 1024 {
            return Err(AppError::BadRequest("GitHub 资料超过上限".into()));
        }
        data.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&data).map_err(|_| AppError::BadRequest("GitHub 返回格式无效".into()))
}
fn attribute<'a>(tag: &'a str, key: &str) -> Option<&'a str> {
    let start = tag.find(&format!("{key}="))? + key.len() + 1;
    let quote = tag.as_bytes().get(start)?;
    if !matches!(quote, b'\'' | b'"') {
        return None;
    }
    let rest = &tag[start + 1..];
    Some(&rest[..rest.find(*quote as char)?])
}
pub fn candidates(markdown: &str, base: &str) -> Vec<Value> {
    let mut raw = Vec::<(String, String)>::new();
    let mut current: Option<(String, String)> = None;
    for event in Parser::new(markdown) {
        match event {
            Event::Start(Tag::Image { dest_url, .. }) => {
                current = Some((dest_url.to_string(), String::new()))
            }
            Event::Text(t) | Event::Code(t) => {
                if let Some((_, alt)) = &mut current {
                    if alt.len() < 200 {
                        alt.push_str(&t);
                    }
                }
            }
            Event::End(TagEnd::Image) => {
                if let Some(item) = current.take() {
                    raw.push(item);
                }
            }
            Event::Html(html) | Event::InlineHtml(html) => {
                for tag in html.split('<').filter(|s| s.starts_with("img ")) {
                    if let Some(src) = attribute(tag, "src") {
                        raw.push((
                            src.to_string(),
                            attribute(tag, "alt").unwrap_or_default().to_string(),
                        ));
                    }
                }
            }
            _ => {}
        }
    }
    let base = reqwest::Url::parse(base).ok();
    let mut out = Vec::new();
    for (url, alt) in raw {
        let lower = url.to_ascii_lowercase();
        if lower.contains("shields.io") || lower.contains("badge") || lower.contains("/actions/") {
            continue;
        }
        let url = if rules::https(&url) {
            url
        } else {
            let Some(url) = base
                .as_ref()
                .and_then(|b| b.join(&url).ok())
                .filter(|u| rules::https(u.as_str()))
            else {
                continue;
            };
            url.to_string()
        };
        if out.iter().any(|v: &Value| v["url"] == url) {
            continue;
        }
        let hint = format!("{} {}", url, alt).to_ascii_lowercase();
        let role = if hint.contains("logo") || hint.contains("icon") {
            "logo"
        } else if hint.contains("screenshot") || hint.contains("preview") {
            "screenshot"
        } else {
            "image"
        };
        out.push(json!({"url":url,"label":alt.chars().take(100).collect::<String>(),"role":role,"source":"README"}));
        if out.len() >= 12 {
            break;
        }
    }
    out
}
pub async fn import(
    State(s): State<AppState>,
    _: AuthAdmin,
    Json(input): Json<Input>,
) -> AppResult<Json<Value>> {
    let name = repository(&input.repository)?;
    let meta = get(&s.http, &format!("/repos/{name}")).await?;
    if meta.is_null() {
        return Err(AppError::not_found("公开 GitHub 仓库"));
    }
    let canonical = meta["full_name"]
        .as_str()
        .filter(|v| crate::collectors::repo_sources::valid_name(v))
        .ok_or_else(|| AppError::BadRequest("GitHub 仓库标识无效".into()))?;
    let readme = get(&s.http, &format!("/repos/{canonical}/readme")).await?;
    let content = readme["content"]
        .as_str()
        .filter(|v| v.len() < 1024 * 1024)
        .map(|v| v.chars().filter(|c| !c.is_whitespace()).collect::<String>())
        .and_then(|v| STANDARD.decode(v).ok())
        .and_then(|v| String::from_utf8(v).ok())
        .unwrap_or_default();
    let base = readme["download_url"].as_str().unwrap_or_default();
    let assets = candidates(&content, base);
    let icon = assets
        .iter()
        .find(|a| a["role"] == "logo")
        .and_then(|a| a["url"].as_str())
        .unwrap_or_default();
    let cover = assets
        .iter()
        .find(|a| a["role"] == "screenshot")
        .and_then(|a| a["url"].as_str())
        .unwrap_or_default();
    let homepage = meta["homepage"]
        .as_str()
        .filter(|v| rules::https(v))
        .map(str::to_string)
        .unwrap_or_else(|| format!("https://github.com/{canonical}"));
    Ok(Json(
        json!({"project":{"full_name":canonical,"github_id":meta["id"],"name":meta["name"],"title":meta["name"],"summary":meta["description"].as_str().unwrap_or_default(),"category":"dev","related_categories":[],"tags":[],"effort":"","cover_url":cover,"cover_caption":"项目 README 展示图","icon_url":icon,"icon_source_url":icon,"homepage_url":homepage,"source_url":format!("https://github.com/{canonical}#readme"),"highlights":[],"audience":"","getting_started":"","notes":[],"featured":true,"order":1,"cover_template":"auto","cover_variant":"balanced","cover_lines":[],"readme_sha":readme["sha"],"collected_at":chrono::Utc::now()},"assets":assets,"notes":["仅生成待编辑资料，不会自动保存或发布。","请核对素材身份、图片使用许可和中文推荐内容；作者头像不自动作为项目 Logo。"]}),
    ))
}
