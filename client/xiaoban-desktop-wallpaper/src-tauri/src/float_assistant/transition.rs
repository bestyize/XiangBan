//! Let the page finish its morph before changing the native window rectangle.
use super::*;
use std::sync::mpsc::{self, Sender};

#[derive(Default)]
pub(super) struct State {
    active: AtomicBool,
    next: AtomicU64,
    right: AtomicBool,
    pending: Mutex<Option<(u64, &'static str, Sender<()>)>>,
}

pub(super) fn active(app: &tauri::AppHandle) -> bool {
    app.state::<AssistantState>().transition.active.load(Ordering::SeqCst)
}
pub(super) fn fitting_allowed(app: &tauri::AppHandle) -> bool {
    app.state::<AssistantState>().transition.pending.lock().unwrap().as_ref().is_some_and(|(_, step, _)| *step == "enter")
}
fn finish(app: &tauri::AppHandle) {
    let state = app.state::<AssistantState>();
    state.transition.pending.lock().unwrap().take();
    state.transition.active.store(false, Ordering::SeqCst);
}
fn signal(app: &tauri::AppHandle, id: u64, target: &str, step: &str) {
    if let Some(window) = app.get_webview_window(LABEL) {
        let _ = window.emit("assistant-transition", json!({ "id": id, "target": target, "step": step, "right": app.state::<AssistantState>().transition.right.load(Ordering::SeqCst) }));
    }
}
fn wait_for(app: &tauri::AppHandle, id: u64, step: &'static str) -> mpsc::Receiver<()> {
    let (tx, rx) = mpsc::channel();
    *app.state::<AssistantState>().transition.pending.lock().unwrap() = Some((id, step, tx));
    rx
}
pub(super) fn ready(window: &WebviewWindow, id: u64, step: &str) -> Result<(), String> {
    if window.label() != LABEL { return Err("只有超级小伴可以完成形态切换".into()); }
    let state = window.state::<AssistantState>();
    let mut pending = state.transition.pending.lock().unwrap();
    if pending.as_ref().is_some_and(|(own_id, own_step, _)| *own_id == id && *own_step == step) {
        if let Some((_, _, sender)) = pending.take() { let _ = sender.send(()); }
    }
    Ok(())
}
/// No event-loop waits: if the page fails or reloads, bounded timeouts still finish the switch.
pub(super) fn begin(app: &tauri::AppHandle, target: &'static str) -> Result<bool, String> {
    let state = app.state::<AssistantState>();
    if !state.ready.load(Ordering::SeqCst) { return Ok(false); }
    if state.capturing.load(Ordering::SeqCst) || state.dragging.load(Ordering::SeqCst) { return Ok(true); }
    if state.transition.active.swap(true, Ordering::SeqCst) { return Ok(true); }
    let id = state.transition.next.fetch_add(1, Ordering::SeqCst) + 1;
    let source = mode(app);
    let ball = if source == Mode::Ball {
        app.get_webview_window(LABEL).and_then(|window| Some((window.outer_position().ok()?, window.outer_size().ok()?)))
            .map(|(position, size)| (position.x, position.y, size.width as i32)).ok_or("没有找到悬浮球位置".to_string())
    } else { ball_target(app) };
    if let Ok((x, y, size)) = ball {
        if let Ok(Some(monitor)) = app.monitor_from_point(f64::from(x + size / 2), f64::from(y + size / 2)) {
            let area = area_of(&monitor);
            state.transition.right.store(x + size / 2 >= area.x + area.w / 2, Ordering::SeqCst);
        }
    }
    let leave = wait_for(app, id, "leave");
    signal(app, id, target, "leave");
    let app = app.clone();
    std::thread::spawn(move || {
        let _ = leave.recv_timeout(Duration::from_millis(700));
        let next = app.clone();
        if app.run_on_main_thread(move || {
            if mode(&next) != source { finish(&next); signal(&next, id, target, "cancel"); return; }
            let result = (|| -> Result<(), String> {
                let window = window(&next)?;
                // Reposition only while hidden, so the old rectangle is never painted at the new location.
                window.hide().map_err(|e| e.to_string())?;
                if target == "capsule" { expand_surface(&next, false)?; }
                else { collapse_surface(&next, false)?; }
                let enter = wait_for(&next, id, "enter");
                signal(&next, id, target, "enter");
                let ready_app = next.clone();
                std::thread::spawn(move || {
                    let _ = enter.recv_timeout(Duration::from_millis(400));
                    let visible_app = ready_app.clone();
                    if ready_app.run_on_main_thread(move || {
                        if let Some(window) = visible_app.get_webview_window(LABEL) {
                            if mode(&visible_app) != Mode::Hidden {
                                let _ = window.show();
                                if mode(&visible_app) == Mode::Capsule { let _ = window.set_focus(); }
                            }
                        }
                        signal(&visible_app, id, target, "show");
                        finish(&visible_app);
                    }).is_err() { finish(&ready_app); }
                });
                Ok(())
            })();
            if let Err(error) = result {
                eprintln!("超级小伴形态切换失败：{error}");
                if let Some(window) = next.get_webview_window(LABEL) { if mode(&next) != Mode::Hidden { let _ = window.show(); } }
                signal(&next, id, target, "cancel"); finish(&next);
            }
        }).is_err() { finish(&app); }
    });
    Ok(true)
}
