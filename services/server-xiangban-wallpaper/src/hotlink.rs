//! Hotlink policy: may clients use a provider's original link directly, or must they get our own
//! stored copy?
//!
//! Policy `source`: clients try the provider's link first and use our copy only as a fallback.
//! Policy `copy`: clients get our copy (made on demand), as before.
//!
//! The per-provider defaults below are not arbitrary. Pixabay's API documentation says
//! "permanent hotlinking of images (using Pixabay URLs in your app) is not allowed ... please
//! download them to your server first. Videos may be embedded directly in your applications",
//! and its `webformatURL` links are only valid for 24 hours. So Pixabay images default to `copy`
//! and Pixabay videos to `source`. Pexels and Unsplash allow hotlinking their CDN links.
//!
//! Admins can override a provider per media type through the settings
//! `provider.<id>.hotlink.image` and `provider.<id>.hotlink.video` (value `source` or `copy`).
//! Items that are not provider-collected (custom or admin uploads) and the demo provider always
//! use `copy`, since there is no outside link to prefer.
use crate::{error::AppError, models::MediaType, models::WallpaperItem, store::Store};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Policy {
    Source,
    Copy,
}

impl Policy {
    pub fn as_str(self) -> &'static str {
        match self {
            Policy::Source => "source",
            Policy::Copy => "copy",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "source" => Some(Policy::Source),
            "copy" => Some(Policy::Copy),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Image,
    Video,
}

impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Image => "image",
            Kind::Video => "video",
        }
    }

    pub fn of(media_type: MediaType) -> Self {
        match media_type {
            MediaType::Video => Kind::Video,
            MediaType::Image | MediaType::LivePhoto => Kind::Image,
        }
    }
}

pub fn setting_key(provider: &str, kind: Kind) -> String {
    format!("provider.{provider}.hotlink.{}", kind.as_str())
}

/// The policy a provider has when nothing is configured.
pub fn default_policy(provider: &str, kind: Kind) -> Policy {
    match (provider, kind) {
        // Pixabay forbids permanent hotlinking of images and its image links expire after 24 h.
        ("pixabay", Kind::Image) => Policy::Copy,
        ("pixabay" | "pexels" | "unsplash" | "okslop" | "bing", _) => Policy::Source,
        // Unknown providers: keep serving our own copy, the safe choice.
        _ => Policy::Copy,
    }
}

/// The effective policy for one item. `configured` is the stored setting value, if any; values
/// other than `source` / `copy` are ignored.
pub fn resolve(provider: &str, source_type: &str, kind: Kind, configured: Option<&str>) -> Policy {
    if source_type != "collected" || provider == "demo" {
        return Policy::Copy;
    }
    configured
        .and_then(Policy::parse)
        .unwrap_or_else(|| default_policy(provider, kind))
}

/// The stored overrides, loaded once per request (one query) and then consulted per item.
#[derive(Clone, Debug, Default)]
pub struct Overrides(HashMap<String, String>);

impl Overrides {
    pub async fn load(store: &Store) -> Result<Self, AppError> {
        Ok(Self(store.settings_matching("provider.%.hotlink.%").await?))
    }

    pub fn configured(&self, provider: &str, kind: Kind) -> Option<&str> {
        self.0.get(&setting_key(provider, kind)).map(String::as_str)
    }

    /// The provider's own policy, ignoring the item-level rules (what the admin console edits).
    pub fn provider_policy(&self, provider: &str, kind: Kind) -> Policy {
        self.configured(provider, kind)
            .and_then(Policy::parse)
            .unwrap_or_else(|| default_policy(provider, kind))
    }

    pub fn policy(&self, item: &WallpaperItem) -> Policy {
        let kind = Kind::of(item.media_type);
        resolve(&item.provider, &item.source_type, kind, self.configured(&item.provider, kind))
    }

    #[cfg(test)]
    pub fn from_pairs(pairs: &[(&str, &str)]) -> Self {
        Self(pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_follow_each_providers_hotlink_rules() {
        assert_eq!(default_policy("pexels", Kind::Image), Policy::Source);
        assert_eq!(default_policy("pexels", Kind::Video), Policy::Source);
        assert_eq!(default_policy("pixabay", Kind::Image), Policy::Copy);
        assert_eq!(default_policy("pixabay", Kind::Video), Policy::Source);
        assert_eq!(default_policy("unsplash", Kind::Image), Policy::Source);
        assert_eq!(default_policy("unsplash", Kind::Video), Policy::Source);
        assert_eq!(default_policy("okslop", Kind::Image), Policy::Source);
        assert_eq!(default_policy("okslop", Kind::Video), Policy::Source);
        assert_eq!(default_policy("bing", Kind::Image), Policy::Source);
        assert_eq!(default_policy("bing", Kind::Video), Policy::Source);
        assert_eq!(default_policy("something-new", Kind::Image), Policy::Copy);
    }

    #[test]
    fn configured_value_beats_the_default() {
        assert_eq!(resolve("pixabay", "collected", Kind::Image, Some("source")), Policy::Source);
        assert_eq!(resolve("pexels", "collected", Kind::Video, Some("copy")), Policy::Copy);
        assert_eq!(resolve("pexels", "collected", Kind::Video, None), Policy::Source);
    }

    #[test]
    fn nonsense_settings_fall_back_to_the_default() {
        assert_eq!(resolve("pixabay", "collected", Kind::Image, Some("hotlink")), Policy::Copy);
        assert_eq!(resolve("pexels", "collected", Kind::Image, Some("")), Policy::Source);
    }

    #[test]
    fn uploads_and_demo_content_always_use_the_copy() {
        for source_type in ["custom", "upload", ""] {
            assert_eq!(resolve("pexels", source_type, Kind::Image, Some("source")), Policy::Copy);
        }
        assert_eq!(resolve("custom", "custom", Kind::Video, Some("source")), Policy::Copy);
        assert_eq!(resolve("demo", "collected", Kind::Image, Some("source")), Policy::Copy);
    }

    #[test]
    fn setting_keys_are_per_provider_and_media_type() {
        assert_eq!(setting_key("pixabay", Kind::Image), "provider.pixabay.hotlink.image");
        assert_eq!(setting_key("okslop", Kind::Video), "provider.okslop.hotlink.video");
    }

    #[test]
    fn overrides_are_looked_up_by_key() {
        let overrides = Overrides::from_pairs(&[("provider.pixabay.hotlink.image", "source")]);
        assert_eq!(overrides.provider_policy("pixabay", Kind::Image), Policy::Source);
        assert_eq!(overrides.provider_policy("pixabay", Kind::Video), Policy::Source);
        assert_eq!(overrides.provider_policy("pexels", Kind::Image), Policy::Source);
        assert_eq!(overrides.configured("pixabay", Kind::Video), None);
    }
}
