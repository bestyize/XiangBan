use rusqlite::Connection;
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub fn init_schema(conn: &Connection) -> rusqlite::Result<()> {
    // 启用 WAL 模式提高并发性能，并启用外键约束
    conn.execute_batch(
        "
        PRAGMA journal_mode = WAL;
        PRAGMA foreign_keys = ON;
        PRAGMA synchronous = NORMAL;
        ",
    )?;

    // 1. 用户表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            salt TEXT NOT NULL,
            is_admin INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL
        );",
        [],
    )?;

    // 2. 文件元数据表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS files (
            id TEXT PRIMARY KEY,
            user_id INTEGER NOT NULL,
            parent_id TEXT,
            name TEXT NOT NULL,
            size INTEGER NOT NULL DEFAULT 0,
            mime_type TEXT NOT NULL,
            is_dir INTEGER NOT NULL DEFAULT 0,
            storage_path TEXT,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE,
            FOREIGN KEY(parent_id) REFERENCES files(id) ON DELETE CASCADE
        );",
        [],
    )?;

    // 创建索引加速查询
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_files_user_parent ON files(user_id, parent_id);",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_files_name ON files(name);",
        [],
    )?;

    // 3. 分享表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS shares (
            code TEXT PRIMARY KEY,
            user_id INTEGER NOT NULL,
            file_id TEXT NOT NULL,
            password TEXT,
            note TEXT,
            expires_at INTEGER,
            view_count INTEGER NOT NULL DEFAULT 0,
            download_count INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL,
            FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE,
            FOREIGN KEY(file_id) REFERENCES files(id) ON DELETE CASCADE
        );",
        [],
    )?;

    // 兼容旧库：若 shares 表缺少 note 列（早期版本或 WIP 期间创建的库）则补齐
    let has_note: bool = conn
        .prepare("SELECT 1 FROM pragma_table_info('shares') WHERE name = 'note'")?
        .exists([])?;
    if !has_note {
        conn.execute("ALTER TABLE shares ADD COLUMN note TEXT;", [])?;
        tracing::info!("数据库迁移：已为 shares 表补充 note 列");
    }

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_shares_user ON shares(user_id);",
        [],
    )?;

    // 初始化默认管理员 admin / 123456
    let user_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM users WHERE username = 'admin'",
        [],
        |row| row.get(0),
    )?;

    if user_count == 0 {
        let salt = Uuid::new_v4().simple().to_string();
        let password_hash = hash_password_with_salt("123456", &salt);
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT INTO users (username, password_hash, salt, is_admin, created_at) VALUES (?1, ?2, ?3, 1, ?4)",
            (&"admin", &password_hash, &salt, now),
        )?;
        tracing::info!("系统初始化完成：默认管理员账号 admin，默认密码 123456 已创建");
    }

    Ok(())
}

pub fn hash_password_with_salt(password: &str, salt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(salt.as_bytes());
    hex::encode(hasher.finalize())
}
