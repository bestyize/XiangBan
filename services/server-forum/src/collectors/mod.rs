//! 来源任务调度与草稿接入。所有网络运行在独立后台任务，仅生成待审核内容。
pub mod credentials;
pub mod model_sources;
pub mod news_sources;
pub mod repo_sources;
pub mod schema;
pub mod tool_sources;
use crate::{
    error::{AppError, AppResult},
    state::AppState,
};
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct CollectedItem {
    pub kind: String,
    pub key: String,
    pub data: Value,
    pub sort_order: i32,
    pub published_at: Option<DateTime<Utc>>,
    pub source_name: String,
    pub source_url: String,
}
pub fn title(source: &str) -> Option<&'static str> {
    match source {
        "artificial_analysis" => Some("Artificial Analysis 能力榜"),
        "openrouter" => Some("OpenRouter 调用量榜"),
        "github_repos" => Some("GitHub 开源热榜"),
        "github_tools" => Some("GitHub 工具版本"),
        "official_news" => Some("官方 AI 日报"),
        _ => None,
    }
}
pub fn credential_required(source: &str) -> bool { source != "official_news" }
pub fn check_source(source: &str) -> AppResult<()> {
    if title(source).is_none() {
        return Err(AppError::not_found("采集来源"));
    }
    Ok(())
}

/// 领取十分钟租约；运行上限四分钟，因此过期任务不会和接替者并行写入。
pub async fn start(s: AppState, source: String, scheduled: bool) -> AppResult<Uuid> {
    check_source(&source)?;
    let mut tx = s.pool.begin().await?;
    let row: Option<(Option<Vec<u8>>,bool,DateTime<Utc>,Option<DateTime<Utc>>,Option<Uuid>,Value)>=sqlx::query_as(
        "SELECT credential,enabled,next_run_at,lease_until,active_run,config FROM collection_sources WHERE id=$1 FOR UPDATE")
        .bind(&source).fetch_optional(&mut *tx).await?;
    let (sealed, enabled, next, lease, old_run, config) =
        row.ok_or_else(|| AppError::not_found("采集来源"))?;
    if lease.is_some_and(|t| t > Utc::now()) {
        return Err(AppError::Conflict("此来源已有采集任务运行中".into()));
    }
    if scheduled && (!enabled || next > Utc::now()) {
        return Err(AppError::Conflict("此来源尚未到执行时间".into()));
    }
    if source == "github_repos" {
        repo_sources::configured_names(&config)?;
    }
    if source == "github_tools" {
        let clean = tool_sources::sanitize_config(config.clone())?;
        if clean["repositories"].as_array().is_none_or(|v|v.is_empty()) {
            return Err(AppError::BadRequest("请先配置至少一个工具仓库".into()));
        }
    }
    if source == "official_news" {
        let config = news_sources::sanitize_config(config.clone())?;
        if config["feed_ids"].as_array().is_none_or(|v| v.is_empty()) {
            return Err(AppError::BadRequest("请先选择至少一个官方日报来源".into()));
        }
    }
    let secret = if credential_required(&source) { credentials::decrypt(
        &source,
        &sealed.ok_or_else(|| AppError::BadRequest("请先配置来源凭据".into()))?,
    )? } else { String::new() };
    let http = credentials::http_client()?;
    if let Some(old) = old_run {
        sqlx::query("UPDATE collection_runs SET status='failed',finished_at=now(),error='运行中断，租约已过期' WHERE id=$1 AND status='running'").bind(old).execute(&mut *tx).await?;
    }
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO collection_runs(id,source,status,config) VALUES($1,$2,'running',$3)")
        .bind(id)
        .bind(&source)
        .bind(&config)
        .execute(&mut *tx)
        .await?;
    sqlx::query("UPDATE collection_sources SET active_run=$2,lease_until=now()+interval '10 minutes',next_run_at=CASE WHEN interval_minutes=1440 THEN date_trunc('day',(now()-interval '4 hours') AT TIME ZONE 'UTC') AT TIME ZONE 'UTC'+interval '1 day 4 hours' ELSE now()+make_interval(mins=>interval_minutes) END WHERE id=$1")
        .bind(&source).bind(id).execute(&mut *tx).await?;
    tx.commit().await?;
    tokio::spawn(async move {
        let fetched = tokio::time::timeout(std::time::Duration::from_secs(240), async {
            if source == "github_repos" {
                repo_sources::collect(&s.pool, &secret, &http, &config, id).await
            } else if source == "github_tools" {
                tool_sources::collect(&http, &config, &secret).await
            } else if source == "official_news" {
                news_sources::collect(&http, &config).await
            } else {
                model_sources::collect(&source, &secret, &http).await
            }
        })
        .await;
        let result = match fetched {
            Ok(Ok(items)) => {
                if items.is_empty() && (source == "github_repos" || source == "official_news") {
                    Ok(0)
                } else {
                    crate::handlers::discover_editor::write_collected_batch(
                        &s.pool, &source, id, items,
                    )
                    .await
                    .map(|count| count as i32)
                }
            }
            // Adapter errors may include upstream strings; only a fixed category is persisted.
            Ok(Err(message)) => Err(AppError::BadRequest(if matches!(source.as_str(), "official_news" | "github_tools") { message } else {
                "来源采集失败，请检查凭据、额度和来源可用性".into()
            })),
            Err(_) => Err(AppError::BadRequest("来源采集超时".into())),
        };
        if let Err(_) = finish(&s.pool, &source, id, result).await {
            tracing::error!(source=%source,run_id=%id,"无法保存采集运行结果");
        }
    });
    Ok(id)
}
async fn finish(pool: &PgPool, source: &str, id: Uuid, result: AppResult<i32>) -> AppResult<()> {
    let (count, error) = match result {
        Ok(count) => (count, None),
        Err(AppError::Conflict(_)) => (0, Some("草稿已被人工修改或审核，请处理冲突后重试")),
        Err(AppError::BadRequest(ref message)) if matches!(source, "official_news" | "github_tools") => (0, Some(message.as_str())),
        Err(AppError::BadRequest(_)) => (
            0,
            Some("采集来源返回失败、超时或无效数据，请检查来源后重试"),
        ),
        Err(_) => (0, Some("采集存储失败，请稍后重试")),
    };
    let mut tx = pool.begin().await?;
    // 与领取/草稿落库保持相同锁序，过期接替与完成写入不会形成循环等待。
    sqlx::query("SELECT id FROM collection_sources WHERE id=$1 FOR UPDATE")
        .bind(source)
        .execute(&mut *tx)
        .await?;
    sqlx::query("UPDATE collection_runs SET status=$2,finished_at=now(),item_count=$3,error=$4 WHERE id=$1 AND status='running'")
        .bind(id).bind(if error.is_none(){"succeeded"}else{"failed"}).bind(count).bind(error).execute(&mut *tx).await?;
    sqlx::query("UPDATE collection_sources SET active_run=NULL,lease_until=NULL,last_error=$3,last_success_at=CASE WHEN $3::text IS NULL THEN now() ELSE last_success_at END WHERE id=$1 AND active_run=$2")
        .bind(source).bind(id).bind(error).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(())
}

pub fn spawn(state: AppState) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            let sources: Result<Vec<(String,)>,_>=sqlx::query_as("SELECT id FROM collection_sources WHERE enabled AND (credential IS NOT NULL OR id='official_news') AND next_run_at<=now() AND (lease_until IS NULL OR lease_until<=now()) ORDER BY id").fetch_all(&state.pool).await;
            match sources {
                Ok(sources) => {
                    for (source,) in sources {
                        if let Err(error) = start(state.clone(), source.clone(), true).await {
                            if !matches!(error, AppError::Conflict(_)) {
                                tracing::warn!(source=%source,"采集任务暂无法启动，请检查凭据与部署配置");
                            }
                        }
                    }
                }
                Err(_) => tracing::warn!("无法查询采集计划"),
            }
        }
    });
}
