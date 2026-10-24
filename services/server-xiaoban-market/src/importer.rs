//! 暂存项入库：二次抓取详情 / 安装包、校验、解析分类，并在一个事务里写入条目与版本。
use crate::{
    error::{AppError, Result},
    items, package, storage,
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
    match enrichment {
        Enrichment::Package { version, changelog, bytes } => {
            let info = package::inspect_zip(&bytes).map_err(|e| AppError::BadRequest(format!("安装包未通过校验：{e}")))?;
            let (_, front_description) = package::frontmatter(&info.skill_md);
            if summary.is_empty() {
                summary = front_description.unwrap_or_default();
            }
            description = info.skill_md.clone();
            latest_version = Some(version.clone());
            let sha = package::sha256_hex(&bytes);
            storage::store_package(&state.config.data_dir, &sha, &bytes).await.map_err(|e| AppError::Internal(format!("保存安装包失败：{e}")))?;
            package_row = Some((version, changelog, bytes.len() as i64, sha, info));
        }
        Enrichment::Expert { content, dependencies } => {
            description = content;
            metadata["dependencies"] = dependencies;
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
    let origin = source.config["origin"].as_str().map(str::to_string).unwrap_or_else(|| source.key.clone());
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

    if let Some((version, changelog, size, sha, info)) = &package_row {
        sqlx::query(
            "INSERT INTO versions (item_id, version, changelog, manifest, manifest_sha256, package_sha256, package_size)
             VALUES ($1,$2,$3,$4,$5,$6,$7) ON CONFLICT (item_id, version) DO NOTHING",
        )
        .bind(item_id)
        .bind(version)
        .bind(changelog)
        .bind(Json(json!({ "files": info.files })))
        .bind(&info.manifest_sha256)
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
