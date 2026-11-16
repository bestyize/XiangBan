//! Converts an ordinary local HTML document into a portable V2 app pack.

use super::pack::Files;
use crate::standalone_html::ENTRY;
use serde_json::{json, Value};
use std::fmt::Write;

const READY_SCRIPT: &str = r#"<script>
document.addEventListener('DOMContentLoaded', () => {
  requestAnimationFrame(() => window.xb?.ready?.());
}, { once: true });
</script>
"#;

pub const LOCAL_HTML: &str = "standaloneHtml";

const ICON: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="192" height="192" viewBox="0 0 192 192">
  <rect width="192" height="192" rx="36" fill="#324475"/>
  <path d="M70 71 43 96l27 25m52-50 27 25-27 25m-17-57-18 64" fill="none" stroke="#e4ebff" stroke-width="10" stroke-linecap="round" stroke-linejoin="round"/>
</svg>"##;

pub(crate) fn check_document(html: &str) -> Result<(), String> {
    if html.trim_start_matches('\u{feff}').trim().is_empty() || html.contains('\0') {
        return Err("请选择非空的 UTF-8 HTML 文件".into());
    }
    Ok(())
}

pub(crate) fn identity(stem: &str, fallback: &str) -> Result<(String, String), String> {
    let mut name: String = stem
        .trim()
        .chars()
        .filter(|c| !c.is_control() && !matches!(c, '\u{fffe}' | '\u{ffff}'))
        .take(24)
        .collect();
    name.truncate(name.trim_end().len());
    if name.is_empty() {
        name = fallback.into();
    }

    let mut random = [0u8; 16];
    getrandom::fill(&mut random).map_err(|e| format!("无法生成作品 id：{e}"))?;
    let mut id = String::from("html-");
    for byte in random {
        write!(id, "{byte:02x}").expect("writing to a String cannot fail");
    }
    Ok((id, name))
}

pub fn manifest_and_artwork(stem: &str, game: bool) -> Result<(Value, Files), String> {
    let (kind, category, fallback) = if game { ("game", "casual", "HTML 游戏") } else { ("app", "tool", "HTML 作品") };
    let (id, name) = identity(stem, fallback)?;
    let manifest = json!({
        "schemaVersion": 3, "runtimeProtocol": 2, "id": id, "name": name,
        "kind": kind, "category": category, "description": "从 HTML 导入的作品",
        "version": "1.0.0", "entry": ENTRY, "orientation": "any",
        "input": ["touch", "mouse", "keyboard"], "capabilities": [],
        "icon": "icon.svg", "cover": "cover.svg",
    });
    let title = name
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    let cover = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="640" height="360" viewBox="0 0 640 360">
  <rect width="640" height="360" fill="#324475"/>
  <path d="M286 106 250 140l36 34m68-68 36 34-36 34m-22-78-24 88" fill="none" stroke="#e4ebff" stroke-width="12" stroke-linecap="round" stroke-linejoin="round"/>
  <text x="320" y="265" fill="#fff" font-family="sans-serif" font-size="23" text-anchor="middle">{title}</text>
</svg>"##
    );
    let files = vec![
        ("icon.svg".into(), ICON.as_bytes().to_vec()),
        ("cover.svg".into(), cover.into_bytes()),
    ];
    Ok((manifest, files))
}

pub fn prepare_document(bytes: Vec<u8>) -> Result<Vec<u8>, String> {
    let mut html = String::from_utf8(bytes).map_err(|_| "作品需要使用 UTF-8 HTML")?;
    // Preserve the original document mode and register the handshake before author scripts.
    let document = html.strip_prefix('\u{feff}').unwrap_or(&html);
    let offset = html.len() - document.len() + prolog_end(document);
    html.insert_str(offset, READY_SCRIPT);
    Ok(html.into_bytes())
}

fn html_space(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\r' | '\u{c}')
}

fn prolog_end(html: &str) -> usize {
    let mut rest = html;
    loop {
        rest = rest.trim_start_matches(html_space);
        if let Some(end) = rest
            .strip_prefix("<!--")
            .and_then(|comment| comment.find("-->"))
        {
            rest = &rest[end + 7..];
            continue;
        }
        let is_prolog_tag = ["<!doctype", "<html", "<head"].into_iter().any(|tag| {
            rest.get(..tag.len())
                .is_some_and(|prefix| prefix.eq_ignore_ascii_case(tag))
                && rest[tag.len()..]
                    .chars()
                    .next()
                    .is_some_and(|c| html_space(c) || c == '>' || c == '/')
        });
        if !is_prolog_tag {
            return html.len() - rest.len();
        }
        let mut quote = None;
        let end = rest.char_indices().find_map(|(index, c)| match (quote, c) {
            (None, '>') => Some(index + 1),
            (None, '\'' | '"') => {
                quote = Some(c);
                None
            }
            (Some(open), _) if c == open => {
                quote = None;
                None
            }
            _ => None,
        });
        match end {
            Some(end) => rest = &rest[end..],
            None => return html.len() - rest.len(),
        }
    }
}
