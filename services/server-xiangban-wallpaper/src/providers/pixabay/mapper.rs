use super::dto::{PixabayImageHit, PixabayVideoHit};
use crate::models::{
    wallpaper::{
        Author, DownloadUrls, MediaType, Orientation, Resolution, Stats, VideoMeta, VideoQuality,
    },
    WallpaperItem,
};

pub fn map_pixabay_image(hit: PixabayImageHit) -> WallpaperItem {
    let orientation = Orientation::from_dimensions(hit.image_width, hit.image_height);
    let resolution = Resolution::new(hit.image_width, hit.image_height);

    let tags: Vec<String> = hit
        .tags
        .split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();

    let title = if let Some(first_tag) = tags.first() {
        let mut chars = first_tag.chars();
        match chars.next() {
            None => "Wallpaper".to_string(),
            Some(f) => format!("{} Wallpaper", f.to_uppercase().collect::<String>() + chars.as_str()),
        }
    } else {
        "Pixabay Wallpaper".to_string()
    };

    let downloads = DownloadUrls {
        original: hit.image_url.clone().unwrap_or_else(|| hit.large_image_url.clone()),
        large: hit.full_hd_url.clone().or_else(|| Some(hit.large_image_url.clone())),
        medium: Some(hit.large_image_url.clone()),
        small: Some(hit.webformat_url.clone()),
        thumb: hit.preview_url.clone(),
    };

    WallpaperItem {
        id: format!("pixabay_{}", hit.id),
        provider: "pixabay".to_string(),
        source_id: hit.id.to_string(),
        media_type: MediaType::Image,
        title,
        description: Some(format!("Tags: {}", hit.tags)),
        tags,
        resolution,
        orientation,
        dominant_color: None,
        preview_url: hit.large_image_url.clone(),
        regular_url: hit.large_image_url,
        downloads,
        video: None,
        author: Author {
            name: hit.user.clone(),
            username: hit.user.clone(),
            avatar_url: hit.user_image_url,
            profile_url: Some(format!("https://pixabay.com/users/{}-{}", hit.user, hit.user_id)),
        },
        stats: Stats {
            views: hit.views.unwrap_or(0),
            likes: hit.likes.unwrap_or(0),
            downloads: hit.downloads.unwrap_or(0),
        },
        source_page_url: hit.page_url,
        license: "Pixabay Content License (Free for commercial use)".to_string(),
        status: "published".to_string(),
        source_type: "collected".to_string(),
        category: None,
        local_path: None,
        collected_at: Some(chrono::Utc::now()),
    }
}

pub fn map_pixabay_video(hit: PixabayVideoHit) -> WallpaperItem {
    let mut qualities = Vec::new();
    let mut max_width = 1920;
    let mut max_height = 1080;

    if let Some(l) = &hit.videos.large {
        qualities.push(VideoQuality {
            quality: if l.width >= 3840 { "4K".to_string() } else { "1080p".to_string() },
            url: l.url.clone(),
            width: l.width,
            height: l.height,
            size_bytes: l.size,
        });
        max_width = l.width;
        max_height = l.height;
    }

    if let Some(m) = &hit.videos.medium {
        qualities.push(VideoQuality {
            quality: if m.width >= 1920 { "1080p".to_string() } else { "720p".to_string() },
            url: m.url.clone(),
            width: m.width,
            height: m.height,
            size_bytes: m.size,
        });
        if hit.videos.large.is_none() {
            max_width = m.width;
            max_height = m.height;
        }
    }

    if let Some(s) = &hit.videos.small {
        qualities.push(VideoQuality {
            quality: "720p".to_string(),
            url: s.url.clone(),
            width: s.width,
            height: s.height,
            size_bytes: s.size,
        });
    }

    if let Some(t) = &hit.videos.tiny {
        qualities.push(VideoQuality {
            quality: "480p".to_string(),
            url: t.url.clone(),
            width: t.width,
            height: t.height,
            size_bytes: t.size,
        });
    }

    let orientation = Orientation::from_dimensions(max_width, max_height);
    let resolution = Resolution::new(max_width, max_height);

    let tags: Vec<String> = hit
        .tags
        .split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();

    let title = if let Some(first_tag) = tags.first() {
        let mut chars = first_tag.chars();
        match chars.next() {
            None => "Live Video Wallpaper".to_string(),
            Some(f) => format!("{} Live Wallpaper", f.to_uppercase().collect::<String>() + chars.as_str()),
        }
    } else {
        "Pixabay Live Wallpaper".to_string()
    };

    // Poster preview image
    let preview_image_url = if let Some(pic_id) = &hit.picture_id {
        format!("https://i.vimeocdn.com/video/{}_640x360.jpg", pic_id)
    } else {
        String::new()
    };

    let highest_res_url = qualities
        .first()
        .map(|q| q.url.clone())
        .unwrap_or_default();

    let video_meta = VideoMeta {
        duration_secs: hit.duration,
        preview_url: qualities.last().map(|q| q.url.clone()).unwrap_or_else(|| highest_res_url.clone()),
        qualities,
    };

    WallpaperItem {
        id: format!("pixabay_v_{}", hit.id),
        provider: "pixabay".to_string(),
        source_id: hit.id.to_string(),
        media_type: MediaType::Video,
        title,
        description: Some(format!("Live wallpaper ({}s). Tags: {}", hit.duration, hit.tags)),
        tags,
        resolution,
        orientation,
        dominant_color: None,
        preview_url: preview_image_url.clone(),
        regular_url: preview_image_url.clone(),
        downloads: DownloadUrls {
            original: highest_res_url.clone(),
            large: Some(highest_res_url),
            medium: None,
            small: None,
            thumb: preview_image_url,
        },
        video: Some(video_meta),
        author: Author {
            name: hit.user.clone(),
            username: hit.user.clone(),
            avatar_url: hit.user_image_url,
            profile_url: Some(format!("https://pixabay.com/users/{}-{}", hit.user, hit.user_id)),
        },
        stats: Stats {
            views: hit.views.unwrap_or(0),
            likes: hit.likes.unwrap_or(0),
            downloads: hit.downloads.unwrap_or(0),
        },
        source_page_url: hit.page_url,
        license: "Pixabay Content License (Free for commercial use)".to_string(),
        status: "published".to_string(),
        source_type: "collected".to_string(),
        category: None,
        local_path: None,
        collected_at: Some(chrono::Utc::now()),
    }
}
