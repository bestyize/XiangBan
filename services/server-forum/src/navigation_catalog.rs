//! Versioned navigation seeds preserve edited content, visibility and removals.
use crate::{
    discover,
    error::{AppError, AppResult},
};
use serde_json::{json, Value};
use sqlx::PgPool;
use std::sync::OnceLock;

pub const MIGRATION: &str = "navigation-catalog-20261103-v2";
pub fn catalog() -> &'static Value {
    static CATALOG: OnceLock<Value> = OnceLock::new();
    CATALOG.get_or_init(|| {
        serde_json::from_str(include_str!("../editorial/navigation.json"))
            .expect("valid navigation catalog")
    })
}
pub async fn seed(pool: &PgPool) -> AppResult<()> {
    let mut tx = pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
        .bind(MIGRATION)
        .execute(&mut *tx)
        .await?;
    let (done,): (bool,) =
        sqlx::query_as("SELECT EXISTS(SELECT 1 FROM data_migrations WHERE name=$1)")
            .bind(MIGRATION)
            .fetch_one(&mut *tx)
            .await?;
    if done {
        return Ok(());
    }
    let (legacy_done,): (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM data_migrations WHERE name='navigation-catalog-20261103-v1')",
    )
    .fetch_one(&mut *tx)
    .await?;
    for (index, item) in catalog()["items"].as_array().unwrap().iter().enumerate() {
        let key = item["key"].as_str().unwrap();
        let old: Option<(Value,)> = sqlx::query_as(
            "SELECT data FROM discover_items WHERE kind='nav' AND key=$1 FOR UPDATE",
        )
        .bind(key)
        .fetch_optional(&mut *tx)
        .await?;
        if let Some((mut data,)) = old {
            // Reclassify v1 entries once. Explicitly classified editorial records remain authoritative.
            if data["category"].as_str().unwrap_or_default().is_empty() {
                let legacy = &catalog()["legacy_items"][key];
                for field in ["name", "vendor", "url"] {
                    if !legacy.is_null() && data[field] == legacy[field] {
                        data[field] = item[field].clone();
                    }
                }
                for field in ["category", "group", "subtype", "region"] {
                    data[field] = item[field].clone();
                }
                if data["summary"].as_str().unwrap_or_default().is_empty() {
                    data["summary"] = item["summary"].clone();
                }
                if data["logo"].as_str().unwrap_or_default().is_empty()
                    && data["vendor"] == item["vendor"]
                {
                    data["logo"] = item["logo"].clone();
                }
                sqlx::query("UPDATE discover_items SET data=$2 WHERE kind='nav' AND key=$1")
                    .bind(key)
                    .bind(data)
                    .execute(&mut *tx)
                    .await?;
            }
        } else {
            if legacy_done && !catalog()["legacy_items"][key].is_null() {
                continue;
            }
            let mut data = discover::validate("nav", item.clone())
                .map_err(|e| AppError::Internal(format!("导航目录无效: {e}")))?;
            data["sample"] = json!(false);
            sqlx::query("INSERT INTO discover_items(kind,key,sort_order,data) VALUES('nav',$1,$2,$3) ON CONFLICT(kind,key) DO NOTHING").bind(key).bind(index as i32).bind(data).execute(&mut *tx).await?;
        }
    }
    sqlx::query("INSERT INTO data_migrations(name) VALUES($1) ON CONFLICT DO NOTHING")
        .bind(MIGRATION)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}
