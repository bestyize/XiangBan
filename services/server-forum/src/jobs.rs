//! 后台定时任务：重算热度。

use crate::{models, state::AppState};
use std::time::Duration;

pub fn spawn(state: AppState) {
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_secs(600));
        loop {
            tick.tick().await;
            if let Err(e) = models::refresh_hot_scores(&state.pool, None).await {
                tracing::warn!("重算热度失败: {e}");
            }
        }
    });
}
