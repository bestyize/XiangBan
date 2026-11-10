//! Durable, per-account image generation jobs. Only a queued row can be claimed.
//! A lost worker response leaves an unknown outcome and cannot restart a paid call.

use crate::{error::AppError, extract::AuthUser, media, products::Capability, proxy::{error_response, map_body_error, Api}, state::AppState};
use axum::{
    extract::{rejection::BytesRejection, Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use bytes::Bytes;
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::{FromRow, PgPool};
use std::time::Duration;
use uuid::Uuid;

const MAX_JOB_REQUEST: usize = 32 * 1024;
const JOB_TIMEOUT: Duration = Duration::from_secs(25 * 60);
const UNKNOWN: &str = "生图结果尚未确认，请核对已收到的图片和点数。";

#[derive(Deserialize)]
struct Submission {
    job_id: Uuid,
    parameters: Value,
}

#[derive(Clone)]
pub(crate) struct ImageJobKey {
    uid: Uuid,
    product: String,
    id: Uuid,
}

#[derive(FromRow)]
struct Job {
    status: String,
    request: Value,
    result: Option<Value>,
    error: Option<Value>,
}

impl ImageJobKey {
    fn new(auth: &AuthUser, id: Uuid) -> Self {
        Self { uid: auth.uid, product: auth.product.product_id.clone(), id }
    }

    async fn read(&self, pool: &PgPool) -> Result<Option<Job>, AppError> {
        Ok(sqlx::query_as::<_, Job>(
            "SELECT CASE WHEN status = 'running' AND updated_at < now() - interval '26 minutes'
                         THEN CASE WHEN result IS NULL THEN 'unknown' ELSE 'partial' END ELSE status END AS status,
                    request,
                    CASE WHEN status NOT IN ('queued', 'running') OR updated_at < now() - interval '26 minutes'
                         THEN result END AS result, error
             FROM image_jobs WHERE uid = $1 AND product_id = $2 AND job_id = $3",
        ).bind(self.uid).bind(&self.product).bind(self.id).fetch_optional(pool).await?)
    }

    pub(crate) async fn record_partial(&self, pool: &PgPool, images: Value) -> Result<(), AppError> {
        self.update(pool, "running", Some(images), None).await
    }

    async fn update(&self, pool: &PgPool, status: &str, result: Option<Value>, error: Option<Value>) -> Result<(), AppError> {
        sqlx::query("UPDATE image_jobs SET status = $4, result = COALESCE($5, result), error = $6, updated_at = now()
                     WHERE uid = $1 AND product_id = $2 AND job_id = $3 AND status = 'running'")
            .bind(self.uid).bind(&self.product).bind(self.id).bind(status).bind(result).bind(error)
            .execute(pool).await?;
        Ok(())
    }

    fn response(&self, job: Job, http: StatusCode) -> Response {
        let mut body = json!({ "job_id": self.id, "status": job.status, "result": job.result, "error": job.error });
        if job.status == "unknown" && body["error"].is_null() {
            body["error"] = json!({ "code": "image_result_unknown", "message": UNKNOWN });
        }
        (http, [(header::CACHE_CONTROL, "no-store")], Json(body)).into_response()
    }
}

async fn execute(state: &AppState, auth: AuthUser, key: &ImageJobKey) -> Result<(), AppError> {
    let request: Option<Value> = sqlx::query_scalar(
        "UPDATE image_jobs SET status = 'running', updated_at = now()
         WHERE uid = $1 AND product_id = $2 AND job_id = $3 AND status = 'queued' RETURNING request",
    ).bind(key.uid).bind(&key.product).bind(key.id).fetch_optional(&state.pool).await?;
    let Some(request) = request else { return Ok(()) };

    let wanted = request["n"].as_u64().unwrap_or(1) as usize;
    let result = tokio::time::timeout(JOB_TIMEOUT, async {
        let setting = media::setting(state, &auth, Capability::Image).await?;
        let payload = media::image_payload(&setting.model, &setting.sizes, &serde_json::to_vec(&request)
            .map_err(|error| AppError::Internal(error.to_string()))?)?;
        media::generate_images(state, auth, setting, payload, Some(key)).await
    }).await;
    let (status, images, error) = match result {
        Ok(Ok(images)) => {
            let count = images["data"].as_array().map_or(0, Vec::len);
            (if count >= wanted { "succeeded" } else { "partial" }, Some(images), None)
        }
        Ok(Err(error)) => {
            let (status, code, message) = error.parts();
            if status.is_client_error() || matches!(code, "image_unavailable" | "insufficient_quota" | "paid_required") {
                ("failed", None, Some(json!({ "code": code, "message": message })))
            } else {
                ("unknown", None, Some(json!({ "code": "image_result_unknown", "message": UNKNOWN })))
            }
        }
        Err(_) => ("unknown", None, Some(json!({ "code": "image_result_unknown", "message": UNKNOWN }))),
    };
    key.update(&state.pool, status, images, error).await
}

fn start_if_queued(state: AppState, auth: AuthUser, key: ImageJobKey, job: &Job) {
    if job.status == "queued" {
        tokio::spawn(async move {
            if let Err(error) = execute(&state, auth, &key).await {
                tracing::warn!("image job {}: {error}", key.id);
            }
        });
    }
}

pub async fn submit(State(state): State<AppState>, auth: Result<AuthUser, AppError>, body: Result<Bytes, BytesRejection>) -> Response {
    let result = async {
        let auth = auth?;
        let body = body.map_err(|error| map_body_error(error.status()))?;
        if body.len() > MAX_JOB_REQUEST {
            return Err(map_body_error(StatusCode::PAYLOAD_TOO_LARGE));
        }
        let submission: Submission = serde_json::from_slice(&body).map_err(|_| AppError::BadRequest("生图任务参数无效".into()))?;
        let key = ImageJobKey::new(&auth, submission.job_id);
        let request = submission.parameters;
        if key.read(&state.pool).await?.is_none() {
            let setting = media::setting(&state, &auth, Capability::Image).await?;
            let body = serde_json::to_vec(&request).map_err(|error| AppError::Internal(error.to_string()))?;
            let payload = media::image_payload(&setting.model, &setting.sizes, &body)?;
            media::validate_image_route(&setting.model, &payload)?;
            sqlx::query("INSERT INTO image_jobs (uid, product_id, job_id, request) VALUES ($1, $2, $3, $4)
                         ON CONFLICT (uid, product_id, job_id) DO NOTHING")
                .bind(key.uid).bind(&key.product).bind(key.id).bind(&request).execute(&state.pool).await?;
        }
        let job = key.read(&state.pool).await?.ok_or_else(|| AppError::Internal("accepted image job is missing".into()))?;
        if job.status != "expired" && job.request != request {
            return Err(AppError::Conflict("生图任务编号已关联其他参数".into()));
        }
        start_if_queued(state, auth, key.clone(), &job);
        Ok(key.response(job, StatusCode::ACCEPTED))
    }.await;
    result.unwrap_or_else(|error| error_response(Api::ChatCompletions, error))
}

pub async fn status(State(state): State<AppState>, auth: Result<AuthUser, AppError>, Path(id): Path<Uuid>) -> Response {
    let result = async {
        let auth = auth?;
        let key = ImageJobKey::new(&auth, id);
        let job = key.read(&state.pool).await?.ok_or_else(|| AppError::Coded {
            status: StatusCode::NOT_FOUND, code: "image_job_not_found", message: "未找到这次生图任务".into(),
        })?;
        start_if_queued(state, auth, key.clone(), &job);
        Ok(key.response(job, StatusCode::OK))
    }.await;
    result.unwrap_or_else(|error| error_response(Api::ChatCompletions, error))
}

pub async fn maintain(pool: PgPool) {
    let mut ticker = tokio::time::interval(Duration::from_secs(60 * 60));
    loop {
        ticker.tick().await;
        // Keep a small tombstone so even a very late retry cannot restart a paid operation.
        if let Err(error) = sqlx::query(
            "UPDATE image_jobs SET status = 'expired', request = '{}'::jsonb, result = NULL, error = NULL
             WHERE status <> 'expired' AND updated_at < now() - interval '7 days'",
        ).execute(&pool).await {
            tracing::warn!("清理过期生图任务失败: {error}");
        }
    }
}
