use super::dto::{PexelsPhoto, PexelsVideo};
use crate::models::{
    wallpaper::{
        Author, DownloadUrls, MediaType, Orientation, Resolution, Stats, VideoMeta, VideoQuality,
    },
    WallpaperItem,
};

const LICENSE: &str = "Pexels License (Free for commercial and personal use)";
const MEDIUM_WIDTH: u32 = 1080;
const LARGE_WIDTH: u32 = 2560;

fn sized_rendition(original: &str, width: u32) -> String {
    let separator = if original.contains('?') { '&' } else { '?' };
    format!("{original}{separator}auto=compress&cs=tinysrgb&w={width}")
}

pub fn map_pexels_photo(photo: PexelsPhoto) -> WallpaperItem {
    let orientation = Orientation::from_dimensions(photo.width, photo.height);
    let resolution = Resolution::new(photo.width, photo.height);

    let alt = photo.alt.as_deref().map(str::trim).filter(|s| !s.is_empty());
    let title = match alt {
        Some(s) if s.chars().count() > 60 => {
            format!("{}...", s.chars().take(57).collect::<String>())
        }
        Some(s) => s.to_string(),
        None => format!("Wallpaper by {}", photo.photographer),
    };

    // Pexels' named sizes are crops bounded by height (`large` is only 650 px tall), so portrait
    // photos come out small. Asking the image server for a width instead gives a true rendition.
    let downloads = DownloadUrls {
        large: Some(sized_rendition(&photo.src.original, LARGE_WIDTH)),
        medium: Some(sized_rendition(&photo.src.original, MEDIUM_WIDTH)),
        small: Some(photo.src.medium),
        thumb: photo.src.small,
        original: photo.src.original,
    };
    let medium = downloads.medium.clone().unwrap_or_default();

    WallpaperItem {
        id: format!("pexels_{}", photo.id),
        provider: "pexels".to_string(),
        source_id: photo.id.to_string(),
        media_type: MediaType::Image,
        title,
        description: alt.map(str::to_string),
        tags: Vec::new(),
        resolution,
        orientation,
        dominant_color: photo.avg_color,
        preview_url: medium.clone(),
        regular_url: medium,
        downloads,
        video: None,
        author: Author {
            name: photo.photographer.clone(),
            username: photo.photographer,
            avatar_url: None,
            profile_url: photo.photographer_url,
        },
        stats: Stats::default(),
        source_page_url: photo.url,
        license: LICENSE.to_string(),
        status: "published".to_string(),
        source_type: "collected".to_string(),
        category: None,
        local_path: None,
        collected_at: Some(chrono::Utc::now()),
    }
}

fn quality_label(height: u32) -> &'static str {
    match height {
        h if h >= 2160 => "4K",
        h if h >= 1440 => "1440p",
        h if h >= 1080 => "1080p",
        h if h >= 720 => "720p",
        _ => "480p",
    }
}

pub fn map_pexels_video(video: PexelsVideo) -> WallpaperItem {
    // Orientation-agnostic: pick the longer side so portrait clips are ranked by their true size.
    let size = |f: &super::dto::PexelsVideoFile| f.width.unwrap_or(0).max(f.height.unwrap_or(0));

    let mut files: Vec<_> = video
        .video_files
        .iter()
        .filter(|f| f.file_type.as_deref().map_or(true, |t| t == "video/mp4"))
        .filter(|f| f.width.is_some() && f.height.is_some())
        .collect();
    files.sort_by_key(|f| std::cmp::Reverse(size(f)));

    let qualities: Vec<VideoQuality> = files
        .iter()
        .map(|f| {
            let (w, h) = (f.width.unwrap_or(0), f.height.unwrap_or(0));
            VideoQuality {
                quality: quality_label(w.min(h)).to_string(),
                url: f.link.clone(),
                width: w,
                height: h,
                size_bytes: None,
            }
        })
        .collect();

    let highest_res_url = qualities.first().map(|q| q.url.clone()).unwrap_or_default();
    let lowest_res_url = qualities
        .last()
        .map(|q| q.url.clone())
        .unwrap_or_else(|| highest_res_url.clone());

    let (width, height) = qualities
        .first()
        .map(|q| (q.width, q.height))
        .unwrap_or((video.width, video.height));

    let user_name = video.user.name;

    WallpaperItem {
        id: format!("pexels_v_{}", video.id),
        provider: "pexels".to_string(),
        source_id: video.id.to_string(),
        media_type: MediaType::Video,
        title: format!("Live Wallpaper by {user_name}"),
        description: Some(format!("Live wallpaper ({}s)", video.duration)),
        tags: Vec::new(),
        resolution: Resolution::new(width, height),
        orientation: Orientation::from_dimensions(width, height),
        dominant_color: None,
        preview_url: video.image.clone(),
        regular_url: video.image.clone(),
        downloads: DownloadUrls {
            original: highest_res_url.clone(),
            large: Some(highest_res_url),
            medium: None,
            small: None,
            thumb: video.image,
        },
        video: Some(VideoMeta {
            duration_secs: video.duration,
            preview_url: lowest_res_url,
            qualities,
        }),
        author: Author {
            name: user_name.clone(),
            username: user_name,
            avatar_url: None,
            profile_url: video.user.url,
        },
        stats: Stats::default(),
        source_page_url: video.url,
        license: LICENSE.to_string(),
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
    fn renditions_are_sized_by_width_not_by_the_height_bound_named_sizes() {
        let original = "https://images.pexels.com/photos/1/pexels-photo-1.jpeg";
        assert_eq!(
            sized_rendition(original, MEDIUM_WIDTH),
            "https://images.pexels.com/photos/1/pexels-photo-1.jpeg?auto=compress&cs=tinysrgb&w=1080"
        );
        assert!(sized_rendition("https://x/p.jpeg?a=1", LARGE_WIDTH).ends_with("?a=1&auto=compress&cs=tinysrgb&w=2560"));
    }
}
