//! V2 App Runtime URI scheme protocol: `app-pack`.
//! Serves application files safely from builtin packs and installed user packs.
//! Enforces sandboxed paths (no traversal), safe MIME types, Range headers, and strict CSP.

use std::{
    fs::{self, File},
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};

pub const MAX_SERVED_BYTES: u64 = 64 * 1024 * 1024; // 64 MB per file limit

pub fn base_url(app_id: &str) -> String {
    if cfg!(windows) {
        format!("http://app-pack.localhost/{app_id}/")
    } else {
        format!("app-pack://localhost/{app_id}/")
    }
}

pub fn csp(windows: bool) -> String {
    let origin = if windows {
        "app-pack: http://app-pack.localhost"
    } else {
        "app-pack:"
    };
    format!(
        "default-src 'none'; script-src 'self' {origin} 'unsafe-inline'; style-src 'self' {origin} 'unsafe-inline'; \
         img-src 'self' {origin} data: blob:; media-src 'self' {origin} data: blob:; font-src 'self' {origin} data:; \
         connect-src 'none'; frame-src 'none'; worker-src 'self' blob:; object-src 'none'; base-uri 'none'; form-action 'none'"
    )
}

fn mime_for(ext: &str) -> Option<&'static str> {
    Some(match ext {
        "html" => "text/html; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "mp3" => "audio/mpeg",
        "ogg" => "audio/ogg",
        "wav" => "audio/wav",
        "woff2" => "font/woff2",
        "wasm" => "application/wasm",
        "txt" => "text/plain; charset=utf-8",
        _ => return None,
    })
}

fn percent_decode(raw: &str) -> Option<String> {
    let bytes = raw.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let hex = std::str::from_utf8(bytes.get(i + 1..i + 3)?).ok()?;
            out.push(u8::from_str_radix(hex, 16).ok()?);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}

fn safe_segments(path: &str) -> Option<Vec<String>> {
    let mut segments = Vec::new();
    for part in path.split('/') {
        if part.is_empty() {
            continue;
        }
        let decoded = percent_decode(part)?;
        if decoded == ".." || decoded == "." || decoded.starts_with('.') || decoded.contains('\\') || decoded.contains('\0') {
            return None;
        }
        segments.push(decoded);
    }
    Some(segments)
}

fn parse_range(header: &str, length: u64) -> Option<(u64, u64)> {
    let range = header.trim();
    if !range.starts_with("bytes=") {
        return None;
    }
    let spec = &range["bytes=".len()..];
    let (start_str, end_str) = spec.split_once('-')?;
    if start_str.is_empty() {
        let suffix: u64 = end_str.parse().ok()?;
        if suffix == 0 {
            return None;
        }
        let start = length.saturating_sub(suffix);
        Some((start, length - 1))
    } else {
        let start: u64 = start_str.parse().ok()?;
        if start >= length {
            return None;
        }
        let end = if end_str.is_empty() {
            length - 1
        } else {
            let e: u64 = end_str.parse().ok()?;
            if e < start {
                return None;
            }
            e.min(length - 1)
        };
        Some((start, end))
    }
}

pub struct Served {
    pub status: u16,
    pub mime: &'static str,
    pub body: Vec<u8>,
    pub range: Option<String>,
}

impl Served {
    fn status(status: u16) -> Self {
        Served {
            status,
            mime: "text/plain; charset=utf-8",
            body: Vec::new(),
            range: None,
        }
    }
}

pub fn resolve_app_file(root: &Path, segments: &[String]) -> Option<(PathBuf, &'static str)> {
    if segments.is_empty() {
        return None;
    }
    let app_id = &segments[0];
    if !super::storage::valid_app_id(app_id) {
        return None;
    }

    let rel_parts = if segments.len() == 1 {
        vec!["index.html"]
    } else {
        segments[1..].iter().map(|s| s.as_str()).collect()
    };

    let filename = rel_parts.last()?;
    let ext = filename.rsplit('.').next()?;
    let mime = mime_for(ext)?;

    // Check 1: User installed packs under apps/packs/<id>
    let user_pack_dir = root.join("apps").join("packs").join(app_id);
    let mut candidate = user_pack_dir.clone();
    for p in &rel_parts {
        candidate.push(p);
    }
    if candidate.is_file() {
        if let (Ok(real_base), Ok(real_cand)) = (user_pack_dir.canonicalize(), candidate.canonicalize()) {
            if real_cand.starts_with(real_base) {
                return Some((real_cand, mime));
            }
        }
    }

    // Check 2: Built-in packs in public/game-packs/<id>
    let exe_dir = std::env::current_exe().ok().and_then(|p| p.parent().map(|p| p.to_path_buf())).unwrap_or_else(|| PathBuf::from("."));
    let builtin_dirs = [
        PathBuf::from("public/game-packs").join(app_id),
        PathBuf::from("client/xiaoban-desktop/public/game-packs").join(app_id),
        exe_dir.join("public/game-packs").join(app_id),
        exe_dir.join("../public/game-packs").join(app_id),
    ];
    for b_dir in builtin_dirs {
        let mut b_candidate = b_dir.clone();
        for p in &rel_parts {
            b_candidate.push(p);
        }
        if b_candidate.is_file() {
            if let (Ok(real_base), Ok(real_cand)) = (b_dir.canonicalize(), b_candidate.canonicalize()) {
                if real_cand.starts_with(real_base) {
                    return Some((real_cand, mime));
                }
            }
        }
    }

    None
}

pub fn serve_from(root: &Path, raw_path: &str, range_header: Option<&str>) -> Served {
    let Some(segments) = safe_segments(raw_path) else {
        return Served::status(400);
    };

    let Some((file_path, mime)) = resolve_app_file(root, &segments) else {
        return Served::status(404);
    };

    let Ok(metadata) = fs::metadata(&file_path) else {
        return Served::status(404);
    };

    let length = metadata.len();
    if length > MAX_SERVED_BYTES {
        return Served::status(413);
    }

    let Ok(mut file) = File::open(&file_path) else {
        return Served::status(404);
    };

    if let Some(range_str) = range_header {
        if let Some((start, end)) = parse_range(range_str, length) {
            let part_len = (end - start + 1) as usize;
            if file.seek(SeekFrom::Start(start)).is_ok() {
                let mut buf = vec![0u8; part_len];
                if file.read_exact(&mut buf).is_ok() {
                    return Served {
                        status: 206,
                        mime,
                        body: buf,
                        range: Some(format!("bytes {start}-{end}/{length}")),
                    };
                }
            }
        }
        return Served {
            range: Some(format!("bytes */{length}")),
            ..Served::status(416)
        };
    }

    let mut body = Vec::with_capacity(length as usize);
    if file.read_to_end(&mut body).is_err() {
        return Served::status(500);
    }

    Served {
        status: 200,
        mime,
        body,
        range: None,
    }
}

pub fn serve(app: &tauri::AppHandle, request: tauri::http::Request<Vec<u8>>) -> tauri::http::Response<Vec<u8>> {
    let root = crate::storage::root(app);
    let range = request.headers().get("range").and_then(|v| v.to_str().ok());
    let served = match root {
        Ok(root) => serve_from(&root, request.uri().path(), range),
        Err(_) => Served::status(404),
    };

    let mut response = tauri::http::Response::builder()
        .status(served.status)
        .header("Content-Type", served.mime)
        .header("Content-Length", served.body.len())
        .header("Content-Security-Policy", csp(cfg!(windows)))
        .header("X-Content-Type-Options", "nosniff")
        .header("Referrer-Policy", "no-referrer")
        .header("Access-Control-Allow-Origin", "*")
        .header("Accept-Ranges", "bytes")
        .header("Cache-Control", "no-store");

    if let Some(range_val) = served.range {
        response = response.header("Content-Range", range_val);
    }

    response.body(served.body).unwrap_or_else(|_| tauri::http::Response::new(Vec::new()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_url_format() {
        let url = base_url("mbti");
        if cfg!(windows) {
            assert_eq!(url, "http://app-pack.localhost/mbti/");
        } else {
            assert_eq!(url, "app-pack://localhost/mbti/");
        }
    }

    #[test]
    fn test_safe_segments() {
        assert_eq!(safe_segments("/mbti/index.html"), Some(vec!["mbti".into(), "index.html".into()]));
        assert_eq!(safe_segments("/mbti/../secrets"), None);
        assert_eq!(safe_segments("/mbti/.hidden"), None);
        assert_eq!(safe_segments("/mbti/file\\name"), None);
    }

    #[test]
    fn test_mime_for() {
        assert_eq!(mime_for("html"), Some("text/html; charset=utf-8"));
        assert_eq!(mime_for("mjs"), Some("text/javascript; charset=utf-8"));
        assert_eq!(mime_for("wasm"), Some("application/wasm"));
        assert_eq!(mime_for("exe"), None);
    }
}
