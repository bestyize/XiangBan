//! Same installed executable, no Node/Python/npx dependency and no secrets in client settings.
use super::{autostart,config,failure,PORT};
use serde_json::Value;
use std::{io::{self,BufRead,Write},path::PathBuf,time::Duration};
pub fn main()->Option<i32>{
    let args:Vec<_>=std::env::args().collect();if !args.iter().any(|a|a=="--design-mcp-stdio"){return None}
    let arg=|name:&str|args.iter().position(|a|a==name).and_then(|i|args.get(i+1)).cloned();
    let result=(||->Result<(),String>{
        let path=PathBuf::from(arg("--config").ok_or("缺少 --config")?);let client=arg("--client").ok_or("缺少 --client")?;
        if !config::CLIENTS.contains(&client.as_str()){return Err("未知客户端".into())}
        let http=reqwest::blocking::Client::builder().no_proxy().timeout(Duration::from_secs(120)).build().map_err(|e|e.to_string())?;
        let mut out=io::stdout().lock();
        for line in io::stdin().lock().lines(){
            let line=line.map_err(|e|e.to_string())?;if line.trim().is_empty(){continue}
            let request:Value=match serde_json::from_str(&line){Ok(v)=>v,Err(_)=>{writeln!(out,"{}",failure(Value::Null,-32700,"Parse error")).map_err(|e|e.to_string())?;out.flush().map_err(|e|e.to_string())?;continue}};
            let id=request.get("id").cloned();
            let response=(||->Result<Option<Value>,String>{
                let token=autostart::ensure_ready(&path,&client)?;
                let response=http.post(format!("http://127.0.0.1:{PORT}/mcp")).bearer_auth(token).header("Content-Type","application/json").header("Accept","application/json, text/event-stream").body(line).send().map_err(|_|"MCP 请求传输失败，执行结果未知；请先读取项目核对，不要直接重试写入")?;
                if response.status()==reqwest::StatusCode::ACCEPTED{return Ok(None)}
                if !response.status().is_success(){return Err(format!("小伴设计 MCP 返回 {}，请检查连接授权",response.status()))}
                serde_json::from_str(&response.text().map_err(|e|e.to_string())?).map(Some).map_err(|e|e.to_string())
            })();
            let reply=match response{Ok(v)=>v,Err(e)=>id.clone().map(|id|failure(id,-32000,&e))};
            if let Some(reply)=reply{writeln!(out,"{reply}").map_err(|e|e.to_string())?;out.flush().map_err(|e|e.to_string())?;}
        }Ok(())
    })();
    Some(match result{Ok(())=>0,Err(e)=>{eprintln!("{e}");1}})
}
