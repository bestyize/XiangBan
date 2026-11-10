//! The mobile session log stores references to private images. Hydrate only the
//! selected context and cap the amount loaded for one model request.

use agent_kernel::{ChatRequest, HarnessError, Message, Part, Role};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const REF_PREFIX: &str = "xiaoban-attachment:";
const MAX_IMAGE_BYTES: u64 = 2_800_000;
const MAX_SESSION_MEDIA_BYTES: u64 = 128 * 1024 * 1024;
static NEXT_IMAGE: AtomicU64 = AtomicU64::new(0);

fn directory(root: &Path, session: &str) -> Result<PathBuf, HarnessError> {
    if !crate::policy::valid_session(session) {
        return Err(HarnessError::Store("无效的图片会话".into()));
    }
    Ok(root.join(crate::policy::session_component(session)))
}

fn storage_error(error: std::io::Error) -> HarnessError {
    HarnessError::Store(format!("无法保存对话图片: {error}"))
}

pub fn compact(root: &Path, session: &str, messages: &mut [Message]) -> Result<(), HarnessError> {
    let directory = directory(root, session)?;
    fs::create_dir_all(&directory).map_err(storage_error)?;
    let stored = fs::read_dir(&directory).map_err(storage_error)?
        .filter_map(Result::ok).filter_map(|entry| entry.metadata().ok())
        .filter(|meta| meta.is_file()).map(|meta| meta.len()).sum::<u64>();
    let additional = messages.iter().flat_map(|message| &message.parts).filter_map(|part| match part {
        Part::Image { url } if url.starts_with("data:image/") => Some(url.len() as u64),
        _ => None,
    }).sum::<u64>();
    if stored.saturating_add(additional) > MAX_SESSION_MEDIA_BYTES {
        return Err(HarnessError::Store("此对话的图片存储已达上限，请使用新对话。".into()));
    }
    for part in messages.iter_mut().flat_map(|message| &mut message.parts) {
        let Part::Image { url } = part else { continue };
        if !url.starts_with("data:image/") { continue; }
        if url.len() as u64 > MAX_IMAGE_BYTES {
            return Err(HarnessError::Store("对话图片超过大小上限".into()));
        }
        let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
        let name = format!("{time:x}-{:x}-{:x}", std::process::id(), NEXT_IMAGE.fetch_add(1, Ordering::Relaxed));
        let temp = directory.join(format!("{name}.tmp"));
        let saved = (|| {
            let mut file = fs::File::create_new(&temp)?;
            file.write_all(url.as_bytes())?;
            file.sync_all()?;
            fs::rename(&temp, directory.join(&name))
        })();
        if let Err(error) = saved {
            let _ = fs::remove_file(temp);
            return Err(storage_error(error));
        }
        *url = format!("{REF_PREFIX}{name}");
    }
    Ok(())
}

pub fn has_references(messages: &[Message]) -> bool {
    messages.iter().flat_map(|message| &message.parts)
        .any(|part| matches!(part, Part::Image { url } if url.starts_with(REF_PREFIX)))
}

pub fn hydrate(root: &Path, session: &str, request: &mut ChatRequest) -> Result<(), HarnessError> {
    let directory = directory(root, session)?;
    let current = request.messages.iter().rposition(|message| message.role == Role::User && !message.synthetic);
    let mut remaining: u64 = 8 * 1024 * 1024 - 4096;
    for (index, message) in request.messages.iter_mut().enumerate().rev() {
        for part in &mut message.parts {
            let Part::Image { url } = part else { continue };
            let Some(name) = url.strip_prefix(REF_PREFIX) else {
                remaining = remaining.saturating_sub(url.len() as u64);
                continue;
            };
            let stored = if !name.is_empty() && name.len() <= 100 &&
                name.bytes().all(|byte| byte.is_ascii_hexdigit() || byte == b'-')
            {
                let path = directory.join(name);
                fs::metadata(&path).ok()
                    .filter(|meta| meta.is_file() && (1..=MAX_IMAGE_BYTES.min(remaining)).contains(&meta.len()))
                    .and_then(|_| fs::read_to_string(path).ok())
                    .filter(|data| data.starts_with("data:image/"))
            } else { None };
            if let Some(data) = stored {
                remaining -= data.len() as u64;
                *url = data;
            } else if Some(index) == current {
                return Err(HarnessError::Store("无法读取当前消息的图片，请重新选择图片。".into()));
            } else {
                *part = Part::Text { text: "[历史图片已省略，可重新引用。]".into() };
            }
        }
    }
    Ok(())
}
