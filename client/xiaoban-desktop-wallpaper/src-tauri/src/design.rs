//! Desktop adapter for the built-in design tool. Preview content never receives IPC.
use open_claude_design::{Project, Store};
use serde_json::{json,Value};
use std::{collections::BTreeMap,fs,io::Read};
use tauri::{Emitter,Manager};
use tauri_plugin_dialog::DialogExt;

pub fn store(app:&tauri::AppHandle)->Result<Store,String>{Ok(Store::new(crate::storage::root(app)?.join("data-library/files")))}
fn guard(window:&tauri::WebviewWindow)->Result<(),String>{if window.label()=="control"{Ok(())}else{Err("请在主窗口编辑设计稿".into())}}
pub fn saved(app:&tauri::AppHandle,p:&Project)->Result<(),String>{
    crate::data_library::upsert_design(app,p)?;
    let _=app.emit_to("control","design-changed",open_claude_design::summary(p));Ok(())
}
fn payload(p:Project)->Value{json!({"project":p,"origin":if cfg!(windows){"http://design-preview.localhost"}else{"design-preview://localhost"}})}
#[tauri::command]
pub fn design_list(app:tauri::AppHandle,window:tauri::WebviewWindow,session:String)->Result<Vec<Value>,String>{guard(&window)?;Ok(store(&app)?.list(&session)?.iter().map(open_claude_design::summary).collect())}
#[tauri::command]
pub fn design_read(app:tauri::AppHandle,window:tauri::WebviewWindow,id:String)->Result<Value,String>{guard(&window)?;let s=store(&app)?;let p=s.load(&id)?;let mut result=payload(p.clone());
    match s.draft(&id) {
        Ok(Some(draft)) if draft.files!=p.files||draft.canvas!=p.canvas||draft.title!=p.title => result["draft"]=json!(draft),
        Err(e) => result["draftError"]=json!(format!("恢复草稿损坏，已打开最后保存版本：{e}")),
        _ => ()
    }
    Ok(result)}
#[tauri::command]
pub fn design_draft_save(app:tauri::AppHandle,window:tauri::WebviewWindow,project:Project)->Result<(),String>{guard(&window)?;store(&app)?.save_draft(&project)}
#[tauri::command]
pub fn design_draft_clear(app:tauri::AppHandle,window:tauri::WebviewWindow,project:Project)->Result<(),String>{guard(&window)?;store(&app)?.clear_draft(&project)}
#[tauri::command]
pub fn design_save(app:tauri::AppHandle,window:tauri::WebviewWindow,project:Project,expected_revision:u64)->Result<Value,String>{
    guard(&window)?;if project.revision!=expected_revision{return Err("保存版本不匹配，请先刷新".into());}
    let p=store(&app)?.save(project,expected_revision)?;saved(&app,&p)?;Ok(payload(p))
}
#[tauri::command]
pub fn design_history(app:tauri::AppHandle,window:tauri::WebviewWindow,id:String)->Result<Vec<Value>,String>{guard(&window)?;Ok(store(&app)?.history(&id)?.iter().map(open_claude_design::summary).collect())}
#[tauri::command]
pub fn design_restore(app:tauri::AppHandle,window:tauri::WebviewWindow,id:String,revision:u64,expected_revision:u64)->Result<Value,String>{
    guard(&window)?;let s=store(&app)?;let p=s.history(&id)?.into_iter().find(|p|p.revision==revision).ok_or("这个版本已经不在了")?;
    let p=s.save(p,expected_revision)?;saved(&app,&p)?;Ok(payload(p))
}
#[tauri::command]
pub async fn design_export(window:tauri::WebviewWindow,id:String,format:String,project:Option<Project>)->Result<Option<String>,String>{
    guard(&window)?;let app=window.app_handle().clone();
    tauri::async_runtime::spawn_blocking(move||{
        let saved=store(&app)?.load(&id)?;
        let p=project.unwrap_or(saved);if p.id!=id{return Err("设计稿编号不一致".into());}open_claude_design::validate(&p)?;
        let (ext,bytes)=match format.as_str(){"zip"=>("zip",open_claude_design::archive::to_zip(&p)?),"html"=>("html",open_claude_design::archive::standalone(&p)?.into_bytes()),_=>return Err("不支持的导出格式".into())};
        let name=p.title.chars().filter(|c|!"/\\:*?\"<>|".contains(*c)&&!c.is_control()).collect::<String>();
        let Some(file)=app.dialog().file().set_title("导出设计稿").set_file_name(format!("{name}.{ext}")).add_filter("设计稿",&[ext]).blocking_save_file() else{return Ok(None)};
        let path=file.into_path().map_err(|_|"请选择本机文件")?;fs::write(&path,bytes).map_err(|e|e.to_string())?;Ok(Some(path.display().to_string()))
    }).await.map_err(|e|e.to_string())?
}
#[tauri::command]
pub async fn design_import(window:tauri::WebviewWindow,session:String)->Result<Option<Value>,String>{
    guard(&window)?;let app=window.app_handle().clone();
    tauri::async_runtime::spawn_blocking(move||{
        let Some(file)=app.dialog().file().set_title("导入设计稿").add_filter("设计源包或画板",&["zip","html"]).blocking_pick_file()else{return Ok(None)};
        let path=file.into_path().map_err(|_|"请选择本机文件")?;
        let bytes=fs::File::open(&path).map_err(|e|e.to_string())?.take(16*1024*1024+1).bytes().collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())?;
        if bytes.len()>16*1024*1024{return Err("设计源包超过 16MiB".into());}
        let (canvas,files)=if path.extension().and_then(|e|e.to_str()).is_some_and(|e|e.eq_ignore_ascii_case("zip")) {open_claude_design::archive::from_zip(&bytes)?}else{
            let source=String::from_utf8(bytes).map_err(|_|"HTML 需要使用 UTF-8 编码")?;
            (json!({"v":3,"boards":{"Main.dc.html":{"title":"导入画板","x":0,"y":0,"w":1280,"h":800}},"order":["Main.dc.html"],"notes":{}}),BTreeMap::from([("Main.dc.html".into(),source)]))
        };
        let title=path.file_stem().and_then(|s|s.to_str()).unwrap_or("导入设计稿").chars().take(120).collect();
        let p=store(&app)?.create(title,&session,canvas,files)?;saved(&app,&p)?;Ok(Some(payload(p)))
    }).await.map_err(|e|e.to_string())?
}
fn decode(s:&str)->Option<String>{let b=s.as_bytes();let(mut i,mut out)=(0,Vec::new());while i<b.len(){if b[i]==b'%'{out.push(u8::from_str_radix(std::str::from_utf8(b.get(i+1..i+3)?).ok()?,16).ok()?);i+=3;}else{out.push(b[i]);i+=1;}}String::from_utf8(out).ok()}
/// This CSP applies only to the opaque preview origin, never the privileged control UI.
pub fn csp()->&'static str{"default-src 'none'; script-src 'unsafe-inline' 'unsafe-eval'; style-src 'unsafe-inline' https://fonts.googleapis.com; img-src data: blob:; font-src data: https://fonts.gstatic.com; connect-src 'none'; frame-src 'none'; object-src 'none'; base-uri 'none'; form-action 'none'"}
pub fn serve(app:&tauri::AppHandle,request:tauri::http::Request<Vec<u8>>)->tauri::http::Response<Vec<u8>>{
    let result=(||->Result<Vec<u8>,String>{
        let path=decode(request.uri().path()).ok_or("路径编码无效")?;let mut parts=path.trim_start_matches('/').splitn(3,'/');
        let id=parts.next().ok_or("缺少 id")?;let revision=parts.next().ok_or("缺少版本")?.parse::<u64>().map_err(|_|"版本无效")?;let name=parts.next().ok_or("缺少画板")?;
        let p=store(app)?.load(id)?;if revision!=p.revision{return Err("预览版本已更新，请刷新".into());}
        let text=p.files.get(name).ok_or("设计文件不存在")?;if !name.ends_with(".dc.html"){return Err("此预览仅允许画板".into());}
        Ok(open_claude_design::archive::preview(text).into_bytes())
    })();
    let (status,body)=match result{Ok(b)=>(200,b),Err(e)=>(400,format!("<!doctype html><meta charset=utf-8><p>预览未就绪：{}</p>",e.replace('&',"&amp;").replace('<',"&lt;")).into_bytes())};
    tauri::http::Response::builder().status(status).header("Content-Type","text/html; charset=utf-8").header("Content-Security-Policy",csp()).header("X-Content-Type-Options","nosniff").header("Referrer-Policy","no-referrer").header("Cache-Control","no-store").body(body).unwrap_or_default()
}
