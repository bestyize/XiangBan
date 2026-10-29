use super::dto::BingImage;
use crate::models::{
    wallpaper::{Author, DownloadUrls, MediaType, Orientation, Resolution, Stats},
    WallpaperItem,
};
use chrono::Utc;

const BING_ORIGIN: &str = "https://www.bing.com";
const LICENSE: &str = "© Microsoft Bing (for personal wallpaper use only)";

/// Extracts a clean source_id from `urlbase` or `url`.
pub fn extract_source_id(image: &BingImage) -> String {
    if let Some(base) = &image.urlbase {
        let trimmed = base.trim();
        if let Some(id) = trimmed.strip_prefix("/th?id=") {
            return id.to_string();
        }
        if !trimmed.is_empty() {
            return trimmed.trim_start_matches('/').to_string();
        }
    }

    if let Some(pos) = image.url.find("id=") {
        let rest = &image.url[pos + 3..];
        let end = rest.find('&').unwrap_or(rest.len());
        let id_with_ext = &rest[..end];
        if let Some(stripped) = id_with_ext.strip_suffix("_1920x1080.jpg") {
            return stripped.to_string();
        }
        return id_with_ext.to_string();
    }

    if let Some(hsh) = &image.hsh {
        if !hsh.trim().is_empty() {
            return hsh.trim().to_string();
        }
    }

    image
        .startdate
        .clone()
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string())
}

/// Parses author name from Bing copyright text, which typically ends with `(© Author/Agency)`.
fn extract_author_name(copyright: &str) -> String {
    let text = copyright.trim();
    for open_marker in ["(©", "（©", "(c)", "（c）", "(C)"] {
        if let Some(start) = text.rfind(open_marker) {
            let after = &text[start + open_marker.len()..];
            let end = after.find([')', '）']).unwrap_or(after.len());
            let author = after[..end].trim();
            if !author.is_empty() {
                return author.to_string();
            }
        }
    }
    "Microsoft Bing".to_string()
}

/// Derives a concise title from image.title or from the description portion of copyright.
fn extract_title(image: &BingImage) -> String {
    if let Some(title) = &image.title {
        let trimmed = title.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    if let Some(copyright) = &image.copyright {
        let trimmed = copyright.trim();
        let end = trimmed.find(['(', '（']).unwrap_or(trimmed.len());
        let subject = trimmed[..end].trim();
        if !subject.is_empty() {
            return subject.to_string();
        }
    }

    "Bing Daily Wallpaper".to_string()
}

/// Extracts relevant tags from title and copyright text.
fn extract_tags(title: &str, copyright: Option<&str>) -> Vec<String> {
    let mut tags = vec!["bing".to_string(), "daily-wallpaper".to_string(), "4k".to_string()];

    let mut add_token = |token: &str| {
        let t = token.trim();
        if t.len() >= 2 && !tags.iter().any(|existing| existing.eq_ignore_ascii_case(t)) {
            tags.push(t.to_string());
        }
    };

    for part in title.split([' ', ',', '，', '、', '-', '·']) {
        add_token(part);
    }

    if let Some(cp) = copyright {
        let end = cp.find(['(', '（']).unwrap_or(cp.len());
        for part in cp[..end].split([' ', ',', '，', '、', '-', '·']) {
            add_token(part);
        }
    }

    tags
}

pub fn map_bing_image(image: BingImage) -> WallpaperItem {
    let source_id = extract_source_id(&image);
    let id = format!("bing_{source_id}");
    let title = extract_title(&image);
    let author_name = image
        .copyright
        .as_deref()
        .map(extract_author_name)
        .unwrap_or_else(|| "Microsoft Bing".to_string());
    let tags = extract_tags(&title, image.copyright.as_deref());

    let (original_url, large_url, medium_url, small_url, thumb_url, preview_url, regular_url) =
        if let Some(base) = &image.urlbase {
            let clean_base = if base.starts_with('/') {
                format!("{BING_ORIGIN}{base}")
            } else {
                format!("{BING_ORIGIN}/{base}")
            };
            (
                format!("{clean_base}_UHD.jpg&pid=hp"),
                Some(format!("{clean_base}_1920x1080.jpg&pid=hp")),
                Some(format!("{clean_base}_1366x768.jpg&pid=hp")),
                Some(format!("{clean_base}_800x480.jpg&pid=hp")),
                format!("{clean_base}_800x480.jpg&pid=hp"),
                format!("{clean_base}_800x480.jpg&pid=hp"),
                format!("{clean_base}_1920x1080.jpg&pid=hp"),
            )
        } else {
            let full_url = if image.url.starts_with('/') {
                format!("{BING_ORIGIN}{}", image.url)
            } else {
                image.url.clone()
            };
            (
                full_url.clone(),
                Some(full_url.clone()),
                Some(full_url.clone()),
                Some(full_url.clone()),
                full_url.clone(),
                full_url.clone(),
                full_url,
            )
        };

    let downloads = DownloadUrls {
        original: original_url,
        large: large_url,
        medium: medium_url,
        small: small_url,
        thumb: thumb_url,
    };

    let source_page_url = image
        .copyrightlink
        .as_deref()
        .map(|link| {
            if link.starts_with("http") {
                link.to_string()
            } else if link.starts_with('/') {
                format!("{BING_ORIGIN}{link}")
            } else {
                format!("{BING_ORIGIN}/{link}")
            }
        })
        .unwrap_or_else(|| BING_ORIGIN.to_string());

    WallpaperItem {
        id,
        provider: "bing".to_string(),
        source_id,
        media_type: MediaType::Image,
        title,
        description: image.copyright.clone(),
        tags,
        resolution: Resolution::new(3840, 2160),
        orientation: Orientation::Landscape,
        dominant_color: None,
        preview_url,
        regular_url,
        downloads,
        video: None,
        author: Author {
            name: author_name,
            username: "bing".to_string(),
            avatar_url: None,
            profile_url: Some(BING_ORIGIN.to_string()),
        },
        stats: Stats::default(),
        source_page_url,
        license: LICENSE.to_string(),
        status: "published".to_string(),
        source_type: "collected".to_string(),
        category: None,
        local_path: None,
        local_copy: None,
        collected_at: Some(Utc::now()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_real_bing_image_sample() {
        let json = r#"{
            "startdate": "20261009",
            "fullstartdate": "202610091600",
            "enddate": "20261010",
            "url": "/th?id=OHR.CormorantsFlight_ZH-CN0770863330_1920x1080.jpg&rf=LaDigue_1920x1080.jpg&pid=hp",
            "urlbase": "/th?id=OHR.CormorantsFlight_ZH-CN0770863330",
            "copyright": "蒙特雷湾上空的双冠鸬鹚，加利福尼亚州，美国 (© Hiroya Minakuchi/Minden Pictures)",
            "copyrightlink": "https://www.bing.com/search?q=test&form=hpcapt",
            "title": "迁飞路线上的生命",
            "hsh": "5334aa6c5bad613d8231ab1fb253585d"
        }"#;

        let img: BingImage = serde_json::from_str(json).unwrap();
        let item = map_bing_image(img);

        assert_eq!(item.provider, "bing");
        assert_eq!(item.source_id, "OHR.CormorantsFlight_ZH-CN0770863330");
        assert_eq!(item.id, "bing_OHR.CormorantsFlight_ZH-CN0770863330");
        assert_eq!(item.title, "迁飞路线上的生命");
        assert_eq!(item.author.name, "Hiroya Minakuchi/Minden Pictures");
        assert_eq!(item.orientation, Orientation::Landscape);
        assert_eq!(item.resolution.width, 3840);
        assert_eq!(item.resolution.height, 2160);
        assert_eq!(
            item.downloads.original,
            "https://www.bing.com/th?id=OHR.CormorantsFlight_ZH-CN0770863330_UHD.jpg&pid=hp"
        );
        assert_eq!(
            item.downloads.thumb,
            "https://www.bing.com/th?id=OHR.CormorantsFlight_ZH-CN0770863330_800x480.jpg&pid=hp"
        );
        assert!(item.tags.contains(&"bing".to_string()));
        assert!(item.tags.contains(&"4k".to_string()));
    }

    #[test]
    fn falls_back_to_copyright_when_title_is_missing() {
        let img = BingImage {
            startdate: Some("20261009".into()),
            fullstartdate: None,
            enddate: None,
            url: "/th?id=OHR.Test_1920x1080.jpg".into(),
            urlbase: Some("/th?id=OHR.Test".into()),
            copyright: Some("科西嘉岛的岩石前哨 (© Photographer)".into()),
            copyrightlink: None,
            title: None,
            hsh: None,
            wp: None,
        };

        let item = map_bing_image(img);
        assert_eq!(item.title, "科西嘉岛的岩石前哨");
        assert_eq!(item.author.name, "Photographer");
    }

    #[test]
    fn extracts_id_when_urlbase_is_none() {
        let img = BingImage {
            startdate: None,
            fullstartdate: None,
            enddate: None,
            url: "/th?id=OHR.Fallback_ZH-CN123456_1920x1080.jpg&pid=hp".into(),
            urlbase: None,
            copyright: None,
            copyrightlink: None,
            title: Some("Fallback Title".into()),
            hsh: None,
            wp: None,
        };

        let item = map_bing_image(img);
        assert_eq!(item.source_id, "OHR.Fallback_ZH-CN123456");
        assert_eq!(item.id, "bing_OHR.Fallback_ZH-CN123456");
    }
}
