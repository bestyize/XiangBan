//! Tools that let the assistant run the desktop app itself: wallpapers, effects, the companion
//! and care reminders. They go through the same functions the settings window uses, so every
//! rule those enforce (valid values, locks, saving) applies here too.
//!
//! The tools talk to a `Desktop`, so they can be tested without a running app.

use super::community;
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
    fn design_removed(&self,_id:&str)->Result<(),String>{Ok(())}
    fn design_saved(&self, _project: &open_claude_design::Project) -> Result<(), String> { Ok(()) }
    fn connector_search(&self, _keyword: &str) -> Result<Value, String> { Err("当前宿主不支持连接器目录".into()) }
    fn connector_request(&self, _references: &[String], _ctx: &ToolCtx) -> Result<Value, String> { Err("当前宿主不支持连接器授权".into()) }
    fn connector_call(&self, _name: &str, _args: Value, _ctx: &ToolCtx) -> agent_kernel::ToolResult { agent_kernel::ToolResult::error("当前宿主不支持连接器调用") }
    fn connector_target(&self, _name: &str) -> Result<Option<Arc<dyn Tool>>, String> { Ok(None) }
    fn pet_catalog(&self)->Result<Value,String>{Err("当前宿主不支持宠物库".into())}
    fn pet_save_theme(&self,_theme:crate::pet_packs::Theme,_overwrite:bool)->Result<Value,String>{Err("当前宿主不支持宠物创作".into())}
    fn pet_delete_theme(&self,_id:String)->Result<Value,String>{Err("当前宿主不支持宠物主题".into())}
    fn pet_add(&self,_theme:String,_pet:String,_layer:Option<String>)->Result<Value,String>{Err("当前宿主不支持宠物实例".into())}
    fn pet_update(&self,_id:&str,_patch:crate::pets::Patch)->Result<Value,String>{Err("当前宿主不支持宠物实例".into())}
    fn pet_remove(&self,_id:&str)->Result<Value,String>{Err("当前宿主不支持宠物实例".into())}
    fn status(&self) -> Result<Value, String>;
    fn windows(&self) -> Result<Value, String> { Err("当前宿主不支持窗口查询".into()) }
    fn window_update(&self, _change: &crate::window_control::Change) -> Result<Value, String> { Err("当前宿主不支持窗口控制".into()) }
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
    fn widget_add(&self, kind: &str, custom_id: Option<&str>) -> Result<Value, String>;
    fn widget_update(&self, id: &str, patch: Value) -> Result<Value, String>;
    fn widget_remove(&self, id: &str) -> Result<(), String>;
    fn custom_widgets(&self) -> Result<Value, String>;
    fn create_custom_widget(&self, pack:&Value, place_now:bool, overwrite:bool)->Result<Value,String>;
    fn delete_custom_widget(&self, id: &str) -> Result<(), String>;
    fn games(&self) -> Result<Value, String>;
    fn game_open(&self, id: Option<&str>) -> Result<(), String>;
    fn game_close(&self) -> Result<(), String>;
    /// Writes a user game folder from text files. Returns `{ id, name, directory, files, save, replaced }`.
    fn create_game(&self, id: &str, name: &str, description: &str, files: &[(String, String)], save: bool, overwrite: bool) -> Result<Value, String>;
    /// Deletes a user game, closing the game space first when it is open.
    fn delete_game(&self, id: &str) -> Result<(), String>;
    fn request_permissions(&self) -> Result<Value, String>;
    fn computer_use_diagnose(&self) -> Result<Value, String> { Err("当前宿主不支持电脑控制诊断".into()) }
    fn computer_use_runtime_root(&self) -> Result<std::path::PathBuf, String> { Err("当前宿主不支持电脑控制资源".into()) }
    fn computer_use_repair_permissions(&self, _permission: &str) -> Result<Value, String> { Err("当前宿主不支持电脑控制权限修复".into()) }
    /// Saves one of the user's own works as a `.zip` resource pack through the save dialog. `None` when they cancel; otherwise the file path and its size.
    fn export_pack(&self, kind: &str, id: &str) -> Result<Option<(String, u64)>, String>;
    /// Opens an https address in the system browser.
    fn open_url(&self, url: &str) -> Result<(), String>;
    /// Light/dark/system look, theme colour and pausing the animation; returns the values now in effect.
    fn set_display(&self, _appearance: Option<&str>, _accent: Option<&str>, _paused: Option<bool>) -> Result<Value, String> { Err("当前宿主不支持显示设置".into()) }
    /// The product catalog's skill search. `Ok(None)` when the website has no such route, so the search goes to SkillHub.
    fn skill_catalog(&self, _keyword: &str, _page: u32, _sort_by: &str) -> Result<Option<Value>, String> { Ok(None) }
    /// Brings the console forward on `page` (and its `tab`). `page`/`tab` are already validated by `app_guide`.
    fn open_page(&self, _page: &str, _tab: Option<&str>) -> Result<(), String> { Err("当前宿主不支持打开页面".into()) }
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
    fn connector_target(&self, name: &str) -> Result<Option<Arc<dyn Tool>>, String> {
        super::connector_manager(&self.app)?.tools().into_iter().find(|tool| tool.spec().name == name)
            .map(Some).ok_or_else(|| "连接器工具当前不可用，请检查连接状态".into())
    }
    fn design_removed(&self,id:&str)->Result<(),String>{crate::design::removed(&self.app,id)}
    fn design_saved(&self, project: &open_claude_design::Project) -> Result<(), String> { crate::design::saved(&self.app, project) }
    fn connector_search(&self, keyword: &str) -> Result<Value, String> { super::connectors::search(&self.app, keyword) }
    fn connector_request(&self, references: &[String], ctx: &ToolCtx) -> Result<Value, String> { super::connectors::request(&self.app, references, ctx) }
    fn connector_call(&self, name: &str, args: Value, ctx: &ToolCtx) -> agent_kernel::ToolResult {
        match super::connector_manager(&self.app) {
            Ok(manager) => manager.call_authorized(name, args, ctx, &self.app.state::<super::ChatState>().permission),
            Err(error) => agent_kernel::ToolResult::error(error),
        }
    }
    fn pet_catalog(&self)->Result<Value,String>{crate::pets::catalog(&self.app)}
    fn pet_save_theme(&self,mut theme:crate::pet_packs::Theme,overwrite:bool)->Result<Value,String>{crate::pet_packs::resolve_image_refs(&mut theme,&crate::pets::themes(&self.app)?)?;crate::pets::save_theme(&self.app,theme,overwrite)}
    fn pet_delete_theme(&self,id:String)->Result<Value,String>{crate::pets::delete_pet_theme(self.app.clone(),id)}
    fn pet_add(&self,theme:String,pet:String,layer:Option<String>)->Result<Value,String>{let result=crate::pets::add(&self.app,theme,pet,layer)?;if !crate::settings(&self.app).companion_enabled{tauri::async_runtime::block_on(crate::set_companion_enabled(self.app.clone(),true))?;}Ok(result)}
    fn pet_update(&self,id:&str,patch:crate::pets::Patch)->Result<Value,String>{crate::pets::update(&self.app,id,patch)}
    fn pet_remove(&self,id:&str)->Result<Value,String>{crate::pets::remove(&self.app,id)}
    fn windows(&self) -> Result<Value, String> { crate::window_control::snapshot(&self.app) }
    fn window_update(&self, change: &crate::window_control::Change) -> Result<Value, String> { crate::window_control::update(&self.app, change) }

    fn export_pack(&self, kind: &str, id: &str) -> Result<Option<(String, u64)>, String> {
        let saved = crate::resource_packs::export_with_dialog(&self.app, kind, id)?;
        Ok(saved.map(|(path, bytes)| (path.display().to_string(), bytes)))
    }

    fn open_url(&self, url: &str) -> Result<(), String> {
        crate::content::open_download_url(url.to_string())
    }

    fn set_display(&self, appearance: Option<&str>, accent: Option<&str>, paused: Option<bool>) -> Result<Value, String> {
        if let Some(mode) = appearance { crate::set_appearance(self.app.clone(), mode.to_string())?; }
        if let Some(accent) = accent { crate::set_accent(self.app.clone(), accent.to_string())?; }
        if let Some(paused) = paused {
            let s = crate::settings(&self.app);
            crate::update_settings(self.app.clone(), s.expression, s.size, s.theme, paused, s.fps)?;
        }
        let s = crate::settings(&self.app);
        Ok(json!({ "appearance": s.appearance, "accent": s.accent, "animationPaused": s.paused }))
    }

    fn skill_catalog(&self, keyword: &str, page: u32, sort_by: &str) -> Result<Option<Value>, String> {
        tauri::async_runtime::block_on(super::store::search_skills(&self.app, keyword, page, sort_by))
    }

    fn open_page(&self, page: &str, tab: Option<&str>) -> Result<(), String> {
        use tauri::Emitter;
        crate::present_control(&self.app)?;
        self.app.emit_to("control", "control-navigate", json!({ "page": page, "tab": tab })).map_err(|e| e.to_string())
    }

    fn request_permissions(&self) -> Result<Value, String> {
        crate::computer_permissions::request(&self.app)
    }
    fn computer_use_diagnose(&self) -> Result<Value, String> { crate::computer_permissions::diagnose(&self.app) }
    fn computer_use_runtime_root(&self) -> Result<std::path::PathBuf, String> { crate::computer_permissions::runtime_root(&self.app) }
    fn computer_use_repair_permissions(&self, permission: &str) -> Result<Value, String> { crate::computer_permissions::repair(&self.app, permission) }
    fn status(&self) -> Result<Value, String> {
        let s = crate::settings(&self.app);
        Ok(json!({
            "mode": s.mode, "wallpaper": s.wallpaper_id, "effect": s.effects.selected,
            "companionEnabled": s.companion_enabled, "companionSkin": s.companion_skin,
            "expression": s.expression, "animationPaused": s.paused, "appearance": s.appearance, "accent": s.accent,
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

    fn widget_add(&self, kind: &str, custom_id: Option<&str>) -> Result<Value, String> {
        let app = self.app.clone();
        let kind = kind.to_string();
        let custom_id = custom_id.map(str::to_string);
        tauri::async_runtime::block_on(crate::widgets::add_desktop_widget(app, kind, None, custom_id))
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

    fn custom_widgets(&self)->Result<Value,String>{
        let root=crate::storage::root(&self.app)?;
        Ok(json!(crate::custom_widgets::load_all(&root.join("custom_widgets"))))
    }

    fn create_custom_widget(&self,pack:&Value,place_now:bool,overwrite:bool)->Result<Value,String>{
        let root=crate::storage::root(&self.app)?;
        let dir=root.join("custom_widgets");
        let path=crate::custom_widgets::save(&dir,pack,overwrite)?;
        let id=pack["id"].as_str().unwrap_or_default().to_string();
        if place_now{
            let kind="custom".to_string();
            let app=self.app.clone();
            let custom=id.clone();
            tauri::async_runtime::block_on(crate::widgets::add_desktop_widget(app,kind,None,Some(custom)))?;
        }
        crate::widgets::sync(&self.app);
        let name=pack["name"].as_str().unwrap_or_default();
        Ok(json!({"id":id,"name":name,"file":path.file_name().and_then(|n|n.to_str()).unwrap_or_default(),"placed":place_now}))
    }

    fn delete_custom_widget(&self,id:&str)->Result<(),String>{
        let root=crate::storage::root(&self.app)?;
        crate::custom_widgets::remove(&root.join("custom_widgets"),id)?;
        crate::widgets::drop_custom_cards(&self.app,id)?;
        Ok(())
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

    fn resource(&self, _: &Value) -> Result<Option<agent_kernel::ToolResource>, String> {
        Ok(if self.spec.risk == Risk::Read { None }
        else if matches!(self.spec.name.as_str(), "computer_use_repair_permissions" | "request_accessibility_permission") {
            Some(agent_kernel::ToolResource::RunExclusive("computer".into()))
        } else if matches!(self.spec.name.as_str(), "open_page" | "desktop_window_set" | "game_open" | "game_close") {
            // Focus/window changes must not interrupt another run's screen-control sequence.
            Some(agent_kernel::ToolResource::Named("computer".into()))
        } else { Some(agent_kernel::ToolResource::Named("app-state".into())) })
    }

    fn execute(&self, args: Value, ctx: &ToolCtx) -> Result<String, String> {
        if ctx.is_cancelled() { return Err("操作开始前已取消".into()); }
        (self.run)(self.desktop.as_ref(), &args)
    }
}

fn object(properties: Value, required: &[&str]) -> Value {
    json!({ "type": "object", "properties": properties, "required": required })
}

pub(super) fn text_arg<'a>(args: &'a Value, key: &str) -> Result<&'a str, String> {
    args.get(key).and_then(Value::as_str).map(str::trim).filter(|s| !s.is_empty()).ok_or_else(|| format!("`{key}` is required and must be a non-empty string"))
}

fn optional_text<'a>(args: &'a Value, key: &str) -> Result<Option<&'a str>, String> {
    match args.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => Ok(Some(s.trim())),
        Some(_) => Err(format!("`{key}` must be a string")),
    }
}

pub(super) fn optional_bool(args: &Value, key: &str) -> Result<Option<bool>, String> {
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
    let task = if rule["kind"] == "agent" { "定时任务：" } else { "" };
    format!("{id} | {when} | {task}{text}{state}")
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
        if !["custom", "off-work", "rest", "agent"].contains(&kind) {
            return Err("kind 只能是 custom（普通提醒）、off-work（下班提醒）、rest（定时休息）或 agent（定时任务）；天气提醒请让用户在界面里设置".into());
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

/// How a reminder will reach the user, from the rule and the care settings, for the assistant to tell them as it is.
fn how_it_reminds(rule: &Rule, config: &Value) -> String {
    if config["enabled"].as_bool() == Some(false) {
        return "注意：关怀提醒的总开关现在是关的，到点不会弹出，需要用户在「提醒」页打开".into();
    }
    let on = |key: &str| config[key].as_bool().unwrap_or(false);
    let once = rule.repeat == "once" && rule.kind == "custom";
    let sound = on("sound") || (once && config["onceSound"].as_bool().unwrap_or(true));
    let speaks = on("voice") || (once && config["onceVoice"].as_bool().unwrap_or(true));
    let when = if rule.kind == "rest" { format!("使用电脑每满 {} 分钟", rule.interval_minutes) } else {
        match rule.repeat.as_str() {
            "once" => rule.once_at.replace('T', " "),
            "weekdays" => format!("每周{}的 {}", rule.weekdays.iter().map(|d| ["一", "二", "三", "四", "五", "六", "日"].get(*d as usize - 1).copied().unwrap_or("?")).collect::<Vec<_>>().join("、"), rule.time),
            _ => format!("每天 {}", rule.time),
        }
    };
    if rule.kind == "agent" {
        return format!("{when}到点，小伴会在后台去做这件事，结果显示在提醒气泡里；小伴需要在运行");
    }
    let stays = if rule.repeat == "once" && rule.kind == "custom" {
        "一次性提醒的气泡会一直留着，直到点掉，或点「稍后提醒」（10 分钟后再弹）"
    } else {
        "气泡显示约 10 秒（开了语音播报是 30 秒），可点「稍后提醒」10 分钟后再弹"
    };
    let quiet = if on("quietEnabled") { format!("安静时段（{}–{}）、", config["quietStart"].as_str().unwrap_or("22:00"), config["quietEnd"].as_str().unwrap_or("08:00")) } else { String::new() };
    format!("{when}到点会在屏幕右上角弹出提醒气泡，{stays}；提示音{}、朗读提醒内容{}（可在「提醒」页调整）；小伴需要在运行，{quiet}屏幕锁定、专注模式、玩游戏时先压住，之后补弹",
        if sound { "已开" } else { "关" }, if speaks { "已开" } else { "关" })
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
    // The confirmation for scheduled tasks keys on `kind` in the call, so a change to one must say so.
    if existing.as_ref().is_some_and(|rule| rule.kind == "agent") && optional_text(args, "kind")? != Some("agent") {
        return Err("这是一条定时任务，修改时请带上 kind=agent".into());
    }
    let creating = existing.is_none();
    let rule = rule_from_args(args, existing)?;
    desktop.care_upsert(rule.clone())?;
    let how = how_it_reminds(&rule, &desktop.care().map(|v| v["config"].clone()).unwrap_or_default());
    Ok(format!("{}：{}。{how}", if creating { "已添加提醒" } else { "已更新提醒" }, rule.text))
}


fn widget_kind_label(kind: &str) -> &'static str {
    match kind {
        "clock" => "桌面时钟",
        "reminders" => "今日日程",
        "stocks" => "股票行情",
        "woodfish" => "电子木鱼",
        "calendar" => "小伴日历",
        "weather" => "天气时钟",
        "notes" => "待办事项",
        "countdown" => "倒数日／纪念日",
        "custom" => "自创组件",
        _ => "其他组件",
    }
}

fn widget_theme_label(theme: &str) -> &'static str {
    match theme {
        "ivory" => "暖白",
        "glass" => "毛玻璃",
        "paper" => "温润纸张",
        "transparent" => "透明无底",
        _ => "默认",
    }
}

fn widget_list(desktop: &dyn Desktop, _: &Value) -> Result<String, String> {
    let value = desktop.widgets()?;
    let widgets = value["widgets"].as_array().cloned().unwrap_or_default();
    let mut lines = if widgets.is_empty() {
        vec!["桌面当前没有放置任何组件。可用 widget_add 添加（支持 clock 时钟、weather 天气、notes 待办、reminders 日程、stocks 股票、woodfish 木鱼、calendar 月历、countdown 纪念日，或用 widget_create 创作自创组件）。".to_string()]
    } else { vec![format!("桌面当前放置了 {} 个组件：", widgets.len())] };
    for w in &widgets {
        let id = w["id"].as_str().unwrap_or("?");
        let kind = w["kind"].as_str().unwrap_or("");
        let theme = w["theme"].as_str().unwrap_or("");
        let scale = w["scale"].as_f64().unwrap_or(1.0);
        let enabled = w["enabled"].as_bool().unwrap_or(true);
        let city = w.get("city").and_then(|c| c.get("name")).and_then(Value::as_str).map(|n| format!(" | 城市: {n}")).unwrap_or_default();
        let state = if enabled { "显示中" } else { "已隐藏" };
        let layer = if w["layer"].as_str() == Some("topmost") { "始终置顶" } else { "贴桌面" };
        lines.push(format!("{id} | {} ({kind}) | {}风格 ({theme}) | 缩放 {scale:.2}{city} | {state} | {layer} | 位置 ({}, {}) | 显示器 {}", widget_kind_label(kind), widget_theme_label(theme), w["x"], w["y"], w["monitor"]));
        lines.extend(super::widget_content::summary(w));
    }
    let templates = desktop.custom_widgets()?;
    if let Some(templates) = templates.as_array().filter(|items| !items.is_empty()) {
        lines.push("自创组件模板（放到桌面用 widget_add(kind=\"custom\", custom_id=行首的 id)，删除模板用 widget_delete_custom）：".into());
        for template in templates {
            lines.push(format!("{} | {}", template["id"].as_str().unwrap_or("?"), template["name"].as_str().unwrap_or("自创组件")));
        }
    }
    Ok(lines.join("\n"))
}

fn widget_content_schema() -> Value {
    json!({"type": "object", "additionalProperties": true, "description": "组件内容，按类型：notes {title?, items: [文字 或 {text, done?, date?, time?, notify?, repeat?, id?}], append?}；countdown {title, date: YYYY-MM-DD, mode?: countdown|elapsed, repeat?: none|yearly}；reminders {items: [{title, at: YYYY-MM-DDTHH:MM, end?, location?, notify?, repeat?, done?, id?}], append?}；stocks {symbols: [1.600519 沪, 0.000001 深, 116.00700 港，≤8], symbol?}。items 默认整体替换，append=true 追加；改某条先 widget_list 看 id，替换时带上。"})
}

fn widget_add(desktop: &dyn Desktop, args: &Value) -> Result<String, String> {
    let kind = text_arg(args, "kind")?;
    if !crate::widgets::KINDS.contains(&kind) {
        return Err("kind 只能是 clock（时钟）、weather（天气）、notes（待办）、reminders（日程）、stocks（股票）、woodfish（木鱼）、calendar（月历）、countdown（纪念日）或 custom（自创组件，需同时给 custom_id）".into());
    }
    // A custom widget is placed from a saved template, named by `custom_id` (listed at the end of widget_list).
    let custom_id = optional_text(args, "custom_id")?.filter(|id| !id.is_empty());
    if kind == "custom" && custom_id.is_none() {
        return Err("添加自创组件必须给 custom_id：先用 widget_list 查看「自创组件模板」里每行开头的 id，再调用 widget_add(kind=\"custom\", custom_id=该 id)".into());
    }
    if kind != "custom" && custom_id.is_some() {
        return Err("custom_id 只用于 kind=custom；其他类型不需要它".into());
    }
    // Validate all options before creating anything, and never hide a failed theme update.
    let theme = optional_text(args, "theme")?;
    if theme.is_some_and(|theme| !["ivory"].contains(&theme)) {
        return Err("theme 必须是 ivory（设计稿材质）".into());
    }
    let layer = optional_text(args, "layer")?;
    if layer.is_some_and(|layer| !["desktop", "topmost"].contains(&layer)) {
        return Err("layer 只能是 desktop（贴在桌面）或 topmost（始终置顶）".into());
    }
    // A bad `content` is refused before the widget exists, checked against what a new widget of this kind holds.
    if let Some(content) = args.get("content").filter(|v| !v.is_null()) {
        super::widget_content::patch(&json!({ "kind": kind }), content)?;
    }
    let res = desktop.widget_add(kind, custom_id)?;
    let id = res["widgets"].as_array().and_then(|w| w.last()).and_then(|w| w.get("id")).and_then(Value::as_str).ok_or("组件创建结果没有返回 id")?;
    if let Some(theme) = theme {
        desktop.widget_update(id, json!({ "theme": theme })).map_err(|e| format!("组件已添加（id: {id}），但设置外观失败：{e}"))?;
    }
    if let Some(layer) = layer {
        desktop.widget_update(id, json!({ "layer": layer })).map_err(|e| format!("组件已添加（id: {id}），但设置层级失败：{e}"))?;
    }
    if let Some(content) = args.get("content").filter(|v| !v.is_null()) {
        let added = res["widgets"].as_array().and_then(|w| w.last()).cloned().unwrap_or_default();
        let patch = super::widget_content::patch(&added, content).map_err(|e| format!("组件已添加（id: {id}），但内容没有设置：{e}"))?;
        desktop.widget_update(id, patch).map_err(|e| format!("组件已添加（id: {id}），但内容没有设置：{e}"))?;
    }
    Ok(format!("已在桌面添加组件：{}（id: {id}）", widget_kind_label(kind)))
}

fn parse_widget_patch(args: &Value) -> Result<Value, String> {
    let obj = widget_patch_fields(args)?;
    if obj.is_empty() {
        return Err("请至少指定 theme、layer、scale、x、y、enabled、city 或 content 之一".into());
    }
    Ok(Value::Object(obj))
}

/// The fields of a `widget_update` call that become the layout patch; may be empty.
fn widget_patch_fields(args: &Value) -> Result<serde_json::Map<String, Value>, String> {
    let mut obj = serde_json::Map::new();
    if let Some(v) = args.get("theme").filter(|v| !v.is_null()) {
        let theme = v.as_str().ok_or("theme 必须是字符串")?;
        if !["ivory"].contains(&theme) {
            return Err("theme 必须是 ivory（设计稿材质）".into());
        }
        obj.insert("theme".into(), json!(theme));
    }
    if let Some(v) = args.get("layer").filter(|v| !v.is_null()) {
        let layer = v.as_str().ok_or("layer 必须是字符串")?;
        if !["desktop", "topmost"].contains(&layer) {
            return Err("layer 只能是 desktop（贴在桌面）或 topmost（始终置顶）".into());
        }
        obj.insert("layer".into(), json!(layer));
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
    Ok(obj)
}

fn widget_update(desktop: &dyn Desktop, args: &Value) -> Result<String, String> {
    let id = text_arg(args, "id")?;
    let mut fields = widget_patch_fields(args)?;
    if let Some(content) = args.get("content").filter(|v| !v.is_null()) {
        let widgets = desktop.widgets()?;
        let widget = widgets["widgets"].as_array().and_then(|all| all.iter().find(|w| w["id"].as_str() == Some(id))).ok_or_else(|| format!("没有 id 为 {id} 的组件，可先用 widget_list 查看"))?;
        if let Value::Object(content_fields) = super::widget_content::patch(widget, content)? { fields.extend(content_fields); }
    }
    if fields.is_empty() {
        return Err("请至少指定 theme、layer、scale、x、y、enabled、city 或 content 之一".into());
    }
    desktop.widget_update(id, Value::Object(fields))?;
    Ok(format!("已更新桌面组件 {id}"))
}

fn widget_remove(desktop: &dyn Desktop, args: &Value) -> Result<String, String> {
    let id = text_arg(args, "id")?;
    desktop.widget_remove(id)?;
    Ok(format!("已从桌面移除组件 {id}"))
}

/// Whether a CSS background value draws nothing.
fn clear_background(value: &str) -> bool {
    let value = value.trim().trim_end_matches("!important").trim().to_ascii_lowercase();
    if ["transparent", "none", "inherit", "initial", "unset", ""].contains(&value.as_str()) || value.starts_with("var(") {
        return true;
    }
    let alpha = |inner: &str| inner.rsplit([',', '/']).next().map(str::trim).is_some_and(|a| a.trim_end_matches('%').parse::<f64>().is_ok_and(|a| a == 0.0));
    ["rgba(", "hsla(", "rgb(", "hsl("].iter().any(|f| value.strip_prefix(f).and_then(|v| v.strip_suffix(')')).is_some_and(|inner| (inner.contains(',') && inner.split(',').count() == 4 || inner.contains('/')) && alpha(inner)))
}

/// The widget rules a page can be checked against before it is saved (see `WIDGET_FORMAT`). Only what is clearly
/// wrong: drag regions or Tauri the host already provides, and a background drawn on the whole page.
fn widget_html_problems(html: &str) -> Vec<String> {
    let lower = html.to_ascii_lowercase();
    let mut problems = Vec::new();
    if lower.contains("-webkit-app-region") {
        problems.push("不要使用 -webkit-app-region：宿主已经提供按住空白处拖动".to_string());
    }
    if lower.contains("__tauri") || lower.contains("@tauri-apps") {
        problems.push("不要直接调用 Tauri，组件在沙盒里运行".to_string());
    }
    let mut opaque = |what: &str, declarations: &str| {
        for declaration in declarations.split(';') {
            let Some((property, value)) = declaration.split_once(':') else { continue };
            let property = property.trim();
            if (property == "background" || property == "background-color") && !clear_background(value) {
                problems.push(format!("{what} 的背景是 `{}`：页面和主体要保持透明，需要底板时画在组件内容里", value.trim()));
                return;
            }
        }
    };
    let mut rest = lower.as_str();
    while let Some(start) = rest.find("<style") {
        let after = &rest[start..];
        let Some(open) = after.find('>') else { break };
        let end = after.find("</style").unwrap_or(after.len());
        let css = &after[(open + 1).min(end)..end];
        let mut selector_start = 0;
        while let Some(brace) = css[selector_start..].find('{').map(|i| i + selector_start) {
            let Some(close) = css[brace..].find('}').map(|i| i + brace) else { break };
            let selector = css[selector_start..brace].rsplit("*/").next().unwrap_or("");
            if let Some(page) = selector.split(',').map(str::trim).find(|s| *s == "html" || *s == "body" || *s == ":root") {
                opaque(page, &css[brace + 1..close]);
            }
            selector_start = close + 1;
        }
        rest = &after[end..];
    }
    if let Some(tag) = lower.find("<body").map(|i| &lower[i..]).and_then(|t| t.split_once('>').map(|(tag, _)| tag)) {
        if let Some(style) = tag.split_once("style=").map(|(_, v)| v.trim_start_matches(['"', '\'']).split(['"', '\'']).next().unwrap_or("")) {
            opaque("body", style);
        }
    }
    problems
}

fn widget_create(desktop: &dyn Desktop, args: &Value) -> Result<String, String> {
    let id = text_arg(args, "id")?;
    let name = text_arg(args, "name")?;
    let html = text_arg(args, "html")?;
    let description = optional_text(args, "description")?;
    let width = args.get("width").and_then(Value::as_f64);
    let height = args.get("height").and_then(Value::as_f64);
    let overwrite = optional_bool(args, "overwrite")?.unwrap_or(false);
    let place_now = optional_bool(args, "place_now")?.unwrap_or(true);
    let problems = widget_html_problems(html);
    if !problems.is_empty() {
        return Err(format!("小组件没有保存：\n- {}\n请按 widget_read_format 的规范修改后再保存。", problems.join("\n- ")));
    }

    let mut pack = json!({
        "schemaVersion": 1,
        "id": id,
        "name": name,
        "html": html,
        "theme": "transparent",
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
        lines.extend(mine.into_iter().map(|g| format!("{} {}", line(g), if g["layer"] == "desktop" { "[自创·桌面层：浮在桌面其他窗口上方运行]" } else { "[自创]" })));
    }
    for error in data["errors"].as_array().into_iter().flatten().filter_map(Value::as_str) {
        lines.push(format!("无法加载的自创游戏文件夹：{error}"));
    }
    lines.push(r#"可用 game_open 启动（例如 game_open(id: "snake-arena") 打开贪吃蛇），game_close 退出游戏空间或停止桌面层游戏；game_create 可以为用户创作新游戏。"#.into());
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
    let on_desktop = match target.as_deref() {
        Some(id) => desktop.games()?["games"].as_array().into_iter().flatten().any(|g| g["id"].as_str() == Some(id) && g["layer"] == "desktop"),
        None => false,
    };
    Ok(match target {
        Some(id) if on_desktop => format!("已把桌面层游戏放到桌面上：{id}。它浮在其他窗口上方运行，用 game_close 停止。"),
        Some(id) => format!("已进入游戏空间并启动游戏：{id}"),
        None => "已打开小伴的游戏空间页面".into(),
    })
}

fn game_create(desktop: &dyn Desktop, args: &Value, workspace: &std::path::Path) -> Result<String, String> {
    let id = text_arg(args, "id")?;
    let name = text_arg(args, "name")?;
    let description = text_arg(args, "description")?;
    let save = optional_bool(args, "save")?.unwrap_or(false);
    let overwrite = optional_bool(args, "overwrite")?.unwrap_or(false);
    let mut files = if let Some(source) = optional_text(args, "source_dir")? {
        if args.get("files").is_some() { return Err("files 和 source_dir 只能提供一个".into()); }
        super::game_draft::read(workspace, source)?
    } else {
        let items = args.get("files").and_then(Value::as_array).ok_or("请提供 files 或 source_dir")?;
        let mut files = Vec::with_capacity(items.len());
        for item in items {
            let path = item.get("path").and_then(Value::as_str).ok_or("each file needs a string `path`")?;
            let content = item.get("content").and_then(Value::as_str).ok_or_else(|| format!("文件 {path} 缺少文本 `content`"))?;
            files.push((path.to_string(), content.to_string()));
        }
        files
    };
    files.sort_by(|a, b| a.0.cmp(&b.0));
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

const GAME_CREATE_DESCRIPTION: &str = "为用户创作一款离线小游戏，保存成游戏文件夹（文件夹即游戏）。先调用 game_read_format 阅读游戏包规范（文件、脚本、运行环境、握手、成绩、存档）再写。\
优先在工作目录的草稿子目录（如 game-drafts/<id>/）用 write_file 分文件写、edit_file 局部改，完成后用 source_dir 安装；files 仅适合简短作品，二者必须二选一。\
创作时同时提供与游戏玩法、名称相匹配的 cover.svg 封面（640×360、16:9，纯 SVG，不引用外部资源）。安装失败不会改动已有游戏。保存后不会自动打开，试玩用 game_open(id)。";

/// What `game_read_format` returns: the full game pack rules, read only when a game is being made.
const GAME_FORMAT: &str = "游戏包规范（小伴程序内置，无需查找本地文件）：\n\
0. 创作方式：在当前工作目录的专用子目录（如 game-drafts/<id>/）使用 write_file 逐个写入 index.html、style.css、game.js 等，较长脚本继续拆分；用 edit_file 局部修改，保留已完成文件。完成后调用 game_create，source_dir 指向该草稿目录，无需在参数中重传源码。安装失败不会改动已有游戏，修正草稿后再安装。不要通过缩减玩法或反复重写整包来应付输出超限。游戏保存在用户数据目录 games/packs/<id>/；可用 game_list 查看、game_delete 删除。\n\
1. 文件：files 必须包含 index.html 作为入口，其余文件用相对路径引用（如 <script src=\"game.js\"></script>、<link rel=\"stylesheet\" href=\"style.css\">）。只能写 html、js、css、json、svg、md、txt 文本文件，最多 40 个，单个不超过 1 MB，合计不超过 4 MB；路径只用字母数字和 . _ -，用 / 分层，最多 4 层，不以 . 开头。不要写 manifest.json（由 game_create 生成）。创作时同时写入 cover.svg：640×360、16:9，以游戏玩法插画与名称构图，纯 SVG，不引用外部资源；安装时自动登记为封面。旧游戏未提供封面时，客户端会按名称和类型生成默认插画。\n\
2. 脚本：用普通经典脚本（<script src>），不要 ES modules（type=module、import/export），所有资源用相对路径，不联网、不引用任何外部 CDN、字体或图片；图片用内联 SVG 或 canvas 绘制。\n\
3. 运行环境：游戏在沙盒 iframe 里运行，没有网络、localStorage、cookie、IndexedDB 和任何系统权限。画布与布局适配 100vw × 100vh，同时支持键盘和鼠标/触摸。宿主会自动注入 desktop-host.js，由它处理 Esc 退出，游戏不要自己再处理 Esc，也不要自己引用它。\n\
4. 握手协议（宿主注入的 desktop-host.js 会在收到 host.init 时自动回复 game.ready，所以不读存档的游戏可以不写；需要存档或成绩时按下面实现）：在页面脚本顶层同步监听 window 的 message。宿主在 iframe 加载完成后发送 {channel:'xiaban.game-host', protocol:1, type:'host.init', nonce, save, safeArea}，其中 save 是上次存档对象或 null。只接受 event.source === window.parent 且 channel、protocol、nonce 都匹配的消息；nonce 与地址栏参数 hostNonce 相同（new URLSearchParams(location.search).get('hostNonce')）。收到 host.init 后回复 window.parent.postMessage({channel:'xiaban.game-pack', protocol:1, nonce, type:'game.ready'}, '*')；game.ready 之前发出的其它消息会被忽略。\n\
5. 可选成绩：开始一局发 type:'game.start'，之后可发 type:'game.progress' 和一次 type:'game.result'，都带非负整数 score（消息结构同上，channel 为 'xiaban.game-pack'，带 nonce）。\n\
6. 存档（仅 save=true 时有效）：游戏在 game.ready 之后用 {channel:'xiaban.game-pack', protocol:1, nonce, type:'game.save', data:{…}} 提交存档，data 必须是普通 JSON 对象，序列化后不超过 32 KiB；宿主存在本机，下次启动时放进 host.init 的 save 字段。进度变化时就发送，不要依赖退出时才存。save=false 时不要发送 game.save，宿主会忽略。";

/// How to make a game worth playing, returned after the rules so the model reads both before writing.
const GAME_CRAFT: &str = include_str!("../../../docs/craft/game.md");

fn game_read_format(_: &dyn Desktop, _: &Value) -> Result<String, String> {
    Ok(format!("{GAME_FORMAT}\n\n{GAME_CRAFT}"))
}

/// What `widget_read_format` returns: the rules for a custom widget's HTML.
const WIDGET_FORMAT: &str = "自定义小组件规范（小伴程序内置，无需查找本地文件）：\n\
1. 形式：一个完整的自包含单文件 HTML（包含 <style>、DOM 结构与 <script> 逻辑），如番茄钟、极简时钟、每日诗词、倒数日、喝水提醒、趣味小工具。默认宽 280、高 240，内容在声明的 width、height 视口内自适应，避免横向溢出和滚动条。\n\
2. 透明：宿主固定透明无底，不提供外壳主题、底板或纹理。html、body 背景透明，默认不绘制铺满组件的底板；仅在用户明确需要时，在组件内容中绘制底板。\n\
3. 阴影：允许小范围、低透明度的柔和投影，建议 box-shadow:0 1px 3px -2px rgba(0,0,0,0.12)，filter:drop-shadow(0 1px 1px rgba(0,0,0,0.08))，周围留至少 4px 空间；避免大模糊、大偏移、多层外投影以及贴着视口边缘画阴影，防止被截断。\n\
4. 裁切：宿主按声明的矩形视口裁切。角色头部、耳朵、蝴蝶结、突出装饰及动画最大位移都必须包含在视口内，并预留透明安全边距。绝对定位前确认实际的 position 容器；探出卡片的角色应以卡片为定位父级，并在整体视口中预留顶部空间，不能用负 top 让内容伸出整个视口，不能依赖 overflow:visible 越过 iframe 或窗口边界。\n\
5. 拖动：宿主统一提供按住非交互区域拖动和位置保存，无需自行实现窗口拖拽；不要使用 Electron 的 -webkit-app-region 或直接调用 Tauri。按钮、输入框等使用语义化元素；自定义手势区域加 data-no-drag，保留可拖动的背景区域。";

/// How to make a widget look good, returned after the rules so the model reads both before writing.
const WIDGET_CRAFT: &str = include_str!("../../../docs/craft/widget.md");

fn widget_read_format(_: &dyn Desktop, _: &Value) -> Result<String, String> {
    Ok(format!("{WIDGET_FORMAT}\n\n{WIDGET_CRAFT}"))
}

fn game_close(desktop: &dyn Desktop, _: &Value) -> Result<String, String> {
    desktop.game_close()?;
    Ok("已退出游戏空间或停止桌面层游戏，返回桌面".into())
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

fn pet_read_format(_: &dyn Desktop,_args:&Value)->Result<String,String>{
    let example=json!({"schemaVersion":1,"id":"my-garden","name":"我的小花园","description":"一株摇摆的新叶","color":"#91af6a","pets":[{"id":"leaf","name":"嫩芽","description":"原花盆上的小叶子","kind":"plant","parts":[{"id":"leaf","name":"叶片","pivot":[160,157],"motion":"sway","shapes":[{"type":"path","d":"M160 157L160 93","fill":"none","stroke":"#6b9653","strokeWidth":7},{"type":"path","d":"M160 108Q106 104 99 58Q151 57 160 108Z","fill":"#91af6a"}]}]}]});
    Ok(json!({"format":include_str!("../../../docs/pets.md"),"expressions":EXPRESSIONS,"example":example}).to_string())
}
fn pet_catalog(desktop:&dyn Desktop,args:&Value)->Result<String,String>{
 let value=desktop.pet_catalog()?;
 if let Some(id)=optional_text(args,"theme_id")?{let mut value=value["themes"].as_array().and_then(|items|items.iter().find(|t|t["id"]==id)).cloned().ok_or("主题不存在")?;value.as_object_mut().unwrap().remove("source");let mut theme:crate::pet_packs::Theme=serde_json::from_value(value).map_err(|e|e.to_string())?;crate::pet_packs::compact_images(&mut theme)?;return serde_json::to_string(&theme).map_err(|e|e.to_string());}
 let themes=value["themes"].as_array().into_iter().flatten().map(|t|json!({"id":t["id"],"name":t["name"],"source":t["source"],"pets":t["pets"].as_array().into_iter().flatten().map(|p|json!({"id":p["id"],"name":p["name"],"kind":p["kind"]})).collect::<Vec<_>>()})).collect::<Vec<_>>();
 Ok(json!({"themes":themes,"instances":value["layout"]["instances"],"errors":value["errors"]}).to_string())
}
#[cfg(test)]
pub fn tools(desktop: Arc<dyn Desktop>) -> Vec<Arc<dyn Tool>> {
    tools_in_workspace(desktop, std::path::Path::new("."))
}

pub fn tools_in_workspace(desktop: Arc<dyn Desktop>, workspace: &std::path::Path) -> Vec<Arc<dyn Tool>> {
    let game_workspace = workspace.to_path_buf();
    let none = || object(json!({}), &[]);
    let status_run = |d: &dyn Desktop, _: &Value| -> Result<String, String> { Ok(d.status()?.to_string()) };
    vec![
        tool(&desktop, ToolSpec::new("app_guide", "查小伴桌面的功能清单：每项功能能做什么、在控制台哪个页面、对应哪些工具、哪些要用户自己在页面里操作。用户问“能不能…”“在哪里…”“怎么设置…”，或你想说“不支持”之前先调用。topic 可填功能名、页面名、工具名或关键词（如 置顶、代理、提醒）；不填则返回完整目录。", object(json!({"topic":{"type":"string"}}), &[])).risk(Risk::Read).parallel(), |_, a| Ok(super::app_guide::describe(optional_text(a, "topic")?))),
        tool(&desktop, ToolSpec::new("open_page", "把小伴控制台带到前台并打开指定页面，用来带用户去对话里做不了、或用户想自己操作的地方（如代理、API 密钥、权限、插件商店）。page: new-task 新建任务、ai-store AI 插件、library 资料库、studio 装扮、games 游玩、care 提醒、settings 设置。studio 的 tab: wallpapers 壁纸、effects 特效、widgets 组件、pets 伙伴；settings 的 tab: general、appearance、ai、assistant、voice、network、safety、about。具体页面对应什么功能见 app_guide。", object(json!({
            "page":{"type":"string","enum":super::app_guide::PAGES},
            "tab":{"type":"string","description":"studio 或 settings 的子页，其他页面不要填"}
        }), &["page"])).risk(Risk::Write).self_managed().serial(), |d, a| {
            let page = text_arg(a, "page")?;
            let tab = optional_text(a, "tab")?.filter(|t| !t.is_empty());
            super::app_guide::validate_page(page, tab)?;
            d.open_page(page, tab)?;
            Ok(match tab { Some(tab) => format!("已打开 {page} 页面的 {tab}"), None => format!("已打开 {page} 页面") })
        }),
        tool(&desktop, ToolSpec::new("desktop_settings_set", "切换小伴桌面的外观：深色/浅色/跟随系统（appearance）、主题色（accent）、暂停或恢复桌面动画（animationPaused，省电、专心时用）。至少给一项；当前值用 desktop_status 查看。", object(json!({
            "appearance":{"type":"string","enum":crate::preferences::APPEARANCES,"description":"system 跟随系统，light 浅色，dark 深色"},
            "accent":{"type":"string","enum":crate::preferences::ACCENTS,"description":"主题色：red 红、green 绿、blue 蓝、purple 紫、amber 琥珀、graphite 石墨"},
            "animationPaused":{"type":"boolean","description":"true 暂停桌面动画，false 恢复"}
        }), &[])).risk(Risk::Write).self_managed().serial(), |d, a| {
            let appearance = optional_text(a, "appearance")?;
            let accent = optional_text(a, "accent")?;
            let paused = optional_bool(a, "animationPaused")?;
            if appearance.is_none() && accent.is_none() && paused.is_none() { return Err("请至少给 appearance、accent 或 animationPaused 之一".into()); }
            if appearance.is_some_and(|v| !crate::preferences::APPEARANCES.contains(&v)) { return Err(format!("appearance 只能是 {}", crate::preferences::APPEARANCES.join("、"))); }
            if accent.is_some_and(|v| !crate::preferences::ACCENTS.contains(&v)) { return Err(format!("accent 只能是 {}", crate::preferences::ACCENTS.join("、"))); }
            d.set_display(appearance, accent, paused).map(|v| v.to_string())
        }),
        tool(&desktop, ToolSpec::new("desktop_windows", "查看小伴自己的窗口和显示器：盆栽 companion、超级小伴 assistant、控制台 control、桌面组件 widget:<id> 的位置、显示状态和层级。移动前用本工具确认目标与显示器。", none()).risk(Risk::Read).parallel(), |d, _| d.windows().map(|v| v.to_string())),
        tool(&desktop, ToolSpec::new("desktop_window_set", "调整小伴自己窗口的位置、层级（置顶/贴桌面）、显示/隐藏，不操作其他应用。target: companion 是盆栽，assistant 是超级小伴悬浮条（用户让你收起来时用 action=collapse），control 是设置控制台，widget:<id> 是组件。移动 x/y 使用屏内比例（0 左/上，0.5 中间，1 右/下），可指定 desktop_windows 返回的 monitor。layer: desktop 贴桌面、normal 普通窗口（组件只能 desktop/topmost）、topmost 置顶；组件（widget:<id>）的层级会保存、重启后仍保持；盆栽、超级小伴、控制台的层级只在本次运行有效。隐藏盆栽或组件会保存开关；收起助手不退出应用、不取消任务。", object(json!({
            "target":{"type":"string"},"action":{"type":"string","enum":["show","hide","collapse"]},
            "x":{"type":"number","minimum":0,"maximum":1},"y":{"type":"number","minimum":0,"maximum":1},
            "monitor":{"type":"string"},"layer":{"type":"string","enum":["desktop","normal","topmost"]}
        }), &["target"])).risk(Risk::Write).self_managed().serial(), |d, args| {
            let change: crate::window_control::Change=serde_json::from_value(args.clone()).map_err(|e|format!("窗口参数无效：{e}"))?;
            change.validate()?;
            d.window_update(&change).map(|v|v.to_string())
        }),
        tool(&desktop, ToolSpec::new("desktop_status", "查看小伴桌面当前的状态：显示模式、正在用的壁纸和特效、小伴的外壳与表情。", none()).risk(Risk::Read).parallel(), status_run),
        tool(&desktop, ToolSpec::new("wallpaper_list", "列出可用的壁纸（id、标题、类型、是否已下载）和当前正在使用的那张。", none()).risk(Risk::Read).parallel(), wallpaper_list),
        tool(
            &desktop,
            ToolSpec::new("wallpaper_apply", "把桌面壁纸换成指定的壁纸，用 wallpaper_list 里的 id。id 为 companion 表示换回没有壁纸的系统桌面。", object(json!({"id": {"type": "string"}}), &["id"]))
                .risk(Risk::Write).self_managed()
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
        tool(&desktop, ToolSpec::new("effect_list", "列出桌面特效（雨、雪、花瓣等）和正在使用的那个。", none()).risk(Risk::Read).parallel(), effect_list),
        tool(
            &desktop,
            ToolSpec::new("effect_read_format", "读取程序内置的完整桌面特效包格式与可用 JSON 示例。创作特效前先调用本工具；无需搜索工作目录、安装目录或联网查找文档。", none()).risk(Risk::Read),
            effect_read_format,
        ),
        tool(
            &desktop,
            ToolSpec::new("effect_set", "切换桌面特效，用 effect_list 里的 id；id 为 none 表示关闭特效。", object(json!({"id": {"type": "string"}}), &["id"])).risk(Risk::Write).self_managed().serial(),
            effect_set,
        ),
        tool(
            &desktop,
            ToolSpec::new(
                "effect_create",
                "为用户创作一个新的桌面特效包并保存到特效文件夹，保存后它就能像内置特效一样被选用。先调用 effect_read_format 阅读内置的完整格式和示例，再写一份完整的 JSON 放进 pack（对象或 JSON 字符串），必须包含 schemaVersion、id、name、layers。",
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
                "设置桌面盆栽：表情、植物，或显示/隐藏。花盆保持不变，skin=pot 为绿叶，pot-white-flower 为白色小雏菊。至少给一项。",
                object(
                    json!({
                        "expression": {"type": "string", "enum": EXPRESSIONS},
                        "skin": {"type": "string", "enum": SKINS},
                        "enabled": {"type": "boolean", "description": "true 显示小伴，false 隐藏小伴"},
                    }),
                    &[],
                ),
            )
            .risk(Risk::Write).self_managed()
            .serial(),
            companion_set,
        ),
        tool(&desktop,ToolSpec::new("pet_read_format","读取宠物主题创作格式和完整示例，创作或改造宠物前必读。",none()).risk(Risk::Read).parallel(),pet_read_format),
        tool(&desktop,ToolSpec::new("pet_catalog","查看官方/自定义宠物主题及独立桌面实例。提供theme_id时读取该主题的完整作品，便于参考或调整形象。",object(json!({"theme_id":{"type":"string"}}),&[])).risk(Risk::Read).parallel(),pet_catalog),
        tool(&desktop,ToolSpec::new("pet_theme_save","保存AI创作的宠物主题，格式先读pet_read_format。保留动态表情系统。保存模板不会自动放到桌面，随后用pet_add。",object(json!({"theme":{"type":"object"},"overwrite":{"type":"boolean"}}),&["theme"])).risk(Risk::Write).serial(),|d,a|{let theme:crate::pet_packs::Theme=serde_json::from_value(a["theme"].clone()).map_err(|e|format!("主题格式无效：{e}"))?;let result=json!({"saved":true,"theme_id":theme.id,"name":theme.name,"pet_count":theme.pets.len()});d.pet_save_theme(theme,optional_bool(a,"overwrite")?.unwrap_or(false))?;Ok(result.to_string())}),
        tool(&desktop,ToolSpec::new("pet_theme_delete","删除自定义宠物主题。官方主题受保护；先移除该主题的桌面实例。",object(json!({"id":{"type":"string"}}),&["id"])).risk(Risk::Write).serial(),|d,a|{let id=text_arg(a,"id")?;d.pet_delete_theme(id.into())?;Ok(json!({"deleted":true,"theme_id":id}).to_string())}),
        tool(&desktop,ToolSpec::new("pet_add","从官方或自定义主题添加一只独立桌面宠物，允许多只以及同款重复添加。返回桌面实例id。",object(json!({"theme_id":{"type":"string"},"pet_id":{"type":"string"},"layer":{"type":"string","enum":["desktop","topmost"],"description":"desktop 贴桌面，topmost 始终置顶；会保存，重启后仍保持"}}),&["theme_id","pet_id"])).risk(Risk::Write).self_managed().serial(),|d,a|d.pet_add(text_arg(a,"theme_id")?.into(),text_arg(a,"pet_id")?.into(),optional_text(a,"layer")?.map(String::from)).map(|v|v.to_string())),
        tool(&desktop,ToolSpec::new("pet_update","只调整指定桌面宠物实例，不影响同款其他宠物。id来自pet_catalog的instances。",object(json!({"id":{"type":"string"},"patch":{"type":"object","properties":{"name":{"type":"string"},"size":{"type":"number","minimum":80,"maximum":420},"expression":{"type":"string","enum":EXPRESSIONS},"enabled":{"type":"boolean"},"links":{"type":"boolean"},"x":{"type":"number","minimum":0,"maximum":1},"y":{"type":"number","minimum":0,"maximum":1},"layer":{"type":"string","enum":["desktop","topmost"],"description":"desktop 贴桌面，topmost 始终置顶；会保存，重启后仍保持"},"monitor":{"type":"string"},"themeId":{"type":"string"},"petId":{"type":"string"}},"additionalProperties":false}}),&["id","patch"])).risk(Risk::Write).self_managed().serial(),|d,a|{let patch=serde_json::from_value(a["patch"].clone()).map_err(|e|format!("宠物配置无效：{e}"))?;let id=text_arg(a,"id")?;d.pet_update(id,patch)?;Ok(json!({"updated":true,"id":id,"patch":a["patch"]}).to_string())}),
        tool(&desktop,ToolSpec::new("pet_remove","移除指定桌面宠物实例，主题作品仍保留，其他实例不受影响。",object(json!({"id":{"type":"string"}}),&["id"])).risk(Risk::Write).self_managed().serial(),|d,a|{let id=text_arg(a,"id")?;d.pet_remove(id)?;Ok(json!({"removed":true,"id":id}).to_string())}),
        tool(&desktop, ToolSpec::new("care_list", "列出关怀提醒的规则（id、时间、文案）和总开关状态。", none()).risk(Risk::Read).parallel(), care_list),
        tool(
            &desktop,
            ToolSpec::new(
                "care_set",
                "添加或修改一条关怀提醒。不给 id 是添加，给了 id 是修改那一条（没给的字段保持不变）。时间用 24 小时制的本地时间。kind=agent 是定时任务：到时小伴在后台只用读取类工具（读文件、搜索、网页）完成 text 里写的事，结果显示在提醒里；每次添加或修改都要用户确认。",
                object(
                    json!({
                        "id": {"type": "string", "description": "要修改的提醒 id，添加时不填"},
                        "text": {"type": "string", "description": "提醒的文字，最多 240 字；定时任务写要小伴做的事，最多 1000 字"},
                        "kind": {"type": "string", "enum": ["custom", "off-work", "rest", "agent"], "description": "custom 普通提醒（默认），off-work 下班提醒，rest 每隔一段时间提醒休息，agent 定时任务（修改定时任务时也要带上）"},
                        "repeat": {"type": "string", "enum": ["daily", "weekdays", "once"], "description": "daily 每天 / weekdays 每周的指定几天（必须同时给 weekdays 数组，工作日写 [1,2,3,4,5]）/ once 只提醒一次"},
                        "time": {"type": "string", "description": "HH:MM，例如 15:00"},
                        "onceAt": {"type": "string", "description": "repeat 为 once 时的本地日期时间，格式 YYYY-MM-DDTHH:MM，必须是未来的时间"},
                        "weekdays": {"type": "array", "items": {"type": "integer", "minimum": 1, "maximum": 7}, "description": "repeat 为 weekdays 时用，1 是周一，7 是周日"},
                        "intervalMinutes": {"type": "integer", "minimum": 5, "maximum": 480, "description": "kind 为 rest 时的间隔分钟数"},
                        "enabled": {"type": "boolean"},
                    }),
                    &[],
                ),
            )
            .risk(Risk::Write).self_managed()
            .serial(),
            care_set,
        ),
        tool(&desktop, ToolSpec::new("care_delete", "删除一条关怀提醒，id 来自 care_list。", object(json!({"id": {"type": "string"}}), &["id"])).risk(Risk::Write).self_managed().serial(), care_delete),
        tool(&desktop, ToolSpec::new("widget_list", "列出新版桌面组件的 id、类型、缩放、层级和显示状态，并列出待办、日程、纪念日、股票的内容（含条目 id）。", none()).risk(Risk::Read).parallel(), widget_list),
        tool(
            &desktop,
            ToolSpec::new(
                "widget_add",
                "在桌面放置新版时钟、天气、待办、今日日程、股票、电子木鱼、月历或纪念日组件。kind 必填；用户没说要哪种时先用 ask_user 问，不要空参数调用。可用 content 同时填内容（待办、纪念日、日程、股票），如“加个待办：买花”。可设层级（layer=topmost 置顶）。固定 ivory 材质，支持多实例，不设组件数量上限。",
                object(
                    json!({
                        "kind": {"type": "string", "enum": crate::widgets::KINDS, "description": "组件类型；custom 表示放置一个已保存的自创组件，此时必须同时给 custom_id"},
                        "custom_id": {"type": "string", "description": "仅 kind=custom 时必填：自创组件模板的 id，取自 widget_list 输出里「自创组件模板」每行开头的 id"},
                        "theme": {"type": "string", "enum": ["ivory"], "description": "可选外观风格，固定 ivory（设计稿材质）"},
                        "layer": {"type": "string", "enum": ["desktop", "topmost"], "description": "可选层级：desktop 贴在桌面（默认），topmost 始终置顶（浮在其他窗口上方）。会保存，重启后仍保持，不是临时的"},
                        "content": widget_content_schema(),
                    }),
                    &["kind"],
                ),
            ).risk(Risk::Write).self_managed().serial(),
            widget_add,
        ),
        tool(&desktop, ToolSpec::new("widget_read_format", "读取自定义小组件的 HTML 规范（透明、阴影、裁切、拖动）。用 widget_create 创作前先调用；规范已内置在程序中。", none()).risk(Risk::Read).parallel(), widget_read_format),
        tool(
            &desktop,
            ToolSpec::new(
                "widget_create",
                "为用户创作一个新的自定义单文件 HTML 桌面小组件（番茄钟、每日诗词、倒数日等），默认宽 280、高 240，完成后默认放到桌面。先调用 widget_read_format 阅读透明、裁切、拖动等规范再写。",
                object(
                    json!({
                        "id": { "type": "string", "description": "小组件英文标识（小写字母、数字、-、_，如 pomodoro-timer）" },
                        "name": { "type": "string", "description": "小组件中文名称（最多 24 字，如 专注番茄钟）" },
                        "html": { "type": "string", "description": "完整的自包含单文件 HTML，按 widget_read_format 的规范编写" },
                        "description": { "type": "string", "description": "一句话介绍（最多 80 字）" },
                        "width": { "type": "number", "minimum": 100, "maximum": 800, "description": "建议默认宽度（像素，默认 280）" },
                        "height": { "type": "number", "minimum": 80, "maximum": 800, "description": "建议默认高度（像素，默认 240）" },
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
        tool(
            &desktop,
            ToolSpec::new(
                "widget_update",
                "修改已有的桌面组件：内容（content，改前先 widget_list 看 id）、层级/置顶（layer）、外观风格、缩放大小（0.75-1.5）、位置（x, y 为 0.0-1.0 比例）、显示/隐藏或城市。至少给一项修改。",
                object(
                    json!({
                        "id": {"type": "string", "description": "组件 id，来自 widget_list"},
                        "theme": {"type": "string", "enum": ["ivory"]},
                        "layer": {"type": "string", "enum": ["desktop", "topmost"], "description": "desktop 贴在桌面，topmost 始终置顶（浮在其他窗口上方）。会保存，重启后仍保持，不是临时的"},
                        "content": widget_content_schema(),
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
            ).risk(Risk::Write).self_managed().serial(),
            widget_update,
        ),
        tool(
            &desktop,
            ToolSpec::new("widget_remove", "从桌面移除指定的组件，id 来自 widget_list。", object(json!({"id": {"type": "string"}}), &["id"])).risk(Risk::Write).self_managed().serial(),
            widget_remove,
        ),
        tool(&desktop, ToolSpec::new("game_list", "列出小伴支持的小游戏（内置的贪吃蛇、俄罗斯方块、2048、消失高手、奶茶大师、水墨剑客、暮色滑翔，以及用户自创游戏）以及游戏空间状态。", none()).risk(Risk::Read).parallel(), game_list),
        tool(
            &desktop,
            ToolSpec::new(
                "game_open",
                "在单独的窗口里直接玩一款游戏，必须指定游戏 id（例如 snake-arena 为小伴贪吃蛇，falling-blocks 为俄罗斯方块，merge-2048 为 2048，block-puzzle 为消失高手，water-sort 为奶茶大师，ink-blade 为水墨剑客，dusk-glider 为暮色滑翔；用户自创游戏用 game_list 里的 id）。窗口默认不全屏，玩家可用系统标题栏切换全屏。不指定 id 只打开游戏空间页面，不会启动游戏。",
                object(
                    json!({
                        "id": {
                            "type": "string",
                            "description": "游戏标识或内置游戏名称；自创游戏只认精确 id",
                        },
                    }),
                    &[],
                ),
            ).risk(Risk::Write).self_managed().serial(),
            game_open,
        ),
        tool(&desktop, ToolSpec::new("game_read_format", "读取游戏包规范（文件、脚本、运行环境、握手协议、成绩、存档）。用 game_create 创作前先调用；规范已内置在程序中。", none()).risk(Risk::Read).parallel(), game_read_format),
        tool(
            &desktop,
            ToolSpec::new(
                "game_create",
                GAME_CREATE_DESCRIPTION,
                {
                let mut schema = object(
                    json!({
                        "id": { "type": "string", "description": "游戏英文标识（小写字母、数字、-，2 到 40 个字符，如 star-catcher；不能与内置游戏重名）" },
                        "name": { "type": "string", "description": "游戏中文名称（最多 24 字）" },
                        "description": { "type": "string", "description": "一句话介绍（最多 80 字）" },
                        "source_dir": { "type": "string", "description": "当前工作目录内的游戏草稿子目录，与 files 二选一；用文件工具逐步创作后再安装，无需重传源码" },
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
                    &["id", "name", "description"],
                );
                schema["oneOf"] = json!([{"required": ["files"], "not": {"required": ["source_dir"]}}, {"required": ["source_dir"], "not": {"required": ["files"]}}]);
                schema
                },
            )
            .risk(Risk::Write)
            .serial(),
            move |desktop, args| game_create(desktop, args, &game_workspace),
        ),
        tool(
            &desktop,
            ToolSpec::new("game_delete", "删除指定的自创游戏（只能删除 game_create 创作的、game_list 标注为自创的游戏，内置游戏受保护）。若游戏空间正在运行会先关闭，游戏的本机存档也会一并清除。", object(json!({"id": {"type": "string"}}), &["id"])).risk(Risk::Write).serial(),
            game_delete,
        ),
        tool(&desktop, ToolSpec::new("game_close", "退出游戏空间，回到正常的桌面生活空间；如果正在运行的是桌面层游戏，就停止它。", none()).risk(Risk::Write).self_managed().serial(), game_close),
        tool(
            &desktop,
            ToolSpec::new(
                "community_share",
                "把用户自己创作的壁纸、特效、桌面组件、桌宠或游戏分享到超级小伴论坛：先弹出保存对话框，把作品导出成 zip 资源包，再在浏览器打开对应板块（壁纸、特效、组件、桌宠去「小伴壁纸」，游戏去「小伴游戏」）。发帖要用户自己登录论坛完成，本工具只负责导出和打开页面。id 来自 wallpaper_list、effect_list、widget_list、pet_catalog、game_list，只能是用户自创的作品，内置的不能导出。用户说“分享到论坛”“发到论坛”“导出给别人用”时使用。",
                object(
                    json!({
                        "kind": {"type": "string", "enum": community::KINDS, "description": "作品类型"},
                        "id": {"type": "string", "description": "作品的 id"},
                        "open_forum": {"type": "boolean", "description": "导出后是否打开论坛板块，默认 true"},
                    }),
                    &["kind", "id"],
                ),
            )
            .risk(Risk::Write)
            .serial()
            .timeout(std::time::Duration::from_secs(300)),
            community::share,
        ),
        tool(
            &desktop,
            ToolSpec::new(
                "community_open",
                "在系统浏览器打开小伴桌面官网或超级小伴论坛的指定页面。用户想去官网看看、下载、看使用指南，或去论坛某个板块时使用。想了解页面里写了什么，用 web_fetch 读取，不要只靠打开。",
                object(
                    json!({"target": {"type": "string", "enum": community::PAGES.map(|(target, _, _)| target), "description": "site 官网首页，site_images 图片壁纸，site_videos 动态视频壁纸，site_guide 使用指南，forum 论坛首页，forum_wallpaper 小伴壁纸板块，forum_game 小伴游戏板块，forum_app 小伴App板块，forum_feedback 建议反馈板块"}}),
                    &["target"],
                ),
            )
            .risk(Risk::Write).self_managed()
            .serial(),
            community::open,
        ),
        tool(
            &desktop,
            ToolSpec::new(
                "request_accessibility_permission",
                "仅在 macOS 通过内置 Open Computer Use 引擎启动真实权限引导。先用 computer_use_diagnose 确认缺失项，用户在系统界面授权后再次检查；启动引导不代表已获授权。不重置权限，不通过 AppleScript 额外申请主程序无障碍。",
                none(),
            ).risk(Risk::Write).self_managed().serial(),
            |d: &dyn Desktop, _: &Value| -> Result<String, String> {
                let res = d.request_permissions()?;
                Ok(res["message"].as_str().unwrap_or("已申请系统权限").to_string())
            },
        ),
        tool(
            &desktop,
            ToolSpec::new(
                "computer_use_repair_permissions",
                "修复 macOS 内置引擎旧授权记录。先诊断；仅在确认旧签名冲突或用户明确要求定向修复时调用。逐次用户确认后，用 tccutil 只重置 com.ifuryst.opencomputeruse 的所选权限，再打开当前引擎与授权引导。同标识独立安装版的该权限也会被清除，需重新授权。不会授予权限、不重置其他应用或其他权限。参数校验失败应修正原调用，不能使用此修复。",
                object(json!({"permission": {"type": "string", "enum": ["accessibility", "screenRecording"], "description": "只选择诊断失败的权限；保留其他有效授权"}}), &["permission"]),
            ).risk(Risk::Write).always_ask().serial().timeout(std::time::Duration::from_secs(60)),
            |d: &dyn Desktop, args: &Value| -> Result<String, String> {
                Ok(d.computer_use_repair_permissions(text_arg(args, "permission")?)?.to_string())
            },
        ),
        tool(
            &desktop,
            ToolSpec::new(
                "computer_use_diagnose",
                "检查电脑控制开关、真实权限、稳定签名和本引擎近期 TCC 日志中的旧签名冲突。只读，不截图、不打开弹窗、不修改 TCC。参数校验失败应修正原调用，不等于权限错误。MCP 连接成功不代表权限可用。",
                none(),
            ).risk(Risk::Read).serial().timeout(std::time::Duration::from_secs(25)),
            |d: &dyn Desktop, _: &Value| -> Result<String, String> { Ok(d.computer_use_diagnose()?.to_string()) },
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
        extra_widgets: Mutex<Vec<Value>>,
        fail: Mutex<Option<String>>,
        saved: Mutex<Option<(String, u64)>>,
    }

    impl Fake {
        fn log(&self, entry: impl Into<String>) -> Result<(), String> {
            self.calls.lock().unwrap_or_else(|e| e.into_inner()).push(entry.into());
            match self.fail.lock().unwrap_or_else(|e| e.into_inner()).clone() {
                Some(error) => Err(error),
                None => Ok(()),
            }
        }
        fn calls(&self) -> Vec<String> {
            self.calls.lock().unwrap_or_else(|e| e.into_inner()).clone()
        }
    }

    impl Desktop for Fake {
        fn pet_catalog(&self)->Result<Value,String>{let themes=crate::pet_packs::builtin().into_iter().map(|t|{let mut v=json!(t);v["source"]=json!("builtin");v}).collect::<Vec<_>>();Ok(json!({"themes":themes,"layout":{"instances":[]},"errors":[]}))}
        fn pet_save_theme(&self,theme:crate::pet_packs::Theme,_overwrite:bool)->Result<Value,String>{crate::pet_packs::validate(&theme)?;self.log(format!("pet-save {}",theme.id))?;self.pet_catalog()}
        fn pet_update(&self,id:&str,patch:crate::pets::Patch)->Result<Value,String>{self.log(format!("pet-update {id} {}",patch.pet_id.unwrap_or_default()))?;self.pet_catalog()}
        fn windows(&self) -> Result<Value,String> { Ok(json!({"windows":[{"target":"companion"},{"target":"assistant"}],"monitors":[{"id":"main"}]})) }
        fn window_update(&self, change: &crate::window_control::Change) -> Result<Value,String> {
            self.log(format!("window {change:?}"))?;
            Ok(json!({"ok":true,"target":change.target}))
        }
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
            Ok(json!({"config": {"enabled": true}, "rules": self.rules.lock().unwrap_or_else(|e| e.into_inner()).clone()}))
        }
        fn care_upsert(&self, rule: Rule) -> Result<(), String> {
            self.log(format!("upsert {} {} {} {:?} {:?} {} {}", rule.id, rule.kind, rule.repeat, rule.time, rule.weekdays, rule.text, rule.enabled))
        }
        fn care_delete(&self, id: &str) -> Result<(), String> {
            self.log(format!("delete {id}"))
        }
        fn widgets(&self) -> Result<Value, String> {
            let mut widgets = vec![
                json!({"id": "wgt-1", "kind": "clock", "theme": "ivory", "scale": 1.0, "x": 0.5, "y": 0.1, "enabled": true}),
                json!({"id": "wgt-2", "kind": "weather", "theme": "ivory", "scale": 1.2, "x": 0.8, "y": 0.1, "enabled": false, "city": {"name": "上海"}}),
            ];
            widgets.extend(self.extra_widgets.lock().unwrap_or_else(|e| e.into_inner()).clone());
            Ok(json!({"widgets": widgets}))
        }
        fn widget_add(&self, kind: &str, custom_id: Option<&str>) -> Result<Value, String> {
            self.log(match custom_id { Some(id) => format!("widget-add {kind} {id}"), None => format!("widget-add {kind}") })?;
            Ok(json!({"widgets": [{"id": "wgt-new", "kind": kind}]}))
        }
        fn widget_update(&self, id: &str, patch: Value) -> Result<Value, String> {
            self.log(format!("widget-update {id} {patch}"))?;
            if let Some(error) = self.fail_widget_update.lock().unwrap_or_else(|e| e.into_inner()).clone() { return Err(error); }
            Ok(json!({}))
        }
        fn widget_remove(&self, id: &str) -> Result<(), String> {
            self.log(format!("widget-remove {id}"))
        }
        fn custom_widgets(&self) -> Result<Value, String> {
            Ok(json!(self.templates.lock().unwrap_or_else(|e| e.into_inner()).clone()))
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
                {"id": "desk-pal", "name": "桌面伙伴", "description": "在窗口上走来走去", "source": "user", "layer": "desktop"},
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
        fn export_pack(&self, kind: &str, id: &str) -> Result<Option<(String, u64)>, String> {
            self.log(format!("export-pack {kind} {id}"))?;
            Ok(self.saved.lock().unwrap_or_else(|e| e.into_inner()).clone())
        }
        fn open_url(&self, url: &str) -> Result<(), String> {
            self.log(format!("open-url {url}"))
        }
        fn open_page(&self, page: &str, tab: Option<&str>) -> Result<(), String> {
            self.log(format!("open-page {page} {}", tab.unwrap_or("-")))
        }
        fn set_display(&self, appearance: Option<&str>, accent: Option<&str>, paused: Option<bool>) -> Result<Value, String> {
            self.log(format!("display {} {} {}", appearance.unwrap_or("-"), accent.unwrap_or("-"), paused.map_or("-".to_string(), |p| p.to_string())))?;
            Ok(json!({"appearance": appearance.unwrap_or("system"), "accent": accent.unwrap_or("blue"), "animationPaused": paused.unwrap_or(false)}))
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
    fn connector_discovery_waits_for_user_action_and_external_calls_keep_their_gate() {
        let all = super::super::connector_tools::tools(Arc::new(Fake::default()));
        let request = all.iter().find(|t| t.spec().name == "connector_request").unwrap();
        assert_eq!(request.spec().risk, Risk::Read);
        assert!(request.execute(json!({"references":[]}), &ToolCtx::detached("s1")).unwrap_err().contains("1 至 3"));
        assert!(request.execute(json!({"references":["a","b","c","d"]}), &ToolCtx::detached("s1")).is_err());
        let call = all.iter().find(|t| t.spec().name == "connector_call").unwrap();
        assert_eq!(call.spec().risk, Risk::Exec);
        assert!(call.spec().always_ask && call.spec().untrusted_output);
        assert!(call.execute_result(json!({"tool":"x","arguments":"invalid"}), &ToolCtx::detached("s1")).is_error);
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
        assert_eq!(names.len(), 44);
        for tool in &all {
            let spec = tool.spec();
            assert!(spec.name.len() <= 64 && spec.name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'), "{}", spec.name);
            assert_eq!(spec.schema["type"], "object", "{}", spec.name);
            assert!(!spec.description.is_empty());
        }
        let read: Vec<&str> = all.iter().filter(|t| t.spec().risk == Risk::Read).map(|t| t.spec().name.as_str()).collect();
        let mut read = read;
        read.sort();
        assert_eq!(read, vec!["app_guide", "care_list", "computer_use_diagnose", "desktop_status", "desktop_windows", "effect_list", "effect_read_format", "game_list", "game_read_format", "pet_catalog", "pet_read_format", "wallpaper_list", "widget_list", "widget_read_format"]);
        for tool in all.iter().filter(|t| t.spec().risk == Risk::Write) {
            assert!(!tool.spec().parallel_safe, "{} changes shared settings, so it runs alone", tool.spec().name);
        }
    }

    #[test]
    fn pet_tools_preserve_the_pot_and_create_pets_with_compact_results() {
        let fake=Arc::new(Fake::default());
        let format:Value=serde_json::from_str(&call(&fake,"pet_read_format",json!({})).unwrap()).unwrap();
        let theme:crate::pet_packs::Theme=serde_json::from_value(format["example"].clone()).unwrap();
        crate::pet_packs::validate(&theme).unwrap();
        assert!(theme.pets.iter().all(|p|p.kind=="plant"));
        assert!(format["example"]["pets"][0].get("body").is_none());
        let catalog=call(&fake,"pet_catalog",json!({})).unwrap();
        assert!(catalog.contains("花朵心语")&&catalog.contains("多肉点点"));
        assert!(!catalog.contains("萌宠乐趣"));
        let saved=call(&fake,"pet_theme_save",json!({"theme":theme})).unwrap();
        assert!(saved.len()<200&&saved.contains("my-garden"));
        call(&fake,"pet_update",json!({"id":"one","patch":{"themeId":"succulent-dots","petId":"jade"}})).unwrap();
        assert_eq!(fake.calls(),vec!["pet-save my-garden","pet-update one jade"]);
    }

    #[test]
    fn window_tools_expose_self_collapse_and_normalized_movement_without_shell() {
        let fake=Arc::new(Fake::default());
        assert!(call(&fake,"desktop_windows",json!({})).unwrap().contains("companion"));
        for args in [json!({"target":"assistant","action":"collapse"}),json!({"target":"companion","x":0.0,"y":0.0}),json!({"target":"widget:abc123","layer":"topmost"})] {
            assert!(call(&fake,"desktop_window_set",args).unwrap().contains("true"));
        }
        assert_eq!(fake.calls().len(),3);
        assert!(call(&fake,"desktop_window_set",json!({"target":"other-app","action":"hide"})).is_err());
        assert!(call(&fake,"desktop_window_set",json!({"target":"companion","x":12,"y":0})).is_err());
        assert_eq!(fake.calls().len(),3);
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
        *fake.fail.lock().unwrap_or_else(|e| e.into_inner()) = Some("id「rain」是内置特效的名字，请换一个".into());
        let pack = json!({"schemaVersion": 1, "id": "rain", "name": "冒名", "layers": []});
        assert_eq!(call(&fake, "effect_create", json!({"pack": pack})).unwrap_err(), "id「rain」是内置特效的名字，请换一个");
    }

    #[test]
    fn desktop_failures_come_back_as_the_tool_error() {
        let fake = Arc::new(Fake::default());
        *fake.fail.lock().unwrap_or_else(|e| e.into_inner()) = Some("请先退出游戏空间".into());
        assert_eq!(call(&fake, "wallpaper_apply", json!({"id": "w1"})).unwrap_err(), "请先退出游戏空间");
    }

    #[test]
    fn the_companion_can_switch_to_white_flowers_using_the_existing_tool() {
        let fake=Arc::new(Fake::default());
        assert!(call(&fake,"companion_set",json!({"skin":"pot-white-flower"})).unwrap().contains("pot-white-flower"));
        assert!(fake.calls()[0].contains("pot-white-flower"));
        let all=tools(fake);
        let spec=all.iter().find(|t|t.spec().name=="companion_set").unwrap().spec();
        assert_eq!(spec.schema["properties"]["skin"]["enum"],json!(SKINS));
    }

    #[test]
    fn the_companion_accepts_only_known_expressions_and_shells() {
        let fake = Arc::new(Fake::default());
        let out = call(&fake, "companion_set", json!({"expression": "happy", "skin": "pot", "enabled": true})).unwrap();
        assert_eq!(out, "已设置：表情 happy，外壳 pot，显示小伴");
        assert_eq!(call(&fake, "companion_set", json!({"enabled": false})).unwrap(), "已设置：隐藏小伴");
        assert!(call(&fake, "companion_set", json!({"expression": "furious"})).unwrap_err().contains("可用："));
        assert!(call(&fake, "companion_set", json!({"skin": "dragon"})).unwrap_err().contains("pot"));
        assert!(call(&fake, "companion_set", json!({})).unwrap_err().contains("至少"));
        assert!(call(&fake, "companion_set", json!({"enabled": "yes"})).unwrap_err().contains("true or false"));
        assert_eq!(fake.calls().len(), 2);
    }

    #[test]
    fn a_new_reminder_gets_defaults_and_only_the_fields_given() {
        let fake = Arc::new(Fake::default());
        let out = call(&fake, "care_set", json!({"text": "喝水", "time": "15:30"})).unwrap();
        assert!(out.starts_with("已添加提醒：喝水。每天 "), "{out}");
        let out = call(&fake, "care_set", json!({"text": "开会", "repeat": "weekdays", "weekdays": [1, 3, 5], "time": "09:00"})).unwrap();
        assert!(out.starts_with("已添加提醒：开会。"), "{out}");
        assert_eq!(fake.calls(), vec!["upsert  custom daily \"15:30\" [] 喝水 true", "upsert  custom weekdays \"09:00\" [1, 3, 5] 开会 true"]);
    }

    #[test]
    fn editing_starts_from_the_saved_reminder_and_unknown_ids_are_refused() {
        let fake = Arc::new(Fake::default());
        let saved = Rule { id: "care-1".into(), text: "喝水".into(), time: "15:00".into(), revision: 4, ..Default::default() };
        fake.rules.lock().unwrap_or_else(|e| e.into_inner()).push(serde_json::to_value(&saved).unwrap());
        let out = call(&fake, "care_set", json!({"id": "care-1", "time": "16:00", "enabled": false})).unwrap();
        assert!(out.starts_with("已更新提醒：喝水。"), "{out}");
        assert_eq!(fake.calls(), vec!["upsert care-1 custom daily \"16:00\" [] 喝水 false"]);
        assert!(call(&fake, "care_set", json!({"id": "care-404", "text": "x"})).unwrap_err().contains("care_list"));
    }

    #[test]
    fn a_widget_that_breaks_the_clear_rules_is_explained_and_not_saved() {
        let ok = [
            "<style>html,body{margin:0;background:transparent}.card{background:#fff;border-radius:12px}</style><div class=card>好</div>",
            "<style>body { background: rgba(0, 0, 0, 0) !important }</style>",
            "<style>:root{--c:#fff} body{background:var(--clear)}</style><body style=\"margin:0\">",
            "<style>/* body */ .body{background:#000}</style>",
        ];
        for html in ok {
            assert!(widget_html_problems(html).is_empty(), "{html}: {:?}", widget_html_problems(html));
        }
        for (html, fragment) in [
            ("<style>body{background:#fffaf0}</style>", "body 的背景"),
            ("<style>html , body{background-color: white}</style>", "背景"),
            ("<body style=\"background:linear-gradient(#fff,#eee)\">", "body 的背景"),
            ("<style>.drag{-webkit-app-region:drag}</style>", "app-region"),
            ("<script>window.__TAURI__.core.invoke('x')</script>", "Tauri"),
        ] {
            let problems = widget_html_problems(html).join("\n");
            assert!(problems.contains(fragment), "{html}: {problems}");
        }
        let fake = Arc::new(Fake::default());
        let error = call(&fake, "widget_create", json!({"id": "card", "name": "卡片", "html": "<style>body{background:#fff}</style>"})).unwrap_err();
        assert!(error.contains("没有保存") && error.contains("widget_read_format"), "{error}");
        assert!(fake.calls().is_empty(), "nothing is saved");
    }

    #[test]
    fn a_scheduled_task_is_set_with_kind_agent_and_changed_only_by_saying_so() {
        let fake = Arc::new(Fake::default());
        let out = call(&fake, "care_set", json!({"text": "总结昨天改过的文件", "kind": "agent", "time": "08:00"})).unwrap();
        assert!(out.starts_with("已添加提醒：总结昨天改过的文件。每天 08:00到点，小伴会在后台去做这件事"), "{out}");
        assert_eq!(fake.calls(), vec!["upsert  agent daily \"08:00\" [] 总结昨天改过的文件 true"]);
        let saved = Rule { id: "task".into(), kind: "agent".into(), text: "看待办".into(), time: "09:00".into(), ..Default::default() };
        fake.rules.lock().unwrap_or_else(|e| e.into_inner()).push(serde_json::to_value(&saved).unwrap());
        assert!(call(&fake, "care_set", json!({"id": "task", "text": "删掉所有文件"})).unwrap_err().contains("kind=agent"), "the confirmation must see it");
        assert!(call(&fake, "care_set", json!({"id": "task", "kind": "custom", "text": "x"})).unwrap_err().contains("kind=agent"));
        assert!(call(&fake, "care_set", json!({"id": "task", "kind": "agent", "time": "10:00"})).unwrap().starts_with("已更新提醒：看待办。"));
        assert!(call(&fake, "care_list", json!({})).unwrap().contains("task | 每天 09:00 | 定时任务：看待办"));
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
        *fake.rules.lock().unwrap_or_else(|e| e.into_inner()) = rules.iter().map(|r| serde_json::to_value(r).unwrap()).collect();
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
        assert!(out.contains("wgt-1 | 桌面时钟 (clock) | 暖白风格 (ivory) | 缩放 1.00 | 显示中"));
        assert!(out.contains("wgt-2 | 天气时钟 (weather) | 暖白风格 (ivory) | 缩放 1.20 | 城市: 上海 | 已隐藏"));
    }

    #[test]
    fn widget_add_validates_kind_and_theme() {
        let fake = Arc::new(Fake::default());
        let out = call(&fake, "widget_add", json!({"kind": "clock", "theme": "ivory"})).unwrap();
        assert_eq!(out, "已在桌面添加组件：桌面时钟（id: wgt-new）");
        assert_eq!(fake.calls(), vec!["widget-add clock", r#"widget-update wgt-new {"theme":"ivory"}"#]);

        assert!(call(&fake, "widget_add", json!({"kind": "unknown"})).unwrap_err().contains("kind 只能是"));
        assert!(call(&fake, "widget_add", json!({"kind": "clock", "theme": "neon"})).unwrap_err().contains("theme 必须是"));
        assert_eq!(fake.calls().len(), 2, "invalid options never create a widget");
    }

    #[test]
    fn widget_tools_match_the_builtin_types_city_schema_and_custom_library() {
        let fake = Arc::new(Fake::default());
        for (kind, label) in [("notes", "待办事项"), ("countdown", "倒数日／纪念日"), ("woodfish", "电子木鱼")] {
            assert!(call(&fake, "widget_add", json!({"kind":kind})).unwrap().contains(label));
        }
        *fake.templates.lock().unwrap_or_else(|e| e.into_inner()) = vec![json!({"id":"my-clock","name":"专注时钟"})];
        assert!(call(&fake, "widget_list", json!({})).unwrap().contains("my-clock | 专注时钟"));
        let all = tools(fake.clone());
        let add = all.iter().find(|tool| tool.spec().name == "widget_add").unwrap().spec();
        assert_eq!(add.schema["properties"]["kind"]["enum"], json!(crate::widgets::KINDS));
        let update = all.iter().find(|tool| tool.spec().name == "widget_update").unwrap().spec();
        assert_eq!(update.schema["properties"]["city"]["required"], json!(["name", "latitude", "longitude"]));
        assert!(parse_widget_patch(&json!({"city":{"name":"上海"}})).is_err());
        assert!(parse_widget_patch(&json!({"city":{"name":"上海","latitude":31.23,"longitude":121.47}})).is_ok());
    }

    #[test]
    fn a_custom_widget_is_added_by_template_id_and_the_error_names_the_missing_parameter() {
        let fake = Arc::new(Fake::default());
        let all = tools(fake.clone());
        let add = all.iter().find(|tool| tool.spec().name == "widget_add").unwrap().spec();
        assert_eq!(add.schema["properties"]["custom_id"]["type"], "string", "the model must be able to see the parameter");
        let missing = call(&fake, "widget_add", json!({"kind":"custom"})).unwrap_err();
        assert!(missing.contains("custom_id") && missing.contains("widget_list"), "{missing}");
        assert!(call(&fake, "widget_add", json!({"kind":"clock","custom_id":"x"})).unwrap_err().contains("只用于 kind=custom"));
        assert!(fake.calls().is_empty(), "nothing is created when the arguments are wrong");
        call(&fake, "widget_add", json!({"kind":"custom","custom_id":"hello-kitty-clock","layer":"topmost"})).unwrap();
        assert_eq!(fake.calls(), vec!["widget-add custom hello-kitty-clock", r#"widget-update wgt-new {"layer":"topmost"}"#]);
    }

    #[test]
    fn widget_add_reports_a_failed_theme_update_instead_of_claiming_success() {
        let fake = Arc::new(Fake::default());
        *fake.fail_widget_update.lock().unwrap_or_else(|e| e.into_inner()) = Some("写入失败".into());
        let error = call(&fake, "widget_add", json!({"kind":"notes","theme":"ivory"})).unwrap_err();
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
    fn app_guide_covers_every_tool_so_the_model_can_find_each_one() {
        let fake: Arc<dyn Desktop> = Arc::new(Fake::default());
        let covered = super::super::app_guide::covered_tools();
        let missing: Vec<String> = tools(fake).iter().map(|t| t.spec().name.clone()).filter(|name| !covered.contains(name.as_str())).collect();
        assert!(missing.is_empty(), "add these tools to chat/app_guide.rs: {missing:?}");
    }

    #[test]
    fn display_settings_switch_the_look_and_pause_the_animation() {
        let fake = Arc::new(Fake::default());
        let out: Value = serde_json::from_str(&call(&fake, "desktop_settings_set", json!({"appearance": "dark", "animationPaused": true})).unwrap()).unwrap();
        assert_eq!((out["appearance"].as_str(), out["animationPaused"].as_bool()), (Some("dark"), Some(true)));
        call(&fake, "desktop_settings_set", json!({"accent": "green"})).unwrap();
        assert_eq!(fake.calls(), vec!["display dark - true", "display - green -"]);
        assert!(call(&fake, "desktop_settings_set", json!({})).unwrap_err().contains("至少"));
        assert!(call(&fake, "desktop_settings_set", json!({"appearance": "neon"})).unwrap_err().contains("appearance 只能是"));
        assert!(call(&fake, "desktop_settings_set", json!({"accent": "pink"})).unwrap_err().contains("accent 只能是"));
        assert_eq!(fake.calls().len(), 2, "a refused call changes nothing");
    }

    #[test]
    fn a_new_reminder_says_how_it_will_reach_the_user() {
        let once = Rule { kind: "custom".into(), repeat: "once".into(), once_at: "2026-10-24T09:57".into(), text: "买花".into(), ..Default::default() };
        let quiet = json!({"enabled": true, "sound": false, "voice": true, "quietEnabled": true, "quietStart": "22:00", "quietEnd": "08:00"});
        let how = how_it_reminds(&once, &quiet);
        assert!(how.starts_with("2026-10-24 09:57到点会在屏幕右上角弹出提醒气泡"), "{how}");
        assert!(how.contains("一直留着") && how.contains("提示音已开、朗读提醒内容已开") && how.contains("安静时段（22:00–08:00）"), "{how}");
        let silenced = how_it_reminds(&once, &json!({"enabled": true, "sound": false, "onceSound": false, "onceVoice": false}));
        assert!(silenced.contains("提示音关、朗读提醒内容关"), "{silenced}");
        assert!(how_it_reminds(&once, &json!({"enabled": true})).contains("提示音已开"), "a setting that is missing counts as the default, on");
        let daily = Rule { time: "15:30".into(), ..Default::default() };
        let how = how_it_reminds(&daily, &json!({"enabled": true}));
        assert!(how.starts_with("每天 15:30") && how.contains("约 10 秒") && !how.contains("一直留着") && !how.contains("安静时段"), "{how}");
        let days = Rule { repeat: "weekdays".into(), weekdays: vec![1, 3, 5], time: "09:00".into(), ..Default::default() };
        assert!(how_it_reminds(&days, &json!({})).starts_with("每周一、三、五的 09:00"));
        let rest = Rule { kind: "rest".into(), interval_minutes: 45, ..Default::default() };
        assert!(how_it_reminds(&rest, &json!({})).starts_with("使用电脑每满 45 分钟"));
        assert!(how_it_reminds(&once, &json!({"enabled": false})).contains("总开关现在是关的"));
    }

    #[test]
    fn open_page_validates_and_app_guide_answers_without_the_app() {
        let fake = Arc::new(Fake::default());
        assert_eq!(call(&fake, "open_page", json!({"page":"settings","tab":"network"})).unwrap(), "已打开 settings 页面的 network");
        assert_eq!(call(&fake, "open_page", json!({"page":"care"})).unwrap(), "已打开 care 页面");
        assert_eq!(fake.calls(), vec!["open-page settings network", "open-page care -"]);
        assert!(call(&fake, "open_page", json!({"page":"chat"})).is_err());
        assert!(call(&fake, "open_page", json!({"page":"studio","tab":"network"})).is_err());
        assert_eq!(fake.calls().len(), 2, "a rejected page opens nothing");
        assert!(call(&fake, "app_guide", json!({"topic":"置顶"})).unwrap().contains("widget_update"));
        assert!(call(&fake, "app_guide", json!({})).unwrap().contains("settings-network"));
    }

    #[test]
    fn widget_content_is_set_on_add_and_on_update_and_listed_with_ids() {
        let fake = Arc::new(Fake::default());
        fake.extra_widgets.lock().unwrap_or_else(|e| e.into_inner()).push(json!({"id": "wgt-3", "kind": "notes", "theme": "ivory", "scale": 1.0, "x": 0.5, "y": 0.1, "enabled": true,
            "note": {"title": "今日待办", "mode": "todo", "text": "", "items": [{"id": "a1", "text": "买花", "done": false, "time": "", "date": "", "notify": false, "repeat": "none", "completedOn": ""}]}}));
        let listed = call(&fake, "widget_list", json!({})).unwrap();
        assert!(listed.contains("[a1] 买花"), "{listed}");

        call(&fake, "widget_update", json!({"id": "wgt-3", "content": {"items": [{"id": "a1", "text": "买花", "done": true}]}})).unwrap();
        let last = fake.calls().last().unwrap().clone();
        assert!(last.starts_with("widget-update wgt-3 ") && last.contains(r#""done":true"#) && last.contains(r#""mode":"todo""#), "{last}");

        call(&fake, "widget_add", json!({"kind": "notes", "content": {"items": ["喝水"]}})).unwrap();
        let last = fake.calls().last().unwrap().clone();
        assert!(last.starts_with("widget-update wgt-new ") && last.contains("喝水"), "{last}");

        let before = fake.calls().len();
        assert!(call(&fake, "widget_add", json!({"kind": "notes", "content": {"items": [{"text": "x", "date": "明天"}]}})).unwrap_err().contains("YYYY-MM-DD"));
        assert!(call(&fake, "widget_add", json!({"kind": "clock", "content": {"title": "x"}})).is_err());
        assert_eq!(fake.calls().len(), before, "a bad content creates nothing");
        assert!(call(&fake, "widget_update", json!({"id": "wgt-1", "content": {"title": "x"}})).unwrap_err().contains("notes"));
        assert!(call(&fake, "widget_update", json!({"id": "nope", "content": {"title": "x"}})).unwrap_err().contains("widget_list"));
        assert!(call(&fake, "widget_update", json!({"id": "wgt-3", "content": {"items": ["x"]}, "layer": "topmost"})).unwrap().contains("wgt-3"));
        assert!(fake.calls().last().unwrap().contains(r#""layer":"topmost""#));
    }

    #[test]
    fn widget_layer_can_be_set_on_add_and_update() {
        let fake = Arc::new(Fake::default());
        call(&fake, "widget_add", json!({"kind":"clock","layer":"topmost"})).unwrap();
        assert_eq!(fake.calls(), vec!["widget-add clock", r#"widget-update wgt-new {"layer":"topmost"}"#]);
        assert!(call(&fake, "widget_add", json!({"kind":"clock","layer":"normal"})).unwrap_err().contains("layer 只能是"));
        call(&fake, "widget_update", json!({"id":"wgt-1","layer":"desktop"})).unwrap();
        assert_eq!(fake.calls().last().unwrap(), r#"widget-update wgt-1 {"layer":"desktop"}"#);
        assert!(call(&fake, "widget_update", json!({"id":"wgt-1","layer":"normal"})).unwrap_err().contains("layer 只能是"));
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
        assert_eq!(lobby_out, "已打开小伴的游戏空间页面");
        let close_out = call(&fake, "game_close", json!({})).unwrap();
        assert_eq!(close_out, "已退出游戏空间或停止桌面层游戏，返回桌面");

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
        let format = call(&fake, "game_read_format", json!({})).unwrap();
        for needle in ["host.init", "game.ready", "game.save", "32 KiB", "xiaban.game-pack", "hostNonce", "ES modules", "desktop-host.js", "source_dir"] {
            assert!(format.contains(needle), "the format mentions {needle}");
        }
        assert!(format.contains("游戏创作手册") && format.contains("安装前自查"), "the craft guide follows the rules");
        let widget = call(&fake, "widget_read_format", json!({})).unwrap();
        assert!(widget.starts_with("自定义小组件规范") && widget.contains("小组件创作手册") && widget.contains("保存前自查"), "{widget}");
        let spec_text = tools(fake.clone()).iter().find(|t| t.spec().name == "game_create").unwrap().spec().description.clone();
        assert!(spec_text.contains("game_read_format") && !spec_text.contains("host.init"), "the long rules are read on demand");
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

        assert!(call(&fake, "wallpaper_delete", json!({})).is_err());
        assert!(call(&fake, "effect_delete", json!({})).is_err());
    }

    #[test]
    fn sharing_a_work_exports_it_then_opens_the_matching_forum_board() {
        let fake = Arc::new(Fake::default());
        *fake.saved.lock().unwrap_or_else(|e| e.into_inner()) = Some(("/Users/a/effect-stars.zip".to_string(), 2048));
        let out = call(&fake, "community_share", json!({"kind": "effect", "id": "stars"})).unwrap();
        assert_eq!(fake.calls(), vec!["export-pack effect stars", "open-url https://chat.xiangban.xyz/c/xiaoban-wallpaper"]);
        assert!(out.contains("/Users/a/effect-stars.zip") && out.contains("小伴壁纸") && out.contains("不能代发"), "{out}");
        assert!(!out.contains("超过 5 MB"));

        call(&fake, "community_share", json!({"kind": "game", "id": "star-catcher"})).unwrap();
        assert_eq!(fake.calls()[3], "open-url https://chat.xiangban.xyz/c/xiaoban-game");
    }

    #[test]
    fn cancelling_the_save_dialog_opens_nothing_and_open_forum_false_stays_in_the_app() {
        let fake = Arc::new(Fake::default());
        let out = call(&fake, "community_share", json!({"kind": "wallpaper", "id": "local-1"})).unwrap();
        assert!(out.contains("取消"), "{out}");
        assert_eq!(fake.calls(), vec!["export-pack wallpaper local-1"]);

        *fake.saved.lock().unwrap_or_else(|e| e.into_inner()) = Some(("/tmp/a.zip".to_string(), 6 * 1024 * 1024));
        let out = call(&fake, "community_share", json!({"kind": "wallpaper", "id": "local-1", "open_forum": false})).unwrap();
        assert!(out.contains("没有打开论坛") && out.contains("超过 5 MB"), "{out}");
        assert_eq!(fake.calls().len(), 2, "no browser call");
    }

    #[test]
    fn sharing_rejects_unknown_kinds_before_touching_the_desktop() {
        let fake = Arc::new(Fake::default());
        assert!(call(&fake, "community_share", json!({"kind": "skin", "id": "x"})).is_err());
        assert!(call(&fake, "community_share", json!({"kind": "effect"})).is_err());
        assert!(fake.calls().is_empty());
    }

    #[test]
    fn community_open_only_knows_the_listed_official_pages() {
        let fake = Arc::new(Fake::default());
        let out = call(&fake, "community_open", json!({"target": "forum_wallpaper"})).unwrap();
        assert!(out.contains("https://chat.xiangban.xyz/c/xiaoban-wallpaper"));
        call(&fake, "community_open", json!({"target": "site"})).unwrap();
        assert_eq!(fake.calls(), vec!["open-url https://chat.xiangban.xyz/c/xiaoban-wallpaper", "open-url https://wallpaper.xiangban.xyz"]);
        assert!(call(&fake, "community_open", json!({"target": "https://evil.example"})).is_err());
        assert_eq!(fake.calls().len(), 2);
    }

    #[test]
    fn desktop_layer_games_are_labelled_and_announced_as_living_on_the_desktop() {
        let fake = Arc::new(Fake::default());
        let list = call(&fake, "game_list", json!({})).unwrap();
        assert!(list.contains("桌面伙伴 (desk-pal)") && list.contains("[自创·桌面层"), "{list}");
        assert!(list.contains("接星星 (star-catcher): 接住落下的星星 [自创]"), "{list}");
        let opened = call(&fake, "game_open", json!({"id": "desk-pal"})).unwrap();
        assert!(opened.contains("桌面层游戏") && opened.contains("game_close"), "{opened}");
        let space = call(&fake, "game_open", json!({"id": "star-catcher"})).unwrap();
        assert_eq!(space, "已进入游戏空间并启动游戏：star-catcher");
    }
}
