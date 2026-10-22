//! A native bootstrap screen: it works even when no WebView can be created.
use super::{prepare, runtime_ready};
use ::windows::{
    core::{w, HRESULT, PCWSTR},
    Win32::{
        Foundation::{CloseHandle, HANDLE, HWND, LPARAM, S_OK, WPARAM},
        System::{Com::CoCreateGuid, JobObjects::*},
        UI::{Controls::*, WindowsAndMessaging::*},
    },
};
use std::{
    io::{BufRead, BufReader},
    os::windows::{io::AsRawHandle, process::CommandExt},
    path::PathBuf,
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};

const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Clone, Copy, Default)]
enum Stage {
    #[default]
    Download,
    Verify,
    Install,
    Recheck,
}

impl Stage {
    fn text(self) -> &'static str {
        match self {
            Self::Download => "正在下载必要资源。首次准备需要联网，完成后会自动打开小伴。",
            Self::Verify => "正在检查下载资源，请稍后。",
            Self::Install => "正在安装必要组件，请稍后。",
            Self::Recheck => "正在完成启动前检查，马上就好。",
        }
    }
}

#[derive(Default)]
struct Progress {
    stage: Stage,
    result: Option<Result<(), String>>,
}

struct DialogContext {
    progress: Arc<Mutex<Progress>>,
    cancel: Arc<AtomicBool>,
    #[cfg(debug_assertions)]
    simulate_cancel: bool,
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect()
}

fn detect() -> Option<String> {
    tauri::webview_version().ok()
}

pub fn before_start() -> bool {
    let os = windows_version::OsVersion::current();
    let version = detect();
    if std::env::args().any(|arg| arg == "--check-runtime") {
        println!(
            "{}",
            serde_json::json!({
                "windowsMajor": os.major, "windowsBuild": os.build,
                "webviewVersion": version, "ready": os.major >= 10 && runtime_ready(version.as_deref()),
            })
        );
        return false;
    }
    #[cfg(debug_assertions)]
    if std::env::args().any(|arg| arg == "--runtime-ui-smoke") {
        ui_smoke();
        return false;
    }
    if os.major < 10 {
        message(
            "小伴需要 Windows 10 或更新的系统。请升级系统后再打开。",
            MB_OK | MB_ICONINFORMATION,
        );
        return false;
    }
    if runtime_ready(version.as_deref()) {
        return true;
    }
    loop {
        // Recheck before every attempt: another app may have installed the runtime.
        if runtime_ready(detect().as_deref()) {
            return true;
        }
        let (result, cancelled) = loading(
            |progress, cancel| prepare(detect, || install(&progress, &cancel, false)),
            false,
        );
        if cancelled {
            return false;
        }
        match result {
            Ok(()) => return true,
            Err(error) => {
                let text = format!("必要资源准备失败。请检查网络或系统安装权限后重试。\n\n{error}");
                if message(&text, MB_RETRYCANCEL | MB_ICONWARNING) != IDRETRY {
                    return false;
                }
            }
        }
    }
}

fn message(text: &str, style: MESSAGEBOX_STYLE) -> MESSAGEBOX_RESULT {
    let text = wide(text);
    unsafe { MessageBoxW(None, PCWSTR(text.as_ptr()), w!("小伴"), style) }
}

pub fn show_startup_error(error: &str) {
    message(
        &format!(
            "小伴暂时无法打开。请重试；如果问题持续，请检查系统运行组件或安装权限。\n\n{error}"
        ),
        MB_OK | MB_ICONERROR,
    );
}

unsafe extern "system" fn dialog_event(
    hwnd: HWND,
    event: TASKDIALOG_NOTIFICATIONS,
    param: WPARAM,
    _: LPARAM,
    context: isize,
) -> HRESULT {
    let context = &*(context as *const DialogContext);
    if event == TDN_CREATED {
        SendMessageW(
            hwnd,
            TDM_SET_PROGRESS_BAR_MARQUEE.0 as u32,
            Some(WPARAM(1)),
            Some(LPARAM(35)),
        );
    } else if event == TDN_BUTTON_CLICKED {
        if context
            .progress
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .result
            .is_none()
        {
            context.cancel.store(true, Ordering::Relaxed);
        }
    } else if event == TDN_TIMER {
        let (stage, done) = {
            let progress = context.progress.lock().unwrap_or_else(|e| e.into_inner());
            (progress.stage, progress.result.is_some())
        };
        #[cfg(debug_assertions)]
        let done = done || (context.simulate_cancel && param.0 >= 400);
        #[cfg(not(debug_assertions))]
        let _ = param;
        if done {
            SendMessageW(
                hwnd,
                TDM_CLICK_BUTTON.0 as u32,
                Some(WPARAM(IDCANCEL.0 as usize)),
                None,
            );
        } else {
            let text = wide(stage.text());
            SendMessageW(
                hwnd,
                TDM_SET_ELEMENT_TEXT.0 as u32,
                Some(WPARAM(TDE_CONTENT.0 as usize)),
                Some(LPARAM(text.as_ptr() as isize)),
            );
        }
    }
    S_OK
}

fn loading(
    worker: impl FnOnce(Arc<Mutex<Progress>>, Arc<AtomicBool>) -> Result<(), String> + Send + 'static,
    _simulate_cancel: bool,
) -> (Result<(), String>, bool) {
    let progress = Arc::new(Mutex::new(Progress::default()));
    let cancel = Arc::new(AtomicBool::new(false));
    let context = Box::new(DialogContext {
        progress: progress.clone(),
        cancel: cancel.clone(),
        #[cfg(debug_assertions)]
        simulate_cancel: _simulate_cancel,
    });
    let worker_progress = progress.clone();
    let worker_cancel = cancel.clone();
    let thread = std::thread::spawn(move || {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            worker(worker_progress.clone(), worker_cancel)
        }))
        .unwrap_or_else(|_| Err("准备过程意外中断，请重试。".into()));
        worker_progress
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .result = Some(result);
    });
    let button_text = wide("暂不启动");
    let button = TASKDIALOG_BUTTON {
        nButtonID: IDCANCEL.0,
        pszButtonText: PCWSTR(button_text.as_ptr()),
    };
    let content = wide(Stage::Download.text());
    let config = TASKDIALOGCONFIG {
        cbSize: std::mem::size_of::<TASKDIALOGCONFIG>() as u32,
        pszWindowTitle: w!("小伴 · 首次准备"),
        pszMainInstruction: w!("正在加载必要资源，请稍后"),
        pszContent: PCWSTR(content.as_ptr()),
        pszFooter: w!("准备完成后，再次打开将直接进入小伴。"),
        dwFlags: TDF_CALLBACK_TIMER | TDF_SHOW_MARQUEE_PROGRESS_BAR | TDF_ALLOW_DIALOG_CANCELLATION,
        cButtons: 1,
        pButtons: &button,
        nDefaultButton: IDCANCEL.0,
        pfCallback: Some(dialog_event),
        lpCallbackData: (&*context as *const DialogContext) as isize,
        cxWidth: 280,
        ..Default::default()
    };
    let dialog = unsafe { TaskDialogIndirect(&config, None, None, None) };
    if dialog.is_err() {
        cancel.store(true, Ordering::Relaxed);
    }
    let _ = thread.join();
    let result = progress
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .result
        .take()
        .unwrap_or_else(|| Err("准备过程已中断。".into()));
    match dialog {
        Ok(()) => (result, cancel.load(Ordering::Relaxed)),
        Err(error) => (Err(format!("无法显示准备窗口：{error}")), false),
    }
}

// Close the job on cancellation/timeout to avoid orphaned downloader processes.
struct ProcessJob(HANDLE);
impl ProcessJob {
    fn new() -> Result<Self, String> {
        unsafe {
            let job = Self(CreateJobObjectW(None, PCWSTR::null()).map_err(|e| e.to_string())?);
            let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
            info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            SetInformationJobObject(
                job.0,
                JobObjectExtendedLimitInformation,
                &info as *const _ as _,
                std::mem::size_of_val(&info) as u32,
            )
            .map_err(|e| e.to_string())?;
            Ok(job)
        }
    }
}
impl Drop for ProcessJob {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

struct TempDirectory(PathBuf);
impl Drop for TempDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn install(
    progress: &Arc<Mutex<Progress>>,
    cancel: &Arc<AtomicBool>,
    download_only: bool,
) -> Result<(), String> {
    if cancel.load(Ordering::Relaxed) {
        return Err("已取消准备。".into());
    }
    // A unique private work directory prevents two launches sharing an executable.
    let guid = unsafe { CoCreateGuid().map_err(|e| e.to_string())? };
    let path = std::env::temp_dir().join(format!("xiaoban-runtime-{guid:?}"));
    std::fs::create_dir(&path).map_err(|e| e.to_string())?;
    let directory = TempDirectory(path);
    let script = directory.0.join("prepare.ps1");
    std::fs::write(&script, include_str!("bootstrap-webview.ps1")).map_err(|e| e.to_string())?;
    let installer = directory.0.join("MicrosoftEdgeWebview2Setup.exe");
    // Use the system PowerShell, never an executable resolved from the working directory.
    let mut system_dir = [0u16; 32768];
    let length = unsafe {
        windows_sys::Win32::System::SystemInformation::GetSystemDirectoryW(
            system_dir.as_mut_ptr(),
            system_dir.len() as u32,
        )
    } as usize;
    if length == 0 || length >= system_dir.len() {
        return Err("无法找到系统组件下载工具。".into());
    }
    let powershell = PathBuf::from(String::from_utf16_lossy(&system_dir[..length]))
        .join("WindowsPowerShell/v1.0/powershell.exe");
    let job = ProcessJob::new()?;
    let mut command = Command::new(powershell);
    command
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
        ])
        .arg(&script)
        .arg("-InstallerPath")
        .arg(installer);
    if download_only {
        command.arg("-DownloadOnly");
    }
    let mut child = command
        .creation_flags(CREATE_NO_WINDOW)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("无法启动组件下载工具：{e}"))?;
    if let Err(error) = unsafe { AssignProcessToJobObject(job.0, HANDLE(child.as_raw_handle())) } {
        let _ = child.kill();
        let _ = child.wait();
        return Err(format!("无法管理组件安装进程：{error}"));
    }
    let output = child.stdout.take().ok_or("无法读取下载进度")?;
    let reader_progress = progress.clone();
    let reader = std::thread::spawn(move || {
        let mut error = String::new();
        for line in BufReader::new(output).lines().map_while(Result::ok) {
            let stage = match line.as_str() {
                "XIAOBAN_STAGE_DOWNLOAD" => Some(Stage::Download),
                "XIAOBAN_STAGE_VERIFY" => Some(Stage::Verify),
                "XIAOBAN_STAGE_INSTALL" => Some(Stage::Install),
                _ => None,
            };
            if let Some(stage) = stage {
                reader_progress
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .stage = stage;
            } else if let Some(detail) = line.strip_prefix("XIAOBAN_ERROR:") {
                error = detail.chars().take(2000).collect();
            }
        }
        error
    });
    let deadline = Instant::now() + Duration::from_secs(780);
    let result = loop {
        if cancel.load(Ordering::Relaxed) {
            break Err("已取消准备。".into());
        }
        if Instant::now() >= deadline {
            break Err("准备超时，请检查网络后重试。".into());
        }
        match child.try_wait() {
            Ok(Some(status)) => {
                break if status.success() {
                    Ok(())
                } else {
                    Err("必要组件下载或安装失败。".into())
                }
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(100)),
            Err(error) => break Err(error.to_string()),
        }
    };
    drop(job);
    let _ = child.wait();
    let detail = reader.join().unwrap_or_default();
    if !detail.is_empty() {
        return Err(detail);
    }
    result?;
    if download_only {
        return Ok(());
    }
    progress.lock().unwrap_or_else(|e| e.into_inner()).stage = Stage::Recheck;
    // The updater can exit just before the per-user registration is visible.
    for _ in 0..60 {
        if cancel.load(Ordering::Relaxed) {
            return Err("已取消准备。".into());
        }
        if runtime_ready(detect().as_deref()) {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    Ok(()) // prepare() performs the final check; never trust the installer exit alone.
}

#[cfg(debug_assertions)]
fn ui_smoke() {
    let (success, _) = loading(
        |progress, _| {
            for stage in [
                Stage::Download,
                Stage::Verify,
                Stage::Install,
                Stage::Recheck,
            ] {
                progress.lock().unwrap_or_else(|e| e.into_inner()).stage = stage;
                std::thread::sleep(Duration::from_millis(250));
            }
            Ok(())
        },
        false,
    );
    let (failure, _) = loading(
        |_, _| {
            std::thread::sleep(Duration::from_millis(300));
            Err("simulated offline".into())
        },
        false,
    );
    let (_, cancelled) = loading(
        |_, cancel| {
            while !cancel.load(Ordering::Relaxed) {
                std::thread::sleep(Duration::from_millis(50));
            }
            Err("cancelled".into())
        },
        true,
    );
    let download = if std::env::args().any(|arg| arg == "--verify-runtime-download") {
        let (result, _) = loading(|progress, cancel| install(&progress, &cancel, true), false);
        Some(result)
    } else {
        None
    };
    let report = serde_json::json!({"success": success.is_ok(), "failure": failure.is_err(), "cancelled": cancelled,
        "download": download.map(|result| result.map(|_| "ok"))});
    if let Ok(path) = std::env::var("XIAOBAN_RUNTIME_REPORT") {
        let _ = std::fs::write(path, report.to_string());
    }
}
