//! 社区人工编排。只引用真实内容包，不制造使用量、投票或积分。
pub mod schema;
use crate::{
    content,
    error::{AppError, AppResult},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{PgConnection, PgPool};

const JOINS: &str = "LEFT JOIN users u ON u.uid=p.author_uid LEFT JOIN topics t ON t.id=p.topic_id";
const ELIGIBLE:&str="p.status='published' AND (p.author_uid IS NULL OR u.status<>'banned') AND (p.topic_id IS NULL OR (t.status='normal' AND t.min_read_level=0))";
const PACK_JSON:&str="jsonb_build_object('id',p.id,'kind',p.kind,'slug',p.slug,'title',p.title,'summary',p.summary,'tags',p.tags,
 'author',CASE WHEN p.author_uid IS NULL THEN NULL ELSE jsonb_build_object('uid',p.author_uid,'nickname',u.nickname) END,
 'trust_level',p.trust_level,'status',p.status,'version',p.current_version,'cover_key',p.cover_key,'cover_url',NULL,
 'apply_count',p.apply_count,'topic_id',p.topic_id,'created_at',p.created_at,'updated_at',p.updated_at)";
fn pack_json(mut value: Value) -> Value {
    value["one_click"] = json!(content::one_click_allowed(
        value["kind"].as_str().unwrap_or_default(),
        value["trust_level"].as_i64().unwrap_or(2) as i16,
        value["status"].as_str().unwrap_or_default()
    ));
    value["pure_data"] = json!(content::is_pure_data(
        value["kind"].as_str().unwrap_or_default()
    ));
    value
}
async fn log(
    conn: &mut PgConnection,
    operator: &str,
    action: &str,
    target_type: &str,
    target: &str,
    detail: &str,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO admin_audit(admin,action,target_type,target_id,detail) VALUES($1,$2,$3,$4,$5)",
    )
    .bind(operator)
    .bind(action)
    .bind(target_type)
    .bind(target)
    .bind(detail)
    .execute(conn)
    .await?;
    Ok(())
}
/// Home 兼容作品字段；宿主按 cover_key 补资源机 cover_url。
pub async fn featured(pool: &PgPool, limit: i64) -> AppResult<Vec<Value>> {
    let rows:Vec<(Value,)>=sqlx::query_as(&format!("SELECT {PACK_JSON} FROM community_featured f JOIN packs p ON p.id=f.pack_id {JOINS} WHERE {ELIGIBLE} ORDER BY f.sort_order,p.id DESC LIMIT $1"))
        .bind(limit.clamp(1,30)).fetch_all(pool).await?;
    Ok(rows.into_iter().map(|(value,)| pack_json(value)).collect())
}
/// 供宿主复用既有 PackRow 序列化。
pub async fn featured_ids(pool: &PgPool, limit: i64) -> AppResult<Vec<i64>> {
    let rows:Vec<(i64,)>=sqlx::query_as(&format!("SELECT p.id FROM community_featured f JOIN packs p ON p.id=f.pack_id {JOINS} WHERE {ELIGIBLE} ORDER BY f.sort_order,p.id DESC LIMIT $1"))
        .bind(limit.clamp(1,30)).fetch_all(pool).await?;
    Ok(rows.into_iter().map(|r| r.0).collect())
}
pub async fn featured_admin(pool: &PgPool) -> AppResult<Value> {
    let mut tx = pool.begin().await?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY")
        .execute(&mut *tx)
        .await?;
    let (revision,): (i64,) =
        sqlx::query_as("SELECT revision FROM community_featured_state WHERE singleton=true")
            .fetch_one(&mut *tx)
            .await?;
    let rows:Vec<(Value,i32,bool)>=sqlx::query_as(&format!("SELECT {PACK_JSON},f.sort_order,({ELIGIBLE}) AS visible FROM community_featured f JOIN packs p ON p.id=f.pack_id {JOINS} ORDER BY f.sort_order,p.id DESC"))
        .fetch_all(&mut *tx).await?;
    tx.commit().await?;
    Ok(
        json!({"revision":revision,"items":rows.into_iter().map(|(pack,sort_order,visible)|{let pack_id=pack["id"].clone();json!({"pack":pack_json(pack),"pack_id":pack_id,"sort_order":sort_order,"visible":visible})}).collect::<Vec<_>>()}),
    )
}
pub async fn candidates(pool: &PgPool, q: &str) -> AppResult<Vec<Value>> {
    if q.chars().count() > 100 {
        return Err(AppError::BadRequest("搜索词最多 100 字".into()));
    }
    let escaped = q
        .trim()
        .chars()
        .flat_map(|c| {
            if matches!(c, '%' | '_' | '\\') {
                vec!['\\', c]
            } else {
                vec![c]
            }
        })
        .collect::<String>();
    let rows:Vec<(Value,)>=sqlx::query_as(&format!("SELECT {PACK_JSON} FROM packs p {JOINS} WHERE {ELIGIBLE} AND (p.title ILIKE $1 OR p.slug ILIKE $1) ORDER BY p.updated_at DESC,p.id DESC LIMIT 30"))
        .bind(format!("%{escaped}%")).fetch_all(pool).await?;
    Ok(rows.into_iter().map(|r| pack_json(r.0)).collect())
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FeaturedItem {
    pub pack_id: i64,
    pub sort_order: i32,
}
pub async fn replace_featured(
    pool: &PgPool,
    operator: &str,
    expected_revision: i64,
    items: Vec<FeaturedItem>,
) -> AppResult<()> {
    if items.len() > 30 {
        return Err(AppError::BadRequest("最多精选 30 件作品".into()));
    }
    let mut ids = items.iter().map(|v| v.pack_id).collect::<Vec<_>>();
    ids.sort();
    if ids.iter().any(|id| *id <= 0) || ids.windows(2).any(|w| w[0] == w[1]) {
        return Err(AppError::BadRequest("作品编号不能重复且必须有效".into()));
    }
    let mut tx = pool.begin().await?;
    let (revision,): (i64,) = sqlx::query_as(
        "SELECT revision FROM community_featured_state WHERE singleton=true FOR UPDATE",
    )
    .fetch_one(&mut *tx)
    .await?;
    if revision != expected_revision {
        return Err(AppError::Conflict(
            "精选已被其他管理员修改，请重新加载".into(),
        ));
    }
    let eligible:Vec<(i64,)>=sqlx::query_as(&format!("SELECT p.id FROM packs p {JOINS} WHERE p.id=ANY($1) AND {ELIGIBLE} ORDER BY p.id FOR SHARE OF p"))
        .bind(&ids).fetch_all(&mut *tx).await?;
    if eligible.len() != ids.len() {
        return Err(AppError::BadRequest(
            "只能精选已上架、作者正常且公开可读的作品".into(),
        ));
    }
    sqlx::query("DELETE FROM community_featured")
        .execute(&mut *tx)
        .await?;
    for item in items {
        sqlx::query("INSERT INTO community_featured(pack_id,sort_order) VALUES($1,$2)")
            .bind(item.pack_id)
            .bind(item.sort_order)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("UPDATE community_featured_state SET revision=revision+1 WHERE singleton=true")
        .execute(&mut *tx)
        .await?;
    log(
        &mut tx,
        operator,
        "community.featured.replace",
        "community_featured",
        "home",
        &format!("pack_ids={ids:?}"),
    )
    .await?;
    tx.commit().await?;
    Ok(())
}
