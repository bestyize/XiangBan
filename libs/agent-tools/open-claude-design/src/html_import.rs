//! Decode official standalone HTML data islands without executing the publisher's JS.
//! Exported titles, dimensions, templates and embedded resources survive; editor-only
//! canvas positions, notes and original source names are not present in this format.
use super::*;
use base64::{engine::general_purpose::STANDARD, Engine};
use flate2::read::GzDecoder;
use regex::Regex;
use serde::Deserialize;
use std::io::Read;

type Files = BTreeMap<String, String>;
#[derive(Deserialize)]
struct Entry { mime: String, data: String, #[serde(default)] compressed: bool }
#[derive(Deserialize)]
struct External { id: String, uuid: String }
struct Bundle { manifest: BTreeMap<String, Entry>, template: String, pages: Vec<String>, external: Vec<External> }

fn scripts() -> Regex { Regex::new(r#"(?is)<script\b((?:[^>"']|"[^"]*"|'[^']*')*)>(.*?)</script\s*>"#).unwrap() }
fn attrs(tag: &str) -> BTreeMap<String, String> {
    let re=Regex::new(r#"([\w:-]+)\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s>]+))"#).unwrap();
    re.captures_iter(tag).map(|c|(c[1].to_ascii_lowercase(),unescape(c.get(2).or(c.get(3)).or(c.get(4)).unwrap().as_str()))).collect()
}
fn unescape(s:&str)->String {
    let re=Regex::new(r"&(?:#x([0-9a-fA-F]+)|#([0-9]+)|(amp|lt|gt|quot|apos));").unwrap();
    re.replace_all(s,|c:&regex::Captures| {
        if let Some(n)=c.get(1) {u32::from_str_radix(n.as_str(),16).ok().and_then(char::from_u32).map(|v|v.to_string())}
        else if let Some(n)=c.get(2) {n.as_str().parse().ok().and_then(char::from_u32).map(|v|v.to_string())}
        else {Some(match &c[3] {"amp"=>"&","lt"=>"<","gt"=>">","quot"=>"\"","apos"=>"'",_=>""}.into())}
        .unwrap_or_else(||c[0].into())
    }).into_owned()
}
fn bundle(source:&str)->Result<Option<Bundle>,String> {
    let mut islands=BTreeMap::new();
    for c in scripts().captures_iter(source) {
        if let Some(kind)=attrs(&c[1]).get("type").filter(|t|t.starts_with("__bundler/")) {
            if islands.insert(kind.clone(),c[2].to_string()).is_some(){return Err(format!("HTML 中有重复的 {kind}"));}
        }
    }
    if islands.is_empty(){return Ok(None)}
    fn parse<T:serde::de::DeserializeOwned>(islands:&Files,key:&str,default:Option<&str>)->Result<T,String>{
        let text=islands.get(key).map(String::as_str).or(default).ok_or_else(||format!("HTML 缺少 {key}"))?;
        serde_json::from_str(text).map_err(|e|format!("{key} 数据损坏：{e}"))
    }
    let b=Bundle{manifest:parse(&islands,"__bundler/manifest",None)?,template:parse(&islands,"__bundler/template",None)?,pages:parse(&islands,"__bundler/page_order",Some("[]"))?,external:parse(&islands,"__bundler/ext_resources",Some("[]"))?};
    if b.manifest.len()>500||b.pages.len()>100 {return Err("HTML 包含过多资源或画板".into())}
    let id=Regex::new(r"^[a-zA-Z0-9_-]{1,100}$").unwrap();
    if b.manifest.keys().any(|k|!id.is_match(k)) {return Err("HTML 资源编号无效".into())}
    Ok(Some(b))
}
fn decode(entry:&Entry)->Result<Vec<u8>,String>{
    let bytes=STANDARD.decode(&entry.data).map_err(|_|"HTML 资源 base64 无效")?;
    if !entry.compressed{return Ok(bytes)}
    let mut out=Vec::new();GzDecoder::new(bytes.as_slice()).read_to_end(&mut out).map_err(|e|format!("HTML gzip 资源损坏：{e}"))?;Ok(out)
}
fn text(bytes:Vec<u8>)->Result<String,String>{String::from_utf8(bytes).map_err(|_|"HTML 文本资源不是 UTF-8".into())}
fn insert(files:&mut Files,name:String,value:String)->Result<String,String>{
    if !file_name(&name){return Err(format!("HTML 资源路径无效：{name}"))}
    if let Some(old)=files.get(&name){if old!=&value{return Err(format!("HTML 资源路径冲突：{name}"))}}
    else {if files.len()>=300{return Err("HTML 解包资源超过 300 个".into())}files.insert(name.clone(),value);}
    Ok(name)
}
fn asset(files:&mut Files,entry:&Entry,bytes:Vec<u8>)->Result<String,String>{
    let mime=entry.mime.split(';').next().unwrap_or("").trim().to_ascii_lowercase();
    let (ext,value)=match mime.as_str(){
        "text/javascript"|"application/javascript"|"application/x-javascript"=>("js",text(bytes)?),
        "text/css"=>("css",text(bytes)?),
        "text/html"=>("html",text(bytes)?),
        "application/json"=>("json",text(bytes)?),
        "text/plain"=>("txt",text(bytes)?),
        _=>("bin",assets::encode(&bytes)?),
    };
    if let Some((name,_))=files.iter().find(|(n,v)|n.starts_with("_import/")&&*v==&value){return Ok(name.clone())}
    let name=if ext=="bin"{format!("_import/_blob/asset-{}",files.len())}
        else if ext=="js"&&value.starts_with("// GENERATED from dc-runtime/src/"){format!("_import/runtime-{}/support.js",files.len())}
        else {format!("_import/asset-{}.{}",files.len(),ext)};
    insert(files,name,value)
}
fn replace_refs(source:&str,mapping:&BTreeMap<String,String>)->String {
    // A single pass avoids rewriting newly inserted paths or data.
    if mapping.is_empty(){return source.into()}
    let mut keys:Vec<_>=mapping.keys().collect();keys.sort_by_key(|s|std::cmp::Reverse(s.len()));
    let re=Regex::new(&keys.into_iter().map(|s|regex::escape(s)).collect::<Vec<_>>().join("|")).unwrap();
    re.replace_all(source,|c:&regex::Captures|mapping[&c[0]].clone()).into_owned()
}
fn unpack(b:Bundle,base:&str,files:&mut Files,depth:usize)->Result<String,String>{
    if depth>16 {return Err("HTML 嵌套超过 16 层".into())}
    if !b.pages.is_empty(){return Err("暂不支持画板内嵌套多画板 HTML；请保留原件并分别导入".into())}
    let mut mapping=BTreeMap::new();
    let mut pending=Vec::new();
    for (id,entry) in &b.manifest {
        let bytes=decode(entry).map_err(|e|format!("资源 {id}：{e}"))?;
        let name=asset(files,entry,bytes)?;
        mapping.insert(id.clone(),format!("/{name}"));
        pending.push((name,entry.mime.clone()));
    }
    for ext in &b.external {
        let target=mapping.get(&ext.uuid).ok_or_else(||format!("HTML 缺少资源 {}",ext.uuid))?.clone();
        if !ext.id.contains(':')&&!ext.id.starts_with("//") {
            let path=archive::resolve_rel(base,&ext.id).filter(|p|file_name(p)).ok_or_else(||format!("HTML 资源路径越界：{}",ext.id))?;
            let value=files[target.trim_start_matches('/')].clone();
            insert(files,path.clone(),value)?;
            // Keep relative names for dc-import/module resolution; UUID references use the same alias.
            mapping.insert(ext.uuid.clone(),format!("/{path}"));
        } else {mapping.insert(ext.id.clone(),target);}
    }
    let mut template=replace_refs(&b.template,&mapping);
    // Decode nested component bundles and normalize their local embedded references.
    let aliases:Vec<_>=b.external.iter().filter(|e|!e.id.contains(':')&&!e.id.starts_with("//")).filter_map(|e|archive::resolve_rel(base,&e.id)).collect();
    for name in aliases {
        let value=files[&name].clone();
        let value=if let Some(nested)=bundle(&value)? {unpack(nested,&name,files,depth+1)?}else{replace_refs(&value,&mapping)};
        files.insert(name,value);
    }
    for (name,mime) in pending {
        if mime.starts_with("text/css") {
            let value=replace_refs(&files[&name],&mapping);files.insert(name,value);
        }
    }
    // Source scripts serialized by the browser contain sc-camel-* attributes;
    // the local DC runtime already understands them. The publisher runtime is
    // retained as a source asset but replaced by our runtime at preview time.
    if template.starts_with('\u{feff}') {template.remove(0);}
    Ok(template)
}
fn dimension(attributes:&BTreeMap<String,String>,key:&str,default:u32)->Result<u32,String>{
    match attributes.get(key){None=>Ok(default),Some(s)=>s.trim_end_matches("px").parse::<u32>().ok().filter(|v|(1..=16000).contains(v)).ok_or_else(||format!("画板 {key} 尺寸无效"))}
}
fn unused_board(files:&Files,preferred:&str)->String {
    if !files.contains_key(preferred){return preferred.into()}
    let parent=preferred.rsplit_once('/').map(|(p,_)|format!("{p}/")).unwrap_or_default();
    for n in 1..=500 {let file=format!("{parent}__imported_board_{n}.dc.html");if !files.contains_key(&file){return file}}
    unreachable!("resource count is bounded at 300")
}
fn preview_dimensions(template:&str)->(u32,u32){
    for c in scripts().captures_iter(template){
        let attributes=attrs(&c[1]);
        if let Some(config)=attributes.get("data-props").and_then(|s|serde_json::from_str::<Value>(s).ok()){
            let size=|key:&str,default|config["$preview"][key].as_u64().filter(|v|(1..=16000).contains(v)).map(|v|v as u32).unwrap_or(default);
            return (size("width",1280),size("height",800));
        }
    }
    (1280,800)
}
pub fn from_html(source:&str)->Result<(Value,Files),String>{
    let Some(root)=bundle(source)? else {
        return Ok((json!({"v":3,"boards":{"Main.dc.html":{"title":"导入画板","x":0,"y":0,"w":1280,"h":800}},"order":["Main.dc.html"],"notes":{}}),BTreeMap::from([("Main.dc.html".into(),source.into())])));
    };
    let mut files=Files::new();let mut boards=serde_json::Map::new();let mut order=Vec::new();
    let frames=Regex::new(r#"(?is)<iframe\b((?:[^>"']|"[^"]*"|'[^']*')*)>"#).unwrap();
    let page_set:std::collections::BTreeSet<_>=root.pages.iter().collect();
    if page_set.len()!=root.pages.len(){return Err("HTML 画板编号重复".into())}
    let mut y=0u32;let mut seen=std::collections::BTreeSet::new();
    for c in frames.captures_iter(&root.template) {
        let a=attrs(&c[1]);let src=a.get("src").map(String::as_str).unwrap_or("");
        let Some(id)=src.strip_prefix("about:blank#").map(|s|s.split('#').next().unwrap_or("")) else{continue};
        if !seen.insert(id.to_string()){return Err("HTML 中有重复的画板 frame".into())}
        if !page_set.contains(&id.to_string()){return Err(format!("HTML 画板未登记：{id}"))}
        let entry=root.manifest.get(id).ok_or_else(||format!("HTML 缺少画板：{id}"))?;
        let raw=text(decode(entry)?)?;let mut file=format!("pages/{:03}/Main.dc.html",order.len()+1);
        let template=if let Some(b)=bundle(&raw)? {unpack(b,&file,&mut files,0)?}else{raw};
        let (w,h)=(dimension(&a,"width",1280)?,dimension(&a,"height",800)?);
        if files.contains_key(&file){file=unused_board(&files,&file);}
        insert(&mut files,file.clone(),template)?;
        boards.insert(file.clone(),json!({"title":a.get("title").cloned().unwrap_or_else(||format!("画板 {}",order.len()+1)),"w":w,"h":h,"x":0,"y":y}));
        y=y.saturating_add(h+80);order.push(file);
    }
    if root.pages.is_empty(){
        let template=unpack(root,"Main.dc.html",&mut files,0)?;
        let file=unused_board(&files,"Main.dc.html");
        let (w,h)=preview_dimensions(&template);
        insert(&mut files,file.clone(),template)?;
        boards.insert(file.clone(),json!({"title":"导入画板","w":w,"h":h,"x":0,"y":0}));order.push(file);
    }else if order.len()!=root.pages.len(){return Err("HTML 画板清单与页面内容不一致".into())}
    Ok((json!({"v":3,"boards":boards,"order":order,"notes":{},"xiaoban":{"import":{"format":"claude-bundled-html","layout":"export-order","notice":"已按导出顺序恢复画板；官方 HTML 未包含原画布坐标、分组、批注和完整源文件名。"}}}),files))
}

#[cfg(test)]
mod tests{
 use super::*;
 use flate2::{write::GzEncoder,Compression};
 use std::io::Write;
 fn pack(template:&str,manifest:Value,pages:Value,ext:Value)->String{
  format!("<!doctype html><script>throw new Error('must never execute')</script><script type='__bundler/manifest'>{}</script><script type='__bundler/page_order'>{}</script><script type='__bundler/ext_resources'>{}</script><script type='__bundler/template'>{}</script>",manifest,pages,ext,serde_json::to_string(template).unwrap().replace("</","<\\/"))
 }
 fn entry(mime:&str,data:&[u8])->Value{
  let mut e=GzEncoder::new(Vec::new(),Compression::default());e.write_all(data).unwrap();
  json!({"mime":mime,"data":STANDARD.encode(e.finish().unwrap()),"compressed":true})
 }
 #[test]
 fn decodes_nested_pages_components_media_and_dimensions_without_running_scripts(){
  let png=b"\x89PNG\r\n\x1a\nfixture";
  let board=pack("<html><head><script src='runtime'></script></head><body><x-dc><img src='image'><dc-import name='Card'></dc-import></x-dc></body></html>",
    json!({"image":entry("image/png",png),"runtime":entry("text/javascript",b"// GENERATED from dc-runtime/src/test"),"component":entry("text/html",b"<x-dc><button>Card</button></x-dc>")}),json!([]),json!([{"id":"./Card.dc.html","uuid":"component"}]));
  let root=pack("<iframe title='A &amp; B' src='about:blank#one' width='390' height='844'></iframe><iframe title='Wide' src='about:blank#two' width='1200' height='700'></iframe>",
   json!({"one":entry("text/html",board.as_bytes()),"two":entry("text/html",board.as_bytes())}),json!(["one","two"]),json!([]));
  let (canvas,files)=from_html(&root).unwrap();
  assert_eq!(canvas["boards"]["pages/001/Main.dc.html"]["title"],"A & B");assert_eq!(canvas["boards"]["pages/001/Main.dc.html"]["w"],390);
  assert_eq!(canvas["boards"]["pages/002/Main.dc.html"]["y"],924);
  assert_eq!(files.keys().filter(|p|assets::is_asset(p)).count(),1);
  assert_eq!(files["pages/001/Card.dc.html"],"<x-dc><button>Card</button></x-dc>");
  assert!(files["pages/001/Main.dc.html"].contains("/_import/_blob/"));
  assert!(files["pages/001/Main.dc.html"].contains("/support.js"));
 }
 #[test]
 fn keeps_main_component_separate_and_reads_single_board_dimensions(){
  let original="<x-dc><dc-import name='Main'></dc-import></x-dc><script type='text/x-dc' data-props='{\"$preview\":{\"width\":390,\"height\":844}}'></script>";
  let html=pack(original,json!({"component":entry("text/html",b"<x-dc>Component</x-dc>")}),json!([]),json!([{"id":"./Main.dc.html","uuid":"component"}]));
  let (canvas,files)=from_html(&html).unwrap();
  let board=canvas["order"][0].as_str().unwrap();assert_ne!(board,"Main.dc.html");
  assert_eq!(files["Main.dc.html"],"<x-dc>Component</x-dc>");assert_eq!(files[board],original);
  assert_eq!(canvas["boards"][board]["w"],390);assert_eq!(canvas["boards"][board]["h"],844);
 }
 #[test]
 fn rejects_corruption_missing_pages_and_path_escape(){
  assert!(from_html("<script type='__bundler/manifest'>{}</script>").is_err());
  let missing=pack("<iframe src='about:blank#lost'></iframe>",json!({}),json!(["lost"]),json!([]));assert!(from_html(&missing).unwrap_err().contains("缺少画板"));
  let corrupt=pack("<x-dc/>",json!({"broken":{"mime":"image/png","data":"AA==","compressed":true}}),json!([]),json!([]));assert!(from_html(&corrupt).unwrap_err().contains("gzip"));
  let escaped=pack("<x-dc/>",json!({"component":entry("text/html",b"<x-dc/>")}),json!([]),json!([{"id":"../../outside.html","uuid":"component"}]));
  assert!(from_html(&escaped).unwrap_err().contains("越界"));
 }
 #[test]
 fn accepts_plain_html_above_previous_file_and_project_size_limits(){
  let source=format!("<!doctype html><body>{}</body>","x".repeat(17*1024*1024));
  let (canvas,files)=from_html(&source).unwrap();
  let p=Project{status:ProjectStatus::Ready,id:"design-000000000000000000000000".into(),title:"large".into(),session_id:"test".into(),session_ids:vec![],revision:1,updated_ms:0,canvas,files,request_id:String::new()};
  validate(&p).unwrap();assert_eq!(p.files["Main.dc.html"],source);
 }
}
