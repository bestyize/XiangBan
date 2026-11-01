//! Private account assets. Client sessions never become background-job credentials.
mod delivery;
mod schedule;
#[cfg(test)]
mod tests;
mod worker;

use crate::{
    auth::AuthUser,
    error::{AppError, AppResult},
    state::AppState,
};
use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Duration, Utc};
use schedule::Schedule;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

pub async fn init(pool: &PgPool) -> AppResult<()> {
    sqlx::raw_sql(include_str!("schema.sql"))
        .execute(pool)
        .await?;
    Ok(())
}
pub fn spawn(state: AppState) {
    worker::spawn(state);
}
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/me/information-subscriptions", get(list).post(create))
        .route(
            "/me/information-subscriptions/{id}",
            axum::routing::put(update).delete(remove),
        )
        .route(
            "/me/information-subscriptions/{id}/verification",
            post(send_verification),
        )
        .route("/me/information-subscriptions/{id}/verify", post(verify))
        .route("/me/information-editions", get(history))
        .route("/me/information-editions/{id}", get(edition))
}
#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct Subscription {
    id: Uuid,
    owner: Uuid,
    revision: i64,
    title: String,
    interests: String,
    recipient: String,
    schedule: Value,
    timezone: String,
    next_run: Option<DateTime<Utc>>,
    status: String,
    verified_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    id: Uuid,
    revision: i64,
    title: String,
    interests: String,
    recipient: String,
    schedule: Schedule,
    timezone: String,
    active: bool,
    confirmed: bool,
}
impl Input {
    fn validate(&mut self) -> AppResult<Option<DateTime<Utc>>> {
        self.title = self.title.trim().to_string();
        self.interests = self.interests.trim().to_string();
        self.recipient = self.recipient.trim().to_ascii_lowercase();
        if !self.confirmed
            || self.title.is_empty()
            || self.title.chars().count() > 100
            || self.interests.is_empty()
            || self.interests.chars().count() > 2000
        {
            return Err(AppError::BadRequest(
                "请确认主题、关注内容、收件人和时间（标题最多100字，关注内容最多2000字）".into(),
            ));
        }
        if self.recipient.len() > 254
            || self.recipient.contains(['<', '>', '\r', '\n', ' '])
            || self.recipient.parse::<lettre::Address>().is_err()
        {
            return Err(AppError::BadRequest("请填写有效的收件邮箱".into()));
        }
        let next = self.schedule.next(&self.timezone, Utc::now())?;
        if self.active && next.is_none() {
            return Err(AppError::BadRequest("一次性发送时间必须在未来".into()));
        }
        Ok(next)
    }
}
async fn list(State(s): State<AppState>, auth: AuthUser) -> AppResult<Json<Value>> {
    let rows=sqlx::query_as::<_,Subscription>("SELECT * FROM information_subscriptions WHERE owner=$1 AND status<>'deleted' ORDER BY created_at DESC")
        .bind(auth.user.uid).fetch_all(&s.pool).await?;
    Ok(Json(
        json!({"items":rows,"delivery_configured":delivery::configured(),"worker_enabled":worker::enabled()}),
    ))
}
async fn create(
    State(s): State<AppState>,
    auth: AuthUser,
    Json(mut input): Json<Input>,
) -> AppResult<Json<Subscription>> {
    let next = input.validate()?;
    let mut tx = s.pool.begin().await?;
    // Per-account lock protects both asset and request limits across instances.
    sqlx::query("SELECT uid FROM users WHERE uid=$1 FOR UPDATE")
        .bind(auth.user.uid)
        .execute(&mut *tx)
        .await?;
    if let Some(existing) = sqlx::query_as::<_, Subscription>(
        "SELECT * FROM information_subscriptions WHERE id=$1 AND owner=$2",
    )
    .bind(input.id)
    .bind(auth.user.uid)
    .fetch_optional(&mut *tx)
    .await?
    {
        return Ok(Json(existing));
    }
    let count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM information_subscriptions WHERE owner=$1 AND status<>'deleted'",
    )
    .bind(auth.user.uid)
    .fetch_one(&mut *tx)
    .await?;
    if count >= 20 {
        return Err(AppError::BadRequest(
            "最多保存20份订阅，请先删除不再需要的订阅".into(),
        ));
    }
    sqlx::query(
        "INSERT INTO information_limits(owner,day) VALUES($1,CURRENT_DATE) ON CONFLICT DO NOTHING",
    )
    .bind(auth.user.uid)
    .execute(&mut *tx)
    .await?;
    let used=sqlx::query("UPDATE information_limits SET creates=creates+1 WHERE owner=$1 AND day=CURRENT_DATE AND creates<30").bind(auth.user.uid).execute(&mut *tx).await?;
    if used.rows_affected() == 0 {
        return Err(limit());
    }
    let row=sqlx::query_as::<_,Subscription>("INSERT INTO information_subscriptions(id,owner,title,interests,recipient,schedule,timezone,next_run,status) VALUES($1,$2,$3,$4,$5,$6,$7,$8,'unverified') RETURNING *")
        .bind(input.id).bind(auth.user.uid).bind(input.title).bind(input.interests).bind(input.recipient).bind(json!(input.schedule)).bind(input.timezone).bind(next).fetch_one(&mut *tx).await?;
    tx.commit().await?;
    Ok(Json(row))
}
fn limit() -> AppError {
    AppError::TooManyRequests {
        message: "请求过于频繁，请稍后再试".into(),
        retry_after: 3600,
    }
}
async fn update(
    State(s): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(mut input): Json<Input>,
) -> AppResult<Json<Subscription>> {
    let next = input.validate()?;
    if id != input.id {
        return Err(AppError::BadRequest("订阅编号不一致".into()));
    }
    let mut tx = s.pool.begin().await?;
    let old = owned(&mut tx, id, auth.user.uid).await?;
    if input.revision != old.revision {
        return Err(AppError::Conflict(
            "订阅已在另一端更新，请刷新后重试".into(),
        ));
    }
    let verified = if old.recipient == input.recipient {
        old.verified_at
    } else {
        None
    };
    let status = if verified.is_none() {
        "unverified"
    } else if input.active {
        "active"
    } else {
        "paused"
    };
    let row=sqlx::query_as::<_,Subscription>("UPDATE information_subscriptions SET title=$2,interests=$3,recipient=$4,schedule=$5,timezone=$6,next_run=$7,status=$8,verified_at=$9,revision=revision+1,updated_at=now() WHERE id=$1 RETURNING *")
        .bind(id).bind(input.title).bind(input.interests).bind(input.recipient).bind(json!(input.schedule)).bind(input.timezone).bind(next).bind(status).bind(verified).fetch_one(&mut *tx).await?;
    cancel(&mut tx, id).await?;
    sqlx::query("DELETE FROM information_verifications WHERE subscription_id=$1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(Json(row))
}
async fn owned(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    id: Uuid,
    owner: Uuid,
) -> AppResult<Subscription> {
    sqlx::query_as("SELECT * FROM information_subscriptions WHERE id=$1 AND owner=$2 AND status<>'deleted' FOR UPDATE").bind(id).bind(owner).fetch_optional(&mut **tx).await?.ok_or_else(||AppError::not_found("订阅"))
}
async fn cancel(tx: &mut sqlx::Transaction<'_, sqlx::Postgres>, id: Uuid) -> AppResult<()> {
    sqlx::query("UPDATE information_editions SET status='cancelled',lease_id=NULL,lease_until=NULL WHERE subscription_id=$1 AND status IN ('queued','building','retry','ready')").bind(id).execute(&mut **tx).await?;
    Ok(())
}
#[derive(Deserialize)]
struct Revision {
    revision: i64,
}
async fn remove(
    State(s): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(input): Json<Revision>,
) -> AppResult<Json<Value>> {
    let mut tx = s.pool.begin().await?;
    let old = owned(&mut tx, id, auth.user.uid).await?;
    if old.revision != input.revision {
        return Err(AppError::Conflict("订阅已更新，请刷新".into()));
    }
    sqlx::query("UPDATE information_subscriptions SET status='deleted',next_run=NULL,revision=revision+1,updated_at=now() WHERE id=$1").bind(id).execute(&mut *tx).await?;
    cancel(&mut tx, id).await?;
    sqlx::query("DELETE FROM information_verifications WHERE subscription_id=$1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(Json(json!({"ok":true})))
}
fn digest(id: Uuid, code: &str) -> Vec<u8> {
    ring::digest::digest(&ring::digest::SHA256, format!("{id}:{code}").as_bytes())
        .as_ref()
        .to_vec()
}
async fn send_verification(
    State(s): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    if !delivery::configured() {
        return Err(AppError::ServiceUnavailable(
            "发信服务尚未配置，订阅已保存".into(),
        ));
    }
    let code = Uuid::new_v4().simple().to_string();
    let mut tx = s.pool.begin().await?;
    sqlx::query("SELECT uid FROM users WHERE uid=$1 FOR UPDATE")
        .bind(auth.user.uid)
        .execute(&mut *tx)
        .await?;
    let sub = owned(&mut tx, id, auth.user.uid).await?;
    if sub.verified_at.is_some() {
        return Ok(Json(json!({"verified":true})));
    }
    let recent:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM information_verifications WHERE subscription_id=$1 AND sent_at>now()-interval '5 minutes')").bind(id).fetch_one(&mut *tx).await?;
    if recent {
        return Err(limit());
    }
    sqlx::query(
        "INSERT INTO information_limits(owner,day) VALUES($1,CURRENT_DATE) ON CONFLICT DO NOTHING",
    )
    .bind(auth.user.uid)
    .execute(&mut *tx)
    .await?;
    if sqlx::query("UPDATE information_limits SET verifications=verifications+1 WHERE owner=$1 AND day=CURRENT_DATE AND verifications<5").bind(auth.user.uid).execute(&mut *tx).await?.rows_affected()==0{return Err(limit());}
    sqlx::query("INSERT INTO information_verifications(subscription_id,digest,expires_at) VALUES($1,$2,$3) ON CONFLICT(subscription_id) DO UPDATE SET digest=$2,expires_at=$3,attempts=0,sent_at=now()")
        .bind(id).bind(digest(id,&code)).bind(Utc::now()+Duration::minutes(30)).execute(&mut *tx).await?;
    tx.commit().await?;
    let text=format!("小伴资讯订阅邮箱确认\n\n请在小伴资讯订阅页面粘贴以下确认码，30分钟内有效：\n{code}\n\n仅在你申请订阅时确认。我们不会读取你的收件箱。若非本人申请，请忽略。");
    delivery::send(
        &sub.recipient,
        "确认小伴资讯订阅",
        &format!("<pre>{}</pre>", delivery::escape(&text)),
        &text,
        Uuid::new_v4(),
    )
    .await
    .map_err(|_| {
        AppError::ServiceUnavailable("确认邮件发送结果不明，请检查邮箱，5分钟后可重新申请".into())
    })?;
    Ok(Json(json!({"ok":true})))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Code {
    code: String,
}
async fn verify(
    State(s): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(input): Json<Code>,
) -> AppResult<Json<Value>> {
    let mut tx = s.pool.begin().await?;
    let sub = owned(&mut tx, id, auth.user.uid).await?;
    let row:Option<(Vec<u8>,)>=sqlx::query_as("UPDATE information_verifications SET attempts=attempts+1 WHERE subscription_id=$1 AND attempts<5 AND expires_at>now() RETURNING digest").bind(id).fetch_optional(&mut *tx).await?;
    let valid = row.is_some_and(|r| r.0 == digest(id, input.code.trim()));
    if !valid {
        tx.commit().await?;
        return Err(AppError::BadRequest("确认码无效或已过期".into()));
    }
    let schedule: Schedule = serde_json::from_value(sub.schedule)
        .map_err(|_| AppError::Internal("invalid stored schedule".into()))?;
    let next = schedule.next(&sub.timezone, Utc::now())?;
    sqlx::query("UPDATE information_subscriptions SET verified_at=now(),status=$2,next_run=$3,revision=revision+1,updated_at=now() WHERE id=$1").bind(id).bind(if next.is_some(){"active"}else{"completed"}).bind(next).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM information_verifications WHERE subscription_id=$1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(Json(json!({"ok":true})))
}
#[derive(Deserialize)]
struct Page {
    before: Option<DateTime<Utc>>,
    subscription_id: Option<Uuid>,
}
async fn history(
    State(s): State<AppState>,
    auth: AuthUser,
    Query(q): Query<Page>,
) -> AppResult<Json<Value>> {
    let items:Vec<(Value,)>=sqlx::query_as("SELECT jsonb_build_object('id',id,'subscription_id',subscription_id,'title',snapshot->>'title','status',status,'error',error,'accounting_status',accounting_status,'created_at',created_at,'sent_at',sent_at,'due_at',due_at) FROM information_editions WHERE owner=$1 AND ($2::timestamptz IS NULL OR created_at<$2) AND ($3::uuid IS NULL OR subscription_id=$3) ORDER BY created_at DESC LIMIT 30").bind(auth.user.uid).bind(q.before).bind(q.subscription_id).fetch_all(&s.pool).await?;
    Ok(Json(
        json!({"items":items.into_iter().map(|x|x.0).collect::<Vec<_>>()}),
    ))
}
async fn edition(
    State(s): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    let row:Option<(Value,)>=sqlx::query_as("SELECT jsonb_build_object('id',id,'title',snapshot->>'title','status',status,'plaintext',plaintext,'html',html,'sources',sources,'created_at',created_at) FROM information_editions WHERE id=$1 AND owner=$2").bind(id).bind(auth.user.uid).fetch_optional(&s.pool).await?;
    Ok(Json(row.ok_or_else(|| AppError::not_found("期刊"))?.0))
}
