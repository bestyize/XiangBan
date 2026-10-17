//! 兑换码：用户兑换、管理员生成批次、作废与查询。码本身见 `codes.rs`。

use crate::{
    binding, codes,
    error::AppError,
    products::Product,
    quota,
    state::AppState,
    sync,
};
use axum::http::StatusCode;
use chrono::{DateTime, Duration, Utc};
use serde::Serialize;
use sqlx::FromRow;
use std::collections::HashSet;
use uuid::Uuid;

fn mistake(status: StatusCode, code: &'static str, message: &str) -> AppError {
    AppError::Coded { status, code, message: message.into() }
}

/// 同一句话同时用于「格式不对」和「格式对但不存在」，不让人通过提示区分哪些码是真的。
fn invalid_code() -> AppError {
    mistake(StatusCode::BAD_REQUEST, "code_invalid", "兑换码不对，请检查后重新输入（注意区分字母和数字）")
}

/// 属于用户自己输错或误用的情况才计入失败次数；数据库、网关故障不算他的错。
fn counts_as_user_mistake(error: &AppError) -> bool {
    matches!(error, AppError::Coded { code, .. } if code.starts_with("code_"))
}

#[derive(Debug, Serialize)]
pub struct Redeemed {
    pub kind: String,
    pub quota: i64,
    pub expires_at: DateTime<Utc>,
    pub balance: i64,
}

pub async fn redeem(state: &AppState, uid: Uuid, product: &Product, raw: &str, ip: &str) -> Result<Redeemed, AppError> {
    let user_key = format!("u:{uid}");
    let ip_key = format!("ip:{ip}");
    let blocked = state.redeem_user_limiter.blocked(&user_key).max(state.redeem_ip_limiter.blocked(&ip_key));
    if let Some(wait) = blocked {
        return Err(AppError::Coded {
            status: StatusCode::TOO_MANY_REQUESTS,
            code: "too_many_attempts",
            message: format!("输错的次数太多了，请 {} 分钟后再试", (wait + 59) / 60),
        });
    }

    let result = redeem_once(state, uid, product, raw).await;
    if result.as_ref().is_err_and(counts_as_user_mistake) {
        state.redeem_user_limiter.record_failure(&user_key);
        state.redeem_ip_limiter.record_failure(&ip_key);
    }
    result
}

#[derive(FromRow)]
struct CodeRow {
    id: i64,
    status: String,
    product_id: String,
    kind: String,
    quota: i64,
    valid_days: i32,
}

const CODE_SELECT: &str = "SELECT c.id, c.status, b.product_id, b.kind, b.quota, b.valid_days
    FROM redeem_codes c JOIN redeem_batches b ON b.id = c.batch_id WHERE c.code_hash = $1";

fn check_usable(row: &CodeRow, product: &Product) -> Result<(), AppError> {
    match row.status.as_str() {
        "unused" => {}
        "redeemed" => return Err(mistake(StatusCode::CONFLICT, "code_used", "这个兑换码已经被使用过了")),
        _ => return Err(mistake(StatusCode::CONFLICT, "code_void", "这个兑换码已失效，请联系卖家")),
    }
    if row.product_id != product.product_id {
        return Err(mistake(StatusCode::BAD_REQUEST, "code_wrong_product", "这个兑换码不能用于当前应用"));
    }
    Ok(())
}

async fn redeem_once(state: &AppState, uid: Uuid, product: &Product, raw: &str) -> Result<Redeemed, AppError> {
    let canonical = codes::canonical(raw).ok_or_else(invalid_code)?;
    let hash = state.keys.code_hash(&canonical);

    let peek: CodeRow = sqlx::query_as(CODE_SELECT).bind(&hash).fetch_optional(&state.pool).await?.ok_or_else(invalid_code)?;
    check_usable(&peek, product)?;

    // 开通要访问 New API，所以放在入账事务之前；放在确认码有效之后，乱输码不会白建用户。
    let active = binding::ensure_active(state, uid, product).await?;

    let mut tx = state.pool.begin().await?;
    let locked: CodeRow = sqlx::query_as(&format!("{CODE_SELECT} FOR UPDATE OF c"))
        .bind(&hash)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(invalid_code)?;
    // 并发兑换同一个码时，后到的在这里看到已经用掉了。
    check_usable(&locked, product)?;

    let expires_at = Utc::now() + Duration::days(i64::from(locked.valid_days));
    let source = format!("code:{}", locked.id);
    let bucket_id = quota::grant(&mut tx, active.id, &locked.kind, locked.quota, Some(expires_at), &source, "兑换码")
        .await?
        .ok_or_else(|| AppError::Internal(format!("兑换码 {} 入账时没有产生额度桶", locked.id)))?;
    sqlx::query("UPDATE redeem_codes SET status = 'redeemed', redeemed_by = $2, binding_id = $3, bucket_id = $4, redeemed_at = now() WHERE id = $1")
        .bind(locked.id)
        .bind(uid)
        .bind(active.id)
        .bind(bucket_id)
        .execute(&mut *tx)
        .await?;
    let balance = quota::live_total(&mut tx, active.id).await?;
    tx.commit().await?;

    // 入账已经提交；推送到 New API 失败也不影响结果，后台对账会补上。
    if let Err(e) = sync::sync_binding(state, active.id).await {
        tracing::warn!("binding {} 兑换后对账失败，稍后由后台任务重试: {e}", active.id);
    }
    Ok(Redeemed { kind: locked.kind, quota: locked.quota, expires_at, balance })
}

// ---- 管理员：批次 ----

pub const MAX_BATCH_SIZE: i64 = 2000;

pub struct NewBatch {
    pub product_id: String,
    pub kind: String,
    pub quota: i64,
    pub valid_days: i32,
    pub count: i64,
    pub note: String,
}

#[derive(Debug, Serialize, FromRow)]
pub struct BatchRow {
    pub id: i64,
    pub product_id: String,
    pub kind: String,
    pub quota: i64,
    pub valid_days: i32,
    pub note: String,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub total: i64,
    pub unused: i64,
    pub redeemed: i64,
    pub void: i64,
}

const BATCH_SELECT: &str = "SELECT b.id, b.product_id, b.kind, b.quota, b.valid_days, b.note, b.created_by, b.created_at,
        COUNT(c.id) AS total,
        COUNT(c.id) FILTER (WHERE c.status = 'unused') AS unused,
        COUNT(c.id) FILTER (WHERE c.status = 'redeemed') AS redeemed,
        COUNT(c.id) FILTER (WHERE c.status = 'void') AS void
    FROM redeem_batches b LEFT JOIN redeem_codes c ON c.batch_id = b.id";

pub async fn list_batches(state: &AppState) -> Result<Vec<BatchRow>, AppError> {
    Ok(sqlx::query_as(&format!("{BATCH_SELECT} GROUP BY b.id ORDER BY b.id DESC LIMIT 100")).fetch_all(&state.pool).await?)
}

/// 建批次并生成码，返回明文（四个一组的展示形式）。明文只在这一次返回，数据库里只有哈希。
pub async fn create_batch(state: &AppState, admin: &str, req: NewBatch) -> Result<(BatchRow, Vec<String>), AppError> {
    let mut tx = state.pool.begin().await?;
    let (batch_id,): (i64,) = sqlx::query_as(
        "INSERT INTO redeem_batches (product_id, kind, quota, valid_days, note, created_by) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
    )
    .bind(&req.product_id)
    .bind(&req.kind)
    .bind(req.quota)
    .bind(req.valid_days)
    .bind(&req.note)
    .bind(admin)
    .fetch_one(&mut *tx)
    .await?;

    let wanted = req.count as usize;
    let mut issued: Vec<String> = Vec::with_capacity(wanted);
    let mut seen: HashSet<Vec<u8>> = HashSet::with_capacity(wanted);
    for _round in 0..5 {
        if issued.len() == wanted {
            break;
        }
        let fresh: Vec<String> = (0..wanted - issued.len()).map(|_| codes::generate()).collect();
        let hashes: Vec<Vec<u8>> = fresh.iter().map(|c| state.keys.code_hash(c)).collect();
        // 撞库的概率约 2^-70，这里仍按冲突处理：没插进去的下一轮重新生成。
        let inserted: Vec<(Vec<u8>,)> = sqlx::query_as(
            "INSERT INTO redeem_codes (batch_id, code_hash) SELECT $1, h FROM unnest($2::bytea[]) AS h
             ON CONFLICT (code_hash) DO NOTHING RETURNING code_hash",
        )
        .bind(batch_id)
        .bind(&hashes)
        .fetch_all(&mut *tx)
        .await?;
        let accepted: HashSet<Vec<u8>> = inserted.into_iter().map(|(h,)| h).collect();
        for (code, hash) in fresh.into_iter().zip(hashes) {
            if accepted.contains(&hash) && seen.insert(hash) {
                issued.push(code);
            }
        }
    }
    if issued.len() != wanted {
        return Err(AppError::Internal(format!("兑换码生成不足：需要 {wanted}，实际 {}", issued.len())));
    }

    let batch: BatchRow = sqlx::query_as(&format!("{BATCH_SELECT} WHERE b.id = $1 GROUP BY b.id")).bind(batch_id).fetch_one(&mut *tx).await?;
    tx.commit().await?;
    Ok((batch, issued.iter().map(|c| codes::display(c)).collect()))
}

/// 作废批次里所有还没兑换的码，返回作废了多少个。已经兑换的不受影响。
pub async fn void_batch(state: &AppState, batch_id: i64) -> Result<u64, AppError> {
    let exists: Option<(i64,)> = sqlx::query_as("SELECT id FROM redeem_batches WHERE id = $1").bind(batch_id).fetch_optional(&state.pool).await?;
    if exists.is_none() {
        return Err(AppError::NotFound("批次不存在".into()));
    }
    let result = sqlx::query("UPDATE redeem_codes SET status = 'void', voided_at = now() WHERE batch_id = $1 AND status = 'unused'")
        .bind(batch_id)
        .execute(&state.pool)
        .await?;
    Ok(result.rows_affected())
}

// ---- 管理员：查单个码 ----

#[derive(Debug, Serialize, FromRow)]
pub struct CodeLookup {
    pub id: i64,
    pub status: String,
    pub batch_id: i64,
    pub product_id: String,
    pub kind: String,
    pub quota: i64,
    pub valid_days: i32,
    pub note: String,
    pub redeemed_by: Option<Uuid>,
    pub redeemed_at: Option<DateTime<Utc>>,
    pub voided_at: Option<DateTime<Utc>>,
    pub binding_id: Option<i64>,
    pub bucket_id: Option<i64>,
    pub bucket_remaining: Option<i64>,
    pub bucket_expires_at: Option<DateTime<Utc>>,
    pub bucket_closed_reason: Option<String>,
}

pub async fn lookup(state: &AppState, raw: &str) -> Result<CodeLookup, AppError> {
    let canonical = codes::canonical(raw).ok_or_else(|| AppError::BadRequest("这不是一个格式正确的兑换码，请检查有没有漏字或写错".into()))?;
    sqlx::query_as(
        "SELECT c.id, c.status, c.batch_id, b.product_id, b.kind, b.quota, b.valid_days, b.note,
                c.redeemed_by, c.redeemed_at, c.voided_at, c.binding_id, c.bucket_id,
                q.remaining AS bucket_remaining, q.expires_at AS bucket_expires_at, q.closed_reason AS bucket_closed_reason
         FROM redeem_codes c JOIN redeem_batches b ON b.id = c.batch_id LEFT JOIN quota_buckets q ON q.id = c.bucket_id
         WHERE c.code_hash = $1",
    )
    .bind(state.keys.code_hash(&canonical))
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("系统里没有这个码（不是本系统生成的）".into()))
}

/// 作废一个还没兑换的码（淘宝订单退款时用）。已经兑换的要改用「扣回额度」。
pub async fn void_code(state: &AppState, code_id: i64) -> Result<(), AppError> {
    let result = sqlx::query("UPDATE redeem_codes SET status = 'void', voided_at = now() WHERE id = $1 AND status = 'unused'")
        .bind(code_id)
        .execute(&state.pool)
        .await?;
    if result.rows_affected() > 0 {
        return Ok(());
    }
    let status: Option<(String,)> = sqlx::query_as("SELECT status FROM redeem_codes WHERE id = $1").bind(code_id).fetch_optional(&state.pool).await?;
    Err(match status {
        None => AppError::NotFound("兑换码不存在".into()),
        Some((s,)) if s == "redeemed" => AppError::Conflict("这个码已经被兑换了，请改用「扣回额度」".into()),
        Some(_) => AppError::Conflict("这个码已经作废过了".into()),
    })
}
