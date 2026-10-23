//! The local inference engine, `xiaoban-infer` (source: `tools/xiaoban-infer`). It runs ONNX models on the CPU through
//! the onnxruntime that sherpa-onnx links into it; one process serves one capability (`asr`, `turn`, …), so a slow
//! request of one never holds up another. This module fetches the engine and the model files a capability needs,
//! checks them, starts a process for a capability and exchanges its line-delimited JSON messages.
//!
//! A new capability needs a module in the engine, its model list, and a caller here that starts it by name; fetching,
//! checking and process handling stay as they are.
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    time::{Duration, Instant},
};

/// Where the engine and the models are fetched from (a ModelScope repository).
pub const SOURCE: &str = "https://modelscope.cn/api/v1/models/bestyize/xiaoban-asr/repo?Revision=master&FilePath=";
/// A second place for the engine itself, used when ModelScope cannot be reached.
pub const ENGINE_FALLBACK: &str = "https://wallpaper.xiangban.xyz/app-packages/";
#[cfg(windows)]
pub const ENGINE_NAME: &str = "xiaoban-infer.exe";
#[cfg(not(windows))]
pub const ENGINE_NAME: &str = "xiaoban-infer-macos-arm64";

/// A model file, pinned by size and SHA-256.
pub struct ModelFile { pub name: &'static str, pub bytes: u64, pub sha256: &'static str }

/// The folder that holds the engine and every model, in the app's data directory.
pub fn directory(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(crate::storage::root(app)?.join("inference"))
}

/// The engine, in the folder or (for development) next to the app.
pub fn engine(directory: &Path) -> Option<PathBuf> {
    let beside_app = std::env::current_exe().ok().and_then(|exe| exe.parent().map(|dir| dir.join(ENGINE_NAME)));
    [Some(directory.join(ENGINE_NAME)), beside_app].into_iter().flatten().find(|path| path.is_file())
}

/// The engine and these models are on the disk (sizes only; contents are checked when fetching).
pub fn present(directory: &Path, files: &[&ModelFile]) -> bool {
    files.iter().all(|file| fs::metadata(directory.join(file.name)).is_ok_and(|m| m.is_file() && m.len() == file.bytes))
        && engine(directory).is_some()
}

/// Fetches whichever of `files` are missing or damaged, then the engine if it is missing. `progress` gets the file
/// being fetched and the bytes done so far across all of `files`; `check` can stop the work between chunks.
pub fn ensure(directory: &Path, files: &[&ModelFile], check: impl Fn() -> Result<(), String>, progress: impl Fn(&str, u64)) -> Result<(), String> {
    fs::create_dir_all(directory).map_err(|e| format!("无法创建模型目录：{e}"))?;
    let client = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36 XiaoBan/1.0")
        .https_only(true).connect_timeout(Duration::from_secs(15))
        .timeout(Duration::from_secs(600)).build().map_err(|e| e.to_string())?;
    let mut completed = 0;
    for spec in files { fetch_model(&client, directory, spec, &check, &progress, &mut completed)?; }
    if engine(directory).is_none() { fetch_engine(&client, directory, &check, &progress, completed)?; }
    Ok(())
}

/// A file that was fully hashed once is remembered by its size and modification time, so the next start does not read
/// 239 MB again to learn the same thing. Any change to the file, or no record at all, means it is hashed again.
fn stamp_path(path: &Path) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".verified");
    path.with_file_name(name)
}
fn stamp_of(path: &Path, spec: &ModelFile) -> Option<String> {
    let meta = fs::metadata(path).ok()?;
    let modified = meta.modified().ok()?.duration_since(std::time::UNIX_EPOCH).ok()?;
    Some(format!("{} {} {}.{:09} {}", spec.name, meta.len(), modified.as_secs(), modified.subsec_nanos(), spec.sha256))
}
fn remember_verified(path: &Path, spec: &ModelFile) {
    if let Some(stamp) = stamp_of(path, spec) { let _ = fs::write(stamp_path(path), stamp); }
}
fn stamped(path: &Path, spec: &ModelFile) -> bool {
    stamp_of(path, spec).is_some_and(|now| fs::read_to_string(stamp_path(path)).is_ok_and(|saved| saved == now))
}
fn verified(path: &Path, spec: &ModelFile, check: &impl Fn() -> Result<(), String>) -> Result<bool, String> {
    if !fs::metadata(path).is_ok_and(|m| m.is_file() && m.len() == spec.bytes) { return Ok(false); }
    if stamped(path, spec) { return Ok(true); }
    let mut file = fs::File::open(path).map_err(|e| format!("模型读取失败：{e}"))?;
    let mut digest = Sha256::new();
    let mut buffer = [0; 65536];
    loop { check()?; let n = file.read(&mut buffer).map_err(|e| e.to_string())?; if n == 0 { break; } digest.update(&buffer[..n]); }
    let good = format!("{:x}", digest.finalize()) == spec.sha256;
    if good { remember_verified(path, spec); }
    Ok(good)
}

fn fetch_model(client: &reqwest::blocking::Client, directory: &Path, spec: &ModelFile, check: &impl Fn() -> Result<(), String>, progress: &impl Fn(&str, u64), completed: &mut u64) -> Result<(), String> {
    check()?;
    let target = directory.join(spec.name);
    if verified(&target, spec, check)? { *completed += spec.bytes; progress(spec.name, *completed); return Ok(()); }
    let temporary = directory.join(format!("{}.part", spec.name));
    let result: Result<(), String> = (|| {
        progress(spec.name, *completed);
        let mut response = client.get(format!("{SOURCE}{}", spec.name)).send()
            .and_then(reqwest::blocking::Response::error_for_status).map_err(|e| format!("魔搭模型下载失败，请检查网络后重试：{e}"))?;
        if response.content_length().is_some_and(|n| n != spec.bytes) { return Err("魔搭返回的模型大小不匹配，请稍后重试".into()); }
        let mut file = fs::File::create(&temporary).map_err(|e| format!("模型文件无法写入：{e}"))?;
        let mut received = 0;
        let mut digest = Sha256::new();
        let mut buffer = [0; 65536];
        let mut last_event = Instant::now();
        loop {
            check()?;
            let n = response.read(&mut buffer).map_err(|e| format!("模型下载中断，请重试：{e}"))?;
            if n == 0 { break; }
            received += n as u64;
            if received > spec.bytes { return Err("下载的模型文件过大".into()); }
            digest.update(&buffer[..n]);
            file.write_all(&buffer[..n]).map_err(|e| format!("模型写入失败，请检查剩余空间：{e}"))?;
            if last_event.elapsed() >= Duration::from_millis(150) { progress(spec.name, *completed + received); last_event = Instant::now(); }
        }
        if received != spec.bytes || format!("{:x}", digest.finalize()) != spec.sha256 { return Err("模型完整性校验失败，请重试".into()); }
        file.sync_all().map_err(|e| e.to_string())?;
        drop(file);
        check()?;
        // Windows rename cannot replace an existing, damaged file.
        if target.exists() { fs::remove_file(&target).map_err(|e| e.to_string())?; }
        fs::rename(&temporary, &target).map_err(|e| format!("模型保存失败：{e}"))?;
        remember_verified(&target, spec);
        Ok(())
    })();
    if result.is_err() { let _ = fs::remove_file(&temporary); }
    result?;
    *completed += spec.bytes;
    progress(spec.name, *completed);
    Ok(())
}

fn fetch_engine(client: &reqwest::blocking::Client, directory: &Path, check: &impl Fn() -> Result<(), String>, progress: &impl Fn(&str, u64), completed: u64) -> Result<(), String> {
    let target = directory.join(ENGINE_NAME);
    let temporary = directory.join(format!("{ENGINE_NAME}.part"));
    let result: Result<(), String> = (|| {
        progress(ENGINE_NAME, completed);
        let mut response = match client.get(format!("{SOURCE}{ENGINE_NAME}")).send().and_then(|r| r.error_for_status()) {
            Ok(response) => response,
            Err(error) => {
                eprintln!("从魔搭下载推理引擎失败，尝试备用源：{error}");
                client.get(format!("{ENGINE_FALLBACK}{ENGINE_NAME}")).send()
                    .and_then(reqwest::blocking::Response::error_for_status)
                    .map_err(|e| format!("推理引擎下载失败：{e}"))?
            }
        };
        let mut file = fs::File::create(&temporary).map_err(|e| format!("推理引擎写入失败：{e}"))?;
        let mut buffer = [0; 65536];
        loop {
            check()?;
            let n = response.read(&mut buffer).map_err(|e| format!("推理引擎下载中断：{e}"))?;
            if n == 0 { break; }
            file.write_all(&buffer[..n]).map_err(|e| format!("推理引擎写入失败：{e}"))?;
        }
        file.sync_all().map_err(|e| e.to_string())?;
        drop(file);
        if target.exists() { fs::remove_file(&target).map_err(|e| e.to_string())?; }
        fs::rename(&temporary, &target).map_err(|e| format!("推理引擎保存失败：{e}"))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&target, fs::Permissions::from_mode(0o755));
        }
        Ok(())
    })();
    if result.is_err() { let _ = fs::remove_file(&temporary); }
    result
}

/// One engine process serving one capability.
pub struct Engine { child: Child, stdin: ChildStdin, stdout: BufReader<ChildStdout> }

impl Engine {
    /// Starts the engine for `capability` with the models in `directory`, and waits until it has loaded them.
    pub fn start(directory: &Path, capability: &str) -> Result<Self, String> {
        let path = engine(directory).ok_or("推理引擎尚未下载")?;
        let mut command = Command::new(&path);
        command.arg("--capability").arg(capability).arg("--dir").arg(directory)
            .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000); // No console window.
        }
        let mut child = command.spawn().map_err(|e| format!("启动推理引擎失败：{e}"))?;
        let stdin = child.stdin.take().ok_or("无法连接推理引擎")?;
        let stdout = BufReader::new(child.stdout.take().ok_or("无法连接推理引擎")?);
        let mut engine = Self { child, stdin, stdout };
        let ready = engine.read()?;
        let serves = ready.get("capabilities").and_then(|c| c.as_array()).is_some_and(|c| c.iter().any(|v| v == capability));
        if ready.get("status").and_then(|s| s.as_str()) != Some("ready") || !serves {
            return Err(format!("推理引擎无法提供「{capability}」"));
        }
        Ok(engine)
    }

    /// Sends one request and returns its response; an engine-side error comes back as `Err` with its message.
    pub fn request(&mut self, request: &serde_json::Value) -> Result<serde_json::Value, String> {
        writeln!(self.stdin, "{request}").map_err(|e| format!("推理引擎通信失败：{e}"))?;
        self.stdin.flush().map_err(|e| format!("推理引擎通信失败：{e}"))?;
        self.read()
    }

    fn read(&mut self) -> Result<serde_json::Value, String> {
        let mut line = String::new();
        if self.stdout.read_line(&mut line).map_err(|e| format!("推理引擎通信失败：{e}"))? == 0 { return Err("推理引擎已退出".into()); }
        let response: serde_json::Value = serde_json::from_str(line.trim()).map_err(|e| format!("推理引擎响应无法解析：{e}"))?;
        if response.get("status").and_then(|s| s.as_str()) == Some("error") {
            return Err(response.get("error").and_then(|e| e.as_str()).unwrap_or("推理引擎出错").to_string());
        }
        Ok(response)
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        let _ = writeln!(self.stdin, "{{\"type\":\"exit\"}}");
        let _ = self.stdin.flush();
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_verified_file_is_not_hashed_again_until_it_changes() {
        let dir = std::env::temp_dir().join(format!("xiaoban-infer-stamp-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("model.bin");
        fs::write(&path, b"abc").unwrap();
        let spec = ModelFile { name: "model.bin", bytes: 3, sha256: "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad" };
        let reads = std::cell::Cell::new(0);
        let counting = || { reads.set(reads.get() + 1); Ok(()) };
        assert!(verified(&path, &spec, &counting).unwrap());
        let first = reads.get();
        assert!(first > 0, "the first check reads the file");
        assert!(verified(&path, &spec, &counting).unwrap());
        assert_eq!(reads.get(), first, "the second check trusts the record");
        std::thread::sleep(std::time::Duration::from_millis(20));
        fs::write(&path, b"abd").unwrap();
        assert!(!verified(&path, &spec, &counting).unwrap());
        assert!(reads.get() > first);
        fs::write(stamp_path(&path), "garbage").unwrap();
        assert!(!verified(&path, &spec, &counting).unwrap());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn present_needs_every_model_and_the_engine() {
        let dir = std::env::temp_dir().join(format!("xiaoban-infer-present-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let file = ModelFile { name: "a.onnx", bytes: 3, sha256: "" };
        let files = [&file];
        fs::write(dir.join("a.onnx"), b"abc").unwrap();
        assert!(!present(&dir, &files), "no engine yet");
        fs::write(dir.join(ENGINE_NAME), b"").unwrap();
        assert!(present(&dir, &files));
        fs::write(dir.join("a.onnx"), b"ab").unwrap();
        assert!(!present(&dir, &files), "wrong size");
        let _ = fs::remove_dir_all(dir);
    }

    /// Talks to a real engine: set XIAOBAN_INFER_DIR to a folder holding the engine (named ENGINE_NAME) and the turn
    /// models.
    #[test]
    #[ignore = "needs the xiaoban-infer engine and models; set XIAOBAN_INFER_DIR"]
    fn the_engine_serves_the_capability_it_was_started_for() {
        let Some(dir) = std::env::var_os("XIAOBAN_INFER_DIR") else { return };
        let mut turn = Engine::start(Path::new(&dir), "turn").unwrap();
        let reply = turn.request(&serde_json::json!({ "type": "vad", "stream": "t", "pcm": vec![0i16; 1600] })).unwrap();
        assert_eq!(reply["values"].as_array().unwrap().len(), 3);
        assert!(turn.request(&serde_json::json!({ "type": "transcribe", "samples": vec![0.0f32; 1600] })).is_err());
        assert!(Engine::start(Path::new(&dir), "nonexistent").is_err());
    }
}
