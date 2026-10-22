//! Isolated-profile check of the desktop effect layer, run by `XIAOBAN_EFFECTS_SMOKE=1` with the media smoke:
//! every built-in effect reaches the page and the preferences file, the layer does not depend on which
//! wallpaper is applied (including none), and choosing "no effects" removes the window.
use crate::{effect_choice::EffectsChoice, effects::LABEL, settings, DesktopState};
use serde_json::{json, Value};
use std::time::{Duration, Instant};
use tauri::Manager;

const SCENES: [&str; 12] = ["rain", "snow", "fireflies", "petals", "maple", "meteors", "stars", "butterfly-dream", "lanterns", "fireworks", "aurora", "galaxy"];

async fn settle(app: &tauri::AppHandle, expect_ready: bool) -> Result<(), String> {
    let handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || crate::effects::reconcile(&handle)).await.map_err(|e| e.to_string())?;
    if !expect_ready { return Ok(()); }
    let handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let deadline = Instant::now() + Duration::from_secs(25);
        while Instant::now() < deadline {
            let ready = handle.state::<DesktopState>().ready_scenes.lock().unwrap_or_else(|e| e.into_inner()).contains(LABEL);
            if ready && handle.state::<crate::content::ContentState>().reports.lock().unwrap_or_else(|e| e.into_inner()).contains_key(LABEL) { return Ok(()); }
            std::thread::sleep(Duration::from_millis(100));
        }
        Err("桌面特效窗口没有在 25 秒内完成加载".to_string())
    }).await.map_err(|e| e.to_string())?
}

fn report_of(app: &tauri::AppHandle, id: &str, expected: &EffectsChoice) -> Result<Value, String> {
    let mut report = app.state::<crate::content::ContentState>().reports.lock().unwrap_or_else(|e| e.into_inner()).get(LABEL).cloned().ok_or("特效验证没有收到场景报告")?;
    let effects = &report["effects"];
    let current = settings(app);
    let params = serde_json::to_value(expected.params_of(id)).map_err(|e| e.to_string())?;
    let same = |name: &str| effects["params"][name].as_f64().zip(params[name].as_f64()).is_some_and(|(a, b)| (a - b).abs() < 1e-9);
    if report["privilegedIpcBlocked"] != true || effects["effect"] != id || effects["canvasPresent"] != true
        || !["density", "speed", "size"].into_iter().all(same)
        || effects["paused"] != current.paused || effects["fps"].as_u64() != Some(current.fps as u64) {
        return Err(format!("桌面特效配置或隔离报告不一致：{id} {report}"));
    }
    if effects["canvasWidth"].as_u64().unwrap_or(0) == 0 || effects["canvasHeight"].as_u64().unwrap_or(0) == 0 { return Err(format!("桌面特效 Canvas 绘图区为空：{id}")); }
    if !effects["particles"].as_array().is_some_and(|layers| !layers.is_empty() && layers.iter().all(|n| n.as_u64().unwrap_or(0) > 0)) {
        return Err(format!("特效没有为每一层生成粒子：{id}"));
    }
    #[cfg(windows)]
    { report["native"] = crate::windows::diagnostics(app); }
    Ok(report)
}

fn persisted(app: &tauri::AppHandle, expected: &EffectsChoice) -> Result<(), String> {
    let disk = crate::preferences::load(&crate::storage::root(app)?.join("config/preferences.json")).map_err(|e| format!("特效持久化配置无法读取：{e}"))?;
    if disk.settings.effects != *expected || settings(app).effects != *expected { return Err("桌面特效没有同时写入内存和用户配置".into()); }
    Ok(())
}

fn choose(app: &tauri::AppHandle, selected: &str, params: Value) -> Result<EffectsChoice, String> {
    let choice = EffectsChoice::from_value(&json!({"selected": selected, "params": params}));
    crate::set_desktop_effects(app.clone(), serde_json::to_value(&choice).map_err(|e| e.to_string())?)?;
    Ok(choice)
}

pub async fn run(app: &tauri::AppHandle) -> Result<Value, String> {
    if !app.state::<DesktopState>().isolated { return Err("桌面特效验证只能使用隔离配置".into()); }
    let previous_mode = settings(app).mode;
    crate::set_mode(app.clone(), "wallpaper".into()).await?;
    crate::content::apply_wallpaper(app.clone(), "companion".into(), Default::default(), false).await?;
    let mut cases = Vec::new();
    for id in SCENES {
        let handle = app.clone();
        // A fresh window per effect, so the page reports exactly this configuration.
        tauri::async_runtime::spawn_blocking(move || crate::effects::close(&handle)).await.map_err(|e| e.to_string())??;
        let choice = choose(app, id, json!({id: {"density": 1.3, "speed": 1.5, "size": 1.2}}))?;
        settle(app, true).await?;
        cases.push(json!({"effect": id, "report": report_of(app, id, &choice)?, "persisted": persisted(app, &choice).is_ok()}));
        persisted(app, &choice)?;
    }
    let kept = choose(app, "meteors", json!({"meteors": {"density": 1.0, "speed": 1.5, "size": 1.0}}))?;
    settle(app, true).await?;
    let mut independent = Vec::new();
    for id in ["test-image", "test-video", "test-canvas", "companion"] {
        crate::content::apply_wallpaper(app.clone(), id.into(), Default::default(), false).await?;
        settle(app, false).await?;
        if app.get_webview_window(LABEL).is_none() || settings(app).effects != kept { return Err(format!("应用壁纸 {id} 后桌面特效消失或被改动")); }
        let mut state = json!({"id": id, "overlayPresent": true});
        #[cfg(windows)]
        {
            state["native"] = crate::windows::diagnostics(app);
            let active = app.state::<crate::content::ContentState>().label.lock().unwrap_or_else(|e| e.into_inner()).clone();
            if let Some(label) = active {
                state["composition"] = app.state::<crate::content::ContentState>().reports.lock().unwrap_or_else(|e| e.into_inner())
                    .get(&label).cloned().unwrap_or(Value::Null);
            }
        }
        independent.push(state);
    }
    let off = choose(app, "", json!({}))?;
    settle(app, false).await?;
    if app.get_webview_window(LABEL).is_some() { return Err("选择“无特效”后覆盖窗口没有关闭".into()); }
    persisted(app, &off)?;
    crate::set_mode(app.clone(), previous_mode).await?;
    Ok(json!({"cases": cases, "independent": independent, "turnedOff": true, "persistence": {"diskConfigMatch": true}}))
}
