//! 社区深链接：`xiaoban://community/apply/{kind}/{pack_id}?version=&sha256=` 与 `xiaoban://community/open/{target}/{id}`。
//! 链接只能请求“展示什么”，应用任何内容都要用户在确认窗口里点头；哈希（若带）用来锁定用户在网页上看到的那一版。

use serde_json::{json, Value};
use url::Url;

const KINDS: &[&str] = &[
    "effect", "reminder", "routine", "task", "goal", "widget", "game", "wallpaper", "skill", "expert", "workflow",
];
const TARGETS: &[&str] = &["pack", "topic", "scheme", "project", "weekly"];

#[derive(Debug, PartialEq)]
pub enum Link {
    Apply { kind: String, pack_id: i64, version: Option<i32>, sha256: Option<String> },
    Open { target: String, id: String },
}

impl Link {
    pub fn to_json(&self) -> Value {
        match self {
            Link::Apply { kind, pack_id, version, sha256 } => {
                json!({ "action": "apply", "kind": kind, "packId": pack_id, "version": version, "sha256": sha256 })
            }
            Link::Open { target, id } => json!({ "action": "open", "target": target, "id": id }),
        }
    }
}

fn hex64(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit())
}

/// 周标识、项目 id 与数字编号：只允许安全字符，防止拼出别的路径。
fn safe_id(value: &str) -> bool {
    !value.is_empty() && value.len() <= 64 && value.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
}

pub fn parse_link(text: &str) -> Result<Link, String> {
    let bad = || "无效的小伴社区链接".to_string();
    if text.len() > 600 {
        return Err(bad());
    }
    let url = Url::parse(text).map_err(|_| bad())?;
    if url.scheme() != "xiaoban" || url.host_str() != Some("community") || !url.username().is_empty() || url.port().is_some() {
        return Err(bad());
    }
    let parts: Vec<&str> = url.path_segments().map(|s| s.collect()).unwrap_or_default();
    match parts[..] {
        ["apply", kind, id] => {
            if !KINDS.contains(&kind) {
                return Err(bad());
            }
            let pack_id: i64 = id.parse().ok().filter(|n| *n > 0).ok_or_else(bad)?;
            let (mut version, mut sha256) = (None, None);
            for (key, value) in url.query_pairs() {
                match key.as_ref() {
                    "version" => version = Some(value.parse::<i32>().ok().filter(|v| *v > 0).ok_or_else(bad)?),
                    "sha256" if hex64(&value) => sha256 = Some(value.to_ascii_lowercase()),
                    "sha256" => return Err(bad()),
                    _ => {}
                }
            }
            Ok(Link::Apply { kind: kind.to_string(), pack_id, version, sha256 })
        }
        ["open", target, id] if TARGETS.contains(&target) && safe_id(id) => {
            Ok(Link::Open { target: target.to_string(), id: id.to_string() })
        }
        _ => Err(bad()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_links_are_strict() {
        let sha = "A".repeat(64);
        let ok = parse_link(&format!("xiaoban://community/apply/effect/12?version=3&sha256={sha}")).unwrap();
        assert_eq!(ok, Link::Apply { kind: "effect".into(), pack_id: 12, version: Some(3), sha256: Some("a".repeat(64)) });
        assert_eq!(
            parse_link("xiaoban://community/apply/reminder/7").unwrap(),
            Link::Apply { kind: "reminder".into(), pack_id: 7, version: None, sha256: None }
        );
        for bad in [
            "https://community/apply/effect/1",
            "xiaoban://market/apply/effect/1",
            "xiaoban://community/apply/nope/1",
            "xiaoban://community/apply/effect/0",
            "xiaoban://community/apply/effect/-1",
            "xiaoban://community/apply/effect/abc",
            "xiaoban://community/apply/effect/1/extra",
            "xiaoban://community/apply/effect",
            "xiaoban://community/apply/effect/1?version=0",
            "xiaoban://community/apply/effect/1?version=../x",
            "xiaoban://community/apply/effect/1?sha256=short",
            "xiaoban://user@community/apply/effect/1",
            "xiaoban://community:80/apply/effect/1",
            "xiaoban://community/run/effect/1",
        ] {
            assert!(parse_link(bad).is_err(), "{bad}");
        }
        assert!(parse_link(&format!("xiaoban://community/apply/effect/1?x={}", "a".repeat(700))).is_err());
    }

    #[test]
    fn open_links_only_name_known_targets() {
        assert_eq!(
            parse_link("xiaoban://community/open/weekly/2026-W44").unwrap(),
            Link::Open { target: "weekly".into(), id: "2026-W44".into() }
        );
        assert!(parse_link("xiaoban://community/open/topic/123").is_ok());
        assert!(parse_link("xiaoban://community/open/admin/1").is_err());
        assert!(parse_link("xiaoban://community/open/topic/..%2Fx").is_err());
        assert!(parse_link("xiaoban://community/open/topic/").is_err());
    }
}
