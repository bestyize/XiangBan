//! Import a downloaded HTML into an isolated directory and render for inspection.
//! cargo run --example import_html -- /path/export.html /tmp/import-inspection
use open_claude_design::{html_import,archive,Store,boards,diagnostics};
use std::{fs,path::Path};
fn main()->Result<(),Box<dyn std::error::Error>>{
 let args:Vec<_>=std::env::args().collect();let input=Path::new(args.get(1).ok_or("缺少 HTML 文件")?);let out=Path::new(args.get(2).ok_or("缺少输出目录")?);
 let bytes=fs::read(input)?;let (canvas,files)=html_import::from_html(std::str::from_utf8(&bytes)?)?;
 let store=Store::new(out.join("store"));let p=store.import(input.file_stem().unwrap().to_string_lossy().into(),"import-test",canvas,files,&bytes,&input.file_name().unwrap().to_string_lossy())?;
 fs::write(out.join("project.json"),serde_json::to_vec(&p)?)?;
 if !args.iter().any(|arg|arg=="--no-render"){
 let first=args.iter().any(|arg|arg=="--first-board");
 for (name,_) in boards(&p.canvas)?.into_iter().take(if first{1}else{usize::MAX}) {let path=out.join(&name);fs::create_dir_all(path.parent().unwrap())?;fs::write(path,archive::preview_in_project(&p,&name)?)?;}
 if !first {fs::write(out.join("index.html"),archive::standalone(&p)?)?;}
 }
 let issues=diagnostics(&p);println!("Imported {} boards, {} files, {} unresolved references; original byte match: {}",boards(&p.canvas)?.len(),p.files.len(),issues.len(),store.original(&p.id)?.1==bytes);
 for issue in issues.iter().take(8){println!("{issue}");}
 Ok(())
}
