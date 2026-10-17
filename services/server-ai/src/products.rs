//! 产品与档位：客户端只认档位名，真实模型只存在这里。

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
const TIER_COLUMNS: &str = "product_id, tier, display_name, description, model, requires_paid, is_default, sort_order, enabled";

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
    fn slugs_are_lowercase_identifiers() {
        for good in ["standard", "xiangban-wallpaper", "tier_2"] {
            assert!(valid_slug(good), "{good}");
        }
        for bad in ["", "Standard", "2tier", "has space", "名字", &"x".repeat(41)] {
            assert!(!valid_slug(bad), "{bad}");
        }
    }
}
