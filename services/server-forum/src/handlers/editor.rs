//! 站方图文创作：管理员入口和限定作者、可撤销的本地创作令牌共用业务接口。
use super::{admin::AuthAdmin, images, topics};
use crate::{
    error::{AppError, AppResult},
    markdown,
    state::AppState,
    users,
};
use axum::{
    extract::{DefaultBodyLimit, FromRequestParts, Path, Query, State},
    http::{header, request::Parts},
    routing::{get, post},
    Json, Router,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::PgConnection;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

const MAX_IMAGES: usize = 20;
const MAX_IMAGE_BYTES: usize = 20 * 1024 * 1024;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/context", get(context))
        .route(
            "/drafts",
            get(list)
                .post(save)
                .layer(DefaultBodyLimit::max(30 * 1024 * 1024)),
        )
        .route("/drafts/{id}", get(detail).delete(delete))
        .route("/drafts/{id}/publish", post(publish))
}
pub fn admin_routes() -> Router<AppState> {
    Router::new()
        .route("/authors", get(authors).post(create_author))
        .route("/authors/bind", post(bind_author))
        .route("/tokens", get(tokens).post(create_token))
        .route("/tokens/{id}", axum::routing::delete(revoke_token))
}

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: String,
    jti: Uuid,
    aud: String,
    exp: i64,
}
pub struct Editor {
    author: Option<Uuid>,
    can_publish: bool,
    operator: String,
}
impl Editor {
    fn check(&self, uid: Uuid) -> AppResult<()> {
        if self.author.is_some_and(|a| a != uid) {
            return Err(AppError::Forbidden("创作令牌只能操作绑定作者的稿件".into()));
        }
        Ok(())
    }
}
impl FromRequestParts<AppState> for Editor {
    type Rejection = AppError;
    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> AppResult<Self> {
        let raw = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or_else(|| AppError::Unauthorized("缺少创作凭据".into()))?;
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_audience(&["forum-editor"]);
        if let Ok(data) = decode::<Claims>(
            raw,
            &DecodingKey::from_secret(&state.config.admin_jwt_secret),
            &validation,
        ) {
            let uid = Uuid::parse_str(&data.claims.sub)
                .map_err(|_| AppError::Unauthorized("创作凭据无效".into()))?;
            let found: Option<(bool,)> = sqlx::query_as("SELECT can_publish FROM editor_tokens WHERE id=$1 AND author_uid=$2 AND revoked_at IS NULL AND expires_at>now()")
                .bind(data.claims.jti).bind(uid).fetch_optional(&state.pool).await?;
            let (can_publish,) =
                found.ok_or_else(|| AppError::Unauthorized("创作令牌已过期或被撤销".into()))?;
            return Ok(Self {
                author: Some(uid),
                can_publish,
                operator: format!("editor:{}", data.claims.jti),
            });
        }
        let AuthAdmin(admin) = AuthAdmin::from_request_parts(parts, state).await?;
        Ok(Self {
            author: None,
            can_publish: true,
            operator: admin.username,
        })
    }
}
async fn log(
    conn: &mut PgConnection,
    operator: &str,
    action: &str,
    id: impl ToString,
    detail: &str,
) -> AppResult<()> {
    sqlx::query("INSERT INTO admin_audit (admin,action,target_type,target_id,detail) VALUES ($1,$2,'editor',$3,$4)")
        .bind(operator).bind(action).bind(id.to_string()).bind(detail).execute(conn).await?;
    Ok(())
}
async fn authors(State(s): State<AppState>, _: AuthAdmin) -> AppResult<Json<Value>> {
    let rows: Vec<(Uuid, String, i16, String)> = sqlx::query_as("SELECT u.uid,u.nickname,u.trust_level,u.status FROM editor_authors e JOIN users u ON u.uid=e.uid ORDER BY e.created_at DESC")
        .fetch_all(&s.pool).await?;
    Ok(Json(
        json!({"items": rows.into_iter().map(|(uid,nickname,trust_level,status)|json!({"uid":uid,"nickname":nickname,"trust_level":trust_level,"status":status})).collect::<Vec<_>>(), "provision_enabled":s.config.account_provision_key.is_some()}),
    ))
}
#[derive(Deserialize)]
struct AuthorInput {
    request_id: Uuid,
    username: String,
    password: String,
    nickname: String,
    avatar_url: Option<String>,
    trust_level: i16,
}
async fn create_author(
    State(s): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Json(req): Json<AuthorInput>,
) -> AppResult<Json<Value>> {
    validate_author(&req.nickname, req.trust_level)?;
    let key = s.config.account_provision_key.as_deref().ok_or_else(|| {
        AppError::BadRequest("请先配置账号中心创建通道，或绑定已有论坛用户".into())
    })?;
    let response=s.http.post(format!("{}/api/internal/forum/users",s.config.account_issuer)).bearer_auth(key)
        .json(&json!({"request_id":req.request_id,"username":req.username,"password":req.password,"avatar_url":req.avatar_url}))
        .send().await.map_err(|_|AppError::ServiceUnavailable("账号中心暂时不可用，可保留表单重试".into()))?;
    let ok = response.status().is_success();
    let result: Value = response
        .json()
        .await
        .map_err(|_| AppError::ServiceUnavailable("账号中心响应异常，可保留表单重试".into()))?;
    if !ok {
        return Err(AppError::BadRequest(
            result["message"]
                .as_str()
                .unwrap_or("创建账号失败")
                .to_string(),
        ));
    }
    let uid: Uuid = result["uid"]
        .as_str()
        .and_then(|v| v.parse().ok())
        .ok_or_else(|| AppError::Internal("账号中心没有返回 UID".into()))?;
    let mut tx = s.pool.begin().await?;
    sqlx::query("INSERT INTO users (uid,nickname,avatar_url,trust_level) VALUES ($1,$2,$3,$4) ON CONFLICT(uid) DO NOTHING")
        .bind(uid).bind(req.nickname.trim()).bind(&req.avatar_url).bind(req.trust_level).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO editor_authors(uid) VALUES ($1) ON CONFLICT DO NOTHING")
        .bind(uid)
        .execute(&mut *tx)
        .await?;
    log(
        &mut tx,
        &admin.username,
        "editor.author.create",
        uid,
        req.nickname.trim(),
    )
    .await?;
    tx.commit().await?;
    Ok(Json(json!({"uid":uid})))
}
#[derive(Deserialize)]
struct BindInput {
    uid: Uuid,
}
async fn bind_author(
    State(s): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Json(req): Json<BindInput>,
) -> AppResult<Json<Value>> {
    let user = users::find(&s.pool, req.uid)
        .await?
        .ok_or_else(|| AppError::not_found("论坛用户"))?;
    if user.trust_level < 1 || user.status != "active" {
        return Err(AppError::BadRequest(
            "请选择正常状态且至少为成员的用户".into(),
        ));
    }
    let mut tx = s.pool.begin().await?;
    sqlx::query("INSERT INTO editor_authors(uid) VALUES ($1) ON CONFLICT DO NOTHING")
        .bind(req.uid)
        .execute(&mut *tx)
        .await?;
    log(
        &mut tx,
        &admin.username,
        "editor.author.bind",
        req.uid,
        &user.nickname,
    )
    .await?;
    tx.commit().await?;
    Ok(Json(json!({"uid":req.uid})))
}
fn validate_author(name: &str, level: i16) -> AppResult<()> {
    if !(1..=24).contains(&name.trim().chars().count()) || !(1..=3).contains(&level) {
        return Err(AppError::BadRequest(
            "昵称需为 1-24 字，编辑账号等级需为 1-3".into(),
        ));
    }
    Ok(())
}
#[derive(Deserialize)]
struct TokenInput {
    name: String,
    author_uid: Uuid,
    can_publish: bool,
    days: i64,
}
async fn create_token(
    State(s): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Json(req): Json<TokenInput>,
) -> AppResult<Json<Value>> {
    if req.name.trim().is_empty() || req.name.chars().count() > 60 || !(1..=365).contains(&req.days)
    {
        return Err(AppError::BadRequest(
            "名称需为 1-60 字，有效期需为 1-365 天".into(),
        ));
    }
    let exists: (bool,)=sqlx::query_as("SELECT EXISTS(SELECT 1 FROM editor_authors e JOIN users u ON u.uid=e.uid WHERE e.uid=$1 AND u.status='active' AND u.trust_level>=1)")
        .bind(req.author_uid).fetch_one(&s.pool).await?;
    if !exists.0 {
        return Err(AppError::BadRequest("请选择有效的编辑账号".into()));
    }
    let id = Uuid::new_v4();
    let expiry = Utc::now() + Duration::days(req.days);
    let token = encode(
        &Header::new(Algorithm::HS256),
        &Claims {
            sub: req.author_uid.to_string(),
            jti: id,
            aud: "forum-editor".into(),
            exp: expiry.timestamp(),
        },
        &EncodingKey::from_secret(&s.config.admin_jwt_secret),
    )
    .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut tx = s.pool.begin().await?;
    sqlx::query("INSERT INTO editor_tokens(id,name,author_uid,can_publish,expires_at) VALUES ($1,$2,$3,$4,$5)")
        .bind(id).bind(req.name.trim()).bind(req.author_uid).bind(req.can_publish).bind(expiry).execute(&mut *tx).await?;
    log(
        &mut tx,
        &admin.username,
        "editor.token.create",
        id,
        &req.author_uid.to_string(),
    )
    .await?;
    tx.commit().await?;
    Ok(Json(json!({"id":id,"token":token,"expires_at":expiry})))
}
async fn tokens(State(s): State<AppState>, _: AuthAdmin) -> AppResult<Json<Value>> {
    let rows:Vec<(Uuid,String,Uuid,bool,DateTime<Utc>,Option<DateTime<Utc>>)>=sqlx::query_as("SELECT id,name,author_uid,can_publish,expires_at,revoked_at FROM editor_tokens ORDER BY created_at DESC LIMIT 100").fetch_all(&s.pool).await?;
    Ok(Json(
        json!({"items":rows.into_iter().map(|(id,name,author_uid,can_publish,expires_at,revoked_at)|json!({"id":id,"name":name,"author_uid":author_uid,"can_publish":can_publish,"expires_at":expires_at,"revoked_at":revoked_at})).collect::<Vec<_>>()}),
    ))
}
async fn revoke_token(
    State(s): State<AppState>,
    AuthAdmin(a): AuthAdmin,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    let mut tx = s.pool.begin().await?;
    sqlx::query("UPDATE editor_tokens SET revoked_at=now() WHERE id=$1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    log(&mut tx, &a.username, "editor.token.revoke", id, "").await?;
    tx.commit().await?;
    Ok(Json(json!({"ok":true})))
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Asset {
    pub path: String,
    pub data_base64: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DraftInput {
    pub author_uid: Uuid,
    pub external_id: String,
    pub title: String,
    pub category: String,
    pub body: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub images: Vec<Asset>,
    #[serde(default)]
    pub overwrite: bool,
    pub expected_updated_at: Option<DateTime<Utc>>,
}
#[derive(sqlx::FromRow, Serialize)]
struct Draft {
    id: Uuid,
    author_uid: Uuid,
    external_id: String,
    category_id: i32,
    title: String,
    body_md: String,
    tags: Vec<String>,
    topic_id: Option<i64>,
    updated_at: DateTime<Utc>,
}
const COLS: &str = "id,author_uid,external_id,category_id,title,body_md,tags,topic_id,updated_at";
#[derive(Deserialize)]
struct Filter {
    author_uid: Option<Uuid>,
    page: Option<i64>,
    published: Option<bool>,
}
async fn context(State(s): State<AppState>, e: Editor) -> AppResult<Json<Value>> {
    let categories = super::meta::categories(State(s.clone())).await?.0;
    Ok(Json(
        json!({"author_uid":e.author,"can_publish":e.can_publish,"categories":categories,"limits":{"images":MAX_IMAGES,"image_bytes":s.config.upload_limit_image,"total_image_bytes":MAX_IMAGE_BYTES}}),
    ))
}
async fn list(
    State(s): State<AppState>,
    e: Editor,
    Query(f): Query<Filter>,
) -> AppResult<Json<Value>> {
    if let Some(uid) = f.author_uid {
        e.check(uid)?;
    }
    let uid = e.author.or(f.author_uid);
    let page = f.page.unwrap_or(1).clamp(1, 100000);
    let rows:Vec<Draft>=sqlx::query_as(&format!("SELECT {COLS} FROM editor_drafts WHERE ($1::uuid IS NULL OR author_uid=$1) AND ($2::bool IS NULL OR (topic_id IS NOT NULL)=$2) ORDER BY updated_at DESC,id LIMIT 30 OFFSET $3"))
        .bind(uid).bind(f.published).bind((page-1)*30).fetch_all(&s.pool).await?;
    let (total,):(i64,)=sqlx::query_as("SELECT count(*) FROM editor_drafts WHERE ($1::uuid IS NULL OR author_uid=$1) AND ($2::bool IS NULL OR (topic_id IS NOT NULL)=$2)")
        .bind(uid).bind(f.published).fetch_one(&s.pool).await?;
    // 列表只返回摘要，避免把所有正文带给模型。
    let items:Vec<Value>=rows.into_iter().map(|d|json!({"id":d.id,"author_uid":d.author_uid,"external_id":d.external_id,"category_id":d.category_id,"title":d.title,"tags":d.tags,"topic_id":d.topic_id,"updated_at":d.updated_at})).collect();
    Ok(Json(
        json!({"items":items,"total":total,"page":page,"page_size":30}),
    ))
}
fn clean_path(path: &str) -> AppResult<String> {
    let path = path.strip_prefix("./").unwrap_or(path);
    if path.is_empty()
        || path.len() > 300
        || path.contains(['\\', '\0', ':', '?', '#', '%'])
        || path.starts_with('/')
        || path
            .split('/')
            .any(|v| v.is_empty() || v == ".." || v == ".")
    {
        return Err(AppError::BadRequest(format!("图片路径无效：{path}")));
    }
    Ok(path.to_string())
}
// 通过 Markdown 语法树识别插图，支持行内和引用式图片，不替换代码块中的示例。
fn rewrite(md: &str, mut replace: impl FnMut(&str) -> AppResult<String>) -> AppResult<String> {
    let mut replacements = Vec::new();
    let mut current: Option<(usize, String, String, String)> = None;
    let parser = Parser::new_ext(
        md,
        Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS,
    )
    .into_offset_iter();
    for (event, range) in parser {
        match event {
            Event::Start(Tag::Image {
                dest_url, title, ..
            }) => {
                current = Some((
                    range.start,
                    dest_url.to_string(),
                    title.to_string(),
                    String::new(),
                ))
            }
            Event::Text(text) | Event::Code(text) => {
                if let Some((_, _, _, alt)) = &mut current {
                    alt.push_str(&text);
                }
            }
            Event::End(TagEnd::Image) => {
                if let Some((start, dest, title, alt)) = current.take() {
                    let dest = replace(&dest)?;
                    let alt = alt
                        .replace('\\', "\\\\")
                        .replace('[', "\\[")
                        .replace(']', "\\]");
                    let title = if title.is_empty() {
                        String::new()
                    } else {
                        format!(" \"{}\"", title.replace('\\', "\\\\").replace('"', "\\\""))
                    };
                    replacements.push((start..range.end, format!("![{alt}](<{dest}>{title})")));
                }
            }
            Event::Html(text) | Event::InlineHtml(text)
                if text.to_ascii_lowercase().contains("<img") =>
            {
                return Err(AppError::BadRequest(
                    "请使用 Markdown 图片语法，不要使用 HTML img 标签".into(),
                ))
            }
            _ => {}
        }
    }
    let mut out = md.to_string();
    for (range, text) in replacements.into_iter().rev() {
        out.replace_range(range, &text);
    }
    Ok(out)
}
async fn check_category(
    conn: &mut PgConnection,
    uid: Uuid,
    slug: &str,
) -> AppResult<(i32, String)> {
    let user:users::ForumUser=sqlx::query_as("SELECT uid,nickname,avatar_url,trust_level,points,status,muted_until,read_count,visit_days,created_at,last_seen_at FROM users WHERE uid=$1 FOR UPDATE")
        .bind(uid).fetch_optional(&mut *conn).await?.ok_or_else(||AppError::not_found("作者"))?;
    if user.status == "banned" || user.trust_level < 1 {
        return Err(AppError::Forbidden("作者已被封禁或等级不足".into()));
    }
    users::ensure_can_write(&user)?;
    let cat:Option<(i32,String,i16,bool,bool)>=sqlx::query_as("SELECT id,kind,min_post_level,admin_only,EXISTS(SELECT 1 FROM categories k WHERE k.parent_id=c.id) FROM categories c WHERE slug=$1 FOR SHARE OF c")
        .bind(slug).fetch_optional(&mut *conn).await?;
    let (id, kind, min, admin, children) =
        cat.ok_or_else(|| AppError::BadRequest("板块不存在".into()))?;
    if children || user.trust_level < topics::required_post_level(min, admin) {
        return Err(AppError::Forbidden(
            "请选择作者有发帖权限的具体子板块".into(),
        ));
    }
    Ok((id, kind))
}
async fn save(
    State(s): State<AppState>,
    e: Editor,
    Json(input): Json<DraftInput>,
) -> AppResult<Json<Value>> {
    e.check(input.author_uid)?;
    if input.external_id.is_empty()
        || input.external_id.len() > 100
        || !input
            .external_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
    {
        return Err(AppError::BadRequest(
            "稿件 id 需为 1-100 位字母、数字、点、下划线或连字符".into(),
        ));
    }
    let title = topics::clean_title(&input.title)?;
    let tags = topics::clean_tags(&input.tags)?;
    if input.images.len() > MAX_IMAGES {
        return Err(AppError::BadRequest("每篇最多 20 张图片".into()));
    }
    let mut assets = HashMap::new();
    let mut total = 0;
    for a in input.images {
        let path = clean_path(&a.path)?;
        let data = STANDARD
            .decode(&a.data_base64)
            .map_err(|_| AppError::BadRequest(format!("图片编码无效：{path}")))?;
        total += data.len();
        if data.len() as u64 > s.config.upload_limit_image || total > MAX_IMAGE_BYTES {
            return Err(AppError::BadRequest("单张图片或整篇图片总大小超限".into()));
        }
        let mime = images::sniff(&data)
            .ok_or_else(|| AppError::BadRequest(format!("只支持 PNG/JPEG/GIF/WebP：{path}")))?;
        if assets.insert(path, (Uuid::new_v4(), mime, data)).is_some() {
            return Err(AppError::BadRequest("图片路径重复".into()));
        }
    }
    let mut used = HashSet::new();
    let body = rewrite(&input.body, |path| {
        let path = clean_path(path)?;
        let (id, _, _) = assets.get(&path).ok_or_else(|| {
            AppError::BadRequest(format!("缺少本地图片：{path}，请将配图放入上传包"))
        })?;
        used.insert(path);
        Ok(format!("/api/images/{id}"))
    })?;
    let mut tx = s.pool.begin().await?;
    let (editor,): (bool,) =
        sqlx::query_as("SELECT EXISTS(SELECT 1 FROM editor_authors WHERE uid=$1)")
            .bind(input.author_uid)
            .fetch_one(&mut *tx)
            .await?;
    if !editor {
        return Err(AppError::BadRequest("请先创建或绑定编辑账号".into()));
    }
    // 锁定稿件编号，重试和并发导入不会创建两条稿件。
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
        .bind(format!("{}:{}", input.author_uid, input.external_id))
        .execute(&mut *tx)
        .await?;
    let existing:Option<(Uuid,Option<i64>,DateTime<Utc>)>=sqlx::query_as("SELECT id,topic_id,updated_at FROM editor_drafts WHERE author_uid=$1 AND external_id=$2 FOR UPDATE")
        .bind(input.author_uid).bind(&input.external_id).fetch_optional(&mut *tx).await?;
    if let Some((_, topic, updated)) = &existing {
        if topic.is_some() {
            return Err(AppError::Conflict(format!(
                "稿件已发布为帖子 {}",
                topic.unwrap()
            )));
        }
        if !input.overwrite {
            return Err(AppError::Conflict(
                "稿件编号已存在；如需更新，请开启覆盖待发布稿".into(),
            ));
        }
        if input.expected_updated_at.is_some_and(|v| v != *updated) {
            return Err(AppError::Conflict(
                "稿件已被其他操作修改，请重新打开".into(),
            ));
        }
    }
    let (category, kind) = check_category(&mut tx, input.author_uid, &input.category).await?;
    let body = topics::clean_body(&body, if kind == "deal" { 0 } else { 10 })?;
    let id = existing.map_or_else(Uuid::new_v4, |v| v.0);
    sqlx::query("INSERT INTO editor_drafts(id,author_uid,external_id,category_id,title,body_md,tags) VALUES ($1,$2,$3,$4,$5,$6,$7) ON CONFLICT(id) DO UPDATE SET category_id=$4,title=$5,body_md=$6,tags=$7,updated_at=clock_timestamp()")
        .bind(id).bind(input.author_uid).bind(&input.external_id).bind(category).bind(title).bind(body).bind(tags).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM editor_draft_assets WHERE draft_id=$1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    let mut written = Vec::new();
    let result:AppResult<()>=async {
        tokio::fs::create_dir_all(&s.config.upload_dir).await.map_err(|e|AppError::Internal(e.to_string()))?;
        for path in used {
            let (image,mime,data)=&assets[&path];let file=s.config.upload_dir.join(format!("img-{image}"));
            written.push(file.clone());
            tokio::fs::write(file,data).await.map_err(|e|AppError::Internal(e.to_string()))?;
            sqlx::query("INSERT INTO images(id,uploader_uid,mime,size,editor_pending) VALUES ($1,$2,$3,$4,true)")
                .bind(image).bind(input.author_uid).bind(mime).bind(data.len() as i64).execute(&mut *tx).await?;
            sqlx::query("INSERT INTO editor_draft_assets(draft_id,image_id,path) VALUES ($1,$2,$3)").bind(id).bind(image).bind(path).execute(&mut *tx).await?;
        }
        log(&mut tx,&e.operator,"editor.draft.save",id,&input.external_id).await?;
        Ok(())
    }.await;
    if let Err(err) = result {
        for p in written {
            let _ = tokio::fs::remove_file(p).await;
        }
        return Err(err);
    }
    // 提交响应丢失时数据库可能已提交，保留文件以免破坏已保存的稿件。
    tx.commit().await?;
    Ok(Json(json!({"id":id,"external_id":input.external_id})))
}
async fn get_draft(s: &AppState, e: &Editor, id: Uuid) -> AppResult<Draft> {
    let row: Draft = sqlx::query_as(&format!("SELECT {COLS} FROM editor_drafts WHERE id=$1"))
        .bind(id)
        .fetch_optional(&s.pool)
        .await?
        .ok_or_else(|| AppError::not_found("稿件"))?;
    e.check(row.author_uid)?;
    Ok(row)
}
async fn detail(
    State(s): State<AppState>,
    e: Editor,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    let d = get_draft(&s, &e, id).await?;
    let rows:Vec<(Uuid,String,String)>=sqlx::query_as("SELECT i.id,a.path,i.mime FROM editor_draft_assets a JOIN images i ON i.id=a.image_id WHERE a.draft_id=$1").bind(id).fetch_all(&s.pool).await?;
    let mut assets = Vec::new();
    let mut paths = HashMap::new();
    let mut html = markdown::render(&d.body_md, true);
    for (image, path, mime) in rows {
        let data = tokio::fs::read(s.config.upload_dir.join(format!("img-{image}")))
            .await
            .map_err(|_| AppError::Conflict(format!("图片文件缺失：{path}")))?;
        let b64 = STANDARD.encode(data);
        let url = format!("/api/images/{image}");
        html = html.replace(
            &format!("src=\"{url}\""),
            &format!("src=\"data:{mime};base64,{b64}\""),
        );
        paths.insert(url, path.clone());
        assets.push(Asset {
            path,
            data_base64: b64,
        });
    }
    let body = rewrite(&d.body_md, |url| {
        Ok(paths.get(url).cloned().unwrap_or_else(|| url.to_string()))
    })?;
    let (category,): (String,) = sqlx::query_as("SELECT slug FROM categories WHERE id=$1")
        .bind(d.category_id)
        .fetch_one(&s.pool)
        .await?;
    Ok(Json(
        json!({"draft":d,"body":body,"category":category,"images":assets,"html":html}),
    ))
}
async fn delete(
    State(s): State<AppState>,
    e: Editor,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    let d = get_draft(&s, &e, id).await?;
    let mut tx = s.pool.begin().await?;
    let result = sqlx::query("DELETE FROM editor_drafts WHERE id=$1 AND topic_id IS NULL")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::Conflict(
            "已发布记录不能删除，请使用帖子管理".into(),
        ));
    }
    log(
        &mut tx,
        &e.operator,
        "editor.draft.delete",
        id,
        &d.external_id,
    )
    .await?;
    tx.commit().await?;
    Ok(Json(json!({"ok":true})))
}
async fn publish(
    State(s): State<AppState>,
    e: Editor,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    if !e.can_publish {
        return Err(AppError::Forbidden("这个令牌只允许导入待发布稿".into()));
    }
    let mut tx = s.pool.begin().await?;
    let d: Draft = sqlx::query_as(&format!(
        "SELECT {COLS} FROM editor_drafts WHERE id=$1 FOR UPDATE"
    ))
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::not_found("稿件"))?;
    e.check(d.author_uid)?;
    if let Some(topic) = d.topic_id {
        return Ok(Json(
            json!({"id":topic,"url":format!("{}/t/{topic}",s.config.public_url),"already_published":true}),
        ));
    }
    let (slug,): (String,) = sqlx::query_as("SELECT slug FROM categories WHERE id=$1")
        .bind(d.category_id)
        .fetch_one(&mut *tx)
        .await?;
    let (_, kind) = check_category(&mut tx, d.author_uid, &slug).await?;
    let title = topics::clean_title(&d.title)?;
    let body = topics::clean_body(&d.body_md, if kind == "deal" { 0 } else { 10 })?;
    let mut tags = topics::clean_tags(&d.tags)?;
    tags.sort();
    // 发布前确认所有文件仍存在，再在同一事务里公开图片和正式帖子。
    let assets: Vec<(Uuid,)> =
        sqlx::query_as("SELECT image_id FROM editor_draft_assets WHERE draft_id=$1")
            .bind(id)
            .fetch_all(&mut *tx)
            .await?;
    for (image,) in assets {
        tokio::fs::metadata(s.config.upload_dir.join(format!("img-{image}")))
            .await
            .map_err(|_| AppError::Conflict("配图文件缺失，请重新导入稿件".into()))?;
    }
    let (topic,):(i64,)=sqlx::query_as("INSERT INTO topics(category_id,author_uid,title,body_md,body_html,excerpt) VALUES ($1,$2,$3,$4,$5,$6) RETURNING id")
        .bind(d.category_id).bind(d.author_uid).bind(title).bind(&body).bind(markdown::render(&body,true)).bind(markdown::excerpt(&body,140)).fetch_one(&mut *tx).await?;
    for tag in tags {
        let (tag_id,):(i32,)=sqlx::query_as("INSERT INTO tags(name) VALUES ($1) ON CONFLICT(name) DO UPDATE SET name=EXCLUDED.name RETURNING id").bind(tag).fetch_one(&mut *tx).await?;
        sqlx::query("INSERT INTO topic_tags(topic_id,tag_id) VALUES ($1,$2)")
            .bind(topic)
            .bind(tag_id)
            .execute(&mut *tx)
            .await?;
    }
    if kind == "deal" {
        sqlx::query("INSERT INTO deals(topic_id) VALUES ($1)")
            .bind(topic)
            .execute(&mut *tx)
            .await?;
    }
    super::help::create_for_topic(&mut tx, topic, d.author_uid, d.category_id, None).await?;
    sqlx::query("UPDATE images SET editor_pending=false WHERE id IN (SELECT image_id FROM editor_draft_assets WHERE draft_id=$1)").bind(id).execute(&mut *tx).await?;
    sqlx::query("UPDATE editor_drafts SET topic_id=$2,updated_at=clock_timestamp() WHERE id=$1")
        .bind(id)
        .bind(topic)
        .execute(&mut *tx)
        .await?;
    log(
        &mut tx,
        &e.operator,
        "editor.publish",
        topic,
        &d.external_id,
    )
    .await?;
    tx.commit().await?;
    Ok(Json(
        json!({"id":topic,"url":format!("{}/t/{topic}",s.config.public_url),"already_published":false}),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn paths_cannot_escape_or_use_remote_urls() {
        for bad in [
            "../a.png",
            "/a.png",
            "https://a/x",
            "a\\b.png",
            "%2e%2e/a.png",
            "a/./b.png",
        ] {
            assert!(clean_path(bad).is_err(), "{bad}");
        }
        assert_eq!(
            clean_path("./images/示意图.png").unwrap(),
            "images/示意图.png"
        );
    }
    #[test]
    fn replaces_inline_and_reference_images_but_keeps_code() {
        let source="![示意](images/a.png)\n\n![引用][img]\n\n[img]: images/a.png\n\n```md\n![例子](images/missing.png)\n```";
        let mut count = 0;
        let out = rewrite(source, |path| {
            assert_eq!(path, "images/a.png");
            count += 1;
            Ok("/api/images/id".into())
        })
        .unwrap();
        assert_eq!(count, 2);
        assert_eq!(out.matches("</api/images/id>").count(), 2);
        assert!(out.contains("![例子](images/missing.png)"));
        assert!(markdown::render(&out, true).contains("src=\"/api/images/id\""));
    }
    #[test]
    fn rejects_missing_images_and_html_image_bypass() {
        assert!(rewrite("![x](missing.png)", |_| Err(AppError::BadRequest(
            "missing".into()
        )))
        .is_err());
        assert!(rewrite("<img src='local.png'>", |p| Ok(p.to_string())).is_err());
    }
}

#[cfg(test)]
#[path = "editor_tests.rs"]
mod integration_tests;
