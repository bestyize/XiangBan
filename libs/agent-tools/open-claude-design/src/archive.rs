//! Bounded source archives. No extraction to arbitrary filesystem paths.
use super::*;
use std::io::{Cursor, Read, Write};
use regex::Regex;

pub fn preview(source: &str) -> String {
    let support = Regex::new(r#"(?is)<script\b[^>]*\bsrc\s*=\s*["'][^"']*support\.js["'][^>]*>\s*</script\s*>"#).unwrap();
    let source=support.replace_all(source, "");
    let runtime=format!("<script>{RUNTIME}</script>");
    let head=Regex::new("(?i)<head(?:\\s[^>]*)?>").unwrap();
    if let Some(m)=head.find(&source) { format!("{}{}{}", &source[..m.end()],runtime,&source[m.end()..]) }
    else { format!("{runtime}{source}") }
}
pub fn to_zip(project: &Project) -> Result<Vec<u8>,String> {
    validate(project)?;
    let mut files=project.files.clone();
    files.insert("canvas.json".into(),serde_json::to_string_pretty(&project.canvas).map_err(|e|e.to_string())?);
    // Runtime is supplied only where absent; imported originals remain unchanged.
    for name in project.files.keys().filter(|n|n.ends_with(".dc.html")) {
        let dir=Path::new(name).parent().unwrap_or(Path::new(""));
        files.entry(dir.join("support.js").to_string_lossy().replace('\\',"/")).or_insert_with(||RUNTIME.into());
    }
    let mut writer=zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name,source) in files {
        writer.start_file(name,zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated)).map_err(|e|e.to_string())?;
        writer.write_all(source.as_bytes()).map_err(|e|e.to_string())?;
    }
    Ok(writer.finish().map_err(|e|e.to_string())?.into_inner())
}
pub fn from_zip(bytes: &[u8]) -> Result<(Value,BTreeMap<String,String>),String> {
    if bytes.len()>MAX_PROJECT { return Err("ZIP 超过 16MiB".into()); }
    let mut archive=zip::ZipArchive::new(Cursor::new(bytes)).map_err(|e|e.to_string())?;
    if archive.len()>500 { return Err("ZIP 文件数量过多".into()); }
    let mut all=BTreeMap::new();let mut total=0;
    for i in 0..archive.len() {
        let mut f=archive.by_index(i).map_err(|e|e.to_string())?;
        if f.is_dir(){continue;}
        if f.unix_mode().is_some_and(|m|m&0o170000==0o120000){return Err("拒绝符号链接".into());}
        let name=f.name().map_err(|e|e.to_string())?.into_owned();
        if !file_name(&name){return Err(format!("暂不支持的归档资源：{name}。当前支持文本源包；二进制素材请先内嵌为 data URL。"));}
        if f.size()>MAX_FILE as u64 {return Err(format!("{name} 超过 2MiB"));}
        let mut buf=Vec::new();f.by_ref().take(MAX_FILE as u64+1).read_to_end(&mut buf).map_err(|e|e.to_string())?;
        total+=buf.len();if buf.len()>MAX_FILE||total>MAX_PROJECT{return Err("ZIP 解压内容过大".into());}
        let text=String::from_utf8(buf).map_err(|_|format!("{name} 不是 UTF-8 文本"))?;
        if all.insert(name,text).is_some(){return Err("ZIP 中有重复文件路径".into());}
    }
    let indexes:Vec<_>=all.keys().filter(|n|n.as_str()=="canvas.json"||n.ends_with("/canvas.json")).cloned().collect();
    if indexes.len()!=1{return Err("源包必须含唯一的 canvas.json".into());}
    let index=&indexes[0];let prefix=index.strip_suffix("canvas.json").unwrap();
    let canvas=serde_json::from_str(all.get(index).unwrap()).map_err(|e|format!("索引无效：{e}"))?;
    if all.keys().any(|name|!name.starts_with(prefix)){return Err("源文件必须位于 canvas.json 所在目录内".into());}
    let files:BTreeMap<String,String>=all.into_iter().filter_map(|(n,t)|n.strip_prefix(prefix).filter(|n|*n!="canvas.json").map(|n|(n.to_string(),t))).collect();
    for (name,_) in boards(&canvas)? {if !files.contains_key(&name){return Err(format!("缺少画板源文件：{name}"));}}
    Ok((canvas,files))
}
pub fn standalone(project:&Project)->Result<String,String>{
    validate(project)?;
    let sources:BTreeMap<_,_>=project.files.iter().filter(|(name,_)|name.ends_with(".dc.html")).map(|(name,text)|(name.clone(),preview(text))).collect();
    let data=json!({"boards":boards(&project.canvas)?,"sources":sources}).to_string().replace('<',"\\u003c");
    let title=project.title.replace('&',"&amp;").replace('<',"&lt;").replace('>',"&gt;");
    Ok(format!(r#"<!doctype html><html><meta charset="utf-8"><title>{title}</title><style>body{{margin:0;font:13px system-ui;background:#f5f5f7}}header{{position:sticky;top:0;background:white;padding:12px 20px;z-index:2}}main{{display:flex;gap:24px;padding:24px;align-items:start}}section{{flex:none}}iframe{{border:1px solid #ddd;background:white}}h2{{font-size:12px}}</style><header>{title} · 独立预览</header><main></main><script>const p={data};const frames=[];p.boards.forEach(([file,b])=>{{const s=document.createElement('section'),h=document.createElement('h2'),f=document.createElement('iframe');h.textContent=b.title||file;f.sandbox='allow-scripts';f.width=b.w;f.height=b.h;f.srcdoc='<meta http-equiv="Content-Security-Policy" content="default-src &apos;none&apos;; script-src &apos;unsafe-inline&apos; &apos;unsafe-eval&apos;; style-src &apos;unsafe-inline&apos; https://fonts.googleapis.com; img-src data: blob:; font-src data: https://fonts.gstatic.com; connect-src &apos;none&apos;; base-uri &apos;none&apos;; form-action &apos;none&apos;;">'+p.sources[file];s.append(h,f);document.querySelector('main').append(s);frames.push({{file,f}});}});addEventListener('message',e=>{{const frame=frames.find(x=>x.f.contentWindow===e.source);if(!frame||e.data?.channel!=='open-claude-design')return;if(e.data.type==='ready')frame.f.contentWindow.postMessage({{channel:'open-claude-design',type:'mode',editing:false}},'*');if(e.data.type==='error'){{const error=document.createElement('p');error.textContent='预览错误：'+String(e.data.message);error.style.color='#b54454';frame.f.parentElement.append(error);}}if(e.data.type==='navigate'){{const parts=frame.file.split('/');parts.pop();for(const part of e.data.file.split('/')){{if(part==='..')parts.pop();else if(part!=='.')parts.push(part);}}const dest=frames.find(x=>x.file===parts.join('/'));dest?.f.scrollIntoView({{behavior:'smooth'}});}}}});</script></html>"#))
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn source_roundtrip_keeps_indexes_and_logic(){let canvas=json!({"v":3,"boards":{"a/Main.dc.html":{"w":400,"h":800,"custom":123}},"order":["a/Main.dc.html"],"notes":{}});let p=Project{id:format!("design-{}",nonce().unwrap()),title:"demo".into(),session_id:"s".into(),session_ids:Vec::new(),revision:1,updated_ms:0,canvas:canvas.clone(),files:[("a/Main.dc.html".into(),"<x-dc>{{value}}</x-dc>".into())].into(),request_id:String::new()};let (c,f)=from_zip(&to_zip(&p).unwrap()).unwrap();assert_eq!(c,canvas);assert_eq!(f["a/Main.dc.html"],p.files["a/Main.dc.html"]);assert!(f.contains_key("a/support.js"));assert!(standalone(&p).unwrap().contains("sandbox"));}
 #[test]fn support_is_replaced_only_for_preview(){let s="<html><head><script src='./support.js'></script></head><body><x-dc>text</x-dc></body></html>";let p=preview(s);assert!(!p.contains("src='./support.js'"));assert!(p.contains("DCLogic"));}
 fn zip_files(files:Vec<(&str,Vec<u8>)>)->Vec<u8>{let mut w=zip::ZipWriter::new(Cursor::new(Vec::new()));for(n,b)in files{w.start_file(n,zip::write::SimpleFileOptions::default()).unwrap();w.write_all(&b).unwrap();}w.finish().unwrap().into_inner()}
 #[test]fn rejects_unsafe_or_incomplete_archives_before_import(){
   assert!(from_zip(&zip_files(vec![("../escape.dc.html",b"x".to_vec())])).is_err());
   assert!(from_zip(&zip_files(vec![("asset.png",vec![0,255])])).unwrap_err().contains("二进制"));
   let index=br#"{"v":3,"boards":{"Main.dc.html":{"w":100,"h":200}},"order":["Main.dc.html"]}"#.to_vec();
   assert!(from_zip(&zip_files(vec![("canvas.json",index.clone())])).unwrap_err().contains("缺少画板"));
   assert!(from_zip(&zip_files(vec![("canvas.json",index.clone()),("other/canvas.json",index.clone())])).unwrap_err().contains("唯一"));
   assert!(from_zip(&zip_files(vec![("canvas.json",index.clone()),("Main.dc.html",vec![b'x';MAX_FILE+1])])).unwrap_err().contains("2MiB"));
 }
 #[test]fn maximum_source_count_roundtrips_with_generated_runtime(){
   let mut files=BTreeMap::from([("Main.dc.html".into(),"<x-dc>ok</x-dc>".into())]);for n in 0..299{files.insert(format!("asset{n}.css"),"body{}".into());}
   let mut p=Project{id:format!("design-{}",nonce().unwrap()),title:"full".into(),session_id:"s".into(),session_ids:Vec::new(),revision:1,updated_ms:0,canvas:json!({"v":3,"boards":{"Main.dc.html":{"w":100,"h":200}},"order":["Main.dc.html"]}),files,request_id:String::new()};
   let(c,f)=from_zip(&to_zip(&p).unwrap()).unwrap();p.canvas=c;p.files=f;assert_eq!(p.files.len(),301);validate(&p).unwrap();
 }

}
