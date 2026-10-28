//! 站内通知：回复、被回复、被 @、回答被采纳、内容包审核结果。只做站内，不发邮件和系统推送。
//!
//! 通知在业务事务里写入，所以回复成功通知才会有；自己对自己的操作不通知；屏蔽了对方的人也不会收到对方带来的通知。

use crate::error::AppResult;
use sqlx::PgConnection;
use uuid::Uuid;

pub const REPLY: &str = "reply";
pub const REPLY_TO: &str = "reply_to";
pub const MENTION: &str = "mention";
pub const ACCEPTED: &str = "accepted";
pub const PACK_APPROVED: &str = "pack_approved";
pub const PACK_REJECTED: &str = "pack_rejected";

/// 一条通知要记的内容；标题和摘要是当时的快照，之后帖子改名也不影响通知。
pub struct Note<'a> {
    pub kind: &'a str,
    pub actor: Option<Uuid>,
    pub topic_id: Option<i64>,
    pub post_id: Option<i64>,
    pub pack_id: Option<i64>,
    pub title: &'a str,
    pub excerpt: &'a str,
}

fn clip(text: &str, max: usize) -> String {
    text.chars().take(max).collect()
}

/// 写一条通知；对自己、被对方屏蔽的情况静默跳过。返回是否写入。
pub async fn notify(conn: &mut PgConnection, to: Uuid, note: &Note<'_>) -> AppResult<bool> {
    if note.actor == Some(to) {
        return Ok(false);
    }
    if let Some(actor) = note.actor {
        let blocked: (bool,) = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM user_blocks WHERE blocker = $1 AND blocked = $2)")
            .bind(to)
            .bind(actor)
            .fetch_one(&mut *conn)
            .await?;
        if blocked.0 {
            return Ok(false);
        }
    }
    sqlx::query(
        "INSERT INTO notifications (uid, kind, actor_uid, topic_id, post_id, pack_id, title, excerpt)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(to)
    .bind(note.kind)
    .bind(note.actor)
    .bind(note.topic_id)
    .bind(note.post_id)
    .bind(note.pack_id)
    .bind(clip(note.title, 80))
    .bind(clip(note.excerpt, 80))
    .execute(conn)
    .await?;
    Ok(true)
}

/// 正文里的 `@用户名`：用户名是唯一账号，只收字母、数字、下划线、连字符和中文，最多取前 5 个不同的。
pub fn mentions(text: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let name_char = |c: char| c.is_alphanumeric() || c == '_' || c == '-';
    let mut i = 0;
    while i < chars.len() {
        // `@` 前面不能紧贴着字母数字（邮箱地址里的 @ 不算提及）。
        if chars[i] == '@' && (i == 0 || !chars[i - 1].is_alphanumeric()) {
            let name: String = chars[i + 1..].iter().copied().take_while(|c| name_char(*c)).take(32).collect();
            if !name.is_empty() && !found.contains(&name) {
                found.push(name);
                if found.len() == 5 {
                    break;
                }
            }
        }
        i += 1;
    }
    found
}

/// 给正文里 @ 到的人发通知；排除已经因为别的原因收到通知的人（`skip`）。
pub async fn notify_mentions(conn: &mut PgConnection, text: &str, actor: Uuid, skip: &[Uuid], note: &Note<'_>) -> AppResult<()> {
    for name in mentions(text) {
        let target: Option<(Uuid,)> = sqlx::query_as("SELECT uid FROM users WHERE nickname = $1 AND status <> 'banned'")
            .bind(&name)
            .fetch_optional(&mut *conn)
            .await?;
        if let Some((uid,)) = target.filter(|(u,)| *u != actor && !skip.contains(u)) {
            notify(conn, uid, note).await?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_mentions_but_not_emails() {
        assert_eq!(mentions("你好 @小明 和 @bob_1，还有 @小明"), vec!["小明", "bob_1"]);
        assert!(mentions("联系 me@example.com").is_empty());
        assert!(mentions("@ 空").is_empty());
        assert_eq!(mentions("（@张三）").len(), 1);
        assert_eq!(mentions("@a @b @c @d @e @f @g").len(), 5, "最多取 5 个");
        assert_eq!(mentions("@" .to_string().repeat(3).as_str()).len(), 0);
    }
}
