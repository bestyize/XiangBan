//! 公开目录 API：`/api/v1/market/*`，供网页和小伴桌面客户端使用（只读，不需要登录）。
use crate::{
    error::{AppError, Result},
    package::sha256_hex,
    state::AppState,
    storage,
};
use axum::{
    body::Body,
    extract::{Path, Query, Request, State},
    http::{header, HeaderValue},
    response::Response,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::{types::Json as PgJson, PgPool};
use tower::ServiceExt;
use tower_http::services::ServeFile;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/categories", get(categories))
        .route("/suggest", get(suggest))
        .route("/updates", post(updates))
        .route("/{kind}", get(list))
        .route("/{kind}/{namespace}/{slug}", get(detail))
        .route("/{kind}/{namespace}/{slug}/versions/{version}/download", get(download))
}

pub fn kind_of(plural: &str) -> Result<&'static str> {
    match plural {
        "skills" | "skill" => Ok("skill"),
        "experts" | "expert" => Ok("expert"),
        "connectors" | "connector" => Ok("connector"),
        _ => Err(AppError::NotFound("未知的类型，应为 skills / experts / connectors".into())),
    }
}

fn plural(kind: &str) -> &'static str {
    match kind {
        "skill" => "skills",
        "expert" => "experts",
        _ => "connectors",
    }
}

#[derive(sqlx::FromRow)]
pub struct ItemRow {
    id: Uuid,
    kind: String,
    namespace: String,
    slug: String,
    name: String,
    summary: String,
    category_slug: Option<String>,
    category_name: Option<String>,
    tags: Vec<String>,
    icon_url: Option<String>,
    source: String,
    origin: String,
    variant: String,
    member_count: Option<i32>,
    latest_version: Option<String>,
    download_count: i64,
    favorite_count: i64,
    featured: bool,
    updated_at: DateTime<Utc>,
    total: i64,
}

fn item_json(r: &ItemRow) -> Value {
    json!({
        "id": r.id, "kind": r.kind, "namespace": r.namespace, "slug": r.slug, "name": r.name, "summary": r.summary,
        "category": r.category_slug.as_ref().map(|s| json!({"slug": s, "name": r.category_name})),
        "tags": r.tags, "iconUrl": r.icon_url, "source": r.source, "origin": r.origin, "variant": r.variant, "memberCount": r.member_count,
        "version": r.latest_version, "downloads": r.download_count, "favorites": r.favorite_count, "heat": r.download_count + r.favorite_count * 5,
        "featured": r.featured, "updatedAt": r.updated_at,
    })
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ListQuery {
    keyword: String,
    category: String,
    source: String,
    sort_by: String,
    variant: String,
    origin: String,
    page: i64,
    page_size: i64,
}

/// 关键词拆成最多 5 个词，每个词做不区分大小写的子串匹配；`%`、`_`、`\` 已转义。
pub fn patterns(keyword: &str) -> Vec<String> {
    keyword
        .split_whitespace()
        .take(5)
        .map(|t| {
            let t: String = t.chars().take(40).collect::<String>().to_lowercase();
            format!("%{}%", t.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_"))
        })
        .collect()
}

pub struct SearchArgs {
    pub keyword: String,
    pub category: Option<String>,
    pub source: Option<String>,
    pub sort: String,
    pub variant: Option<String>,
    pub origin: Option<String>,
    pub limit: i64,
    pub offset: i64,
    pub exclude: Vec<Uuid>,
}

const SELECT: &str = "SELECT i.id, i.kind, i.namespace, i.slug, i.name, i.summary, c.slug AS category_slug, c.name AS category_name,
  i.tags, i.icon_url, i.source, i.origin, i.variant, jsonb_array_length(i.metadata->'members') AS member_count,
  i.latest_version, i.download_count, i.favorite_count, i.featured, i.updated_at,
  count(*) OVER() AS total
  FROM items i LEFT JOIN categories c ON c.id = i.category_id
  WHERE i.status = 'published' AND i.kind = $1 AND ($2::text IS NULL OR c.slug = $2) AND ($3::text IS NULL OR i.source = $3)
    AND NOT (i.id = ANY($4::uuid[])) AND ($5::text IS NULL OR i.variant = $5) AND ($6::text IS NULL OR i.origin = $6)";

const HEAT: &str = "(i.download_count + i.favorite_count * 5)";

pub async fn search(pool: &PgPool, kind: &str, a: &SearchArgs, fuzzy: bool) -> Result<Vec<ItemRow>> {
    let first = a.keyword.split_whitespace().next().unwrap_or("").to_lowercase();
    let rows = if fuzzy {
        let sql = format!(
            "{SELECT} AND (similarity(i.search_text, $9) > 0.15 OR word_similarity($9, i.search_text) > 0.4)
             ORDER BY GREATEST(similarity(i.search_text, $9), word_similarity($9, i.search_text)) DESC, {HEAT} DESC LIMIT $7 OFFSET $8"
        );
        sqlx::query_as::<_, ItemRow>(&sql)
            .bind(kind)
            .bind(&a.category)
            .bind(&a.source)
            .bind(&a.exclude)
            .bind(&a.variant)
            .bind(&a.origin)
            .bind(a.limit)
            .bind(a.offset)
            .bind(a.keyword.to_lowercase())
            .fetch_all(pool)
            .await?
    } else {
        let rank = "CASE WHEN lower(i.name) = $10 THEN 0 WHEN strpos(lower(i.name), $10) = 1 THEN 1 WHEN strpos(lower(i.name), $10) > 0 THEN 2
                    WHEN strpos(lower(array_to_string(i.tags, ' ')), $10) > 0 THEN 3 ELSE 4 END";
        let order = match a.sort.as_str() {
            "hot" | "downloads" => format!("{HEAT} DESC, i.updated_at DESC"),
            "new" | "updated" => "i.published_at DESC NULLS LAST, i.updated_at DESC".to_string(),
            _ if !first.is_empty() => format!("{rank}, i.featured DESC, {HEAT} DESC"),
            _ => format!("i.featured DESC, {HEAT} DESC, i.updated_at DESC"),
        };
        let sql = format!("{SELECT} AND i.search_text ILIKE ALL($9::text[]) AND $10::text IS NOT NULL ORDER BY {order} LIMIT $7 OFFSET $8");
        sqlx::query_as::<_, ItemRow>(&sql)
            .bind(kind)
            .bind(&a.category)
            .bind(&a.source)
            .bind(&a.exclude)
            .bind(&a.variant)
            .bind(&a.origin)
            .bind(a.limit)
            .bind(a.offset)
            .bind(patterns(&a.keyword))
            .bind(first)
            .fetch_all(pool)
            .await?
    };
    Ok(rows)
}

async fn list(State(state): State<AppState>, Path(kind): Path<String>, Query(q): Query<ListQuery>) -> Result<Json<Value>> {
    let kind = kind_of(&kind)?;
    let page = q.page.max(1);
    let page_size = if q.page_size <= 0 { 24 } else { q.page_size.min(50) };
    let keyword: String = q.keyword.trim().chars().take(80).collect();
    let args = SearchArgs {
        keyword: keyword.clone(),
        category: Some(q.category.trim().to_string()).filter(|s| !s.is_empty() && s != "all"),
        source: Some(q.source.trim().to_string()).filter(|s| matches!(s.as_str(), "official" | "community" | "enterprise")),
        sort: q.sort_by.clone(),
        variant: Some(q.variant.trim().to_string()).filter(|s| matches!(s.as_str(), "single" | "team" | "bundle")),
        origin: Some(q.origin.trim().to_string()).filter(|s| !s.is_empty() && s.len() <= 40),
        limit: page_size,
        offset: (page - 1) * page_size,
        exclude: vec![],
    };
    let rows = search(&state.pool, kind, &args, false).await?;
    let total = rows.first().map(|r| r.total).unwrap_or(0);
    // 精确结果很少时，在第一页补充相近匹配（容错：错别字、拼写变体）。
    let mut near = Vec::new();
    if page == 1 && !keyword.is_empty() && total < 5 {
        let near_args = SearchArgs { limit: 6, offset: 0, exclude: rows.iter().map(|r| r.id).collect(), ..args };
        near = search(&state.pool, kind, &near_args, true).await?;
    }
    Ok(Json(json!({
        "results": rows.iter().map(item_json).collect::<Vec<_>>(),
        "near": near.iter().map(item_json).collect::<Vec<_>>(),
        "page": page, "pageSize": page_size, "total": total, "hasMore": page * page_size < total,
    })))
}

#[derive(Deserialize)]
struct KindQuery {
    kind: String,
}

async fn categories(State(state): State<AppState>, Query(q): Query<KindQuery>) -> Result<Json<Value>> {
    let kind = kind_of(&q.kind)?;
    let rows: Vec<(String, String, i64)> = sqlx::query_as(
        "SELECT c.slug, c.name, count(i.id) FROM categories c
         LEFT JOIN items i ON i.category_id = c.id AND i.status = 'published'
         WHERE c.kind = $1 AND c.active GROUP BY c.id ORDER BY c.sort_order, c.id",
    )
    .bind(kind)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(json!({ "results": rows.into_iter().map(|(slug, name, count)| json!({"slug": slug, "name": name, "count": count})).collect::<Vec<_>>() })))
}

#[derive(Deserialize)]
struct SuggestQuery {
    kind: String,
    #[serde(default)]
    q: String,
}

async fn suggest(State(state): State<AppState>, Query(q): Query<SuggestQuery>) -> Result<Json<Value>> {
    let kind = kind_of(&q.kind)?;
    let keyword: String = q.q.trim().chars().take(40).collect::<String>().to_lowercase();
    if keyword.is_empty() {
        return Ok(Json(json!({ "results": [] })));
    }
    let like = format!("%{}%", keyword.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_"));
    let rows: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT name, namespace, slug FROM items WHERE status = 'published' AND kind = $1 AND search_text ILIKE $2
         ORDER BY (strpos(lower(name), $3) = 1) DESC, (download_count + favorite_count * 5) DESC LIMIT 8",
    )
    .bind(kind)
    .bind(like)
    .bind(&keyword)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(json!({ "results": rows.into_iter().map(|(name, namespace, slug)| json!({"name": name, "namespace": namespace, "slug": slug})).collect::<Vec<_>>() })))
}

#[derive(sqlx::FromRow)]
struct DetailRow {
    id: Uuid,
    kind: String,
    namespace: String,
    slug: String,
    name: String,
    summary: String,
    description: String,
    category_slug: Option<String>,
    category_name: Option<String>,
    tags: Vec<String>,
    icon_url: Option<String>,
    source: String,
    origin: String,
    origin_url: Option<String>,
    license: Option<String>,
    variant: String,
    latest_version: Option<String>,
    download_count: i64,
    favorite_count: i64,
    featured: bool,
    prompts: PgJson<Value>,
    metadata: PgJson<Value>,
    updated_at: DateTime<Utc>,
    published_at: Option<DateTime<Utc>>,
}

#[derive(sqlx::FromRow)]
struct VersionRow {
    version: String,
    changelog: String,
    manifest: PgJson<Value>,
    manifest_sha256: String,
    package_sha256: String,
    package_size: i64,
    yanked: bool,
    created_at: DateTime<Utc>,
}

async fn detail(State(state): State<AppState>, Path((kind, namespace, slug)): Path<(String, String, String)>) -> Result<Json<Value>> {
    let kind = kind_of(&kind)?;
    let r: DetailRow = sqlx::query_as(
        "SELECT i.id, i.kind, i.namespace, i.slug, i.name, i.summary, i.description, c.slug AS category_slug, c.name AS category_name,
           i.tags, i.icon_url, i.source, i.origin, i.origin_url, i.license, i.variant, i.latest_version, i.download_count, i.favorite_count,
           i.featured, i.prompts, i.metadata, i.updated_at, i.published_at
         FROM items i LEFT JOIN categories c ON c.id = i.category_id
         WHERE i.kind = $1 AND i.namespace = $2 AND i.slug = $3 AND i.status = 'published'",
    )
    .bind(kind)
    .bind(&namespace)
    .bind(&slug)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("条目不存在或未上架".into()))?;
    let versions: Vec<VersionRow> = sqlx::query_as(
        "SELECT version, changelog, manifest, manifest_sha256, package_sha256, package_size, yanked, created_at
         FROM versions WHERE item_id = $1 ORDER BY created_at DESC, version DESC",
    )
    .bind(r.id)
    .fetch_all(&state.pool)
    .await?;
    let members = if r.variant == "single" {
        vec![]
    } else {
        // 团队的成员是专家，套件的成员是技能；只展示已上架的，并保持登记的顺序。
        let member_kind = if r.variant == "team" { "expert" } else { "skill" };
        sqlx::query_scalar::<_, PgJson<Value>>(
            "SELECT to_jsonb(x) FROM (
               SELECT i.kind, i.namespace, i.slug, i.name, i.summary, i.icon_url AS \"iconUrl\", i.latest_version AS version, i.variant
               FROM jsonb_array_elements($1::jsonb) WITH ORDINALITY AS m(v, n)
               JOIN items i ON i.kind = $2 AND i.namespace = m.v->>'namespace' AND i.slug = m.v->>'slug' AND i.status = 'published'
               ORDER BY m.n) x",
        )
        .bind(PgJson(r.metadata.0["members"].clone()))
        .bind(member_kind)
        .fetch_all(&state.pool)
        .await?
        .into_iter()
        .map(|j| j.0)
        .collect()
    };
    let base = format!("/api/v1/market/{}/{}/{}", plural(&r.kind), r.namespace, r.slug);
    let latest = versions.iter().find(|v| !v.yanked);
    Ok(Json(json!({
        "item": {
            "id": r.id, "kind": r.kind, "namespace": r.namespace, "slug": r.slug, "name": r.name, "summary": r.summary,
            "description": r.description, "category": r.category_slug.as_ref().map(|s| json!({"slug": s, "name": r.category_name})),
            "tags": r.tags, "iconUrl": r.icon_url, "source": r.source, "origin": r.origin, "originUrl": r.origin_url,
            "license": r.license, "variant": r.variant, "version": r.latest_version, "downloads": r.download_count, "favorites": r.favorite_count,
            "heat": r.download_count + r.favorite_count * 5, "featured": r.featured, "prompts": r.prompts.0, "metadata": r.metadata.0,
            "updatedAt": r.updated_at, "publishedAt": r.published_at,
        },
        "versions": versions.iter().map(|v| json!({
            "version": v.version, "changelog": v.changelog, "size": v.package_size, "yanked": v.yanked, "createdAt": v.created_at,
            "manifestSha256": v.manifest_sha256, "packageSha256": v.package_sha256,
            "downloadUrl": (!v.yanked).then(|| format!("{base}/versions/{}/download", v.version)),
        })).collect::<Vec<_>>(),
        "files": latest.map(|v| v.manifest["files"].clone()).unwrap_or(json!([])),
        "members": members,
    })))
}

async fn download(
    State(state): State<AppState>,
    Path((kind, namespace, slug, version)): Path<(String, String, String, String)>,
    req: Request,
) -> Result<Response> {
    let kind = kind_of(&kind)?;
    let row: Option<(Uuid, String, bool)> = sqlx::query_as(
        "SELECT v.id, v.package_sha256, v.yanked FROM versions v JOIN items i ON i.id = v.item_id
         WHERE i.kind = $1 AND i.namespace = $2 AND i.slug = $3 AND v.version = $4 AND i.status = 'published'",
    )
    .bind(kind)
    .bind(&namespace)
    .bind(&slug)
    .bind(&version)
    .fetch_optional(&state.pool)
    .await?;
    let (version_id, sha, yanked) = row.ok_or_else(|| AppError::NotFound("版本不存在".into()))?;
    if yanked {
        return Err(AppError::NotFound("该版本已撤回，请使用更新的版本".into()));
    }
    // 同一天同一来源对同一版本只计一次；带 Range 的续传请求不重复计数。
    if !req.headers().contains_key(header::RANGE) {
        let ip = req
            .headers()
            .get("x-forwarded-for")
            .or_else(|| req.headers().get("x-real-ip"))
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.split(',').next())
            .unwrap_or("-")
            .trim()
            .to_string();
        let client = sha256_hex(ip.as_bytes())[..16].to_string();
        let fresh = sqlx::query("INSERT INTO download_log (version_id, day, client) VALUES ($1, current_date, $2) ON CONFLICT DO NOTHING")
            .bind(version_id)
            .bind(client)
            .execute(&state.pool)
            .await?
            .rows_affected()
            == 1;
        if fresh {
            sqlx::query("UPDATE items SET download_count = download_count + 1 WHERE kind = $1 AND namespace = $2 AND slug = $3")
                .bind(kind)
                .bind(&namespace)
                .bind(&slug)
                .execute(&state.pool)
                .await?;
        }
    }
    let path = storage::package_path(&state.config.data_dir, &sha);
    let mut response = ServeFile::new(path)
        .oneshot(req)
        .await
        .map_err(|_| AppError::Internal("读取安装包失败".into()))?
        .map(Body::new);
    if response.status() == axum::http::StatusCode::NOT_FOUND {
        return Err(AppError::Internal("安装包文件缺失".into()));
    }
    let headers = response.headers_mut();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("application/zip"));
    headers.insert(header::CONTENT_DISPOSITION, HeaderValue::from_str(&format!("attachment; filename=\"{slug}-{version}.zip\"")).map_err(|e| AppError::Internal(e.to_string()))?);
    headers.insert("x-content-sha256", HeaderValue::from_str(&sha).map_err(|e| AppError::Internal(e.to_string()))?);
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("public, max-age=31536000, immutable"));
    Ok(response)
}

#[derive(Deserialize)]
struct UpdateRequest {
    items: Vec<InstalledItem>,
}

#[derive(Deserialize)]
struct InstalledItem {
    #[serde(default = "skill")]
    kind: String,
    namespace: String,
    slug: String,
    #[serde(default)]
    version: String,
}

fn skill() -> String {
    "skill".into()
}

/// 客户端上报已安装的条目，返回哪些有更新（用于「版本自动升级」）。
async fn updates(State(state): State<AppState>, Json(req): Json<UpdateRequest>) -> Result<Json<Value>> {
    if req.items.len() > 200 {
        return Err(AppError::BadRequest("一次最多检查 200 个条目".into()));
    }
    let mut results = Vec::new();
    for it in req.items {
        let kind = kind_of(&it.kind)?;
        let latest: Option<(String, String, DateTime<Utc>)> = sqlx::query_as(
            "SELECT v.version, v.manifest_sha256, v.created_at FROM versions v JOIN items i ON i.id = v.item_id
             WHERE i.kind = $1 AND i.namespace = $2 AND i.slug = $3 AND i.status = 'published' AND NOT v.yanked
             ORDER BY v.created_at DESC, v.version DESC LIMIT 1",
        )
        .bind(kind)
        .bind(&it.namespace)
        .bind(&it.slug)
        .fetch_optional(&state.pool)
        .await?;
        let Some((version, manifest, created)) = latest else {
            results.push(json!({"kind": kind, "namespace": it.namespace, "slug": it.slug, "installed": it.version, "available": false}));
            continue;
        };
        let installed_at: Option<DateTime<Utc>> = sqlx::query_scalar(
            "SELECT v.created_at FROM versions v JOIN items i ON i.id = v.item_id
             WHERE i.kind = $1 AND i.namespace = $2 AND i.slug = $3 AND v.version = $4",
        )
        .bind(kind)
        .bind(&it.namespace)
        .bind(&it.slug)
        .bind(&it.version)
        .fetch_optional(&state.pool)
        .await?;
        let has_update = version != it.version && installed_at.map(|t| created > t).unwrap_or(true);
        results.push(json!({
            "kind": kind, "namespace": it.namespace, "slug": it.slug, "installed": it.version, "available": true,
            "latest": version, "hasUpdate": has_update, "manifestSha256": manifest,
            "downloadUrl": format!("/api/v1/market/{}/{}/{}/versions/{}/download", plural(kind), it.namespace, it.slug, version),
        }));
    }
    Ok(Json(json!({ "results": results })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyword_patterns_escape_wildcards_and_limit_terms() {
        assert_eq!(patterns("PDF 工具"), vec!["%pdf%", "%工具%"]);
        assert_eq!(patterns("100%_a\\b"), vec!["%100\\%\\_a\\\\b%"]);
        assert_eq!(patterns("a b c d e f g").len(), 5);
        assert!(patterns("   ").is_empty());
    }

    #[test]
    fn kinds_accept_singular_and_plural() {
        assert_eq!(kind_of("skills").unwrap(), "skill");
        assert_eq!(kind_of("connector").unwrap(), "connector");
        assert!(kind_of("apps").is_err());
    }
}
