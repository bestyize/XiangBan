//! Adapters for the public SkillHub website's expert-pack and MCP catalogs.
//! These routes are observed website APIs, not yet listed in the Open API contract.
use super::{bytes, endpoint, json_url, valid_slug};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

pub fn valid_namespace(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 80
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}
pub fn browse(kind: &str, keyword: &str, page: u32, category: &str) -> Result<Value, String> {
    if keyword.chars().count() > 80 || !(1..=1000).contains(&page) || category.len() > 100 {
        return Err("invalid catalog query".into());
    }
    let route = match kind {
        "experts" => "/api/v1/skillsets",
        "connectors" => "/api/v1/mcp/servers",
        _ => return Err("invalid catalog".into()),
    };
    let page_text = page.to_string();
    let raw = json_url(&endpoint(
        route,
        &[
            ("page", &page_text),
            ("pageSize", "24"),
            ("keyword", keyword),
            (
                if kind == "experts" {
                    "scene"
                } else {
                    "category"
                },
                category,
            ),
        ],
    )?)?;
    normalize_page(kind, &raw, page)
}
pub fn normalize_page(kind: &str, raw: &Value, page: u32) -> Result<Value, String> {
    let key = if kind == "experts" {
        "skillSets"
    } else {
        "items"
    };
    let items = raw[key]
        .as_array()
        .ok_or("SkillHub catalog response changed; please retry later")?;
    let results = items
        .iter()
        .filter(|v| v["banned"] != true && v["published"] != 0)
        .map(|v| normalize(kind, v))
        .collect::<Vec<_>>();
    let total = raw["total"]
        .as_u64()
        .ok_or("SkillHub catalog total missing")?;
    Ok(json!({"results":results,"page":page,"total":total,"hasMore":u64::from(page)*24 < total}))
}
fn normalize(kind: &str, v: &Value) -> Value {
    json!({"slug":v["slug"],"name":if kind=="experts" {&v["displayName"]}else{&v["name"]},
        "description":v["summaryZh"].as_str().or(v["summary"].as_str()).unwrap_or(""),
        "category":v[if kind=="experts" {"scene"}else{"category"}],
        "iconUrl":v["iconUrl"],"skillCount":v["skills"].as_array().or(v["skillSlugs"].as_array()).map(Vec::len),
        "source":"SkillHub"})
}
pub fn pack(slug: &str) -> Result<Value, String> {
    if !valid_slug(slug) {
        return Err("invalid expert slug".into());
    }
    let v = json_url(&endpoint(&format!("/api/v1/skillsets/{slug}"), &[])?)?;
    if v["slug"] != slug || v["published"] == 0 {
        return Err("expert pack unavailable".into());
    }
    Ok(v)
}
pub fn connector(slug: &str) -> Result<Value, String> {
    if !valid_slug(slug) {
        return Err("invalid connector slug".into());
    }
    let v = json_url(&endpoint(&format!("/api/v1/mcp/servers/{slug}"), &[])?)?;
    if v["slug"] != slug || v["banned"] == true {
        return Err("connector unavailable".into());
    }
    let raw = bytes(
        &endpoint(&format!("/api/v1/mcp/servers/{slug}/readme"), &[])?,
        512 * 1024,
    )?;
    if let Some(expected) = v["readme"]["sha256"].as_str() {
        if format!("{:x}", Sha256::digest(&raw)) != expected {
            return Err("MCP README SHA256 mismatch".into());
        }
    }
    let readme = String::from_utf8(raw).map_err(|_| "MCP README is not UTF-8")?;
    let mut configs = Vec::new();
    for block in readme
        .split("~~~")
        .flat_map(|s| s.split("\u{60}\u{60}\u{60}"))
        .skip(1)
        .step_by(2)
    {
        let Some((lang, body)) = block.split_once('\n') else {
            continue;
        };
        if !matches!(lang.trim(), "json" | "") {
            continue;
        }
        if let Ok(value) = serde_json::from_str::<Value>(body) {
            if let Some(servers) = value["mcpServers"].as_object() {
                for (name, config) in servers {
                    configs.push(json!({"name":name,"config":config}));
                }
            }
        }
    }
    Ok(json!({"entry":normalize("connectors",&v),"readme":readme,"configs":configs}))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn catalog_formats_and_unavailable_entries() {
        let packs = json!({"skillSets":[{"slug":"test-pack","displayName":"测试","published":1,"skills":[{"slug":"one"}]}],"total":25});
        let v = normalize_page("experts", &packs, 1).unwrap();
        assert_eq!(v["results"][0]["name"], "测试");
        assert_eq!(v["results"][0]["skillCount"], 1);
        assert_eq!(v["hasMore"], true);
        assert!(normalize_page("experts", &json!({"unexpected":[]}), 1).is_err());
        let v=normalize_page("connectors",&json!({"items":[{"slug":"blocked","banned":true},{"slug":"ok","name":"可用"}],"total":2}),1).unwrap();
        assert_eq!(v["results"].as_array().unwrap().len(), 1);
    }
}
