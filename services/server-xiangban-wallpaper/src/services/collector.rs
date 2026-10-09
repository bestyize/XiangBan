use crate::{
    db::Database,
    error::AppError,
    models::{MediaType, Orientation, SearchQuery},
    providers::ProviderRegistry,
};
use serde::Serialize;
use std::{sync::Arc, time::Duration};
use tokio::time::sleep;
use tracing::{error, info};

#[derive(Debug, Clone, Serialize)]
pub struct SyncReport {
    pub total_collected: usize,
    pub keywords_processed: Vec<String>,
    pub providers_used: Vec<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone)]
pub struct WallpaperCollector {
    registry: ProviderRegistry,
    db: Database,
    sync_interval_mins: u64,
}

impl WallpaperCollector {
    pub fn new(registry: ProviderRegistry, db: Database, sync_interval_mins: u64) -> Self {
        Self {
            registry,
            db,
            sync_interval_mins,
        }
    }

    pub fn start_background_loop(self: Arc<Self>) {
        if self.sync_interval_mins == 0 {
            info!("Background wallpaper collector loop is disabled (interval = 0).");
            return;
        }

        tokio::spawn(async move {
            info!(
                "Starting background wallpaper collector loop with interval: {} minutes",
                self.sync_interval_mins
            );

            // Wait 10 seconds after server launch before first crawl
            sleep(Duration::from_secs(10)).await;

            loop {
                info!("Triggering scheduled wallpaper collection...");
                match self.run_sync().await {
                    Ok(report) => {
                        info!(
                            "Scheduled collection complete: collected {} items across {} keywords",
                            report.total_collected,
                            report.keywords_processed.len()
                        );
                    }
                    Err(e) => {
                        error!("Scheduled collection encountered error: {e}");
                    }
                }

                sleep(Duration::from_secs(self.sync_interval_mins * 60)).await;
            }
        });
    }

    pub async fn run_sync(&self) -> Result<SyncReport, AppError> {
        let keywords = vec![
            "4K",
            "nature",
            "cyberpunk",
            "space",
            "minimalist",
            "city",
            "anime",
            "ocean",
        ];

        let mut total_collected = 0;
        let mut providers_used = Vec::new();

        let active_providers: Vec<_> = self
            .registry
            .all()
            .iter()
            .filter(|p| p.is_configured())
            .cloned()
            .collect();

        for prov in &active_providers {
            if !providers_used.contains(&prov.name().to_string()) {
                providers_used.push(prov.name().to_string());
            }

            for &kw in &keywords {
                let query = SearchQuery {
                    q: Some(kw.to_string()),
                    category: None,
                    media_type: Some(MediaType::Image),
                    orientation: Some(Orientation::Landscape),
                    provider: Some(prov.id().to_string()),
                    page: Some(1),
                    per_page: Some(20),
                    sort: Some("popular".to_string()),
                };

                match prov.search(&query).await {
                    Ok(items) => {
                        let count = items.len();
                        if count > 0 {
                            let _ = self.db.upsert_wallpapers(&items).await;
                            total_collected += count;
                            let _ = self
                                .db
                                .record_sync_task(prov.id(), kw, count as i32, "success", None)
                                .await;
                        }
                    }
                    Err(e) => {
                        let err_str = e.to_string();
                        let _ = self
                            .db
                            .record_sync_task(prov.id(), kw, 0, "failed", Some(&err_str))
                            .await;
                    }
                }

                // If provider supports videos, collect a few live video wallpapers too
                if prov.supported_media_types().contains(&MediaType::Video) {
                    let v_query = SearchQuery {
                        q: Some(kw.to_string()),
                        category: None,
                        media_type: Some(MediaType::Video),
                        orientation: Some(Orientation::Landscape),
                        provider: Some(prov.id().to_string()),
                        page: Some(1),
                        per_page: Some(10),
                        sort: Some("popular".to_string()),
                    };

                    if let (Ok(videos),Some(pool)) = (prov.videos(&v_query).await,self.db.pg_pool()) {
                        let store=crate::store::Store::from_pool(pool);
                        // Under the `source` policy this keeps the provider's links and downloads
                        // nothing, so a sync round stays fast.
                        let overrides=crate::hotlink::Overrides::load(&store).await.unwrap_or_default();
                        for mut video in videos {
                            let policy=overrides.policy(&video);
                            match crate::desktop::prepare_with_policy(&store,&mut video,policy).await {
                                Ok(())=>{if self.db.upsert_wallpapers(&[video]).await.is_ok(){total_collected+=1;}},
                                Err(error)=>error!("Video materialisation failed for {}: {error}",video.id),
                            }
                        }
                    }
                }

                // Slight pause to be polite to external rate limits
                sleep(Duration::from_millis(500)).await;
            }
        }

        Ok(SyncReport {
            total_collected,
            keywords_processed: keywords.into_iter().map(|s| s.to_string()).collect(),
            providers_used,
            timestamp: chrono::Utc::now(),
        })
    }
}
