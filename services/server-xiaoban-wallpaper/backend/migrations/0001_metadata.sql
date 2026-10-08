CREATE TABLE administrators (
    id SMALLINT PRIMARY KEY CHECK (id = 1),
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    initial_password BOOLEAN NOT NULL DEFAULT TRUE,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE TABLE admin_sessions (
    token_hash TEXT PRIMARY KEY,
    administrator_id SMALLINT NOT NULL REFERENCES administrators(id) ON DELETE CASCADE,
    csrf_token TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL
);
CREATE INDEX admin_sessions_expiry ON admin_sessions(expires_at);
CREATE TABLE wallpapers (
    id TEXT PRIMARY KEY,
    definition JSONB NOT NULL,
    published BOOLEAN NOT NULL DEFAULT FALSE,
    sort_order INTEGER NOT NULL DEFAULT 0,
    revision BIGINT NOT NULL DEFAULT 1,
    verified_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE TABLE application_release (
    id SMALLINT PRIMARY KEY CHECK (id = 1),
    draft JSONB NOT NULL DEFAULT '{"schemaVersion":1,"version":"0.1.0","notes":"","platforms":{}}',
    published JSONB,
    revision BIGINT NOT NULL DEFAULT 1
);
INSERT INTO application_release(id) VALUES (1);
