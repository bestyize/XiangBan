//! WorkBuddy 格式的插件（专家、专家团、连接器、技能包）：本地安装、列表和卸载。
//! 和 AI 商店其他命令一样：只有控制窗口能调用，网络和文件工作不占用界面线程，安装前先审查、按清单摘要锁定。
//! 安装只写文件，不执行插件里的任何代码；插件自带的技能由技能加载器作为只读根读取。
use super::{store, Dirs};
use agent_skill_discovery::plugin;
use serde_json::{json, Value};
use std::path::PathBuf;

const MAX_PLUGIN_BYTES: u64 = 12 * 1024 * 1024;

fn read_zip(path: &str) -> Result<Vec<u8>, String> {
    let path = PathBuf::from(path);
    let meta = std::fs::metadata(&path).map_err(|e| format!("无法读取插件包：{e}"))?;
    if !meta.is_file() || meta.len() > MAX_PLUGIN_BYTES {
        return Err("插件包必须是不超过 12 MiB 的 ZIP 文件".into());
    }
    std::fs::read(&path).map_err(|e| format!("无法读取插件包：{e}"))
}

/// 已安装的插件（每个插件的安装回执）。
#[tauri::command]
pub async fn ai_plugin_installed(app: tauri::AppHandle, window: tauri::WebviewWindow) -> Result<Value, String> {
    store::control(&window)?;
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = store::MUTATIONS.lock().map_err(|e| e.to_string())?;
        Ok(Value::Array(plugin::installed(&Dirs::of(&app)?.plugins())))
    })
    .await
    .map_err(|e| e.to_string())?
}

/// 审查本地插件包：返回名称、版本、类型、文件清单、提示和清单摘要，不写任何文件。
/// 选择要安装的插件包（.zip）。WebView 的文件输入拿不到绝对路径，所以走系统对话框；取消时返回 `None`。
#[tauri::command]
pub async fn ai_plugin_pick_file(app: tauri::AppHandle, window: tauri::WebviewWindow) -> Result<Option<String>, String> {
    store::control(&window)?;
    use tauri_plugin_dialog::DialogExt;
    tauri::async_runtime::spawn_blocking(move || {
        let chosen = app.dialog().file().set_parent(&window).set_title("选择插件包").add_filter("插件包", &["zip"]).blocking_pick_file();
        let Some(chosen) = chosen else { return Ok(None) };
        Ok(Some(chosen.into_path().map_err(|e| e.to_string())?.display().to_string()))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn ai_plugin_inspect_file(window: tauri::WebviewWindow, path: String) -> Result<Value, String> {
    store::control(&window)?;
    tauri::async_runtime::spawn_blocking(move || Ok(plugin::inspect_zip(&read_zip(&path)?)?.summary()))
        .await
        .map_err(|e| e.to_string())?
}

/// 安装已审查的本地插件包。`sha256` 是审查时看到的清单摘要，包被换过就拒绝。
#[tauri::command]
pub async fn ai_plugin_install_file(app: tauri::AppHandle, window: tauri::WebviewWindow, path: String, sha256: String) -> Result<Value, String> {
    store::control(&window)?;
    tauri::async_runtime::spawn_blocking(move || {
        let pkg = plugin::inspect_zip(&read_zip(&path)?)?;
        if !pkg.manifest_sha256.eq_ignore_ascii_case(&sha256) {
            return Err("插件包与审查时不一致，已拒绝安装".to_string());
        }
        let _guard = store::MUTATIONS.lock().map_err(|e| e.to_string())?;
        let dirs = Dirs::of(&app)?;
        plugin::install(&dirs.plugins(), &dirs.plugin_backups(), &pkg, "local", json!({"service": "local-file"}))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn ai_plugin_remove(app: tauri::AppHandle, window: tauri::WebviewWindow, name: String) -> Result<(), String> {
    store::control(&window)?;
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = store::MUTATIONS.lock().map_err(|e| e.to_string())?;
        plugin::remove(&Dirs::of(&app)?.plugins(), &name)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Installed experts and teams, as the composer's expert menu and the AI store list them. `plugin` marks them
/// as WorkBuddy-style personas (the older SkillHub packs have none).
pub fn installed_experts(plugins: &std::path::Path) -> Vec<Value> {
    plugin::installed(plugins).into_iter().filter(|r| matches!(r["kind"].as_str(), Some("agent" | "team"))).filter_map(|receipt| {
        let name = receipt["name"].as_str()?.to_string();
        let manifest: Value = std::fs::read(plugins.join(&name).join(".codebuddy-plugin/plugin.json"))
            .or_else(|_| std::fs::read(plugins.join(&name).join(".workbuddy-plugin/plugin.json")))
            .ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or(Value::Null);
        let pick = |key: &str, zh: bool| -> String {
            let v = &manifest[key];
            let (a, b) = if zh { ("zh", "en") } else { ("en", "zh") };
            v[a].as_str().filter(|s| !s.is_empty()).or_else(|| v[b].as_str()).or_else(|| v.as_str()).unwrap_or("").to_string()
        };
        let prompts: Vec<String> = manifest["quickPrompts"].as_array().into_iter().flatten()
            .map(|p| p["zh"].as_str().filter(|s| !s.is_empty()).or_else(|| p["en"].as_str()).unwrap_or("").to_string()).filter(|s| !s.is_empty()).collect();
        let tags: Vec<String> = manifest["tags"].as_array().into_iter().flatten()
            .map(|p| p["zh"].as_str().filter(|s| !s.is_empty()).or_else(|| p["en"].as_str()).or_else(|| p.as_str()).unwrap_or("").to_string()).filter(|s| !s.is_empty()).collect();
        let display = pick("displayName", true);
        let shown = if display.is_empty() { name.clone() } else { display };
        let described = pick("displayDescription", true);
        let description = if described.is_empty() { manifest["description"].as_str().unwrap_or("").to_string() } else { described };
        let profession = pick("profession", true);
        let agent = manifest["teamInfo"]["leadAgent"].as_str().or(manifest["agentName"].as_str()).map(String::from);
        let has_avatar = manifest["avatar"].as_str().is_some_and(|a| !a.is_empty());
        let from_market = receipt["source"]["service"] == "xiaoban-market";
        Some(json!({
            "slug": name, "plugin": name, "kind": receipt["kind"], "ready": true,
            "name": shown, "profession": profession, "description": description, "agent": agent,
            "quickPrompts": prompts, "tags": tags,
            "version": receipt["version"], "hasAvatar": has_avatar,
            "namespace": receipt["source"]["namespace"], "manifestSha256": receipt["manifestSha256"], "files": receipt["files"],
            "market": from_market, "marketKind": "expert",
        }))
    }).collect()
}

/// An installed expert in the shape of the store's installed list, so the AI store page treats it like any package.
pub fn as_installed_entry(e: Value) -> Value {
    json!({
        "slug": e["slug"], "name": e["name"], "description": e["description"], "version": e["version"], "namespace": e["namespace"],
        "iconUrl": Value::Null, "removable": true, "market": e["market"], "marketKind": "expert",
        "files": e["files"], "manifestSha256": e["manifestSha256"],
        "source": if e["market"] == true { "小伴市场" } else { "本地" },
        "plugin": true, "profession": e["profession"], "agent": e["agent"], "quickPrompts": e["quickPrompts"], "tags": e["tags"],
        "hasAvatar": e["hasAvatar"], "pluginKind": e["kind"],
    })
}

/// The expert's avatar as a data URL (the page cannot read the plugin folder directly). `None` when it has none.
#[tauri::command]
pub async fn ai_plugin_avatar(app: tauri::AppHandle, window: tauri::WebviewWindow, name: String) -> Result<Option<String>, String> {
    store::control(&window)?;
    tauri::async_runtime::spawn_blocking(move || {
        use base64::Engine;
        let plugins = Dirs::of(&app)?.plugins();
        plugin::read_receipt(&plugins, &name)?;
        let manifest: Value = std::fs::read(plugins.join(&name).join(".codebuddy-plugin/plugin.json"))
            .or_else(|_| std::fs::read(plugins.join(&name).join(".workbuddy-plugin/plugin.json")))
            .ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or(Value::Null);
        let Some(rel) = manifest["avatar"].as_str().map(|a| a.trim_start_matches("./")).filter(|a| !a.is_empty()) else { return Ok(None) };
        if rel.contains("..") || rel.starts_with('/') || rel.contains(':') { return Ok(None); }
        let bytes = std::fs::read(plugins.join(&name).join(rel)).map_err(|e| e.to_string())?;
        let mime = match bytes.as_slice() {
            b if b.starts_with(&[0x89, b'P', b'N', b'G']) => "image/png",
            b if b.starts_with(&[0xFF, 0xD8, 0xFF]) => "image/jpeg",
            b if b.len() > 12 && &b[0..4] == b"RIFF" && &b[8..12] == b"WEBP" => "image/webp",
            _ => return Ok(None),
        };
        if bytes.len() > 4 * 1024 * 1024 { return Ok(None); }
        Ok(Some(format!("data:{mime};base64,{}", base64::engine::general_purpose::STANDARD.encode(bytes))))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn expert_zip(kind: &str) -> Vec<u8> {
        let manifest = json!({
            "name": "accounting-supervisor", "version": "1.2.0", "description": "Accountant.",
            "agents": ["./agents/accounting-supervisor.md"], "expertType": kind, "agentName": "accounting-supervisor",
            "teamInfo": if kind == "team" { json!({"leadAgent": "accounting-supervisor", "memberAgents": []}) } else { Value::Null },
            "displayName": {"en": "AI Accounting Supervisor", "zh": "AI 会计主管"}, "profession": {"en": "Accounting Supervisor", "zh": "会计主管"},
            "displayDescription": {"zh": "聚焦会计核算与财务分析", "en": "Accounting"}, "avatar": "avatars/expert.png",
            "tags": [{"en": "Tax", "zh": "税务核算"}, {"en": "Ledger", "zh": "账簿"}], "quickPrompts": [{"en": "Review", "zh": "请审核这笔分录"}, {"en": "Two", "zh": "二"}],
        });
        let mut w = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        for (name, data) in [
            (".codebuddy-plugin/plugin.json", manifest.to_string().into_bytes()),
            ("agents/accounting-supervisor.md", b"---\nname: accounting-supervisor\ndescription: d\n---\n# persona".to_vec()),
            ("avatars/expert.png", [&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A][..], &[0u8; 16][..]].concat()),
        ] {
            w.start_file(name, zip::write::SimpleFileOptions::default()).unwrap();
            w.write_all(&data).unwrap();
        }
        w.finish().unwrap().into_inner()
    }

    #[test]
    fn installed_experts_list_their_presentation_and_skip_other_plugins() {
        let root = std::env::temp_dir().join(format!("installed-experts-{}", std::process::id()));
        let (plugins, backups) = (root.join("plugins"), root.join("backups"));
        let pkg = plugin::inspect_zip(&expert_zip("agent")).unwrap();
        plugin::install(&plugins, &backups, &pkg, "xiaoban-market", json!({"service": "xiaoban-market", "namespace": "acme", "kind": "expert"})).unwrap();
        // 一个只有技能的插件不算专家。
        let skills_only = {
            let mut w = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
            w.start_file(".codebuddy-plugin/plugin.json", zip::write::SimpleFileOptions::default()).unwrap();
            w.write_all(br#"{"name":"just-skills","version":"1.0.0","description":"d"}"#).unwrap();
            w.finish().unwrap().into_inner()
        };
        plugin::install(&plugins, &backups, &plugin::inspect_zip(&skills_only).unwrap(), "local", json!({})).unwrap();
        let experts = installed_experts(&plugins);
        assert_eq!(experts.len(), 1);
        let e = &experts[0];
        assert_eq!((e["slug"].as_str(), e["name"].as_str(), e["profession"].as_str(), e["agent"].as_str()), (Some("accounting-supervisor"), Some("AI 会计主管"), Some("会计主管"), Some("accounting-supervisor")));
        assert_eq!((e["quickPrompts"][0].as_str(), e["tags"].as_array().map(Vec::len), e["hasAvatar"].as_bool(), e["market"].as_bool()), (Some("请审核这笔分录"), Some(2), Some(true), Some(true)));
        assert_eq!((e["kind"].as_str(), e["version"].as_str()), (Some("agent"), Some("1.2.0")));
        // 专家团用主理人。
        let team = plugin::inspect_zip(&expert_zip("team")).unwrap();
        plugin::install(&plugins, &backups, &team, "local", json!({})).unwrap();
        assert_eq!(installed_experts(&plugins)[0]["kind"], "team");
        let _ = std::fs::remove_dir_all(root);
    }
}
