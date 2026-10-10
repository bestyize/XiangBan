use crate::error::AppError;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Email,
    Phone,
}

impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Email => "email",
            Kind::Phone => "phone",
        }
    }

    pub fn from_db(s: &str) -> Option<Self> {
        match s {
            "email" => Some(Kind::Email),
            "phone" => Some(Kind::Phone),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Identifier {
    pub kind: Kind,
    pub value: String,
}

impl Identifier {
    /// 含 `@` 视为邮箱，否则按中国大陆手机号处理。
    pub fn parse(raw: &str) -> Result<Self, AppError> {
        let raw = raw.trim();
        if raw.contains('@') {
            parse_email(raw).map(|value| Self { kind: Kind::Email, value })
        } else {
            parse_phone(raw).map(|value| Self { kind: Kind::Phone, value })
        }
    }

    pub fn masked(&self) -> String {
        mask(self.kind, &self.value)
    }
}

pub fn mask(kind: Kind, value: &str) -> String {
    match kind {
        Kind::Email => match value.split_once('@') {
            Some((local, domain)) => {
                let first: String = local.chars().take(1).collect();
                format!("{first}***@{domain}")
            }
            None => "***".into(),
        },
        Kind::Phone => {
            let digits = value.trim_start_matches("+86");
            if digits.len() == 11 {
                format!("+86{}****{}", &digits[..3], &digits[7..])
            } else {
                "***".into()
            }
        }
    }
}

fn parse_email(raw: &str) -> Result<String, AppError> {
    let bad = || AppError::BadRequest("邮箱格式不正确".into());
    let email = raw.to_ascii_lowercase();
    if email.len() > 254 || !email.is_ascii() {
        return Err(bad());
    }
    let (local, domain) = email.split_once('@').ok_or_else(bad)?;
    if local.is_empty() || local.len() > 64 || domain.contains('@') {
        return Err(bad());
    }
    let local_ok = local
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || "._%+-'".contains(c))
        && !local.starts_with('.')
        && !local.ends_with('.')
        && !local.contains("..");
    let labels: Vec<&str> = domain.split('.').collect();
    let domain_ok = labels.len() >= 2
        && labels.iter().all(|l| {
            !l.is_empty()
                && l.len() <= 63
                && !l.starts_with('-')
                && !l.ends_with('-')
                && l.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        })
        && labels.last().map(|tld| tld.len() >= 2).unwrap_or(false);
    if local_ok && domain_ok {
        Ok(email)
    } else {
        Err(bad())
    }
}

fn parse_phone(raw: &str) -> Result<String, AppError> {
    let cleaned: String = raw.chars().filter(|c| !matches!(c, ' ' | '-')).collect();
    let national = if let Some(rest) = cleaned.strip_prefix("+86") {
        rest
    } else if cleaned.len() == 13 && cleaned.starts_with("86") {
        &cleaned[2..]
    } else {
        cleaned.as_str()
    };
    let valid = national.len() == 11
        && national.starts_with('1')
        && national.as_bytes()[1] >= b'3'
        && national.chars().all(|c| c.is_ascii_digit());
    if valid {
        Ok(format!("+86{national}"))
    } else {
        Err(AppError::BadRequest("请输入正确的邮箱或中国大陆手机号".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_normalizes_and_validates() {
        assert_eq!(Identifier::parse("  Foo.Bar@Example.COM ").unwrap().value, "foo.bar@example.com");
        for bad in ["a@b", "a@@b.com", "@b.com", "a b@c.com", "a..b@c.com", ".a@c.com", "a@-c.com", "a@c.c", "é@c.com"] {
            assert!(Identifier::parse(bad).is_err(), "{bad} should be rejected");
        }
    }

    #[test]
    fn phone_normalizes_to_e164() {
        for ok in ["13800138000", "+86 138-0013-8000", "8613800138000"] {
            assert_eq!(Identifier::parse(ok).unwrap().value, "+8613800138000");
        }
        for bad in ["12800138000", "1380013800", "138001380000", "+1 415 555 0100", "abc"] {
            assert!(Identifier::parse(bad).is_err(), "{bad} should be rejected");
        }
    }

    #[test]
    fn masks_hide_the_middle() {
        assert_eq!(Identifier::parse("alice@example.com").unwrap().masked(), "a***@example.com");
        assert_eq!(Identifier::parse("13800138000").unwrap().masked(), "+86138****8000");
    }
}
