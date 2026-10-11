//! 后台定时任务：处理到期羊毛、重算热度。

use crate::{models, state::AppState};
use std::time::Duration;

pub fn spawn(state: AppState) {
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_secs(600));
        loop {
            tick.tick().await;
            match models::expire_deals(&state.pool).await {
                Ok(n) if n > 0 => tracing::info!("{n} 条羊毛到期，已标为失效"),
                Ok(_) => {}
                Err(e) => tracing::warn!("处理到期羊毛失败: {e}"),
            }
            if let Err(e) = models::refresh_hot_scores(&state.pool, None).await {
                tracing::warn!("重算热度失败: {e}");
            }
        }
    });
}
