//! Runs the user's previously saved yt-dlp tool with fixed arguments.
use agent_kernel::{Risk, Tool, ToolCtx, ToolResult, ToolSpec};
use serde_json::{Value, json};
use std::{
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{Arc, atomic::{AtomicU64, Ordering}},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tauri::Manager;

const TOOL_DIR: &str = "community-tools/yt-dlp";
pub(crate) const DOWNLOAD_TOOL: &str = "community_download_video";
pub(crate) const BILIBILI_HTTP_567: &str = "bilibili_http_567";

const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(300);

fn unique_id() -> String {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis();
    format!("{timestamp}-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed))
}

fn asset() -> Result<&'static str, String> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64" | "x86_64") => Ok("yt-dlp_macos"),
        ("windows", "x86_64") => Ok("yt-dlp.exe"),
        ("windows", "aarch64") => Ok("yt-dlp_arm64.exe"),
        _ => Err("当前系统暂无适配的 yt-dlp 安装包".into()),
    }
}

fn tool_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(crate::storage::root(app)?.join(TOOL_DIR))
}

fn program(binary: &Path) -> Command {
    let mut command = Command::new(binary);
    command.env_clear();
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
        for key in ["SystemRoot", "TEMP", "TMP"] {
            if let Some(value) = std::env::var_os(key) { command.env(key, value); }
        }
    }
    command
}

fn media_url(input: &str) -> Result<url::Url, String> {
    let mut url = url::Url::parse(input.trim()).map_err(|_| "请提供有效的视频链接")?;
    if !matches!(url.scheme(), "https" | "http") || url.domain().is_none()
        || url.host_str().is_some_and(|host| host.eq_ignore_ascii_case("localhost") || host.ends_with(".localhost"))
        || !url.username().is_empty() || url.password().is_some() {
        return Err("请提供公开网站的 HTTP 或 HTTPS 视频链接".into());
    }
    if matches!(url.host_str(), Some("www.bilibili.com" | "bilibili.com" | "m.bilibili.com"))
        && url.port().is_none()
    {
        let segments: Vec<_> = url.path().trim_end_matches('/').split('/').skip(1).collect();
        let id = match segments.as_slice() {
            ["video", id] | ["zh-Hans" | "zh-Hant" | "en", "video", id] => Some(*id),
            _ => None,
        };
        if let Some(id) = id.filter(|id| {
            id.strip_prefix("BV").is_some_and(|suffix| {
                suffix.len() == 10 && suffix.bytes().all(|byte| byte.is_ascii_alphanumeric())
            }) || id.strip_prefix("av").is_some_and(|suffix| {
                !suffix.is_empty() && suffix.bytes().all(|byte| byte.is_ascii_digit())
            })
        }) {
            let path = format!("/video/{id}/");
            let selectors: Vec<_> = url.query_pairs()
                .filter(|(key, _)| matches!(key.as_ref(), "p" | "t"))
                .map(|(key, value)| (key.into_owned(), value.into_owned())).collect();
            url.set_host(Some("www.bilibili.com")).map_err(|_| "无效的 B 站链接")?;
            url.set_path(&path);
            url.set_query(None);
            url.set_fragment(None);
            if !selectors.is_empty() { url.query_pairs_mut().extend_pairs(selectors); }
        }
    }
    Ok(url)
}

pub(crate) fn bilibili_target(input: &str) -> Option<url::Url> {
    let url = media_url(input).ok()?;
    (url.host_str() == Some("www.bilibili.com") && url.port().is_none()
        && url.path().starts_with("/video/")).then_some(url)
}

/// Use the same FFmpeg installation to check for both streams. Missing ffprobe
/// leaves the download usable, but cannot supply evidence for experience review.
fn verify_streams(path: &Path, ffmpeg: &Path, cancelled: &impl Fn() -> bool) -> bool {
    let probe = ffmpeg.with_file_name("ffprobe");
    if !probe.is_file() || cancelled() { return false; }
    let Ok(mut child) = program(&probe)
        .args(["-v", "error", "-protocol_whitelist", "file", "-show_entries", "stream=codec_type", "-of", "json", "-i"])
        .arg(path).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn()
    else { return false; };
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if cancelled() || Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return false;
        }
        match child.try_wait() {
            Ok(Some(status)) if status.success() => break,
            Ok(None) => std::thread::sleep(Duration::from_millis(50)),
            _ => return false,
        }
    }
    let Some(stdout) = child.stdout.take() else { return false; };
    let Ok(value) = serde_json::from_reader::<_, Value>(stdout.take(16 * 1024)) else { return false; };
    value["streams"].as_array().is_some_and(|streams|
        streams.iter().any(|stream| stream["codec_type"] == "video")
        && streams.iter().any(|stream| stream["codec_type"] == "audio"))
}

fn download(app: &tauri::AppHandle, input: &str, range_retry: bool, cancelled: impl Fn() -> bool) -> Result<Value, String> {
    let url = media_url(input)?;
    if range_retry && bilibili_target(input).is_none() {
        return Err("Range 重试仅适用于 B 站视频链接".into());
    }
    let root = tool_dir(app)?;
    let binary = root.join(asset()?);
    if !binary.is_file() { return Err("找不到此前保存的 yt-dlp 工具".into()); }
    let parent = app.path().download_dir().map_err(|e| e.to_string())?.join("小伴");
    fs::create_dir_all(&parent).map_err(|e| e.to_string())?;
    let destination = parent.join(format!("视频-{}", unique_id()));
    fs::create_dir(&destination).map_err(|e| e.to_string())?;
    let result = (|| {
        let log = File::create(destination.join(".error.log")).map_err(|e| e.to_string())?;
        let mut command = program(&binary);
        let ffmpeg = if cfg!(target_os = "macos") {
            [Path::new("/opt/homebrew/bin/ffmpeg"), Path::new("/usr/local/bin/ffmpeg")]
                .into_iter().find(|path| path.is_file())
        } else { None };
        if let Some(ffmpeg) = ffmpeg {
            command.arg("--ffmpeg-location").arg(ffmpeg);
        }
        if range_retry {
            command.args(["--add-headers", "Range:bytes=0-"]);
        }
        let mut child = command.current_dir(&destination)
            .args(["--ignore-config", "--no-plugin-dirs", "--no-js-runtimes", "--no-remote-components",
                "--no-cache-dir", "--no-playlist",
                "--playlist-items", "1", "--no-overwrites", "--max-filesize", "512M",
                "--socket-timeout", "15", "--retries", "1", "--no-progress",
                "-o", "video.%(ext)s", "--", url.as_str()])
            .stdin(Stdio::null()).stdout(Stdio::null()).stderr(log).spawn().map_err(|e| e.to_string())?;
        let deadline = Instant::now() + DOWNLOAD_TIMEOUT;
        loop {
            if cancelled() || Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                return Err("下载已取消或超时".into());
            }
            if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
                if !status.success() {
                    let mut error = String::new();
                    if let Ok(log) = File::open(destination.join(".error.log")) {
                        let _ = log.take(1500).read_to_string(&mut error);
                    }
                    return Err(format!("yt-dlp 下载失败：{}", error.trim()));
                }
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        let output = fs::read_dir(&destination).map_err(|e| e.to_string())?
            .filter_map(Result::ok).find(|entry| {
                entry.path().file_stem().is_some_and(|stem| stem == "video")
                    && !entry.file_name().to_string_lossy().ends_with(".part")
                    && entry.metadata().is_ok_and(|metadata| metadata.is_file() && metadata.len() > 0)
            }).ok_or("下载未生成可用文件")?;
        let path = output.path();
        let media_verified = ffmpeg.is_some_and(|ffmpeg| verify_streams(&path, ffmpeg, &cancelled));
        if cancelled() { return Err("下载已取消".into()); }
        Ok(json!({"path":path,"mediaVerified":media_verified}))
    })();
    if result.is_err() { let _ = fs::remove_dir_all(&destination); }
    else { let _ = fs::remove_file(destination.join(".error.log")); }
    result
}

struct MediaTool {
    app: tauri::AppHandle,
    spec: ToolSpec,
}
impl Tool for MediaTool {
    fn spec(&self) -> &ToolSpec { &self.spec }
    fn execute_result(&self, args: Value, ctx: &ToolCtx) -> ToolResult {
        if ctx.is_cancelled() { return ToolResult::error("操作已取消"); }
        let Some(url) = args["url"].as_str() else { return ToolResult::error("缺少视频链接"); };
        let range_retry = args["range_retry"].as_bool().unwrap_or(false);
        match download(&self.app, url, range_retry, || ctx.is_cancelled()) {
            Ok(result) => {
                let verified = range_retry && result["mediaVerified"] == true;
                let output = ToolResult::ok(result.to_string());
                if verified {
                    output.with_structured_content(json!({"verified_recovery": BILIBILI_HTTP_567}))
                } else {
                    output
                }
            }
            Err(error) => {
                let initial_567 = !range_retry && bilibili_target(url).is_some()
                    && error.starts_with("yt-dlp 下载失败：") && error.contains("HTTP Error 567:");
                let output = ToolResult::error(error);
                if initial_567 {
                    output.with_structured_content(json!({"recoverable_error": BILIBILI_HTTP_567}))
                } else {
                    output
                }
            }
        }
    }
}

pub(crate) fn saved_tool(app: &tauri::AppHandle) -> Option<Arc<dyn Tool>> {
    if !tool_dir(app).ok()?.join("saved").is_file() { return None; }
    Some(Arc::new(MediaTool {
        app: app.clone(),
        spec: ToolSpec::new(DOWNLOAD_TOOL,
            "调用用户已保存的 yt-dlp 工具。用户明确要求下载视频并给出 HTTP(S) 链接时使用。自动规范化常见 B 站分享链接。下载单个视频到系统下载目录的小伴子目录，返回文件路径。HTTP 567 表示媒体服务器拒绝请求，单凭状态码无法判断是否需要登录。",
            json!({"type":"object","properties":{"url":{"type":"string"},"range_retry":{"type":"boolean","description":"默认 false。仅在 B 站媒体流 HTTP 567 且已查到证据支持时，对相同视频设为 true 重试一次，添加 Range: bytes=0-。不读取 Cookie。"}},"required":["url"],"additionalProperties":false}))
            .risk(Risk::Exec).always_ask().untrusted_output().serial().timeout(DOWNLOAD_TIMEOUT + Duration::from_secs(10)),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_bilibili_share_urls_and_preserves_the_selected_part() {
        let input = "https://www.bilibili.com/zh-Hans/video/BV15N4V6hEmF/?trackid=abc&p=2&spm_id_from=xyz";
        assert_eq!(media_url(input).unwrap().as_str(),
            "https://www.bilibili.com/video/BV15N4V6hEmF/?p=2");
        let short = "https://b23.tv/example";
        assert_eq!(media_url(short).unwrap().as_str(), short);
    }
}
