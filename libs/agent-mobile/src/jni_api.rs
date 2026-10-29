//! The JNI surface used by the Android app (`xyz.xiangban.xiaoban.common.community.AgentNative`).
//!
//! Everything crosses as strings: a config, then JSON lines in both directions (see the crate docs). The app polls
//! for output on a thread of its own; `poll` blocks for up to the given time.

use crate::{Engine, Polled};
use jni::objects::{JClass, JString};
use jni::sys::{jlong, jstring};
use jni::JNIEnv;
use std::collections::HashMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

static NEXT_HANDLE: AtomicI64 = AtomicI64::new(1);

fn engines() -> &'static Mutex<HashMap<i64, Arc<Engine>>> {
    static ENGINES: OnceLock<Mutex<HashMap<i64, Arc<Engine>>>> = OnceLock::new();
    ENGINES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn find(handle: jlong) -> Option<Arc<Engine>> {
    engines().lock().unwrap_or_else(|e| e.into_inner()).get(&handle).cloned()
}

fn throw(env: &mut JNIEnv, message: &str) {
    let _ = env.throw_new("java/lang/IllegalStateException", message);
}

fn text(env: &mut JNIEnv, value: &JString) -> Result<String, String> {
    env.get_string(value).map(|s| s.into()).map_err(|e| e.to_string())
}

/// Starts an engine and returns its handle. Throws `IllegalStateException` when the config is wrong.
#[no_mangle]
pub extern "system" fn Java_xyz_xiangban_xiaoban_common_community_AgentNative_nativeStart(mut env: JNIEnv, _class: JClass, config: JString) -> jlong {
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<jlong, String> {
        let config = text(&mut env, &config)?;
        let engine = Engine::start(&config)?;
        let handle = NEXT_HANDLE.fetch_add(1, Ordering::Relaxed);
        engines().lock().unwrap_or_else(|e| e.into_inner()).insert(handle, engine);
        Ok(handle)
    }));
    match result {
        Ok(Ok(handle)) => handle,
        Ok(Err(message)) => {
            throw(&mut env, &message);
            0
        }
        Err(_) => {
            throw(&mut env, "the agent engine crashed while starting");
            0
        }
    }
}

/// Sends one JSON line to the engine. Throws when the line is not JSON or the engine has stopped.
#[no_mangle]
pub extern "system" fn Java_xyz_xiangban_xiaoban_common_community_AgentNative_nativeSend(mut env: JNIEnv, _class: JClass, handle: jlong, line: JString) {
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<(), String> {
        let line = text(&mut env, &line)?;
        find(handle).ok_or_else(|| "the engine has stopped".to_string())?.send(&line)
    }));
    match result {
        Ok(Ok(())) => {}
        Ok(Err(message)) => throw(&mut env, &message),
        Err(_) => throw(&mut env, "the agent engine crashed"),
    }
}

/// The next line from the engine, or `null` when none arrived within `timeout_ms`. After the engine has stopped it
/// returns `{"type":"closed"}` once and `null` from then on.
#[no_mangle]
pub extern "system" fn Java_xyz_xiangban_xiaoban_common_community_AgentNative_nativePoll(env: JNIEnv, _class: JClass, handle: jlong, timeout_ms: jlong) -> jstring {
    let polled = catch_unwind(AssertUnwindSafe(|| match find(handle) {
        Some(engine) => engine.poll(Duration::from_millis(timeout_ms.max(0) as u64)),
        None => Polled::Closed,
    }));
    match polled {
        Ok(Polled::Line(line)) => env.new_string(line).map(|s| s.into_raw()).unwrap_or(std::ptr::null_mut()),
        _ => std::ptr::null_mut(),
    }
}

/// Stops the engine and forgets the handle. Safe to call twice.
#[no_mangle]
pub extern "system" fn Java_xyz_xiangban_xiaoban_common_community_AgentNative_nativeStop(_env: JNIEnv, _class: JClass, handle: jlong) {
    let engine = engines().lock().unwrap_or_else(|e| e.into_inner()).remove(&handle);
    if let Some(engine) = engine {
        let _ = catch_unwind(AssertUnwindSafe(|| engine.stop()));
    }
}
