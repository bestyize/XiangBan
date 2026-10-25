//! 小伴市场: browse, review, install and update packages from the self-hosted market service.
//!
//! The market is first-party, but nothing it returns is trusted blindly: a version is pinned by its
//! manifest digest, which is recomputed from the file list; the ZIP is checked against that list file
//! by file; and installing never runs code. Experts are packaged like skills (a `SKILL.md` workflow),
//! so they install into the same skills directory. Teams and bundles install their members together.
use crate::{
    catalog::valid_namespace,
    package::{self, is_script, market_plan, Plan},
    plugin, valid_slug, valid_version, validate_registry_skill,
};
use agent_kernel::net::blocking_builder;
use reqwest::{blocking::Client, redirect::Policy, Url};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, io::Read, path::Path, time::Duration};

const MAX_JSON: u64 = 2 * 1024 * 1024;
const MAX_ZIP: u64 = 12 * 1024 * 1024;
const MAX_MEMBERS: usize = 24;

/// Accepts an `https` origin, or `http` only for the local machine (development and tests).
/// Returns `scheme://host[:port]` with nothing after it.
pub fn valid_origin(text: &str) -> Result<String, String> {
    let url = Url::parse(text.trim()).map_err(|_| "小伴市场地址无效".to_string())?;
    let host = url.host_str().ok_or("小伴市场地址无效")?;
    let local = matches!(host, "localhost" | "127.0.0.1" | "[::1]" | "::1");
    let allowed = url.scheme() == "https" || (url.scheme() == "http" && local);
    if !allowed || !url.username().is_empty() || url.password().is_some()
        || !matches!(url.path(), "" | "/") || url.query().is_some() || url.fragment().is_some() {
        return Err("小伴市场地址必须是 https 站点根地址".into());
    }
    Ok(url.origin().ascii_serialization())
}

fn http() -> Result<Client, String> {
    // The market never redirects; refusing redirects keeps downloads on the pinned origin.
    blocking_builder(Client::builder().timeout(Duration::from_secs(30)).redirect(Policy::none()))?
        .build().map_err(|e| e.to_string())
}

fn plural(kind: &str) -> Result<&'static str, String> {
    match kind {
        "skill" | "skills" => Ok("skills"),
        "expert" | "experts" => Ok("experts"),
        "connector" | "connectors" => Ok("connectors"),
        _ => Err("未知的市场类型".into()),
    }
}
fn check_coordinate(namespace: &str, slug: &str) -> Result<(), String> {
    if valid_namespace(namespace) && valid_slug(slug) { Ok(()) } else { Err("无效的市场条目标识".into()) }
}
fn hex64(text: &str) -> bool {
    text.len() == 64 && text.bytes().all(|b| b.is_ascii_hexdigit())
}

/// A package to install: `kind` is `skills` or `experts`.
#[derive(Clone, Debug)]
pub struct Package { pub kind: String, pub namespace: String, pub slug: String, pub version: String }

pub struct Market { origin: String }

struct Verified { plan: Plan, files: BTreeMap<String, Vec<u8>>, name: String, summary: String }

impl Market {
    pub fn new(origin: &str) -> Result<Self, String> {
        Ok(Self { origin: valid_origin(origin)? })
    }

    fn url(&self, path: &str, pairs: &[(&str, &str)]) -> Result<Url, String> {
        Url::parse_with_params(&format!("{}/api/v1/market{path}", self.origin), pairs.iter().filter(|(_, v)| !v.is_empty()))
            .map_err(|e| e.to_string())
    }
    fn read(&self, request: reqwest::blocking::RequestBuilder, max: u64) -> Result<Vec<u8>, String> {
        let response = request.send().map_err(|_| "暂时无法连接小伴市场".to_string())?;
        let status = response.status();
        let mut body = Vec::new();
        response.take(max + 1).read_to_end(&mut body).map_err(|_| "小伴市场响应读取失败".to_string())?;
        if body.len() as u64 > max { return Err("小伴市场响应过大".into()); }
        if !status.is_success() {
            let message = serde_json::from_slice::<Value>(&body).ok()
                .and_then(|v| v["message"].as_str().map(str::to_string));
            return Err(message.unwrap_or_else(|| format!("小伴市场暂不可用（HTTP {}）", status.as_u16())));
        }
        Ok(body)
    }
    fn get_json(&self, path: &str, pairs: &[(&str, &str)]) -> Result<Value, String> {
        let url = self.url(path, pairs)?;
        let body = self.read(http()?.get(url), MAX_JSON)?;
        serde_json::from_slice(&body).map_err(|_| "小伴市场返回了无效数据".to_string())
    }

    /// One page of a listing, shaped like the SkillHub results the store page already renders.
    pub fn browse(&self, kind: &str, keyword: &str, page: u32, category: &str, source: &str, sort_by: &str, variant: &str) -> Result<Value, String> {
        if keyword.chars().count() > 80 || !(1..=1000).contains(&page) || category.len() > 100 {
            return Err("invalid catalog query".into());
        }
        let page_text = page.to_string();
        let raw = self.get_json(&format!("/{}", plural(kind)?), &[
            ("keyword", keyword), ("page", &page_text), ("category", category), ("source", source),
            ("sortBy", sort_by), ("variant", variant),
        ])?;
        let list = |key: &str| -> Result<Vec<Value>, String> {
            Ok(raw[key].as_array().ok_or("小伴市场目录格式发生变化，请更新客户端")?.iter().map(normalize).collect())
        };
        Ok(json!({
            "results": list("results")?.into_iter().map(|i| self.absolute(i)).collect::<Vec<_>>(), "near": raw["near"].as_array().map(|a| a.iter().map(normalize).map(|i| self.absolute(i)).collect::<Vec<_>>()).unwrap_or_default(),
            "page": page, "total": raw["total"].as_u64(), "hasMore": raw["hasMore"].as_bool().unwrap_or(false),
        }))
    }

    pub fn categories(&self, kind: &str) -> Result<Value, String> {
        let raw = self.get_json("/categories", &[("kind", plural(kind)?.trim_end_matches('s'))])?;
        let entries = raw["results"].as_array().ok_or("小伴市场分类格式发生变化")?;
        Ok(json!({"results": entries.iter().filter_map(|c| Some(json!({"key": c["slug"].as_str()?, "name": c["name"].as_str()?}))).collect::<Vec<_>>()}))
    }

    /// The listing detail: description, versions, members of a team or bundle, connector configs.
    pub fn detail(&self, kind: &str, namespace: &str, slug: &str) -> Result<Value, String> {
        check_coordinate(namespace, slug)?;
        let mut detail = self.get_json(&format!("/{}/{namespace}/{slug}", plural(kind)?), &[])?;
        for path in ["/item/iconUrl", "/item/metadata/expert/avatarUrl"] {
            if let Some(v) = detail.pointer_mut(path) { *v = self.absolute_url(v); }
        }
        for key in ["/members", "/item/metadata/expert/members"] {
            for member in detail.pointer_mut(key).and_then(Value::as_array_mut).into_iter().flatten() {
                for field in ["iconUrl", "avatarUrl"] {
                    if let Some(v) = member.get_mut(field) { *v = self.absolute_url(v); }
                }
            }
        }
        Ok(detail)
    }

    /// Avatars the market stores itself come back as `/api/v1/market/assets/...`; make them loadable from the app.
    fn absolute_url(&self, v: &Value) -> Value {
        match v.as_str() {
            Some(path) if path.starts_with("/api/v1/market/assets/") => json!(format!("{}{path}", self.origin)),
            _ => v.clone(),
        }
    }

    fn absolute(&self, mut item: Value) -> Value {
        if let Some(v) = item.get_mut("iconUrl") { *v = self.absolute_url(v); }
        if let Some(v) = item.pointer_mut("/expert/avatarUrl") { *v = self.absolute_url(v); }
        item
    }

    /// Downloads and fully verifies one version. Nothing is written to disk.
    /// The version manifest the market lists for one version, checked against what was asked for.
    fn version_manifest(&self, p: &Package) -> Result<(&'static str, Value), String> {
        let kind = plural(&p.kind)?;
        if kind == "connectors" { return Err("连接器不是安装包".into()); }
        check_coordinate(&p.namespace, &p.slug)?;
        if !valid_version(&p.version) { return Err("无效的版本号".into()); }
        let manifest = self.get_json(&format!("/{kind}/{}/{}/versions/{}", p.namespace, p.slug, p.version), &[])?;
        if manifest["namespace"] != p.namespace.as_str() || manifest["slug"] != p.slug.as_str() || manifest["version"] != p.version.as_str() {
            return Err("小伴市场返回的版本与请求不一致".into());
        }
        Ok((kind, manifest))
    }

    /// Experts are WorkBuddy plugins (a persona, optional bundled skills); skills and older experts are `SKILL.md` packages.
    pub fn is_plugin(&self, p: &Package) -> Result<bool, String> {
        Ok(has_plugin_manifest(&self.version_manifest(p)?.1))
    }

    /// Downloads and fully verifies a plugin version. Nothing is written to disk.
    fn fetch_plugin(&self, p: &Package, kind: &str, manifest: &Value) -> Result<plugin::Package, String> {
        let files = manifest["files"].as_array().ok_or("小伴市场缺少文件清单")?;
        let url = self.url(&format!("/{kind}/{}/{}/versions/{}/download", p.namespace, p.slug, p.version), &[])?;
        let archive = self.read(http()?.get(url), MAX_ZIP)?;
        if let Some(expected) = manifest["packageSha256"].as_str() {
            if format!("{:x}", Sha256::digest(&archive)) != expected.to_ascii_lowercase() {
                return Err("下载的安装包与小伴市场记录不一致，已拒绝".into());
            }
        }
        let pkg = plugin::inspect_zip(&archive)?;
        let declared = manifest["manifestSha256"].as_str().unwrap_or("").to_ascii_lowercase();
        if pkg.manifest_sha256 != declared {
            return Err("清单摘要与小伴市场记录不一致，已拒绝".into());
        }
        if pkg.name != p.slug {
            return Err("插件名与小伴市场标识不一致，已拒绝".into());
        }
        let listed: BTreeMap<&str, &str> = files.iter().filter_map(|f| Some((f["path"].as_str()?, f["sha256"].as_str()?))).collect();
        let same = listed.len() == pkg.files.len() && pkg.files.iter().all(|f| listed.get(f.path.as_str()).is_some_and(|s| s.eq_ignore_ascii_case(&f.sha256)));
        if !same {
            return Err("文件清单与下载的安装包不一致，已拒绝".into());
        }
        Ok(pkg)
    }

    /// The review of a plugin expert: the persona, the skills it carries, scripts to read before trusting it.
    fn inspect_plugin(&self, p: &Package, kind: &str, manifest: &Value) -> Result<Value, String> {
        let pkg = self.fetch_plugin(p, kind, manifest)?;
        let docs = pkg.agent_docs();
        let lead = pkg.manifest["agentName"].as_str().and_then(|n| docs.values().find(|d| d.name == n)).or_else(|| docs.values().next());
        let scripts: Vec<Value> = pkg.files.iter().filter(|f| is_plugin_script(&f.path)).map(|f| {
            let preview = String::from_utf8(pkg.contents[&f.path].clone()).ok().map(|s| s.chars().take(8000).collect::<String>());
            json!({"path": f.path, "sha256": f.sha256, "preview": preview, "previewTruncated": f.size > 8000})
        }).collect();
        let mut summary = pkg.summary();
        let files: Vec<Value> = pkg.files.iter().map(|f| json!({"path": f.path, "size": f.size, "sha256": f.sha256, "executableScript": is_plugin_script(&f.path)})).collect();
        summary["files"] = json!(files);
        summary["sha256"] = json!(pkg.manifest_sha256);
        Ok(json!({
            "provider": "xiaoban-market", "slug": p.slug, "namespace": p.namespace, "kind": "plugin", "pluginKind": pkg.kind.as_str(),
            "name": manifest["name"].as_str().unwrap_or(&p.slug), "description": manifest["summary"].as_str().unwrap_or(""),
            "manifest": summary, "sha256": pkg.manifest_sha256,
            "content": lead.map(|d| d.body.trim().to_string()).unwrap_or_default(),
            "agents": docs.values().map(|d| json!({"name": d.name, "displayName": d.display_name.as_ref().map(|b| b.pick(true)), "profession": d.profession.as_ref().map(|b| b.pick(true)), "maxTurns": d.max_turns})).collect::<Vec<_>>(),
            "skills": pkg.manifest["skills"].as_array().map(|a| a.iter().filter_map(Value::as_str).map(|s| s.trim_start_matches("./skills/").to_string()).collect::<Vec<_>>()).unwrap_or_default(),
            "scripts": scripts, "warnings": pkg.warnings,
            "requiresInstallationApproval": true, "requiresExecutionApproval": true,
            "warning": "SHA256 只能证明内容与审查时一致，不代表安全。专家自带的脚本以本机账户权限运行，没有沙箱；专家不会增加你没有授权的工具或权限。",
        }))
    }

    /// Installs (or replaces, keeping the old copy in `backups`) a reviewed plugin version. `sha256` is the digest the user saw.
    pub fn install_plugin(&self, plugins: &Path, backups: &Path, p: &Package, sha256: &str) -> Result<Value, String> {
        let (kind, manifest) = self.version_manifest(p)?;
        let pkg = self.fetch_plugin(p, kind, &manifest)?;
        if !hex64(sha256) || pkg.manifest_sha256 != sha256.to_ascii_lowercase() {
            return Err("清单摘要与审查时不一致，已拒绝安装".into());
        }
        plugin::install(plugins, backups, &pkg, "xiaoban-market", json!({
            "service": "xiaoban-market", "kind": kind.trim_end_matches('s'), "namespace": p.namespace, "slug": p.slug,
        }))
    }

    /// Downloads and fully verifies one version. Nothing is written to disk.
    fn fetch(&self, p: &Package) -> Result<Verified, String> {
        let (kind, manifest) = self.version_manifest(p)?;
        if has_plugin_manifest(&manifest) { return Err("这是插件包，不是技能包".into()); }
        let files = manifest["files"].as_array().ok_or("小伴市场缺少文件清单")?;
        let mut plan = market_plan(&p.version, files)?;
        let declared = manifest["manifestSha256"].as_str().unwrap_or("").to_ascii_lowercase();
        if declared != plan.sha256 {
            return Err("清单摘要与小伴市场记录不一致，已拒绝".into());
        }
        let url = self.url(&format!("/{kind}/{}/{}/versions/{}/download", p.namespace, p.slug, p.version), &[])?;
        let archive = self.read(http()?.get(url), MAX_ZIP)?;
        if let Some(expected) = manifest["packageSha256"].as_str() {
            if format!("{:x}", Sha256::digest(&archive)) != expected.to_ascii_lowercase() {
                return Err("下载的安装包与小伴市场记录不一致，已拒绝".into());
            }
        }
        let files = package::verified_files(&p.slug, &plan, &archive)?;
        let summary = manifest["summary"].as_str().filter(|s| !s.trim().is_empty()).unwrap_or("").to_string();
        let name = manifest["name"].as_str().unwrap_or(&p.slug).to_string();
        // SKILL.md without a header relies on registry metadata, which is pinned in the lock file.
        let metadata = json!({"name": p.slug, "description": summary});
        let skill = plan.files.iter().find(|f| f.path == "SKILL.md").ok_or("SKILL.md missing")?;
        validate_registry_skill(&p.slug, files.get("SKILL.md").ok_or("SKILL.md missing")?, &skill.sha256, Some(&metadata))?;
        plan.namespace = Some(p.namespace.clone());
        plan.metadata = Some(metadata);
        plan.origin = Some(json!({"service": "xiaoban-market", "kind": p.kind.trim_end_matches('s')}));
        Ok(Verified { plan, files, name, summary })
    }

    /// Everything the review dialog shows; same shape as the SkillHub review.
    pub fn inspect(&self, p: &Package) -> Result<Value, String> {
        let (kind, manifest) = self.version_manifest(p)?;
        if has_plugin_manifest(&manifest) { return self.inspect_plugin(p, kind, &manifest); }
        let v = self.fetch(p)?;
        let markdown = String::from_utf8_lossy(&v.files["SKILL.md"]).into_owned();
        let scripts: Vec<Value> = v.plan.files.iter().filter(|f| is_script(&f.path)).map(|item| {
            let preview = String::from_utf8(v.files[&item.path].clone()).ok().map(|s| s.chars().take(8000).collect::<String>());
            json!({"path": item.path, "sha256": item.sha256, "preview": preview, "previewTruncated": item.size > 8000})
        }).collect();
        Ok(json!({
            "provider": "xiaoban-market", "slug": p.slug, "name": v.name, "description": v.summary, "namespace": p.namespace,
            "manifest": v.plan.summary(), "sha256": v.plan.sha256, "content": markdown, "scripts": scripts,
            "requiresInstallationApproval": true, "requiresExecutionApproval": true,
            "warning": "SHA256 只能证明内容与审查时一致，不代表安全。第三方脚本以本机账户权限运行，没有沙箱。",
        }))
    }

    /// Installs a reviewed version. `sha256` is the manifest digest the user saw.
    pub fn install(&self, root: &Path, p: &Package, sha256: &str) -> Result<(), String> {
        let v = self.fetch(p)?;
        pinned(&v, sha256)?;
        package::install(root, &p.slug, &v.plan, &v.files)
    }

    /// Replaces an installed market package with a reviewed newer version. The old one is kept in `backups`.
    pub fn update(&self, root: &Path, backups: &Path, p: &Package, sha256: &str) -> Result<(), String> {
        let v = self.fetch(p)?;
        pinned(&v, sha256)?;
        package::replace(root, &p.slug, &v.plan, &v.files, backups)
    }

    /// Members of an expert team or skill bundle, in the listing's order.
    fn members(&self, kind: &str, namespace: &str, slug: &str) -> Result<(String, Vec<Package>), String> {
        let detail = self.detail(kind, namespace, slug)?;
        let item = &detail["item"];
        let variant = item["variant"].as_str().unwrap_or("single");
        if !matches!(variant, "team" | "bundle") { return Err("这个条目不是专家团或技能套件".into()); }
        let member_kind = if variant == "team" { "experts" } else { "skills" };
        let list = detail["members"].as_array().ok_or("缺少成员列表")?;
        if list.is_empty() || list.len() > MAX_MEMBERS { return Err("成员数量不在支持范围内".into()); }
        let mut members = Vec::new();
        for m in list {
            let (ns, sl, ver) = (m["namespace"].as_str().unwrap_or(""), m["slug"].as_str().unwrap_or(""), m["version"].as_str().unwrap_or(""));
            check_coordinate(ns, sl)?;
            members.push(Package { kind: member_kind.into(), namespace: ns.into(), slug: sl.into(), version: ver.into() });
        }
        Ok((item["name"].as_str().unwrap_or(slug).to_string(), members))
    }

    /// Reviews every member of a team or bundle. `sha256` identifies the whole set.
    pub fn inspect_group(&self, root: &Path, kind: &str, namespace: &str, slug: &str) -> Result<Value, String> {
        let (name, members) = self.members(kind, namespace, slug)?;
        let mut reviews = Vec::new();
        let mut pins = Sha256::new();
        for m in &members {
            let mut review = self.inspect(m)?;
            review["state"] = json!(if is_installed(root, m, review["sha256"].as_str().unwrap_or("")) { "installed" } else { "pending" });
            review["kind"] = json!(m.kind);
            pins.update(format!("{}/{}@{}:{}\n", m.namespace, m.slug, m.version, review["sha256"].as_str().unwrap_or("")));
            reviews.push(review);
        }
        Ok(json!({"name": name, "kind": kind, "sha256": format!("{:x}", pins.finalize()), "members": reviews}))
    }

    /// Installs all missing members, or none: members are verified first, and a failure while writing
    /// removes what this call added. Members that are already installed at that version are skipped.
    pub fn install_group(&self, root: &Path, kind: &str, namespace: &str, slug: &str, sha256: &str) -> Result<Value, String> {
        let (_, members) = self.members(kind, namespace, slug)?;
        let mut verified = Vec::new();
        let mut pins = Sha256::new();
        for m in &members {
            let v = self.fetch(m)?;
            pins.update(format!("{}/{}@{}:{}\n", m.namespace, m.slug, m.version, v.plan.sha256));
            verified.push((m, v));
        }
        if format!("{:x}", pins.finalize()) != sha256.to_ascii_lowercase() {
            return Err("成员内容与审查时不一致，已拒绝安装".into());
        }
        let (mut added, mut skipped): (Vec<&str>, usize) = (Vec::new(), 0);
        for (m, v) in &verified {
            if is_installed(root, m, &v.plan.sha256) { skipped += 1; continue; }
            if let Err(e) = package::install(root, &m.slug, &v.plan, &v.files) {
                for slug in &added { let _ = fs::remove_dir_all(root.join(slug)); }
                return Err(format!("安装「{}」失败，已撤销本次新增的内容：{e}", m.slug));
            }
            added.push(&m.slug);
        }
        Ok(json!({"installed": added.len(), "skipped": skipped}))
    }

    /// Asks the market whether installed market packages have newer versions.
    pub fn check_updates(&self, root: &Path) -> Result<Value, String> {
        self.updates_for(installed_packages(root))
    }

    /// Like `check_updates`, for the skills directory and the plugins directory together.
    pub fn check_updates_all(&self, skills: &Path, plugins: &Path) -> Result<Value, String> {
        let mut installed = installed_packages(skills);
        installed.extend(installed_plugin_packages(plugins));
        self.updates_for(installed)
    }

    fn updates_for(&self, installed: Vec<Value>) -> Result<Value, String> {
        if installed.is_empty() { return Ok(json!({"results": []})); }
        let items: Vec<Value> = installed.iter().take(200)
            .map(|i| json!({"kind": i["kind"], "namespace": i["namespace"], "slug": i["slug"], "version": i["version"]})).collect();
        let body = json!({"items": items}).to_string();
        let url = self.url("/updates", &[])?;
        let raw = self.read(http()?.post(url).header("content-type", "application/json").body(body), MAX_JSON)?;
        let value: Value = serde_json::from_slice(&raw).map_err(|_| "小伴市场返回了无效数据".to_string())?;
        let results = value["results"].as_array().ok_or("小伴市场更新数据格式发生变化")?;
        Ok(json!({"results": results.iter().filter(|r| r["hasUpdate"] == true).collect::<Vec<_>>()}))
    }
}

const PLUGIN_MANIFESTS: [&str; 2] = [".codebuddy-plugin/plugin.json", ".workbuddy-plugin/plugin.json"];

fn has_plugin_manifest(manifest: &Value) -> bool {
    manifest["files"].as_array().is_some_and(|files| files.iter().any(|f| f["path"].as_str().is_some_and(|p| PLUGIN_MANIFESTS.contains(&p))))
}

/// Scripts a plugin or the skills it carries may run: anything in a `scripts` folder with a runnable extension.
fn is_plugin_script(path: &str) -> bool {
    path.split('/').any(|part| part == "scripts") && matches!(path.rsplit('.').next(), Some("py" | "js" | "mjs" | "cjs" | "sh" | "ps1" | "rb" | "bat" | "cmd"))
}

fn pinned(v: &Verified, sha256: &str) -> Result<(), String> {
    if !hex64(sha256) || v.plan.sha256 != sha256.to_ascii_lowercase() {
        return Err("清单摘要与审查时不一致，已拒绝安装".into());
    }
    Ok(())
}

fn normalize(item: &Value) -> Value {
    json!({
        "slug": item["slug"], "name": item["name"], "description": item["summary"], "version": item["version"],
        "namespace": item["namespace"], "iconUrl": item["iconUrl"], "source": item["source"], "origin": item["origin"],
        "downloads": item["downloads"], "stars": item["favorites"], "variant": item["variant"], "memberCount": item["memberCount"],
        "category": item["category"]["name"], "categorySlug": item["category"]["slug"], "kind": item["kind"], "expert": item["expert"],
    })
}

fn lock_of(root: &Path, slug: &str) -> Option<Value> {
    if !valid_slug(slug) { return None; }
    serde_json::from_slice(&fs::read(root.join(slug).join(".skillhub-lock.json")).ok()?).ok()
}
fn is_installed(root: &Path, p: &Package, sha256: &str) -> bool {
    lock_of(root, &p.slug).is_some_and(|l| l["origin"]["service"] == "xiaoban-market"
        && l["version"] == p.version.as_str() && l["manifestSha256"] == sha256)
}
/// Packages in the skills directory that were installed from the market.
pub fn installed_packages(root: &Path) -> Vec<Value> {
    let Ok(entries) = fs::read_dir(root) else { return vec![] };
    let mut result = Vec::new();
    for entry in entries.flatten() {
        let Some(name) = entry.file_name().to_str().map(str::to_string) else { continue };
        let Some(lock) = lock_of(root, &name) else { continue };
        if lock["origin"]["service"] != "xiaoban-market" || lock["slug"] != name.as_str() { continue; }
        let (Some(ns), Some(version)) = (lock["namespace"].as_str(), lock["version"].as_str()) else { continue };
        result.push(json!({"kind": lock["origin"]["kind"].as_str().unwrap_or("skill"), "namespace": ns, "slug": name,
            "version": version, "manifestSha256": lock["manifestSha256"]}));
    }
    result.sort_by(|a, b| a["slug"].as_str().cmp(&b["slug"].as_str()));
    result
}

/// Plugins installed from the market (their receipts), shaped like `installed_packages` for the update check.
pub fn installed_plugin_packages(plugins: &Path) -> Vec<Value> {
    plugin::installed(plugins).into_iter().filter(|r| r["source"]["service"] == "xiaoban-market").filter_map(|r| {
        Some(json!({"kind": r["source"]["kind"].as_str().unwrap_or("expert"), "namespace": r["source"]["namespace"].as_str()?, "slug": r["name"].as_str()?,
            "version": r["version"].as_str()?, "manifestSha256": r["manifestSha256"]}))
    }).collect()
}

/// A request from the market website: `xiaoban://market/{open|install|connect}/{kind}/{namespace}/{slug}?version=&sha256=`.
/// It can only ask the app to show something; the user still reviews and confirms everything.
pub fn parse_link(text: &str) -> Result<Value, String> {
    let bad = || "无效的小伴市场链接".to_string();
    if text.len() > 600 { return Err(bad()); }
    let url = Url::parse(text).map_err(|_| bad())?;
    if url.scheme() != "xiaoban" || url.host_str() != Some("market") || !url.username().is_empty() || url.port().is_some() {
        return Err(bad());
    }
    let parts: Vec<&str> = url.path_segments().map(|s| s.collect()).unwrap_or_default();
    let [action, kind, namespace, slug] = parts[..] else { return Err(bad()) };
    let kind = match kind { "skills" | "experts" | "connectors" => kind, _ => return Err(bad()) };
    let allowed = match action {
        "open" => true,
        "install" => kind != "connectors",
        "connect" => kind == "connectors",
        _ => false,
    };
    if !allowed { return Err(bad()); }
    check_coordinate(namespace, slug)?;
    let (mut version, mut sha256) = (None, None);
    for (key, value) in url.query_pairs() {
        match key.as_ref() {
            "version" if valid_version(&value) => version = Some(value.into_owned()),
            "sha256" if hex64(&value) => sha256 = Some(value.to_ascii_lowercase()),
            "version" | "sha256" => return Err(bad()),
            _ => {}
        }
    }
    Ok(json!({"action": action, "kind": kind, "namespace": namespace, "slug": slug, "version": version, "sha256": sha256}))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{io::{Read, Write}, net::TcpListener, sync::{Arc, Mutex}, thread};
    use zip::write::SimpleFileOptions;

    #[test]
    fn manifest_digest_matches_the_server_algorithm() {
        // The same vector is asserted by the market server's package tests.
        let files = [("SKILL.md", &b"# hi"[..]), ("scripts/run.py", &b"print(1)"[..])];
        let manifest = files.iter().map(|(p, b)| json!({"path": p, "size": b.len(), "sha256": format!("{:x}", Sha256::digest(b))})).collect::<Vec<_>>();
        let plan = market_plan("1.0.0", &manifest).unwrap();
        assert_eq!(plan.sha256, "ec97286e918f9e7ff5b49adfef3d8f61a1a502409ed93201e5046588cf0a427f");
        let reversed: Vec<Value> = manifest.iter().rev().cloned().collect();
        assert_eq!(market_plan("1.0.0", &reversed).unwrap().sha256, plan.sha256, "order of the list must not matter");
    }

    #[test]
    fn origins_are_site_roots_over_https_or_loopback() {
        assert_eq!(valid_origin("https://market.example.com/").unwrap(), "https://market.example.com");
        assert_eq!(valid_origin("http://127.0.0.1:55010").unwrap(), "http://127.0.0.1:55010");
        for bad in ["http://market.example.com", "https://u:p@market.example.com", "https://market.example.com/api", "https://market.example.com/?x=1", "ftp://x", "nonsense"] {
            assert!(valid_origin(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn links_are_strict_and_only_request_a_view() {
        let sha = "A".repeat(64);
        let ok = parse_link(&format!("xiaoban://market/install/skills/user_1/pdf-toolkit?version=1.2.0&sha256={sha}")).unwrap();
        assert_eq!((ok["action"].as_str(), ok["kind"].as_str(), ok["namespace"].as_str(), ok["slug"].as_str(), ok["version"].as_str()),
            (Some("install"), Some("skills"), Some("user_1"), Some("pdf-toolkit"), Some("1.2.0")));
        assert_eq!(ok["sha256"], "a".repeat(64));
        assert!(parse_link("xiaoban://market/open/experts/acme/helper").is_ok());
        assert!(parse_link("xiaoban://market/connect/connectors/acme/web-search").is_ok());
        for bad in [
            "https://market/install/skills/a/b", "xiaoban://other/install/skills/a/b", "xiaoban://market/install/connectors/a/b",
            "xiaoban://market/connect/skills/a/b", "xiaoban://market/run/skills/a/b", "xiaoban://market/install/skills/a",
            "xiaoban://market/install/skills/a/b/c", "xiaoban://market/install/skills/../b", "xiaoban://market/install/skills/a/B",
            "xiaoban://market/install/skills/a/pdf?version=../x", "xiaoban://market/install/skills/a/pdf?sha256=short",
            "xiaoban://user@market/install/skills/a/pdf", "xiaoban://market:80/install/skills/a/pdf",
        ] {
            assert!(parse_link(bad).is_err(), "{bad}");
        }
    }

    // ---- a small in-process market ----
    struct Fixture { origin: String, hits: Arc<Mutex<Vec<String>>>, tamper: Arc<Mutex<bool>> }
    fn zip_of(files: &[(&str, &[u8])]) -> Vec<u8> {
        let mut w = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        for (path, bytes) in files {
            w.start_file(*path, SimpleFileOptions::default()).unwrap();
            w.write_all(bytes).unwrap();
        }
        w.finish().unwrap().into_inner()
    }
    fn manifest_of(files: &[(&str, &[u8])]) -> Vec<Value> {
        files.iter().map(|(p, b)| json!({"path": p, "size": b.len(), "sha256": format!("{:x}", Sha256::digest(b))})).collect()
    }
    fn package_files(name: &str, extra: &str) -> Vec<(String, Vec<u8>)> {
        vec![("SKILL.md".into(), format!("---\nname: {name}\ndescription: demo\n---\nUse it. {extra}\n").into_bytes()),
             ("scripts/run.py".into(), format!("print('{extra}')\n").into_bytes())]
    }
    fn serve() -> Fixture {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let origin = format!("http://{}", listener.local_addr().unwrap());
        let hits = Arc::new(Mutex::new(Vec::new()));
        let tamper = Arc::new(Mutex::new(false));
        let (h, t) = (hits.clone(), tamper.clone());
        thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { break };
                let (h, t) = (h.clone(), t.clone());
                thread::spawn(move || {
                    let mut buf = [0u8; 8192];
                    let n = stream.read(&mut buf).unwrap_or(0);
                    let text = String::from_utf8_lossy(&buf[..n]).to_string();
                    let line = text.lines().next().unwrap_or("").to_string();
                    let target = line.split_whitespace().nth(1).unwrap_or("").to_string();
                    h.lock().unwrap().push(format!("{} {}", line.split_whitespace().next().unwrap_or(""), target));
                    let path = target.split('?').next().unwrap_or("");
                    let (status, content_type, body) = route(path, &text, *t.lock().unwrap());
                    let head = format!("HTTP/1.1 {status} X\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len());
                    let _ = stream.write_all(head.as_bytes());
                    let _ = stream.write_all(&body);
                });
            }
        });
        Fixture { origin, hits, tamper }
    }
    fn version_doc(name: &str, extra: &str) -> (Value, Vec<u8>) {
        let owned = package_files(name, extra);
        let files: Vec<(&str, &[u8])> = owned.iter().map(|(p, b)| (p.as_str(), b.as_slice())).collect();
        let zip = zip_of(&files);
        let plan = market_plan("1.0.0", &manifest_of(&files)).unwrap();
        (json!({"kind": "skill", "namespace": "acme", "slug": name, "name": name, "summary": "demo skill", "version": "1.0.0",
            "manifestSha256": plan.sha256, "packageSha256": format!("{:x}", Sha256::digest(&zip)), "size": zip.len(), "files": manifest_of(&files)}), zip)
    }
    const PERSONA: &str = "---\nname: accounting-supervisor\ndescription: Senior accountant.\ndisplayName:\n  en: AI Accounting Supervisor\n  zh: AI 会计主管\nprofession:\n  en: Accounting Supervisor\n  zh: 会计主管\nmaxTurns: 50\n---\n# AI 会计主管\n\n你是会计主管。\n";
    fn plugin_files() -> Vec<(String, Vec<u8>)> {
        let manifest = json!({"name": "accounting-supervisor", "version": "1.0.0", "description": "Accountant.", "agents": ["./agents/accounting-supervisor.md"],
            "skills": ["./skills/ledger-check"], "expertType": "agent", "agentName": "accounting-supervisor"});
        vec![
            (".codebuddy-plugin/plugin.json".into(), manifest.to_string().into_bytes()),
            ("agents/accounting-supervisor.md".into(), PERSONA.as_bytes().to_vec()),
            ("skills/ledger-check/SKILL.md".into(), b"---\nname: ledger-check\ndescription: d\n---\nCheck the ledger.".to_vec()),
            ("skills/ledger-check/scripts/check.py".into(), b"print('ok')\n".to_vec()),
        ]
    }
    fn plugin_doc() -> (Value, Vec<u8>) {
        let owned = plugin_files();
        let files: Vec<(&str, &[u8])> = owned.iter().map(|(p, b)| (p.as_str(), b.as_slice())).collect();
        let zip = zip_of(&files);
        let mut sorted = manifest_of(&files);
        sorted.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
        let mut hash = Sha256::new();
        for f in &sorted { hash.update(format!("{}  {}\n", f["sha256"].as_str().unwrap(), f["path"].as_str().unwrap()).as_bytes()); }
        (json!({"kind": "expert", "namespace": "acme", "slug": "accounting-supervisor", "name": "AI 会计主管", "summary": "会计核算", "version": "1.0.0",
            "manifestSha256": format!("{:x}", hash.finalize()), "packageSha256": format!("{:x}", Sha256::digest(&zip)), "size": zip.len(), "files": sorted}), zip)
    }
    fn route(path: &str, request: &str, tamper: bool) -> (u16, &'static str, Vec<u8>) {
        let json_ok = |v: Value| (200, "application/json", v.to_string().into_bytes());
        match path {
            "/api/v1/market/skills" => json_ok(json!({"results": [{"kind": "skill", "namespace": "acme", "slug": "pdf-kit", "name": "PDF", "summary": "处理 PDF", "version": "1.0.0",
                "source": "official", "downloads": 3, "favorites": 1, "variant": "single", "category": {"slug": "doc", "name": "文档处理"}}], "near": [], "page": 1, "total": 1, "hasMore": false})),
            "/api/v1/market/categories" => json_ok(json!({"results": [{"slug": "doc", "name": "文档处理", "count": 2}]})),
            "/api/v1/market/skills/acme/pdf-kit/versions/1.0.0" => {
                let (mut doc, _) = version_doc("pdf-kit", "a");
                if tamper { doc["manifestSha256"] = json!("0".repeat(64)); }
                json_ok(doc)
            }
            "/api/v1/market/skills/acme/pdf-kit/versions/1.0.0/download" => {
                let (_, mut zip) = version_doc("pdf-kit", "a");
                if tamper { let n = zip.len(); zip[n / 2] ^= 0xff; }
                (200, "application/zip", zip)
            }
            "/api/v1/market/skills/acme/gone/versions/1.0.0" => (404, "application/json", json!({"error": "not_found", "message": "该版本已撤回，请使用更新的版本"}).to_string().into_bytes()),
            "/api/v1/market/experts/acme/accounting-supervisor" => json_ok(json!({"item": {"name": "AI 会计主管", "variant": "single", "version": "1.0.0", "iconUrl": "/api/v1/market/assets/abc",
                "metadata": {"expert": {"avatarUrl": "/api/v1/market/assets/abc", "members": [{"avatarUrl": "https://example.com/a.png"}]}}}, "members": []})),
            "/api/v1/market/experts/acme/accounting-supervisor/versions/1.0.0" => json_ok(plugin_doc().0),
            "/api/v1/market/experts/acme/accounting-supervisor/versions/1.0.0/download" => (200, "application/zip", plugin_doc().1),
            "/api/v1/market/experts/acme/team-a" => json_ok(json!({"item": {"name": "小组", "variant": "team", "version": "1.0.0"},
                "members": [{"namespace": "acme", "slug": "member-one", "version": "1.0.0"}, {"namespace": "acme", "slug": "member-two", "version": "1.0.0"}]})),
            p if p.starts_with("/api/v1/market/experts/acme/member-") && p.ends_with("/versions/1.0.0") => {
                let slug = p.rsplit('/').nth(2).unwrap();
                let (mut doc, _) = version_doc(slug, slug);
                doc["kind"] = json!("expert");
                json_ok(doc)
            }
            p if p.starts_with("/api/v1/market/experts/acme/member-") && p.ends_with("/download") => {
                let slug = p.rsplit('/').nth(3).unwrap();
                (200, "application/zip", version_doc(slug, slug).1)
            }
            "/api/v1/market/updates" => {
                assert!(request.contains("pdf-kit") || request.contains("accounting-supervisor"), "an installed package must be reported");
                json_ok(json!({"results": [{"kind": "skill", "namespace": "acme", "slug": "pdf-kit", "installed": "1.0.0", "available": true, "latest": "1.1.0", "hasUpdate": true},
                    {"kind": "skill", "namespace": "acme", "slug": "other", "installed": "1", "available": true, "hasUpdate": false}]}))
            }
            _ => (404, "application/json", json!({"error": "not_found", "message": "条目不存在"}).to_string().into_bytes()),
        }
    }
    fn temp(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("xiaoban-market-{name}-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }
    fn skill(slug: &str) -> Package { Package { kind: "skills".into(), namespace: "acme".into(), slug: slug.into(), version: "1.0.0".into() } }

    fn expert(slug: &str) -> Package { Package { kind: "experts".into(), namespace: "acme".into(), slug: slug.into(), version: "1.0.0".into() } }

    #[test]
    fn a_plugin_expert_is_reviewed_pinned_and_installed_as_a_plugin() {
        let fx = serve();
        let market = Market::new(&fx.origin).unwrap();
        assert!(market.is_plugin(&expert("accounting-supervisor")).unwrap());
        assert!(!market.is_plugin(&skill("pdf-kit")).unwrap());
        let review = market.inspect(&expert("accounting-supervisor")).unwrap();
        assert_eq!((review["kind"].as_str(), review["pluginKind"].as_str(), review["manifest"]["fileCount"].as_u64()), (Some("plugin"), Some("agent"), Some(4)));
        assert!(review["content"].as_str().unwrap().contains("你是会计主管"));
        assert_eq!((review["skills"][0].as_str(), review["scripts"][0]["path"].as_str()), (Some("ledger-check"), Some("skills/ledger-check/scripts/check.py")));
        assert_eq!(review["agents"][0]["profession"], "会计主管");
        let (plugins, backups, skills) = (temp("plugins"), temp("plugin-backups"), temp("skills"));
        let sha = review["sha256"].as_str().unwrap().to_string();
        assert!(market.install_plugin(&plugins, &backups, &expert("accounting-supervisor"), &"0".repeat(64)).unwrap_err().contains("不一致"));
        assert!(plugin::installed(&plugins).is_empty());
        let receipt = market.install_plugin(&plugins, &backups, &expert("accounting-supervisor"), &sha).unwrap();
        assert_eq!((receipt["source"]["namespace"].as_str(), receipt["kind"].as_str()), (Some("acme"), Some("agent")));
        assert!(plugins.join("accounting-supervisor/skills/ledger-check/SKILL.md").is_file());
        assert_eq!(plugin::read_agent(&plugins, "accounting-supervisor", "accounting-supervisor").unwrap().max_turns, Some(50));
        // The update check sees plugins too; a skill package is not mistaken for one.
        assert_eq!(installed_plugin_packages(&plugins)[0]["slug"], "accounting-supervisor");
        assert_eq!(market.check_updates_all(&skills, &plugins).unwrap()["results"][0]["slug"], "pdf-kit");
        // Avatars the market keeps itself are made loadable; remote ones are left alone.
        let detail = market.detail("experts", "acme", "accounting-supervisor").unwrap();
        assert_eq!(detail["item"]["iconUrl"], format!("{}/api/v1/market/assets/abc", fx.origin).as_str());
        assert_eq!(detail["item"]["metadata"]["expert"]["members"][0]["avatarUrl"], "https://example.com/a.png");
        for dir in [plugins, backups, skills] { let _ = fs::remove_dir_all(dir); }
    }

    #[test]
    fn browse_and_categories_use_the_store_page_shape() {
        let fx = serve();
        let market = Market::new(&fx.origin).unwrap();
        let page = market.browse("skills", "pdf & 表格", 1, "doc", "official", "downloads", "").unwrap();
        assert_eq!(page["results"][0]["description"], "处理 PDF");
        assert_eq!(page["results"][0]["category"], "文档处理");
        assert_eq!((page["total"].as_u64(), page["hasMore"].as_bool()), (Some(1), Some(false)));
        let hit = fx.hits.lock().unwrap().iter().find(|h| h.contains("/skills?")).cloned().unwrap();
        assert!(hit.contains("sortBy=downloads") && hit.contains("source=official") && !hit.contains("variant="), "{hit}");
        assert_eq!(market.categories("skills").unwrap()["results"][0]["key"], "doc");
        assert!(market.browse("skills", "", 0, "", "", "", "").is_err());
        assert!(market.browse("nonsense", "", 1, "", "", "", "").is_err());
    }

    #[test]
    fn inspect_then_install_writes_a_verified_package_with_a_market_lock() {
        let fx = serve();
        let market = Market::new(&fx.origin).unwrap();
        let review = market.inspect(&skill("pdf-kit")).unwrap();
        assert_eq!(review["manifest"]["fileCount"], 2);
        assert_eq!(review["scripts"][0]["path"], "scripts/run.py");
        assert!(review["content"].as_str().unwrap().contains("Use it."));
        let sha = review["sha256"].as_str().unwrap().to_string();
        let root = temp("install");
        assert!(market.install(&root, &skill("pdf-kit"), &"0".repeat(64)).unwrap_err().contains("不一致"));
        assert!(!root.join("pdf-kit").exists(), "a wrong digest must not leave files behind");
        market.install(&root, &skill("pdf-kit"), &sha).unwrap();
        let lock: Value = serde_json::from_slice(&fs::read(root.join("pdf-kit/.skillhub-lock.json")).unwrap()).unwrap();
        assert_eq!((lock["origin"]["service"].as_str(), lock["origin"]["kind"].as_str(), lock["namespace"].as_str(), lock["manifestSha256"].as_str()),
            (Some("xiaoban-market"), Some("skill"), Some("acme"), Some(sha.as_str())));
        assert!(root.join("pdf-kit/scripts/run.py").is_file());
        assert!(market.install(&root, &skill("pdf-kit"), &sha).unwrap_err().contains("already exists"));
        assert_eq!(installed_packages(&root)[0]["slug"], "pdf-kit");
        let updates = market.check_updates(&root).unwrap();
        assert_eq!(updates["results"].as_array().unwrap().len(), 1);
        assert_eq!(updates["results"][0]["latest"], "1.1.0");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn a_digest_or_archive_that_disagrees_with_the_listing_is_refused() {
        let fx = serve();
        let market = Market::new(&fx.origin).unwrap();
        *fx.tamper.lock().unwrap() = true;
        assert!(market.inspect(&skill("pdf-kit")).unwrap_err().contains("清单摘要"));
        assert_eq!(market.inspect(&skill("gone")).unwrap_err(), "该版本已撤回，请使用更新的版本");
        assert!(market.inspect(&Package { kind: "connectors".into(), ..skill("pdf-kit") }).is_err());
        assert!(market.inspect(&Package { version: "../1".into(), ..skill("pdf-kit") }).is_err());
    }

    #[test]
    fn update_keeps_the_old_version_as_a_backup_and_only_replaces_market_packages() {
        let fx = serve();
        let market = Market::new(&fx.origin).unwrap();
        let root = temp("update");
        let backups = temp("backups");
        let sha = market.inspect(&skill("pdf-kit")).unwrap()["sha256"].as_str().unwrap().to_string();
        // Not installed from the market yet: nothing to replace.
        fs::create_dir_all(root.join("pdf-kit")).unwrap();
        fs::write(root.join("pdf-kit/SKILL.md"), "mine").unwrap();
        assert!(market.update(&root, &backups, &skill("pdf-kit"), &sha).unwrap_err().contains("not managed"));
        assert_eq!(fs::read_to_string(root.join("pdf-kit/SKILL.md")).unwrap(), "mine");
        fs::remove_dir_all(root.join("pdf-kit")).unwrap();
        market.install(&root, &skill("pdf-kit"), &sha).unwrap();
        market.update(&root, &backups, &skill("pdf-kit"), &sha).unwrap();
        assert!(root.join("pdf-kit/SKILL.md").is_file());
        assert_eq!(fs::read_dir(&backups).unwrap().count(), 1, "the previous directory is kept");
        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(backups).unwrap();
    }

    #[test]
    fn a_team_installs_all_missing_members_or_none() {
        let fx = serve();
        let market = Market::new(&fx.origin).unwrap();
        let root = temp("team");
        let review = market.inspect_group(&root, "experts", "acme", "team-a").unwrap();
        assert_eq!(review["members"].as_array().unwrap().len(), 2);
        assert_eq!(review["members"][0]["state"], "pending");
        let sha = review["sha256"].as_str().unwrap().to_string();
        assert!(market.install_group(&root, "experts", "acme", "team-a", &"1".repeat(64)).is_err());
        assert!(!root.join("member-one").exists());
        // A member that cannot be written makes the whole group roll back.
        fs::write(root.join("member-two"), "a file is in the way").unwrap();
        let error = market.install_group(&root, "experts", "acme", "team-a", &sha).unwrap_err();
        assert!(error.contains("member-two") && error.contains("撤销"), "{error}");
        assert!(!root.join("member-one").exists(), "the member added before the failure is removed");
        fs::remove_file(root.join("member-two")).unwrap();
        assert_eq!(market.install_group(&root, "experts", "acme", "team-a", &sha).unwrap()["installed"], 2);
        let again = market.install_group(&root, "experts", "acme", "team-a", &sha).unwrap();
        assert_eq!((again["installed"].as_u64(), again["skipped"].as_u64()), (Some(0), Some(2)));
        assert_eq!(market.inspect_group(&root, "experts", "acme", "team-a").unwrap()["members"][1]["state"], "installed");
        fs::remove_dir_all(root).unwrap();
    }
}
