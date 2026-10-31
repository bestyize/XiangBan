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

use crate::users::ForumUser;
use chrono::{DateTime, Utc};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChallengeInput {
    pub title: String,
    pub description: String,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub submission_tag: String,
    pub topic_id: Option<i64>,
    pub expected_revision: i64,
}
#[derive(sqlx::FromRow)]
struct Challenge {
    key: String,
    title: String,
    description: String,
    starts_at: DateTime<Utc>,
    ends_at: DateTime<Utc>,
    submission_tag: String,
    topic_id: Option<i64>,
    status: String,
    revision: i64,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}
const CHALLENGE_COLUMNS:&str="key,title,description,starts_at,ends_at,submission_tag,topic_id,status,revision,created_at,updated_at";
fn valid_key(key: &str) -> bool {
    (2..=48).contains(&key.len()) && content::valid_slug(key)
}
fn clean(input: &str, max: usize) -> AppResult<String> {
    let value = input.trim();
    if value.is_empty()
        || value.chars().count() > max
        || value
            .chars()
            .any(|c| c.is_control() && c != '\n' && c != '\t')
    {
        return Err(AppError::BadRequest(format!(
            "文本须为 1–{max} 字且不能含控制字符"
        )));
    }
    Ok(value.to_string())
}
async fn challenge(conn: &mut PgConnection, key: &str, lock: bool) -> AppResult<Challenge> {
    if !valid_key(key) {
        return Err(AppError::not_found("主题挑战"));
    }
    sqlx::query_as(&format!(
        "SELECT {CHALLENGE_COLUMNS} FROM community_challenges WHERE key=$1 {}",
        if lock { "FOR UPDATE" } else { "" }
    ))
    .bind(key)
    .fetch_optional(conn)
    .await?
    .ok_or_else(|| AppError::not_found("主题挑战"))
}
async fn public_topic(conn: &mut PgConnection, id: Option<i64>) -> AppResult<Option<i64>> {
    let Some(id) = id else {
        return Ok(None);
    };
    let row: Option<(i64,)> = sqlx::query_as(
        "SELECT id FROM topics WHERE id=$1 AND status='normal' AND min_read_level=0",
    )
    .bind(id)
    .fetch_optional(conn)
    .await?;
    Ok(row.map(|r| r.0))
}
impl Challenge {
    fn phase(&self, now: DateTime<Utc>) -> &str {
        match self.status.as_str() {
            "draft" => "draft",
            "closed" => "closed",
            _ if now < self.starts_at => "upcoming",
            _ if now >= self.ends_at => "ended",
            _ => "open",
        }
    }
    fn json(&self, admin: bool, topic: Option<i64>, entry_count: i64) -> Value {
        let mut value = json!({"key":self.key,"title":self.title,"description":self.description,"starts_at":self.starts_at,"ends_at":self.ends_at,
            "submission_tag":self.submission_tag,"topic_id":topic,"status":self.phase(Utc::now()),"entry_count":entry_count,"created_at":self.created_at,"updated_at":self.updated_at});
        if admin {
            value["publication_status"] = json!(self.status);
            value["revision"] = json!(self.revision);
            value["topic_id"] = json!(self.topic_id);
        }
        value
    }
}
async fn entry_count(conn: &mut PgConnection, key: &str, admin: bool) -> AppResult<i64> {
    let (n,):(i64,)=sqlx::query_as(&format!("SELECT count(*) FROM community_challenge_entries e JOIN packs p ON p.id=e.pack_id {JOINS} WHERE e.challenge_key=$1 AND ($2 OR ({ELIGIBLE} AND p.author_uid=e.submitted_by))"))
        .bind(key).bind(admin).fetch_one(conn).await?;
    Ok(n)
}
pub async fn list_challenges(pool: &PgPool, admin: bool, page: i64) -> AppResult<Value> {
    let page = page.clamp(1, 100000);
    let mut tx = pool.begin().await?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY")
        .execute(&mut *tx)
        .await?;
    let rows:Vec<Challenge>=sqlx::query_as(&format!("SELECT {CHALLENGE_COLUMNS} FROM community_challenges WHERE ($1 OR status<>'draft') ORDER BY starts_at DESC,key LIMIT 25 OFFSET $2"))
        .bind(admin).bind((page-1)*25).fetch_all(&mut *tx).await?;
    let (total,): (i64,) =
        sqlx::query_as("SELECT count(*) FROM community_challenges WHERE ($1 OR status<>'draft')")
            .bind(admin)
            .fetch_one(&mut *tx)
            .await?;
    let mut items = Vec::new();
    for row in rows {
        let topic = public_topic(&mut tx, row.topic_id).await?;
        let n = entry_count(&mut tx, &row.key, false).await?;
        items.push(row.json(admin, topic, n));
    }
    tx.commit().await?;
    Ok(json!({"items":items,"total":total,"page":page,"page_size":25}))
}
pub async fn challenge_detail(
    pool: &PgPool,
    key: &str,
    admin: bool,
    page: i64,
) -> AppResult<Value> {
    let page = page.clamp(1, 100000);
    let mut tx = pool.begin().await?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY")
        .execute(&mut *tx)
        .await?;
    let row = challenge(&mut tx, key, false).await?;
    if !admin && row.status == "draft" {
        return Err(AppError::not_found("主题挑战"));
    }
    let total = entry_count(&mut tx, key, admin).await?;
    let visible_count = entry_count(&mut tx, key, false).await?;
    let topic = public_topic(&mut tx, row.topic_id).await?;
    let entries:Vec<(Value,bool,i32,i64,bool,Option<i64>)>=sqlx::query_as(&format!("SELECT {PACK_JSON},e.featured,e.sort_order,e.revision,({ELIGIBLE} AND p.author_uid=e.submitted_by) AS visible,t.like_count::bigint FROM community_challenge_entries e JOIN packs p ON p.id=e.pack_id {JOINS}
        WHERE e.challenge_key=$1 AND ($2 OR ({ELIGIBLE} AND p.author_uid=e.submitted_by)) ORDER BY e.featured DESC,e.sort_order,e.created_at,p.id LIMIT 50 OFFSET $3"))
        .bind(key).bind(admin).bind((page-1)*50).fetch_all(&mut *tx).await?;
    tx.commit().await?;
    let items=entries.into_iter().map(|(pack,featured,sort_order,revision,visible,likes)|{
        let id=pack["id"].clone();let topic=if visible {pack["topic_id"].clone()}else{Value::Null};
        let mut value=json!({"pack_id":id,"pack":pack_json(pack),"featured":featured,"sort_order":sort_order,"vote_method":"topic_like","vote_topic_id":topic,"likes":if topic.is_null(){None}else{likes}});
        if admin {value["revision"]=json!(revision);value["visible"]=json!(visible);} value
    }).collect::<Vec<_>>();
    Ok(
        json!({"challenge":row.json(admin,topic,visible_count),"entries":{"items":items,"page":page,"page_size":50,"total":total}}),
    )
}
pub async fn save_challenge(
    pool: &PgPool,
    operator: &str,
    key: &str,
    mut input: ChallengeInput,
) -> AppResult<()> {
    if !valid_key(key) {
        return Err(AppError::BadRequest(
            "挑战标识须为 2–48 位小写字母、数字、连字符或下划线".into(),
        ));
    }
    input.title = clean(&input.title, 80)?;
    input.description = clean(&input.description, 3000)?;
    input.submission_tag = input.submission_tag.trim().to_ascii_lowercase();
    if input.submission_tag.len() > 40
        || !content::valid_slug(&input.submission_tag)
        || input.ends_at <= input.starts_at
        || input.expected_revision < 0
    {
        return Err(AppError::BadRequest(
            "投稿标签须为有效短标识，截止时间须晚于开始时间".into(),
        ));
    }
    let mut tx = pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
        .bind(format!("challenge:{key}"))
        .execute(&mut *tx)
        .await?;
    let old: Option<(String, i64)> =
        sqlx::query_as("SELECT status,revision FROM community_challenges WHERE key=$1 FOR UPDATE")
            .bind(key)
            .fetch_optional(&mut *tx)
            .await?;
    if old.as_ref().map_or(0, |r| r.1) != input.expected_revision {
        return Err(AppError::Conflict("挑战已被修改，请重新读取".into()));
    }
    if old.as_ref().is_some_and(|r| r.0 != "draft") {
        return Err(AppError::Conflict(
            "已发布的挑战不能直接改正文，请先撤回为草稿".into(),
        ));
    }
    if input.topic_id.is_some() && public_topic(&mut tx, input.topic_id).await? != input.topic_id {
        return Err(AppError::BadRequest("讨论须为公开可读的正常帖子".into()));
    }
    sqlx::query("INSERT INTO community_challenges(key,title,description,starts_at,ends_at,submission_tag,topic_id,created_by,updated_by) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$8)
        ON CONFLICT(key) DO UPDATE SET title=$2,description=$3,starts_at=$4,ends_at=$5,submission_tag=$6,topic_id=$7,updated_by=$8,updated_at=clock_timestamp(),revision=nextval('community_editor_revision_seq')")
        .bind(key).bind(&input.title).bind(&input.description).bind(input.starts_at).bind(input.ends_at).bind(&input.submission_tag).bind(input.topic_id).bind(operator).execute(&mut *tx).await?;
    log(
        &mut tx,
        operator,
        "community.challenge.save",
        "community_challenge",
        key,
        "保存待审草稿",
    )
    .await?;
    tx.commit().await?;
    Ok(())
}
pub async fn transition_challenge(
    pool: &PgPool,
    operator: &str,
    key: &str,
    revision: i64,
    action: &str,
) -> AppResult<()> {
    let mut tx = pool.begin().await?;
    let row = challenge(&mut tx, key, true).await?;
    if row.revision != revision {
        return Err(AppError::Conflict("挑战已被修改，请重新读取后审核".into()));
    }
    let status = match action {
        "publish" if row.status == "draft" && row.ends_at > Utc::now() => "published",
        "close" if row.status == "published" => "closed",
        "unpublish" if row.status == "published" => "draft",
        _ => {
            return Err(AppError::Conflict(
                "当前状态不能执行此操作，或截止时间已经过去".into(),
            ))
        }
    };
    if action == "publish"
        && row.topic_id.is_some()
        && public_topic(&mut tx, row.topic_id).await? != row.topic_id
    {
        return Err(AppError::BadRequest(
            "关联讨论已不可公开，请先修改草稿".into(),
        ));
    }
    sqlx::query("UPDATE community_challenges SET status=$2,revision=nextval('community_editor_revision_seq'),updated_by=$3,updated_at=clock_timestamp() WHERE key=$1")
        .bind(key).bind(status).bind(operator).execute(&mut *tx).await?;
    log(
        &mut tx,
        operator,
        &format!("community.challenge.{action}"),
        "community_challenge",
        key,
        &format!("reviewed_revision={revision}"),
    )
    .await?;
    tx.commit().await?;
    Ok(())
}
pub async fn submit_entry(
    pool: &PgPool,
    key: &str,
    pack_id: i64,
    user: &ForumUser,
) -> AppResult<()> {
    crate::users::ensure_can_write(user)?;
    if user.status == "banned" {
        return Err(AppError::Forbidden("账号已被封禁".into()));
    }
    let mut tx = pool.begin().await?;
    let row = challenge(&mut tx, key, true).await?;
    if row.status == "draft" {
        return Err(AppError::not_found("主题挑战"));
    }
    let owner: Option<(Option<uuid::Uuid>,)> = sqlx::query_as(&format!(
        "SELECT p.author_uid FROM packs p {JOINS} WHERE p.id=$1 AND {ELIGIBLE} FOR SHARE OF p"
    ))
    .bind(pack_id)
    .fetch_optional(&mut *tx)
    .await?;
    let owner = owner
        .ok_or_else(|| AppError::BadRequest("只能提交已上架且公开可读的作品".into()))?
        .0;
    if owner != Some(user.uid) {
        return Err(AppError::Forbidden("只能提交自己的作品".into()));
    }
    let (open,):(bool,)=sqlx::query_as("SELECT status='published' AND starts_at<=clock_timestamp() AND ends_at>clock_timestamp() FROM community_challenges WHERE key=$1")
        .bind(key).fetch_one(&mut *tx).await?;
    if !open {
        return Err(AppError::Conflict("挑战尚未开始、已经截止或已关闭".into()));
    }
    sqlx::query("INSERT INTO community_challenge_entries(challenge_key,pack_id,submitted_by) VALUES($1,$2,$3) ON CONFLICT(challenge_key,pack_id) DO NOTHING")
        .bind(key).bind(pack_id).bind(user.uid).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(())
}
pub async fn feature_entry(
    pool: &PgPool,
    operator: &str,
    key: &str,
    pack_id: i64,
    revision: i64,
    featured: bool,
    sort_order: i32,
) -> AppResult<()> {
    let mut tx = pool.begin().await?;
    let row = challenge(&mut tx, key, false).await?;
    if row.status == "draft" {
        return Err(AppError::Conflict("挑战未发布，不能精选投稿".into()));
    }
    let entry:Option<(i64,bool)>=sqlx::query_as(&format!("SELECT e.revision,({ELIGIBLE} AND p.author_uid=e.submitted_by) FROM community_challenge_entries e JOIN packs p ON p.id=e.pack_id {JOINS} WHERE e.challenge_key=$1 AND e.pack_id=$2 FOR UPDATE OF e"))
        .bind(key).bind(pack_id).fetch_optional(&mut *tx).await?;
    let (old, visible) = entry.ok_or_else(|| AppError::not_found("投稿作品"))?;
    if old != revision {
        return Err(AppError::Conflict("投稿精选已被修改，请重新读取".into()));
    }
    if featured && !visible {
        return Err(AppError::BadRequest(
            "只能精选当前公开可读的投稿作品".into(),
        ));
    }
    sqlx::query("UPDATE community_challenge_entries SET featured=$3,sort_order=$4,revision=nextval('community_editor_revision_seq'),updated_at=clock_timestamp() WHERE challenge_key=$1 AND pack_id=$2")
        .bind(key).bind(pack_id).bind(featured).bind(sort_order).execute(&mut *tx).await?;
    log(
        &mut tx,
        operator,
        "community.challenge.entry.feature",
        "community_challenge",
        key,
        &format!("pack_id={pack_id}; featured={featured}; sort_order={sort_order}"),
    )
    .await?;
    tx.commit().await?;
    Ok(())
}
