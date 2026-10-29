//! Ephemeral capture windows: frozen images live only for one user-initiated session.
use std::{collections::{HashMap, HashSet}, sync::{Mutex, mpsc}, time::{Duration, Instant}};
use tauri::{Manager, WebviewWindow, WebviewWindowBuilder, WebviewUrl};
use crate::assistant_capture::Frame;
const PREFIX: &str = "capture-";
type Outcome = Result<Option<String>, String>;
struct Session { frames: HashMap<String, Frame>, ready: HashSet<String>, sender: mpsc::Sender<Outcome> }
#[derive(Default)]
pub struct CaptureState(Mutex<Option<Session>>);

pub fn permitted(label: &str, command: &str) -> Option<bool> {
    if command.starts_with("capture_") { return Some(label.starts_with(PREFIX) && ["capture_frame", "capture_show", "capture_finish"].contains(&command)); }
    label.starts_with(PREFIX).then_some(false)
}
fn owns(session: &Session, label: &str) -> bool { session.frames.contains_key(label) }
#[tauri::command]
pub fn capture_frame(window: WebviewWindow) -> Result<Frame, String> {
    let state = window.state::<CaptureState>();
    let result = state.0.lock().unwrap_or_else(|e| e.into_inner()).as_ref()
        .and_then(|session| session.frames.get(window.label()).cloned()).ok_or("截图已结束".into());
    result
}
#[tauri::command]
pub fn capture_show(window: WebviewWindow) -> Result<(), String> {
    {
        let state = window.state::<CaptureState>();
        let mut guard = state.0.lock().unwrap_or_else(|e| e.into_inner());
        let session = guard.as_mut().filter(|s| owns(s, window.label())).ok_or("截图已结束")?;
        session.ready.insert(window.label().to_string());
    }
    window.show().and_then(|_| window.set_focus()).map_err(|e|e.to_string())
}
#[tauri::command]
pub fn capture_finish(window: WebviewWindow, image: Option<String>, error: Option<String>) -> Result<(), String> {
    if let Some(ref image) = image { crate::assistant_capture::validate_data_url(image)?; }
    let state = window.state::<CaptureState>();
    let mut guard = state.0.lock().unwrap_or_else(|e| e.into_inner());
    if !guard.as_ref().is_some_and(|s| owns(s, window.label())) { return Err("截图已结束".into()); }
    let session = guard.take().unwrap();
    let outcome = if let Some(error) = error { Err(error.chars().take(300).collect()) } else { Ok(image) };
    let _ = session.sender.send(outcome);
    Ok(())
}
fn closed(app: &tauri::AppHandle, label: &str) {
    let state = app.state::<CaptureState>();
    let mut guard = state.0.lock().unwrap_or_else(|e| e.into_inner());
    if guard.as_ref().is_some_and(|s| owns(s, label)) {
        if let Some(session) = guard.take() { let _ = session.sender.send(Ok(None)); }
    }
}
/// Runs on a blocking worker, never on the window event-loop thread.
pub fn edit(app: &tauri::AppHandle, frames: Vec<Frame>) -> Outcome {
    if frames.is_empty() || frames.len() > 16 { return Err("没有可截图的显示器".into()); }
    let mut random = [0u8; 8]; getrandom::fill(&mut random).map_err(|e|e.to_string())?;
    let token: String = random.iter().map(|b| format!("{b:02x}")).collect();
    let entries: HashMap<_,_> = frames.into_iter().enumerate().map(|(i,f)|(format!("{PREFIX}{token}-{i}"),f)).collect();
    let labels: Vec<_> = entries.keys().cloned().collect();
    let geometries: Vec<_> = entries.iter().map(|(l,f)|(l.clone(), f.x, f.y, f.width, f.height)).collect();
    let (sender, receiver) = mpsc::channel();
    *app.state::<CaptureState>().0.lock().unwrap_or_else(|e| e.into_inner()) = Some(Session { frames: entries, ready: HashSet::new(), sender });
    let result = (|| {
        for (label,x,y,width,height) in geometries {
            if app.state::<CaptureState>().0.lock().unwrap_or_else(|e| e.into_inner()).is_none() { break; }
            let builder = WebviewWindowBuilder::new(app, &label, WebviewUrl::App("index.html?scene=capture".into()))
                .title("小伴 · 截图").decorations(false).shadow(false).resizable(false)
                .skip_taskbar(true).always_on_top(true).visible(false).focused(false)
                .accept_first_mouse(true).disable_drag_drop_handler().inner_size(width, height);
            #[cfg(windows)]
            let builder = builder.data_directory(crate::storage::webview(&crate::storage::root(app)?));
            let window = builder.build().map_err(|e| e.to_string())?;
            #[cfg(target_os="macos")] {
                crate::macos::configure_scene(&window, false, false)?;
                crate::macos::place_widget(&window, x, y, width, height)?;
            }
            #[cfg(not(target_os="macos"))] {
                window.set_position(tauri::PhysicalPosition::new(x as i32,y as i32)).map_err(|e|e.to_string())?;
                window.set_size(tauri::PhysicalSize::new(width as u32,height as u32)).map_err(|e|e.to_string())?;
            }
            let handle = app.clone(); let own_label = label.clone();
            window.on_window_event(move |event| {
                if matches!(event, tauri::WindowEvent::CloseRequested { .. } | tauri::WindowEvent::Destroyed) { closed(&handle, &own_label); }
            });
        }
        let start = Instant::now();
        loop {
            match receiver.recv_timeout(Duration::from_millis(250)) {
                Ok(outcome) => break outcome,
                Err(mpsc::RecvTimeoutError::Disconnected) => break Ok(None),
                Err(mpsc::RecvTimeoutError::Timeout) => {}
            }
            let loading = app.state::<CaptureState>().0.lock().unwrap_or_else(|e| e.into_inner()).as_ref().is_some_and(|s| s.ready.len() < s.frames.len());
            if loading && start.elapsed() > Duration::from_secs(25) { break Err("截图窗口加载超时，请重试。".into()); }
            if start.elapsed() > Duration::from_secs(600) { break Err("截图已超时，请重新截取。".into()); }
        }
    })();
    app.state::<CaptureState>().0.lock().unwrap_or_else(|e| e.into_inner()).take();
    for label in labels { if let Some(window) = app.get_webview_window(&label) { let _ = window.destroy(); } }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn editor_windows_have_only_capture_commands() {
        assert_eq!(permitted("capture-session-0","capture_frame"),Some(true));
        for label in ["control","assistant","widget-0","content"] { assert_eq!(permitted(label,"capture_frame"),Some(false)); }
        for command in ["chat_send","assistant_screenshot","open_download_url","capture_unknown"] { assert_eq!(permitted("capture-session-0",command),Some(false)); }
    }
}
