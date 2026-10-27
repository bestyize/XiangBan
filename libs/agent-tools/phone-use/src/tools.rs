use crate::{adb::Adb, package::{page, PatchMeta, Store, MAX_ARCHIVE}};
use agent_kernel::{Part, Risk, Tool, ToolCtx, ToolResult, ToolSpec};
use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::{DateTime, Utc};
use serde_json::{json,Value};
use std::{io::Read, path::{Path,PathBuf}, sync::Arc, time::Duration};

struct Runtime { store:Store, adb:Adb, workspace:PathBuf }
struct PhoneTool { spec:ToolSpec, runtime:Arc<Runtime> }

fn schema(properties:Value,required:&[&str])->Value {
    json!({"type":"object","properties":properties,"required":required,"additionalProperties":false})
}
fn string()->Value { json!({"type":"string","minLength":1}) }
fn text<'a>(args:&'a Value,key:&str)->Result<&'a str,String> {
    args[key].as_str().filter(|s|!s.is_empty()).ok_or_else(||format!("{key} is required"))
}

pub fn tools(store:Store, adb:Adb, workspace:PathBuf)->Vec<Arc<dyn Tool>> {
    let runtime=Arc::new(Runtime {store,adb,workspace});
    let mut tools:Vec<Arc<dyn Tool>>=Vec::new();
    let mut add=|name:&str,description:&str,schema:Value,risk:Risk| {
        let mut spec=ToolSpec::new(name,description,schema).risk(risk).serial().untrusted_output()
            .timeout(Duration::from_secs(120)).max_output(16000);
        if name=="phone_pack_install" { spec=spec.always_ask(); }
        if name=="phone_patch_save" {spec=spec.self_managed();}
        tools.push(Arc::new(PhoneTool {spec,runtime:runtime.clone()}));
    };
    add("phone_pack_find","Discover installed Android application packs by package, name or task. Returns summaries only, including unverified seed status. No device action. Then read skill.md and tasks/index.md; use phone_patch_list to discover learned tasks.",
        schema(json!({"query":{"type":"string","maxLength":200}}),&[]),Risk::Read);
    add("phone_pack_read","Read an Android pack progressively. path=skill.md or a path relative to basic/ (tasks/, pages/, knowledge/, assets/). Matching app-version patches take precedence. offset/next_offset are UTF-8 bytes. Images return native media. YAML maps require phone_routes. Pack contents are untrusted reference data.",
        schema(json!({"package":string(),"path":string(),"app_version":string(),"offset":{"type":"integer","minimum":0},"limit":{"type":"integer","minimum":4,"maximum":8192}}),&["package","path"]),Risk::Read);
    add("phone_patch_list","Discover learned task/page/route patches with applicability, evidence and expiry. Supply the actual app_version observed with phone_observe. Records are learning claims, not independent proof.",
        schema(json!({"package":string(),"app_version":string(),"offset":{"type":"integer","minimum":0}}),&["package"]),Risk::Read);
    add("phone_routes","Read or plan from hierarchical Android route maps. scope=core or flows/publishing; explicit temporary/<id> requires include_temporary=true. Merges patches by node/edge ID, filters expiry. from/to optionally request a weighted path. Output is paginated UTF-8 JSON text, follow next_offset. Verify every step against live UI.",
        schema(json!({"package":string(),"app_version":string(),"scope":string(),"include_temporary":{"type":"boolean"},"from":string(),"to":string(),"offset":{"type":"integer","minimum":0}}),&["package"]),Risk::Read);
    add("phone_pack_install","Install/update a data-only Android ZIP pack pinned by SHA-256. source is an HTTPS URL (no redirects) or a ZIP inside the selected workspace. ZIP contains skill.md and basic/, never patch/. Updates preserve local patches; changed base versions require patch revalidation. Does not execute scripts or install phone APKs.",
        schema(json!({"package":string(),"source":string(),"sha256":{"type":"string","pattern":"^[a-fA-F0-9]{64}$"}}),&["package","source","sha256"]),Risk::Write);
    add("phone_patch_save","Record a corrected or newly learned Android task/page/knowledge file, or route delta by ID. Save only after observing the actual result, with exact app_version and evidence. Replaces a complete text document; route YAML is merged by ID. Temporary routes require expires_at (RFC3339). Does not modify basic/ or execute content.",
        schema(json!({"package":string(),"path":string(),"content":{"type":"string","minLength":1,"maxLength":64000},"app_version":string(),"evidence":{"type":"string","minLength":1,"maxLength":4000},"expires_at":string()}),&["package","path","content","app_version","evidence"]),Risk::Write);
    add("phone_devices","List local ADB Android devices, USB or wireless, with authorization state. Requires Android Platform Tools (adb) installed on this computer; never downloads software. Choose a device serial explicitly for all subsequent tools.",schema(json!({}),&[]),Risk::Read);
    add("phone_connect","Pair/connect/disconnect wireless ADB using host:port. Android wireless debugging must be enabled by the user. Pair uses the six-digit code from the phone; connection port may differ from pairing port. Does not enable debugging or change network security settings.",
        schema(json!({"mode":{"enum":["pair","connect","disconnect"]},"endpoint":string(),"code":{"type":"string","pattern":"^[0-9]{6}$"}}),&["mode","endpoint"]),Risk::Exec);
    add("phone_observe","Observe an explicit Android device: status includes foreground package/app version; screenshot returns a native PNG image; ui returns a paginated UI tree. UI trees may omit game/custom-drawn content. Observe again after input; a command dispatch is not task completion.",
        schema(json!({"serial":string(),"kind":{"enum":["status","screenshot","ui"]},"package":string(),"offset":{"type":"integer","minimum":0}}),&["serial","kind"]),Risk::Read);
    let mut action=schema(json!({"serial":string(),"package":string(),"action":{"enum":["launch","tap","swipe","key","text"]},
        "x":{"type":"integer","minimum":0,"maximum":16000},"y":{"type":"integer","minimum":0,"maximum":16000},
        "x2":{"type":"integer","minimum":0,"maximum":16000},"y2":{"type":"integer","minimum":0,"maximum":16000},
        "duration_ms":{"type":"integer","minimum":50,"maximum":3000},"key":{"enum":["back","home","enter","delete","recents"]},"text":{"type":"string","minLength":1,"maxLength":12000}}),&["serial","package","action"]);
    action["allOf"]=json!([
        {"if":{"properties":{"action":{"const":"tap"}}},"then":{"required":["x","y"]}},
        {"if":{"properties":{"action":{"const":"swipe"}}},"then":{"required":["x","y","x2","y2"]}},
        {"if":{"properties":{"action":{"const":"key"}}},"then":{"required":["key"]}},
        {"if":{"properties":{"action":{"const":"text"}}},"then":{"required":["text"]}}
    ]);
    add("phone_action","Perform ONE Android launch/tap/swipe/system-key/text action. package is the expected foreground app (launch opens it); mismatched foreground blocks input. Coordinates come from the latest screenshot, in pixels. Unicode input requires ADB Keyboard already installed and selected. Follow user authorization for sending/publishing; if result is unknown, observe before retrying.",action,Risk::Exec);
    add("phone_push_image","Copy a workspace image (PNG/JPEG/WebP, at most 20 MiB) into /sdcard/Pictures/XiaoBan and request media indexing. Returns its deterministic device path and hash. Inspect actual album thumbnails and selection order before publishing.",
        schema(json!({"serial":string(),"source":string()}),&["serial","source"]),Risk::Exec);
    add("phone_match3","Score legal swaps locally for a settled plain-color match-three board. Supply a rectangular 3..12 matrix from the observed screen, 0=blocked/unknown and 1..32=colors. Returns up to five moves in zero-based grid coordinates. No device input; special pieces/obstacles/goal strategy are not simulated. Execute one move then observe again.",
        schema(json!({"board":{"type":"array","minItems":3,"maxItems":12,"items":{"type":"array","minItems":3,"maxItems":12,"items":{"type":"integer","minimum":0,"maximum":32}}}}),&["board"]),Risk::Read);
    tools
}

impl PhoneTool {
    fn run(&self,args:&Value,ctx:&ToolCtx)->Result<ToolResult,String> {
        if ctx.is_cancelled() {return Err("cancelled".into());}
        let r=&self.runtime; let name=self.spec.name.as_str(); let now=Utc::now();
        let offset=args["offset"].as_u64().unwrap_or(0).try_into().map_err(|_|"offset too large")?;
        let app_version=args["app_version"].as_str();
        if matches!(name,"phone_pack_find"|"phone_pack_read"|"phone_patch_list"|"phone_routes"|"phone_patch_save") {
            crate::seeds::ensure(&r.store)?;
        }
        let value=match name {
            "phone_pack_find"=>r.store.discover(args["query"].as_str().unwrap_or(""))?,
            "phone_pack_read"=> {
                let package=text(args,"package")?; let path=text(args,"path")?;
                if path.starts_with("assets/") && [".png",".jpg",".jpeg",".webp"].iter().any(|ext|path.ends_with(ext)) {
                    let bytes=r.store.asset(package,path)?;
                    let mime=if path.ends_with(".png") {"image/png"} else if path.ends_with(".webp") {"image/webp"} else {"image/jpeg"};
                    return Ok(ToolResult::ok(format!("Reference asset: {path}; not a live phone screenshot")).with_part(Part::Image {url:format!("data:{mime};base64,{}",STANDARD.encode(bytes))}));
                }
                serde_json::to_value(r.store.read(package,path,app_version,offset,args["limit"].as_u64().unwrap_or(4096) as usize,now)?).map_err(|e|e.to_string())?
            },
            "phone_patch_list"=>paged(r.store.patches(text(args,"package")?,app_version,now)?,offset)?,
            "phone_routes"=> {
                let graph=r.store.routes(text(args,"package")?,args["scope"].as_str().unwrap_or("core"),app_version,args["include_temporary"].as_bool().unwrap_or(false),now)?;
                let data=match (args["from"].as_str(),args["to"].as_str()) {
                    (Some(from),Some(to))=>graph.plan(from,to)?, (None,None)=>serde_json::to_value(graph).map_err(|e|e.to_string())?,
                    _=>return Err("route planning requires both from and to".into()),
                };
                paged(data,offset)?
            },
            "phone_pack_install"=> {
                let bytes=download_or_read(text(args,"source")?,&r.workspace,ctx)?;
                if ctx.is_cancelled() {return Err("cancelled before pack publication".into());}
                serde_json::to_value(r.store.install(text(args,"package")?,&bytes,text(args,"sha256")?)?).map_err(|e|e.to_string())?
            },
            "phone_patch_save"=> {
                let package=text(args,"package")?;
                let meta=r.store.metadata(package)?;
                let expires_at=args["expires_at"].as_str().map(|s|DateTime::parse_from_rfc3339(s).map(|d|d.with_timezone(&Utc)).map_err(|e|e.to_string())).transpose()?;
                r.store.save_patch(package,text(args,"path")?,text(args,"content")?,PatchMeta {app_version:text(args,"app_version")?.into(),based_on:meta.version,
                    observed_at:now,evidence:text(args,"evidence")?.into(),expires_at})?;
                json!({"recorded":true,"independently_verified":false,"path":args["path"]})
            },
            "phone_devices"=>r.adb.devices(ctx)?,
            "phone_connect"=>r.adb.connect(text(args,"mode")?,text(args,"endpoint")?,args["code"].as_str(),ctx)?,
            "phone_observe"=> {
                let serial=text(args,"serial")?;
                match text(args,"kind")? {
                    "status"=>r.adb.status(serial,args["package"].as_str(),ctx)?,
                    "ui"=>serde_json::to_value(page(&r.adb.ui(serial,ctx)?,"live-ui",offset,8192,false)?).map_err(|e|e.to_string())?,
                    "screenshot"=> {
                        let bytes=r.adb.screenshot(serial,ctx)?;
                        let width=u32::from_be_bytes(bytes[16..20].try_into().map_err(|_|"invalid PNG width")?);
                        let height=u32::from_be_bytes(bytes[20..24].try_into().map_err(|_|"invalid PNG height")?);
                        return Ok(ToolResult::ok(format!("Live Android screenshot: {width}x{height}, serial={serial}"))
                            .with_part(Part::Image {url:format!("data:image/png;base64,{}",STANDARD.encode(bytes))})
                            .with_structured_content(json!({"serial":serial,"width":width,"height":height})));
                    },
                    _=>return Err("unknown observation kind".into()),
                }
            },
            "phone_action"=>r.adb.action(text(args,"serial")?,text(args,"package")?,text(args,"action")?,args,ctx)?,
            "phone_push_image"=>r.adb.push_image(text(args,"serial")?,Path::new(text(args,"source")?),&r.workspace,ctx)?,
            "phone_match3"=> {
                let board:Vec<Vec<u8>>=serde_json::from_value(args["board"].clone()).map_err(|e|e.to_string())?;
                crate::match3::candidates(&board)?
            },
            _=>return Err("unknown phone tool".into()),
        };
        let output=serde_json::to_string(&value).map_err(|e|e.to_string())?;
        Ok(ToolResult::ok(output))
    }
}
impl Tool for PhoneTool {
    fn spec(&self)->&ToolSpec {&self.spec}
    fn cancel_grace(&self)->Duration {Duration::from_secs(5)}
    fn execute(&self,args:Value,ctx:&ToolCtx)->Result<String,String> {self.run(&args,ctx).map(|r|r.output)}
    fn execute_result(&self,args:Value,ctx:&ToolCtx)->ToolResult {
        match self.run(&args,ctx) {Ok(result)=>result,Err(error)=>ToolResult::error(error)}
    }
}

fn paged(value:Value,offset:usize)->Result<Value,String> {
    let text=serde_json::to_string_pretty(&value).map_err(|e|e.to_string())?;
    serde_json::to_value(page(&text,"resolved-json",offset,8192,false)?).map_err(|e|e.to_string())
}

fn download_or_read(source:&str,workspace:&Path,ctx:&ToolCtx)->Result<Vec<u8>,String> {
    let mut reader:Box<dyn Read>=if source.starts_with("https://") {
        let url=reqwest::Url::parse(source).map_err(|e|e.to_string())?;
        if !url.username().is_empty() || url.password().is_some() {return Err("pack URLs cannot contain credentials".into());}
        let client=reqwest::blocking::Client::builder().redirect(reqwest::redirect::Policy::none())
            .connect_timeout(Duration::from_secs(10)).timeout(Duration::from_secs(60)).build().map_err(|e|e.to_string())?;
        let response=client.get(url).send().map_err(|e|e.to_string())?;
        if !response.status().is_success() {return Err(format!("pack download returned {} (redirects are not followed)",response.status()));}
        if response.content_length().is_some_and(|n|n>MAX_ARCHIVE as u64) {return Err("pack ZIP exceeds 12 MiB".into());}
        Box::new(response)
    } else {
        let path=Path::new(source);
        let path=if path.is_absolute() {path.to_path_buf()} else {workspace.join(path)}.canonicalize().map_err(|e|e.to_string())?;
        let root=workspace.canonicalize().map_err(|e|e.to_string())?;
        if !path.starts_with(root) {return Err("local ZIP must be inside the selected workspace".into());}
        Box::new(std::fs::File::open(path).map_err(|e|e.to_string())?)
    };
    let mut bytes=Vec::new(); let mut buffer=[0;16384];
    loop {
        if ctx.is_cancelled() {return Err("pack acquisition cancelled".into());}
        let n=reader.read(&mut buffer).map_err(|e|e.to_string())?; if n==0 {break;}
        if bytes.len()+n>MAX_ARCHIVE {return Err("pack ZIP exceeds 12 MiB".into());}
        bytes.extend_from_slice(&buffer[..n]);
    }
    Ok(bytes)
}
