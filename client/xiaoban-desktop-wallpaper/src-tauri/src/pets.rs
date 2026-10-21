//! Independent desktop pet instances, backed by declarative theme packs.
use crate::pet_packs::{self, Pet, Theme};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    fs,
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
    time::Duration,
};
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Instance {
    pub id: String,
    pub theme_id: String,
    pub pet_id: String,
    pub name: String,
    pub size: f64,
    pub x: f64,
    pub y: f64,
    pub layer: String,
    pub enabled: bool,
    pub expression: String,
    pub links: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub monitor: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Layout {
    pub version: u32,
    pub instances: Vec<Instance>,
}
impl Default for Layout {
    fn default() -> Self {
        Self {
            version: 1,
            instances: vec![],
        }
    }
}
#[derive(Default)]
pub struct PetState {
    layout: Mutex<Layout>,
    operation: Mutex<()>,
    error: Mutex<Option<String>>,
    active: AtomicBool,
    syncing: AtomicBool,
    applied: Mutex<std::collections::HashMap<String, String>>,
    last_preferences: Mutex<Option<(f64, String, String, String)>>,
}
pub fn active(app: &tauri::AppHandle) -> bool {
    app.state::<PetState>().active.load(Ordering::Relaxed)
}
pub fn is_pet(label: &str) -> bool {
    label == "pet" || label.starts_with("pet-")
}
fn label(p: &Instance) -> String {
    if p.id == "legacy" {
        "pet".into()
    } else {
        format!("pet-{}", p.id)
    }
}
pub fn permitted(label: &str, command: &str) -> Option<bool> {
    if label == "companion-greeting" {
        return Some(["get_companion_greeting", "respond_companion_greeting"].contains(&command));
    }
    if is_pet(label) {
        return Some(
            [
                "get_pet_instance",
                "move_pet_instance",
                "set_pet_hit_test",
                "manage_pet_instance",
                "pet_proactive_hover",
                "pet_proactive_poke",
                "scene_ready",
            ]
            .contains(&command),
        );
    }
    if [
        "get_pet_catalog",
        "add_pet",
        "update_pet",
        "remove_pet",
        "save_pet_theme",
        "delete_pet_theme",
        "export_pet_theme",
        "get_proactivity_settings",
        "configure_proactivity",
    ]
    .contains(&command)
    {
        return Some(label == "control");
    }
    if [
        "get_pet_instance",
        "move_pet_instance",
        "set_pet_hit_test",
        "manage_pet_instance",
        "pet_proactive_hover",
        "pet_proactive_poke",
        "get_companion_greeting",
        "respond_companion_greeting",
    ]
    .contains(&command)
    {
        return Some(false);
    }
    None
}
pub fn validate(layout: &Layout) -> Result<(), String> {
    if layout.version != 1 {
        return Err("宠物布局版本不受支持".into());
    }
    let mut ids = std::collections::HashSet::new();
    for p in &layout.instances {
        if !pet_packs::valid_id(&p.id)
            || !ids.insert(&p.id)
            || !pet_packs::valid_id(&p.theme_id)
            || !pet_packs::valid_id(&p.pet_id)
            || p.name.trim().is_empty()
            || p.name.encode_utf16().count() > 40
            || !p.size.is_finite()
            || !(80.0..=420.).contains(&p.size)
            || ![p.x, p.y]
                .iter()
                .all(|n| n.is_finite() && (0.0..=1.).contains(n))
            || !["desktop", "topmost"].contains(&p.layer.as_str())
            || !crate::preferences::EXPRESSIONS.contains(&p.expression.as_str())
            || p.monitor
                .as_ref()
                .is_some_and(|s| s.is_empty() || s.len() > 512)
        {
            return Err("宠物实例参数无效".into());
        }
    }
    Ok(())
}
fn dir(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    Ok(crate::storage::root(app)?.join("pets/themes"))
}
pub fn themes(app: &tauri::AppHandle) -> Result<Vec<Theme>, String> {
    let mut list = pet_packs::builtin();
    list.extend(pet_packs::load(&dir(app)?)?);
    Ok(list)
}
fn definition(app: &tauri::AppHandle, p: &Instance) -> Result<Pet, String> {
    themes(app)?
        .into_iter()
        .find(|t| t.id == p.theme_id)
        .and_then(|t| t.pets.into_iter().find(|d| d.id == p.pet_id))
        .ok_or_else(|| format!("没有找到宠物作品 {} / {}", p.theme_id, p.pet_id))
}
fn persist(app: &tauri::AppHandle, layout: &Layout) -> Result<(), String> {
    validate(layout)?;
    if let Some(error) = app.state::<PetState>().error.lock().unwrap().clone() {
        return Err(error);
    }
    let bytes = serde_json::to_vec_pretty(layout).map_err(|e| e.to_string())?;
    if bytes.len() > 512 * 1024 {
        return Err("宠物布局文件过大".into());
    }
    let root = crate::storage::root(app)?.join("config");
    fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    let temp = root.join("pets.tmp");
    fs::write(&temp, bytes).map_err(|e| e.to_string())?;
    fs::rename(temp, root.join("pets.json")).map_err(|e| e.to_string())
}
fn mutate(
    app: &tauri::AppHandle,
    change: impl FnOnce(&mut Layout) -> Result<(), String>,
) -> Result<Value, String> {
    {
        let state = app.state::<PetState>();
        let _operation = state.operation.lock().unwrap();
        let mut next = state.layout.lock().unwrap().clone();
        change(&mut next)?;
        persist(app, &next)?;
        *state.layout.lock().unwrap() = next;
    }
    sync(app);
    let data = catalog(app)?;
    publish_catalog(app, &data);
    Ok(data)
}
fn publish_catalog(app: &tauri::AppHandle, data: &Value) {
    if let Some(w) = app.get_webview_window("control") {
        let _ = w.eval(format!(
            "window.dispatchEvent(new CustomEvent('pets-changed',{{detail:{data}}}));"
        ));
    }
}
pub fn catalog(app: &tauri::AppHandle) -> Result<Value, String> {
    let state = app.state::<PetState>();
    let mut errors = state
        .error
        .lock()
        .unwrap()
        .clone()
        .into_iter()
        .collect::<Vec<_>>();
    let all = match themes(app) {
        Ok(t) => t,
        Err(e) => {
            errors.push(e);
            pet_packs::builtin()
        }
    };
    let builtin = pet_packs::builtin();
    let list: Vec<_> = all
        .into_iter()
        .map(|t| {
            let source = if builtin.iter().any(|b| b.id == t.id) {
                "builtin"
            } else {
                "custom"
            };
            let mut value = json!(t);
            value["source"] = json!(source);
            value
        })
        .collect();
    Ok(json!({"themes":list,"layout":state.layout.lock().unwrap().clone(),"errors":errors}))
}
pub fn window_layer(app: &tauri::AppHandle, window_label: &str) -> Option<String> {
    if !active(app) {
        return None;
    }
    app.state::<PetState>()
        .layout
        .lock()
        .unwrap()
        .instances
        .iter()
        .find(|p| label(p) == window_label)
        .map(|p| p.layer.clone())
}
pub(crate) fn own(window: &tauri::WebviewWindow) -> Result<Instance, String> {
    window
        .app_handle()
        .state::<PetState>()
        .layout
        .lock()
        .unwrap()
        .instances
        .iter()
        .find(|p| label(p) == window.label())
        .cloned()
        .ok_or("没有找到这个桌面宠物".into())
}
fn geometry(app: &tauri::AppHandle, p: &Instance) -> Result<(f64, f64, f64, f64), String> {
    let monitors = crate::widgets::monitors(app)?;
    let m =
        crate::widget_geometry::resolve(&monitors, p.monitor.as_deref()).ok_or("没有找到显示器")?;
    let size = (p.size + 16.) * m.scale;
    let (x, y) = crate::widget_geometry::position(m, size, size, p.x, p.y);
    Ok((x, y, size, m.scale))
}
fn payload(app: &tauri::AppHandle, p: &Instance) -> Result<Value, String> {
    let s = crate::settings(app);
    let (_, _, _, scale) = geometry(app, p)?;
    Ok(
        json!({"instance":p,"definition":definition(app,p)?,"paused":s.paused,"fps":s.fps,"links":s.companion_links,"effect":s.effects.selected,"displayScale":if cfg!(windows){Some(scale)}else{None}}),
    )
}
fn place(
    app: &tauri::AppHandle,
    window: &tauri::WebviewWindow,
    p: &Instance,
) -> Result<(), String> {
    let (x, y, size, _) = geometry(app, p)?;
    #[cfg(target_os = "macos")]
    return crate::macos::place_widget(window, x, y, size, size);
    #[cfg(windows)]
    {
        if p.layer == "desktop" {
            return crate::windows::place_widget(
                window,
                tauri::PhysicalPosition::new(x.round() as i32, y.round() as i32),
                size.round() as i32,
                size.round() as i32,
            );
        }
        crate::windows::place_interactive(window, x, y, size, size)
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        window
            .set_size(tauri::PhysicalSize::new(size as u32, size as u32))
            .map_err(|e| e.to_string())?;
        window
            .set_position(tauri::PhysicalPosition::new(x as i32, y as i32))
            .map_err(|e| e.to_string())
    }
}
pub fn sync(app: &tauri::AppHandle) {
    let state = app.state::<PetState>();
    if !active(app) || state.syncing.swap(true, Ordering::SeqCst) {
        return;
    }
    let all = state.layout.lock().unwrap().instances.clone();
    let s = crate::settings(app);
    let hidden = !s.companion_enabled || crate::games::is_active(app);
    for (key, w) in app.webview_windows() {
        if is_pet(&key) && !all.iter().any(|p| label(p) == key && p.enabled) {
            let _ = w.destroy();
            state.applied.lock().unwrap().remove(&key);
        }
    }
    for p in all.iter().filter(|p| p.enabled) {
        let result = (|| -> Result<(), String> {
            if hidden {
                if let Some(w) = app.get_webview_window(&label(p)) {
                    let _ = w.hide();
                }
                return Ok(());
            }
            let data = match payload(app, p) {
                Ok(data) => data,
                Err(error) => {
                    if let Some(w) = app.get_webview_window(&label(p)) {
                        let _ = w.destroy();
                    }
                    state.applied.lock().unwrap().remove(&label(p));
                    return Err(error);
                }
            };
            let signature = format!("{data}:{:?}", geometry(app, p)?);
            if state.applied.lock().unwrap().get(&label(p)) == Some(&signature)
                && app
                    .get_webview_window(&label(p))
                    .is_some_and(|w| w.is_visible().unwrap_or(false))
            {
                return Ok(());
            }
            let window = if let Some(w) = app.get_webview_window(&label(p)) {
                w
            } else {
                let (x, y, size, scale) = geometry(app, p)?;
                let builder = WebviewWindowBuilder::new(
                    app,
                    label(p),
                    WebviewUrl::App(format!("index.html?scene=pet-instance&id={}", p.id).into()),
                )
                .title(format!("小伴 · {}", p.name))
                .decorations(false)
                .shadow(false)
                .transparent(true)
                .resizable(false)
                .visible(false)
                .focused(false)
                .skip_taskbar(true)
                .inner_size(size / scale, size / scale)
                .position(x / scale, y / scale)
                .initialization_script(format!("window.__XIAOBAN_PET__={data};"));
                #[cfg(target_os = "macos")]
                let builder = builder.incognito(true).background_throttling(
                    tauri::utils::config::BackgroundThrottlingPolicy::Disabled,
                );
                #[cfg(windows)]
                let builder =
                    builder.data_directory(crate::storage::webview(&crate::storage::root(app)?));
                let w = builder.build().map_err(|e| e.to_string())?;
                w.set_ignore_cursor_events(true)
                    .map_err(|e| e.to_string())?;
                w
            };
            crate::window_control::apply_layer(&window, &p.layer)?;
            place(app, &window, p)?;
            window
                .eval(format!(
                    "window.dispatchEvent(new CustomEvent('pet-config',{{detail:{data}}}));"
                ))
                .map_err(|e| e.to_string())?;
            window
                .set_title(&format!("小伴 · {}", p.name))
                .map_err(|e| e.to_string())?;
            window.show().map_err(|e| e.to_string())?;
            crate::window_control::apply_layer(&window, &p.layer)?;
            state.applied.lock().unwrap().insert(label(p), signature);
            Ok(())
        })();
        if let Err(e) = result {
            eprintln!("宠物 {}：{e}", p.name);
        }
    }
    state.syncing.store(false, Ordering::SeqCst);
}
pub fn sync_preferences(app: &tauri::AppHandle) {
    if !active(app) {
        return;
    }
    let s = crate::settings(app);
    let current = (
        s.size,
        s.expression.clone(),
        s.companion_skin.clone(),
        s.mode.clone(),
    );
    let state = app.state::<PetState>();
    let previous = state
        .last_preferences
        .lock()
        .unwrap()
        .replace(current.clone());
    if let Some(old) = previous {
        if old != current {
            let _ = mutate(app, |layout| {
                if let Some(p) = layout.instances.iter_mut().find(|p| p.id == "legacy") {
                    if old.0 != current.0 {
                        p.size = current.0;
                    }
                    if old.1 != current.1 {
                        p.expression = current.1.clone();
                    }
                    if old.2 != current.2 {
                        p.theme_id = "flower-whispers".into();
                        p.pet_id = if current.2 == "pot-white-flower" {
                            "daisy"
                        } else {
                            "green-sprout"
                        }
                        .into();
                    }
                    if old.3 != current.3 {
                        p.layer = if current.3 == "wallpaper" {
                            "desktop"
                        } else {
                            "topmost"
                        }
                        .into();
                    }
                }
                Ok(())
            });
        }
    }
    sync(app);
}
pub fn start(app: &tauri::AppHandle) {
    let init = (|| -> Result<(), String> {
        themes(app)?;
        let root = crate::storage::root(app)?;
        let path = root.join("config/pets.json");
        let s = crate::settings(app);
        let layout = if path.exists() {
            let meta = fs::metadata(&path).map_err(|e| e.to_string())?;
            if meta.len() > 512 * 1024 {
                return Err("宠物布局过大，原文件已保留".into());
            }
            serde_json::from_slice::<Layout>(&fs::read(&path).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?
        } else {
            Layout {
                version: 1,
                instances: vec![Instance {
                    id: "legacy".into(),
                    theme_id: "flower-whispers".into(),
                    pet_id: if s.companion_skin == "pot-white-flower" {
                        "daisy"
                    } else {
                        "green-sprout"
                    }
                    .into(),
                    name: "小伴".into(),
                    size: s.size,
                    x: 0.9,
                    y: 0.85,
                    layer: if s.mode == "wallpaper" {
                        "desktop"
                    } else {
                        "topmost"
                    }
                    .into(),
                    enabled: s.companion_enabled,
                    expression: s.expression.clone(),
                    links: true,
                    monitor: None,
                }],
            }
        };
        validate(&layout)?;
        if !path.exists() {
            persist(app, &layout)?;
        }
        let state = app.state::<PetState>();
        *state.layout.lock().unwrap() = layout;
        *state.last_preferences.lock().unwrap() =
            Some((s.size, s.expression, s.companion_skin, s.mode));
        state.active.store(true, Ordering::Relaxed);
        Ok(())
    })();
    if let Err(error) = init {
        let state = app.state::<PetState>();
        *state.error.lock().unwrap() = Some(error);
        state.active.store(true, Ordering::Relaxed);
        return;
    }
    sync(app);
    let handle = app.clone();
    std::thread::spawn(move || {
        let mut ticks = 0;
        let mut previous_cursor = None;
        loop {
            std::thread::sleep(Duration::from_millis(33));
            ticks += 1;
            if ticks % 90 == 0 {
                sync(&handle);
            }
            if let Ok((x, y)) = crate::widgets::cursor(&handle) {
                if previous_cursor == Some((x, y)) && ticks % 8 != 0 { continue; }
                previous_cursor = Some((x, y));
                for (key, w) in handle.webview_windows() {
                    if !is_pet(&key) || !w.is_visible().unwrap_or(false) {
                        continue;
                    }
                    let Ok((wx, wy, _, _)) = crate::window_control::frame(&w) else {
                        continue;
                    };
                    let ratio = if cfg!(windows) {
                        w.scale_factor().unwrap_or(1.)
                    } else {
                        1.
                    };
                    let data = json!({"x":(x-wx)/ratio,"y":(y-wy)/ratio});
                    let _ = w.eval(format!(
                        "window.dispatchEvent(new CustomEvent('pet-pointer',{{detail:{data}}}));"
                    ));
                }
            }
        }
    });
}
pub fn add(
    app: &tauri::AppHandle,
    theme_id: String,
    pet_id: String,
    layer: Option<String>,
) -> Result<Value, String> {
    let def = themes(app)?
        .into_iter()
        .find(|t| t.id == theme_id)
        .and_then(|t| t.pets.into_iter().find(|p| p.id == pet_id))
        .ok_or("没有找到这个主题内的宠物")?;
    let mut random = [0u8; 8];
    getrandom::fill(&mut random).map_err(|e| e.to_string())?;
    let id = random
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    mutate(app, |l| {
        let index = l.instances.len();
        l.instances.push(Instance {
            id: id.clone(),
            theme_id,
            pet_id,
            name: def.name,
            size: 210.,
            x: (0.1 + (index % 4) as f64 * 0.23).min(0.9),
            y: (0.15 + (index / 4) as f64 * 0.18).min(0.85),
            layer: layer.unwrap_or("topmost".into()),
            enabled: true,
            expression: "idle".into(),
            links: true,
            monitor: None,
        });
        Ok(())
    })?;
    Ok(json!({"id":id}))
}
#[derive(Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Patch {
    pub name: Option<String>,
    pub size: Option<f64>,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub layer: Option<String>,
    pub enabled: Option<bool>,
    pub expression: Option<String>,
    pub links: Option<bool>,
    pub monitor: Option<String>,
    pub theme_id: Option<String>,
    pub pet_id: Option<String>,
}
pub fn update(app: &tauri::AppHandle, id: &str, patch: Patch) -> Result<Value, String> {
    mutate(app, |l| {
        let p = l
            .instances
            .iter_mut()
            .find(|p| p.id == id)
            .ok_or("宠物已被移除")?;
        if let Some(v) = patch.name {
            p.name = v;
        }
        if let Some(v) = patch.size {
            p.size = v;
        }
        if let Some(v) = patch.x {
            p.x = v;
        }
        if let Some(v) = patch.y {
            p.y = v;
        }
        if let Some(v) = patch.layer {
            p.layer = v;
        }
        if let Some(v) = patch.enabled {
            p.enabled = v;
        }
        if let Some(v) = patch.expression {
            p.expression = v;
        }
        if let Some(v) = patch.links {
            p.links = v;
        }
        if let Some(v) = patch.monitor {
            p.monitor = Some(v);
        }
        if let Some(v) = patch.theme_id {
            p.theme_id = v;
        }
        if let Some(v) = patch.pet_id {
            p.pet_id = v;
        }
        definition(app, p)?;
        Ok(())
    })
}
pub fn remove(app: &tauri::AppHandle, id: &str) -> Result<Value, String> {
    mutate(app, |l| {
        if !l.instances.iter().any(|p| p.id == id) {
            return Err("没有找到这个宠物".into());
        }
        l.instances.retain(|p| p.id != id);
        Ok(())
    })
}
pub fn save_theme(app: &tauri::AppHandle, theme: Theme, overwrite: bool) -> Result<Value, String> {
    let state = app.state::<PetState>();
    {
        let _op = state.operation.lock().unwrap();
        let current = state.layout.lock().unwrap().clone();
        if current
            .instances
            .iter()
            .any(|p| p.theme_id == theme.id && !theme.pets.iter().any(|d| d.id == p.pet_id))
        {
            return Err("主题仍有宠物放在桌面上，不能删除其作品定义".into());
        }
        pet_packs::save(&dir(app)?, &theme, overwrite)?;
    }
    sync(app);
    let data = catalog(app)?;
    publish_catalog(app, &data);
    Ok(data)
}
#[tauri::command]
pub fn get_pet_catalog(app: tauri::AppHandle) -> Result<Value, String> {
    catalog(&app)
}
#[tauri::command]
pub fn add_pet(
    app: tauri::AppHandle,
    theme_id: String,
    pet_id: String,
    layer: Option<String>,
) -> Result<Value, String> {
    add(&app, theme_id, pet_id, layer)
}
#[tauri::command]
pub fn update_pet(app: tauri::AppHandle, id: String, patch: Patch) -> Result<Value, String> {
    update(&app, &id, patch)
}
#[tauri::command]
pub fn remove_pet(app: tauri::AppHandle, id: String) -> Result<Value, String> {
    remove(&app, &id)
}
#[tauri::command]
pub fn save_pet_theme(
    app: tauri::AppHandle,
    theme: Theme,
    overwrite: Option<bool>,
) -> Result<Value, String> {
    save_theme(&app, theme, overwrite.unwrap_or(false))
}
#[tauri::command]
pub fn delete_pet_theme(app: tauri::AppHandle, id: String) -> Result<Value, String> {
    if !pet_packs::valid_id(&id) || pet_packs::builtin().iter().any(|t| t.id == id) {
        return Err("不能删除官方主题".into());
    }
    {
        let state = app.state::<PetState>();
        let _op = state.operation.lock().unwrap();
        if state
            .layout
            .lock()
            .unwrap()
            .instances
            .iter()
            .any(|p| p.theme_id == id)
        {
            return Err("请先移除该主题已添加的桌面宠物".into());
        }
        fs::remove_file(dir(&app)?.join(format!("{id}.json"))).map_err(|e| e.to_string())?;
    }
    let data = catalog(&app)?;
    publish_catalog(&app, &data);
    Ok(data)
}
#[tauri::command]
pub fn get_pet_instance(window: tauri::WebviewWindow) -> Result<Value, String> {
    payload(window.app_handle(), &own(&window)?)
}
#[tauri::command]
pub fn set_pet_hit_test(window: tauri::WebviewWindow, hit: bool) -> Result<(), String> {
    own(&window)?;
    window
        .set_ignore_cursor_events(!hit)
        .map_err(|e| e.to_string())
}
#[tauri::command]
pub fn manage_pet_instance(window: tauri::WebviewWindow) -> Result<(), String> {
    let p = own(&window)?;
    crate::present_control(window.app_handle())?;
    if let Some(control) = window.app_handle().get_webview_window("control") {
        control
            .eval(format!(
                "window.dispatchEvent(new CustomEvent('pets-manage',{{detail:{}}}));",
                json!({"id":p.id})
            ))
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
#[tauri::command]
pub fn move_pet_instance(
    window: tauri::WebviewWindow,
    grab_x: f64,
    grab_y: f64,
    commit: bool,
) -> Result<(), String> {
    if ![grab_x, grab_y]
        .iter()
        .all(|n| n.is_finite() && (0.0..=1.).contains(n))
    {
        return Err("抓取点无效".into());
    }
    let mut p = own(&window)?;
    let app = window.app_handle();
    let point = crate::widgets::cursor(app)?;
    let areas = crate::widgets::monitors(app)?;
    let m = crate::widget_geometry::at_point(&areas, point.0, point.1).ok_or("没有找到显示器")?;
    let size = (p.size + 16.) * m.scale;
    let (x, y) = crate::widget_geometry::drag_position(m, size, size, point, (grab_x, grab_y));
    p.x = x;
    p.y = y;
    p.monitor = Some(m.id.clone());
    if commit {
        update(
            app,
            &p.id,
            Patch {
                x: Some(x),
                y: Some(y),
                monitor: p.monitor,
                ..Patch::default()
            },
        )?;
    } else {
        place(app, &window, &p)?;
    }
    Ok(())
}
#[tauri::command]
pub async fn export_pet_theme(app: tauri::AppHandle, id: String) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let theme = themes(&app)?
        .into_iter()
        .find(|t| t.id == id)
        .ok_or("没有找到这个主题")?;
    tauri::async_runtime::spawn_blocking(move || {
        let file = app
            .dialog()
            .file()
            .set_file_name(format!("{}.json", theme.id))
            .add_filter("宠物主题", &["json"])
            .blocking_save_file();
        let Some(file) = file else { return Ok(None) };
        let path = file.into_path().map_err(|e| e.to_string())?;
        fs::write(
            &path,
            serde_json::to_vec_pretty(&theme).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        Ok(Some(path.to_string_lossy().into_owned()))
    })
    .await
    .map_err(|e| e.to_string())?
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn multiple_pets_round_trip_without_shared_state() {
        let a = Instance {
            id: "one".into(),
            theme_id: "flower-whispers".into(),
            pet_id: "daisy".into(),
            name: "花".into(),
            size: 180.,
            x: 0.1,
            y: 0.2,
            layer: "desktop".into(),
            enabled: true,
            expression: "idle".into(),
            links: true,
            monitor: None,
        };
        let mut b = a.clone();
        b.id = "two".into();
        b.expression = "sleeping".into();
        b.size = 300.;
        let layout = Layout {
            version: 1,
            instances: vec![a, b],
        };
        validate(&layout).unwrap();
        let mut restored: Layout =
            serde_json::from_slice(&serde_json::to_vec(&layout).unwrap()).unwrap();
        restored.instances[0].expression = "happy".into();
        assert_eq!(restored.instances[1].expression, "sleeping");
        assert_eq!(restored.instances[1].size, 300.);
    }
    #[test]
    fn pet_windows_cannot_mutate_other_instances_or_themes() {
        assert_eq!(permitted("pet-one", "save_pet_theme"), Some(false));
        assert_eq!(permitted("pet-one", "update_pet"), Some(false));
        assert_eq!(permitted("pet-one", "get_pet_instance"), Some(true));
        assert_eq!(permitted("wallpaper", "get_pet_instance"), Some(false));
        assert_eq!(permitted("control", "save_pet_theme"), Some(true));
    }
}
