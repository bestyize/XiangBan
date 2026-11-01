//! Isolated Baidu personal OOB authorization. The remote page has no Tauri IPC.
use super::super::*;
use std::{collections::HashMap, sync::OnceLock, time::Instant};
use tauri::{WebviewUrl, WebviewWindowBuilder};
use url::Url;

const PREFIX: &str = "baidu-auth-";
const CLIENT_ID: &str = "QHOuRXiepJBMjtk0esLhrPoNlQyYd0mF";
const TOKEN_KEY: &str = "BAIDU_NETDISK_ACCESS_TOKEN";
const TTL: Duration = Duration::from_secs(300);

struct Flow {
    id: String,
    state: String,
    created: Instant,
    cancelled: Arc<AtomicBool>,
    phase: Mutex<Phase>,
}
#[derive(Clone)]
enum Phase { Pending, Connecting, Cancelling(Option<&'static str>), Connected, Cancelled, Error(&'static str) }
static FLOWS: OnceLock<Mutex<HashMap<String, Arc<Flow>>>> = OnceLock::new();
fn flows() -> &'static Mutex<HashMap<String, Arc<Flow>>> { FLOWS.get_or_init(|| Mutex::new(HashMap::new())) }
fn random() -> Result<String, String> {
    let mut b = [0u8; 32];
    getrandom::fill(&mut b).map_err(|_| "无法启动安全授权")?;
    Ok(b.iter().map(|v| format!("{v:02x}")).collect())
}
fn trusted_navigation(url: &Url) -> bool {
    url.scheme() == "https" && url.username().is_empty() && url.password().is_none()
        && url.port_or_known_default() == Some(443)
        && url.host_str().is_some_and(|h| h == "baidu.com" || h.ends_with(".baidu.com"))
}
fn callback_endpoint(url: &Url) -> bool {
    trusted_navigation(url) && url.host_str() == Some("openapi.baidu.com") && url.path() == "/oauth/2.0/login_success"
}
// Accept only a single matching state and a single fragment token. Never include URL in an error.
fn parse_callback(url: &Url, expected: &str) -> Result<Option<String>, &'static str> {
    if !callback_endpoint(url) { return Ok(None); }
    let fragment = url.fragment().unwrap_or("");
    let pairs: Vec<_> = url::form_urlencoded::parse(fragment.as_bytes()).collect();
    let query: Vec<_> = url.query_pairs().collect();
    let states: Vec<_> = pairs.iter().chain(query.iter()).filter(|(k, _)| k == "state").map(|(_, v)| v.as_ref()).collect();
    if states.as_slice() != [expected] { return Err("百度授权状态不匹配，请重新授权。"); }
    let tokens: Vec<_> = pairs.iter().filter(|(k, _)| k == "access_token").map(|(_, v)| v.as_ref()).collect();
    if tokens.len() != 1 || tokens[0].is_empty() || tokens[0].len() > 4096 || tokens[0].chars().any(char::is_control) {
        return Err("百度没有返回有效授权，请重试或使用手动方式。");
    }
    Ok(Some(tokens[0].to_owned()))
}
fn cancel(flow: &Flow, error: Option<&'static str>) {
    let mut phase = flow.phase.lock().unwrap_or_else(|e| e.into_inner());
    match *phase {
        Phase::Pending => {
            flow.cancelled.store(true, Ordering::SeqCst);
            *phase = error.map(Phase::Error).unwrap_or(Phase::Cancelled);
        }
        Phase::Connecting => {
            flow.cancelled.store(true, Ordering::SeqCst);
            // The network operation may have just committed. Keep reporting
            // pending until the worker returns its actual outcome.
            *phase = Phase::Cancelling(error);
        }
        _ => {}
    }
}
fn complete(flow: &Flow, success: bool) {
    let mut phase = flow.phase.lock().unwrap_or_else(|e| e.into_inner());
    *phase = if success { Phase::Connected } else {
        match *phase {
            Phase::Cancelling(Some(error)) => Phase::Error(error),
            Phase::Cancelling(None) => Phase::Cancelled,
            _ => Phase::Error("百度授权已收到，但连接失败；请重试或使用手动方式。"),
        }
    };
}
fn close(app: &tauri::AppHandle, label: &str) {
    if let Some(w) = app.get_webview_window(label) { let _ = w.close(); }
}
fn handle_callback(app: tauri::AppHandle, flow: Arc<Flow>, label: String, url: &Url) -> bool {
    if !callback_endpoint(url) { return false; }
    let token = match parse_callback(url, &flow.state) {
        Ok(Some(token)) => token,
        _ => {
            cancel(&flow, Some("百度授权结果无效，请重试或使用手动方式。"));
            std::thread::spawn(move || close(&app, &label));
            return true;
        }
    };
    {
        let mut phase = flow.phase.lock().unwrap_or_else(|e| e.into_inner());
        if !matches!(*phase, Phase::Pending) || flow.cancelled.load(Ordering::SeqCst) || flow.created.elapsed() >= TTL { return true; }
        *phase = Phase::Connecting;
    }
    std::thread::spawn(move || {
        let result = (|| -> Result<(), String> {
            let _guard = super::super::store::MUTATIONS.lock().map_err(|_| "连接器暂不可用")?;
            if flow.cancelled.load(Ordering::SeqCst) { return Err("授权已取消".into()); }
            let e = super::find(&Dirs::of(&app)?.plugins(), &flow.id)?;
            if e.plugin.name != "baidu-netdisk" || !e.plugin.server(&e.server)?.is_baidu_netdisk_personal() { return Err("连接器不匹配".into()); }
            let values = std::collections::BTreeMap::from([(TOKEN_KEY.to_string(), token)]);
            super::super::connector_manager(&app)?.connect_plugin_transactional(&e.plugin, &e.server, &values, true, flow.cancelled.clone())?;
            Ok(())
        })();
        complete(&flow, result.is_ok());
        close(&app, &label);
    });
    true
}
#[tauri::command]
pub async fn ai_connector_baidu_start(app: tauri::AppHandle, window: tauri::WebviewWindow, id: String) -> Result<Value, String> {
    super::super::store::control(&window)?;
    let e = super::find(&Dirs::of(&app)?.plugins(), &id)?;
    if e.plugin.name != "baidu-netdisk" || !e.plugin.server(&e.server)?.is_baidu_netdisk_personal() { return Err("此连接器不支持百度个人授权".into()); }
    let key = random()?;
    let state = random()?;
    let flow = Arc::new(Flow { id: id.clone(), state: state.clone(), created: Instant::now(), cancelled: Arc::new(AtomicBool::new(false)), phase: Mutex::new(Phase::Pending) });
    {
        let mut all = flows().lock().map_err(|_| "授权状态不可用")?;
        all.retain(|_, v| v.created.elapsed() < TTL + Duration::from_secs(300));
        if all.values().any(|v| v.id == id && matches!(*v.phase.lock().unwrap_or_else(|e| e.into_inner()), Phase::Pending | Phase::Connecting | Phase::Cancelling(_))) {
            return Err("百度授权正在进行，请先完成或取消。".into());
        }
        all.insert(key.clone(), flow.clone());
    }
    let label = format!("{PREFIX}{key}");
    let mut url = Url::parse("https://openapi.baidu.com/oauth/2.0/authorize").map_err(|_| "授权地址无效")?;
    url.query_pairs_mut().extend_pairs([("client_id", CLIENT_ID), ("redirect_uri", "oob"), ("response_type", "token"), ("scope", "basic,netdisk"), ("state", state.as_str())]);
    let nav_app = app.clone(); let nav_flow = flow.clone(); let nav_label = label.clone();
    let load_app = app.clone(); let load_flow = flow.clone(); let load_label = label.clone();
    let built = WebviewWindowBuilder::new(&app, &label, WebviewUrl::External(url))
        .title("百度网盘授权").inner_size(760., 720.).min_inner_size(360., 480.)
        .incognito(true)
        .on_navigation(move |url| {
            if handle_callback(nav_app.clone(), nav_flow.clone(), nav_label.clone(), url) { return false; }
            trusted_navigation(url)
        })
        .on_page_load(move |_, payload| { handle_callback(load_app.clone(), load_flow.clone(), load_label.clone(), payload.url()); })
        .on_new_window(|_, _| tauri::webview::NewWindowResponse::Deny)
        .build();
    let auth_window = match built { Ok(w) => w, Err(_) => { flows().lock().unwrap_or_else(|e| e.into_inner()).remove(&key); return Err("无法打开百度授权窗口，请使用手动方式。".into()); } };
    let event_flow = flow.clone();
    auth_window.on_window_event(move |event| if matches!(event, tauri::WindowEvent::Destroyed | tauri::WindowEvent::CloseRequested { .. }) {
        cancel(&event_flow, None);
    });
    let timer_app = app.clone(); let timer_label = label.clone();
    std::thread::spawn(move || {
        while flow.created.elapsed() < TTL {
            if !matches!(*flow.phase.lock().unwrap_or_else(|e| e.into_inner()), Phase::Pending | Phase::Connecting | Phase::Cancelling(_)) { return; }
            std::thread::sleep(Duration::from_secs(1));
        }
        cancel(&flow, Some("百度授权等待超时，请重试。"));
        close(&timer_app, &timer_label);
    });
    Ok(json!({"flowId": key}))
}
fn get_flow(key: &str) -> Result<Arc<Flow>, String> {
    flows().lock().map_err(|_| "授权状态不可用")?.get(key).cloned().ok_or("授权已结束，请重新开始".into())
}
#[tauri::command]
pub async fn ai_connector_baidu_status(window: tauri::WebviewWindow, flow_id: String) -> Result<Value, String> {
    super::super::store::control(&window)?;
    let flow = get_flow(&flow_id)?;
    let phase = flow.phase.lock().map_err(|_| "授权状态不可用")?.clone();
    Ok(match phase {
        Phase::Pending | Phase::Connecting | Phase::Cancelling(_) => json!({"status":"pending"}),
        Phase::Connected => json!({"status":"connected"}),
        Phase::Cancelled => json!({"status":"cancelled"}),
        Phase::Error(error) => json!({"status":"error", "error":error}),
    })
}
#[tauri::command]
pub async fn ai_connector_baidu_cancel(app: tauri::AppHandle, window: tauri::WebviewWindow, flow_id: String) -> Result<(), String> {
    super::super::store::control(&window)?;
    let flow = get_flow(&flow_id)?;
    cancel(&flow, None);
    close(&app, &format!("{PREFIX}{flow_id}"));
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn callback_requires_exact_endpoint_one_state_and_fragment_token() {
        let url = |s| Url::parse(s).unwrap();
        let good = url("https://openapi.baidu.com/oauth/2.0/login_success#access_token=example&state=random");
        assert_eq!(parse_callback(&good, "random").unwrap().as_deref(), Some("example"));
        for bad in [
            "https://openapi.baidu.com/oauth/2.0/login_success#access_token=example&state=wrong",
            "https://openapi.baidu.com/oauth/2.0/login_success#access_token=example&state=random&state=random",
            "https://openapi.baidu.com/oauth/2.0/login_success?access_token=example&state=random",
            "https://openapi.baidu.com/oauth/2.0/login_success#access_token=a&access_token=b&state=random",
        ] { assert!(parse_callback(&url(bad), "random").is_err()); }
        for bad in [
            "https://openapi.baidu.com.evil.example/oauth/2.0/login_success#access_token=a&state=random",
            "http://openapi.baidu.com/oauth/2.0/login_success#access_token=a&state=random",
            "https://user@openapi.baidu.com/oauth/2.0/login_success#access_token=a&state=random",
            "https://openapi.baidu.com/oauth/2.0/else#access_token=a&state=random",
        ] { assert!(parse_callback(&url(bad), "random").unwrap().is_none()); }
    }
    #[test]
    fn cancellation_waits_for_the_connection_commit_outcome() {
        let flow = Flow { id: "test".into(), state: "state".into(), created: Instant::now(), cancelled: Arc::new(AtomicBool::new(false)), phase: Mutex::new(Phase::Connecting) };
        cancel(&flow, None);
        assert!(flow.cancelled.load(Ordering::SeqCst));
        assert!(matches!(*flow.phase.lock().unwrap(), Phase::Cancelling(None)));
        complete(&flow, true);
        assert!(matches!(*flow.phase.lock().unwrap(), Phase::Connected));
        *flow.phase.lock().unwrap() = Phase::Connecting;
        cancel(&flow, Some("超时"));
        complete(&flow, false);
        assert!(matches!(*flow.phase.lock().unwrap(), Phase::Error("超时")));
    }
}
