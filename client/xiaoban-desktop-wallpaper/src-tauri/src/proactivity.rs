//! Desktop-only bridge for the independent proactivity engine.
//! Reuses the existing presence sampler; never accesses the keyboard, screen, or external apps.
use agent_proactivity::{Engine, EngineState, Event, EventKind, EventSource, Observation, Policy};
use std::{
    fs,
    sync::{atomic::{AtomicBool, Ordering}, Mutex},
};
use tauri::Manager;

pub struct ProactivityState {
    engine: Mutex<Engine>,
    /// Damaged settings must not be silently overwritten on the next presence event.
    read_error: AtomicBool,
}
impl Default for ProactivityState {
    fn default() -> Self {
        Self { engine: Mutex::new(Engine::new(Policy::default())), read_error: AtomicBool::new(false) }
    }
}

fn data_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    Ok(crate::storage::root(app)?.join("config/proactivity.json"))
}
fn map_presence(kind: crate::presence::Kind, at_ms: i64) -> Event {
    let kind = match kind {
        crate::presence::Kind::FirstToday => EventKind::FirstActivityOfDay,
        crate::presence::Kind::WelcomeBack => EventKind::UserReturned,
        crate::presence::Kind::LateNight => EventKind::LateNightActive,
    };
    Event::new(kind, EventSource::Desktop, at_ms)
}
pub fn start(app: &tauri::AppHandle) {
    let state = app.state::<ProactivityState>();
    let loaded = data_path(app).and_then(|path| match fs::read(&path) {
        Ok(bytes) => serde_json::from_slice::<EngineState>(&bytes)
            .map(Some).map_err(|e| format!("主动感知状态损坏：{e}")),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!("主动感知状态无法读取：{error}")),
    });
    match loaded {
        Ok(Some(snapshot)) => {
            *state.engine.lock().unwrap() = Engine::restore(Policy::default(), snapshot);
        }
        Ok(None) => {}
        Err(error) => {
            state.read_error.store(true, Ordering::Relaxed);
            eprintln!("proactivity: {error}; preserving original file");
        }
    }
}

/// This is observation only; existing presence animations remain untouched.
/// The explicit product opt-in + display pipeline will be added in a later PR.
pub fn observe_presence(app: &tauri::AppHandle, kind: crate::presence::Kind, at_ms: i64) {
    let state = app.state::<ProactivityState>();
    if state.read_error.load(Ordering::Relaxed) { return; }
    let snapshot = {
        let mut engine = state.engine.lock().unwrap();
        if engine.observe(&map_presence(kind, at_ms)) != Observation::Accepted { return; }
        engine.snapshot()
    };
    if let Err(error) = data_path(app).and_then(|path| crate::library::save_json(&path, &snapshot)) {
        eprintln!("proactivity: could not persist aggregates: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn maps_existing_presence_without_personal_content() {
        for (kind, expected) in [
            (crate::presence::Kind::FirstToday, EventKind::FirstActivityOfDay),
            (crate::presence::Kind::WelcomeBack, EventKind::UserReturned),
            (crate::presence::Kind::LateNight, EventKind::LateNightActive),
        ] {
            let event = map_presence(kind, 1_792_540_000_000);
            assert_eq!(event.kind, expected);
            assert_eq!(event.subject_id, None);
            assert!(event.valid());
        }
    }
    #[test]
    fn native_bridge_cannot_enable_notifications_by_default() {
        let engine = Engine::new(Policy::default());
        assert_eq!(engine.snapshot().attention().units_today(), 0);
        assert!(!Policy::default().enabled);
    }
}
