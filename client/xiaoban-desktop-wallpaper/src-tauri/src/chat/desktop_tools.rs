//! Tools that let the assistant run the desktop app itself: wallpapers, effects, the companion
//! and care reminders. They go through the same functions the settings window uses, so every
//! rule those enforce (valid values, locks, saving) applies here too.
//!
//! The tools talk to a `Desktop`, so they can be tested without a running app.

use crate::preferences::{EXPRESSIONS, SKINS};
use crate::reminders::Rule;
use agent_kernel::{Risk, Tool, ToolCtx, ToolSpec};
use serde_json::{json, Value};
use std::sync::Arc;
use tauri::Manager;

#[derive(Default, Clone, Debug, PartialEq)]
pub struct CompanionChange {
    pub expression: Option<String>,
    pub skin: Option<String>,
    pub enabled: Option<bool>,
}

pub trait Desktop: Send + Sync {
    fn status(&self) -> Result<Value, String>;
    /// `{ items: [{ definition: { id, title, kind }, installed, source }], activeId }`
    fn wallpapers(&self) -> Result<Value, String>;
    fn apply_wallpaper(&self, id: &str) -> Result<(), String>;
    fn create_wallpaper(&self, title: &str, description: Option<&str>, html: &str, apply_now: bool) -> Result<Value, String>;
    fn delete_wallpaper(&self, id: &str) -> Result<String, String>;
    /// `{ packs: [{ id, name }] }` plus `selected` for the one in use.
    fn effects(&self) -> Result<Value, String>;
    /// Empty turns the effect off.
    fn set_effect(&self, id: &str) -> Result<(), String>;
    /// Writes a new effect pack the user asked for. Returns the file name it landed in.
    fn create_effect(&self, pack: &Value, overwrite: bool) -> Result<String, String>;
    fn delete_effect(&self, id: &str) -> Result<String, String>;
    fn set_companion(&self, change: &CompanionChange) -> Result<(), String>;
    /// `{ config, rules }`
    fn care(&self) -> Result<Value, String>;
    fn care_upsert(&self, rule: Rule) -> Result<(), String>;
    fn care_delete(&self, id: &str) -> Result<(), String>;
    fn widgets(&self) -> Result<Value, String>;
    fn widget_add(&self, kind: &str) -> Result<Value, String>;
    fn widget_update(&self, id: &str, patch: Value) -> Result<Value, String>;
    fn widget_remove(&self, id: &str) -> Result<(), String>;
    fn custom_widgets(&self) -> Result<Value, String>;
    fn create_custom_widget(&self, pack: &Value, place_now: bool, overwrite: bool) -> Result<Value, String>;
    fn delete_custom_widget(&self, id: &str) -> Result<(), String>;
    fn games(&self) -> Result<Value, String>;
    fn game_open(&self, id: Option<&str>) -> Result<(), String>;
    fn game_close(&self) -> Result<(), String>;
    /// Writes a user game folder from text files. Returns `{ id, name, directory, files, save, replaced }`.
    fn create_game(&self, id: &str, name: &str, description: &str, files: &[(String, String)], save: bool, overwrite: bool) -> Result<Value, String>;
    /// Deletes a user game, closing the game space first when it is open.
    fn delete_game(&self, id: &str) -> Result<(), String>;
    fn request_permissions(&self) -> Result<Value, String>;
}

pub struct TauriDesktop {
    pub app: tauri::AppHandle,
}

impl TauriDesktop {
    fn control(&self) -> Result<tauri::WebviewWindow, String> {
        self.app.get_webview_window("control").ok_or_else(|| "控制台窗口没有打开".to_string())
    }
}

impl Desktop for TauriDesktop {
    fn request_permissions(&self) -> Result<Value, String> {
        #[cfg(target_os = "macos")]
        {
            for (pane, label) in [("Privacy_Accessibility", "辅助功能"), ("Privacy_ScreenCapture", "屏幕录制")] {
                let status = std::process::Command::new("/usr/bin/open")
                    .arg(format!("x-apple.systempreferences:com.apple.preference.security?{pane}"))
                    .status().map_err(|e| format!("无法打开{label}设置：{e}"))?;
                if !status.success() { return Err(format!("无法打开{label}设置，请在系统设置中手动查看")); }
            }
            Ok(json!({
                "ok": true,
                "message": "已依次打开 macOS 辅助功能与屏幕录制设置页面。请用户检查并按需开启小伴的权限；打开设置不代表已经获得授权。"
            }))
        }
        #[cfg(not(target_os = "macos"))]
        {
            Err("此权限设置引导仅适用于 macOS，当前平台未执行任何授权操作。请根据具体工具返回的错误处理当前系统权限。".into())
        }
    }
    fn status(&self) -> Result<Value, String> {
        let s = crate::settings(&self.app);
        Ok(json!({
            "mode": s.mode, "wallpaper": s.wallpaper_id, "effect": s.effects.selected,
            "companionEnabled": s.companion_enabled, "companionSkin": s.companion_skin,
            "expression": s.expression, "animationPaused": s.paused,
        }))
    }

    fn wallpapers(&self) -> Result<Value, String> {
        crate::content::get_wallpaper_library(self.app.clone())
    }

    fn apply_wallpaper(&self, id: &str) -> Result<(), String> {
        let app = self.app.clone();
        tauri::async_runtime::block_on(crate::content::apply_wallpaper(app, id.to_string(), Default::default(), false)).map(|_| ())
    }

    fn create_wallpaper(&self, title: &str, description: Option<&str>, html: &str, apply_now: bool) -> Result<Value, String> {
        let root = crate::content::root(&self.app)?;
        let item = crate::local_media::save_local_html(&root, title, description, html)?;
        if apply_now {
            self.apply_wallpaper(&item.id)?;
        }
        Ok(json!({ "id": item.id, "title": item.title, "applied": apply_now }))
    }

    fn effects(&self) -> Result<Value, String> {
        let mut value = crate::effects::list_desktop_effects(self.control()?)?;
        value["selected"] = json!(crate::settings(&self.app).effects.selected);
        Ok(value)
    }

    fn set_effect(&self, id: &str) -> Result<(), String> {
        let mut choice = serde_json::to_value(&crate::settings(&self.app).effects).map_err(|e| e.to_string())?;
        choice["selected"] = json!(id);
        crate::set_desktop_effects(self.app.clone(), choice).map(|_| ())
    }

    fn create_effect(&self, pack: &Value, overwrite: bool) -> Result<String, String> {
        crate::effects::save_pack(&self.app, pack, overwrite)
    }

    fn set_companion(&self, change: &CompanionChange) -> Result<(), String> {
        let app = &self.app;
        if let Some(enabled) = change.enabled {
            tauri::async_runtime::block_on(crate::set_companion_enabled(app.clone(), enabled))?;
        }
        if let Some(skin) = &change.skin {
            crate::set_companion_skin(app.clone(), skin.clone())?;
        }
        if let Some(expression) = &change.expression {
            let s = crate::settings(app);
            crate::update_settings(app.clone(), expression.clone(), s.size, s.theme, s.paused, s.fps)?;
        }
        Ok(())
    }

    fn care(&self) -> Result<Value, String> {
        crate::reminders::get_care_settings(self.control()?)
    }

    fn care_upsert(&self, rule: Rule) -> Result<(), String> {
        crate::reminders::upsert_care_rule(self.control()?, rule).map(|_| ())
    }

    fn care_delete(&self, id: &str) -> Result<(), String> {
        crate::reminders::delete_care_rule(self.control()?, id.to_string()).map(|_| ())
    }

    fn widgets(&self) -> Result<Value, String> {
        crate::widgets::get_desktop_widgets(self.app.clone())
    }

    fn widget_add(&self, kind: &str) -> Result<Value, String> {
        let app = self.app.clone();
        let kind = kind.to_string();
        tauri::async_runtime::block_on(crate::widgets::add_desktop_widget(app, kind, None))
    }

    fn widget_update(&self, id: &str, patch: Value) -> Result<Value, String> {
        let app = self.app.clone();
        let id = id.to_string();
        let patch_obj: crate::widgets::Patch = serde_json::from_value(patch).map_err(|e| e.to_string())?;
        tauri::async_runtime::block_on(crate::widgets::update_desktop_widget(app, id, patch_obj))
    }

    fn widget_remove(&self, id: &str) -> Result<(), String> {
        let app = self.app.clone();
        let id = id.to_string();
        tauri::async_runtime::block_on(crate::widgets::remove_desktop_widget(app, id)).map(|_| ())
    }

    fn custom_widgets(&self) -> Result<Value, String> {
        let dir = crate::storage::root(&self.app)?.join("custom_widgets");
        Ok(json!(crate::custom_widgets::load_all(&dir)))
    }

    fn create_custom_widget(&self, pack: &Value, place_now: bool, overwrite: bool) -> Result<Value, String> {
        let dir = crate::storage::root(&self.app)?.join("custom_widgets");
        let path = crate::custom_widgets::save(&dir, pack, overwrite)?;
        let id = pack["id"].as_str().unwrap_or_default();
        let name = pack["name"].as_str().unwrap_or_default();
        if place_now {
            let app = self.app.clone();
            let _ = tauri::async_runtime::block_on(crate::widgets::add_desktop_widget(app, "custom".to_string(), Some(id.to_string())))?;
        }
        Ok(json!({ "id": id, "name": name, "file": path.file_name().unwrap_or_default().to_string_lossy(), "placed": place_now }))
    }

    fn delete_wallpaper(&self, id: &str) -> Result<String, String> {
        let root = crate::content::root(&self.app)?;
        let hash = crate::local_media::delete_local_wallpaper(&root, id)?;
        let current_id = crate::settings(&self.app).wallpaper_id;
        if current_id == id {
            self.apply_wallpaper("companion")?;
        }
        Ok(hash)
    }

    fn delete_effect(&self, id: &str) -> Result<String, String> {
        crate::effects::delete_pack(&self.app, id)
    }

    fn delete_custom_widget(&self, id: &str) -> Result<(), String> {
        crate::widgets::delete_custom_pack(&self.app, id)
    }

    fn games(&self) -> Result<Value, String> {
        let catalog = crate::games::catalog(&self.app)?;
        let active = crate::games::is_active(&self.app);
        Ok(json!({ "active": active, "games": catalog.games, "errors": catalog.errors }))
    }

    fn create_game(&self, id: &str, name: &str, description: &str, files: &[(String, String)], save: bool, overwrite: bool) -> Result<Value, String> {
        let root = crate::storage::root(&self.app)?;
        crate::game_packs::create_pack(&root, id, name, description, files, save, overwrite)
    }

    fn delete_game(&self, id: &str) -> Result<(), String> {
        let app = self.app.clone();
        tauri::async_runtime::block_on(crate::games::delete_user_game_by_id(&app, id))
    }

    fn game_open(&self, id: Option<&str>) -> Result<(), String> {
        let app = self.app.clone();
        let game_id = id.map(str::to_string);
        tauri::async_runtime::block_on(crate::games::open_game_space(app, game_id))
    }

    fn game_close(&self) -> Result<(), String> {
        let app = self.app.clone();
        tauri::async_runtime::block_on(crate::games::close(&app))
    }
}

type Run = Box<dyn Fn(&dyn Desktop, &Value) -> Result<String, String> + Send + Sync>;

struct DesktopTool {
    spec: ToolSpec,
    desktop: Arc<dyn Desktop>,
    run: Run,
}

impl Tool for DesktopTool {
    fn spec(&self) -> &ToolSpec {
        &self.spec
    }

    fn execute(&self, args: Value, _ctx: &ToolCtx) -> Result<String, String> {
        (self.run)(self.desktop.as_ref(), &args)
    }
}

fn object(properties: Value, required: &[&str]) -> Value {
    json!({ "type": "object", "properties": properties, "required": required })
}

fn text_arg<'a>(args: &'a Value, key: &str) -> Result<&'a str, String> {
    args.get(key).and_then(Value::as_str).map(str::trim).filter(|s| !s.is_empty()).ok_or_else(|| format!("`{key}` is required and must be a non-empty string"))
}

fn optional_text<'a>(args: &'a Value, key: &str) -> Result<Option<&'a str>, String> {
    match args.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => Ok(Some(s.trim())),
        Some(_) => Err(format!("`{key}` must be a string")),
    }
}

fn optional_bool(args: &Value, key: &str) -> Result<Option<bool>, String> {
    match args.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Bool(b)) => Ok(Some(*b)),
        Some(_) => Err(format!("`{key}` must be true or false")),
    }
}

const LIST_LIMIT: usize = 200;

fn wallpaper_list(desktop: &dyn Desktop, _: &Value) -> Result<String, String> {
    let library = desktop.wallpapers()?;
    let active = library["activeId"].as_str().unwrap_or("companion").to_string();
    let mut lines = vec![format!("当前使用：{}", if active == "companion" { "系统桌面（没有壁纸）".to_string() } else { active.clone() })];
    let items = library["items"].as_array().cloned().unwrap_or_default();
    for item in items.iter().take(LIST_LIMIT) {
        let definition = &item["definition"];
        let id = definition["id"].as_str().unwrap_or("?");
        lines.push(format!(
            "{id} | {} | {} | {}{}",
            definition["title"].as_str().unwrap_or("未命名"),
            definition["kind"].as_str().unwrap_or("?"),
            if item["installed"].as_bool().unwrap_or(false) { "已下载" } else { "未下载，应用时会先下载" },
            if id == active { " | 正在使用" } else { "" },
        ));
    }
    if items.len() > LIST_LIMIT {
        lines.push(format!("（还有 {} 个没有列出）", items.len() - LIST_LIMIT));
    }
    Ok(lines.join("\n"))
}

fn wallpaper_apply(desktop: &dyn Desktop, args: &Value) -> Result<String, String> {
    let id = text_arg(args, "id")?;
    desktop.apply_wallpaper(id)?;
    Ok(if id == "companion" { "已换回系统桌面".to_string() } else { format!("已应用壁纸 {id}") })
}

fn wallpaper_create(desktop: &dyn Desktop, args: &Value) -> Result<String, String> {
    let title = text_arg(args, "title")?;
    let html = text_arg(args, "html")?;
    let description = optional_text(args, "description")?;
    let apply_now = optional_bool(args, "apply_now")?.unwrap_or(true);

    let res = desktop.create_wallpaper(title, description, html, apply_now)?;
    let id = res["id"].as_str().unwrap_or_default();
    let applied_msg = if res["applied"].as_bool().unwrap_or(false) {
        "，并已立即应用为当前桌面壁纸"
    } else {
        "（可在壁纸库随时选用）"
    };
    Ok(format!("已成功创作动态网页壁纸「{title}」（id: {id}）{applied_msg}。"))
}

fn wallpaper_delete(desktop: &dyn Desktop, args: &Value) -> Result<String, String> {
    let id = text_arg(args, "id")?;
    desktop.delete_wallpaper(id)?;
    Ok(format!("已删除本地壁纸 {id}。若当前正在使用，已自动换回系统桌面。"))
}

fn effect_list(desktop: &dyn Desktop, _: &Value) -> Result<String, String> {
    let value = desktop.effects()?;
    let selected = value["selected"].as_str().unwrap_or("");
    let mut lines = vec![format!("当前特效：{}", if selected.is_empty() { "无" } else { selected })];
    for pack in value["packs"].as_array().into_iter().flatten().take(LIST_LIMIT) {
        let id = pack["id"].as_str().unwrap_or("?");
        lines.push(format!("{id} | {}{}", pack["name"].as_str().unwrap_or(id), if id == selected { " | 正在使用" } else { "" }));
    }
    Ok(lines.join("\n"))
}

fn effect_set(desktop: &dyn Desktop, args: &Value) -> Result<String, String> {
    let id = text_arg(args, "id")?;
    if id == "none" {
        desktop.set_effect("")?;
        return Ok("已关闭桌面特效".into());
    }
    desktop.set_effect(id)?;
    Ok(format!("已切换到特效 {id}"))
}

/// The pack the model wrote: an object, or the same thing as a JSON string.
fn effect_pack_arg(args: &Value) -> Result<Value, String> {
    match args.get("pack") {
        Some(Value::String(text)) => serde_json::from_str(text.trim_start_matches('\u{feff}'))
            .map_err(|e| format!("pack 不是有效的 JSON：{e}")),
        Some(value @ Value::Object(_)) => Ok(value.clone()),
        Some(_) => Err("pack 需要是一个 JSON 对象".into()),
        None => Err("`pack` is required and must be a non-empty string".into()),
    }
}

fn effect_create(desktop: &dyn Desktop, args: &Value) -> Result<String, String> {
    let pack = effect_pack_arg(args)?;
    let overwrite = optional_bool(args, "overwrite")?.unwrap_or(false);
    let file = desktop.create_effect(&pack, overwrite)?;
    let name = pack["name"].as_str().unwrap_or_default();
    let id = pack["id"].as_str().unwrap_or_default();
    Ok(format!("已创建特效「{name}」（id: {id}，文件 {file}），可在“桌面特效”页选用。"))
}

fn effect_delete(desktop: &dyn Desktop, args: &Value) -> Result<String, String> {
    let id = text_arg(args, "id")?;
    desktop.delete_effect(id)?;
    Ok(format!("已删除自定义特效 {id}。若当前正在使用，已自动重置为无特效。"))
}

fn companion_set(desktop: &dyn Desktop, args: &Value) -> Result<String, String> {
    let change = CompanionChange {
        expression: optional_text(args, "expression")?.map(str::to_string),
        skin: optional_text(args, "skin")?.map(str::to_string),
        enabled: optional_bool(args, "enabled")?,
    };
    if change == CompanionChange::default() {
        return Err("请至少给出 expression、skin 或 enabled 之一".into());
    }
    if let Some(expression) = &change.expression {
        if !EXPRESSIONS.contains(&expression.as_str()) {
            return Err(format!("没有这个表情“{expression}”，可用：{}", EXPRESSIONS.join("、")));
        }
    }
    if let Some(skin) = &change.skin {
        if !SKINS.contains(&skin.as_str()) {
            return Err(format!("没有这个外壳“{skin}”，可用：{}", SKINS.join("、")));
        }
    }
    desktop.set_companion(&change)?;
    let mut done = Vec::new();
    if let Some(e) = &change.expression { done.push(format!("表情 {e}")); }
    if let Some(s) = &change.skin { done.push(format!("外壳 {s}")); }
    if let Some(on) = change.enabled { done.push(if on { "显示小伴".to_string() } else { "隐藏小伴".to_string() }); }
    Ok(format!("已设置：{}", done.join("，")))
}

const WEEKDAYS: [&str; 7] = ["周一", "周二", "周三", "周四", "周五", "周六", "周日"];

fn describe_rule(rule: &Value) -> String {
    let id = rule["id"].as_str().unwrap_or("?");
    let text = rule["text"].as_str().unwrap_or("");
    let when = match (rule["kind"].as_str().unwrap_or(""), rule["repeat"].as_str().unwrap_or("")) {
        ("rest", _) => format!("每 {} 分钟", rule["intervalMinutes"].as_u64().unwrap_or(0)),
        ("weather", _) => format!("天气 {}", rule["time"].as_str().unwrap_or("")),
        (_, "once") => format!("一次 {}", rule["onceAt"].as_str().unwrap_or("")),
        (_, "weekdays") => {
            let days: Vec<&str> = rule["weekdays"].as_array().into_iter().flatten().filter_map(Value::as_u64).filter(|d| (1..=7).contains(d)).map(|d| WEEKDAYS[d as usize - 1]).collect();
            format!("{} {}", days.join("、"), rule["time"].as_str().unwrap_or(""))
        }
        _ => format!("每天 {}", rule["time"].as_str().unwrap_or("")),
    };
    let state = if rule["enabled"].as_bool().unwrap_or(true) { "" } else { " | 已停用" };
    format!("{id} | {when} | {text}{state}")
}

fn care_list(desktop: &dyn Desktop, _: &Value) -> Result<String, String> {
    let value = desktop.care()?;
    let rules = value["rules"].as_array().cloned().unwrap_or_default();
    let on = value["config"]["enabled"].as_bool().unwrap_or(false);
    let mut lines = vec![format!("关怀提醒总开关：{}，共 {} 条规则", if on { "开" } else { "关" }, rules.len())];
    lines.extend(rules.iter().map(describe_rule));
    Ok(lines.join("\n"))
}

/// Builds the rule to save: the saved one when `id` names it, otherwise a new one, with the given fields on top.
fn rule_from_args(args: &Value, existing: Option<Rule>) -> Result<Rule, String> {
    let mut rule = existing.unwrap_or_default();
    if let Some(text) = optional_text(args, "text")? {
        rule.text = text.to_string();
    }
    if let Some(kind) = optional_text(args, "kind")? {
        if !["custom", "off-work", "rest"].contains(&kind) {
            return Err("kind 只能是 custom（普通提醒）、off-work（下班提醒）或 rest（定时休息）；天气提醒请让用户在界面里设置".into());
        }
        rule.kind = kind.to_string();
    }
    if let Some(repeat) = optional_text(args, "repeat")? {
        if !["daily", "weekdays", "once"].contains(&repeat) {
            return Err("repeat 只能是 daily、weekdays 或 once".into());
        }
        rule.repeat = repeat.to_string();
    }
    if let Some(time) = optional_text(args, "time")? {
        rule.time = time.to_string();
    }
    if let Some(once_at) = optional_text(args, "onceAt")? {
        rule.once_at = once_at.to_string();
    }
    if let Some(days) = args.get("weekdays").filter(|v| !v.is_null()) {
        let days = days.as_array().ok_or("weekdays 必须是数组，1 到 7 分别是周一到周日")?;
        rule.weekdays = days.iter().map(|d| d.as_u64().filter(|d| (1..=7).contains(d)).map(|d| d as u32).ok_or("weekdays 里只能是 1 到 7")).collect::<Result<_, _>>()?;
    }
    if let Some(minutes) = args.get("intervalMinutes").filter(|v| !v.is_null()) {
        rule.interval_minutes = minutes.as_u64().filter(|m| *m <= 100_000).ok_or("intervalMinutes 必须是 5 到 480 之间的整数")? as u32;
    }
    if let Some(enabled) = optional_bool(args, "enabled")? {
        rule.enabled = enabled;
    }
    if rule.kind == "weather" {
        return Err("不能修改天气提醒，请让用户在“关怀提醒”页里操作".into());
    }
    Ok(rule)
}

fn care_set(desktop: &dyn Desktop, args: &Value) -> Result<String, String> {
    let id = optional_text(args, "id")?.filter(|s| !s.is_empty());
    let existing = match id {
        Some(id) => {
            let rules = desktop.care()?["rules"].as_array().cloned().unwrap_or_default();
            let found = rules.into_iter().find(|r| r["id"].as_str() == Some(id)).ok_or_else(|| format!("没有 id 为 {id} 的提醒，可先用 care_list 查看"))?;
            Some(serde_json::from_value::<Rule>(found).map_err(|e| format!("读取已有提醒失败：{e}"))?)
        }
        None => None,
    };
    let creating = existing.is_none();
    let rule = rule_from_args(args, existing)?;
    desktop.care_upsert(rule.clone())?;
    Ok(if creating { format!("已添加提醒：{}", rule.text) } else { format!("已更新提醒：{}", rule.text) })
}


fn widget_kind_label(kind: &str) -> &'static str {
    match kind {
        "fortune" => "木鱼求签",
        "calendar" => "小伴日历",
        "weather" => "天气时钟",
        "notes" => "便签待办",
        "countdown" => "倒数日／纪念日",
        "custom" => "自创组件",
        _ => "其他组件",
    }
}

fn widget_theme_label(theme: &str) -> &'static str {
    match theme {
        "glass" => "毛玻璃",
        "paper" => "纸张",
        "transparent" => "透明",
        _ => "默认",
    }
}

fn widget_list(desktop: &dyn Desktop, _: &Value) -> Result<String, String> {
    let value = desktop.widgets()?;
    let widgets = value["widgets"].as_array().cloned().unwrap_or_default();
    let mut lines = if widgets.is_empty() {
        vec!["桌面当前没有放置任何组件。可用 widget_add 添加（支持 fortune 木鱼求签、calendar 小伴日历、weather 天气时钟、notes 便签待办、countdown 倒数日／纪念日）。".to_string()]
    } else { vec![format!("桌面当前放置了 {} 个组件（最多 12 个）：", widgets.len())] };
    for w in &widgets {
        let id = w["id"].as_str().unwrap_or("?");
        let kind = w["kind"].as_str().unwrap_or("");
        let theme = w["theme"].as_str().unwrap_or("");
        let scale = w["scale"].as_f64().unwrap_or(1.0);
        let enabled = w["enabled"].as_bool().unwrap_or(true);
        let city = w.get("city").and_then(|c| c.get("name")).and_then(Value::as_str).map(|n| format!(" | 城市: {n}")).unwrap_or_default();
        let state = if enabled { "显示中" } else { "已隐藏" };
        lines.push(format!("{id} | {} ({kind}) | {}风格 ({theme}) | 缩放 {scale:.2}{city} | {state}", widget_kind_label(kind), widget_theme_label(theme)));
    }
    let templates = desktop.custom_widgets()?;
    if let Some(templates) = templates.as_array().filter(|items| !items.is_empty()) {
        lines.push("自创组件模板（可在组件库添加到桌面，删除模板用 widget_delete_custom）：".into());
        for template in templates {
            lines.push(format!("{} | {}", template["id"].as_str().unwrap_or("?"), template["name"].as_str().unwrap_or("自创组件")));
        }
    }
    Ok(lines.join("\n"))
}

fn widget_add(desktop: &dyn Desktop, args: &Value) -> Result<String, String> {
    let kind = text_arg(args, "kind")?;
    if !crate::custom_widgets::BUILTIN_KINDS.contains(&kind) {
        return Err("kind 只能是 fortune（木鱼求签）、calendar（小伴日历）、weather（天气时钟）、notes（便签待办）或 countdown（倒数日／纪念日）".into());
    }
    // Validate all options before creating anything, and never hide a failed theme update.
    let theme = optional_text(args, "theme")?;
    if theme.is_some_and(|theme| !["glass", "paper", "transparent"].contains(&theme)) {
        return Err("theme 必须是 glass（毛玻璃）、paper（纸张）或 transparent（透明）".into());
    }
    let res = desktop.widget_add(kind)?;
    let id = res["widgets"].as_array().and_then(|w| w.last()).and_then(|w| w.get("id")).and_then(Value::as_str).ok_or("组件创建结果没有返回 id")?;
    if let Some(theme) = theme {
        desktop.widget_update(id, json!({ "theme": theme })).map_err(|e| format!("组件已添加（id: {id}），但设置外观失败：{e}"))?;
    }
    Ok(format!("已在桌面添加组件：{}（id: {id}）", widget_kind_label(kind)))
}

fn parse_widget_patch(args: &Value) -> Result<Value, String> {
    let mut obj = serde_json::Map::new();
    if let Some(v) = args.get("theme").filter(|v| !v.is_null()) {
        let theme = v.as_str().ok_or("theme 必须是字符串")?;
        if !["glass", "paper", "transparent"].contains(&theme) {
            return Err("theme 必须是 glass（毛玻璃）、paper（纸张）或 transparent（透明）".into());
        }
        obj.insert("theme".into(), json!(theme));
    }
    if let Some(v) = args.get("scale").filter(|v| !v.is_null()) {
        let s = v.as_f64().ok_or("scale 必须是数字")?;
        if !s.is_finite() || !(0.75..=1.5).contains(&s) {
            return Err("scale 缩放范围为 0.75 到 1.5".into());
        }
        obj.insert("scale".into(), json!(s));
    }
    if let Some(v) = args.get("x").filter(|v| !v.is_null()) {
        let x = v.as_f64().ok_or("x 必须是数字")?;
        if !x.is_finite() || !(0.0..=1.0).contains(&x) {
            return Err("x 坐标范围为 0.0 到 1.0".into());
        }
        obj.insert("x".into(), json!(x));
    }
    if let Some(v) = args.get("y").filter(|v| !v.is_null()) {
        let y = v.as_f64().ok_or("y 必须是数字")?;
        if !y.is_finite() || !(0.0..=1.0).contains(&y) {
            return Err("y 坐标范围为 0.0 到 1.0".into());
        }
        obj.insert("y".into(), json!(y));
    }
    if let Some(v) = args.get("enabled").filter(|v| !v.is_null()) {
        let b = v.as_bool().ok_or("enabled 必须是 true 或 false")?;
        obj.insert("enabled".into(), json!(b));
    }
    if let Some(v) = args.get("city").filter(|v| !v.is_null()) {
        for coordinate in ["latitude", "longitude"] {
            if v.get(coordinate).and_then(Value::as_f64).is_none() { return Err(format!("city 需要有效的 {coordinate} 经纬度")); }
        }
        let city: crate::care_weather::City = serde_json::from_value(v.clone()).map_err(|e| format!("城市格式无效：{e}"))?;
        city.validate()?;
        obj.insert("city".into(), serde_json::to_value(&city).map_err(|e| e.to_string())?);
    }
    if obj.is_empty() {
        return Err("请至少指定 theme、scale、x、y、enabled 或 city 之一".into());
    }
    Ok(Value::Object(obj))
}

fn widget_update(desktop: &dyn Desktop, args: &Value) -> Result<String, String> {
    let id = text_arg(args, "id")?;
    let patch = parse_widget_patch(args)?;
    desktop.widget_update(id, patch)?;
    Ok(format!("已更新桌面组件 {id}"))
}

fn widget_remove(desktop: &dyn Desktop, args: &Value) -> Result<String, String> {
    let id = text_arg(args, "id")?;
    desktop.widget_remove(id)?;
    Ok(format!("已从桌面移除组件 {id}"))
}

fn widget_create(desktop: &dyn Desktop, args: &Value) -> Result<String, String> {
    let id = text_arg(args, "id")?;
    let name = text_arg(args, "name")?;
    let html = text_arg(args, "html")?;
    let description = optional_text(args, "description")?;
    let width = args.get("width").and_then(Value::as_f64);
    let height = args.get("height").and_then(Value::as_f64);
    let theme = optional_text(args, "theme")?.unwrap_or("transparent");
    let overwrite = optional_bool(args, "overwrite")?.unwrap_or(false);
    let place_now = optional_bool(args, "place_now")?.unwrap_or(true);

    let mut pack = json!({
        "schemaVersion": 1,
        "id": id,
        "name": name,
        "html": html,
        "theme": theme,
    });
    if let Some(desc) = description {
        pack["description"] = json!(desc);
    }
    if let Some(w) = width {
        pack["width"] = json!(w);
    }
    if let Some(h) = height {
        pack["height"] = json!(h);
    }
    let res = desktop.create_custom_widget(&pack, place_now, overwrite)?;
    let placed_msg = if res["placed"].as_bool().unwrap_or(false) {
        "，并已直接添加到桌面上展示"
    } else {
        "（未自动放置到桌面，可在组件库随时添加）"
    };
    Ok(format!("已成功创建小组件「{name}」（id: {id}）{placed_msg}。"))
}

fn widget_delete_custom(desktop: &dyn Desktop, args: &Value) -> Result<String, String> {
    let id = text_arg(args, "id")?;
    desktop.delete_custom_widget(id)?;
    Ok(format!("已删除自定义小组件「{id}」模板，桌面上若有该组件实例也已同步清理。"))
}


pub fn resolve_game_id(input: &str) -> Option<&'static str> {
    let lower = input.trim().to_lowercase();
    for &id in &crate::games::IDS {
        if id == lower {
            return Some(id);
        }
    }
    match lower.as_str() {
        "贪吃蛇" | "小伴贪吃蛇" | "蛇" | "snake" | "snake_arena" => Some("snake-arena"),
        "俄罗斯方块" | "方块" | "tetris" | "falling_blocks" => Some("falling-blocks"),
        "2048" | "merge" | "merge_2048" => Some("merge-2048"),
        "消失高手" | "消消乐" | "block_puzzle" => Some("block-puzzle"),
        "奶茶大师" | "倒水" | "分杯" | "奶茶" | "water_sort" => Some("water-sort"),
        "水墨剑客" | "剑客" | "水墨" | "ink_blade" => Some("ink-blade"),
        "暮色滑翔" | "滑翔" | "滑翔机" | "dusk_glider" => Some("dusk-glider"),
        _ => None,
    }
}

fn is_user_game(game: &Value) -> bool {
    game["source"].as_str() == Some("user")
}

fn game_list(desktop: &dyn Desktop, _: &Value) -> Result<String, String> {
    let data = desktop.games()?;
    let active = data["active"].as_bool().unwrap_or(false);
    let state = if active { "当前正在游戏空间中" } else { "游戏空间未打开" };
    let games = data["games"].as_array().cloned().unwrap_or_default();
    let line = |g: &Value| {
        let id = g["id"].as_str().unwrap_or("");
        format!("- {} ({id}): {}", g["name"].as_str().unwrap_or(id), g["description"].as_str().unwrap_or(""))
    };
    let mut lines = vec![format!("小伴内置 7 款独立桌面小游戏（{state}）：")];
    lines.extend(games.iter().filter(|g| !is_user_game(g)).map(line));
    let mine: Vec<&Value> = games.iter().filter(|g| is_user_game(g)).collect();
    if !mine.is_empty() {
        lines.push(format!("用户自创游戏（{} 款，来源：游戏文件夹，可用 game_delete 删除）：", mine.len()));
        lines.extend(mine.into_iter().map(|g| format!("{} [自创]", line(g))));
    }
    for error in data["errors"].as_array().into_iter().flatten().filter_map(Value::as_str) {
        lines.push(format!("无法加载的自创游戏文件夹：{error}"));
    }
    lines.push(r#"可用 game_open 启动（例如 game_open(id: "snake-arena") 打开贪吃蛇），game_close 退出游戏空间；game_create 可以为用户创作新游戏。"#.into());
    Ok(lines.join("\n"))
}

fn game_open(desktop: &dyn Desktop, args: &Value) -> Result<String, String> {
    let id_opt = optional_text(args, "id")?.filter(|s| !s.is_empty());
    let target = match id_opt {
        Some(raw) => {
            if let Some(id) = resolve_game_id(raw) {
                Some(id.to_string())
            } else {
                // User games match on their exact id only.
                let games = desktop.games()?;
                let mine: Vec<&Value> = games["games"].as_array().into_iter().flatten().filter(|g| is_user_game(g)).collect();
                match mine.iter().find(|g| g["id"].as_str() == Some(raw)) {
                    Some(game) => game["id"].as_str().map(str::to_string),
                    None => {
                        let extra: String = mine.iter().filter_map(|g| g["id"].as_str().map(|id| format!("、{} ({id})", g["name"].as_str().unwrap_or(id)))).collect();
                        return Err(format!("没有找到游戏“{raw}”。支持的游戏：小伴贪吃蛇 (snake-arena)、俄罗斯方块 (falling-blocks)、2048 (merge-2048)、消失高手 (block-puzzle)、奶茶大师 (water-sort)、水墨剑客 (ink-blade)、暮色滑翔 (dusk-glider){extra}"));
                    }
                }
            }
        }
        None => None,
    };
    desktop.game_open(target.as_deref())?;
    Ok(match target {
        Some(id) => format!("已进入游戏空间并启动游戏：{id}"),
        None => "已打开小伴游戏空间大厅".into(),
    })
}

fn game_create(desktop: &dyn Desktop, args: &Value) -> Result<String, String> {
    let id = text_arg(args, "id")?;
    let name = text_arg(args, "name")?;
    let description = text_arg(args, "description")?;
    let save = optional_bool(args, "save")?.unwrap_or(false);
    let overwrite = optional_bool(args, "overwrite")?.unwrap_or(false);
    let items = args.get("files").and_then(Value::as_array).ok_or("`files` is required and must be an array of { path, content }")?;
    let mut files = Vec::with_capacity(items.len());
    for item in items {
        let path = item.get("path").and_then(Value::as_str).ok_or("each file needs a string `path`")?;
        let content = item.get("content").and_then(Value::as_str).ok_or_else(|| format!("文件 {path} 缺少文本 `content`"))?;
        files.push((path.to_string(), content.to_string()));
    }
    // Same checks the writer runs, so a bad request is refused before the desktop is involved.
    crate::game_packs::validate_request(id, name, description, &files)?;
    let made = desktop.create_game(id, name, description, &files, save, overwrite)?;
    let replaced = if made["replaced"].as_bool().unwrap_or(false) { "（已覆盖同 id 的旧版本）" } else { "" };
    let saves = if save { "，已启用本机存档" } else { "" };
    Ok(format!("已保存游戏「{name}」（id: {id}）{replaced}{saves}，共 {} 个文件。游戏还没有打开，需要试玩时调用 game_open(id: \"{id}\")。", files.len() + 1))
}

fn game_delete(desktop: &dyn Desktop, args: &Value) -> Result<String, String> {
    let id = text_arg(args, "id")?;
    if crate::game_packs::is_builtin(id) {
        return Err(format!("「{id}」是内置游戏，不能删除；只能删除 game_create 创作的自创游戏"));
    }
    desktop.delete_game(id)?;
    Ok(format!("已删除自创游戏 {id}（游戏空间若正在运行已先关闭，它的本机存档也一并清除）。"))
}

const GAME_CREATE_DESCRIPTION: &str = "为用户创作一款全屏离线小游戏，保存成游戏文件夹（用户数据目录 games/packs/<id>/，文件夹即游戏）。保存后不会自动打开，试玩请再调用 game_open(id)；可用 game_list 查看、game_delete 删除。\n\
游戏包规范：\n\
1. 文件：files 必须包含 index.html 作为入口，其余文件用相对路径引用（如 <script src=\"game.js\"></script>、<link rel=\"stylesheet\" href=\"style.css\">）。只能写 html、js、css、json、svg、md、txt 文本文件，最多 40 个，单个不超过 1 MB，合计不超过 4 MB；路径只用字母数字和 . _ -，用 / 分层，最多 4 层，不以 . 开头。不要写 manifest.json（由本工具生成）；若包含 cover.svg 会用作封面。\n\
2. 脚本：用普通经典脚本（<script src>），不要 ES modules（type=module、import/export），所有资源用相对路径，不联网、不引用任何外部 CDN、字体或图片；图片用内联 SVG 或 canvas 绘制。\n\
3. 运行环境：游戏在沙盒 iframe 里全屏运行，没有网络、localStorage、cookie、IndexedDB 和任何系统权限。画布与布局适配 100vw × 100vh，同时支持键盘和鼠标/触摸。宿主会自动注入 desktop-host.js，由它处理 Esc 退出，游戏不要自己再处理 Esc，也不要自己引用它。\n\
4. 握手协议（宿主注入的 desktop-host.js 会在收到 host.init 时自动回复 game.ready，所以不读存档的游戏可以不写；需要存档或成绩时按下面实现）：在页面脚本顶层同步监听 window 的 message。宿主在 iframe 加载完成后发送 {channel:'xiaban.game-host', protocol:1, type:'host.init', nonce, save, safeArea}，其中 save 是上次存档对象或 null。只接受 event.source === window.parent 且 channel、protocol、nonce 都匹配的消息；nonce 与地址栏参数 hostNonce 相同（new URLSearchParams(location.search).get('hostNonce')）。收到 host.init 后回复 window.parent.postMessage({channel:'xiaban.game-pack', protocol:1, nonce, type:'game.ready'}, '*')；game.ready 之前发出的其它消息会被忽略。\n\
5. 可选成绩：开始一局发 type:'game.start'，之后可发 type:'game.progress' 和一次 type:'game.result'，都带非负整数 score（消息结构同上，channel 为 'xiaban.game-pack'，带 nonce）。\n\
6. 存档（仅 save=true 时有效）：游戏在 game.ready 之后用 {channel:'xiaban.game-pack', protocol:1, nonce, type:'game.save', data:{…}} 提交存档，data 必须是普通 JSON 对象，序列化后不超过 32 KiB；宿主存在本机，下次启动时放进 host.init 的 save 字段。进度变化时就发送，不要依赖退出时才存。save=false 时不要发送 game.save，宿主会忽略。";

fn game_close(desktop: &dyn Desktop, _: &Value) -> Result<String, String> {
    desktop.game_close()?;
    Ok("已退出游戏空间，返回桌面".into())
}

fn care_delete(desktop: &dyn Desktop, args: &Value) -> Result<String, String> {
    let id = text_arg(args, "id")?;
    desktop.care_delete(id)?;
    Ok(format!("已删除提醒 {id}"))
}

fn tool(desktop: &Arc<dyn Desktop>, spec: ToolSpec, run: impl Fn(&dyn Desktop, &Value) -> Result<String, String> + Send + Sync + 'static) -> Arc<dyn Tool> {
    Arc::new(DesktopTool { spec, desktop: desktop.clone(), run: Box::new(run) })
}

fn effect_read_format(_: &dyn Desktop, _: &Value) -> Result<String, String> {
    Ok(format!(
        "以下是小伴程序内置的完整特效包格式和示例，可离线读取，无需查找本地文件。文档中的源码路径仅供开发参考，示例已附在末尾。\n\n{}\n\n## 完整可用示例\n\n```json\n{}\n```",
        crate::effect_packs::FORMAT_GUIDE,
        crate::effect_packs::EXAMPLE,
    ))
}

pub fn tools(desktop: Arc<dyn Desktop>) -> Vec<Arc<dyn Tool>> {
    let none = || object(json!({}), &[]);
    let status_run = |d: &dyn Desktop, _: &Value| -> Result<String, String> { Ok(d.status()?.to_string()) };
    vec![
        tool(&desktop, ToolSpec::new("desktop_status", "查看小伴桌面当前的状态：显示模式、正在用的壁纸和特效、小伴的外壳与表情。", none()), status_run),
        tool(&desktop, ToolSpec::new("wallpaper_list", "列出可用的壁纸（id、标题、类型、是否已下载）和当前正在使用的那张。", none()), wallpaper_list),
        tool(
            &desktop,
            ToolSpec::new("wallpaper_apply", "把桌面壁纸换成指定的壁纸，用 wallpaper_list 里的 id。id 为 companion 表示换回没有壁纸的系统桌面。", object(json!({"id": {"type": "string"}}), &["id"]))
                .risk(Risk::Write)
                .serial()
                .timeout(std::time::Duration::from_secs(180)),
            wallpaper_apply,
        ),
        tool(
            &desktop,
            ToolSpec::new(
                "wallpaper_create",
                "为用户创作一份全新的自包含 HTML 动态/交互桌面壁纸（或整屏桌面小游戏）。可编写单文件 HTML、CSS 与 JavaScript 脚本（支持 Canvas、WebGL、CSS 动效及动画循环），自动适应 100vw/100vh 屏幕尺寸。创作完成后会自动保存至本地壁纸库并去重，默认立即应用到当前桌面。",
                object(
                    json!({
                        "title": { "type": "string", "description": "动态壁纸标题（最多 48 字，如 水墨涟漪交互壁纸）" },
                        "html": { "type": "string", "description": "完整的自包含单文件 HTML 内容（包含完整的 <!DOCTYPE html>、<style>、Canvas/DOM 结构与动画逻辑）" },
                        "description": { "type": "string", "description": "一句话介绍（最多 80 字）" },
                        "apply_now": { "type": "boolean", "description": "创作完成后是否立即应用为桌面壁纸，默认 true" },
                    }),
                    &["title", "html"],
                ),
            )
            .risk(Risk::Write)
            .serial(),
            wallpaper_create,
        ),
        tool(
            &desktop,
            ToolSpec::new("wallpaper_delete", "删除指定的本地导入或创作的壁纸，id 来自 wallpaper_list（必须是 local- 开头）。若该壁纸正在使用，会自动换回系统桌面。", object(json!({"id": {"type": "string"}}), &["id"])).risk(Risk::Write).serial(),
            wallpaper_delete,
        ),
        tool(&desktop, ToolSpec::new("effect_list", "列出桌面特效（雨、雪、花瓣等）和正在使用的那个。", none()), effect_list),
        tool(
            &desktop,
            ToolSpec::new("effect_read_format", "读取程序内置的完整桌面特效包格式与可用 JSON 示例。创作特效前先调用本工具；无需搜索工作目录、安装目录或联网查找文档。", none()).risk(Risk::Read),
            effect_read_format,
        ),
        tool(
            &desktop,
            ToolSpec::new("effect_set", "切换桌面特效，用 effect_list 里的 id；id 为 none 表示关闭特效。", object(json!({"id": {"type": "string"}}), &["id"])).risk(Risk::Write).serial(),
            effect_set,
        ),
        tool(
            &desktop,
            ToolSpec::new(
                "effect_create",
                format!("为用户创作一个新的桌面特效包并保存到特效文件夹，保存后它就能像内置特效一样被选用。先调用 effect_read_format 阅读内置的完整格式和示例，再按下面的格式写一份完整的 JSON 放进 pack。pack 是一个 JSON 对象（也可写成 JSON 字符串），必须包含 schemaVersion、id、name、layers。{}", crate::effects::format_help()),
                object(
                    json!({
                        "pack": {
                            "type": "object",
                            "description": "完整的特效包，字段见工具说明。",
                            "properties": {
                                "schemaVersion": {"type": "integer", "const": 1},
                                "id": {"type": "string", "description": "小写字母、数字、- 和 _，不能与内置特效重名"},
                                "name": {"type": "string", "description": "显示名，最多 24 字"},
                                "description": {"type": "string", "description": "一句话说明，最多 80 字"},
                                "sky": {"type": "array", "items": {"type": "string"}, "description": "2 到 4 个十六进制颜色"},
                                "layers": {"type": "array", "items": {"type": "object"}, "description": "1 到 6 层粒子"},
                            },
                            "required": ["schemaVersion", "id", "name", "layers"],
                        },
                        "overwrite": {"type": "boolean", "description": "true 时覆盖同 id 的已有自定义特效，默认 false"},
                    }),
                    &["pack"],
                ),
            ).risk(Risk::Write).serial(),
            effect_create,
        ),
        tool(
            &desktop,
            ToolSpec::new("effect_delete", "删除指定的自定义特效包，id 来自 effect_list（内置的 12 个特效受保护不可删）。若该特效正在使用，会自动重置为无特效。", object(json!({"id": {"type": "string"}}), &["id"])).risk(Risk::Write).serial(),
            effect_delete,
        ),
        tool(
            &desktop,
            ToolSpec::new(
                "companion_set",
                "设置桌面小伴：表情、外壳，或显示/隐藏。至少给一项。",
                object(
                    json!({
                        "expression": {"type": "string", "enum": EXPRESSIONS},
                        "skin": {"type": "string", "enum": SKINS},
                        "enabled": {"type": "boolean", "description": "true 显示小伴，false 隐藏小伴"},
                    }),
                    &[],
                ),
            )
            .risk(Risk::Write)
            .serial(),
            companion_set,
        ),
        tool(&desktop, ToolSpec::new("care_list", "列出关怀提醒的规则（id、时间、文案）和总开关状态。", none()), care_list),
        tool(
            &desktop,
            ToolSpec::new(
                "care_set",
                "添加或修改一条关怀提醒。不给 id 是添加，给了 id 是修改那一条（没给的字段保持不变）。时间用 24 小时制的本地时间。",
                object(
                    json!({
                        "id": {"type": "string", "description": "要修改的提醒 id，添加时不填"},
                        "text": {"type": "string", "description": "提醒的文字，最多 240 字"},
                        "kind": {"type": "string", "enum": ["custom", "off-work", "rest"], "description": "custom 普通提醒（默认），off-work 下班提醒，rest 每隔一段时间提醒休息"},
                        "repeat": {"type": "string", "enum": ["daily", "weekdays", "once"], "description": "每天 / 每周的指定几天 / 只提醒一次"},
                        "time": {"type": "string", "description": "HH:MM，例如 15:00"},
                        "onceAt": {"type": "string", "description": "repeat 为 once 时的本地日期时间，格式 YYYY-MM-DDTHH:MM，必须是未来的时间"},
                        "weekdays": {"type": "array", "items": {"type": "integer", "minimum": 1, "maximum": 7}, "description": "repeat 为 weekdays 时用，1 是周一，7 是周日"},
                        "intervalMinutes": {"type": "integer", "minimum": 5, "maximum": 480, "description": "kind 为 rest 时的间隔分钟数"},
                        "enabled": {"type": "boolean"},
                    }),
                    &[],
                ),
            )
            .risk(Risk::Write)
            .serial(),
            care_set,
        ),
        tool(&desktop, ToolSpec::new("care_delete", "删除一条关怀提醒，id 来自 care_list。", object(json!({"id": {"type": "string"}}), &["id"])).risk(Risk::Write).serial(), care_delete),
        tool(&desktop, ToolSpec::new("widget_list", "列出桌面当前放置的组件（id、类型、风格、缩放、显示状态）以及自创组件模板的 id 和名称。", none()), widget_list),
        tool(
            &desktop,
            ToolSpec::new(
                "widget_add",
                "在桌面放置新组件：fortune（木鱼求签）、calendar（小伴日历）、weather（天气时钟）、notes（便签待办）或 countdown（倒数日／纪念日），可选 theme（glass 毛玻璃 / paper 纸张 / transparent 透明）。桌面最多 12 个组件。",
                object(
                    json!({
                        "kind": {"type": "string", "enum": crate::custom_widgets::BUILTIN_KINDS, "description": "组件类型"},
                        "theme": {"type": "string", "enum": ["glass", "paper", "transparent"], "description": "可选外观风格，默认 transparent（透明无底）"},
                    }),
                    &["kind"],
                ),
            ).risk(Risk::Write).serial(),
            widget_add,
        ),
        tool(
            &desktop,
            ToolSpec::new(
                "widget_update",
                "修改已有的桌面组件：外观风格、缩放大小（0.75-1.5）、位置（x, y 为 0.0-1.0 比例）、显示/隐藏或城市。至少给一项修改。",
                object(
                    json!({
                        "id": {"type": "string", "description": "组件 id，来自 widget_list"},
                        "theme": {"type": "string", "enum": ["glass", "paper", "transparent"]},
                        "scale": {"type": "number", "minimum": 0.75, "maximum": 1.5, "description": "缩放比例"},
                        "enabled": {"type": "boolean", "description": "true 显示，false 隐藏"},
                        "x": {"type": "number", "minimum": 0.0, "maximum": 1.0, "description": "水平位置比例"},
                        "y": {"type": "number", "minimum": 0.0, "maximum": 1.0, "description": "垂直位置比例"},
                        "city": {
                            "type": "object", "description": "天气城市，需提供真实经纬度；不确定时请让用户在组件页选择城市",
                            "properties": {
                                "name": {"type": "string", "description": "城市名"},
                                "country": {"type": "string"}, "admin1": {"type": "string"},
                                "latitude": {"type": "number", "minimum": -90, "maximum": 90},
                                "longitude": {"type": "number", "minimum": -180, "maximum": 180}
                            },
                            "required": ["name", "latitude", "longitude"]
                        },
                    }),
                    &["id"],
                ),
            ).risk(Risk::Write).serial(),
            widget_update,
        ),
        tool(
            &desktop,
            ToolSpec::new("widget_remove", "从桌面移除指定的组件，id 来自 widget_list。", object(json!({"id": {"type": "string"}}), &["id"])).risk(Risk::Write).serial(),
            widget_remove,
        ),
        tool(
            &desktop,
            ToolSpec::new(
                "widget_create",
                "为用户创作一个新的自定义单文件 HTML 桌面小组件。可编写包含完整 HTML、CSS 样式与 JavaScript 交互逻辑的独立微卡片（如番茄钟、极简时钟、每日诗词、倒数日、健康喝水提醒、趣味小工具等）。默认透明无底，HTML 的页面和主体也不要绘制整块背景；仅在用户需要时添加底板。支持外壳主题（glass 透光玻璃 / paper 温润纸张 / transparent 透明无底），默认宽 280、高 240。创作完成后默认自动放置到桌面上。",
                object(
                    json!({
                        "id": { "type": "string", "description": "小组件英文标识（小写字母、数字、-、_，如 pomodoro-timer）" },
                        "name": { "type": "string", "description": "小组件中文名称（最多 24 字，如 专注番茄钟）" },
                        "html": { "type": "string", "description": "完整的自包含单文件 HTML 内容（包含 <style>、DOM 结构与 <script> 逻辑）；默认让 html、body 背景透明，不绘制铺满组件的底板；内容应在指定 width、height 的视口内自适应，避免横向溢出和滚动条" },
                        "description": { "type": "string", "description": "一句话介绍（最多 80 字）" },
                        "width": { "type": "number", "minimum": 100, "maximum": 800, "description": "建议默认宽度（像素，默认 280）" },
                        "height": { "type": "number", "minimum": 80, "maximum": 800, "description": "建议默认高度（像素，默认 240）" },
                        "theme": { "type": "string", "enum": ["glass", "paper", "transparent"], "description": "底板外观风格，默认 transparent（透明无底）" },
                        "place_now": { "type": "boolean", "description": "创作完成后是否立即放置到桌面上，默认 true" },
                        "overwrite": { "type": "boolean", "description": "true 时覆盖同 id 的已有小组件，默认 false" }
                    }),
                    &["id", "name", "html"],
                ),
            )
            .risk(Risk::Write)
            .serial(),
            widget_create,
        ),
        tool(
            &desktop,
            ToolSpec::new("widget_delete_custom", "删除指定的自定义小组件模板，id 来自 widget_create 定义的 id。若桌面上摆有该组件实例，也会一并从桌面移除。", object(json!({"id": {"type": "string"}}), &["id"])).risk(Risk::Write).serial(),
            widget_delete_custom,
        ),
        tool(&desktop, ToolSpec::new("game_list", "列出小伴支持的小游戏（内置的贪吃蛇、俄罗斯方块、2048、消失高手、奶茶大师、水墨剑客、暮色滑翔，以及用户自创游戏）以及游戏空间状态。", none()), game_list),
        tool(
            &desktop,
            ToolSpec::new(
                "game_open",
                "进入游戏空间全屏玩游戏，可指定游戏 id（例如 snake-arena 为小伴贪吃蛇，falling-blocks 为俄罗斯方块，merge-2048 为 2048，block-puzzle 为消失高手，water-sort 为奶茶大师，ink-blade 为水墨剑客，dusk-glider 为暮色滑翔；用户自创游戏用 game_list 里的 id）。不指定 id 则打开游戏大厅。",
                object(
                    json!({
                        "id": {
                            "type": "string",
                            "description": "游戏标识或内置游戏名称；自创游戏只认精确 id",
                        },
                    }),
                    &[],
                ),
            ).risk(Risk::Write).serial(),
            game_open,
        ),
        tool(
            &desktop,
            ToolSpec::new(
                "game_create",
                GAME_CREATE_DESCRIPTION,
                object(
                    json!({
                        "id": { "type": "string", "description": "游戏英文标识（小写字母、数字、-，2 到 40 个字符，如 star-catcher；不能与内置游戏重名）" },
                        "name": { "type": "string", "description": "游戏中文名称（最多 24 字）" },
                        "description": { "type": "string", "description": "一句话介绍（最多 80 字）" },
                        "files": {
                            "type": "array",
                            "maxItems": 40,
                            "description": "游戏文件列表，必须包含 index.html",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "path": { "type": "string", "description": "相对路径，如 index.html、js/main.js" },
                                    "content": { "type": "string", "description": "UTF-8 文本内容" }
                                },
                                "required": ["path", "content"]
                            }
                        },
                        "save": { "type": "boolean", "description": "游戏是否需要本机存档，默认 false" },
                        "overwrite": { "type": "boolean", "description": "true 时覆盖同 id 的已有自创游戏，默认 false" }
                    }),
                    &["id", "name", "description", "files"],
                ),
            )
            .risk(Risk::Write)
            .serial(),
            game_create,
        ),
        tool(
            &desktop,
            ToolSpec::new("game_delete", "删除指定的自创游戏（只能删除 game_create 创作的、game_list 标注为自创的游戏，内置游戏受保护）。若游戏空间正在运行会先关闭，游戏的本机存档也会一并清除。", object(json!({"id": {"type": "string"}}), &["id"])).risk(Risk::Write).serial(),
            game_delete,
        ),
        tool(&desktop, ToolSpec::new("game_close", "退出游戏空间，回到正常的桌面生活空间。", none()).risk(Risk::Write).serial(), game_close),
        tool(
            &desktop,
            ToolSpec::new(
                "request_accessibility_permission",
                "仅在 macOS 打开辅助功能与屏幕录制设置页面，供用户手动检查和授权。打开设置不等于已获授权；其他平台不支持此引导。",
                none(),
            ).risk(Risk::Write).serial(),
            |d: &dyn Desktop, _: &Value| -> Result<String, String> {
                let res = d.request_permissions()?;
                Ok(res["message"].as_str().unwrap_or("已申请系统权限").to_string())
            },
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[derive(Default)]
    struct Fake {
        calls: Mutex<Vec<String>>,
        rules: Mutex<Vec<Value>>,
        templates: Mutex<Vec<Value>>,
        fail_widget_update: Mutex<Option<String>>,
        fail: Mutex<Option<String>>,
    }

    impl Fake {
        fn log(&self, entry: impl Into<String>) -> Result<(), String> {
            self.calls.lock().unwrap().push(entry.into());
            match self.fail.lock().unwrap().clone() {
                Some(error) => Err(error),
                None => Ok(()),
            }
        }
        fn calls(&self) -> Vec<String> {
            self.calls.lock().unwrap().clone()
        }
    }

    impl Desktop for Fake {
        fn status(&self) -> Result<Value, String> {
            Ok(json!({"mode": "floating", "wallpaper": "w1"}))
        }
        fn wallpapers(&self) -> Result<Value, String> {
            Ok(json!({"activeId": "w1", "items": [
                {"definition": {"id": "w1", "title": "海边", "kind": "image"}, "installed": true, "source": "builtin"},
                {"definition": {"id": "w2", "title": "森林", "kind": "video"}, "installed": false, "source": "subscription"},
            ]}))
        }
        fn apply_wallpaper(&self, id: &str) -> Result<(), String> {
            self.log(format!("apply {id}"))
        }
        fn create_wallpaper(&self, title: &str, _desc: Option<&str>, _html: &str, apply_now: bool) -> Result<Value, String> {
            self.log(format!("create-wallpaper {title} apply_now={apply_now}"))?;
            Ok(json!({"id": "local-mock", "title": title, "applied": apply_now}))
        }
        fn delete_wallpaper(&self, id: &str) -> Result<String, String> {
            self.log(format!("delete-wallpaper {id}"))?;
            Ok("hash".to_string())
        }
        fn effects(&self) -> Result<Value, String> {
            Ok(json!({"selected": "rain", "packs": [{"id": "rain", "name": "细雨"}, {"id": "snow", "name": "飘雪"}]}))
        }
        fn set_effect(&self, id: &str) -> Result<(), String> {
            self.log(format!("effect {id:?}"))
        }
        fn create_effect(&self, pack: &Value, overwrite: bool) -> Result<String, String> {
            self.log(format!("create-effect {} overwrite={overwrite}", pack["id"].as_str().unwrap_or("?")))?;
            Ok(format!("{}.json", pack["id"].as_str().unwrap_or("?")))
        }
        fn delete_effect(&self, id: &str) -> Result<String, String> {
            self.log(format!("delete-effect {id}"))?;
            Ok(format!("{id}.json"))
        }
        fn set_companion(&self, change: &CompanionChange) -> Result<(), String> {
            self.log(format!("companion {change:?}"))
        }
        fn care(&self) -> Result<Value, String> {
            Ok(json!({"config": {"enabled": true}, "rules": self.rules.lock().unwrap().clone()}))
        }
        fn care_upsert(&self, rule: Rule) -> Result<(), String> {
            self.log(format!("upsert {} {} {} {:?} {:?} {} {}", rule.id, rule.kind, rule.repeat, rule.time, rule.weekdays, rule.text, rule.enabled))
        }
        fn care_delete(&self, id: &str) -> Result<(), String> {
            self.log(format!("delete {id}"))
        }
        fn widgets(&self) -> Result<Value, String> {
            Ok(json!({"widgets": [
                {"id": "wgt-1", "kind": "fortune", "theme": "paper", "scale": 1.0, "x": 0.5, "y": 0.1, "enabled": true},
                {"id": "wgt-2", "kind": "weather", "theme": "glass", "scale": 1.2, "x": 0.8, "y": 0.1, "enabled": false, "city": {"name": "上海"}},
            ]}))
        }
        fn widget_add(&self, kind: &str) -> Result<Value, String> {
            self.log(format!("widget-add {kind}"))?;
            Ok(json!({"widgets": [{"id": "wgt-new", "kind": kind}]}))
        }
        fn widget_update(&self, id: &str, patch: Value) -> Result<Value, String> {
            self.log(format!("widget-update {id} {patch}"))?;
            if let Some(error) = self.fail_widget_update.lock().unwrap().clone() { return Err(error); }
            Ok(json!({}))
        }
        fn widget_remove(&self, id: &str) -> Result<(), String> {
            self.log(format!("widget-remove {id}"))
        }
        fn custom_widgets(&self) -> Result<Value, String> {
            Ok(json!(self.templates.lock().unwrap().clone()))
        }
        fn create_custom_widget(&self, pack: &Value, place_now: bool, overwrite: bool) -> Result<Value, String> {
            let id = pack["id"].as_str().unwrap_or("?");
            self.log(format!("create-custom-widget {id} place_now={place_now} overwrite={overwrite}"))?;
            Ok(json!({"id": id, "name": pack["name"], "file": format!("{id}.json"), "placed": place_now}))
        }
        fn delete_custom_widget(&self, id: &str) -> Result<(), String> {
            self.log(format!("delete-custom-widget {id}"))
        }
        fn games(&self) -> Result<Value, String> {
            Ok(json!({"active": false, "games": [
                {"id": "snake-arena", "name": "小伴贪吃蛇", "description": "吃糖果长成大蛇"},
                {"id": "falling-blocks", "name": "俄罗斯方块", "description": "单人下落方块"},
                {"id": "star-catcher", "name": "接星星", "description": "接住落下的星星", "source": "user"},
            ], "errors": ["「broken」：缺少 manifest.json"]}))
        }
        fn game_open(&self, id: Option<&str>) -> Result<(), String> {
            self.log(format!("game-open {}", id.unwrap_or("lobby")))
        }
        fn game_close(&self) -> Result<(), String> {
            self.log("game-close".to_string())
        }
        fn create_game(&self, id: &str, _name: &str, _description: &str, files: &[(String, String)], save: bool, overwrite: bool) -> Result<Value, String> {
            self.log(format!("create-game {id} files={} save={save} overwrite={overwrite}", files.len()))?;
            Ok(json!({"id": id, "replaced": overwrite}))
        }
        fn delete_game(&self, id: &str) -> Result<(), String> {
            self.log(format!("delete-game {id}"))
        }
        fn request_permissions(&self) -> Result<Value, String> {
            self.log("request-permissions".to_string())?;
            Ok(json!({"ok": true, "message": "已申请系统权限"}))
        }
    }

    fn call(fake: &Arc<Fake>, name: &str, args: Value) -> Result<String, String> {
        let desktop: Arc<dyn Desktop> = fake.clone();
        let all = tools(desktop);
        let found = all.iter().find(|t| t.spec().name == name).unwrap_or_else(|| panic!("no tool {name}"));
        found.execute(args, &ToolCtx::detached("s1"))
    }

    #[test]
    fn every_tool_has_a_unique_valid_name_a_schema_and_the_right_risk() {
        let fake: Arc<dyn Desktop> = Arc::new(Fake::default());
        let all = tools(fake);
        let mut names: Vec<String> = all.iter().map(|t| t.spec().name.clone()).collect();
        names.sort();
        let mut unique = names.clone();
        unique.dedup();
        assert_eq!(names, unique);
        assert_eq!(names.len(), 26);
        for tool in &all {
            let spec = tool.spec();
            assert!(spec.name.len() <= 64 && spec.name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'), "{}", spec.name);
            assert_eq!(spec.schema["type"], "object", "{}", spec.name);
            assert!(!spec.description.is_empty());
        }
        let read: Vec<&str> = all.iter().filter(|t| t.spec().risk == Risk::Read).map(|t| t.spec().name.as_str()).collect();
        let mut read = read;
        read.sort();
        assert_eq!(read, vec!["care_list", "desktop_status", "effect_list", "effect_read_format", "game_list", "wallpaper_list", "widget_list"]);
        for tool in all.iter().filter(|t| t.spec().risk == Risk::Write) {
            assert!(!tool.spec().parallel_safe, "{} changes shared settings, so it runs alone", tool.spec().name);
        }
    }

    #[test]
    fn listings_are_compact_and_mark_what_is_in_use() {
        let fake = Arc::new(Fake::default());
        let out = call(&fake, "wallpaper_list", json!({})).unwrap();
        assert_eq!(out, "当前使用：w1\nw1 | 海边 | image | 已下载 | 正在使用\nw2 | 森林 | video | 未下载，应用时会先下载");
        let out = call(&fake, "effect_list", json!({})).unwrap();
        assert_eq!(out, "当前特效：rain\nrain | 细雨 | 正在使用\nsnow | 飘雪");
        assert!(call(&fake, "desktop_status", json!({})).unwrap().contains("\"wallpaper\":\"w1\""));
    }

    #[test]
    fn effect_format_is_embedded_complete_and_read_only() {
        let fake = Arc::new(Fake::default());
        let out = call(&fake, "effect_read_format", json!({})).unwrap();
        assert!(out.contains(crate::effect_packs::FORMAT_GUIDE));
        assert!(out.contains(crate::effect_packs::EXAMPLE));
        assert!(out.contains("## 粒子层") && out.contains("## 形状"));
        let example: Value = serde_json::from_str(crate::effect_packs::EXAMPLE).unwrap();
        crate::effect_packs::validate(&example).unwrap();
        assert!(fake.calls().is_empty(), "reading documentation never touches desktop state");
        let all = tools(fake);
        assert!(all.iter().find(|tool| tool.spec().name == "effect_create").unwrap().spec().description.contains("effect_read_format"));
    }

    #[test]
    fn changing_the_wallpaper_and_effect_goes_through_the_desktop() {
        let fake = Arc::new(Fake::default());
        assert_eq!(call(&fake, "wallpaper_apply", json!({"id": "  w2 "})).unwrap(), "已应用壁纸 w2");
        assert_eq!(call(&fake, "wallpaper_apply", json!({"id": "companion"})).unwrap(), "已换回系统桌面");
        assert_eq!(call(&fake, "effect_set", json!({"id": "snow"})).unwrap(), "已切换到特效 snow");
        assert_eq!(call(&fake, "effect_set", json!({"id": "none"})).unwrap(), "已关闭桌面特效");
        assert_eq!(fake.calls(), vec!["apply w2", "apply companion", "effect \"snow\"", "effect \"\""]);
        for args in [json!({}), json!({"id": ""}), json!({"id": "   "}), json!({"id": 5})] {
            assert!(call(&fake, "wallpaper_apply", args).unwrap_err().contains("`id`"));
        }
        assert_eq!(fake.calls().len(), 4, "invalid input never reaches the desktop");
    }

    #[test]
    fn creating_an_effect_accepts_an_object_or_a_json_string_and_says_where_it_landed() {
        let fake = Arc::new(Fake::default());
        let pack = json!({"schemaVersion": 1, "id": "glitter", "name": "星尘", "layers": [{"count": 5, "shape": {"kind": "circle"}, "colors": ["#fff"]}]});
        let expected = "已创建特效「星尘」（id: glitter，文件 glitter.json），可在“桌面特效”页选用。";
        assert_eq!(call(&fake, "effect_create", json!({"pack": pack})).unwrap(), expected);
        assert_eq!(call(&fake, "effect_create", json!({"pack": pack.to_string()})).unwrap(), expected, "a JSON string works too");
        assert_eq!(call(&fake, "effect_create", json!({"pack": pack, "overwrite": true})).unwrap(), expected);
        assert_eq!(
            fake.calls(),
            vec!["create-effect glitter overwrite=false", "create-effect glitter overwrite=false", "create-effect glitter overwrite=true"]
        );

        for (args, fragment) in [
            (json!({}), "`pack`"),
            (json!({"pack": 5}), "JSON 对象"),
            (json!({"pack": "{ not json"}), "不是有效的 JSON"),
            (json!({"pack": pack, "overwrite": "yes"}), "true or false"),
        ] {
            let error = call(&fake, "effect_create", args.clone()).unwrap_err();
            assert!(error.contains(fragment), "{args}: {error}");
        }
        assert_eq!(fake.calls().len(), 3, "invalid input never reaches the desktop");
    }

    #[test]
    fn the_effect_pack_is_refused_by_the_host_with_its_reason() {
        let fake = Arc::new(Fake::default());
        *fake.fail.lock().unwrap() = Some("id「rain」是内置特效的名字，请换一个".into());
        let pack = json!({"schemaVersion": 1, "id": "rain", "name": "冒名", "layers": []});
        assert_eq!(call(&fake, "effect_create", json!({"pack": pack})).unwrap_err(), "id「rain」是内置特效的名字，请换一个");
    }

    #[test]
    fn desktop_failures_come_back_as_the_tool_error() {
        let fake = Arc::new(Fake::default());
        *fake.fail.lock().unwrap() = Some("请先退出游戏空间".into());
        assert_eq!(call(&fake, "wallpaper_apply", json!({"id": "w1"})).unwrap_err(), "请先退出游戏空间");
    }

    #[test]
    fn the_companion_accepts_only_known_expressions_and_shells() {
        let fake = Arc::new(Fake::default());
        let out = call(&fake, "companion_set", json!({"expression": "happy", "skin": "bunny", "enabled": true})).unwrap();
        assert_eq!(out, "已设置：表情 happy，外壳 bunny，显示小伴");
        assert_eq!(call(&fake, "companion_set", json!({"enabled": false})).unwrap(), "已设置：隐藏小伴");
        assert!(call(&fake, "companion_set", json!({"expression": "furious"})).unwrap_err().contains("可用："));
        assert!(call(&fake, "companion_set", json!({"skin": "dragon"})).unwrap_err().contains("cat"));
        assert!(call(&fake, "companion_set", json!({})).unwrap_err().contains("至少"));
        assert!(call(&fake, "companion_set", json!({"enabled": "yes"})).unwrap_err().contains("true or false"));
        assert_eq!(fake.calls().len(), 2);
    }

    #[test]
    fn a_new_reminder_gets_defaults_and_only_the_fields_given() {
        let fake = Arc::new(Fake::default());
        let out = call(&fake, "care_set", json!({"text": "喝水", "time": "15:30"})).unwrap();
        assert_eq!(out, "已添加提醒：喝水");
        let out = call(&fake, "care_set", json!({"text": "开会", "repeat": "weekdays", "weekdays": [1, 3, 5], "time": "09:00"})).unwrap();
        assert_eq!(out, "已添加提醒：开会");
        assert_eq!(fake.calls(), vec!["upsert  custom daily \"15:30\" [] 喝水 true", "upsert  custom weekdays \"09:00\" [1, 3, 5] 开会 true"]);
    }

    #[test]
    fn editing_starts_from_the_saved_reminder_and_unknown_ids_are_refused() {
        let fake = Arc::new(Fake::default());
        let saved = Rule { id: "care-1".into(), text: "喝水".into(), time: "15:00".into(), revision: 4, ..Default::default() };
        fake.rules.lock().unwrap().push(serde_json::to_value(&saved).unwrap());
        let out = call(&fake, "care_set", json!({"id": "care-1", "time": "16:00", "enabled": false})).unwrap();
        assert_eq!(out, "已更新提醒：喝水");
        assert_eq!(fake.calls(), vec!["upsert care-1 custom daily \"16:00\" [] 喝水 false"]);
        assert!(call(&fake, "care_set", json!({"id": "care-404", "text": "x"})).unwrap_err().contains("care_list"));
    }

    #[test]
    fn bad_reminder_fields_are_explained_before_anything_is_saved() {
        let fake = Arc::new(Fake::default());
        for (args, fragment) in [
            (json!({"text": "x", "kind": "weather"}), "天气提醒"),
            (json!({"text": "x", "kind": "party"}), "kind 只能是"),
            (json!({"text": "x", "repeat": "hourly"}), "repeat 只能是"),
            (json!({"text": "x", "weekdays": [0]}), "1 到 7"),
            (json!({"text": "x", "weekdays": [8]}), "1 到 7"),
            (json!({"text": "x", "weekdays": "mon"}), "数组"),
            (json!({"text": "x", "intervalMinutes": -5}), "intervalMinutes"),
            (json!({"text": "x", "enabled": "no"}), "true or false"),
            (json!({"text": 5}), "must be a string"),
        ] {
            let error = call(&fake, "care_set", args.clone()).unwrap_err();
            assert!(error.contains(fragment), "{args}: {error}");
        }
        assert!(fake.calls().is_empty());
    }

    #[test]
    fn reminders_are_listed_in_plain_words_and_deleted_by_id() {
        let fake = Arc::new(Fake::default());
        assert_eq!(call(&fake, "care_list", json!({})).unwrap(), "关怀提醒总开关：开，共 0 条规则");
        let rules = [
            Rule { id: "a".into(), text: "喝水".into(), time: "15:00".into(), ..Default::default() },
            Rule { id: "b".into(), text: "开会".into(), repeat: "weekdays".into(), weekdays: vec![1, 5], time: "09:00".into(), enabled: false, ..Default::default() },
            Rule { id: "c".into(), text: "起来走走".into(), kind: "rest".into(), interval_minutes: 45, ..Default::default() },
            Rule { id: "d".into(), text: "生日".into(), repeat: "once".into(), once_at: "2030-01-02T08:00".into(), ..Default::default() },
        ];
        *fake.rules.lock().unwrap() = rules.iter().map(|r| serde_json::to_value(r).unwrap()).collect();
        let out = call(&fake, "care_list", json!({})).unwrap();
        assert_eq!(
            out,
            "关怀提醒总开关：开，共 4 条规则\na | 每天 15:00 | 喝水\nb | 周一、周五 09:00 | 开会 | 已停用\nc | 每 45 分钟 | 起来走走\nd | 一次 2030-01-02T08:00 | 生日"
        );
        assert_eq!(call(&fake, "care_delete", json!({"id": "b"})).unwrap(), "已删除提醒 b");
        assert!(call(&fake, "care_delete", json!({})).is_err());
        assert_eq!(fake.calls(), vec!["delete b"]);
    }
    #[test]
    fn widget_list_formats_compactly_with_styles_and_enabled_status() {
        let fake = Arc::new(Fake::default());
        let out = call(&fake, "widget_list", json!({})).unwrap();
        assert!(out.contains("桌面当前放置了 2 个组件"));
        assert!(out.contains("wgt-1 | 木鱼求签 (fortune) | 纸张风格 (paper) | 缩放 1.00 | 显示中"));
        assert!(out.contains("wgt-2 | 天气时钟 (weather) | 毛玻璃风格 (glass) | 缩放 1.20 | 城市: 上海 | 已隐藏"));
    }

    #[test]
    fn widget_add_validates_kind_and_theme() {
        let fake = Arc::new(Fake::default());
        let out = call(&fake, "widget_add", json!({"kind": "fortune", "theme": "paper"})).unwrap();
        assert_eq!(out, "已在桌面添加组件：木鱼求签（id: wgt-new）");
        assert_eq!(fake.calls(), vec!["widget-add fortune", r#"widget-update wgt-new {"theme":"paper"}"#]);

        assert!(call(&fake, "widget_add", json!({"kind": "unknown"})).unwrap_err().contains("kind 只能是"));
        assert!(call(&fake, "widget_add", json!({"kind": "fortune", "theme": "neon"})).unwrap_err().contains("theme 必须是"));
        assert_eq!(fake.calls().len(), 2, "invalid options never create a widget");
    }

    #[test]
    fn widget_tools_match_the_builtin_types_city_schema_and_custom_library() {
        let fake = Arc::new(Fake::default());
        for (kind, label) in [("notes", "便签待办"), ("countdown", "倒数日／纪念日")] {
            assert!(call(&fake, "widget_add", json!({"kind":kind})).unwrap().contains(label));
        }
        *fake.templates.lock().unwrap() = vec![json!({"id":"my-clock","name":"专注时钟"})];
        assert!(call(&fake, "widget_list", json!({})).unwrap().contains("my-clock | 专注时钟"));
        let all = tools(fake.clone());
        let add = all.iter().find(|tool| tool.spec().name == "widget_add").unwrap().spec();
        assert_eq!(add.schema["properties"]["kind"]["enum"], json!(crate::custom_widgets::BUILTIN_KINDS));
        let update = all.iter().find(|tool| tool.spec().name == "widget_update").unwrap().spec();
        assert_eq!(update.schema["properties"]["city"]["required"], json!(["name", "latitude", "longitude"]));
        assert!(parse_widget_patch(&json!({"city":{"name":"上海"}})).is_err());
        assert!(parse_widget_patch(&json!({"city":{"name":"上海","latitude":31.23,"longitude":121.47}})).is_ok());
    }

    #[test]
    fn widget_add_reports_a_failed_theme_update_instead_of_claiming_success() {
        let fake = Arc::new(Fake::default());
        *fake.fail_widget_update.lock().unwrap() = Some("写入失败".into());
        let error = call(&fake, "widget_add", json!({"kind":"notes","theme":"paper"})).unwrap_err();
        assert!(error.contains("组件已添加") && error.contains("设置外观失败") && error.contains("写入失败"));
    }

    #[test]
    fn widget_update_validates_ranges_and_updates_widget() {
        let fake = Arc::new(Fake::default());
        let out = call(&fake, "widget_update", json!({"id": "wgt-1", "scale": 1.2, "enabled": false})).unwrap();
        assert_eq!(out, "已更新桌面组件 wgt-1");
        assert_eq!(fake.calls(), vec![r#"widget-update wgt-1 {"enabled":false,"scale":1.2}"#]);

        assert!(call(&fake, "widget_update", json!({"id": "wgt-1", "scale": 0.5})).unwrap_err().contains("缩放范围"));
        assert!(call(&fake, "widget_update", json!({"id": "wgt-1", "x": 1.5})).unwrap_err().contains("x 坐标范围"));
        assert!(call(&fake, "widget_update", json!({"id": "wgt-1"})).unwrap_err().contains("至少指定"));
    }

    #[test]
    fn widget_remove_removes_by_id() {
        let fake = Arc::new(Fake::default());
        let out = call(&fake, "widget_remove", json!({"id": "wgt-1"})).unwrap();
        assert_eq!(out, "已从桌面移除组件 wgt-1");
        assert_eq!(fake.calls(), vec!["widget-remove wgt-1"]);
        assert!(call(&fake, "widget_remove", json!({})).is_err());
    }

    #[test]
    fn game_tools_resolve_names_list_games_open_and_close() {
        assert_eq!(resolve_game_id("snake-arena"), Some("snake-arena"));
        assert_eq!(resolve_game_id("贪吃蛇"), Some("snake-arena"));
        assert_eq!(resolve_game_id("小伴贪吃蛇"), Some("snake-arena"));
        assert_eq!(resolve_game_id("2048"), Some("merge-2048"));
        assert_eq!(resolve_game_id("俄罗斯方块"), Some("falling-blocks"));
        assert_eq!(resolve_game_id("unknown"), None);

        let fake = Arc::new(Fake::default());
        let list_out = call(&fake, "game_list", json!({})).unwrap();
        assert!(list_out.contains("小伴内置 7 款独立桌面小游戏"));
        assert!(list_out.contains("小伴贪吃蛇 (snake-arena)"));

        let open_out = call(&fake, "game_open", json!({"id": "贪吃蛇"})).unwrap();
        assert_eq!(open_out, "已进入游戏空间并启动游戏：snake-arena");
        let lobby_out = call(&fake, "game_open", json!({})).unwrap();
        assert_eq!(lobby_out, "已打开小伴游戏空间大厅");
        let close_out = call(&fake, "game_close", json!({})).unwrap();
        assert_eq!(close_out, "已退出游戏空间，返回桌面");

        assert_eq!(fake.calls(), vec!["game-open snake-arena", "game-open lobby", "game-close"]);
        assert!(call(&fake, "game_open", json!({"id": "不存在的游戏"})).unwrap_err().contains("没有找到游戏"));
    }

    #[test]
    fn game_list_marks_user_games_and_game_open_matches_them_by_exact_id() {
        let fake = Arc::new(Fake::default());
        let out = call(&fake, "game_list", json!({})).unwrap();
        assert!(out.contains("- 接星星 (star-catcher): 接住落下的星星 [自创]"), "{out}");
        assert!(out.contains("- 小伴贪吃蛇 (snake-arena): 吃糖果长成大蛇\n"), "built-in games are not marked");
        assert!(out.contains("无法加载的自创游戏文件夹：「broken」"));
        assert_eq!(call(&fake, "game_open", json!({"id": "star-catcher"})).unwrap(), "已进入游戏空间并启动游戏：star-catcher");
        assert!(call(&fake, "game_open", json!({"id": "接星星"})).unwrap_err().contains("star-catcher"));
        assert!(call(&fake, "game_open", json!({"id": "STAR-CATCHER"})).is_err(), "user ids match exactly");
        assert_eq!(fake.calls(), vec!["game-open star-catcher"]);
    }

    fn game_args() -> Value {
        json!({"id": "star-catcher", "name": "接星星", "description": "接住落下的星星",
            "files": [{"path": "index.html", "content": "<html><head></head></html>"}, {"path": "js/main.js", "content": "1"}]})
    }

    #[test]
    fn game_create_checks_the_request_then_saves_without_opening() {
        let fake = Arc::new(Fake::default());
        let out = call(&fake, "game_create", game_args()).unwrap();
        assert!(out.contains("star-catcher") && out.contains("共 3 个文件") && out.contains("game_open(id: \"star-catcher\")"), "{out}");
        let mut args = game_args();
        args["save"] = json!(true);
        args["overwrite"] = json!(true);
        let out = call(&fake, "game_create", args).unwrap();
        assert!(out.contains("已覆盖") && out.contains("本机存档"));
        assert_eq!(fake.calls(), vec!["create-game star-catcher files=2 save=false overwrite=false", "create-game star-catcher files=2 save=true overwrite=true"]);

        let calls_before = fake.calls().len();
        let bad = |edit: &dyn Fn(&mut Value)| {
            let mut args = game_args();
            edit(&mut args);
            call(&fake, "game_create", args)
        };
        assert!(bad(&|a| a["id"] = json!("snake-arena")).unwrap_err().contains("内置"));
        assert!(bad(&|a| a["id"] = json!("Bad Id")).is_err());
        assert!(bad(&|a| a["files"] = json!([{"path": "game.js", "content": "x"}])).unwrap_err().contains("index.html"));
        assert!(bad(&|a| a["files"].as_array_mut().unwrap().push(json!({"path": "manifest.json", "content": "{}"}))).unwrap_err().contains("manifest.json"));
        assert!(bad(&|a| a["files"].as_array_mut().unwrap().push(json!({"path": "../evil.js", "content": "x"}))).is_err());
        assert!(bad(&|a| a["files"].as_array_mut().unwrap().push(json!({"path": "logo.png", "content": "x"}))).unwrap_err().contains("类型"));
        assert!(bad(&|a| a["files"].as_array_mut().unwrap().push(json!({"path": "big.js", "content": "x".repeat(1024 * 1024 + 1)}))).unwrap_err().contains("1 MB"));
        assert!(bad(&|a| a["files"] = json!("nope")).is_err());
        assert!(bad(&|a| a["files"] = json!([{"path": "index.html"}])).is_err());
        assert!(bad(&|a| a["name"] = json!("字".repeat(25))).is_err());
        assert!(bad(&|a| a["save"] = json!("yes")).is_err());
        assert_eq!(fake.calls().len(), calls_before, "refused requests never reach the desktop");
        let spec_text = tools(fake.clone()).iter().find(|t| t.spec().name == "game_create").unwrap().spec().description.clone();
        for needle in ["host.init", "game.ready", "game.save", "32 KiB", "xiaban.game-pack", "hostNonce", "ES modules", "desktop-host.js"] {
            assert!(spec_text.contains(needle), "description mentions {needle}");
        }
    }

    #[test]
    fn game_delete_only_removes_user_games() {
        let fake = Arc::new(Fake::default());
        assert!(call(&fake, "game_delete", json!({"id": "ink-blade"})).unwrap_err().contains("内置"));
        assert!(call(&fake, "game_delete", json!({})).is_err());
        assert!(call(&fake, "game_delete", json!({"id": "star-catcher"})).unwrap().contains("已删除"));
        assert_eq!(fake.calls(), vec!["delete-game star-catcher"]);
    }

    #[test]
    fn widget_create_validates_and_calls_desktop() {
        let fake = Arc::new(Fake::default());
        let res = call(
            &fake,
            "widget_create",
            json!({
                "id": "pomodoro-timer",
                "name": "番茄钟",
                "html": "<div id='app'>25:00</div>",
                "width": 260,
                "height": 200,
                "theme": "glass"
            }),
        )
        .unwrap();
        assert!(res.contains("已成功创建小组件「番茄钟」（id: pomodoro-timer）"));
        assert!(res.contains("并已直接添加到桌面上展示"));
        assert_eq!(fake.calls(), vec!["create-custom-widget pomodoro-timer place_now=true overwrite=false"]);

        let res_no_place = call(
            &fake,
            "widget_create",
            json!({
                "id": "poetry-daily",
                "name": "每日诗词",
                "html": "<div>床前明月光</div>",
                "place_now": false
            }),
        )
        .unwrap();
        assert!(res_no_place.contains("未自动放置到桌面"));
        assert_eq!(fake.calls()[1], "create-custom-widget poetry-daily place_now=false overwrite=false");
    }

    #[test]
    fn wallpaper_create_validates_and_calls_desktop() {
        let fake = Arc::new(Fake::default());
        let res = call(
            &fake,
            "wallpaper_create",
            json!({
                "title": "水墨涟漪交互壁纸",
                "html": "<!DOCTYPE html><html><body><canvas></canvas></body></html>",
                "apply_now": true
            }),
        )
        .unwrap();
        assert!(res.contains("已成功创作动态网页壁纸「水墨涟漪交互壁纸」"));
        assert!(res.contains("并已立即应用为当前桌面壁纸"));
        assert_eq!(fake.calls(), vec!["create-wallpaper 水墨涟漪交互壁纸 apply_now=true"]);

        let res_no_apply = call(
            &fake,
            "wallpaper_create",
            json!({
                "title": "星空桌面",
                "html": "<html><body>stars</body></html>",
                "apply_now": false
            }),
        )
        .unwrap();
        assert!(res_no_apply.contains("可在壁纸库随时选用"));
        assert_eq!(fake.calls()[1], "create-wallpaper 星空桌面 apply_now=false");
    }

    #[test]
    fn deletion_tools_validate_and_call_desktop() {
        let fake = Arc::new(Fake::default());
        let res_wall = call(&fake, "wallpaper_delete", json!({"id": "local-123"})).unwrap();
        assert!(res_wall.contains("已删除本地壁纸 local-123"));
        assert_eq!(fake.calls()[0], "delete-wallpaper local-123");

        let res_eff = call(&fake, "effect_delete", json!({"id": "my-effect"})).unwrap();
        assert!(res_eff.contains("已删除自定义特效 my-effect"));
        assert_eq!(fake.calls()[1], "delete-effect my-effect");

        let res_wgt = call(&fake, "widget_delete_custom", json!({"id": "my-clock"})).unwrap();
        assert!(res_wgt.contains("已删除自定义小组件「my-clock」模板"));
        assert_eq!(fake.calls()[2], "delete-custom-widget my-clock");

        assert!(call(&fake, "wallpaper_delete", json!({})).is_err());
        assert!(call(&fake, "effect_delete", json!({})).is_err());
        assert!(call(&fake, "widget_delete_custom", json!({})).is_err());
    }
}

