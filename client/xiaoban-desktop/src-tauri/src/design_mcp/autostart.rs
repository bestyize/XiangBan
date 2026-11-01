//! Start only an authorized, enabled desktop service; never replay a design mutation.
use super::{config,PORT};
use serde_json::{json,Value};
use std::{fs::{File,OpenOptions,TryLockError},path::{Path,PathBuf},process::{Command,Stdio},thread,time::{Duration,Instant}};

pub const BACKGROUND_ARG:&str="--design-mcp-background";
pub fn background(args:impl IntoIterator<Item=impl AsRef<str>>)->bool{
    args.into_iter().any(|a|matches!(a.as_ref(),"--autostart"|BACKGROUND_ARG))
}
fn token(path:&Path,client:&str)->Result<String,String>{
    let cfg=config::read(path)?;
    if !cfg.enabled{return Err("小伴设计 MCP 已停止，请在小伴桌面 → 设置 → 外部 Agent 中启动".into())}
    cfg.clients.get(client).cloned().ok_or_else(||"此客户端授权已撤销，请在小伴桌面重新连接".into())
}
fn mac_bundle(exe:&Path)->Option<PathBuf>{
    let macos=exe.parent()?;let contents=macos.parent()?;let bundle=contents.parent()?;
    (macos.file_name()?=="MacOS"&&contents.file_name()?=="Contents"&&bundle.extension()?=="app").then(||bundle.to_path_buf())
}
fn launch(exe:&Path)->Result<(),String>{
    #[cfg(target_os="macos")]
    let mut command=if let Some(bundle)=mac_bundle(exe){
        let mut c=Command::new("/usr/bin/open");
        // Exact bundle path prevents a development build from launching the installed release.
        // -n avoids a Dock reopen (which would foreground the console); Tauri still enforces one instance.
        c.arg("-n").arg("-g").arg("-a").arg(bundle).arg("--args").arg(BACKGROUND_ARG);c
    }else{let mut c=Command::new(exe);c.arg(BACKGROUND_ARG);c};
    #[cfg(not(target_os="macos"))]
    let mut command={let mut c=Command::new(exe);c.arg(BACKGROUND_ARG);c};
    #[cfg(windows)]{
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x00000008|0x00000200); // DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP
    }
    #[cfg(unix)]{
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let mut child=command.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().map_err(|e|format!("无法启动小伴桌面：{e}"))?;
    thread::spawn(move||{let _=child.wait();});Ok(())
}
fn startup_lock(path:&Path)->Result<File,String>{
    let path=path.with_file_name("design-mcp-start.lock");
    if std::fs::symlink_metadata(&path).is_ok_and(|m|m.file_type().is_symlink()){return Err("MCP 启动锁不能是符号链接".into())}
    let mut options=OpenOptions::new();options.read(true).write(true).create(true);
    #[cfg(unix)]{use std::os::unix::fs::OpenOptionsExt;options.mode(0o600);}
    options.open(path).map_err(|e|format!("无法创建 MCP 启动锁：{e}"))
}
/// Only ping is retried. A failed authenticated HTTP response is not evidence that the app is absent.
fn probe(http:&reqwest::blocking::Client,url:&str,token:&str)->Result<bool,String>{
    let response=match http.post(url).bearer_auth(token).header("Content-Type","application/json").header("Accept","application/json, text/event-stream")
        .body(json!({"jsonrpc":"2.0","id":"xiaoban-startup-probe","method":"ping"}).to_string()).send(){
        Ok(r)=>r,
        Err(e) if e.is_connect()=>return Ok(false),
        Err(_)=>return Err("小伴 MCP 探测失败或超时；未重复启动应用，请检查本机服务".into()),
    };
    if !response.status().is_success(){return Err(format!("小伴 MCP 探测返回 {}，请检查服务与授权",response.status()))}
    let response:Value=serde_json::from_str(&response.text().map_err(|_|"无法读取 MCP 探测结果")?).map_err(|_|"本机端口返回的不是 MCP 响应")?;
    if response["jsonrpc"]!="2.0"||response["id"]!="xiaoban-startup-probe"||!response["result"].is_object(){return Err("本机端口未返回有效 MCP 探测结果".into())}
    Ok(true)
}
fn wait_ready(path:&Path,client:&str,timeout:Duration,mut probe:impl FnMut(&str)->Result<bool,String>,mut launch:impl FnMut()->Result<(),String>)->Result<String,String>{
    let deadline=Instant::now()+timeout;
    let mut held=None;
    loop{
        let current=token(path,client)?;
        if probe(&current)?{return Ok(current)}
        if Instant::now()>=deadline{return Err("等待小伴 MCP 启动超时，请打开小伴检查服务状态后重试".into())}
        if held.is_none(){
            let file=startup_lock(path)?;
            match file.try_lock(){
                Ok(())=>{
                    // Another client may have finished starting the app while we acquired the lock.
                    let current=token(path,client)?;
                    if probe(&current)?{return Ok(current)}
                    launch()?;held=Some(file);
                },
                Err(TryLockError::WouldBlock)=>(),
                Err(TryLockError::Error(e))=>return Err(format!("无法取得 MCP 启动锁：{e}")),
            }
        }
        thread::sleep(Duration::from_millis(200).min(deadline.saturating_duration_since(Instant::now())));
    }
}
pub fn ensure_ready(path:&Path,client:&str)->Result<String,String>{
    // Check before even resolving the executable or creating a startup lock.
    token(path,client)?;
    let exe=std::env::current_exe().map_err(|e|e.to_string())?;
    let http=reqwest::blocking::Client::builder().no_proxy().connect_timeout(Duration::from_millis(500)).timeout(Duration::from_secs(2)).build().map_err(|e|e.to_string())?;
    wait_ready(path,client,Duration::from_secs(30),|token|probe(&http,&format!("http://127.0.0.1:{PORT}/mcp"),token),||launch(&exe))
}

#[cfg(test)]mod tests{
    use super::*;
    use std::sync::{Arc,atomic::{AtomicBool,AtomicUsize,Ordering}};
    struct Fixture(PathBuf);
    impl Fixture{fn new()->Self{let p=std::env::temp_dir().join(format!("xiaoban-start-{}",super::super::secret().unwrap())).join("config.json");let mut cfg=config::Config::default();cfg.enabled=true;cfg.clients.insert("test".into(),"token".into());config::save(&p,&cfg).unwrap();Self(p)}}
    impl Drop for Fixture{fn drop(&mut self){let _=std::fs::remove_dir_all(self.0.parent().unwrap());}}
    #[test]fn disabled_and_revoked_never_probe_or_launch(){let f=Fixture::new();for enabled in [false,true]{let cfg=config::Config{enabled,clients:Default::default()};config::save(&f.0,&cfg).unwrap();assert!(wait_ready(&f.0,"test",Duration::from_millis(20),|_|panic!("must not probe"),||panic!("must not launch")).is_err());}}
    #[test]fn existing_service_does_not_launch(){let f=Fixture::new();assert_eq!(wait_ready(&f.0,"test",Duration::from_secs(1),|_|Ok(true),||panic!("must not launch")).unwrap(),"token");}
    #[test]fn timeout_launches_only_once_and_releases_lock(){let f=Fixture::new();let calls=AtomicUsize::new(0);assert!(wait_ready(&f.0,"test",Duration::from_millis(10),|_|Ok(false),||{calls.fetch_add(1,Ordering::SeqCst);Ok(())}).unwrap_err().contains("超时"));assert_eq!(calls.load(Ordering::SeqCst),1);assert!(startup_lock(&f.0).unwrap().try_lock().is_ok());}
    #[test]fn concurrent_clients_start_one_app(){let f=Fixture::new();let ready=Arc::new(AtomicBool::new(false));let count=Arc::new(AtomicUsize::new(0));let barrier=Arc::new(std::sync::Barrier::new(2));let threads:Vec<_>=(0..2).map(|_|{let p=f.0.clone();let ready=ready.clone();let count=count.clone();let barrier=barrier.clone();thread::spawn(move||{barrier.wait();wait_ready(&p,"test",Duration::from_secs(2),|_|Ok(ready.load(Ordering::SeqCst)),||{count.fetch_add(1,Ordering::SeqCst);thread::sleep(Duration::from_millis(40));ready.store(true,Ordering::SeqCst);Ok(())})})}).collect();for t in threads{assert!(t.join().unwrap().is_ok())}assert_eq!(count.load(Ordering::SeqCst),1);}
    #[test]fn revocation_during_start_stops_waiting(){let f=Fixture::new();assert!(wait_ready(&f.0,"test",Duration::from_secs(1),|_|Ok(false),||{let mut cfg=config::read(&f.0)?;cfg.clients.clear();config::save(&f.0,&cfg)}).unwrap_err().contains("撤销"));}
    #[test]fn probe_error_does_not_start_app(){let f=Fixture::new();assert!(wait_ready(&f.0,"test",Duration::from_secs(1),|_|Err("unauthorized".into()),||panic!("must not launch")).is_err());}
    #[test]fn bundled_paths_and_background_mode(){assert_eq!(mac_bundle(Path::new("/Applications/小伴.app/Contents/MacOS/xiaoban-desktop")),Some(PathBuf::from("/Applications/小伴.app")));assert!(mac_bundle(Path::new("/tmp/xiaoban-desktop")).is_none());assert!(background(["app",BACKGROUND_ARG]));assert!(background(["app","--autostart"]));assert!(!background(["app","--design-mcp-stdio"]));}
}
