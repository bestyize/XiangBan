//! 连接器插件（WorkBuddy 的连接器）的桌面端命令：已安装列表、配置令牌并连接、启停、解绑、重新授权，
//! 以及输入框 `@` 菜单的数据。插件装在 `<存储>/chat/plugins/<name>/`，连接、凭据保险箱和 OAuth 都由 `ConnectorManager` 管；
//! 这里只把它们拼成界面要的样子，列表里永远不含令牌的值，只有已保存的字段名。
use super::*;
use agent_connector_tools::plugin::{AuthSpec, ConnectorPlugin, ServerSpec, Transport};
use agent_skill_discovery::plugin;
use std::collections::BTreeMap;
use std::sync::atomic::AtomicBool as Flag;

/// 图标最大读取字节数；更大的不显示。
const MAX_ICON_BYTES: u64 = 512 * 1024;
/// 启动后第一次看连接器列表时，在后台把已启用的连接器恢复起来（可能要拉起本地命令或联网），只做一次。
static RESTORE_STARTED: Flag = Flag::new(false);

fn icon_data_url(plugin: &ConnectorPlugin) -> Option<String> {
    use base64::Engine;
    let path = plugin.icon.as_ref()?;
    if fs::metadata(path).ok()?.len() > MAX_ICON_BYTES {
        return None;
    }
    let mime = match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        _ => return None,
    };
    Some(format!("data:{mime};base64,{}", base64::engine::general_purpose::STANDARD.encode(fs::read(path).ok()?)))
}

fn schema_json(spec: &ServerSpec) -> Option<Value> {
    let schema = spec.schema()?;
    Some(json!({
        "title": schema.title, "titleEn": schema.title_en, "description": schema.description, "descriptionEn": schema.description_en,
        "fields": schema.fields.iter().map(|f| json!({
            "key": f.key, "label": f.label, "labelEn": f.label_en,
            "type": if f.field_type == "password" { "password" } else { "text" },
            "required": f.required, "placeholder": f.placeholder, "description": f.description,
        })).collect::<Vec<_>>(),
    }))
}

fn first<'a>(options: &[&'a str]) -> &'a str {
    options.iter().copied().find(|s| !s.trim().is_empty()).unwrap_or("")
}

/// 一个已安装的连接器插件里的一项服务；多数插件只有一项。
struct Entry {
    plugin: ConnectorPlugin,
    server: String,
    id: String,
    market: bool,
    namespace: Option<String>,
}

fn entries(plugins: &Path) -> Vec<Entry> {
    let mut out = Vec::new();
    for receipt in plugin::installed(plugins) {
        if receipt["kind"].as_str() != Some("connector") {
            continue;
        }
        let Some(name) = receipt["name"].as_str() else { continue };
        let Ok(loaded) = ConnectorPlugin::load(&plugins.join(name)) else { continue };
        let market = receipt["source"]["service"] == "xiaoban-market";
        for spec in &loaded.servers {
            out.push(Entry { id: loaded.connection_id(&spec.name), server: spec.name.clone(), plugin: loaded.clone(), market, namespace: receipt["source"]["namespace"].as_str().map(String::from) });
        }
    }
    out
}

/// 界面上的状态：已停用、已连接、需要授权、出错、恢复中、未连接。
fn status_of(status: Option<&Value>, restoring: bool) -> (&'static str, bool, Option<String>) {
    let Some(status) = status else { return ("disconnected", false, None) };
    let enabled = status["enabled"] == true;
    if !enabled {
        return ("disabled", false, None);
    }
    if status["connected"] == true {
        return ("connected", true, None);
    }
    match status["error"].as_str().filter(|e| !e.is_empty()) {
        Some(error) if error.contains("401") || error.contains("授权") || error.to_ascii_lowercase().contains("oauth") => ("needs_auth", true, Some(error.to_string())),
        Some(error) => ("error", true, Some(error.to_string())),
        None if restoring => ("connecting", true, None),
        None => ("disconnected", true, None),
    }
}

fn entry_json(manager: &agent_connector_tools::ConnectorManager, e: &Entry, statuses: &Value, restoring: bool) -> Value {
    let spec = e.plugin.server(&e.server).expect("server of a loaded plugin");
    let status = statuses.as_array().and_then(|a| a.iter().find(|s| s["id"].as_str() == Some(e.id.as_str())));
    let (state, enabled, error) = status_of(status, restoring);
    let d = &e.plugin.display;
    let name = first(&[&d.name_zh, &d.name, &d.name_en, &e.plugin.name]);
    let multi = e.plugin.servers.len() > 1;
    let skills: Vec<String> = e.plugin.skills_dir.as_ref().and_then(|dir| fs::read_dir(dir).ok()).into_iter().flatten().flatten()
        .filter(|f| f.path().join("SKILL.md").is_file()).filter_map(|f| f.file_name().to_str().map(String::from)).collect();
    let browser_auth = spec.browser_auth_config().is_some();
    let auth = if browser_auth { "oauth" } else { match &spec.auth {
        AuthSpec::None if spec.transport != Transport::Stdio => "auto",
        AuthSpec::None => "none",
        AuthSpec::Token { .. } => "token",
        AuthSpec::OAuth => "oauth",
        AuthSpec::Cli => "cli",
    }};
    let examples: Vec<&String> = if d.examples_zh.is_empty() { d.examples_en.iter().collect() } else { d.examples_zh.iter().collect() };
    json!({
        "id": e.id, "plugin": e.plugin.name, "server": e.server,
        "name": if multi { format!("{name} · {}", e.server) } else { name.to_string() },
        "nameEn": first(&[&d.name_en, &d.name]),
        "description": first(&[&d.description_zh, &d.description_en]),
        "version": e.plugin.version, "source": if e.market { "market" } else { "local" },
        "namespace": e.namespace,
        "iconUrl": icon_data_url(&e.plugin),
        "transport": spec.transport, "command": spec.command, "args": spec.args,
        "auth": { "type": auth, "browser": browser_auth, "schema": schema_json(spec), "configured": manager.configured_plugin_fields(&e.id).unwrap_or_default() },
        "status": state, "enabled": enabled, "error": error,
        "tools": manager.tool_list(&e.id).into_iter().map(|(name, description)| json!({"name": name, "description": description})).collect::<Vec<_>>(),
        "examples": examples, "skills": skills,
        "unsupported": spec.unsupported(),
    })
}

fn find(plugins: &Path, id: &str) -> Result<Entry, String> {
    entries(plugins).into_iter().find(|e| e.id == id).ok_or_else(|| "没有找到这个连接器，可能已被卸载".to_string())
}

fn one(app: &tauri::AppHandle, id: &str) -> Result<Value, String> {
    let manager = super::connector_manager(app)?;
    let e = find(&Dirs::of(app)?.plugins(), id)?;
    Ok(entry_json(&manager, &e, &manager.statuses(), false))
}

fn installed_suggestion(item: Value) -> Value {
    json!({"reference":format!("installed:{}", item["id"].as_str().unwrap_or("")), "name":item["name"], "description":item["description"], "installed":item})
}
fn market_suggestion(item: &Value) -> Option<Value> {
    let (namespace, slug, version) = (item["namespace"].as_str()?, item["slug"].as_str()?, item["version"].as_str()?);
    if version.is_empty() || !item["connector"].is_object() { return None; }
    Some(json!({"reference":format!("market:{namespace}/{slug}"), "name":item["name"], "description":item["description"], "namespace":namespace, "slug":slug, "version":version, "connector":item["connector"]}))
}
pub(super) fn search(app: &tauri::AppHandle, keyword: &str) -> Result<Value, String> {
    let keyword: String = keyword.chars().take(80).collect();
    let manager = super::connector_manager(app)?;
    let statuses = manager.statuses();
    let list = entries(&Dirs::of(app)?.plugins());
    let mut results: Vec<Value> = list.iter().filter_map(|e| {
        let item = entry_json(&manager, e, &statuses, false);
        format!("{} {} {}", item["name"], item["nameEn"], item["description"]).to_lowercase().contains(&keyword.to_lowercase()).then(|| installed_suggestion(item))
    }).take(3).collect();
    let mut warning = None;
    if results.len() < 3 && !keyword.is_empty() {
        match super::store_market::market()?.browse("connectors", &keyword, 1, "", "", "", "") {
            Ok(page) => for item in page["results"].as_array().into_iter().flatten() {
                if list.iter().any(|e| Some(e.plugin.name.as_str()) == item["slug"].as_str() && e.namespace.as_deref() == item["namespace"].as_str()) { continue; }
                if let Some(item) = market_suggestion(item) { results.push(item); }
                if results.len() == 3 { break; }
            },
            Err(error) => warning = Some(format!("市场暂不可用：{error}")),
        }
    }
    Ok(json!({"results":results,"warning":warning}))
}
pub(super) fn request(app: &tauri::AppHandle, references: &[String], ctx: &ToolCtx) -> Result<Value, String> {
    let manager = super::connector_manager(app)?;
    let dirs = Dirs::of(app)?;
    let mut candidates = Vec::new();
    for reference in references {
        if candidates.iter().any(|c: &Value| c["reference"] == *reference) { continue; }
        if let Some(id) = reference.strip_prefix("installed:") {
            candidates.push(installed_suggestion(one(app, id)?));
        } else if let Some(path) = reference.strip_prefix("market:") {
            let (namespace, slug) = path.split_once('/').ok_or("市场连接器引用无效")?;
            let detail = super::store_market::market()?.detail("connectors", namespace, slug)?;
            let candidate = market_suggestion(&detail["item"]).ok_or("此连接器暂未提供可安装版本")?;
            if candidate["reference"] != *reference { return Err("连接器来源不匹配".into()); }
            candidates.push(candidate);
        } else { return Err("请使用 connector_search 返回的 reference".into()); }
    }
    if ctx.is_cancelled() { return Err("连接请求已取消".into()); }
    let state = app.state::<ChatState>();
    let broker = &state.broker;
    let (id, rx) = broker.open_question_payload(ctx.session(), json!({"question":"连接服务后继续当前任务", "options":[], "connectors":candidates}));
    emit(app, "chat-pending-changed", json!({"session":ctx.session(), "pending":broker.snapshot(ctx.session())}));
    let answer = ctx.wait_for(&rx).flatten();
    broker.close_question(&id);
    emit(app, "chat-pending-changed", json!({"session":ctx.session(), "pending":broker.snapshot(ctx.session())}));
    let Some(answer) = answer else { return Ok(json!({"status":"skipped"})); };
    let candidate = candidates.iter().find(|c| c["reference"] == answer).ok_or("连接器选择无效")?;
    let statuses = manager.statuses();
    let connected: Vec<Value> = entries(&dirs.plugins()).iter().filter(|e| {
        if candidate["installed"].is_object() {
            candidate["installed"]["id"] == e.id && candidate["installed"]["namespace"].as_str() == e.namespace.as_deref()
        } else {
            candidate["slug"] == e.plugin.name && candidate["namespace"].as_str() == e.namespace.as_deref()
        }
    }).map(|e| entry_json(&manager, e, &statuses, false)).filter(|c| c["status"] == "connected").collect();
    if connected.is_empty() { return Err("连接尚未成功，请完成连接后再继续".into()); }
    let ids = connected.iter().filter_map(|c| c["id"].as_str().map(String::from)).collect::<Vec<_>>();
    let tools: Vec<Value> = manager.tools_for(&ids).into_iter()
        .map(|t| json!({"name":t.spec().name,"description":t.spec().description,"schema":t.spec().schema})).collect();
    Ok(json!({"status":"connected", "connectors":connected, "tools":tools}))
}

#[tauri::command]
pub async fn ai_connector_installed(app: tauri::AppHandle, window: tauri::WebviewWindow) -> Result<Value, String> {
    super::store::control(&window)?;
    tauri::async_runtime::spawn_blocking(move || {
        let manager = super::connector_manager(&app)?;
        let list = entries(&Dirs::of(&app)?.plugins());
        // 恢复在后台做；第一次列出时界面显示“连接中”，之后读到的是真实状态。
        let restoring = !list.is_empty() && !RESTORE_STARTED.swap(true, Ordering::SeqCst);
        if restoring {
            let manager = manager.clone();
            std::thread::spawn(move || {
                if let Err(error) = manager.restore(Arc::new(AtomicBool::new(false))) {
                    eprintln!("部分连接器暂不可用：{error}");
                }
            });
        }
        let statuses = manager.statuses();
        Ok(Value::Array(list.iter().map(|e| entry_json(&manager, e, &statuses, restoring)).collect()))
    })
    .await
    .map_err(|e| e.to_string())?
}

/// 保存令牌并连接；自动发现 OAuth 时直接返回浏览器授权流程。
#[tauri::command]
pub async fn ai_connector_configure(app: tauri::AppHandle, window: tauri::WebviewWindow, id: String, values: BTreeMap<String, String>, confirmed: bool, prefer_browser: Option<bool>) -> Result<Value, String> {
    super::store::control(&window)?;
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = super::store::MUTATIONS.lock().map_err(|e| e.to_string())?;
        let manager = super::connector_manager(&app)?;
        let e = find(&Dirs::of(&app)?.plugins(), &id)?;
        let values: BTreeMap<String, String> = values.into_iter().filter(|(_, v)| !v.is_empty()).collect();
        let result = manager.begin_connect_plugin_with_auth(&e.plugin, &e.server, &values, confirmed, prefer_browser.unwrap_or(values.is_empty()))?;
        if result["status"] == "authorization_required" {
            Ok(result)
        } else {
            one(&app, &id)
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn ai_connector_set_enabled(app: tauri::AppHandle, window: tauri::WebviewWindow, id: String, enabled: bool) -> Result<Value, String> {
    super::store::control(&window)?;
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = super::store::MUTATIONS.lock().map_err(|e| e.to_string())?;
        find(&Dirs::of(&app)?.plugins(), &id)?;
        super::connector_manager(&app)?.set_plugin_enabled(&id, enabled)?;
        one(&app, &id)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// 断开并清除保存的凭据（令牌、覆盖项、授权会话）；插件本身仍然装着。
#[tauri::command]
pub async fn ai_connector_unbind(app: tauri::AppHandle, window: tauri::WebviewWindow, id: String) -> Result<(), String> {
    super::store::control(&window)?;
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = super::store::MUTATIONS.lock().map_err(|e| e.to_string())?;
        super::connector_manager(&app)?.unbind_plugin(&id)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// 重新授权或首次连接不需要令牌的连接器：已能连上就返回已连接，否则返回要在浏览器里打开的授权页和 `flowId`。
#[tauri::command]
pub async fn ai_connector_reauthorize(app: tauri::AppHandle, window: tauri::WebviewWindow, id: String) -> Result<Value, String> {
    super::store::control(&window)?;
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = super::store::MUTATIONS.lock().map_err(|e| e.to_string())?;
        let e = find(&Dirs::of(&app)?.plugins(), &id)?;
        super::connector_manager(&app)?.begin_connect_plugin(&e.plugin, &e.server, &BTreeMap::new(), true)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// 卸载连接器插件之前调用：断开它的每项服务并清除凭据，不然保险箱里会留下没人管的令牌。
pub fn forget_plugin(app: &tauri::AppHandle, name: &str) {
    let Ok(dirs) = Dirs::of(app) else { return };
    let Ok(manager) = super::connector_manager(app) else { return };
    for e in entries(&dirs.plugins()).into_iter().filter(|e| e.plugin.name == name) {
        if let Err(error) = manager.unbind_plugin(&e.id) {
            eprintln!("清除连接器 {name} 的凭据失败：{error}");
        }
    }
}

/// 输入框 `@` 菜单：已安装的连接器、专家和技能。
#[tauri::command]
pub async fn ai_mentions(app: tauri::AppHandle, window: tauri::WebviewWindow) -> Result<Value, String> {
    super::store::control(&window)?;
    tauri::async_runtime::spawn_blocking(move || {
        let dirs = Dirs::of(&app)?;
        let manager = super::connector_manager(&app)?;
        let statuses = manager.statuses();
        let mut items: Vec<Value> = Vec::new();
        let plugin_entries = entries(&dirs.plugins());
        for e in &plugin_entries {
            let v = entry_json(&manager, e, &statuses, false);
            items.push(json!({"kind": "connector", "id": e.id, "name": v["name"], "description": v["description"], "iconUrl": v["iconUrl"]}));
        }
        // 老办法配置的 MCP 连接也能被点名。
        for s in statuses.as_array().into_iter().flatten() {
            if s["id"].as_str().is_some_and(|id| !plugin_entries.iter().any(|e| e.id == id)) {
                items.push(json!({"kind": "connector", "id": s["id"], "name": s["name"], "description": "", "iconUrl": null}));
            }
        }
        for e in super::plugins::installed_experts(&dirs.plugins()).into_iter().chain(super::store::market_workflow_experts(&dirs.skills())) {
            let profession = e["profession"].as_str().unwrap_or("");
            let description = e["description"].as_str().unwrap_or("");
            items.push(json!({"kind": "expert", "id": e["slug"], "name": e["name"], "description": if profession.is_empty() { description.to_string() } else { format!("{profession} · {description}") }, "iconUrl": null}));
        }
        let skills = super::skills_for(&dirs, dirs.chat.join("bundled-skills"));
        for s in super::skill_invocation::menu(&skills, &dirs.plugins(), &dirs.skills()) {
            items.push(json!({"kind": "skill", "id": s["name"], "name": s["displayName"], "description": s["description"], "iconUrl": null}));
        }
        Ok(Value::Array(items))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    fn install(root: &Path, name: &str, mcp: Value, token: bool) {
        use std::io::Write;
        let mut manifest = json!({"name": name, "version": "1.2.0", "description": "d", "extensions": {"ai.workbuddy": {
            "display": {"name": "飞书", "name_en": "Feishu", "name_zh": "飞书", "description_zh": "读写飞书文档", "description_en": "Feishu docs", "examples_zh": ["总结本周文档"]}}}});
        if token {
            manifest["extensions"]["ai.workbuddy"]["setup"] = json!({"mcpServers": {"feishu": {"auth": {"type": "token", "schema": "./ai.workbuddy/token-schema.json", "headers": {"Authorization": {"field": "FEISHU_TOKEN", "prefix": "Bearer "}}}}}});
        }
        let schema = json!({"title": "配置飞书令牌", "fields": [{"key": "FEISHU_TOKEN", "label": "令牌", "type": "password", "required": true}]});
        let mut w = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        let mut files = vec![(".codebuddy-plugin/plugin.json", manifest.to_string()), ("mcp.json", mcp.to_string()), ("skills/feishu-usage/SKILL.md", "---
name: feishu-usage
description: d
---
steps".to_string())];
        if token {
            files.push(("ai.workbuddy/token-schema.json", schema.to_string()));
        }
        for (path, data) in files {
            w.start_file(path, zip::write::SimpleFileOptions::default()).unwrap();
            w.write_all(data.as_bytes()).unwrap();
        }
        let pkg = plugin::inspect_zip(&w.finish().unwrap().into_inner()).unwrap();
        plugin::install(&root.join("plugins"), &root.join("backups"), &pkg, "xiaoban-market", json!({"service": "xiaoban-market"})).unwrap();
    }

    #[test]
    fn an_installed_connector_is_listed_with_its_form_but_never_with_a_secret() {
        let root = std::env::temp_dir().join(format!("connectors-list-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        install(&root, "feishu", json!({"mcpServers": {"feishu": {"type": "streamable-http", "url": "https://mcp.example.com/feishu"}}}), true);
        install(&root, "legacy", json!({"mcpServers": {"old": {"type": "sse", "url": "https://mcp.example.com/sse"}}}), false);
        let manager = agent_connector_tools::ConnectorManager::new_lazy(root.join("connectors"), root.clone()).unwrap();
        let list = entries(&root.join("plugins"));
        assert_eq!(list.len(), 2);
        let statuses = manager.statuses();
        let feishu = list.iter().find(|e| e.plugin.name == "feishu").map(|e| entry_json(&manager, e, &statuses, false)).unwrap();
        assert_eq!((feishu["name"].as_str(), feishu["source"].as_str(), feishu["status"].as_str(), feishu["enabled"].as_bool()), (Some("飞书"), Some("market"), Some("disconnected"), Some(false)));
        assert_eq!(feishu["auth"]["type"], "token");
        assert_eq!(feishu["auth"]["schema"]["fields"][0]["key"], "FEISHU_TOKEN");
        assert_eq!(feishu["auth"]["schema"]["fields"][0]["type"], "password");
        assert_eq!(feishu["auth"]["configured"], json!([]));
        assert_eq!((feishu["examples"][0].as_str(), feishu["skills"][0].as_str(), feishu["unsupported"].is_null()), (Some("总结本周文档"), Some("feishu-usage"), true));
        let legacy = list.iter().find(|e| e.plugin.name == "legacy").map(|e| entry_json(&manager, e, &statuses, false)).unwrap();
        assert_eq!(legacy["auth"]["type"], "auto");
        assert_eq!(legacy["transport"], "sse");
        assert!(legacy["unsupported"].is_null(), "legacy SSE servers connect now: {legacy}");
        assert!(find(&root.join("plugins"), &list[0].id).is_ok() && find(&root.join("plugins"), "plugin-nope").is_err());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn market_suggestions_require_an_installable_version_and_keep_the_source_identity() {
        let item = json!({"namespace":"acme", "slug":"docs", "version":"1.0", "name":"文档", "description":"查找文档", "connector":{}, "metadata":{"internal":"not-for-the-card"}});
        let suggestion = market_suggestion(&item).unwrap();
        assert_eq!(suggestion["reference"], "market:acme/docs");
        assert_eq!(suggestion["namespace"], "acme");
        assert!(suggestion.get("metadata").is_none());
        let mut missing_version = item.clone(); missing_version["version"] = json!("");
        assert!(market_suggestion(&missing_version).is_none());
        let mut legacy_template = item; legacy_template["connector"] = Value::Null;
        assert!(market_suggestion(&legacy_template).is_none());
    }

    #[test]
    fn the_state_shown_comes_from_the_manager_and_never_guesses_a_connection() {
        let status = |v: Value| status_of(Some(&v), false);
        assert_eq!(status_of(None, true), ("disconnected", false, None), "not configured yet");
        assert_eq!(status(json!({"enabled": false, "connected": false})), ("disabled", false, None));
        assert_eq!(status(json!({"enabled": true, "connected": true})), ("connected", true, None));
        assert_eq!(status(json!({"enabled": true, "connected": false, "error": "MCP HTTP 401"})).0, "needs_auth");
        assert_eq!(status(json!({"enabled": true, "connected": false, "error": "连接超时"})), ("error", true, Some("连接超时".into())));
        assert_eq!(status(json!({"enabled": true, "connected": false})).0, "disconnected");
        assert_eq!(status_of(Some(&json!({"enabled": true, "connected": false})), true).0, "connecting");
    }
}
