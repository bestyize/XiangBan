use async_trait::async_trait;
use std::sync::Arc;
use wallpaper_collect::{
    db::Database,
    error::AppError,
    models::{MediaType, Orientation, ProviderInfo, Resolution, SearchQuery, WallpaperItem},
    providers::{
        DemoProvider, PexelsProvider, PixabayProvider, ProviderRegistry, UnsplashProvider,
        WallpaperProvider,
    },
    runtime::Runtime,
    services::{SearchCache, WallpaperAggregator},
};

#[tokio::test]
async fn test_resolution_and_orientation() {
    let res_16_9 = Resolution::new(3840, 2160);
    assert_eq!(res_16_9.aspect_ratio, "16:9");
    assert_eq!(Orientation::from_dimensions(3840, 2160), Orientation::Landscape);

    let _res_portrait = Resolution::new(1080, 2400);
    assert_eq!(Orientation::from_dimensions(1080, 2400), Orientation::Portrait);

    let _res_square = Resolution::new(1000, 1000);
    assert_eq!(Orientation::from_dimensions(1000, 1000), Orientation::Squarish);
}

#[tokio::test]
async fn test_demo_provider_search() {
    let provider = DemoProvider::new();
    assert_eq!(provider.id(), "demo");
    assert!(provider.is_configured());

    // Search all images
    let query = SearchQuery {
        q: Some("cyberpunk".to_string()),
        category: None,
        media_type: Some(MediaType::Image),
        orientation: None,
        provider: None,
        page: Some(1),
        per_page: Some(10),
        sort: None,
    };

    let results = provider.search(&query).await.expect("search should succeed");
    assert!(!results.is_empty());
    assert!(results[0].title.to_lowercase().contains("cyberpunk") || results[0].tags.iter().any(|t| t.contains("cyberpunk")));
}

#[tokio::test]
async fn test_demo_provider_videos() {
    let provider = DemoProvider::new();

    // Query video wallpapers
    let query = SearchQuery {
        q: None,
        category: None,
        media_type: Some(MediaType::Video),
        orientation: None,
        provider: None,
        page: Some(1),
        per_page: Some(10),
        sort: None,
    };

    let videos = provider.videos(&query).await.expect("videos should succeed");
    assert!(!videos.is_empty());
    let first = &videos[0];
    assert_eq!(first.media_type, MediaType::Video);
    assert!(first.video.is_some());
    let meta = first.video.as_ref().unwrap();
    assert!(meta.duration_secs > 0);
    assert!(!meta.qualities.is_empty());
}

#[tokio::test]
async fn test_providers_decoupled_creation() {
    let runtime = Runtime::empty();

    let unsplash = UnsplashProvider::new(&runtime);
    assert_eq!(unsplash.id(), "unsplash");
    assert!(!unsplash.is_configured());
    assert_eq!(unsplash.supported_media_types(), vec![MediaType::Image]);

    let pixabay = PixabayProvider::new(&runtime);
    assert_eq!(pixabay.id(), "pixabay");
    assert!(!pixabay.is_configured());
    assert!(pixabay.supported_media_types().contains(&MediaType::Video));
    assert!(pixabay.supported_media_types().contains(&MediaType::Image));

    let pexels = PexelsProvider::new(&runtime);
    assert_eq!(pexels.id(), "pexels");
    assert!(!pexels.is_configured());
    assert!(pexels.supported_media_types().contains(&MediaType::Video));
    assert!(pexels.supported_media_types().contains(&MediaType::Image));

    let registry = ProviderRegistry::new(&runtime, true);
    let list = registry.list_info();
    assert_eq!(list.len(), 4);
}

#[tokio::test]
async fn test_database_in_memory_fallback_and_favorites() {
    let db = Database::init(None).await;
    assert!(!db.is_postgres_connected());

    let provider = DemoProvider::new();
    let query = SearchQuery {
        q: None,
        category: None,
        media_type: None,
        orientation: None,
        provider: None,
        page: Some(1),
        per_page: Some(2),
        sort: None,
    };
    let items = provider.search(&query).await.unwrap();
    assert!(!items.is_empty());

    let item = &items[0];

    // Add to favorite
    db.add_favorite(item).await.expect("add favorite should succeed");

    // List favorites
    let favs = db.list_favorites().await.expect("list favorites should succeed");
    assert!(favs.iter().any(|f| f.id == item.id));

    // Remove favorite
    let removed = db.remove_favorite(&item.id).await.expect("remove favorite should succeed");
    assert!(removed);

    let favs_after = db.list_favorites().await.unwrap();
    assert!(!favs_after.iter().any(|f| f.id == item.id));
}

/// Without PostgreSQL there is nowhere to keep results, so every search goes to the providers.
#[tokio::test]
async fn test_aggregator_without_database_does_not_cache() {
    let runtime = Runtime::empty();
    let registry = ProviderRegistry::new(&runtime, true);
    let db = Database::init(None).await;
    let cache = SearchCache::new(None, runtime);
    assert!(!cache.is_active());
    let aggregator = WallpaperAggregator::new(registry, db, cache);

    let query = SearchQuery {
        q: Some("space".to_string()),
        category: None,
        media_type: Some(MediaType::Image),
        orientation: None,
        provider: None,
        page: Some(1),
        per_page: Some(10),
        sort: None,
    };

    let first = aggregator.search(&query).await.expect("search should work");
    let second = aggregator.search(&query).await.expect("search should work");
    assert!(!first.items.is_empty());
    assert_eq!(second.items.len(), first.items.len());
    assert_ne!(second.providers, vec!["cache"]);
}

#[tokio::test]
async fn test_category_searches_by_its_keywords() {
    let runtime = Runtime::empty();
    let registry = ProviderRegistry::new(&runtime, true);
    let db = Database::init(None).await;
    let aggregator = WallpaperAggregator::new(registry, db, SearchCache::new(None, runtime));

    let by_category = SearchQuery {
        q: None,
        category: Some("space".to_string()),
        media_type: Some(MediaType::Image),
        orientation: None,
        provider: None,
        page: Some(1),
        per_page: Some(10),
        sort: None,
    };
    let result = aggregator.search(&by_category).await.expect("category search should work");
    assert!(!result.items.is_empty(), "the demo set has space wallpapers");

    let unknown = SearchQuery {
        category: Some("no-such-category".to_string()),
        ..by_category
    };
    assert!(aggregator.search(&unknown).await.is_err());
}

/// A provider that returns every demo wallpaper whatever the query asks for, like a real provider
/// whose endpoint has no orientation option.
struct IgnoresFilters;

#[async_trait]
impl WallpaperProvider for IgnoresFilters {
    fn id(&self) -> &'static str {
        "ignores"
    }
    fn name(&self) -> &'static str {
        "Ignores"
    }
    fn is_configured(&self) -> bool {
        true
    }
    fn supported_media_types(&self) -> Vec<MediaType> {
        vec![MediaType::Image]
    }
    fn provider_info(&self) -> ProviderInfo {
        DemoProvider::new().provider_info()
    }
    async fn search(&self, _query: &SearchQuery) -> Result<Vec<WallpaperItem>, AppError> {
        DemoProvider::new().curated(1, 100).await
    }
    async fn curated(&self, _page: u32, _per_page: u32) -> Result<Vec<WallpaperItem>, AppError> {
        DemoProvider::new().curated(1, 100).await
    }
    async fn videos(&self, _query: &SearchQuery) -> Result<Vec<WallpaperItem>, AppError> {
        Ok(Vec::new())
    }
    async fn detail(&self, _id: &str) -> Result<Option<WallpaperItem>, AppError> {
        Ok(None)
    }
}

fn orientation_query(orientation: Orientation, per_page: u32) -> SearchQuery {
    SearchQuery {
        q: None,
        category: None,
        media_type: Some(MediaType::Image),
        orientation: Some(orientation),
        provider: None,
        page: Some(1),
        per_page: Some(per_page),
        sort: None,
    }
}

#[tokio::test]
async fn test_orientation_filter_holds_even_when_a_provider_ignores_it() {
    let registry = ProviderRegistry::with_providers(vec![Arc::new(IgnoresFilters)]);
    let aggregator = WallpaperAggregator::new(
        registry,
        Database::init(None).await,
        SearchCache::new(None, Runtime::empty()),
    );

    let everything = aggregator.search(&orientation_query(Orientation::All, 50)).await.unwrap();
    let landscape_count = everything.items.iter().filter(|i| i.orientation == Orientation::Landscape).count();
    let portrait_count = everything.items.iter().filter(|i| i.orientation == Orientation::Portrait).count();
    assert!(landscape_count > 0 && portrait_count > 0, "the fixture needs both kinds to prove anything");

    for wanted in [Orientation::Landscape, Orientation::Portrait] {
        let result = aggregator.search(&orientation_query(wanted, 50)).await.unwrap();
        assert!(!result.items.is_empty(), "{wanted:?}");
        assert!(
            result.items.iter().all(|i| i.orientation == wanted),
            "{wanted:?} filter let through: {:?}",
            result.items.iter().map(|i| (&i.id, i.orientation)).collect::<Vec<_>>()
        );
    }
}

/// With a page size smaller than what the provider returned, the filter leaves a short page, but
/// the provider clearly has more, so the page must still say another one follows.
#[tokio::test]
async fn test_filtered_short_page_still_reports_more_pages() {
    let registry = ProviderRegistry::with_providers(vec![Arc::new(IgnoresFilters)]);
    let aggregator = WallpaperAggregator::new(
        registry,
        Database::init(None).await,
        SearchCache::new(None, Runtime::empty()),
    );

    let per_page = 2;
    let result = aggregator.search(&orientation_query(Orientation::Portrait, per_page)).await.unwrap();
    assert!(
        result.items.len() < per_page as usize,
        "the filter should leave fewer than a full page (got {})",
        result.items.len()
    );
    assert!(result.has_more, "a thinned-out page must not end the list");
}
