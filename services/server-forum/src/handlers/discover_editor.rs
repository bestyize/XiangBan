//! 发现页编辑副本、人工终审、定时发布与仅能写草稿的采集令牌。
use super::admin::{audit, AuthAdmin};
use crate::{
    discover,
    error::{AppError, AppResult},
    state::AppState,
};
use axum::{
    extract::{FromRequestParts, Path, Query, State},
    http::{header, request::Parts},
    routing::{get, post, put},
    Json, Router,
};
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{PgConnection, PgPool};
use std::collections::HashSet;
use uuid::Uuid;

pub fn admin_routes() -> Router<AppState> {
    Router::new().nest(
        "/discover-workflow",
        Router::new()
            .merge(editor_routes())
            .route("/drafts/{kind}/{key}/review", post(review))
            .route("/drafts/{kind}/{key}/cancel", post(cancel))
            .route("/drafts/{kind}/{key}/discard", post(discard))
            .route("/tokens", get(tokens).post(create_token))
            .route("/tokens/{id}", axum::routing::delete(revoke_token)),
    )
}

/// 可供本地采集工具调用；审查、发布、删除、凭据管理始终要求管理员。
pub fn editor_routes() -> Router<AppState> {
    Router::new()
        .route("/drafts", get(list))
        .route("/drafts/import", post(import))
        .route("/drafts/{kind}/{key}", get(detail).put(save))
        .route("/preview/{kind}", put(preview))
}

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: Uuid,
    aud: String,
    exp: i64,
}

pub struct Editor {
    kinds: Vec<String>,
    operator: String,
    admin: bool,
}

impl Editor {
    fn check(&self, kind: &str) -> AppResult<()> {
        if !discover::is_kind(kind) {
            return Err(AppError::BadRequest("未知的类型".into()));
        }
        if !self.kinds.iter().any(|k| k == kind) {
            return Err(AppError::Forbidden("采集令牌没有此类型的权限".into()));
        }
        Ok(())
    }
}

impl FromRequestParts<AppState> for Editor {
    type Rejection = AppError;
    async fn from_request_parts(parts: &mut Parts, s: &AppState) -> AppResult<Self> {
        let raw = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or_else(|| AppError::Unauthorized("缺少编辑凭据".into()))?;
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_audience(&["forum-discover-editor"]);
        validation.leeway = 0;
        if let Ok(token) = decode::<Claims>(
            raw,
            &DecodingKey::from_secret(&s.config.admin_jwt_secret),
            &validation,
        ) {
            let kinds: Option<(Vec<String>,)> = sqlx::query_as("SELECT kinds FROM discover_tokens WHERE id=$1 AND revoked_at IS NULL AND expires_at>now()")
                .bind(token.claims.sub).fetch_optional(&s.pool).await?;
            let kinds = kinds
                .ok_or_else(|| AppError::Unauthorized("采集令牌已过期或被撤销".into()))?
                .0;
            return Ok(Self {
                kinds,
                operator: format!("discover-editor:{}", token.claims.sub),
                admin: false,
            });
        }
        let AuthAdmin(admin) = AuthAdmin::from_request_parts(parts, s).await?;
        Ok(Self {
            kinds: discover::KINDS.iter().map(|k| k.to_string()).collect(),
            operator: admin.username,
            admin: true,
        })
    }
}

async fn log(
    conn: &mut PgConnection,
    operator: &str,
    action: &str,
    kind: &str,
    key: &str,
    detail: &str,
) -> AppResult<()> {
    sqlx::query("INSERT INTO admin_audit(admin,action,target_type,target_id,detail) VALUES($1,$2,'discover',$3,$4)")
        .bind(operator).bind(action).bind(format!("{kind}/{key}")).bind(detail).execute(conn).await?;
    Ok(())
}

/// 所有发现页写入（含旧管理接口）共用此锁，以确定的顺序处理并发与批量导入。
pub async fn lock(conn: &mut PgConnection, kind: &str, key: &str) -> AppResult<()> {
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
        .bind(format!("discover:{kind}:{key}"))
        .execute(conn)
        .await?;
    Ok(())
}

/// 旧接口直接改线上内容后，撤销此前的定时审核，避免旧副本覆盖新内容。
pub async fn invalidate(conn: &mut PgConnection, kind: &str, key: &str) -> AppResult<()> {
    sqlx::query("UPDATE discover_drafts SET status='draft',publish_after=NULL,reviewed_by=NULL,reviewed_at=NULL,
        revision=nextval('discover_draft_revision_seq'),last_error='线上内容已改变，请重新保存并审核',updated_at=clock_timestamp()
        WHERE kind=$1 AND key=$2 AND status='scheduled'")
        .bind(kind).bind(key).execute(conn).await?;
    Ok(())
}

#[derive(sqlx::FromRow, Serialize)]
struct Draft {
    kind: String,
    key: String,
    sort_order: i32,
    data: Value,
    published_at: Option<DateTime<Utc>>,
    source_name: String,
    source_url: String,
    revision: i64,
    base_updated_at: Option<DateTime<Utc>>,
    status: String,
    publish_after: Option<DateTime<Utc>>,
    reviewed_by: Option<String>,
    reviewed_at: Option<DateTime<Utc>>,
    last_error: Option<String>,
    updated_by: String,
    updated_at: DateTime<Utc>,
}
const COLS: &str = "kind,key,sort_order,data,published_at,source_name,source_url,revision,base_updated_at,status,publish_after,reviewed_by,reviewed_at,last_error,updated_by,updated_at";

#[derive(Deserialize)]
struct Filter {
    kind: Option<String>,
    status: Option<String>,
    page: Option<i64>,
}
async fn list(
    State(s): State<AppState>,
    e: Editor,
    Query(f): Query<Filter>,
) -> AppResult<Json<Value>> {
    if let Some(k) = &f.kind {
        e.check(k)?;
    }
    if f.status
        .as_deref()
        .is_some_and(|s| !["draft", "scheduled", "published"].contains(&s))
    {
        return Err(AppError::BadRequest("未知的草稿状态".into()));
    }
    let page = f.page.unwrap_or(1).clamp(1, 100_000);
    let rows: Vec<Draft> = sqlx::query_as(&format!(
        "SELECT {COLS} FROM discover_drafts WHERE kind=ANY($1) AND ($2::text IS NULL OR kind=$2)
        AND ($3::text IS NULL OR status=$3) ORDER BY updated_at DESC,kind,key LIMIT 50 OFFSET $4"
    ))
    .bind(&e.kinds)
    .bind(&f.kind)
    .bind(&f.status)
    .bind((page - 1) * 50)
    .fetch_all(&s.pool)
    .await?;
    let (total,): (i64,) = sqlx::query_as("SELECT count(*) FROM discover_drafts WHERE kind=ANY($1) AND ($2::text IS NULL OR kind=$2) AND ($3::text IS NULL OR status=$3)")
        .bind(&e.kinds).bind(&f.kind).bind(&f.status).fetch_one(&s.pool).await?;
    Ok(Json(
        json!({"items":rows,"total":total,"page":page,"page_size":50}),
    ))
}
async fn read(conn: &mut PgConnection, kind: &str, key: &str) -> AppResult<Draft> {
    sqlx::query_as(&format!(
        "SELECT {COLS} FROM discover_drafts WHERE kind=$1 AND key=$2"
    ))
    .bind(kind)
    .bind(key)
    .fetch_optional(conn)
    .await?
    .ok_or_else(|| AppError::not_found("草稿"))
}
async fn detail(
    State(s): State<AppState>,
    e: Editor,
    Path((kind, key)): Path<(String, String)>,
) -> AppResult<Json<Value>> {
    e.check(&kind)?;
    let mut conn = s.pool.acquire().await?;
    let d = read(&mut conn, &kind, &key).await?;
    let live: Option<(Value, DateTime<Utc>)> =
        sqlx::query_as("SELECT data,updated_at FROM discover_items WHERE kind=$1 AND key=$2")
            .bind(&kind)
            .bind(&key)
            .fetch_optional(&mut *conn)
            .await?;
    Ok(Json(
        json!({"draft":d,"live":live.map(|(data,updated_at)|json!({"data":data,"updated_at":updated_at}))}),
    ))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    data: Value,
    #[serde(default)]
    sort_order: i32,
    published_at: Option<DateTime<Utc>>,
    #[serde(default)]
    sample: bool,
    #[serde(default)]
    source_name: String,
    #[serde(default)]
    source_url: String,
    /// 0 表示新建；更新必须带上读取到的版本，禁止无条件覆盖。
    expected_revision: i64,
}
fn validate(e: &Editor, kind: &str, key: &str, input: &mut Input) -> AppResult<()> {
    e.check(kind)?;
    if !discover::valid_key(key) {
        return Err(AppError::BadRequest(
            "key 只能用小写字母、数字、连字符和下划线".into(),
        ));
    }
    if kind == "model_board" && key != "artificial-analysis" {
        return Err(AppError::BadRequest(
            "能力榜标识必须为 artificial-analysis".into(),
        ));
    }
    if kind == "model_usage"
        && (input.data["date"].as_str() != Some(key)
            || chrono::NaiveDate::parse_from_str(key, "%Y-%m-%d")
                .map(|d| d.to_string() != key)
                .unwrap_or(true))
    {
        return Err(AppError::BadRequest(
            "调用量榜标识必须与统计日期一致".into(),
        ));
    }
    if input.expected_revision < 0 {
        return Err(AppError::BadRequest("版本号不能为负数".into()));
    }
    if !e.admin && input.sample {
        return Err(AppError::Forbidden("采集稿不能标记为示例".into()));
    }
    input.source_name = input.source_name.trim().to_string();
    input.source_url = input.source_url.trim().to_string();
    if input.source_name.chars().count() > 100 || input.source_url.len() > 2048 {
        return Err(AppError::BadRequest("来源名称或地址过长".into()));
    }
    if !input.source_url.is_empty() {
        let url = reqwest::Url::parse(&input.source_url)
            .map_err(|_| AppError::BadRequest("来源需为 HTTPS 地址".into()))?;
        if url.scheme() != "https"
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
        {
            return Err(AppError::BadRequest("来源需为不含凭据的 HTTPS 地址".into()));
        }
    }
    input.data = discover::validate(kind, std::mem::take(&mut input.data))?;
    input.data["sample"] = json!(input.sample);
    Ok(())
}
async fn write(
    conn: &mut PgConnection,
    e: &Editor,
    kind: &str,
    key: &str,
    i: Input,
) -> AppResult<Draft> {
    lock(conn, kind, key).await?;
    let old: Option<(i64,)> =
        sqlx::query_as("SELECT revision FROM discover_drafts WHERE kind=$1 AND key=$2")
            .bind(kind)
            .bind(key)
            .fetch_optional(&mut *conn)
            .await?;
    if old.map_or(0, |o| o.0) != i.expected_revision {
        return Err(AppError::Conflict(format!(
            "{kind}/{key} 已存在或已修改，请重新读取草稿"
        )));
    }
    let base: Option<(DateTime<Utc>,)> =
        sqlx::query_as("SELECT updated_at FROM discover_items WHERE kind=$1 AND key=$2")
            .bind(kind)
            .bind(key)
            .fetch_optional(&mut *conn)
            .await?;
    sqlx::query("INSERT INTO discover_drafts(kind,key,sort_order,data,published_at,source_name,source_url,base_updated_at,updated_by)
        VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9) ON CONFLICT(kind,key) DO UPDATE SET
        sort_order=$3,data=$4,published_at=$5,source_name=$6,source_url=$7,base_updated_at=$8,updated_by=$9,
        revision=nextval('discover_draft_revision_seq'),status='draft',publish_after=NULL,reviewed_by=NULL,reviewed_at=NULL,last_error=NULL,updated_at=clock_timestamp()")
        .bind(kind).bind(key).bind(i.sort_order).bind(i.data).bind(i.published_at).bind(i.source_name).bind(i.source_url)
        .bind(base.map(|b|b.0)).bind(&e.operator).execute(&mut *conn).await?;
    let d = read(conn, kind, key).await?;
    log(
        conn,
        &e.operator,
        "discover.draft.save",
        kind,
        key,
        &format!("revision={}", d.revision),
    )
    .await?;
    Ok(d)
}
async fn save(
    State(s): State<AppState>,
    e: Editor,
    Path((kind, key)): Path<(String, String)>,
    Json(mut i): Json<Input>,
) -> AppResult<Json<Draft>> {
    validate(&e, &kind, &key, &mut i)?;
    let mut tx = s.pool.begin().await?;
    let d = write(&mut tx, &e, &kind, &key, i).await?;
    tx.commit().await?;
    Ok(Json(d))
}
#[derive(Deserialize)]
struct ImportItem {
    kind: String,
    key: String,
    #[serde(flatten)]
    input: Input,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ImportInput {
    items: Vec<ImportItem>,
}
async fn import(
    State(s): State<AppState>,
    e: Editor,
    Json(mut batch): Json<ImportInput>,
) -> AppResult<Json<Value>> {
    if batch.items.is_empty() || batch.items.len() > 100 {
        return Err(AppError::BadRequest("每批导入 1–100 条草稿".into()));
    }
    let mut keys = HashSet::new();
    for item in &mut batch.items {
        validate(&e, &item.kind, &item.key, &mut item.input)?;
        if !keys.insert((item.kind.clone(), item.key.clone())) {
            return Err(AppError::BadRequest("同一批次的 kind/key 不能重复".into()));
        }
    }
    batch
        .items
        .sort_by(|a, b| (&a.kind, &a.key).cmp(&(&b.kind, &b.key)));
    let mut tx = s.pool.begin().await?;
    let mut saved = Vec::new();
    for item in batch.items {
        saved.push(write(&mut tx, &e, &item.kind, &item.key, item.input).await?);
    }
    tx.commit().await?;
    Ok(Json(json!({"items":saved})))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PreviewInput {
    data: Value,
    #[serde(default)]
    sample: bool,
}
async fn preview(
    e: Editor,
    Path(kind): Path<String>,
    Json(i): Json<PreviewInput>,
) -> AppResult<Json<Value>> {
    let mut input = Input {
        data: i.data,
        sample: i.sample,
        sort_order: 0,
        published_at: None,
        source_name: String::new(),
        source_url: String::new(),
        expected_revision: 0,
    };
    let preview_key = match kind.as_str() {
        "model_board" => "artificial-analysis".to_string(),
        "model_usage" => input.data["date"].as_str().unwrap_or_default().to_string(),
        _ => "preview".to_string(),
    };
    validate(&e, &kind, &preview_key, &mut input)?;
    if kind == "model" {
        input.data["tier"] = json!(discover::tier_for(input.data["iq"].as_f64().unwrap_or(0.0)));
    }
    Ok(Json(json!({"kind":kind,"data":input.data})))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReviewInput {
    expected_revision: i64,
    publish_after: Option<DateTime<Utc>>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RevisionInput {
    expected_revision: i64,
}
fn check_revision(d: &Draft, revision: i64) -> AppResult<()> {
    if d.revision != revision {
        return Err(AppError::Conflict("草稿已被修改，请重新预览并审核".into()));
    }
    Ok(())
}
async fn check_base(conn: &mut PgConnection, d: &Draft) -> AppResult<()> {
    let live: Option<(DateTime<Utc>,)> =
        sqlx::query_as("SELECT updated_at FROM discover_items WHERE kind=$1 AND key=$2")
            .bind(&d.kind)
            .bind(&d.key)
            .fetch_optional(conn)
            .await?;
    if live.map(|r| r.0) != d.base_updated_at {
        return Err(AppError::Conflict(
            "线上内容已改变，请重新保存并审核".into(),
        ));
    }
    Ok(())
}
async fn publish(conn: &mut PgConnection, d: &Draft, operator: &str) -> AppResult<()> {
    check_base(conn, d).await?;
    let mut data = discover::validate(&d.kind, d.data.clone())?;
    data["sample"] = json!(d.data["sample"].as_bool().unwrap_or(false));
    let (updated,): (DateTime<Utc>,) = sqlx::query_as("INSERT INTO discover_items(kind,key,sort_order,status,published_at,data)
        VALUES($1,$2,$3,'published',COALESCE($4,now()),$5) ON CONFLICT(kind,key) DO UPDATE SET
        sort_order=$3,status='published',published_at=COALESCE($4,discover_items.published_at),data=$5,updated_at=clock_timestamp() RETURNING updated_at")
        .bind(&d.kind).bind(&d.key).bind(d.sort_order).bind(d.published_at).bind(data).fetch_one(&mut *conn).await?;
    sqlx::query("UPDATE discover_drafts SET status='published',publish_after=NULL,base_updated_at=$3,last_error=NULL,updated_at=clock_timestamp() WHERE kind=$1 AND key=$2")
        .bind(&d.kind).bind(&d.key).bind(updated).execute(&mut *conn).await?;
    log(
        conn,
        operator,
        "discover.publish",
        &d.kind,
        &d.key,
        &format!(
            "revision={}; reviewed_by={}",
            d.revision,
            d.reviewed_by.as_deref().unwrap_or(operator)
        ),
    )
    .await?;
    Ok(())
}
async fn review(
    State(s): State<AppState>,
    AuthAdmin(a): AuthAdmin,
    Path((kind, key)): Path<(String, String)>,
    Json(i): Json<ReviewInput>,
) -> AppResult<Json<Value>> {
    let mut tx = s.pool.begin().await?;
    lock(&mut tx, &kind, &key).await?;
    let mut d = read(&mut tx, &kind, &key).await?;
    check_revision(&d, i.expected_revision)?;
    if d.status != "draft" {
        return Err(AppError::Conflict("只能审核待发布草稿".into()));
    }
    if i.publish_after.is_some_and(|t| t <= Utc::now()) {
        return Err(AppError::BadRequest("定时发布时间必须在未来".into()));
    }
    check_base(&mut tx, &d).await?;
    let after = i.publish_after;
    sqlx::query("UPDATE discover_drafts SET reviewed_by=$3,reviewed_at=now(),status=$4,publish_after=$5,revision=nextval('discover_draft_revision_seq'),updated_at=clock_timestamp() WHERE kind=$1 AND key=$2")
        .bind(&kind).bind(&key).bind(&a.username).bind(if after.is_some(){"scheduled"}else{"published"}).bind(after).execute(&mut *tx).await?;
    d = read(&mut tx, &kind, &key).await?;
    log(
        &mut tx,
        &a.username,
        "discover.review",
        &kind,
        &key,
        &format!("revision={}; publish_after={after:?}", d.revision),
    )
    .await?;
    if after.is_none() {
        publish(&mut tx, &d, &a.username).await?;
    }
    tx.commit().await?;
    Ok(Json(
        json!({"status":if after.is_some(){"scheduled"}else{"published"}}),
    ))
}
async fn cancel(
    State(s): State<AppState>,
    AuthAdmin(a): AuthAdmin,
    Path((kind, key)): Path<(String, String)>,
    Json(i): Json<RevisionInput>,
) -> AppResult<Json<Value>> {
    let mut tx = s.pool.begin().await?;
    lock(&mut tx, &kind, &key).await?;
    let d = read(&mut tx, &kind, &key).await?;
    check_revision(&d, i.expected_revision)?;
    if d.status != "scheduled" {
        return Err(AppError::Conflict("此草稿没有定时发布计划".into()));
    }
    invalidate(&mut tx, &kind, &key).await?;
    sqlx::query("UPDATE discover_drafts SET last_error=NULL WHERE kind=$1 AND key=$2")
        .bind(&kind)
        .bind(&key)
        .execute(&mut *tx)
        .await?;
    log(
        &mut tx,
        &a.username,
        "discover.cancel",
        &kind,
        &key,
        &format!("revision={}", d.revision),
    )
    .await?;
    tx.commit().await?;
    Ok(Json(json!({"ok":true})))
}
async fn discard(
    State(s): State<AppState>,
    AuthAdmin(a): AuthAdmin,
    Path((kind, key)): Path<(String, String)>,
    Json(i): Json<RevisionInput>,
) -> AppResult<Json<Value>> {
    let mut tx = s.pool.begin().await?;
    lock(&mut tx, &kind, &key).await?;
    let d = read(&mut tx, &kind, &key).await?;
    check_revision(&d, i.expected_revision)?;
    sqlx::query("DELETE FROM discover_drafts WHERE kind=$1 AND key=$2")
        .bind(&kind)
        .bind(&key)
        .execute(&mut *tx)
        .await?;
    log(
        &mut tx,
        &a.username,
        "discover.draft.discard",
        &kind,
        &key,
        &format!("revision={}", d.revision),
    )
    .await?;
    tx.commit().await?;
    Ok(Json(json!({"ok":true})))
}

/// 只执行已由管理员审核的具体版本。每条单独事务；重试/多实例不重复发布。
pub async fn publish_due(pool: &PgPool) -> AppResult<u64> {
    let due: Vec<(String,String)> = sqlx::query_as("SELECT kind,key FROM discover_drafts WHERE status='scheduled' AND publish_after<=now() ORDER BY publish_after,kind,key LIMIT 100")
        .fetch_all(pool).await?;
    let mut n = 0;
    for (kind, key) in due {
        let mut tx = pool.begin().await?;
        lock(&mut tx, &kind, &key).await?;
        let exists: Option<(bool,)> = sqlx::query_as("SELECT status='scheduled' AND publish_after<=now() FROM discover_drafts WHERE kind=$1 AND key=$2")
            .bind(&kind).bind(&key).fetch_optional(&mut *tx).await?;
        if exists != Some((true,)) {
            continue;
        }
        let d = read(&mut tx, &kind, &key).await?;
        match publish(&mut tx, &d, "discover-scheduler").await {
            Ok(()) => n += 1,
            Err(AppError::Conflict(message)) | Err(AppError::BadRequest(message)) => {
                sqlx::query("UPDATE discover_drafts SET status='draft',publish_after=NULL,reviewed_by=NULL,reviewed_at=NULL,revision=nextval('discover_draft_revision_seq'),last_error=$3,updated_at=clock_timestamp() WHERE kind=$1 AND key=$2")
                    .bind(&kind).bind(&key).bind(&message).execute(&mut *tx).await?;
                log(
                    &mut tx,
                    "discover-scheduler",
                    "discover.publish.failed",
                    &kind,
                    &key,
                    &message,
                )
                .await?;
            }
            Err(e) => return Err(e),
        }
        tx.commit().await?;
    }
    Ok(n)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TokenInput {
    name: String,
    kinds: Vec<String>,
    days: i64,
}
async fn create_token(
    State(s): State<AppState>,
    AuthAdmin(a): AuthAdmin,
    Json(mut i): Json<TokenInput>,
) -> AppResult<Json<Value>> {
    i.name = i.name.trim().to_string();
    if i.name.is_empty()
        || i.name.chars().count() > 60
        || !(1..=365).contains(&i.days)
        || i.kinds.is_empty()
        || i.kinds.iter().any(|k| !discover::is_kind(k))
    {
        return Err(AppError::BadRequest(
            "名称需为 1–60 字，有效期 1–365 天，类型不能为空且必须有效".into(),
        ));
    }
    i.kinds.sort();
    i.kinds.dedup();
    let id = Uuid::new_v4();
    let expires_at = Utc::now() + Duration::days(i.days);
    let token = encode(
        &Header::new(Algorithm::HS256),
        &Claims {
            sub: id,
            aud: "forum-discover-editor".into(),
            exp: expires_at.timestamp(),
        },
        &EncodingKey::from_secret(&s.config.admin_jwt_secret),
    )
    .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut tx = s.pool.begin().await?;
    sqlx::query("INSERT INTO discover_tokens(id,name,kinds,expires_at) VALUES($1,$2,$3,$4)")
        .bind(id)
        .bind(&i.name)
        .bind(&i.kinds)
        .bind(expires_at)
        .execute(&mut *tx)
        .await?;
    audit(
        &mut tx,
        &a,
        "discover.token.create",
        "discover_token",
        id,
        i.kinds.join(","),
    )
    .await?;
    tx.commit().await?;
    Ok(Json(json!({"id":id,"token":token,"expires_at":expires_at})))
}
async fn tokens(State(s): State<AppState>, _: AuthAdmin) -> AppResult<Json<Value>> {
    let rows: Vec<(Uuid,String,Vec<String>,DateTime<Utc>,Option<DateTime<Utc>>)> = sqlx::query_as("SELECT id,name,kinds,expires_at,revoked_at FROM discover_tokens ORDER BY created_at DESC LIMIT 100").fetch_all(&s.pool).await?;
    Ok(Json(
        json!({"items":rows.into_iter().map(|(id,name,kinds,expires_at,revoked_at)|json!({"id":id,"name":name,"kinds":kinds,"expires_at":expires_at,"revoked_at":revoked_at})).collect::<Vec<_>>()}),
    ))
}
async fn revoke_token(
    State(s): State<AppState>,
    AuthAdmin(a): AuthAdmin,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    let mut tx = s.pool.begin().await?;
    let n =
        sqlx::query("UPDATE discover_tokens SET revoked_at=COALESCE(revoked_at,now()) WHERE id=$1")
            .bind(id)
            .execute(&mut *tx)
            .await?
            .rows_affected();
    if n == 0 {
        return Err(AppError::not_found("采集令牌"));
    }
    audit(
        &mut tx,
        &a,
        "discover.token.revoke",
        "discover_token",
        id,
        "",
    )
    .await?;
    tx.commit().await?;
    Ok(Json(json!({"ok":true})))
}

/// 采集/来源快照时刻不构成内容变化；保留第一次待审的 as_of，实质数据变化才出新版本。
fn collected_content(value: &Value) -> Value {
    let mut value = value.clone();
    if let Some(object) = value.as_object_mut() {
        object.remove("as_of");
    }
    value
}

/// 采集批次以相同草稿锁和序列写入；任何条目冲突使整批回滚。
pub async fn write_collected_batch(
    pool: &PgPool,
    source: &str,
    run_id: Uuid,
    mut items: Vec<crate::collectors::CollectedItem>,
) -> AppResult<()> {
    if items.is_empty() || items.len() > 1000 {
        return Err(AppError::BadRequest("采集批次应含 1–1000 项".into()));
    }
    let operator = format!("collector:{source}");
    let e = Editor {
        kinds: discover::KINDS.iter().map(|s| s.to_string()).collect(),
        operator: operator.clone(),
        admin: false,
    };
    items.sort_by(|a, b| (&a.kind, &a.key).cmp(&(&b.kind, &b.key)));
    if items
        .windows(2)
        .any(|w| w[0].kind == w[1].kind && w[0].key == w[1].key)
    {
        return Err(AppError::BadRequest("采集批次出现重复标识".into()));
    }
    let mut tx = pool.begin().await?;
    // 锁住租约至写入完成，过期 worker 不得写入接替者的任务。
    let active: Option<(Uuid,)> = sqlx::query_as("SELECT active_run FROM collection_sources WHERE id=$1 AND active_run=$2 AND lease_until>now() FOR UPDATE")
        .bind(source).bind(run_id).fetch_optional(&mut *tx).await?;
    if active.is_none() {
        return Err(AppError::Conflict("采集租约已失效".into()));
    }
    for item in items {
        let allowed = match source {
            "artificial_analysis" => {
                item.kind == "model_board" && item.key == "artificial-analysis"
            }
            "openrouter" => {
                item.kind == "model_usage"
                    && chrono::NaiveDate::parse_from_str(&item.key, "%Y-%m-%d").is_ok()
            }
            _ => false,
        };
        if !allowed {
            return Err(AppError::BadRequest("采集来源越过内容边界".into()));
        }
        lock(&mut tx, &item.kind, &item.key).await?;
        let old: Option<Draft> = sqlx::query_as(&format!(
            "SELECT {COLS} FROM discover_drafts WHERE kind=$1 AND key=$2"
        ))
        .bind(&item.kind)
        .bind(&item.key)
        .fetch_optional(&mut *tx)
        .await?;
        let owned: Option<(String, i64)> = sqlx::query_as(
            "SELECT source,revision FROM collection_owned_items WHERE kind=$1 AND key=$2",
        )
        .bind(&item.kind)
        .bind(&item.key)
        .fetch_optional(&mut *tx)
        .await?;
        let mut input = Input {
            data: item.data,
            sort_order: item.sort_order,
            published_at: item.published_at,
            source_name: item.source_name,
            source_url: item.source_url,
            sample: false,
            expected_revision: old.as_ref().map_or(0, |d| d.revision),
        };
        validate(&e, &item.kind, &item.key, &mut input)?;
        let same = old.as_ref().is_some_and(|d| {
            collected_content(&d.data) == collected_content(&input.data)
                && d.sort_order == input.sort_order
                && d.published_at == input.published_at
                && d.source_name == input.source_name
                && d.source_url == input.source_url
        });
        if let Some(d) = &old {
            check_base(&mut tx, d).await?;
            let owner_matches = owned.as_ref().is_some_and(|(owner, revision)| {
                owner == source && (*revision == d.revision || d.status == "published")
            });
            if !owner_matches || d.updated_by != operator || (!same && d.status == "scheduled") {
                return Err(AppError::Conflict("草稿已被人工修改或审核".into()));
            }
        } else {
            let live: Option<(bool,)> =
                sqlx::query_as("SELECT true FROM discover_items WHERE kind=$1 AND key=$2")
                    .bind(&item.kind)
                    .bind(&item.key)
                    .fetch_optional(&mut *tx)
                    .await?;
            if live.is_some() {
                return Err(AppError::Conflict(
                    "已有手录线上内容，不允许采集覆盖".into(),
                ));
            }
        }
        let revision = if same {
            old.as_ref().unwrap().revision
        } else {
            write(&mut tx, &e, &item.kind, &item.key, input)
                .await?
                .revision
        };
        sqlx::query("INSERT INTO collection_owned_items(source,kind,key,revision) VALUES($1,$2,$3,$4) ON CONFLICT(kind,key) DO UPDATE SET revision=$4")
            .bind(source).bind(&item.kind).bind(&item.key).bind(revision).execute(&mut *tx).await?;
        sqlx::query(
            "INSERT INTO collection_run_items(run_id,kind,key,revision) VALUES($1,$2,$3,$4)",
        )
        .bind(run_id)
        .bind(&item.kind)
        .bind(&item.key)
        .bind(revision)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BatchReviewItem {
    pub kind: String,
    pub key: String,
    pub expected_revision: i64,
}

/// 整批按预览过的 revision 立即发布；复用单条发布校验及审计。
pub async fn review_collected_batch(
    pool: &PgPool,
    run_id: Uuid,
    operator: &str,
    mut items: Vec<BatchReviewItem>,
) -> AppResult<usize> {
    let mut tx = pool.begin().await?;
    let run: Option<(String, Option<DateTime<Utc>>)> =
        sqlx::query_as("SELECT status,reviewed_at FROM collection_runs WHERE id=$1 FOR UPDATE")
            .bind(run_id)
            .fetch_optional(&mut *tx)
            .await?;
    let (status, reviewed) = run.ok_or_else(|| AppError::not_found("采集批次"))?;
    if status != "succeeded" || reviewed.is_some() {
        return Err(AppError::Conflict("此批次尚未成功或已审核".into()));
    }
    let manifest: Vec<(String, String, i64)> = sqlx::query_as(
        "SELECT kind,key,revision FROM collection_run_items WHERE run_id=$1 ORDER BY kind,key",
    )
    .bind(run_id)
    .fetch_all(&mut *tx)
    .await?;
    items.sort_by(|a, b| (&a.kind, &a.key).cmp(&(&b.kind, &b.key)));
    if manifest.is_empty()
        || manifest.len() != items.len()
        || !manifest
            .iter()
            .zip(items.iter())
            .all(|(a, b)| a.0 == b.kind && a.1 == b.key && a.2 == b.expected_revision)
    {
        return Err(AppError::Conflict("请重新读取并完整审核本批次草稿".into()));
    }
    for item in &items {
        lock(&mut tx, &item.kind, &item.key).await?;
    }
    for item in &items {
        let d = read(&mut tx, &item.kind, &item.key).await?;
        check_revision(&d, item.expected_revision)?;
        if d.status == "scheduled" {
            return Err(AppError::Conflict(
                "此批次有单独定时的草稿，请先取消定时".into(),
            ));
        }
        check_base(&mut tx, &d).await?;
    }
    for item in &items {
        let mut d = read(&mut tx, &item.kind, &item.key).await?;
        if d.status == "published" {
            continue;
        }
        sqlx::query("UPDATE discover_drafts SET reviewed_by=$3,reviewed_at=now(),status='published',publish_after=NULL,revision=nextval('discover_draft_revision_seq'),updated_at=clock_timestamp() WHERE kind=$1 AND key=$2")
            .bind(&item.kind).bind(&item.key).bind(operator).execute(&mut *tx).await?;
        d = read(&mut tx, &item.kind, &item.key).await?;
        log(
            &mut tx,
            operator,
            "discover.review",
            &item.kind,
            &item.key,
            &format!("batch={run_id}; revision={}", d.revision),
        )
        .await?;
        publish(&mut tx, &d, operator).await?;
    }
    sqlx::query("UPDATE collection_runs SET reviewed_by=$2,reviewed_at=now() WHERE id=$1")
        .bind(run_id)
        .bind(operator)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(items.len())
}

/// 一个一致性快照内返回批次版本、当前草稿与线上内容，供逐项预览。
pub async fn collected_batch_preview(pool: &PgPool, run_id: Uuid) -> AppResult<Value> {
    let mut tx = pool.begin().await?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY")
        .execute(&mut *tx)
        .await?;
    let run: Option<(String, String, Option<DateTime<Utc>>)> =
        sqlx::query_as("SELECT source,status,reviewed_at FROM collection_runs WHERE id=$1")
            .bind(run_id)
            .fetch_optional(&mut *tx)
            .await?;
    let (source, status, reviewed_at) = run.ok_or_else(|| AppError::not_found("采集批次"))?;
    let manifest: Vec<(String, String, i64)> = sqlx::query_as(
        "SELECT kind,key,revision FROM collection_run_items WHERE run_id=$1 ORDER BY kind,key",
    )
    .bind(run_id)
    .fetch_all(&mut *tx)
    .await?;
    let mut items = Vec::new();
    for (kind, key, revision) in manifest {
        let draft: Option<Draft> = sqlx::query_as(&format!(
            "SELECT {COLS} FROM discover_drafts WHERE kind=$1 AND key=$2"
        ))
        .bind(&kind)
        .bind(&key)
        .fetch_optional(&mut *tx)
        .await?;
        let live:Option<(Value,i32,DateTime<Utc>,DateTime<Utc>,String)>=sqlx::query_as("SELECT data,sort_order,published_at,updated_at,status FROM discover_items WHERE kind=$1 AND key=$2")
            .bind(&kind).bind(&key).fetch_optional(&mut *tx).await?;
        let conflict = draft.as_ref().is_none_or(|d| d.revision != revision);
        items.push(json!({"kind":kind,"key":key,"expected_revision":revision,"draft":draft,"conflict":conflict,
            "live":live.map(|(data,sort_order,published_at,updated_at,status)|json!({"data":data,"sort_order":sort_order,"published_at":published_at,"updated_at":updated_at,"status":status}))}));
    }
    tx.commit().await?;
    Ok(
        json!({"run_id":run_id,"source":source,"status":status,"reviewed_at":reviewed_at,"items":items}),
    )
}
