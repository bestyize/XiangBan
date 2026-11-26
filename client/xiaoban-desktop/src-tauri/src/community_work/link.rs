pub const PREFIX: &str = "xiaoban://community/run/";

#[derive(Debug, PartialEq)]
pub enum Work {
    Builtin(String),
    Idea(String),
    Pack(u64, i32),
}

impl Work {
    pub fn parse(raw: &str) -> Option<Self> {
        if raw.len() > 120 { return None; }
        let parts: Vec<_> = raw.strip_prefix(PREFIX)?.split('/').collect();
        match parts.as_slice() {
            [kind, slug] if (2..=40).contains(&slug.len())
                && slug.as_bytes()[0] != b'-'
                && slug.bytes().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-') => {
                match *kind {
                    "builtin" => Some(Self::Builtin((*slug).into())),
                    "idea" => Some(Self::Idea((*slug).into())),
                    _ => None,
                }
            }
            ["pack", id, version] => {
                let number: u64 = id.parse().ok()?;
                let revision: i32 = version.parse().ok()?;
                ((1..=9_007_199_254_740_991).contains(&number) && revision > 0
                    && number.to_string() == *id && revision.to_string() == *version)
                    .then_some(Self::Pack(number, revision))
            }
            _ => None,
        }
    }

    pub fn web_path(&self) -> String {
        match self {
            Self::Builtin(id) => format!("/create/run/{id}"),
            Self::Idea(id) => format!("/create/run/ideas/{id}"),
            Self::Pack(id, version) => format!("/create/run/{id}/{version}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_identity_and_pinned_version() {
        for (link, path) in [
            ("builtin/mbti", "/create/run/mbti"),
            ("idea/jiangshan-map", "/create/run/ideas/jiangshan-map"),
            ("pack/38/7", "/create/run/38/7"),
        ] {
            assert_eq!(Work::parse(&format!("{PREFIX}{link}")).unwrap().web_path(), path);
        }
    }

    #[test]
    fn rejects_other_actions_and_incomplete_or_noncanonical_targets() {
        assert_eq!(Work::parse("xiaoban://community/open/pack/38"), None);
        for suffix in ["idea/", "idea/Bad", "builtin/-foo", "idea/mbti?x=1", "pack/2", "pack/2/0",
                       "pack/2/01", "pack/2/2147483648", "pack/9007199254740992/1", "pack/2/3/4"] {
            assert_eq!(Work::parse(&format!("{PREFIX}{suffix}")), None);
        }
    }
}
