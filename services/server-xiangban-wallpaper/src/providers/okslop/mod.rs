pub mod client;
pub mod dto;
pub mod mapper;

use crate::{
    error::AppError,
    models::{MediaType, Orientation, ProviderInfo, SearchQuery, WallpaperItem},
    providers::traits::WallpaperProvider,
    runtime::Runtime,
};
use async_trait::async_trait;
use client::{page_plan, OkslopClient, PagePlan};
use mapper::{is_wallpaper, map_okslop_photo};

pub struct OkslopProvider {
    client: OkslopClient,
}

impl OkslopProvider {
    pub fn new(runtime: &Runtime) -> Self {
        Self { client: OkslopClient::new(runtime.upstream("okslop")) }
    }
}

impl OkslopProvider {
    /// Fetches the upstream pages of `plan` one after another (the free tier allows 30 requests a minute, so no bursts),
    /// stops early when the library runs out, and cuts out exactly our page.
    async fn stitched<F, Fut>(&self, plan: &PagePlan, fetch: F) -> Result<Vec<WallpaperItem>, AppError>
    where
        F: Fn(u32) -> Fut,
        Fut: std::future::Future<Output = Result<Vec<dto::OkslopPhoto>, AppError>>,
    {
        let mut photos = Vec::new();
        for upstream_page in plan.first..plan.first + plan.count {
            let got = fetch(upstream_page).await?;
            let short = got.len() < client::UPSTREAM_PAGE_SIZE as usize;
            photos.extend(got);
            if short {
                break;
            }
        }
        Ok(photos
            .into_iter()
            .skip(plan.skip)
            .take(plan.take)
            .filter(is_wallpaper)
            .map(map_okslop_photo)
            .collect())
    }
}

#[async_trait]
impl WallpaperProvider for OkslopProvider {
    fn id(&self) -> &'static str {
        "okslop"
    }

    fn name(&self) -> &'static str {
        "okslop"
    }

    fn is_configured(&self) -> bool {
        self.client.is_configured()
    }

    fn supported_media_types(&self) -> Vec<MediaType> {
        vec![MediaType::Image]
    }

    fn provider_info(&self) -> ProviderInfo {
        let configured = self.is_configured();
        ProviderInfo {
            id: self.id().to_string(),
            name: self.name().to_string(),
            description: "A library of AI-generated images that can be used commercially without attribution.".to_string(),
            is_configured: configured,
            supported_media_types: self.supported_media_types(),
            official_url: "https://okslop.com".to_string(),
            status: if configured { "ready".to_string() } else { "missing_api_key".to_string() },
        }
    }

    async fn search(&self, query: &SearchQuery) -> Result<Vec<WallpaperItem>, AppError> {
        if !self.is_configured() {
            return Ok(Vec::new());
        }
        let q = query.get_q();
        let orientation = query.get_orientation();
        let plan = page_plan(query.get_page(), query.get_per_page());
        // The plain list has no orientation option, so a filter has to go through search.
        let use_list = q.is_empty() && orientation == Orientation::All;
        let keyword = if q.is_empty() { "wallpaper" } else { q };
        self.stitched(&plan, |upstream_page| async move {
            if use_list {
                self.client.list_page(upstream_page).await
            } else {
                self.client.search_page(keyword, upstream_page, &orientation).await
            }
        })
        .await
    }

    async fn curated(&self, page: u32, per_page: u32) -> Result<Vec<WallpaperItem>, AppError> {
        if !self.is_configured() {
            return Ok(Vec::new());
        }
        let plan = page_plan(page, per_page);
        self.stitched(&plan, |upstream_page| async move { self.client.list_page(upstream_page).await }).await
    }

    async fn videos(&self, _query: &SearchQuery) -> Result<Vec<WallpaperItem>, AppError> {
        // okslop only offers images.
        Ok(Vec::new())
    }

    async fn detail(&self, id: &str) -> Result<Option<WallpaperItem>, AppError> {
        if !self.is_configured() {
            return Ok(None);
        }
        Ok(self.client.detail(id).await?.map(map_okslop_photo))
    }
}
