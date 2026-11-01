//! Binary sources remain byte-for-byte exportable. The JSON store uses data URLs,
//! while previews resolve them inside their opaque sandbox (never through IPC).
use base64::{engine::general_purpose::STANDARD, Engine};

pub(crate) fn is_asset(name: &str) -> bool {
    name.split('/').any(|p| p == "_blob") ||
    ["png", "jpg", "jpeg", "gif", "webp", "avif", "ico", "woff", "woff2", "ttf", "otf", "mp3", "wav", "ogg", "mp4", "webm"]
        .iter().any(|ext| name.to_ascii_lowercase().ends_with(&format!(".{ext}")))
}

fn mime(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") { Some("image/png") }
    else if bytes.starts_with(b"\xff\xd8\xff") { Some("image/jpeg") }
    else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") { Some("image/gif") }
    else if bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP") { Some("image/webp") }
    else if bytes.get(4..12) == Some(b"ftypavif") || bytes.get(4..12) == Some(b"ftypavis") { Some("image/avif") }
    else if bytes.starts_with(b"\0\0\x01\0") { Some("image/x-icon") }
    else if bytes.starts_with(b"wOF2") { Some("font/woff2") }
    else if bytes.starts_with(b"wOFF") { Some("font/woff") }
    else if bytes.starts_with(b"OTTO") { Some("font/otf") }
    else if bytes.starts_with(b"\0\x01\0\0") { Some("font/ttf") }
    else if bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WAVE") { Some("audio/wav") }
    else if bytes.starts_with(b"OggS") { Some("audio/ogg") }
    else if bytes.starts_with(b"ID3") || bytes.first() == Some(&255) && bytes.get(1).is_some_and(|b| b & 0xe0 == 0xe0) { Some("audio/mpeg") }
    else if bytes.get(4..8) == Some(b"ftyp") { Some("video/mp4") }
    else if bytes.starts_with(b"\x1aE\xdf\xa3") { Some("video/webm") }
    else if std::str::from_utf8(bytes).is_ok_and(|s| {
        let s = s.trim_start_matches('\u{feff}').trim_start();
        s.starts_with("<svg") || s.starts_with("<?xml") && s.contains("<svg")
    }) { Some("image/svg+xml") }
    else { None }
}

pub(crate) fn encode(bytes: &[u8]) -> Result<String, String> {
    let mime = mime(bytes).ok_or("不支持或无效的二进制素材类型")?;
    Ok(format!("data:{mime};base64,{}", STANDARD.encode(bytes)))
}

pub(crate) fn decode(text: &str) -> Result<Vec<u8>, String> {
    if text.len() > super::MAX_FILE.div_ceil(3) * 4 + 100 { return Err("素材超过 2MiB".into()); }
    let (header, body) = text.split_once(",").ok_or("素材需要 base64 data URL")?;
    let bytes = STANDARD.decode(body).map_err(|_| "素材 base64 无效")?;
    if bytes.len() > super::MAX_FILE { return Err("素材超过 2MiB".into()); }
    let detected = mime(&bytes).ok_or("不支持或无效的二进制素材类型")?;
    if header != format!("data:{detected};base64") { return Err("素材 MIME 与内容不一致".into()); }
    Ok(bytes)
}
