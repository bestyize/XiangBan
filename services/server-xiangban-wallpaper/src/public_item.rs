//! How an item is shown to the public (search and detail JSON) once our own stored copy may exist.
//!
//! The database keeps the provider's links in the URL fields and our copy in `local_copy`.
//! - Policy `source`: the URL fields keep the provider's links and an optional `fallback` object
//!   carries the site-relative `/uploads/...` paths of our copy, for whichever differ.
//! - Policy `copy`: the URL fields hold our copy's paths when a copy exists, as the site always
//!   did for videos; there is no `fallback` because it would equal the primary.
//!
//! Rows written before `local_copy` existed already hold `/uploads/` paths and have no
//! `local_copy`; they pass through unchanged.
use crate::{
    hotlink::{Overrides, Policy},
    models::{
        wallpaper::VideoQuality, CopyFile, LocalCopy, MediaType, SearchResult, WallpaperItem,
    },
};
use serde::Serialize;

pub fn is_remote(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://") || url.starts_with("//")
}

/// The URLs our copy can stand in for. `None` means there is no copy for that field.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CopyUrls {
    pub preview_url: Option<String>,
    pub regular_url: Option<String>,
    pub original: Option<String>,
    pub large: Option<String>,
    pub medium: Option<String>,
    pub small: Option<String>,
    pub thumb: Option<String>,
}

/// Maps a copy onto the item's URL fields. Videos follow the layout the site has always used for
/// stored videos: the cover is the preview and thumbnail, the video file is the full-size link.
pub fn copy_urls(item: &WallpaperItem, copy: &LocalCopy) -> CopyUrls {
    let media = copy.media.as_ref().filter(|f| f.is_safe()).map(|f| f.path.clone());
    let cover = copy.cover.as_ref().filter(|f| f.is_safe()).map(|f| f.path.clone());
    let mut urls = CopyUrls::default();
    if let Some(media) = &media {
        urls.regular_url = Some(media.clone());
        urls.original = Some(media.clone());
        urls.large = Some(media.clone());
        urls.medium = Some(media.clone());
    }
    if item.media_type == MediaType::Video {
        if let Some(cover) = cover {
            urls.preview_url = Some(cover.clone());
            urls.small = Some(cover.clone());
            urls.thumb = Some(cover);
        }
    } else if let Some(media) = media {
        // An image copy is the only picture we have, so it also serves the grid.
        urls.preview_url = Some(media.clone());
        urls.small = Some(media.clone());
        urls.thumb = Some(media);
    }
    urls
}

fn set(field: &mut String, new: &Option<String>) {
    if let Some(new) = new {
        *field = new.clone();
    }
}

fn set_option(field: &mut Option<String>, new: &Option<String>) {
    if let Some(new) = new {
        *field = Some(new.clone());
    }
}

/// Whether any of the item's links points outside the site, i.e. is still a provider's link.
fn holds_remote_link(item: &WallpaperItem) -> bool {
    let downloads = &item.downloads;
    [&item.preview_url, &item.regular_url, &downloads.original, &downloads.thumb]
        .into_iter()
        .chain([&downloads.large, &downloads.medium, &downloads.small].into_iter().flatten())
        .any(|url| is_remote(url))
        || item.video.as_ref().is_some_and(|v| {
            is_remote(&v.preview_url) || v.qualities.iter().any(|q| is_remote(&q.url))
        })
}

/// Writes the copy's paths into the item. With `only_local` (collection time), an item that still
/// holds any remote provider link is left untouched, so collection never destroys a source link;
/// custom uploads and rows from before `local_copy` hold only local paths and are rewritten.
pub fn apply_copy(item: &mut WallpaperItem, copy: &LocalCopy, only_local: bool) {
    if only_local && holds_remote_link(item) {
        return;
    }
    let urls = copy_urls(item, copy);
    set(&mut item.preview_url, &urls.preview_url);
    set(&mut item.regular_url, &urls.regular_url);
    set(&mut item.downloads.original, &urls.original);
    set_option(&mut item.downloads.large, &urls.large);
    set_option(&mut item.downloads.medium, &urls.medium);
    set_option(&mut item.downloads.small, &urls.small);
    set(&mut item.downloads.thumb, &urls.thumb);
    let Some(media) = copy.media.as_ref().filter(|f| f.is_safe()) else {
        return;
    };
    let (width, height) = (item.resolution.width, item.resolution.height);
    if let Some(video) = &mut item.video {
        video.preview_url = media.path.clone();
        video.qualities = vec![VideoQuality {
            quality: format!("{height}p"),
            url: media.path.clone(),
            width,
            height,
            size_bytes: Some(media.bytes),
        }];
    }
}

pub fn copy_file(path: &str, sha256: &str, bytes: u64, mime: &str) -> CopyFile {
    CopyFile {
        path: path.to_string(),
        sha256: sha256.to_string(),
        bytes,
        mime: mime.to_string(),
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct FallbackDownloads {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub large: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub medium: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub small: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumb: Option<String>,
}

/// Our copy's paths, for clients whose attempt on the provider link failed.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Fallback {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regular_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub downloads: Option<FallbackDownloads>,
}

/// Keeps a copy path only when it differs from what the item already presents.
fn differing(copy: &Option<String>, primary: Option<&str>) -> Option<String> {
    copy.as_ref().filter(|c| Some(c.as_str()) != primary).cloned()
}

fn fallback_for(item: &WallpaperItem, copy: &LocalCopy) -> Option<Fallback> {
    let urls = copy_urls(item, copy);
    let downloads = FallbackDownloads {
        original: differing(&urls.original, Some(&item.downloads.original)),
        large: differing(&urls.large, item.downloads.large.as_deref()),
        medium: differing(&urls.medium, item.downloads.medium.as_deref()),
        small: differing(&urls.small, item.downloads.small.as_deref()),
        thumb: differing(&urls.thumb, Some(&item.downloads.thumb)),
    };
    let fallback = Fallback {
        preview_url: differing(&urls.preview_url, Some(&item.preview_url)),
        regular_url: differing(&urls.regular_url, Some(&item.regular_url)),
        downloads: (downloads != FallbackDownloads::default()).then_some(downloads),
    };
    (fallback != Fallback::default()).then_some(fallback)
}

#[derive(Debug, Clone, Serialize)]
pub struct PublicItem {
    #[serde(flatten)]
    pub item: WallpaperItem,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback: Option<Fallback>,
}

/// The public view of one item under its hotlink policy. `local_copy` itself is not exposed.
pub fn public_item(mut item: WallpaperItem, policy: Policy) -> PublicItem {
    let Some(copy) = item.local_copy.take().filter(|c| !c.is_empty()) else {
        return PublicItem { item, fallback: None };
    };
    match policy {
        Policy::Copy => {
            apply_copy(&mut item, &copy, false);
            PublicItem { item, fallback: None }
        }
        Policy::Source => {
            let fallback = fallback_for(&item, &copy);
            PublicItem { item, fallback }
        }
    }
}

/// Same fields and order as `SearchResult`, with items in their public view.
#[derive(Debug, Clone, Serialize)]
pub struct PublicSearchResult {
    pub items: Vec<PublicItem>,
    pub total: usize,
    pub page: u32,
    pub per_page: u32,
    pub has_more: bool,
    pub providers: Vec<String>,
    pub execution_time_ms: u64,
}

pub fn public_result(result: SearchResult, overrides: &Overrides) -> PublicSearchResult {
    PublicSearchResult {
        items: result
            .items
            .into_iter()
            .map(|item| {
                let policy = overrides.policy(&item);
                public_item(item, policy)
            })
            .collect(),
        total: result.total,
        page: result.page,
        per_page: result.per_page,
        has_more: result.has_more,
        providers: result.providers,
        execution_time_ms: result.execution_time_ms,
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::models::wallpaper::{Author, DownloadUrls, Resolution, Stats, VideoMeta};
    use serde_json::{json, Value};

    pub(crate) fn sample(provider: &str, media_type: MediaType) -> WallpaperItem {
        let video = media_type == MediaType::Video;
        let (original, thumb) = if video {
            ("https://cdn.example.com/v/large.mp4", "https://cdn.example.com/v/poster.jpg")
        } else {
            ("https://cdn.example.com/p/original.jpg", "https://cdn.example.com/p/small.jpg")
        };
        WallpaperItem {
            id: format!("{provider}_1"),
            provider: provider.into(),
            source_id: "1".into(),
            media_type,
            title: "Sample".into(),
            description: None,
            tags: vec![],
            resolution: Resolution::new(1920, 1080),
            orientation: crate::models::Orientation::Landscape,
            dominant_color: None,
            preview_url: if video { thumb.into() } else { "https://cdn.example.com/p/large.jpg".into() },
            regular_url: if video { thumb.into() } else { "https://cdn.example.com/p/large.jpg".into() },
            downloads: DownloadUrls {
                original: original.into(),
                large: Some(original.into()),
                medium: if video { None } else { Some("https://cdn.example.com/p/medium.jpg".into()) },
                small: if video { None } else { Some("https://cdn.example.com/p/webformat.jpg".into()) },
                thumb: thumb.into(),
            },
            video: video.then(|| VideoMeta {
                duration_secs: 10,
                preview_url: original.into(),
                qualities: vec![VideoQuality {
                    quality: "1080p".into(),
                    url: original.into(),
                    width: 1920,
                    height: 1080,
                    size_bytes: None,
                }],
            }),
            author: Author { name: "a".into(), username: "a".into(), avatar_url: None, profile_url: None },
            stats: Stats::default(),
            source_page_url: String::new(),
            license: String::new(),
            status: "published".into(),
            source_type: "collected".into(),
            category: None,
            local_path: None,
            local_copy: None,
            collected_at: None,
        }
    }

    const SHA: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn media_copy(ext: &str, mime: &str) -> LocalCopy {
        LocalCopy {
            media: Some(copy_file(&format!("/uploads/desktop-{SHA}.{ext}"), SHA, 1234, mime)),
            cover: None,
        }
    }

    fn json_of(public: &PublicItem) -> Value {
        serde_json::to_value(public).unwrap()
    }

    #[test]
    fn without_a_copy_the_json_has_no_fallback_and_keeps_every_field() {
        let item = sample("pexels", MediaType::Image);
        let plain = serde_json::to_value(&item).unwrap();
        let public = json_of(&public_item(item, Policy::Source));
        assert_eq!(public, plain, "no local_copy: identical to the stored item");
        assert!(public.get("fallback").is_none());
        assert!(public.get("local_copy").is_none());
    }

    #[test]
    fn source_policy_keeps_provider_links_and_adds_the_copy_as_fallback() {
        let mut item = sample("pexels", MediaType::Image);
        item.local_copy = Some(media_copy("jpg", "image/jpeg"));
        let path = format!("/uploads/desktop-{SHA}.jpg");
        let public = json_of(&public_item(item, Policy::Source));
        assert_eq!(public["preview_url"], "https://cdn.example.com/p/large.jpg");
        assert_eq!(public["downloads"]["original"], "https://cdn.example.com/p/original.jpg");
        assert_eq!(
            public["fallback"],
            json!({
                "preview_url": path,
                "regular_url": path,
                "downloads": {"original": path, "large": path, "medium": path, "small": path, "thumb": path}
            })
        );
        assert!(public.get("local_copy").is_none(), "the stored copy details stay private");
    }

    #[test]
    fn copy_policy_serves_the_copy_without_a_fallback() {
        let mut item = sample("pixabay", MediaType::Image);
        item.local_copy = Some(media_copy("jpg", "image/jpeg"));
        let path = format!("/uploads/desktop-{SHA}.jpg");
        let public = json_of(&public_item(item, Policy::Copy));
        assert!(public.get("fallback").is_none());
        assert_eq!(public["preview_url"], path);
        assert_eq!(public["regular_url"], path);
        assert_eq!(public["downloads"]["original"], path);
        assert_eq!(public["downloads"]["thumb"], path);
    }

    #[test]
    fn copy_policy_without_a_copy_keeps_the_provider_links() {
        let item = sample("pixabay", MediaType::Image);
        let plain = serde_json::to_value(&item).unwrap();
        assert_eq!(json_of(&public_item(item, Policy::Copy)), plain);
    }

    #[test]
    fn video_copy_uses_the_established_layout_and_leaves_no_fallback_under_copy_policy() {
        let mut item = sample("pixabay", MediaType::Video);
        let media = format!("/uploads/desktop-{SHA}.mp4");
        let cover = format!("/uploads/desktop-{}.jpg", "b".repeat(64));
        item.local_copy = Some(LocalCopy {
            media: Some(copy_file(&media, SHA, 1234, "video/mp4")),
            cover: Some(copy_file(&cover, &"b".repeat(64), 99, "image/jpeg")),
        });
        let as_source = json_of(&public_item(item.clone(), Policy::Source));
        assert_eq!(as_source["video"]["qualities"][0]["url"], "https://cdn.example.com/v/large.mp4");
        assert_eq!(as_source["fallback"]["preview_url"], cover);
        assert_eq!(as_source["fallback"]["regular_url"], media);
        assert_eq!(as_source["fallback"]["downloads"]["original"], media);
        assert_eq!(as_source["fallback"]["downloads"]["thumb"], cover);
        assert!(as_source["fallback"]["downloads"].get("medium").is_some(), "legacy layout fills medium");

        let as_copy = json_of(&public_item(item, Policy::Copy));
        assert!(as_copy.get("fallback").is_none());
        assert_eq!(as_copy["preview_url"], cover);
        assert_eq!(as_copy["regular_url"], media);
        assert_eq!(as_copy["video"]["preview_url"], media);
        assert_eq!(as_copy["video"]["qualities"][0]["url"], media);
        assert_eq!(as_copy["video"]["qualities"][0]["size_bytes"], 1234);
    }

    #[test]
    fn a_fallback_equal_to_the_primary_is_omitted() {
        // A legacy row already rewritten to /uploads, whose copy was recorded afterwards.
        let mut item = sample("pexels", MediaType::Image);
        let path = format!("/uploads/desktop-{SHA}.jpg");
        item.preview_url = path.clone();
        item.regular_url = path.clone();
        item.downloads = DownloadUrls {
            original: path.clone(),
            large: Some(path.clone()),
            medium: Some(path.clone()),
            small: Some(path.clone()),
            thumb: path,
        };
        item.local_copy = Some(media_copy("jpg", "image/jpeg"));
        assert!(json_of(&public_item(item, Policy::Source)).get("fallback").is_none());
    }

    #[test]
    fn only_the_differing_keys_appear_in_a_partial_fallback() {
        let mut item = sample("pexels", MediaType::Image);
        let path = format!("/uploads/desktop-{SHA}.jpg");
        item.downloads.thumb = path.clone();
        item.local_copy = Some(media_copy("jpg", "image/jpeg"));
        let public = json_of(&public_item(item, Policy::Source));
        assert!(public["fallback"]["downloads"].get("thumb").is_none());
        assert_eq!(public["fallback"]["downloads"]["original"], path);
    }

    #[test]
    fn unsafe_copy_paths_are_never_published() {
        let mut item = sample("pexels", MediaType::Image);
        for bad in ["//evil.example/x.jpg", "/uploads/../secret", "https://evil.example/x.jpg", "/other/x.jpg"] {
            item.local_copy = Some(LocalCopy {
                media: Some(copy_file(bad, SHA, 1, "image/jpeg")),
                cover: None,
            });
            assert!(json_of(&public_item(item.clone(), Policy::Source)).get("fallback").is_none(), "{bad}");
        }
    }

    #[test]
    fn collection_rewrites_only_fields_without_a_remote_link() {
        let copy = media_copy("mp4", "video/mp4");
        let mut provider_item = sample("pexels", MediaType::Video);
        let before = serde_json::to_value(&provider_item).unwrap();
        apply_copy(&mut provider_item, &copy, true);
        assert_eq!(serde_json::to_value(&provider_item).unwrap(), before, "provider links survive");

        let mut upload = sample("custom", MediaType::Video);
        let local = "/uploads/own.mp4".to_string();
        upload.downloads.original = local.clone();
        upload.downloads.large = Some(local.clone());
        upload.regular_url = local.clone();
        upload.preview_url = local.clone();
        upload.downloads.thumb = local.clone();
        if let Some(video) = &mut upload.video {
            video.preview_url = local.clone();
            video.qualities[0].url = local;
        }
        apply_copy(&mut upload, &copy, true);
        let media = format!("/uploads/desktop-{SHA}.mp4");
        assert_eq!(upload.regular_url, media);
        assert_eq!(upload.downloads.original, media);
        assert_eq!(upload.video.unwrap().qualities[0].url, media);
    }

    #[test]
    fn search_result_keeps_its_shape_and_applies_each_items_policy() {
        let mut pexels = sample("pexels", MediaType::Image);
        pexels.local_copy = Some(media_copy("jpg", "image/jpeg"));
        let mut pixabay = sample("pixabay", MediaType::Image);
        pixabay.id = "pixabay_1".into();
        pixabay.local_copy = Some(media_copy("jpg", "image/jpeg"));
        let result = SearchResult {
            items: vec![pexels, pixabay],
            total: 2,
            page: 1,
            per_page: 24,
            has_more: false,
            providers: vec!["local_db".into()],
            execution_time_ms: 3,
        };
        let value = serde_json::to_value(public_result(result, &Overrides::default())).unwrap();
        for key in ["items", "total", "page", "per_page", "has_more", "providers", "execution_time_ms"] {
            assert!(value.get(key).is_some(), "{key}");
        }
        assert!(value["items"][0]["fallback"].is_object(), "pexels defaults to source");
        assert!(value["items"][1].get("fallback").is_none(), "pixabay images default to copy");
        assert!(value["items"][1]["regular_url"].as_str().unwrap().starts_with("/uploads/"));
    }
}
