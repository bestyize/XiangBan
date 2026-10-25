//! SkillHub 适配器：技能走 `/api/skills`，专家包走 `/api/v1/skillsets`，MCP 走 `/api/v1/mcp/servers`。
//! 字段与路由来自桌面客户端 `libs/agent-tools/skill-discovery` 与壁纸站 `ai_plugins.rs` 已验证的用法。
use super::*;
use sha2::{Digest, Sha256};

const DEFAULT_BASE: &str = "https://api.skillhub.cn";
const PAGE_SIZE: u64 = 24;

fn base(ctx: &Ctx, source: &SourceRow) -> Result<Url, String> {
    validate_url(source.config["base_url"].as_str().unwrap_or(DEFAULT_BASE), ctx.allow_private)
}

fn api_key(source: &SourceRow) -> Option<&str> {
    source.config["api_key"].as_str().filter(|s| !s.is_empty())
}

fn endpoint(base: &Url, route: &str, pairs: &[(&str, String)]) -> Result<Url, String> {
    let mut url = base.join(route).map_err(|e| e.to_string())?;
    let pairs: Vec<_> = pairs.iter().filter(|(_, v)| !v.is_empty()).collect();
    if !pairs.is_empty() {
        let mut q = url.query_pairs_mut();
        for (k, v) in pairs {
            q.append_pair(k, v);
        }
    }
    Ok(url)
}

fn get(ctx: &Ctx, source: &SourceRow, url: Url) -> reqwest::RequestBuilder {
    let req = ctx.http.get(url);
    match api_key(source) {
        Some(key) => req.header("X-API-Key", key),
        None => req,
    }
}

fn namespace_of(v: &Value) -> String {
    v["namespace"]["handle"]
        .as_str()
        .or(v["namespace"].as_str())
        .or(v["ownerName"].as_str())
        .filter(|s| valid_namespace(s))
        .unwrap_or("")
        .to_string()
}

fn https_icon(v: &Value) -> Option<String> {
    v["iconUrl"]
        .as_str()
        .and_then(|s| Url::parse(s).ok())
        .filter(|u| u.scheme() == "https" && u.username().is_empty() && u.password().is_none())
        .map(|u| u.to_string())
}

fn dependencies(v: &Value) -> Value {
    if let Some(list) = v["skills"].as_array().filter(|a| !a.is_empty()) {
        return Value::Array(
            list.iter()
                .filter_map(|s| {
                    let slug = s["slug"].as_str().filter(|s| valid_slug(s))?;
                    Some(json!({"slug": slug, "namespace": namespace_of(s), "version": s["version"]}))
                })
                .collect(),
        );
    }
    Value::Array(
        v["skillSlugs"]
            .as_array()
            .map(|a| a.iter().filter_map(|s| s.as_str().filter(|s| valid_slug(s))).map(|s| json!({"slug": s})).collect())
            .unwrap_or_default(),
    )
}

pub fn normalize(kind: &str, v: &Value) -> Option<Normalized> {
    let slug = v["slug"].as_str().filter(|s| valid_slug(s))?.to_string();
    let source_label = normalize_source_label(v["source"].as_str());
    let base = |namespace: String, upstream_id: String, name: String, summary: String, category: Option<String>| Normalized {
        upstream_id,
        kind: kind.to_string(),
        namespace,
        slug: slug.clone(),
        name,
        description: summary.clone(),
        summary,
        category,
        tags: vec![],
        icon_url: https_icon(v),
        license: first_text(v, &["license"]),
        origin_url: None,
        source: source_label.clone(),
        version: text(&v["version"]),
        metadata: json!({}),
        stats: json!({}),
    };
    Some(match kind {
        "skill" => {
            let ns = namespace_of(v);
            let upstream_id = if ns.is_empty() { slug.clone() } else { format!("{ns}/{slug}") };
            let mut n = base(
                if ns.is_empty() { "skillhub".into() } else { ns },
                upstream_id,
                first_text(v, &["name", "displayName"]).unwrap_or_else(|| slug.clone()),
                first_text(v, &["description_zh", "summary_zh", "description", "summary"]).unwrap_or_default(),
                text(&v["category"]),
            );
            n.tags = tags_of(&v["labels"]);
            n.stats = json!({
                "downloads": v["downloads"].as_u64().or(v["stats"]["downloads"].as_u64()).unwrap_or(0),
                "stars": v["stars"].as_u64().or(v["stats"]["stars"].as_u64()).unwrap_or(0),
            });
            n
        }
        "expert" => {
            let mut n = base(
                "skillhub".into(),
                slug.clone(),
                first_text(v, &["displayName", "name"]).unwrap_or_else(|| slug.clone()),
                first_text(v, &["summaryZh", "summary_zh", "summary"]).unwrap_or_default(),
                first_text(v, &["scene"]),
            );
            n.metadata = json!({ "dependencies": dependencies(v) });
            n
        }
        _ => {
            let mut n = base(
                "skillhub".into(),
                slug.clone(),
                first_text(v, &["name", "displayName"]).unwrap_or_else(|| slug.clone()),
                first_text(v, &["summaryZh", "summary_zh", "summary", "description"]).unwrap_or_default(),
                first_text(v, &["category"]),
            );
            n.tags = tags_of(&v["tags"]);
            n
        }
    })
}

pub async fn fetch_page(ctx: &Ctx, source: &SourceRow, params: &Params, page: u32) -> UpstreamResult<Page> {
    let base = base(ctx, source)?;
    let page_text = page.to_string();
    let (route, pairs): (&str, Vec<(&str, String)>) = match source.kind.as_str() {
        "skill" => (
            "/api/skills",
            vec![
                ("keyword", params.keyword.clone()),
                ("page", page_text),
                ("category", params.category.clone()),
                ("source", params.source.clone()),
                ("sortBy", params.sort_by.clone()),
            ],
        ),
        "expert" => (
            "/api/v1/skillsets",
            vec![
                ("page", page_text),
                ("pageSize", PAGE_SIZE.to_string()),
                ("keyword", params.keyword.clone()),
                ("scene", params.category.clone()),
            ],
        ),
        "connector" => (
            "/api/v1/mcp/servers",
            vec![
                ("page", page_text),
                ("pageSize", PAGE_SIZE.to_string()),
                ("keyword", params.keyword.clone()),
                ("category", params.category.clone()),
            ],
        ),
        other => return Err(UpstreamError::permanent(format!("SkillHub 不支持的类型：{other}"))),
    };
    let response = send_json(get(ctx, source, endpoint(&base, route, &pairs)?)).await?;
    let (list, total) = match source.kind.as_str() {
        "skill" => (response["data"]["skills"].as_array(), response["data"]["total"].as_u64()),
        "expert" => (response["skillSets"].as_array(), response["total"].as_u64()),
        _ => (response["items"].as_array(), response["total"].as_u64()),
    };
    let list = list.ok_or_else(|| UpstreamError::permanent("SkillHub 目录格式发生变化，请检查上游接口"))?;
    let mut entries = Vec::new();
    let mut skipped = 0;
    for v in list {
        let unavailable = v["banned"] == true || (source.kind == "expert" && v["published"] == 0);
        match (unavailable, normalize(&source.kind, v)) {
            (false, Some(normalized)) => entries.push(RawEntry { raw: v.clone(), normalized }),
            _ => skipped += 1,
        }
    }
    let has_more = !list.is_empty() && total.map(|t| u64::from(page) * PAGE_SIZE < t).unwrap_or(list.len() as u64 >= PAGE_SIZE);
    Ok(Page { entries, total, has_more, skipped, response })
}

fn split_id(upstream_id: &str) -> (&str, &str) {
    upstream_id.split_once('/').unwrap_or(("", upstream_id))
}

fn valid_version(s: &str) -> bool {
    !s.is_empty() && s.len() < 45 && s.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'+'))
}

/// 从 README 的代码块里提取 `mcpServers` 配置模板（只读展示，不执行）。
pub fn extract_configs(readme: &str) -> Value {
    let normalized = readme.replace("~~~", "```");
    let mut out = Vec::new();
    for block in normalized.split("```").skip(1).step_by(2) {
        let Some((lang, body)) = block.split_once('\n') else { continue };
        if !matches!(lang.trim(), "json" | "") {
            continue;
        }
        if let Ok(value) = serde_json::from_str::<Value>(body) {
            if let Some(servers) = value["mcpServers"].as_object() {
                for (name, config) in servers {
                    out.push(json!({"name": name, "config": config}));
                }
            }
        }
    }
    Value::Array(out)
}

pub async fn enrich(ctx: &Ctx, source: &SourceRow, n: &Normalized) -> UpstreamResult<Enrichment> {
    let base = base(ctx, source)?;
    let slug = &n.slug;
    match n.kind.as_str() {
        "skill" => {
            let (ns, _) = split_id(&n.upstream_id);
            let ns = ns.to_string();
            let detail = send_json(get(ctx, source, endpoint(&base, &format!("/api/v1/skills/{slug}"), &[("namespace", ns.clone())])?)).await?;
            if detail["skill"]["slug"] != json!(slug) || detail["skill"]["banned"] == true {
                return Err(UpstreamError::permanent("SkillHub 技能不可用或标识不一致"));
            }
            let version = detail["latestVersion"]["version"]
                .as_str()
                .filter(|v| valid_version(v))
                .ok_or_else(|| UpstreamError::permanent("SkillHub 缺少最新版本"))?
                .to_string();
            let changelog = detail["latestVersion"]["changelog"].as_str().unwrap_or("").to_string();
            let url = endpoint(&base, "/api/v1/download", &[("slug", slug.clone()), ("version", version.clone()), ("namespace", ns)])?;
            let bytes = fetch_object(ctx, url, api_key(source), crate::package::MAX_ZIP_BYTES).await?;
            Ok(Enrichment::Package { version, changelog, bytes })
        }
        "expert" => {
            let detail = send_json(get(ctx, source, endpoint(&base, &format!("/api/v1/skillsets/{slug}"), &[])?)).await?;
            if detail["slug"] != json!(slug) || detail["published"] == 0 {
                return Err(UpstreamError::permanent("专家包不可用"));
            }
            let content = detail["content"].as_str().filter(|s| !s.trim().is_empty()).ok_or_else(|| UpstreamError::permanent("专家工作流正文缺失"))?;
            if content.len() > 100 * 1024 {
                return Err(UpstreamError::permanent("专家工作流正文超过 100 KiB"));
            }
            if content.contains("register_expert.py") || content.contains("~/.workbuddy/") {
                return Err(UpstreamError::permanent("此专家包依赖 WorkBuddy 专属注册，与小伴不兼容"));
            }
            Ok(Enrichment::Expert { content: content.to_string(), dependencies: dependencies(&detail) })
        }
        _ => {
            let detail = send_json(get(ctx, source, endpoint(&base, &format!("/api/v1/mcp/servers/{slug}"), &[])?)).await?;
            if detail["slug"] != json!(slug) || detail["banned"] == true {
                return Err(UpstreamError::permanent("连接器不可用"));
            }
            // README 接口会 302 到腾讯 COS 的文件地址，必须走会处理重定向的下载函数。
            let raw = fetch_object(ctx, endpoint(&base, &format!("/api/v1/mcp/servers/{slug}/readme"), &[])?, api_key(source), 512 * 1024).await?;
            if let Some(expected) = detail["readme"]["sha256"].as_str() {
                if format!("{:x}", Sha256::digest(&raw)) != expected {
                    return Err(UpstreamError::permanent("MCP README 的 SHA-256 与上游声明不一致"));
                }
            }
            let readme = String::from_utf8(raw).map_err(|_| UpstreamError::permanent("MCP README 不是 UTF-8"))?;
            let configs = extract_configs(&readme);
            Ok(Enrichment::Connector { readme, configs })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skill_entries_keep_publisher_and_skip_bad_slugs() {
        let ok = json!({"slug":"pdf-toolkit","name":"PDF","description_zh":"处理 PDF","namespace":{"handle":"user_1"},"category":"docs","source":"official","labels":["pdf","表格"],"downloads":5});
        let n = normalize("skill", &ok).unwrap();
        assert_eq!((n.upstream_id.as_str(), n.namespace.as_str(), n.source.as_str()), ("user_1/pdf-toolkit", "user_1", "official"));
        assert_eq!(n.summary, "处理 PDF");
        assert_eq!(n.tags, vec!["pdf", "表格"]);
        assert_eq!(n.stats["downloads"], 5);
        assert!(normalize("skill", &json!({"slug":"Bad Slug"})).is_none());
    }

    #[test]
    fn expert_and_connector_entries() {
        let e = normalize("expert", &json!({"slug":"weekly","displayName":"周报","summaryZh":"写周报","scene":"办公","skills":[{"slug":"a-skill","namespace":"u1"}]})).unwrap();
        assert_eq!((e.name.as_str(), e.category.as_deref()), ("周报", Some("办公")));
        assert_eq!(e.metadata["dependencies"][0]["slug"], "a-skill");
        let c = normalize("connector", &json!({"slug":"github","name":"GitHub","summary":"仓库","category":"dev","tags":["git"]})).unwrap();
        assert_eq!(c.tags, vec!["git"]);
    }

    #[test]
    fn readme_configs_are_extracted_but_not_executed() {
        let readme = "说明\n```json\n{\"mcpServers\":{\"demo\":{\"command\":\"npx\",\"args\":[\"-y\",\"x\"]}}}\n```\n";
        let configs = extract_configs(readme);
        assert_eq!(configs[0]["name"], "demo");
        assert_eq!(configs[0]["config"]["command"], "npx");
        assert_eq!(extract_configs("没有代码块").as_array().unwrap().len(), 0);
    }
}
