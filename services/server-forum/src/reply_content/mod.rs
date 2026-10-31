//! 回复只保存公开作品的引用，附件沿用原上传、阅读权限和购买账本。
pub mod schema;
use crate::{
    error::{AppError, AppResult},
    handlers::{
        attachments::{AttachmentItem, AttachmentPriceInput},
        content::public_content_card,
    },
    state::AppState,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;
#[derive(Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct WorkRef {
    pub kind: String,
    pub id: i64,
    pub version: Option<i32>,
}
/// 省略字段保持旧客户端兼容；新增引用固定到确实存在的作品版本。
pub async fn prepare(s: &AppState, refs: Option<Vec<WorkRef>>) -> AppResult<Option<Vec<WorkRef>>> {
    let Some(refs) = refs else { return Ok(None) };
    if refs.len() > 3 {
        return Err(AppError::BadRequest(
            "一条回复最多引用 3 项作品或方案".into(),
        ));
    }
    let mut normalized = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for mut r in refs {
        if !seen.insert((r.kind.clone(), r.id)) {
            return Err(AppError::BadRequest("不能重复引用同一内容".into()));
        }
        let card = public_content_card(s, &r.kind, r.id, r.version)
            .await?
            .ok_or_else(|| AppError::BadRequest("引用的内容或版本不可公开访问".into()))?;
        if r.kind == "pack" {
            let version = r
                .version
                .unwrap_or(card["version"].as_i64().unwrap_or(0) as i32);
            if public_content_card(s, "pack", r.id, Some(version))
                .await?
                .is_none()
            {
                return Err(AppError::BadRequest("引用的作品版本不存在".into()));
            }
            r.version = Some(version);
        }
        normalized.push(r);
    }
    Ok(Some(normalized))
}
/// 编辑保留既有引用即使作品后来下架；新加引用仍必须公开可读，读取时统一输出安全占位。
pub async fn prepare_edit(
    s: &AppState,
    post: i64,
    uid: Uuid,
    refs: Option<Vec<WorkRef>>,
) -> AppResult<Option<Vec<WorkRef>>> {
    let Some(refs) = refs else { return Ok(None) };
    if refs.len() > 3 {
        return Err(AppError::BadRequest(
            "一条回复最多引用 3 项作品或方案".into(),
        ));
    }
    let old:Option<(Value,)>=sqlx::query_as("SELECT r.refs FROM reply_work_refs r JOIN posts p ON p.id=r.post_id WHERE p.id=$1 AND p.author_uid=$2").bind(post).bind(uid).fetch_optional(&s.pool).await?;
    let old: Vec<WorkRef> = serde_json::from_value(old.map(|r| r.0).unwrap_or(json!([])))
        .map_err(|_| AppError::Internal("回复引用数据无效".into()))?;
    let mut prepared = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for r in refs {
        if !seen.insert((r.kind.clone(), r.id)) {
            return Err(AppError::BadRequest("不能重复引用同一内容".into()));
        }
        if old.contains(&r) {
            prepared.push(r)
        } else {
            prepared.extend(prepare(s, Some(vec![r])).await?.unwrap_or_default());
        }
    }
    Ok(Some(prepared))
}
pub async fn set(
    tx: &mut Transaction<'_, Postgres>,
    post: i64,
    topic: i64,
    uid: Uuid,
    refs: Option<&[WorkRef]>,
    ids: Option<&[Uuid]>,
    prices: Option<&[AttachmentPriceInput]>,
) -> AppResult<Vec<Uuid>> {
    if let Some(refs) = refs {
        sqlx::query("INSERT INTO reply_work_refs(post_id,refs) VALUES($1,$2) ON CONFLICT(post_id) DO UPDATE SET refs=$2").bind(post).bind(serde_json::to_value(refs).map_err(|_|AppError::BadRequest("作品引用无效".into()))?).execute(&mut **tx).await?;
    }
    let mut removed = Vec::new();
    if let Some(ids) = ids {
        let unique: std::collections::HashSet<_> = ids.iter().collect();
        if ids.len() > 5 || ids.len() != unique.len() {
            return Err(AppError::BadRequest(
                "一条回复最多 5 个附件且不能重复".into(),
            ));
        }
        // 与帖子编辑一样先锁 topic；附件按 id 固定顺序加锁，防止同时被其他回复领取。
        sqlx::query(
            "SELECT id FROM attachments WHERE post_id=$1 OR id=ANY($2) ORDER BY id FOR UPDATE",
        )
        .bind(post)
        .bind(ids)
        .fetch_all(&mut **tx)
        .await?;
        let rows: Vec<(Uuid,)> = sqlx::query_as(
            "DELETE FROM attachments WHERE post_id=$1 AND NOT(id=ANY($2)) RETURNING id",
        )
        .bind(post)
        .bind(ids)
        .fetch_all(&mut **tx)
        .await?;
        removed = rows.into_iter().map(|r| r.0).collect();
        sqlx::query("UPDATE attachments SET post_id=$1,topic_id=$2 WHERE id=ANY($3) AND uploader_uid=$4 AND topic_id IS NULL AND post_id IS NULL").bind(post).bind(topic).bind(ids).bind(uid).execute(&mut **tx).await?;
        let (count,):(i64,)=sqlx::query_as("SELECT count(*) FROM attachments WHERE post_id=$1 AND topic_id=$2 AND uploader_uid=$3 AND id=ANY($4)").bind(post).bind(topic).bind(uid).bind(ids).fetch_one(&mut **tx).await?;
        if count as usize != ids.len() {
            return Err(AppError::BadRequest(
                "附件不属于你、已被使用或已失效，请重新上传".into(),
            ));
        }
    }
    if let Some(prices) = prices {
        if prices.len() > 5 {
            return Err(AppError::BadRequest("附件定价数量超过上限".into()));
        }
        let mut seen = std::collections::HashSet::new();
        for price in prices {
            if !(0..=crate::points::MAX_ATTACHMENT_PRICE).contains(&price.price)
                || !seen.insert(price.id)
            {
                return Err(AppError::BadRequest(
                    "附件价格须为 0～10 的整数且不能重复".into(),
                ));
            }
            let n = sqlx::query(
                "UPDATE attachments SET price=$4 WHERE post_id=$1 AND uploader_uid=$2 AND id=$3",
            )
            .bind(post)
            .bind(uid)
            .bind(price.id)
            .bind(price.price)
            .execute(&mut **tx)
            .await?
            .rows_affected();
            if n != 1 {
                return Err(AppError::BadRequest(
                    "只能设置自己在本回复中的附件价格".into(),
                ));
            }
        }
    }
    Ok(removed)
}
pub async fn source(
    pool: &PgPool,
    post: i64,
    viewer: Option<Uuid>,
) -> AppResult<(Vec<WorkRef>, Vec<AttachmentItem>)> {
    let refs: Option<(Value,)> =
        sqlx::query_as("SELECT refs FROM reply_work_refs WHERE post_id=$1")
            .bind(post)
            .fetch_optional(pool)
            .await?;
    let refs = serde_json::from_value(refs.map(|r| r.0).unwrap_or(json!([])))
        .map_err(|_| AppError::Internal("回复引用数据无效".into()))?;
    let attachments=sqlx::query_as("SELECT a.id,a.filename AS name,a.size,a.created_at,a.price,(a.uploader_uid=$2 OR EXISTS(SELECT 1 FROM attachment_purchases p WHERE p.attachment_id=a.id AND p.buyer_uid=$2)) IS TRUE AS purchased FROM attachments a WHERE a.post_id=$1 ORDER BY a.created_at,a.id").bind(post).bind(viewer).fetch_all(pool).await?;
    Ok((refs, attachments))
}
pub async fn resolve(
    s: &AppState,
    post: i64,
    viewer: Option<Uuid>,
) -> AppResult<(Vec<Value>, Vec<AttachmentItem>)> {
    let (refs, attachments) = source(&s.pool, post, viewer).await?;
    let mut resolved = Vec::new();
    for r in refs {
        let mut value = json!({"kind":r.kind,"id":r.id,"version":r.version,"available":false});
        if let Some(card) = public_content_card(s, &r.kind, r.id, r.version).await? {
            value["available"] = json!(true);
            value["content"] = card;
        }
        resolved.push(value);
    }
    Ok((resolved, attachments))
}
/// 已隐藏、删除、封禁作者或被读者屏蔽的回复附件不可通过旧下载直链绕过。
pub async fn ensure_attachment_visible(
    pool: &PgPool,
    id: Uuid,
    viewer: Option<Uuid>,
) -> AppResult<()> {
    let row:Option<(String,String,Uuid)>=sqlx::query_as("SELECT p.status,u.status,p.author_uid FROM attachments a JOIN posts p ON p.id=a.post_id JOIN users u ON u.uid=p.author_uid WHERE a.id=$1").bind(id).fetch_optional(pool).await?;
    if let Some((status, author_status, author)) = row {
        let (blocked,): (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM user_blocks WHERE blocker=$1 AND blocked=$2)",
        )
        .bind(viewer)
        .bind(author)
        .fetch_one(pool)
        .await?;
        if status != "normal" || author_status == "banned" || blocked {
            return Err(AppError::not_found("附件"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
