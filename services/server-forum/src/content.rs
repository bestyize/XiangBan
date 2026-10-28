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
    ("game", 8 * MB, false),
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
}

/// 检查资源包清单（`manifest.json`）：结构、类型与内容包类型一致、id 与版本号合法。
pub fn validate_manifest(manifest: &Value, kind: &str) -> Result<ManifestInfo, String> {
    let obj = manifest.as_object().ok_or("manifest 必须是对象")?;
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
    Ok(ManifestInfo { slug: slug.to_string(), version_label: version.to_string() })
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
        assert_eq!(kind_limit("game"), Some(8 * MB));
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
