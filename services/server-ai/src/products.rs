//! 产品、档位与媒体能力：客户端只认档位名和能力名，真实模型只存在这里。

use crate::error::AppError;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{FromRow, PgPool};

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Product {
    pub product_id: String,
    pub name: String,
    pub client_ids: Vec<String>,
    pub trial_quota: i64,
    pub trial_days: i32,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Tier {
    pub reasoning_efforts: Option<Vec<String>>,
    pub product_id: String,
    pub tier: String,
    pub display_name: String,
    pub description: String,
    pub model: String,
    pub requires_paid: bool,
    pub is_default: bool,
    pub sort_order: i32,
    pub enabled: bool,
}

const PRODUCT_COLUMNS: &str = "product_id, name, client_ids, trial_quota, trial_days, enabled, created_at, updated_at";
const TIER_COLUMNS: &str = "product_id, tier, display_name, description, model, requires_paid, is_default, sort_order, enabled, reasoning_efforts";

/// 令牌 `aud` 属于哪个已启用的产品；一个客户端只能属于一个产品。
pub async fn for_client(pool: &PgPool, client_id: &str) -> Result<Option<Product>, AppError> {
    Ok(sqlx::query_as(&format!("SELECT {PRODUCT_COLUMNS} FROM products WHERE enabled AND $1 = ANY(client_ids)"))
        .bind(client_id)
        .fetch_optional(pool)
        .await?)
}

pub async fn get(pool: &PgPool, product_id: &str) -> Result<Option<Product>, AppError> {
    Ok(sqlx::query_as(&format!("SELECT {PRODUCT_COLUMNS} FROM products WHERE product_id = $1"))
        .bind(product_id)
        .fetch_optional(pool)
        .await?)
}

pub async fn list(pool: &PgPool) -> Result<Vec<Product>, AppError> {
    Ok(sqlx::query_as(&format!("SELECT {PRODUCT_COLUMNS} FROM products ORDER BY created_at, product_id"))
        .fetch_all(pool)
        .await?)
}

pub async fn tiers(pool: &PgPool, product_id: &str, only_enabled: bool) -> Result<Vec<Tier>, AppError> {
    Ok(sqlx::query_as(&format!(
        "SELECT {TIER_COLUMNS} FROM product_tiers WHERE product_id = $1 AND (NOT $2 OR enabled) ORDER BY sort_order, tier"
    ))
    .bind(product_id)
    .bind(only_enabled)
    .fetch_all(pool)
    .await?)
}

/// 请求里的 `model` 填档位名；不填则用默认档，没有默认档时用排序第一的启用档位。
pub fn resolve_tier<'a>(tiers: &'a [Tier], requested: Option<&str>) -> Option<&'a Tier> {
    let enabled = || tiers.iter().filter(|t| t.enabled);
    match requested.map(str::trim).filter(|s| !s.is_empty()) {
        Some(name) => enabled().find(|t| t.tier == name),
        None => enabled().find(|t| t.is_default).or_else(|| enabled().next()),
    }
}

/// 对话之外的四种能力。每个产品各配一个模型，在后台填写。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Capability {
    Speech,
    Image,
    Transcribe,
    Realtime,
}

impl Capability {
    pub const ALL: [Capability; 4] = [Capability::Speech, Capability::Image, Capability::Transcribe, Capability::Realtime];

    pub fn as_str(self) -> &'static str {
        match self {
            Capability::Speech => "speech",
            Capability::Image => "image",
            Capability::Transcribe => "transcribe",
            Capability::Realtime => "realtime",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|c| c.as_str() == value)
    }

    /// 给用户看的名字，用在「暂未开放」之类的提示里。
    pub fn label(self) -> &'static str {
        match self {
            Capability::Speech => "语音",
            Capability::Image => "图片生成",
            Capability::Transcribe => "语音识别",
            Capability::Realtime => "实时语音",
        }
    }
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct MediaSetting {
    pub product_id: String,
    pub capability: String,
    pub model: String,
    pub enabled: bool,
    pub requires_paid: bool,
    pub voices: Vec<String>,
    pub sizes: Vec<String>,
    pub max_secs: i32,
    pub updated_at: DateTime<Utc>,
}

const MEDIA_COLUMNS: &str = "product_id, capability, model, enabled, requires_paid, voices, sizes, max_secs, updated_at";

/// 实时语音单次对话的时长范围（秒）。
pub const REALTIME_SECS: std::ops::RangeInclusive<i32> = 60..=4 * 3600;

pub async fn media_settings(pool: &PgPool, product_id: &str) -> Result<Vec<MediaSetting>, AppError> {
    Ok(sqlx::query_as(&format!("SELECT {MEDIA_COLUMNS} FROM product_media WHERE product_id = $1 ORDER BY capability"))
        .bind(product_id)
        .fetch_all(pool)
        .await?)
}

/// 这个产品已开放的某项能力；没配置或已停用都算没开放。
pub async fn media(pool: &PgPool, product_id: &str, capability: Capability) -> Result<Option<MediaSetting>, AppError> {
    Ok(sqlx::query_as(&format!("SELECT {MEDIA_COLUMNS} FROM product_media WHERE product_id = $1 AND capability = $2 AND enabled"))
        .bind(product_id)
        .bind(capability.as_str())
        .fetch_optional(pool)
        .await?)
}

/// 音色名各家不同（`alloy`、`zh_female_shuangkuaisisi_moon_bigtts`、`longxiaochun`），只挡住明显填错的。
pub fn valid_voice(value: &str) -> bool {
    (1..=80).contains(&value.len()) && value.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

/// `1024x1024` 这样的尺寸，或交给模型决定的 `auto`。
pub fn valid_size(value: &str) -> bool {
    if value == "auto" {
        return true;
    }
    let Some((w, h)) = value.split_once('x') else { return false };
    [w, h].iter().all(|n| (2..=5).contains(&n.len()) && n.chars().all(|c| c.is_ascii_digit()))
}

pub fn valid_slug(value: &str) -> bool {
    (1..=40).contains(&value.len())
        && value.chars().next().is_some_and(|c| c.is_ascii_lowercase())
        && value.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '-' | '_'))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tier(name: &str, is_default: bool, enabled: bool) -> Tier {
        Tier {
            reasoning_efforts: None,
            product_id: "p".into(),
            tier: name.into(),
            display_name: name.into(),
            description: String::new(),
            model: format!("model-{name}"),
            requires_paid: false,
            is_default,
            sort_order: 0,
            enabled,
        }
    }

    #[test]
    fn requested_tier_must_exist_and_be_enabled() {
        let tiers = [tier("standard", true, true), tier("premium", false, false)];
        assert_eq!(resolve_tier(&tiers, Some("standard")).unwrap().tier, "standard");
        assert!(resolve_tier(&tiers, Some("premium")).is_none());
        assert!(resolve_tier(&tiers, Some("gpt-4")).is_none());
    }

    #[test]
    fn missing_tier_falls_back_to_default_then_first_enabled() {
        let tiers = [tier("a", false, true), tier("b", true, true)];
        assert_eq!(resolve_tier(&tiers, None).unwrap().tier, "b");
        assert_eq!(resolve_tier(&tiers, Some("  ")).unwrap().tier, "b");
        let no_default = [tier("a", false, false), tier("b", false, true)];
        assert_eq!(resolve_tier(&no_default, None).unwrap().tier, "b");
        assert!(resolve_tier(&[], None).is_none());
    }

    #[test]
    fn capabilities_round_trip_through_their_names() {
        for capability in Capability::ALL {
            assert_eq!(Capability::parse(capability.as_str()), Some(capability));
        }
        assert_eq!(Capability::parse("video"), None);
    }

    #[test]
    fn voices_and_sizes_reject_obvious_typos_only() {
        for good in ["alloy", "zh_female_shuangkuaisisi_moon_bigtts", "longxiaochun-v2", "en.US.1"] {
            assert!(valid_voice(good), "{good}");
        }
        for bad in ["", "有空格 的", "alloy,nova", "a b", &"x".repeat(81)] {
            assert!(!valid_voice(bad), "{bad}");
        }
        for good in ["1024x1024", "1536x1024", "auto", "512x512"] {
            assert!(valid_size(good), "{good}");
        }
        for bad in ["", "1024", "1024*1024", "1024x", "x1024", "999999x1", "AUTO"] {
            assert!(!valid_size(bad), "{bad}");
        }
    }

    #[test]
    fn slugs_are_lowercase_identifiers() {
        for good in ["standard", "xiangban-wallpaper", "tier_2"] {
            assert!(valid_slug(good), "{good}");
        }
        for bad in ["", "Standard", "2tier", "has space", "名字", &"x".repeat(41)] {
            assert!(!valid_slug(bad), "{bad}");
        }
    }
}
