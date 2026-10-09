use crate::{
    error::AppError,
    models::{
        query::{SearchQuery, SearchResult},
        wallpaper::{
            Author, DownloadUrls, LocalCopy, MediaType, Orientation, Resolution, Stats, VideoMeta,
        },
        WallpaperItem,
    },
};
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions, PgRow},
    PgPool, QueryBuilder, Row,
};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use tracing::{info, warn};

fn row_to_wallpaper(r: &PgRow) -> WallpaperItem {
    let media_type = match r.get::<String, _>("media_type").as_str() {
        "video" => MediaType::Video,
        "live_photo" => MediaType::LivePhoto,
        _ => MediaType::Image,
    };
    let orientation = match r.get::<String, _>("orientation").as_str() {
        "landscape" => Orientation::Landscape,
        "portrait" => Orientation::Portrait,
        "squarish" => Orientation::Squarish,
        _ => Orientation::All,
    };

    let downloads_val: serde_json::Value = r.get("downloads");
    let video_val: Option<serde_json::Value> = r.get("video");
    let author_val: serde_json::Value = r.get("author");
    let stats_val: serde_json::Value = r.get("stats");

    let downloads: DownloadUrls = serde_json::from_value(downloads_val).unwrap_or(DownloadUrls {
        original: r.get("regular_url"),
        large: None,
        medium: None,
        small: None,
        thumb: r.get("preview_url"),
    });

    let video: Option<VideoMeta> = video_val.and_then(|v| serde_json::from_value(v).ok());
    let author: Author = serde_json::from_value(author_val).unwrap_or(Author {
        name: "Unknown".into(),
        username: "unknown".into(),
        avatar_url: None,
        profile_url: None,
    });
    let stats: Stats = serde_json::from_value(stats_val).unwrap_or_default();

    WallpaperItem {
        id: r.get("id"),
        provider: r.get("provider"),
        source_id: r.get("source_id"),
        media_type,
        title: r.get("title"),
        description: r.get("description"),
        tags: r.get("tags"),
        resolution: Resolution {
            width: r.get::<i32, _>("width") as u32,
            height: r.get::<i32, _>("height") as u32,
            aspect_ratio: r.get("aspect_ratio"),
        },
        orientation,
        dominant_color: r.get("dominant_color"),
        preview_url: r.get("preview_url"),
        regular_url: r.get("regular_url"),
        downloads,
        video,
        author,
        stats,
        source_page_url: r.get("source_page_url"),
        license: r.get("license"),
        status: r.try_get("status").unwrap_or_else(|_| "published".to_string()),
        source_type: r.try_get("source_type").unwrap_or_else(|_| "collected".to_string()),
        category: r.try_get("category").ok(),
        local_path: r.try_get("local_path").ok(),
        local_copy: r
            .try_get::<Option<serde_json::Value>, _>("local_copy")
            .ok()
            .flatten()
            .and_then(|v| serde_json::from_value(v).ok()),
        collected_at: r.get("collected_at"),
    }
}

#[derive(Clone)]
pub struct Database {
    pool: Option<PgPool>,
    mem_wallpapers: Arc<RwLock<HashMap<String, WallpaperItem>>>,
    mem_favorites: Arc<RwLock<Vec<String>>>,
}

impl Database {
    pub async fn init(database_url: Option<&str>) -> Self {
        let mem_wallpapers = Arc::new(RwLock::new(HashMap::new()));
        let mem_favorites = Arc::new(RwLock::new(Vec::new()));

        let pool = if let Some(url) = database_url {
            // Log where we connect, never the credentials embedded in the URL.
            let target = url
                .parse::<PgConnectOptions>()
                .map(|o| format!("{}:{}/{}", o.get_host(), o.get_port(), o.get_database().unwrap_or("")))
                .unwrap_or_else(|_| "(unparseable DATABASE_URL)".to_string());
            info!("Attempting to connect to PostgreSQL at: {target}");
            match PgPoolOptions::new()
                .max_connections(10)
                .acquire_timeout(std::time::Duration::from_secs(5))
                .connect(url)
                .await
            {
                Ok(p) => {
                    info!("Successfully connected to PostgreSQL database!");
                    if let Err(e) = Self::run_migrations(&p).await {
                        warn!("PostgreSQL migration failed: {e}. Falling back to in-memory mode.");
                        None
                    } else {
                        Some(p)
                    }
                }
                Err(e) => {
                    warn!("Could not connect to PostgreSQL ({e}). Operating in in-memory cache mode.");
                    None
                }
            }
        } else {
            info!("DATABASE_URL not configured. Operating in in-memory storage mode.");
            None
        };

        Self {
            pool,
            mem_wallpapers,
            mem_favorites,
        }
    }

    async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
        let sql = include_str!("../../migrations/20261008000000_init.sql");
        sqlx::raw_sql(sql).execute(pool).await?;
        info!("Database schema migrations applied successfully.");
        Ok(())
    }

    pub fn is_postgres_connected(&self) -> bool {
        self.pool.is_some()
    }

    /// The shared connection pool, for the admin console and search cache.
    pub fn pg_pool(&self) -> Option<PgPool> {
        self.pool.clone()
    }

    pub async fn upsert_wallpapers(&self, items: &[WallpaperItem]) -> Result<usize, AppError> {
        if items.is_empty() {
            return Ok(0);
        }

        // Always update in-memory cache
        {
            let mut mem = self.mem_wallpapers.write().await;
            for item in items {
                let mut stored = item.clone();
                // Like the SQL below, an upsert without copy information keeps the known copy.
                if stored.local_copy.is_none() {
                    stored.local_copy = mem.get(&item.id).and_then(|old| old.local_copy.clone());
                }
                mem.insert(item.id.clone(), stored);
            }
        }

        if let Some(pool) = &self.pool {
            let mut saved = 0;
            for item in items {
                let media_type_str = match item.media_type {
                    MediaType::Image => "image",
                    MediaType::Video => "video",
                    MediaType::LivePhoto => "live_photo",
                };

                let orientation_str = match item.orientation {
                    Orientation::Landscape => "landscape",
                    Orientation::Portrait => "portrait",
                    Orientation::Squarish => "squarish",
                    Orientation::All => "all",
                };

                let downloads_json = serde_json::to_value(&item.downloads).unwrap_or_default();
                let video_json = item.video.as_ref().and_then(|v| serde_json::to_value(v).ok());
                let author_json = serde_json::to_value(&item.author).unwrap_or_default();
                let stats_json = serde_json::to_value(&item.stats).unwrap_or_default();
                let local_copy_json = item
                    .local_copy
                    .as_ref()
                    .filter(|c| !c.is_empty())
                    .and_then(|c| serde_json::to_value(c).ok());

                let res = sqlx::query(
                    r#"
                    INSERT INTO wallpapers (
                        id, provider, source_id, media_type, title, description, tags,
                        width, height, aspect_ratio, orientation, dominant_color,
                        preview_url, regular_url, downloads, video, author, stats,
                        source_page_url, license, status, source_type, category, local_path, local_copy, updated_at
                    ) VALUES (
                        $1, $2, $3, $4, $5, $6, $7,
                        $8, $9, $10, $11, $12,
                        $13, $14, $15, $16, $17, $18,
                        $19, $20, $21, $22, $23, $24, $25, NOW()
                    )
                    ON CONFLICT (id) DO UPDATE SET
                        title = EXCLUDED.title,
                        description = EXCLUDED.description,
                        tags = EXCLUDED.tags,
                        stats = EXCLUDED.stats,
                        dominant_color = EXCLUDED.dominant_color,
                        preview_url = EXCLUDED.preview_url,
                        regular_url = EXCLUDED.regular_url,
                        downloads = EXCLUDED.downloads,
                        video = EXCLUDED.video,
                        status = EXCLUDED.status,
                        category = COALESCE(EXCLUDED.category, wallpapers.category),
                        local_path = COALESCE(EXCLUDED.local_path, wallpapers.local_path),
                        local_copy = COALESCE(EXCLUDED.local_copy, wallpapers.local_copy),
                        updated_at = NOW()
                    "#
                )
                .bind(&item.id)
                .bind(&item.provider)
                .bind(&item.source_id)
                .bind(media_type_str)
                .bind(&item.title)
                .bind(&item.description)
                .bind(&item.tags)
                .bind(item.resolution.width as i32)
                .bind(item.resolution.height as i32)
                .bind(&item.resolution.aspect_ratio)
                .bind(orientation_str)
                .bind(&item.dominant_color)
                .bind(&item.preview_url)
                .bind(&item.regular_url)
                .bind(downloads_json)
                .bind(video_json)
                .bind(author_json)
                .bind(stats_json)
                .bind(&item.source_page_url)
                .bind(&item.license)
                .bind(&item.status)
                .bind(&item.source_type)
                .bind(&item.category)
                .bind(&item.local_path)
                .bind(local_copy_json)
                .execute(pool)
                .await;

                if res.is_ok() {
                    saved += 1;
                }
            }
            Ok(saved)
        } else {
            Ok(items.len())
        }
    }

    pub async fn get_wallpaper(&self, id: &str) -> Result<Option<WallpaperItem>, AppError> {
        // First check in-memory
        {
            let mem = self.mem_wallpapers.read().await;
            if let Some(item) = mem.get(id) {
                return Ok(Some(item.clone()));
            }
        }

        if let Some(pool) = &self.pool {
            let row = sqlx::query("SELECT * FROM wallpapers WHERE id = $1")
                .bind(id)
                .fetch_optional(pool)
                .await?;

            if let Some(r) = row {
                return Ok(Some(row_to_wallpaper(&r)));
            }
        }

        Ok(None)
    }

    /// Records our stored copy of an item's media without touching its other fields.
    pub async fn update_local_copy(&self, id: &str, copy: &LocalCopy) -> Result<(), AppError> {
        {
            let mut mem = self.mem_wallpapers.write().await;
            if let Some(item) = mem.get_mut(id) {
                item.local_copy = Some(copy.clone());
            }
        }

        if let Some(pool) = &self.pool {
            sqlx::query("UPDATE wallpapers SET local_copy = $1, updated_at = NOW() WHERE id = $2")
                .bind(serde_json::to_value(copy).unwrap_or_default())
                .bind(id)
                .execute(pool)
                .await?;
        }
        Ok(())
    }

    pub async fn add_favorite(&self, item: &WallpaperItem) -> Result<(), AppError> {
        // Save wallpaper first
        self.upsert_wallpapers(&[item.clone()]).await?;

        // In-memory
        {
            let mut favs = self.mem_favorites.write().await;
            if !favs.contains(&item.id) {
                favs.push(item.id.clone());
            }
        }

        if let Some(pool) = &self.pool {
            sqlx::query(
                "INSERT INTO favorites (wallpaper_id) VALUES ($1) ON CONFLICT DO NOTHING"
            )
            .bind(&item.id)
            .execute(pool)
            .await?;
        }

        Ok(())
    }

    pub async fn remove_favorite(&self, id: &str) -> Result<bool, AppError> {
        let mut removed = false;
        {
            let mut favs = self.mem_favorites.write().await;
            if let Some(pos) = favs.iter().position(|x| x == id) {
                favs.remove(pos);
                removed = true;
            }
        }

        if let Some(pool) = &self.pool {
            let res = sqlx::query("DELETE FROM favorites WHERE wallpaper_id = $1")
                .bind(id)
                .execute(pool)
                .await?;
            if res.rows_affected() > 0 {
                removed = true;
            }
        }

        Ok(removed)
    }

    pub async fn list_favorites(&self) -> Result<Vec<WallpaperItem>, AppError> {
        if let Some(pool) = &self.pool {
            let rows = sqlx::query(
                r#"
                SELECT w.* FROM wallpapers w
                INNER JOIN favorites f ON w.id = f.wallpaper_id
                ORDER BY f.created_at DESC
                "#
            )
            .fetch_all(pool)
            .await?;

            let results = rows.iter().map(row_to_wallpaper).collect();
            Ok(results)
        } else {
            let favs = self.mem_favorites.read().await;
            let mem = self.mem_wallpapers.read().await;
            let mut list = Vec::new();
            for id in favs.iter() {
                if let Some(item) = mem.get(id) {
                    list.push(item.clone());
                }
            }
            Ok(list)
        }
    }

    /// Search published wallpapers from the local database for the public front-end
    pub async fn search_published_wallpapers(&self, query: &SearchQuery) -> Result<SearchResult, AppError> {
        let started = std::time::Instant::now();
        let page = query.get_page();
        let per_page = query.get_per_page();
        let offset = ((page - 1) * per_page) as i64;
        let limit = per_page as i64;

        let req_media = query.get_media_type();
        let media_str = match req_media {
            MediaType::Image => "image",
            MediaType::Video => "video",
            MediaType::LivePhoto => "live_photo",
        };

        let wanted_orient = query.get_orientation();
        let orient_str = match wanted_orient {
            Orientation::Landscape => Some("landscape"),
            Orientation::Portrait => Some("portrait"),
            Orientation::Squarish => Some("squarish"),
            Orientation::All => None,
        };

        let q_term = query.get_q().to_string();

        if let Some(pool) = &self.pool {
            // Count total
            let mut count_qb = QueryBuilder::new("SELECT COUNT(*) FROM wallpapers WHERE status = 'published'");
            count_qb.push(" AND media_type = ");
            count_qb.push_bind(media_str);

            if let Some(orient) = orient_str {
                count_qb.push(" AND orientation = ");
                count_qb.push_bind(orient);
            }

            if let Some(cat) = query.category.as_deref().filter(|c| !c.is_empty()) {
                count_qb.push(" AND (category = ");
                count_qb.push_bind(cat);
                count_qb.push(" OR ");
                count_qb.push_bind(cat);
                count_qb.push(" = ANY(tags))");
            }

            if !q_term.is_empty() {
                let pattern = format!("%{q_term}%");
                count_qb.push(" AND (title ILIKE ");
                count_qb.push_bind(pattern);
                count_qb.push(" OR ");
                count_qb.push_bind(q_term.clone());
                count_qb.push(" = ANY(tags))");
            }

            let total: i64 = count_qb.build_query_scalar().fetch_one(pool).await.unwrap_or(0);

            // Select rows
            let mut qb = QueryBuilder::new("SELECT * FROM wallpapers WHERE status = 'published'");
            qb.push(" AND media_type = ");
            qb.push_bind(media_str);

            if let Some(orient) = orient_str {
                qb.push(" AND orientation = ");
                qb.push_bind(orient);
            }

            if let Some(cat) = query.category.as_deref().filter(|c| !c.is_empty()) {
                qb.push(" AND (category = ");
                qb.push_bind(cat);
                qb.push(" OR ");
                qb.push_bind(cat);
                qb.push(" = ANY(tags))");
            }

            if !q_term.is_empty() {
                let pattern = format!("%{q_term}%");
                qb.push(" AND (title ILIKE ");
                qb.push_bind(pattern);
                qb.push(" OR ");
                qb.push_bind(q_term.clone());
                qb.push(" = ANY(tags))");
            }

            qb.push(" ORDER BY collected_at DESC LIMIT ");
            qb.push_bind(limit);
            qb.push(" OFFSET ");
            qb.push_bind(offset);

            let rows = qb.build().fetch_all(pool).await?;
            let items: Vec<WallpaperItem> = rows.iter().map(row_to_wallpaper).collect();
            let has_more = (offset as usize + items.len()) < total as usize;

            Ok(SearchResult {
                total: total as usize,
                page,
                per_page,
                has_more,
                items,
                providers: vec!["local_db".to_string()],
                execution_time_ms: started.elapsed().as_millis() as u64,
            })
        } else {
            // In-memory fallback
            let mem = self.mem_wallpapers.read().await;
            let mut filtered: Vec<WallpaperItem> = mem
                .values()
                .filter(|w| w.status == "published")
                .filter(|w| w.media_type == req_media)
                .filter(|w| wanted_orient == Orientation::All || w.orientation == wanted_orient)
                .filter(|w| {
                    if let Some(cat) = query.category.as_deref().filter(|c| !c.is_empty()) {
                        w.category.as_deref() == Some(cat) || w.tags.iter().any(|t| t.eq_ignore_ascii_case(cat))
                    } else {
                        true
                    }
                })
                .filter(|w| {
                    if !q_term.is_empty() {
                        w.title.to_lowercase().contains(&q_term.to_lowercase())
                            || w.tags.iter().any(|t| t.to_lowercase().contains(&q_term.to_lowercase()))
                    } else {
                        true
                    }
                })
                .cloned()
                .collect();

            filtered.sort_by(|a, b| b.collected_at.cmp(&a.collected_at));
            let total = filtered.len();
            let start = ((page - 1) * per_page) as usize;
            let items: Vec<WallpaperItem> = filtered.into_iter().skip(start).take(per_page as usize).collect();
            let has_more = start + items.len() < total;

            Ok(SearchResult {
                total,
                page,
                per_page,
                has_more,
                items,
                providers: vec!["memory".to_string()],
                execution_time_ms: started.elapsed().as_millis() as u64,
            })
        }
    }

    /// List wallpapers with multiple filters for the admin console
    pub async fn admin_list_wallpapers(
        &self,
        status: Option<&str>,
        category: Option<&str>,
        media_type: Option<&str>,
        q: Option<&str>,
        page: u32,
        per_page: u32,
    ) -> Result<(Vec<WallpaperItem>, usize), AppError> {
        let offset = ((page.max(1) - 1) * per_page) as i64;
        let limit = per_page.clamp(1, 100) as i64;

        if let Some(pool) = &self.pool {
            let mut count_qb = QueryBuilder::new("SELECT COUNT(*) FROM wallpapers WHERE 1=1");
            if let Some(s) = status.filter(|s| !s.is_empty() && *s != "all") {
                count_qb.push(" AND status = ");
                count_qb.push_bind(s);
            }
            if let Some(c) = category.filter(|c| !c.is_empty() && *c != "all") {
                count_qb.push(" AND category = ");
                count_qb.push_bind(c);
            }
            if let Some(m) = media_type.filter(|m| !m.is_empty() && *m != "all") {
                count_qb.push(" AND media_type = ");
                count_qb.push_bind(m);
            }
            if let Some(term) = q.filter(|t| !t.trim().is_empty()) {
                let pattern = format!("%{}%", term.trim());
                count_qb.push(" AND (title ILIKE ");
                count_qb.push_bind(pattern);
                count_qb.push(" OR ");
                count_qb.push_bind(term.trim().to_string());
                count_qb.push(" = ANY(tags))");
            }
            let total: i64 = count_qb.build_query_scalar().fetch_one(pool).await.unwrap_or(0);

            let mut qb = QueryBuilder::new("SELECT * FROM wallpapers WHERE 1=1");
            if let Some(s) = status.filter(|s| !s.is_empty() && *s != "all") {
                qb.push(" AND status = ");
                qb.push_bind(s);
            }
            if let Some(c) = category.filter(|c| !c.is_empty() && *c != "all") {
                qb.push(" AND category = ");
                qb.push_bind(c);
            }
            if let Some(m) = media_type.filter(|m| !m.is_empty() && *m != "all") {
                qb.push(" AND media_type = ");
                qb.push_bind(m);
            }
            if let Some(term) = q.filter(|t| !t.trim().is_empty()) {
                let pattern = format!("%{}%", term.trim());
                qb.push(" AND (title ILIKE ");
                qb.push_bind(pattern);
                qb.push(" OR ");
                qb.push_bind(term.trim().to_string());
                qb.push(" = ANY(tags))");
            }

            qb.push(" ORDER BY collected_at DESC LIMIT ");
            qb.push_bind(limit);
            qb.push(" OFFSET ");
            qb.push_bind(offset);

            let rows = qb.build().fetch_all(pool).await?;
            let items: Vec<WallpaperItem> = rows.iter().map(row_to_wallpaper).collect();

            Ok((items, total as usize))
        } else {
            let mem = self.mem_wallpapers.read().await;
            let mut filtered: Vec<WallpaperItem> = mem
                .values()
                .filter(|w| status.map_or(true, |s| s == "all" || w.status == s))
                .filter(|w| category.map_or(true, |c| c == "all" || w.category.as_deref() == Some(c)))
                .filter(|w| {
                    media_type.map_or(true, |m| {
                        m == "all"
                            || match (m, w.media_type) {
                                ("image", MediaType::Image) => true,
                                ("video", MediaType::Video) => true,
                                ("live_photo", MediaType::LivePhoto) => true,
                                _ => false,
                            }
                    })
                })
                .filter(|w| {
                    q.map_or(true, |term| {
                        let t = term.trim().to_lowercase();
                        t.is_empty()
                            || w.title.to_lowercase().contains(&t)
                            || w.tags.iter().any(|tag| tag.to_lowercase().contains(&t))
                    })
                })
                .cloned()
                .collect();

            filtered.sort_by(|a, b| b.collected_at.cmp(&a.collected_at));
            let total = filtered.len();
            let start = offset as usize;
            let items = filtered.into_iter().skip(start).take(limit as usize).collect();
            Ok((items, total))
        }
    }

    /// Update wallpaper status (e.g. published, offline)
    pub async fn update_status(&self, id: &str, status: &str) -> Result<bool, AppError> {
        {
            let mut mem = self.mem_wallpapers.write().await;
            if let Some(item) = mem.get_mut(id) {
                item.status = status.to_string();
            }
        }

        if let Some(pool) = &self.pool {
            let res = sqlx::query("UPDATE wallpapers SET status = $1, updated_at = NOW() WHERE id = $2")
                .bind(status)
                .bind(id)
                .execute(pool)
                .await?;
            return Ok(res.rows_affected() > 0);
        }

        Ok(true)
    }

    /// Update wallpaper metadata (title, category, tags, status)
    pub async fn update_wallpaper_meta(
        &self,
        id: &str,
        title: &str,
        category: Option<&str>,
        tags: &[String],
        status: &str,
    ) -> Result<bool, AppError> {
        {
            let mut mem = self.mem_wallpapers.write().await;
            if let Some(item) = mem.get_mut(id) {
                item.title = title.to_string();
                item.category = category.map(|s| s.to_string());
                item.tags = tags.to_vec();
                item.status = status.to_string();
            }
        }

        if let Some(pool) = &self.pool {
            let res = sqlx::query(
                "UPDATE wallpapers SET title = $1, category = $2, tags = $3, status = $4, updated_at = NOW() WHERE id = $5"
            )
            .bind(title)
            .bind(category)
            .bind(tags)
            .bind(status)
            .bind(id)
            .execute(pool)
            .await?;
            return Ok(res.rows_affected() > 0);
        }

        Ok(true)
    }

    /// Delete a wallpaper permanently
    pub async fn delete_wallpaper(&self, id: &str) -> Result<bool, AppError> {
        {
            let mut mem = self.mem_wallpapers.write().await;
            mem.remove(id);
        }

        if let Some(pool) = &self.pool {
            let res = sqlx::query("DELETE FROM wallpapers WHERE id = $1")
                .bind(id)
                .execute(pool)
                .await?;
            return Ok(res.rows_affected() > 0);
        }

        Ok(true)
    }

    /// Check which IDs already exist in the database (for collector status badge)
    pub async fn check_collected_ids(&self, ids: &[String]) -> Result<Vec<String>, AppError> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        if let Some(pool) = &self.pool {
            let rows = sqlx::query("SELECT id FROM wallpapers WHERE id = ANY($1)")
                .bind(ids)
                .fetch_all(pool)
                .await?;

            let existing: Vec<String> = rows.into_iter().map(|r| r.get::<String, _>("id")).collect();
            Ok(existing)
        } else {
            let mem = self.mem_wallpapers.read().await;
            let existing: Vec<String> = ids.iter().filter(|id| mem.contains_key(*id)).cloned().collect();
            Ok(existing)
        }
    }

    pub async fn record_sync_task(
        &self,
        provider: &str,
        keyword: &str,
        count: i32,
        status: &str,
        err: Option<&str>,
    ) -> Result<(), AppError> {
        if let Some(pool) = &self.pool {
            sqlx::query(
                "INSERT INTO sync_tasks (provider, keyword, collected_count, status, error_message) VALUES ($1, $2, $3, $4, $5)"
            )
            .bind(provider)
            .bind(keyword)
            .bind(count)
            .bind(status)
            .bind(err)
            .execute(pool)
            .await?;
        }
        Ok(())
    }
}
