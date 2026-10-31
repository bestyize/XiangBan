//! 内容包/方案的私有收藏与举报。举报理由仅对管理员开放，内容状态沿用原有发布表。
pub mod schema;
pub mod discussion;
use crate::{
    error::{AppError, AppResult},
    handlers::{
        admin::{audit, AdminIdentity},
        content::public_content_card,
        interact::REPORT_REASONS,
    },
    notifications::{self, Note},
    state::AppState,
};
use serde_json::{json, Value};
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

/// 与公开内容卡保持一致：作品关联讨论必须公开，方案引用由统一 helper 逐项安全解析。
const VISIBLE_TARGETS:&str="SELECT 'pack'::text AS kind,p.id FROM packs p LEFT JOIN users u ON u.uid=p.author_uid LEFT JOIN topics t ON t.id=p.topic_id
 WHERE p.status='published' AND (p.author_uid IS NULL OR u.status<>'banned') AND (p.topic_id IS NULL OR (t.status='normal' AND t.min_read_level=0))
 UNION ALL SELECT 'scheme'::text AS kind,s.id FROM schemes s JOIN users u ON u.uid=s.author_uid WHERE s.status='published' AND u.status<>'banned'";
fn check_kind(kind: &str) -> AppResult<()> {
    if !["pack", "scheme"].contains(&kind) {
        return Err(AppError::BadRequest("内容类型只能是 pack 或 scheme".into()));
    }
    Ok(())
}
async fn target_lock(conn: &mut PgConnection, kind: &str, id: i64) -> AppResult<()> {
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
        .bind(format!("content-social:{kind}:{id}"))
        .execute(conn)
        .await?;
    Ok(())
}
async fn require_visible(conn: &mut PgConnection, kind: &str, id: i64) -> AppResult<()> {
    check_kind(kind)?;
    let (visible,): (bool,) = sqlx::query_as(&format!(
        "SELECT EXISTS(SELECT 1 FROM ({VISIBLE_TARGETS}) v WHERE v.kind=$1 AND v.id=$2)"
    ))
    .bind(kind)
    .bind(id)
    .fetch_one(conn)
    .await?;
    if !visible {
        return Err(AppError::not_found("内容"));
    }
    Ok(())
}
pub async fn state(pool: &PgPool, uid: Uuid, kind: &str, id: i64) -> AppResult<Value> {
    let mut tx = pool.begin().await?;
    require_visible(&mut tx, kind, id).await?;
    let (bookmarked,): (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM content_bookmarks WHERE uid=$1 AND kind=$2 AND target_id=$3)",
    )
    .bind(uid)
    .bind(kind)
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(json!({"bookmarked":bookmarked}))
}
pub async fn bookmark(pool: &PgPool, uid: Uuid, kind: &str, id: i64, on: bool) -> AppResult<Value> {
    check_kind(kind)?;
    if id <= 0 {
        return Err(AppError::not_found("内容"));
    }
    let mut tx = pool.begin().await?;
    if on {
        target_lock(&mut tx, kind, id).await?;
        require_visible(&mut tx, kind, id).await?;
        sqlx::query("INSERT INTO content_bookmarks(uid,kind,target_id) VALUES($1,$2,$3) ON CONFLICT DO NOTHING").bind(uid).bind(kind).bind(id).execute(&mut *tx).await?;
    } else {
        // 用户始终能清除自己的失效收藏，不暴露对象是否存在，也不能触碰他人记录。
        sqlx::query("DELETE FROM content_bookmarks WHERE uid=$1 AND kind=$2 AND target_id=$3")
            .bind(uid)
            .bind(kind)
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(json!({"bookmarked":on}))
}
pub async fn bookmarks(s: &AppState, uid: Uuid, kind: Option<&str>, page: i64) -> AppResult<Value> {
    if let Some(kind) = kind {
        check_kind(kind)?;
    }
    let page = page.clamp(1, 100000);
    let size = 20;
    let filter=format!("FROM content_bookmarks b JOIN ({VISIBLE_TARGETS}) v ON v.kind=b.kind AND v.id=b.target_id WHERE b.uid=$1 AND ($2::text IS NULL OR b.kind=$2)");
    let (total,): (i64,) = sqlx::query_as(&format!("SELECT count(*) {filter}"))
        .bind(uid)
        .bind(kind)
        .fetch_one(&s.pool)
        .await?;
    let rows:Vec<(String,i64,chrono::DateTime<chrono::Utc>)>=sqlx::query_as(&format!("SELECT b.kind,b.target_id,b.created_at {filter} ORDER BY b.created_at DESC,b.kind,b.target_id DESC LIMIT $3 OFFSET $4"))
        .bind(uid).bind(kind).bind(size).bind((page-1)*size).fetch_all(&s.pool).await?;
    let mut items = Vec::new();
    for (kind, id, created_at) in rows {
        if let Some(content) = public_content_card(s, &kind, id, None).await? {
            items.push(json!({"kind":kind,"id":id,"created_at":created_at,"content":content}));
        }
    }
    Ok(json!({"items":items,"total":total,"page":page,"page_size":size}))
}
pub async fn report(
    pool: &PgPool,
    uid: Uuid,
    kind: &str,
    id: i64,
    reason: &str,
    detail: &str,
) -> AppResult<()> {
    check_kind(kind)?;
    if !REPORT_REASONS.contains(&reason) {
        return Err(AppError::BadRequest("请选择有效的举报原因".into()));
    }
    let detail = detail.trim();
    if detail.chars().count() > 500
        || detail
            .chars()
            .any(|c| c.is_control() && c != '\n' && c != '\t')
    {
        return Err(AppError::BadRequest(
            "举报补充说明最多 500 字且不能含控制字符".into(),
        ));
    }
    let mut tx = pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
        .bind(format!("content-report-user:{uid}"))
        .execute(&mut *tx)
        .await?;
    target_lock(&mut tx, kind, id).await?;
    require_visible(&mut tx, kind, id).await?;
    let (existing,):(bool,)=sqlx::query_as("SELECT EXISTS(SELECT 1 FROM content_reports WHERE reporter_uid=$1 AND kind=$2 AND target_id=$3 AND status='open')").bind(uid).bind(kind).bind(id).fetch_one(&mut *tx).await?;
    if !existing {
        let (count,):(i64,)=sqlx::query_as("SELECT (SELECT count(*) FROM reports WHERE reporter_uid=$1 AND created_at>now()-interval '1 day')+(SELECT count(*) FROM content_reports WHERE reporter_uid=$1 AND created_at>now()-interval '1 day')")
            .bind(uid).fetch_one(&mut *tx).await?;
        if count >= 30 {
            return Err(AppError::TooManyRequests {
                message: "今天举报次数太多，请稍后再试".into(),
                retry_after: 3600,
            });
        }
    }
    sqlx::query("INSERT INTO content_reports(kind,target_id,reporter_uid,reason,detail) VALUES($1,$2,$3,$4,$5) ON CONFLICT(reporter_uid,kind,target_id) WHERE status='open' DO UPDATE SET reason=$4,detail=$5,updated_at=clock_timestamp(),revision=nextval('content_report_revision_seq')")
        .bind(kind).bind(id).bind(uid).bind(reason).bind(detail).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(())
}
pub async fn reports(pool: &PgPool, status: &str, page: i64) -> AppResult<Value> {
    if !["open", "resolved", "dismissed", "all"].contains(&status) {
        return Err(AppError::BadRequest("未知的举报状态".into()));
    }
    let filter = if status == "all" { None } else { Some(status) };
    let page = page.clamp(1, 100000);
    let (total,): (i64,) = sqlx::query_as(
        "SELECT count(*) FROM content_reports WHERE ($1::text IS NULL OR status=$1)",
    )
    .bind(filter)
    .fetch_one(pool)
    .await?;
    let rows:Vec<(Value,)>=sqlx::query_as("SELECT jsonb_build_object('id',r.id,'kind',r.kind,'target_id',r.target_id,'reason',r.reason,'detail',r.detail,'status',r.status,'revision',r.revision,'created_at',r.created_at,'handled_at',r.handled_at,'handled_by',r.handled_by,'result',r.result,
        'reporter',jsonb_build_object('uid',r.reporter_uid,'nickname',u.nickname),'target',jsonb_build_object('title',COALESCE(p.title,s.title),'status',COALESCE(p.status,s.status),'exists',COALESCE(p.id,s.id) IS NOT NULL))
        FROM content_reports r LEFT JOIN users u ON u.uid=r.reporter_uid LEFT JOIN packs p ON r.kind='pack' AND p.id=r.target_id LEFT JOIN schemes s ON r.kind='scheme' AND s.id=r.target_id
        WHERE ($1::text IS NULL OR r.status=$1) ORDER BY CASE WHEN r.status='open' THEN 0 ELSE 1 END,r.created_at,r.id LIMIT 20 OFFSET $2")
        .bind(filter).bind((page-1)*20).fetch_all(pool).await?;
    Ok(
        json!({"items":rows.into_iter().map(|r|r.0).collect::<Vec<_>>(),"total":total,"page":page,"page_size":20}),
    )
}
pub async fn handle_report(
    pool: &PgPool,
    admin: &AdminIdentity,
    id: i64,
    revision: i64,
    action: &str,
    note: &str,
) -> AppResult<Value> {
    if !["dismiss", "remove"].contains(&action) {
        return Err(AppError::BadRequest("请选择驳回或下架".into()));
    }
    let note = note.trim();
    if note.is_empty() || note.chars().count() > 500 {
        return Err(AppError::BadRequest("请填写 1–500 字的内部处理说明".into()));
    }
    let mut tx = pool.begin().await?;
    let target: Option<(String, i64)> =
        sqlx::query_as("SELECT kind,target_id FROM content_reports WHERE id=$1")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?;
    let (kind, target) = target.ok_or_else(|| AppError::not_found("举报"))?;
    target_lock(&mut tx, &kind, target).await?;
    let (old, status): (i64, String) =
        sqlx::query_as("SELECT revision,status FROM content_reports WHERE id=$1 FOR UPDATE")
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
    if old != revision || status != "open" {
        return Err(AppError::Conflict("举报已更新或已处理，请重新加载".into()));
    }
    let mut removed = false;
    if action == "remove" {
        let table = if kind == "pack" { "packs" } else { "schemes" };
        let update = if kind == "pack" {
            "status='removed',updated_at=clock_timestamp()"
        } else {
            "status='removed'"
        };
        let author: Option<(Option<Uuid>,)> = sqlx::query_as(&format!(
            "SELECT author_uid FROM {table} WHERE id=$1 FOR UPDATE"
        ))
        .bind(target)
        .fetch_optional(&mut *tx)
        .await?;
        removed = sqlx::query(&format!(
            "UPDATE {table} SET {update} WHERE id=$1 AND status<>'removed'"
        ))
        .bind(target)
        .execute(&mut *tx)
        .await?
        .rows_affected()
            > 0;
        if removed {
            if let Some((Some(uid),)) = author {
                notifications::notify(
                    &mut tx,
                    uid,
                    &Note {
                        kind: "content_removed",
                        actor: None,
                        topic_id: None,
                        post_id: None,
                        pack_id: if kind == "pack" { Some(target) } else { None },
                        title: "社区内容已下架",
                        excerpt: "你的作品或方案已由管理员下架，请到“我的作品”查看。",
                    },
                )
                .await?;
            }
        }
    }
    let affected = if action == "remove" {
        sqlx::query("UPDATE content_reports SET status='resolved',handled_by=$3,handled_at=clock_timestamp(),result=$4,revision=nextval('content_report_revision_seq') WHERE kind=$1 AND target_id=$2 AND status='open'")
            .bind(&kind).bind(target).bind(&admin.username).bind(note).execute(&mut *tx).await?.rows_affected()
    } else {
        sqlx::query("UPDATE content_reports SET status='dismissed',handled_by=$2,handled_at=clock_timestamp(),result=$3,revision=nextval('content_report_revision_seq') WHERE id=$1")
            .bind(id).bind(&admin.username).bind(note).execute(&mut *tx).await?.rows_affected()
    };
    audit(
        &mut tx,
        admin,
        &format!("content.report.{action}"),
        "content_report",
        id,
        format!("kind={kind}; target={target}; affected={affected}; note={note}"),
    )
    .await?;
    tx.commit().await?;
    Ok(json!({"ok":true,"target_removed":removed,"reports_handled":affected}))
}
