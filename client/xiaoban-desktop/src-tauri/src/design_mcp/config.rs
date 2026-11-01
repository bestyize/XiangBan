//! Preserve client settings; only this app's named MCP entry is installed/removed.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::BTreeMap, fs, io::Write, path::{Path, PathBuf}};

pub const NAME: &str = "xiaoban-design";
pub const CLIENTS: &[&str] = &["codex", "claude-code", "claude-desktop", "chatgpt-web"];
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Config { pub enabled: bool, pub clients: BTreeMap<String, String> }
pub fn read(path: &Path) -> Result<Config,String> {
    match fs::read(path) { Ok(b)=>serde_json::from_slice(&b).map_err(|e|format!("协作配置损坏：{e}")), Err(e) if e.kind()==std::io::ErrorKind::NotFound=>Ok(Config::default()), Err(e)=>Err(e.to_string()) }
}
pub fn write(path:&Path, bytes:&[u8], backup:bool)->Result<(),String>{
    let parent=path.parent().ok_or("配置路径无效")?;
    fs::create_dir_all(parent).map_err(|e|e.to_string())?;
    if fs::symlink_metadata(path).is_ok_and(|m|m.file_type().is_symlink()) { return Err("配置文件是符号链接，请手动配置，避免覆盖链接目标".into()) }
    if backup && path.exists() {
        let stamp=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos();
        let backup_path=path.with_extension(format!("xiaoban-{stamp}.bak"));
        write(&backup_path,&fs::read(path).map_err(|e|e.to_string())?,false)?;
    }
    let temp=parent.join(format!(".xiaoban-{}.tmp",super::secret()?));
    let result=(||{
        let mut options=fs::OpenOptions::new();options.write(true).create_new(true);
        #[cfg(unix)] { use std::os::unix::fs::OpenOptionsExt; options.mode(0o600); }
        let mut f=options.open(&temp).map_err(|e|e.to_string())?;
        f.write_all(bytes).and_then(|_|f.sync_all()).map_err(|e|e.to_string())?;
        fs::rename(&temp,path).map_err(|e|e.to_string())
    })();
    if result.is_err(){let _=fs::remove_file(&temp);}result
}
pub fn save(path:&Path,cfg:&Config)->Result<(),String>{write(path,&serde_json::to_vec_pretty(cfg).map_err(|e|e.to_string())?,false)}
pub fn client_path(client:&str)->Result<PathBuf,String>{
    let home=dirs::home_dir().ok_or("无法找到用户目录")?;
    match client {
        "codex"=>Ok(std::env::var_os("CODEX_HOME").map(PathBuf::from).unwrap_or_else(||home.join(".codex")).join("config.toml")),
        "claude-code"=>Ok(std::env::var_os("CLAUDE_CONFIG_DIR").map(|d|PathBuf::from(d).join(".claude.json")).unwrap_or_else(||home.join(".claude.json"))),
        "claude-desktop"=>{
            #[cfg(target_os="macos")] let base=home.join("Library/Application Support/Claude");
            #[cfg(not(target_os="macos"))] let base=dirs::config_dir().ok_or("无法找到应用配置目录")?.join("Claude");
            Ok(base.join("claude_desktop_config.json"))
        },
        _=>Err("此客户端需要在网页完成连接".into())
    }
}
/// Claude's gateway mode uses a separate profile. Configure both installed profiles so
/// switching sign-in modes does not silently lose the local MCP connection.
pub fn client_paths(client:&str)->Result<Vec<PathBuf>,String>{
    let primary=client_path(client)?;
    let mut paths=vec![primary.clone()];
    if client=="claude-desktop" {
        if let Some(base)=primary.parent().and_then(Path::parent) {
            let gateway=base.join("Claude-3p");
            if gateway.is_dir(){paths.push(gateway.join("claude_desktop_config.json"));}
        }
    }
    Ok(paths)
}
pub fn entry(exe:&Path,service_config:&Path,client:&str)->Value {
    json!({"command":exe.to_string_lossy(),"args":["--design-mcp-stdio","--config",service_config.to_string_lossy(),"--client",client]})
}
fn owned(v:&Value)->bool { v["args"].as_array().is_some_and(|a|a.first().and_then(Value::as_str)==Some("--design-mcp-stdio")) }
pub fn merge_json(source:&str,entry:Option<Value>)->Result<String,String>{
    let mut doc:Value=serde_json::from_str(source).map_err(|e|format!("客户端 JSON 配置无效：{e}"))?;
    let root=doc.as_object_mut().ok_or("客户端配置必须是对象")?;
    let servers=root.entry("mcpServers").or_insert(json!({})).as_object_mut().ok_or("mcpServers 必须是对象")?;
    if let Some(old)=servers.get(NAME){if !owned(old){return Err("同名 MCP 已存在且不属于小伴，请先手动更名".into())}}
    if let Some(value)=entry {servers.insert(NAME.into(),value);}else{servers.remove(NAME);}
    serde_json::to_string_pretty(&doc).map_err(|e|e.to_string())
}
pub fn merge_toml(source:&str,entry:Option<Value>)->Result<String,String>{
    use toml_edit::{DocumentMut,Item,Table,value,Array};
    let mut doc=source.parse::<DocumentMut>().map_err(|e|format!("Codex 配置无效：{e}"))?;
    if doc.get("mcp_servers").is_none(){doc["mcp_servers"]=Item::Table(Table::new());}
    let servers=doc["mcp_servers"].as_table_like_mut().ok_or("mcp_servers 必须是表")?;
    if let Some(old)=servers.get(NAME){
        if old.get("args").and_then(Item::as_array).and_then(|a|a.get(0)).and_then(toml_edit::Value::as_str)!=Some("--design-mcp-stdio") {return Err("同名 MCP 已存在且不属于小伴，请先手动更名".into())}
    }
    if let Some(v)=entry{
        let mut table=Table::new();table["command"]=value(v["command"].as_str().ok_or("缺少程序路径")?);
        let mut args=Array::new();for arg in v["args"].as_array().ok_or("缺少启动参数")? {args.push(arg.as_str().ok_or("参数无效")?);}
        table["args"]=value(args);servers.insert(NAME,Item::Table(table));
    }else{servers.remove(NAME);}
    Ok(doc.to_string())
}
pub fn update(path:&Path,client:&str,entry:Option<Value>)->Result<(),String>{
    let source=match fs::read_to_string(path){Ok(s)=>s,Err(e) if e.kind()==std::io::ErrorKind::NotFound=>if client=="codex"{String::new()}else{"{}".into()},Err(e)=>return Err(e.to_string())};
    let text=if client=="codex"{merge_toml(&source,entry)?}else{merge_json(&source,entry)?};
    if text!=source{write(path,text.as_bytes(),true)?;}Ok(())
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn preserves_other_clients_and_preferences(){
        let e=entry(Path::new("/a b/小伴"),Path::new("/config"),"codex");
        let source="# user comment\nmodel = 'example'\n[mcp_servers.other]\ncommand='keep'\n";
        let installed=merge_toml(source,Some(e.clone())).unwrap();
        assert!(installed.contains("# user comment"));assert!(installed.contains("command='keep'"));
        let removed=merge_toml(&installed,None).unwrap();assert!(!removed.contains(NAME));assert!(removed.contains("model = 'example'"));
        let json=merge_json(r#"{"theme":"dark","mcpServers":{"other":{"command":"keep"}}}"#,Some(e)).unwrap();
        let doc:Value=serde_json::from_str(&merge_json(&json,None).unwrap()).unwrap();assert_eq!(doc["theme"],"dark");assert_eq!(doc["mcpServers"]["other"]["command"],"keep");
    }
    #[test] fn refuses_unowned_and_malformed_config(){
        assert!(merge_json(r#"{"mcpServers":{"xiaoban-design":{"command":"other"}}}"#,None).is_err());
        assert!(merge_toml("[mcp_servers.xiaoban-design]\ncommand='other'",None).is_err());
        assert!(merge_json("invalid",None).is_err());assert!(merge_toml("invalid [",None).is_err());
    }
}
