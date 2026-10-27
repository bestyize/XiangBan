use crate::package::{valid_package, Result};
use agent_kernel::ToolCtx;
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};
use std::{fs, io::{Read, Write}, path::{Path, PathBuf}, process::{Command, Stdio}, sync::{mpsc, Arc}, thread, time::{Duration, Instant}};

pub trait Transport: Send + Sync {
    fn run(&self, args: &[String], stdin: &[u8], ctx: &ToolCtx, max: usize) -> Result<Vec<u8>>;
}

pub struct ProcessTransport { executable: PathBuf }
impl ProcessTransport {
    pub fn new(executable: impl Into<PathBuf>) -> Self { Self { executable:executable.into() } }
}

impl Transport for ProcessTransport {
    fn run(&self, args: &[String], input: &[u8], ctx: &ToolCtx, max: usize) -> Result<Vec<u8>> {
        if ctx.is_cancelled() { return Err("cancelled before ADB dispatch".into()); }
        let mut command=Command::new(&self.executable);
        command.args(args).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
        #[cfg(windows)] {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }
        let mut child=command.spawn().map_err(|e| format!("无法启动 ADB：{e}。请安装 Android Platform Tools，并把 adb 加入 PATH，或设置 XIAOBAN_ADB。"))?;
        if let Some(mut stdin)=child.stdin.take() {
            if let Err(error)=stdin.write_all(input) { let _=child.kill(); let _=child.wait(); return Err(error.to_string()); }
        }
        let (tx,rx)=mpsc::channel();
        for (kind,pipe,cap) in [
            ("stdout",child.stdout.take().map(|p| Box::new(p) as Box<dyn Read+Send>),max),
            ("stderr",child.stderr.take().map(|p| Box::new(p) as Box<dyn Read+Send>),32*1024),
        ] {
            if let Some(pipe)=pipe {
                let tx=tx.clone();
                thread::spawn(move || {
                    let mut bytes=Vec::new();
                    let result=pipe.take(cap as u64+1).read_to_end(&mut bytes).map_err(|e|e.to_string()).and_then(|_| {
                        if bytes.len()>cap { Err("ADB output exceeded budget".into()) } else { Ok(bytes) }
                    });
                    let _=tx.send((kind,result));
                });
            }
        }
        drop(tx);
        let start=Instant::now();
        let status=loop {
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) if ctx.is_cancelled() || start.elapsed()>Duration::from_secs(45) => {
                    let _=child.kill(); let _=child.wait();
                    return Err("ADB cancelled or timed out; an action may already have happened. Observe the device before retrying.".into());
                },
                Ok(None) => thread::sleep(Duration::from_millis(25)),
                Err(error) => { let _=child.kill(); let _=child.wait(); return Err(error.to_string()); },
            }
        };
        let mut stdout=Vec::new(); let mut stderr=Vec::new();
        for _ in 0..2 {
            let (kind,result)=rx.recv_timeout(Duration::from_secs(2)).map_err(|_| "ADB output reader did not finish")?;
            if kind=="stdout" { stdout=result?; } else { stderr=result?; }
        }
        if !status.success() { return Err(format!("ADB failed: {} {}",String::from_utf8_lossy(&stderr),String::from_utf8_lossy(&stdout))); }
        Ok(stdout)
    }
}

#[derive(Clone)]
pub struct Adb { transport: Arc<dyn Transport> }
impl Adb {
    pub fn new(executable: impl Into<PathBuf>) -> Self { Self { transport:Arc::new(ProcessTransport::new(executable)) } }
    pub fn with_transport(transport: Arc<dyn Transport>) -> Self { Self { transport } }

    fn command(&self, args: &[&str], input: &[u8], ctx: &ToolCtx, max: usize) -> Result<Vec<u8>> {
        if ctx.is_cancelled() { return Err("cancelled before ADB dispatch".into()); }
        self.transport.run(&args.iter().map(|s|s.to_string()).collect::<Vec<_>>(),input,ctx,max)
    }

    pub fn devices(&self, ctx: &ToolCtx) -> Result<Value> {
        let bytes=self.command(&["devices","-l"],&[],ctx,64*1024)?;
        let text=String::from_utf8_lossy(&bytes);
        let devices:Vec<_>=text.lines().filter_map(|line| {
            let mut parts=line.split_whitespace(); let serial=parts.next()?; let state=parts.next()?;
            if !matches!(state,"device"|"offline"|"unauthorized"|"recovery"|"sideload") { return None; }
            Some(json!({"serial":serial,"state":state,"details":parts.collect::<Vec<_>>().join(" ")}))
        }).collect();
        Ok(json!({"devices":devices,"requires_explicit_serial":true}))
    }

    fn ready(&self, serial: &str, ctx: &ToolCtx) -> Result<()> {
        if serial.is_empty() || serial.len()>200 || serial.starts_with('-')
            || !serial.bytes().all(|b| b.is_ascii_alphanumeric() || b"._-:[]".contains(&b)) {
            return Err("invalid explicit device serial".into());
        }
        let devices=self.devices(ctx)?;
        if !devices["devices"].as_array().is_some_and(|items| items.iter().any(|d|d["serial"]==serial && d["state"]=="device")) {
            return Err("device is not connected and authorized; inspect phone_devices and approve debugging on the phone".into());
        }
        Ok(())
    }

    fn shell(&self, serial: &str, args: &[&str], ctx: &ToolCtx) -> Result<String> {
        // adb shell joins arguments on the remote side; quote EVERY token for Android's POSIX shell.
        let command=args.iter().map(|s|shell_quote(s)).collect::<Vec<_>>().join(" ");
        let bytes=self.command(&["-s",serial,"shell",&command],&[],ctx,1024*1024)?;
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }

    fn foreground(&self, serial: &str, ctx: &ToolCtx) -> Result<String> {
        let text=self.shell(serial,&["dumpsys","activity","activities"],ctx)?;
        foreground_from(&text).ok_or("unable to identify foreground application; unlock the phone and observe it".into())
    }

    fn guard(&self, serial: &str, expected: &str, ctx: &ToolCtx) -> Result<()> {
        if !valid_package(expected) { return Err("invalid expected package".into()); }
        let actual=self.foreground(serial,ctx)?;
        if actual!=expected { return Err(format!("foreground changed: expected {expected}, observed {actual}; no input dispatched")); }
        Ok(())
    }

    pub fn status(&self, serial: &str, package: Option<&str>, ctx: &ToolCtx) -> Result<Value> {
        self.ready(serial,ctx)?;
        let foreground=self.foreground(serial,ctx)?;
        let package=package.unwrap_or(&foreground);
        if !valid_package(package) { return Err("invalid package".into()); }
        let details=self.shell(serial,&["dumpsys","package",package],ctx)?;
        let version=details.lines().find_map(|line|line.trim().strip_prefix("versionName=")).map(str::to_owned);
        let dimensions=self.shell(serial,&["wm","size"],ctx)?;
        Ok(json!({"serial":serial,"foreground_package":foreground,"package_name":package,"app_version":version,"display":dimensions.trim()}))
    }

    pub fn screenshot(&self, serial: &str, ctx: &ToolCtx) -> Result<Vec<u8>> {
        self.ready(serial,ctx)?;
        let data=self.command(&["-s",serial,"exec-out","screencap","-p"],&[],ctx,4*1024*1024)?;
        if data.len()<24 || !data.starts_with(b"\x89PNG\r\n\x1a\n") { return Err("device did not return a PNG screenshot".into()); }
        Ok(data)
    }

    pub fn ui(&self, serial: &str, ctx: &ToolCtx) -> Result<String> {
        self.ready(serial,ctx)?;
        let text=self.shell(serial,&["uiautomator","dump","/dev/tty"],ctx)?;
        let start=text.find("<?xml").or_else(||text.find("<hierarchy")).ok_or("UI tree unavailable; use a screenshot for custom-drawn pages")?;
        let end=text.rfind("</hierarchy>").ok_or("incomplete UI tree")?+"</hierarchy>".len();
        Ok(text[start..end].to_string())
    }

    pub fn connect(&self, mode: &str, endpoint: &str, code: Option<&str>, ctx: &ToolCtx) -> Result<Value> {
        validate_endpoint(endpoint)?;
        if !matches!(mode,"pair"|"connect"|"disconnect") { return Err("invalid connection mode".into()); }
        let input=if mode=="pair" {
            let code=code.ok_or("pairing requires the six-digit code displayed on the phone")?;
            if code.len()!=6 || !code.bytes().all(|b|b.is_ascii_digit()) { return Err("pairing code must be six digits".into()); }
            format!("{code}\n")
        } else { String::new() };
        let output=self.command(&[mode,endpoint],input.as_bytes(),ctx,32*1024)?;
        let text=String::from_utf8_lossy(&output);
        if ["failed","cannot","unable","error"].iter().any(|bad|text.to_lowercase().contains(bad)) { return Err(text.into_owned()); }
        Ok(json!({"result":text.trim(),"verify_with":"phone_devices"}))
    }

    pub fn action(&self, serial: &str, package: &str, action: &str, args: &Value, ctx: &ToolCtx) -> Result<Value> {
        self.ready(serial,ctx)?;
        if !valid_package(package) { return Err("invalid package name".into()); }
        if action!="launch" { self.guard(serial,package,ctx)?; }
        let result=match action {
            "launch" => self.shell(serial,&["monkey","-p",package,"-c","android.intent.category.LAUNCHER","1"],ctx)?,
            "tap" => {
                let x=coordinate(args,"x")?.to_string(); let y=coordinate(args,"y")?.to_string();
                self.shell(serial,&["input","tap",&x,&y],ctx)?
            },
            "swipe" => {
                let x=coordinate(args,"x")?.to_string(); let y=coordinate(args,"y")?.to_string();
                let x2=coordinate(args,"x2")?.to_string(); let y2=coordinate(args,"y2")?.to_string();
                let duration=args["duration_ms"].as_u64().unwrap_or(300);
                if !(50..=3000).contains(&duration) { return Err("swipe duration must be 50..3000 ms".into()); }
                self.shell(serial,&["input","swipe",&x,&y,&x2,&y2,&duration.to_string()],ctx)?
            },
            "key" => {
                let key=match args["key"].as_str() {
                    Some("back")=>"4",Some("home")=>"3",Some("enter")=>"66",Some("delete")=>"67",Some("recents")=>"187",
                    _=>return Err("unsupported system key".into()),
                };
                self.shell(serial,&["input","keyevent",key],ctx)?
            },
            "text" => {
                let text=args["text"].as_str().ok_or("text is required")?;
                if text.is_empty() || text.len()>12000 || text.chars().any(|c|c=='\0' || (c.is_control() && c!='\n' && c!='\t')) {
                    return Err("invalid text or text exceeds 12000 UTF-8 bytes".into());
                }
                if !text.is_ascii() || text.contains("%s") || text.contains(['\n','\t']) {
                    let ime=self.shell(serial,&["settings","get","secure","default_input_method"],ctx)?;
                    if ime.trim()!="com.android.adbkeyboard/.AdbIME" {
                        return Err("Unicode input requires ADB Keyboard installed and selected on the phone (com.android.adbkeyboard/.AdbIME). No input or keyboard setting was changed.".into());
                    }
                    self.guard(serial,package,ctx)?;
                    self.shell(serial,&["am","broadcast","-a","ADB_INPUT_B64","-p","com.android.adbkeyboard","--es","msg",&STANDARD.encode(text.as_bytes())],ctx)?
                } else {
                    self.shell(serial,&["input","text",&text.replace(' ',"%s")],ctx)?
                }
            },
            _=>return Err("unknown phone action".into()),
        };
        if action=="launch" && result.contains("No activities found") { return Err(result); }
        Ok(json!({"dispatched":true,"result":result.trim(),"requires_live_verification":true}))
    }

    pub fn push_image(&self, serial: &str, source: &Path, workspace: &Path, ctx: &ToolCtx) -> Result<Value> {
        use sha2::{Digest,Sha256};
        self.ready(serial,ctx)?;
        let root=workspace.canonicalize().map_err(|e|e.to_string())?;
        let source=if source.is_absolute() {source.to_path_buf()} else {workspace.join(source)}.canonicalize().map_err(|e|e.to_string())?;
        if !source.starts_with(&root) || !source.is_file() || fs::metadata(&source).map_err(|e|e.to_string())?.len()>20*1024*1024 {
            return Err("image must be a file within the selected workspace, at most 20 MiB".into());
        }
        let extension=source.extension().and_then(|s|s.to_str()).unwrap_or("").to_lowercase();
        if !matches!(extension.as_str(),"png"|"jpg"|"jpeg"|"webp") { return Err("supported images: png, jpg, jpeg, webp".into()); }
        let bytes=fs::read(&source).map_err(|e|e.to_string())?;
        let hash=format!("{:x}",Sha256::digest(&bytes));
        let destination=format!("/sdcard/Pictures/XiaoBan/{}.{}",&hash[..24],extension);
        self.shell(serial,&["mkdir","-p","/sdcard/Pictures/XiaoBan"],ctx)?;
        self.command(&["-s",serial,"push",source.to_str().ok_or("non UTF-8 source path")?,&destination],&[],ctx,32*1024)?;
        let scanned=self.shell(serial,&["am","broadcast","-a","android.intent.action.MEDIA_SCANNER_SCAN_FILE","-d",&format!("file://{destination}")],ctx)?;
        Ok(json!({"device_path":destination,"sha256":hash,"scan_result":scanned.trim(),"note":"Inspect the album and selected thumbnails before publishing; media indexing can be delayed."}))
    }
}

pub fn shell_quote(value: &str) -> String { format!("'{}'",value.replace('\'',"'\\''")) }
fn coordinate(args:&Value,key:&str)->Result<u64> {
    args[key].as_u64().filter(|v|*v<=16000).ok_or_else(||format!("{key} must be an integer screenshot coordinate in 0..16000"))
}
pub fn foreground_from(text:&str)->Option<String> {
    text.lines().filter(|l|l.contains("topResumedActivity")||l.contains("mResumedActivity")).find_map(|line| {
        line.split_whitespace().filter_map(|part|part.split_once('/').map(|(pkg,_)|pkg))
            .find(|pkg|valid_package(pkg)).map(str::to_owned)
    })
}
pub fn validate_endpoint(endpoint:&str)->Result<()> {
    if endpoint.len()>200 || endpoint.starts_with('-') || !endpoint.bytes().all(|b|b.is_ascii_alphanumeric()||b".-:[]".contains(&b)) {
        return Err("endpoint must be host:port".into());
    }
    let (host,port)=endpoint.rsplit_once(':').ok_or("endpoint must include a port")?;
    if host.is_empty() || port.parse::<u16>().ok().is_none_or(|p|p==0) { return Err("invalid host or port".into()); }
    Ok(())
}
