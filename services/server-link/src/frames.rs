//! 服务端发给设备的文本帧（JSON）。规范见 docs/agent/设备互联协议.md。
use serde_json::{json, Value};

pub fn ready(device_id: &str) -> String {
    json!({ "op": "ready", "device_id": device_id, "server_time": chrono::Utc::now().timestamp() }).to_string()
}
/// 连接建立后发一次：同一账号下、另一种类型的全部设备（含在线状态）。
pub fn peers(items: &[Value]) -> String {
    json!({ "op": "peers", "items": items }).to_string()
}
pub fn peer_added(device: &Value) -> String {
    json!({ "op": "peer.added", "device": device }).to_string()
}
pub fn peer_removed(device: &str) -> String {
    json!({ "op": "peer.removed", "device": device }).to_string()
}
pub fn presence(device: &str, online: bool) -> String {
    json!({ "op": "presence", "device": device, "online": online }).to_string()
}
/// `data` 是发送方给的文本，中转原样转交（来源 `from` 由中转按认证后的设备填写，不能伪造）。
pub fn deliver(from: &str, data: &str) -> String {
    json!({ "op": "deliver", "from": from, "data": data }).to_string()
}
pub fn undelivered(to: &str, reason: &str) -> String {
    json!({ "op": "undelivered", "to": to, "reason": reason }).to_string()
}
pub fn error(code: &str) -> String {
    json!({ "op": "error", "code": code }).to_string()
}
pub fn pong() -> String {
    json!({ "op": "pong" }).to_string()
}
