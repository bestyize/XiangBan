//! 暂存项入库：二次抓取详情 / 安装包、校验、解析分类，并在一个事务里写入条目与版本。
use crate::{
    error::{AppError, Result},
    convert, experts, items, package, plugin, storage,
    state::AppState,
    upstream::{self, Enrichment, Normalized, SourceRow},
};
use serde_json::{json, Value};
use sqlx::types::Json;
use uuid::Uuid;

#[derive(Debug, Clone, Default)]
pub struct ImportOpts {
    /// 新建条目的状态；对已存在的条目，`None` 表示保持当前状态。
    pub status: Option<String>,
    pub category_id: Option<i32>,
}

#[derive(sqlx::FromRow)]
struct StagedRow {
    source_id: i32,
    normalized: Json<Value>,
}

pub async fn import_staged(state: &AppState, id: Uuid, opts: &ImportOpts) -> Result<Uuid> {
    match import_inner(state, id, opts).await {
        Ok(item_id) => Ok(item_id),
        Err(err) => {
            let message = err.to_string();
            sqlx::query("UPDATE staged_items SET import_error = $2 WHERE id = $1").bind(id).bind(&message).execute(&state.pool).await.ok();
            Err(err)
        }
    }
}

async fn import_inner(state: &AppState, id: Uuid, opts: &ImportOpts) -> Result<Uuid> {
    let pool = &state.pool;
    let staged: StagedRow = sqlx::query_as("SELECT source_id, normalized FROM staged_items WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("暂存项不存在".into()))?;
    let source: SourceRow = sqlx::query_as("SELECT * FROM sources WHERE id = $1").bind(staged.source_id).fetch_one(pool).await?;
    let n: Normalized = serde_json::from_value(staged.normalized.0).map_err(|e| AppError::Internal(format!("暂存数据损坏：{e}")))?;
    if !upstream::valid_slug(&n.slug) || !upstream::valid_namespace(&n.namespace) {
        return Err(AppError::BadRequest("上游条目的标识不合法，无法入库".into()));
    }

    let enrichment = upstream::enrich(&state.ctx(), &source, &n).await.map_err(|e| AppError::BadRequest(e.message))?;
    let mut summary = n.summary.clone();
    let mut description = n.description.clone();
    let mut metadata = n.metadata.clone();
    let mut latest_version = n.version.clone();
    let mut package_row = None;
    let origin = source.config["origin"].as_str().map(str::to_string).unwrap_or_else(|| source.key.clone());
    match enrichment {
        Enrichment::Package { version, changelog, bytes } => {
            let bytes = package::prepare_package(&bytes, &n.slug).map_err(|e| AppError::BadRequest(format!("安装包未通过校验：{e}")))?;
            let info = package::inspect_zip(&bytes).map_err(|e| AppError::BadRequest(format!("安装包未通过校验：{e}")))?;
            let (_, front_description) = package::frontmatter(&info.skill_md);
            if summary.is_empty() {
                summary = front_description.unwrap_or_default();
            }
            description = info.skill_md.clone();
            latest_version = Some(version.clone());
            let sha = package::sha256_hex(&bytes);
            storage::store_package(&state.config.data_dir, &sha, &bytes).await.map_err(|e| AppError::Internal(format!("保存安装包失败：{e}")))?;
            package_row = Some((version, changelog, bytes.len() as i64, sha, info.files.clone(), info.manifest_sha256.clone()));
        }
        Enrichment::Expert { content, dependencies, detail, children, missing } => {
            // 专家转成 WorkBuddy 插件：工作流成为人设，依赖的子技能打进 `skills/`。
            let category_id = items::resolve_category(pool, &n.kind, n.category.as_deref(), None).await?;
            let category_slug: String = match category_id {
                Some(id) => sqlx::query_scalar("SELECT slug FROM categories WHERE id = $1").bind(id).fetch_one(pool).await?,
                None => "12-IndustryConsultant".into(),
            };
            let category_slug = if crate::plugin::CATEGORIES.iter().any(|(c, _, _)| *c == category_slug) { category_slug } else { "12-IndustryConsultant".to_string() };
            let name_en = detail["displayNameEn"].as_str().unwrap_or("").to_string();
            let summary_en = detail["summaryEn"].as_str().unwrap_or("").to_string();
            let base_version = package::expert_version(&content, "", None);
            let mut bytes = convert::skillhub_expert(&expert_source(&n, &name_en, &summary_en, &content, &category_slug, &base_version), &children, &missing).map_err(|e| AppError::BadRequest(format!("转换专家包失败：{e}")))?;
            let mut version = base_version.clone();
            // 同一个版本号下内容变了（上游改了正文或子技能），用内容摘要作后缀开一个新版本，客户端才能发现更新。
            let stored: Option<String> = sqlx::query_scalar(
                "SELECT v.package_sha256 FROM versions v JOIN items i ON i.id = v.item_id WHERE i.kind = $1 AND i.origin = $2 AND i.origin_ref = $3 AND v.version = $4",
            )
            .bind(&n.kind)
            .bind(&origin)
            .bind(&n.upstream_id)
            .bind(&version)
            .fetch_optional(pool)
            .await?;
            if stored.as_deref().is_some_and(|s| s != package::sha256_hex(&bytes)) {
                version = format!("{base_version}+{}", &package::sha256_hex(&bytes)[..8]);
                bytes = convert::skillhub_expert(&expert_source(&n, &name_en, &summary_en, &content, &category_slug, &version), &children, &missing).map_err(|e| AppError::BadRequest(format!("转换专家包失败：{e}")))?;
            }
            let pkg = plugin::inspect_plugin_zip(&bytes).map_err(|e| AppError::BadRequest(format!("转换后的插件包未通过校验：{e}")))?;
            let issues = plugin::validate_expert(&pkg.manifest, &pkg.unpacked.contents, true);
            if let Some(error) = issues.iter().find(|i| i.error && !i.message.contains("头像")) {
                return Err(AppError::BadRequest(format!("转换后的插件包未通过校验：{}", error.message)));
            }
            let sha = package::sha256_hex(&bytes);
            storage::store_package(&state.config.data_dir, &sha, &bytes).await.map_err(|e| AppError::Internal(format!("保存安装包失败：{e}")))?;
            let mut expert = experts::presentation(state, &pkg, n.icon_url.as_deref()).await?;
            expert["generated"] = json!(true);
            expert["missingSkills"] = json!(missing);
            metadata["expert"] = expert;
            metadata["dependencies"] = dependencies;
            description = content.clone();
            latest_version = Some(version.clone());
            package_row = Some((version, String::new(), bytes.len() as i64, sha, pkg.unpacked.files.clone(), pkg.unpacked.manifest_sha256.clone()));
        }
        Enrichment::Connector { readme, configs } => {
            description = readme;
            metadata["configs"] = configs;
        }
        Enrichment::None => {}
    }

    let category_id = items::resolve_category(pool, &n.kind, n.category.as_deref(), opts.category_id).await?;
    let category_name: String = match category_id {
        Some(cid) => sqlx::query_scalar("SELECT name FROM categories WHERE id = $1").bind(cid).fetch_one(pool).await?,
        None => String::new(),
    };
    let search = items::search_text(&n.name, &n.slug, &n.namespace, &summary, &n.tags, &category_name);

    let mut tx = pool.begin().await?;
    let existing: Option<(Uuid, Option<i32>)> =
        sqlx::query_as("SELECT id, category_id FROM items WHERE kind = $1 AND origin = $2 AND origin_ref = $3 FOR UPDATE")
            .bind(&n.kind)
            .bind(&origin)
            .bind(&n.upstream_id)
            .fetch_optional(&mut *tx)
            .await?;
    let item_id = if let Some((item_id, current_category)) = existing {
        let category = if opts.category_id.is_some() { category_id } else { current_category.or(category_id) };
        sqlx::query(
            "UPDATE items SET name=$2, summary=$3, description=$4, tags=$5, icon_url=$6, source=$7, origin_url=$8, license=$9,
               metadata=$10, latest_version=COALESCE($11, latest_version), search_text=$12, category_id=$13, updated_at=now()
             WHERE id=$1",
        )
        .bind(item_id)
        .bind(&n.name)
        .bind(&summary)
        .bind(&description)
        .bind(&n.tags)
        .bind(&n.icon_url)
        .bind(&n.source)
        .bind(&n.origin_url)
        .bind(&n.license)
        .bind(Json(&metadata))
        .bind(&latest_version)
        .bind(&search)
        .bind(category)
        .execute(&mut *tx)
        .await?;
        if let Some(status) = &opts.status {
            sqlx::query("UPDATE items SET status=$2, published_at = CASE WHEN $2='published' THEN COALESCE(published_at, now()) ELSE published_at END WHERE id=$1")
                .bind(item_id)
                .bind(status)
                .execute(&mut *tx)
                .await?;
        }
        item_id
    } else {
        let taken: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM items WHERE kind=$1 AND namespace=$2 AND slug=$3)")
            .bind(&n.kind)
            .bind(&n.namespace)
            .bind(&n.slug)
            .fetch_one(&mut *tx)
            .await?;
        if taken {
            return Err(AppError::Conflict(format!("标识 {}/{} 已被其他条目占用", n.namespace, n.slug)));
        }
        let status = opts.status.clone().unwrap_or_else(|| "draft".into());
        sqlx::query_scalar(
            "INSERT INTO items (kind, namespace, slug, name, summary, description, category_id, tags, icon_url, source, origin, origin_ref,
               origin_url, license, status, latest_version, metadata, search_text, published_at)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18, CASE WHEN $15='published' THEN now() END)
             RETURNING id",
        )
        .bind(&n.kind)
        .bind(&n.namespace)
        .bind(&n.slug)
        .bind(&n.name)
        .bind(&summary)
        .bind(&description)
        .bind(category_id)
        .bind(&n.tags)
        .bind(&n.icon_url)
        .bind(&n.source)
        .bind(&origin)
        .bind(&n.upstream_id)
        .bind(&n.origin_url)
        .bind(&n.license)
        .bind(&status)
        .bind(&latest_version)
        .bind(Json(&metadata))
        .bind(&search)
        .fetch_one(&mut *tx)
        .await?
    };

    if let Some((version, changelog, size, sha, files, manifest_sha256)) = &package_row {
        sqlx::query(
            "INSERT INTO versions (item_id, version, changelog, manifest, manifest_sha256, package_sha256, package_size)
             VALUES ($1,$2,$3,$4,$5,$6,$7) ON CONFLICT (item_id, version) DO NOTHING",
        )
        .bind(item_id)
        .bind(version)
        .bind(changelog)
        .bind(Json(json!({ "files": files })))
        .bind(manifest_sha256)
        .bind(sha)
        .bind(size)
        .execute(&mut *tx)
        .await?;
    }
    if let (Some(cid), Some(label)) = (opts.category_id, n.category.as_deref()) {
        sqlx::query("INSERT INTO category_aliases (kind, alias, category_id) VALUES ($1,$2,$3) ON CONFLICT (kind, alias) DO UPDATE SET category_id = EXCLUDED.category_id")
            .bind(&n.kind)
            .bind(label.trim().to_lowercase())
            .bind(cid)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("UPDATE staged_items SET status='imported', item_id=$2, import_error=NULL WHERE id=$1").bind(id).bind(item_id).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(item_id)
}

/// 启动时把已入库的技能包整理成当前规则（剔除注册表元数据、SKILL.md 的 name 对齐 slug），幂等。
/// 内容变了就换成新包并更新清单摘要；旧包文件保留，不影响其他引用。返回修正的版本数。
pub async fn align_stored_packages(pool: &sqlx::PgPool, data_dir: &std::path::Path) -> Result<usize> {
    let rows: Vec<(Uuid, String, String)> = sqlx::query_as(
        "SELECT v.id, i.slug, v.package_sha256 FROM versions v JOIN items i ON i.id = v.item_id WHERE i.kind = 'skill'",
    )
    .fetch_all(pool)
    .await?;
    let mut fixed = 0;
    for (id, slug, sha) in rows {
        let Ok(bytes) = tokio::fs::read(storage::package_path(data_dir, &sha)).await else { continue };
        let Ok(next) = package::prepare_package(&bytes, &slug) else { continue };
        if next == bytes {
            continue;
        }
        let info = match package::inspect_zip(&next) {
            Ok(info) => info,
            Err(e) => {
                tracing::warn!("技能包 {slug} 整理后仍未通过校验，保持原样：{e}");
                continue;
            }
        };
        let new_sha = package::sha256_hex(&next);
        storage::store_package(data_dir, &new_sha, &next).await.map_err(|e| AppError::Internal(format!("保存安装包失败：{e}")))?;
        sqlx::query("UPDATE versions SET manifest = $2, manifest_sha256 = $3, package_sha256 = $4, package_size = $5 WHERE id = $1")
            .bind(id)
            .bind(Json(json!({ "files": info.files })))
            .bind(&info.manifest_sha256)
            .bind(&new_sha)
            .bind(next.len() as i64)
            .execute(pool)
            .await?;
        tracing::info!("已整理技能包 {slug}（SKILL.md 的 name、注册表元数据）");
        fixed += 1;
    }
    Ok(fixed)
}

/// 给还没有安装包的专家补上包（早先只存了正文和依赖清单，桌面端提示“暂未提供可安装的版本”）。幂等。
pub async fn backfill_expert_packages(pool: &sqlx::PgPool, data_dir: &std::path::Path) -> Result<usize> {
    let rows: Vec<(Uuid, String, String)> = sqlx::query_as(
        "SELECT i.id, i.slug, i.description FROM items i
         WHERE i.kind = 'expert' AND i.variant = 'single' AND length(trim(i.description)) > 0
           AND NOT EXISTS (SELECT 1 FROM versions v WHERE v.item_id = i.id)",
    )
    .fetch_all(pool)
    .await?;
    let mut done = 0;
    for (id, slug, content) in rows {
        let built = package::expert_package(&content, &slug).and_then(|bytes| package::inspect_zip(&bytes).map(|info| (bytes, info)));
        let (bytes, info) = match built {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!("专家 {slug} 无法打成安装包，跳过：{e}");
                continue;
            }
        };
        let sha = package::sha256_hex(&bytes);
        let version = package::expert_version(&info.skill_md, &sha, None);
        storage::store_package(data_dir, &sha, &bytes).await.map_err(|e| AppError::Internal(format!("保存安装包失败：{e}")))?;
        let mut tx = pool.begin().await?;
        sqlx::query(
            "INSERT INTO versions (item_id, version, changelog, manifest, manifest_sha256, package_sha256, package_size)
             VALUES ($1, $2, '', $3, $4, $5, $6) ON CONFLICT (item_id, version) DO NOTHING",
        )
        .bind(id)
        .bind(&version)
        .bind(Json(json!({ "files": info.files })))
        .bind(&info.manifest_sha256)
        .bind(&sha)
        .bind(bytes.len() as i64)
        .execute(&mut *tx)
        .await?;
        sqlx::query("UPDATE items SET latest_version = $2 WHERE id = $1").bind(id).bind(&version).execute(&mut *tx).await?;
        tx.commit().await?;
        done += 1;
    }
    Ok(done)
}

fn expert_source<'a>(n: &'a Normalized, name_en: &'a str, summary_en: &'a str, content: &'a str, category_id: &'a str, version: &'a str) -> convert::Source<'a> {
    convert::Source { slug: &n.slug, name_zh: &n.name, name_en, summary: &n.summary, summary_en, content, category_id, version }
}
