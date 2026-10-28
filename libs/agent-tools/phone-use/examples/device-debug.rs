//! Invoke the actual agent tools for device debugging without starting the desktop or an LLM.
//! device-debug <pack-root> <workspace> <request.json> <media-output-directory>
//! Request: {"tool":"phone_observe","arguments":{"serial":"...","kind":"screenshot"}}
use agent_kernel::{Part,ToolCtx};
use agent_phone_use_tools::{adb::Adb,Store};
use base64::{engine::general_purpose::STANDARD,Engine};
use serde_json::{json,Value};
use std::path::PathBuf;

fn main()->Result<(),Box<dyn std::error::Error>> {
    let args=std::env::args_os().skip(1).map(PathBuf::from).collect::<Vec<_>>();
    if args.len()!=4 {return Err("usage: device-debug <pack-root> <workspace> <request.json> <media-output-directory>".into());}
    let request:Value=serde_json::from_slice(&std::fs::read(&args[2])?)?;
    let name=request["tool"].as_str().ok_or("missing tool name")?;
    let adb=Adb::new(std::env::var_os("XIAOBAN_ADB").map(PathBuf::from).unwrap_or_else(||PathBuf::from("adb")));
    let tools=agent_phone_use_tools::tools(Store::new(&args[0]),adb.clone(),args[1].clone());
    let tool=tools.iter().find(|tool|tool.spec().name==name).ok_or("unknown phone tool")?;
    let context=ToolCtx::detached("device-debug");
    let result=tool.execute_result(request["arguments"].clone(),&context);
    let mut media=Vec::new();
    let mut artifacts=Vec::new();
    if !result.is_error && name=="phone_observe" && request["arguments"]["kind"]=="ui" {
        let mut page:Value=serde_json::from_str(&result.output)?;
        let mut complete=page["content"].as_str().ok_or("missing UI content")?.to_string();
        while page["has_more"]==true {
            let mut arguments=request["arguments"].clone();
            arguments["offset"]=page["next_offset"].clone();
            arguments["snapshot_id"]=page["snapshot_id"].clone();
            let next=tool.execute_result(arguments,&context);
            if next.is_error {return Err(next.output.into());}
            page=serde_json::from_str(&next.output)?;
            complete.push_str(page["content"].as_str().ok_or("missing UI continuation")?);
            if complete.len()>1024*1024 {return Err("UI snapshot exceeds budget".into());}
        }
        std::fs::create_dir_all(&args[3])?;
        let target=args[3].join("ui.xml");
        std::fs::write(&target,complete)?;
        artifacts.push(target);
    }
    for (index,part) in result.parts.iter().enumerate() {
        if let Part::Image {url}=part {
            let (header,data)=url.split_once(",").ok_or("invalid media URL")?;
            let extension=if header.contains("image/png") {"png"} else if header.contains("image/webp") {"webp"} else {"jpg"};
            std::fs::create_dir_all(&args[3])?;
            let target=args[3].join(format!("{name}-{index}.{extension}"));
            std::fs::write(&target,STANDARD.decode(data)?)?;
            media.push(target);
        }
    }
    println!("{}",serde_json::to_string_pretty(&json!({"is_error":result.is_error,"output":result.output,"structured_content":result.structured_content,"media":media,"artifacts":artifacts}))?);
    adb.close()?;
    if result.is_error {std::process::exit(1);}
    Ok(())
}
