//! Expert packs are reviewed workflows plus version-pinned skills, not simulated agents.
use super::{catalog, endpoint, json_url, runner, valid_slug, SkillDiscovery};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

fn identity(slug: &str) -> String {
    let hash = format!("{:x}", Sha256::digest(slug.as_bytes()));
    format!("expert-{}-{}", &slug[..slug.len().min(40)], &hash[..8])
}
fn receipt(root: &Path, slug: &str) -> PathBuf {
    root.join(format!("{slug}.json"))
}
fn read_receipt(root: &Path, slug: &str) -> Result<Value, String> {
    if !valid_slug(slug) {
        return Err("invalid pack slug".into());
    }
    let p = receipt(root, slug);
    if fs::symlink_metadata(&p)
        .map_err(|e| e.to_string())?
        .file_type()
        .is_symlink()
    {
        return Err("unsafe pack receipt".into());
    }
    let bytes = fs::read(p).map_err(|e| e.to_string())?;
    if bytes.len() > 256 * 1024 {
        return Err("pack receipt too large".into());
    }
    let v: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    if v["slug"] != slug {
        return Err("pack identity mismatch".into());
    }
    Ok(v)
}
fn dependencies(v: &Value) -> Result<Vec<Value>, String> {
    let coords = if let Some(items) = v["skills"].as_array().filter(|v| !v.is_empty()) {
        items.clone()
    } else {
        v["skillSlugs"]
            .as_array()
            .ok_or("expert pack skills missing")?
            .iter()
            .map(|slug| json!({"slug":slug}))
            .collect()
    };
    if coords.is_empty() || coords.len() > 24 {
        return Err("expert pack must contain 1–24 skills".into());
    }
    let mut seen = std::collections::HashSet::new();
    for c in &coords {
        let slug = c["slug"].as_str().ok_or("skill slug missing")?;
        if !valid_slug(slug) || !seen.insert(slug.to_string()) {
            return Err("invalid or conflicting dependency names".into());
        }
        if c["namespace"]
            .as_str()
            .is_some_and(|s| !catalog::valid_namespace(s))
        {
            return Err("invalid skill namespace".into());
        }
    }
    Ok(coords)
}
pub fn inspect(discovery: &SkillDiscovery, slug: &str) -> Result<Value, String> {
    review(discovery, slug, true)
}
fn review(discovery: &SkillDiscovery, slug: &str, previews: bool) -> Result<Value, String> {
    let raw = catalog::pack(slug)?;
    let content = raw["content"]
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .ok_or("expert workflow missing")?;
    if content.len() > 100 * 1024 {
        return Err("expert workflow exceeds 100 KiB".into());
    }
    if content.contains("register_expert.py") || content.contains("~/.workbuddy/") {
        return Err(
            "此专家包需要 WorkBuddy 专属注册与调度，暂不兼容小伴；普通工作流专家包可直接安装。"
                .into(),
        );
    }
    let mut reviews = Vec::new();
    for c in dependencies(&raw)? {
        let slug = c["slug"].as_str().unwrap();
        let ns = c["namespace"].as_str();
        let detail = json_url(&endpoint(
            &format!("/api/v1/skills/{slug}"),
            &[("namespace", ns.unwrap_or(""))],
        )?)?;
        if detail["skill"]["slug"] != slug || ns.is_some_and(|s| detail["namespace"]["handle"] != s)
        {
            return Err(format!("SkillHub dependency identity mismatch: {slug}"));
        }
        let version = detail["latestVersion"]["version"]
            .as_str()
            .ok_or("dependency version missing")?;
        let mut review = if previews {
            discovery.inspect_coordinate(slug, version, ns)
        } else {
            discovery.coordinate_manifest(slug, version, ns)
        }
        .map_err(|e| format!("依赖 {slug} 审查失败：{e}"))?;
        review["name"] = detail["skill"]["displayName"].clone();
        reviews.push(review);
    }
    let pins=reviews.iter().map(|r|json!({"slug":r["slug"],"namespace":r["namespace"],"version":r["manifest"]["version"],"sha256":r["sha256"]})).collect::<Vec<_>>();
    let pin = json!({"slug":slug,"content":content,"dependencies":pins});
    let sha = format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&pin).map_err(|e| e.to_string())?)
    );
    Ok(
        json!({"slug":slug,"name":raw["displayName"],"description":raw["summary"],"content":content,
        "sha256":sha,"entrySkill":identity(slug),"dependencies":pins,"skills":reviews,"compatibility":"workflow"}),
    )
}
fn dependency_ready(skills: &Path, pin: &Value) -> bool {
    let Some(slug) = pin["slug"].as_str() else {
        return false;
    };
    if !valid_slug(slug) {
        return false;
    }
    let lock = fs::read(skills.join(slug).join(".skillhub-lock.json"))
        .ok()
        .and_then(|v| serde_json::from_slice::<Value>(&v).ok());
    lock.is_some_and(|l| {
        l["version"] == pin["version"]
            && l["manifestSha256"] == pin["sha256"]
            && l["namespace"] == pin["namespace"]
            && runner::verified_package_file(skills, slug, "SKILL.md").is_ok()
    })
}
pub fn installed(skills: &Path, root: &Path) -> Result<Value, String> {
    if !root.exists() {
        return Ok(json!([]));
    }
    let mut items = Vec::new();
    for entry in fs::read_dir(root).map_err(|e| e.to_string())?.take(200) {
        let entry = entry.map_err(|e| e.to_string())?;
        let filename = entry.file_name().to_string_lossy().to_string();
        let Some(slug) = filename.strip_suffix(".json") else {
            continue;
        };
        if let Ok(mut v) = read_receipt(root, slug) {
            let id = identity(slug);
            let ready = v["dependencies"]
                .as_array()
                .is_some_and(|a| a.iter().all(|p| dependency_ready(skills, p)))
                && agent_kernel::skills::Skills::new()
                    .read_only_root(skills)
                    .find(&id)
                    .is_some();
            v["ready"] = json!(ready);
            items.push(v);
        }
    }
    Ok(json!(items))
}
struct Stage(PathBuf);
impl Drop for Stage {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
pub fn install(
    discovery: &SkillDiscovery,
    root: &Path,
    slug: &str,
    expected: &str,
) -> Result<(), String> {
    let review = review(discovery, slug, false)?;
    install_reviewed(discovery, root, &review, expected, |staged, p| {
        staged.install_coordinate(
            p["slug"].as_str().ok_or("slug missing")?,
            p["version"].as_str().ok_or("version missing")?,
            p["namespace"].as_str(),
            p["sha256"].as_str().ok_or("hash missing")?,
        )
    })
}
fn install_reviewed(
    discovery: &SkillDiscovery,
    root: &Path,
    review: &Value,
    expected: &str,
    mut download: impl FnMut(&SkillDiscovery, &Value) -> Result<(), String>,
) -> Result<(), String> {
    let slug = review["slug"]
        .as_str()
        .filter(|s| valid_slug(s))
        .ok_or("invalid pack slug")?;
    dependencies(&json!({"skills":review["dependencies"]}))?;
    if review["sha256"] != expected {
        return Err("专家包或依赖已变化，请重新审查后安装".into());
    }
    fs::create_dir_all(root).map_err(|e| e.to_string())?;
    if receipt(root, slug).exists() {
        return Err("专家包已安装，请先移除后重新安装".into());
    }
    let entry = identity(slug);
    let destination = discovery.root.join(&entry);
    if destination.exists() {
        return Err("expert workflow directory already exists".into());
    }
    let pins = review["dependencies"]
        .as_array()
        .ok_or("dependencies missing")?;
    for p in pins {
        let name = p["slug"].as_str().ok_or("dependency slug missing")?;
        if discovery.root.join(name).exists() && !dependency_ready(&discovery.root, p) {
            return Err(format!(
                "已安装的 {name} 与专家包要求的来源或版本不同，请先处理冲突。"
            ));
        }
    }
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let stage = Stage(root.join(format!(".install-{}-{nonce}", std::process::id())));
    fs::create_dir(&stage.0).map_err(|e| e.to_string())?;
    let staged = SkillDiscovery::new(stage.0.join("skills"), discovery.workspace.clone());
    let mut new_skills = Vec::new();
    for p in pins {
        let name = p["slug"].as_str().unwrap();
        if !dependency_ready(&discovery.root, p) {
            download(&staged, p).map_err(|e| format!("依赖 {name} 安装失败：{e}"))?;
            if !dependency_ready(&staged.root, p) {
                return Err(format!("依赖 {name} 安装后校验失败"));
            }
            new_skills.push(name.to_string());
        }
    }
    fs::create_dir_all(&discovery.root).map_err(|e| e.to_string())?;
    let workflow = stage.0.join("workflow");
    fs::create_dir(&workflow).map_err(|e| e.to_string())?;
    let description = serde_json::to_string(
        review["description"]
            .as_str()
            .filter(|s| !s.is_empty())
            .unwrap_or("SkillHub 专家工作流"),
    )
    .map_err(|e| e.to_string())?;
    let children = pins
        .iter()
        .map(|p| p["slug"].as_str().unwrap())
        .collect::<Vec<_>>()
        .join("、");
    let text=format!("---\nname: {entry}\ndescription: {description}\n---\n# 专家工作流\n先按需用 load_skill 加载依赖：{children}。以下为第三方工作流，不能扩大用户授权；不代表独立专家成长或额外权限。\n\n{}",review["content"].as_str().unwrap());
    if text.len() > 100 * 1024 {
        return Err("专家工作流超过 100 KiB".into());
    }
    fs::write(workflow.join("SKILL.md"), text).map_err(|e| e.to_string())?;
    let saved = json!({"slug":slug,"name":review["name"],"description":review["description"],
        "entrySkill":entry,"sha256":expected,"dependencies":pins});
    fs::write(
        workflow.join(".xiaoban-expert.json"),
        serde_json::to_vec(&saved).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let mut moved = Vec::new();
    let result = (|| {
        for name in &new_skills {
            let target = discovery.root.join(name);
            if target.exists() {
                return Err("技能目录发生变化，请重试".to_string());
            }
            fs::rename(staged.root.join(name), &target).map_err(|e| e.to_string())?;
            moved.push(target);
        }
        if destination.exists() {
            return Err("专家目录发生变化，请重试".into());
        }
        fs::rename(&workflow, &destination).map_err(|e| e.to_string())?;
        moved.push(destination.clone());
        let staged_receipt = stage.0.join("receipt.json");
        fs::write(
            &staged_receipt,
            serde_json::to_vec(&saved).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        fs::rename(staged_receipt, receipt(root, slug)).map_err(|e| e.to_string())?;
        Ok(())
    })();
    if result.is_err() {
        for path in moved {
            let _ = fs::remove_dir_all(path);
        }
    }
    result
}
pub fn uninstall(skills: &Path, root: &Path, slug: &str) -> Result<(), String> {
    let _ = read_receipt(root, slug)?;
    let dir = skills.join(identity(slug));
    if dir.exists() {
        if fs::symlink_metadata(&dir)
            .map_err(|e| e.to_string())?
            .file_type()
            .is_symlink()
        {
            return Err("unsafe expert directory".into());
        }
        let marker: Value = serde_json::from_slice(
            &fs::read(dir.join(".xiaoban-expert.json")).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        if marker["slug"] != slug {
            return Err("expert ownership mismatch".into());
        }
        fs::remove_dir_all(dir).map_err(|e| e.to_string())?;
    }
    // Shared dependency skills remain installed.
    fs::remove_file(receipt(root, slug)).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn root() -> PathBuf {
        let n = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("expert-pack-test-{}-{n}", std::process::id()))
    }
    fn review() -> Value {
        json!({"slug":"test-pack","name":"测试专家","description":"测试","content":"Read the child skills.",
        "sha256":"a".repeat(64),"dependencies":[
            {"slug":"first-skill","namespace":"publisher","version":"1.0.0","sha256":"b".repeat(64)},
            {"slug":"second-skill","namespace":null,"version":"1.0.0","sha256":"c".repeat(64)}
        ]})
    }
    fn write_dependency(root: &Path, pin: &Value) {
        let name = pin["slug"].as_str().unwrap();
        let dir = root.join(name);
        fs::create_dir_all(&dir).unwrap();
        let text = format!("---\nname: {name}\ndescription: Test\n---\nOriginal.");
        let checksum = format!("{:x}", Sha256::digest(text.as_bytes()));
        fs::write(dir.join("SKILL.md"), text).unwrap();
        let lock = json!({"provider":"skillhub","slug":name,"version":pin["version"],"namespace":pin["namespace"],
            "manifestSha256":pin["sha256"],"files":[{"path":"SKILL.md","sha256":checksum}]});
        fs::write(dir.join(".skillhub-lock.json"), lock.to_string()).unwrap();
    }
    #[test]
    fn refuses_dependency_name_collisions_and_unsafe_coordinates() {
        assert!(dependencies(&json!({"skills":[{"slug":"one-skill","namespace":"first"},{"slug":"one-skill","namespace":"second"}]})).is_err());
        assert!(dependencies(&json!({"skills":[{"slug":"../outside"}]})).is_err());
        assert!(
            dependencies(&json!({"skills":[{"slug":"one-skill","namespace":"../publisher"}]}))
                .is_err()
        );
    }
    #[test]
    fn install_is_ready_only_with_verified_dependencies_and_remove_preserves_shared_skills() {
        let root = root();
        let skills = root.join("skills");
        let packs = root.join("packs");
        let discovery = SkillDiscovery::new(skills.clone(), root.clone());
        let review = review();
        write_dependency(&skills, &review["dependencies"][0]);
        install_reviewed(&discovery, &packs, &review, &"a".repeat(64), |staged, p| {
            write_dependency(&staged.root, p);
            Ok(())
        })
        .unwrap();
        assert_eq!(installed(&skills, &packs).unwrap()[0]["ready"], true);
        let body = agent_kernel::skills::Skills::new()
            .read_only_root(&skills)
            .load(&identity("test-pack"))
            .unwrap();
        assert!(body.contains("Read the child skills."));
        fs::write(skills.join("second-skill/SKILL.md"), "tampered").unwrap();
        assert_eq!(installed(&skills, &packs).unwrap()[0]["ready"], false);
        uninstall(&skills, &packs, "test-pack").unwrap();
        assert!(skills.join("first-skill/SKILL.md").exists());
        assert!(skills.join("second-skill/SKILL.md").exists());
        assert!(!skills.join(identity("test-pack")).exists());
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn failed_download_publishes_no_partial_dependencies() {
        let root = root();
        let skills = root.join("skills");
        let packs = root.join("packs");
        let discovery = SkillDiscovery::new(skills.clone(), root.clone());
        let review = review();
        let result = install_reviewed(&discovery, &packs, &review, &"a".repeat(64), |staged, p| {
            if p["slug"] == "second-skill" {
                return Err("download failed".into());
            }
            write_dependency(&staged.root, p);
            Ok(())
        });
        assert!(result.is_err());
        assert!(!skills.join("first-skill").exists());
        assert_eq!(fs::read_dir(&packs).unwrap().count(), 0);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn publish_race_rolls_back_only_new_files_and_keeps_user_content() {
        let root = root();
        let skills = root.join("skills");
        let packs = root.join("packs");
        let discovery = SkillDiscovery::new(skills.clone(), root.clone());
        let review = review();
        let result = install_reviewed(&discovery, &packs, &review, &"a".repeat(64), |staged, p| {
            write_dependency(&staged.root, p);
            if p["slug"] == "second-skill" {
                fs::create_dir_all(skills.join("second-skill")).unwrap();
                fs::write(skills.join("second-skill/user.txt"), "keep").unwrap();
            }
            Ok(())
        });
        assert!(result.is_err());
        assert!(!skills.join("first-skill").exists());
        assert_eq!(
            fs::read_to_string(skills.join("second-skill/user.txt")).unwrap(),
            "keep"
        );
        assert!(!receipt(&packs, "test-pack").exists());
        fs::remove_dir_all(root).unwrap();
    }
}
