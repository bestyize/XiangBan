//! 兑换码的格式：16 个字符，前 15 个随机，最后 1 个是校验位。输错一个字符时能在查库之前就发现，也方便用户自查。
//! 字母表去掉了容易看混的 0/O 和 1/I，剩 32 个，每个字符正好 5 比特，随机取值没有偏差。

use crate::keys::random_bytes;

const ALPHABET: &[u8; 32] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
pub const CODE_LEN: usize = 16;
const BODY_LEN: usize = CODE_LEN - 1;

/// 按位置加权求和再模 32。权重都是奇数，与 32 互质，所以任何单个字符写错都一定会被发现。
fn checksum(values: &[u8]) -> u8 {
    let sum: u32 = values.iter().enumerate().map(|(i, v)| (2 * i as u32 + 1) * u32::from(*v)).sum();
    (sum % 32) as u8
}

fn render(values: &[u8]) -> String {
    values.iter().map(|v| ALPHABET[usize::from(*v)] as char).collect()
}

/// 生成一个规范化的码（16 个字符，不含分隔符）。
pub fn generate() -> String {
    let mut values: Vec<u8> = random_bytes::<BODY_LEN>().iter().map(|b| b & 0x1f).collect();
    values.push(checksum(&values));
    render(&values)
}

/// 给人看的样子：四个一组，用连字符隔开。
pub fn display(canonical: &str) -> String {
    canonical
        .as_bytes()
        .chunks(4)
        .map(|chunk| String::from_utf8_lossy(chunk).into_owned())
        .collect::<Vec<_>>()
        .join("-")
}

/// 把用户输入整理成规范形式；长度、字符或校验位不对时返回 `None`。
/// 容忍小写、空格、各种连字符、全角字符和复制时带进来的零宽字符。
pub fn canonical(raw: &str) -> Option<String> {
    let mut values = Vec::with_capacity(CODE_LEN);
    for ch in raw.chars() {
        let ch = match ch {
            '\u{FF01}'..='\u{FF5E}' => char::from_u32(u32::from(ch) - 0xFEE0).unwrap_or(ch),
            other => other,
        };
        if ch.is_whitespace() || ch.is_control() || matches!(ch, '-' | '\u{2010}'..='\u{2015}' | '\u{2212}' | '\u{200B}'..='\u{200D}' | '\u{FEFF}') {
            continue;
        }
        if !ch.is_ascii() {
            return None;
        }
        let index = ALPHABET.iter().position(|a| *a == ch.to_ascii_uppercase() as u8)?;
        values.push(index as u8);
        if values.len() > CODE_LEN {
            return None;
        }
    }
    if values.len() != CODE_LEN {
        return None;
    }
    let (body, check) = values.split_at(BODY_LEN);
    (checksum(body) == check[0]).then(|| render(&values))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn generated_codes_are_valid_unique_and_use_only_the_alphabet() {
        let mut seen = HashSet::new();
        for _ in 0..5000 {
            let code = generate();
            assert_eq!(code.len(), CODE_LEN);
            assert!(code.bytes().all(|b| ALPHABET.contains(&b)), "{code}");
            assert_eq!(canonical(&code).as_deref(), Some(code.as_str()));
            assert!(seen.insert(code));
        }
    }

    #[test]
    fn the_alphabet_has_no_lookalikes() {
        assert_eq!(ALPHABET.iter().collect::<HashSet<_>>().len(), 32);
        for banned in b"0O1I" {
            assert!(!ALPHABET.contains(banned), "{}", *banned as char);
        }
    }

    #[test]
    fn any_single_wrong_character_is_caught() {
        let code = generate();
        for position in 0..CODE_LEN {
            for replacement in ALPHABET.iter().map(|b| *b as char) {
                if replacement == code.as_bytes()[position] as char {
                    continue;
                }
                let mut typo = code.clone().into_bytes();
                typo[position] = replacement as u8;
                assert!(canonical(&String::from_utf8(typo).unwrap()).is_none(), "{code} -> position {position} {replacement}");
            }
        }
    }

    #[test]
    fn swapped_neighbours_are_caught_unless_their_values_differ_by_exactly_sixteen() {
        let code = generate();
        let bytes = code.as_bytes();
        let value = |b: u8| ALPHABET.iter().position(|a| *a == b).unwrap() as i32;
        for i in 0..CODE_LEN - 1 {
            if bytes[i] == bytes[i + 1] {
                continue;
            }
            let mut swapped = bytes.to_vec();
            swapped.swap(i, i + 1);
            let accepted = canonical(&String::from_utf8(swapped).unwrap()).is_some();
            assert!(!accepted || (value(bytes[i]) - value(bytes[i + 1])).abs() == 16, "{code} swap at {i}");
        }
    }

    #[test]
    fn input_is_forgiven_for_case_spacing_dashes_and_full_width() {
        let code = generate();
        let shown = display(&code);
        assert_eq!(shown.len(), CODE_LEN + 3);
        for variant in [
            shown.clone(),
            shown.to_lowercase(),
            code.clone(),
            format!("  {}  ", shown.replace('-', " ")),
            shown.replace('-', "\u{2014}"),
            format!("\u{200B}{shown}\u{FEFF}"),
            format!("{shown}\n"),
            shown.chars().map(|c| if c.is_ascii_alphanumeric() { char::from_u32(u32::from(c) + 0xFEE0).unwrap() } else { '\u{FF0D}' }).collect::<String>(),
        ] {
            assert_eq!(canonical(&variant).as_deref(), Some(code.as_str()), "{variant:?}");
        }
    }

    #[test]
    fn wrong_length_or_characters_are_rejected() {
        let code = generate();
        for bad in [String::new(), "ABCD".into(), format!("{code}A"), code[..15].to_string(), code.replace(&code[..1], "0"), "我是兑换码我是兑换码我是兑换码".into(), "A".repeat(10_000)] {
            assert!(canonical(&bad).is_none(), "{bad:?}");
        }
    }
}
