use crate::{
    auth::{self, Admin},
    error::{ApiError, Result},
    protocol::{self, Release, Wallpaper},
    AppState,
};
use axum::{
    extract::{ConnectInfo, DefaultBodyLimit, Path, Query, Request, State},
    http::{header, HeaderMap, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{any, get, post, put},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{types::Json as DbJson, PgConnection, Postgres, Row, Transaction};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::Instant,
};
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

#[derive(Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
struct Record {
    id: String,
    definition: DbJson<Wallpaper>,
    published: bool,
    sort_order: i32,
    revision: i64,
    verified_at: Option<DateTime<Utc>>,
    updated_at: DateTime<Utc>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Save {
    definition: Wallpaper,
    #[serde(default)]
    published: bool,
    #[serde(default)]
    sort_order: i32,
    revision: Option<i64>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Import {
    data: Value,
    #[serde(default)]
    overwrite: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Revision {
    revision: i64,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Publish {
    revision: i64,
    published: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Login {
    username: String,
    password: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Credentials {
    current_password: String,
    username: String,
    new_password: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ReleaseSave {
    definition: Release,
    revision: i64,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateQuery {
    current_version: Option<String>,
    target: Option<String>,
}

pub fn router(state: AppState) -> Router {
    let assets = state.config.frontend_dist.clone();
    Router::new()
        .route(
            "/api/health",
            get(|| async { Json(json!({"ok":true,"service":"xiaoban-wallpaper"})) }),
        )
        .route("/api/catalog", get(public_catalog))
        .route("/api/site", get(site))
        .route("/api/updates", get(updates))
        .route(
            "/api/admin/login",
            post(login).route_layer(middleware::from_fn_with_state(state.clone(), login_limit)),
        )
        .route("/api/admin/me", get(me))
        .route("/api/admin/logout", post(logout))
        .route("/api/admin/account", put(credentials))
        .route("/api/admin/wallpapers", get(list).post(create))
        .route("/api/admin/wallpapers/import", post(import))
        .route("/api/admin/wallpapers/{id}", put(update).delete(remove))
        .route("/api/admin/wallpapers/{id}/publish", post(publish))
        .route("/api/admin/wallpapers/{id}/verify", post(verify))
        .route("/api/admin/release", get(get_release).put(save_release))
        .route("/api/admin/release/publish", post(publish_release))
        .route(
            "/api/{*path}",
            any(|| async { (StatusCode::NOT_FOUND, Json(json!({"message":"接口不存在"}))) }),
        )
        .fallback_service(
            ServeDir::new(&assets).fallback(ServeFile::new(assets.join("index.html"))),
        )
        .layer(DefaultBodyLimit::max(2 * 1024 * 1024))
        .layer(middleware::from_fn(security_headers))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
async fn security_headers(req: Request, next: Next) -> Response {
    let api = req.uri().path().starts_with("/api/admin");
    let management_page = req.uri().path().starts_with("/admin");
    let mut response = next.run(req).await;
    let headers = response.headers_mut();
    headers.insert("x-content-type-options", "nosniff".parse().unwrap());
    headers.insert("referrer-policy", "no-referrer".parse().unwrap());
    headers.insert("content-security-policy","default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' https: data:; media-src https:; connect-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self'".parse().unwrap());
    if management_page {
        headers.insert("x-robots-tag", "noindex, nofollow".parse().unwrap());
    }
    if api || management_page {
        headers.insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
    }
    response
}
async fn login_limit(State(state): State<AppState>, req: Request, next: Next) -> Response {
    let ip = req
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|c| c.0.ip())
        .unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST));
    let allowed = {
        let mut limits = state.login_limits.lock().unwrap();
        limits.retain(|_, (t, _)| t.elapsed().as_secs() < 300);
        if limits.len() >= 10000 && !limits.contains_key(&ip) {
            false
        } else {
            let (start, count) = limits.entry(ip).or_insert((Instant::now(), 0));
            if start.elapsed().as_secs() > 60 {
                *start = Instant::now();
                *count = 0;
            }
            *count += 1;
            *count <= 10
        }
    };
    if !allowed {
        return ApiError(
            StatusCode::TOO_MANY_REQUESTS,
            "登录尝试过多，请稍后重试".into(),
        )
        .into_response();
    }
    next.run(req).await
}
async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<Login>,
) -> Result<Response> {
    auth::origin(&headers, &state)?;
    if body.username.len() > 64 || body.password.len() > 256 {
        return Err(ApiError::bad("登录信息长度不合法"));
    }
    let row = sqlx::query("SELECT username,password_hash FROM administrators WHERE id=1")
        .fetch_one(&state.pool)
        .await?;
    let valid = auth::verify_password(body.password, row.get("password_hash")).await;
    if !valid || body.username.trim() != row.get::<String, _>("username") {
        return Err(ApiError(
            StatusCode::UNAUTHORIZED,
            "用户名或密码不正确".into(),
        ));
    }
    let token = auth::random_token();
    let csrf = auth::random_token();
    sqlx::query("DELETE FROM admin_sessions WHERE expires_at<=NOW()")
        .execute(&state.pool)
        .await?;
    sqlx::query("INSERT INTO admin_sessions(token_hash,administrator_id,csrf_token,expires_at) VALUES($1,1,$2,NOW()+INTERVAL '8 hours')")
      .bind(auth::digest(&token)).bind(&csrf).execute(&state.pool).await?;
    Ok((
        [(
            header::SET_COOKIE,
            auth::cookie(&token, state.config.secure_cookie, false),
        )],
        Json(json!({"csrfToken":csrf,"username":body.username.trim()})),
    )
        .into_response())
}
async fn me(admin: Admin) -> Json<Value> {
    Json(
        json!({"username":admin.username,"csrfToken":admin.csrf,"initialPassword":admin.initial_password}),
    )
}
async fn logout(
    State(state): State<AppState>,
    admin: Admin,
    headers: HeaderMap,
) -> Result<Response> {
    auth::mutation(&headers, &admin, &state)?;
    sqlx::query("DELETE FROM admin_sessions WHERE token_hash=$1")
        .bind(admin.session_hash)
        .execute(&state.pool)
        .await?;
    Ok((
        [(
            header::SET_COOKIE,
            auth::cookie("", state.config.secure_cookie, true),
        )],
        Json(json!({"ok":true})),
    )
        .into_response())
}
async fn credentials(
    State(state): State<AppState>,
    admin: Admin,
    headers: HeaderMap,
    Json(body): Json<Credentials>,
) -> Result<Response> {
    auth::mutation(&headers, &admin, &state)?;
    let username = body.username.trim();
    if username.is_empty()
        || username.len() > 64
        || username.chars().any(char::is_control)
        || (!body.new_password.is_empty() && !(6..=128).contains(&body.new_password.len()))
        || body.current_password.len() > 256
    {
        return Err(ApiError::bad("用户名需为 1–64 字节，新密码需为 6–128 字节"));
    }
    let mut tx = state.pool.begin().await?;
    let hash = sqlx::query("SELECT password_hash FROM administrators WHERE id=1 FOR UPDATE")
        .fetch_one(&mut *tx)
        .await?
        .get("password_hash");
    if !auth::verify_password(body.current_password, hash).await {
        return Err(ApiError::bad("当前密码不正确"));
    }
    let hash = if body.new_password.is_empty() {
        None
    } else {
        Some(auth::hash_password(body.new_password).await?)
    };
    sqlx::query("UPDATE administrators SET username=$1,password_hash=COALESCE($2,password_hash),initial_password=CASE WHEN $2::text IS NULL THEN initial_password ELSE FALSE END,updated_at=NOW() WHERE id=1").bind(username).bind(hash).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM admin_sessions")
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok((
        [(
            header::SET_COOKIE,
            auth::cookie("", state.config.secure_cookie, true),
        )],
        Json(json!({"ok":true})),
    )
        .into_response())
}
async fn locked(state: &AppState) -> Result<Transaction<'_, Postgres>> {
    let mut tx = state.pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(550061)")
        .execute(&mut *tx)
        .await?;
    Ok(tx)
}
async fn catalog(conn: &mut PgConnection) -> Result<Value> {
    let items: Vec<DbJson<Wallpaper>> = sqlx::query_scalar(
        "SELECT definition FROM wallpapers WHERE published=TRUE ORDER BY sort_order,id",
    )
    .fetch_all(conn)
    .await?;
    if items.len() > 100 {
        return Err(ApiError::bad(
            "桌面订阅最多发布 100 张壁纸，请先下架部分资源",
        ));
    }
    let mut value = json!({"schemaVersion":1,"revision":"","items":items});
    value["revision"] = json!(auth::digest(
        &serde_json::to_string(&value["items"]).map_err(ApiError::internal)?
    ));
    if serde_json::to_vec(&value)
        .map_err(ApiError::internal)?
        .len()
        > protocol::MAX_CATALOG
    {
        return Err(ApiError::bad("公开壁纸清单超过 1 MiB"));
    }
    Ok(value)
}
async fn public_catalog(State(state): State<AppState>) -> Result<Json<Value>> {
    Ok(Json(catalog(&mut *state.pool.acquire().await?).await?))
}
async fn site(State(state): State<AppState>) -> Result<Json<Value>> {
    let c = catalog(&mut *state.pool.acquire().await?).await?;
    let release: Option<Value> =
        sqlx::query_scalar("SELECT published FROM application_release WHERE id=1")
            .fetch_one(&state.pool)
            .await?;
    Ok(Json(
        json!({"name":"小伴桌面","catalog":c,"release":release}),
    ))
}
async fn updates(
    State(state): State<AppState>,
    Query(query): Query<UpdateQuery>,
) -> Result<Response> {
    let current = semver::Version::parse(query.current_version.as_deref().unwrap_or("0.0.0"))
        .map_err(|_| ApiError::bad("currentVersion 格式不正确"))?;
    if query
        .target
        .as_deref()
        .is_some_and(|s| !["windows-x86_64", "darwin-aarch64", "darwin-x86_64"].contains(&s))
    {
        return Err(ApiError::bad("target 不支持"));
    }
    let release: Option<Value> =
        sqlx::query_scalar("SELECT published FROM application_release WHERE id=1")
            .fetch_one(&state.pool)
            .await?;
    if let Some(release) = release {
        if semver::Version::parse(release["version"].as_str().unwrap_or(""))
            .map_err(ApiError::internal)?
            > current
        {
            return Ok(Json(release).into_response());
        }
    }
    Ok(StatusCode::NO_CONTENT.into_response())
}
async fn list(State(state): State<AppState>, _: Admin) -> Result<Json<Value>> {
    let items = sqlx::query_as::<_, Record>(
        "SELECT * FROM wallpapers ORDER BY sort_order,created_at DESC,id",
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(json!({"items":items})))
}
async fn record(conn: &mut PgConnection, id: &str) -> Result<Record> {
    sqlx::query_as("SELECT * FROM wallpapers WHERE id=$1")
        .bind(id)
        .fetch_optional(conn)
        .await?
        .ok_or_else(|| ApiError(StatusCode::NOT_FOUND, "壁纸不存在".into()))
}
fn version_guard(old: &Record, new: &Wallpaper, published: bool) -> Result<()> {
    if old.published
        && published
        && (old.definition.resource.sha256 != new.resource.sha256
            || old.definition.assets != new.assets
            || old.definition.cover != new.cover)
        && semver::Version::parse(&new.version).unwrap()
            <= semver::Version::parse(&old.definition.version).unwrap()
    {
        return Err(ApiError::conflict("已发布资源内容改变时，请提高壁纸版本号"));
    }
    Ok(())
}
async fn create(
    State(state): State<AppState>,
    admin: Admin,
    headers: HeaderMap,
    Json(body): Json<Save>,
) -> Result<Json<Record>> {
    auth::mutation(&headers, &admin, &state)?;
    protocol::validate_wallpaper(
        &body.definition,
        body.published,
        state.config.allow_loopback_resources,
    )?;
    let mut tx = locked(&state).await?;
    let result=sqlx::query("INSERT INTO wallpapers(id,definition,published,sort_order) VALUES($1,$2,$3,$4) ON CONFLICT(id) DO NOTHING")
      .bind(&body.definition.id).bind(DbJson(&body.definition)).bind(body.published).bind(body.sort_order).execute(&mut *tx).await?;
    if result.rows_affected() == 0 {
        return Err(ApiError::conflict("这个 ID 已存在"));
    }
    catalog(&mut tx).await?;
    let result = record(&mut tx, &body.definition.id).await?;
    tx.commit().await?;
    Ok(Json(result))
}
async fn update(
    State(state): State<AppState>,
    admin: Admin,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<Save>,
) -> Result<Json<Record>> {
    auth::mutation(&headers, &admin, &state)?;
    if id != body.definition.id {
        return Err(ApiError::bad("不能改变现有壁纸 ID"));
    }
    protocol::validate_wallpaper(
        &body.definition,
        body.published,
        state.config.allow_loopback_resources,
    )?;
    let mut tx = locked(&state).await?;
    let old = record(&mut tx, &id).await?;
    if body.revision != Some(old.revision) {
        return Err(ApiError::conflict("壁纸已被更新，请刷新后再编辑"));
    }
    version_guard(&old, &body.definition, body.published)?;
    sqlx::query("UPDATE wallpapers SET definition=$1,published=$2,sort_order=$3,revision=revision+1,verified_at=NULL,updated_at=NOW() WHERE id=$4")
      .bind(DbJson(body.definition)).bind(body.published).bind(body.sort_order).bind(&id).execute(&mut *tx).await?;
    catalog(&mut tx).await?;
    let result = record(&mut tx, &id).await?;
    tx.commit().await?;
    Ok(Json(result))
}
async fn publish(
    State(state): State<AppState>,
    admin: Admin,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<Publish>,
) -> Result<Json<Record>> {
    auth::mutation(&headers, &admin, &state)?;
    let mut tx = locked(&state).await?;
    let old = record(&mut tx, &id).await?;
    if old.revision != body.revision {
        return Err(ApiError::conflict("壁纸已被更新，请刷新"));
    }
    protocol::validate_wallpaper(
        &old.definition,
        body.published,
        state.config.allow_loopback_resources,
    )?;
    sqlx::query(
        "UPDATE wallpapers SET published=$1,revision=revision+1,updated_at=NOW() WHERE id=$2",
    )
    .bind(body.published)
    .bind(&id)
    .execute(&mut *tx)
    .await?;
    catalog(&mut tx).await?;
    let result = record(&mut tx, &id).await?;
    tx.commit().await?;
    Ok(Json(result))
}
async fn remove(
    State(state): State<AppState>,
    admin: Admin,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<Revision>,
) -> Result<Json<Value>> {
    auth::mutation(&headers, &admin, &state)?;
    let mut tx = locked(&state).await?;
    let rows = sqlx::query("DELETE FROM wallpapers WHERE id=$1 AND revision=$2")
        .bind(&id)
        .bind(body.revision)
        .execute(&mut *tx)
        .await?
        .rows_affected();
    if rows == 0 {
        return Err(ApiError::conflict("壁纸不存在或已更新，请刷新"));
    }
    tx.commit().await?;
    Ok(Json(json!({"ok":true})))
}
async fn import(
    State(state): State<AppState>,
    admin: Admin,
    headers: HeaderMap,
    Json(body): Json<Import>,
) -> Result<Json<Value>> {
    auth::mutation(&headers, &admin, &state)?;
    let items = protocol::parse_import(body.data)?;
    for item in &items {
        protocol::validate_wallpaper(item, false, state.config.allow_loopback_resources)?;
    }
    let mut tx = locked(&state).await?;
    for item in &items {
        let exists = sqlx::query("SELECT id FROM wallpapers WHERE id=$1")
            .bind(&item.id)
            .fetch_optional(&mut *tx)
            .await?
            .is_some();
        if exists && !body.overwrite {
            return Err(ApiError::conflict(format!(
                "ID {} 已存在，未导入任何项目",
                item.id
            )));
        }
        sqlx::query("INSERT INTO wallpapers(id,definition) VALUES($1,$2) ON CONFLICT(id) DO UPDATE SET definition=EXCLUDED.definition,published=FALSE,revision=wallpapers.revision+1,verified_at=NULL,updated_at=NOW()")
          .bind(&item.id).bind(DbJson(item)).execute(&mut *tx).await?;
    }
    tx.commit().await?;
    Ok(Json(json!({"imported":items.len()})))
}
async fn verify(
    State(state): State<AppState>,
    admin: Admin,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<Revision>,
) -> Result<Json<Record>> {
    auth::mutation(&headers, &admin, &state)?;
    let old = record(&mut *state.pool.acquire().await?, &id).await?;
    if old.revision != body.revision {
        return Err(ApiError::conflict("壁纸已更新，请刷新"));
    }
    let allowed = state.config.allow_loopback_resources;
    let item = tokio::time::timeout(
        std::time::Duration::from_secs(120),
        probe_wallpaper(old.definition.0.clone(), allowed),
    )
    .await
    .map_err(|_| ApiError::bad("壁纸探测超过 120 秒，未保存任何媒体文件"))??;
    if old.published && item != old.definition.0 {
        return Err(ApiError::conflict(
            "外部资源已改变，请先下架并提高版本号后重新探测",
        ));
    }
    let mut tx = locked(&state).await?;
    let result=sqlx::query("UPDATE wallpapers SET definition=$1,verified_at=NOW(),revision=revision+1,updated_at=NOW() WHERE id=$2 AND revision=$3")
      .bind(DbJson(item)).bind(&id).bind(body.revision).execute(&mut *tx).await?;
    if result.rows_affected() == 0 {
        return Err(ApiError::conflict("探测期间壁纸已更新，请刷新"));
    }
    let result = record(&mut tx, &id).await?;
    tx.commit().await?;
    Ok(Json(result))
}
async fn probe_wallpaper(mut item: Wallpaper, allowed: bool) -> Result<Wallpaper> {
    item.resource = crate::probe::inspect(
        &item.resource,
        if item.kind == "canvas" {
            2 * 1024 * 1024
        } else {
            protocol::MAX_RESOURCE
        },
        allowed,
    )
    .await?;
    let mut total = item.resource.bytes;
    if let Some(cover) = &mut item.cover {
        *cover = crate::probe::inspect(cover, 2 * 1024 * 1024, allowed).await?;
    }
    for asset in item.assets.values_mut() {
        let limit =
            (protocol::MAX_RESOURCE - total).min(if asset.mime == "application/javascript" {
                2 * 1024 * 1024
            } else {
                protocol::MAX_RESOURCE
            });
        *asset = crate::probe::inspect(asset, limit, allowed).await?;
        total += asset.bytes;
    }
    protocol::validate_wallpaper(&item, true, allowed)?;
    Ok(item)
}
async fn get_release(State(state): State<AppState>, _: Admin) -> Result<Json<Value>> {
    let row = sqlx::query("SELECT draft,published,revision FROM application_release WHERE id=1")
        .fetch_one(&state.pool)
        .await?;
    Ok(Json(
        json!({"definition":row.get::<Value,_>("draft"),"published":row.get::<Option<Value>,_>("published"),"revision":row.get::<i64,_>("revision")}),
    ))
}
async fn save_release(
    State(state): State<AppState>,
    admin: Admin,
    headers: HeaderMap,
    Json(body): Json<ReleaseSave>,
) -> Result<Json<Value>> {
    auth::mutation(&headers, &admin, &state)?;
    protocol::validate_release(
        &body.definition,
        false,
        state.config.allow_loopback_resources,
    )?;
    let result = sqlx::query(
        "UPDATE application_release SET draft=$1,revision=revision+1 WHERE id=1 AND revision=$2",
    )
    .bind(DbJson(body.definition))
    .bind(body.revision)
    .execute(&state.pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(ApiError::conflict("发布信息已更新，请刷新"));
    }
    get_release(State(state), admin).await
}
async fn publish_release(
    State(state): State<AppState>,
    admin: Admin,
    headers: HeaderMap,
    Json(body): Json<Revision>,
) -> Result<Json<Value>> {
    auth::mutation(&headers, &admin, &state)?;
    let mut tx = state.pool.begin().await?;
    let row = sqlx::query("SELECT * FROM application_release WHERE id=1 FOR UPDATE")
        .fetch_one(&mut *tx)
        .await?;
    if row.get::<i64, _>("revision") != body.revision {
        return Err(ApiError::conflict("发布信息已更新，请刷新"));
    }
    let mut release: Release =
        serde_json::from_value(row.get("draft")).map_err(ApiError::internal)?;
    protocol::validate_release(&release, true, state.config.allow_loopback_resources)?;
    if let Some(previous) = row.get::<Option<Value>, _>("published") {
        let previous: Release = serde_json::from_value(previous).map_err(ApiError::internal)?;
        if release != previous
            && semver::Version::parse(&release.version).unwrap()
                <= semver::Version::parse(&previous.version).unwrap()
        {
            return Err(ApiError::conflict("新发布版本需要高于当前公开版本"));
        }
    }
    if release.published_at.is_none() {
        release.published_at = Some(Utc::now().to_rfc3339());
    }
    sqlx::query(
        "UPDATE application_release SET published=$1,draft=$1,revision=revision+1 WHERE id=1",
    )
    .bind(DbJson(release))
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    get_release(State(state), admin).await
}
