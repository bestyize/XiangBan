//! Window operations for the app's own interactive surfaces, never arbitrary OS windows.
use serde::Deserialize;
use serde_json::{json, Value};
use std::{collections::HashMap, sync::Mutex};
use tauri::Manager;

#[derive(Default)]
pub struct WindowState {
    layers: Mutex<HashMap<String, String>>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Change {
    pub target: String,
    pub action: Option<String>,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub monitor: Option<String>,
    pub layer: Option<String>,
}

impl Change {
    pub fn validate(&self) -> Result<String, String> {
        let label = match self.target.as_str() {
            "companion" => "pet".to_string(),
            "assistant" | "control" => self.target.clone(),
            id if id.starts_with("widget:") && id[7..].len() <= 32 && !id[7..].is_empty()
                && id[7..].bytes().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()) => format!("widget-{}", &id[7..]),
            _ => return Err("target 必须是 companion（盆栽）、assistant（超级小伴）、control（控制台）或 widget:<组件id>".into()),
        };
        if self
            .action
            .as_deref()
            .is_some_and(|a| !["show", "hide", "collapse"].contains(&a))
        {
            return Err("action 只能是 show、hide、collapse".into());
        }
        if self.action.as_deref() == Some("collapse") && label != "assistant" {
            return Err("collapse 只适用于超级小伴".into());
        }
        if label.starts_with("widget-")&&self.layer.as_deref()==Some("normal"){return Err("组件层级只能是 desktop（贴在桌面）或 topmost（始终置顶）".into());}
        if self
            .layer
            .as_deref()
            .is_some_and(|v| !["desktop", "normal", "topmost"].contains(&v))
        {
            return Err("layer 只能是 desktop、normal、topmost".into());
        }
        if label == "control" && self.layer.as_deref() == Some("desktop") {
            return Err("控制台不能嵌入桌面层".into());
        }
        if self.x.is_some() != self.y.is_some()
            || [self.x, self.y]
                .into_iter()
                .flatten()
                .any(|v| !v.is_finite() || !(0.0..=1.0).contains(&v))
        {
            return Err("移动时需同时提供 x、y，范围 0 到 1（左上到右下，0.5 为居中）".into());
        }
        if self
            .monitor
            .as_ref()
            .is_some_and(|s| s.is_empty() || s.len() > 512)
            || (self.monitor.is_some() && self.x.is_none())
        {
            return Err("monitor 需来自 desktop_windows，并同时提供 x、y".into());
        }
        if self.action.is_none() && self.x.is_none() && self.layer.is_none() {
            return Err("请指定 action、位置或 layer".into());
        }
        if matches!(self.action.as_deref(), Some("hide" | "collapse"))
            && (self.x.is_some() || self.layer.is_some())
        {
            return Err("隐藏或收起不能同时移动或调整层级".into());
        }
        Ok(label)
    }
}

pub fn layer(app: &tauri::AppHandle, label: &str, default: &str) -> String {
    if let Some(layer)=crate::pets::window_layer(app,label){return layer;}
    if let Some(layer)=crate::widgets::window_layer(app,label){return layer;}

    app.state::<WindowState>()
        .layers
        .lock()
        .unwrap()
        .get(label)
        .cloned()
        .unwrap_or_else(|| default.into())
}

pub fn apply_saved(
    app: &tauri::AppHandle,
    window: &tauri::WebviewWindow,
    default: &str,
) -> Result<(), String> {
    apply_layer(window, &layer(app, window.label(), default))
}

pub(crate) fn apply_layer(window: &tauri::WebviewWindow, layer: &str) -> Result<(), String> {
    #[cfg(windows)]
    {
        if window.label() != "control" {
            if layer == "desktop" && (crate::widgets::is_widget(window.label()) || crate::pets::is_pet(window.label())) {
                window.set_always_on_top(false).map_err(|e|e.to_string())?;
                return crate::windows::attach_widget(window);
            }
            crate::windows::configure_companion(window, layer == "desktop")?;
        }
    }
    // On macOS the framework flag and the final AppKit level must be set in
    // one main-thread operation. A queued "not topmost" update sets level 0.
    #[cfg(target_os = "macos")]
    { crate::macos::set_interactive_layer(window, layer) }
    #[cfg(not(target_os = "macos"))]
    { window.set_always_on_top(layer == "topmost").map_err(|e| e.to_string()) }

}

pub(crate) fn frame(window: &tauri::WebviewWindow) -> Result<(f64, f64, f64, f64), String> {
    #[cfg(target_os = "macos")]
    {
        return crate::macos::interactive_frame(window);
    }
    #[cfg(not(target_os = "macos"))]
    {
        let p = window.outer_position().map_err(|e| e.to_string())?;
        let s = window.outer_size().map_err(|e| e.to_string())?;
        Ok((p.x as f64, p.y as f64, s.width as f64, s.height as f64))
    }
}

pub fn place(
    window: &tauri::WebviewWindow,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        return crate::macos::place_widget(window, x, y, width, height);
    }
    #[cfg(windows)]
    {
        return crate::windows::place_interactive(window, x, y, width, height);
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        window
            .set_position(tauri::PhysicalPosition::new(x, y))
            .map_err(|e| e.to_string())
    }
}

pub fn snapshot(app: &tauri::AppHandle) -> Result<Value, String> {
    let areas = crate::widgets::monitors(app)?;
    let monitors: Vec<_> = areas.iter().map(|m| json!({"id":m.id,"name":m.name,"width":m.width/m.scale,"height":m.height/m.scale,"primary":m.primary})).collect();
    let settings = crate::settings(app);
    let mut windows = vec![];
    for (label, name, default) in [
        (
            "pet",
            "盆栽小伴",
            if settings.mode == "wallpaper" {
                "desktop"
            } else {
                "topmost"
            },
        ),
        ("assistant", "超级小伴", "topmost"),
        ("control", "控制台", "normal"),
    ] {
        let window = app.get_webview_window(label);
        let bounds = window
            .as_ref()
            .and_then(|w| frame(w).ok())
            .map(|(x, y, width, height)| json!({"x":x,"y":y,"width":width,"height":height}));
        windows.push(json!({"target":if label=="pet"{"companion"}else{label},"name":name,"visible":window.as_ref().is_some_and(|w|w.is_visible().unwrap_or(false)),"layer":layer(app,label,default),"bounds":bounds}));
    }
    let widgets = crate::widgets::get_desktop_widgets(app.clone())?;
    for w in widgets["widgets"].as_array().into_iter().flatten() {
        let id = w["id"].as_str().unwrap_or_default();
        windows.push(json!({"target":format!("widget:{id}"),"kind":w["kind"],"visible":w["enabled"],"x":w["x"],"y":w["y"],"monitor":w["monitor"],"layer":w["layer"].as_str().unwrap_or("desktop")}));
    }
    Ok(
        json!({"monitors":monitors,"windows":windows,"coordinates":"移动使用屏内比例 0..1；bounds 为原生屏幕坐标。组件层级与位置会保存；其他窗口层级本次运行有效。壁纸和特效层不支持任意移动。"}),
    )
}

pub fn update(app: &tauri::AppHandle, change: &Change) -> Result<Value, String> {
    let label = change.validate()?;
    if let Some(id) = change.monitor.as_deref() {
        if !crate::widgets::monitors(app)?.iter().any(|m| m.id == id) {
            return Err("没有找到指定显示器，请重新调用 desktop_windows".into());
        }
    }
    if let Some(id) = label.strip_prefix("widget-") {
        let mut patch = json!({});
        if let Some(layer)=&change.layer{patch["layer"]=json!(layer);}
        if let Some(action) = change.action.as_deref() {
            patch["enabled"] = json!(action == "show");
        }
        if let (Some(x), Some(y)) = (change.x, change.y) {
            patch["x"] = json!(x);
            patch["y"] = json!(y);
        }
        if let Some(monitor) = &change.monitor {
            patch["monitor"] = json!(monitor);
        }
        if patch.as_object().is_some_and(|p| !p.is_empty()) {
            let patch = serde_json::from_value(patch).map_err(|e| format!("窗口配置无效：{e}"))?;
            let result = tauri::async_runtime::block_on(crate::widgets::update_desktop_widget(
                app.clone(),
                id.into(),
                patch,
            ))?;
            if let Some(errors) = result["errors"]
                .as_array()
                .filter(|errors| !errors.is_empty())
            {
                return Err(format!("布局已保存，但窗口应用失败：{}", json!(errors)));
            }
        }
        // A widget's layer is written to its saved layout, so it is still there after a restart.
        return Ok(json!({"ok":true,"target":change.target,"action":change.action,"x":change.x,"y":change.y,"layer":change.layer,"saved":true}));
    } else if let Some(action) = change.action.as_deref() {
        match label.as_str() {
            "assistant" => match action {
                "show" => crate::float_assistant::expand(app, false)?,
                "hide" => crate::float_assistant::hide_surface(app)?,
                _ => crate::float_assistant::collapse(app)?,
            },
            "pet" => {
                tauri::async_runtime::block_on(crate::set_companion_enabled(
                    app.clone(),
                    action == "show",
                ))?;
            }
            "control" => {
                let w = app
                    .get_webview_window("control")
                    .ok_or("控制台窗口不存在")?;
                if action == "show" { w.show() } else { w.hide() }.map_err(|e| e.to_string())?;
            }
            _ => unreachable!(),
        }
    }
    if change.x.is_some() || change.layer.is_some() {
        let window = app
            .get_webview_window(&label)
            .ok_or("窗口尚未显示，请先 show 再调整位置或层级")?;
        if let Some(layer) = &change.layer {
            apply_layer(&window, layer)?;
            app.state::<WindowState>()
                .layers
                .lock()
                .unwrap()
                .insert(label.clone(), layer.clone());
        }
        if !crate::widgets::is_widget(&label) {
            if let (Some(x), Some(y)) = (change.x, change.y) {
                let areas = crate::widgets::monitors(app)?;
                let (old_x, old_y, width, height) = frame(&window)?;
                let monitor = if let Some(id) = change.monitor.as_deref() {
                    crate::widget_geometry::resolve(&areas, Some(id))
                } else {
                    crate::widget_geometry::at_point(
                        &areas,
                        old_x + width / 2.0,
                        old_y + height / 2.0,
                    )
                    .or_else(|| crate::widget_geometry::resolve(&areas, None))
                }
                .ok_or("没有找到显示器")?;
                #[cfg(not(target_os = "macos"))]
                let (width, height) = {
                    let dpi = window.scale_factor().map_err(|e| e.to_string())?;
                    (width / dpi * monitor.scale, height / dpi * monitor.scale)
                };
                let (px, py) = crate::widget_geometry::position(monitor, width, height, x, y);
                place(&window, px, py, width, height)?;
                if label == "assistant" {
                    crate::float_assistant::remember_position(app, &window);
                }
            }
        } else {
            crate::widgets::sync(app);
        }
    }
    // Other windows keep the layer only until the app quits.
    Ok(
        json!({"ok":true,"target":change.target,"action":change.action,"x":change.x,"y":change.y,"layer":change.layer,"saved":change.layer.is_none(),"layerLastsUntilQuit":change.layer.is_some()}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]fn widget_layers_are_persistable_modes(){
        for layer in ["desktop","topmost"]{assert!(serde_json::from_value::<Change>(json!({"target":"widget:abc123","layer":layer})).unwrap().validate().is_ok());}
        assert!(serde_json::from_value::<Change>(json!({"target":"widget:abc123","layer":"normal"})).unwrap().validate().is_err());
        assert!(serde_json::from_value::<Change>(json!({"target":"assistant","layer":"normal"})).unwrap().validate().is_ok());
    }
    #[test]
    fn only_app_surfaces_and_valid_normalized_positions_are_accepted() {
        for value in [
            json!({"target":"companion","x":0.0,"y":1.0}),
            json!({"target":"assistant","action":"collapse"}),
            json!({"target":"widget:abc123","layer":"topmost"}),
        ] {
            assert!(serde_json::from_value::<Change>(value)
                .unwrap()
                .validate()
                .is_ok());
        }
        for value in [
            json!({"target":"wallpaper","action":"hide"}),
            json!({"target":"widget:../file","action":"hide"}),
            json!({"target":"companion","x":2,"y":0}),
            json!({"target":"companion","x":0}),
            json!({"target":"control","layer":"desktop"}),
            json!({"target":"companion","action":"collapse"}),
            json!({"target":"assistant","action":"hide","x":0,"y":0}),
            json!({"target":"assistant","layer":"secret"}),
            json!({"target":"companion"}),
        ] {
            assert!(
                serde_json::from_value::<Change>(value.clone())
                    .unwrap()
                    .validate()
                    .is_err(),
                "{value}"
            );
        }
    }
}

