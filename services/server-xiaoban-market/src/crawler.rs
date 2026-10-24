//! 采集任务运行器：按来源逐页抓取上游目录，写入暂存区；可选自动入库。
use crate::{
    error::{AppError, Result},
    importer::{self, ImportOpts},
    state::AppState,
    upstream::{self, Params, RawEntry, SourceRow},
};
use serde::Serialize;
use sqlx::{types::Json, PgPool};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};
use uuid::Uuid;

#[derive(Debug, Default, Clone, Serialize)]
pub struct Stats {
    pub pages: u32,
    pub fetched: u32,
    pub new: u32,
    pub changed: u32,
    pub unchanged: u32,
    pub skipped: u32,
    pub failed: u32,
    pub imported: u32,
    pub import_failed: u32,
}

#[derive(Default)]
pub struct Runner {
    cancels: Mutex<HashMap<Uuid, Arc<AtomicBool>>>,
}

impl Runner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cancel(&self, job_id: Uuid) -> bool {
        match self.cancels.lock().unwrap().get(&job_id) {
            Some(flag) => {
                flag.store(true, Ordering::SeqCst);
                true
            }
            None => false,
        }
    }

    fn register(&self, job_id: Uuid) -> Arc<AtomicBool> {
        let flag = Arc::new(AtomicBool::new(false));
        self.cancels.lock().unwrap().insert(job_id, flag.clone());
        flag
    }

    fn finish(&self, job_id: Uuid) {
        self.cancels.lock().unwrap().remove(&job_id);
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Change {
    New,
    Changed,
    Unchanged,
}

pub async fn log(pool: &PgPool, job_id: Uuid, level: &str, message: &str) {
    let message: String = message.chars().take(500).collect();
    sqlx::query("INSERT INTO crawl_logs (job_id, level, message) VALUES ($1, $2, $3)")
        .bind(job_id)
        .bind(level)
        .bind(message)
        .execute(pool)
        .await
        .ok();
}

/// 服务重启后，上一次进程里未结束的任务不会再继续，统一标记失败。
pub async fn recover(pool: &PgPool) -> Result<()> {
    sqlx::query("UPDATE crawl_jobs SET status='failed', error='服务重启，任务中断', finished_at=now() WHERE status IN ('queued','running')")
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn start(state: &AppState, source_id: i32, params: Params, trigger: &str, created_by: Option<&str>) -> Result<Uuid> {
    let source: SourceRow = sqlx::query_as("SELECT * FROM sources WHERE id = $1")
        .bind(source_id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("采集来源不存在".into()))?;
    if !source.enabled {
        return Err(AppError::Conflict("该来源已停用".into()));
    }
    let inserted = sqlx::query_scalar::<_, Uuid>("INSERT INTO crawl_jobs (source_id, trigger, params, created_by) VALUES ($1, $2, $3, $4) RETURNING id")
        .bind(source_id)
        .bind(trigger)
        .bind(Json(&params))
        .bind(created_by)
        .fetch_one(&state.pool)
        .await;
    let job_id = match inserted {
        Ok(id) => id,
        Err(sqlx::Error::Database(e)) if e.is_unique_violation() => return Err(AppError::Conflict("该来源已有进行中的任务".into())),
        Err(e) => return Err(e.into()),
    };
    sqlx::query("UPDATE sources SET last_job_at = now() WHERE id = $1").bind(source_id).execute(&state.pool).await?;
    let cancel = state.runner.register(job_id);
    let state = state.clone();
    tokio::spawn(async move { run_job(state, job_id, source, params, cancel).await });
    Ok(job_id)
}

async fn run_job(state: AppState, job_id: Uuid, source: SourceRow, params: Params, cancel: Arc<AtomicBool>) {
    let outcome = execute(&state, job_id, &source, &params, &cancel).await;
    state.runner.finish(job_id);
    let (status, error) = match outcome {
        Ok(true) => ("succeeded", None),
        Ok(false) => ("cancelled", None),
        Err(message) => ("failed", Some(message)),
    };
    match &error {
        Some(message) => log(&state.pool, job_id, "error", &format!("任务失败：{message}")).await,
        None if status == "cancelled" => log(&state.pool, job_id, "warn", "任务已取消").await,
        None => log(&state.pool, job_id, "info", "任务完成").await,
    }
    sqlx::query("UPDATE crawl_jobs SET status=$2, error=$3, finished_at=now() WHERE id=$1")
        .bind(job_id)
        .bind(status)
        .bind(error)
        .execute(&state.pool)
        .await
        .ok();
}

async fn save_stats(pool: &PgPool, job_id: Uuid, stats: &Stats) {
    sqlx::query("UPDATE crawl_jobs SET stats = $2 WHERE id = $1").bind(job_id).bind(Json(stats)).execute(pool).await.ok();
}

/// `Ok(true)` 完成，`Ok(false)` 被取消，`Err` 失败。
async fn execute(state: &AppState, job_id: Uuid, source: &SourceRow, params: &Params, cancel: &AtomicBool) -> std::result::Result<bool, String> {
    let pool = &state.pool;
    sqlx::query("UPDATE crawl_jobs SET status='running', started_at=now() WHERE id=$1").bind(job_id).execute(pool).await.map_err(|e| e.to_string())?;
    let max_pages = if params.max_pages == 0 { source.config["max_pages"].as_u64().unwrap_or(5) as u32 } else { params.max_pages }.clamp(1, 200);
    let delay = Duration::from_millis(source.config["request_interval_ms"].as_u64().unwrap_or(800).min(10_000));
    let retry_base = source.config["retry_base_ms"].as_u64().unwrap_or(2000).min(30_000);
    let ctx = state.ctx();
    let mut stats = Stats::default();
    log(pool, job_id, "info", &format!("开始采集：{}，最多 {max_pages} 页", source.name)).await;

    for page in 1..=max_pages {
        if cancel.load(Ordering::SeqCst) {
            return Ok(false);
        }
        let mut attempt = 0u32;
        let result = loop {
            match upstream::fetch_page(&ctx, source, params, page).await {
                Ok(result) => break result,
                Err(e) if e.transient && attempt < 3 => {
                    attempt += 1;
                    let wait = e.retry_after.map(Duration::from_secs).unwrap_or(Duration::from_millis(retry_base << (attempt - 1))).min(Duration::from_secs(60));
                    log(pool, job_id, "warn", &format!("{}，{} 秒后重试第 {page} 页（{attempt}/3）", e.message, wait.as_secs())).await;
                    tokio::time::sleep(wait).await;
                    if cancel.load(Ordering::SeqCst) {
                        return Ok(false);
                    }
                }
                Err(e) => return Err(format!("第 {page} 页：{}", e.message)),
            }
        };
        stats.pages += 1;
        stats.skipped += result.skipped as u32;
        if result.skipped > 0 {
            log(pool, job_id, "warn", &format!("第 {page} 页跳过 {} 条：标识不合法或已被上游下架", result.skipped)).await;
        }
        let (mut new, mut changed, mut unchanged) = (0, 0, 0);
        let count = result.entries.len();
        for entry in &result.entries {
            stats.fetched += 1;
            match stage_upsert(pool, source.id, job_id, entry).await {
                Ok(Change::New) => new += 1,
                Ok(Change::Changed) => changed += 1,
                Ok(Change::Unchanged) => unchanged += 1,
                Err(e) => {
                    stats.failed += 1;
                    log(pool, job_id, "error", &format!("暂存失败（{}）：{e}", entry.normalized.upstream_id)).await;
                }
            }
        }
        stats.new += new;
        stats.changed += changed;
        stats.unchanged += unchanged;
        log(pool, job_id, "info", &format!("第 {page} 页：{count} 条（新增 {new} · 更新 {changed} · 无变化 {unchanged}）")).await;
        save_stats(pool, job_id, &stats).await;
        if !result.has_more || count == 0 {
            break;
        }
        if page < max_pages {
            tokio::time::sleep(delay).await;
        }
    }

    if source.auto_import != "none" {
        let rows: Vec<(Uuid, Option<Uuid>)> = sqlx::query_as("SELECT id, item_id FROM staged_items WHERE last_job_id = $1 AND status = 'pending' ORDER BY first_seen_at")
            .bind(job_id)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;
        log(pool, job_id, "info", &format!("自动入库 {} 项（新条目为「{}」）", rows.len(), source.auto_import)).await;
        for (id, existing) in rows {
            // 已存在的条目保持当前状态，避免把管理员下架的内容重新上架。
            let opts = ImportOpts { status: if existing.is_some() { None } else { Some(source.auto_import.clone()) }, category_id: None };
            match importer::import_staged(state, id, &opts).await {
                Ok(_) => stats.imported += 1,
                Err(e) => {
                    stats.import_failed += 1;
                    log(pool, job_id, "error", &format!("入库失败：{e}")).await;
                }
            }
        }
        save_stats(pool, job_id, &stats).await;
    }
    Ok(true)
}

pub async fn stage_upsert(pool: &PgPool, source_id: i32, job_id: Uuid, entry: &RawEntry) -> Result<Change> {
    let n = &entry.normalized;
    let hash = n.content_hash();
    let search = n.search_text();
    let existing: Option<(Uuid, String)> = sqlx::query_as("SELECT id, content_hash FROM staged_items WHERE source_id = $1 AND upstream_id = $2")
        .bind(source_id)
        .bind(&n.upstream_id)
        .fetch_optional(pool)
        .await?;
    match existing {
        None => {
            sqlx::query(
                "INSERT INTO staged_items (source_id, upstream_id, kind, name, summary, category, raw, normalized, content_hash, search_text, last_job_id)
                 VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)",
            )
            .bind(source_id)
            .bind(&n.upstream_id)
            .bind(&n.kind)
            .bind(&n.name)
            .bind(&n.summary)
            .bind(&n.category)
            .bind(Json(&entry.raw))
            .bind(Json(n))
            .bind(&hash)
            .bind(&search)
            .bind(job_id)
            .execute(pool)
            .await?;
            Ok(Change::New)
        }
        Some((id, old)) if old != hash => {
            // 内容变化：已忽略的保持忽略，其余回到待处理（已入库的会显示为「有更新」）。
            sqlx::query(
                "UPDATE staged_items SET name=$2, summary=$3, category=$4, raw=$5, normalized=$6, content_hash=$7, search_text=$8,
                   last_seen_at=now(), last_job_id=$9, import_error=NULL,
                   status = CASE WHEN status = 'ignored' THEN 'ignored' ELSE 'pending' END
                 WHERE id=$1",
            )
            .bind(id)
            .bind(&n.name)
            .bind(&n.summary)
            .bind(&n.category)
            .bind(Json(&entry.raw))
            .bind(Json(n))
            .bind(&hash)
            .bind(&search)
            .bind(job_id)
            .execute(pool)
            .await?;
            Ok(Change::Changed)
        }
        Some((id, _)) => {
            sqlx::query("UPDATE staged_items SET last_seen_at=now(), last_job_id=$2 WHERE id=$1").bind(id).bind(job_id).execute(pool).await?;
            Ok(Change::Unchanged)
        }
    }
}

/// 每分钟检查一次到期的定时来源。
pub async fn scheduler(state: AppState) {
    let mut tick = tokio::time::interval(Duration::from_secs(60));
    loop {
        tick.tick().await;
        let due: Vec<i32> = sqlx::query_scalar(
            "SELECT id FROM sources WHERE enabled AND interval_minutes > 0
               AND (last_job_at IS NULL OR last_job_at < now() - make_interval(mins => interval_minutes))",
        )
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default();
        for id in due {
            match start(&state, id, Params::default(), "schedule", None).await {
                Ok(job) => tracing::info!("定时采集已启动：来源 {id}，任务 {job}"),
                Err(AppError::Conflict(_)) => {}
                Err(e) => tracing::warn!("定时采集启动失败：来源 {id}：{e}"),
            }
        }
    }
}
