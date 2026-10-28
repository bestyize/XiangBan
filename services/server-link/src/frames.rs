//! 服务端发给设备的文本帧（JSON）。规范见 docs/agent/设备互联协议.md §4.1。
use serde_json::{json, Value};
use xiaoban_link_protocol::envelope::Envelope;

pub fn ready(device_id: &str) -> String {
    json!({ "op": "ready", "device_id": device_id, "server_time": chrono::Utc::now().timestamp() }).to_string()
}
pub fn presence(device: &str, online: bool) -> String {
    json!({ "op": "presence", "device": device, "online": online }).to_string()
}
pub fn pair_request(pairing_id: &str, phone: &Value, proof: &str) -> String {
    json!({ "op": "pair.request", "pairing_id": pairing_id, "phone": phone, "proof": proof }).to_string()
}
pub fn pair_confirmed(pairing_id: &str, desktop: &Value) -> String {
    json!({ "op": "pair.confirmed", "pairing_id": pairing_id, "desktop": desktop }).to_string()
}
pub fn pair_rejected(pairing_id: &str) -> String {
    json!({ "op": "pair.rejected", "pairing_id": pairing_id }).to_string()
}
pub fn unpaired(device: &str) -> String {
    json!({ "op": "unpaired", "device": device }).to_string()
}
pub fn deliver(env: &Envelope) -> String {
    json!({ "op": "deliver", "env": env }).to_string()
}
pub fn undelivered(to: &str, seq: u64, reason: &str) -> String {
    json!({ "op": "undelivered", "to": to, "seq": seq, "reason": reason }).to_string()
}
pub fn error(code: &str) -> String {
    json!({ "op": "error", "code": code }).to_string()
}
pub fn pong() -> String {
    json!({ "op": "pong" }).to_string()
}
