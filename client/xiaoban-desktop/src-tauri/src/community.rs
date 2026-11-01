//! Community packs. A `xiaoban://community/apply/...` link (or the community page) asks the console to show a pack.
//! The pack is downloaded, hash-checked and described first; nothing is applied until the user confirms in the console.
//! Download, hash and archive checks live in the `xiaoban-community` crate; this file only stages the result and applies it.
use serde_json::{json, Value};
use std::sync::{Mutex, Arc, atomic::{AtomicU64, Ordering}};
use tauri::{Emitter, Manager, WebviewUrl, webview::WebviewBuilder, window::WindowBuilder};
use xiaoban_community as community;

const DEFAULT_ORIGIN: &str = "https://chat.xiangban.xyz";

pub(crate) fn origin() -> String {
    // Debug builds may point at a local forum while developing; release builds cannot be redirected.
    #[cfg(debug_assertions)]
    if let Ok(url) = std::env::var("XIAOBAN_COMMUNITY_URL") { return url; }
    DEFAULT_ORIGIN.into()
}

const COMMUNITY_WINDOW: &str = "community-window";
const COMMUNITY_CHROME: &str = "community-chrome";
const COMMUNITY_CONTENT: &str = "community-web";
const TITLE_HEIGHT: f64 = 44.;
const EDGE: f64 = 5.;
static COMMUNITY_OPEN_LOCK: Mutex<()> = Mutex::new(());

/// Production is fixed; development may only use a loopback web server.
fn validate_web_origin(configured: &str, debug: bool) -> Result<url::Url, String> {
    let parsed = url::Url::parse(configured).map_err(|_| "社区地址无效")?;
    let local = matches!(parsed.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"));
    if !(parsed.origin() == url::Url::parse(DEFAULT_ORIGIN).expect("fixed origin").origin() || (debug && local && matches!(parsed.scheme(), "http" | "https")))
        || !parsed.username().is_empty() || parsed.password().is_some()
        || parsed.path() != "/" || parsed.query().is_some() || parsed.fragment().is_some() {
        return Err("社区地址必须是官方地址或调试时的本机地址".into());
    }
    Ok(parsed)
}

fn web_origin() -> Result<url::Url, String> {
    validate_web_origin(&if cfg!(debug_assertions) { origin() } else { DEFAULT_ORIGIN.into() }, cfg!(debug_assertions))
}

fn same_origin(url: &url::Url, origin: &url::Url) -> bool {
    url.origin() == origin.origin() && url.username().is_empty() && url.password().is_none()
}

/// The remote site has no native permissions and does not receive the desktop account token.
#[tauri::command]
pub async fn community_web_open(app: tauri::AppHandle, caller: tauri::WebviewWindow) -> Result<(), String> {
    control(&caller)?;
    let _guard = COMMUNITY_OPEN_LOCK.lock().map_err(|_| "社区窗口暂不可用")?;
    if let Some(existing) = app.get_window(COMMUNITY_WINDOW) {
        existing.unminimize().map_err(|e| e.to_string())?;
        if let Some(monitor) = existing.current_monitor().map_err(|e| e.to_string())?.or(caller.current_monitor().map_err(|e| e.to_string())?) {
            let area = monitor.work_area(); let factor = monitor.scale_factor();
            let size = existing.outer_size().map_err(|e| e.to_string())?;
            let position = existing.outer_position().map_err(|e| e.to_string())?;
            let width = size.width.min(area.size.width).max(1); let height = size.height.min(area.size.height).max(1);
            existing.set_min_size(Some(tauri::LogicalSize::new(360f64.min(area.size.width as f64 / factor), 480f64.min(area.size.height as f64 / factor)))).map_err(|e| e.to_string())?;
            if size.width != width || size.height != height { existing.set_size(tauri::PhysicalSize::new(width, height)).map_err(|e| e.to_string())?; }
            let right = i64::from(area.position.x) + i64::from(area.size.width) - i64::from(width);
            let bottom = i64::from(area.position.y) + i64::from(area.size.height) - i64::from(height);
            existing.set_position(tauri::PhysicalPosition::new(i64::from(position.x).clamp(i64::from(area.position.x), right) as i32, i64::from(position.y).clamp(i64::from(area.position.y), bottom) as i32)).map_err(|e| e.to_string())?;
        }
        existing.show().map_err(|e| e.to_string())?;
        return existing.set_focus().map_err(|e| e.to_string());
    }
    let origin = web_origin()?;
    let nav_origin = origin.clone();
    let nav_app = app.clone();
    let new_origin = origin.clone();
    let new_app = app.clone();
    let monitor = caller.current_monitor().map_err(|e| e.to_string())?.or(app.primary_monitor().map_err(|e| e.to_string())?);
    let (width, height) = monitor.as_ref().map(|m| {
        let area = m.work_area(); let scale = m.scale_factor();
        (1100f64.min((area.size.width as f64 / scale - 32.).max(1.)), 760f64.min((area.size.height as f64 / scale - 64.).max(1.)))
    }).unwrap_or((1100., 760.));
    let mut builder = WindowBuilder::new(&app, COMMUNITY_WINDOW).title("小伴社区")
        .inner_size(width, height).min_inner_size(360f64.min(width), 480f64.min(height))
        .decorations(false).resizable(true).visible(false);
    if let Some(m) = &monitor {
        let a = m.work_area(); let f = m.scale_factor();
        builder = builder.position(a.position.x as f64 / f + (a.size.width as f64 / f - width) / 2., a.position.y as f64 / f + (a.size.height as f64 / f - height) / 2.);
    } else { builder = builder.center(); }
    let window = builder.build().map_err(|e| e.to_string())?;
    let result = (|| -> Result<(), String> {
        window.add_child(WebviewBuilder::new(COMMUNITY_CHROME, WebviewUrl::App("community-chrome.html".into())).auto_resize(),
            tauri::LogicalPosition::new(0., 0.), tauri::LogicalSize::new(width, height)).map_err(|e| e.to_string())?;
        let content = WebviewBuilder::new(COMMUNITY_CONTENT, WebviewUrl::External(origin))
            .on_navigation(move |url| {
                if same_origin(url, &nav_origin) { return true; }
                if url.scheme() == "xiaoban" && url.host_str() == Some("community") {
                    let app = nav_app.clone(); let link = url.to_string();
                    std::thread::spawn(move || receive_link(&app, &link));
                } else if matches!(url.scheme(), "http" | "https") {
                    let link = url.to_string();
                    std::thread::spawn(move || { let _ = crate::content::open_download_url(link); });
                }
                false
            })
            .on_new_window(move |url, _| {
                let app = new_app.clone();
                if same_origin(&url, &new_origin) {
                    std::thread::spawn(move || { if let Some(view) = app.get_webview(COMMUNITY_CONTENT) { let _ = view.navigate(url); } });
                } else if url.scheme() == "xiaoban" && url.host_str() == Some("community") {
                    std::thread::spawn(move || receive_link(&app, url.as_str()));
                } else if matches!(url.scheme(), "http" | "https") {
                    std::thread::spawn(move || { let _ = crate::content::open_download_url(url.to_string()); });
                }
                tauri::webview::NewWindowResponse::Deny
            });
        let content = window.add_child(content, tauri::LogicalPosition::new(EDGE, TITLE_HEIGHT),
            tauri::LogicalSize::new((width - EDGE * 2.).max(1.), (height - TITLE_HEIGHT - EDGE).max(1.))).map_err(|e| e.to_string())?;
        // Tauri's auto_resize scales offsets proportionally; the titlebar and resize edges
        // must remain fixed logical pixels, including when a window changes DPI.
        let scale = Arc::new(AtomicU64::new(window.scale_factor().map_err(|e| e.to_string())?.to_bits()));
        window.on_window_event(move |event| {
            let size = match event {
                tauri::WindowEvent::Resized(size) => Some(*size),
                tauri::WindowEvent::ScaleFactorChanged { scale_factor, new_inner_size, .. } => {
                    scale.store(scale_factor.to_bits(), Ordering::Relaxed); Some(*new_inner_size)
                }
                _ => None,
            };
            if let Some(size) = size {
                let factor = f64::from_bits(scale.load(Ordering::Relaxed));
                let logical = size.to_logical::<f64>(factor);
                let _ = content.set_position(tauri::LogicalPosition::new(EDGE, TITLE_HEIGHT));
                let _ = content.set_size(tauri::LogicalSize::new((logical.width - 2. * EDGE).max(1.), (logical.height - TITLE_HEIGHT - EDGE).max(1.)));
            }
        });
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())
    })();
    if result.is_err() { let _ = window.close(); }
    result
}

pub fn invocation_permission(label: &str, command: &str) -> Option<bool> {
    match label {
        COMMUNITY_CONTENT => Some(false),
        COMMUNITY_CHROME => Some(command == "community_chrome_action"),
        _ => None,
    }
}

fn chrome_action_allowed(view: &str, window: &str) -> bool { view == COMMUNITY_CHROME && window == COMMUNITY_WINDOW }

/// Only the bundled chrome can operate its own parent window.
#[tauri::command]
pub fn community_chrome_action(view: tauri::Webview, window: tauri::Window, action: String) -> Result<(), String> {
    if !chrome_action_allowed(view.label(), window.label()) { return Err("无权操作社区窗口".into()); }
    match action.as_str() {
        "close" => window.close(),
        "minimize" => window.minimize(),
        "maximize" => if window.is_maximized().map_err(|e| e.to_string())? { window.unmaximize() } else { window.maximize() },
        "fullscreen" => window.set_fullscreen(!window.is_fullscreen().map_err(|e| e.to_string())?),
        "drag" => window.start_dragging(),
        "n" | "s" | "e" | "w" | "ne" | "nw" | "se" | "sw" => {
            let edge = match action.as_str() { "n" => "North", "s" => "South", "e" => "East", "w" => "West", "ne" => "NorthEast", "nw" => "NorthWest", "se" => "SouthEast", _ => "SouthWest" };
            window.start_resize_dragging(serde_json::from_value(json!(edge)).map_err(|e| e.to_string())?)
        }
        _ => return Err("未知窗口操作".into()),
    }.map_err(|e| e.to_string())
}

/// What the console is waiting to show: a link that arrived, and the one pack that was checked and is waiting for a yes.
#[derive(Default)]
pub struct CommunityState {
    link: Mutex<Option<Value>>,
    staged: Mutex<Option<Staged>>,
}
struct Staged { token: String, pack_id: i64, version: i32, kind: String, payload: Value, can_apply: bool }

/// Anonymous usage counts (PRD §8.3): a random device id that is not tied to any account, a fixed list of event names,
/// and at most a pack kind. No content, file names or search words are ever sent. Off with one switch; failures are ignored.
const DESKTOP_EVENTS: [&str; 6] = [
    "desktop_community_open", "desktop_apply_open", "desktop_apply_confirm", "desktop_apply_success", "desktop_apply_cancel", "desktop_scheme_replicate",
];

fn analytics_dir(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> { Ok(crate::storage::root(app)?.join("config")) }

fn analytics_enabled(app: &tauri::AppHandle) -> bool {
    analytics_dir(app).is_ok_and(|dir| !dir.join("community-analytics-off").exists())
}

/// The device id lives in one small file; turning statistics off deletes it, so turning them on again starts a new anonymous device.
fn device_id(app: &tauri::AppHandle) -> Option<String> {
    let path = analytics_dir(app).ok()?.join("community-device");
    if let Ok(text) = std::fs::read_to_string(&path) {
        let id = text.trim().to_string();
        if (16..=64).contains(&id.len()) && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-') { return Some(id); }
    }
    let mut random = [0u8; 16];
    getrandom::fill(&mut random).ok()?;
    let id: String = random.iter().map(|b| format!("{b:02x}")).collect();
    std::fs::create_dir_all(path.parent()?).ok()?;
    std::fs::write(&path, &id).ok()?;
    Some(id)
}

pub fn track(app: &tauri::AppHandle, name: &'static str, dim: Option<String>) {
    if !DESKTOP_EVENTS.contains(&name) || !analytics_enabled(app) { return; }
    let Some(device) = device_id(app) else { return };
    std::thread::spawn(move || {
        if let Ok(client) = community::Community::new(&origin()) { client.track(&device, &[(name, dim.as_deref())]); }
    });
}

#[tauri::command]
pub fn community_analytics_get(app: tauri::AppHandle, window: tauri::WebviewWindow) -> Result<bool, String> {
    control(&window)?;
    Ok(analytics_enabled(&app))
}

#[tauri::command]
pub fn community_analytics_set(app: tauri::AppHandle, window: tauri::WebviewWindow, on: bool) -> Result<(), String> {
    control(&window)?;
    let dir = analytics_dir(&app)?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let off = dir.join("community-analytics-off");
    if on { let _ = std::fs::remove_file(&off); } else {
        std::fs::write(&off, b"off").map_err(|e| e.to_string())?;
        let _ = std::fs::remove_file(dir.join("community-device"));
    }
    Ok(())
}

/// Events the console itself notices (the page opened, a confirmation was cancelled, a scheme was replicated).
#[tauri::command]
pub fn community_track(app: tauri::AppHandle, window: tauri::WebviewWindow, name: String, dim: Option<String>) -> Result<(), String> {
    control(&window)?;
    if let Some(known) = DESKTOP_EVENTS.iter().find(|e| **e == name) { track(&app, *known, dim); }
    Ok(())
}

fn control(window: &tauri::WebviewWindow) -> Result<(), String> {
    if window.label() != "control" { return Err("只有设置面板可以使用社区内容".into()); }
    Ok(())
}

/// A link handed to the app. A closed app is started by it, so the console asks for it when it starts and again
/// whenever it is told a new one arrived.
pub fn receive_link(app: &tauri::AppHandle, text: &str) {
    let Ok(link) = community::parse_link(text) else { return };
    *app.state::<CommunityState>().link.lock().unwrap_or_else(|e| e.into_inner()) = Some(link.to_json());
    let _ = crate::present_control(app);
    let _ = app.emit("community-link", ());
    // When the link started the app, the console window may not exist yet; show it once it does.
    let handle = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(1500));
        let _ = crate::present_control(&handle);
    });
}

#[tauri::command]
pub fn community_take_link(app: tauri::AppHandle, window: tauri::WebviewWindow) -> Result<Option<Value>, String> {
    control(&window)?;
    Ok(app.state::<CommunityState>().link.lock().unwrap_or_else(|e| e.into_inner()).take())
}

/// Read-only browsing for the community page: weekly issues and pack lists, passed through from the forum.
#[tauri::command]
pub async fn community_browse(window: tauri::WebviewWindow, path: String) -> Result<Value, String> {
    control(&window)?;
    tauri::async_runtime::spawn_blocking(move || community::Community::new(&origin())?.browse(&path))
        .await.map_err(|e| e.to_string())?
}

/// Downloads a pack, checks it, and describes what applying it would do. Nothing is changed on this machine.
#[tauri::command]
pub async fn community_prepare(
    app: tauri::AppHandle, window: tauri::WebviewWindow, pack_id: i64, version: Option<i32>, sha256: Option<String>,
) -> Result<Value, String> {
    control(&window)?;
    tauri::async_runtime::spawn_blocking(move || {
        let client = community::Community::new(&origin())?;
        let info = client.download_info(pack_id, version, sha256.as_deref())?;
        let bytes = client.fetch(&info)?;
        let found = community::inspect(&bytes, &info.kind)?;
        let payload = found.payload.clone().ok_or("这个内容包没有可应用的内容")?;
        // What the user will be asked to confirm: for rules the whole text, for an effect its own checks.
        let (items, can_apply, payload) = match info.kind.as_str() {
            "effect" => {
                crate::effect_packs::validate(&payload)?;
                (vec![json!({ "title": payload["name"], "detail": payload["description"] })], info.one_click, payload)
            }
            "reminder" | "routine" => {
                let rules = community::rules_for(&info.kind, &payload)?;
                let items: Vec<Value> = rules.iter().map(|rule| json!({
                    "title": rule["kind"], "detail": rule["text"], "schedule": { "repeat": rule["repeat"], "time": rule["time"], "weekdays": rule["weekdays"], "intervalMinutes": rule["intervalMinutes"] },
                })).collect();
                (items, info.one_click, payload)
            }
            "task" => {
                let task = community::task_template(&payload)?;
                let mut detail = format!("将发给助手的提示词：\n{}", task["prompt"].as_str().unwrap_or_default());
                for (label, key) in [("适用场景", "scenario"), ("预期产出", "expectedOutput"), ("风险说明", "risk")] {
                    if let Some(text) = task[key].as_str().filter(|t| !t.is_empty()) { detail.push_str(&format!("\n{label}：{text}")); }
                }
                if let Some(skills) = task["requiredSkills"].as_array().filter(|l| !l.is_empty()) {
                    detail.push_str(&format!("\n需要的技能：{}", skills.iter().filter_map(Value::as_str).collect::<Vec<_>>().join("、")));
                }
                (vec![json!({ "title": task["title"], "detail": detail })], info.one_click, task)
            }
            "goal" => {
                let goal = community::goal_template(&payload)?;
                let mut detail = String::new();
                for (label, key) in [("结果", "outcome"), ("验收方式", "verification"), ("约束", "constraints"), ("边界", "boundaries"), ("停止条件", "stopWhen")] {
                    if let Some(text) = goal[key].as_str().filter(|t| !t.is_empty()) { detail.push_str(&format!("{label}：{text}\n")); }
                }
                (vec![json!({ "title": "长期目标", "detail": detail.trim_end() })], info.one_click, goal)
            }
            other => return Err(format!("桌面端暂时还不能应用「{other}」类型的内容包，请先在网页上查看")),
        };
        let token = format!("{:x}{:x}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos());
        *app.state::<CommunityState>().staged.lock().unwrap_or_else(|e| e.into_inner()) = Some(Staged {
            token: token.clone(), pack_id: info.pack_id, version: info.version, kind: info.kind.clone(), payload, can_apply,
        });
        track(&app, "desktop_apply_open", Some(info.kind.clone()));
        Ok(json!({
            "token": token, "packId": info.pack_id, "kind": info.kind, "title": found.manifest["name"],
            "version": info.label, "trustLevel": info.trust_level, "canApply": can_apply, "size": info.size,
            "sha256": info.sha256, "items": items,
            // Routines run for the user while they are away, so they arrive switched off and the user turns them on after reading them.
            "startsDisabled": info.kind == "routine",
        }))
    }).await.map_err(|e| e.to_string())?
}

/// Applies the pack that `community_prepare` staged, after the user confirmed it in the console.
#[tauri::command]
pub async fn community_apply(app: tauri::AppHandle, window: tauri::WebviewWindow, token: String) -> Result<Value, String> {
    control(&window)?;
    let staged = {
        let state = app.state::<CommunityState>();
        let mut slot = state.staged.lock().unwrap_or_else(|e| e.into_inner());
        match slot.take() {
            Some(staged) if staged.token == token => staged,
            other => { *slot = other; return Err("这次确认已经失效，请重新打开这个内容".into()); }
        }
    };
    if !staged.can_apply { return Err("这个内容还没有通过审核，不能一键应用".into()); }
    track(&app, "desktop_apply_confirm", Some(staged.kind.clone()));
    tauri::async_runtime::spawn_blocking(move || {
        // `status` tells the console what to do next: "applied" is finished; "task" and "goal" hand the template to the
        // console, which puts it in front of the user to start themselves (nothing is sent or created here).
        let result = match staged.kind.as_str() {
            "effect" => {
                let file = crate::effects::save_pack(&app, &staged.payload, false)?;
                json!({ "status": "applied", "message": format!("已添加特效（{file}），到装扮里选用它") })
            }
            "reminder" => {
                let rules = community::rules_for("reminder", &staged.payload)?;
                json!({ "status": "applied", "message": format!("已添加 {} 条提醒", crate::reminders::import_rules(&app, rules, true)?) })
            }
            "routine" => {
                let rules = community::rules_for("routine", &staged.payload)?;
                json!({ "status": "applied", "message": format!("已添加 {} 个例行事项，默认未启用，到提醒页确认后打开", crate::reminders::import_rules(&app, rules, false)?) })
            }
            "task" => json!({ "status": "task", "message": "已放进新对话的输入框，确认后再发送", "task": staged.payload }),
            "goal" => json!({ "status": "goal", "message": "已填入长期目标表单，确认后再创建", "goal": staged.payload }),
            other => return Err(format!("不支持应用「{other}」类型的内容包")),
        };
        if let Ok(client) = community::Community::new(&origin()) { client.report_applied(staged.pack_id, staged.version); }
        track(&app, "desktop_apply_success", Some(staged.kind.clone()));
        Ok(result)
    }).await.map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_community_deep_links() {
        let sha = "a".repeat(64);
        let link_url = format!("xiaoban://community/apply/effect/7?version=2&sha256={sha}");
        let parsed = community::parse_link(&link_url).unwrap();
        let json = parsed.to_json();
        assert_eq!(json["action"], "apply");
        assert_eq!(json["kind"], "effect");
        assert_eq!(json["packId"], 7);
        assert_eq!(json["version"], 2);
        assert_eq!(json["sha256"], sha);

        let open_url = "xiaoban://community/open/weekly/2026-W44";
        let open_parsed = community::parse_link(open_url).unwrap();
        let open_json = open_parsed.to_json();
        assert_eq!(open_json["action"], "open");
        assert_eq!(open_json["target"], "weekly");
        assert_eq!(open_json["id"], "2026-W44");

        assert!(community::parse_link("xiaoban://other/path").is_err());
        assert!(community::parse_link("https://chat.xiangban.xyz").is_err());
    }

    #[test]
    fn community_state_stages_and_takes_link() {
        let state = CommunityState::default();
        assert!(state.link.lock().unwrap().is_none());

        let sha = "b".repeat(64);
        let link = community::parse_link(&format!("xiaoban://community/apply/reminder/12?version=1&sha256={sha}")).unwrap();
        *state.link.lock().unwrap() = Some(link.to_json());

        let taken = state.link.lock().unwrap().take().unwrap();
        assert_eq!(taken["action"], "apply");
        assert_eq!(taken["kind"], "reminder");
        assert_eq!(taken["packId"], 12);
        assert!(state.link.lock().unwrap().is_none(), "take 之后清空暂存");
    }
}

#[cfg(test)]
mod web_window_tests {
    use super::*;
    #[test]
    fn only_local_chrome_controls_its_parent() {
        assert_eq!(invocation_permission("community-web", "community_chrome_action"), Some(false));
        assert_eq!(invocation_permission("community-web", "get_diagnostics"), Some(false));
        assert_eq!(invocation_permission("community-chrome", "community_chrome_action"), Some(true));
        assert_eq!(invocation_permission("community-chrome", "get_diagnostics"), Some(false));
        assert_eq!(invocation_permission("control", "community_web_open"), None);
        assert!(chrome_action_allowed("community-chrome", "community-window"));
        assert!(!chrome_action_allowed("community-web", "community-window"));
        assert!(!chrome_action_allowed("community-chrome", "control"));
    }
    #[test]
    fn origin_is_fixed_in_release_and_loopback_only_in_debug() {
        assert!(validate_web_origin(DEFAULT_ORIGIN, false).is_ok());
        assert!(validate_web_origin("http://localhost:5173", true).is_ok());
        assert!(validate_web_origin("http://localhost:5173", false).is_err());
        assert!(validate_web_origin("https://evil.example", true).is_err());
        assert!(validate_web_origin("https://chat.xiangban.xyz.evil.example", true).is_err());
        assert!(validate_web_origin("https://user@chat.xiangban.xyz", true).is_err());
        assert!(validate_web_origin("https://chat.xiangban.xyz/path", true).is_err());
        let base = validate_web_origin(DEFAULT_ORIGIN, false).unwrap();
        assert!(same_origin(&url::Url::parse("https://chat.xiangban.xyz/topics/1").unwrap(), &base));
        assert!(!same_origin(&url::Url::parse("https://chat.xiangban.xyz:444/topics/1").unwrap(), &base));
    }
}
