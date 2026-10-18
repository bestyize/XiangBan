use crate::error::AppError;

pub const MAX_WIDTH: usize = 20;

/// 用户名一旦设置不可修改，也不作为验证码登录的标识，所以必须和邮箱、手机号永远分得清：
/// 以英文字母或汉字开头，只含英文字母、数字、下划线和汉字。
/// 下限：含汉字时汉字加字母至少 2 个，纯英文至少 3 个字母（数字和下划线不计入）；上限：汉字算 2，共 20。
/// 汉字只收「中日韩统一表意文字」基本区（U+4E00-U+9FFF），不含兼容字符和全角字母，避免长得一样却不相等的名字。
const RESERVED_EXACT: &[&str] = &[
    "support", "service", "moderator", "staff", "guest", "null", "undefined", "anonymous", "nobody", "everyone",
    "help", "mod", "owner", "bot", "api", "www", "mail", "email",
];
/// 以这些开头的名字容易冒充官方或管理员。
const RESERVED_PREFIX: &[&str] = &[
    "admin", "root", "system", "official", "xiaoban", "xiaban", "xiangban",
    "管理员", "客服", "官方", "系统", "站长", "版主", "小伴", "相伴", "下班了么",
];

pub fn is_han(c: char) -> bool {
    ('\u{4E00}'..='\u{9FFF}').contains(&c)
}

fn is_start(c: char) -> bool {
    c.is_ascii_alphabetic() || is_han(c)
}

fn width(name: &str) -> usize {
    name.chars().map(|c| if is_han(c) { 2 } else { 1 }).sum()
}

pub struct Username {
    /// 展示用，保留用户输入的大小写。
    pub display: String,
    /// 唯一键，全小写。
    pub key: String,
}

pub fn parse(raw: &str) -> Result<Username, AppError> {
    let name = raw.trim();
    let mut chars = name.chars();
    if !chars.next().is_some_and(is_start) {
        return Err(AppError::BadRequest("用户名需以字母或汉字开头".into()));
    }
    if !chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || is_han(c)) {
        return Err(AppError::BadRequest("用户名只能包含字母、数字、下划线和汉字".into()));
    }
    let han = name.chars().filter(|c| is_han(*c)).count();
    let letters = name.chars().filter(char::is_ascii_alphabetic).count();
    if (han > 0 && han + letters < 2) || (han == 0 && letters < 3) {
        return Err(AppError::BadRequest("用户名至少要有 2 个汉字或 3 个英文字母".into()));
    }
    if width(name) > MAX_WIDTH {
        return Err(AppError::BadRequest(format!(
            "用户名最长 {MAX_WIDTH} 个字符（汉字算 2 个，最多 {} 个汉字）",
            MAX_WIDTH / 2
        )));
    }
    let key = name.to_ascii_lowercase();
    if RESERVED_EXACT.contains(&key.as_str()) || RESERVED_PREFIX.iter().any(|p| key.starts_with(p)) {
        return Err(AppError::BadRequest("这个用户名不能使用".into()));
    }
    Ok(Username { display: name.to_string(), key })
}

/// 登录标识里区分「用户名」与「邮箱 / 手机号」：邮箱含 `@`，手机号以数字或 `+` 开头，用户名以字母或汉字开头。
pub fn looks_like_username(raw: &str) -> bool {
    let raw = raw.trim();
    !raw.contains('@') && raw.chars().next().is_some_and(is_start)
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
    fn enforces_minimum_start_charset_and_maximum() {
        for bad in [
            "ab", "ab1", "ab_", "a_b", "1abc", "_abc", "ab cd", "ab-cd", "a@b.cd", "13800138000", "用", "用1", "用_", "张_1", "张三！", "用户 名",
            "ＡＢＣＤ", "1用户名", "_用户名", "a23456789012345678901", "用户名用户名用户名用户",
        ] {
            assert!(parse(bad).is_err(), "{bad}");
        }
        for ok in ["abc", "Abc1", "abc45678901234567890", "张三", "张a", "a张", "ab张"] {
            assert!(parse(ok).is_ok(), "{ok}");
        }
    }

    #[test]
    fn accepts_chinese_names_by_width() {
        let u = parse(" 张三 ").unwrap();
        assert_eq!(u.display, "张三");
        assert_eq!(u.key, "张三");
        for ok in ["用户名abcd", "张三丰_01", "A小伴同学", "用户名用户名用户名用", "李雷"] {
            assert!(parse(ok).is_ok(), "{ok}");
        }
        assert_eq!(parse("Li王").unwrap().key, "li王");
    }

    #[test]
    fn blocks_reserved_names_and_impersonation_prefixes() {
        for bad in ["support", "Moderator", "admin01", "ROOT_user", "xiaoban_official", "XiangBan2026", "system_msg", "管理员小李", "官方客服", "小伴助手", "相伴团队"] {
            assert!(parse(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn tells_usernames_from_emails_and_phones() {
        assert!(looks_like_username("alice_01"));
        assert!(!looks_like_username("a@b.com"));
        assert!(!looks_like_username("13800138000"));
        assert!(!looks_like_username("+8613800138000"));
        assert!(looks_like_username("张三丰"));
        assert!(!looks_like_username("张三@qq.com"));
    }
}
