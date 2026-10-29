//! The operating system's own speech: macOS `say`, Windows SAPI. Always available offline, so it is the
//! fallback when an online voice fails; the voice is plainer. Returns WAV audio.
use std::path::PathBuf;
#[cfg(any(target_os = "macos", windows))]
use std::process::Command;

fn temp(name: &str) -> Result<PathBuf, String> {
    let mut bytes = [0u8; 8];
    getrandom::fill(&mut bytes).map_err(|_| "无法创建临时文件")?;
    Ok(std::env::temp_dir().join(format!("xiaoban-tts-{}-{name}", bytes.iter().map(|b| format!("{b:02x}")).collect::<String>())))
}

struct Cleanup(Vec<PathBuf>);
impl Drop for Cleanup { fn drop(&mut self) { for path in &self.0 { let _ = std::fs::remove_file(path); } } }

/// `rate` is the app's -50..100 percent adjustment.
pub fn speak(text: &str, rate: i32) -> Result<Vec<u8>, String> {
    let input = temp("in.txt")?;
    let output = temp("out.wav")?;
    let _cleanup = Cleanup(vec![input.clone(), output.clone()]);
    std::fs::write(&input, text).map_err(|_| "无法写入朗读文本")?;
    run(&input, &output, rate)?;
    let audio = std::fs::read(&output).map_err(|_| "系统朗读没有生成音频")?;
    if audio.len() <= 44 { return Err("系统朗读没有生成音频".into()); }
    Ok(audio)
}

#[cfg(target_os = "macos")]
fn run(input: &std::path::Path, output: &std::path::Path, rate: i32) -> Result<(), String> {
    let words_per_minute = (180.0 * (1.0 + rate as f64 / 100.0)).round().clamp(90.0, 360.0).to_string();
    let attempt = |voice: Option<&str>| {
        let mut command = Command::new("/usr/bin/say");
        if let Some(voice) = voice { command.args(["-v", voice]); }
        command.args(["-r", &words_per_minute, "--file-format=WAVE", "--data-format=LEI16@22050", "-o"]).arg(output).arg("-f").arg(input);
        command.status().map(|s| s.success()).unwrap_or(false)
    };
    // Tingting (Mandarin) ships with macOS; without it the system voice still reads.
    if attempt(Some("Tingting")) || attempt(None) { Ok(()) } else { Err("系统朗读不可用".into()) }
}

#[cfg(windows)]
fn run(input: &std::path::Path, output: &std::path::Path, rate: i32) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    const SCRIPT: &str = "Add-Type -AssemblyName System.Speech;\
        $s = New-Object System.Speech.Synthesis.SpeechSynthesizer;\
        $v = $s.GetInstalledVoices() | Where-Object { $_.Enabled -and $_.VoiceInfo.Culture.Name -like 'zh*' } | Select-Object -First 1;\
        if ($v) { $s.SelectVoice($v.VoiceInfo.Name) };\
        $s.Rate = [int]$env:XB_TTS_RATE;\
        $s.SetOutputToWaveFile($env:XB_TTS_OUT);\
        $s.Speak([IO.File]::ReadAllText($env:XB_TTS_IN, [Text.Encoding]::UTF8));\
        $s.Dispose()";
    // SAPI rates run -10..10.
    let sapi_rate = ((rate as f64) / 10.0).round().clamp(-10.0, 10.0) as i32;
    let status = Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", SCRIPT])
        .env("XB_TTS_IN", input).env("XB_TTS_OUT", output).env("XB_TTS_RATE", sapi_rate.to_string())
        .creation_flags(0x08000000) // No console window.
        .status().map_err(|_| "系统朗读不可用")?;
    if status.success() { Ok(()) } else { Err("系统朗读不可用".into()) }
}

#[cfg(not(any(target_os = "macos", windows)))]
fn run(_input: &std::path::Path, _output: &std::path::Path, _rate: i32) -> Result<(), String> {
    Err("当前系统不支持系统朗读".into())
}
