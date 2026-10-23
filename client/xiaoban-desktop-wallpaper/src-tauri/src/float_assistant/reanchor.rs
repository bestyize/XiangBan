//! Moving the stage around a ball that is showing. The page fades the ball out, the window moves, the page puts the
//! ball at its new spot and fades it back in, so the move itself is never seen.
use super::*;
use std::sync::mpsc::{self, Sender};

#[derive(Default)]
pub(super) struct State {
    active: AtomicBool,
    next: AtomicU64,
    pending: Mutex<Option<(u64, Sender<()>)>>,
}

/// How long the page may take to fade the ball out before the window moves anyway.
const FADE_WAIT: Duration = Duration::from_millis(300);

pub(super) fn ready(window: &WebviewWindow, id: u64) -> Result<(), String> {
    if window.label() != LABEL { return Err("只有超级小伴可以移动悬浮球".into()); }
    let state = window.state::<AssistantState>();
    let mut pending = state.reanchor.pending.lock().unwrap_or_else(|e| e.into_inner());
    if pending.as_ref().is_some_and(|(own, _)| *own == id) {
        if let Some((_, sender)) = pending.take() { let _ = sender.send(()); }
    }
    Ok(())
}

/// Moves the ball's window to `stage`. Without a loaded page there is nothing to fade: it moves at once.
pub(super) fn begin(app: &tauri::AppHandle, stage: Stage) {
    let state = app.state::<AssistantState>();
    let Some(window) = app.get_webview_window(LABEL) else { return };
    if !state.ready.load(Ordering::SeqCst) || !window.is_visible().unwrap_or(false) {
        let _ = set_stage(app, &window, stage);
        return;
    }
    // One at a time; the one under way looks again when it is done.
    if state.reanchor.active.swap(true, Ordering::SeqCst) { return; }
    let id = state.reanchor.next.fetch_add(1, Ordering::SeqCst) + 1;
    let (tx, rx) = mpsc::channel();
    *state.reanchor.pending.lock().unwrap_or_else(|e| e.into_inner()) = Some((id, tx));
    let _ = window.emit("assistant-reanchor", json!({ "id": id }));
    let app = app.clone();
    std::thread::spawn(move || {
        let _ = rx.recv_timeout(FADE_WAIT);
        let main = app.clone();
        let done = app.run_on_main_thread(move || {
            let state = main.state::<AssistantState>();
            state.reanchor.pending.lock().unwrap_or_else(|e| e.into_inner()).take();
            if let Some(window) = main.get_webview_window(LABEL) {
                // Only a ball is moved this way; anything else that happened meanwhile placed the window itself.
                if mode(&main) == Mode::Ball && !state.dragging.load(Ordering::SeqCst) {
                    if let Err(error) = set_stage(&main, &window, stage) { eprintln!("超级小伴：{error}"); }
                }
                let _ = window.emit("assistant-reanchor-done", json!({ "id": id }));
            }
            state.reanchor.active.store(false, Ordering::SeqCst);
        });
        if done.is_err() { app.state::<AssistantState>().reanchor.active.store(false, Ordering::SeqCst); }
    });
}
