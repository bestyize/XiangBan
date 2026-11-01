//! Immutable content-addressed sources. A revision is one atomic small manifest;
//! unchanged images and templates are shared with its history and recovery draft.
use super::*;
use sha2::{Digest,Sha256};

pub(crate) fn digest(bytes:&[u8])->String {format!("{:x}",Sha256::digest(bytes))}
pub(crate) fn regular(path:&Path)->Result<(),String>{
    if fs::symlink_metadata(path).is_ok_and(|m|m.file_type().is_symlink()){return Err("设计存储拒绝符号链接".into())}Ok(())
}
fn object_path(dir:&Path,hash:&str)->Result<PathBuf,String>{
    if hash.len()!=64||!hash.bytes().all(|b|b.is_ascii_hexdigit()){return Err("设计资源摘要无效".into())}
    let objects=dir.join("objects");regular(&objects)?;let path=objects.join(hash);regular(&path)?;Ok(path)
}
pub(crate) fn put(dir:&Path,bytes:&[u8])->Result<String,String>{
    let hash=digest(bytes);let path=object_path(dir,&hash)?;
    if path.exists(){
        // Read-time digest verification detects damaged objects. Never silently overwrite history.
        return Ok(hash)
    }
    fs::create_dir_all(path.parent().unwrap()).map_err(|e|e.to_string())?;
    let temp=path.with_extension(format!("{}.tmp",nonce()?));
    fs::write(&temp,bytes).map_err(|e|e.to_string())?;
    fs::rename(&temp,&path).map_err(|e|e.to_string())?;Ok(hash)
}
fn get(dir:&Path,hash:&str)->Result<Vec<u8>,String>{
    let bytes=fs::read(object_path(dir,hash)?).map_err(|e|format!("设计资源无法读取：{e}"))?;
    if digest(&bytes)!=hash{return Err("设计资源内容损坏（摘要不匹配）".into())}Ok(bytes)
}
pub(crate) fn write(dir:&Path,path:&Path,project:&Project)->Result<(),String>{
    regular(path)?;if let Some(parent)=path.parent(){regular(parent)?;}
    let metadata=Project{files:BTreeMap::new(),status:project.status,id:project.id.clone(),title:project.title.clone(),session_id:project.session_id.clone(),session_ids:project.session_ids.clone(),revision:project.revision,updated_ms:project.updated_ms,canvas:project.canvas.clone(),request_id:project.request_id.clone()};
    let mut value=serde_json::to_value(metadata).map_err(|e|e.to_string())?;
    let mut objects=serde_json::Map::new();
    for (name,source) in &project.files{
        let binary=assets::is_asset(name);
        let bytes=if binary{assets::decode(source)?}else{source.as_bytes().to_vec()};
        objects.insert(name.clone(),json!({"sha256":put(dir,&bytes)?,"binary":binary}));
    }
    value["storageVersion"]=json!(2);value["fileObjects"]=Value::Object(objects);
    fs::write(path,serde_json::to_vec(&value).map_err(|e|e.to_string())?).map_err(|e|e.to_string())
}
fn document(path:&Path)->Result<Value,String>{
    regular(path)?;
    serde_json::from_slice(&fs::read(path).map_err(|e|format!("设计稿无法读取：{e}"))?).map_err(|e|format!("设计稿内容损坏：{e}"))
}
fn source(dir:&Path,name:&str,entry:&Value)->Result<String,String>{
    if !file_name(name){return Err("设计资源路径无效".into())}
    let bytes=get(dir,entry["sha256"].as_str().ok_or("设计资源缺少摘要")?)?;
    if entry["binary"].as_bool().ok_or("设计资源缺少编码")? != assets::is_asset(name){return Err("设计资源编码不匹配".into())}
    if assets::is_asset(name){assets::encode(&bytes)}else{String::from_utf8(bytes).map_err(|_|"设计源码不是 UTF-8".into())}
}
pub(crate) fn read(dir:&Path,path:&Path)->Result<Project,String>{
    read_with_assets(dir,path,None)
}
fn read_with_assets(dir:&Path,path:&Path,asset_root:Option<&str>)->Result<Project,String>{
    let value=document(path)?;
    let mut project:Project=serde_json::from_value(value.clone()).map_err(|e|format!("设计稿内容损坏：{e}"))?;
    if let Some(version)=value.get("storageVersion"){
        if version.as_u64()!=Some(2){return Err("设计存储版本不支持".into())}
        let objects=value["fileObjects"].as_object().ok_or("设计稿资源索引损坏")?;
        if objects.len()>405{return Err("设计稿文件过多".into())}
        if !project.files.is_empty(){return Err("设计稿混用了两种存储格式".into())}
        for (name,entry) in objects{
            if !file_name(name){return Err("设计资源路径无效".into())}
            let value=if let Some(root)=asset_root.filter(|_|assets::is_asset(name)){
                if entry["binary"]!=true{return Err("设计资源类型无效".into())}
                asset_url(root,name)
            }else{source(dir,name,entry)?};
            project.files.insert(name.clone(),value);
        }
    }else if let Some(root)=asset_root{
        for (name,value) in &mut project.files{if assets::is_asset(name){*value=asset_url(root,name);}}
    }
    Ok(project)
}
fn asset_url(root:&str,name:&str)->String{
    let encoded=name.bytes().map(|b|if b.is_ascii_alphanumeric()||b"-_./~".contains(&b){(b as char).to_string()}else{format!("%{b:02X}")}).collect::<String>();
    format!("{root}{encoded}")
}
/// Read just one asset; do not load every board or every other image.
pub(crate) fn asset(dir:&Path,revision:u64,name:&str)->Result<Vec<u8>,String>{
    if !file_name(name)||!assets::is_asset(name){return Err("预览资源路径无效".into())}
    let value=document(&dir.join("project.json"))?;
    if value["revision"].as_u64()!=Some(revision){return Err("预览版本已更新，请刷新".into())}
    if value.get("storageVersion").is_some(){
        if value["storageVersion"].as_u64()!=Some(2){return Err("设计存储版本不支持".into())}
        let entry=value["fileObjects"].get(name).ok_or("预览资源不存在")?;
        if entry["binary"]!=true{return Err("预览资源类型无效".into())}
        let bytes=get(dir,entry["sha256"].as_str().ok_or("设计资源缺少摘要")?)?;
        assets::mime(&bytes).ok_or("预览资源类型无效")?;Ok(bytes)
    }else{assets::decode(value["files"][name].as_str().ok_or("预览资源不存在")?)}
}

impl Store{
    /// Preview loads text sources but leaves binary objects on disk until requested.
    pub fn preview_project(&self,id:&str,revision:u64,asset_root:&str)->Result<Project,String>{
        let dir=self.dir(id)?;
        let p=read_with_assets(&dir,&dir.join("project.json"),Some(asset_root))?;
        if p.id!=id||p.revision!=revision{return Err("预览版本已更新，请刷新".into())}
        Ok(p)
    }
    pub fn read_asset(&self,id:&str,revision:u64,name:&str)->Result<Vec<u8>,String>{asset(&self.dir(id)?,revision,name)}
    /// Keep a byte-exact original outside the editable sources, before publishing a new project.
    pub fn import(&self,title:String,session:&str,canvas:Value,files:Files,original:&[u8],filename:&str)->Result<Project,String>{
        let mut p=Project{status:ProjectStatus::Draft,id:format!("design-{}",nonce()?),title,session_id:session.into(),session_ids:vec![],revision:0,updated_ms:0,canvas,files,request_id:String::new()};
        let issues=diagnostics(&p);
        if issues.is_empty(){p.status=ProjectStatus::Ready;}
        if !p.canvas["xiaoban"].is_object(){p.canvas["xiaoban"]=json!({});}
        if !p.canvas["xiaoban"]["import"].is_object(){p.canvas["xiaoban"]["import"]=json!({});}
        p.canvas["xiaoban"]["import"]["issues"]=json!(issues);
        p.canvas["xiaoban"]["import"]["originalFile"]=json!(filename);
        validate(&p)?;
        let dir=self.dir(&p.id)?;fs::create_dir_all(&dir).map_err(|e|e.to_string())?;
        let result=(||{
            let hash=put(&dir,original)?;
            fs::write(dir.join("original.json"),serde_json::to_vec(&json!({"filename":filename,"sha256":hash})).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
            self.save(p,0)
        })();
        if result.is_err(){let _=fs::remove_dir_all(&dir);}result
    }
    pub fn original(&self,id:&str)->Result<(String,Vec<u8>),String>{
        let dir=self.dir(id)?;
        if document(&dir.join("project.json"))?["id"]!=id{return Err("设计稿编号不匹配".into())}
        let data=document(&dir.join("original.json"))?;
        let name=data["filename"].as_str().ok_or("导入原件记录损坏")?.to_string();
        Ok((name,get(&dir,data["sha256"].as_str().ok_or("导入原件记录损坏")?)?))
    }
}
type Files=BTreeMap<String,String>;

#[cfg(test)]
mod tests{
 use super::*;
 fn fixture()->(Store,PathBuf){let root=std::env::temp_dir().join(format!("ocd-file-store-{}",nonce().unwrap()));(Store::new(&root),root)}
 fn input()->(Value,Files){
  (json!({"v":3,"boards":{"Main.dc.html":{"w":100,"h":100,"x":0,"y":0}},"order":["Main.dc.html"]}),
  BTreeMap::from([("Main.dc.html".into(),"<x-dc>hello</x-dc>".into()),("assets/large.png".into(),assets::encode(&[b"\x89PNG\r\n\x1a\n".as_slice(),&vec![1;3*1024*1024]].concat()).unwrap())]))
 }
 #[test]
 fn sources_are_shared_across_history_drafts_and_original_is_immutable(){
  let (s,root)=fixture();let (c,f)=input();let original=b"unchanged imported bytes";
  let p=s.import("test".into(),"test",c,f.clone(),original,"official.html").unwrap();let dir=s.dir(&p.id).unwrap();
  assert!(fs::metadata(dir.join("project.json")).unwrap().len()<4096);
  assert_eq!(s.load(&p.id).unwrap().files,f);
  let count=fs::read_dir(dir.join("objects")).unwrap().count();
  let mut edited=p.clone();edited.title="renamed".into();let newer=s.save(edited,p.revision).unwrap();
  assert_eq!(fs::read_dir(dir.join("objects")).unwrap().count(),count);
  assert_eq!(s.history(&p.id).unwrap()[0],p);
  s.save_draft(&newer).unwrap();assert_eq!(s.draft(&p.id).unwrap().unwrap(),newer);
  assert_eq!(s.original(&p.id).unwrap().1,original);
  assert_eq!(s.read_asset(&p.id,newer.revision,"assets/large.png").unwrap().len(),3*1024*1024+8);
  assert!(s.read_asset(&p.id,p.revision,"assets/large.png").is_err());
  assert!(s.read_asset(&p.id,newer.revision,"../original.json").is_err());
  let preview=s.preview_project(&p.id,newer.revision,"design-preview://localhost/test/_asset/").unwrap();
  assert_eq!(preview.files["assets/large.png"],"design-preview://localhost/test/_asset/assets/large.png");
  assert_eq!(preview.files["Main.dc.html"],f["Main.dc.html"]);
  let copy=s.copy(&p.id,"other",None).unwrap();assert_eq!(s.original(&copy.id).unwrap().1,original);
  // A damaged editable source must not prevent recovery of the preserved original.
  fs::write(dir.join("objects").join(digest(f["Main.dc.html"].as_bytes())),b"damaged").unwrap();
  assert!(s.load(&p.id).is_err());assert_eq!(s.original(&p.id).unwrap().1,original);
  fs::remove_dir_all(root).unwrap();
 }
 #[test]
 fn migrates_legacy_inline_sources_on_next_save_and_rejects_damaged_objects(){
  let (s,root)=fixture();let (c,f)=input();let p=s.create("legacy".into(),"test",c,f).unwrap();let dir=s.dir(&p.id).unwrap();
  fs::write(dir.join("project.json"),serde_json::to_vec(&p).unwrap()).unwrap();assert_eq!(s.load(&p.id).unwrap(),p);
  let saved=s.save(p.clone(),p.revision).unwrap();assert_eq!(s.load(&p.id).unwrap(),saved);
  let manifest:Value=serde_json::from_slice(&fs::read(dir.join("project.json")).unwrap()).unwrap();
  let hash=manifest["fileObjects"]["Main.dc.html"]["sha256"].as_str().unwrap();
  fs::write(dir.join("objects").join(hash),b"changed").unwrap();assert!(s.load(&p.id).unwrap_err().contains("摘要"));
  fs::remove_dir_all(root).unwrap();
 }
}
