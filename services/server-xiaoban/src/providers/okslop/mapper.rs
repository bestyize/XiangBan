use super::dto::OkslopPhoto;
use crate::models::{
    wallpaper::{Author, DownloadUrls, MediaType, Orientation, Resolution, Stats},
    WallpaperItem,
};

const TITLE_CHARS: usize = 60;

/// The licence line shown on every item. Taken from okslop's own front page ("AI-generated images you can
/// use commercially without attribution"); its full terms could not be read, see the README.
const LICENSE: &str = "AI-generated image from okslop.com (free for commercial use, no attribution required)";

fn https(value: &Option<String>) -> Option<String> {
    value.as_deref().map(str::trim).filter(|v| v.starts_with("https://")).map(str::to_string)
}

/// The first usable link among the candidates, so a size the site did not send is replaced by a neighbour.
fn pick(candidates: &[&Option<String>]) -> Option<String> {
    candidates.iter().find_map(|c| https(c))
}

/// Cut on a character boundary: titles here are AI prompts that may be in any language.
fn short_title(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.chars().count() <= TITLE_CHARS {
        return trimmed.to_string();
    }
    let cut: String = trimmed.chars().take(TITLE_CHARS - 3).collect();
    format!("{}...", cut.trim_end())
}

/// Layout graphics (posters, logos) are not wallpapers.
pub fn is_wallpaper(photo: &OkslopPhoto) -> bool {
    photo.is_design != Some(true)
}

pub fn map_okslop_photo(photo: OkslopPhoto) -> WallpaperItem {
    let orientation = Orientation::from_dimensions(photo.width, photo.height);
    let resolution = Resolution::new(photo.width, photo.height);
    let user = photo.user.unwrap_or_default();
    let author_name = if user.name.trim().is_empty() { user.username.clone() } else { user.name.clone() };

    let described = [photo.description.as_deref(), photo.alt_description.as_deref()]
        .into_iter()
        .flatten()
        .map(str::trim)
        .find(|s| !s.is_empty());
    let title = match described {
        Some(text) => short_title(text),
        None if !author_name.is_empty() => format!("Wallpaper by {author_name}"),
        None => "okslop wallpaper".to_string(),
    };

    let urls = &photo.urls;
    // The site's own order of sizes; the original comes last as a stand-in for anything it did not send.
    let original = pick(&[&urls.raw, &urls.full, &urls.regular, &urls.small, &urls.thumb]).unwrap_or_default();
    let regular = pick(&[&urls.regular, &urls.full, &urls.small, &urls.raw, &urls.thumb]).unwrap_or_else(|| original.clone());
    let downloads = DownloadUrls {
        original: original.clone(),
        large: pick(&[&urls.full, &urls.raw, &urls.regular]),
        medium: Some(regular.clone()),
        small: pick(&[&urls.small, &urls.regular, &urls.thumb]),
        thumb: pick(&[&urls.thumb, &urls.small, &urls.regular]).unwrap_or_else(|| original.clone()),
    };

    let source_page_url = photo
        .links
        .as_ref()
        .and_then(|l| https(&l.html))
        .unwrap_or_else(|| format!("https://okslop.com/photo/{}", photo.id));

    let mut tags: Vec<String> = photo.tags.iter().filter_map(|t| t.text()).collect();
    tags.insert(0, "ai-generated".to_string());
    tags.dedup();

    let avatar_url = user.profile_image.as_ref().and_then(|img| {
        [&img.large, &img.medium, &img.small].into_iter().find_map(|candidate| https(candidate))
    });
    let profile_url = https(&user.portfolio_url);

    WallpaperItem {
        id: format!("okslop_{}", photo.id),
        provider: "okslop".to_string(),
        source_id: photo.id,
        media_type: MediaType::Image,
        title,
        description: described.map(str::to_string),
        tags: tags,
        resolution,
        orientation,
        dominant_color: photo.color.filter(|c| !c.trim().is_empty()),
        preview_url: regular,
        regular_url: downloads.medium.clone().unwrap_or_default(),
        downloads,
        video: None,
        author: Author {
            name: author_name,
            username: user.username,
            avatar_url,
            profile_url,
        },
        stats: Stats {
            views: 0,
            likes: photo.likes.unwrap_or(0),
            downloads: 0,
        },
        source_page_url,
        license: LICENSE.to_string(),
        status: "published".to_string(),
        source_type: "collected".to_string(),
        category: None,
        local_path: None,
        local_copy: None,
        collected_at: Some(chrono::Utc::now()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::MediaType;

    /// Shaped after the fields of the official client library's `Photo` model (okslop 0.0.1, PyPI).
    /// It is NOT a captured live response: the live API refused every unauthenticated request.
    const PHOTO: &str = r##"{
        "id": "abc123", "created_at": "2026-01-02T03:04:05Z", "updated_at": "2026-01-02T03:04:05Z",
        "width": 1536, "height": 2688, "color": "#1a2b3c",
        "description": "A neon-lit street at night, rain on the asphalt", "alt_description": "neon street",
        "urls": {"raw": "https://okslop.com/img/abc123/raw", "full": "https://okslop.com/img/abc123/full",
                 "regular": "https://okslop.com/img/abc123/regular", "small": "https://okslop.com/img/abc123/small",
                 "thumb": "https://okslop.com/img/abc123/thumb"},
        "links": {"self": "https://api.okslop.com/api/v1/photos/abc123", "html": "https://okslop.com/photo/abc123",
                  "download": "https://api.okslop.com/api/v1/photos/abc123/download",
                  "download_location": "https://api.okslop.com/api/v1/photos/abc123/download"},
        "likes": 42,
        "user": {"id": "u1", "username": "slopper", "name": "Slop Er"},
        "blur_hash": "LEHV6nWB2yk8pyo0adR*.7kCMdnj", "downloads": 7, "views": 99
    }"##;

    fn photo(json: &str) -> OkslopPhoto {
        serde_json::from_str(json).expect("photo json")
    }

    #[test]
    fn maps_a_photo_keeping_the_providers_own_links() {
        let item = map_okslop_photo(photo(PHOTO));
        assert_eq!(item.id, "okslop_abc123");
        assert_eq!(item.provider, "okslop");
        assert_eq!(item.source_id, "abc123");
        assert!(matches!(item.media_type, MediaType::Image));
        assert_eq!(item.preview_url, "https://okslop.com/img/abc123/regular");
        assert_eq!(item.regular_url, "https://okslop.com/img/abc123/regular");
        assert_eq!(item.downloads.original, "https://okslop.com/img/abc123/raw");
        assert_eq!(item.downloads.large.as_deref(), Some("https://okslop.com/img/abc123/full"));
        assert_eq!(item.downloads.medium.as_deref(), Some("https://okslop.com/img/abc123/regular"));
        assert_eq!(item.downloads.small.as_deref(), Some("https://okslop.com/img/abc123/small"));
        assert_eq!(item.downloads.thumb, "https://okslop.com/img/abc123/thumb");
    }

    #[test]
    fn nothing_is_stored_locally_at_collection_time() {
        let item = map_okslop_photo(photo(PHOTO));
        assert!(item.local_path.is_none());
        assert!(item.local_copy.is_none());
        assert!(item.video.is_none());
        assert_eq!(item.source_type, "collected");
        assert_eq!(item.status, "published");
    }

    #[test]
    fn carries_size_colour_author_likes_and_the_page_link() {
        let item = map_okslop_photo(photo(PHOTO));
        assert_eq!((item.resolution.width, item.resolution.height), (1536, 2688));
        assert!(matches!(item.orientation, crate::models::Orientation::Portrait));
        assert_eq!(item.dominant_color.as_deref(), Some("#1a2b3c"));
        assert_eq!(item.author.name, "Slop Er");
        assert_eq!(item.author.username, "slopper");
        assert_eq!(item.stats.likes, 42);
        assert_eq!(item.source_page_url, "https://okslop.com/photo/abc123");
        assert_eq!(item.title, "A neon-lit street at night, rain on the asphalt");
    }

    #[test]
    fn says_in_the_licence_that_the_images_are_ai_generated() {
        let item = map_okslop_photo(photo(PHOTO));
        assert!(item.license.contains("AI"), "{}", item.license);
        assert!(item.tags.iter().any(|t| t == "ai-generated"), "{:?}", item.tags);
    }

    #[test]
    fn a_missing_optional_field_does_not_lose_the_photo() {
        let sparse = r##"{"id":"z9","width":1000,"height":500,
            "urls":{"raw":"https://okslop.com/img/z9/raw","full":"https://okslop.com/img/z9/full","regular":"https://okslop.com/img/z9/regular","small":"https://okslop.com/img/z9/small","thumb":"https://okslop.com/img/z9/thumb"},
            "links":{"html":"https://okslop.com/photo/z9"},"user":{"id":"u","username":"u","name":"U"}}"##;
        let item = map_okslop_photo(photo(sparse));
        assert_eq!(item.id, "okslop_z9");
        assert_eq!(item.stats.likes, 0);
        assert_eq!(item.dominant_color, None);
        assert_eq!(item.title, "Wallpaper by U");
    }

    #[test]
    fn a_missing_size_variant_falls_back_to_the_next_one() {
        let json = r##"{"id":"y","width":10,"height":10,
            "urls":{"raw":"https://okslop.com/img/y/raw","regular":"https://okslop.com/img/y/regular","thumb":"https://okslop.com/img/y/thumb"},
            "links":{"html":"https://okslop.com/photo/y"},"user":{"id":"u","username":"u","name":"U"}}"##;
        let item = map_okslop_photo(photo(json));
        assert_eq!(item.downloads.small.as_deref(), Some("https://okslop.com/img/y/regular"));
        assert_eq!(item.downloads.large.as_deref(), Some("https://okslop.com/img/y/raw"));
    }

    #[test]
    fn a_long_title_is_cut_on_a_character_boundary() {
        let long = "霓虹街道".repeat(30);
        let json = PHOTO.replace("A neon-lit street at night, rain on the asphalt", &long);
        let item = map_okslop_photo(photo(&json));
        assert!(item.title.ends_with("..."));
        assert!(item.title.chars().count() <= 63);
        let ascii = PHOTO.replace("A neon-lit street at night, rain on the asphalt", &"word ".repeat(30));
        assert!(map_okslop_photo(photo(&ascii)).title.chars().count() <= 63);
    }

    #[test]
    fn an_empty_description_uses_the_alt_text_and_then_the_author() {
        let no_desc = PHOTO.replace(r#""description": "A neon-lit street at night, rain on the asphalt""#, r#""description": "  ""#);
        assert_eq!(map_okslop_photo(photo(&no_desc)).title, "neon street");
    }

    #[test]
    fn only_https_links_are_accepted() {
        let plain = PHOTO.replace("https://okslop.com/img/abc123/regular", "http://okslop.com/img/abc123/regular");
        let item = map_okslop_photo(photo(&plain));
        assert!(item.preview_url.starts_with("https://"), "{}", item.preview_url);
    }

    #[test]
    fn layout_designs_are_not_collected_as_wallpapers() {
        let design = PHOTO.replace(r#""likes": 42,"#, r#""likes": 42, "is_design": true,"#);
        assert!(!is_wallpaper(&photo(&design)));
        assert!(is_wallpaper(&photo(PHOTO)));
        let null = PHOTO.replace(r#""likes": 42,"#, r#""likes": 42, "is_design": null,"#);
        assert!(is_wallpaper(&photo(&null)));
    }

    #[test]
    fn the_sites_tags_are_kept_whether_they_come_as_words_or_objects() {
        let words = PHOTO.replace(r#""likes": 42,"#, r#""likes": 42, "tags": ["Neon", " rain "],"#);
        let tags = map_okslop_photo(photo(&words)).tags;
        assert!(tags.contains(&"neon".to_string()) && tags.contains(&"rain".to_string()), "{tags:?}");
        let objects = PHOTO.replace(r#""likes": 42,"#, r#""likes": 42, "tags": [{"title":"Street"},{"name":"night"}],"#);
        let tags = map_okslop_photo(photo(&objects)).tags;
        assert!(tags.contains(&"street".to_string()) && tags.contains(&"night".to_string()), "{tags:?}");
        assert!(tags.contains(&"ai-generated".to_string()));
    }

    #[test]
    fn the_authors_picture_and_page_are_used() {
        let with_avatar = PHOTO.replace(
            r#""name": "Slop Er"}"#,
            r#""name": "Slop Er", "portfolio_url": "https://okslop.com/@slopper",
               "profile_image": {"small":"https://okslop.com/img/p/thumb","medium":"https://okslop.com/img/p/thumb","large":"https://okslop.com/img/p/large"}}"#,
        );
        let item = map_okslop_photo(photo(&with_avatar));
        assert_eq!(item.author.avatar_url.as_deref(), Some("https://okslop.com/img/p/large"));
        assert_eq!(item.author.profile_url.as_deref(), Some("https://okslop.com/@slopper"));
    }

    /// Captured from `GET /api/v1/search/photos?query=mountain&per_page=2` on 2026-10-09 without an API key.
    const LIVE: &str = include_str!("../../../tests/fixtures/okslop_search.json");

    #[test]
    fn the_real_response_parses_and_maps_to_usable_items() {
        let page: super::super::dto::OkslopPage = serde_json::from_str(LIVE).expect("live response");
        assert_eq!(page.results.len(), 2);
        assert_eq!(page.total, Some(200));
        for raw in page.results {
            assert!(is_wallpaper(&raw));
            let item = map_okslop_photo(raw);
            assert!(item.id.starts_with("okslop_"));
            assert!(item.preview_url.starts_with("https://"), "{}", item.preview_url);
            assert!(item.downloads.thumb.starts_with("https://"));
            assert!(item.downloads.original.starts_with("https://okslop.com/img/"), "{}", item.downloads.original);
            assert!(item.source_page_url.starts_with("https://okslop.com/image/"), "{}", item.source_page_url);
            assert!(item.resolution.width > 0 && item.resolution.height > 0);
            assert!(!item.title.is_empty());
            assert!(!item.author.name.is_empty());
            assert!(item.author.avatar_url.as_deref().is_some_and(|u| u.starts_with("https://")));
            assert!(item.tags.contains(&"ai-generated".to_string()));
        }
    }
}

