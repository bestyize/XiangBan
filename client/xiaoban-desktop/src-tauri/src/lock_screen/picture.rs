use base64::{engine::general_purpose::STANDARD, Engine};
use image::{ImageFormat, ImageReader, Limits};
use sha2::{Digest, Sha256};
use std::{fs, io::Cursor, path::{Path, PathBuf}};

#[cfg(windows)]
pub(super) static OPERATION: std::sync::Mutex<()> = std::sync::Mutex::new(());

const MAX_JPEG_BYTES: usize = 12 * 1024 * 1024;
const MAX_EDGE: u32 = 4096;

pub(super) fn prepare(directory: &Path, jpeg: &str) -> Result<(String, PathBuf), String> {
    let encoded = jpeg.strip_prefix("data:image/jpeg;base64,").ok_or("需要 JPEG 锁屏图片")?;
    if encoded.len() > MAX_JPEG_BYTES.div_ceil(3) * 4 {
        return Err("锁屏图片过大".into());
    }
    let bytes = STANDARD.decode(encoded).map_err(|_| "锁屏图片编码无效")?;
    if bytes.len() > MAX_JPEG_BYTES {
        return Err("锁屏图片过大".into());
    }
    let mut reader = ImageReader::with_format(Cursor::new(&bytes), ImageFormat::Jpeg);
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_EDGE);
    limits.max_image_height = Some(MAX_EDGE);
    limits.max_alloc = Some(128 * 1024 * 1024);
    reader.limits(limits);
    reader.decode().map_err(|_| "锁屏图片无法解码或尺寸过大")?;

    let id = format!("{:x}", Sha256::digest(&bytes));
    let path = prepared_path(directory, &id)?;
    fs::create_dir_all(directory).map_err(|e| e.to_string())?;
    if fs::read(&path).is_ok_and(|current| current == bytes) {
        return Ok((id, path));
    }
    let temporary = path.with_extension("tmp");
    let result = fs::write(&temporary, bytes).and_then(|()| fs::rename(&temporary, &path));
    if let Err(error) = result {
        let _ = fs::remove_file(temporary);
        return Err(format!("无法准备锁屏图片：{error}"));
    }
    Ok((id, path))
}

pub(super) fn prepared_path(directory: &Path, id: &str) -> Result<PathBuf, String> {
    if !is_picture_id(id) {
        return Err("锁屏图片标识无效".into());
    }
    Ok(directory.join(format!("{id}.jpg")))
}

fn is_picture_id(id: &str) -> bool {
    id.len() == 64 && id.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

// Keep the latest file behind the lock screen and only remove names owned by this feature.
#[cfg(windows)]
pub(super) fn remove_previous(directory: &Path, keep: &Path) {
    let Ok(entries) = fs::read_dir(directory) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path != keep
            && path.extension().is_some_and(|ext| ext == "jpg")
            && path.file_stem().and_then(|stem| stem.to_str()).is_some_and(is_picture_id)
        {
            let _ = fs::remove_file(path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{codecs::jpeg::JpegEncoder, ExtendedColorType};

    #[test]
    fn prepares_only_valid_jpeg_and_uses_a_safe_content_id() {
        let directory = std::env::temp_dir().join(format!("xiaoban-lock-picture-{}", std::process::id()));
        let mut jpeg = Vec::new();
        JpegEncoder::new(&mut jpeg).encode(&[255, 0, 1], 1, 1, ExtendedColorType::Rgb8).unwrap();
        let data = format!("data:image/jpeg;base64,{}", STANDARD.encode(&jpeg));
        let (id, path) = prepare(&directory, &data).unwrap();
        assert_eq!(prepared_path(&directory, &id).unwrap(), path);
        assert_eq!(fs::read(&path).unwrap(), jpeg);
        assert_eq!(prepare(&directory, &data).unwrap().1, path);
        assert!(prepared_path(&directory, "../secret").is_err());
        assert!(prepare(&directory, "data:image/jpeg;base64,aGVsbG8=").is_err());
        assert!(prepare(&directory, "data:image/png;base64,aGVsbG8=").is_err());
        fs::remove_dir_all(directory).unwrap();
    }
}
