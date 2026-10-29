//! Edge read-aloud framing, ported from the xiaban-api Kotlin TTS implementation.
use super::drm;

use chrono::{TimeZone, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const HOST: &str = "speech.platform.bing.com";
pub const BASE: &str = "/consumer/speech/synthesize/readaloud";
pub const TOKEN: &str = "6A5AA1D4EAFF4E9FB37E23D68491D6F4";
pub const SEC_MS_GEC_VERSION: &str = "2-143.0.3650.75";
pub const ORIGIN: &str = "chrome-extension://jdiccldimpdaibmpdkjnbmckianbfold";
pub const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/143.0.0.0 Safari/537.36 Edg/143.0.0.0";
pub const MAX_CHARS: usize = 10_000;
pub const MAX_AUDIO: usize = 16 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Options {
    pub voice: String,
    pub rate: i32,
    pub volume: i32,
    pub pitch: i32,
}
impl Default for Options {
    fn default() -> Self { Self { voice: "zh-CN-XiaoxiaoNeural".into(), rate: 0, volume: 100, pitch: 0 } }
}
impl Options {
    pub fn validate(&self) -> Result<(), String> {
        if self.voice.is_empty() || self.voice.len() > 64
            || !self.voice.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-')
            || !(-50..=100).contains(&self.rate) || !(0..=100).contains(&self.volume)
            || !(-50..=50).contains(&self.pitch) {
            return Err("音色、语速、音量或音调设置无效".into());
        }
        Ok(())
    }
}
pub fn gec(unix: i64) -> String {
    drm::generate_sec_ms_gec_v2(unix)
}
pub fn random_id() -> Result<String, String> {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).map_err(|_| "无法生成语音请求标识")?;
    Ok(bytes.iter().map(|b| format!("{b:02X}")).collect())
}
pub fn ws_url(id: &str, unix: i64) -> String {
    format!("wss://{HOST}{BASE}/edge/v1?TrustedClientToken={TOKEN}&ConnectionId={id}&Sec-MS-GEC={}&Sec-MS-GEC-Version={SEC_MS_GEC_VERSION}", gec(unix))
}
pub fn voices_url(unix: i64) -> String {
    format!("https://{HOST}{BASE}/voices/list?trustedclienttoken={TOKEN}&Sec-MS-GEC={}&Sec-MS-GEC-Version={SEC_MS_GEC_VERSION}", gec(unix))
}
fn timestamp(unix: i64) -> String {
    Utc.timestamp_opt(unix, 0).single().unwrap_or_else(Utc::now)
        .format("%a %b %d %Y %H:%M:%S GMT+0000 (Coordinated Universal Time)").to_string()
}
pub fn xml(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
        .replace('"', "&quot;").replace('\'', "&apos;")
}
pub fn voice_name(short: &str) -> String {
    let parts: Vec<_> = short.split('-').collect();
    if parts.len() < 3 || !short.ends_with("Neural") { return short.into(); }
    let language_region = parts[..parts.len() - 1].join("-");
    format!("Microsoft Server Speech Text to Speech Voice ({language_region}, {})", parts[parts.len() - 1])
}
pub fn config(unix: i64) -> String {
    format!("X-Timestamp:{}\r\nContent-Type:application/json; charset=utf-8\r\nPath:speech.config\r\n\r\n{}", timestamp(unix),
        serde_json::json!({"context":{"synthesis":{"audio":{"metadataoptions":{"sentenceBoundaryEnabled":"false","wordBoundaryEnabled":"false"},"outputFormat":"audio-24khz-48kbitrate-mono-mp3"}}}}))
}
pub fn ssml(id: &str, options: &Options, text: &str, unix: i64) -> String {
    format!("X-RequestId:{id}\r\nContent-Type:application/ssml+xml\r\nX-Timestamp:{}Z\r\nPath:ssml\r\n\r\n<speak version='1.0' xmlns='http://www.w3.org/2001/10/synthesis' xml:lang='en-US'><voice name='{}'><prosody pitch='{:+}Hz' rate='{:+}%' volume='{:+}%'>{}</prosody></voice></speak>",
        timestamp(unix), xml(&voice_name(&options.voice)), options.pitch, options.rate, options.volume - 100, xml(text))
}
pub fn chunks(text: &str) -> Result<Vec<String>, String> {
    if text.chars().count() > MAX_CHARS { return Err("回复较长，请分段朗读（每次最多 10000 字）".into()); }
    let clean: String = text.chars().filter(|c| !c.is_control() || matches!(c, '\n' | '\r' | '\t')).collect();
    let mut rest = clean.trim();
    let mut result = Vec::new();
    while !rest.is_empty() {
        let mut end = 0;
        for (i, c) in rest.char_indices() {
            if i + c.len_utf8() > 4096 { break; }
            end = i + c.len_utf8();
        }
        if end < rest.len() {
            if let Some((i, _)) = rest[..end].char_indices().rev()
                .find(|(i, c)| *i >= end / 2 && (c.is_whitespace() || "。！？.!?".contains(*c))) {
                end = i + rest[i..].chars().next().unwrap().len_utf8();
            }
        }
        result.push(rest[..end].trim().to_string());
        rest = rest[end..].trim_start();
    }
    if result.is_empty() { return Err("没有可以朗读的文本".into()); }
    Ok(result)
}
fn headers(raw: &str) -> BTreeMap<String, String> {
    raw.lines().filter_map(|line| line.split_once(':')).map(|(k, v)| (k.trim().to_ascii_lowercase(), v.trim().into())).collect()
}
pub fn text_path(frame: &str) -> Result<String, String> {
    let (head, _) = frame.split_once("\r\n\r\n").ok_or("语音响应缺少帧头")?;
    headers(head).remove("path").ok_or_else(|| "语音响应缺少 Path".into())
}
pub fn audio_frame(frame: &[u8]) -> Result<&[u8], String> {
    if frame.len() < 2 { return Err("语音响应帧不完整".into()); }
    let len = u16::from_be_bytes([frame[0], frame[1]]) as usize;
    if len == 0 && frame.len() == 2 { return Ok(&[]); }
    if len < 4 || len + 2 > frame.len() || frame[len..len + 2] != *b"\r\n" { return Err("语音响应帧长度无效".into()); }
    let raw = std::str::from_utf8(&frame[2..len]).map_err(|_| "语音响应帧头无效")?;
    if headers(raw).get("path").map(String::as_str) != Some("audio") { return Err("收到非音频响应帧".into()); }
    Ok(&frame[len + 2..])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn escapes_after_utf8_split_and_never_cuts_a_character() {
        let input = "小伴 & <你好> 😀 ".repeat(500);
        let parts = chunks(&input).unwrap();
        assert!(parts.iter().all(|p| p.len() <= 4096));
        assert!(ssml("id", &Options::default(), "a<&", 0).contains("a&lt;&amp;"));
        assert_eq!(voice_name("zh-CN-XiaoxiaoNeural"), "Microsoft Server Speech Text to Speech Voice (zh-CN, XiaoxiaoNeural)");
    }
    #[test] fn frame_validation_and_zero_length_end_marker() {
        let head = b"Path:audio\r\nContent-Type:audio/mpeg\r\n";
        let mut data = (head.len() as u16).to_be_bytes().to_vec();
        data.extend(head); data.extend(b"MP3");
        assert_eq!(audio_frame(&data).unwrap(), b"MP3");
        assert!(audio_frame(&[0, 1, 2]).is_err());
        assert_eq!(audio_frame(&[0, 0]).unwrap(), b"");
    }
    #[test] fn hash_window_and_options_are_validated() {
        assert_eq!(gec(1_700_000_000), "2CD23A2CA0B2595A-F3C846C370E05F9CC83ACC5F995C5F5D87A4D0F14CE8936A0F902E64099E38A2");
        assert_eq!(gec(1_700_000_000), gec(1_700_000_001));
        assert_ne!(gec(1_700_000_000), gec(1_700_000_300));
        assert!(Options { voice: "voice'><script>".into(), ..Options::default() }.validate().is_err());
        assert!(Options { rate: 101, ..Options::default() }.validate().is_err());
        assert!(chunks("\u{0000}\u{0001}").is_err());
    }

    #[test]
    #[ignore]
    fn test_live_websocket_handshake() {
        use std::net::{TcpStream, ToSocketAddrs};
        use std::time::Duration;
        use tungstenite::client::IntoClientRequest;

        let now = chrono::Utc::now().timestamp();
        let connection = random_id().unwrap();
        let mut request = ws_url(&connection, now).into_client_request().unwrap();
        for (name, value) in [
            ("User-Agent", USER_AGENT.to_string()),
            ("Origin", ORIGIN.to_string()),
            ("Accept", "*/*".into()),
            ("Accept-Language", "en-US,en;q=0.9".into()),
            ("Pragma", "no-cache".into()),
            ("Cache-Control", "no-cache".into()),
            ("Cookie", format!("muid={};", random_id().unwrap())),
        ] {
            request.headers_mut().insert(
                tungstenite::http::HeaderName::from_bytes(name.as_bytes()).unwrap(),
                value.parse().unwrap(),
            );
        }

        let addresses = (HOST, 443).to_socket_addrs().unwrap();
        let mut stream = None;
        for address in addresses.take(3) {
            if let Ok(tcp) = TcpStream::connect_timeout(&address, Duration::from_secs(5)) {
                stream = Some(tcp);
                break;
            }
        }
        let tcp = stream.expect("TCP connect failed");
        tcp.set_read_timeout(Some(Duration::from_secs(15))).unwrap();
        tcp.set_write_timeout(Some(Duration::from_secs(15))).unwrap();

        let (mut socket, response) =
            tungstenite::client_tls_with_config(request, tcp, None, None).expect("Handshake failed");
        assert_eq!(response.status(), 101);

        socket.send(tungstenite::Message::Text(config(now).into())).unwrap();
        socket.send(tungstenite::Message::Text(ssml(&random_id().unwrap(), &Options::default(), "你好", now).into())).unwrap();
    }
}
