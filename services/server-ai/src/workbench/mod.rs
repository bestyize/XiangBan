//! Account-scoped cloud conversations. Only the worker owns model and tool execution.
//! Browsers replay the same ordered events whether they arrive during or after a run.
mod worker;
use crate::{error::AppError, extract::AuthUser, state::AppState};
use axum::{
    extract::{DefaultBodyLimit, Path, Query, State},
    http::HeaderMap,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
    routing::{get, post},
    Json, Router,
};
use futures_util::stream;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{FromRow, PgPool, Postgres, Transaction};
use std::{collections::VecDeque, convert::Infallible, time::Duration};
use uuid::Uuid;

pub(super) fn routes() -> Router<AppState> {
    Router::new()
        .route("/sessions", get(list).post(create))
        .route(
            "/sessions/{session}",
            axum::routing::patch(rename).delete(remove),
        )
        .route("/sessions/{session}/messages", post(send))
        .route("/sessions/{session}/events", get(events))
        .route("/sessions/{session}/runs/{run}/cancel", post(cancel))
        .route(
            "/sessions/{session}/runs/{run}/approvals/{approval}",
            post(decide),
        )
        .layer(DefaultBodyLimit::max(256 * 1024))
}

#[derive(Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
struct Session {
    id: Uuid,
    title: String,
    created_at: i64,
    updated_at: i64,
}
const SUMMARY: &str = "id,title,(extract(epoch from created_at)*1000)::bigint AS created_at,(extract(epoch from updated_at)*1000)::bigint AS updated_at";
fn missing() -> AppError {
    AppError::NotFound("对话不存在".into())
}

async fn owned(
    tx: &mut Transaction<'_, Postgres>,
    auth: &AuthUser,
    session: Uuid,
) -> Result<Session, AppError> {
    sqlx::query_as(&format!("SELECT {SUMMARY} FROM workbench_sessions WHERE id=$1 AND uid=$2 AND product_id=$3 FOR UPDATE"))
        .bind(session).bind(auth.uid).bind(&auth.product.product_id).fetch_optional(&mut **tx).await?.ok_or_else(missing)
}

async fn list(State(s): State<AppState>, auth: AuthUser) -> Result<Json<Vec<Session>>, AppError> {
    Ok(Json(sqlx::query_as(&format!("SELECT {SUMMARY} FROM workbench_sessions WHERE uid=$1 AND product_id=$2 ORDER BY updated_at DESC,id"))
        .bind(auth.uid).bind(auth.product.product_id).fetch_all(&s.pool).await?))
}

#[derive(Deserialize)]
struct Create {
    id: Uuid,
}

async fn create(
    State(s): State<AppState>,
    auth: AuthUser,
    Json(body): Json<Create>,
) -> Result<Json<Session>, AppError> {
    let mut tx = s.pool.begin().await?;
    sqlx::query("INSERT INTO workbench_sessions(id,uid,product_id) VALUES ($1,$2,$3) ON CONFLICT DO NOTHING")
        .bind(body.id).bind(auth.uid).bind(&auth.product.product_id).execute(&mut *tx).await?;
    let result = owned(&mut tx, &auth, body.id).await?;
    tx.commit().await?;
    Ok(Json(result))
}

#[derive(Deserialize)]
struct Rename {
    title: String,
}

async fn rename(
    State(s): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<Rename>,
) -> Result<Json<Session>, AppError> {
    let title = body.title.trim();
    if title.is_empty() || title.chars().count() > 100 {
        return Err(AppError::BadRequest("标题需要 1 到 100 个字".into()));
    }
    let row=sqlx::query_as(&format!("UPDATE workbench_sessions SET title=$4,updated_at=now() WHERE id=$1 AND uid=$2 AND product_id=$3 RETURNING {SUMMARY}"))
        .bind(id).bind(auth.uid).bind(auth.product.product_id).bind(title).fetch_optional(&s.pool).await?.ok_or_else(missing)?;
    Ok(Json(row))
}

async fn remove(
    State(s): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let mut tx = s.pool.begin().await?;
    owned(&mut tx, &auth, id).await?;
    if active(&mut tx, id).await? {
        return Err(AppError::Conflict("请先停止当前回答".into()));
    }
    sqlx::query("DELETE FROM workbench_sessions WHERE id=$1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(Json(json!({})))
}

async fn active(tx: &mut Transaction<'_, Postgres>, id: Uuid) -> Result<bool, AppError> {
    Ok(sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM workbench_runs WHERE session_id=$1 AND status='running')",
    )
    .bind(id)
    .fetch_one(&mut **tx)
    .await?)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SendMessage {
    request_id: Uuid,
    text: String,
    tier: String,
}

async fn send(
    State(s): State<AppState>,
    auth: AuthUser,
    Path(session): Path<Uuid>,
    Json(body): Json<SendMessage>,
) -> Result<Json<Value>, AppError> {
    let text = body.text.trim();
    if text.is_empty() || body.tier.is_empty() || body.tier.len() > 100 {
        return Err(AppError::BadRequest("请填写消息并选择模型档位".into()));
    }
    let mut tx = s.pool.begin().await?;
    let summary = owned(&mut tx, &auth, session).await?;
    let previous: Option<(Uuid, String, String)> = sqlx::query_as(
        "SELECT id,input,tier FROM workbench_runs WHERE session_id=$1 AND request_id=$2",
    )
    .bind(session)
    .bind(body.request_id)
    .fetch_optional(&mut *tx)
    .await?;
    if let Some((id, input, tier)) = previous {
        if input != text || tier != body.tier {
            return Err(AppError::Conflict("这次发送的内容已改变".into()));
        }
        return Ok(Json(json!({"runId":id,"session":summary})));
    }
    if active(&mut tx, session).await? {
        return Err(AppError::Conflict("这个对话正在回答".into()));
    }
    let slot =
        s.workbench_slots.clone().try_acquire_owned().map_err(|_| {
            AppError::ServiceUnavailable("工作台正在处理其他任务，请稍后再发".into())
        })?;
    let run = Uuid::new_v4();
    sqlx::query("INSERT INTO workbench_runs(id,session_id,request_id,input,tier,status) VALUES($1,$2,$3,$4,$5,'running')")
        .bind(run).bind(session).bind(body.request_id).bind(text).bind(&body.tier).execute(&mut *tx).await?;
    record(&mut tx,session,run,json!({"type":"user_message","initial":true,"requestId":body.request_id,"message":{"role":"user","parts":[{"type":"text","text":text}]}})).await?;
    let summary:Session=sqlx::query_as(&format!("UPDATE workbench_sessions SET updated_at=now(), title=CASE WHEN title='新对话' AND NOT EXISTS(SELECT 1 FROM workbench_runs WHERE session_id=$1 AND id<>$2) THEN $3 ELSE title END WHERE id=$1 RETURNING {SUMMARY}"))
        .bind(session).bind(run).bind(text.chars().take(32).collect::<String>()).fetch_one(&mut *tx).await?;
    tx.commit().await?;
    let input = text.to_owned();
    tokio::spawn(async move {
        let _slot = slot;
        worker::execute(s, auth, session, run, input, body.tier).await;
    });
    Ok(Json(json!({"runId":run,"session":summary})))
}

async fn record(
    tx: &mut Transaction<'_, Postgres>,
    session: Uuid,
    run: Uuid,
    payload: Value,
) -> Result<(), AppError> {
    sqlx::query("INSERT INTO workbench_events(session_id,run_id,payload) VALUES($1,$2,$3)")
        .bind(session)
        .bind(run)
        .bind(sqlx::types::Json(payload))
        .execute(&mut **tx)
        .await?;
    Ok(())
}

async fn lock_run(
    tx: &mut Transaction<'_, Postgres>,
    session: Uuid,
    run: Uuid,
) -> Result<String, AppError> {
    sqlx::query_scalar("SELECT status FROM workbench_runs WHERE id=$1 AND session_id=$2 FOR UPDATE")
        .bind(run)
        .bind(session)
        .fetch_optional(&mut **tx)
        .await?
        .ok_or_else(missing)
}

async fn settle(
    tx: &mut Transaction<'_, Postgres>,
    session: Uuid,
    run: Uuid,
    status: &str,
    event: Value,
    history: Value,
) -> Result<(), AppError> {
    sqlx::query("UPDATE workbench_runs SET status=$2,history=$3 WHERE id=$1")
        .bind(run)
        .bind(status)
        .bind(sqlx::types::Json(history))
        .execute(&mut **tx)
        .await?;
    record(tx, session, run, event).await
}

async fn cancel(
    State(s): State<AppState>,
    auth: AuthUser,
    Path((session, run)): Path<(Uuid, Uuid)>,
) -> Result<Json<Value>, AppError> {
    let mut tx = s.pool.begin().await?;
    owned(&mut tx, &auth, session).await?;
    if lock_run(&mut tx, session, run).await? == "running" {
        settle(
            &mut tx,
            session,
            run,
            "cancelled",
            json!({"type":"cancelled"}),
            json!([]),
        )
        .await?;
    }
    tx.commit().await?;
    Ok(Json(json!({})))
}

#[derive(Deserialize)]
struct Decision {
    decision: String,
}

async fn decide(
    State(s): State<AppState>,
    auth: AuthUser,
    Path((session, run, id)): Path<(Uuid, Uuid, Uuid)>,
    Json(input): Json<Decision>,
) -> Result<Json<Value>, AppError> {
    if !matches!(input.decision.as_str(), "once" | "deny") {
        return Err(AppError::BadRequest("请选择允许一次或拒绝".into()));
    }
    let mut tx = s.pool.begin().await?;
    owned(&mut tx, &auth, session).await?;
    let status = lock_run(&mut tx, session, run).await?;
    let previous: Option<String> =
        sqlx::query_scalar("SELECT decision FROM workbench_approvals WHERE id=$1 AND run_id=$2")
            .bind(id)
            .bind(run)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(missing)?;
    if previous.as_deref() == Some(&input.decision) {
        return Ok(Json(json!({})));
    }
    if status != "running" || previous.is_some() {
        return Err(AppError::Conflict("这个请求已结束".into()));
    }
    sqlx::query("UPDATE workbench_approvals SET decision=$2 WHERE id=$1")
        .bind(id)
        .bind(input.decision)
        .execute(&mut *tx)
        .await?;
    record(
        &mut tx,
        session,
        run,
        json!({"type":"approval_resolved","id":id}),
    )
    .await?;
    tx.commit().await?;
    Ok(Json(json!({})))
}

#[derive(Deserialize, Default)]
struct Cursor {
    #[serde(default)]
    after: i64,
}

async fn events(
    State(s): State<AppState>,
    auth: AuthUser,
    Path(session): Path<Uuid>,
    Query(cursor): Query<Cursor>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let mut tx = s.pool.begin().await?;
    owned(&mut tx, &auth, session).await?;
    tx.commit().await?;
    let first = headers
        .get("last-event-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse().ok())
        .unwrap_or(cursor.after);
    let deadline = tokio::time::Instant::now() + Duration::from_secs(55);
    let pool = s.pool;
    let stream = stream::unfold(
        (
            first,
            VecDeque::<(i64, Uuid, sqlx::types::Json<Value>)>::new(),
            false,
            pool,
        ),
        move |(mut after, mut queue, mut caught_up, pool)| {
            let auth = auth.clone();
            async move {
                loop {
                    if let Some((seq, run, payload)) = queue.pop_front() {
                        after = seq;
                        let event = Event::default()
                            .id(seq.to_string())
                            .data(json!({"runId":run,"event":payload.0}).to_string());
                        return Some((Ok::<_, Infallible>(event), (after, queue, caught_up, pool)));
                    }
                    if tokio::time::Instant::now() >= deadline {
                        return None;
                    }
                    let rows=sqlx::query_as("SELECT e.seq,e.run_id,e.payload FROM workbench_events e JOIN workbench_sessions s ON s.id=e.session_id WHERE s.id=$1 AND s.uid=$2 AND s.product_id=$3 AND e.seq>$4 ORDER BY e.seq LIMIT 256")
                .bind(session).bind(auth.uid).bind(auth.product.product_id.clone()).bind(after).fetch_all(&pool).await.ok()?;
                    queue = VecDeque::from(rows);
                    if !queue.is_empty() {
                        continue;
                    }
                    if !caught_up {
                        caught_up = true;
                        return Some((
                            Ok(Event::default().event("ready").data("{}")),
                            (after, queue, caught_up, pool),
                        ));
                    }
                    tokio::time::sleep(Duration::from_millis(450)).await;
                }
            }
        },
    );
    Ok((
        [("cache-control", "no-store"), ("x-accel-buffering", "no")],
        Sse::new(stream).keep_alive(KeepAlive::default()),
    ))
}

/// A worker holds its lease across browser disconnects and approval waits. A process crash
/// terminates the abandoned turn once; a replacement never replays a tool or model call.
pub(super) async fn maintain(pool: PgPool) {
    loop {
        if let Err(error) = reap(&pool).await {
            tracing::warn!("workbench recovery: {error}");
        }
        tokio::time::sleep(Duration::from_secs(20)).await;
    }
}
pub(super) async fn reap(pool: &PgPool) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;
    let rows:Vec<(Uuid,Uuid)>=sqlx::query_as("SELECT session_id,id FROM workbench_runs WHERE status='running' AND heartbeat<now()-interval '90 seconds' ORDER BY heartbeat LIMIT 100 FOR UPDATE SKIP LOCKED")
        .fetch_all(&mut *tx).await?;
    for (session, run) in rows {
        settle(
            &mut tx,
            session,
            run,
            "error",
            json!({"type":"error","message":"服务重启中断了上次回答，已保留收到的内容。"}),
            json!([]),
        )
        .await?;
    }
    tx.commit().await?;
    Ok(())
}
