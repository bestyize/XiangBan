//! 重新生成 vectors/v1.json：`cargo run --example gen_vectors`
fn main() {
    let text = serde_json::to_string_pretty(&xiaoban_link_protocol::vectors::generate()).unwrap();
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/vectors/v1.json");
    std::fs::write(path, text + "\n").unwrap();
    println!("已写入 {path}");
}
