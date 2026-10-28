//! Community packs. A `xiaoban://community/apply/...` link (or the community page) asks the console to show a pack.
//! The pack is downloaded, hash-checked and described first; nothing is applied until the user confirms in the console.
//! Download, hash and archive checks live in the `xiaoban-community` crate; this file only stages the result and applies it.
use serde_json::{json, Value};
use std::sync::Mutex;
use tauri::{Emitter, Manager};
use xiaoban_community as community;

const DEFAULT_ORIGIN: &str = "https://chat.xiangban.xyz";

fn origin() -> String {
    // Debug builds may point at a local forum while developing; release builds cannot be redirected.
    #[cfg(debug_assertions)]
    if let Ok(url) = std::env::var("XIAOBAN_COMMUNITY_URL") { return url; }
    DEFAULT_ORIGIN.into()
}

/// What the console is waiting to show: a link that arrived, and the one pack that was checked and is waiting for a yes.
#[derive(Default)]
pub struct CommunityState {
    link: Mutex<Option<Value>>,
    staged: Mutex<Option<Staged>>,
}
struct Staged { token: String, pack_id: i64, version: i32, kind: String, payload: Value, can_apply: bool }

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
        let (items, can_apply) = match info.kind.as_str() {
            "effect" => {
                crate::effect_packs::validate(&payload)?;
                (vec![json!({ "title": payload["name"], "detail": payload["description"] })], info.one_click)
            }
            "reminder" | "task" => {
                let rules = community::rules_for(&info.kind, &payload)?;
                let items: Vec<Value> = rules.iter().map(|rule| json!({
                    "title": rule["kind"], "detail": rule["text"], "schedule": { "repeat": rule["repeat"], "time": rule["time"], "weekdays": rule["weekdays"], "intervalMinutes": rule["intervalMinutes"] },
                })).collect();
                (items, info.one_click)
            }
            other => return Err(format!("桌面端暂时还不能应用「{other}」类型的内容包，请先在网页上查看")),
        };
        let token = format!("{:x}{:x}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos());
        *app.state::<CommunityState>().staged.lock().unwrap_or_else(|e| e.into_inner()) = Some(Staged {
            token: token.clone(), pack_id: info.pack_id, version: info.version, kind: info.kind.clone(), payload, can_apply,
        });
        Ok(json!({
            "token": token, "packId": info.pack_id, "kind": info.kind, "title": found.manifest["name"],
            "version": info.label, "trustLevel": info.trust_level, "canApply": can_apply, "size": info.size,
            "sha256": info.sha256, "items": items,
            // Tasks act for the user, so they arrive switched off and the user turns them on after reading them.
            "startsDisabled": info.kind == "task",
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
    tauri::async_runtime::spawn_blocking(move || {
        let message = match staged.kind.as_str() {
            "effect" => {
                let file = crate::effects::save_pack(&app, &staged.payload, false)?;
                format!("已添加特效（{file}），到装扮里选用它")
            }
            "reminder" => {
                let rules = community::rules_for("reminder", &staged.payload)?;
                format!("已添加 {} 条提醒", crate::reminders::import_rules(&app, rules, true)?)
            }
            "task" => {
                let rules = community::rules_for("task", &staged.payload)?;
                format!("已添加 {} 个定时任务，默认未启用，到提醒页确认后打开", crate::reminders::import_rules(&app, rules, false)?)
            }
            other => return Err(format!("不支持应用「{other}」类型的内容包")),
        };
        if let Ok(client) = community::Community::new(&origin()) { client.report_applied(staged.pack_id, staged.version); }
        Ok(json!({ "status": "applied", "message": message }))
    }).await.map_err(|e| e.to_string())?
}
