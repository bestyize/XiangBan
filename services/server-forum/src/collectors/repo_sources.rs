//! 白名单公开仓库的每日观测，不采集用户 star 明细，不将观测净变化冒充自然日新增。
//! API: https://docs.github.com/en/rest/repos/repos#get-a-repository
//! Authentication check: https://docs.github.com/en/rest/rate-limit/rate-limit#get-rate-limit-status-for-the-authenticated-user
use super::CollectedItem;
use crate::error::{AppError, AppResult};
use chrono::{DateTime, Duration, NaiveDate, Utc};
use reqwest::{Client, Url};
use serde_json::{json, Value};
use sqlx::PgPool;
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;
const API: &str = "https://api.github.com";
const BODY_LIMIT: usize = 256 * 1024;
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
pub fn sanitize_config(config: Value) -> AppResult<Value> {
    let object = config
        .as_object()
        .ok_or_else(|| AppError::BadRequest("仓库配置必须为对象".into()))?;
    if object.len() != 1 || !object.contains_key("repositories") {
        return Err(AppError::BadRequest("仓库配置只支持 repositories".into()));
    }
    let repositories = object["repositories"]
        .as_array()
        .filter(|v| v.len() <= 30)
        .ok_or_else(|| AppError::BadRequest("最多配置 30 个公开仓库".into()))?;
    let mut names = BTreeSet::new();
    for name in repositories {
        let name = name
            .as_str()
            .ok_or_else(|| AppError::BadRequest("仓库名称应为 owner/repo".into()))?
            .trim()
            .to_ascii_lowercase();
        if !valid_name(&name) || !names.insert(name) {
            return Err(AppError::BadRequest(
                "仓库名称须为唯一的 owner/repo，不能填链接".into(),
            ));
        }
    }
    Ok(json!({"repositories":names.into_iter().collect::<Vec<_>>()}))
}
pub fn configured_names(config: &Value) -> AppResult<Vec<String>> {
    let clean = sanitize_config(config.clone())?;
    let names = clean["repositories"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect::<Vec<_>>();
    if names.is_empty() {
        return Err(AppError::BadRequest("请至少配置一个公开仓库".into()));
    }
    Ok(names)
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

#[derive(Clone, Debug)]
struct Repository {
    id: i64,
    full_name: String,
    description: String,
    language: String,
    stars: i64,
}
impl Repository {
    fn parse(value: Value) -> SourceResult<Self> {
        if value["private"] != false || value["visibility"].as_str().is_some_and(|v| v != "public")
        {
            return Err("仅允许公开 GitHub 仓库".into());
        }
        let id = value["id"]
            .as_i64()
            .filter(|v| *v > 0)
            .ok_or_else(invalid)?;
        let full_name = value["full_name"]
            .as_str()
            .filter(|v| valid_name(v))
            .ok_or_else(invalid)?
            .to_string();
        let stars = value["stargazers_count"]
            .as_i64()
            .filter(|v| *v >= 0 && *v <= 1_000_000_000)
            .ok_or_else(invalid)?;
        let description = value["description"]
            .as_str()
            .unwrap_or_default()
            .chars()
            .filter(|c| !c.is_control())
            .take(120)
            .collect();
        let language = value["language"]
            .as_str()
            .unwrap_or_default()
            .chars()
            .filter(|c| !c.is_control())
            .take(40)
            .collect();
        Ok(Self {
            id,
            full_name,
            description,
            language,
            stars,
        })
    }
}
async fn fetch(api: &str, name: &str, key: &str, http: &Client) -> SourceResult<Repository> {
    if !valid_name(name) {
        return Err(invalid());
    }
    let mut response = get(
        http,
        Url::parse(&format!("{api}/repos/{name}")).map_err(|_| invalid())?,
        key,
    )
    .await?;
    // Follow only GitHub's numeric repository relocation, never an arbitrary Location target.
    if response.status().as_u16() == 301
        || response.status().as_u16() == 302
        || response.status().as_u16() == 307
    {
        let location = response
            .headers()
            .get(reqwest::header::LOCATION)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(invalid)?;
        let url = Url::parse(location).map_err(|_| invalid())?;
        let origin = Url::parse(api).map_err(|_| invalid())?;
        let repo_id = url
            .path()
            .strip_prefix("/repositories/")
            .filter(|id| !id.is_empty() && id.len() <= 20 && id.bytes().all(|b| b.is_ascii_digit()))
            .ok_or_else(invalid)?;
        if url.origin() != origin.origin()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
        {
            return Err("GitHub 跳转目标不受信任".into());
        }
        let fixed = Url::parse(&format!("{api}/repositories/{repo_id}")).map_err(|_| invalid())?;
        response = get(http, fixed, key).await?;
    }
    Repository::parse(body(response).await?)
}

#[cfg(test)]
async fn save_snapshot(pool: &PgPool, repo: &Repository, at: DateTime<Utc>) -> AppResult<()> {
    let mut tx = pool.begin().await?;
    sqlx::query("INSERT INTO collection_github_repositories(repo_id,full_name,description,language,updated_at) VALUES($1,$2,$3,$4,$5) ON CONFLICT(repo_id) DO UPDATE SET full_name=$2,description=$3,language=$4,updated_at=$5")
        .bind(repo.id).bind(&repo.full_name).bind(&repo.description).bind(&repo.language).bind(at).execute(&mut *tx).await?;
    // The first successful observation of a UTC day is stable on retries; exact time is retained.
    sqlx::query("INSERT INTO collection_github_snapshots(repo_id,day,star_total,observed_at) VALUES($1,$2,$3,$4) ON CONFLICT(repo_id,day) DO NOTHING")
        .bind(repo.id).bind(at.date_naive()).bind(repo.stars).bind(at).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn collect(
    pool: &PgPool,
    key: &str,
    http: &Client,
    config: &Value,
    run_id: Uuid,
) -> SourceResult<Vec<CollectedItem>> {
    collect_with_clock(pool, key, http, config, run_id, API, None).await
}
async fn collect_with_clock(
    pool: &PgPool,
    key: &str,
    http: &Client,
    config: &Value,
    run_id: Uuid,
    api: &str,
    clock: Option<DateTime<Utc>>,
) -> SourceResult<Vec<CollectedItem>> {
    let names = configured_names(config).map_err(|_| "GitHub 候选仓库配置无效".to_string())?;
    let mut repos = BTreeMap::new();
    for name in names {
        let repo = fetch(api, &name, key, http).await?;
        let now = clock.unwrap_or_else(Utc::now);
        if repos.contains_key(&repo.id) {
            return Err("多个候选仓库指向同一仓库，请更新白名单".into());
        }
        // Every local write checks ownership too: an expired worker cannot alter its successor's baselines.
        let mut tx = pool
            .begin()
            .await
            .map_err(|_| "GitHub 快照存储失败".to_string())?;
        let lease:Option<(bool,)>=sqlx::query_as("SELECT true FROM collection_sources WHERE id='github_repos' AND active_run=$1 AND lease_until>now() AND config=(SELECT config FROM collection_runs WHERE id=$1) FOR UPDATE")
            .bind(run_id).fetch_optional(&mut *tx).await.map_err(|_|"GitHub 租约读取失败".to_string())?;
        if lease.is_none() {
            return Err("GitHub 采集租约已失效".into());
        }
        // Saving uses this same connection, so source lease remains locked until the snapshot commits.
        sqlx::query("INSERT INTO collection_github_repositories(repo_id,full_name,description,language,updated_at) VALUES($1,$2,$3,$4,$5) ON CONFLICT(repo_id) DO UPDATE SET full_name=$2,description=$3,language=$4,updated_at=$5")
            .bind(repo.id).bind(&repo.full_name).bind(&repo.description).bind(&repo.language).bind(now).execute(&mut *tx).await.map_err(|_|"GitHub 快照存储失败".to_string())?;
        sqlx::query("INSERT INTO collection_github_snapshots(repo_id,day,star_total,observed_at) VALUES($1,$2,$3,$4) ON CONFLICT(repo_id,day) DO NOTHING")
            .bind(repo.id).bind(now.date_naive()).bind(repo.stars).bind(now).execute(&mut *tx).await.map_err(|_|"GitHub 快照存储失败".to_string())?;
        tx.commit()
            .await
            .map_err(|_| "GitHub 快照存储失败".to_string())?;
        repos.insert(repo.id, repo);
    }
    build_issue(pool, &repos, clock.unwrap_or_else(Utc::now))
        .await
        .map_err(|_| "GitHub 榜单快照不完整或无法读取".into())
}

#[cfg(test)]
async fn collect_at(
    pool: &PgPool,
    key: &str,
    http: &Client,
    config: &Value,
    run_id: Uuid,
    api: &str,
    now: DateTime<Utc>,
) -> SourceResult<Vec<CollectedItem>> {
    collect_with_clock(pool, key, http, config, run_id, api, Some(now)).await
}

async fn build_issue(
    pool: &PgPool,
    repos: &BTreeMap<i64, Repository>,
    now: DateTime<Utc>,
) -> AppResult<Vec<CollectedItem>> {
    let end = now.date_naive();
    let earliest = end - Duration::days(7);
    let mut snapshots = BTreeMap::new();
    let mut common: Option<BTreeSet<NaiveDate>> = None;
    for id in repos.keys() {
        let rows:Vec<(NaiveDate,i64,DateTime<Utc>)>=sqlx::query_as("SELECT day,star_total,observed_at FROM collection_github_snapshots WHERE repo_id=$1 AND day BETWEEN $2 AND $3 ORDER BY day")
            .bind(id).bind(earliest).bind(end).fetch_all(pool).await?;
        let days = rows.iter().map(|r| r.0).collect::<BTreeSet<_>>();
        common = Some(match common {
            Some(old) => old.intersection(&days).copied().collect(),
            None => days,
        });
        snapshots.insert(*id, rows);
    }
    let Some(common) = common else {
        return Ok(vec![]);
    };
    if common.len() < 2 || !common.contains(&end) {
        return Ok(vec![]);
    }
    let start = *common.first().unwrap();
    let observed_days = (end - start).num_days();
    let mut complete = observed_days == 7;
    let mut ranks = Vec::new();
    let mut total_gain = 0;
    let mut total_daily = vec![0i64; 7];
    for (id, repo) in repos {
        let rows = snapshots[id]
            .iter()
            .filter(|r| r.0 >= start)
            .collect::<Vec<_>>();
        let first = rows.first().ok_or_else(bad)?;
        let last = rows.last().ok_or_else(bad)?;
        let full = rows.len() == 8
            && observed_days == 7
            && rows.windows(2).all(|w| (w[1].0 - w[0].0).num_days() == 1);
        complete &= full;
        let suspicious = rows.windows(2).any(|w| {
            let gain = w[1].1 - w[0].1;
            gain < 0 || (gain >= 10000 && gain as f64 >= w[0].1.max(1) as f64 * 0.5)
        });
        let gain = last.1 - first.1;
        total_gain += gain;
        if full {
            for (i, w) in rows.windows(2).enumerate() {
                total_daily[i] += w[1].1 - w[0].1;
            }
        }
        ranks.push(json!({"repo_id":id.to_string(),"full_name":repo.full_name,"url":format!("https://github.com/{}",repo.full_name),"description":repo.description,"language":repo.language,
            "language_color":"","star_total":last.1,"star_gain":gain,"spark":rows.iter().map(|r|r.1).collect::<Vec<_>>(),"snapshot_dates":rows.iter().map(|r|r.0.to_string()).collect::<Vec<_>>(),
            "snapshot_times":rows.iter().map(|r|r.2.to_rfc3339()).collect::<Vec<_>>(),"icon":"code","tint":"#E6F4EA","observed_days":observed_days,"complete":full,"suspected_growth":suspicious,
            "observation_start":first.2.to_rfc3339(),"observation_end":last.2.to_rfc3339()}));
    }
    ranks.sort_by(|a, b| {
        b["star_gain"]
            .as_i64()
            .cmp(&a["star_gain"].as_i64())
            .then_with(|| a["repo_id"].as_str().cmp(&b["repo_id"].as_str()))
    });
    let key = format!("github-rolling-{end}");
    let issue_no = end
        .format("%Y%m%d")
        .to_string()
        .parse::<i32>()
        .map_err(|_| bad())?;
    let data = json!({"source":"GitHub","source_url":"https://github.com","as_of":now.to_rfc3339(),"metric":"observed net star change","issue_no":issue_no,
        "week_start":start.to_string(),"week_end":end.to_string(),"observation_start":start.to_string(),"observation_end":end.to_string(),"observed_days":observed_days,"complete":complete,
        "total_gain":total_gain,"wow_pct":null,"daily_gain":if complete {total_daily}else{vec![]},"ranks":ranks});
    let data = validate_issue(data)?;
    Ok(vec![CollectedItem {
        kind: "repo_issue".into(),
        key,
        data,
        sort_order: 0,
        published_at: None,
        source_name: "GitHub".into(),
        source_url: "https://github.com".into(),
    }])
}

/// 独立验证真实观测数据，避免旧手录 DTO 丢弃完整性与异常标识。
pub fn validate_issue(value: Value) -> AppResult<Value> {
    check_issue(&value).map_err(|_| bad())?;
    Ok(value)
}
fn check_issue(v: &Value) -> SourceResult<()> {
    if serde_json::to_vec(v).map_err(|_| invalid())?.len() > BODY_LIMIT
        || v["source"] != "GitHub"
        || v["source_url"] != "https://github.com"
        || v["metric"] != "observed net star change"
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
    if !(1..=7).contains(&n)
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
            .filter(|a| a.len() == dates.len() && (2..=8).contains(&a.len()))
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
        if rank["star_gain"].as_i64() != Some(gain)
            || rank["star_total"].as_i64() != Some(points.last().unwrap().1)
            || rank["complete"].as_bool() != Some(full)
            || rank["suspected_growth"].as_bool() != Some(suspicious)
        {
            return Err(invalid());
        }
        if let Some((prev, ref prev_id)) = previous {
            let id_str = id.to_string();
            if gain > prev || (gain == prev && prev_id >= &id_str) {
                return Err(invalid());
            }
        }
        previous = Some((gain, id.to_string()));
        total += gain;
        all_complete &= full;
        if full {
            for (i, w) in points.windows(2).enumerate() {
                daily[i] += w[1].1 - w[0].1;
            }
        }
    }
    if v["total_gain"].as_i64() != Some(total)
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
