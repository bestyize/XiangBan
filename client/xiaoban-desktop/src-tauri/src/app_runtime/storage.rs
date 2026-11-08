//! V2 App Runtime persistent sandboxed storage.
//! Each application gets its own JSON store under `apps/storage/<id>.json`.
//! Enforces 10 MB maximum capacity, key format checks, and atomic writes via `.tmp + rename`.

use serde_json::{Map, Value};
use std::{
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
};

pub const MAX_STORAGE_BYTES: usize = 10 * 1024 * 1024;
pub const MAX_KEY_CHARS: usize = 128;

pub fn valid_app_id(id: &str) -> bool {
    let b = id.as_bytes();
    !b.is_empty()
        && b.len() <= 64
        && (b[0].is_ascii_lowercase() || b[0].is_ascii_digit())
        && b.iter().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == b'-')
}

pub fn validate_key(key: &str) -> Result<(), String> {
    let count = key.chars().count();
    if count == 0 || count > MAX_KEY_CHARS {
        return Err("invalid".into());
    }
    if key.contains('\0') {
        return Err("invalid".into());
    }
    Ok(())
}

fn storage_dir(root: &Path) -> PathBuf {
    root.join("apps").join("storage")
}

fn storage_path(root: &Path, app_id: &str) -> Result<PathBuf, String> {
    if !valid_app_id(app_id) {
        return Err("invalid_app_id".into());
    }
    Ok(storage_dir(root).join(format!("{app_id}.json")))
}

fn load_map_from_path(path: &Path) -> Result<Map<String, Value>, String> {
    if !path.exists() {
        return Ok(Map::new());
    }
    let content = fs::read_to_string(path).map_err(|e| format!("读取存储失败: {e}"))?;
    if content.trim().is_empty() {
        return Ok(Map::new());
    }
    let value: Value = serde_json::from_str(&content).map_err(|e| format!("解析存储失败: {e}"))?;
    match value {
        Value::Object(map) => Ok(map),
        _ => Err("存储文件格式损坏".into()),
    }
}

fn save_map_to_path(dir: &Path, path: &Path, map: &Map<String, Value>) -> Result<(), String> {
    let bytes = serde_json::to_vec(map).map_err(|e| format!("序列化失败: {e}"))?;
    if bytes.len() > MAX_STORAGE_BYTES {
        return Err("quota".into());
    }

    fs::create_dir_all(dir).map_err(|e| format!("创建存储目录失败: {e}"))?;

    let filename = path.file_stem().and_then(|s| s.to_str()).unwrap_or("app");
    let temp_path = dir.join(format!("{filename}-{}.tmp", std::process::id()));

    let write_res = (|| -> io::Result<()> {
        let mut file = File::create(&temp_path)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temp_path, path)
    })();

    if write_res.is_err() {
        let _ = fs::remove_file(&temp_path);
    }
    write_res.map_err(|e| format!("写入存储失败: {e}"))
}

pub fn get(root: &Path, app_id: &str, key: &str) -> Result<Option<Value>, String> {
    validate_key(key)?;
    let path = storage_path(root, app_id)?;
    let map = load_map_from_path(&path)?;
    Ok(map.get(key).cloned())
}

pub fn set(root: &Path, app_id: &str, key: &str, value: Value) -> Result<(), String> {
    validate_key(key)?;
    let dir = storage_dir(root);
    let path = storage_path(root, app_id)?;
    let mut map = load_map_from_path(&path)?;
    map.insert(key.to_string(), value);
    save_map_to_path(&dir, &path, &map)
}

pub fn remove(root: &Path, app_id: &str, key: &str) -> Result<(), String> {
    validate_key(key)?;
    let dir = storage_dir(root);
    let path = storage_path(root, app_id)?;
    if !path.exists() {
        return Ok(());
    }
    let mut map = load_map_from_path(&path)?;
    map.remove(key);
    save_map_to_path(&dir, &path, &map)
}

pub fn keys(root: &Path, app_id: &str) -> Result<Vec<String>, String> {
    let path = storage_path(root, app_id)?;
    let map = load_map_from_path(&path)?;
    Ok(map.keys().cloned().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_test_root(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("xiaoban-storage-test-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn test_valid_app_id() {
        assert!(valid_app_id("mbti"));
        assert!(valid_app_id("ink-blade"));
        assert!(valid_app_id("game-123"));
        assert!(!valid_app_id(""));
        assert!(!valid_app_id("-bad"));
        assert!(!valid_app_id("UPPERCASE"));
        assert!(!valid_app_id("has/slash"));
        assert!(!valid_app_id("has..dot"));
    }

    #[test]
    fn test_validate_key() {
        assert!(validate_key("score").is_ok());
        assert!(validate_key("a".repeat(128).as_str()).is_ok());
        assert_eq!(validate_key("").unwrap_err(), "invalid");
        assert_eq!(validate_key("a".repeat(129).as_str()).unwrap_err(), "invalid");
        assert_eq!(validate_key("key\0null").unwrap_err(), "invalid");
    }

    #[test]
    fn test_storage_crud_lifecycle() {
        let root = temp_test_root("crud");
        let app_id = "test-app";

        // Initial get returns None
        assert_eq!(get(&root, app_id, "score").unwrap(), None);
        assert_eq!(keys(&root, app_id).unwrap(), Vec::<String>::new());

        // Set
        set(&root, app_id, "score", serde_json::json!(100)).unwrap();
        set(&root, app_id, "name", serde_json::json!("Alice")).unwrap();

        // Get
        assert_eq!(get(&root, app_id, "score").unwrap(), Some(serde_json::json!(100)));
        assert_eq!(get(&root, app_id, "name").unwrap(), Some(serde_json::json!("Alice")));

        let mut k = keys(&root, app_id).unwrap();
        k.sort();
        assert_eq!(k, vec!["name", "score"]);

        // Remove
        remove(&root, app_id, "name").unwrap();
        assert_eq!(get(&root, app_id, "name").unwrap(), None);
        assert_eq!(keys(&root, app_id).unwrap(), vec!["score"]);

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn test_quota_exceeded() {
        let root = temp_test_root("quota");
        let app_id = "test-app";

        // Try to insert a payload exceeding MAX_STORAGE_BYTES
        let large_string = "x".repeat(MAX_STORAGE_BYTES + 1024);
        let err = set(&root, app_id, "large", Value::String(large_string)).unwrap_err();
        assert_eq!(err, "quota");

        let _ = fs::remove_dir_all(&root);
    }
}
