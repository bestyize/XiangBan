//! 内容包 zip 的本地检查：结构安全、清单与类型一致、纯数据类带 `payload.json`。
//! 论坛的资源机上传时已经查过一遍；这里不信任何一方，下载后在本机再查，才让内容到达确认窗口。

use serde_json::Value;
use std::io::{Cursor, Read};

const MAX_ENTRIES: usize = 2000;
const MAX_UNCOMPRESSED: u64 = 128 * 1024 * 1024;
const MAX_MANIFEST: u64 = 64 * 1024;
const MAX_PAYLOAD: u64 = 256 * 1024;
const MB: u64 = 1024 * 1024;

/// 纯数据类：包里只有 `manifest.json` 与 `payload.json`（可带说明文件），不含可执行内容。
const PURE_DATA: &[&str] = &["effect", "reminder", "routine", "task", "goal"];

/// 各类型压缩包大小上限，与论坛、资源机一致。
pub fn size_limit(kind: &str) -> Option<u64> {
    match kind {
        "effect" | "reminder" | "routine" | "task" | "goal" | "expert" => Some(MB),
        "widget" | "game" | "skill" | "workflow" => Some(8 * MB),
        "wallpaper" => Some(32 * MB),
        _ => None,
    }
}

pub fn is_pure_data(kind: &str) -> bool {
    PURE_DATA.contains(&kind)
}

#[derive(Debug)]
pub struct Inspected {
    pub manifest: Value,
    /// 纯数据类的载荷；其他类型为 `None`。
    pub payload: Option<Value>,
    pub files: Vec<String>,
}

fn safe_name(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with('/')
        && !name.contains('\\')
        && !name.contains('\0')
        && !name.split('/').any(|part| part == "..")
}

fn read_limited(entry: &mut zip::read::ZipFile<'_, Cursor<&[u8]>>, limit: u64, what: &str) -> Result<Vec<u8>, String> {
    let mut buf = Vec::new();
    // 以实际读到的为准，不信条目声明的大小。
    entry.by_ref().take(limit + 1).read_to_end(&mut buf).map_err(|_| "压缩包已损坏".to_string())?;
    if buf.len() as u64 > limit {
        return Err(format!("{what} 过大"));
    }
    Ok(buf)
}

pub fn inspect(bytes: &[u8], kind: &str) -> Result<Inspected, String> {
    let limit = size_limit(kind).ok_or("不支持的内容类型")?;
    if bytes.len() as u64 > limit {
        return Err(format!("压缩包超过该类型 {} MB 的上限", limit / MB));
    }
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|_| "不是有效的 zip 压缩包".to_string())?;
    if archive.len() > MAX_ENTRIES {
        return Err("压缩包里的文件太多".into());
    }
    let (mut total, mut manifest, mut payload, mut files) = (0u64, None, None, Vec::new());
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|_| "压缩包已损坏".to_string())?;
        let name = entry.name().map_err(|_| "压缩包里有无法识别的文件名".to_string())?.to_string();
        if !safe_name(&name) {
            return Err(format!("压缩包里有不安全的路径：{name}"));
        }
        if entry.is_symlink() {
            return Err(format!("压缩包里不允许符号链接：{name}"));
        }
        total = total.saturating_add(entry.size());
        if total > MAX_UNCOMPRESSED {
            return Err("压缩包解压后过大".into());
        }
        match name.as_str() {
            "manifest.json" => {
                let raw = read_limited(&mut entry, MAX_MANIFEST, "manifest.json")?;
                manifest = Some(serde_json::from_slice::<Value>(&raw).map_err(|_| "manifest.json 不是有效的 JSON".to_string())?);
            }
            "payload.json" => {
                let raw = read_limited(&mut entry, MAX_PAYLOAD, "payload.json")?;
                payload = Some(serde_json::from_slice::<Value>(&raw).map_err(|_| "payload.json 不是有效的 JSON".to_string())?);
            }
            _ => {}
        }
        if !entry.is_dir() {
            files.push(name);
        }
    }
    let manifest = manifest.ok_or("压缩包根目录缺少 manifest.json")?;
    let declared = manifest.get("kind").and_then(Value::as_str).ok_or("manifest 缺少 kind")?;
    if declared != kind {
        return Err(format!("包里声明的类型是 {declared}，与要应用的 {kind} 不一致"));
    }
    if manifest.get("schemaVersion").and_then(Value::as_i64).is_none_or(|v| v < 1) {
        return Err("manifest 缺少有效的 schemaVersion".into());
    }
    if is_pure_data(kind) {
        if payload.is_none() {
            return Err("这个类型的内容包需要 payload.json".into());
        }
        // 纯数据类不应夹带别的文件：只允许清单、载荷和说明文字。
        if let Some(extra) = files.iter().find(|f| !matches!(f.as_str(), "manifest.json" | "payload.json" | "README.md" | "preview.png")) {
            return Err(format!("纯数据内容包不能包含其他文件：{extra}"));
        }
    }
    Ok(Inspected { manifest, payload: if is_pure_data(kind) { payload } else { None }, files })
}

/// 提醒与任务模板载荷是 `{ "rules": [...] }`；这里只检查数量与类型归属，字段细节由桌面端已有的提醒校验负责。
/// `reminder` 只能含普通提醒与休息提醒；`task` 只能含小伴定时任务（agent），因为它会让助手代为执行。
pub fn rules_for(kind: &str, payload: &Value) -> Result<Vec<Value>, String> {
    let allowed: &[&str] = match kind {
        "reminder" => &["custom", "off-work", "rest"],
        "task" => &["agent"],
        _ => return Err("这个类型不含提醒规则".into()),
    };
    let rules = payload.get("rules").and_then(Value::as_array).ok_or("payload.json 需要 rules 数组")?;
    if rules.is_empty() || rules.len() > 10 {
        return Err("一个内容包包含 1～10 条规则".into());
    }
    for rule in rules {
        let rule_kind = rule.get("kind").and_then(Value::as_str).unwrap_or("custom");
        if !allowed.contains(&rule_kind) {
            return Err(format!("{kind} 内容包不能包含「{rule_kind}」类型的规则"));
        }
    }
    Ok(rules.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;

    pub fn zip_of(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut w = zip::ZipWriter::new(Cursor::new(Vec::new()));
        for (name, data) in entries {
            w.start_file(*name, zip::write::SimpleFileOptions::default()).unwrap();
            w.write_all(data).unwrap();
        }
        w.finish().unwrap().into_inner()
    }

    fn manifest(kind: &str) -> Vec<u8> {
        json!({"schemaVersion":1,"kind":kind,"id":"a","name":"A","version":"1"}).to_string().into_bytes()
    }

    #[test]
    fn accepts_a_sound_pure_data_pack() {
        let zip = zip_of(&[("manifest.json", &manifest("effect")), ("payload.json", br#"{"id":"a"}"#), ("README.md", b"hi")]);
        let found = inspect(&zip, "effect").unwrap();
        assert_eq!(found.payload.unwrap()["id"], "a");
        assert_eq!(found.manifest["kind"], "effect");
    }

    #[test]
    fn rejects_unsafe_or_inconsistent_packs() {
        let m = manifest("effect");
        assert!(inspect(&zip_of(&[("payload.json", b"{}")]), "effect").unwrap_err().contains("manifest.json"));
        assert!(inspect(&zip_of(&[("manifest.json", &m)]), "effect").unwrap_err().contains("payload.json"));
        assert!(inspect(&zip_of(&[("manifest.json", &m), ("payload.json", b"{}")]), "reminder").unwrap_err().contains("不一致"));
        assert!(inspect(&zip_of(&[("manifest.json", &m), ("payload.json", b"{}"), ("run.sh", b"x")]), "effect").unwrap_err().contains("其他文件"));
        assert!(inspect(&zip_of(&[("manifest.json", &m), ("payload.json", b"{}"), ("../x", b"x")]), "effect").unwrap_err().contains("不安全"));
        assert!(inspect(&zip_of(&[("manifest.json", b"nope"), ("payload.json", b"{}")]), "effect").unwrap_err().contains("JSON"));
        assert!(inspect(b"not a zip", "effect").is_err());
        assert!(inspect(&zip_of(&[]), "unknown").is_err());
        let huge = vec![b'a'; (MB + 1) as usize];
        assert!(inspect(&huge, "effect").unwrap_err().contains("上限"));
    }

    #[test]
    fn rules_follow_the_kind() {
        let reminders = json!({"rules":[{"kind":"custom","text":"喝水"},{"kind":"rest","text":"站起来"}]});
        assert_eq!(rules_for("reminder", &reminders).unwrap().len(), 2);
        assert!(rules_for("task", &reminders).is_err(), "任务模板只能含 agent 规则");
        let tasks = json!({"rules":[{"kind":"agent","text":"整理周报"}]});
        assert!(rules_for("task", &tasks).is_ok());
        assert!(rules_for("reminder", &tasks).is_err(), "提醒包不能夹带会代为执行的任务");
        assert!(rules_for("reminder", &json!({"rules":[]})).is_err());
        assert!(rules_for("reminder", &json!({})).is_err());
        assert!(rules_for("effect", &reminders).is_err());
    }
}
