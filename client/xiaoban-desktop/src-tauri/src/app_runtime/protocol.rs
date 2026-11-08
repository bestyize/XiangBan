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

pub fn csp(windows: bool, network_allowed: bool) -> String {
    let origin = if windows {
        "app-pack: http://app-pack.localhost"
    } else {
        "app-pack:"
    };
    let remote = if network_allowed { " https:" } else { "" };
    let remote_connect = if network_allowed { " https: wss:" } else { "" };
    format!(
        "default-src 'none'; script-src 'self' {origin}{remote} 'unsafe-inline'; style-src 'self' {origin}{remote} 'unsafe-inline'; \
         img-src 'self' {origin}{remote} data: blob:; media-src 'self' {origin}{remote} data: blob:; font-src 'self' {origin}{remote} data:; \
         connect-src 'self' {origin}{remote_connect}; frame-src 'none'; worker-src 'self' blob:; object-src 'none'; base-uri 'none'; form-action 'none'"
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
        if decoded == ".." || decoded == "." || decoded.starts_with('.') || decoded.contains('/') || decoded.contains('\\') || decoded.contains('\0') {
            return None;
        }
        segments.push(decoded);
    }
    Some(segments)
}

fn parse_range(header: &str, length: u64) -> Option<(u64, u64)> {
    if length == 0 { return None; }
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

    // The desktop creator and the app plaza both list packs from games/packs.
    let user_pack_dir = crate::game_packs::packs_dir(root).join(app_id);
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
        PathBuf::from("../public/game-packs").join(app_id),
        PathBuf::from("public/game-packs").join(app_id),
        PathBuf::from("client/xiaoban-desktop/public/game-packs").join(app_id),
        exe_dir.join("../../../public/game-packs").join(app_id),
        exe_dir.join("../../public/game-packs").join(app_id),
        exe_dir.join("../public/game-packs").join(app_id),
        exe_dir.join("public/game-packs").join(app_id),
        exe_dir.join("../Resources/public/game-packs").join(app_id),
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

pub fn serve_for(root: &Path, window_label: &str, raw_path: &str, range_header: Option<&str>) -> Served {
    let Some(app_id) = super::window::app_id_from_label(window_label) else {
        return Served::status(403);
    };
    let Some(segments) = safe_segments(raw_path) else {
        return Served::status(400);
    };
    if segments.first().map(String::as_str) != Some(app_id) {
        return Served::status(403);
    }
    serve_from(root, raw_path, range_header)
}

// The installed desktop app embeds its frontend in the executable. Only fall
// back to those assets for the official MBTI bundle and its own runtime window.
fn serve_embedded_mbti<F>(window_label: &str, raw_path: &str, range_header: Option<&str>, fetch: F) -> Served
where F: FnOnce(String) -> Option<Vec<u8>> {
    if super::window::app_id_from_label(window_label) != Some("mbti") { return Served::status(403); }
    let Some(mut segments) = safe_segments(raw_path) else { return Served::status(400); };
    if segments.first().map(String::as_str) != Some("mbti") { return Served::status(403); }
    if segments.len() == 1 { segments.push("index.html".into()); }
    let Some(mime) = segments.last().and_then(|name| name.rsplit('.').next()).and_then(mime_for) else { return Served::status(404); };
    let Some(bytes) = fetch(format!("game-packs/{}", segments.join("/"))) else { return Served::status(404); };
    let length = bytes.len() as u64;
    if length > MAX_SERVED_BYTES { return Served::status(413); }
    if let Some(header) = range_header {
        if let Some((start, end)) = parse_range(header, length) {
            return Served { status: 206, mime, body: bytes[start as usize..=end as usize].to_vec(), range: Some(format!("bytes {start}-{end}/{length}")) };
        }
        return Served { range: Some(format!("bytes */{length}")), ..Served::status(416) };
    }
    Served { status: 200, mime, body: bytes, range: None }
}

pub fn serve(app: &tauri::AppHandle, window_label: &str, request: tauri::http::Request<Vec<u8>>) -> tauri::http::Response<Vec<u8>> {
    let root = crate::storage::root(app);
    let range = request.headers().get("range").and_then(|v| v.to_str().ok());
    let mut served = match root {
        Ok(root) => serve_for(&root, window_label, request.uri().path(), range),
        Err(_) => Served::status(404),
    };
    if served.status == 404 && window_label == "app-runtime-mbti" {
        served = serve_embedded_mbti(window_label, request.uri().path(), range, |path| {
            let assets = app.asset_resolver();
            // AssetResolver::get falls back to the desktop index.html for unknown paths.
            // Do not let that fallback masquerade as an MBTI package file.
            if !cfg!(debug_assertions) && !assets.iter().any(|(key, _)| key.trim_start_matches('/') == path) { return None; }
            assets.get(path).map(|asset| asset.bytes)
        });
    }

    let mut response = tauri::http::Response::builder()
        .status(served.status)
        .header("Content-Type", served.mime)
        .header("Content-Length", served.body.len())
        .header("Content-Security-Policy", csp(cfg!(windows), super::window::network_allowed()))
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
    fn packaged_mbti_assets_are_scoped_and_support_ranges() {
        let fetch = |path: String| (path == "game-packs/mbti/assets/test-qr.png").then(|| b"012345".to_vec());
        let ok = serve_embedded_mbti("app-runtime-mbti", "/mbti/assets/test-qr.png", Some("bytes=1-3"), fetch);
        assert_eq!((ok.status, ok.body.as_slice(), ok.range.as_deref()), (206, &b"123"[..], Some("bytes 1-3/6")));
        assert_eq!(serve_embedded_mbti("app-runtime-other", "/mbti/assets/test-qr.png", None, fetch).status, 403);
        assert_eq!(serve_embedded_mbti("app-runtime-mbti", "/mbti/%2e%2e%2fsecret.png", None, fetch).status, 400);
        assert_eq!(serve_embedded_mbti("app-runtime-mbti", "/other/assets/test-qr.png", None, fetch).status, 403);
    }

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
    fn one_runtime_window_cannot_request_another_pack() {
        let root = std::env::temp_dir().join(format!("xiaoban-app-runtime-request-{}", std::process::id()));
        let pack = crate::game_packs::packs_dir(&root).join("one");
        fs::create_dir_all(&pack).unwrap();
        fs::write(pack.join("index.html"), "<title>one</title>").unwrap();
        assert_eq!(serve_for(&root, "app-runtime-one", "/one/index.html", None).status, 200);
        assert_eq!(serve_for(&root, "app-runtime-two", "/one/index.html", None).status, 403);
        assert_eq!(serve_for(&root, "control", "/one/index.html", None).status, 403);
        fs::remove_dir_all(root).unwrap();
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

    #[test]
    fn test_resolve_builtin_mbti() {
        let root = PathBuf::from("temp-fake-root");
        let segments = vec!["mbti".to_string(), "index.html".to_string()];
        let resolved = resolve_app_file(&root, &segments);
        assert!(resolved.is_some(), "mbti index.html must be resolvable");
        let (path, mime) = resolved.unwrap();
        assert_eq!(mime, "text/html; charset=utf-8");
        assert!(path.ends_with("index.html"));
    }
}
