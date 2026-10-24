//! 配置驱动的 JSON 适配器，用于结构未知或会变化的上游（如魔搭社区的 MCP / 技能 / 智能体广场）。
//!
//! 来源 `config` 示例：
//! ```json
//! { "url": "https://example.com/api/list", "method": "POST",
//!   "body": { "page": "{page}", "page_size": "{page_size}", "keyword": "{keyword}" },
//!   "items_path": "/data/items", "total_path": "/data/total", "page_size": 20,
//!   "mapping": { "upstream_id": "/id", "name": "/name", "summary": "/desc", "tags": "/tags" } }
//! ```
//! `mapping` 的值是相对每个条目的 JSON 指针；数组表示按顺序取第一个非空值；以 `=` 开头表示字面量。
use super::*;
use serde_json::Map;

#[derive(Debug, Default, serde::Deserialize)]
#[serde(default)]
struct Cfg {
    url: String,
    method: String,
    query: Map<String, Value>,
    body: Value,
    headers: Map<String, Value>,
    items_path: String,
    total_path: String,
    page_size: u32,
    /// 上游页码的起点，缺省为 1；从 0 开始的接口需要显式填 0。
    page_start: Option<u32>,
    mapping: Map<String, Value>,
    origin_url_template: String,
    namespace: String,
    source_label: String,
    config_path: String,
}

fn cfg(source: &SourceRow) -> Result<Cfg, String> {
    let cfg: Cfg = serde_json::from_value(source.config.0.clone()).map_err(|e| format!("来源配置无效：{e}"))?;
    if cfg.url.trim().is_empty() {
        return Err("接口地址尚未配置：请先在来源配置里填写列表接口，并用「试采一页」确认字段映射".into());
    }
    if cfg.mapping.get("name").is_none() {
        return Err("字段映射缺少 name".into());
    }
    Ok(cfg)
}

struct Vars<'a> {
    page: u32,
    page_size: u32,
    keyword: &'a str,
    category: &'a str,
}

/// 把模板中的 `{page}` 等占位符替换成值；整串恰好是数字占位符时输出 JSON 数字。
fn fill(v: &Value, vars: &Vars) -> Value {
    match v {
        Value::String(s) => match s.as_str() {
            "{page}" => json!(vars.page),
            "{page_size}" => json!(vars.page_size),
            _ => Value::String(
                s.replace("{page}", &vars.page.to_string())
                    .replace("{page_size}", &vars.page_size.to_string())
                    .replace("{keyword}", vars.keyword)
                    .replace("{category}", vars.category),
            ),
        },
        Value::Array(a) => Value::Array(a.iter().map(|x| fill(x, vars)).collect()),
        Value::Object(o) => Value::Object(o.iter().map(|(k, x)| (k.clone(), fill(x, vars))).collect()),
        other => other.clone(),
    }
}

fn scalar(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

fn pick_one(item: &Value, rule: &str) -> Option<Value> {
    if let Some(lit) = rule.strip_prefix('=') {
        return Some(Value::String(lit.to_string()));
    }
    item.pointer(rule).filter(|v| !v.is_null() && !(v.is_string() && v.as_str().unwrap_or("").trim().is_empty())).cloned()
}

fn pick(item: &Value, rule: Option<&Value>) -> Option<Value> {
    match rule? {
        Value::String(s) => pick_one(item, s),
        Value::Array(rules) => rules.iter().filter_map(|r| r.as_str()).find_map(|r| pick_one(item, r)),
        _ => None,
    }
}

fn pick_text(item: &Value, cfg: &Cfg, key: &str) -> Option<String> {
    pick(item, cfg.mapping.get(key)).and_then(|v| text(&v))
}

fn normalize(source: &SourceRow, cfg: &Cfg, item: &Value) -> Option<Normalized> {
    let name = pick_text(item, cfg, "name")?;
    let upstream_id = pick_text(item, cfg, "upstream_id").or_else(|| pick_text(item, cfg, "slug")).unwrap_or_else(|| name.clone());
    let slug = slugify(&pick_text(item, cfg, "slug").unwrap_or_else(|| name.clone()), &upstream_id);
    let namespace = pick_text(item, cfg, "namespace")
        .map(|n| slugify(&n, &n))
        .or_else(|| Some(cfg.namespace.clone()).filter(|n| valid_namespace(n)))
        .unwrap_or_else(|| source.config["origin"].as_str().unwrap_or(&source.key).to_string());
    let summary = pick_text(item, cfg, "summary").unwrap_or_default();
    let description = pick_text(item, cfg, "description").unwrap_or_else(|| summary.clone());
    let origin_url = pick_text(item, cfg, "origin_url").or_else(|| {
        Some(cfg.origin_url_template.clone())
            .filter(|t| !t.is_empty())
            .map(|t| t.replace("{upstream_id}", &upstream_id).replace("{slug}", &slug))
    });
    let mut metadata = json!({});
    if !cfg.config_path.is_empty() {
        if let Some(v) = item.pointer(&cfg.config_path) {
            let parsed = match v {
                Value::String(s) => serde_json::from_str::<Value>(s).ok(),
                other => Some(other.clone()),
            };
            if let Some(parsed) = parsed {
                let servers = parsed["mcpServers"].as_object().cloned();
                let configs = match servers {
                    Some(map) => map.into_iter().map(|(name, config)| json!({"name": name, "config": config})).collect::<Vec<_>>(),
                    None => vec![json!({"name": slug, "config": parsed})],
                };
                metadata = json!({ "configs": configs });
            }
        }
    }
    Some(Normalized {
        upstream_id,
        kind: source.kind.clone(),
        namespace,
        slug,
        name,
        summary,
        description,
        category: pick_text(item, cfg, "category"),
        tags: pick(item, cfg.mapping.get("tags")).map(|v| tags_of(&v)).unwrap_or_default(),
        icon_url: pick_text(item, cfg, "icon_url").filter(|u| u.starts_with("https://")),
        license: pick_text(item, cfg, "license"),
        origin_url,
        source: normalize_source_label(Some(&cfg.source_label).filter(|s| !s.is_empty()).map(|s| s.as_str()).or(Some("community"))),
        version: pick_text(item, cfg, "version"),
        metadata,
        stats: json!({
            "downloads": pick(item, cfg.mapping.get("downloads")).and_then(|v| v.as_u64()).unwrap_or(0),
            "stars": pick(item, cfg.mapping.get("stars")).and_then(|v| v.as_u64()).unwrap_or(0),
        }),
    })
}

/// 试采用：逐个字段报告映射是否命中第一条数据，以及取到的样例值。
pub fn mapping_report(source: &SourceRow, item: &Value) -> Vec<Value> {
    let Ok(cfg) = cfg(source) else { return vec![] };
    cfg.mapping
        .iter()
        .map(|(field, rule)| {
            let sample = pick(item, Some(rule));
            json!({
                "field": field,
                "rule": rule,
                "hit": sample.is_some(),
                "sample": sample.map(|v| {
                    let s = scalar(&v);
                    if s.chars().count() > 80 { format!("{}…", s.chars().take(80).collect::<String>()) } else { s }
                }),
            })
        })
        .collect()
}

pub async fn fetch_page(ctx: &Ctx, source: &SourceRow, params: &Params, page: u32) -> UpstreamResult<Page> {
    let cfg = cfg(source)?;
    let url = validate_url(&cfg.url, ctx.allow_private)?;
    let page_size = if cfg.page_size == 0 { 20 } else { cfg.page_size.min(100) };
    let vars = Vars { page: cfg.page_start.unwrap_or(1) + page.saturating_sub(1), page_size, keyword: &params.keyword, category: &params.category };
    let method = if cfg.method.is_empty() { "GET".to_string() } else { cfg.method.to_uppercase() };
    let method = reqwest::Method::from_bytes(method.as_bytes()).map_err(|_| UpstreamError::permanent("不支持的请求方法"))?;
    if !matches!(method.as_str(), "GET" | "POST" | "PUT") {
        return Err(UpstreamError::permanent("请求方法只支持 GET / POST / PUT"));
    }
    let mut req = ctx.http.request(method.clone(), url).header("accept", "application/json");
    let query: Vec<(String, String)> = cfg
        .query
        .iter()
        .map(|(k, v)| (k.clone(), scalar(&fill(v, &vars))))
        .filter(|(_, v)| !v.is_empty())
        .collect();
    if !query.is_empty() {
        req = req.query(&query);
    }
    for (k, v) in &cfg.headers {
        if let Some(v) = v.as_str() {
            req = req.header(k.as_str(), v);
        }
    }
    if method != reqwest::Method::GET && !cfg.body.is_null() {
        req = req.json(&fill(&cfg.body, &vars));
    }
    let response = send_json(req).await?;
    let list = if cfg.items_path.is_empty() { Some(&response) } else { response.pointer(&cfg.items_path) }
        .and_then(|v| v.as_array())
        .ok_or_else(|| UpstreamError::permanent(format!("按 items_path「{}」没有找到条目数组，请检查配置", cfg.items_path)))?;
    let total = if cfg.total_path.is_empty() { None } else { response.pointer(&cfg.total_path).and_then(|v| v.as_u64()) };
    let mut entries = Vec::new();
    let mut skipped = 0;
    for item in list {
        match normalize(source, &cfg, item) {
            Some(normalized) => entries.push(RawEntry { raw: item.clone(), normalized }),
            None => skipped += 1,
        }
    }
    let seen = u64::from(page.saturating_sub(1)) * u64::from(page_size) + list.len() as u64;
    let has_more = !list.is_empty() && total.map(|t| seen < t).unwrap_or(list.len() as u64 >= u64::from(page_size));
    Ok(Page { entries, total, has_more, skipped, response })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(config: Value) -> SourceRow {
        SourceRow {
            id: 1,
            key: "demo".into(),
            name: "demo".into(),
            kind: "connector".into(),
            adapter: "generic_json".into(),
            enabled: true,
            config: Json(config),
            interval_minutes: 0,
            auto_import: "none".into(),
            last_job_at: None,
        }
    }

    #[test]
    fn mapping_supports_fallbacks_literals_and_connector_configs() {
        let src = source(json!({
            "url": "https://example.com/list", "namespace": "demo-ns", "origin_url_template": "https://example.com/{upstream_id}",
            "config_path": "/cfg",
            "mapping": {"upstream_id": "/id", "name": ["/zh_name", "/name"], "summary": "/desc", "tags": "/tags", "source": "=official", "category": "/cat"}
        }));
        let c = cfg(&src).unwrap();
        let item = json!({"id": "s-1", "zh_name": "", "name": "File Server", "desc": "读写文件", "tags": ["文件", {"name": "本地"}], "cat": "tools",
            "cfg": "{\"mcpServers\":{\"fs\":{\"command\":\"npx\"}}}"});
        let n = normalize(&src, &c, &item).unwrap();
        assert_eq!(n.name, "File Server");
        assert_eq!(n.upstream_id, "s-1");
        assert_eq!(n.slug, "file-server");
        assert_eq!(n.namespace, "demo-ns");
        assert_eq!(n.tags, vec!["文件", "本地"]);
        assert_eq!(n.origin_url.as_deref(), Some("https://example.com/s-1"));
        assert_eq!(n.metadata["configs"][0]["name"], "fs");
        assert!(normalize(&src, &c, &json!({"id": "x"})).is_none());
    }

    #[test]
    fn config_errors_are_actionable() {
        assert!(cfg(&source(json!({}))).unwrap_err().contains("接口地址尚未配置"));
        assert!(cfg(&source(json!({"url": "https://e.com"}))).unwrap_err().contains("name"));
    }

    #[test]
    fn templates_fill_numbers_and_text() {
        let vars = Vars { page: 3, page_size: 20, keyword: "a b", category: "" };
        let v = fill(&json!({"p": "{page}", "q": "k={keyword}", "n": {"s": "{page_size}"}}), &vars);
        assert_eq!(v, json!({"p": 3, "q": "k=a b", "n": {"s": 20}}));
    }

    #[test]
    fn chinese_names_get_stable_fallback_slugs() {
        let a = slugify("网页转换", "id-1");
        assert!(valid_slug(&a) && a.starts_with("x-"));
        assert_eq!(a, slugify("网页转换", "id-1"));
        assert_ne!(a, slugify("网页转换", "id-2"));
    }
}
