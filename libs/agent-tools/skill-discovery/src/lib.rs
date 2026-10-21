//! Discover, review and install version-pinned SkillHub skills.
//! Third-party instructions and scripts are untrusted. Installing never executes code.
mod acquisition;
mod package;
mod runner;
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
fn file_info(slug: &str, version: &str) -> Result<Plan, String> {
    let url = endpoint(&format!("/api/v1/skills/{slug}/files"), &[("version", version)])?;
    let result = json_url(&url)?;
    if result["version"].as_str() != Some(version) {
        return Err("SkillHub version changed unexpectedly".into());
    }
    let files = result["files"].as_array().ok_or("SkillHub file list is missing")?;
    plan(version, files)
}
fn content(slug: &str, version: &str, path: &str, max: u64) -> Result<Vec<u8>, String> {
    let url = endpoint(&format!("/api/v1/skills/{slug}/file"),
        &[("version", version), ("path", path)])?;
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

pub struct SkillDiscovery { root: PathBuf, workspace: PathBuf }
impl SkillDiscovery {
    pub fn new(root: PathBuf, workspace: PathBuf) -> Self { Self { root, workspace } }
    /// Product UI browsing; no model invocation or installation is involved.
    pub fn browse(keyword: &str, page: u32) -> Result<Value, String> {
        let keyword = keyword.trim();
        if keyword.chars().count() > 80 || !(1..=1000).contains(&page) {
            return Err("invalid SkillHub query/page".into());
        }
        let page_text = page.to_string();
        let url = endpoint("/api/skills", &[("keyword", keyword), ("page", &page_text),
            ("pageSize", "24"), ("sortBy", "downloads"), ("order", "desc")])?;
        let result = json_url(&url)?;
        browse_result(&result, page)
    }
    /// UI callers must display this review before offering an install confirmation.
    pub fn inspect(&self, slug: &str, version: &str) -> Result<Value, String> {
        let raw = self.ui_tool("skill_inspect").run(json!({"slug":slug,"version":version}), &|| false)?;
        serde_json::from_str(&raw).map_err(|e| e.to_string())
    }
    /// Only call after a user confirms the exact reviewed version and manifest hash.
    /// This shares all package validation with the Agent tool and never runs scripts.
    pub fn install_reviewed(&self, slug: &str, version: &str, sha256: &str) -> Result<(), String> {
        self.ui_tool("skill_install").run(json!({"slug":slug,"version":version,"sha256":sha256}), &|| false)?;
        Ok(())
    }
    fn ui_tool(&self, name: &str) -> DiscoveryTool {
        DiscoveryTool { root: self.root.clone(), spec: ToolSpec::new(name, "", json!({"type":"object"})) }
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
                        "reason":{"type":"string","enum":["capability_gap","repeated_failure"],"description":"For proactive searches only; omit when user explicitly asks for Skill discovery."},
                        "gap":{"type":"string","description":"Specific capability unavailable among installed tools and skills; required for proactive discovery."},
                        "alternatives":{"type":"string","description":"Existing tools/skills considered and why they cannot solve the task; required for proactive discovery."}
                    },"required":["keyword"]}),
                    "skill_inspect" => json!({"type":"object","properties":{
                        "slug":{"type":"string"},"version":{"type":"string"}},"required":["slug","version"]}),
                    _ => json!({"type":"object","properties":{
                        "slug":{"type":"string"},"version":{"type":"string"},
                        "sha256":{"type":"string","minLength":64,"maxLength":64}},
                        "required":["slug","version","sha256"]})
                };
                let spec = ToolSpec::new(name,
                    "Search, review or install a pinned SkillHub package. Review its complete file list and script previews before installing. Installing does not execute scripts.",
                    schema)
                    .risk(if name == "skill_install" { Risk::Write } else { Risk::Network });
                let spec = if name == "skill_install" { spec.always_ask() }
                    else { spec.untrusted_output() };
                Arc::new(DiscoveryTool { root: self.root.clone(), spec }) as Arc<dyn Tool>
            }).collect();
        tools.push(Arc::new(runner::SkillRead::new(self.root.clone())));
        tools.push(Arc::new(runner::SkillRun::new(self.root.clone(), self.workspace.clone())));
        tools
    }
}
struct DiscoveryTool { spec: ToolSpec, root: PathBuf }
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
            let url = endpoint("/api/skills", &[("keyword", keyword), ("pageSize", "10")])?;
            let j = json_url(&url)?;
            let items = j["data"]["skills"].as_array().ok_or("invalid SkillHub search response")?;
            return Ok(json!({"provider":"skillhub","results":items.iter().take(10).map(|s|json!({
                "slug":s["slug"],"name":s["name"],"version":s["version"],
                "description":s["description_zh"].as_str().or(s["description"].as_str()).unwrap_or(""),
                "homepage":s["homepage"]
            })).collect::<Vec<_>>()}).to_string());
        }
        let slug = args["slug"].as_str().ok_or("slug required")?;
        let version = args["version"].as_str().ok_or("version required")?;
        if !valid_slug(slug) || !valid_version(version) {
            return Err("invalid SkillHub slug/version".into());
        }
        let package = file_info(slug, version)?;
        if self.spec.name == "skill_inspect" {
            let skill_file = package.files.iter().find(|f| f.path == "SKILL.md").ok_or("SKILL.md missing")?;
            let markdown = validate_skill(slug,
                &content(slug, version, "SKILL.md", 100 * 1024)?, &skill_file.sha256)?;
            let mut scripts = Vec::new();
            for item in package.files.iter().filter(|f| is_script(&f.path)) {
                if cancelled() { return Err("cancelled".into()); }
                // Truncated display is not a security audit. Every byte is still pinned by SHA256.
                let preview = if item.size <= 1024 * 1024 {
                    match content(slug, version, &item.path, 1024 * 1024) {
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
                "manifest":package.summary(),"sha256":package.sha256,"content":markdown,"scripts":scripts,
                "requiresInstallationApproval":true,"requiresExecutionApproval":true,
                "warning":"SHA256 checks integrity, not safety. Third-party skills run with local user permissions; no sandbox."}).to_string());
        }
        let expected = args["sha256"].as_str().ok_or("expected manifest SHA256 required")?;
        if expected.len() != 64 || !expected.bytes().all(|b| b.is_ascii_hexdigit())
            || package.sha256 != expected.to_ascii_lowercase() {
            return Err("SkillHub file manifest differs from the reviewed SHA256".into());
        }
        if cancelled() { return Err("cancelled".into()); }
        let url = endpoint("/api/v1/download", &[("slug", slug), ("version", version)])?;
        let archive = bytes(&url, 12 * 1024 * 1024)?;
        if cancelled() { return Err("cancelled".into()); }
        // Validate SKILL.md metadata after unpacking and before materializing any file.
        let files = package::verified_files(slug, &package, &archive)?;
        let skill_file = package.files.iter().find(|f| f.path == "SKILL.md").ok_or("SKILL.md missing")?;
        validate_skill(slug, files.get("SKILL.md").ok_or("SKILL.md missing")?, &skill_file.sha256)?;
        package::install(&self.root, slug, &package, &archive)?;
        Ok(format!("Installed SkillHub {slug}@{version}: {} verified files (including {} scripts); manifest SHA256={}",
            package.files.len(), package.files.iter().filter(|f| is_script(&f.path)).count(), package.sha256))
    }
}
fn browse_result(result: &Value, page: u32) -> Result<Value, String> {
    let items = result["data"]["skills"].as_array().ok_or("invalid SkillHub browse response")?;
    let total = result["data"]["total"].as_u64();
    let results = items.iter().take(24).map(|s| json!({
        "slug":s["slug"], "name":s["name"], "version":s["version"],
        "description":s["description_zh"].as_str().or(s["description"].as_str()).unwrap_or("")
    })).collect::<Vec<_>>();
    Ok(json!({"results":results, "page":page, "total":total,
        "hasMore":total.map(|n| u64::from(page)*24 < n).unwrap_or(items.len() >= 24)}))
}
#[cfg(test)]
mod tests {
    use super::*;
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

