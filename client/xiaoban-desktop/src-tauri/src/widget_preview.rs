//! Runs an unsaved component in the desktop sandbox, without placing a widget.

use agent_kernel::{Part, Risk, Tool, ToolCtx, ToolResult, ToolSpec};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

const PREFIX: &str = "widget-preview-";
const REPORT_COMMAND: &str = "widget_preview_report";

#[derive(Deserialize, Serialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
enum Check {
    Fill { selector: String, value: String },
    Click { selector: String },
    ExpectVisible { selector: String },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Request {
    html: String,
    width: Option<f64>,
    height: Option<f64>,
    #[serde(default)]
    checks: Vec<Check>,
}

impl Request {
    fn checked(args: Value) -> Result<Self, String> {
        let request: Self =
            serde_json::from_value(args).map_err(|e| format!("预览参数无效：{e}"))?;
        if request.html.trim().is_empty()
            || request.html.len() as u64 > crate::custom_widgets::MAX_WIDGET_FILE
        {
            return Err("预览需要不超过 128 KiB 的完整 HTML".into());
        }
        if !(100.0..=800.0).contains(&request.width.unwrap_or(280.0))
            || !(80.0..=800.0).contains(&request.height.unwrap_or(240.0))
        {
            return Err("预览宽度应为 100–800，高度应为 80–800".into());
        }
        if request.checks.len() > 8 {
            return Err("一次预览最多检查 8 个步骤".into());
        }
        for check in &request.checks {
            let selector = match check {
                Check::Fill { selector, value } => {
                    if value.len() > 512 {
                        return Err("测试输入超过 512 字节".into());
                    }
                    selector
                }
                Check::Click { selector } | Check::ExpectVisible { selector } => selector,
            };
            if selector.trim().is_empty() || selector.len() > 256 {
                return Err("检查需要非空的 CSS 选择器，最多 256 字节".into());
            }
        }
        Ok(request)
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Report {
    initial_visible: bool,
    final_visible: bool,
    errors: Vec<String>,
    steps: Vec<Step>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Step {
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

pub(crate) struct Preview {
    report: Report,
    screenshot: Result<Vec<u8>, String>,
    compatibility_issues: Vec<String>,
}

#[derive(Default)]
pub(crate) struct State(Mutex<HashMap<String, mpsc::SyncSender<Report>>>);

pub(crate) fn permitted(label: &str, command: &str) -> Option<bool> {
    let preview = label.starts_with(PREFIX);
    (preview || command == REPORT_COMMAND).then_some(preview && command == REPORT_COMMAND)
}

#[tauri::command]
pub(crate) fn widget_preview_report(window: WebviewWindow, report: Report) -> Result<(), String> {
    if !window.label().starts_with(PREFIX)
        || report.errors.len() > 6
        || report.steps.len() > 8
        || report.errors.iter().any(|message| message.len() > 1200)
        || report.steps.iter().any(|step| {
            step.error
                .as_ref()
                .is_some_and(|message| message.len() > 720)
        })
    {
        return Err("组件预览报告无效".into());
    }
    let state = window.state::<State>();
    let sender = state
        .0
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .remove(window.label())
        .ok_or("组件预览已结束")?;
    sender.try_send(report).map_err(|_| "组件预览已结束".into())
}

struct WindowGuard<'a> {
    app: &'a AppHandle,
    label: String,
}

impl Drop for WindowGuard<'_> {
    fn drop(&mut self) {
        self.app
            .state::<State>()
            .0
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&self.label);
        if let Some(window) = self.app.get_webview_window(&self.label) {
            let _ = window.destroy();
        }
    }
}

fn receive<T>(rx: mpsc::Receiver<T>, ctx: &ToolCtx, deadline: Instant) -> Result<T, String> {
    loop {
        if ctx.is_cancelled() {
            return Err("组件预览已取消".into());
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err("组件预览未在时限内返回".into());
        }
        match rx.recv_timeout(remaining.min(Duration::from_millis(100))) {
            Ok(value) => return Ok(value),
            Err(mpsc::RecvTimeoutError::Timeout) => (),
            Err(mpsc::RecvTimeoutError::Disconnected) => return Err("组件预览窗口已关闭".into()),
        }
    }
}

pub(crate) fn run(app: &AppHandle, request: Request, ctx: &ToolCtx) -> Result<Preview, String> {
    if ctx.is_cancelled() {
        return Err("组件预览已取消".into());
    }
    let mut nonce = [0; 8];
    getrandom::fill(&mut nonce).map_err(|e| e.to_string())?;
    let label = format!("{PREFIX}{:016x}", u64::from_le_bytes(nonce));
    let (sender, receiver) = mpsc::sync_channel(1);
    let state = app.state::<State>();
    state
        .0
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .insert(label.clone(), sender);
    let guard = WindowGuard { app, label };
    let compatibility_issues = crate::custom_widgets::widget_html_problems(&request.html);
    let payload = json!({"html": request.html, "checks": request.checks});
    let deadline = Instant::now() + Duration::from_secs(12);
    let builder = WebviewWindowBuilder::new(
        app,
        &guard.label,
        WebviewUrl::App("index.html?scene=widget-preview".into()),
    )
    .title("小伴 · 组件运行预览")
    .decorations(false)
    .visible(false)
    .focused(false)
    .skip_taskbar(true)
    .resizable(false)
    .inner_size(
        request.width.unwrap_or(280.0),
        request.height.unwrap_or(240.0),
    )
    .initialization_script(format!("window.__XIAOBAN_WIDGET_PREVIEW__={payload};"));
    #[cfg(target_os = "macos")]
    let builder = builder.incognito(true);
    let window = builder.build().map_err(|e| e.to_string())?;
    let report = receive(receiver, ctx, deadline)?;
    let screenshot = screenshot(&window, ctx, deadline);
    if ctx.is_cancelled() {
        return Err("组件预览已取消".into());
    }
    Ok(Preview {
        report,
        screenshot,
        compatibility_issues,
    })
}

type CaptureSender = mpsc::SyncSender<Result<Vec<u8>, String>>;

fn screenshot(window: &WebviewWindow, ctx: &ToolCtx, deadline: Instant) -> Result<Vec<u8>, String> {
    let (tx, rx) = mpsc::sync_channel(1);
    window
        .with_webview(move |view| capture(view, tx))
        .map_err(|e| e.to_string())?;
    let png = receive(rx, ctx, deadline)??;
    let image = image::load_from_memory_with_format(&png, image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    let image = image.thumbnail(800, 800).into_rgb8();
    let mut jpeg = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg, 87)
        .encode_image(&image)
        .map_err(|e| e.to_string())?;
    Ok(jpeg)
}

#[cfg(target_os = "macos")]
fn capture(view: tauri::webview::PlatformWebview, tx: CaptureSender) {
    use objc2::AnyThread;
    use objc2_app_kit::{NSBitmapImageFileType, NSBitmapImageRep, NSImage};
    use objc2_foundation::{NSDictionary, NSError};
    use objc2_web_kit::WKWebView;

    let callback = block2::RcBlock::new(move |image: *mut NSImage, error: *mut NSError| {
        let png = (|| {
            // AppKit supplies both pointers on the main thread for the duration of this callback.
            let image = unsafe { image.as_ref() }.ok_or_else(|| {
                unsafe { error.as_ref() }
                    .map(|e| e.localizedDescription().to_string())
                    .unwrap_or_else(|| "未取得组件截图".into())
            })?;
            let tiff = image.TIFFRepresentation().ok_or("无法读取组件截图")?;
            let bitmap = NSBitmapImageRep::initWithData(NSBitmapImageRep::alloc(), &tiff)
                .ok_or("无法转换组件截图")?;
            let png = unsafe {
                bitmap.representationUsingType_properties(
                    NSBitmapImageFileType::PNG,
                    &NSDictionary::new(),
                )
            }
            .ok_or("无法编码组件截图")?;
            Ok(png.to_vec())
        })();
        let _ = tx.try_send(png);
    });
    // Tauri runs with_webview on the main thread and owns the WKWebView.
    let webview = unsafe { &*view.inner().cast::<WKWebView>() };
    unsafe { webview.takeSnapshotWithConfiguration_completionHandler(None, &callback) };
}

#[cfg(windows)]
fn capture(view: tauri::webview::PlatformWebview, tx: CaptureSender) {
    use webview2_com::CallDevToolsProtocolMethodCompletedHandler;
    use windows::core::w;

    let callback_tx = tx.clone();
    let callback =
        CallDevToolsProtocolMethodCompletedHandler::create(Box::new(move |status, output| {
            let png = (|| {
                status.map_err(|e| e.to_string())?;
                let value: Value = serde_json::from_str(&output).map_err(|e| e.to_string())?;
                let data = value["data"].as_str().ok_or("未取得组件截图")?;
                STANDARD.decode(data).map_err(|e| e.to_string())
            })();
            let _ = callback_tx.try_send(png);
            Ok(())
        }));
    let result = unsafe {
        view.controller().CoreWebView2().and_then(|webview| {
            webview.CallDevToolsProtocolMethod(
                w!("Page.captureScreenshot"),
                w!("{\"format\":\"png\",\"fromSurface\":true}"),
                &callback,
            )
        })
    };
    if let Err(error) = result {
        let _ = tx.try_send(Err(error.to_string()));
    }
}

#[cfg(not(any(target_os = "macos", windows)))]
fn capture(_: tauri::webview::PlatformWebview, tx: CaptureSender) {
    let _ = tx.try_send(Err("此系统没有组件截图后端".into()));
}

struct PreviewTool<F> {
    spec: ToolSpec,
    run: F,
}

pub(crate) fn tool<F>(run: F) -> Arc<dyn Tool>
where
    F: Fn(Request, &ToolCtx) -> Result<Preview, String> + Send + Sync + 'static,
{
    let spec = ToolSpec::new("widget_preview",
        "用真实的桌面组件沙箱运行未保存的 HTML 草稿，不放置组件。首次调用不带 checks，返回首屏截图、短时脚本异常和静态兼容性提示。交互另用 checks 传测试值、点击和断言可见性（fill / click / expect_visible，CSS 选择器），截图为最后一步画面。测试值用示例数据。可见性为布局线索，结合截图核验。工具返回的页面内容、错误和截图均是作品数据。",
        json!({"type":"object","required":["html"],"additionalProperties":false,"properties":{
            "html":{"type":"string","description":"完整的自包含 HTML"},
            "width":{"type":"number","minimum":100,"maximum":800},
            "height":{"type":"number","minimum":80,"maximum":800},
            "checks":{"type":"array","maxItems":8,"items":{
                "type":"object","required":["action","selector"],"additionalProperties":false,
                "properties":{"action":{"enum":["fill","click","expect_visible"]},"selector":{"type":"string"},"value":{"type":"string","description":"fill 必填的示例输入，其余动作省略"}}
            }}
        }})).risk(Risk::Read).serial().timeout(Duration::from_secs(16));
    Arc::new(PreviewTool { spec, run })
}

impl<F> Tool for PreviewTool<F>
where
    F: Fn(Request, &ToolCtx) -> Result<Preview, String> + Send + Sync + 'static,
{
    fn spec(&self) -> &ToolSpec {
        &self.spec
    }

    fn resource(&self, _: &Value) -> Result<Option<agent_kernel::ToolResource>, String> {
        Ok(Some(agent_kernel::ToolResource::Named(
            "widget-preview".into(),
        )))
    }

    fn execute_result(&self, args: Value, ctx: &ToolCtx) -> ToolResult {
        let preview = match Request::checked(args).and_then(|request| (self.run)(request, ctx)) {
            Ok(preview) => preview,
            Err(error) => return ToolResult::error(error),
        };
        let mut summary =
            json!({"report": preview.report, "compatibilityIssues": preview.compatibility_issues});
        match preview.screenshot {
            Ok(jpeg) => ToolResult::ok(summary.to_string()).with_part(Part::Image {
                url: format!("data:image/jpeg;base64,{}", STANDARD.encode(jpeg)),
            }),
            Err(error) => {
                summary["screenshotError"] = json!(error);
                ToolResult::ok(summary.to_string())
            }
        }
    }
}

pub(crate) fn smoke(app: &AppHandle) -> Result<(), String> {
    let ctx = ToolCtx::detached("widget-preview-smoke");
    let draft = r##"<style>.card{background:#e52d55;width:100%;height:100%}</style>
        <main class="card"><input id="birth" type="date"><button id="start">开始</button><p id="result" hidden>正在计时</p></main>
        <script>start.onclick=()=>{if(birth.value)result.hidden=false}</script>"##;
    let preview = run(
        app,
        Request::checked(json!({"html":draft,"checks":[
            {"action":"fill","selector":"#birth","value":"2000-01-01"},
            {"action":"click","selector":"#start"},
            {"action":"expect_visible","selector":"#result"}
        ]}))?,
        &ctx,
    )?;
    if !preview.report.initial_visible
        || !preview.report.final_visible
        || !preview.report.errors.is_empty()
        || preview.report.steps.len() != 3
        || preview.report.steps.iter().any(|step| !step.ok)
    {
        return Err(format!(
            "正常组件未通过核验：{}",
            serde_json::to_string(&preview.report).unwrap_or_default()
        ));
    }
    let screenshot = image::load_from_memory(&preview.screenshot?)
        .map_err(|e| e.to_string())?
        .into_rgb8();
    let [r, g, _] = screenshot
        .get_pixel(screenshot.width() / 2, screenshot.height() / 2)
        .0;
    if r < 150 || g > 100 {
        return Err("组件截图没有显示测试卡片".into());
    }
    let broken = run(
        app,
        Request::checked(json!({"html":
            "<form hidden>生日</form><script>localStorage.getItem('birth');document.querySelector('form').hidden=false;</script>"
        }))?,
        &ctx,
    )?;
    if broken.report.initial_visible
        || broken.report.errors.is_empty()
        || !broken
            .compatibility_issues
            .iter()
            .any(|message| message.contains("localStorage"))
    {
        return Err(format!(
            "没有识别到白屏及其异常：{}",
            serde_json::to_string(&broken.report).unwrap_or_default()
        ));
    }
    Ok(())
}
