//! SenseVoice Small ONNX INT8 via standalone xiaoban-asr companion process.
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    time::{Duration, Instant},
};

// ModelScope's sherpa-compatible export of iic/SenseVoiceSmall. Pin both revision and hashes.
pub const SOURCE: &str = "https://modelscope.cn/api/v1/models/pengzhendong/sherpa-onnx-sense-voice-zh-en-ja-ko-yue/repo?Revision=73eca47697f980daa3d16112404174b6b950b514&FilePath=";

// Runtime fallback download source (official site / release)
pub const RUNTIME_FALLBACK: &str = "https://wallpaper.xiangban.xyz/app-packages/xiaoban-asr.exe";

#[cfg(windows)]
pub const RUNTIME_NAME: &str = "xiaoban-asr.exe";
#[cfg(not(windows))]
pub const RUNTIME_NAME: &str = "xiaoban-asr";

pub struct ModelFile { pub name: &'static str, pub bytes: u64, pub sha256: &'static str }
pub const FILES: [ModelFile; 2] = [
    ModelFile { name: "model.int8.onnx", bytes: 239233841, sha256: "c71f0ce00bec95b07744e116345e33d8cbbe08cef896382cf907bf4b51a2cd51" },
    ModelFile { name: "tokens.txt", bytes: 315894, sha256: "f449eb28dc567533d7fa59be34e2abca8784f771850c78a47fb731a31429a1dc" },
];
pub const TOTAL_BYTES: u64 = FILES[0].bytes + FILES[1].bytes;
pub const MAX_SAMPLES: usize = 20 * 16000;

pub fn resolve_exe(directory: &Path) -> Option<PathBuf> {
    let in_dir = directory.join(RUNTIME_NAME);
    if in_dir.is_file() {
        return Some(in_dir);
    }
    if let Ok(current) = std::env::current_exe() {
        if let Some(parent) = current.parent() {
            let next_to_app = parent.join(RUNTIME_NAME);
            if next_to_app.is_file() {
                return Some(next_to_app);
            }
        }
    }
    let in_cwd = PathBuf::from(RUNTIME_NAME);
    if in_cwd.is_file() {
        return Some(in_cwd);
    }
    None
}

pub fn present(directory: &Path) -> bool {
    FILES.iter().all(|file| fs::metadata(directory.join(file.name)).is_ok_and(|m| m.is_file() && m.len() == file.bytes))
        && resolve_exe(directory).is_some()
}

/// A file that was fully hashed once is remembered by its size and modification time, so the next start does not read
/// 239 MB again to learn the same thing. Any change to the file, or no record at all, means it is hashed again.
fn stamp_path(path: &Path) -> std::path::PathBuf {
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
    let mut file = fs::File::open(path).map_err(|e| format!("语音模型读取失败：{e}"))?;
    let mut digest = Sha256::new();
    let mut buffer = [0; 65536];
    loop { check()?; let n = file.read(&mut buffer).map_err(|e| e.to_string())?; if n == 0 { break; } digest.update(&buffer[..n]); }
    let good = format!("{:x}", digest.finalize()) == spec.sha256;
    if good { remember_verified(path, spec); }
    Ok(good)
}

pub fn ensure_model(directory: &Path, check: impl Fn() -> Result<(), String>, progress: impl Fn(&str, u64)) -> Result<(), String> {
    fs::create_dir_all(directory).map_err(|e| format!("无法创建语音模型目录：{e}"))?;
    let client = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36 XiaoBan/1.0")
        .https_only(true).connect_timeout(Duration::from_secs(15))
        .timeout(Duration::from_secs(600)).build().map_err(|e| e.to_string())?;
    let mut completed = 0;
    for spec in &FILES {
        check()?;
        let target = directory.join(spec.name);
        if verified(&target, spec, &check)? { completed += spec.bytes; progress(spec.name, completed); continue; }
        let temporary = directory.join(format!("{}.part", spec.name));
        let result: Result<(), String> = (|| {
            progress(spec.name, completed);
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
                if last_event.elapsed() >= Duration::from_millis(150) { progress(spec.name, completed + received); last_event = Instant::now(); }
            }
            if received != spec.bytes || format!("{:x}", digest.finalize()) != spec.sha256 { return Err("语音模型完整性校验失败，请重试".into()); }
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
        completed += spec.bytes;
        progress(spec.name, completed);
    }

    // Ensure runtime executable
    if resolve_exe(directory).is_none() {
        let target = directory.join(RUNTIME_NAME);
        let temporary = directory.join(format!("{RUNTIME_NAME}.part"));
        let result: Result<(), String> = (|| {
            progress(RUNTIME_NAME, completed);
            let mut response = client.get(RUNTIME_FALLBACK).send()
                .and_then(reqwest::blocking::Response::error_for_status).map_err(|e| format!("语音引擎组件下载失败：{e}"))?;
            let mut file = fs::File::create(&temporary).map_err(|e| format!("语音组件写入失败：{e}"))?;
            let mut buffer = [0; 65536];
            loop {
                check()?;
                let n = response.read(&mut buffer).map_err(|e| format!("语音组件下载中断：{e}"))?;
                if n == 0 { break; }
                file.write_all(&buffer[..n]).map_err(|e| format!("语音组件写入失败：{e}"))?;
            }
            file.sync_all().map_err(|e| e.to_string())?;
            drop(file);
            if target.exists() { fs::remove_file(&target).map_err(|e| e.to_string())?; }
            fs::rename(&temporary, &target).map_err(|e| format!("语音组件保存失败：{e}"))?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = fs::set_permissions(&target, fs::Permissions::from_mode(0o755));
            }
            Ok(())
        })();
        if result.is_err() { let _ = fs::remove_file(&temporary); }
        result?;
    }

    Ok(())
}

pub struct AsrProcess {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl AsrProcess {
    pub fn spawn(directory: &Path) -> Result<Self, String> {
        let exe_path = resolve_exe(directory).ok_or_else(|| {
            format!("找不到语音识别引擎组件（{}），请重新开启语音识别自动下载", RUNTIME_NAME)
        })?;

        let mut cmd = Command::new(&exe_path);
        cmd.arg("--dir")
            .arg(directory)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        let mut child = cmd.spawn().map_err(|e| format!("启动语音识别引擎失败：{e}"))?;
        let stdin = child.stdin.take().ok_or("无法连接语音引擎输入流")?;
        let stdout_raw = child.stdout.take().ok_or("无法连接语音引擎输出流")?;
        let mut stdout = BufReader::new(stdout_raw);

        let mut line = String::new();
        stdout.read_line(&mut line).map_err(|e| format!("读取语音引擎就绪响应失败：{e}"))?;
        let resp: serde_json::Value = serde_json::from_str(line.trim())
            .map_err(|e| format!("语音引擎响应解析失败：{e}，响应为：{line}"))?;

        if resp.get("status").and_then(|s| s.as_str()) != Some("ready") {
            let error = resp.get("error").and_then(|e| e.as_str()).unwrap_or("引擎启动异常");
            return Err(format!("语音引擎初始化失败：{error}"));
        }

        Ok(Self { child, stdin, stdout })
    }

    pub fn transcribe(&mut self, samples: &[f32]) -> Result<String, String> {
        validate_audio(samples)?;
        let req = serde_json::json!({
            "type": "transcribe",
            "samples": samples
        });
        let req_str = serde_json::to_string(&req).map_err(|e| e.to_string())?;
        writeln!(self.stdin, "{req_str}").map_err(|e| format!("向语音引擎发送音频失败：{e}"))?;
        self.stdin.flush().map_err(|e| format!("刷新语音引擎输入流失败：{e}"))?;

        let mut line = String::new();
        self.stdout.read_line(&mut line).map_err(|e| format!("读取语音识别结果失败：{e}"))?;
        let resp: serde_json::Value = serde_json::from_str(line.trim())
            .map_err(|e| format!("语音识别结果解析失败：{e}"))?;

        match resp.get("status").and_then(|s| s.as_str()) {
            Some("ok") => {
                let text = resp.get("text").and_then(|t| t.as_str()).unwrap_or("");
                Ok(text.to_string())
            }
            Some("error") => {
                let err = resp.get("error").and_then(|e| e.as_str()).unwrap_or("识别出错");
                Err(err.to_string())
            }
            _ => Err("语音引擎返回未知响应".into()),
        }
    }
}

impl Drop for AsrProcess {
    fn drop(&mut self) {
        let _ = writeln!(self.stdin, "{{\"type\":\"exit\"}}");
        let _ = self.stdin.flush();
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

pub fn load(directory: &Path) -> Result<AsrProcess, String> {
    AsrProcess::spawn(directory)
}

pub fn validate_audio(samples: &[f32]) -> Result<(), String> {
    if samples.len() < 1600 || samples.len() > MAX_SAMPLES || samples.iter().any(|s| !s.is_finite() || s.abs() > 1.01) {
        return Err("语音数据无效，请使用 0.1–20 秒、16kHz 单声道 PCM 音频".into());
    }
    Ok(())
}

pub fn transcribe(recognizer: &mut AsrProcess, samples: &[f32]) -> Result<String, String> {
    recognizer.transcribe(samples)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn rejects_invalid_pcm() {
        assert!(validate_audio(&vec![0.1; 1600]).is_ok());
        assert!(validate_audio(&vec![0.0; MAX_SAMPLES + 1]).is_err());
        assert!(validate_audio(&[f32::NAN; 1600]).is_err());
        assert!(validate_audio(&[f32::INFINITY; 1600]).is_err());
        assert!(validate_audio(&[2.0; 1600]).is_err());
        assert!(validate_audio(&[]).is_err());
    }
    #[test] fn a_verified_file_is_not_hashed_again_until_it_changes() {
        let dir = std::env::temp_dir().join(format!("xiaoban-asr-stamp-{}", std::process::id()));
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
}
