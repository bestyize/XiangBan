//! 受限讨论的插图在公开资源入口隐藏，阅读接口按帖子和读者签发限时地址。

use crate::{content, error::AppResult, models::Reader, state::AppState, users};
use sqlx::{Executor, PgPool};
use std::collections::HashSet;
use uuid::Uuid;

const SCHEMA: &str = r#"
ALTER TABLE images ADD COLUMN IF NOT EXISTS discussion_private BOOLEAN NOT NULL DEFAULT false;

CREATE OR REPLACE FUNCTION discussion_image_ids(html text) RETURNS SETOF uuid
LANGUAGE sql IMMUTABLE STRICT AS $$
    SELECT m[1]::uuid FROM regexp_matches(html, ' src="[^"]*/(?:api|forum-content)/images/([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})(?:[?#][^"]*)?"', 'gi') m
$$;

CREATE OR REPLACE FUNCTION mark_private_discussion_images() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF TG_TABLE_NAME = 'topics' THEN
        IF NEW.min_read_level > 0 THEN
            UPDATE images SET discussion_private=true
            WHERE NOT discussion_private AND uploader_uid=NEW.author_uid
              AND id IN (SELECT discussion_image_ids(NEW.body_html));
            IF TG_OP = 'UPDATE' AND OLD.min_read_level = 0 THEN
                UPDATE images i SET discussion_private=true FROM (
                    SELECT p.author_uid, discussion_image_ids(p.body_html) AS image_id
                    FROM posts p WHERE p.topic_id=NEW.id
                ) refs WHERE i.id=refs.image_id AND i.uploader_uid=refs.author_uid AND NOT i.discussion_private;
            END IF;
        END IF;
    ELSIF EXISTS (SELECT 1 FROM topics WHERE id=NEW.topic_id AND min_read_level > 0) THEN
        UPDATE images SET discussion_private=true
        WHERE NOT discussion_private AND uploader_uid=NEW.author_uid
          AND id IN (SELECT discussion_image_ids(NEW.body_html));
    END IF;
    RETURN NEW;
END
$$;

DROP TRIGGER IF EXISTS protect_topic_images ON topics;
CREATE TRIGGER protect_topic_images AFTER INSERT OR UPDATE OF body_html, min_read_level ON topics
    FOR EACH ROW EXECUTE FUNCTION mark_private_discussion_images();
DROP TRIGGER IF EXISTS protect_reply_images ON posts;
CREATE TRIGGER protect_reply_images AFTER INSERT OR UPDATE OF body_html ON posts
    FOR EACH ROW EXECUTE FUNCTION mark_private_discussion_images();
"#;

pub async fn init(pool: &PgPool) -> AppResult<()> {
    let mut tx = pool.begin().await?;
    (&mut *tx).execute(SCHEMA).await?;
    let added = sqlx::query("INSERT INTO data_migrations(name) VALUES ('restricted-discussion-images-v1') ON CONFLICT DO NOTHING")
        .execute(&mut *tx).await?.rows_affected();
    if added != 0 {
        sqlx::query(
            "WITH restricted AS (
                SELECT t.author_uid, discussion_image_ids(t.body_html) AS image_id FROM topics t WHERE t.min_read_level > 0
                UNION
                SELECT p.author_uid, discussion_image_ids(p.body_html) FROM posts p
                JOIN topics t ON t.id=p.topic_id WHERE t.min_read_level > 0
             )
             UPDATE images i SET discussion_private=true FROM restricted r
             WHERE i.uploader_uid=r.author_uid AND i.id=r.image_id",
        ).execute(&mut *tx).await?;
    }
    tx.commit().await?;
    Ok(())
}

pub async fn publicly_available(
    pool: &PgPool,
    id: Uuid,
    uploader: Uuid,
    private: bool,
    avatar_pending: bool,
) -> AppResult<bool> {
    if !private || avatar_pending {
        return Ok(true);
    }
    let public = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM users WHERE uid=$2 AND right(avatar_url, 37)='/' || $1::uuid::text)
          OR EXISTS(SELECT 1 FROM topics t WHERE t.author_uid=$2 AND t.min_read_level=0 AND t.status='normal'
                   AND $1 IN (SELECT discussion_image_ids(t.body_html)))
          OR EXISTS(SELECT 1 FROM posts p JOIN topics t ON t.id=p.topic_id
                    WHERE p.author_uid=$2 AND p.status='normal' AND t.status='normal' AND t.min_read_level=0
                      AND $1 IN (SELECT discussion_image_ids(p.body_html)))",
    ).bind(id).bind(uploader).fetch_one(pool).await?;
    Ok(public)
}

pub struct ImageView<'a> {
    state: &'a AppState,
    topic: i64,
    viewer: Uuid,
    expires: i64,
}

impl<'a> ImageView<'a> {
    pub fn new(state: &'a AppState, topic: i64, viewer: Uuid) -> Self {
        // A single expiry per half hour keeps URLs stable while browsing a list.
        let now = chrono::Utc::now().timestamp();
        let expires = (now / 1800 + 2) * 1800;
        Self {
            state,
            topic,
            viewer,
            expires,
        }
    }

    fn image_id(&self, source: &str) -> Option<Uuid> {
        let source = source
            .strip_prefix(self.state.config.public_url.as_str())
            .unwrap_or(source);
        let id = source.strip_prefix("/api/images/").or_else(|| {
            let base = self.state.config.resource_base_url.as_deref()?;
            source
                .strip_prefix(base)?
                .strip_prefix("/forum-content/images/")
        })?;
        Uuid::parse_str(id.split(['?', '#']).next()?).ok()
    }

    pub fn url(&self, source: &str) -> Option<String> {
        self.image_id(source).map(|id| self.signed_url(id))
    }

    fn signed_url(&self, id: Uuid) -> String {
        let claims = format!("{}.{}.{}", self.topic, self.viewer, self.expires);
        let sig = content::sign_callback(
            message(id, &claims).as_bytes(),
            &self.state.config.resource_secret,
        );
        format!("/api/images/{id}/view?key={claims}.{sig}")
    }

    pub fn html(&self, html: &str) -> String {
        self.rewrite_html(html, |id| Some(self.signed_url(id)))
    }

    pub async fn preview_html(&self, html: String) -> AppResult<String> {
        let ids: Vec<_> = html
            .split(" src=\"")
            .skip(1)
            .filter_map(|span| self.image_id(span.split_once('"')?.0))
            .collect();
        if ids.is_empty() {
            return Ok(html);
        }
        let private: HashSet<Uuid> = sqlx::query_scalar(
            "SELECT id FROM images WHERE id=ANY($1) AND uploader_uid=$2 AND discussion_private",
        )
        .bind(ids)
        .bind(self.viewer)
        .fetch_all(&self.state.pool)
        .await?
        .into_iter()
        .collect();
        if private.is_empty() {
            return Ok(html);
        }
        Ok(self.rewrite_html(&html, |id| {
            private.contains(&id).then(|| self.signed_url(id))
        }))
    }

    fn rewrite_html(&self, html: &str, rewrite: impl Fn(Uuid) -> Option<String>) -> String {
        let mut spans = html.split(" src=\"");
        let mut result = spans.next().unwrap_or_default().to_owned();
        for span in spans {
            result.push_str(" src=\"");
            if let Some((source, tail)) = span.split_once('"') {
                result.push_str(
                    self.image_id(source)
                        .and_then(&rewrite)
                        .as_deref()
                        .unwrap_or(source),
                );
                result.push('"');
                result.push_str(tail);
            } else {
                result.push_str(span);
            }
        }
        result
    }
}

pub fn sign_topic(state: &AppState, topic: &mut crate::models::TopicItem, uid: Option<Uuid>) {
    if let Some(uid) = uid.filter(|_| topic.min_read_level > 0 && !topic.images.is_empty()) {
        let view = ImageView::new(state, topic.id, uid);
        for url in &mut topic.images {
            if let Some(signed) = view.url(url) {
                *url = signed;
            }
        }
    }
}

pub async fn topic_page(
    state: &AppState,
    filter: &crate::models::ListFilter,
) -> AppResult<crate::models::Page<crate::models::TopicItem>> {
    let mut page = crate::models::list_topics(&state.pool, filter).await?;
    for topic in &mut page.items {
        sign_topic(state, topic, filter.reader.uid);
    }
    Ok(page)
}

fn message(id: Uuid, claims: &str) -> String {
    format!("discussion-image\n{id}\n{claims}")
}

pub async fn can_view(
    state: &AppState,
    id: Uuid,
    uploader: Uuid,
    public: bool,
    key: &str,
) -> AppResult<bool> {
    let Some((claims, sig)) = key.rsplit_once('.') else {
        return Ok(false);
    };
    let mut fields = claims.split('.');
    let topic = fields.next().and_then(|s| s.parse::<i64>().ok());
    let viewer = fields.next().and_then(|s| Uuid::parse_str(s).ok());
    let expires = fields.next().and_then(|s| s.parse::<i64>().ok());
    let (Some(topic), Some(viewer), Some(expires), None) = (topic, viewer, expires, fields.next())
    else {
        return Ok(false);
    };
    if expires < chrono::Utc::now().timestamp()
        || !content::verify_callback(
            message(id, claims).as_bytes(),
            sig,
            &state.config.resource_secret,
        )
    {
        return Ok(false);
    }
    if public || viewer == uploader {
        return Ok(true);
    }
    let Some(user) = users::find(&state.pool, viewer)
        .await?
        .filter(|u| u.status != "banned")
    else {
        return Ok(false);
    };
    let row: Option<(i16, Uuid)> = sqlx::query_as(
        "SELECT t.min_read_level, t.author_uid FROM topics t WHERE t.id=$1 AND t.status='normal' AND (
            (t.author_uid=$2 AND $3 IN (SELECT discussion_image_ids(t.body_html))) OR EXISTS(
                SELECT 1 FROM posts p WHERE p.topic_id=t.id AND p.author_uid=$2 AND p.status='normal'
                    AND $3 IN (SELECT discussion_image_ids(p.body_html))
            )
         ) AND NOT EXISTS(SELECT 1 FROM content_discussions d WHERE d.topic_id=t.id AND d.managed)",
    ).bind(topic).bind(uploader).bind(id).fetch_optional(&state.pool).await?;
    Ok(row
        .is_some_and(|(level, author)| Reader::from_user(Some(&user)).check(level, author).is_ok()))
}
