//! Authenticated loopback MCP. Desktop owns writes/events; stdio clients only relay requests.
mod config;
pub(crate) mod autostart;
mod stdio;
use agent_kernel::{Tool,ToolCtx};
use axum::{body::Bytes,extract::{DefaultBodyLimit,State},http::{HeaderMap,StatusCode},response::{IntoResponse,Response},routing::post,Json,Router};
use open_claude_design::{DesignTool,Store};
use serde_json::{json,Value};
use std::{collections::BTreeMap,path::PathBuf,sync::{Arc,Mutex},time::{SystemTime,UNIX_EPOCH}};
use tauri::Manager;
const PORT:u16=58140;
const READ:&[&str]=&["format","list","read","manifest"];
const EDIT:&[&str]=&["create","save","complete","copy"];
const GUIDE:&str="Operate the user's open-claude-design projects in Xiaoban Desktop. Call xiaoban_design_read action=format first. Find the intended project with list; read its current revision and source file before editing. Preserve omitted files and unknown canvas fields. Save changed files with expected_revision; on conflict read again. Use a stable request_id for create/copy retries. Complete only after checking issues. Changes appear in the user's live canvas. Use xiaoban_design_open with the project id to open or focus its native Xiaoban canvas when the user wants to view it. Never claim a visual check without seeing a rendered preview.";
fn lock<T>(m:&Mutex<T>)->std::sync::MutexGuard<'_,T>{m.lock().unwrap_or_else(|e|e.into_inner())}
fn now()->u64{SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64}
fn secret()->Result<String,String>{let mut b=[0u8;32];getrandom::fill(&mut b).map_err(|e|e.to_string())?;Ok(b.iter().map(|b|format!("{b:02x}")).collect())}
fn config_path(app:&tauri::AppHandle)->Result<PathBuf,String>{Ok(crate::storage::root(app)?.join("config/design-mcp.json"))}
#[derive(Default)]
pub struct DesignMcpState {
    server:Mutex<Option<tokio::task::JoinHandle<()>>>,
    access:Arc<Mutex<config::Config>>,
    seen:Arc<Mutex<BTreeMap<String,u64>>>,
    handshakes:Arc<Mutex<BTreeMap<String,u64>>>,
    operation:tokio::sync::Mutex<()>,
    error:Mutex<Option<String>>,
}
#[derive(Clone)]
struct Service { open:Arc<dyn Fn(&str)->Result<(),String>+Send+Sync>, tool:Arc<DesignTool>, store:Store, access:Arc<Mutex<config::Config>>, seen:Arc<Mutex<BTreeMap<String,u64>>>, handshakes:Arc<Mutex<BTreeMap<String,u64>>> }
fn tools(tool:&DesignTool)->Value{
    let spec=tool.spec();
    let entries=[("xiaoban_design_read",READ,true),("xiaoban_design_edit",EDIT,false)].map(|(name,actions,read)|{
        let mut schema=spec.schema.clone();schema["properties"]["action"]["enum"]=json!(actions);
        json!({"name":name,"description":if read{"Read Xiaoban design projects. Use format first, list to find a project, read with id and optional file for its source, manifest for design tokens."}else{spec.description.as_str()},"inputSchema":schema,"annotations":{"readOnlyHint":read,"destructiveHint":!read,"openWorldHint":false}})
    });
    let mut entries=entries.to_vec();
    entries.push(json!({"name":"xiaoban_design_open","description":"Open a design project in Xiaoban Desktop's native canvas window, or focus its existing window. Use when the user asks to open/show a design, or to present a created design. Does not modify project contents. Get the design id with xiaoban_design_read action=list or from a create result.","inputSchema":{"type":"object","properties":{"id":{"type":"string","description":"Existing design project id (design-...)."}},"required":["id"],"additionalProperties":false},"annotations":{"readOnlyHint":false,"destructiveHint":false,"idempotentHint":true,"openWorldHint":false}}));
    json!({"tools":entries})
}
fn failure(id:Value,code:i32,message:&str)->Value{json!({"jsonrpc":"2.0","id":id,"error":{"code":code,"message":message}})}
fn dispatch(service:&Service,client:&str,request:Value)->Option<Value>{
    let id=request.get("id").cloned();
    if request["jsonrpc"]!="2.0"||!request["method"].is_string(){return Some(failure(id.unwrap_or(Value::Null),-32600,"Invalid Request"))}
    let method=request["method"].as_str().unwrap();
    let Some(id)=id else{return None};
    let result=match method{
        "initialize"=>{
            lock(&service.handshakes).insert(client.into(),now());
            let requested=request["params"]["protocolVersion"].as_str().unwrap_or("");
            let protocol=if ["2024-11-05","2025-03-26","2025-06-18"].contains(&requested){requested}else{"2025-06-18"};
            json!({"protocolVersion":protocol,"capabilities":{"tools":{}},"serverInfo":{"name":"xiaoban-design","version":env!("CARGO_PKG_VERSION")},"instructions":GUIDE})
        },
        "ping"=>json!({}),
        "tools/list"=>tools(&service.tool),
        "tools/call"=>{
            let name=request["params"]["name"].as_str().unwrap_or("");
            let args=request["params"].get("arguments").cloned().unwrap_or(json!({}));
            let action=args["action"].as_str().unwrap_or("");
            let allowed=match name{"xiaoban_design_read"=>READ.contains(&action),"xiaoban_design_edit"=>EDIT.contains(&action),"xiaoban_design_open"=>args.as_object().is_some_and(|o|o.len()==1&&o.get("id").is_some_and(Value::is_string)),_=>false};
            if !allowed {return Some(failure(id,-32602,"Unknown tool or unsupported action"))}
            lock(&service.seen).insert(client.into(),now());
            // A read must not link the project into an unrelated chat or change its revision.
            if name=="xiaoban_design_open" {
                let result=(||->Result<Value,String>{
                    let project=service.store.load(args["id"].as_str().ok_or("缺少 id")?)?;
                    (service.open)(&project.id)?;
                    Ok(json!({"opened":true,"id":project.id,"title":project.title,"revision":project.revision,"location":"小伴桌面独立设计画板"}))
                })();
                match result{Ok(value)=>json!({"content":[{"type":"text","text":value.to_string()}],"structuredContent":value,"isError":false}),Err(e)=>json!({"content":[{"type":"text","text":e}],"isError":true})}
            }else if action=="read" {
                let result=(||->Result<Value,String>{let p=service.store.load(args["id"].as_str().ok_or("缺少 id")?)?;
                    let file=args["file"].as_str().map(|f|p.files.get(f).map(|s|json!({"name":f,"text":s})).ok_or("设计文件不存在")).transpose()?;
                    Ok(json!({"project":open_claude_design::summary(&p),"canvas":p.canvas,"files":p.files.keys().collect::<Vec<_>>(),"file":file}))})();
                match result{Ok(value)=>json!({"content":[{"type":"text","text":value.to_string()}],"structuredContent":value,"isError":false}),Err(e)=>json!({"content":[{"type":"text","text":e}],"isError":true})}
            }else{
                let out=service.tool.execute_result(args,&ToolCtx::detached(&format!("mcp-{client}")));
                let mut result=json!({"content":[{"type":"text","text":out.output}],"isError":out.is_error});
                if let Some(value)=out.structured_content {result["structuredContent"]=if value.is_object(){value}else{json!({"items":value})};}result
            }
        },
        _=>return Some(failure(id,-32601,"Method not found"))
    };
    Some(json!({"jsonrpc":"2.0","id":id,"result":result}))
}
fn authorized(headers:&HeaderMap,cfg:&config::Config)->Option<String>{
    if !cfg.enabled||headers.contains_key("origin"){return None}
    let token=headers.get("authorization")?.to_str().ok()?.strip_prefix("Bearer ")?;
    cfg.clients.iter().find(|(_,v)|v.len()==token.len()&&v.bytes().zip(token.bytes()).fold(0u8,|a,(x,y)|a|(x^y))==0).map(|(c,_)|c.clone())
}
async fn handle(State(service):State<Service>,headers:HeaderMap,body:Bytes)->Response{
    let Some(client)=authorized(&headers,&lock(&service.access))else{return StatusCode::UNAUTHORIZED.into_response()};
    let request=match serde_json::from_slice(&body){Ok(v)=>v,Err(_)=>return (StatusCode::BAD_REQUEST,Json(failure(Value::Null,-32700,"Parse error"))).into_response()};
    match tauri::async_runtime::spawn_blocking(move||dispatch(&service,&client,request)).await{
        Ok(Some(value))=>Json(value).into_response(),Ok(None)=>StatusCode::ACCEPTED.into_response(),Err(_)=>StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}
async fn ensure_running(app:&tauri::AppHandle,cfg:&config::Config)->Result<(),String>{
    let state=app.state::<DesignMcpState>();
    if lock(&state.server).as_ref().is_some_and(|h|!h.is_finished()){*lock(&state.access)=cfg.clone();return Ok(())}
    let listener=tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST,PORT)).await.map_err(|e|format!("本机 MCP 端口 {PORT} 无法启动：{e}"))?;
    let store=crate::design::store(app)?;let host=app.clone();let open_host=app.clone();
    let service=Service{open:Arc::new(move|id|{
        let control=open_host.get_webview_window("control").ok_or("小伴控制台未就绪")?;
        tauri::async_runtime::block_on(crate::design_window::design_window_open(open_host.clone(),control,id.to_string()))
    }),tool:Arc::new(DesignTool::new(store.clone(),move|p|crate::design::saved(&host,p))),store,access:state.access.clone(),seen:state.seen.clone(),handshakes:state.handshakes.clone()};
    *lock(&state.access)=cfg.clone();
    let router=Router::new().route("/mcp",post(handle)).layer(DefaultBodyLimit::disable()).with_state(service);
    *lock(&state.server)=Some(tokio::spawn(async move{let _=axum::serve(listener,router).await;}));
    *lock(&state.error)=None;Ok(())
}
fn guard(window:&tauri::WebviewWindow)->Result<(),String>{if window.label()=="control"{Ok(())}else{Err("请在小伴设置中管理协作连接".into())}}
fn status(app:&tauri::AppHandle)->Result<Value,String>{
    let state=app.state::<DesignMcpState>();let cfg=config::read(&config_path(app)?)?;let seen=lock(&state.seen);
    Ok(json!({"running":lock(&state.server).as_ref().is_some_and(|h|!h.is_finished()),"enabled":cfg.enabled,"url":format!("http://127.0.0.1:{PORT}/mcp"),"error":*lock(&state.error),"clients":config::CLIENTS.iter().map(|c|json!({"id":c,"configured":cfg.clients.contains_key(*c),"lastCall":seen.get(*c),"lastHandshake":lock(&state.handshakes).get(*c),"configPath":config::client_paths(c).ok().map(|paths|paths.iter().map(|p|p.to_string_lossy().to_string()).collect::<Vec<_>>().join("\n"))})).collect::<Vec<_>>()}))
}
#[tauri::command]
pub fn design_mcp_status(app:tauri::AppHandle,window:tauri::WebviewWindow)->Result<Value,String>{guard(&window)?;status(&app)}
#[tauri::command]
pub async fn design_mcp_set_enabled(app:tauri::AppHandle,window:tauri::WebviewWindow,enabled:bool)->Result<Value,String>{
    guard(&window)?;let state=app.state::<DesignMcpState>();let _op=state.operation.lock().await;
    let path=config_path(&app)?;let mut cfg=config::read(&path)?;cfg.enabled=enabled;
    config::save(&path,&cfg)?;
    if enabled{ensure_running(&app,&cfg).await?;}else{lock(&state.access).enabled=false;let task={lock(&state.server).take()};if let Some(task)=task{task.abort();let _=task.await;}}
    status(&app)
}
#[tauri::command]
pub async fn design_mcp_configure(app:tauri::AppHandle,window:tauri::WebviewWindow,client:String,remove:bool)->Result<Value,String>{
    guard(&window)?;if !config::CLIENTS.contains(&client.as_str()){return Err("未知协作客户端".into())}
    let state=app.state::<DesignMcpState>();let _op=state.operation.lock().await;
    let path=config_path(&app)?;let mut cfg=config::read(&path)?;
    if remove{
        // Revoke first, including existing stdio processes; cleanup never restores an obsolete backup.
        cfg.clients.remove(&client);config::save(&path,&cfg)?;*lock(&state.access)=cfg.clone();lock(&state.seen).remove(&client);lock(&state.handshakes).remove(&client);
        if client!="chatgpt-web"{for target in config::client_paths(&client)?{config::update(&target,&client,None)?;}}
    }else{
        if !cfg.clients.contains_key(&client){cfg.clients.insert(client.clone(),secret()?);}
        cfg.enabled=true;
        if client!="chatgpt-web"{for target in config::client_paths(&client)?{config::update(&target,&client,Some(config::entry(&std::env::current_exe().map_err(|e|e.to_string())?,&path,&client)))?;}}
        config::save(&path,&cfg)?;ensure_running(&app,&cfg).await?;
    }
    let mut value=status(&app)?;
    if client=="chatgpt-web"&&!remove{
        // No token in the command. The stdio bridge reads a local owner-only config file.
        let command=format!("{} --design-mcp-stdio --config {} --client chatgpt-web",quote(&std::env::current_exe().map_err(|e|e.to_string())?.to_string_lossy()),quote(&path.to_string_lossy()));
        value["tunnelCommand"]=json!(command);
    }
    Ok(value)
}
fn quote(s:&str)->String{if cfg!(windows){format!("\"{}\"",s.replace('"',"\\\""))}else{format!("'{}'",s.replace('\'',"'\\''"))}}
#[tauri::command]
pub async fn design_mcp_open_settings(app:tauri::AppHandle,window:tauri::WebviewWindow)->Result<(),String>{
    if window.label()!="control"&&crate::design_window::project_id(window.label()).is_none(){return Err("请从设计画板打开协作设置".into())}
    let control=app.get_webview_window("control").ok_or("控制台未就绪")?;
    crate::configuration_window::configuration_window_open(app,control,"settings".into(),Some("agent".into()),None).await
}
pub fn start(app:&tauri::AppHandle){
    let app=app.clone();tauri::async_runtime::spawn(async move{
        let state=app.state::<DesignMcpState>();let _op=state.operation.lock().await;
        let result=async{let path=config_path(&app)?;let cfg=config::read(&path)?;if cfg.enabled{ensure_running(&app,&cfg).await?;}Ok::<_,String>(())}.await;
        if let Err(e)=result{*lock(&app.state::<DesignMcpState>().error)=Some(e);}
    });
}
pub fn stdio_main()->Option<i32>{stdio::main()}

#[cfg(test)] mod tests{
    use super::*;
    fn service()->Service{
        let root=std::env::temp_dir().join(format!("design-mcp-test-{}",secret().unwrap()));let store=Store::new(root);
        Service{open:Arc::new(|_|Ok(())),tool:Arc::new(DesignTool::new(store.clone(),|_|Ok(()))),store,access:Default::default(),seen:Default::default(),handshakes:Default::default()}
    }
    #[test] fn authenticate_and_revoke(){let mut cfg=config::Config::default();cfg.enabled=true;cfg.clients.insert("codex".into(),"secret".into());let mut h=HeaderMap::new();assert!(authorized(&h,&cfg).is_none());h.insert("authorization","Bearer secret".parse().unwrap());assert_eq!(authorized(&h,&cfg).as_deref(),Some("codex"));h.insert("origin","https://evil.test".parse().unwrap());assert!(authorized(&h,&cfg).is_none());h.remove("origin");cfg.clients.clear();assert!(authorized(&h,&cfg).is_none());}
    #[test] fn protocol_and_tool_boundaries(){let s=service();let call=|method:&str,params:Value|dispatch(&s,"codex",json!({"jsonrpc":"2.0","id":1,"method":method,"params":params})).unwrap();
        assert_eq!(call("initialize",json!({"protocolVersion":"2025-03-26"}))["result"]["protocolVersion"],"2025-03-26");
        assert!(lock(&s.handshakes).contains_key("codex"));assert!(lock(&s.seen).is_empty());
        assert_eq!(call("tools/list",json!({}))["result"]["tools"].as_array().unwrap().len(),3);
        assert!(call("tools/call",json!({"name":"xiaoban_design_read","arguments":{"action":"save"}}))["error"].is_object());
        assert!(call("tools/call",json!({"name":"xiaoban_design_edit","arguments":{"action":"delete"}}))["error"].is_object());
        assert!(dispatch(&s,"codex",json!({"jsonrpc":"2.0","method":"notifications/initialized"})).is_none());
        assert_eq!(call("tools/call",json!({"name":"xiaoban_design_read","arguments":{"action":"format"}}))["result"]["isError"],false);
    }
    #[test] fn opening_validates_project_and_reports_window_failures_without_changing_content(){
        let mut s=service();let opened=Arc::new(Mutex::new(Vec::<String>::new()));let capture=opened.clone();
        s.open=Arc::new(move|id|{lock(&capture).push(id.into());Ok(())});
        let call=|s:&Service,id:&str|dispatch(s,"codex",json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"xiaoban_design_open","arguments":{"id":id}}})).unwrap();
        assert_eq!(call(&s,"../../outside")["result"]["isError"],true);assert!(lock(&opened).is_empty());
        let p=s.store.create("Open test".into(),"test",json!({"v":3,"boards":{"Main.dc.html":{"title":"Home","w":390,"h":844,"x":0,"y":0}},"order":["Main.dc.html"]}),std::collections::BTreeMap::from([("Main.dc.html".into(),"<x-dc>Test</x-dc>".into())])).unwrap();
        for _ in 0..2 {let out=call(&s,&p.id);assert_eq!(out["result"]["isError"],false);assert_eq!(out["result"]["structuredContent"]["opened"],true);}
        assert_eq!(lock(&opened).len(),2);assert_eq!(s.store.load(&p.id).unwrap(),p);
        s.open=Arc::new(|_|Err("window failed".into()));let out=call(&s,&p.id);assert_eq!(out["result"]["isError"],true);assert!(out["result"]["structuredContent"].is_null());s.store.remove(&p.id).unwrap();
    }
    #[tokio::test] async fn http_roundtrip_writes_shared_store_and_rejects_stale_revision(){
        let s=service();{let mut cfg=lock(&s.access);cfg.enabled=true;cfg.clients.insert("test".into(),"test-secret".into());}
        let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();let addr=listener.local_addr().unwrap();
        let router=Router::new().route("/mcp",post(handle)).with_state(s.clone());
        let task=tokio::spawn(async move{axum::serve(listener,router).await.unwrap()});
        let http=reqwest::Client::new();let url=format!("http://{addr}/mcp");
        assert_eq!(http.post(&url).body("{}").send().await.unwrap().status(),401);
        let request=|action:Value|json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"xiaoban_design_edit","arguments":action}}).to_string();
        let create=request(json!({"action":"create","request_id":"once","title":"MCP integration","canvas":{"v":3,"boards":{"Main.dc.html":{"title":"Home","w":390,"h":844,"x":0,"y":0}},"order":["Main.dc.html"]},"files":{"Main.dc.html":"<x-dc><h1>Hello</h1></x-dc>"}}));
        let response=http.post(&url).bearer_auth("test-secret").body(create.clone()).send().await.unwrap().text().await.unwrap();let out:Value=serde_json::from_str(&response).unwrap();
        assert_eq!(out["result"]["isError"],false,"{out}");let id=out["result"]["structuredContent"]["project"]["id"].as_str().unwrap();let rev=out["result"]["structuredContent"]["project"]["revision"].as_u64().unwrap();
        assert_eq!(s.store.load(id).unwrap().files["Main.dc.html"],"<x-dc><h1>Hello</h1></x-dc>");
        let response=http.post(&url).bearer_auth("test-secret").body(create).send().await.unwrap().text().await.unwrap();let retry:Value=serde_json::from_str(&response).unwrap();assert_eq!(retry["result"]["structuredContent"]["project"]["id"],id);
        for (expected,error) in [(rev,false),(rev,true)] {
            let response=http.post(&url).bearer_auth("test-secret").body(request(json!({"action":"save","id":id,"expected_revision":expected,"files":{"Main.dc.html":"<x-dc><h1>Edited</h1></x-dc>"}}))).send().await.unwrap().text().await.unwrap();let out:Value=serde_json::from_str(&response).unwrap();assert_eq!(out["result"]["isError"],error,"{out}");
        }
        lock(&s.access).clients.clear();assert_eq!(http.post(&url).bearer_auth("test-secret").body("{}").send().await.unwrap().status(),401);
        s.store.remove(id).unwrap();task.abort();
    }

}
