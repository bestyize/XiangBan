//! Render the checked-in Claude reference with our runtime for browser comparison.
//! Usage: cargo run --example render_reference -- /absolute/output/directory
use open_claude_design::{archive,Project,ProjectStatus};
use std::{collections::BTreeMap,fs,path::Path};
fn main() -> Result<(),Box<dyn std::error::Error>> {
 let dest=std::env::args().nth(1).ok_or("缺少输出目录")?;let dest=Path::new(&dest);
 let root=Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../client/xiaoban-desktop/docs/design-specs/ui-upgrade");
 let canvas=serde_json::from_slice(&fs::read(root.join("canvas.json"))?)?;let mut files=BTreeMap::new();
 for path in fs::read_dir(&root)?.map(|e|e.unwrap().path()).chain(fs::read_dir(root.join("_blob"))?.map(|e|e.unwrap().path())) {
  if !path.is_file(){continue;}let name=path.strip_prefix(&root)?.to_string_lossy().replace('\\',"/");
  if name=="canvas.json"{continue;}files.insert(name.clone(),archive::source_from_bytes(&name,fs::read(path)?)?);
 }
 let p=Project{status:ProjectStatus::Ready,id:"design-000000000000000000000000".into(),title:"Claude Design 兼容性验收".into(),session_id:"preview".into(),session_ids:vec![],revision:1,updated_ms:0,canvas,files,request_id:String::new()};
 open_claude_design::validate(&p)?;fs::create_dir_all(dest)?;
 fs::write(dest.join("project.json"),serde_json::to_vec(&p)?)?;
 for (name,_) in open_claude_design::boards(&p.canvas)? {fs::write(dest.join(&name),archive::preview_in_project(&p,&name)?)?;}
 fs::write(dest.join("index.html"),archive::standalone(&p)?)?;
 println!("Rendered {} reference boards",open_claude_design::boards(&p.canvas)?.len());Ok(())
}
