use crate::{
    auth::{generate_token, hash_password, verify_password, AuthAdmin},
    config::AppConfig,
    error::AppError,
    models::{
        AdminStats, AppItem, AppQuery, Category, ChangeAdminCredentialsRequest, CreateAppRequest,
        LoginRequest, LoginResponse, StatusUpdateRequest, UpdateAppRequest,
    },
};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde_json::json;
use sqlx::{PgPool, Row};

pub async fn admin_login(
    State(pool): State<PgPool>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    let row = sqlx::query("SELECT password_hash FROM appstore_admins WHERE username = $1")
        .bind(&req.username)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::Unauthorized("用户名或密码错误".to_string()))?;

    let hash: String = row.get("password_hash");
    if !verify_password(&req.password, &hash) {
        return Err(AppError::Unauthorized("用户名或密码错误".to_string()));
    }

    let config = AppConfig::from_env();
    let token = generate_token(&req.username, &config.jwt_secret)?;

    Ok(Json(LoginResponse {
        token,
        username: req.username,
    }))
}

pub async fn get_me(
    AuthAdmin(claims): AuthAdmin,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(json!({
        "username": claims.sub,
        "role": "admin",
    })))
}

pub async fn change_admin_credentials(
    AuthAdmin(claims): AuthAdmin,
    State(pool): State<PgPool>,
    Json(req): Json<ChangeAdminCredentialsRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    // 1. 查找当前管理员记录
    let row = sqlx::query("SELECT id, username, password_hash FROM appstore_admins WHERE username = $1")
        .bind(&claims.sub)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::Unauthorized("管理员账号不存在".to_string()))?;

    let admin_id: i32 = row.get("id");
    let current_hash: String = row.get("password_hash");

    // 2. 验证当前原密码
    if !verify_password(&req.current_password, &current_hash) {
        return Err(AppError::BadRequest("当前原密码输入错误，校验失败".to_string()));
    }

    let mut target_username = claims.sub.clone();

    // 3. 修改用户名 (如提供)
    if let Some(new_user) = req.new_username {
        let clean_user = new_user.trim().to_string();
        if !clean_user.is_empty() && clean_user != claims.sub {
            if clean_user.len() < 3 {
                return Err(AppError::BadRequest("新账号名称长度至少 3 个字符".to_string()));
            }

            let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM appstore_admins WHERE username = $1)")
                .bind(&clean_user)
                .fetch_one(&pool)
                .await?;

            if exists {
                return Err(AppError::BadRequest("该账号名称已被占用，请换一个".to_string()));
            }

            sqlx::query("UPDATE appstore_admins SET username = $1 WHERE id = $2")
                .bind(&clean_user)
                .bind(admin_id)
                .execute(&pool)
                .await?;

            target_username = clean_user;
        }
    }

    // 4. 修改新密码 (如提供)
    if let Some(new_pass) = req.new_password {
        let clean_pass = new_pass.trim().to_string();
        if !clean_pass.is_empty() {
            if clean_pass.len() < 6 {
                return Err(AppError::BadRequest("新密码长度至少 6 个字符".to_string()));
            }

            let new_hash = hash_password(&clean_pass)?;
            sqlx::query("UPDATE appstore_admins SET password_hash = $1 WHERE id = $2")
                .bind(new_hash)
                .bind(admin_id)
                .execute(&pool)
                .await?;
        }
    }

    // 5. 签发新的 JWT 凭证
    let config = AppConfig::from_env();
    let new_token = generate_token(&target_username, &config.jwt_secret)?;

    Ok(Json(json!({
        "success": true,
        "message": "管理员账号和密码修改成功",
        "username": target_username,
        "token": new_token,
    })))
}

pub async fn admin_get_stats(
    AuthAdmin(_): AuthAdmin,
    State(pool): State<PgPool>,
) -> Result<Json<AdminStats>, AppError> {
    let row = sqlx::query(
        r#"
        SELECT
            COUNT(*)::bigint as total_apps,
            COUNT(CASE WHEN status = 'published' THEN 1 END)::bigint as published_apps,
            COUNT(CASE WHEN status = 'unlisted' THEN 1 END)::bigint as unlisted_apps,
            COALESCE(SUM(downloads_count), 0)::bigint as total_downloads
        FROM appstore_apps
        "#,
    )
    .fetch_one(&pool)
    .await?;

    let cat_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM appstore_categories")
        .fetch_one(&pool)
        .await?;

    let total_apps: i64 = row.get("total_apps");
    let published_apps: i64 = row.get("published_apps");
    let unlisted_apps: i64 = row.get("unlisted_apps");
    let total_downloads: i64 = row.get("total_downloads");

    Ok(Json(AdminStats {
        total_apps,
        published_apps,
        unlisted_apps,
        total_categories: cat_count,
        total_downloads,
    }))
}

pub async fn admin_list_apps(
    AuthAdmin(_): AuthAdmin,
    State(pool): State<PgPool>,
    Query(query): Query<AppQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut sql = String::from(
        r#"
        SELECT
            a.id, a.slug, a.name, a.icon_url, a.category_id,
            c.name as category_name, c.slug as category_slug,
            a.summary, a.comment, a.referrer, a.referrer_avatar,
            a.pricing_type, a.platforms, a.badges, a.status,
            a.is_featured, a.sort_order, a.download_url, a.download_alt_url,
            a.official_url, a.appstore_url, a.github_url, a.github_stars,
            a.brew_cask, a.winget_id, a.mac_download_url, a.win_download_url,
            a.linux_download_url, a.file_size, a.release_notes, a.auto_sync_source, a.last_synced_at,
            a.pros, a.cons, a.alternatives,
            a.version, a.downloads_count, a.created_at, a.updated_at
        FROM appstore_apps a
        JOIN appstore_categories c ON a.category_id = c.id
        WHERE 1=1
        "#,
    );

    if let Some(status) = &query.status {
        if !status.is_empty() && status != "all" {
            sql.push_str(&format!(" AND a.status = '{status}'"));
        }
    }

    if let Some(cat) = &query.category {
        if !cat.is_empty() && cat != "all" {
            sql.push_str(&format!(" AND (c.slug = '{cat}' OR c.name = '{cat}')"));
        }
    }

    if let Some(s) = &query.search {
        let clean = s.replace('\'', "''");
        if !clean.is_empty() {
            sql.push_str(&format!(
                " AND (a.name ILIKE '%{clean}%' OR a.slug ILIKE '%{clean}%' OR a.summary ILIKE '%{clean}%')"
            ));
        }
    }

    sql.push_str(" ORDER BY a.sort_order DESC, a.updated_at DESC");

    let apps: Vec<AppItem> = sqlx::query_as(&sql).fetch_all(&pool).await?;

    Ok(Json(json!({
        "apps": apps,
        "total": apps.len(),
    })))
}

pub async fn admin_create_app(
    AuthAdmin(_): AuthAdmin,
    State(pool): State<PgPool>,
    Json(req): Json<CreateAppRequest>,
) -> Result<Json<AppItem>, AppError> {
    let referrer = req.referrer.unwrap_or_else(|| "小伴精选".to_string());
    let status = req.status.unwrap_or_else(|| "published".to_string());
    let is_featured = req.is_featured.unwrap_or(false);
    let sort_order = req.sort_order.unwrap_or(0);
    let github_stars = req.github_stars.unwrap_or(0);
    let pros = req.pros.unwrap_or_else(|| json!([]));
    let cons = req.cons.unwrap_or_else(|| json!([]));
    let alternatives = req.alternatives.unwrap_or_else(|| json!([]));

    let row = sqlx::query_as::<_, AppItem>(
        r#"
        INSERT INTO appstore_apps (
            slug, name, icon_url, category_id, summary, comment, referrer, referrer_avatar,
            pricing_type, platforms, badges, status, is_featured, sort_order,
            download_url, download_alt_url, official_url, appstore_url, github_url, github_stars,
            brew_cask, winget_id, mac_download_url, win_download_url, linux_download_url,
            file_size, release_notes, auto_sync_source, pros, cons, alternatives, version
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8,
            $9, $10, $11, $12, $13, $14,
            $15, $16, $17, $18, $19, $20,
            $21, $22, $23, $24, $25,
            $26, $27, $28, $29, $30, $31, $32
        )
        RETURNING
            id, slug, name, icon_url, category_id,
            NULL::varchar as category_name, NULL::varchar as category_slug,
            summary, comment, referrer, referrer_avatar,
            pricing_type, platforms, badges, status,
            is_featured, sort_order, download_url, download_alt_url,
            official_url, appstore_url, github_url, github_stars,
            brew_cask, winget_id, mac_download_url, win_download_url, linux_download_url,
            file_size, release_notes, auto_sync_source, last_synced_at,
            pros, cons, alternatives,
            version, downloads_count, created_at, updated_at
        "#,
    )
    .bind(req.slug)
    .bind(req.name)
    .bind(req.icon_url)
    .bind(req.category_id)
    .bind(req.summary)
    .bind(req.comment)
    .bind(referrer)
    .bind(req.referrer_avatar)
    .bind(req.pricing_type)
    .bind(req.platforms)
    .bind(req.badges)
    .bind(status)
    .bind(is_featured)
    .bind(sort_order)
    .bind(req.download_url)
    .bind(req.download_alt_url)
    .bind(req.official_url)
    .bind(req.appstore_url)
    .bind(req.github_url)
    .bind(github_stars)
    .bind(req.brew_cask)
    .bind(req.winget_id)
    .bind(req.mac_download_url)
    .bind(req.win_download_url)
    .bind(req.linux_download_url)
    .bind(req.file_size)
    .bind(req.release_notes)
    .bind(req.auto_sync_source)
    .bind(pros)
    .bind(cons)
    .bind(alternatives)
    .bind(req.version)
    .fetch_one(&pool)
    .await?;

    Ok(Json(row))
}

pub async fn admin_update_app(
    AuthAdmin(_): AuthAdmin,
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateAppRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _existing = sqlx::query("SELECT id FROM appstore_apps WHERE id = $1")
        .bind(id)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("应用 ID {id} 不存在")))?;

    if let Some(name) = req.name {
        sqlx::query("UPDATE appstore_apps SET name = $1, updated_at = NOW() WHERE id = $2")
            .bind(name).bind(id).execute(&pool).await?;
    }
    if let Some(slug) = req.slug {
        sqlx::query("UPDATE appstore_apps SET slug = $1, updated_at = NOW() WHERE id = $2")
            .bind(slug).bind(id).execute(&pool).await?;
    }
    if let Some(icon) = req.icon_url {
        sqlx::query("UPDATE appstore_apps SET icon_url = $1, updated_at = NOW() WHERE id = $2")
            .bind(icon).bind(id).execute(&pool).await?;
    }
    if let Some(cat_id) = req.category_id {
        sqlx::query("UPDATE appstore_apps SET category_id = $1, updated_at = NOW() WHERE id = $2")
            .bind(cat_id).bind(id).execute(&pool).await?;
    }
    if let Some(summary) = req.summary {
        sqlx::query("UPDATE appstore_apps SET summary = $1, updated_at = NOW() WHERE id = $2")
            .bind(summary).bind(id).execute(&pool).await?;
    }
    if let Some(comment) = req.comment {
        sqlx::query("UPDATE appstore_apps SET comment = $1, updated_at = NOW() WHERE id = $2")
            .bind(comment).bind(id).execute(&pool).await?;
    }
    if let Some(ref_name) = req.referrer {
        sqlx::query("UPDATE appstore_apps SET referrer = $1, updated_at = NOW() WHERE id = $2")
            .bind(ref_name).bind(id).execute(&pool).await?;
    }
    if let Some(pricing) = req.pricing_type {
        sqlx::query("UPDATE appstore_apps SET pricing_type = $1, updated_at = NOW() WHERE id = $2")
            .bind(pricing).bind(id).execute(&pool).await?;
    }
    if let Some(platforms) = req.platforms {
        sqlx::query("UPDATE appstore_apps SET platforms = $1, updated_at = NOW() WHERE id = $2")
            .bind(platforms).bind(id).execute(&pool).await?;
    }
    if let Some(badges) = req.badges {
        sqlx::query("UPDATE appstore_apps SET badges = $1, updated_at = NOW() WHERE id = $2")
            .bind(badges).bind(id).execute(&pool).await?;
    }
    if let Some(status) = req.status {
        sqlx::query("UPDATE appstore_apps SET status = $1, updated_at = NOW() WHERE id = $2")
            .bind(status).bind(id).execute(&pool).await?;
    }
    if let Some(is_feat) = req.is_featured {
        sqlx::query("UPDATE appstore_apps SET is_featured = $1, updated_at = NOW() WHERE id = $2")
            .bind(is_feat).bind(id).execute(&pool).await?;
    }
    if let Some(sort) = req.sort_order {
        sqlx::query("UPDATE appstore_apps SET sort_order = $1, updated_at = NOW() WHERE id = $2")
            .bind(sort).bind(id).execute(&pool).await?;
    }
    if let Some(dl) = req.download_url {
        sqlx::query("UPDATE appstore_apps SET download_url = $1, updated_at = NOW() WHERE id = $2")
            .bind(dl).bind(id).execute(&pool).await?;
    }
    if let Some(official) = req.official_url {
        sqlx::query("UPDATE appstore_apps SET official_url = $1, updated_at = NOW() WHERE id = $2")
            .bind(official).bind(id).execute(&pool).await?;
    }
    if let Some(github) = req.github_url {
        sqlx::query("UPDATE appstore_apps SET github_url = $1, updated_at = NOW() WHERE id = $2")
            .bind(github).bind(id).execute(&pool).await?;
    }
    if let Some(stars) = req.github_stars {
        sqlx::query("UPDATE appstore_apps SET github_stars = $1, updated_at = NOW() WHERE id = $2")
            .bind(stars).bind(id).execute(&pool).await?;
    }
    if let Some(brew) = req.brew_cask {
        sqlx::query("UPDATE appstore_apps SET brew_cask = $1, updated_at = NOW() WHERE id = $2")
            .bind(brew).bind(id).execute(&pool).await?;
    }
    if let Some(avatar) = req.referrer_avatar {
        sqlx::query("UPDATE appstore_apps SET referrer_avatar = $1, updated_at = NOW() WHERE id = $2")
            .bind(avatar).bind(id).execute(&pool).await?;
    }
    if let Some(dl_alt) = req.download_alt_url {
        sqlx::query("UPDATE appstore_apps SET download_alt_url = $1, updated_at = NOW() WHERE id = $2")
            .bind(dl_alt).bind(id).execute(&pool).await?;
    }
    if let Some(appstore) = req.appstore_url {
        sqlx::query("UPDATE appstore_apps SET appstore_url = $1, updated_at = NOW() WHERE id = $2")
            .bind(appstore).bind(id).execute(&pool).await?;
    }
    if let Some(winget) = req.winget_id {
        sqlx::query("UPDATE appstore_apps SET winget_id = $1, updated_at = NOW() WHERE id = $2")
            .bind(winget).bind(id).execute(&pool).await?;
    }
    if let Some(ver) = req.version {
        sqlx::query("UPDATE appstore_apps SET version = $1, updated_at = NOW() WHERE id = $2")
            .bind(ver).bind(id).execute(&pool).await?;
    }
    if let Some(mac_dl) = req.mac_download_url {
        sqlx::query("UPDATE appstore_apps SET mac_download_url = $1, updated_at = NOW() WHERE id = $2")
            .bind(mac_dl).bind(id).execute(&pool).await?;
    }
    if let Some(win_dl) = req.win_download_url {
        sqlx::query("UPDATE appstore_apps SET win_download_url = $1, updated_at = NOW() WHERE id = $2")
            .bind(win_dl).bind(id).execute(&pool).await?;
    }
    if let Some(linux_dl) = req.linux_download_url {
        sqlx::query("UPDATE appstore_apps SET linux_download_url = $1, updated_at = NOW() WHERE id = $2")
            .bind(linux_dl).bind(id).execute(&pool).await?;
    }
    if let Some(size) = req.file_size {
        sqlx::query("UPDATE appstore_apps SET file_size = $1, updated_at = NOW() WHERE id = $2")
            .bind(size).bind(id).execute(&pool).await?;
    }
    if let Some(notes) = req.release_notes {
        sqlx::query("UPDATE appstore_apps SET release_notes = $1, updated_at = NOW() WHERE id = $2")
            .bind(notes).bind(id).execute(&pool).await?;
    }
    if let Some(source) = req.auto_sync_source {
        sqlx::query("UPDATE appstore_apps SET auto_sync_source = $1, updated_at = NOW() WHERE id = $2")
            .bind(source).bind(id).execute(&pool).await?;
    }
    if let Some(pros) = req.pros {
        sqlx::query("UPDATE appstore_apps SET pros = $1, updated_at = NOW() WHERE id = $2")
            .bind(pros).bind(id).execute(&pool).await?;
    }
    if let Some(cons) = req.cons {
        sqlx::query("UPDATE appstore_apps SET cons = $1, updated_at = NOW() WHERE id = $2")
            .bind(cons).bind(id).execute(&pool).await?;
    }
    if let Some(alt) = req.alternatives {
        sqlx::query("UPDATE appstore_apps SET alternatives = $1, updated_at = NOW() WHERE id = $2")
            .bind(alt).bind(id).execute(&pool).await?;
    }

    Ok(Json(json!({ "success": true, "id": id })))
}

pub async fn admin_delete_app(
    AuthAdmin(_): AuthAdmin,
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("DELETE FROM appstore_apps WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await?;

    Ok(Json(json!({ "success": true, "id": id })))
}

// 核心功能：一键上架 / 下架切换
pub async fn admin_update_status(
    AuthAdmin(_): AuthAdmin,
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
    Json(req): Json<StatusUpdateRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    if req.status != "published" && req.status != "unlisted" && req.status != "draft" {
        return Err(AppError::BadRequest("无效的状态值，必须为 published / unlisted / draft".to_string()));
    }

    let rows_affected = sqlx::query(
        "UPDATE appstore_apps SET status = $1, updated_at = NOW() WHERE id = $2",
    )
    .bind(&req.status)
    .bind(id)
    .execute(&pool)
    .await?
    .rows_affected();

    if rows_affected == 0 {
        return Err(AppError::NotFound(format!("应用 ID {id} 不存在")));
    }

    Ok(Json(json!({
        "success": true,
        "id": id,
        "status": req.status,
    })))
}

// 核心功能：单个应用一键采集最新版本和直接下载包链接
pub async fn sync_app_version(
    AuthAdmin(_): AuthAdmin,
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>, AppError> {
    let app = sqlx::query_as::<_, AppItem>(
        r#"
        SELECT
            a.id, a.slug, a.name, a.icon_url, a.category_id,
            c.name as category_name, c.slug as category_slug,
            a.summary, a.comment, a.referrer, a.referrer_avatar,
            a.pricing_type, a.platforms, a.badges, a.status,
            a.is_featured, a.sort_order, a.download_url, a.download_alt_url,
            a.official_url, a.appstore_url, a.github_url, a.github_stars,
            a.brew_cask, a.winget_id, a.mac_download_url, a.win_download_url,
            a.linux_download_url, a.file_size, a.release_notes, a.auto_sync_source, a.last_synced_at,
            a.pros, a.cons, a.alternatives,
            a.version, a.downloads_count, a.created_at, a.updated_at
        FROM appstore_apps a
        JOIN appstore_categories c ON a.category_id = c.id
        WHERE a.id = $1
        "#
    )
    .bind(id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("应用 ID {id} 不存在")))?;

    if let Some(collected) = crate::services::collector::collect_app_version(
        &app.slug,
        app.brew_cask.as_deref(),
        app.github_url.as_deref(),
    ).await {
        if let Some(v) = &collected.version {
            sqlx::query("UPDATE appstore_apps SET version = $1 WHERE id = $2").bind(v).bind(id).execute(&pool).await?;
        }
        if let Some(m) = &collected.mac_download_url {
            sqlx::query("UPDATE appstore_apps SET mac_download_url = $1 WHERE id = $2").bind(m).bind(id).execute(&pool).await?;
        }
        if let Some(w) = &collected.win_download_url {
            sqlx::query("UPDATE appstore_apps SET win_download_url = $1 WHERE id = $2").bind(w).bind(id).execute(&pool).await?;
        }
        if let Some(l) = &collected.linux_download_url {
            sqlx::query("UPDATE appstore_apps SET linux_download_url = $1 WHERE id = $2").bind(l).bind(id).execute(&pool).await?;
        }
        if let Some(s) = &collected.file_size {
            sqlx::query("UPDATE appstore_apps SET file_size = $1 WHERE id = $2").bind(s).bind(id).execute(&pool).await?;
        }
        if let Some(r) = &collected.release_notes {
            sqlx::query("UPDATE appstore_apps SET release_notes = $1 WHERE id = $2").bind(r).bind(id).execute(&pool).await?;
        }
        sqlx::query("UPDATE appstore_apps SET last_synced_at = NOW(), updated_at = NOW() WHERE id = $1").bind(id).execute(&pool).await?;

        Ok(Json(json!({
            "success": true,
            "version": collected.version,
            "mac_download_url": collected.mac_download_url,
            "win_download_url": collected.win_download_url,
            "linux_download_url": collected.linux_download_url,
            "file_size": collected.file_size,
        })))
    } else {
        Err(AppError::BadRequest("未能从配置的源采集到直接下载地址，请手动配置直链".to_string()))
    }
}

// 核心功能：批量全量更新所有软件的最新版本与安装包
pub async fn sync_all_versions(
    AuthAdmin(_): AuthAdmin,
    State(pool): State<PgPool>,
) -> Result<Json<serde_json::Value>, AppError> {
    let apps = sqlx::query_as::<_, AppItem>(
        r#"
        SELECT
            a.id, a.slug, a.name, a.icon_url, a.category_id,
            c.name as category_name, c.slug as category_slug,
            a.summary, a.comment, a.referrer, a.referrer_avatar,
            a.pricing_type, a.platforms, a.badges, a.status,
            a.is_featured, a.sort_order, a.download_url, a.download_alt_url,
            a.official_url, a.appstore_url, a.github_url, a.github_stars,
            a.brew_cask, a.winget_id, a.mac_download_url, a.win_download_url,
            a.linux_download_url, a.file_size, a.release_notes, a.auto_sync_source, a.last_synced_at,
            a.pros, a.cons, a.alternatives,
            a.version, a.downloads_count, a.created_at, a.updated_at
        FROM appstore_apps a
        JOIN appstore_categories c ON a.category_id = c.id
        "#
    )
    .fetch_all(&pool)
    .await?;

    let mut updated_count = 0;
    for app in &apps {
        if let Some(collected) = crate::services::collector::collect_app_version(
            &app.slug,
            app.brew_cask.as_deref(),
            app.github_url.as_deref(),
        ).await {
            if let Some(v) = &collected.version {
                let _ = sqlx::query("UPDATE appstore_apps SET version = $1 WHERE id = $2").bind(v).bind(app.id).execute(&pool).await;
            }
            if let Some(m) = &collected.mac_download_url {
                let _ = sqlx::query("UPDATE appstore_apps SET mac_download_url = $1 WHERE id = $2").bind(m).bind(app.id).execute(&pool).await;
            }
            if let Some(w) = &collected.win_download_url {
                let _ = sqlx::query("UPDATE appstore_apps SET win_download_url = $1 WHERE id = $2").bind(w).bind(app.id).execute(&pool).await;
            }
            if let Some(l) = &collected.linux_download_url {
                let _ = sqlx::query("UPDATE appstore_apps SET linux_download_url = $1 WHERE id = $2").bind(l).bind(app.id).execute(&pool).await;
            }
            if let Some(s) = &collected.file_size {
                let _ = sqlx::query("UPDATE appstore_apps SET file_size = $1 WHERE id = $2").bind(s).bind(app.id).execute(&pool).await;
            }
            if let Some(r) = &collected.release_notes {
                let _ = sqlx::query("UPDATE appstore_apps SET release_notes = $1 WHERE id = $2").bind(r).bind(app.id).execute(&pool).await;
            }
            let _ = sqlx::query("UPDATE appstore_apps SET last_synced_at = NOW(), updated_at = NOW() WHERE id = $1").bind(app.id).execute(&pool).await;
            updated_count += 1;
        }
    }

    Ok(Json(json!({
        "success": true,
        "total": apps.len(),
        "updated": updated_count,
    })))
}

pub async fn admin_list_categories(
    AuthAdmin(_): AuthAdmin,
    State(pool): State<PgPool>,
) -> Result<Json<Vec<Category>>, AppError> {
    let categories = sqlx::query_as::<_, Category>(
        r#"
        SELECT
            c.id, c.slug, c.name, c.icon, c.description, c.sort_order,
            COUNT(a.id) as app_count
        FROM appstore_categories c
        LEFT JOIN appstore_apps a ON c.id = a.category_id
        GROUP BY c.id
        ORDER BY c.sort_order DESC, c.id ASC
        "#,
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(categories))
}
