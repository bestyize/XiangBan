//! Verified, content-addressed official game and app packages.
use crate::{app_runtime::pack, library::{self, Resource}};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{collections::HashSet, fs, io::{Read, Write}, path::{Path, PathBuf}, sync::{atomic::{AtomicU64, Ordering}, Mutex, OnceLock}, time::Duration};

#[derive(Clone, Deserialize)]
struct Game {
    id: String,
    #[serde(rename = "packId")]
    pack_id: Option<String>,
    kind: String,
    url: String,
    sha256: String,
    bytes: u64,
    #[serde(rename = "sourceSha256")]
    source_sha256: String,
}
impl Game {
    fn pack_id(&self) -> &str { self.pack_id.as_deref().unwrap_or(&self.id) }

    fn matches_manifest(&self, manifest: &Value) -> bool {
        manifest["id"] == self.pack_id() && manifest["kind"] == self.kind
    }
}
#[derive(Deserialize)]
struct Distribution { version: u32, games: Vec<Game> }

fn distribution() -> Result<Vec<Game>, String> {
    let mut data: Distribution = serde_json::from_str(include_str!("../../../../assets/game-packs/distribution.json")).map_err(|e| e.to_string())?;
    let ids: HashSet<_> = data.games.iter().map(|game| game.id.as_str()).collect();
    if data.version != 1 || data.games.len() != crate::games::IDS.len() || ids != crate::games::IDS.into_iter().collect() {
        return Err("内置游戏目录不一致".into());
    }
    // Keep the original package IDs available for users who import the same ZIP.
    let featured = [
        (include_str!("featured-aquarium.json"), "app"),
        (include_str!("featured-whale-girl.json"), "game"),
    ];
    for ((source, kind), id) in featured.into_iter().zip(crate::games::FEATURED_PACK_IDS) {
        let game: Game = serde_json::from_str(source).map_err(|e| e.to_string())?;
        if game.id != id || game.kind != kind {
            return Err(format!("精选作品 {id} 的分发清单无效"));
        }
        data.games.push(game);
    }
    for game in &data.games {
        let md5 = game.url.strip_prefix("https://res.xiangban.xyz/files/").and_then(|url| url.strip_suffix(".zip"));
        if !md5.is_some_and(|hash| hash.len() == 32 && hash.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)))
            || !pack::valid_id(game.pack_id()) || !library::hash_name(&game.sha256) || !library::hash_name(&game.source_sha256) || !(1..=pack::MAX_PACK_BYTES).contains(&game.bytes) || !["game", "app"].contains(&game.kind.as_str()) {
            return Err("内置游戏资源信息无效".into());
        }
    }
    Ok(data.games)
}

fn game(id: &str) -> Result<Game, String> {
    distribution()?.into_iter().find(|game| game.id == id).ok_or_else(|| "没有找到内置游戏".into())
}

fn target(root: &Path, game: &Game) -> PathBuf { root.join("games/builtin").join(format!("{}-{}", game.id, game.sha256)) }

pub fn cached_dir(root: &Path, id: &str) -> Option<PathBuf> {
    let game = game(id).ok()?;
    let path = target(root, &game);
    if fs::read_to_string(path.join(".complete")).ok()?.trim() != game.sha256 { return None; }
    let manifest: Value = serde_json::from_slice(&fs::read(path.join("manifest.json")).ok()?).ok()?;
    if !game.matches_manifest(&manifest) { return None; }
    let paths = pack::check_manifest(&manifest).ok()?;
    if paths.iter().any(|item| !path.join(item).is_file()) { return None; }
    Some(path)
}

static LOCKS: OnceLock<Vec<Mutex<()>>> = OnceLock::new();
static STAGING: AtomicU64 = AtomicU64::new(0);

fn verify_cached(dir: &Path, game: &Game) -> Result<bool, String> {
    fn collect(base: &Path, dir: &Path, files: &mut Vec<(String, PathBuf)>) -> Result<(), String> {
        for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            let kind = entry.file_type().map_err(|e| e.to_string())?;
            if kind.is_symlink() { return Err("游戏缓存包含符号链接".into()); }
            if kind.is_dir() { collect(base, &path, files)?; }
            else if kind.is_file() {
                let name = path.strip_prefix(base).map_err(|e| e.to_string())?.to_str().ok_or("游戏缓存路径无效")?.replace('\\', "/");
                if name != ".complete" { files.push((name, path)); }
                if files.len() > pack::MAX_FILES { return Err("游戏缓存文件过多".into()); }
            } else { return Err("游戏缓存包含不支持的文件".into()); }
        }
        Ok(())
    }
    let mut files = Vec::new();
    collect(dir, dir, &mut files)?;
    files.sort_by(|a, b| a.0.cmp(&b.0));
    let mut tree = Sha256::new();
    let mut total = 0u64;
    for (name, path) in files {
        let size = fs::metadata(&path).map_err(|e| e.to_string())?.len();
        total = total.saturating_add(size);
        if size > pack::MAX_FILE_BYTES || total > pack::MAX_PACK_BYTES { return Ok(false); }
        let mut file = fs::File::open(path).map_err(|e| e.to_string())?;
        let mut digest = Sha256::new();
        let mut buffer = [0u8; 65536];
        loop {
            let read = file.read(&mut buffer).map_err(|e| e.to_string())?;
            if read == 0 { break; }
            digest.update(&buffer[..read]);
        }
        tree.update(name.as_bytes());
        tree.update([0]);
        tree.update(digest.finalize());
    }
    Ok(format!("{:x}", tree.finalize()) == game.source_sha256)
}

pub fn ensure(root: &Path, id: &str) -> Result<PathBuf, String> {
    let games = distribution()?;
    let index = games.iter().position(|item| item.id == id).ok_or("没有找到内置游戏")?;
    let locks = LOCKS.get_or_init(|| games.iter().map(|_| Mutex::new(())).collect());
    let _lock = locks[index].lock().map_err(|_| "游戏缓存暂不可用")?;
    let game = &games[index];
    if let Some(cached) = cached_dir(root, id) {
        if verify_cached(&cached, game).unwrap_or(false) { return Ok(cached); }
    }
    let resource = Resource { url: game.url.clone(), sha256: game.sha256.clone(), bytes: game.bytes, mime: "application/octet-stream".into() };
    let archive = root.join("cache").join(&game.sha256);
    if archive.exists() {
        let valid = (|| -> Result<bool, String> {
            let mut file = fs::File::open(&archive).map_err(|e| e.to_string())?;
            if file.metadata().map_err(|e| e.to_string())?.len() != game.bytes { return Ok(false); }
            let mut digest = Sha256::new();
            let mut buffer = [0u8; 65536];
            loop {
                let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
                if n == 0 { break; }
                digest.update(&buffer[..n]);
            }
            Ok(format!("{:x}", digest.finalize()) == game.sha256)
        })().unwrap_or(false);
        // Windows cannot atomically rename over an existing corrupt file.
        if !valid { fs::remove_file(&archive).map_err(|e| e.to_string())?; }
    }
    library::download(root, &resource, |_| {})?;
    let parent = root.join("games/builtin");
    fs::create_dir_all(&parent).map_err(|e| e.to_string())?;
    let staging = parent.join(format!(".unpack-{}-{}-{}", id, std::process::id(), STAGING.fetch_add(1, Ordering::Relaxed)));
    fs::create_dir(&staging).map_err(|e| e.to_string())?;
    let result = (|| {
        extract(&archive, &staging, game)?;
        fs::write(staging.join(".complete"), &game.sha256).map_err(|e| e.to_string())?;
        let dest = target(root, game);
        if dest.exists() { fs::remove_dir_all(&dest).map_err(|e| e.to_string())?; }
        fs::rename(&staging, &dest).map_err(|e| e.to_string())?;
        if let Ok(entries) = fs::read_dir(&parent) {
            let prefix = format!("{}-", game.id);
            for entry in entries.flatten() {
                if entry.path() != dest && entry.file_name().to_string_lossy().starts_with(&prefix) && entry.path().is_dir() {
                    let _ = fs::remove_dir_all(entry.path());
                }
            }
        }
        Ok(dest)
    })();
    let _ = fs::remove_dir_all(&staging);
    result
}

fn extract(archive_path: &Path, staging: &Path, game: &Game) -> Result<(), String> {
    let file = fs::File::open(archive_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|_| "游戏压缩包无效")?;
    if archive.is_empty() || archive.len() > pack::MAX_FILES { return Err("游戏文件数量无效".into()); }
    let mut manifest = None;
    let mut files = Vec::new();
    let mut total = 0u64;
    for i in 0..archive.len() {
        let mut item = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = item.name().map_err(|_| "游戏文件名无效")?.into_owned();
        if item.is_dir() || item.unix_mode().is_some_and(|mode| mode & 0o170000 != 0 && mode & 0o170000 != 0o100000) {
            return Err("游戏包包含不支持的文件".into());
        }
        if name != "manifest.json" { pack::check_file_path(&name)?; }
        let limit = if name == "manifest.json" { pack::MAX_MANIFEST_BYTES } else { pack::MAX_FILE_BYTES };
        if item.size() > limit || total.saturating_add(item.size()) > pack::MAX_PACK_BYTES { return Err("游戏包解压大小超出限制".into()); }
        let mut bytes = Vec::new();
        (&mut item).take(limit + 1).read_to_end(&mut bytes).map_err(|e| e.to_string())?;
        if bytes.len() as u64 != item.size() { return Err("游戏文件长度不匹配".into()); }
        total += bytes.len() as u64;
        if name == "manifest.json" {
            let value = serde_json::from_slice::<Value>(&bytes).map_err(|_| "游戏清单无效")?;
            if manifest.replace((value, bytes)).is_some() { return Err("游戏清单重复".into()); }
        } else { files.push((name, bytes)); }
    }
    let (manifest, manifest_bytes) = manifest.ok_or("游戏缺少清单")?;
    if !game.matches_manifest(&manifest) { return Err("游戏包标识不匹配".into()); }
    pack::validate(&manifest, &files)?;
    fs::write(staging.join("manifest.json"), manifest_bytes).map_err(|e| e.to_string())?;
    for (name, bytes) in files {
        let path = staging.join(name);
        fs::create_dir_all(path.parent().ok_or("游戏目录无效")?).map_err(|e| e.to_string())?;
        fs::File::create(path).and_then(|mut file| file.write_all(&bytes)).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub async fn ensure_async(app: &tauri::AppHandle, id: &str) -> Result<(), String> {
    let root = crate::storage::root(app)?;
    let id = id.to_string();
    tauri::async_runtime::spawn_blocking(move || ensure(&root, &id)).await.map_err(|e| e.to_string())??;
    Ok(())
}

pub fn start(app: &tauri::AppHandle) {
    let app = app.clone();
    std::thread::spawn(move || {
        let mut pending = crate::games::IDS.to_vec();
        for delay in [0, 20, 60, 300, 900] {
            if delay > 0 { std::thread::sleep(Duration::from_secs(delay)); }
            if let Ok(root) = crate::storage::root(&app) { pending.retain(|id| ensure(&root, id).is_err()); }
            if pending.is_empty() { break; }
        }
    });
}

#[cfg(debug_assertions)]
pub async fn smoke(app: &tauri::AppHandle) -> Result<Vec<String>, String> {
    use tauri::Manager;
    let mut ready = Vec::new();
    for game in crate::games::builtin_catalog()? {
        let id = game["id"].as_str().ok_or("游戏编号无效")?;
        crate::app_runtime::window::open_app(app, id, game["name"].as_str().unwrap_or(""), game["orientation"].as_str(), game["entry"].as_str()).await?;
        let deadline = std::time::Instant::now() + Duration::from_secs(30);
        loop {
            if app.state::<crate::app_runtime::AppRuntimeState>().smoke_ready.lock().map_err(|e| e.to_string())?.contains(id) { break; }
            if std::time::Instant::now() > deadline { return Err(format!("{id} 未收到就绪信号")); }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        ready.push(id.to_string());
        crate::app_runtime::window::close_app(app, id)?;
    }
    Ok(ready)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn catalog_is_pinned_and_covers_every_builtin() {
        let games = distribution().unwrap();
        assert_eq!(games.len(), crate::games::IDS.len() + crate::games::FEATURED_PACK_IDS.len());
        assert!(games.iter().all(|item| item.url.starts_with("https://res.xiangban.xyz/files/") && item.bytes > 0));
        let aquarium = games.iter().find(|item| item.id == crate::games::FEATURED_AQUARIUM).unwrap();
        assert_eq!(aquarium.pack_id(), "aquarium");
        assert!(!crate::game_packs::is_builtin(aquarium.pack_id()));
        let whale_girl = games.iter().find(|item| item.id == crate::games::FEATURED_WHALE_GIRL).unwrap();
        assert_eq!(whale_girl.pack_id(), "whale-girl");
        assert_eq!(whale_girl.kind, "game");
        assert!(!crate::game_packs::is_builtin(whale_girl.pack_id()));
    }

    #[test]
    fn cached_packages_are_revalidated_and_can_be_repaired() {
        let Some(bundle_dir) = std::env::var_os("XIAOBAN_BUILTIN_LOCAL_TEST") else { return; };
        let root = std::env::temp_dir().join(format!("xiaoban-builtin-repair-{}", std::process::id()));
        fs::create_dir_all(root.join("cache")).unwrap();
        for game in distribution().unwrap() {
            let filename = game.url.rsplit('/').next().unwrap();
            let archive = root.join("cache").join(&game.sha256);
            fs::copy(Path::new(&bundle_dir).join(filename), &archive).unwrap();
            let cached = ensure(&root, &game.id).unwrap();
            assert!(verify_cached(&cached, &game).unwrap());
            if game.id == "falling-blocks" {
                fs::write(cached.join("index.html"), b"corrupt").unwrap();
                assert_eq!(ensure(&root, &game.id).unwrap(), cached);
                assert!(verify_cached(&cached, &game).unwrap());
                fs::write(cached.join("index.html"), b"corrupt again").unwrap();
                fs::write(&archive, b"corrupt archive").unwrap();
                assert_eq!(ensure(&root, &game.id).unwrap(), cached);
                assert!(verify_cached(&cached, &game).unwrap());
            }
            fs::remove_file(archive).unwrap();
            assert_eq!(ensure(&root, &game.id).unwrap(), cached);
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn official_packages_download_and_serve_from_offline_cache() {
        if std::env::var_os("XIAOBAN_BUILTIN_LIVE_TEST").is_none() { return; }
        let root = std::env::temp_dir().join(format!("xiaoban-builtin-live-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        for id in crate::games::IDS {
            let cached = ensure(&root, id).unwrap_or_else(|e| panic!("{id}: {e}"));
            assert_eq!(cached_dir(&root, id).as_deref(), Some(cached.as_path()));
            let modern = crate::app_runtime::protocol::serve_for(&root, &format!("app-runtime-{id}"), &format!("/{id}/index.html"), None);
            assert_eq!(modern.status, 200, "{id} V2");
            assert!(!modern.body.is_empty());
            let legacy = crate::game_packs::serve_from(&root, &format!("/{id}/index.html"), None);
            assert_eq!(legacy.status, 200, "{id} legacy host");
            if id == "falling-blocks" {
                fs::write(cached.join("index.html"), b"corrupt").unwrap();
                assert_eq!(ensure(&root, id).unwrap(), cached);
                assert_eq!(crate::app_runtime::protocol::serve_for(&root, &format!("app-runtime-{id}"), &format!("/{id}/index.html"), None).body, modern.body);
            }
            let zip = root.join("cache").join(game(id).unwrap().sha256);
            fs::remove_file(zip).unwrap();
            assert_eq!(ensure(&root, id).unwrap(), cached, "{id} offline cache");
        }
        fs::remove_dir_all(root).unwrap();
    }
}
