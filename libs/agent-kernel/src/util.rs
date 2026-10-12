use std::fmt::Write;
#[cfg(any(feature = "skills", feature = "memory", feature = "fs"))]
use std::path::Path;

pub(crate) fn nearest_boundary(text: &str, index: usize, forward: bool) -> usize {
    let mut index = index.min(text.len());
    while !text.is_char_boundary(index) {
        index = if forward { index + 1 } else { index - 1 };
    }
    index
}

/// Keeps the start and end of `text`, since errors and summaries usually sit at the ends.
pub(crate) fn truncate_middle(text: &str, max: usize) -> String {
    if text.len() <= max {
        return text.to_string();
    }
    let head = nearest_boundary(text, max * 2 / 3, false);
    let tail = nearest_boundary(text, text.len() - (max - max * 2 / 3), true);
    format!("{}\n[... {} bytes truncated ...]\n{}", &text[..head], tail - head, &text[tail..])
}

/// Injective, path-safe form of an arbitrary id: anything outside `[a-z0-9_-]` becomes `%XX`.
/// Uppercase is escaped too, because macOS and Windows file systems ignore case by default.
pub(crate) fn file_safe(id: &str) -> String {
    let mut out = String::with_capacity(id.len());
    for b in id.bytes() {
        match b {
            b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-' => out.push(b as char),
            _ => {
                let _ = write!(out, "%{b:02X}");
            }
        }
    }
    out
}

#[cfg(feature = "skills")]
pub(crate) fn now_millis() -> u128 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_millis())
}

/// Names for skills and notes: 1 to 64 characters of `a-z`, `0-9` and `-`, not starting or ending with `-`.
#[cfg(any(feature = "skills", feature = "memory"))]
pub(crate) fn valid_slug(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && !name.starts_with('-')
        && !name.ends_with('-')
        && name.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

/// Writes through a temp file in the same directory and renames, so readers never see a partial file.
/// An existing file keeps its permissions.
#[cfg(any(feature = "skills", feature = "memory", feature = "fs"))]
pub(crate) fn atomic_write(path: &Path, data: &[u8]) -> std::io::Result<()> {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let name = path.file_name().map_or_else(|| "file".into(), |n| n.to_string_lossy().into_owned());
    let tmp = path.with_file_name(format!(".{name}.{}.{}.tmp", std::process::id(), COUNTER.fetch_add(1, Ordering::Relaxed)));
    let result = (|| {
        let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(&tmp)?;
        std::io::Write::write_all(&mut file, data)?;
        file.sync_all()?;
        if let Ok(meta) = std::fs::metadata(path) {
            let _ = std::fs::set_permissions(&tmp, meta.permissions());
        }
        std::fs::rename(&tmp, path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}
