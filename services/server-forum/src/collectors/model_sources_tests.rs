use super::*;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

fn now() -> DateTime<Utc> {
    "2026-05-12T12:00:00Z".parse().unwrap()
}
fn model(id: &str) -> Value {
    json!({"id":id,"slug":id,"name":"Fixture model","release_date":null,
        "model_creator":{"id":"vendor","name":"Fixture vendor"},
        "evaluations":{"artificial_analysis_intelligence_index":131.2,
            "artificial_analysis_coding_index":null,"artificial_analysis_agentic_index":null},
        "pricing":{"price_1m_input_tokens":1.5,"price_1m_output_tokens":null},
        "performance":{"median_output_tokens_per_second":80.0,"median_time_to_first_token_seconds":null}})
}
fn aa_page_fixture(page: u64, pages: u64, id: &str) -> Value {
    json!({"tier":"free","intelligence_index_version":5.1,
        "pagination":{"page":page,"page_size":200,"total_pages":pages,"has_more":page<pages},"data":[model(id)]})
}
fn usage_fixture(start: &str, end: &str, data: Vec<Value>) -> Value {
    json!({"data":data,"meta":{"start_date":start,"end_date":end,"as_of":"2026-05-12T02:00:00Z","version":"v1"}})
}
fn usage_day(date: &str) -> Vec<Value> {
    vec![
        json!({"date":date,"model_permaslug":"vendor/model:free","total_tokens":"9007199254740993"}),
        json!({"date":date,"model_permaslug":"vendor/model","total_tokens":"9007199254740992"}),
        json!({"date":date,"model_permaslug":"other","total_tokens":"12"}),
    ]
}
fn valid_usage() -> Value {
    let date = NaiveDate::from_ymd_opt(2026, 5, 11).unwrap();
    usage_items(
        usage_fixture("2026-05-11", "2026-05-11", usage_day("2026-05-11")),
        date,
        date,
        now(),
    )
    .unwrap()
    .remove(0)
    .data
}

#[test]
fn decimal_exact_add_sort_and_boundaries() {
    assert_eq!(
        decimal_add("9007199254740993", "9007199254740992"),
        "18014398509481985"
    );
    assert_eq!(
        decimal_add("99999999999999999999999999999999999999", "1"),
        "100000000000000000000000000000000000000"
    );
    assert_eq!(
        decimal_cmp("100000000000000000000", "99999999999999999999"),
        Ordering::Greater
    );
    for bad in [
        json!(12),
        json!(12.5),
        json!("1e20"),
        json!("-1"),
        json!("01"),
        json!(""),
        json!("NaN"),
        json!("1".repeat(81)),
    ] {
        assert!(decimal(&bad).is_err());
    }
}
#[test]
fn usage_preserves_big_counts_and_variants_and_excludes_other() {
    let v = valid_usage();
    assert_eq!(v["total_tokens"], "18014398509481997");
    assert_eq!(v["other_tokens"], "12");
    assert_eq!(v["models"].as_array().unwrap().len(), 2);
    assert_eq!(v["models"][0]["id"], "vendor/model:free");
    assert_eq!(v["models"][0]["tokens"], "9007199254740993");
    assert_eq!(v["models"][1]["rank"], 2);
    assert!(validate_model_usage(v).is_ok());
}
#[test]
fn usage_validates_order_totals_dates_links_and_shares() {
    for field in ["total_tokens", "other_tokens"] {
        let mut v = valid_usage();
        v[field] = json!("1");
        assert!(validate_model_usage(v).is_err());
    }
    for (field, value) in [
        ("tokens", json!("-1")),
        ("tokens", json!(9007199254740993_u64)),
        ("rank", json!(3)),
        ("share_pct", json!(101)),
        ("id", json!("other")),
        ("name", json!("x".repeat(201))),
    ] {
        let mut v = valid_usage();
        v["models"][0][field] = value;
        assert!(validate_model_usage(v).is_err(), "{field}");
    }
    for url in [
        "http://openrouter.ai/rankings",
        "https://openrouter.ai.evil.test/rankings",
        "https://x@openrouter.ai/rankings",
        "https://openrouter.ai:444/rankings",
        "https://openrouter.ai/rankings?secret=x",
    ] {
        let mut v = valid_usage();
        v["source_url"] = json!(url);
        assert!(validate_model_usage(v).is_err());
    }
    let mut v = valid_usage();
    v["models"].as_array_mut().unwrap().swap(0, 1);
    assert!(validate_model_usage(v).is_err());
    let mut v = valid_usage();
    v["models"][1]["id"] = v["models"][0]["id"].clone();
    assert!(validate_model_usage(v).is_err());
    for date in ["2026-5-11", "2026-02-30", "2099-01-01", "2026-05-12"] {
        let mut v = valid_usage();
        v["date"] = json!(date);
        assert!(validate_model_usage(v).is_err());
    }
}
#[test]
fn usage_missing_dates_duplicate_ids_and_unready_snapshot_fail() {
    let start = NaiveDate::from_ymd_opt(2026, 5, 9).unwrap();
    let end = start + Days::days(2);
    let raw = usage_fixture("2026-05-09", "2026-05-11", usage_day("2026-05-11"));
    assert!(usage_items(raw, start, end, now()).is_err());
    let mut rows = usage_day("2026-05-11");
    rows.push(rows[0].clone());
    let raw = usage_fixture("2026-05-11", "2026-05-11", rows);
    assert!(usage_items(raw, end, end, now()).is_err());
    let mut raw = usage_fixture("2026-05-11", "2026-05-11", usage_day("2026-05-11"));
    raw["meta"]["as_of"] = json!("2026-05-11T23:59:59Z");
    assert!(usage_items(raw, end, end, now()).is_err());
}
#[test]
fn usage_ties_sort_by_id_and_missing_other_means_empty_tail() {
    let end = NaiveDate::from_ymd_opt(2026, 5, 11).unwrap();
    let raw = usage_fixture(
        "2026-05-11",
        "2026-05-11",
        vec![
            json!({"date":"2026-05-11","model_permaslug":"z/model","total_tokens":"10"}),
            json!({"date":"2026-05-11","model_permaslug":"a/model","total_tokens":"10"}),
        ],
    );
    let v = usage_items(raw, end, end, now()).unwrap().remove(0).data;
    assert_eq!(v["other_tokens"], "0");
    assert_eq!(v["total_tokens"], "20");
    assert_eq!(v["models"][0]["id"], "a/model");
    assert_eq!(v["models"][0]["share_pct"], 50.0);
}
#[test]
fn aa_nulls_versions_units_and_identity_are_preserved() {
    let raw = aa_page_fixture(1, 1, "first");
    let v = aa_board(raw["data"].as_array().unwrap().clone(), &raw, now()).unwrap();
    assert_eq!(v["index_version"], "5.1");
    assert_eq!(v["models"][0]["metrics"]["intelligence"], 131.2);
    assert!(v["models"][0]["metrics"]["coding"].is_null());
    assert!(v["models"][0]["metrics"]["economics"].is_null());
    assert!(v["models"][0]["pricing"]["output"].is_null());
    assert!(v["vendors"][0]["country"].is_null());
    assert_eq!(v["models"][0]["pricing"]["input"], 1.5);
    assert!(validate_model_board(v).is_ok());
}
#[test]
fn aa_rejects_duplicate_identity_bad_measurements_missing_vendor_and_pagination() {
    let raw = aa_page_fixture(1, 1, "first");
    assert!(aa_board(vec![model("first"), model("first")], &raw, now()).is_err());
    let mut row = model("first");
    row["pricing"]["price_1m_input_tokens"] = json!(-1);
    assert!(aa_board(vec![row], &raw, now()).is_err());
    let mut row = model("first");
    row["model_creator"]["id"] = Value::Null;
    assert!(aa_board(vec![row], &raw, now()).is_err());
    for field in ["page", "total_pages", "page_size"] {
        let mut raw = aa_page_fixture(1, 1, "first");
        raw["pagination"][field] = json!(0);
        assert!(aa_page(&raw, 1).is_err());
    }
    let mut raw = aa_page_fixture(1, 1, "first");
    raw["pagination"]["has_more"] = json!(true);
    assert!(aa_page(&raw, 1).is_err());
    assert!(aa_page(&json!({"tier":"free","data":[]}), 1).is_err());
}

// No external network, real API key, or additional mocking dependency needed.
async fn mock(
    responses: Vec<(u16, String)>,
    delay: Duration,
) -> (String, tokio::task::JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let task = tokio::spawn(async move {
        let mut requests = Vec::new();
        for (status, body) in responses {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut bytes = vec![0; 8192];
            let n = stream.read(&mut bytes).await.unwrap();
            requests.push(String::from_utf8_lossy(&bytes[..n]).into_owned());
            tokio::time::sleep(delay).await;
            let response=format!("HTTP/1.1 {status} Mock\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len());
            let _ = stream.write_all(response.as_bytes()).await;
        }
        requests
    });
    (url, task)
}
fn client() -> Client {
    Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .no_proxy()
        .build()
        .unwrap()
}
#[tokio::test]
async fn aa_collect_paginates_with_sensitive_header_and_stable_identity() {
    let (url, task) = mock(
        vec![
            (200, aa_page_fixture(1, 2, "a").to_string()),
            (200, aa_page_fixture(2, 2, "b").to_string()),
        ],
        Duration::ZERO,
    )
    .await;
    let items = collect_at(
        "artificial_analysis",
        "fixture-key",
        &client(),
        &url,
        "",
        now(),
    )
    .await
    .unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].kind, "model_board");
    assert_eq!(items[0].key, "artificial-analysis");
    assert_eq!(items[0].data["models"].as_array().unwrap().len(), 2);
    assert!(items[0].published_at.is_none());
    let requests = task.await.unwrap();
    assert!(requests[0].contains("page=1"));
    assert!(requests[1].contains("page=2"));
    assert!(requests[0].contains("x-api-key: fixture-key"));
}
#[tokio::test]
async fn aa_collect_rejects_inconsistent_versions_between_pages() {
    let mut second = aa_page_fixture(2, 2, "b");
    second["intelligence_index_version"] = json!(6);
    let (url, task) = mock(
        vec![
            (200, aa_page_fixture(1, 2, "a").to_string()),
            (200, second.to_string()),
        ],
        Duration::ZERO,
    )
    .await;
    assert!(
        collect_at("artificial_analysis", "key", &client(), &url, "", now())
            .await
            .is_err()
    );
    task.await.unwrap();
}
#[tokio::test]
async fn usage_collect_requests_exact_three_complete_utc_days() {
    let rows = ["2026-05-09", "2026-05-10", "2026-05-11"]
        .into_iter()
        .flat_map(usage_day)
        .collect();
    let body = usage_fixture("2026-05-09", "2026-05-11", rows).to_string();
    let (url, task) = mock(vec![(200, body)], Duration::ZERO).await;
    let items = collect_at("openrouter", "fixture-key", &client(), "", &url, now())
        .await
        .unwrap();
    assert_eq!(items.len(), 3);
    assert_eq!(items[2].key, "2026-05-11");
    assert_eq!(items[2].kind, "model_usage");
    let requests = task.await.unwrap();
    assert!(requests[0].contains("period=day&start_date=2026-05-09&end_date=2026-05-11"));
    assert!(requests[0].contains("authorization: Bearer fixture-key"));
}
#[tokio::test]
async fn network_failures_are_classified_without_leaking_credentials_or_bodies() {
    for (status, expected) in [
        (401, "密钥无效"),
        (403, "权限不足"),
        (429, "限流"),
        (500, "服务请求失败"),
        (302, "跳转"),
    ] {
        let (url, task) = mock(
            vec![(status, "SECRET-UPSTREAM-BODY".into())],
            Duration::ZERO,
        )
        .await;
        let err = request(
            &client(),
            Url::parse(&url).unwrap(),
            "openrouter",
            "SECRET-KEY",
            TIMEOUT,
        )
        .await
        .unwrap_err();
        assert!(err.contains(expected));
        assert!(!err.contains("SECRET"));
        task.await.unwrap();
    }
    let (url, task) = mock(vec![(200, "secret-invalid-json".into())], Duration::ZERO).await;
    let err = request(
        &client(),
        Url::parse(&url).unwrap(),
        "openrouter",
        "SECRET-KEY",
        TIMEOUT,
    )
    .await
    .unwrap_err();
    assert!(!err.contains("secret"));
    task.await.unwrap();
}
#[tokio::test]
async fn timeout_and_size_limit_fail_cleanly() {
    let (url, task) = mock(vec![(200, "{}".into())], Duration::from_millis(100)).await;
    let err = request(
        &client(),
        Url::parse(&url).unwrap(),
        "openrouter",
        "key",
        Duration::from_millis(10),
    )
    .await
    .unwrap_err();
    assert!(err.contains("超时"));
    task.abort();
    let (url, task) = mock(vec![(200, "x".repeat(MAX_BODY + 1))], Duration::ZERO).await;
    assert!(request(
        &client(),
        Url::parse(&url).unwrap(),
        "openrouter",
        "key",
        TIMEOUT
    )
    .await
    .unwrap_err()
    .contains("大小限制"));
    task.await.unwrap();
}
#[tokio::test]
async fn unknown_source_and_invalid_header_rejected_without_network() {
    assert!(collect("unknown", "key", &client()).await.is_err());
    assert!(verify_credential("unknown", "key", &client())
        .await
        .is_err());
    assert!(request(
        &client(),
        Url::parse("http://127.0.0.1:1").unwrap(),
        "openrouter",
        "key\nINJECT",
        TIMEOUT
    )
    .await
    .unwrap_err()
    .contains("密钥无效"));
}
#[test]
fn utc_day_boundaries_do_not_depend_on_local_timezone() {
    let utc: DateTime<Utc> = "2026-05-12T00:00:00Z".parse().unwrap();
    let end = utc.date_naive() - Days::days(1);
    let start = end - Days::days(2);
    assert_eq!(
        usage_url(OR_ENDPOINT, start, end).unwrap().query().unwrap(),
        "period=day&start_date=2026-05-09&end_date=2026-05-11"
    );
}
