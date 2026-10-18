//! SenseVoice Small ONNX INT8. Only the inference library is linked into the app.
use sha2::{Digest, Sha256};
use sherpa_onnx::{OfflineRecognizer, OfflineRecognizerConfig, OfflineSenseVoiceModelConfig};
use std::{fs, io::{Read, Write}, path::Path, time::{Duration, Instant}};

// ModelScope's sherpa-compatible export of iic/SenseVoiceSmall. Pin both revision and hashes.
pub const SOURCE: &str = "https://modelscope.cn/api/v1/models/pengzhendong/sherpa-onnx-sense-voice-zh-en-ja-ko-yue/repo?Revision=73eca47697f980daa3d16112404174b6b950b514&FilePath=";
pub struct ModelFile { pub name: &'static str, pub bytes: u64, pub sha256: &'static str }
pub const FILES: [ModelFile; 2] = [
    ModelFile { name: "model.int8.onnx", bytes: 239233841, sha256: "c71f0ce00bec95b07744e116345e33d8cbbe08cef896382cf907bf4b51a2cd51" },
    ModelFile { name: "tokens.txt", bytes: 315894, sha256: "f449eb28dc567533d7fa59be34e2abca8784f771850c78a47fb731a31429a1dc" },
];
pub const TOTAL_BYTES: u64 = FILES[0].bytes + FILES[1].bytes;
pub const MAX_SAMPLES: usize = 20 * 16000;

pub fn present(directory: &Path) -> bool {
    FILES.iter().all(|file| fs::metadata(directory.join(file.name)).is_ok_and(|m| m.is_file() && m.len() == file.bytes))
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
            // It was hashed while it was written, so there is no need to read it again for the record.
            remember_verified(&target, spec);
            Ok(())
        })();
        if result.is_err() { let _ = fs::remove_file(&temporary); }
        result?;
        completed += spec.bytes;
        progress(spec.name, completed);
    }
    Ok(())
}
pub fn load(directory: &Path) -> Result<OfflineRecognizer, String> {
    let mut config = OfflineRecognizerConfig::default();
    config.model_config.sense_voice = OfflineSenseVoiceModelConfig {
        model: Some(directory.join(FILES[0].name).to_str().ok_or("模型路径无效")?.into()),
        language: Some("auto".into()), use_itn: true,
    };
    config.model_config.tokens = Some(directory.join(FILES[1].name).to_str().ok_or("词表路径无效")?.into());
    config.model_config.num_threads = 2;
    config.model_config.provider = Some("cpu".into());
    OfflineRecognizer::create(&config).ok_or_else(|| "SenseVoice 引擎加载失败，请关闭后重新开启语音识别".into())
}
pub fn validate_audio(samples: &[f32]) -> Result<(), String> {
    if samples.len() < 1600 || samples.len() > MAX_SAMPLES || samples.iter().any(|s| !s.is_finite() || s.abs() > 1.01) {
        return Err("语音数据无效，请使用 0.1–20 秒、16kHz 单声道 PCM 音频".into());
    }
    Ok(())
}
pub fn transcribe(recognizer: &OfflineRecognizer, samples: &[f32]) -> Result<String, String> {
    validate_audio(samples)?;
    let stream = recognizer.create_stream();
    stream.accept_waveform(16000, samples);
    recognizer.decode(&stream);
    let text = stream.get_result().ok_or("语音识别未返回结果")?.text;
    Ok(clean_text(&text))
}
fn clean_text(text: &str) -> String {
    let mut remaining = text;
    let mut output = String::new();
    while let Some(start) = remaining.find("<|") {
        output.push_str(&remaining[..start]);
        match remaining[start + 2..].find("|>") {
            Some(end) => remaining = &remaining[start + 2 + end + 2..],
            None => { remaining = &remaining[start..]; break; }
        }
    }
    output.push_str(remaining);
    output.trim().into()
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
    /// `XIAOBAN_ASR_DIR=<model folder> cargo test --bin xiaoban-desktop-wallpaper measure_startup_cost -- --ignored --nocapture`
    #[test] #[ignore = "needs the downloaded model"]
    fn measure_startup_cost() {
        let directory = std::path::PathBuf::from(std::env::var("XIAOBAN_ASR_DIR").expect("set XIAOBAN_ASR_DIR"));
        let started = Instant::now();
        ensure_model(&directory, || Ok(()), |_, _| {}).unwrap();
        println!("verify  {:?}", started.elapsed());
        let started = Instant::now();
        let recognizer = load(&directory).unwrap();
        println!("load    {:?}", started.elapsed());
        for round in 1..=3 {
            let started = Instant::now();
            transcribe(&recognizer, &vec![0.0; 16000]).unwrap();
            println!("decode {round}  {:?}", started.elapsed());
        }
    }
    #[test] fn a_verified_file_is_not_hashed_again_until_it_changes() {
        let dir = std::env::temp_dir().join(format!("xiaoban-asr-stamp-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("model.bin");
        fs::write(&path, b"abc").unwrap();
        // sha256("abc")
        let spec = ModelFile { name: "model.bin", bytes: 3, sha256: "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad" };
        let reads = std::cell::Cell::new(0);
        let counting = || { reads.set(reads.get() + 1); Ok(()) };
        assert!(verified(&path, &spec, &counting).unwrap());
        let first = reads.get();
        assert!(first > 0, "the first check reads the file");
        assert!(verified(&path, &spec, &counting).unwrap());
        assert_eq!(reads.get(), first, "the second check trusts the record");
        // same size, different content: the modification time moves, so it is read and rejected
        std::thread::sleep(std::time::Duration::from_millis(20));
        fs::write(&path, b"abd").unwrap();
        assert!(!verified(&path, &spec, &counting).unwrap());
        assert!(reads.get() > first);
        // a record for other content never vouches for a file
        fs::write(stamp_path(&path), "garbage").unwrap();
        fs::write(&path, b"abc").unwrap();
        assert!(verified(&path, &spec, &counting).unwrap());
        fs::remove_dir_all(&dir).unwrap();
    }
    #[test] fn removes_sensevoice_tags() {
        assert_eq!(clean_text("<|zh|><|NEUTRAL|><|Speech|><|withitn|>你好。"), "你好。");
        assert_eq!(clean_text("<|en|>Hello <|HAPPY|>world!"), "Hello world!");
        assert_eq!(clean_text("<|nospeech|><|BGM|>"), "");
    }
    #[test] fn refuses_same_length_corruption_and_partial_files() {
        let path = std::env::temp_dir().join(format!("xiaoban-asr-{}", std::process::id()));
        fs::write(&path, b"bad").unwrap();
        let spec = ModelFile { name: "test", bytes: 3, sha256: "0000000000000000000000000000000000000000000000000000000000000000" };
        assert!(!verified(&path, &spec, &|| Ok(())).unwrap());
        assert!(verified(&path, &spec, &|| Err("cancelled".into())).is_err());
        fs::remove_file(path).unwrap();
    }
}
