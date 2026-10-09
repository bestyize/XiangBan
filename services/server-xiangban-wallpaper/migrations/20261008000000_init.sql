-- Wallpapers Table
CREATE TABLE IF NOT EXISTS wallpapers (
    id VARCHAR(128) PRIMARY KEY,
    provider VARCHAR(64) NOT NULL,
    source_id VARCHAR(128) NOT NULL,
    media_type VARCHAR(32) NOT NULL DEFAULT 'image',
    title VARCHAR(256) NOT NULL,
    description TEXT,
    tags TEXT[] NOT NULL DEFAULT '{}',
    width INT NOT NULL,
    height INT NOT NULL,
    aspect_ratio VARCHAR(32) NOT NULL,
    orientation VARCHAR(32) NOT NULL,
    dominant_color VARCHAR(32),
    preview_url TEXT NOT NULL,
    regular_url TEXT NOT NULL,
    downloads JSONB NOT NULL,
    video JSONB,
    author JSONB NOT NULL,
    stats JSONB NOT NULL,
    source_page_url TEXT NOT NULL,
    license TEXT NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'published',
    source_type VARCHAR(32) NOT NULL DEFAULT 'collected',
    category VARCHAR(64),
    local_path TEXT,
    collected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE wallpapers ADD COLUMN IF NOT EXISTS status VARCHAR(32) NOT NULL DEFAULT 'published';
ALTER TABLE wallpapers ADD COLUMN IF NOT EXISTS source_type VARCHAR(32) NOT NULL DEFAULT 'collected';
ALTER TABLE wallpapers ADD COLUMN IF NOT EXISTS category VARCHAR(64);
ALTER TABLE wallpapers ADD COLUMN IF NOT EXISTS local_path TEXT;
-- Our own stored copy of the media: {"media": {path, sha256, bytes, mime}, "cover": {...}}.
-- The provider's links stay in the URL columns.
ALTER TABLE wallpapers ADD COLUMN IF NOT EXISTS local_copy JSONB;

CREATE INDEX IF NOT EXISTS idx_wallpapers_status ON wallpapers(status);
CREATE INDEX IF NOT EXISTS idx_wallpapers_source_type ON wallpapers(source_type);
CREATE INDEX IF NOT EXISTS idx_wallpapers_category ON wallpapers(category);

CREATE INDEX IF NOT EXISTS idx_wallpapers_provider ON wallpapers(provider);
CREATE INDEX IF NOT EXISTS idx_wallpapers_media_type ON wallpapers(media_type);
CREATE INDEX IF NOT EXISTS idx_wallpapers_orientation ON wallpapers(orientation);
CREATE INDEX IF NOT EXISTS idx_wallpapers_collected_at ON wallpapers(collected_at DESC);
CREATE INDEX IF NOT EXISTS idx_wallpapers_tags ON wallpapers USING GIN(tags);

-- User Favorites / Bookmarks
CREATE TABLE IF NOT EXISTS favorites (
    id SERIAL PRIMARY KEY,
    wallpaper_id VARCHAR(128) NOT NULL REFERENCES wallpapers(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_favorite_wallpaper UNIQUE (wallpaper_id)
);

-- Background Crawler & Collector Sync Task Logs
CREATE TABLE IF NOT EXISTS sync_tasks (
    id SERIAL PRIMARY KEY,
    provider VARCHAR(64) NOT NULL,
    keyword VARCHAR(128) NOT NULL,
    collected_count INT NOT NULL DEFAULT 0,
    status VARCHAR(32) NOT NULL,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
