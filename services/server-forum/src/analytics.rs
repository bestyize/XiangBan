//! 匿名埋点：只记“某个匿名设备某天做过某件事几次”，不关联账号，不记内容、搜索词、文件名，也不保存行为流。
//!
//! 设备标识由客户端随机生成，服务端只存它的 SHA-256（加盐），换个服务端密钥就无法再对应；
//! 事件名和维度值都来自白名单，客户端不能塞自由文本进来。

use ring::digest;

/// 允许上报的事件。来自 PRD §8.2 的漏斗：看到、行动、创作、社区。
pub const EVENTS: &[&str] = &[
    // 网页：看到
    "view_home", "view_weekly", "view_weekly_issue", "view_project", "view_pack", "view_discuss", "search",
    // 网页：行动
    "apply_click", "download_click", "share_copy_link", "share_poster",
    // 创作
    "publish_start", "publish_done",
    // 桌面：社区页与一键应用
    "desktop_community_open", "desktop_apply_open", "desktop_apply_confirm", "desktop_apply_success", "desktop_apply_cancel",
    "desktop_scheme_replicate",
];

/// 维度（可选）：内容类型，或来源。每个事件至多带一个。
pub const DIMS: &[&str] = &[
    "effect", "reminder", "routine", "task", "goal", "widget", "game", "wallpaper", "skill", "expert", "workflow",
    "web", "desktop", "link", "weekly", "pack", "topic", "scheme", "project",
];

pub fn valid_device(id: &str) -> bool {
    (16..=64).contains(&id.len()) && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

/// 设备标识的哈希：加盐，只留前 16 字节的十六进制。
pub fn hash_device(device: &str, salt: &[u8]) -> String {
    let mut ctx = digest::Context::new(&digest::SHA256);
    ctx.update(salt);
    ctx.update(device.as_bytes());
    ctx.finish().as_ref()[..16].iter().map(|b| format!("{b:02x}")).collect()
}

/// 校验一条事件，返回（事件名，维度，空维度记为 "-"）。不在白名单的整条丢弃。
pub fn clean_event<'a>(name: &'a str, dim: Option<&'a str>) -> Option<(&'a str, &'a str)> {
    if !EVENTS.contains(&name) {
        return None;
    }
    match dim {
        None | Some("") => Some((name, "-")),
        Some(d) if DIMS.contains(&d) => Some((name, d)),
        Some(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_whitelisted_events_and_dimensions_pass() {
        assert_eq!(clean_event("view_pack", Some("effect")), Some(("view_pack", "effect")));
        assert_eq!(clean_event("search", None), Some(("search", "-")));
        assert_eq!(clean_event("search", Some("")), Some(("search", "-")));
        assert_eq!(clean_event("view_pack", Some("我的文件名.zip")), None, "维度不能是自由文本");
        assert_eq!(clean_event("keylogger", None), None);
    }

    #[test]
    fn device_ids_are_validated_and_hashed_with_a_salt() {
        assert!(valid_device("0123456789abcdef-0123"));
        assert!(!valid_device("short"));
        assert!(!valid_device("has spaces in it and is long enough"));
        assert!(!valid_device(&"a".repeat(65)));
        let a = hash_device("0123456789abcdef-0123", b"salt-one");
        assert_eq!(a.len(), 32);
        assert_eq!(a, hash_device("0123456789abcdef-0123", b"salt-one"));
        assert_ne!(a, hash_device("0123456789abcdef-0123", b"salt-two"), "换盐后无法对应");
        assert_ne!(a, hash_device("0123456789abcdef-0124", b"salt-one"));
    }
}
