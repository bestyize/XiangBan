//! 社区内容层的纯逻辑：内容包类型与体积上限、信任分级、清单检查、签名地址与上传凭证。
//!
//! 这里不碰数据库和网络，方便单元测试。大文件放在独立的资源机上，本服务只签发地址与凭证：
//! 下载走带过期时间的签名地址，上传由客户端用凭证直传资源机，主机不经手文件内容。

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use ring::hmac;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const MB: u64 = 1024 * 1024;

/// 内容类型：名称、体积上限、是否纯数据（不含可运行的代码）。
const KINDS: &[(&str, u64, bool)] = &[
    ("effect", MB, true),
    ("reminder", MB, true),
    ("routine", MB, true),
    ("task", MB, true),
    ("goal", MB, true),
    ("widget", 8 * MB, false),
    ("game", 100 * MB, false),
    ("app", 100 * MB, false),
    ("wallpaper", 32 * MB, false),
    ("skill", 8 * MB, false),
    ("expert", MB, false),
    ("workflow", 8 * MB, false),
];

/// 用户（T2）发布后无需审核就可以一键应用的类型：只有纯数据且不会代替用户执行动作的那几种。
const T2_ONE_CLICK: &[&str] = &["effect", "reminder"];

/// 封面图：不是内容包，走同一套直传，只收小图。
pub const COVER: &str = "cover";
pub const COVER_LIMIT: u64 = 2 * MB;

pub fn kind_limit(kind: &str) -> Option<u64> {
    KINDS.iter().find(|(k, _, _)| *k == kind).map(|(_, limit, _)| *limit)
}

pub fn is_pure_data(kind: &str) -> bool {
    KINDS.iter().any(|(k, _, pure)| *k == kind && *pure)
}

pub fn is_known_kind(kind: &str) -> bool {
    kind_limit(kind).is_some()
}

/// 上传凭证可用的类型：内容包类型或封面。
pub fn upload_limit(kind: &str) -> Option<u64> {
    if kind == COVER { Some(COVER_LIMIT) } else { kind_limit(kind) }
}

/// 信任分级：0 官方，1 已审核，2 用户未审。
pub const T0: i16 = 0;
pub const T2: i16 = 2;

/// 能不能显示“一键应用”：必须已发布；官方与已审核的都可以；用户未审的只开放纯数据类中的少数几种。
pub fn one_click_allowed(kind: &str, trust: i16, status: &str) -> bool {
    status == "published" && (trust <= 1 || T2_ONE_CLICK.contains(&kind))
}

pub fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// 内容包 id：小写字母、数字、连字符、下划线，1～64 位，不带品牌词的中性标识由发布者自己取。
pub fn valid_slug(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_')
}

/// 对象键只允许相对路径里的安全字符，防止签名地址被拼出目录穿越。
pub fn valid_file_key(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 200
        && !value.starts_with('/')
        && !value.contains("..")
        && value.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'/' | b'-' | b'_' | b'.'))
}

pub struct ManifestInfo {
    pub slug: String,
    pub version_label: String,
    /// 运行时协议版本：清单没有 `runtimeProtocol` 时是 1。
    pub runtime_protocol: i16,
}

/// 运行时协议 2 的能力名单（见 docs/architecture/app-runtime/README.md 第 7.2 节）。
const CAPABILITIES: &[&str] = &[
    "haptics", "share", "clipboard", "screen", "notify", "profile", "camera", "microphone", "contacts", "files", "windows", "pointer",
];
/// 只有桌面专用作品（`layer == "desktop"`）才能声明的能力。
const DESKTOP_ONLY_CAPABILITIES: &[&str] = &["windows", "pointer"];
const GAME_CATEGORIES: &[&str] = &["action", "puzzle", "board", "casual", "story"];
const APP_CATEGORIES: &[&str] = &["tool", "quiz"];
const INPUTS: &[&str] = &["touch", "mouse", "keyboard"];
const ORIENTATIONS: &[&str] = &["portrait", "landscape", "any"];

/// 检查资源包清单（`manifest.json`）：结构、类型与内容包类型一致、id 与版本号合法。
pub fn validate_manifest(manifest: &Value, kind: &str) -> Result<ManifestInfo, String> {
    let obj = manifest.as_object().ok_or("manifest 必须是对象")?;
    let protocol = match obj.get("runtimeProtocol") {
        None => 1,
        Some(v) => match v.as_i64() {
            Some(1) => 1,
            Some(2) => 2,
            _ => return Err("runtimeProtocol 只能是 1 或 2".into()),
        },
    };
    if protocol == 2 {
        return validate_manifest_v2(obj, kind);
    }
    let schema = obj.get("schemaVersion").and_then(Value::as_i64).ok_or("缺少 schemaVersion")?;
    if schema < 1 {
        return Err("schemaVersion 必须大于等于 1".into());
    }
    let declared = obj.get("kind").and_then(Value::as_str).ok_or("缺少 kind")?;
    if declared != kind {
        return Err(format!("manifest 的 kind 是 {declared}，与所选类型 {kind} 不一致"));
    }
    let slug = obj.get("id").and_then(Value::as_str).ok_or("缺少 id")?;
    if !valid_slug(slug) {
        return Err("id 只能用小写字母、数字、连字符和下划线，最长 64 位".into());
    }
    let name = obj.get("name").and_then(Value::as_str).unwrap_or("").trim();
    if name.is_empty() || name.chars().count() > 60 {
        return Err("name 必填，最长 60 个字".into());
    }
    let version = obj.get("version").and_then(Value::as_str).unwrap_or("").trim();
    if version.is_empty() || version.len() > 32 || !version.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-') {
        return Err("version 必填，只能含字母、数字、点和连字符".into());
    }
    Ok(ManifestInfo { slug: slug.to_string(), version_label: version.to_string(), runtime_protocol: 1 })
}

fn nonempty_str<'a>(obj: &'a serde_json::Map<String, Value>, field: &str) -> Result<&'a str, String> {
    match obj.get(field).and_then(Value::as_str).map(str::trim) {
        Some(v) if !v.is_empty() => Ok(v),
        _ => Err(format!("{field} 必填，且必须是非空文本")),
    }
}

/// 取文本列表字段并逐项检查是否在允许范围内；字段不存在时返回 None。
fn str_list<'a>(obj: &'a serde_json::Map<String, Value>, field: &str, allowed: &[&str]) -> Result<Option<Vec<&'a str>>, String> {
    let Some(raw) = obj.get(field) else { return Ok(None) };
    let items = raw.as_array().ok_or_else(|| format!("{field} 必须是文本列表"))?;
    let mut out = Vec::new();
    for item in items {
        let item = item.as_str().ok_or_else(|| format!("{field} 必须是文本列表"))?;
        if !allowed.contains(&item) {
            return Err(format!("{field} 里的 {item} 不是可用的取值"));
        }
        out.push(item);
    }
    Ok(Some(out))
}

/// 运行时协议 2 的清单规则（规范第 3.3 节）。
fn validate_manifest_v2(obj: &serde_json::Map<String, Value>, kind: &str) -> Result<ManifestInfo, String> {
    if obj.get("schemaVersion").and_then(Value::as_i64) != Some(3) {
        return Err("协议 2 的 schemaVersion 必须是 3".into());
    }
    if kind != "game" && kind != "app" {
        return Err("协议 2 只用于 game 和 app".into());
    }
    let declared = obj.get("kind").and_then(Value::as_str).ok_or("缺少 kind")?;
    if declared != kind {
        return Err(format!("manifest 的 kind 是 {declared}，与所选类型 {kind} 不一致"));
    }
    let slug = obj.get("id").and_then(Value::as_str).ok_or("缺少 id")?;
    if !valid_slug(slug) {
        return Err("id 只能用小写字母、数字、连字符和下划线，最长 64 位".into());
    }
    let name = obj.get("name").and_then(Value::as_str).unwrap_or("").trim();
    if name.is_empty() || name.chars().count() > 24 {
        return Err("name 必填，最长 24 个字".into());
    }
    let description = obj.get("description").and_then(Value::as_str).unwrap_or("").trim();
    if description.is_empty() || description.chars().count() > 80 {
        return Err("description 必填，最长 80 个字".into());
    }
    let version = obj.get("version").and_then(Value::as_str).unwrap_or("").trim();
    if version.is_empty() || version.len() > 32 || !version.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-') {
        return Err("version 必填，只能含字母、数字、点和连字符".into());
    }
    let categories = if kind == "game" { GAME_CATEGORIES } else { APP_CATEGORIES };
    let category = obj.get("category").and_then(Value::as_str).unwrap_or("");
    if !categories.contains(&category) {
        return Err(format!("category 必填，{kind} 只能取 {}", categories.join("、")));
    }
    if let Some(v) = obj.get("orientation") {
        if !v.as_str().is_some_and(|o| ORIENTATIONS.contains(&o)) {
            return Err("orientation 只能是 portrait、landscape 或 any".into());
        }
    }
    match str_list(obj, "input", INPUTS)? {
        Some(list) if !list.is_empty() => {}
        _ => return Err("input 必填，至少写 touch、mouse、keyboard 之一".into()),
    }
    nonempty_str(obj, "icon")?;
    nonempty_str(obj, "cover")?;
    if obj.contains_key("entry") {
        nonempty_str(obj, "entry")?;
    }
    let desktop = match obj.get("layer") {
        None => false,
        Some(v) if v.as_str() == Some("desktop") => true,
        Some(_) => return Err("layer 只能写 desktop，其余作品省略".into()),
    };
    if let Some(caps) = str_list(obj, "capabilities", CAPABILITIES)? {
        if !desktop {
            if let Some(c) = caps.iter().find(|c| DESKTOP_ONLY_CAPABILITIES.contains(c)) {
                return Err(format!("{c} 只有桌面专用作品（layer 为 desktop）才能声明"));
            }
        }
    }
    Ok(ManifestInfo { slug: slug.to_string(), version_label: version.to_string(), runtime_protocol: 2 })
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn sign(secret: &[u8], message: &str) -> String {
    let key = hmac::Key::new(hmac::HMAC_SHA256, secret);
    hex(hmac::sign(&key, message.as_bytes()).as_ref())
}

fn verify(secret: &[u8], message: &str, signature: &str) -> bool {
    verify_bytes(secret, message.as_bytes(), signature)
}

/// 资源机下载地址：`{base}/{file_key}?exp=&sig=`，签名覆盖 `GET\n{file_key}\n{exp}`。资源机用同一密钥校验。
pub fn signed_download_url(base: &str, file_key: &str, exp: i64, secret: &[u8]) -> String {
    let sig = sign(secret, &format!("GET\n{file_key}\n{exp}"));
    format!("{}/{file_key}?exp={exp}&sig={sig}", base.trim_end_matches('/'))
}

/// 校验资源机收到的下载请求（资源机一侧的逻辑，本服务测试用它保证两端约定一致）。
#[cfg_attr(not(test), allow(dead_code))]
pub fn verify_download(file_key: &str, exp: i64, sig: &str, now: i64, secret: &[u8]) -> bool {
    exp >= now && verify(secret, &format!("GET\n{file_key}\n{exp}"), sig)
}

/// 上传凭证里的声明：谁、传什么类型、最大多少、预期哈希、存到哪个键、何时过期。
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct UploadClaims {
    pub uid: String,
    pub kind: String,
    pub size: u64,
    pub sha256: String,
    pub file_key: String,
    pub exp: i64,
}

pub fn issue_upload_ticket(claims: &UploadClaims, secret: &[u8]) -> String {
    let body = URL_SAFE_NO_PAD.encode(serde_json::to_vec(claims).unwrap_or_default());
    let sig = sign(secret, &format!("PUT\n{body}"));
    format!("{body}.{sig}")
}

#[cfg_attr(not(test), allow(dead_code))]
pub fn verify_upload_ticket(ticket: &str, now: i64, secret: &[u8]) -> Result<UploadClaims, &'static str> {
    let (body, sig) = ticket.split_once('.').ok_or("上传凭证格式无效")?;
    if !verify(secret, &format!("PUT\n{body}"), sig) {
        return Err("上传凭证无效");
    }
    let raw = URL_SAFE_NO_PAD.decode(body).map_err(|_| "上传凭证无效")?;
    let claims: UploadClaims = serde_json::from_slice(&raw).map_err(|_| "上传凭证无效")?;
    if claims.exp < now {
        return Err("上传凭证已过期，请重新开始上传");
    }
    Ok(claims)
}

/// 资源机上传完成后的回调签名：对请求体做 HMAC，放在 `X-Signature` 头里。
#[cfg_attr(not(test), allow(dead_code))]
pub fn sign_callback(body: &[u8], secret: &[u8]) -> String {
    let key = hmac::Key::new(hmac::HMAC_SHA256, secret);
    hex(hmac::sign(&key, body).as_ref())
}

pub fn verify_callback(body: &[u8], signature: &str, secret: &[u8]) -> bool {
    verify_bytes(secret, body, signature)
}

fn verify_bytes(secret: &[u8], body: &[u8], signature: &str) -> bool {
    let Some(raw) = (0..signature.len() / 2)
        .map(|i| u8::from_str_radix(signature.get(i * 2..i * 2 + 2)?, 16).ok())
        .collect::<Option<Vec<u8>>>()
    else {
        return false;
    };
    signature.len() % 2 == 0 && hmac::verify(&hmac::Key::new(hmac::HMAC_SHA256, secret), body, &raw).is_ok()
}

/// 周标识：`2026-W44`。
pub fn valid_week(value: &str) -> bool {
    let Some((year, week)) = value.split_once("-W") else { return false };
    year.len() == 4
        && year.bytes().all(|b| b.is_ascii_digit())
        && week.len() == 2
        && week.parse::<u8>().is_ok_and(|w| (1..=53).contains(&w))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const SECRET: &[u8] = b"0123456789abcdef0123456789abcdef";

    /// 固定向量：资源机 server-resource 的 sign.rs 断言同样的值，防止两边约定漂移。
    #[test]
    fn signature_vectors() {
        assert_eq!(sign(SECRET, "GET
packs/effect/a.zip
1900000000"), "f8b2ad6a870024d63002c60ec4468de0988451eb421ef3dc8378fd5e5fcd714f");
    }

    #[test]
    fn kind_limits_follow_the_plan() {
        assert_eq!(kind_limit("effect"), Some(MB));
        assert_eq!(kind_limit("game"), Some(100 * MB));
        assert_eq!(kind_limit("app"), Some(100 * MB));
        assert_eq!(upload_limit("app"), Some(100 * MB));
        assert_eq!(kind_limit("widget"), Some(8 * MB));
        assert!(!is_pure_data("app"));
        assert_eq!(kind_limit("wallpaper"), Some(32 * MB));
        assert_eq!(kind_limit("unknown"), None);
        assert!(is_pure_data("reminder"));
        assert!(!is_pure_data("widget"));
    }

    #[test]
    fn one_click_rules_by_trust() {
        assert!(one_click_allowed("game", 0, "published"));
        assert!(one_click_allowed("widget", 1, "published"));
        assert!(one_click_allowed("effect", 2, "published"));
        assert!(!one_click_allowed("scheme", 2, "published"), "方案不是内容包类型");
        assert!(one_click_allowed("reminder", 2, "published"));
        // 用户未审的任务、例行、目标和所有可运行内容都不能一键
        assert!(!one_click_allowed("task", 2, "published"));
        assert!(!one_click_allowed("routine", 2, "published"));
        assert!(!one_click_allowed("goal", 2, "published"));
        assert!(!one_click_allowed("widget", 2, "published"));
        assert!(!one_click_allowed("game", 2, "published"));
        assert!(!one_click_allowed("app", 2, "published"));
        assert!(one_click_allowed("app", 1, "published"));
        // 没发布就不行，无论谁的
        assert!(!one_click_allowed("effect", 0, "pending"));
    }

    #[test]
    fn slug_hash_and_key_validation() {
        assert!(valid_slug("rain-night_01"));
        assert!(!valid_slug("Rain"));
        assert!(!valid_slug(""));
        assert!(!valid_slug(&"a".repeat(65)));
        assert!(valid_sha256(&"a".repeat(64)));
        assert!(!valid_sha256(&"A".repeat(64)));
        assert!(!valid_sha256("abc"));
        assert!(valid_file_key("packs/effect/abc-1.zip"));
        assert!(!valid_file_key("../etc/passwd"));
        assert!(!valid_file_key("/abs/path"));
        assert!(!valid_file_key("a b"));
    }

    #[test]
    fn manifest_checks() {
        let ok = json!({"schemaVersion": 1, "kind": "effect", "id": "rain-night", "name": "雨夜窗台", "version": "1.0.0"});
        let info = validate_manifest(&ok, "effect").unwrap();
        assert_eq!(info.slug, "rain-night");
        assert_eq!(info.version_label, "1.0.0");
        assert!(validate_manifest(&ok, "widget").is_err());
        assert!(validate_manifest(&json!({"kind": "effect"}), "effect").is_err());
        assert!(validate_manifest(&json!({"schemaVersion": 1, "kind": "effect", "id": "Bad Id", "name": "x", "version": "1"}), "effect").is_err());
        assert!(validate_manifest(&json!({"schemaVersion": 1, "kind": "effect", "id": "ok", "name": " ", "version": "1"}), "effect").is_err());
        assert!(validate_manifest(&json!("text"), "effect").is_err());
    }

    fn v2_manifest() -> Value {
        json!({
            "schemaVersion": 3, "runtimeProtocol": 2, "kind": "app", "id": "mbti-lite", "name": "MBTI 性格测评",
            "description": "48 道情境题，约 10 分钟。", "version": "1.0.0", "category": "quiz",
            "icon": "icon.png", "cover": "cover.png", "orientation": "portrait",
            "input": ["touch", "mouse", "keyboard"], "capabilities": ["haptics", "share"]
        })
    }

    #[test]
    fn protocol_two_manifest_accepts_a_good_manifest() {
        let info = validate_manifest(&v2_manifest(), "app").unwrap();
        assert_eq!((info.slug.as_str(), info.version_label.as_str(), info.runtime_protocol), ("mbti-lite", "1.0.0", 2));
        let mut game = v2_manifest();
        game["kind"] = json!("game");
        game["category"] = json!("puzzle");
        game.as_object_mut().unwrap().remove("orientation");
        game.as_object_mut().unwrap().remove("capabilities");
        assert_eq!(validate_manifest(&game, "game").unwrap().runtime_protocol, 2);
        let mut desktop = v2_manifest();
        desktop["layer"] = json!("desktop");
        desktop["capabilities"] = json!(["windows", "pointer"]);
        assert!(validate_manifest(&desktop, "app").is_ok());
    }

    #[test]
    fn protocol_two_manifest_rejects_each_bad_field() {
        let bad = |field: &str, value: Value| {
            let mut m = v2_manifest();
            m[field] = value;
            validate_manifest(&m, "app")
        };
        assert!(bad("schemaVersion", json!(1)).is_err());
        assert!(bad("schemaVersion", json!(2)).is_err());
        assert!(bad("runtimeProtocol", json!(3)).is_err());
        assert!(bad("kind", json!("game")).is_err(), "kind 与所选类型不一致");
        assert!(bad("kind", json!("widget")).is_err());
        assert!(bad("name", json!("")).is_err());
        assert!(bad("name", json!("一".repeat(24))).is_ok());
        assert!(bad("name", json!("一".repeat(25))).is_err());
        assert!(bad("description", json!("")).is_err());
        assert!(bad("description", json!("一".repeat(80))).is_ok());
        assert!(bad("description", json!("一".repeat(81))).is_err());
        assert!(bad("version", json!("1_0")).is_err());
        assert!(bad("version", json!("1".repeat(33))).is_err());
        assert!(bad("category", json!("puzzle")).is_err(), "app 不能用游戏分类");
        assert!(bad("category", json!("tool")).is_ok());
        assert!(bad("orientation", json!("diagonal")).is_err());
        assert!(bad("orientation", json!("any")).is_ok());
        assert!(bad("input", json!([])).is_err());
        assert!(bad("input", json!(["gamepad"])).is_err());
        assert!(bad("input", json!("touch")).is_err());
        assert!(bad("icon", json!("")).is_err());
        assert!(bad("icon", json!(1)).is_err());
        assert!(bad("cover", json!(" ")).is_err());
        assert!(bad("capabilities", json!(["teleport"])).is_err());
        assert!(bad("capabilities", json!("haptics")).is_err());
        assert!(bad("capabilities", json!(["windows"])).is_err(), "windows 只给桌面专用作品");
        assert!(bad("capabilities", json!(["pointer"])).is_err());
        assert!(bad("layer", json!("mobile")).is_err());
        let mut game = v2_manifest();
        game["kind"] = json!("game");
        assert!(validate_manifest(&game, "game").is_err(), "游戏不能用 app 的分类 quiz");
        for field in ["schemaVersion", "id", "name", "description", "version", "category", "input", "icon", "cover"] {
            let mut m = v2_manifest();
            m.as_object_mut().unwrap().remove(field);
            assert!(validate_manifest(&m, "app").is_err(), "缺少 {field} 应拒绝");
        }
        assert!(validate_manifest(&v2_manifest(), "wallpaper").is_err());
    }

    #[test]
    fn protocol_one_manifest_is_unchanged() {
        let old = json!({"schemaVersion": 1, "kind": "game", "id": "ink-blade", "name": "水墨剑客".repeat(10), "version": "1.0.0"});
        let info = validate_manifest(&old, "game").unwrap();
        assert_eq!(info.runtime_protocol, 1, "没有 runtimeProtocol 就是协议 1，60 字名称与缺少新字段都照旧通过");
        let mut explicit = old.clone();
        explicit["runtimeProtocol"] = json!(1);
        assert_eq!(validate_manifest(&explicit, "game").unwrap().runtime_protocol, 1);
        assert!(validate_manifest(&json!({"schemaVersion": 2, "kind": "game", "id": "a", "name": "x", "version": "1"}), "game").is_ok());
    }

    #[test]
    fn download_signature_round_trip() {
        let url = signed_download_url("https://res.example.com/", "packs/effect/a.zip", 2000, SECRET);
        assert!(url.starts_with("https://res.example.com/packs/effect/a.zip?exp=2000&sig="));
        let sig = url.rsplit("sig=").next().unwrap();
        assert!(verify_download("packs/effect/a.zip", 2000, sig, 1500, SECRET));
        assert!(!verify_download("packs/effect/a.zip", 2000, sig, 2001, SECRET), "过期必须失败");
        assert!(!verify_download("packs/effect/b.zip", 2000, sig, 1500, SECRET), "换键必须失败");
        assert!(!verify_download("packs/effect/a.zip", 2000, sig, 1500, b"another-secret-another-secret-xxxx"));
        assert!(!verify_download("packs/effect/a.zip", 2000, "zz", 1500, SECRET));
    }

    #[test]
    fn upload_ticket_round_trip() {
        let claims = UploadClaims {
            uid: "u1".into(), kind: "effect".into(), size: 1000, sha256: "a".repeat(64),
            file_key: "packs/effect/x.zip".into(), exp: 5000,
        };
        let ticket = issue_upload_ticket(&claims, SECRET);
        assert_eq!(verify_upload_ticket(&ticket, 4000, SECRET).unwrap(), claims);
        assert!(verify_upload_ticket(&ticket, 5001, SECRET).is_err(), "过期");
        assert!(verify_upload_ticket(&ticket, 4000, b"another-secret-another-secret-xxxx").is_err(), "换密钥");
        let tampered = format!("{}x", ticket);
        assert!(verify_upload_ticket(&tampered, 4000, SECRET).is_err());
        assert!(verify_upload_ticket("nodot", 4000, SECRET).is_err());
    }

    #[test]
    fn callback_signature() {
        let body = br#"{"file_key":"a","size":1}"#;
        let sig = sign_callback(body, SECRET);
        assert!(verify_callback(body, &sig, SECRET));
        assert!(!verify_callback(br#"{"file_key":"b","size":1}"#, &sig, SECRET));
        assert!(!verify_callback(body, "", SECRET));
    }

    #[test]
    fn week_format() {
        assert!(valid_week("2026-W44"));
        assert!(valid_week("2026-W01"));
        assert!(!valid_week("2026-44"));
        assert!(!valid_week("2026-W54"));
        assert!(!valid_week("2026-W00"));
        assert!(!valid_week("26-W44"));
    }
}
