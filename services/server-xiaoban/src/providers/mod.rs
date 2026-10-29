pub mod bing;
pub mod demo;
pub mod okslop;
pub mod pexels;
pub mod pixabay;
pub mod traits;
pub mod unsplash;

use std::sync::Arc;
pub use bing::BingProvider;
pub use demo::DemoProvider;
pub use okslop::OkslopProvider;
pub use pexels::PexelsProvider;
pub use pixabay::PixabayProvider;
pub use traits::WallpaperProvider;
pub use unsplash::UnsplashProvider;

use crate::{models::ProviderInfo, runtime::Runtime};

#[derive(Clone)]
pub struct ProviderRegistry {
    providers: Vec<Arc<dyn WallpaperProvider>>,
}

impl ProviderRegistry {
    pub fn new(runtime: &Runtime, enable_demo: bool) -> Self {
        let mut providers: Vec<Arc<dyn WallpaperProvider>> = Vec::new();

        // 1. Unsplash provider
        providers.push(Arc::new(UnsplashProvider::new(runtime)));

        // 2. Pixabay provider
        providers.push(Arc::new(PixabayProvider::new(runtime)));

        // 3. Pexels provider
        providers.push(Arc::new(PexelsProvider::new(runtime)));

        // 4. okslop provider
        providers.push(Arc::new(OkslopProvider::new(runtime)));

        // 5. Bing provider
        providers.push(Arc::new(BingProvider::new(runtime)));

        // 6. Demo / Built-in fallback provider
        if enable_demo {
            providers.push(Arc::new(DemoProvider::new()));
        }

        Self { providers }
    }

    /// A registry over an explicit set of providers, for tests.
    pub fn with_providers(providers: Vec<Arc<dyn WallpaperProvider>>) -> Self {
        Self { providers }
    }

    pub fn get(&self, id: &str) -> Option<Arc<dyn WallpaperProvider>> {
        self.providers.iter().find(|p| p.id() == id).cloned()
    }

    pub fn all(&self) -> &[Arc<dyn WallpaperProvider>] {
        &self.providers
    }

    pub fn list_info(&self) -> Vec<ProviderInfo> {
        self.providers.iter().map(|p| p.provider_info()).collect()
    }
}
