//! Tools that connect the assistant to the official website and the 超级小伴 forum.
//!
//! Pages are picked from a fixed list, so the assistant can never send the browser to an address of its own choosing.

use super::desktop_tools::{optional_bool, text_arg, Desktop};
use serde_json::Value;

/// The most a forum member can attach to one post.
const FORUM_ATTACHMENT_BYTES: u64 = 5 * 1024 * 1024;

pub const KINDS: [&str; 4] = ["wallpaper", "effect", "widget", "game"];

/// (target, address, name shown to the user)
pub const PAGES: [(&str, &str, &str); 9] = [
    ("site", "https://wallpaper.xiangban.xyz", "小伴桌面官网首页"),
    ("site_images", "https://wallpaper.xiangban.xyz/images", "官网的图片壁纸"),
    ("site_videos", "https://wallpaper.xiangban.xyz/videos", "官网的动态视频壁纸"),
    ("site_guide", "https://wallpaper.xiangban.xyz/guide", "官网的使用指南"),
    ("forum", "https://chat.xiangban.xyz", "超级小伴论坛首页"),
    ("forum_wallpaper", "https://chat.xiangban.xyz/c/xiaoban-wallpaper", "论坛的「小伴壁纸」板块"),
    ("forum_game", "https://chat.xiangban.xyz/c/xiaoban-game", "论坛的「小伴游戏」板块"),
    ("forum_app", "https://chat.xiangban.xyz/c/xiaoban-app", "论坛的「小伴App」板块"),
    ("forum_feedback", "https://chat.xiangban.xyz/c/site-feedback", "论坛的「建议反馈」板块"),
];

/// Where a shared work of this kind goes: wallpapers, effects and widgets to the wallpaper board, games to the game board.
fn board(kind: &str) -> (&'static str, &'static str) {
    let target = if kind == "game" { "forum_game" } else { "forum_wallpaper" };
    let (_, url, _) = PAGES.iter().find(|(name, _, _)| *name == target).expect("board targets are in PAGES");
    (url, if kind == "game" { "小伴游戏" } else { "小伴壁纸" })
}

fn size_text(bytes: u64) -> String {
    if bytes >= 1024 * 1024 { format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0)) } else { format!("{} KB", bytes.div_ceil(1024)) }
}

pub fn share(desktop: &dyn Desktop, args: &Value) -> Result<String, String> {
    let kind = text_arg(args, "kind")?;
    if !KINDS.contains(&kind) {
        return Err(format!("`kind` must be one of: {}", KINDS.join(", ")));
    }
    let id = text_arg(args, "id")?;
    let open_forum = optional_bool(args, "open_forum")?.unwrap_or(true);

    let Some((path, bytes)) = desktop.export_pack(kind, id)? else {
        return Ok("用户在保存对话框里取消了：资源包没有导出，论坛也没有打开。".into());
    };
    let (url, board_name) = board(kind);
    let mut out = format!("资源包已导出到 {path}（{}）。", size_text(bytes));
    if open_forum {
        match desktop.open_url(url) {
            Ok(()) => out.push_str(&format!("已在浏览器打开超级小伴论坛「{board_name}」板块：{url}。")),
            Err(error) => out.push_str(&format!("没能自动打开论坛（{error}），请让用户自己访问 {url}。")),
        }
    } else {
        out.push_str(&format!("没有打开论坛，板块地址是 {url}。"));
    }
    out.push_str("发帖要用户自己登录论坛来完成，你不能代发。请告诉用户：在论坛点“发帖”，写几句说明，上传预览图，再把刚导出的 .xiaoban 文件作为附件传上去。");
    if bytes > FORUM_ATTACHMENT_BYTES {
        out.push_str("提醒：资源包超过 5 MB，而论坛成员的单个附件上限是 5 MB，可能传不上去。");
    }
    Ok(out)
}

pub fn open(desktop: &dyn Desktop, args: &Value) -> Result<String, String> {
    let target = text_arg(args, "target")?;
    let Some((_, url, name)) = PAGES.iter().find(|(known, _, _)| *known == target) else {
        let known: Vec<&str> = PAGES.iter().map(|(known, _, _)| *known).collect();
        return Err(format!("unknown `target` {target:?}; use one of: {}", known.join(", ")));
    };
    desktop.open_url(url)?;
    Ok(format!("已在浏览器打开{name}：{url}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pages_stay_on_the_two_official_hosts() {
        for (target, url, _) in PAGES {
            let official = url.starts_with(crate::website::DEFAULT_SITE) || url.starts_with("https://chat.xiangban.xyz");
            assert!(official, "{target} points outside the official hosts: {url}");
        }
    }

    #[test]
    fn games_go_to_the_game_board_and_everything_else_to_the_wallpaper_board() {
        assert_eq!(board("game").0, "https://chat.xiangban.xyz/c/xiaoban-game");
        for kind in ["wallpaper", "effect", "widget"] {
            assert_eq!(board(kind).0, "https://chat.xiangban.xyz/c/xiaoban-wallpaper");
        }
    }

    #[test]
    fn sizes_are_readable() {
        assert_eq!(size_text(1), "1 KB");
        assert_eq!(size_text(1500), "2 KB");
        assert_eq!(size_text(6 * 1024 * 1024), "6.0 MB");
    }
}
