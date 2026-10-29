//! Explicit quit must also escape WebView2's nested message pump on Windows.
use std::{sync::atomic::{AtomicBool, Ordering}, time::Duration};
use tauri::Manager;

const EXIT_TIMEOUT: Duration = Duration::from_secs(5);

pub struct ExitState {
    requested: AtomicBool,
    #[cfg(windows)]
    ui_thread: u32,
}

impl Default for ExitState {
    fn default() -> Self {
        Self {
            requested: AtomicBool::new(false),
            #[cfg(windows)]
            ui_thread: unsafe { windows_sys::Win32::System::Threading::GetCurrentThreadId() },
        }
    }
}

fn start_watchdog(timeout: Duration) {
    if let Err(error) = std::thread::Builder::new().name("quit-watchdog".into()).spawn(move || {
        std::thread::sleep(timeout);
        // Only armed after quitting. A blocked WebView or cleanup must not keep
        // the desktop client alive indefinitely; the normal exit runs first.
        eprintln!("小伴退出超过 {} 秒，结束进程", timeout.as_secs_f64());
        std::process::exit(0);
    }) {
        eprintln!("无法启动退出保护：{error}");
    }
}

/// Also covers exits initiated by the OS or another native component.
pub fn guard(app: &tauri::AppHandle) -> bool {
    if app.state::<ExitState>().requested.swap(true, Ordering::SeqCst) { return false; }
    start_watchdog(EXIT_TIMEOUT);
    true
}

#[cfg(windows)]
fn wake_ui_thread(thread: u32) -> Result<(), std::io::Error> {
    if unsafe {
        windows_sys::Win32::UI::WindowsAndMessaging::PostThreadMessageW(
            thread, windows_sys::Win32::UI::WindowsAndMessaging::WM_QUIT, 0, 0,
        )
    } == 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

pub fn request(app: &tauri::AppHandle) {
    if !guard(app) { return; }
    app.exit(0);
    // app.exit queues a Tao event. WebView2's wait_with_pump dispatches native
    // messages but cannot run that outer event loop. WM_QUIT cancels its wait,
    // letting Tauri finish the already-requested exit and normal cleanup.
    #[cfg(windows)]
    if let Err(error) = wake_ui_thread(app.state::<ExitState>().ui_thread) {
        eprintln!("无法唤醒退出消息循环：{error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn watchdog_finishes_even_when_cleanup_stalls() {
        const CHILD: &str = "XIAOBAN_TEST_QUIT_WATCHDOG";
        if std::env::var_os(CHILD).is_some() {
            start_watchdog(Duration::from_millis(200));
            loop { std::thread::park(); }
        }
        let mut child = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "shutdown::tests::watchdog_finishes_even_when_cleanup_stalls"])
            .env(CHILD, "1").stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null())
            .spawn().unwrap();
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(status) = child.try_wait().unwrap() {
                assert!(status.success(), "quit watchdog failed: {status}");
                break;
            }
            if std::time::Instant::now() >= deadline {
                let _ = child.kill(); let _ = child.wait();
                panic!("a stalled cleanup kept the process alive");
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    #[cfg(windows)]
    #[test]
    fn quit_interrupts_a_nested_native_message_pump() {
        use windows_sys::Win32::UI::WindowsAndMessaging::*;
        let (ready_tx, ready_rx) = std::sync::mpsc::channel();
        let (done_tx, done_rx) = std::sync::mpsc::channel();
        let worker = std::thread::spawn(move || unsafe {
            let mut msg: MSG = std::mem::zeroed();
            PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, PM_NOREMOVE);
            ready_tx.send(windows_sys::Win32::System::Threading::GetCurrentThreadId()).unwrap();
            loop {
                let result = GetMessageA(&mut msg, std::ptr::null_mut(), 0, 0);
                if result <= 0 { done_tx.send(result).unwrap(); break; }
                let _ = TranslateMessage(&msg);
                DispatchMessageA(&msg);
            }
        });
        let thread = ready_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        // The queued outer event alone cannot end this WebView2-style wait.
        assert_ne!(unsafe { PostThreadMessageW(thread, WM_APP + 1, 0, 0) }, 0);
        assert!(done_rx.recv_timeout(Duration::from_millis(40)).is_err());
        wake_ui_thread(thread).unwrap();
        assert_eq!(done_rx.recv_timeout(Duration::from_secs(2)).unwrap(), 0);
        worker.join().unwrap();
    }
}
