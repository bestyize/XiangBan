//! GitHub 官方 Trending 周榜采集：公开网页，无需凭据，增量直接取自 GitHub 公布的“stars this week”。
//! 旧的白名单快照榜已停止生成，历史期次仍按 check_issue 校验。
//! Authentication check（工具版本源仍在使用）: https://docs.github.com/en/rest/rate-limit/rate-limit#get-rate-limit-status-for-the-authenticated-user
use super::CollectedItem;
use crate::error::{AppError, AppResult};
use chrono::{DateTime, Duration, NaiveDate, Utc};
use reqwest::{Client, Url};
use serde_json::{json, Value};
use std::collections::BTreeSet;
const API: &str = "https://api.github.com";
const TRENDING_URL: &str = "https://github.com/trending?since=weekly";
const TRENDING_SOURCE_URL: &str = "https://github.com/trending";
const TRENDING_METRIC: &str = "github trending weekly stars";
const BODY_LIMIT: usize = 256 * 1024;
const PAGE_LIMIT: usize = 2 * 1024 * 1024;
const DESCRIPTION_MAX: usize = 300;
type SourceResult<T> = Result<T, String>;
fn invalid() -> String {
    "GitHub 仓库响应或快照数据无效".into()
}
fn bad() -> AppError {
    AppError::BadRequest(invalid())
}

pub fn valid_name(name: &str) -> bool {
    let Some((owner, repo)) = name.split_once('/') else {
        return false;
    };
    !owner.is_empty()
        && owner.len() <= 39
        && owner
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
        && !owner.starts_with('-')
        && !owner.ends_with('-')
        && !repo.is_empty()
        && repo.len() <= 100
        && ![".", ".."].contains(&repo)
        && repo
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
}
fn hex_color(value: &str) -> bool {
    value.len() == 7 && value.starts_with('#') && value[1..].bytes().all(|b| b.is_ascii_hexdigit())
}

async fn get(http: &Client, url: Url, key: &str) -> SourceResult<reqwest::Response> {
    http.get(url)
        .bearer_auth(key)
        .header("accept", "application/vnd.github+json")
        .header("x-github-api-version", "2022-11-28")
        .timeout(std::time::Duration::from_secs(20))
        .send()
        .await
        .map_err(|_| "GitHub 请求失败，请稍后重试".into())
}
async fn body(mut response: reqwest::Response) -> SourceResult<Value> {
    if !response.status().is_success() {
        return Err(format!(
            "GitHub 请求未成功（HTTP {}）",
            response.status().as_u16()
        ));
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| "GitHub 响应读取失败".to_string())?
    {
        if bytes.len() + chunk.len() > BODY_LIMIT {
            return Err("GitHub 响应超过体积上限".into());
        }
        bytes.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&bytes).map_err(|_| invalid())
}
pub async fn verify_credential(key: &str, http: &Client) -> SourceResult<()> {
    verify_at(API, key, http).await
}
async fn verify_at(api: &str, key: &str, http: &Client) -> SourceResult<()> {
    let response = get(
        http,
        Url::parse(&format!("{api}/rate_limit")).map_err(|_| invalid())?,
        key,
    )
    .await?;
    let value = body(response).await?;
    let core = &value["resources"]["core"];
    let limit = core["limit"].as_u64().ok_or_else(invalid)?;
    core["remaining"]
        .as_u64()
        .filter(|v| *v <= limit)
        .ok_or_else(invalid)?;
    // Authenticated API limits are >= 5000; an anonymous response must not validate a token.
    if limit < 5000 {
        return Err("GitHub 未确认已认证访问".into());
    }
    Ok(())
}

async fn fetch_page(http: &Client, url: &str) -> SourceResult<String> {
    let mut response = http
        .get(url)
        .header("accept", "text/html")
        .header("accept-language", "en")
        .timeout(std::time::Duration::from_secs(20))
        .send()
        .await
        .map_err(|_| "GitHub Trending 请求失败，请稍后重试".to_string())?;
    if !response.status().is_success() {
        return Err(format!(
            "GitHub Trending 请求未成功（HTTP {}）",
            response.status().as_u16()
        ));
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| "GitHub Trending 响应读取失败".to_string())?
    {
        if bytes.len() + chunk.len() > PAGE_LIMIT {
            return Err("GitHub Trending 响应超过体积上限".into());
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

fn between<'a>(text: &'a str, start: &str, end: &str) -> Option<&'a str> {
    let from = text.find(start)? + start.len();
    let len = text[from..].find(end)?;
    Some(&text[from..from + len])
}
fn plain_text(html: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' if in_tag => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&#x27;", "'")
        .replace("&amp;", "&")
        .chars()
        .filter(|c| !c.is_control() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
fn count(text: &str) -> Option<i64> {
    let digits = text.trim().replace(',', "");
    if digits.is_empty() || digits.len() > 10 || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok().filter(|v| *v <= 1_000_000_000)
}

fn parse_row(row: &str) -> Option<Value> {
    let name = between(between(row, "<h2", "</h2>")?, "href=\"/", "\"")?;
    if !valid_name(name) {
        return None;
    }
    let description = {
        let full = between(row, "<p class=\"col-9", "</p>").and_then(|p| p.split_once('>')).map(|(_, text)| plain_text(text)).unwrap_or_default();
        if full.chars().count() > DESCRIPTION_MAX {
            format!("{}…", full.chars().take(DESCRIPTION_MAX - 1).collect::<String>().trim_end())
        } else {
            full
        }
    };
    let language: String = between(row, "itemprop=\"programmingLanguage\">", "</span>")
        .map(plain_text)
        .unwrap_or_default()
        .chars()
        .take(40)
        .collect();
    let color = between(row, "repo-language-color\" style=\"background-color: ", "\"")
        .map(|c| c.trim().trim_end_matches(';').trim())
        .filter(|c| hex_color(c))
        .unwrap_or_default();
    let stars_link = between(row, &format!("href=\"/{name}/stargazers\""), "</a>")?;
    let total = count(&plain_text(stars_link.split_once('>')?.1))?;
    let head = &row[..row.find(" stars this week")?];
    let digits_from = head
        .char_indices()
        .rev()
        .find(|(_, c)| !(c.is_ascii_digit() || *c == ','))
        .map_or(0, |(i, c)| i + c.len_utf8());
    let gain = count(&head[digits_from..])?;
    if gain > total {
        return None;
    }
    Some(json!({"full_name":name,"url":format!("https://github.com/{name}"),"description":description,"language":language,
        "language_color":color,"star_total":total,"star_gain":gain,"spark":[],"icon":"code","tint":"#E6F4EA"}))
}
/// 保持 GitHub 页面给出的排序；页面结构变化时整批失败，保留上一份线上榜单。
fn parse_trending(html: &str) -> SourceResult<Vec<Value>> {
    let articles = html
        .split("<article class=\"Box-row\"")
        .skip(1)
        .map(|chunk| chunk.split("</article>").next().unwrap_or(chunk))
        .collect::<Vec<_>>();
    let mut seen = BTreeSet::new();
    let ranks = articles
        .iter()
        .filter_map(|row| parse_row(row))
        .filter(|rank| seen.insert(rank["full_name"].as_str().unwrap_or_default().to_ascii_lowercase()))
        .take(30)
        .collect::<Vec<_>>();
    if articles.len() < 5 || ranks.len() * 2 < articles.len() {
        return Err("GitHub Trending 页面结构已变化，无法解析榜单".into());
    }
    Ok(ranks)
}

pub async fn collect(pool: &sqlx::PgPool, http: &Client) -> SourceResult<Vec<CollectedItem>> {
    let html = fetch_page(http, TRENDING_URL).await?;
    let mut ranks = parse_trending(&html)?;
    // README 与中文简介是附加信息，整理失败不影响榜单发布。
    crate::repo_details::enrich(pool, http, &mut ranks).await;
    issue_from(ranks, Utc::now())
}
#[cfg(test)]
fn build_issue(html: &str, now: DateTime<Utc>) -> SourceResult<Vec<CollectedItem>> {
    issue_from(parse_trending(html)?, now)
}
fn issue_from(ranks: Vec<Value>, now: DateTime<Utc>) -> SourceResult<Vec<CollectedItem>> {
    let end = now.date_naive();
    let start = end - Duration::days(7);
    let total_gain = ranks.iter().filter_map(|r| r["star_gain"].as_i64()).sum::<i64>();
    let issue_no = end
        .format("%Y%m%d")
        .to_string()
        .parse::<i64>()
        .map_err(|_| invalid())?;
    let data = json!({"source":"GitHub","source_url":TRENDING_SOURCE_URL,"as_of":now.to_rfc3339(),"metric":TRENDING_METRIC,"issue_no":issue_no,
        "week_start":start.to_string(),"week_end":end.to_string(),"observation_start":start.to_string(),"observation_end":end.to_string(),"observed_days":7,"complete":true,
        "total_gain":total_gain,"wow_pct":null,"daily_gain":[],"ranks":ranks});
    let data = validate_issue(data).map_err(|_| invalid())?;
    Ok(vec![CollectedItem {
        kind: "repo_issue".into(),
        key: format!("github-rolling-{end}"),
        data,
        sort_order: 0,
        published_at: None,
        source_name: "GitHub Trending".into(),
        source_url: TRENDING_SOURCE_URL.into(),
    }])
}

fn check_trending(v: &Value) -> SourceResult<()> {
    if serde_json::to_vec(v).map_err(|_| invalid())?.len() > BODY_LIMIT
        || v["source"] != "GitHub"
        || v["source_url"] != TRENDING_SOURCE_URL
        || !v["wow_pct"].is_null()
    {
        return Err(invalid());
    }
    let date = |key: &str| {
        NaiveDate::parse_from_str(v[key].as_str().unwrap_or_default(), "%Y-%m-%d").map_err(|_| invalid())
    };
    let (start, end) = (date("observation_start")?, date("observation_end")?);
    if (end - start).num_days() != 7
        || end > Utc::now().date_naive()
        || v["observed_days"].as_i64() != Some(7)
        || v["week_start"] != start.to_string()
        || v["week_end"] != end.to_string()
        || v["issue_no"].as_i64() != end.format("%Y%m%d").to_string().parse::<i64>().ok()
    {
        return Err(invalid());
    }
    let as_of = DateTime::parse_from_rfc3339(v["as_of"].as_str().ok_or_else(invalid)?)
        .map_err(|_| invalid())?;
    if as_of.with_timezone(&Utc).date_naive() != end || as_of > Utc::now() + Duration::minutes(5) {
        return Err(invalid());
    }
    let ranks = v["ranks"]
        .as_array()
        .filter(|a| !a.is_empty() && a.len() <= 30)
        .ok_or_else(invalid)?;
    let mut names = BTreeSet::new();
    let mut total = 0i64;
    for rank in ranks {
        let name = rank["full_name"]
            .as_str()
            .filter(|n| valid_name(n))
            .ok_or_else(invalid)?;
        let (Some(gain), Some(stars)) = (rank["star_gain"].as_i64(), rank["star_total"].as_i64()) else {
            return Err(invalid());
        };
        if !names.insert(name.to_ascii_lowercase())
            || rank["url"] != format!("https://github.com/{name}")
            || !(0..=1_000_000_000).contains(&gain)
            || !(gain..=1_000_000_000).contains(&stars)
            || rank["description"].as_str().is_none_or(|d| d.chars().count() > DESCRIPTION_MAX || d.chars().any(char::is_control))
            || rank["language"].as_str().is_none_or(|l| l.chars().count() > 40)
            || rank["language_color"].as_str().is_none_or(|c| !c.is_empty() && !hex_color(c))
            || rank["spark"] != json!([])
            || rank.get("description_zh").is_some_and(|d| d.as_str().is_none_or(|t| t.chars().count() > 240 || t.chars().any(char::is_control)))
        {
            return Err(invalid());
        }
        total += gain;
    }
    if v["total_gain"].as_i64() != Some(total) || v["complete"] != true || v["daily_gain"] != json!([]) {
        return Err(invalid());
    }
    Ok(())
}

/// 校验 Trending 周榜；旧快照榜的历史期次沿用原有严格校验。
pub fn validate_issue(value: Value) -> AppResult<Value> {
    let checked = if value["metric"] == TRENDING_METRIC {
        check_trending(&value)
    } else {
        check_issue(&value)
    };
    checked.map_err(|_| bad())?;
    Ok(value)
}

fn check_issue(v: &Value) -> SourceResult<()> {
    if serde_json::to_vec(v).map_err(|_| invalid())?.len() > BODY_LIMIT
        || v["source"] != "GitHub"
        || v["source_url"] != "https://github.com"
        || !matches!(v["metric"].as_str(), Some("observed net star change" | "star total baseline"))
        || !v["wow_pct"].is_null()
    {
        return Err(invalid());
    }
    let start = NaiveDate::parse_from_str(
        v["observation_start"].as_str().ok_or_else(invalid)?,
        "%Y-%m-%d",
    )
    .map_err(|_| invalid())?;
    let end = NaiveDate::parse_from_str(
        v["observation_end"].as_str().ok_or_else(invalid)?,
        "%Y-%m-%d",
    )
    .map_err(|_| invalid())?;
    let n = (end - start).num_days();
    let baseline = v["metric"] == "star total baseline";
    if !(0..=7).contains(&n)
        || baseline != (n == 0)
        || end > Utc::now().date_naive()
        || v["observed_days"].as_i64() != Some(n)
        || v["week_start"] != start.to_string()
        || v["week_end"] != end.to_string()
        || v["issue_no"].as_i64() != end.format("%Y%m%d").to_string().parse::<i64>().ok()
    {
        return Err(invalid());
    }
    let as_of = DateTime::parse_from_rfc3339(v["as_of"].as_str().ok_or_else(invalid)?)
        .map_err(|_| invalid())?;
    if as_of.with_timezone(&Utc).date_naive() != end || as_of > Utc::now() + Duration::minutes(5) {
        return Err(invalid());
    }
    let ranks = v["ranks"]
        .as_array()
        .filter(|a| !a.is_empty() && a.len() <= 30)
        .ok_or_else(invalid)?;
    let mut ids = BTreeSet::new();
    let mut names = BTreeSet::new();
    let mut total = 0i64;
    let mut all_complete = true;
    let mut daily = vec![0i64; 7];
    let mut previous: Option<(i64, String)> = None;
    for rank in ranks {
        let id = rank["repo_id"]
            .as_str()
            .and_then(|v| v.parse::<i64>().ok())
            .filter(|v| *v > 0)
            .ok_or_else(invalid)?;
        let name = rank["full_name"]
            .as_str()
            .filter(|v| valid_name(v))
            .ok_or_else(invalid)?;
        if !ids.insert(id)
            || !names.insert(name.to_ascii_lowercase())
            || rank["url"] != format!("https://github.com/{name}")
            || rank["observed_days"].as_i64() != Some(n)
        {
            return Err(invalid());
        }
        let dates = rank["snapshot_dates"].as_array().ok_or_else(invalid)?;
        let times = rank["snapshot_times"]
            .as_array()
            .filter(|a| a.len() == dates.len())
            .ok_or_else(invalid)?;
        let spark = rank["spark"]
            .as_array()
            .filter(|a| a.len() == dates.len() && if baseline { a.len() == 1 } else { (2..=8).contains(&a.len()) })
            .ok_or_else(invalid)?;
        let mut points = Vec::new();
        for ((date, time), stars) in dates.iter().zip(times).zip(spark) {
            let date = NaiveDate::parse_from_str(date.as_str().ok_or_else(invalid)?, "%Y-%m-%d")
                .map_err(|_| invalid())?;
            let time = DateTime::parse_from_rfc3339(time.as_str().ok_or_else(invalid)?)
                .map_err(|_| invalid())?;
            if time.with_timezone(&Utc).date_naive() != date
                || time > as_of
                || date < start
                || date > end
            {
                return Err(invalid());
            }
            let stars = stars
                .as_i64()
                .filter(|v| *v >= 0 && *v <= 1_000_000_000)
                .ok_or_else(invalid)?;
            points.push((date, stars));
        }
        if points.first().unwrap().0 != start
            || points.last().unwrap().0 != end
            || points.windows(2).any(|w| w[1].0 <= w[0].0)
            || rank["observation_start"] != times[0]
            || rank["observation_end"] != times[times.len() - 1]
        {
            return Err(invalid());
        }
        let gain = points.last().unwrap().1 - points[0].1;
        let full = n == 7 && points.len() == 8;
        let suspicious = points.windows(2).any(|w| {
            let gain = w[1].1 - w[0].1;
            gain < 0 || (gain >= 10000 && gain as f64 >= w[0].1.max(1) as f64 * 0.5)
        });
        if rank["star_gain"] != if baseline {Value::Null}else{json!(gain)}
            || rank["star_total"].as_i64() != Some(points.last().unwrap().1)
            || rank["complete"].as_bool() != Some(full)
            || rank["suspected_growth"].as_bool() != Some(suspicious)
        {
            return Err(invalid());
        }
        let order = if baseline { points.last().unwrap().1 } else { gain };
        if let Some((prev, ref prev_id)) = previous {
            let id_str = id.to_string();
            if order > prev || (order == prev && prev_id >= &id_str) {
                return Err(invalid());
            }
        }
        previous = Some((order, id.to_string()));
        total += gain;
        all_complete &= full;
        if full {
            for (i, w) in points.windows(2).enumerate() {
                daily[i] += w[1].1 - w[0].1;
            }
        }
    }
    if v["total_gain"] != if baseline {Value::Null}else{json!(total)}
        || v["complete"].as_bool() != Some(all_complete)
        || v["daily_gain"] != json!(if all_complete { daily } else { vec![] })
    {
        return Err(invalid());
    }
    Ok(())
}

#[cfg(test)]
#[path = "repo_sources_tests.rs"]
mod tests;
