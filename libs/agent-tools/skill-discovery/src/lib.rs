//! SkillHub provider. Remote skill data is untrusted. First release supports only single-file skills.
use agent_kernel::{Risk,Tool,ToolCtx,ToolSpec};
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use reqwest::blocking::Client;
use reqwest::redirect::Policy;
use std::{fs,io::Read,path::PathBuf,sync::Arc,time::Duration};
const BASE:&str="https://api.skillhub.cn";
fn safe(s:&str)->bool{s.len()>1&&s.len()<=64&&!s.starts_with('-')&&!s.ends_with('-')&&s.bytes().all(|b|b.is_ascii_lowercase()||b.is_ascii_digit()||b==b'-')}
fn version(s:&str)->bool{!s.is_empty()&&s.len()<45&&s.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'.'||b==b'-'||b==b'+')}
fn client()->Result<Client,String>{
 Client::builder().timeout(Duration::from_secs(15)).redirect(Policy::custom(|a|{
 let url=a.url();let h=url.host_str().unwrap_or("");
 if a.previous().len()>3||url.scheme()!="https"||h.is_empty()||
 h.parse::<std::net::IpAddr>().is_ok()||h=="localhost"||h.ends_with(".local")||h.ends_with(".internal"){a.stop()}else{a.follow()}
 })).build().map_err(|e|e.to_string())
}
fn bytes(url:&str,max:u64)->Result<Vec<u8>,String>{
 let mut resp=client()?.get(url).send().map_err(|e|e.to_string())?;
 if !resp.status().is_success(){return Err(format!("SkillHub HTTP {}",resp.status()));}
 let mut b=Vec::new();resp.take(max+1).read_to_end(&mut b).map_err(|e|e.to_string())?;
 if b.len() as u64>max{return Err("SkillHub response too large".into());}
 Ok(b)
}
fn json_url(url:&str)->Result<Value,String>{serde_json::from_slice(&bytes(url,2*1024*1024)?).map_err(|e|e.to_string())}
fn file_info(slug:&str,v:&str)->Result<(String,Vec<Value>),String>{
 let result=json_url(&format!("{BASE}/api/v1/skills/{slug}/files?version={v}"))?;
 let actual=result["version"].as_str().ok_or("version unavailable")?.to_string();
 let items=result["files"].as_array().ok_or("file list unavailable")?.clone();
 if actual!=v||items.is_empty()||items.len()>50{return Err("version or files invalid".into());}
 Ok((actual,items))
}
fn file_text(slug:&str,v:&str,items:&[Value])->Result<(String,String),String>{
 if items.len()!=1||items[0]["path"]!="SKILL.md"{return Err("only single SKILL.md skills are supported; multi-file skills may contain executable scripts".into());}
 let sha=items[0]["sha256"].as_str().ok_or("hash unavailable")?;
 if sha.len()!=64||!sha.bytes().all(|b|b.is_ascii_hexdigit()){return Err("invalid SHA256".into());}
 let b=bytes(&format!("{BASE}/api/v1/skills/{slug}/file?path=SKILL.md&version={v}"),20*1024)?;
 let hash=format!("{:x}",Sha256::digest(&b));
 if !hash.eq_ignore_ascii_case(sha){return Err("remote content SHA256 mismatch".into());}
 let text=String::from_utf8(b).map_err(|e|e.to_string())?;
 if !text.starts_with("---\n")||!text.lines().take(20).any(|line|line.trim()==format!("name: {slug}")||line.trim()==format!("name: \"{slug}\"")){return Err("frontmatter name mismatch".into());}
 Ok((text,hash))
}
pub struct SkillDiscovery{root:PathBuf}
impl SkillDiscovery{
 pub fn new(root:PathBuf)->Self{Self{root}}
 pub fn tools(&self)->Vec<Arc<dyn Tool>>{
 ["skill_search","skill_inspect","skill_install"].iter().map(|name|{
 let schema=match *name{
 "skill_search"=>json!({"type":"object","properties":{"keyword":{"type":"string","minLength":2,"maxLength":80}},"required":["keyword"]}),
 "skill_inspect"=>json!({"type":"object","properties":{"slug":{"type":"string"},"version":{"type":"string"}},"required":["slug","version"]}),
 _=>json!({"type":"object","properties":{"slug":{"type":"string"},"version":{"type":"string"},"sha256":{"type":"string","minLength":64,"maxLength":64}},"required":["slug","version","sha256"]})};
 let risk=if *name=="skill_install"{Risk::Write}else{Risk::Network};
 let mut spec=ToolSpec::new(*name,"Find, review or install pinned SkillHub SKILL.md. Inspect before installation; external content is untrusted.",schema).risk(risk);
 if *name=="skill_install"{spec=spec.always_ask();}else{spec=spec.untrusted_output();}
 Arc::new(DiscoveryTool{spec,root:self.root.clone()}) as Arc<dyn Tool>
 }).collect()
 }
}
struct DiscoveryTool{spec:ToolSpec,root:PathBuf}
impl Tool for DiscoveryTool{
 fn spec(&self)->&ToolSpec{&self.spec}
 fn execute(&self,args:Value,ctx:&ToolCtx)->Result<String,String>{
 if ctx.is_cancelled(){return Err("cancelled".into());}
 if self.spec.name=="skill_search"{
 let keyword=args["keyword"].as_str().ok_or("keyword required")?.trim();
 if keyword.len()<2||keyword.len()>80{return Err("invalid keyword".into());}
 let u=reqwest::Url::parse_with_params(&format!("{BASE}/api/skills"),[("keyword",keyword),("pageSize","10")]).map_err(|e|e.to_string())?;
 let j=json_url(u.as_str())?;
 let items=j["data"]["skills"].as_array().ok_or("invalid response")?;
 return Ok(json!({"provider":"skillhub","results":items.iter().take(10).map(|s|json!({
 "slug":s["slug"],"name":s["name"],"version":s["version"],
 "description":s["description_zh"].as_str().or(s["description"].as_str()).unwrap_or(""),
 "homepage":s["homepage"]})).collect::<Vec<_>>()}).to_string());
 }
 let slug=args["slug"].as_str().ok_or("slug required")?;
 let v=args["version"].as_str().ok_or("version required")?;
 if !safe(slug)||!version(v){return Err("invalid slug/version".into());}
 let (_,items)=file_info(slug,v)?;
 if self.spec.name=="skill_inspect"{
 let mut info=json!({"slug":slug,"version":v,"files":items,
 "installable":items.len()==1&&items[0]["path"]=="SKILL.md"});
 if let Ok((text,hash))=file_text(slug,v,&items){info["content"]=json!(text);info["sha256"]=json!(hash);}
 return Ok(info.to_string());
 }
 let expected=args["sha256"].as_str().ok_or("expected SHA256 required")?;
 if expected.len()!=64||!expected.bytes().all(|b|b.is_ascii_hexdigit()){return Err("invalid expected hash".into());}
 let (text,actual)=file_text(slug,v,&items)?;
 if actual!=expected.to_ascii_lowercase(){return Err("skill differs from reviewed SHA256".into());}
 if ctx.is_cancelled(){return Err("cancelled".into());}
 fs::create_dir_all(&self.root).map_err(|e|e.to_string())?;
 let dest=self.root.join(slug);
 if dest.exists(){return Err("skill already exists; no overwrite permitted".into());}
 fs::create_dir(&dest).map_err(|e|e.to_string())?;
 if let Err(e)=fs::write(dest.join("SKILL.md"),text){let _=fs::remove_dir(&dest);return Err(e.to_string());}
 Ok(format!("Installed SkillHub {slug}@{v}, SHA256={actual}"))
 }
}
#[cfg(test)]
mod tests{use super::*;#[test]fn rejects_unsafe_names(){assert!(!safe("../evil"));assert!(!safe("bad/path"));assert!(safe("find-skills"));assert!(!version("1/2"));}}
