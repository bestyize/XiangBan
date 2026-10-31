//! Fixed-origin, read-only model sources. Callers must supply a dedicated HTTP client with
//! `redirect(reqwest::redirect::Policy::none())`. Credentials and upstream bodies never enter errors.
//!
//! Wire references (checked 2026-10-31):
//! https://artificialanalysis.ai/data-api/docs/reference
//! https://openrouter.ai/docs/api/api-reference/datasets/daily-token-totals-for-top-50-models
//! AA Free v2 serves any subscription and preserves missing measurements as null. Its Intelligence
//! Index has an upstream version; scores must not be compared across versions or normalized to 100.
//! OpenRouter counts are decimal strings; only the derived display percentage uses floating point.

use super::CollectedItem;
use crate::error::{AppError, AppResult};
use chrono::{DateTime, Duration as Days, NaiveDate, Utc};
use reqwest::{header::HeaderValue, Client, Url};
use serde_json::{json, Value};
use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
    time::Duration,
};

const AA_ENDPOINT: &str = "https://artificialanalysis.ai/api/v2/language/models/free";
const OR_ENDPOINT: &str = "https://openrouter.ai/api/v1/datasets/rankings-daily";
const AA_SITE: &str = "https://artificialanalysis.ai/";
const OR_SITE: &str = "https://openrouter.ai/rankings";
const MAX_BODY: usize = 1024 * 1024;
const MAX_MODELS: usize = 2000;
const MAX_PAGES: u64 = 10;
const TIMEOUT: Duration = Duration::from_secs(30);
const METRICS: &[(&str, &str)] = &[
    ("intelligence", "artificial_analysis_intelligence_index"),
    (
        "finance_and_accounting",
        "artificial_analysis_finance_and_accounting_index",
    ),
    (
        "strategy_and_ops",
        "artificial_analysis_strategy_and_ops_index",
    ),
    ("legal", "artificial_analysis_legal_index"),
    (
        "healthcare_and_medical",
        "artificial_analysis_healthcare_and_medical_index",
    ),
    ("engineering", "artificial_analysis_engineering_index"),
    ("economics", "artificial_analysis_economics_index"),
];

type SourceResult<T> = Result<T, String>;
fn invalid() -> String {
    "来源数据格式或完整性校验失败".into()
}
fn bad() -> AppError {
    AppError::BadRequest(invalid())
}

fn text(v: &Value, max: usize) -> SourceResult<&str> {
    v.as_str()
        .filter(|s| {
            !s.trim().is_empty() && s.chars().count() <= max && !s.chars().any(char::is_control)
        })
        .ok_or_else(invalid)
}
fn nullable_number(v: &Value) -> SourceResult<Value> {
    if v.is_null() {
        return Ok(Value::Null);
    }
    v.as_f64()
        .filter(|n| n.is_finite() && *n >= 0.0 && *n <= 1e15)
        .map(|_| v.clone())
        .ok_or_else(invalid)
}
fn decimal(v: &Value) -> SourceResult<&str> {
    let s = v.as_str().ok_or_else(invalid)?;
    if s.is_empty()
        || s.len() > 80
        || !s.bytes().all(|c| c.is_ascii_digit())
        || (s.len() > 1 && s.starts_with('0'))
    {
        return Err(invalid());
    }
    Ok(s)
}
fn decimal_cmp(a: &str, b: &str) -> Ordering {
    a.len().cmp(&b.len()).then_with(|| a.cmp(b))
}
fn decimal_add(a: &str, b: &str) -> String {
    let mut carry = 0;
    let mut out = Vec::new();
    let mut a = a.bytes().rev();
    let mut b = b.bytes().rev();
    loop {
        let (x, y) = (a.next(), b.next());
        if x.is_none() && y.is_none() && carry == 0 {
            break;
        }
        let n = x.map_or(0, |v| v - b'0') + y.map_or(0, |v| v - b'0') + carry;
        out.push(b'0' + n % 10);
        carry = n / 10;
    }
    out.reverse();
    String::from_utf8(out).expect("decimal digits")
}
fn percentage(tokens: &str, total: &str) -> f64 {
    if total == "0" {
        return 0.0;
    }
    // Conversion occurs only after exact integer sorting and summation; 80 digits fit in f64.
    let n = tokens.parse::<f64>().expect("validated decimal");
    let d = total.parse::<f64>().expect("validated decimal");
    ((n / d * 100.0) * 1_000_000.0).round() / 1_000_000.0
}
fn day(v: &Value) -> SourceResult<NaiveDate> {
    let s = text(v, 10)?;
    let d = NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|_| invalid())?;
    if d.to_string() != s || d < NaiveDate::from_ymd_opt(2025, 1, 1).unwrap() {
        return Err(invalid());
    }
    Ok(d)
}
fn timestamp(v: &Value) -> SourceResult<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(text(v, 40)?)
        .map(|d| d.with_timezone(&Utc))
        .map_err(|_| invalid())
}
fn official_url(v: &Value, host: &str) -> SourceResult<()> {
    let url = Url::parse(text(v, 500)?).map_err(|_| invalid())?;
    if url.scheme() != "https"
        || url.host_str() != Some(host)
        || url.port().is_some()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
        || url.query().is_some()
    {
        return Err(invalid());
    }
    Ok(())
}
fn validate_size(v: &Value) -> SourceResult<()> {
    if serde_json::to_vec(v).map_err(|_| invalid())?.len() > MAX_BODY {
        return Err(invalid());
    }
    Ok(())
}

async fn request(
    http: &Client,
    url: Url,
    source: &str,
    key: &str,
    timeout: Duration,
) -> SourceResult<Value> {
    if key.is_empty() || key.len() > 4096 {
        return Err("来源密钥无效".into());
    }
    let mut credential = HeaderValue::from_str(if source == "openrouter" { "" } else { key })
        .map_err(|_| "来源密钥无效".to_string())?;
    let mut request = http
        .get(url.clone())
        .timeout(timeout)
        .header("Accept", "application/json");
    if source == "openrouter" {
        credential = HeaderValue::from_str(&format!("Bearer {key}"))
            .map_err(|_| "来源密钥无效".to_string())?;
        credential.set_sensitive(true);
        request = request.header("Authorization", credential);
    } else {
        credential.set_sensitive(true);
        request = request.header("x-api-key", credential);
    }
    let mut response = request.send().await.map_err(|e| {
        if e.is_timeout() {
            "来源请求超时".to_string()
        } else {
            "来源网络连接失败".to_string()
        }
    })?;
    if response.url() != &url {
        return Err("来源跳转已拒绝".into());
    }
    if !response.status().is_success() {
        return Err(match response.status().as_u16() {
            401 => "来源密钥无效",
            403 => "来源凭据权限不足",
            429 => "来源请求限流，请稍后重试",
            300..=399 => "来源跳转已拒绝",
            _ => "来源服务请求失败",
        }
        .into());
    }
    if response
        .content_length()
        .is_some_and(|n| n > MAX_BODY as u64)
    {
        return Err("来源响应超过大小限制".into());
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|e| {
        if e.is_timeout() {
            "来源请求超时".to_string()
        } else {
            "来源响应读取失败".to_string()
        }
    })? {
        if bytes.len() + chunk.len() > MAX_BODY {
            return Err("来源响应超过大小限制".into());
        }
        bytes.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&bytes).map_err(|_| invalid())
}

/// Read-only credential probe: one first-page/model-dataset request; never calls inference.
pub async fn verify_credential(source: &str, key: &str, http: &Client) -> SourceResult<()> {
    match source {
        "artificial_analysis" => {
            let mut url = Url::parse(AA_ENDPOINT).unwrap();
            url.query_pairs_mut().append_pair("page", "1");
            let raw = request(http, url, source, key, TIMEOUT).await?;
            aa_page(&raw, 1)?;
            // Verify that a nominal 200 actually contains the promised model shape.
            aa_board(raw["data"].as_array().unwrap().clone(), &raw, Utc::now())?;
            Ok(())
        }
        "openrouter" => {
            let now = Utc::now();
            let end = now.date_naive() - Days::days(1);
            let raw = request(
                http,
                usage_url(OR_ENDPOINT, end, end)?,
                source,
                key,
                TIMEOUT,
            )
            .await?;
            usage_items(raw, end, end, now).map(|_| ())
        }
        _ => Err("不支持的模型数据来源".into()),
    }
}

pub async fn collect(source: &str, key: &str, http: &Client) -> SourceResult<Vec<CollectedItem>> {
    collect_at(source, key, http, AA_ENDPOINT, OR_ENDPOINT, Utc::now()).await
}

// Only private test helpers accept endpoints. Production entry points always use fixed official URLs.
async fn collect_at(
    source: &str,
    key: &str,
    http: &Client,
    aa: &str,
    openrouter: &str,
    now: DateTime<Utc>,
) -> SourceResult<Vec<CollectedItem>> {
    match source {
        "artificial_analysis" => {
            let mut models = Vec::new();
            let mut first: Option<Value> = None;
            for page in 1..=MAX_PAGES {
                let mut url = Url::parse(aa).map_err(|_| invalid())?;
                url.query_pairs_mut().append_pair("page", &page.to_string());
                let raw = request(http, url, source, key, TIMEOUT).await?;
                let more = aa_page(&raw, page)?;
                if let Some(first) = &first {
                    if first["tier"] != raw["tier"]
                        || first["intelligence_index_version"] != raw["intelligence_index_version"]
                        || first["pagination"]["total_pages"] != raw["pagination"]["total_pages"]
                    {
                        return Err("来源分页快照不一致，请重试".into());
                    }
                } else {
                    first = Some(raw.clone());
                }
                models.extend(raw["data"].as_array().unwrap().iter().cloned());
                if models.len() > MAX_MODELS {
                    return Err(invalid());
                }
                if !more {
                    let data = aa_board(models, first.as_ref().unwrap(), now)?;
                    return Ok(vec![CollectedItem {
                        kind: "model_board".into(),
                        key: "artificial-analysis".into(),
                        data,
                        sort_order: 0,
                        published_at: None,
                        source_name: "Artificial Analysis".into(),
                        source_url: AA_SITE.into(),
                    }]);
                }
            }
            Err("来源分页超过采集上限".into())
        }
        "openrouter" => {
            let end = now.date_naive() - Days::days(1);
            let start = end - Days::days(2);
            let raw = request(
                http,
                usage_url(openrouter, start, end)?,
                source,
                key,
                TIMEOUT,
            )
            .await?;
            usage_items(raw, start, end, now)
        }
        _ => Err("不支持的模型数据来源".into()),
    }
}

fn aa_page(raw: &Value, expected_page: u64) -> SourceResult<bool> {
    if !matches!(raw["tier"].as_str(), Some("free" | "pro" | "commercial")) {
        return Err(invalid());
    }
    let version = raw["intelligence_index_version"]
        .as_f64()
        .ok_or_else(invalid)?;
    if !version.is_finite() || version <= 0.0 || version > 1000.0 {
        return Err(invalid());
    }
    let p = &raw["pagination"];
    let pages = p["total_pages"].as_u64().ok_or_else(invalid)?;
    let size = p["page_size"].as_u64().ok_or_else(invalid)?;
    let more = p["has_more"].as_bool().ok_or_else(invalid)?;
    let data = raw["data"].as_array().ok_or_else(invalid)?;
    if p["page"].as_u64() != Some(expected_page)
        || pages == 0
        || pages > MAX_PAGES
        || expected_page > pages
        || size == 0
        || size > 1000
        || data.is_empty()
        || data.len() > size as usize
        || more != (expected_page < pages)
    {
        return Err(invalid());
    }
    Ok(more)
}

fn aa_board(rows: Vec<Value>, envelope: &Value, now: DateTime<Utc>) -> SourceResult<Value> {
    let mut vendors = BTreeMap::new();
    let mut models = Vec::new();
    for row in rows {
        let creator = &row["model_creator"];
        let vendor_id = text(&creator["id"], 100)?.to_string();
        let vendor =
            json!({"id":vendor_id,"name":text(&creator["name"],160)?,"country":creator["country"]});
        if vendors.get(&vendor_id).is_some_and(|v| v != &vendor) {
            return Err(invalid());
        }
        vendors.insert(vendor_id.clone(), vendor);
        let mut metrics = serde_json::Map::new();
        for (out, input) in METRICS {
            metrics.insert((*out).into(), nullable_number(&row["evaluations"][*input])?);
        }
        // Deprecated source indices remain null even if an obsolete upstream cache has numeric fields.
        metrics.insert("coding".into(), Value::Null);
        metrics.insert("agentic".into(), Value::Null);
        models.push(json!({"id":text(&row["id"],100)?,"slug":text(&row["slug"],200)?,
            "name":text(&row["name"],160)?,"vendor_id":vendor_id,"release_date":row["release_date"],
            "metrics":metrics,"pricing":{"input":nullable_number(&row["pricing"]["price_1m_input_tokens"])?,
                "output":nullable_number(&row["pricing"]["price_1m_output_tokens"])?},
            "performance":{"output_tokens_per_second":nullable_number(&row["performance"]["median_output_tokens_per_second"])?,
                "time_to_first_token_seconds":nullable_number(&row["performance"]["median_time_to_first_token_seconds"])?}}));
    }
    models.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
    let data = json!({"source":"Artificial Analysis","source_url":AA_SITE,"as_of":now.to_rfc3339(),
        "index_version":envelope["intelligence_index_version"].to_string(),"schema_version":"aa-free-v2",
        "tier":envelope["tier"],"price_unit":"USD / 1M tokens","vendors":vendors.into_values().collect::<Vec<_>>(),"models":models});
    check_model_board(&data)?;
    Ok(data)
}

fn usage_url(endpoint: &str, start: NaiveDate, end: NaiveDate) -> SourceResult<Url> {
    let mut url = Url::parse(endpoint).map_err(|_| invalid())?;
    url.query_pairs_mut()
        .append_pair("period", "day")
        .append_pair("start_date", &start.to_string())
        .append_pair("end_date", &end.to_string());
    Ok(url)
}

fn usage_items(
    raw: Value,
    start: NaiveDate,
    end: NaiveDate,
    now: DateTime<Utc>,
) -> SourceResult<Vec<CollectedItem>> {
    let as_of = timestamp(&raw["meta"]["as_of"])?;
    let meta_start = day(&raw["meta"]["start_date"])?;
    let meta_end = day(&raw["meta"]["end_date"])?;
    let version = text(&raw["meta"]["version"], 40)?;
    if meta_start != start
        || meta_end != end
        || as_of > now + Days::minutes(5)
        || as_of.date_naive() <= end
    {
        return Err("来源日榜尚未完整就绪".into());
    }
    let rows = raw["data"].as_array().ok_or_else(invalid)?;
    if rows.is_empty() || rows.len() > 153 {
        return Err(invalid());
    }
    let mut days: BTreeMap<NaiveDate, BTreeMap<String, String>> = BTreeMap::new();
    for row in rows {
        let date = day(&row["date"])?;
        if date < start || date > end {
            return Err(invalid());
        }
        let id = text(&row["model_permaslug"], 200)?;
        let tokens = decimal(&row["total_tokens"])?;
        if days
            .entry(date)
            .or_default()
            .insert(id.to_string(), tokens.to_string())
            .is_some()
        {
            return Err(invalid());
        }
    }
    // A failed backfill cannot masquerade as a complete successful three-day refresh.
    if days.len() != (end - start).num_days() as usize + 1 {
        return Err("来源日榜缺少统计日，请稍后重试".into());
    }
    let mut items = Vec::new();
    for (date, mut entries) in days {
        let other = entries.remove("other").unwrap_or_else(|| "0".into());
        if entries.is_empty() || entries.len() > 50 {
            return Err(invalid());
        }
        let total = entries
            .values()
            .fold(other.clone(), |sum, n| decimal_add(&sum, n));
        let mut sorted: Vec<_> = entries.into_iter().collect();
        sorted.sort_by(|a, b| decimal_cmp(&b.1, &a.1).then_with(|| a.0.cmp(&b.0)));
        let models = sorted.into_iter().enumerate().map(|(i,(id,tokens))|
            json!({"id":id,"name":id,"tokens":tokens,"rank":i+1,"share_pct":percentage(&tokens,&total)})).collect::<Vec<_>>();
        let data = json!({"source":"OpenRouter","date":date.to_string(),"as_of":as_of.to_rfc3339(),
            "source_url":OR_SITE,"version":version,"period":"day","timezone":"UTC",
            "token_unit":"prompt_tokens + completion_tokens","total_tokens":total,"other_tokens":other,"models":models});
        check_model_usage(&data)?;
        items.push(CollectedItem {
            kind: "model_usage".into(),
            key: date.to_string(),
            data,
            sort_order: 0,
            published_at: None,
            source_name: "OpenRouter".into(),
            source_url: OR_SITE.into(),
        });
    }
    Ok(items)
}

/// Validate administrator edits and imports using the same invariants as collected snapshots.
pub fn validate_model_board(v: Value) -> AppResult<Value> {
    check_model_board(&v).map_err(|_| bad())?;
    Ok(v)
}
pub fn validate_model_usage(v: Value) -> AppResult<Value> {
    check_model_usage(&v).map_err(|_| bad())?;
    Ok(v)
}

fn check_model_board(v: &Value) -> SourceResult<()> {
    validate_size(v)?;
    official_url(&v["source_url"], "artificialanalysis.ai")?;
    if v["source"] != "Artificial Analysis"
        || v["schema_version"] != "aa-free-v2"
        || v["price_unit"] != "USD / 1M tokens"
        || !matches!(v["tier"].as_str(), Some("free" | "pro" | "commercial"))
    {
        return Err(invalid());
    }
    if timestamp(&v["as_of"])? > Utc::now() + Days::minutes(5) {
        return Err(invalid());
    }
    let ver = text(&v["index_version"], 20)?
        .parse::<f64>()
        .map_err(|_| invalid())?;
    if !ver.is_finite() || ver <= 0.0 || ver > 1000.0 {
        return Err(invalid());
    }
    let vendors = v["vendors"].as_array().ok_or_else(invalid)?;
    let models = v["models"].as_array().ok_or_else(invalid)?;
    if models.is_empty() || models.len() > MAX_MODELS || vendors.is_empty() || vendors.len() > 500 {
        return Err(invalid());
    }
    let mut vendor_ids = BTreeSet::new();
    for vendor in vendors {
        if !vendor_ids.insert(text(&vendor["id"], 100)?) {
            return Err(invalid());
        }
        text(&vendor["name"], 160)?;
        if !vendor["country"].is_null() {
            let country = text(&vendor["country"], 2)?;
            if country.len() != 2 || !country.bytes().all(|c| c.is_ascii_lowercase()) {
                return Err(invalid());
            }
        }
    }
    let mut ids = BTreeSet::new();
    let mut slugs = BTreeSet::new();
    for model in models {
        if !ids.insert(text(&model["id"], 100)?)
            || !slugs.insert(text(&model["slug"], 200)?)
            || !vendor_ids.contains(text(&model["vendor_id"], 100)?)
        {
            return Err(invalid());
        }
        text(&model["name"], 160)?;
        if !model["release_date"].is_null() {
            let s = text(&model["release_date"], 10)?;
            let d = NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|_| invalid())?;
            if d.to_string() != s {
                return Err(invalid());
            }
        }
        let metrics = model["metrics"].as_object().ok_or_else(invalid)?;
        for (field, _) in METRICS {
            nullable_number(metrics.get(*field).ok_or_else(invalid)?)?;
        }
        if metrics.get("coding") != Some(&Value::Null)
            || metrics.get("agentic") != Some(&Value::Null)
        {
            return Err(invalid());
        }
        for (group, fields) in [
            ("pricing", &["input", "output"][..]),
            (
                "performance",
                &["output_tokens_per_second", "time_to_first_token_seconds"][..],
            ),
        ] {
            let group = model[group].as_object().ok_or_else(invalid)?;
            for field in fields {
                nullable_number(group.get(*field).ok_or_else(invalid)?)?;
            }
        }
    }
    Ok(())
}
fn check_model_usage(v: &Value) -> SourceResult<()> {
    validate_size(v)?;
    official_url(&v["source_url"], "openrouter.ai")?;
    if v["source"] != "OpenRouter"
        || v["period"] != "day"
        || v["timezone"] != "UTC"
        || v["token_unit"] != "prompt_tokens + completion_tokens"
    {
        return Err(invalid());
    }
    text(&v["version"], 40)?;
    let date = day(&v["date"])?;
    let as_of = timestamp(&v["as_of"])?;
    if date >= Utc::now().date_naive()
        || as_of.date_naive() <= date
        || as_of > Utc::now() + Days::minutes(5)
    {
        return Err(invalid());
    }
    let total = decimal(&v["total_tokens"])?;
    let mut sum = decimal(&v["other_tokens"])?.to_string();
    let models = v["models"].as_array().ok_or_else(invalid)?;
    if models.is_empty() || models.len() > 50 {
        return Err(invalid());
    }
    let mut ids = BTreeSet::new();
    let mut previous: Option<(&str, &str)> = None;
    for (i, model) in models.iter().enumerate() {
        let id = text(&model["id"], 200)?;
        if id == "other" || !ids.insert(id) {
            return Err(invalid());
        }
        text(&model["name"], 200)?;
        let tokens = decimal(&model["tokens"])?;
        if model["rank"].as_u64() != Some(i as u64 + 1) {
            return Err(invalid());
        }
        if let Some((prev_tokens, prev_id)) = previous {
            let cmp = decimal_cmp(prev_tokens, tokens);
            if cmp == Ordering::Less || (cmp == Ordering::Equal && prev_id >= id) {
                return Err(invalid());
            }
        }
        let share = model["share_pct"].as_f64().ok_or_else(invalid)?;
        if !share.is_finite()
            || !(0.0..=100.0).contains(&share)
            || (share - percentage(tokens, total)).abs() > 0.000001
        {
            return Err(invalid());
        }
        previous = Some((tokens, id));
        sum = decimal_add(&sum, tokens);
    }
    if sum != total {
        return Err(invalid());
    }
    Ok(())
}

#[cfg(test)]
#[path = "model_sources_tests.rs"]
mod tests;
