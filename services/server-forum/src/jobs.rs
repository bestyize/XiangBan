//! 后台定时任务：重算热度、清理没挂到帖子上的附件。

use crate::{
    handlers::{attachments, help, images},
    models,
    state::AppState,
};
use std::time::Duration;

pub fn spawn(state: AppState) {
    let publisher = state.clone();
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_secs(30));
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            tick.tick().await;
            match crate::handlers::discover_editor::publish_due(&publisher.pool).await {
                Ok(n) if n > 0 => tracing::info!("发布了 {n} 条已审核的发现页内容"),
                Ok(_) => {},
                Err(e) => tracing::warn!("发布发现页内容失败: {e}"),
            }
        }
    });
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_secs(600));
        let mut round: u32 = 0;
        loop {
            tick.tick().await;
            round = round.wrapping_add(1);
            if let Err(e) = help::expire(&state.pool).await {
                tracing::warn!("退还到期悬赏失败: {e}");
            }
            if let Err(e) = models::refresh_hot_scores(&state.pool, None).await {
                tracing::warn!("重算热度失败: {e}");
            }
            match attachments::purge_orphans(&state.pool, &state.config.upload_dir).await {
                Ok(n) if n > 0 => tracing::info!("清理了 {n} 个没有挂到帖子上的附件"),
                Ok(_) => {}
                Err(e) => tracing::warn!("清理附件失败: {e}"),
            }
            // 已读超过 90 天的通知清掉，未读的一直留着。
            if round % 6 == 3 {
                if let Err(e) = sqlx::query("DELETE FROM notifications WHERE read_at < now() - interval '90 days'").execute(&state.pool).await {
                    tracing::warn!("清理通知失败: {e}");
                }
            }
            // 要在所有正文里搜图片 id，一小时做一次就够。
            if round % 6 == 1 {
                match images::purge_unused(&state.pool, &state.config.upload_dir).await {
                    Ok(n) if n > 0 => tracing::info!("清理了 {n} 张没有被正文引用的图片"),
                    Ok(_) => {}
                    Err(e) => tracing::warn!("清理图片失败: {e}"),
                }
            }
        }
    });
}
