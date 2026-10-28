//! Bundled reference packs: deliberately not marked as verified against any APP version.
use crate::{package::Result, Store};
use sha2::{Digest,Sha256};
use std::io::{Cursor,Write};

pub struct Seed { pub package:&'static str, pub files:&'static [(&'static str,&'static str)] }
macro_rules! file { ($package:literal,$path:literal) => { ($path,include_str!(concat!("../packs/",$package,"/",$path))) }; }
pub const SEEDS:&[Seed]=&[
    Seed {package:"com.xingin.xhs",files:&[
        file!("com.xingin.xhs","skill.md"),file!("com.xingin.xhs","basic/package.json"),
        file!("com.xingin.xhs","basic/tasks/index.md"),file!("com.xingin.xhs","basic/tasks/publish-image-note.md"),
        file!("com.xingin.xhs","basic/tasks/open-image-picker.yaml"),file!("com.xingin.xhs","basic/tasks/prepare-image-note.yaml"),
        file!("com.xingin.xhs","basic/pages/home.md"),file!("com.xingin.xhs","basic/pages/album-picker.md"),
        file!("com.xingin.xhs","basic/pages/creation-methods.md"),file!("com.xingin.xhs","basic/pages/image-editor.md"),
        file!("com.xingin.xhs","basic/pages/note-editor.md"),file!("com.xingin.xhs","basic/pages/publish-result.md"),
        file!("com.xingin.xhs","basic/knowledge/text-input.md"),file!("com.xingin.xhs","basic/routes/index.md"),
        file!("com.xingin.xhs","basic/routes/core.yaml"),file!("com.xingin.xhs","basic/routes/flows/publishing.yaml"),
    ]},
    Seed {package:"com.happyelements.AndroidAnimal",files:&[
        file!("com.happyelements.AndroidAnimal","skill.md"),file!("com.happyelements.AndroidAnimal","basic/package.json"),
        file!("com.happyelements.AndroidAnimal","basic/tasks/index.md"),file!("com.happyelements.AndroidAnimal","basic/tasks/play-level.md"),
        file!("com.happyelements.AndroidAnimal","basic/pages/level-map.md"),file!("com.happyelements.AndroidAnimal","basic/pages/board.md"),
        file!("com.happyelements.AndroidAnimal","basic/pages/result.md"),file!("com.happyelements.AndroidAnimal","basic/knowledge/match3.md"),
        file!("com.happyelements.AndroidAnimal","basic/knowledge/level-4181.md"),
        file!("com.happyelements.AndroidAnimal","basic/routes/index.md"),file!("com.happyelements.AndroidAnimal","basic/routes/core.yaml"),
        file!("com.happyelements.AndroidAnimal","basic/routes/flows/playing.yaml"),
    ]},
];

pub fn archive(seed:&Seed)->Result<Vec<u8>> {
    let mut zip=zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name,body) in seed.files {
        zip.start_file(*name,zip::write::SimpleFileOptions::default()).map_err(|e|e.to_string())?;
        zip.write_all(body.as_bytes()).map_err(|e|e.to_string())?;
    }
    Ok(zip.finish().map_err(|e|e.to_string())?.into_inner())
}

pub fn ensure(store:&Store)->Result<()> {
    for seed in SEEDS {
        if !store.contains(seed.package)? {
            let bytes=archive(seed)?;
            store.install(seed.package,&bytes,&format!("{:x}",Sha256::digest(&bytes)))?;
        }
    }
    Ok(())
}
