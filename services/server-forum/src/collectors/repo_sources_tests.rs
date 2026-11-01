use super::*;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    task::JoinHandle,
};

async fn mock(path: &'static str, status: u16, value: Value) -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let origin = format!("http://{}", listener.local_addr().unwrap());
    let handle = tokio::spawn(async move {
        let (mut stream, _) =
            tokio::time::timeout(std::time::Duration::from_secs(5), listener.accept())
                .await
                .unwrap()
                .unwrap();
        let mut buffer = vec![0u8; 16384];
        let n = stream.read(&mut buffer).await.unwrap();
        let request = String::from_utf8_lossy(&buffer[..n]);
        assert_eq!(request.lines().next().unwrap().split_whitespace().nth(1), Some(path));
        assert!(request.to_ascii_lowercase().contains("authorization: bearer test-token"));
        let body = value.to_string();
        let response = format!("HTTP/1.1 {status} Response\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
        stream.write_all(response.as_bytes()).await.unwrap();
    });
    (origin, handle)
}

fn row(name: &str, description: &str, language: &str, color: &str, total: &str, gain: &str) -> String {
    let desc = if description.is_empty() {
        String::new()
    } else {
        format!(r#"<p class="col-9 color-fg-muted my-1 tmp-pr-4">
      {description}
    </p>"#)
    };
    let lang = if language.is_empty() {
        String::new()
    } else {
        format!(r#"<span class="tmp-mr-3 d-inline-block ml-0"><span class="repo-language-color" style="background-color: {color}"></span>
  <span itemprop="programmingLanguage">{language}</span></span>"#)
    };
    let (owner, repo) = name.split_once('/').unwrap();
    format!(r#"<article class="Box-row">
  <div class="float-right d-flex"><a href="/login?return_to=%2F{owner}%2F{repo}" class="btn-sm btn">Star</a>
    <a href="/sponsors/{owner}" class="btn">Sponsor</a></div>
  <h2 class="h3 lh-condensed"><a data-hydro-click="{{&quot;record_id&quot;:1}}" href="/{name}" class="Link"><svg class="octicon"><path d="M2 2.5Z"></path></svg>
      <span class="text-normal">
        {owner} /
</span>
      {repo}</a></h2>
  {desc}
  <div class="f6 color-fg-muted mt-2">{lang}
      <a href="/{name}/stargazers" class="Link"><svg class="octicon"><path d="M8 .25Z"></path></svg>
        {total}</a>
      <a href="/{name}/forks" class="Link"><svg><path d="M1Z"></path></svg>
        5,837</a>
      <span class="d-inline-block float-sm-right"><svg><path d="M8Z"></path></svg>
        {gain} stars this week
      </span></div>
</article>"#)
}
fn page(rows: &[String]) -> String {
    format!("<html><body><div class=\"Box\">{}</div></body></html>", rows.join("\n"))
}
fn sample() -> String {
    page(&[
        row("Nutlope/hallmark", "Anti-AI-slop design skill for Claude Code, Cursor, &amp; Codex.", "TypeScript", "#3178c6", "110,001", "12,062"),
        row("pbakaus/impeccable", "The &quot;design&quot; vocabulary &lt;for&gt; agents", "Rust", "#dea584", "98,898", "12,031"),
        row("ghostty-org/ghostty", "", "", "", "30,500", "556"),
        row("a-b/c.d_e", "Plain", "Go", "#00ADD8", "7", "7"),
        row("anthropics/skills", "Agent skills", "Python", "#3572A5", "2,389", "2,389"),
    ])
}

#[test]
fn trending_rows_use_heading_link_not_sponsor_button_and_decode_text() {
    let ranks = parse_trending(&sample()).unwrap();
    let names = ranks.iter().map(|r| r["full_name"].as_str().unwrap()).collect::<Vec<_>>();
    assert_eq!(names, ["Nutlope/hallmark", "pbakaus/impeccable", "ghostty-org/ghostty", "a-b/c.d_e", "anthropics/skills"]);
    assert_eq!(ranks[0]["description"], "Anti-AI-slop design skill for Claude Code, Cursor, & Codex.");
    assert_eq!(ranks[1]["description"], "The \"design\" vocabulary <for> agents");
    assert_eq!((ranks[0]["star_total"].as_i64(), ranks[0]["star_gain"].as_i64()), (Some(110001), Some(12062)));
    assert_eq!((ranks[1]["language"].as_str(), ranks[1]["language_color"].as_str()), (Some("Rust"), Some("#dea584")));
    assert_eq!((ranks[2]["language"].as_str(), ranks[2]["description"].as_str()), (Some(""), Some("")));
}

#[test]
fn long_descriptions_are_cut_at_a_fixed_length_with_an_ellipsis_and_still_validate() {
    let long = "word ".repeat(120);
    let page = page(&[
        row("a/one", &long, "Go", "#00ADD8", "1,000", "10"),
        row("a/two", &"字".repeat(DESCRIPTION_MAX), "Go", "#00ADD8", "1,000", "10"),
        row("a/three", "short", "Go", "#00ADD8", "1,000", "10"),
        row("a/four", "x", "Go", "#00ADD8", "1,000", "10"),
        row("a/five", "x", "Go", "#00ADD8", "1,000", "10"),
    ]);
    let ranks = parse_trending(&page).unwrap();
    let cut = ranks[0]["description"].as_str().unwrap();
    assert!(cut.chars().count() <= DESCRIPTION_MAX && cut.chars().count() > DESCRIPTION_MAX - 6, "{cut}");
    assert!(cut.ends_with('…') && !cut.trim_end_matches('…').ends_with(' '), "{cut}");
    assert_eq!(ranks[1]["description"].as_str().unwrap().chars().count(), DESCRIPTION_MAX, "刚好等于上限的不加省略号");
    assert!(!ranks[1]["description"].as_str().unwrap().ends_with('…'));
    assert!(build_issue(&page, Utc::now() - Duration::minutes(1)).is_ok());
}

#[test]
fn trending_gain_larger_than_total_is_dropped_and_structure_change_fails_whole_batch() {
    let mut rows = vec![row("bad/gain", "x", "Go", "#00ADD8", "10", "99")];
    rows.extend((0..6).map(|i| row(&format!("ok/repo{i}"), "x", "Go", "#00ADD8", "1,000", "10")));
    let ranks = parse_trending(&page(&rows)).unwrap();
    assert_eq!(ranks.len(), 6);
    assert!(ranks.iter().all(|r| r["full_name"] != "bad/gain"));
    let renamed = sample().replace("stars this week", "stars recently");
    assert!(parse_trending(&renamed).is_err());
    assert!(parse_trending("<html>blocked</html>").is_err());
    assert!(parse_trending(&page(&rows[..3])).is_err());
}

#[test]
fn trending_issue_is_a_valid_rolling_issue_with_page_order_and_total() {
    let now = Utc::now() - Duration::minutes(1);
    let item = build_issue(&sample(), now).unwrap().remove(0);
    let end = now.date_naive();
    assert_eq!((item.kind.as_str(), item.key.as_str()), ("repo_issue", format!("github-rolling-{end}").as_str()));
    assert_eq!(item.data["metric"], TRENDING_METRIC);
    assert_eq!(item.data["total_gain"], 12062 + 12031 + 556 + 7 + 2389);
    assert_eq!(item.data["observed_days"], 7);
    assert_eq!(item.data["week_start"], (end - Duration::days(7)).to_string());
    assert_eq!(item.data["ranks"][0]["full_name"], "Nutlope/hallmark");
    assert!(crate::discover::validate("repo_issue", item.data).is_ok());
}

#[test]
fn trending_validation_rejects_tampered_or_inconsistent_issues() {
    let now = Utc::now() - Duration::minutes(1);
    let good = build_issue(&sample(), now).unwrap().remove(0).data;
    let mutations: Vec<(&str, Box<dyn Fn(&mut Value)>)> = vec![
        ("total", Box::new(|v| v["total_gain"] = json!(1))),
        ("wow", Box::new(|v| v["wow_pct"] = json!(1.5))),
        ("source_url", Box::new(|v| v["source_url"] = json!("https://github.com"))),
        ("days", Box::new(|v| v["observed_days"] = json!(6))),
        ("future", Box::new(|v| {
            let next = (Utc::now() + Duration::days(2)).date_naive();
            v["observation_end"] = json!(next.to_string());
            v["week_end"] = json!(next.to_string());
        })),
        ("daily", Box::new(|v| v["daily_gain"] = json!([1, 2, 3, 4, 5, 6, 7]))),
        ("spark", Box::new(|v| v["ranks"][0]["spark"] = json!([1, 2]))),
        ("url", Box::new(|v| v["ranks"][0]["url"] = json!("https://evil.invalid/x"))),
        ("gain>total", Box::new(|v| v["ranks"][0]["star_total"] = json!(1))),
        ("negative", Box::new(|v| v["ranks"][0]["star_gain"] = json!(-1))),
        ("duplicate", Box::new(|v| v["ranks"][1]["full_name"] = json!("nutlope/HALLMARK"))),
        ("color", Box::new(|v| v["ranks"][0]["language_color"] = json!("red"))),
        ("empty", Box::new(|v| v["ranks"] = json!([]))),
        ("too many", Box::new(|v| {
            let first = v["ranks"][0].clone();
            v["ranks"] = json!((0..31).map(|_| first.clone()).collect::<Vec<_>>());
        })),
    ];
    for (name, mutate) in mutations {
        let mut value = good.clone();
        mutate(&mut value);
        assert!(validate_issue(value).is_err(), "{name}");
    }
}

#[test]
fn trending_sample_page_from_environment_parses_when_provided() {
    let Ok(path) = std::env::var("TRENDING_SAMPLE_HTML") else {
        eprintln!("skipped: TRENDING_SAMPLE_HTML is not set");
        return;
    };
    let html = std::fs::read_to_string(path).unwrap();
    let ranks = parse_trending(&html).unwrap();
    assert!(ranks.len() >= 5);
    assert!(build_issue(&html, Utc::now() - Duration::minutes(1)).is_ok());
}

#[tokio::test]
async fn github_auth_check_rejects_failure_and_anonymous_limits_without_leaking_the_token() {
    let http = super::super::credentials::http_client().unwrap();
    let (api, server) = mock("/rate_limit", 200, json!({"resources":{"core":{"limit":5000,"remaining":4999}}})).await;
    verify_at(&api, "test-token", &http).await.unwrap();
    server.await.unwrap();
    for (status, value) in [
        (401, json!({"message":"test-token"})),
        (200, json!({"resources":{"core":{"limit":60,"remaining":59}}})),
    ] {
        let (api, server) = mock("/rate_limit", status, value).await;
        let error = verify_at(&api, "test-token", &http).await.err().unwrap();
        assert!(!error.contains("test-token"));
        server.await.unwrap();
    }
}

#[test]
fn reviewed_production_baseline_retains_eight_exact_observations() {
    let value:Value=serde_json::from_str(include_str!("../../content/community-launch-20261031/research/repo-baseline-2026-10-31.json")).unwrap();
    let checked=crate::discover::validate("repo_issue",value).unwrap();
    assert_eq!(checked["ranks"].as_array().unwrap().len(),8);
    assert_eq!(checked["ranks"][0]["full_name"],"ollama/ollama");
    assert_eq!(checked["ranks"][0]["star_total"],185445);
    assert!(checked["ranks"].as_array().unwrap().iter().all(|r|r["star_gain"].is_null()));
}
