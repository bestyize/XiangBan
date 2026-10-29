//! 手机语音对话的纯逻辑部分：把手机分片上传的录音拼起来、把小伴的回答整理成适合朗读的文字、把合成好的语音切成分片。
//! 识别（SenseVoice）和合成（Edge TTS）由 `phone_link.rs` 调桌面已有的 `asr` / `tts` 模块完成，这里不碰它们，方便单独测试。

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// 手机上传的录音上限：20 秒、16kHz、单声道、16 位小端 PCM（和识别引擎的上限一致）。
pub const MAX_INPUT_BYTES: usize = 20 * 16_000 * 2;
/// 一段录音最多几个分片（每片手机侧约 90 KB）。
pub const MAX_CHUNKS: usize = 12;
/// 回给手机的语音每片的字节数（编码成 base64 后约 120 KB，低于单条消息上限）。
pub const AUDIO_CHUNK_BYTES: usize = 90_000;
/// 朗读的字数上限；更长的只读开头，后面请看屏幕。
pub const MAX_SPOKEN_CHARS: usize = 600;
/// 一段录音从第一片到最后一片的最长间隔，超过就丢掉。
const STALE_AFTER: Duration = Duration::from_secs(60);

struct Partial {
    next: usize,
    bytes: Vec<u8>,
    started: Instant,
}

/// 把手机按顺序发来的分片拼成一段完整录音。
#[derive(Default)]
pub struct Assembler {
    items: HashMap<String, Partial>,
}

impl Assembler {
    /// 收一片。最后一片到了返回完整的录音；还没收完返回 `None`；顺序不对、太长、太多片都报错并丢掉这段。
    pub fn push(&mut self, key: &str, seq: usize, last: bool, data: &[u8]) -> Result<Option<Vec<u8>>, String> {
        self.items.retain(|_, p| p.started.elapsed() < STALE_AFTER);
        if seq == 0 {
            self.items.insert(key.to_string(), Partial { next: 0, bytes: Vec::new(), started: Instant::now() });
        }
        let Some(part) = self.items.get_mut(key) else { return Err("录音没有从头开始，请重录".into()) };
        if seq != part.next || seq >= MAX_CHUNKS {
            self.items.remove(key);
            return Err("录音分片顺序不对或太多，请重录".into());
        }
        part.bytes.extend_from_slice(data);
        part.next += 1;
        if part.bytes.len() > MAX_INPUT_BYTES {
            self.items.remove(key);
            return Err("录音太长，最多 20 秒".into());
        }
        if last {
            let done = self.items.remove(key).map(|p| p.bytes).unwrap_or_default();
            return Ok(Some(done));
        }
        Ok(None)
    }
}

/// 16 位小端 PCM → 识别引擎要的 -1..1 浮点。字节数是奇数就报错。
pub fn pcm16_to_f32(bytes: &[u8]) -> Result<Vec<f32>, String> {
    if bytes.len() % 2 != 0 {
        return Err("录音数据长度不对".into());
    }
    Ok(bytes.chunks_exact(2).map(|b| i16::from_le_bytes([b[0], b[1]]) as f32 / 32768.0).collect())
}

/// 把回答整理成能念出来的话：去掉代码块、链接地址和 Markdown 符号，压缩空白，太长就截短。
pub fn speech_text(markdown: &str) -> String {
    let mut lines = Vec::new();
    let mut in_code = false;
    for raw in markdown.lines() {
        let line = raw.trim();
        if line.starts_with("```") || line.starts_with("~~~") {
            in_code = !in_code;
            continue;
        }
        if in_code || line.is_empty() || line.chars().all(|c| matches!(c, '-' | '*' | '_' | '=' | ' ')) {
            continue;
        }
        let line = line.trim_start_matches(['#', '>', ' ']);
        let line = line.strip_prefix("- ").or_else(|| line.strip_prefix("* ")).or_else(|| line.strip_prefix("+ ")).unwrap_or(line);
        lines.push(strip_inline(line));
    }
    let joined = lines.join("。").replace("。。", "。").replace("：。", "：").replace("；。", "；");
    let text: String = joined.split_whitespace().collect::<Vec<_>>().join(" ");
    if text.chars().count() <= MAX_SPOKEN_CHARS {
        return text;
    }
    let cut: String = text.chars().take(MAX_SPOKEN_CHARS).collect();
    format!("{cut}……后面的内容请看屏幕。")
}

/// 行内：`[文字](地址)` 只留文字，`**粗体**`、`` `代码` ``去掉符号。
fn strip_inline(line: &str) -> String {
    let chars: Vec<char> = line.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '[' => {
                // [label](url)
                if let Some(close) = chars[i..].iter().position(|c| *c == ']') {
                    let end = i + close;
                    if chars.get(end + 1) == Some(&'(') {
                        if let Some(paren) = chars[end..].iter().position(|c| *c == ')') {
                            out.extend(&chars[i + 1..end]);
                            i = end + paren + 1;
                            continue;
                        }
                    }
                }
                out.push('[');
            }
            '*' | '`' | '_' if chars.get(i + 1) != Some(&' ') || chars[i] == '`' => {}
            c => out.push(c),
        }
        i += 1;
    }
    out
}

/// 把合成好的语音按固定大小切成分片，空数据没有分片。
pub fn audio_chunks(audio: &[u8]) -> Vec<&[u8]> {
    audio.chunks(AUDIO_CHUNK_BYTES).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunks_are_put_back_in_order() {
        let mut a = Assembler::default();
        assert_eq!(a.push("p/1", 0, false, &[1, 2]).unwrap(), None);
        assert_eq!(a.push("p/1", 1, true, &[3, 4]).unwrap(), Some(vec![1, 2, 3, 4]));
        // 同一个编号可以再用
        assert_eq!(a.push("p/1", 0, true, &[9, 9]).unwrap(), Some(vec![9, 9]));
    }

    #[test]
    fn out_of_order_or_oversized_recordings_are_dropped() {
        let mut a = Assembler::default();
        assert!(a.push("p/1", 1, true, &[1]).is_err(), "没有第 0 片");
        a.push("p/2", 0, false, &[1]).unwrap();
        assert!(a.push("p/2", 2, false, &[1]).is_err(), "跳片");
        assert!(a.push("p/2", 1, true, &[1]).is_err(), "出错后整段作废");
        a.push("p/3", 0, false, &vec![0; MAX_INPUT_BYTES]).unwrap();
        assert!(a.push("p/3", 1, true, &[0]).is_err(), "太长");
        let mut many = Assembler::default();
        for i in 0..MAX_CHUNKS {
            assert!(many.push("p/4", i, false, &[0, 0]).unwrap().is_none());
        }
        assert!(many.push("p/4", MAX_CHUNKS, true, &[0, 0]).is_err(), "片数太多");
    }

    #[test]
    fn pcm_becomes_floats_and_odd_lengths_fail() {
        let samples = pcm16_to_f32(&[0x00, 0x00, 0x00, 0x40, 0x00, 0xC0]).unwrap();
        assert_eq!(samples, vec![0.0, 0.5, -0.5]);
        assert!(pcm16_to_f32(&[1, 2, 3]).is_err());
    }

    #[test]
    fn answers_become_speakable() {
        let md = "# 结果\n\n- **第一点**：看 [文档](https://a.b/c)\n- 第二点用 `npm test`\n\n```bash\nnpm test\n```\n\n---\n完成了。";
        assert_eq!(speech_text(md), "结果。第一点：看 文档。第二点用 npm test。完成了。");
        let long = "字".repeat(MAX_SPOKEN_CHARS + 50);
        let spoken = speech_text(&long);
        assert!(spoken.ends_with("后面的内容请看屏幕。") && spoken.chars().count() < MAX_SPOKEN_CHARS + 20);
        assert_eq!(speech_text("```\nonly code\n```"), "");
    }

    #[test]
    fn audio_is_cut_into_pieces_that_fit_a_message() {
        let audio = vec![7u8; AUDIO_CHUNK_BYTES * 2 + 5];
        let parts = audio_chunks(&audio);
        assert_eq!(parts.iter().map(|p| p.len()).collect::<Vec<_>>(), vec![AUDIO_CHUNK_BYTES, AUDIO_CHUNK_BYTES, 5]);
        assert!(audio_chunks(&[]).is_empty());
        // base64 后仍低于 240 KiB 的发送控制线
        assert!(AUDIO_CHUNK_BYTES * 4 / 3 + 400 < 240 * 1024);
    }
}
