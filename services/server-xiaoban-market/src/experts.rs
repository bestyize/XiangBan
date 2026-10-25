//! 专家（WorkBuddy 插件格式）入库：上传一个插件包，校验后写成条目、版本和头像。
//!
//! 专家是智能体人设：`agents/<名>.md` 是提示词，展示字段在 plugin.json。包原样保存，版本清单按包内文件算摘要，
//! 桌面端安装后把它当插件读取（见 libs/agent-tools/skill-discovery/src/plugin.rs）。
use crate::{
    error::{AppError, Result},
    connectors, items, package,
    plugin::{self, Issue, Manifest, PluginPackage},
    state::AppState,
    storage,
};
use serde_json::{json, Value};
use sqlx::types::Json;
use uuid::Uuid;

#[derive(Debug, Clone, Default)]
pub struct ImportOpts {
    pub namespace: String,
    /// 新建时的状态；已存在的条目为 `None` 就保持原状态。
    pub status: Option<String>,
    pub source: Option<String>,
    /// 导入别处发布的插件时用宽松模式：展示规范（3 个标签、3 条提问、字数）只给警告。
    pub lenient: bool,
}

#[derive(Debug)]
pub struct Report {
    pub item_id: Uuid,
    pub version: String,
    pub created: bool,
    pub issues: Vec<Issue>,
}

/// 头像必须真是图片：看文件头，不信扩展名。
pub fn image_kind(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) {
        Some("image/png")
    } else if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Some("image/jpeg")
    } else if bytes.len() > 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        Some("image/webp")
    } else {
        None
    }
}

/// 把包里的头像存成内容寻址的资源，返回公开路径。不是图片或太大就是错误。
async fn store_avatar(state: &AppState, path: &str, bytes: &[u8]) -> Result<String> {
    if image_kind(bytes).is_none() {
        return Err(AppError::BadRequest(format!("{path} 不是 PNG、JPG 或 WebP 图片")));
    }
    let sha = package::sha256_hex(bytes);
    storage::store_asset(&state.config.data_dir, &sha, bytes).await.map_err(|e| AppError::Internal(format!("保存头像失败：{e}")))?;
    Ok(format!("/api/v1/market/assets/{sha}"))
}

async fn store_connector_icon(state: &AppState, path: &str, bytes: &[u8]) -> Result<String> {
    if image_kind(bytes).is_none() && !connectors::safe_svg(bytes) {
        return Err(AppError::BadRequest(format!("{path} 不是图片或包含不安全的 SVG 内容")));
    }
    let sha = package::sha256_hex(bytes);
    storage::store_asset(&state.config.data_dir, &sha, bytes).await.map_err(|e| AppError::Internal(format!("保存图标失败：{e}")))?;
    Ok(format!("/api/v1/market/assets/{sha}"))
}

fn text(b: &Option<plugin::Bilingual>) -> (String, String) {
    b.as_ref().map(|v| (v.zh.clone(), v.en.clone())).unwrap_or_default()
}

fn pair(b: &plugin::Bilingual) -> Value {
    json!({ "en": b.en, "zh": b.zh })
}

fn pair_opt(b: &Option<plugin::Bilingual>) -> Value {
    b.as_ref().map(pair).unwrap_or_else(|| json!({ "en": "", "zh": "" }))
}

/// 条目上存的展示信息（`metadata.expert`），列表和详情接口原样返回，网页和桌面端都读它。
pub(crate) async fn presentation(state: &AppState, pkg: &PluginPackage, avatar_url: Option<&str>) -> Result<Value> {
    let m: &Manifest = &pkg.manifest;
    let mut members = Vec::new();
    for member in &m.members {
        let url = match pkg.unpacked.contents.get(member.avatar.strip_prefix("./").unwrap_or(&member.avatar)) {
            Some(bytes) if !member.avatar.is_empty() => Some(store_avatar(state, &member.avatar, bytes).await?),
            _ => None,
        };
        members.push(json!({ "id": member.id, "name": pair(&member.name), "profession": pair(&member.profession), "role": member.role, "avatarUrl": url }));
    }
    let mut agent_docs = Vec::new();
    for (path, doc) in &pkg.agents {
        agent_docs.push(json!({ "path": path, "name": doc.name, "maxTurns": doc.max_turns, "skills": doc.skills }));
    }
    Ok(json!({
        "expertType": m.expert_type, "agentName": m.agent_name,
        "displayName": pair_opt(&m.display_name), "profession": pair_opt(&m.profession), "displayDescription": pair_opt(&m.display_description),
        "categoryId": m.category_id,
        "tags": m.tags.iter().map(pair).collect::<Vec<_>>(),
        "quickPrompts": m.quick_prompts.iter().map(pair).collect::<Vec<_>>(),
        "defaultInitPrompt": pair_opt(&m.default_init_prompt),
        "avatarUrl": avatar_url,
        "skills": m.skills, "agents": agent_docs, "members": members,
        "teamInfo": m.team_info,
    }))
}

/// 上传一个 WorkBuddy 专家插件包。校验有错误时整体拒绝；通过后写入条目、头像和新版本。
pub async fn import_plugin(state: &AppState, bytes: &[u8], opts: &ImportOpts) -> Result<Report> {
    let pkg = plugin::inspect_plugin_zip(bytes).map_err(|e| AppError::BadRequest(format!("插件包未通过校验：{e}")))?;
    let m = &pkg.manifest;
    let kind = connectors::kind(m, &pkg.unpacked.contents).map_err(AppError::BadRequest)?;
    let issues = if kind == "connector" { connectors::validate(m, &pkg.unpacked.contents) } else { plugin::validate_expert(m, &pkg.unpacked.contents, opts.lenient) };
    let errors: Vec<&str> = issues.iter().filter(|i| i.error).map(|i| i.message.as_str()).collect();
    if !errors.is_empty() {
        return Err(AppError::BadRequest(format!("插件包未通过校验：{}", errors.join("；"))));
    }
    if !crate::upstream::valid_namespace(&opts.namespace) || !crate::upstream::valid_slug(&m.name) {
        return Err(AppError::BadRequest("命名空间或插件名不合法".into()));
    }
    let version = m.version.trim().to_string();
    if version.len() > 44 || !version.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'+')) {
        return Err(AppError::BadRequest("版本号只能含字母、数字、. - +，最长 44 位".into()));
    }

    let avatar_url = if kind == "connector" {
        match connectors::icon(&pkg.unpacked.contents) {
            Some((path, bytes)) => Some(store_connector_icon(state, path, bytes).await?),
            None => None,
        }
    } else { match pkg.unpacked.contents.get(m.avatar.strip_prefix("./").unwrap_or(&m.avatar)) {
        Some(bytes) if !m.avatar.is_empty() => Some(store_avatar(state, &m.avatar, bytes).await?),
        _ => None,
    }};
    let display = if kind == "connector" { connectors::presentation(m, &pkg.unpacked.contents) } else { presentation(state, &pkg, avatar_url.as_deref()).await? };

    let (name_zh, name_en) = if kind == "connector" {
        (display["displayName"]["zh"].as_str().unwrap_or_default().into(), display["displayName"]["en"].as_str().unwrap_or_default().into())
    } else { text(&m.display_name) };
    let (profession_zh, profession_en) = text(&m.profession);
    let (summary, summary_en) = if kind == "connector" {
        (display["displayDescription"]["zh"].as_str().unwrap_or_default().into(), display["displayDescription"]["en"].as_str().unwrap_or_default().into())
    } else { text(&m.display_description) };
    let tags: Vec<String> = if kind == "connector" { m.keywords.clone() } else { m.tags.iter().map(|t| if t.zh.is_empty() { t.en.clone() } else { t.zh.clone() }).collect() };
    let prompts: Vec<String> = if kind == "connector" {
        let examples = &display["examples"];
        examples["zh"].as_array().filter(|a| !a.is_empty()).or_else(|| examples["en"].as_array()).map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect()).unwrap_or_default()
    } else { m.quick_prompts.iter().map(|t| if t.zh.is_empty() { t.en.clone() } else { t.zh.clone() }).collect() };
    let description = pkg
        .unpacked
        .contents
        .get("README.md")
        .and_then(|b| String::from_utf8(b.clone()).ok())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| summary.clone());
    let name = if name_zh.is_empty() { m.name.clone() } else { name_zh };
    let category: Option<(i32, String)> = sqlx::query_as("SELECT id, name FROM categories WHERE kind = $1 AND slug = $2")
        .bind(kind)
        .bind(if kind == "connector" { connectors::category(m) } else { &m.category_id })
        .fetch_optional(&state.pool)
        .await?;
    let search = items::search_text(
        &format!("{name} {name_en} {profession_zh} {profession_en}"),
        &m.name,
        &opts.namespace,
        &format!("{summary} {summary_en}"),
        &tags,
        category.as_ref().map(|c| c.1.as_str()).unwrap_or(""),
    );
    let variant = if kind == "expert" && m.expert_type == "team" { "team" } else { "single" };
    let source = opts.source.clone().unwrap_or_else(|| "community".into());
    let status = opts.status.clone();

    let mut tx = state.pool.begin().await?;
    let (item_id, inserted): (Uuid, bool) = sqlx::query_as(
        "INSERT INTO items (kind, namespace, slug, name, summary, description, category_id, tags, icon_url, source, origin, license,
           status, latest_version, metadata, prompts, search_text, variant, published_at)
         VALUES ($17, $1, $2, $3, $4, $5, $6, $7, $8, $9, 'upload', $10, COALESCE($11, 'draft'), $12, $13, $14, $15, $16,
           CASE WHEN $11 = 'published' THEN now() END)
         ON CONFLICT (kind, namespace, slug) DO UPDATE SET
           name = EXCLUDED.name, summary = EXCLUDED.summary, description = EXCLUDED.description, category_id = EXCLUDED.category_id,
           tags = EXCLUDED.tags, icon_url = EXCLUDED.icon_url, license = EXCLUDED.license, metadata = EXCLUDED.metadata,
           prompts = EXCLUDED.prompts, search_text = EXCLUDED.search_text, variant = EXCLUDED.variant, updated_at = now(),
           status = COALESCE($11, items.status),
           published_at = CASE WHEN COALESCE($11, items.status) = 'published' THEN COALESCE(items.published_at, now()) ELSE items.published_at END
         RETURNING id, (xmax = 0) AS inserted",
    )
    .bind(&opts.namespace)
    .bind(&m.name)
    .bind(&name)
    .bind(&summary)
    .bind(&description)
    .bind(category.as_ref().map(|c| c.0))
    .bind(&tags)
    .bind(&avatar_url)
    .bind(&source)
    .bind(&m.license)
    .bind(&status)
    .bind(&version)
    .bind(Json(if kind == "connector" { json!({ "connector": display }) } else { json!({ "expert": display, "members": [] }) }))
    .bind(Json(json!(prompts)))
    .bind(&search)
    .bind(variant)
    .bind(kind)
    .fetch_one(&mut *tx)
    .await?;

    let sha = package::sha256_hex(bytes);
    let existing: Option<String> = sqlx::query_scalar("SELECT package_sha256 FROM versions WHERE item_id = $1 AND version = $2")
        .bind(item_id)
        .bind(&version)
        .fetch_optional(&mut *tx)
        .await?;
    match existing {
        Some(old) if old != sha => return Err(AppError::Conflict("该版本号已存在且内容不同，请提高版本号后再上传".into())),
        Some(_) => {}
        None => {
            storage::store_package(&state.config.data_dir, &sha, bytes).await.map_err(|e| AppError::Internal(format!("保存安装包失败：{e}")))?;
            sqlx::query(
                "INSERT INTO versions (item_id, version, changelog, manifest, manifest_sha256, package_sha256, package_size)
                 VALUES ($1, $2, '', $3, $4, $5, $6)",
            )
            .bind(item_id)
            .bind(&version)
            .bind(Json(json!({ "files": pkg.unpacked.files })))
            .bind(&pkg.unpacked.manifest_sha256)
            .bind(&sha)
            .bind(bytes.len() as i64)
            .execute(&mut *tx)
            .await?;
        }
    }
    tx.commit().await?;
    Ok(Report { item_id, version, created: inserted, issues })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn avatars_are_recognized_by_their_file_header() {
        assert_eq!(image_kind(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0]), Some("image/png"));
        assert_eq!(image_kind(&[0xFF, 0xD8, 0xFF, 0xE0]), Some("image/jpeg"));
        assert_eq!(image_kind(b"RIFF\x00\x00\x00\x00WEBPVP8 "), Some("image/webp"));
        assert_eq!(image_kind(b"<svg xmlns='http://www.w3.org/2000/svg'/>"), None, "SVG 可带脚本，不收");
        assert_eq!(image_kind(b"GIF89a"), None);
        assert_eq!(image_kind(b""), None);
    }
}
