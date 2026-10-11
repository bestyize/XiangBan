//! 管理后台接口（/api/admin/*）：管理员登录、举报队列、内容与用户管理、分类与产品库、操作日志。
//! 管理员账号独立建表，令牌是 HS256 JWT（带 token_version，改密码后旧令牌立即失效）。

use crate::{
    config::AppConfig,
    error::{AppError, AppResult},
    models::{Page, DEAL_STATUSES},
    state::AppState,
};
use axum::{
    extract::{FromRequestParts, Path, Query, State},
    http::{header, request::Parts},
    routing::{get, patch, post, put},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{PgConnection, PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

/// 连续输错密码的上限，达到后锁定 15 分钟。
const MAX_FAILED_LOGINS: i32 = 5;
const LOCK_MINUTES: i64 = 15;
const BAD_CREDENTIALS: &str = "用户名或密码错误";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/me", get(me))
        .route("/password", post(change_password))
        .route("/stats", get(stats))
        .route("/reports", get(list_reports))
        .route("/reports/{id}", post(handle_report))
        .route("/topics", get(list_topics))
        .route("/topics/{id}/moderate", post(moderate_topic))
        .route("/topics/{id}/move", post(move_topic))
        .route("/deals/{id}/status", put(set_deal_status))
        .route("/posts/{id}/moderate", post(moderate_post))
        .route("/users", get(list_users))
        .route("/users/{uid}/trust", post(set_trust))
        .route("/users/{uid}/status", post(set_user_status))
        .route("/categories", get(list_categories).post(create_category))
        .route("/categories/{id}", patch(update_category))
        .route("/products", get(list_products).post(create_product))
        .route("/products/{id}", patch(update_product))
        .route("/audit", get(list_audit))
}

// ───────────────────────── 令牌与密码 ─────────────────────────

mod token {
    use super::*;
    use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct AdminClaims {
        pub sub: String,
        pub tv: i32,
        pub exp: i64,
        pub iat: i64,
    }

    pub fn issue(secret: &[u8], admin_id: Uuid, token_version: i32, ttl_secs: i64) -> AppResult<String> {
        let now = Utc::now().timestamp();
        let claims = AdminClaims { sub: admin_id.to_string(), tv: token_version, exp: now + ttl_secs, iat: now };
        encode(&Header::new(Algorithm::HS256), &claims, &EncodingKey::from_secret(secret))
            .map_err(|e| AppError::Internal(format!("sign admin token: {e}")))
    }

    pub fn verify(secret: &[u8], token: &str) -> AppResult<AdminClaims> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.leeway = 5;
        decode::<AdminClaims>(token, &DecodingKey::from_secret(secret), &validation)
            .map(|data| data.claims)
            .map_err(|_| AppError::Unauthorized("管理员登录已过期，请重新登录".into()))
    }
}

mod password {
    use super::*;
    use argon2::{
        password_hash::{phc::PasswordHash, PasswordHasher, PasswordVerifier},
        Argon2,
    };
    use std::sync::OnceLock;

    pub const MIN_LEN: usize = 8;
    pub const MAX_LEN: usize = 128;

    pub fn validate(password: &str) -> AppResult<()> {
        if !(MIN_LEN..=MAX_LEN).contains(&password.chars().count()) {
            return Err(AppError::BadRequest(format!("密码长度需为 {MIN_LEN}-{MAX_LEN} 位")));
        }
        Ok(())
    }

    fn hash_sync(password: &str) -> Result<String, AppError> {
        Argon2::default()
            .hash_password(password.as_bytes())
            .map(|h: PasswordHash| h.to_string())
            .map_err(|e| AppError::Internal(format!("password hashing failed: {e}")))
    }

    pub async fn hash(password: String) -> AppResult<String> {
        tokio::task::spawn_blocking(move || hash_sync(&password))
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?
    }

    pub async fn verify(password: String, hash: String) -> bool {
        tokio::task::spawn_blocking(move || Argon2::default().verify_password(password.as_bytes(), hash.as_str()).is_ok())
            .await
            .unwrap_or(false)
    }

    /// 用户名不存在时也跑一次同样代价的校验，避免靠响应时间枚举管理员账号。
    pub async fn verify_dummy(password: String) {
        static DUMMY: OnceLock<String> = OnceLock::new();
        let _ = tokio::task::spawn_blocking(move || {
            let hash = DUMMY.get_or_init(|| hash_sync("dummy-password-for-timing").unwrap_or_default());
            let _ = Argon2::default().verify_password(password.as_bytes(), hash.as_str());
        })
        .await;
    }
}

fn normalize_username(raw: &str) -> AppResult<String> {
    let name = raw.trim().to_lowercase();
    let ok = (3..=32).contains(&name.len())
        && name.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'));
    if ok {
        Ok(name)
    } else {
        Err(AppError::BadRequest("管理员用户名需为 3-32 位字母、数字、下划线、点或连字符".into()))
    }
}

/// 没有管理员时用 FORUM_ADMIN_USER / FORUM_ADMIN_PASSWORD 创建初始管理员；已有管理员则不覆盖。
pub async fn seed_admin(pool: &PgPool, config: &AppConfig) -> Result<(), AppError> {
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM admins").fetch_one(pool).await?;
    if count > 0 {
        return Ok(());
    }
    let (Some(user), Some(pass)) = (&config.admin_user, &config.admin_password) else {
        tracing::warn!("尚无管理员账号：设置 FORUM_ADMIN_USER 与 FORUM_ADMIN_PASSWORD 后重启即可创建初始管理员");
        return Ok(());
    };
    let username = normalize_username(user)?;
    password::validate(pass)?;
    let hash = password::hash(pass.clone()).await?;
    sqlx::query("INSERT INTO admins (id, username, password_hash) VALUES ($1, $2, $3)")
        .bind(Uuid::new_v4())
        .bind(&username)
        .bind(hash)
        .execute(pool)
        .await?;
    tracing::info!("已创建初始管理员 {username}，登录后请在后台修改密码并从环境变量中移除 FORUM_ADMIN_PASSWORD");
    Ok(())
}

// ───────────────────────── 提取器与通用工具 ─────────────────────────

pub struct AdminIdentity {
    pub id: Uuid,
    pub username: String,
}

/// 需要管理员令牌的接口都带上它：校验签名、有效期，并核对库里的 token_version。
pub struct AuthAdmin(pub AdminIdentity);

impl FromRequestParts<AppState> for AuthAdmin {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let raw = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or_else(|| AppError::Unauthorized("缺少管理员凭证".into()))?;
        let expired = || AppError::Unauthorized("管理员登录已过期，请重新登录".into());
        let claims = token::verify(&state.config.admin_jwt_secret, raw)?;
        let id = Uuid::parse_str(&claims.sub).map_err(|_| expired())?;
        let row: Option<(String, i32)> = sqlx::query_as("SELECT username, token_version FROM admins WHERE id = $1")
            .bind(id)
            .fetch_optional(&state.pool)
            .await?;
        match row {
            Some((username, version)) if version == claims.tv => Ok(AuthAdmin(AdminIdentity { id, username })),
            _ => Err(expired()),
        }
    }
}

/// 写一条操作日志。传连接是为了能和业务改动放在同一个事务里。
async fn audit(
    conn: &mut PgConnection,
    admin: &AdminIdentity,
    action: &str,
    target_type: &str,
    target_id: impl ToString,
    detail: impl AsRef<str>,
) -> AppResult<()> {
    sqlx::query("INSERT INTO admin_audit (admin, action, target_type, target_id, detail) VALUES ($1, $2, $3, $4, $5)")
        .bind(&admin.username)
        .bind(action)
        .bind(target_type)
        .bind(target_id.to_string())
        .bind(detail.as_ref())
        .execute(conn)
        .await?;
    Ok(())
}

/// 分类、产品的 slug：小写字母、数字、连字符，2-40 位。
fn validate_slug(slug: &str) -> AppResult<()> {
    let ok = (2..=40).contains(&slug.len())
        && slug.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    if ok {
        Ok(())
    } else {
        Err(AppError::BadRequest("slug 需为 2-40 位小写字母、数字或连字符".into()))
    }
}

fn validate_name(name: &str, what: &str, max: usize) -> AppResult<String> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > max {
        return Err(AppError::BadRequest(format!("{what}需为 1-{max} 个字")));
    }
    Ok(name.to_string())
}

fn validate_text(text: &str, what: &str, max: usize) -> AppResult<String> {
    let text = text.trim();
    if text.chars().count() > max {
        return Err(AppError::BadRequest(format!("{what}最多 {max} 个字")));
    }
    Ok(text.to_string())
}

/// 官网地址：空，或 http(s) 开头且不含空白。
fn validate_website(url: &str) -> AppResult<String> {
    let url = url.trim();
    if url.is_empty() {
        return Ok(String::new());
    }
    if url.len() > 300 || !(url.starts_with("https://") || url.starts_with("http://")) || url.contains(char::is_whitespace) {
        return Err(AppError::BadRequest("官网需要是 http(s) 地址".into()));
    }
    Ok(url.to_string())
}

/// 空串当作没传。
fn opt(v: &Option<String>) -> Option<&str> {
    v.as_deref().map(str::trim).filter(|s| !s.is_empty())
}

/// 转义 LIKE 通配符后包成 %q%。
fn like_pattern(q: &str) -> String {
    let escaped = q.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_");
    format!("%{escaped}%")
}

/// 返回 (page, page_size, offset)。
fn paging(page: Option<i64>, page_size: Option<i64>) -> (i64, i64, i64) {
    let page = page.unwrap_or(1).clamp(1, 100_000);
    let size = page_size.unwrap_or(30).clamp(1, 100);
    (page, size, (page - 1) * size)
}

fn is_unique_violation(err: &sqlx::Error) -> bool {
    err.as_database_error().is_some_and(|e| e.is_unique_violation())
}

/// 截取前 n 个字符，单行化，用于日志和摘要。
fn clip(s: &str, n: usize) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ").chars().take(n).collect()
}

// ───────────────────────── 登录与账号 ─────────────────────────

#[derive(Deserialize)]
struct LoginReq {
    username: String,
    password: String,
}

async fn login(State(state): State<AppState>, Json(req): Json<LoginReq>) -> AppResult<Json<Value>> {
    let username = req.username.trim().to_lowercase();
    let row: Option<(Uuid, String, i32, bool)> = sqlx::query_as(
        "SELECT id, password_hash, token_version, COALESCE(locked_until > now(), false) FROM admins WHERE username = $1",
    )
    .bind(&username)
    .fetch_optional(&state.pool)
    .await?;

    let Some((id, hash, token_version, locked)) = row else {
        password::verify_dummy(req.password).await;
        return Err(AppError::Unauthorized(BAD_CREDENTIALS.into()));
    };
    if locked {
        return Err(AppError::TooManyRequests {
            message: format!("密码错误次数过多，请 {LOCK_MINUTES} 分钟后再试"),
            retry_after: LOCK_MINUTES * 60,
        });
    }
    // 超长输入直接当作错误，避免被拿来放大哈希开销。
    let verified = req.password.len() <= 1024 && password::verify(req.password, hash).await;
    if !verified {
        sqlx::query(
            "UPDATE admins SET
                locked_until = CASE WHEN failed_logins + 1 >= $2 THEN now() + make_interval(mins => $3::int) ELSE locked_until END,
                failed_logins = CASE WHEN failed_logins + 1 >= $2 THEN 0 ELSE failed_logins + 1 END
             WHERE id = $1",
        )
        .bind(id)
        .bind(MAX_FAILED_LOGINS)
        .bind(LOCK_MINUTES as i32)
        .execute(&state.pool)
        .await?;
        return Err(AppError::Unauthorized(BAD_CREDENTIALS.into()));
    }

    sqlx::query("UPDATE admins SET last_login_at = now(), failed_logins = 0, locked_until = NULL WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;
    let ttl = state.config.admin_token_ttl_secs;
    let token = token::issue(&state.config.admin_jwt_secret, id, token_version, ttl)?;
    let mut conn = state.pool.acquire().await?;
    audit(&mut conn, &AdminIdentity { id, username: username.clone() }, "admin.login", "admin", id, "").await?;
    Ok(Json(json!({ "token": token, "expires_in": ttl, "admin": { "id": id, "username": username } })))
}

async fn me(AuthAdmin(admin): AuthAdmin) -> Json<Value> {
    Json(json!({ "id": admin.id, "username": admin.username }))
}

#[derive(Deserialize)]
struct ChangePasswordReq {
    current_password: String,
    new_password: String,
}

/// 改密码会让所有旧令牌失效，响应里附带一个新令牌，前端可以直接换上继续用。
async fn change_password(
    State(state): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Json(req): Json<ChangePasswordReq>,
) -> AppResult<Json<Value>> {
    password::validate(&req.new_password)?;
    let (hash,): (String,) =
        sqlx::query_as("SELECT password_hash FROM admins WHERE id = $1").bind(admin.id).fetch_one(&state.pool).await?;
    if !password::verify(req.current_password, hash).await {
        return Err(AppError::Unauthorized("当前密码错误".into()));
    }
    let new_hash = password::hash(req.new_password).await?;
    let mut tx = state.pool.begin().await?;
    let (version,): (i32,) = sqlx::query_as(
        "UPDATE admins SET password_hash = $2, token_version = token_version + 1 WHERE id = $1 RETURNING token_version",
    )
    .bind(admin.id)
    .bind(new_hash)
    .fetch_one(&mut *tx)
    .await?;
    audit(&mut tx, &admin, "admin.password_change", "admin", admin.id, "").await?;
    tx.commit().await?;
    let ttl = state.config.admin_token_ttl_secs;
    let token = token::issue(&state.config.admin_jwt_secret, admin.id, version, ttl)?;
    Ok(Json(json!({ "ok": true, "token": token, "expires_in": ttl })))
}

// ───────────────────────── 数据概览 ─────────────────────────

async fn stats(State(state): State<AppState>, _admin: AuthAdmin) -> AppResult<Json<Value>> {
    let row: (i64, i64, i64, i64, i64, i64, i64, i64, i64, i64) = sqlx::query_as(
        "SELECT
            (SELECT COUNT(*) FROM users),
            (SELECT COUNT(*) FROM users WHERE created_at > now() - interval '1 day'),
            (SELECT COUNT(*) FROM users WHERE created_at > now() - interval '7 days'),
            (SELECT COUNT(*) FROM topics),
            (SELECT COUNT(*) FROM topics WHERE created_at > now() - interval '1 day'),
            (SELECT COUNT(*) FROM posts),
            (SELECT COUNT(*) FROM posts WHERE created_at > now() - interval '1 day'),
            (SELECT COUNT(*) FROM deals d JOIN topics t ON t.id = d.topic_id
                WHERE d.deal_status <> 'expired' AND t.status = 'normal'),
            (SELECT COUNT(*) FROM reports WHERE status = 'open'),
            (SELECT COUNT(*) FROM users WHERE last_seen_at > now() - interval '1 day')",
    )
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(json!({
        "users_total": row.0,
        "users_new_24h": row.1,
        "users_new_7d": row.2,
        "topics_total": row.3,
        "topics_24h": row.4,
        "posts_total": row.5,
        "posts_24h": row.6,
        "deals_active": row.7,
        "reports_open": row.8,
        "dau": row.9,
    })))
}

// ───────────────────────── 举报 ─────────────────────────

#[derive(Deserialize)]
struct ReportQuery {
    status: Option<String>,
    page: Option<i64>,
    page_size: Option<i64>,
}

#[derive(sqlx::FromRow)]
struct ReportRow {
    id: i64,
    target_type: String,
    target_id: i64,
    reporter_uid: Uuid,
    reporter_nickname: Option<String>,
    reason: String,
    detail: String,
    status: String,
    handled_by: Option<String>,
    handled_at: Option<DateTime<Utc>>,
    result: Option<String>,
    created_at: DateTime<Utc>,
    report_count: i64,
    topic_id: Option<i64>,
    topic_title: Option<String>,
    target_status: Option<String>,
    floor: Option<i32>,
    excerpt: Option<String>,
}

impl ReportRow {
    /// 被举报对象的摘要：帖子给 id/标题/状态，回复再加楼层、所在帖子和正文前 120 字。目标已不存在时 exists=false。
    fn target(&self) -> Value {
        let exists = self.target_status.is_some();
        if self.target_type == "topic" {
            json!({ "type": "topic", "id": self.target_id, "exists": exists, "title": self.topic_title, "status": self.target_status })
        } else {
            json!({
                "type": "post", "id": self.target_id, "exists": exists, "topic_id": self.topic_id,
                "topic_title": self.topic_title, "floor": self.floor, "excerpt": self.excerpt, "status": self.target_status,
            })
        }
    }

    fn into_json(self) -> Value {
        let target = self.target();
        json!({
            "id": self.id, "target_type": self.target_type, "target_id": self.target_id, "target": target,
            "reporter": { "uid": self.reporter_uid, "nickname": self.reporter_nickname },
            "reason": self.reason, "detail": self.detail, "status": self.status,
            "handled_by": self.handled_by, "handled_at": self.handled_at, "result": self.result,
            "created_at": self.created_at,
            // 同一对象累计被举报的次数（含已处理的）
            "report_count": self.report_count,
        })
    }
}

async fn list_reports(
    State(state): State<AppState>,
    _admin: AuthAdmin,
    Query(q): Query<ReportQuery>,
) -> AppResult<Json<Page<Value>>> {
    let status = opt(&q.status).unwrap_or("open").to_string();
    if !matches!(status.as_str(), "open" | "resolved" | "dismissed" | "all") {
        return Err(AppError::BadRequest("status 只能是 open / resolved / dismissed / all".into()));
    }
    let (page, page_size, offset) = paging(q.page, q.page_size);
    let filter = |qb: &mut QueryBuilder<'_, Postgres>| {
        if status != "all" {
            qb.push(" WHERE r.status = ").push_bind(status.clone());
        }
    };

    let mut count = QueryBuilder::<Postgres>::new("SELECT COUNT(*) FROM reports r");
    filter(&mut count);
    let (total,): (i64,) = count.build_query_as().fetch_one(&state.pool).await?;

    let mut qb = QueryBuilder::<Postgres>::new(
        "SELECT r.id, r.target_type, r.target_id, r.reporter_uid, u.nickname AS reporter_nickname,
                r.reason, r.detail, r.status, r.handled_by, r.handled_at, r.result, r.created_at,
                (SELECT COUNT(*) FROM reports r2 WHERE r2.target_type = r.target_type AND r2.target_id = r.target_id) AS report_count,
                COALESCE(t.id, p.topic_id) AS topic_id,
                COALESCE(t.title, pt.title) AS topic_title,
                COALESCE(t.status, p.status) AS target_status,
                p.floor,
                left(p.body_md, 120) AS excerpt
         FROM reports r
         LEFT JOIN users u ON u.uid = r.reporter_uid
         LEFT JOIN topics t ON r.target_type = 'topic' AND t.id = r.target_id
         LEFT JOIN posts p ON r.target_type = 'post' AND p.id = r.target_id
         LEFT JOIN topics pt ON pt.id = p.topic_id",
    );
    filter(&mut qb);
    // 待处理的先看最早的，已处理的看最近的
    qb.push(if status == "open" { " ORDER BY r.created_at ASC, r.id ASC" } else { " ORDER BY r.created_at DESC, r.id DESC" });
    qb.push(" LIMIT ").push_bind(page_size).push(" OFFSET ").push_bind(offset);
    let rows: Vec<ReportRow> = qb.build_query_as().fetch_all(&state.pool).await?;

    Ok(Json(Page { items: rows.into_iter().map(ReportRow::into_json).collect(), page, page_size, total }))
}

#[derive(Deserialize)]
struct ReportAction {
    action: String,
    result: Option<String>,
    #[serde(default)]
    hide_target: bool,
}

async fn handle_report(
    State(state): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Path(id): Path<i64>,
    Json(input): Json<ReportAction>,
) -> AppResult<Json<Value>> {
    let new_status = match input.action.as_str() {
        "resolve" => "resolved",
        "dismiss" => "dismissed",
        _ => return Err(AppError::BadRequest("action 只能是 resolve 或 dismiss".into())),
    };
    if input.hide_target && new_status != "resolved" {
        return Err(AppError::BadRequest("驳回举报时不能隐藏内容".into()));
    }
    let result = validate_text(input.result.as_deref().unwrap_or(""), "处理说明", 500)?;

    let mut tx = state.pool.begin().await?;
    let row: Option<(String, i64, String)> =
        sqlx::query_as("SELECT target_type, target_id, status FROM reports WHERE id = $1 FOR UPDATE")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?;
    let (target_type, target_id, status) = row.ok_or_else(|| AppError::not_found("举报"))?;
    if status != "open" {
        return Err(AppError::Conflict("这条举报已经处理过了".into()));
    }

    // 处理结果同样应用到同一对象上其他待处理的举报
    let affected = if new_status == "resolved" {
        sqlx::query(
            "UPDATE reports SET status = 'resolved', handled_by = $3, handled_at = now(), result = $4
             WHERE target_type = $1 AND target_id = $2 AND status = 'open'",
        )
        .bind(&target_type)
        .bind(target_id)
        .bind(&admin.username)
        .bind(&result)
        .execute(&mut *tx)
        .await?
        .rows_affected()
    } else {
        sqlx::query("UPDATE reports SET status = 'dismissed', handled_by = $2, handled_at = now(), result = $3 WHERE id = $1")
            .bind(id)
            .bind(&admin.username)
            .bind(&result)
            .execute(&mut *tx)
            .await?
            .rows_affected()
    };

    let mut hidden = false;
    let mut touched_topic = None;
    if input.hide_target {
        if target_type == "topic" {
            hidden = sqlx::query("UPDATE topics SET status = 'hidden' WHERE id = $1 AND status = 'normal'")
                .bind(target_id)
                .execute(&mut *tx)
                .await?
                .rows_affected()
                > 0;
        } else {
            touched_topic = hide_post(&mut tx, target_id).await?;
            hidden = touched_topic.is_some();
        }
    }
    audit(
        &mut tx,
        &admin,
        &format!("report.{}", input.action),
        "report",
        id,
        format!("{target_type}#{target_id} 共处理 {affected} 条；隐藏内容={hidden}；{}", clip(&result, 100)),
    )
    .await?;
    tx.commit().await?;
    if let Some(topic_id) = touched_topic {
        crate::models::refresh_hot_scores(&state.pool, Some(topic_id)).await?;
    }
    Ok(Json(json!({ "ok": true, "status": new_status, "reports_handled": affected, "target_hidden": hidden })))
}

// ───────────────────────── 帖子与回复 ─────────────────────────

#[derive(Deserialize)]
struct TopicQuery {
    q: Option<String>,
    status: Option<String>,
    /// 分类 id 或 slug；选父分类时包含子分类
    category: Option<String>,
    page: Option<i64>,
    page_size: Option<i64>,
}

#[derive(Serialize, sqlx::FromRow)]
struct AdminTopic {
    id: i64,
    title: String,
    status: String,
    category_id: i32,
    category_name: String,
    author_uid: Uuid,
    author_nickname: String,
    pinned: bool,
    featured: bool,
    locked: bool,
    reply_count: i32,
    created_at: DateTime<Utc>,
    deal_status: Option<String>,
    open_reports: i64,
}

async fn list_topics(
    State(state): State<AppState>,
    _admin: AuthAdmin,
    Query(q): Query<TopicQuery>,
) -> AppResult<Json<Page<AdminTopic>>> {
    let status = opt(&q.status).unwrap_or("all").to_string();
    if !matches!(status.as_str(), "normal" | "hidden" | "deleted" | "all") {
        return Err(AppError::BadRequest("status 只能是 normal / hidden / deleted / all".into()));
    }
    let (page, page_size, offset) = paging(q.page, q.page_size);
    let keyword = opt(&q.q).map(str::to_string);
    let category = opt(&q.category).map(str::to_string);

    let filter = |qb: &mut QueryBuilder<'_, Postgres>| {
        qb.push(" WHERE true");
        if status != "all" {
            qb.push(" AND t.status = ").push_bind(status.clone());
        }
        if let Some(k) = &keyword {
            qb.push(" AND (t.title ILIKE ").push_bind(like_pattern(k));
            if let Ok(id) = k.parse::<i64>() {
                qb.push(" OR t.id = ").push_bind(id);
            }
            qb.push(")");
        }
        if let Some(c) = &category {
            match c.parse::<i32>() {
                Ok(id) => qb.push(" AND (c.id = ").push_bind(id).push(" OR c.parent_id = ").push_bind(id).push(")"),
                Err(_) => qb.push(" AND (c.slug = ").push_bind(c.clone()).push(" OR c.parent_id = (SELECT id FROM categories WHERE slug = ").push_bind(c.clone()).push("))"),
            };
        }
    };
    const FROM: &str = " FROM topics t JOIN categories c ON c.id = t.category_id
         JOIN users u ON u.uid = t.author_uid LEFT JOIN deals d ON d.topic_id = t.id";

    let mut count = QueryBuilder::<Postgres>::new(format!("SELECT COUNT(*){FROM}"));
    filter(&mut count);
    let (total,): (i64,) = count.build_query_as().fetch_one(&state.pool).await?;

    let mut qb = QueryBuilder::<Postgres>::new(format!(
        "SELECT t.id, t.title, t.status, t.category_id, c.name AS category_name, t.author_uid, u.nickname AS author_nickname,
                t.pinned, t.featured, t.locked, t.reply_count, t.created_at, d.deal_status,
                (SELECT COUNT(*) FROM reports r WHERE r.target_type = 'topic' AND r.target_id = t.id AND r.status = 'open') AS open_reports{FROM}"
    ));
    filter(&mut qb);
    qb.push(" ORDER BY t.id DESC LIMIT ").push_bind(page_size).push(" OFFSET ").push_bind(offset);
    let items: Vec<AdminTopic> = qb.build_query_as().fetch_all(&state.pool).await?;
    Ok(Json(Page { items, page, page_size, total }))
}

#[derive(Deserialize)]
struct ModerateInput {
    action: String,
}

async fn moderate_topic(
    State(state): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Path(id): Path<i64>,
    Json(input): Json<ModerateInput>,
) -> AppResult<Json<Value>> {
    let mut tx = state.pool.begin().await?;
    let (status,): (String,) = sqlx::query_as("SELECT status FROM topics WHERE id = $1 FOR UPDATE")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::not_found("帖子"))?;
    let sql = match (input.action.as_str(), status.as_str()) {
        ("hide", "normal") => "UPDATE topics SET status = 'hidden' WHERE id = $1",
        ("hide", _) => return Err(AppError::Conflict("只有正常状态的帖子可以隐藏".into())),
        ("restore", "hidden" | "deleted") => "UPDATE topics SET status = 'normal' WHERE id = $1",
        ("restore", _) => return Err(AppError::Conflict("帖子本来就是正常状态".into())),
        ("delete", "deleted") => return Err(AppError::Conflict("帖子已经删除".into())),
        ("delete", _) => "UPDATE topics SET status = 'deleted' WHERE id = $1",
        ("pin", _) => "UPDATE topics SET pinned = true WHERE id = $1",
        ("unpin", _) => "UPDATE topics SET pinned = false WHERE id = $1",
        ("feature", _) => "UPDATE topics SET featured = true WHERE id = $1",
        ("unfeature", _) => "UPDATE topics SET featured = false WHERE id = $1",
        ("lock", _) => "UPDATE topics SET locked = true WHERE id = $1",
        ("unlock", _) => "UPDATE topics SET locked = false WHERE id = $1",
        _ => return Err(AppError::BadRequest("不支持的操作".into())),
    };
    sqlx::query(sql).bind(id).execute(&mut *tx).await?;
    let snapshot: (String, bool, bool, bool) =
        sqlx::query_as("SELECT status, pinned, featured, locked FROM topics WHERE id = $1").bind(id).fetch_one(&mut *tx).await?;
    audit(&mut tx, &admin, &format!("topic.{}", input.action), "topic", id, format!("{status} -> {}", snapshot.0)).await?;
    tx.commit().await?;
    Ok(Json(json!({ "id": id, "status": snapshot.0, "pinned": snapshot.1, "featured": snapshot.2, "locked": snapshot.3 })))
}

#[derive(Deserialize)]
struct MoveInput {
    category_id: i32,
}

/// 只能移到叶子分类，且和帖子原分类同类型（羊毛帖只在羊毛分类之间移动）。
async fn move_topic(
    State(state): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Path(id): Path<i64>,
    Json(input): Json<MoveInput>,
) -> AppResult<Json<Value>> {
    let mut tx = state.pool.begin().await?;
    let (from_id, from_kind): (i32, String) = sqlx::query_as(
        "SELECT t.category_id, c.kind FROM topics t JOIN categories c ON c.id = t.category_id WHERE t.id = $1 FOR UPDATE OF t",
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::not_found("帖子"))?;
    let (kind, name, has_children): (String, String, bool) = sqlx::query_as(
        "SELECT kind, name, EXISTS (SELECT 1 FROM categories ch WHERE ch.parent_id = categories.id) FROM categories WHERE id = $1",
    )
    .bind(input.category_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::BadRequest("目标分类不存在".into()))?;
    if has_children {
        return Err(AppError::BadRequest("请选择具体的子分类".into()));
    }
    if kind != from_kind {
        return Err(AppError::BadRequest(if from_kind == "deal" {
            "羊毛帖只能移动到羊毛分类".into()
        } else {
            "普通帖不能移动到羊毛分类".into()
        }));
    }
    sqlx::query("UPDATE topics SET category_id = $2 WHERE id = $1").bind(id).bind(input.category_id).execute(&mut *tx).await?;
    audit(&mut tx, &admin, "topic.move", "topic", id, format!("{from_id} -> {} ({name})", input.category_id)).await?;
    tx.commit().await?;
    Ok(Json(json!({ "id": id, "category_id": input.category_id })))
}

#[derive(Deserialize)]
struct DealStatusInput {
    status: String,
}

async fn set_deal_status(
    State(state): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Path(topic_id): Path<i64>,
    Json(input): Json<DealStatusInput>,
) -> AppResult<Json<Value>> {
    if !DEAL_STATUSES.contains(&input.status.as_str()) {
        return Err(AppError::BadRequest("状态只能是 valid / suspect / expired".into()));
    }
    let mut tx = state.pool.begin().await?;
    let old: Option<(String,)> = sqlx::query_as("SELECT deal_status FROM deals WHERE topic_id = $1 FOR UPDATE")
        .bind(topic_id)
        .fetch_optional(&mut *tx)
        .await?;
    let (old,) = old.ok_or_else(|| AppError::not_found("羊毛"))?;
    sqlx::query("UPDATE deals SET deal_status = $2 WHERE topic_id = $1").bind(topic_id).bind(&input.status).execute(&mut *tx).await?;
    audit(&mut tx, &admin, "deal.status", "topic", topic_id, format!("{old} -> {}", input.status)).await?;
    tx.commit().await?;
    Ok(Json(json!({ "topic_id": topic_id, "deal_status": input.status })))
}

/// 隐藏回复并同步楼主帖的回复数；已不是正常状态则什么都不做。返回受影响的帖子 id。
async fn hide_post(conn: &mut PgConnection, post_id: i64) -> AppResult<Option<i64>> {
    let row: Option<(i64,)> =
        sqlx::query_as("UPDATE posts SET status = 'hidden' WHERE id = $1 AND status = 'normal' RETURNING topic_id")
            .bind(post_id)
            .fetch_optional(&mut *conn)
            .await?;
    let Some((topic_id,)) = row else { return Ok(None) };
    sqlx::query("UPDATE topics SET reply_count = GREATEST(reply_count - 1, 0) WHERE id = $1").bind(topic_id).execute(conn).await?;
    Ok(Some(topic_id))
}

async fn restore_post(conn: &mut PgConnection, post_id: i64) -> AppResult<Option<i64>> {
    let row: Option<(i64,)> =
        sqlx::query_as("UPDATE posts SET status = 'normal' WHERE id = $1 AND status = 'hidden' RETURNING topic_id")
            .bind(post_id)
            .fetch_optional(&mut *conn)
            .await?;
    let Some((topic_id,)) = row else { return Ok(None) };
    sqlx::query("UPDATE topics SET reply_count = reply_count + 1 WHERE id = $1").bind(topic_id).execute(conn).await?;
    Ok(Some(topic_id))
}

async fn moderate_post(
    State(state): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Path(id): Path<i64>,
    Json(input): Json<ModerateInput>,
) -> AppResult<Json<Value>> {
    if !matches!(input.action.as_str(), "hide" | "restore") {
        return Err(AppError::BadRequest("action 只能是 hide 或 restore".into()));
    }
    let mut tx = state.pool.begin().await?;
    let (status,): (String,) = sqlx::query_as("SELECT status FROM posts WHERE id = $1 FOR UPDATE")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::not_found("回复"))?;
    let touched = if input.action == "hide" { hide_post(&mut tx, id).await? } else { restore_post(&mut tx, id).await? };
    let Some(topic_id) = touched else {
        return Err(AppError::Conflict(match (input.action.as_str(), status.as_str()) {
            ("restore", "deleted") => "作者已删除的回复不能恢复".into(),
            ("hide", _) => "只有正常状态的回复可以隐藏".into(),
            _ => "回复不在隐藏状态".into(),
        }));
    };
    audit(&mut tx, &admin, &format!("post.{}", input.action), "post", id, format!("topic#{topic_id}")).await?;
    tx.commit().await?;
    crate::models::refresh_hot_scores(&state.pool, Some(topic_id)).await?;
    Ok(Json(json!({ "id": id, "topic_id": topic_id, "status": if input.action == "hide" { "hidden" } else { "normal" } })))
}

// ───────────────────────── 用户 ─────────────────────────

#[derive(Deserialize)]
struct UserQuery {
    /// 昵称模糊匹配，或完整的 uid
    q: Option<String>,
    status: Option<String>,
    page: Option<i64>,
    page_size: Option<i64>,
}

#[derive(Serialize, sqlx::FromRow)]
struct AdminUser {
    uid: Uuid,
    nickname: String,
    trust_level: i16,
    points: i32,
    status: String,
    muted_until: Option<DateTime<Utc>>,
    read_count: i32,
    visit_days: i32,
    created_at: DateTime<Utc>,
    last_seen_at: DateTime<Utc>,
    topic_count: i64,
    post_count: i64,
}

async fn list_users(
    State(state): State<AppState>,
    _admin: AuthAdmin,
    Query(q): Query<UserQuery>,
) -> AppResult<Json<Page<AdminUser>>> {
    let status = opt(&q.status).map(str::to_string);
    if let Some(s) = &status {
        if !matches!(s.as_str(), "active" | "muted" | "banned") {
            return Err(AppError::BadRequest("status 只能是 active / muted / banned".into()));
        }
    }
    let keyword = opt(&q.q).map(str::to_string);
    let (page, page_size, offset) = paging(q.page, q.page_size);

    let filter = |qb: &mut QueryBuilder<'_, Postgres>| {
        qb.push(" WHERE true");
        if let Some(s) = &status {
            qb.push(" AND u.status = ").push_bind(s.clone());
        }
        if let Some(k) = &keyword {
            qb.push(" AND (u.nickname ILIKE ").push_bind(like_pattern(k));
            if let Ok(uid) = Uuid::parse_str(k) {
                qb.push(" OR u.uid = ").push_bind(uid);
            }
            qb.push(")");
        }
    };

    let mut count = QueryBuilder::<Postgres>::new("SELECT COUNT(*) FROM users u");
    filter(&mut count);
    let (total,): (i64,) = count.build_query_as().fetch_one(&state.pool).await?;

    let mut qb = QueryBuilder::<Postgres>::new(
        "SELECT u.uid, u.nickname, u.trust_level, u.points, u.status, u.muted_until, u.read_count, u.visit_days,
                u.created_at, u.last_seen_at,
                (SELECT COUNT(*) FROM topics t WHERE t.author_uid = u.uid AND t.status = 'normal') AS topic_count,
                (SELECT COUNT(*) FROM posts p WHERE p.author_uid = u.uid AND p.status = 'normal') AS post_count
         FROM users u",
    );
    filter(&mut qb);
    qb.push(" ORDER BY u.created_at DESC, u.uid LIMIT ").push_bind(page_size).push(" OFFSET ").push_bind(offset);
    let items: Vec<AdminUser> = qb.build_query_as().fetch_all(&state.pool).await?;
    Ok(Json(Page { items, page, page_size, total }))
}

#[derive(Deserialize)]
struct TrustInput {
    level: i16,
}

async fn set_trust(
    State(state): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Path(uid): Path<Uuid>,
    Json(input): Json<TrustInput>,
) -> AppResult<Json<Value>> {
    if !(0..=3).contains(&input.level) {
        return Err(AppError::BadRequest("信任等级需在 0 到 3 之间".into()));
    }
    let mut tx = state.pool.begin().await?;
    let (old,): (i16,) = sqlx::query_as("SELECT trust_level FROM users WHERE uid = $1 FOR UPDATE")
        .bind(uid)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::not_found("用户"))?;
    sqlx::query("UPDATE users SET trust_level = $2 WHERE uid = $1").bind(uid).bind(input.level).execute(&mut *tx).await?;
    audit(&mut tx, &admin, "user.trust", "user", uid, format!("TL{old} -> TL{}", input.level)).await?;
    tx.commit().await?;
    Ok(Json(json!({ "uid": uid, "trust_level": input.level })))
}

#[derive(Deserialize)]
struct UserStatusInput {
    status: String,
    /// 仅 muted 有效；不传表示无限期
    muted_hours: Option<i64>,
}

async fn set_user_status(
    State(state): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Path(uid): Path<Uuid>,
    Json(input): Json<UserStatusInput>,
) -> AppResult<Json<Value>> {
    if !matches!(input.status.as_str(), "active" | "muted" | "banned") {
        return Err(AppError::BadRequest("status 只能是 active / muted / banned".into()));
    }
    if input.status != "muted" && input.muted_hours.is_some() {
        return Err(AppError::BadRequest("muted_hours 只用于禁言".into()));
    }
    if input.muted_hours.is_some_and(|h| !(1..=24 * 365 * 10).contains(&h)) {
        return Err(AppError::BadRequest("禁言时长需在 1 小时到 10 年之间".into()));
    }
    let mut tx = state.pool.begin().await?;
    let (old,): (String,) = sqlx::query_as("SELECT status FROM users WHERE uid = $1 FOR UPDATE")
        .bind(uid)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::not_found("用户"))?;
    let (muted_until,): (Option<DateTime<Utc>>,) = sqlx::query_as(
        "UPDATE users SET status = $2,
            muted_until = CASE WHEN $2 = 'muted' AND $3::bigint IS NOT NULL THEN now() + make_interval(hours => $3::int) ELSE NULL END
         WHERE uid = $1 RETURNING muted_until",
    )
    .bind(uid)
    .bind(&input.status)
    .bind(input.muted_hours)
    .fetch_one(&mut *tx)
    .await?;
    let detail = match input.muted_hours {
        Some(h) => format!("{old} -> {}（{h} 小时）", input.status),
        None => format!("{old} -> {}", input.status),
    };
    audit(&mut tx, &admin, "user.status", "user", uid, detail).await?;
    tx.commit().await?;
    Ok(Json(json!({ "uid": uid, "status": input.status, "muted_until": muted_until })))
}

// ───────────────────────── 分类 ─────────────────────────

#[derive(Serialize, sqlx::FromRow)]
struct AdminCategory {
    id: i32,
    parent_id: Option<i32>,
    slug: String,
    name: String,
    description: String,
    kind: String,
    sort_order: i32,
    min_post_level: i16,
    on_home: bool,
    award_points: bool,
    admin_only: bool,
    /// 各状态的帖子总数（不含已删除）
    topic_count: i64,
}

const CATEGORY_SELECT: &str = "SELECT c.id, c.parent_id, c.slug, c.name, c.description, c.kind, c.sort_order,
        c.min_post_level, c.on_home, c.award_points, c.admin_only,
        (SELECT COUNT(*) FROM topics t WHERE t.category_id = c.id AND t.status <> 'deleted') AS topic_count
    FROM categories c";

async fn list_categories(State(state): State<AppState>, _admin: AuthAdmin) -> AppResult<Json<Vec<AdminCategory>>> {
    let rows = sqlx::query_as(&format!("{CATEGORY_SELECT} ORDER BY c.sort_order, c.id")).fetch_all(&state.pool).await?;
    Ok(Json(rows))
}

fn d_true() -> bool {
    true
}
fn d_level() -> i16 {
    1
}

#[derive(Deserialize)]
struct CategoryInput {
    parent_id: Option<i32>,
    slug: String,
    name: String,
    #[serde(default)]
    description: String,
    kind: String,
    #[serde(default)]
    sort_order: i32,
    #[serde(default = "d_level")]
    min_post_level: i16,
    #[serde(default = "d_true")]
    on_home: bool,
    #[serde(default = "d_true")]
    award_points: bool,
    #[serde(default)]
    admin_only: bool,
}

fn validate_level(level: i16) -> AppResult<()> {
    if (0..=3).contains(&level) {
        Ok(())
    } else {
        Err(AppError::BadRequest("最低发帖等级需在 0 到 3 之间".into()))
    }
}

async fn create_category(
    State(state): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Json(input): Json<CategoryInput>,
) -> AppResult<Json<AdminCategory>> {
    let slug = input.slug.trim().to_string();
    validate_slug(&slug)?;
    let name = validate_name(&input.name, "分类名", 20)?;
    let description = validate_text(&input.description, "分类说明", 200)?;
    if !matches!(input.kind.as_str(), "deal" | "normal") {
        return Err(AppError::BadRequest("kind 只能是 deal 或 normal".into()));
    }
    validate_level(input.min_post_level)?;

    let mut tx = state.pool.begin().await?;
    if let Some(pid) = input.parent_id {
        let parent: Option<(Option<i32>, String)> =
            sqlx::query_as("SELECT parent_id, kind FROM categories WHERE id = $1").bind(pid).fetch_optional(&mut *tx).await?;
        let (grand, parent_kind) = parent.ok_or_else(|| AppError::BadRequest("父分类不存在".into()))?;
        if grand.is_some() {
            return Err(AppError::BadRequest("分类最多两级".into()));
        }
        if parent_kind != input.kind {
            return Err(AppError::BadRequest("子分类类型必须和父分类一致".into()));
        }
    }
    let inserted: Result<(i32,), sqlx::Error> = sqlx::query_as(
        "INSERT INTO categories (parent_id, slug, name, description, kind, sort_order, min_post_level, on_home, award_points, admin_only)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) RETURNING id",
    )
    .bind(input.parent_id)
    .bind(&slug)
    .bind(&name)
    .bind(&description)
    .bind(&input.kind)
    .bind(input.sort_order)
    .bind(input.min_post_level)
    .bind(input.on_home)
    .bind(input.award_points)
    .bind(input.admin_only)
    .fetch_one(&mut *tx)
    .await;
    let id = match inserted {
        Ok((id,)) => id,
        Err(e) if is_unique_violation(&e) => return Err(AppError::Conflict("slug 已被使用".into())),
        Err(e) => return Err(e.into()),
    };
    audit(&mut tx, &admin, "category.create", "category", id, format!("{slug} {name}")).await?;
    tx.commit().await?;
    let row = sqlx::query_as(&format!("{CATEGORY_SELECT} WHERE c.id = $1")).bind(id).fetch_one(&state.pool).await?;
    Ok(Json(row))
}

#[derive(Deserialize)]
struct CategoryPatch {
    name: Option<String>,
    description: Option<String>,
    sort_order: Option<i32>,
    min_post_level: Option<i16>,
    on_home: Option<bool>,
    award_points: Option<bool>,
    admin_only: Option<bool>,
}

async fn update_category(
    State(state): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Path(id): Path<i32>,
    Json(patch): Json<CategoryPatch>,
) -> AppResult<Json<AdminCategory>> {
    let name = patch.name.as_deref().map(|n| validate_name(n, "分类名", 20)).transpose()?;
    let description = patch.description.as_deref().map(|d| validate_text(d, "分类说明", 200)).transpose()?;
    if let Some(level) = patch.min_post_level {
        validate_level(level)?;
    }
    let mut tx = state.pool.begin().await?;
    let done = sqlx::query(
        "UPDATE categories SET
            name = COALESCE($2, name), description = COALESCE($3, description), sort_order = COALESCE($4, sort_order),
            min_post_level = COALESCE($5, min_post_level), on_home = COALESCE($6, on_home),
            award_points = COALESCE($7, award_points), admin_only = COALESCE($8, admin_only)
         WHERE id = $1",
    )
    .bind(id)
    .bind(&name)
    .bind(&description)
    .bind(patch.sort_order)
    .bind(patch.min_post_level)
    .bind(patch.on_home)
    .bind(patch.award_points)
    .bind(patch.admin_only)
    .execute(&mut *tx)
    .await?
    .rows_affected();
    if done == 0 {
        return Err(AppError::not_found("分类"));
    }
    let mut changed = json!({
        "name": name, "description": description.as_deref().map(|d| clip(d, 30)), "sort_order": patch.sort_order,
        "min_post_level": patch.min_post_level, "on_home": patch.on_home, "award_points": patch.award_points,
        "admin_only": patch.admin_only,
    });
    // 日志里只记这次真正改动的字段
    if let Some(map) = changed.as_object_mut() {
        map.retain(|_, v| !v.is_null());
    }
    audit(&mut tx, &admin, "category.update", "category", id, changed.to_string()).await?;
    tx.commit().await?;
    let row = sqlx::query_as(&format!("{CATEGORY_SELECT} WHERE c.id = $1")).bind(id).fetch_one(&state.pool).await?;
    Ok(Json(row))
}

// ───────────────────────── 产品库 ─────────────────────────

#[derive(Serialize, sqlx::FromRow)]
struct AdminProduct {
    id: i32,
    slug: String,
    name: String,
    vendor: String,
    website: String,
    description: String,
    created_at: DateTime<Utc>,
    deal_count: i64,
}

const PRODUCT_SELECT: &str = "SELECT p.id, p.slug, p.name, p.vendor, p.website, p.description, p.created_at,
        (SELECT COUNT(*) FROM deals d JOIN topics t ON t.id = d.topic_id WHERE d.product_id = p.id AND t.status <> 'deleted') AS deal_count
    FROM products p";

async fn list_products(State(state): State<AppState>, _admin: AuthAdmin) -> AppResult<Json<Vec<AdminProduct>>> {
    let rows = sqlx::query_as(&format!("{PRODUCT_SELECT} ORDER BY p.name, p.id")).fetch_all(&state.pool).await?;
    Ok(Json(rows))
}

#[derive(Deserialize)]
struct ProductInput {
    slug: String,
    name: String,
    #[serde(default)]
    vendor: String,
    #[serde(default)]
    website: String,
    #[serde(default)]
    description: String,
}

async fn create_product(
    State(state): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Json(input): Json<ProductInput>,
) -> AppResult<Json<AdminProduct>> {
    let slug = input.slug.trim().to_string();
    validate_slug(&slug)?;
    let name = validate_name(&input.name, "产品名", 40)?;
    let vendor = validate_text(&input.vendor, "厂商", 40)?;
    let website = validate_website(&input.website)?;
    let description = validate_text(&input.description, "产品说明", 500)?;

    let mut tx = state.pool.begin().await?;
    let inserted: Result<(i32,), sqlx::Error> = sqlx::query_as(
        "INSERT INTO products (slug, name, vendor, website, description) VALUES ($1, $2, $3, $4, $5) RETURNING id",
    )
    .bind(&slug)
    .bind(&name)
    .bind(&vendor)
    .bind(&website)
    .bind(&description)
    .fetch_one(&mut *tx)
    .await;
    let id = match inserted {
        Ok((id,)) => id,
        Err(e) if is_unique_violation(&e) => return Err(AppError::Conflict("slug 已被使用".into())),
        Err(e) => return Err(e.into()),
    };
    audit(&mut tx, &admin, "product.create", "product", id, format!("{slug} {name}")).await?;
    tx.commit().await?;
    let row = sqlx::query_as(&format!("{PRODUCT_SELECT} WHERE p.id = $1")).bind(id).fetch_one(&state.pool).await?;
    Ok(Json(row))
}

/// slug 会出现在产品页地址里，创建后不允许修改。
#[derive(Deserialize)]
struct ProductPatch {
    name: Option<String>,
    vendor: Option<String>,
    website: Option<String>,
    description: Option<String>,
}

async fn update_product(
    State(state): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Path(id): Path<i32>,
    Json(patch): Json<ProductPatch>,
) -> AppResult<Json<AdminProduct>> {
    let name = patch.name.as_deref().map(|n| validate_name(n, "产品名", 40)).transpose()?;
    let vendor = patch.vendor.as_deref().map(|v| validate_text(v, "厂商", 40)).transpose()?;
    let website = patch.website.as_deref().map(validate_website).transpose()?;
    let description = patch.description.as_deref().map(|d| validate_text(d, "产品说明", 500)).transpose()?;

    let mut tx = state.pool.begin().await?;
    let done = sqlx::query(
        "UPDATE products SET name = COALESCE($2, name), vendor = COALESCE($3, vendor),
            website = COALESCE($4, website), description = COALESCE($5, description)
         WHERE id = $1",
    )
    .bind(id)
    .bind(&name)
    .bind(&vendor)
    .bind(&website)
    .bind(&description)
    .execute(&mut *tx)
    .await?
    .rows_affected();
    if done == 0 {
        return Err(AppError::not_found("产品"));
    }
    let fields: Vec<&str> = [
        ("name", name.is_some()),
        ("vendor", vendor.is_some()),
        ("website", website.is_some()),
        ("description", description.is_some()),
    ]
    .into_iter()
    .filter_map(|(k, set)| set.then_some(k))
    .collect();
    audit(&mut tx, &admin, "product.update", "product", id, fields.join(",")).await?;
    tx.commit().await?;
    let row = sqlx::query_as(&format!("{PRODUCT_SELECT} WHERE p.id = $1")).bind(id).fetch_one(&state.pool).await?;
    Ok(Json(row))
}

// ───────────────────────── 操作日志 ─────────────────────────

#[derive(Deserialize)]
struct AuditQuery {
    limit: Option<i64>,
}

#[derive(Serialize, sqlx::FromRow)]
struct AuditRow {
    id: i64,
    admin: String,
    action: String,
    target_type: String,
    target_id: String,
    detail: String,
    created_at: DateTime<Utc>,
}

async fn list_audit(
    State(state): State<AppState>,
    _admin: AuthAdmin,
    Query(q): Query<AuditQuery>,
) -> AppResult<Json<Vec<AuditRow>>> {
    let limit = q.limit.unwrap_or(50).clamp(1, 200);
    let rows = sqlx::query_as(
        "SELECT id, admin, action, target_type, target_id, detail, created_at FROM admin_audit ORDER BY id DESC LIMIT $1",
    )
    .bind(limit)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_roundtrip_and_wrong_secret_rejected() {
        let id = Uuid::new_v4();
        let secret = b"secret-a-secret-a-secret-a-secret-a";
        let token = token::issue(secret, id, 3, 60).unwrap();
        let claims = token::verify(secret, &token).unwrap();
        assert_eq!((claims.sub, claims.tv), (id.to_string(), 3));
        assert!(token::verify(b"secret-b-secret-b-secret-b-secret-b", &token).is_err());
        // 已过期的令牌（超出 5 秒宽限）也要拒绝
        assert!(token::verify(secret, &token::issue(secret, id, 0, -60).unwrap()).is_err());
    }

    #[test]
    fn slugs_are_validated() {
        for ok in ["ab", "deals-llm-api", "a1", &"x".repeat(40)] {
            assert!(validate_slug(ok).is_ok(), "{ok}");
        }
        for bad in ["", "a", "Ab", "has space", "under_score", "中文", &"x".repeat(41)] {
            assert!(validate_slug(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn website_and_username_rules() {
        assert_eq!(validate_website("").unwrap(), "");
        assert!(validate_website("https://openai.com").is_ok());
        assert!(validate_website("ftp://x.com").is_err());
        assert!(validate_website("https://a.com/ b").is_err());
        assert_eq!(normalize_username(" Root.Admin ").unwrap(), "root.admin");
        assert!(normalize_username("ab").is_err());
    }

    #[test]
    fn like_pattern_escapes_wildcards() {
        assert_eq!(like_pattern("50%_off"), "%50\\%\\_off%");
    }
}
