//! Local desktop manifests. Cloud requests contain only deliberately selected public pack references.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::BTreeMap, sync::Mutex};
static IO: Mutex<()> = Mutex::new(());
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Reference {
    pack_id: i64,
    version: i32,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Scheme {
    id: String,
    title: String,
    created_at: String,
    snapshot: Option<Value>,
    refs: Vec<Reference>,
    #[serde(default)]
    cloud_revisions: BTreeMap<String, i64>,
}
#[derive(Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Saved {
    items: Vec<Scheme>,
    cloud_enabled: BTreeMap<String, bool>,
}
fn path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    Ok(crate::storage::root(app)?.join("config/community-schemes.json"))
}
fn read(app: &tauri::AppHandle) -> Result<Saved, String> {
    let p = path(app)?;
    if !p.exists() {
        return Ok(Saved::default());
    }
    let b = std::fs::read(p).map_err(|_| "读取本地方案失败")?;
    if b.len() > 512 * 1024 {
        return Err("本地方案文件超过上限".into());
    }
    serde_json::from_slice(&b).map_err(|_| "本地方案文件损坏，请保留文件后修复".into())
}
fn write(app: &tauri::AppHandle, v: &Saved) -> Result<(), String> {
    if serde_json::to_vec(v).map_err(|e| e.to_string())?.len() > 512 * 1024 {
        return Err("本地方案超过存储上限".into());
    }
    crate::library::save_json(&path(app)?, v)
}
fn control(window: &tauri::WebviewWindow) -> Result<(), String> {
    if window.label() == "control" {
        Ok(())
    } else {
        Err("只能从社区方案窗口操作".into())
    }
}
fn id() -> Result<String, String> {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).map_err(|_| "无法生成方案编号")?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}
fn canonical_id(v: &str) -> Result<String, String> {
    let s = v.replace('-', "");
    if s.len() == 32 && s.bytes().all(|b| b.is_ascii_hexdigit()) {
        Ok(s.to_ascii_lowercase())
    } else {
        Err("方案编号无效".into())
    }
}
fn uid(app: &tauri::AppHandle) -> Result<String, String> {
    crate::account::load_session(app)
        .map(|s| s.user.uid)
        .ok_or_else(|| "请先登录小伴账号".into())
}
// Whitelist extraction is intentionally separate from the Settings/Widget serializers: notes, city,
// reminder text, model services and arbitrary custom widget data never enter this manifest.
fn capture(settings: &Value, layout: &Value) -> Value {
    let widgets:Vec<Value>=layout["widgets"].as_array().into_iter().flatten().take(30).map(|w|{json!({"id":w["id"],"kind":w["kind"],"x":(w["x"].as_f64().unwrap_or(0.)).clamp(0.,1.),"y":(w["y"].as_f64().unwrap_or(0.)).clamp(0.,1.),"scale":w["scale"].as_f64().unwrap_or(1.).clamp(0.5,1.5),"enabled":w["enabled"].as_bool().unwrap_or(true)})}).collect();
    let selected = settings["effects"]["selected"].as_str().unwrap_or("");
    json!({"version":1,"wallpaper_id":settings["wallpaperId"],"effects":{"selected":selected,"params":{(selected):settings["effects"]["params"][selected]}},"companion_skin":settings["companionSkin"],"companion_enabled":settings["companionEnabled"],"widgets":widgets})
}
#[tauri::command]
pub fn community_schemes_status(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
) -> Result<Value, String> {
    control(&window)?;
    let _lock = IO.lock().map_err(|_| "方案锁不可用")?;
    let saved = read(&app)?;
    let user = uid(&app).ok();
    let enabled = user
        .as_ref()
        .is_some_and(|u| saved.cloud_enabled.get(u) == Some(&true));
    let rows:Vec<_>=saved.items.iter().map(|s|json!({"id":s.id,"title":s.title,"created_at":s.created_at,"snapshot":s.snapshot,"refs":s.refs,"cloud_revision":user.as_ref().and_then(|u|s.cloud_revisions.get(u)).copied().unwrap_or(0)})).collect();
    Ok(json!({"items":rows,"cloud_enabled":enabled,"signed_in":user.is_some()}))
}
#[tauri::command]
pub fn community_schemes_save(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    title: String,
    refs: Vec<Reference>,
) -> Result<Value, String> {
    control(&window)?;
    let title = title.trim();
    if title.is_empty() || title.chars().count() > 60 || title.chars().any(char::is_control) {
        return Err("方案名称需要1～60字".into());
    }
    if refs.len() > 20
        || refs.iter().any(|v| v.pack_id <= 0 || v.version <= 0)
        || refs
            .iter()
            .map(|v| v.pack_id)
            .collect::<std::collections::HashSet<_>>()
            .len()
            != refs.len()
    {
        return Err("作品引用无效或重复".into());
    }
    let settings = serde_json::to_value(crate::settings(&app)).map_err(|e| e.to_string())?;
    let layout = crate::widgets::get_desktop_widgets(app.clone())?;
    let snapshot = capture(&settings, &layout);
    let _lock = IO.lock().map_err(|_| "方案锁不可用")?;
    let mut saved = read(&app)?;
    if saved.items.len() >= 100 {
        return Err("最多保存100份本地方案".into());
    }
    let id = id()?;
    saved.items.insert(
        0,
        Scheme {
            id: id.clone(),
            title: title.into(),
            created_at: chrono::Utc::now().to_rfc3339(),
            snapshot: Some(snapshot),
            refs,
            cloud_revisions: BTreeMap::new(),
        },
    );
    write(&app, &saved)?;
    Ok(json!({"id":id}))
}
#[tauri::command]
pub fn community_schemes_remove(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    id: String,
) -> Result<(), String> {
    control(&window)?;
    let id = canonical_id(&id)?;
    let _lock = IO.lock().map_err(|_| "方案锁不可用")?;
    let mut saved = read(&app)?;
    saved.items.retain(|v| v.id != id);
    write(&app, &saved)
}
#[tauri::command]
pub fn community_schemes_cloud_enabled(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    enabled: bool,
) -> Result<(), String> {
    control(&window)?;
    let uid = uid(&app)?;
    let _lock = IO.lock().map_err(|_| "方案锁不可用")?;
    let mut saved = read(&app)?;
    saved.cloud_enabled.insert(uid, enabled);
    write(&app, &saved)
}
#[tauri::command]
pub async fn community_schemes_restore(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    id: String,
) -> Result<Value, String> {
    control(&window)?;
    let id = canonical_id(&id)?;
    let saved = {
        let _lock = IO.lock().map_err(|_| "方案锁不可用")?;
        read(&app)?
    };
    let scheme = saved
        .items
        .into_iter()
        .find(|v| v.id == id)
        .ok_or("方案不存在")?;
    let snap = scheme
        .snapshot
        .ok_or("这份方案只有社区引用，请逐项查看作品")?;
    let mut messages = Vec::new();
    let wall = snap["wallpaper_id"]
        .as_str()
        .unwrap_or("companion")
        .to_string();
    let options = crate::settings(&app).wallpaper_options;
    if let Err(e) = crate::content::apply_wallpaper(app.clone(), wall, options, false, None).await {
        messages.push(format!("壁纸：{e}"));
    }
    if let Err(e) = crate::set_desktop_effects(app.clone(), snap["effects"].clone()) {
        messages.push(format!("特效：{e}"));
    }
    if let Some(skin) = snap["companion_skin"].as_str() {
        if let Err(e) = crate::set_companion_skin(app.clone(), skin.into()) {
            messages.push(format!("伙伴主题：{e}"));
        }
    }
    if let Some(enabled) = snap["companion_enabled"].as_bool() {
        if let Err(e) = crate::set_companion_enabled(app.clone(), enabled).await {
            messages.push(format!("伙伴：{e}"));
        }
    }
    let current = crate::widgets::get_desktop_widgets(app.clone())?;
    for w in snap["widgets"].as_array().into_iter().flatten() {
        let id = w["id"].as_str().unwrap_or("");
        if !current["widgets"]
            .as_array()
            .is_some_and(|ws| ws.iter().any(|v| v["id"] == id && v["kind"] == w["kind"]))
        {
            messages.push(format!(
                "组件 {} 已不存在，已跳过",
                w["kind"].as_str().unwrap_or("")
            ));
            continue;
        }
        let patch=serde_json::from_value::<crate::widgets::Patch>(json!({"x":w["x"].as_f64().unwrap_or(0.),"y":w["y"].as_f64().unwrap_or(0.),"scale":w["scale"],"enabled":w["enabled"]})).map_err(|_|"组件位置无效")?;
        if let Err(e) = crate::widgets::update_desktop_widget(app.clone(), id.into(), patch).await {
            messages.push(format!("组件：{e}"));
        }
    }
    Ok(
        json!({"warnings":messages,"message":"已处理本地方案；缺失内容见提示，便签和提醒正文保持原样"}),
    )
}
fn cloud_payload(scheme: &Scheme, revision: i64) -> Value {
    json!({"schema_version":1,"title":scheme.title,"summary":"从小伴桌面选择的社区作品引用","items":scheme.refs,"expected_revision":revision})
}
async fn cloud_request(
    app: &tauri::AppHandle,
    method: reqwest::Method,
    path: &str,
    body: Option<Value>,
) -> Result<Value, String> {
    let user = uid(app)?;
    {
        let _lock = IO.lock().map_err(|_| "方案锁不可用")?;
        if read(app)?.cloud_enabled.get(&user) != Some(&true) {
            return Err("请先明确开启方案云端备份".into());
        }
    }
    let token = crate::account::fresh_access_token(app)
        .await
        .map_err(|_| "账号授权已失效，请重新登录")?;
    if uid(app)? != user {
        return Err("账号已切换，请重新选择要备份的方案".into());
    }
    // Production origin is fixed; neither the snapshot nor a cloud reply can choose a host.
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(25))
        .build()
        .map_err(|_| "初始化社区连接失败")?;
    let mut request = client
        .request(method, format!("{}{}", super::community::origin(), path))
        .bearer_auth(token);
    if let Some(body) = body {
        request = request.json(&body)
    }
    let mut response = request
        .send()
        .await
        .map_err(|_| "无法连接云端，已保留本地方案")?;
    let status = response.status();
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|_| "云端响应中断")? {
        if bytes.len() + chunk.len() > 256 * 1024 {
            return Err("云端响应超过上限".into());
        }
        bytes.extend_from_slice(&chunk)
    }
    let value: Value = serde_json::from_slice(&bytes).map_err(|_| "云端返回格式无效")?;
    if uid(app)? != user {
        return Err("账号已切换，请重新读取当前账号的备份".into());
    }
    {
        let _lock = IO.lock().map_err(|_| "方案锁不可用")?;
        if read(app)?.cloud_enabled.get(&user) != Some(&true) {
            return Err("云端访问已关闭，请重新选择操作".into());
        }
    }
    if !status.is_success() {
        return Err(if status.as_u16() == 409 {
            "云端版本已变化，请查看云端方案后再保存，未覆盖任何版本".into()
        } else {
            value["message"]
                .as_str()
                .unwrap_or("云端操作失败，本地方案已保留")
                .to_string()
        });
    }
    Ok(value)
}
#[tauri::command]
pub async fn community_schemes_cloud(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    action: String,
    id: Option<String>,
    revision: Option<i64>,
    page: Option<i64>,
) -> Result<Value, String> {
    control(&window)?;
    if action == "list" {
        let page = page.unwrap_or(1).clamp(1, 100);
        return cloud_request(
            &app,
            reqwest::Method::GET,
            &format!("/api/me/scheme-backups?page_size=50&page={page}"),
            None,
        )
        .await;
    }
    let id = canonical_id(&id.ok_or("请选择方案")?)?;
    let uid = uid(&app)?;
    let revision = revision.unwrap_or(0);
    if revision < 0 {
        return Err("版本无效".into());
    }
    if action == "remove" {
        let result = cloud_request(
            &app,
            reqwest::Method::POST,
            &format!("/api/me/scheme-backups/{id}/remove"),
            Some(json!({"expected_revision":revision})),
        )
        .await?;
        let _lock = IO.lock().map_err(|_| "方案锁不可用")?;
        let mut saved = read(&app)?;
        if let Some(local) = saved.items.iter_mut().find(|v| v.id == id) {
            local
                .cloud_revisions
                .insert(uid, result["revision"].as_i64().ok_or("云端版本无效")?);
            write(&app, &saved)?;
        }
        return Ok(result);
    }
    if action == "adopt" {
        let row = cloud_request(
            &app,
            reqwest::Method::GET,
            &format!("/api/me/scheme-backups/{id}"),
            None,
        )
        .await?;
        if row["status"] != "active" {
            return Err("这份云端备份已在回收站".into());
        }
        let title = row["title"].as_str().ok_or("云端方案名称无效")?;
        let refs: Vec<Reference> = row["items"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|v| {
                Some(Reference {
                    pack_id: v["pack_id"].as_i64()?,
                    version: v["version"].as_i64()?.try_into().ok()?,
                })
            })
            .collect();
        if refs.is_empty()
            || refs.len() > 20
            || refs.iter().any(|v| v.pack_id <= 0 || v.version <= 0)
            || title.chars().count() > 60
        {
            return Err("云端方案引用无效".into());
        }
        let revision = row["revision"]
            .as_i64()
            .filter(|v| *v > 0)
            .ok_or("云端版本无效")?;
        let _lock = IO.lock().map_err(|_| "方案锁不可用")?;
        let mut saved = read(&app)?;
        if let Some(local) = saved.items.iter_mut().find(|v| v.id == id) {
            local.title = title.into();
            local.refs = refs;
            local.cloud_revisions.insert(uid, revision);
        } else {
            if saved.items.len() >= 100 {
                return Err("最多保存100份本地方案".into());
            }
            let mut revisions = BTreeMap::new();
            revisions.insert(uid, revision);
            saved.items.insert(
                0,
                Scheme {
                    id,
                    title: title.into(),
                    created_at: chrono::Utc::now().to_rfc3339(),
                    snapshot: None,
                    refs,
                    cloud_revisions: revisions,
                },
            );
        }
        write(&app, &saved)?;
        return Ok(json!({"ok":true}));
    }
    if action != "save" {
        return Err("不支持的云端操作".into());
    }
    let scheme = {
        let _lock = IO.lock().map_err(|_| "方案锁不可用")?;
        read(&app)?
            .items
            .into_iter()
            .find(|v| v.id == id)
            .ok_or("本地方案不存在")?
    };
    if scheme.refs.is_empty() {
        return Err("请先选择要备份的社区作品引用；本地桌面布局不上传".into());
    }
    let result = cloud_request(
        &app,
        reqwest::Method::PUT,
        &format!("/api/me/scheme-backups/{id}"),
        Some(cloud_payload(&scheme, revision)),
    )
    .await?;
    let _lock = IO.lock().map_err(|_| "方案锁不可用")?;
    let mut saved = read(&app)?;
    if let Some(local) = saved.items.iter_mut().find(|v| v.id == id) {
        local
            .cloud_revisions
            .insert(uid, result["revision"].as_i64().ok_or("云端版本无效")?);
        write(&app, &saved)?
    }
    Ok(result)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn capture_is_whitelisted_and_cloud_has_no_desktop_payload() {
        let v = capture(
            &json!({"wallpaperId":"companion","effects":{"selected":"rain","params":{"rain":{"density":1}}},"apiKey":"secret","account":{"token":"secret"},"companionSkin":"pot"}),
            &json!({"desktop":{"width":1000,"height":500},"widgets":[{"id":"clock1","kind":"clock","x":0.5,"y":0.5,"scale":1,"city":{"name":"private"},"note":{"text":"private"},"customData":"private"}]}),
        );
        let text = v.to_string();
        assert!(!text.contains("secret"));
        assert!(!text.contains("private"));
        assert_eq!(v["widgets"][0]["x"], 0.5);
        assert_eq!(v["widgets"][0]["y"], 0.5);
        let refs = vec![Reference {
            pack_id: 1,
            version: 2,
        }];
        let cloud = cloud_payload(
            &Scheme {
                id: "a".repeat(32),
                title: "test".into(),
                created_at: String::new(),
                snapshot: Some(json!({"private":"secret"})),
                refs,
                cloud_revisions: BTreeMap::new(),
            },
            0,
        );
        assert!(cloud.get("snapshot").is_none());
        assert!(!Saved::default().cloud_enabled.values().any(|v| *v));
    }
    #[test]
    fn local_ids_cannot_be_paths() {
        assert!(canonical_id("../x").is_err());
        assert!(canonical_id("../../00000000000000000000000000000000").is_err());
        assert_eq!(
            canonical_id("00000000-0000-0000-0000-000000000000").unwrap(),
            "00000000000000000000000000000000"
        );
    }
}
