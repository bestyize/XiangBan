//! "Folder is a game": user games live in `<storage root>/games/packs/<id>/`.
//!
//! Everything here takes a plain `Path`, so the rules (manifest checks, folder scanning, the
//! `game-pack` protocol's path guard, atomic creation) are testable with a temporary directory.
//! Pack frames still get no IPC and no filesystem access; they only receive files through the protocol.
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};

pub const MAX_USER_GAMES: usize = 40;
pub const MAX_FILES: usize = 40;
pub const MAX_FILE_BYTES: usize = 1024 * 1024;
pub const MAX_TOTAL_BYTES: usize = 4 * 1024 * 1024;
const MAX_MANIFEST_BYTES: u64 = 64 * 1024;
const MAX_SERVED_BYTES: u64 = 32 * 1024 * 1024;
/// Text extensions the Agent may write.
const WRITE_EXTENSIONS: [&str; 7] = ["html", "js", "css", "json", "svg", "md", "txt"];
const COVER_EXTENSIONS: [&str; 5] = ["svg", "png", "jpg", "jpeg", "webp"];
pub const HOST_SCRIPT: &str = include_str!("../../public/game-packs/falling-blocks/desktop-host.js");
const HOST_TAG: &str = "<script src=\"/__host/desktop-host.js\"></script>";
/// Appended to the injected host script: answers host.init with game.ready so a user game that skips the
/// handshake still loads. A second game.ready from the game itself is harmless.
const AUTO_READY: &str = "\n(() => {\n  const nonce = new URLSearchParams(location.search).get('hostNonce')\n  if (!nonce || window.parent === window) return\n  window.addEventListener('message', event => {\n    const d = event.data\n    if (event.source !== window.parent || !d || d.channel !== 'xiaban.game-host' || d.protocol !== 1 || d.nonce !== nonce || d.type !== 'host.init') return\n    window.parent.postMessage({channel:'xiaban.game-pack',protocol:1,type:'game.ready',nonce}, '*')\n  })\n})()\n";
fn injected_host_script() -> Vec<u8> { [HOST_SCRIPT, AUTO_READY].concat().into_bytes() }

/// Serialises create/delete so two tool calls never rename the same folder at once.
static WRITE_LOCK: Mutex<()> = Mutex::new(());

pub fn valid_id(id: &str) -> bool {
    let bytes = id.as_bytes();
    (2..=40).contains(&bytes.len())
        && (bytes[0].is_ascii_lowercase() || bytes[0].is_ascii_digit())
        && bytes.iter().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == b'-')
}

pub fn is_builtin(id: &str) -> bool {
    crate::games::IDS.contains(&id)
}

fn check_id(id: &str) -> Result<(), String> {
    if !valid_id(id) {
        return Err("游戏 id 只能用小写字母、数字和 -，以字母或数字开头，长度 2 到 40".into());
    }
    if is_builtin(id) {
        return Err(format!("id「{id}」与内置游戏重名，请换一个"));
    }
    Ok(())
}

pub fn packs_dir(root: &Path) -> PathBuf {
    root.join("games").join("packs")
}

pub fn save_dir(root: &Path) -> PathBuf {
    root.join("games").join("saves")
}

pub fn user_save_name(id: &str) -> String {
    format!("user-{id}.json")
}

/// Where the games window finds a user game's files.
pub fn base_url(id: &str) -> String {
    if cfg!(windows) {
        format!("http://game-pack.localhost/{id}/")
    } else {
        format!("game-pack://localhost/{id}/")
    }
}

fn check_meta(name: &str, description: &str) -> Result<(String, String), String> {
    let name = name.trim();
    let description = description.trim();
    if name.is_empty() {
        return Err("缺少游戏名称 name".into());
    }
    if name.chars().count() > 24 {
        return Err("游戏名称 name 最多 24 个字".into());
    }
    if description.is_empty() {
        return Err("缺少游戏简介 description".into());
    }
    if description.chars().count() > 80 {
        return Err("游戏简介 description 最多 80 个字".into());
    }
    Ok((name.to_string(), description.to_string()))
}

/// Splits a relative path into safe segments: no empty, `.`/`..`, dot-files, drive letters or backslashes.
fn safe_segments(path: &str) -> Option<Vec<&str>> {
    let path = path.strip_prefix("./").unwrap_or(path);
    if path.is_empty() || path.starts_with('/') {
        return None;
    }
    let segments: Vec<&str> = path.split('/').collect();
    segments
        .iter()
        .all(|s| !s.is_empty() && !s.starts_with('.') && !s.contains(['\\', ':', '\0']))
        .then_some(segments)
}

fn extension(name: &str) -> Option<String> {
    let (stem, ext) = name.rsplit_once('.')?;
    (!stem.is_empty()).then(|| ext.to_ascii_lowercase())
}

/// Resolves `segments` inside `dir` to a real file. Errors are HTTP-style: 403 when the path
/// leaves the folder (symlinks included), 404 when there is nothing there.
fn resolve_file(dir: &Path, segments: &[&str]) -> Result<PathBuf, u16> {
    let base = dir.canonicalize().map_err(|_| 404u16)?;
    let mut full = base.clone();
    full.extend(segments);
    let real = full.canonicalize().map_err(|_| 404u16)?;
    if !real.starts_with(&base) {
        return Err(403);
    }
    if real.is_file() {
        Ok(real)
    } else {
        Err(404)
    }
}

/// Checks a manifest read from `dir` and returns the normalized catalog entry.
pub fn validate_manifest(dir: &Path, folder: &str, manifest: &Value) -> Result<Value, String> {
    check_id(folder)?;
    if manifest.get("id").and_then(Value::as_str) != Some(folder) {
        return Err(format!("manifest 的 id 必须与文件夹名「{folder}」一致"));
    }
    let text = |key: &str| manifest.get(key).and_then(Value::as_str).ok_or(format!("manifest 缺少 {key}"));
    let (name, description) = check_meta(text("name")?, text("description")?)?;
    if text("entry")? != "index.html" {
        return Err("manifest 的 entry 必须是 index.html".into());
    }
    if manifest.get("runtimeProtocol").and_then(Value::as_u64) != Some(1) {
        return Err("manifest 的 runtimeProtocol 必须是 1".into());
    }
    resolve_file(dir, &["index.html"]).map_err(|_| "缺少入口页 index.html".to_string())?;
    let mut entry = json!({
        "id": folder, "name": name, "description": description, "entry": "index.html",
        "runtimeProtocol": 1, "save": false,
    });
    if let Some(cover) = manifest.get("cover").filter(|v| !v.is_null()) {
        let cover = cover.as_str().ok_or("cover 需要是文字路径")?;
        let segments = safe_segments(cover).ok_or("cover 必须是游戏文件夹内的相对路径")?;
        let ext = extension(segments[segments.len() - 1]).unwrap_or_default();
        if !COVER_EXTENSIONS.contains(&ext.as_str()) {
            return Err("cover 需要是 svg、png、jpg 或 webp 图片".into());
        }
        resolve_file(dir, &segments).map_err(|_| format!("找不到封面文件 {cover}"))?;
        entry["cover"] = json!(segments.join("/"));
    }
    if let Some(save) = manifest.get("save").filter(|v| !v.is_null()) {
        entry["save"] = json!(save.as_bool().ok_or("save 需要是 true 或 false")?);
    }
    if let Some(orientation) = manifest.get("orientation").filter(|v| !v.is_null()) {
        match orientation.as_str() {
            Some(value @ ("landscape" | "portrait")) => entry["orientation"] = json!(value),
            _ => return Err("orientation 需要是 landscape 或 portrait".into()),
        }
    }
    Ok(entry)
}

/// Reads and validates `<dir>/manifest.json`; the result carries `source` and `base` for the front end.
pub fn load_manifest(dir: &Path, folder: &str) -> Result<Value, String> {
    check_id(folder)?;
    let path = dir.join("manifest.json");
    match fs::metadata(&path) {
        Ok(m) if m.is_file() && m.len() <= MAX_MANIFEST_BYTES => {}
        Ok(m) if m.is_file() => return Err("manifest.json 超过 64 KB".into()),
        _ => return Err("缺少 manifest.json".into()),
    }
    let text = fs::read_to_string(&path).map_err(|_| "manifest.json 需要是 UTF-8 文本".to_string())?;
    let manifest: Value = serde_json::from_str(text.trim_start_matches('\u{feff}')).map_err(|e| format!("manifest.json 格式错误：{e}"))?;
    let mut entry = validate_manifest(dir, folder, &manifest)?;
    entry["source"] = json!("user");
    entry["base"] = json!(base_url(folder));
    Ok(entry)
}

pub fn find_user_game(root: &Path, id: &str) -> Result<Value, String> {
    check_id(id)?;
    load_manifest(&packs_dir(root).join(id), id)
}

#[derive(Default, Debug)]
pub struct Scan {
    pub games: Vec<Value>,
    pub errors: Vec<String>,
}

/// Lists the user's games. A broken folder is skipped and reported, never fatal.
pub fn scan(root: &Path) -> Scan {
    let mut out = Scan::default();
    let directory = packs_dir(root);
    let Ok(entries) = fs::read_dir(&directory) else { return out };
    let mut names: Vec<String> = entries.flatten().filter_map(|e| e.file_name().into_string().ok()).filter(|n| !n.starts_with('.')).collect();
    names.sort();
    for name in names {
        let path = directory.join(&name);
        if !path.is_dir() {
            continue;
        }
        if out.games.len() >= MAX_USER_GAMES {
            out.errors.push(format!("最多加载 {MAX_USER_GAMES} 个自创游戏，从「{name}」起的文件夹已忽略"));
            break;
        }
        match load_manifest(&path, &name) {
            Ok(entry) => out.games.push(entry),
            Err(error) => out.errors.push(format!("「{name}」：{error}")),
        }
    }
    out
}

// ---------------------------------------------------------------- protocol

pub struct Served {
    pub status: u16,
    pub mime: &'static str,
    pub body: Vec<u8>,
    /// `Content-Range` value when the answer is a 206 or 416.
    pub range: Option<String>,
}

impl Served {
    fn status(status: u16) -> Self {
        Served { status, mime: "text/plain; charset=utf-8", body: Vec::new(), range: None }
    }
}

fn mime_for(ext: &str) -> Option<&'static str> {
    Some(match ext {
        "html" => "text/html; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
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
        "txt" => "text/plain; charset=utf-8",
        _ => return None,
    })
}

/// Content-Security-Policy for pack responses. Frames are sandboxed without `allow-same-origin`, so the
/// frame's own origin is opaque; the protocol origin is therefore also named explicitly next to `'self'`.
pub fn csp(windows: bool) -> String {
    let origin = if windows { "game-pack: http://game-pack.localhost" } else { "game-pack:" };
    format!(
        "default-src 'none'; script-src 'self' {origin} 'unsafe-inline'; style-src 'self' {origin} 'unsafe-inline'; \
         img-src 'self' {origin} data: blob:; media-src 'self' {origin} data: blob:; font-src 'self' {origin} data:; \
         connect-src 'none'; frame-src 'none'; worker-src blob:; object-src 'none'; base-uri 'none'; form-action 'none'"
    )
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

/// Puts the desktop host adapter into an entry page that does not load it itself.
pub fn inject_host(html: &str) -> String {
    if html.contains("desktop-host.js") {
        return html.to_string();
    }
    let lower = html.to_ascii_lowercase();
    for tag in ["<head", "<html"] {
        let mut from = 0;
        while let Some(found) = lower[from..].find(tag) {
            let start = from + found;
            let after = start + tag.len();
            let boundary = lower.as_bytes().get(after).is_none_or(|c| matches!(c, b'>' | b'/' | b' ' | b'\t' | b'\n' | b'\r'));
            if boundary {
                if let Some(close) = lower[after..].find('>') {
                    let at = after + close + 1;
                    return format!("{}{HOST_TAG}{}", &html[..at], &html[at..]);
                }
            }
            from = after;
        }
    }
    format!("{HOST_TAG}{html}")
}

fn parse_range(header: &str, length: u64) -> Option<(u64, u64)> {
    let spec = header.strip_prefix("bytes=")?;
    if spec.contains(',') || length == 0 {
        return None;
    }
    let (start, end) = spec.split_once('-')?;
    if start.is_empty() {
        let suffix: u64 = end.parse().ok()?;
        return (suffix > 0).then(|| (length.saturating_sub(suffix), length - 1));
    }
    let start: u64 = start.parse().ok()?;
    let end = if end.is_empty() { length - 1 } else { end.parse::<u64>().ok()?.min(length - 1) };
    (start <= end && start < length).then_some((start, end))
}

/// Answers one request path (`/<id>/<file>` or `/__host/desktop-host.js`) from `root`.
pub fn serve_from(root: &Path, raw_path: &str, range: Option<&str>) -> Served {
    let Some(path) = percent_decode(raw_path) else { return Served::status(400) };
    if path.contains(['\\', '\0']) {
        return Served::status(400);
    }
    let segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    if segments[0] == "__host" {
        return if segments == ["__host", "desktop-host.js"] {
            Served { status: 200, mime: mime_for("js").unwrap_or_default(), body: injected_host_script(), range: None }
        } else {
            Served::status(404)
        };
    }
    let id = segments[0];
    if check_id(id).is_err() {
        return Served::status(404);
    }
    let directory = packs_dir(root).join(id);
    let rest = &segments[1..];
    let default_entry = ["index.html"];
    let rest: &[&str] = if rest.is_empty() || rest == [""] { &default_entry } else { rest };
    if rest.iter().any(|s| s.is_empty() || s.starts_with('.') || s.contains(':')) {
        return Served::status(403);
    }
    let Some(file_name) = rest.last() else { return Served::status(404) };
    let ext = extension(file_name).unwrap_or_default();
    let Some(mime) = mime_for(&ext) else { return Served::status(403) };
    if load_manifest(&directory, id).is_err() {
        return Served::status(404);
    }
    let file = match resolve_file(&directory, rest) {
        Ok(file) => file,
        Err(status) => return Served::status(status),
    };
    let length = match fs::metadata(&file) {
        Ok(m) if m.len() <= MAX_SERVED_BYTES => m.len(),
        Ok(_) => return Served::status(413),
        Err(_) => return Served::status(404),
    };
    let Ok(bytes) = fs::read(&file) else { return Served::status(404) };
    if rest == ["index.html"] {
        let html = String::from_utf8_lossy(&bytes);
        return Served { status: 200, mime, body: inject_host(&html).into_bytes(), range: None };
    }
    if let Some(header) = range.filter(|_| ext != "html") {
        return match parse_range(header, length) {
            Some((start, end)) => Served { status: 206, mime, body: bytes[start as usize..=end as usize].to_vec(), range: Some(format!("bytes {start}-{end}/{length}")) },
            None => Served { range: Some(format!("bytes */{length}")), ..Served::status(416) },
        };
    }
    Served { status: 200, mime, body: bytes, range: None }
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
        // Frames are sandboxed (opaque origin), so fonts and similar CORS-mode loads need this.
        .header("Access-Control-Allow-Origin", "*")
        .header("Accept-Ranges", "bytes")
        .header("Cache-Control", "no-store");
    if let Some(range) = served.range {
        response = response.header("Content-Range", range);
    }
    response.body(served.body).unwrap_or_else(|_| tauri::http::Response::new(Vec::new()))
}

// ---------------------------------------------------------------- creating and removing

fn valid_file_path(path: &str) -> Result<(), String> {
    let bad = || format!("文件路径「{path}」不合法：只能用字母、数字、. _ - 组成的分段，用 / 分隔，最多 4 层，不能以 . 开头");
    let segments: Vec<&str> = path.split('/').collect();
    if segments.len() > 4
        || !segments.iter().all(|s| !s.is_empty() && !s.starts_with('.') && s.bytes().all(|c| c.is_ascii_alphanumeric() || matches!(c, b'.' | b'_' | b'-')))
    {
        return Err(bad());
    }
    let ext = extension(segments[segments.len() - 1]).unwrap_or_default();
    if !WRITE_EXTENSIONS.contains(&ext.as_str()) {
        return Err(format!("文件「{path}」的类型不支持：只能写入 {} 文本文件", WRITE_EXTENSIONS.join("、")));
    }
    Ok(())
}

/// Every rule a `game_create` request must meet, checked before anything touches the disk.
pub fn validate_request(id: &str, name: &str, description: &str, files: &[(String, String)]) -> Result<(), String> {
    check_id(id)?;
    check_meta(name, description)?;
    if files.is_empty() || files.len() > MAX_FILES {
        return Err(format!("files 需要 1 到 {MAX_FILES} 个文件"));
    }
    let mut seen: Vec<String> = Vec::new();
    let mut total = 0usize;
    for (path, content) in files {
        valid_file_path(path)?;
        if path.eq_ignore_ascii_case("manifest.json") {
            return Err("manifest.json 由工具根据参数生成，files 里不能包含它".into());
        }
        let lower = path.to_ascii_lowercase();
        if seen.contains(&lower) {
            return Err(format!("文件路径「{path}」重复"));
        }
        if content.len() > MAX_FILE_BYTES {
            return Err(format!("文件「{path}」超过 1 MB"));
        }
        total += content.len();
        seen.push(lower);
    }
    if total > MAX_TOTAL_BYTES {
        return Err("所有文件合计超过 4 MB".into());
    }
    if !files.iter().any(|(p, _)| p == "index.html") {
        return Err("files 必须包含入口页 index.html".into());
    }
    // A name cannot be both a file and a folder (`a` and `a/b`).
    if seen.iter().any(|a| seen.iter().any(|b| b.starts_with(&format!("{a}/")))) {
        return Err("文件路径冲突：同一个名字不能既是文件又是文件夹".into());
    }
    Ok(())
}

fn random_tag() -> String {
    let mut bytes = [0u8; 4];
    let _ = getrandom::fill(&mut bytes);
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Moves the finished folder into place. An old folder is set aside first and put back if the move fails.
fn swap_in(temporary: &Path, target: &Path, backup: &Path, had_old: bool) -> Result<(), String> {
    if had_old {
        fs::rename(target, backup).map_err(|e| format!("无法替换旧游戏：{e}"))?;
    }
    if let Err(e) = fs::rename(temporary, target) {
        if had_old {
            let _ = fs::rename(backup, target);
        }
        return Err(format!("无法保存游戏：{e}"));
    }
    if had_old {
        let _ = fs::remove_dir_all(backup);
    }
    Ok(())
}

/// Writes a new game folder. The files go to a temporary folder first and are renamed into place at the
/// end; with `overwrite` the old folder is set aside and put back if the swap fails.
pub fn create_pack(root: &Path, id: &str, name: &str, description: &str, files: &[(String, String)], save: bool, overwrite: bool) -> Result<Value, String> {
    validate_request(id, name, description, files)?;
    let (name, description) = check_meta(name, description)?;
    let _guard = WRITE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let packs = packs_dir(root);
    fs::create_dir_all(&packs).map_err(|e| format!("无法创建游戏文件夹：{e}"))?;
    let target = packs.join(id);
    let exists = fs::symlink_metadata(&target).is_ok();
    if exists && !overwrite {
        return Err(format!("游戏「{id}」已经存在，请换一个 id，或设置 overwrite=true 覆盖它"));
    }
    if !exists {
        let count = scan(root).games.len();
        if count >= MAX_USER_GAMES {
            return Err(format!("自创游戏已满 {MAX_USER_GAMES} 个，请先删除一些"));
        }
    }
    let temporary = packs.join(format!(".tmp-{id}-{}", random_tag()));
    let build = || -> Result<(), String> {
        fs::create_dir_all(&temporary).map_err(|e| format!("无法创建临时文件夹：{e}"))?;
        for (path, content) in files {
            let destination = temporary.join(path);
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent).map_err(|e| format!("无法创建文件夹：{e}"))?;
            }
            fs::write(&destination, content.as_bytes()).map_err(|e| format!("无法写入 {path}：{e}"))?;
        }
        let mut manifest = json!({
            "id": id, "name": name, "description": description, "entry": "index.html",
            "runtimeProtocol": 1, "save": save,
        });
        if files.iter().any(|(p, _)| p == "cover.svg") {
            manifest["cover"] = json!("cover.svg");
        }
        let text = serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())?;
        fs::write(temporary.join("manifest.json"), text).map_err(|e| format!("无法写入 manifest.json：{e}"))?;
        load_manifest(&temporary, id).map(|_| ())
    };
    if let Err(error) = build() {
        let _ = fs::remove_dir_all(&temporary);
        return Err(error);
    }
    let backup = packs.join(format!(".bak-{id}-{}", random_tag()));
    if let Err(error) = swap_in(&temporary, &target, &backup, exists) {
        let _ = fs::remove_dir_all(&temporary);
        return Err(error);
    }
    Ok(json!({"id": id, "name": name, "directory": target, "files": files.len() + 1, "save": save, "replaced": exists}))
}

/// Deletes a user game folder and its save file (so a game created later under the same id starts fresh).
pub fn remove_pack(root: &Path, id: &str) -> Result<(), String> {
    check_id(id)?;
    let _guard = WRITE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let target = packs_dir(root).join(id);
    if fs::symlink_metadata(&target).is_err() {
        return Err(format!("没有找到自创游戏「{id}」"));
    }
    fs::remove_dir_all(&target).map_err(|e| format!("无法删除游戏：{e}"))?;
    let _ = fs::remove_file(save_dir(root).join(user_save_name(id)));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root(tag: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("xiaoban-game-packs-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }
    fn files(list: &[(&str, &str)]) -> Vec<(String, String)> {
        list.iter().map(|(p, c)| (p.to_string(), c.to_string())).collect()
    }
    const PAGE: &str = "<!doctype html><html><head><title>t</title></head><body><script src=\"game.js\"></script></body></html>";
    fn sample() -> Vec<(String, String)> {
        files(&[("index.html", PAGE), ("game.js", "document.title='x'")])
    }
    fn make(root: &Path, id: &str) {
        create_pack(root, id, "小游戏", "一个测试游戏", &sample(), false, false).unwrap();
    }
    fn write_manifest(root: &Path, folder: &str, manifest: Value) {
        let dir = packs_dir(root).join(folder);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("manifest.json"), manifest.to_string()).unwrap();
        fs::write(dir.join("index.html"), PAGE).unwrap();
    }
    fn manifest(id: &str) -> Value {
        json!({"id": id, "name": "好玩", "description": "介绍", "entry": "index.html", "runtimeProtocol": 1})
    }

    #[test]
    fn ids_follow_the_documented_pattern_and_builtin_names_are_taken() {
        for ok in ["ab", "a1", "my-game", "0day", &"a".repeat(40)] {
            assert!(valid_id(ok), "{ok}");
        }
        for bad in ["", "a", "-ab", "Ab", "a_b", "a b", "../x", "a.b", "好玩", &"a".repeat(41)] {
            assert!(!valid_id(bad), "{bad}");
        }
        assert!(check_id("snake-arena").unwrap_err().contains("内置"));
        assert!(check_id("ink-blade").is_err());
    }

    #[test]
    fn a_valid_manifest_is_normalised_and_gets_source_and_base() {
        let r = root("manifest-ok");
        write_manifest(&r, "good-game", json!({"id":"good-game","name":"好玩","description":"介绍","entry":"index.html","runtimeProtocol":1,"save":true,"orientation":"landscape","extra":1}));
        let entry = load_manifest(&packs_dir(&r).join("good-game"), "good-game").unwrap();
        assert_eq!(entry["source"], "user");
        assert_eq!(entry["save"], true);
        assert_eq!(entry["orientation"], "landscape");
        assert!(entry.get("cover").is_none() && entry.get("extra").is_none());
        assert!(entry["base"].as_str().unwrap().ends_with("/good-game/"));
        assert!(entry["base"].as_str().unwrap().contains("game-pack"));
        let _ = fs::remove_dir_all(r);
    }

    #[test]
    fn manifests_with_bad_fields_are_refused_with_a_reason() {
        let r = root("manifest-bad");
        let dir = packs_dir(&r).join("aa-game");
        write_manifest(&r, "aa-game", manifest("aa-game"));
        let check = |m: Value| validate_manifest(&dir, "aa-game", &m);
        let with = |key: &str, value: Value| {
            let mut m = manifest("aa-game");
            m[key] = value;
            m
        };
        assert!(check(manifest("aa-game")).is_ok());
        assert!(check(manifest("other")).unwrap_err().contains("文件夹名"));
        assert!(check(with("name", json!(""))).is_err());
        assert!(check(with("name", json!("字".repeat(25)))).unwrap_err().contains("24"));
        assert!(check(with("name", json!("字".repeat(24)))).is_ok());
        assert!(check(with("description", json!("字".repeat(81)))).unwrap_err().contains("80"));
        assert!(check(with("description", json!("字".repeat(80)))).is_ok());
        assert!(check(with("entry", json!("main.html"))).is_err());
        assert!(check(with("runtimeProtocol", json!(2))).is_err());
        assert!(check(with("save", json!("yes"))).is_err());
        assert!(check(with("orientation", json!("diagonal"))).is_err());
        // Cover must be a relative path to a real image inside the folder.
        for cover in ["../x.svg", "/etc/a.svg", "nope.svg", "a\\b.svg", "c:/x.svg", ".hidden.svg"] {
            assert!(check(with("cover", json!(cover))).is_err(), "{cover}");
        }
        fs::write(dir.join("cover.svg"), "<svg/>").unwrap();
        fs::write(dir.join("cover.txt"), "x").unwrap();
        assert_eq!(check(with("cover", json!("cover.svg"))).unwrap()["cover"], "cover.svg");
        assert!(check(with("cover", json!("cover.txt"))).is_err());
        // Builtin names are reserved even if the folder exists.
        write_manifest(&r, "ink-blade", manifest("ink-blade"));
        assert!(load_manifest(&packs_dir(&r).join("ink-blade"), "ink-blade").unwrap_err().contains("内置"));
        let _ = fs::remove_dir_all(r);
    }

    #[test]
    fn scan_merges_good_folders_and_reports_bad_ones() {
        let r = root("scan");
        assert!(scan(&r).games.is_empty());
        make(&r, "zeta-game");
        make(&r, "alpha-game");
        write_manifest(&r, "broken-one", json!({"id": "wrong"}));
        fs::create_dir_all(packs_dir(&r).join("no-manifest")).unwrap();
        fs::create_dir_all(packs_dir(&r).join("Bad Name")).unwrap();
        fs::create_dir_all(packs_dir(&r).join(".tmp-half-written")).unwrap();
        fs::write(packs_dir(&r).join("stray.txt"), "x").unwrap();
        let found = scan(&r);
        let ids: Vec<&str> = found.games.iter().map(|g| g["id"].as_str().unwrap()).collect();
        assert_eq!(ids, ["alpha-game", "zeta-game"]);
        assert_eq!(found.errors.len(), 3, "{:?}", found.errors);
        assert!(found.errors.iter().any(|e| e.contains("broken-one")));
        assert!(found.errors.iter().any(|e| e.contains("no-manifest")));
        assert!(!found.errors.iter().any(|e| e.contains("tmp") || e.contains("stray")));
        let _ = fs::remove_dir_all(r);
    }

    #[test]
    fn scan_stops_at_forty_games() {
        let r = root("scan-limit");
        for i in 0..42 {
            let id = format!("game-{i:02}");
            write_manifest(&r, &id, manifest(&id));
        }
        let found = scan(&r);
        assert_eq!(found.games.len(), MAX_USER_GAMES);
        assert_eq!(found.errors.len(), 1);
        assert!(found.errors[0].contains("40"));
        assert!(create_pack(&r, "one-more", "n", "d", &sample(), false, false).unwrap_err().contains("40"));
        let _ = fs::remove_dir_all(r);
    }

    #[test]
    fn host_script_is_injected_unless_the_page_already_loads_it() {
        assert_eq!(inject_host("<html><head><title>x</title></head></html>"), format!("<html><head>{HOST_TAG}<title>x</title></head></html>"));
        assert!(inject_host("<HEAD lang=a><meta></HEAD>").starts_with(&format!("<HEAD lang=a>{HOST_TAG}")));
        assert!(inject_host("<header></header><head>").contains(&format!("<head>{HOST_TAG}")), "<header> is not <head>");
        assert!(inject_host("<html><body>x</body></html>").starts_with(&format!("<html>{HOST_TAG}")));
        assert!(inject_host("<p>bare</p>").starts_with(HOST_TAG));
        let own = "<head><script src=\"./desktop-host.js\"></script></head>";
        assert_eq!(inject_host(own), own);
    }

    #[test]
    fn the_protocol_returns_the_entry_page_with_the_host_script_injected() {
        let r = root("serve-entry");
        make(&r, "demo-game");
        for path in ["/demo-game/index.html", "/demo-game/", "/demo-game"] {
            let page = serve_from(&r, path, None);
            assert_eq!(page.status, 200, "{path}");
            assert!(page.mime.starts_with("text/html"));
            let body = String::from_utf8(page.body).unwrap();
            assert!(body.contains(&format!("<head>{HOST_TAG}<title>")), "{body}");
            assert!(body.contains("game.js"));
        }
        let script = serve_from(&r, "/demo-game/game.js", None);
        assert_eq!((script.status, script.mime), (200, "text/javascript; charset=utf-8"));
        assert_eq!(script.body, b"document.title='x'");
        let host = serve_from(&r, "/__host/desktop-host.js", None);
        assert_eq!(host.status, 200);
        assert!(host.body.starts_with(HOST_SCRIPT.as_bytes()));
        assert!(String::from_utf8(host.body.clone()).unwrap().contains("type:'game.ready'"));
        assert!(HOST_SCRIPT.contains("xiaban.game-pack"));
        assert_eq!(serve_from(&r, "/__host/other.js", None).status, 404);
        assert_eq!(serve_from(&r, "/missing-game/index.html", None).status, 404);
        assert_eq!(serve_from(&r, "/demo-game/nope.js", None).status, 404);
        let _ = fs::remove_dir_all(r);
    }

    #[test]
    fn the_protocol_refuses_traversal_absolute_paths_and_odd_input() {
        let r = root("serve-guard");
        make(&r, "demo-game");
        fs::write(r.join("secret.txt"), "top secret").unwrap();
        fs::write(packs_dir(&r).join("outside.txt"), "sibling").unwrap();
        make(&r, "other-game");
        for path in [
            "/demo-game/../secret.txt", "/demo-game/../../secret.txt", "/demo-game/%2e%2e/secret.txt", "/demo-game/%2E%2E%2Fsecret.txt",
            "/demo-game/..%2f..%2fsecret.txt", "/demo-game/../other-game/index.html", "/demo-game//etc/passwd", "/demo-game/c:/windows/win.ini",
            "/demo-game/..\\secret.txt", "/demo-game/%5c..%5csecret.txt", "/demo-game/.hidden.js", "/demo-game/a/./b.js", "/demo-game/%00.js",
        ] {
            let served = serve_from(&r, path, None);
            assert!(matches!(served.status, 400 | 403 | 404) && served.body.is_empty(), "{path} -> {}", served.status);
        }
        assert_eq!(serve_from(&r, "/%2e%2e/secret.txt", None).status, 404);
        assert_eq!(serve_from(&r, "/demo%2fgame/index.html", None).status, 404);
        assert_eq!(serve_from(&r, "/demo-game/%ff.js", None).status, 400);
        // Builtin ids are never served from the user folder, even if one exists on disk.
        write_manifest(&r, "ink-blade", manifest("ink-blade"));
        assert_eq!(serve_from(&r, "/ink-blade/index.html", None).status, 404);
        let _ = fs::remove_dir_all(r);
    }

    #[cfg(unix)]
    #[test]
    fn the_protocol_does_not_follow_symlinks_out_of_the_game_folder() {
        let r = root("serve-symlink");
        make(&r, "demo-game");
        fs::write(r.join("secret.txt"), "top secret").unwrap();
        let dir = packs_dir(&r).join("demo-game");
        std::os::unix::fs::symlink(r.join("secret.txt"), dir.join("leak.txt")).unwrap();
        std::os::unix::fs::symlink(&r, dir.join("up")).unwrap();
        std::os::unix::fs::symlink(dir.join("game.js"), dir.join("inside.js")).unwrap();
        assert_eq!(serve_from(&r, "/demo-game/leak.txt", None).status, 403);
        assert_eq!(serve_from(&r, "/demo-game/up/secret.txt", None).status, 403);
        assert_eq!(serve_from(&r, "/demo-game/inside.js", None).status, 200, "links that stay inside are fine");
        let _ = fs::remove_dir_all(r);
    }

    #[test]
    fn the_protocol_serves_only_whitelisted_extensions_with_the_right_type() {
        let r = root("serve-ext");
        make(&r, "demo-game");
        let dir = packs_dir(&r).join("demo-game");
        for (name, mime) in [
            ("a.css", "text/css; charset=utf-8"), ("a.json", "application/json; charset=utf-8"), ("a.svg", "image/svg+xml"), ("a.png", "image/png"),
            ("a.jpg", "image/jpeg"), ("a.jpeg", "image/jpeg"), ("a.webp", "image/webp"), ("a.gif", "image/gif"), ("a.mp3", "audio/mpeg"),
            ("a.ogg", "audio/ogg"), ("a.wav", "audio/wav"), ("a.woff2", "font/woff2"), ("a.txt", "text/plain; charset=utf-8"), ("A.PNG", "image/png"),
        ] {
            fs::write(dir.join(name), "x").unwrap();
            let served = serve_from(&r, &format!("/demo-game/{name}"), None);
            assert_eq!((served.status, served.mime), (200, mime), "{name}");
        }
        for name in ["a.exe", "a.php", "a.md", "a.wasm", "a.mjs", "noext", "a.html.bak"] {
            fs::write(dir.join(name), "x").unwrap();
            assert_eq!(serve_from(&r, &format!("/demo-game/{name}"), None).status, 403, "{name}");
        }
        let _ = fs::remove_dir_all(r);
    }

    #[test]
    fn the_protocol_answers_byte_ranges_for_media() {
        let r = root("serve-range");
        make(&r, "demo-game");
        fs::write(packs_dir(&r).join("demo-game/s.mp3"), "0123456789").unwrap();
        let part = serve_from(&r, "/demo-game/s.mp3", Some("bytes=2-4"));
        assert_eq!((part.status, part.body.as_slice(), part.range.as_deref()), (206, &b"234"[..], Some("bytes 2-4/10")));
        assert_eq!(serve_from(&r, "/demo-game/s.mp3", Some("bytes=-3")).body, b"789");
        assert_eq!(serve_from(&r, "/demo-game/s.mp3", Some("bytes=7-")).body, b"789");
        assert_eq!(serve_from(&r, "/demo-game/s.mp3", Some("bytes=50-")).status, 416);
        assert_eq!(serve_from(&r, "/demo-game/s.mp3", None).body.len(), 10);
        let _ = fs::remove_dir_all(r);
    }

    #[test]
    fn the_csp_blocks_network_and_names_the_protocol_origin() {
        for windows in [false, true] {
            let policy = csp(windows);
            for needle in ["default-src 'none'", "connect-src 'none'", "frame-src 'none'", "object-src 'none'", "base-uri 'none'", "form-action 'none'", "worker-src blob:", "script-src 'self' game-pack:"] {
                assert!(policy.contains(needle), "{needle} in {policy}");
            }
            assert_eq!(policy.contains("http://game-pack.localhost"), windows);
            assert!(!policy.contains("https:") && !policy.contains("http://*"));
        }
    }

    #[test]
    fn game_create_enforces_file_count_size_path_and_type_limits() {
        let ok = |list: Vec<(String, String)>| validate_request("my-game", "名", "介绍", &list);
        assert!(ok(sample()).is_ok());
        assert!(ok(files(&[("game.js", "x")])).unwrap_err().contains("index.html"));
        assert!(ok(vec![]).is_err());
        // count
        let mut many = vec![("index.html".to_string(), PAGE.to_string())];
        for i in 0..MAX_FILES - 1 {
            many.push((format!("f{i}.js"), String::new()));
        }
        assert_eq!(many.len(), MAX_FILES);
        assert!(ok(many.clone()).is_ok(), "exactly 40 files is allowed");
        many.push(("one-too-many.js".into(), String::new()));
        assert!(ok(many).unwrap_err().contains("40"));
        // size
        let big = "x".repeat(MAX_FILE_BYTES + 1);
        assert!(ok(files(&[("index.html", big.as_str())])).unwrap_err().contains("1 MB"));
        let edge = "x".repeat(MAX_FILE_BYTES);
        assert!(ok(files(&[("index.html", edge.as_str())])).is_ok());
        let five = (0..5).map(|i| (format!("f{i}.js"), edge.clone())).chain([("index.html".to_string(), "x".into())]).collect();
        assert!(ok(five).unwrap_err().contains("4 MB"));
        // paths
        for bad in ["../x.js", "/abs.js", "a//b.js", ".hidden.js", "a/.git/x.js", "a/../b.js", "a\\b.js", "空.js", "a b.js", "a:b.js", "a/b/c/d/e.js", "", "a/"] {
            assert!(ok([sample(), files(&[(bad, "x")])].concat()).is_err(), "{bad:?}");
        }
        assert!(ok([sample(), files(&[("a/b/c/d.js", "x")])].concat()).is_ok(), "four levels are fine");
        // types
        for bad in ["x.png", "x.exe", "x.mjs", "x.wasm", "x", "x.html.exe"] {
            assert!(ok([sample(), files(&[(bad, "x")])].concat()).unwrap_err().contains("类型"), "{bad}");
        }
        for good in ["a.md", "a.txt", "a.json", "a.css", "a.svg", "A.JS"] {
            assert!(ok([sample(), files(&[(good, "x")])].concat()).is_ok(), "{good}");
        }
        // manifest.json, duplicates, file/folder clash
        assert!(ok([sample(), files(&[("manifest.json", "{}")])].concat()).unwrap_err().contains("manifest.json"));
        assert!(ok([sample(), files(&[("Manifest.JSON", "{}")])].concat()).is_err());
        assert!(ok([sample(), files(&[("sub/manifest.json", "{}")])].concat()).is_ok());
        assert!(ok([sample(), files(&[("game.js", "again")])].concat()).unwrap_err().contains("重复"));
        assert!(ok([sample(), files(&[("GAME.js", "again")])].concat()).unwrap_err().contains("重复"));
        assert!(ok([sample(), files(&[("game.js/x.js", "x")])].concat()).unwrap_err().contains("冲突"));
        // meta
        assert!(validate_request("snake-arena", "名", "介绍", &sample()).unwrap_err().contains("内置"));
        assert!(validate_request("Bad_Id", "名", "介绍", &sample()).is_err());
        assert!(validate_request("my-game", &"字".repeat(25), "介绍", &sample()).is_err());
        assert!(validate_request("my-game", "名", &"字".repeat(81), &sample()).is_err());
    }

    #[test]
    fn game_create_writes_a_loadable_folder_and_leaves_no_temporary_files() {
        let r = root("create");
        let list = [sample(), files(&[("cover.svg", "<svg xmlns=\"http://www.w3.org/2000/svg\"/>"), ("assets/level.json", "{}")])].concat();
        let made = create_pack(&r, "my-game", " 我的游戏 ", "简介", &list, true, false).unwrap();
        assert_eq!(made["replaced"], false);
        let entry = find_user_game(&r, "my-game").unwrap();
        assert_eq!((entry["name"].as_str(), entry["save"].as_bool(), entry["cover"].as_str()), (Some("我的游戏"), Some(true), Some("cover.svg")));
        assert!(packs_dir(&r).join("my-game/assets/level.json").is_file());
        let names: Vec<String> = fs::read_dir(packs_dir(&r)).unwrap().flatten().map(|e| e.file_name().into_string().unwrap()).collect();
        assert_eq!(names, ["my-game"]);
        // the finished folder plays through the protocol
        assert_eq!(serve_from(&r, "/my-game/assets/level.json", None).status, 200);
        // existing folder needs overwrite
        let again = create_pack(&r, "my-game", "新名", "简介", &sample(), false, false);
        assert!(again.unwrap_err().contains("overwrite"));
        assert_eq!(find_user_game(&r, "my-game").unwrap()["name"], "我的游戏", "refused call changed nothing");
        let _ = fs::remove_dir_all(r);
    }

    #[test]
    fn overwrite_replaces_the_folder_and_failures_keep_or_restore_the_old_one() {
        let r = root("overwrite");
        create_pack(&r, "my-game", "旧版", "简介", &files(&[("index.html", PAGE), ("old.js", "old")]), false, false).unwrap();
        // invalid request: nothing touched
        assert!(create_pack(&r, "my-game", "新版", "简介", &files(&[("game.js", "x")]), false, true).is_err());
        assert!(packs_dir(&r).join("my-game/old.js").is_file());
        // a request that clashes (file and folder with the same name) is refused before anything is touched
        let clash = files(&[("index.html", PAGE), ("a.js", "x"), ("a.js/b.js", "y")]);
        assert!(create_pack(&r, "my-game", "新版", "简介", &clash, false, true).is_err());
        assert_eq!(find_user_game(&r, "my-game").unwrap()["name"], "旧版");
        assert!(packs_dir(&r).join("my-game/old.js").is_file());
        let leftovers: Vec<String> = fs::read_dir(packs_dir(&r)).unwrap().flatten().map(|e| e.file_name().into_string().unwrap()).collect();
        assert_eq!(leftovers, ["my-game"], "temporary and backup folders are cleaned up");
        // success replaces everything
        let made = create_pack(&r, "my-game", "新版", "简介", &files(&[("index.html", PAGE), ("new.js", "new")]), true, true).unwrap();
        assert_eq!(made["replaced"], true);
        assert!(!packs_dir(&r).join("my-game/old.js").exists() && packs_dir(&r).join("my-game/new.js").is_file());
        assert_eq!(find_user_game(&r, "my-game").unwrap()["name"], "新版");
        let leftovers: Vec<String> = fs::read_dir(packs_dir(&r)).unwrap().flatten().map(|e| e.file_name().into_string().unwrap()).collect();
        assert_eq!(leftovers, ["my-game"]);
        let _ = fs::remove_dir_all(r);
    }

    #[test]
    fn a_failed_swap_puts_the_old_folder_back() {
        let r = root("restore");
        make(&r, "my-game");
        let packs = packs_dir(&r);
        let (target, backup) = (packs.join("my-game"), packs.join(".bak-my-game-test"));
        // The finished folder is missing, so the final rename fails after the old one was set aside.
        let error = swap_in(&packs.join(".tmp-missing"), &target, &backup, true).unwrap_err();
        assert!(error.contains("无法保存"));
        assert!(find_user_game(&r, "my-game").is_ok(), "old game is back");
        assert!(!backup.exists());
        // And the happy path leaves no backup behind.
        let fresh = packs.join(".tmp-fresh");
        fs::create_dir_all(&fresh).unwrap();
        fs::write(fresh.join("marker.txt"), "new").unwrap();
        swap_in(&fresh, &target, &backup, true).unwrap();
        assert!(target.join("marker.txt").is_file() && !backup.exists() && !fresh.exists());
        let _ = fs::remove_dir_all(r);
    }

    #[test]
    fn removing_a_game_deletes_its_folder_and_save_but_not_builtin_or_missing_ones() {
        let r = root("remove");
        make(&r, "my-game");
        fs::create_dir_all(save_dir(&r)).unwrap();
        fs::write(save_dir(&r).join(user_save_name("my-game")), "{}").unwrap();
        fs::write(save_dir(&r).join("ink-blade.json"), "{}").unwrap();
        remove_pack(&r, "my-game").unwrap();
        assert!(!packs_dir(&r).join("my-game").exists());
        assert!(!save_dir(&r).join("user-my-game.json").exists());
        assert!(save_dir(&r).join("ink-blade.json").exists(), "builtin saves are untouched");
        assert!(remove_pack(&r, "my-game").unwrap_err().contains("没有找到"));
        assert!(remove_pack(&r, "ink-blade").unwrap_err().contains("内置"));
        assert!(remove_pack(&r, "../x").is_err());
        let _ = fs::remove_dir_all(r);
    }
}
