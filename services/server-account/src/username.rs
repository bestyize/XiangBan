use crate::error::AppError;

pub const MIN_LEN: usize = 4;
pub const MAX_LEN: usize = 20;

/// 用户名一旦设置不可修改，也不作为验证码登录的标识，所以必须和邮箱、手机号永远分得清：
/// 以字母开头，只含字母、数字、下划线。
const RESERVED_EXACT: &[&str] = &[
    "support", "service", "moderator", "staff", "guest", "null", "undefined", "anonymous", "nobody", "everyone",
    "help", "mod", "owner", "bot", "api", "www", "mail", "email",
];
/// 以这些开头的名字容易冒充官方或管理员。
const RESERVED_PREFIX: &[&str] = &["admin", "root", "system", "official", "xiaoban", "xiaban", "xiangban"];

pub struct Username {
    /// 展示用，保留用户输入的大小写。
    pub display: String,
    /// 唯一键，全小写。
    pub key: String,
}

pub fn parse(raw: &str) -> Result<Username, AppError> {
    let name = raw.trim();
    let len = name.chars().count();
    if !(MIN_LEN..=MAX_LEN).contains(&len) {
        return Err(AppError::BadRequest(format!("用户名需为 {MIN_LEN}-{MAX_LEN} 个字符")));
    }
    let mut chars = name.chars();
    if !chars.next().is_some_and(|c| c.is_ascii_alphabetic()) {
        return Err(AppError::BadRequest("用户名需以字母开头".into()));
    }
    if !chars.all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(AppError::BadRequest("用户名只能包含字母、数字和下划线".into()));
    }
    let key = name.to_ascii_lowercase();
    if RESERVED_EXACT.contains(&key.as_str()) || RESERVED_PREFIX.iter().any(|p| key.starts_with(p)) {
        return Err(AppError::BadRequest("这个用户名不能使用".into()));
    }
    Ok(Username { display: name.to_string(), key })
}

/// 登录标识里区分「用户名」与「邮箱 / 手机号」：邮箱含 `@`，手机号以数字或 `+` 开头，用户名以字母开头。
pub fn looks_like_username(raw: &str) -> bool {
    let raw = raw.trim();
    !raw.contains('@') && raw.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_letters_digits_and_underscores_and_keeps_display_case() {
        let u = parse("  Alice_01 ").unwrap();
        assert_eq!(u.display, "Alice_01");
        assert_eq!(u.key, "alice_01");
    }

    #[test]
    fn enforces_length_start_and_charset() {
        for bad in ["abc", "a23456789012345678901", "1abcd", "_abcd", "ab cd", "ab-cd", "用户名abcd", "a@b.cd", "13800138000"] {
            assert!(parse(bad).is_err(), "{bad}");
        }
        assert!(parse("abcd").is_ok());
        assert!(parse("a2345678901234567890").is_ok());
    }

    #[test]
    fn blocks_reserved_names_and_impersonation_prefixes() {
        for bad in ["support", "Moderator", "admin01", "ROOT_user", "xiaoban_official", "XiangBan2026", "system_msg"] {
            assert!(parse(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn tells_usernames_from_emails_and_phones() {
        assert!(looks_like_username("alice_01"));
        assert!(!looks_like_username("a@b.com"));
        assert!(!looks_like_username("13800138000"));
        assert!(!looks_like_username("+8613800138000"));
    }
}
