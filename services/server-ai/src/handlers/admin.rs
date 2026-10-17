use crate::{
    admin_auth::{self, AdminIdentity},
    error::AppError,
    extract::AuthAdmin,
    password, products, quota,
    quota::LIVE,
    redeem,
    state::AppState,
    sync,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::FromRow;
use uuid::Uuid;

const MAX_ADMIN_FAILURES: i32 = 5;
const BAD_ADMIN_CREDENTIALS: &str = "管理员用户名或密码错误";
/// 单次人工补发的上限，防止手滑多打几个零（约 2000 美元）。
const MAX_GRANT_QUOTA: i64 = 1_000_000_000;

async fn audit(state: &AppState, admin: &AdminIdentity, action: &str, target: Option<&str>, detail: Option<String>) -> Result<(), AppError> {
    sqlx::query("INSERT INTO admin_audit (admin_username, action, target, detail) VALUES ($1, $2, $3, $4)")
        .bind(&admin.username)
        .bind(action)
        .bind(target)
        .bind(detail)
        .execute(&state.pool)
        .await?;
    Ok(())
}

#[derive(Deserialize)]
pub struct LoginReq {
    username: String,
    password: String,
}

pub async fn login(State(state): State<AppState>, Json(req): Json<LoginReq>) -> Result<Json<Value>, AppError> {
    let username = req.username.trim().to_lowercase();
    let row: Option<(Uuid, String, i32, bool)> = sqlx::query_as(
        "SELECT id, password_hash, token_version, COALESCE(locked_until > now(), false) FROM admins WHERE username = $1",
    )
    .bind(&username)
    .fetch_optional(&state.pool)
    .await?;

    let Some((id, hash, token_version, locked)) = row else {
        password::verify(req.password, state.dummy_password_hash.to_string()).await;
        return Err(AppError::Unauthorized(BAD_ADMIN_CREDENTIALS.into()));
    };
    if locked {
        return Err(AppError::Coded {
            status: StatusCode::TOO_MANY_REQUESTS,
            code: "too_many_requests",
            message: "密码错误次数过多，请 15 分钟后再试".into(),
        });
    }
    if !password::verify(req.password, hash).await {
        sqlx::query(
            "UPDATE admins SET
                locked_until = CASE WHEN failed_logins + 1 >= $2 THEN now() + interval '15 minutes' ELSE locked_until END,
                failed_logins = CASE WHEN failed_logins + 1 >= $2 THEN 0 ELSE failed_logins + 1 END
             WHERE id = $1",
        )
        .bind(id)
        .bind(MAX_ADMIN_FAILURES)
        .execute(&state.pool)
        .await?;
        return Err(AppError::Unauthorized(BAD_ADMIN_CREDENTIALS.into()));
    }

    sqlx::query("UPDATE admins SET last_login_at = now(), failed_logins = 0, locked_until = NULL WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;
    let ttl = state.config.admin_token_ttl_secs;
    let token = admin_auth::issue(&state.keys.admin_jwt_secret, id, token_version, ttl)?;
    Ok(Json(json!({ "token": token, "expires_in": ttl, "admin": { "username": username } })))
}

pub async fn me(AuthAdmin(admin): AuthAdmin) -> Json<Value> {
    Json(json!({ "username": admin.username }))
}

#[derive(Deserialize)]
pub struct ChangePasswordReq {
    current_password: String,
    new_password: String,
}

pub async fn change_password(
    State(state): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Json(req): Json<ChangePasswordReq>,
) -> Result<Json<Value>, AppError> {
    password::validate(&req.new_password)?;
    let (hash,): (String,) = sqlx::query_as("SELECT password_hash FROM admins WHERE id = $1").bind(admin.id).fetch_one(&state.pool).await?;
    if !password::verify(req.current_password, hash).await {
        return Err(AppError::Unauthorized("当前密码错误".into()));
    }
    let new_hash = password::hash(req.new_password).await?;
    sqlx::query("UPDATE admins SET password_hash = $2, token_version = token_version + 1 WHERE id = $1")
        .bind(admin.id)
        .bind(new_hash)
        .execute(&state.pool)
        .await?;
    audit(&state, &admin, "admin.password_change", None, None).await?;
    Ok(Json(json!({ "ok": true })))
}

pub async fn overview(State(state): State<AppState>, _admin: AuthAdmin) -> Result<Json<Value>, AppError> {
    let (bindings, active, funded, balance): (i64, i64, i64, i64) = sqlx::query_as(&format!(
        "SELECT COUNT(*), COUNT(*) FILTER (WHERE status = 'active'),
                COUNT(*) FILTER (WHERE bal > 0), COALESCE(SUM(bal), 0)::BIGINT
         FROM (SELECT b.status, COALESCE((SELECT SUM(remaining) FROM quota_buckets WHERE binding_id = b.id AND {LIVE}), 0) AS bal FROM bindings b) t"
    ))
    .fetch_one(&state.pool)
    .await?;
    let (consumed_24h, granted_24h): (i64, i64) = sqlx::query_as(
        "SELECT COALESCE(-SUM(delta) FILTER (WHERE kind IN ('consume', 'consume_overdraft')), 0)::BIGINT,
                COALESCE(SUM(delta) FILTER (WHERE kind = 'grant'), 0)::BIGINT
         FROM quota_ledger WHERE created_at > now() - interval '1 day'",
    )
    .fetch_one(&state.pool)
    .await?;
    let (anomalies,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sync_anomalies WHERE resolved_at IS NULL").fetch_one(&state.pool).await?;
    let per_product: Vec<(String, String, i64)> = sqlx::query_as(
        "SELECT p.product_id, p.name, COUNT(b.id) FROM products p LEFT JOIN bindings b ON b.product_id = p.product_id
         GROUP BY p.product_id, p.name, p.created_at ORDER BY p.created_at, p.product_id",
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(json!({
        "bindings": bindings,
        "active_bindings": active,
        "funded_bindings": funded,
        "balance": balance,
        "consumed_24h": consumed_24h,
        "granted_24h": granted_24h,
        "open_anomalies": anomalies,
        "quota_per_unit": state.config.quota_per_unit,
        "newapi_configured": state.newapi.is_configured(),
        "products": per_product.into_iter().map(|(id, name, count)| json!({ "product_id": id, "name": name, "bindings": count })).collect::<Vec<_>>(),
    })))
}

// ---- 产品 ----

pub async fn list_products(State(state): State<AppState>, _admin: AuthAdmin) -> Result<Json<Value>, AppError> {
    Ok(Json(json!({ "items": products::list(&state.pool).await? })))
}

#[derive(Deserialize)]
pub struct ProductReq {
    name: String,
    client_ids: Vec<String>,
    trial_quota: i64,
    trial_days: i32,
    enabled: bool,
}

pub async fn upsert_product(
    State(state): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Path(product_id): Path<String>,
    Json(req): Json<ProductReq>,
) -> Result<Json<Value>, AppError> {
    if !products::valid_slug(&product_id) {
        return Err(AppError::BadRequest("产品标识需为小写字母开头，由小写字母、数字、连字符或下划线组成，最长 40 位".into()));
    }
    let name = req.name.trim().to_string();
    if name.is_empty() || name.chars().count() > 40 {
        return Err(AppError::BadRequest("产品名称需为 1-40 个字符".into()));
    }
    if !(0..=MAX_GRANT_QUOTA).contains(&req.trial_quota) {
        return Err(AppError::BadRequest("试用额度超出范围".into()));
    }
    if !(1..=365).contains(&req.trial_days) {
        return Err(AppError::BadRequest("试用有效期需为 1-365 天".into()));
    }
    let mut client_ids: Vec<String> = req.client_ids.iter().map(|c| c.trim().to_string()).filter(|c| !c.is_empty()).collect();
    client_ids.sort();
    client_ids.dedup();
    if let Some((other, client)) = sqlx::query_as::<_, (String, String)>(
        "SELECT product_id, c FROM products, unnest(client_ids) AS c WHERE product_id <> $1 AND c = ANY($2) LIMIT 1",
    )
    .bind(&product_id)
    .bind(&client_ids)
    .fetch_optional(&state.pool)
    .await?
    {
        return Err(AppError::Conflict(format!("客户端 {client} 已经属于产品 {other}，一个客户端只能属于一个产品")));
    }

    sqlx::query(
        "INSERT INTO products (product_id, name, client_ids, trial_quota, trial_days, enabled) VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (product_id) DO UPDATE SET name = $2, client_ids = $3, trial_quota = $4, trial_days = $5, enabled = $6, updated_at = now()",
    )
    .bind(&product_id)
    .bind(&name)
    .bind(&client_ids)
    .bind(req.trial_quota)
    .bind(req.trial_days)
    .bind(req.enabled)
    .execute(&state.pool)
    .await?;
    audit(
        &state,
        &admin,
        "product.save",
        Some(&product_id),
        Some(format!("clients={client_ids:?} trial={} days={} enabled={}", req.trial_quota, req.trial_days, req.enabled)),
    )
    .await?;
    let saved = products::get(&state.pool, &product_id).await?;
    Ok(Json(json!({ "product": saved })))
}

// ---- 档位 ----

pub async fn list_tiers(State(state): State<AppState>, _admin: AuthAdmin, Path(product_id): Path<String>) -> Result<Json<Value>, AppError> {
    Ok(Json(json!({ "items": products::tiers(&state.pool, &product_id, false).await? })))
}

#[derive(Deserialize)]
pub struct TierReq {
    display_name: String,
    #[serde(default)]
    description: String,
    model: String,
    requires_paid: bool,
    is_default: bool,
    sort_order: i32,
    enabled: bool,
}

pub async fn upsert_tier(
    State(state): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Path((product_id, tier)): Path<(String, String)>,
    Json(req): Json<TierReq>,
) -> Result<Json<Value>, AppError> {
    if !products::valid_slug(&tier) {
        return Err(AppError::BadRequest("档位标识需为小写字母开头，由小写字母、数字、连字符或下划线组成，最长 40 位".into()));
    }
    let display_name = req.display_name.trim().to_string();
    if display_name.is_empty() || display_name.chars().count() > 20 {
        return Err(AppError::BadRequest("展示名需为 1-20 个字符".into()));
    }
    let description = req.description.trim().to_string();
    if description.chars().count() > 100 {
        return Err(AppError::BadRequest("简介最多 100 个字符".into()));
    }
    if req.is_default && (req.requires_paid || !req.enabled) {
        return Err(AppError::BadRequest("默认档位必须启用，且不能要求付费，否则新用户无法使用".into()));
    }
    if products::get(&state.pool, &product_id).await?.is_none() {
        return Err(AppError::NotFound("产品不存在".into()));
    }

    let model = req.model.trim().to_string();
    let available = state.newapi.list_models().await.map_err(|e| {
        tracing::warn!("保存档位时读取模型列表失败: {e}");
        AppError::BadGateway("读不到 AI 网关的模型列表，暂时无法保存".into())
    })?;
    let Some(pricing) = available.iter().find(|m| m.model_name == model) else {
        return Err(AppError::BadRequest(format!("AI 网关里没有已启用的模型 {model}，请先在 New API 配置渠道")));
    };
    let mut warnings = Vec::new();
    if pricing.model_price == 0.0 && (pricing.model_ratio - state.config.newapi_default_ratio).abs() < f64::EPSILON {
        warnings.push(format!(
            "模型 {model} 的倍率与 New API 未配置时的默认值 {} 相同，很可能还没设置真实定价，请先核对再上线",
            state.config.newapi_default_ratio
        ));
    }

    let mut tx = state.pool.begin().await?;
    if req.is_default {
        sqlx::query("UPDATE product_tiers SET is_default = false WHERE product_id = $1 AND tier <> $2")
            .bind(&product_id)
            .bind(&tier)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query(
        "INSERT INTO product_tiers (product_id, tier, display_name, description, model, requires_paid, is_default, sort_order, enabled)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
         ON CONFLICT (product_id, tier) DO UPDATE SET display_name = $3, description = $4, model = $5,
             requires_paid = $6, is_default = $7, sort_order = $8, enabled = $9, updated_at = now()",
    )
    .bind(&product_id)
    .bind(&tier)
    .bind(&display_name)
    .bind(&description)
    .bind(&model)
    .bind(req.requires_paid)
    .bind(req.is_default)
    .bind(req.sort_order)
    .bind(req.enabled)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    audit(
        &state,
        &admin,
        "tier.save",
        Some(&format!("{product_id}/{tier}")),
        Some(format!("model={model} paid={} default={} enabled={}", req.requires_paid, req.is_default, req.enabled)),
    )
    .await?;
    Ok(Json(json!({ "ok": true, "warnings": warnings })))
}

pub async fn delete_tier(
    State(state): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Path((product_id, tier)): Path<(String, String)>,
) -> Result<Json<Value>, AppError> {
    let result = sqlx::query("DELETE FROM product_tiers WHERE product_id = $1 AND tier = $2")
        .bind(&product_id)
        .bind(&tier)
        .execute(&state.pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("档位不存在".into()));
    }
    audit(&state, &admin, "tier.delete", Some(&format!("{product_id}/{tier}")), None).await?;
    Ok(Json(json!({ "ok": true })))
}

/// 供下拉选择的模型列表，附带「倍率像是没配置」的标记。
pub async fn models(State(state): State<AppState>, _admin: AuthAdmin) -> Result<Json<Value>, AppError> {
    let mut items = state.newapi.list_models().await?;
    items.sort_by(|a, b| a.model_name.cmp(&b.model_name));
    let default_ratio = state.config.newapi_default_ratio;
    let items: Vec<Value> = items
        .into_iter()
        .map(|m| {
            let unconfigured = m.model_price == 0.0 && (m.model_ratio - default_ratio).abs() < f64::EPSILON;
            json!({
                "model_name": m.model_name,
                "model_ratio": m.model_ratio,
                "completion_ratio": m.completion_ratio,
                "ratio_warning": unconfigured,
            })
        })
        .collect();
    Ok(Json(json!({ "items": items, "default_ratio": default_ratio })))
}

// ---- 用户余额 ----

#[derive(Deserialize)]
pub struct BindingQuery {
    q: Option<String>,
    product_id: Option<String>,
    page: Option<i64>,
    page_size: Option<i64>,
}

#[derive(Serialize, FromRow)]
struct BindingRow {
    id: i64,
    uid: Uuid,
    product_id: String,
    status: String,
    balance: i64,
    synced_quota: i64,
    created_at: DateTime<Utc>,
    last_used_at: Option<DateTime<Utc>>,
    last_synced_at: Option<DateTime<Utc>>,
}

pub async fn list_bindings(State(state): State<AppState>, _admin: AuthAdmin, Query(query): Query<BindingQuery>) -> Result<Json<Value>, AppError> {
    let page_size = query.page_size.unwrap_or(20).clamp(1, 100);
    let page = query.page.unwrap_or(1).max(1);
    let keyword = query.q.as_deref().map(str::trim).filter(|s| !s.is_empty()).map(str::to_string);
    let product = query.product_id.as_deref().filter(|s| !s.is_empty()).map(str::to_string);
    let filter = "($1::TEXT IS NULL OR b.uid::TEXT = $1 OR b.id::TEXT = $1) AND ($2::TEXT IS NULL OR b.product_id = $2)";

    let rows: Vec<BindingRow> = sqlx::query_as(&format!(
        "SELECT b.id, b.uid, b.product_id, b.status, b.synced_quota, b.created_at, b.last_used_at, b.last_synced_at,
                COALESCE((SELECT SUM(remaining) FROM quota_buckets WHERE binding_id = b.id AND {LIVE}), 0)::BIGINT AS balance
         FROM bindings b WHERE {filter} ORDER BY b.last_used_at DESC NULLS LAST, b.id DESC LIMIT $3 OFFSET $4"
    ))
    .bind(&keyword)
    .bind(&product)
    .bind(page_size)
    .bind((page - 1) * page_size)
    .fetch_all(&state.pool)
    .await?;
    let (total,): (i64,) = sqlx::query_as(&format!("SELECT COUNT(*) FROM bindings b WHERE {filter}"))
        .bind(&keyword)
        .bind(&product)
        .fetch_one(&state.pool)
        .await?;
    Ok(Json(json!({ "items": rows, "total": total, "page": page, "page_size": page_size })))
}

pub async fn get_binding(State(state): State<AppState>, _admin: AuthAdmin, Path(id): Path<i64>) -> Result<Json<Value>, AppError> {
    let row: Option<BindingRow> = sqlx::query_as(&format!(
        "SELECT b.id, b.uid, b.product_id, b.status, b.synced_quota, b.created_at, b.last_used_at, b.last_synced_at,
                COALESCE((SELECT SUM(remaining) FROM quota_buckets WHERE binding_id = b.id AND {LIVE}), 0)::BIGINT AS balance
         FROM bindings b WHERE b.id = $1"
    ))
    .bind(id)
    .fetch_optional(&state.pool)
    .await?;
    let binding = row.ok_or_else(|| AppError::NotFound("没有这个用户绑定".into()))?;
    let buckets = quota::buckets(&state.pool, id, false).await?;
    let ledger = quota::ledger(&state.pool, id, None, 50).await?;
    let anomalies: Vec<AnomalyRow> = sqlx::query_as(
        "SELECT id, binding_id, kind, detail, created_at, resolved_at FROM sync_anomalies WHERE binding_id = $1 ORDER BY id DESC LIMIT 20",
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(json!({ "binding": binding, "buckets": buckets, "ledger": ledger, "anomalies": anomalies })))
}

#[derive(Deserialize)]
pub struct GrantReq {
    quota: i64,
    valid_days: Option<i64>,
    kind: Option<String>,
    #[serde(default)]
    note: String,
}

pub async fn grant(
    State(state): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Path(id): Path<i64>,
    Json(req): Json<GrantReq>,
) -> Result<Json<Value>, AppError> {
    if !(1..=MAX_GRANT_QUOTA).contains(&req.quota) {
        return Err(AppError::BadRequest("补发额度需大于 0，且不超过单次上限".into()));
    }
    let kind = req.kind.as_deref().unwrap_or("topup");
    if !matches!(kind, "topup" | "monthly") {
        return Err(AppError::BadRequest("额度类型只能是 topup 或 monthly".into()));
    }
    let days = req.valid_days.unwrap_or(if kind == "monthly" { 30 } else { 365 });
    if !(1..=3650).contains(&days) {
        return Err(AppError::BadRequest("有效期需为 1-3650 天".into()));
    }
    let note = req.note.trim().chars().take(100).collect::<String>();

    let mut tx = state.pool.begin().await?;
    let active: Option<(String,)> = sqlx::query_as("SELECT status FROM bindings WHERE id = $1 FOR UPDATE").bind(id).fetch_optional(&mut *tx).await?;
    match active {
        None => return Err(AppError::NotFound("没有这个用户绑定".into())),
        Some((status,)) if status != "active" => return Err(AppError::Conflict("该绑定尚未开通，无法补发".into())),
        Some(_) => {}
    }
    let source = format!("admin:{}", admin.username);
    let note_text = if note.is_empty() { "管理员补发".to_string() } else { format!("管理员补发：{note}") };
    let bucket_id = quota::grant(&mut tx, id, kind, req.quota, Some(Utc::now() + Duration::days(days)), &source, &note_text).await?;
    tx.commit().await?;

    if let Err(e) = sync::sync_binding(&state, id).await {
        tracing::warn!("binding {id} 补发后对账失败，稍后由后台任务重试: {e}");
    }
    audit(&state, &admin, "binding.grant", Some(&id.to_string()), Some(format!("kind={kind} quota={} days={days} note={note}", req.quota))).await?;
    Ok(Json(json!({ "ok": true, "bucket_id": bucket_id })))
}

#[derive(Deserialize)]
pub struct VoidReq {
    #[serde(default)]
    note: String,
}

pub async fn void_bucket(
    State(state): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Path(bucket_id): Path<i64>,
    Json(req): Json<VoidReq>,
) -> Result<Json<Value>, AppError> {
    let note = req.note.trim().chars().take(100).collect::<String>();
    let note_text = if note.is_empty() { "管理员扣回".to_string() } else { format!("管理员扣回：{note}") };
    let mut tx = state.pool.begin().await?;
    let voided = quota::void_bucket(&mut tx, bucket_id, "admin_void", &note_text).await?;
    let Some((binding_id, remaining)) = voided else {
        return Err(AppError::NotFound("额度桶不存在或已经关闭".into()));
    };
    tx.commit().await?;

    if let Err(e) = sync::sync_binding(&state, binding_id).await {
        tracing::warn!("binding {binding_id} 扣回后对账失败，稍后由后台任务重试: {e}");
    }
    audit(&state, &admin, "bucket.void", Some(&bucket_id.to_string()), Some(format!("binding={binding_id} remaining={remaining} note={note}"))).await?;
    Ok(Json(json!({ "ok": true, "voided": remaining })))
}

// ---- 异常与审计 ----

#[derive(Serialize, FromRow)]
struct AnomalyRow {
    id: i64,
    binding_id: i64,
    kind: String,
    detail: String,
    created_at: DateTime<Utc>,
    resolved_at: Option<DateTime<Utc>>,
}

pub async fn anomalies(State(state): State<AppState>, _admin: AuthAdmin) -> Result<Json<Value>, AppError> {
    let items: Vec<AnomalyRow> = sqlx::query_as(
        "SELECT id, binding_id, kind, detail, created_at, resolved_at FROM sync_anomalies
         ORDER BY (resolved_at IS NULL) DESC, id DESC LIMIT 100",
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(json!({ "items": items })))
}

pub async fn resolve_anomaly(State(state): State<AppState>, AuthAdmin(admin): AuthAdmin, Path(id): Path<i64>) -> Result<Json<Value>, AppError> {
    let result = sqlx::query("UPDATE sync_anomalies SET resolved_at = now() WHERE id = $1 AND resolved_at IS NULL")
        .bind(id)
        .execute(&state.pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("异常不存在或已处理".into()));
    }
    audit(&state, &admin, "anomaly.resolve", Some(&id.to_string()), None).await?;
    Ok(Json(json!({ "ok": true })))
}

// ---- 兑换码 ----

pub async fn list_batches(State(state): State<AppState>, _admin: AuthAdmin) -> Result<Json<Value>, AppError> {
    Ok(Json(json!({ "items": redeem::list_batches(&state).await? })))
}

#[derive(Deserialize)]
pub struct BatchReq {
    product_id: String,
    kind: String,
    quota: i64,
    valid_days: i32,
    count: i64,
    #[serde(default)]
    note: String,
}

/// 明文兑换码只在这个响应里出现一次，数据库里只有哈希；关掉页面后无法再找回。
pub async fn create_batch(State(state): State<AppState>, AuthAdmin(admin): AuthAdmin, Json(req): Json<BatchReq>) -> Result<Json<Value>, AppError> {
    if !matches!(req.kind.as_str(), "topup" | "monthly") {
        return Err(AppError::BadRequest("兑换码类型只能是加油包或月卡".into()));
    }
    if !(1..=MAX_GRANT_QUOTA).contains(&req.quota) {
        return Err(AppError::BadRequest("面额需大于 0，且不超过单次上限".into()));
    }
    if !(1..=3650).contains(&req.valid_days) {
        return Err(AppError::BadRequest("有效期需为 1-3650 天".into()));
    }
    if !(1..=redeem::MAX_BATCH_SIZE).contains(&req.count) {
        return Err(AppError::BadRequest(format!("一批最少 1 个、最多 {} 个", redeem::MAX_BATCH_SIZE)));
    }
    let note = req.note.trim().to_string();
    if note.chars().count() > 100 {
        return Err(AppError::BadRequest("备注最多 100 个字符".into()));
    }
    if products::get(&state.pool, &req.product_id).await?.is_none() {
        return Err(AppError::NotFound("产品不存在".into()));
    }

    let (batch, codes) = redeem::create_batch(
        &state,
        &admin.username,
        redeem::NewBatch { product_id: req.product_id, kind: req.kind, quota: req.quota, valid_days: req.valid_days, count: req.count, note },
    )
    .await?;
    audit(
        &state,
        &admin,
        "batch.create",
        Some(&batch.id.to_string()),
        Some(format!("product={} kind={} quota={} days={} count={} note={}", batch.product_id, batch.kind, batch.quota, batch.valid_days, batch.total, batch.note)),
    )
    .await?;
    Ok(Json(json!({ "batch": batch, "codes": codes })))
}

pub async fn void_batch(State(state): State<AppState>, AuthAdmin(admin): AuthAdmin, Path(id): Path<i64>) -> Result<Json<Value>, AppError> {
    let voided = redeem::void_batch(&state, id).await?;
    audit(&state, &admin, "batch.void", Some(&id.to_string()), Some(format!("voided={voided}"))).await?;
    Ok(Json(json!({ "ok": true, "voided": voided })))
}

#[derive(Deserialize)]
pub struct LookupReq {
    code: String,
}

/// 用 POST 而不是 GET：码不进 URL，也就不会落进访问日志。
pub async fn lookup_code(State(state): State<AppState>, _admin: AuthAdmin, Json(req): Json<LookupReq>) -> Result<Json<Value>, AppError> {
    Ok(Json(json!({ "code": redeem::lookup(&state, &req.code).await? })))
}

pub async fn void_code(State(state): State<AppState>, AuthAdmin(admin): AuthAdmin, Path(id): Path<i64>) -> Result<Json<Value>, AppError> {
    redeem::void_code(&state, id).await?;
    audit(&state, &admin, "code.void", Some(&id.to_string()), None).await?;
    Ok(Json(json!({ "ok": true })))
}

#[derive(Deserialize)]
pub struct AuditQuery {
    limit: Option<i64>,
}

#[derive(Serialize, FromRow)]
struct AuditRow {
    admin: String,
    action: String,
    target: Option<String>,
    detail: Option<String>,
    created_at: DateTime<Utc>,
}

pub async fn audit_log(State(state): State<AppState>, _admin: AuthAdmin, Query(query): Query<AuditQuery>) -> Result<Json<Value>, AppError> {
    let items: Vec<AuditRow> = sqlx::query_as(
        "SELECT admin_username AS admin, action, target, detail, created_at FROM admin_audit ORDER BY id DESC LIMIT $1",
    )
    .bind(query.limit.unwrap_or(50).clamp(1, 200))
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(json!({ "items": items })))
}
