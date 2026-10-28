//! 敏感词：发帖、回复、发布内容包前过一遍词表。词表由管理员维护，不内置任何词。
//!
//! 命中“拦截”的词直接拒绝发布（不告诉用户具体是哪个词，避免被试探）；命中“复核”的词先发布，同时进审核队列由人看。
//! 匹配前先归一化：转小写、去掉空白和标点，所以“违 禁-词”也能命中“违禁词”。

use crate::error::{AppError, AppResult};
use sqlx::{PgConnection, PgPool};

pub const BLOCK: &str = "block";
pub const REVIEW: &str = "review";

/// 归一化：只保留字母和数字（含中文），统一小写。
pub fn normalize(text: &str) -> String {
    text.chars().filter(|c| c.is_alphanumeric()).flat_map(char::to_lowercase).collect()
}

/// 词表里的词至少 2 个字，避免一个常用字把整站挡住。
pub fn clean_word(raw: &str) -> Option<String> {
    let word = normalize(raw);
    (word.chars().count() >= 2 && word.chars().count() <= 40).then_some(word)
}

#[derive(Debug, PartialEq)]
pub enum Verdict {
    Clean,
    /// 命中“复核”词：可以发布，但要进审核队列。
    Review(String),
    /// 命中“拦截”词：拒绝发布。
    Block,
}

/// `words` 是（归一化后的词，动作）。拦截优先于复核。
pub fn evaluate(words: &[(String, String)], texts: &[&str]) -> Verdict {
    let haystack: Vec<String> = texts.iter().map(|t| normalize(t)).collect();
    let hit = |w: &str| haystack.iter().any(|h| h.contains(w));
    if words.iter().any(|(w, a)| a == BLOCK && hit(w)) {
        return Verdict::Block;
    }
    match words.iter().find(|(w, a)| a == REVIEW && hit(w)) {
        Some((w, _)) => Verdict::Review(w.clone()),
        None => Verdict::Clean,
    }
}

pub async fn load_words(pool: &PgPool) -> AppResult<Vec<(String, String)>> {
    Ok(sqlx::query_as("SELECT word, action FROM sensitive_words").fetch_all(pool).await?)
}

/// 发布前检查。拦截返回 400；复核返回命中的词，调用方在内容写入后用 [`enqueue`] 入队。
pub async fn screen(pool: &PgPool, texts: &[&str]) -> AppResult<Option<String>> {
    match evaluate(&load_words(pool).await?, texts) {
        Verdict::Clean => Ok(None),
        Verdict::Review(word) => Ok(Some(word)),
        Verdict::Block => Err(AppError::BadRequest("内容包含不允许发布的词语，请修改后再发布".into())),
    }
}

/// 进审核队列；同一对象已经在队列里就更新备注。
pub async fn enqueue(conn: &mut PgConnection, target_type: &str, target_id: i64, note: &str) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO moderation_queue (target_type, target_id, note) VALUES ($1, $2, $3)
         ON CONFLICT (target_type, target_id) WHERE status = 'pending' DO UPDATE SET note = EXCLUDED.note",
    )
    .bind(target_type)
    .bind(target_id)
    .bind(note)
    .execute(conn)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(list: &[(&str, &str)]) -> Vec<(String, String)> {
        list.iter().map(|(w, a)| (w.to_string(), a.to_string())).collect()
    }

    #[test]
    fn normalizes_away_spacing_punctuation_and_case() {
        assert_eq!(normalize("违 禁-词！A.B"), "违禁词ab");
        assert_eq!(clean_word(" Bad Word "), Some("badword".into()));
        assert_eq!(clean_word("字"), None, "单字不收");
        assert_eq!(clean_word("！！"), None);
        assert_eq!(clean_word(&"长".repeat(41)), None);
    }

    #[test]
    fn evasion_by_inserting_separators_still_matches() {
        let w = words(&[("违禁词", BLOCK)]);
        for text in ["这里有违禁词", "违 禁 词", "违-禁*词", "违禁\u{200b}词", "标题", ] {
            let expect = if text == "标题" { Verdict::Clean } else { Verdict::Block };
            assert_eq!(evaluate(&w, &[text]), expect, "{text}");
        }
        assert_eq!(evaluate(&w, &["无关标题", "正文里有 违禁词"]), Verdict::Block, "任一段命中即可");
    }

    #[test]
    fn block_beats_review_and_review_reports_the_word() {
        let w = words(&[("复核词", REVIEW), ("拦截词", BLOCK)]);
        assert_eq!(evaluate(&w, &["只有复核词"]), Verdict::Review("复核词".into()));
        assert_eq!(evaluate(&w, &["复核词和拦截词都有"]), Verdict::Block);
        assert_eq!(evaluate(&[], &["随便"]), Verdict::Clean);
    }
}
