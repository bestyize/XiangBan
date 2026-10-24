//! Discover, review and install version-pinned SkillHub skills.
//! Third-party instructions and scripts are untrusted. Installing never executes code.
mod acquisition;
mod package;
mod runner;
pub mod catalog;
pub mod packs;
use agent_kernel::{Risk, Tool, ToolCtx, ToolSpec};
use package::{is_script, plan, Plan};
use reqwest::{blocking::Client, redirect::Policy, Url};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{fs, io::Read, net::IpAddr, path::PathBuf, sync::Arc, time::Duration};

const BASE: &str = "https://api.skillhub.cn";

fn valid_slug(s: &str) -> bool {
    s.len() > 1 && s.len() <= 64 && !s.starts_with('-') && !s.ends_with('-')
        && s.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
fn valid_version(s: &str) -> bool {
    !s.is_empty() && s.len() < 45
        && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-' || b == b'+')
}

fn client() -> Result<Client, String> {
    Client::builder().timeout(Duration::from_secs(30))
        // Objects live behind HTTPS redirects; never send to a literal private IP.
        .redirect(Policy::custom(|attempt| {
            let u = attempt.url();
            let host = u.host_str().unwrap_or("");
            if attempt.previous().len() >= 4 || u.scheme() != "https" || host.is_empty()
                || host.parse::<IpAddr>().is_ok()
                || host == "localhost" || host.ends_with(".local") || host.ends_with(".internal") {
                attempt.stop()
            } else {
                attempt.follow()
            }
        })).build().map_err(|e| e.to_string())
}
fn bytes(url: &str, max: u64) -> Result<Vec<u8>, String> {
    let mut response = client()?.get(url).send().map_err(|e| e.to_string())?;
    if !response.status().is_success() {
        return Err(format!("SkillHub HTTP {}", response.status()));
    }
    let mut result = Vec::new();
    response.take(max + 1).read_to_end(&mut result).map_err(|e| e.to_string())?;
    if result.len() as u64 > max {
        return Err("SkillHub response exceeds size limit".into());
    }
    Ok(result)
}
fn json_url(url: &str) -> Result<Value, String> {
    serde_json::from_slice(&bytes(url, 2 * 1024 * 1024)?).map_err(|e| e.to_string())
}
fn endpoint(route: &str, pairs: &[(&str, &str)]) -> Result<String, String> {
    Url::parse_with_params(&format!("{BASE}{route}"), pairs)
        .map(|url| url.to_string()).map_err(|e| e.to_string())
}
fn file_info(slug: &str, version: &str, namespace: Option<&str>) -> Result<Plan, String> {
    // Default resolution differs between SkillHub detail, file and download routes.
    // Resolve once, then use the exact publisher on every subsequent request.
    let detail=json_url(&endpoint(&format!("/api/v1/skills/{slug}"),&[("namespace",namespace.unwrap_or(""))])?)?;
    let namespace=canonical_namespace(&detail,slug,namespace)?;
    let url = endpoint(&format!("/api/v1/skills/{slug}/files"), &[("version", version), ("namespace", &namespace)])?;
    let result = json_url(&url)?;
    if result["version"].as_str() != Some(version) {
        return Err("SkillHub version changed unexpectedly".into());
    }
    if result["namespace"]["handle"]!=namespace {return Err("SkillHub file manifest publisher mismatch".into());}
    let files = result["files"].as_array().ok_or("SkillHub file list is missing")?;
    validate_registry_file_count(&result, files.len())?;
    let mut plan = plan(version, files)?;
    plan.namespace = Some(namespace.clone());
    let description=detail["skill"]["summary_zh"].as_str().filter(|s|!s.trim().is_empty())
        .or(detail["skill"]["summary"].as_str()).filter(|s|!s.trim().is_empty()).ok_or("SkillHub description missing")?;
    plan.metadata=Some(json!({"name":slug,"description":description}));
    plan.icon_url=detail["skill"]["iconUrl"].as_str().filter(|s|s.starts_with("https://")).map(str::to_string);
    // Pin registry metadata too: headerless legacy documents keep their original bytes.
    let binding=json!({"namespace":namespace,"metadata":plan.metadata});
    plan.sha256=format!("{:x}",Sha256::digest(format!("{}\0{}",plan.sha256,binding).as_bytes()));
    Ok(plan)
}
// SkillHub promises a complete list plus a count. Refuse incomplete manifests:
 // installing only the returned subset would break review and hash pinning.
fn validate_registry_file_count(manifest: &Value, received: usize) -> Result<(), String> {
    if let Some(raw) = manifest.get("count") {
        let declared = raw.as_u64().ok_or("SkillHub reported an invalid file count")?;
        if declared != received as u64 {
            return Err(format!(
                "SkillHub file manifest is incomplete: server reports {declared} files, but returned {received}"
            ));
        }
    }
    Ok(())
}

fn canonical_namespace(detail:&Value,slug:&str,requested:Option<&str>)->Result<String,String>{
    let namespace=detail["namespace"]["handle"].as_str().filter(|s|catalog::valid_namespace(s))
        .ok_or("SkillHub publisher missing or invalid")?;
    if detail["skill"]["slug"]!=slug || requested.is_some_and(|s|s!=namespace){
        return Err("SkillHub metadata identity mismatch".into());
    }
    Ok(namespace.to_string())
}
fn content(slug: &str, version: &str, namespace: Option<&str>, path: &str, max: u64) -> Result<Vec<u8>, String> {
    let url = endpoint(&format!("/api/v1/skills/{slug}/file"),
        &[("version", version), ("path", path), ("namespace", namespace.unwrap_or(""))])?;
    bytes(&url, max)
}
fn validate_skill(slug: &str, raw: &[u8], expected: &str) -> Result<String, String> {
    let hash = format!("{:x}", Sha256::digest(raw));
    if hash != expected {
        return Err("SkillHub SKILL.md SHA256 mismatch".into());
    }
    let text = String::from_utf8(raw.to_vec()).map_err(|_| "SKILL.md is not UTF-8")?;
    let metadata = agent_kernel::skills::parse_skill_metadata(&text)?;
    if metadata.name != slug {
        return Err(format!(
            "SKILL.md frontmatter name {:?} does not match SkillHub slug {:?}",
            metadata.name, slug
        ));
    }
    Ok(text)
}

fn validate_registry_skill(slug:&str,raw:&[u8],expected:&str,metadata:Option<&Value>)->Result<String,String>{
    if format!("{:x}",Sha256::digest(raw))!=expected{return Err("SkillHub SKILL.md SHA256 mismatch".into());}
    let text=std::str::from_utf8(raw).map_err(|_|"SKILL.md is not UTF-8")?;
    if text.trim_start_matches('\u{feff}').starts_with("---") {
        return validate_skill(slug,raw,expected);
    }
    let metadata=metadata.ok_or("SKILL.md has no frontmatter and no pinned registry metadata")?;
    if metadata["name"]!=slug || metadata["description"].as_str().is_none_or(|s|s.trim().is_empty()) || text.trim().is_empty(){
        return Err("SKILL.md requires valid pinned registry metadata".into());
    }
    Ok(text.to_string())
}


const SKILL_SORTS: &[&str] = &["score", "downloads", "stars", "installs", "updated_at"];

fn valid_category(key: &str) -> bool {
    key.len() <= 80 && key.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
fn validate_browse_filters(category: &str, source: &str, sort_by: &str) -> Result<(), String> {
    if !valid_category(category) || !matches!(source, "" | "official" | "community")
        || !SKILL_SORTS.contains(&sort_by) {
        return Err("invalid SkillHub category, source or sort".into());
    }
    Ok(())
}
fn normalize_skill_categories(raw: &Value) -> Result<Value, String> {
    let entries = raw["items"].as_array().ok_or("SkillHub categories response is invalid")?;
    let mut result = Vec::new();
    for item in entries {
        if item["active"] == false || item["level"].as_u64() != Some(1) { continue; }
        let key = item["key"].as_str().ok_or("SkillHub category key missing")?;
        let name = item["name"].as_str().ok_or("SkillHub category name missing")?;
        if key.is_empty() || !valid_category(key) || name.trim().is_empty() {
            return Err("SkillHub category dictionary contains invalid entries".into());
        }
        result.push(json!({"key": key, "name": name}));
    }
    if result.is_empty() { return Err("SkillHub returned no active categories".into()); }
    Ok(json!({"results": result}))
}

/// Searches the product's own catalog (keyword, page, sort). `Ok(None)` means that server has no such route, and only then
/// does `skill_search` go to SkillHub directly; any other failure is reported, as in the AI plugin page.
pub type CatalogSearch = Arc<dyn Fn(&str, u32, &str) -> Result<Option<Value>, String> + Send + Sync>;

pub struct SkillDiscovery { root: PathBuf, workspace: PathBuf, gateway: Option<CatalogSearch> }
impl SkillDiscovery {
    pub fn new(root: PathBuf, workspace: PathBuf) -> Self { Self { root, workspace, gateway: None } }
    /// Makes `skill_search` ask the product's own catalog first.
    pub fn with_catalog(mut self, search: CatalogSearch) -> Self { self.gateway = Some(search); self }
    /// Product UI browsing; no model invocation or installation is involved.
    pub fn browse(keyword: &str, page: u32, category: &str, source: &str, sort_by: &str) -> Result<Value, String> {
        let keyword = keyword.trim();
        if keyword.chars().count() > 80 || !(1..=1000).contains(&page) {
            return Err("invalid SkillHub query/page".into());
        }
        validate_browse_filters(category, source, sort_by)?;
        let page_text = page.to_string();
        // Category and source are independent API filters, never concatenated to a search term.
        let url = endpoint("/api/skills", &[("keyword", keyword), ("page", &page_text),
            ("pageSize", "24"), ("category", category), ("source", source),
            ("sortBy", sort_by), ("order", "desc")])?;
        let result = json_url(&url)?;
        browse_result(&result, page)
    }
    /// Read live publisher categories; category IDs may change upstream.
    pub fn categories() -> Result<Value, String> {
        normalize_skill_categories(&json_url(&endpoint("/api/v1/categories", &[])?)?)
    }
    /// UI callers must display this review before offering an install confirmation.
    pub fn inspect(&self, slug: &str, version: &str) -> Result<Value, String> {
        let raw = self.ui_tool("skill_inspect").run(json!({"slug":slug,"version":version}), &|| false)?;
        serde_json::from_str(&raw).map_err(|e| e.to_string())
    }
    pub fn inspect_coordinate(&self, slug: &str, version: &str, namespace: Option<&str>) -> Result<Value, String> {
        let raw = self.ui_tool("skill_inspect").run(json!({"slug":slug,"version":version,"namespace":namespace}), &|| false)?;
        serde_json::from_str(&raw).map_err(|e|e.to_string())
    }
    // Install revalidation only needs the pinned manifest; reviewed script previews
    // need not be downloaded again. All actual package bytes are checked on install.
    pub(crate) fn coordinate_manifest(&self,slug:&str,version:&str,namespace:Option<&str>)->Result<Value,String>{
        if !valid_slug(slug)||!valid_version(version)||namespace.is_some_and(|s|!catalog::valid_namespace(s)){
            return Err("invalid SkillHub coordinate".into());
        }
        let package=file_info(slug,version,namespace)?;
        Ok(json!({"slug":slug,"namespace":package.namespace,"manifest":package.summary(),"sha256":package.sha256}))
    }
    pub fn install_coordinate(&self, slug: &str, version: &str, namespace: Option<&str>, sha256: &str) -> Result<(), String> {
        self.ui_tool("skill_install").run(json!({"slug":slug,"version":version,"namespace":namespace,"sha256":sha256}), &|| false)?;
        Ok(())
    }
    /// Only call after a user confirms the exact reviewed version and manifest hash.
    /// This shares all package validation with the Agent tool and never runs scripts.
    pub fn install_reviewed(&self, slug: &str, version: &str, sha256: &str) -> Result<(), String> {
        self.ui_tool("skill_install").run(json!({"slug":slug,"version":version,"sha256":sha256}), &|| false)?;
        Ok(())
    }
    fn ui_tool(&self, name: &str) -> DiscoveryTool {
        DiscoveryTool { root: self.root.clone(), spec: ToolSpec::new(name, "", json!({"type":"object"})), gateway: None }
    }
    pub fn acquisition_policy(&self) -> acquisition::AcquisitionPolicy {
        acquisition::AcquisitionPolicy::default()
    }
    pub fn tools(&self) -> Vec<Arc<dyn Tool>> {
        let mut tools: Vec<Arc<dyn Tool>> = ["skill_search", "skill_inspect", "skill_install"]
            .into_iter().map(|name| {
                let schema = match name {
                    "skill_search" => json!({"type":"object","properties":{
                        "keyword":{"type":"string","minLength":2,"maxLength":80},
                        "page":{"type":"integer","minimum":1,"maximum":5,"description":"Result page, 1–5. Review the first page before fetching more."},
                        "sortBy":{"type":"string","enum":["score","downloads","stars","installs","updated_at"]},
                        "reason":{"type":"string","enum":["capability_gap","repeated_failure"],"description":"For proactive searches only; omit when user explicitly asks for Skill discovery."},
                        "gap":{"type":"string","description":"Specific capability unavailable among installed tools and skills; required for proactive discovery."},
                        "alternatives":{"type":"string","description":"Existing tools/skills considered and why they cannot solve the task; required for proactive discovery."}
                    },"required":["keyword"]}),
                    "skill_inspect" => json!({"type":"object","properties":{
                        "slug":{"type":"string"},"version":{"type":"string"},"namespace":{"type":"string"}},"required":["slug","version"]}),
                    _ => json!({"type":"object","properties":{
                        "slug":{"type":"string"},"version":{"type":"string"},"namespace":{"type":"string"},
                        "sha256":{"type":"string","minLength":64,"maxLength":64}},
                        "required":["slug","version","sha256"]})
                };
                let spec = ToolSpec::new(name,
                    "Search, review or install a pinned SkillHub package. Review its complete file list and script previews before installing. Installing does not execute scripts.",
                    schema)
                    .risk(if name == "skill_install" { Risk::Write } else { Risk::Network });
                let spec = if name == "skill_install" { spec.always_ask() }
                    else { spec.untrusted_output() };
                Arc::new(DiscoveryTool { root: self.root.clone(), spec, gateway: self.gateway.clone() }) as Arc<dyn Tool>
            }).collect();
        tools.push(Arc::new(runner::SkillRead::new(self.root.clone())));
        tools.push(Arc::new(runner::SkillRun::new(self.root.clone(), self.workspace.clone())));
        tools
    }
}
/// The product catalog's answer, in the shape `skill_search` returns from SkillHub.
fn gateway_results(found: &Value, page: u64) -> Result<String, String> {
    let items = found["results"].as_array().ok_or("invalid catalog search response")?;
    Ok(json!({"provider":"skillhub","page":page,"total":found["total"].as_u64(),
        "hasMore":found["hasMore"].as_bool().unwrap_or(false),
        "results":items.iter().take(20).map(|s|json!({
            "slug":s["slug"],"name":s["name"],"version":s["version"],"namespace":s["namespace"],
            "source":s["source"],"description":s["description"],"homepage":s["homepage"]
        })).collect::<Vec<_>>()}).to_string())
}
struct DiscoveryTool { spec: ToolSpec, root: PathBuf, gateway: Option<CatalogSearch> }
impl Tool for DiscoveryTool {
    fn spec(&self) -> &ToolSpec { &self.spec }
    fn execute(&self, args: Value, ctx: &ToolCtx) -> Result<String, String> {
        self.run(args, &|| ctx.is_cancelled())
    }
}
impl DiscoveryTool {
    fn run(&self, args: Value, cancelled: &dyn Fn() -> bool) -> Result<String, String> {
        if cancelled() { return Err("cancelled".into()); }
        if self.spec.name == "skill_search" {
            let keyword = args["keyword"].as_str().ok_or("keyword required")?.trim();
            if keyword.len() < 2 || keyword.len() > 80 { return Err("invalid keyword".into()); }
            let page = args.get("page").map(Value::as_u64).unwrap_or(Some(1))
                .filter(|p| (1..=5).contains(p)).ok_or("Skill search page must be 1–5")?;
            let sort_by = args["sortBy"].as_str().unwrap_or("score");
            validate_browse_filters("", "", sort_by)?;
            if let Some(search) = &self.gateway {
                if let Some(found) = search(keyword, page as u32, sort_by)? { return gateway_results(&found, page); }
            }
            let page_text = page.to_string();
            let url = endpoint("/api/skills", &[("keyword", keyword), ("page", &page_text),
                ("pageSize", "20"), ("sortBy", sort_by), ("order", "desc")])?;
            let j = json_url(&url)?;
            let items = j["data"]["skills"].as_array().ok_or("invalid SkillHub search response")?;
            let total = j["data"]["total"].as_u64();
            return Ok(json!({"provider":"skillhub","page":page,"total":total,
                "hasMore":total.map(|n| page*20<n).unwrap_or(items.len()>=20),
                "results":items.iter().take(20).map(|s|json!({
                    "slug":s["slug"],"name":s["name"],"version":s["version"],
                    "namespace":skill_namespace(s),"source":s["source"],
                    "description":s["description_zh"].as_str().or(s["description"].as_str()).unwrap_or(""),
                    "homepage":s["homepage"]
                })).collect::<Vec<_>>()}).to_string());
        }
        let slug = args["slug"].as_str().ok_or("slug required")?;
        let version = args["version"].as_str().ok_or("version required")?;
        if !valid_slug(slug) || !valid_version(version) {
            return Err("invalid SkillHub slug/version".into());
        }
        let namespace = args["namespace"].as_str().filter(|s| !s.is_empty());
        if namespace.is_some_and(|s| !catalog::valid_namespace(s)) { return Err("invalid SkillHub namespace".into()); }
        let package = file_info(slug, version, namespace)?;
        let namespace = package.namespace.as_deref();
        if self.spec.name == "skill_inspect" {
            let skill_file = package.files.iter().find(|f| f.path == "SKILL.md").ok_or("SKILL.md missing")?;
            let markdown = validate_registry_skill(slug,
                &content(slug, version, namespace, "SKILL.md", 100 * 1024)?, &skill_file.sha256, package.metadata.as_ref())?;
            let mut scripts = Vec::new();
            for item in package.files.iter().filter(|f| is_script(&f.path)) {
                if cancelled() { return Err("cancelled".into()); }
                // Truncated display is not a security audit. Every byte is still pinned by SHA256.
                let preview = if item.size <= 1024 * 1024 {
                    match content(slug, version, namespace, &item.path, 1024 * 1024) {
                        Ok(file) if format!("{:x}", Sha256::digest(&file)) == item.sha256 =>
                            String::from_utf8(file).ok().map(|s| s.chars().take(8000).collect::<String>()),
                        _ => None,
                    }
                } else { None };
                scripts.push(json!({"path":item.path,"sha256":item.sha256,
                    "preview":preview,"previewTruncated":item.size > 8000,
                    "warning":"Untrusted executable code; inspect source and approve each run."}));
            }
            return Ok(json!({"provider":"skillhub","slug":slug,
                "namespace":namespace,"manifest":package.summary(),"sha256":package.sha256,"content":markdown,"scripts":scripts,
                "requiresInstallationApproval":true,"requiresExecutionApproval":true,
                "warning":"SHA256 checks integrity, not safety. Third-party skills run with local user permissions; no sandbox."}).to_string());
        }
        let expected = args["sha256"].as_str().ok_or("expected manifest SHA256 required")?;
        if expected.len() != 64 || !expected.bytes().all(|b| b.is_ascii_hexdigit())
            || package.sha256 != expected.to_ascii_lowercase() {
            return Err("SkillHub file manifest differs from the reviewed SHA256".into());
        }
        if cancelled() { return Err("cancelled".into()); }
        let url = endpoint("/api/v1/download", &[("slug", slug), ("version", version), ("namespace", namespace.unwrap_or(""))])?;
        let files=match bytes(&url,12*1024*1024).and_then(|archive|package::verified_files(slug,&package,&archive)){
            Ok(files)=>files,
            Err(zip_error)=>package::canonical_files(&package,|item|{
                if cancelled(){return Err("cancelled".into());}
                content(slug,version,namespace,&item.path,item.size)
            }).map_err(|e|format!("ZIP unavailable or inconsistent ({zip_error}); canonical download failed: {e}"))?,
        };
        if cancelled() { return Err("cancelled".into()); }
        // Validate original SKILL.md metadata before materializing any file.
        let skill_file = package.files.iter().find(|f| f.path == "SKILL.md").ok_or("SKILL.md missing")?;
        validate_registry_skill(slug, files.get("SKILL.md").ok_or("SKILL.md missing")?, &skill_file.sha256, package.metadata.as_ref())?;
        package::install(&self.root, slug, &package, &files)?;
        Ok(format!("Installed SkillHub {slug}@{version}: {} verified files (including {} scripts); manifest SHA256={}",
            package.files.len(), package.files.iter().filter(|f| is_script(&f.path)).count(), package.sha256))
    }
}
fn skill_namespace(skill: &Value) -> Option<&str> {
    skill["namespace"]["handle"].as_str().filter(|s| catalog::valid_namespace(s))
        .or_else(|| skill["namespace"].as_str().filter(|s| catalog::valid_namespace(s)))
        .or_else(|| skill["ownerName"].as_str().filter(|s| catalog::valid_namespace(s)))
}
fn browse_result(result: &Value, page: u32) -> Result<Value, String> {
    let items = result["data"]["skills"].as_array().ok_or("invalid SkillHub browse response")?;
    let total = result["data"]["total"].as_u64();
    let results = items.iter().take(24).map(|s| json!({
        "slug":s["slug"], "name":s["name"], "version":s["version"],"namespace":skill_namespace(s),
        "iconUrl":s["iconUrl"],"source":s["source"],"downloads":s["downloads"],"stars":s["stars"],
        "description":s["description_zh"].as_str().or(s["description"].as_str()).unwrap_or("")
    })).collect::<Vec<_>>();
    Ok(json!({"results":results, "page":page, "total":total,
        "hasMore":total.map(|n| u64::from(page)*24 < n).unwrap_or(items.len() >= 24)}))
}
#[cfg(test)]
mod tests {
    use super::*;
    fn search_tool(search: CatalogSearch) -> Arc<dyn Tool> {
        let found = SkillDiscovery::new(std::env::temp_dir(), std::env::temp_dir()).with_catalog(search).tools();
        found.into_iter().find(|t| t.spec().name == "skill_search").unwrap()
    }
    #[test]
    fn the_product_catalog_answers_skill_search_first_and_its_failures_are_reported() {
        let seen = Arc::new(std::sync::Mutex::new(Vec::new()));
        let log = seen.clone();
        let tool = search_tool(Arc::new(move |keyword, page, sort| {
            log.lock().unwrap().push(format!("{keyword}|{page}|{sort}"));
            Ok(Some(json!({"results":[{"slug":"docx-helper","name":"Word 文档","version":"1.2.0","namespace":"acme","description":"处理 Word","iconUrl":"https://x/y.png"}],"total":1,"hasMore":false})))
        }));
        let out: Value = serde_json::from_str(&tool.execute(json!({"keyword":"word","page":2,"sortBy":"downloads"}), &agent_kernel::ToolCtx::detached("s")).unwrap()).unwrap();
        assert_eq!(out["results"][0]["slug"], "docx-helper");
        assert_eq!((out["page"].as_u64(), out["total"].as_u64(), out["hasMore"].as_bool()), (Some(2), Some(1), Some(false)));
        assert!(out["results"][0].get("iconUrl").is_none());
        assert_eq!(seen.lock().unwrap().clone(), vec!["word|2|downloads"]);
        // A catalog that is down is an error, not a silent switch to SkillHub.
        let down = search_tool(Arc::new(|_, _, _| Err("暂时无法连接小伴插件市场".into())));
        assert_eq!(down.execute(json!({"keyword":"word"}), &agent_kernel::ToolCtx::detached("s")).unwrap_err(), "暂时无法连接小伴插件市场");
        assert!(gateway_results(&json!({"error":"x"}), 1).is_err());
    }
    #[test]
    fn rejects_incomplete_or_corrupt_registry_file_counts() {
        let full=json!({"count":2,"files":[{"path":"SKILL.md"},{"path":"references/readme.md"}]});
        assert!(validate_registry_file_count(&full,2).is_ok());
        assert!(validate_registry_file_count(&full,1).unwrap_err().contains("incomplete"));
        assert!(validate_registry_file_count(&json!({"count":"2"}),2).unwrap_err().contains("invalid"));
        // Older registries may omit count; the package validator still requires SKILL.md.
        assert!(validate_registry_file_count(&json!({"files":[]}),0).is_ok());
    }

    #[test]
    fn categories_use_real_registry_ids_and_skip_inactive_entries() {
        let raw=json!({"items":[
            {"key":"office-efficiency","name":"办公效率","level":1,"active":true},
            {"key":"content-creation","name":"内容创作","level":1,"active":true},
            {"key":"obsolete","name":"停用","level":1,"active":false},
            {"key":"sub","name":"二级","level":2,"active":true}]});
        let v=normalize_skill_categories(&raw).unwrap();
        assert_eq!(v["results"].as_array().unwrap().len(),2);
        assert_eq!(v["results"][0]["key"],"office-efficiency");
        assert!(normalize_skill_categories(&json!({"items":[]})).is_err());
    }
    #[test]
    fn skill_search_filters_and_pagination_are_independent() {
        assert!(validate_browse_filters("office-efficiency","official","score").is_ok());
        assert!(validate_browse_filters("办公","community","downloads").is_err());
        assert!(validate_browse_filters("","third-party","score").is_err());
        assert!(validate_browse_filters("","","bogus").is_err());
        let v=browse_result(&json!({"data":{"total":49,"skills":[{
            "slug":"demo-skill","name":"Demo","version":"1.0","ownerName":"publisher",
            "source":"official","category":"office-efficiency","downloads":17
        }]}}),2).unwrap();
        assert_eq!(v["hasMore"],true);
        assert_eq!(v["results"][0]["namespace"],"publisher");
        assert_eq!(v["results"][0]["source"],"official");
        assert_eq!(v["results"][0]["downloads"],17);
        assert_eq!(skill_namespace(&json!({"namespace":{"handle":"../unsafe"},"ownerName":"safe-publisher"})),Some("safe-publisher"));
    }
    #[test]
    fn resolves_default_publisher_before_pinning_files_and_downloads(){
        let detail=json!({"skill":{"slug":"demo-skill"},"namespace":{"handle":"actual-publisher"}});
        assert_eq!(canonical_namespace(&detail,"demo-skill",None).unwrap(),"actual-publisher");
        assert!(canonical_namespace(&detail,"demo-skill",Some("other-publisher")).is_err());
        assert!(canonical_namespace(&detail,"other-skill",None).is_err());
        assert!(canonical_namespace(&json!({"skill":{"slug":"demo-skill"}}),"demo-skill",None).is_err());
    }
    #[test]
    fn headerless_registry_skills_require_pinned_identity_and_description() {
        let raw=b"# Legacy skill\nUse these instructions.";
        let hash=format!("{:x}",Sha256::digest(raw));
        let metadata=json!({"name":"legacy-skill","description":"Registry description"});
        assert_eq!(validate_registry_skill("legacy-skill",raw,&hash,Some(&metadata)).unwrap().as_bytes(),raw);
        assert!(validate_registry_skill("other-skill",raw,&hash,Some(&metadata)).is_err());
        assert!(validate_registry_skill("legacy-skill",raw,&hash,None).is_err());
        assert!(validate_registry_skill("legacy-skill",raw,&"0".repeat(64),Some(&metadata)).is_err());
        let malformed=b"---\nname: legacy-skill";
        let hash=format!("{:x}",Sha256::digest(malformed));
        assert!(validate_registry_skill("legacy-skill",malformed,&hash,Some(&metadata)).is_err());
    }
    #[test]
    fn skill_validation_preserves_reviewed_bytes_and_accepts_header_variants() {
        for text in [
            "---\nname: demo-skill\ndescription: Example\n---\nBody".to_string(),
            "\u{feff}---\r\nname: 'demo-skill'\r\ndescription: >-\r\n  Read reports\r\n  and summarize.\r\n---\r\nBody".to_string(),
            format!("---\n{}name: \"demo-skill\" # comment\ndescription: Example\n---\nBody", "# metadata\n".repeat(30)),
        ] {
            let hash = format!("{:x}", Sha256::digest(text.as_bytes()));
            assert_eq!(validate_skill("demo-skill", text.as_bytes(), &hash).unwrap(), text);
        }
    }

    #[test]
    fn skill_validation_keeps_checksum_and_identity_checks() {
        let text = "---\nname: other-skill\ndescription: Example\n---\nBody";
        let hash = format!("{:x}", Sha256::digest(text.as_bytes()));
        let error = validate_skill("demo-skill", text.as_bytes(), &hash).unwrap_err();
        assert!(error.contains("other-skill") && error.contains("demo-skill"));
        assert!(validate_skill("other-skill", text.as_bytes(), &"0".repeat(64))
            .unwrap_err().contains("SHA256 mismatch"));
        let text = "---\nname: demo-skill\n---\ndescription: Body is not metadata";
        let hash = format!("{:x}", Sha256::digest(text.as_bytes()));
        assert!(validate_skill("demo-skill", text.as_bytes(), &hash)
            .unwrap_err().contains("description"));
    }
    #[test]
    fn browsing_preserves_identity_and_paginates() {
        let result = browse_result(&json!({"data":{"skills":[{"slug":"demo-skill","name":"Demo",
            "version":"1.2.3","description":"English","description_zh":"中文"}],"total":25}}), 1).unwrap();
        assert_eq!(result["results"][0]["description"], "中文");
        assert_eq!(result["results"][0]["version"], "1.2.3");
        assert_eq!(result["hasMore"], true);
        assert_eq!(browse_result(&json!({"data":{"skills":[],"total":25}}),2).unwrap()["hasMore"],false);
        assert!(browse_result(&json!({"error":"offline"}),1).is_err());
    }
    #[test]
    fn rejects_unsafe_identity() {
        assert!(!valid_slug("../evil"));
        assert!(!valid_slug("bad/path"));
        assert!(valid_slug("find-skills"));
        assert!(!valid_version("1/2"));
    }
}

