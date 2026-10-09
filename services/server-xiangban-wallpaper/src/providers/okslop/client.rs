use super::dto::{OkslopPage, OkslopPhoto};
use crate::{error::AppError, models::Orientation, runtime::Upstream};
use reqwest::header::AUTHORIZATION;

/// The default of the official client library. Its README names `https://okslop.com/api/v1` instead;
/// the two could not be compared because the live API refused every unauthenticated request.
pub const BASE_URL: &str = "https://api.okslop.com/api/v1";

fn encode(value: &str) -> String {
    let mut out = String::new();
    for b in value.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

fn paging(page: u32) -> String {
    format!("page={}&per_page={}", page.max(1), UPSTREAM_PAGE_SIZE)
}

pub const UPSTREAM_PAGE_SIZE: u32 = 10;
pub const MAX_UPSTREAM_PAGES: u32 = 5;

#[derive(Debug, PartialEq, Eq)]
pub struct PagePlan {
    pub first: u32,
    pub count: u32,
    pub skip: usize,
    pub take: usize,
}

/// Which upstream pages make up one of our pages. The API hands out 10 images a page while we serve more, so
/// a page is stitched together from a few consecutive ones; `skip` drops the part before our page begins.
pub fn page_plan(page: u32, per_page: u32) -> PagePlan {
    let page = page.max(1) as usize;
    let want = per_page.max(1) as usize;
    let size = UPSTREAM_PAGE_SIZE as usize;
    let offset = (page - 1) * want;
    let first = offset / size;
    let last = (offset + want - 1) / size;
    let count = (last - first + 1).min(MAX_UPSTREAM_PAGES as usize);
    let skip = offset % size;
    PagePlan {
        first: first as u32 + 1,
        count: count as u32,
        skip,
        take: want.min(count * size - skip),
    }
}

pub fn search_url(base: &str, query: &str, page: u32, orientation: &Orientation) -> String {
    let mut url = format!("{}/search/photos?query={}&{}", base.trim_end_matches('/'), encode(query), paging(page));
    match orientation {
        Orientation::Landscape => url.push_str("&orientation=landscape"),
        Orientation::Portrait => url.push_str("&orientation=portrait"),
        Orientation::Squarish => url.push_str("&orientation=square"),
        Orientation::All => {}
    }
    url
}

pub fn list_url(base: &str, page: u32) -> String {
    format!("{}/photos?{}", base.trim_end_matches('/'), paging(page))
}

pub fn detail_url(base: &str, id: &str) -> String {
    format!("{}/photos/{}", base.trim_end_matches('/'), encode(id))
}

fn provider_error(message: String) -> AppError {
    AppError::Provider { provider: "okslop", message }
}

#[derive(Clone)]
pub struct OkslopClient {
    upstream: Upstream,
    base_url: String,
}

impl OkslopClient {
    pub fn new(upstream: Upstream) -> Self {
        Self { upstream, base_url: BASE_URL.to_string() }
    }

    pub fn is_configured(&self) -> bool {
        self.upstream.is_configured()
    }

    async fn get(&self, url: &str) -> Result<reqwest::Response, AppError> {
        self.upstream
            .send(|client, key| client.get(url).header(AUTHORIZATION, format!("Bearer {}", key.trim())).header("Accept", "application/json"))
            .await
    }

    async fn page(&self, url: &str) -> Result<Vec<OkslopPhoto>, AppError> {
        let res = self.get(url).await?;
        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            return Err(provider_error(format!("API returned {status}: {}", text.chars().take(200).collect::<String>())));
        }
        let page: OkslopPage = res.json().await.map_err(|e| provider_error(format!("Failed to parse response: {e}")))?;
        Ok(page.results)
    }

    pub async fn search_page(&self, query: &str, upstream_page: u32, orientation: &Orientation) -> Result<Vec<OkslopPhoto>, AppError> {
        self.page(&search_url(&self.base_url, query, upstream_page, orientation)).await
    }

    pub async fn list_page(&self, upstream_page: u32) -> Result<Vec<OkslopPhoto>, AppError> {
        self.page(&list_url(&self.base_url, upstream_page)).await
    }

    pub async fn detail(&self, id: &str) -> Result<Option<OkslopPhoto>, AppError> {
        let res = self.get(&detail_url(&self.base_url, id)).await?;
        if res.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }
        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            return Err(provider_error(format!("API returned {status}: {}", text.chars().take(200).collect::<String>())));
        }
        let photo: OkslopPhoto = res.json().await.map_err(|e| provider_error(format!("Failed to parse detail: {e}")))?;
        Ok(Some(photo))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_encodes_the_keyword_and_always_asks_for_the_free_tier_page_size() {
        let url = search_url("https://x.test/api/v1", "neon city 夜景&x=1", 2, &Orientation::All);
        assert_eq!(
            url,
            "https://x.test/api/v1/search/photos?query=neon%20city%20%E5%A4%9C%E6%99%AF%26x%3D1&page=2&per_page=10"
        );
    }

    #[test]
    fn square_is_spelled_square_because_squarish_makes_the_api_fail_with_a_500() {
        assert!(search_url(BASE_URL, "a", 1, &Orientation::Squarish).ends_with("&orientation=square"));
        assert!(search_url(BASE_URL, "a", 1, &Orientation::Portrait).ends_with("&orientation=portrait"));
        assert!(search_url(BASE_URL, "a", 1, &Orientation::Landscape).ends_with("&orientation=landscape"));
        assert!(!search_url(BASE_URL, "a", 1, &Orientation::All).contains("orientation"));
        assert!(!search_url(BASE_URL, "a", 1, &Orientation::Squarish).contains("squarish"));
    }

    #[test]
    fn paging_never_starts_before_the_first_page() {
        assert_eq!(list_url(BASE_URL, 0), format!("{BASE_URL}/photos?page=1&per_page=10"));
        assert_eq!(list_url(BASE_URL, 3), format!("{BASE_URL}/photos?page=3&per_page=10"));
    }

    #[test]
    fn a_trailing_slash_on_the_base_is_harmless() {
        assert_eq!(list_url("https://x.test/api/v1/", 1), "https://x.test/api/v1/photos?page=1&per_page=10");
    }

    #[test]
    fn the_id_is_encoded_so_it_cannot_change_the_path() {
        assert_eq!(detail_url(BASE_URL, "abc"), format!("{BASE_URL}/photos/abc"));
        assert_eq!(detail_url(BASE_URL, "../stats?x=1"), format!("{BASE_URL}/photos/..%2Fstats%3Fx%3D1"));
    }

    // Our callers ask for 24 items a page, the API gives 10, so a page is stitched together from several.
    #[test]
    fn a_page_of_24_is_built_from_three_upstream_pages() {
        let plan = page_plan(1, 24);
        assert_eq!((plan.first, plan.count, plan.skip, plan.take), (1, 3, 0, 24));
    }

    #[test]
    fn the_next_page_starts_in_the_middle_of_an_upstream_page() {
        let plan = page_plan(2, 24);
        assert_eq!((plan.first, plan.count, plan.skip, plan.take), (3, 3, 4, 24));
    }

    #[test]
    fn a_page_of_exactly_10_maps_to_one_upstream_page() {
        for page in 1..6 {
            let plan = page_plan(page, 10);
            assert_eq!((plan.first, plan.count, plan.skip, plan.take), (page, 1, 0, 10));
        }
    }

    #[test]
    fn page_zero_counts_as_page_one() {
        assert_eq!(page_plan(0, 10).first, 1);
    }

    #[test]
    fn a_huge_page_is_cut_so_one_request_never_costs_more_than_a_few_calls() {
        let plan = page_plan(1, 100);
        assert!(plan.count <= MAX_UPSTREAM_PAGES, "{}", plan.count);
        assert_eq!(plan.take, (plan.count * UPSTREAM_PAGE_SIZE) as usize);
        let later = page_plan(3, 100);
        assert!(later.count <= MAX_UPSTREAM_PAGES);
        assert!(later.skip + later.take <= (later.count * UPSTREAM_PAGE_SIZE) as usize);
    }
}
