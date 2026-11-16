//! Copies stand-alone HTML files into owned storage without a document size limit.
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

pub const ENTRY: &str = "index.html";

pub fn is_html(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("html") || ext.eq_ignore_ascii_case("htm"))
}

pub fn validate_chunk(buffer: &mut [u8], has_text: &mut bool) -> Result<usize, String> {
    let text = match std::str::from_utf8(buffer) {
        Ok(text) => text,
        Err(error) if error.error_len().is_none() => {
            std::str::from_utf8(&buffer[..error.valid_up_to()]).expect("validated UTF-8 prefix")
        }
        Err(_) => return Err("请选择 UTF-8 HTML 文件".into()),
    };
    if text.contains('\0') {
        return Err("HTML 文件包含无效字符".into());
    }
    *has_text |= text.chars().any(|c| c != '\u{feff}' && !c.is_whitespace());
    let len = text.len();
    let tail = buffer.len() - len;
    buffer.copy_within(len.., 0);
    Ok(tail)
}

/// The caller owns a staging folder and removes it on failure.
pub fn copy(source: &Path, target: &Path) -> Result<(), String> {
    if !fs::symlink_metadata(source).is_ok_and(|meta| meta.is_file()) {
        return Err("请选择本机的 HTML 文件".into());
    }
    let mut input = File::open(source).map_err(|e| format!("无法读取 HTML：{e}"))?;
    let expected = input.metadata().map_err(|e| e.to_string())?.len();
    let mut output = File::create(target).map_err(|e| format!("无法保存 HTML：{e}"))?;
    let (mut tail, mut has_text, mut copied) = (0, false, 0u64);
    let mut buffer = [0u8; 65536];
    loop {
        let count = input.read(&mut buffer[tail..]).map_err(|e| e.to_string())?;
        if count == 0 {
            break;
        }
        let end = tail + count;
        copied += count as u64;
        output
            .write_all(&buffer[tail..end])
            .map_err(|e| format!("无法保存 HTML：{e}"))?;
        tail = validate_chunk(&mut buffer[..end], &mut has_text)?;
    }
    if copied != expected {
        return Err("导入时源文件发生变化，请重试".into());
    }
    if tail != 0 {
        return Err("请选择 UTF-8 HTML 文件".into());
    }
    if !has_text {
        return Err("HTML 内容不能为空".into());
    }
    output.sync_all().map_err(|e| format!("无法保存 HTML：{e}"))
}

pub fn installed_dir(base: &Path, id: &str) -> Result<PathBuf, String> {
    if !crate::effect_choice::valid_id(id) {
        return Err("无效的资源 id".into());
    }
    let path = base.join(id);
    if [base, path.as_path()]
        .into_iter()
        .all(|dir| fs::symlink_metadata(dir).is_ok_and(|meta| meta.is_dir()))
    {
        Ok(path)
    } else {
        Err("找不到 HTML 资源目录".into())
    }
}

pub fn document(directory: &Path) -> Result<PathBuf, String> {
    let path = directory.join(ENTRY);
    if fs::symlink_metadata(&path).is_ok_and(|meta| meta.is_file() && meta.len() > 0) {
        Ok(path)
    } else {
        Err("找不到 HTML 源文件".into())
    }
}

/// Commits both a small manifest and its document together.
pub fn install(
    base: &Path,
    id: &str,
    manifest: &serde_json::Value,
    write_html: impl FnOnce(&Path) -> Result<(), String>,
) -> Result<(), String> {
    if !crate::effect_choice::valid_id(id) {
        return Err("无效的资源 id".into());
    }
    fs::create_dir_all(base).map_err(|e| format!("无法创建资源目录：{e}"))?;
    if !fs::symlink_metadata(base).is_ok_and(|meta| meta.is_dir()) {
        return Err("HTML 资源目录无效".into());
    }
    let target = base.join(id);
    if fs::symlink_metadata(&target).is_ok() {
        return Err(format!("资源「{id}」已存在"));
    }
    let staging = base.join(format!(".{id}-{}.tmp", std::process::id()));
    fs::create_dir(&staging).map_err(|e| format!("无法准备 HTML 资源：{e}"))?;
    let result = (|| {
        write_html(&staging.join(ENTRY))?;
        let json = serde_json::to_vec_pretty(manifest).map_err(|e| e.to_string())?;
        fs::write(staging.join("manifest.json"), json)
            .map_err(|e| format!("无法写入资源清单：{e}"))?;
        fs::rename(&staging, target).map_err(|e| format!("无法安装 HTML 资源：{e}"))
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(&staging);
    }
    result
}
