//! 后台定时任务：重算热度、清理没挂到帖子上的附件。

use crate::{handlers::attachments, models, state::AppState};
use std::time::Duration;

pub fn spawn(state: AppState) {
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_secs(600));
        loop {
            tick.tick().await;
            if let Err(e) = models::refresh_hot_scores(&state.pool, None).await {
                tracing::warn!("重算热度失败: {e}");
            }
            match attachments::purge_orphans(&state.pool, &state.config.upload_dir).await {
                Ok(n) if n > 0 => tracing::info!("清理了 {n} 个没有挂到帖子上的附件"),
                Ok(_) => {}
                Err(e) => tracing::warn!("清理附件失败: {e}"),
            }
        }
    });
}
