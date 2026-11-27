//! Install the pinned native engine only when a desktop feature needs it.
use crate::{
    app_runtime::pack,
    library::{self, Resource},
};
use agent_computer_use_tools::{executable, runtime_target, RUNTIME_VERSION};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    fs,
    io::{Cursor, Read},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Mutex, OnceLock,
    },
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Runtime {
    resource: Resource,
    tree_sha256: String,
}

#[derive(Deserialize)]
struct Distribution {
    version: String,
    targets: HashMap<String, Runtime>,
}

static DISTRIBUTION: OnceLock<Distribution> = OnceLock::new();
static INSTALL: Mutex<()> = Mutex::new(());
static STAGING: AtomicU64 = AtomicU64::new(0);

fn runtime() -> Result<&'static Runtime, String> {
    let distribution = DISTRIBUTION.get_or_init(|| {
        serde_json::from_str(include_str!("computer-runtime.json"))
            .expect("valid computer control distribution manifest")
    });
    if distribution.version != RUNTIME_VERSION {
        return Err("电脑控制引擎的分发版本与客户端不匹配".into());
    }
    distribution
        .targets
        .get(&runtime_target())
        .ok_or_else(|| "当前平台没有可下载的电脑控制引擎".into())
}

fn base(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(crate::storage::root(app)?.join("agent-tools/computer-use"))
}

fn target(base: &Path) -> PathBuf {
    base.join(RUNTIME_VERSION).join(runtime_target())
}

pub fn location(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(target(&base(app)?))
}

pub fn ensure(app: &tauri::AppHandle, cancel: &AtomicBool) -> Result<PathBuf, String> {
    install(&base(app)?, runtime()?, cancel)
}

fn valid(dir: &Path, runtime: &Runtime) -> bool {
    dir.symlink_metadata().is_ok_and(|entry| entry.is_dir())
        && pack::tree_sha256(dir, None).is_ok_and(|sha| sha == runtime.tree_sha256)
        && executable(dir).is_ok()
}

fn install(base: &Path, runtime: &Runtime, cancel: &AtomicBool) -> Result<PathBuf, String> {
    let _guard = INSTALL.lock().map_err(|_| "电脑控制引擎缓存暂不可用")?;
    if cancel.load(Ordering::Relaxed) {
        return Err(library::CANCELLED.into());
    }
    let dest = target(base);
    if valid(&dest, runtime) {
        return Ok(dest);
    }

    let resource = &runtime.resource;
    library::download_cancellable(base, resource, cancel, |_| {})
        .map_err(|error| format!("电脑控制引擎下载失败：{error}"))?;
    let file =
        fs::File::open(base.join("cache").join(&resource.sha256)).map_err(|e| e.to_string())?;
    let mut bytes = Vec::new();
    file.take(resource.bytes + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 != resource.bytes
        || format!("{:x}", Sha256::digest(&bytes)) != resource.sha256
    {
        return Err("电脑控制引擎压缩包校验失败".into());
    }
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|e| e.to_string())?;
    let parent = dest.parent().ok_or("电脑控制引擎安装目录无效")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let staging = parent.join(format!(
        ".install-{}-{}",
        std::process::id(),
        STAGING.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&staging).map_err(|e| e.to_string())?;
    let result = (|| {
        archive.extract(&staging).map_err(|e| e.to_string())?;
        if !valid(&staging, runtime) {
            return Err("电脑控制引擎解包校验失败".into());
        }
        if cancel.load(Ordering::Relaxed) {
            return Err(library::CANCELLED.into());
        }
        match dest.symlink_metadata() {
            Ok(entry) if entry.is_dir() => fs::remove_dir_all(&dest).map_err(|e| e.to_string())?,
            Ok(_) => fs::remove_file(&dest).map_err(|e| e.to_string())?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(e) => return Err(e.to_string()),
        }
        fs::rename(&staging, &dest).map_err(|e| e.to_string())?;
        Ok(dest)
    })();
    let _ = fs::remove_dir_all(staging);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn verified_runtime_repairs_from_cached_archive_and_then_runs_without_it() {
        let base = std::env::temp_dir().join(format!(
            "xiaoban-computer-runtime-{}-{}",
            std::process::id(),
            STAGING.fetch_add(1, Ordering::Relaxed)
        ));
        let source = base.join("source");
        let program = match std::env::consts::OS {
            "macos" => "Open Computer Use.app/Contents/MacOS/OpenComputerUse",
            "windows" => "open-computer-use.exe",
            _ => "open-computer-use",
        };
        let metadata =
            serde_json::json!({"version": RUNTIME_VERSION, "target": runtime_target()}).to_string();
        let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
        for (path, bytes) in [
            ("runtime.json", metadata.as_bytes()),
            (program, b"pinned test binary"),
        ] {
            let file = source.join(path);
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(file, bytes).unwrap();
            zip.start_file(
                path,
                zip::write::SimpleFileOptions::default().unix_permissions(0o755),
            )
            .unwrap();
            zip.write_all(bytes).unwrap();
        }
        let bytes = zip.finish().unwrap().into_inner();
        let resource = Resource {
            url: "http://localhost:1/engine.zip".into(),
            sha256: format!("{:x}", Sha256::digest(&bytes)),
            bytes: bytes.len() as u64,
            mime: "application/octet-stream".into(),
        };
        let archive = base.join("cache").join(&resource.sha256);
        fs::create_dir_all(archive.parent().unwrap()).unwrap();
        fs::write(&archive, &bytes).unwrap();
        let runtime = Runtime {
            resource,
            tree_sha256: pack::tree_sha256(&source, None).unwrap(),
        };
        let cancel = AtomicBool::new(false);

        let dir = install(&base, &runtime, &cancel).unwrap();
        fs::write(dir.join(program), b"edited test binary").unwrap();
        assert_eq!(install(&base, &runtime, &cancel).unwrap(), dir);
        assert_eq!(fs::read(dir.join(program)).unwrap(), b"pinned test binary");
        fs::remove_file(archive).unwrap();
        assert_eq!(install(&base, &runtime, &cancel).unwrap(), dir);
        fs::remove_dir_all(base).unwrap();
    }

    #[test]
    #[ignore = "downloads the published native engine"]
    fn published_runtime_downloads_and_keeps_its_native_signature() {
        let base = std::env::temp_dir().join(format!(
            "xiaoban-published-runtime-{}-{}",
            std::process::id(),
            STAGING.fetch_add(1, Ordering::Relaxed)
        ));
        let runtime = runtime().unwrap();
        let cancel = AtomicBool::new(false);
        let dir = install(&base, runtime, &cancel).unwrap();
        #[cfg(target_os = "macos")]
        assert!(std::process::Command::new("/usr/bin/codesign")
            .args(["--verify", "--deep", "--strict",
                "-R=identifier \"com.ifuryst.opencomputeruse\" and certificate root = H\"e8b36162292f910859dfdfd66d81d8d265764779\""])
            .arg(dir.join("Open Computer Use.app")).status().unwrap().success());

        fs::remove_file(base.join("cache").join(&runtime.resource.sha256)).unwrap();
        let offline = Runtime {
            resource: Resource {
                url: "http://localhost:1/engine.zip".into(),
                ..runtime.resource.clone()
            },
            tree_sha256: runtime.tree_sha256.clone(),
        };
        assert_eq!(install(&base, &offline, &cancel).unwrap(), dir);
        fs::remove_dir_all(base).unwrap();
    }
}
