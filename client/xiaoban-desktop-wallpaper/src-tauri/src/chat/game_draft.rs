//! Bounded text-only snapshots of game drafts in the current agent workspace.
use crate::game_packs::{MAX_FILES, MAX_FILE_BYTES, MAX_TOTAL_BYTES};
use std::{fs, io::Read, path::Path};

pub fn read(workspace: &Path, source: &str) -> Result<Vec<(String, String)>, String> {
    let workspace = workspace
        .canonicalize()
        .map_err(|e| format!("无法读取工作目录：{e}"))?;
    let source = workspace
        .join(source)
        .canonicalize()
        .map_err(|e| format!("无法读取游戏草稿目录：{e}"))?;
    if source == workspace || !source.starts_with(&workspace) || !source.is_dir() {
        return Err("source_dir 必须是当前工作目录内专门存放游戏文本文件的子目录".into());
    }
    let mut files = Vec::new();
    visit(&source, "", &mut files, &mut 0, &mut 0)?;
    Ok(files)
}

fn visit(
    dir: &Path,
    prefix: &str,
    files: &mut Vec<(String, String)>,
    entries: &mut usize,
    total: &mut usize,
) -> Result<(), String> {
    let mut children = Vec::new();
    for child in fs::read_dir(dir).map_err(|e| format!("无法读取草稿目录：{e}"))? {
        *entries += 1;
        if *entries > MAX_FILES * 4 {
            return Err("游戏草稿目录过大，请只保留游戏需要的文件".into());
        }
        children.push(child.map_err(|e| e.to_string())?);
    }
    children.sort_by_key(|entry| entry.file_name());
    for child in children {
        let name = child
            .file_name()
            .into_string()
            .map_err(|_| "草稿文件名需要使用字母、数字、. _ -")?;
        if name.starts_with('.')
            || name.is_empty()
            || !name
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'.' | b'_' | b'-'))
        {
            return Err(format!("草稿路径「{name}」不合法"));
        }
        let relative = format!("{prefix}{name}");
        let kind = child.file_type().map_err(|e| e.to_string())?;
        if kind.is_symlink() {
            return Err(format!("草稿不能包含符号链接：{relative}"));
        }
        if kind.is_dir() {
            if relative.split('/').count() >= 4 {
                return Err("草稿文件路径最多 4 层".into());
            }
            visit(
                &child.path(),
                &format!("{relative}/"),
                files,
                entries,
                total,
            )?;
        } else if kind.is_file() {
            crate::game_packs::valid_file_path(&relative)?;
            if relative.eq_ignore_ascii_case("manifest.json") {
                return Err("manifest.json 由工具生成，请勿放入草稿目录".into());
            }
            if files.len() >= MAX_FILES {
                return Err(format!("游戏草稿最多 {MAX_FILES} 个文件"));
            }
            let mut bytes = Vec::new();
            fs::File::open(child.path())
                .map_err(|e| e.to_string())?
                .take((MAX_FILE_BYTES + 1) as u64)
                .read_to_end(&mut bytes)
                .map_err(|e| e.to_string())?;
            if bytes.len() > MAX_FILE_BYTES {
                return Err(format!("草稿文件「{relative}」超过 1 MB"));
            }
            *total += bytes.len();
            if *total > MAX_TOTAL_BYTES {
                return Err("草稿文件合计超过 4 MB".into());
            }
            let text = String::from_utf8(bytes)
                .map_err(|_| format!("草稿文件「{relative}」必须是 UTF-8 文本"))?;
            files.push((relative, text));
        } else {
            return Err(format!("草稿只能包含普通文件和目录：{relative}"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Temp(std::path::PathBuf);
    impl Temp {
        fn new(tag: &str) -> Self {
            let path = std::env::temp_dir()
                .join(format!("xiaoban-game-draft-{tag}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(path.join("work/draft/js")).unwrap();
            fs::write(
                path.join("work/draft/index.html"),
                "<html><head></head><body></body></html>",
            )
            .unwrap();
            fs::write(path.join("work/draft/js/game.js"), "let score = 0;").unwrap();
            Self(path)
        }
        fn work(&self) -> std::path::PathBuf {
            self.0.join("work")
        }
    }
    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn draft_snapshots_install_atomically_and_invalid_replacements_keep_old_games() {
        let temp = Temp::new("install");
        let source = read(&temp.work(), "draft").unwrap();
        assert_eq!(source.len(), 2);
        let storage = temp.0.join("storage");
        crate::game_packs::create_pack(
            &storage,
            "draft-game",
            "草稿游戏",
            "测试",
            &source,
            false,
            false,
        )
        .unwrap();
        let installed = crate::game_packs::packs_dir(&storage).join("draft-game/js/game.js");
        assert_eq!(fs::read_to_string(&installed).unwrap(), "let score = 0;");
        fs::write(temp.work().join("draft/js/game.js"), "let score = 10;").unwrap();
        fs::remove_file(temp.work().join("draft/index.html")).unwrap();
        let invalid = read(&temp.work(), "draft").unwrap();
        assert!(crate::game_packs::create_pack(
            &storage,
            "draft-game",
            "新版",
            "测试",
            &invalid,
            false,
            true
        )
        .is_err());
        assert_eq!(fs::read_to_string(installed).unwrap(), "let score = 0;");
    }

    #[test]
    fn rejects_outside_paths_metadata_unsupported_files_and_resource_limits() {
        let temp = Temp::new("boundaries");
        assert!(read(&temp.work(), "..").is_err());
        assert!(read(&temp.work(), ".").is_err());
        for (path, bytes) in [
            ("manifest.json", b"{}".to_vec()),
            ("unknown.exe", b"no".to_vec()),
            ("bad.js", vec![0xff]),
            ("large.js", vec![b'x'; MAX_FILE_BYTES + 1]),
            (".secret", b"hidden".to_vec()),
        ] {
            let file = temp.work().join("draft").join(path);
            fs::write(&file, bytes).unwrap();
            assert!(read(&temp.work(), "draft").is_err(), "{path}");
            fs::remove_file(file).unwrap();
        }
        for i in 0..MAX_FILES {
            fs::write(temp.work().join(format!("draft/f{i}.js")), "x").unwrap();
        }
        assert!(read(&temp.work(), "draft").unwrap_err().contains("40"));
    }

    #[cfg(unix)]
    #[test]
    fn refuses_symlinks_inside_drafts_and_source_dirs_escaping_workspace() {
        use std::os::unix::fs::symlink;
        let temp = Temp::new("symlink");
        let outside = temp.0.join("outside.js");
        fs::write(&outside, "private").unwrap();
        symlink(&outside, temp.work().join("draft/link.js")).unwrap();
        assert!(read(&temp.work(), "draft")
            .unwrap_err()
            .contains("符号链接"));
        symlink(&temp.0, temp.work().join("escape")).unwrap();
        assert!(read(&temp.work(), "escape").is_err());
    }

    #[test]
    fn refuses_oversized_drafts_and_paths_deeper_than_the_game_format() {
        let temp = Temp::new("total-depth");
        let data = vec![b'x'; MAX_FILE_BYTES];
        for i in 0..4 {
            fs::write(temp.work().join(format!("draft/large{i}.js")), &data).unwrap();
        }
        assert!(read(&temp.work(), "draft").unwrap_err().contains("合计"));
        for i in 0..4 {
            fs::remove_file(temp.work().join(format!("draft/large{i}.js"))).unwrap();
        }
        fs::create_dir_all(temp.work().join("draft/a/b/c")).unwrap();
        fs::write(temp.work().join("draft/a/b/c/deep.js"), "x").unwrap();
        assert!(read(&temp.work(), "draft").is_ok());
        fs::create_dir_all(temp.work().join("draft/a/b/c/d")).unwrap();
        assert!(read(&temp.work(), "draft")
            .unwrap_err()
            .contains("最多 4 层"));
    }
}
