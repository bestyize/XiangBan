use super::dto::UnsplashPhoto;
use crate::models::{
    wallpaper::{Author, DownloadUrls, MediaType, Orientation, Resolution, Stats},
    WallpaperItem,
};

const LARGE_WIDTH: u32 = 2560;

/// A rendition of the original capped at `width`. `urls.full` is the full-size file, often several
/// megabytes, which makes the detail view sit on a low-resolution placeholder while it downloads.
fn sized_rendition(raw: &str, width: u32) -> String {
    let separator = if raw.contains('?') { '&' } else { '?' };
    format!("{raw}{separator}w={width}&q=85&fm=jpg&fit=max")
}

pub fn map_unsplash_photo(photo: UnsplashPhoto) -> WallpaperItem {
    let orientation = Orientation::from_dimensions(photo.width, photo.height);
    let resolution = Resolution::new(photo.width, photo.height);

    let title = photo
        .description
        .as_deref()
        .or(photo.alt_description.as_deref())
        .map(|s| {
            let trimmed = s.trim();
            if trimmed.len() > 60 {
                format!("{}...", &trimmed[..57])
            } else {
                trimmed.to_string()
            }
        })
        .unwrap_or_else(|| format!("Wallpaper by {}", photo.user.name));

    let tags = photo
        .tags
        .unwrap_or_default()
        .into_iter()
        .map(|t| t.title.to_lowercase())
        .collect();

    let avatar_url = photo.user.profile_image.and_then(|img| {
        img.large.or(img.medium).or(img.small)
    });

    let downloads = DownloadUrls {
        original: photo.urls.raw.clone(),
        large: Some(sized_rendition(&photo.urls.raw, LARGE_WIDTH)),
        medium: Some(photo.urls.regular.clone()),
        small: Some(photo.urls.small),
        thumb: photo.urls.thumb,
    };

    WallpaperItem {
        id: format!("unsplash_{}", photo.id),
        provider: "unsplash".to_string(),
        source_id: photo.id,
        media_type: MediaType::Image,
        title,
        description: photo.description.or(photo.alt_description),
        tags,
        resolution,
        orientation,
        dominant_color: photo.color,
        preview_url: photo.urls.regular,
        regular_url: downloads.medium.clone().unwrap_or_else(|| photo.urls.raw.clone()),
        downloads,
        video: None,
        author: Author {
            name: photo.user.name,
            username: photo.user.username.clone(),
            avatar_url,
            profile_url: Some(format!("https://unsplash.com/@{}", photo.user.username)),
        },
        stats: Stats {
            views: 0,
            likes: photo.likes.unwrap_or(0),
            downloads: 0,
        },
        source_page_url: photo.links.html,
        license: "Unsplash License (Free for commercial and personal use)".to_string(),
        status: "published".to_string(),
        source_type: "collected".to_string(),
        category: None,
        local_path: None,
        collected_at: Some(chrono::Utc::now()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rendition_url_caps_the_width_and_keeps_existing_query() {
        let raw = "https://images.unsplash.com/photo-1?ixid=abc&ixlib=rb-4.0.3";
        assert_eq!(
            sized_rendition(raw, 2560),
            "https://images.unsplash.com/photo-1?ixid=abc&ixlib=rb-4.0.3&w=2560&q=85&fm=jpg&fit=max"
        );
        assert_eq!(
            sized_rendition("https://images.unsplash.com/photo-1", 2560),
            "https://images.unsplash.com/photo-1?w=2560&q=85&fm=jpg&fit=max"
        );
    }
}
