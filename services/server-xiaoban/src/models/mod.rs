pub mod category;
pub mod provider;
pub mod query;
pub mod wallpaper;

pub use provider::ProviderInfo;
pub use query::{SearchQuery, SearchResult};
#[allow(unused_imports)]
pub use wallpaper::{
    Author, CopyFile, DownloadUrls, LocalCopy, MediaType, Orientation, Resolution, Stats, VideoMeta, VideoQuality,
    WallpaperItem,
};
