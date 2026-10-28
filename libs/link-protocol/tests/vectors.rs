use xiaoban_link_protocol::vectors::generate;

/// 仓库里的 vectors/v1.json 必须与当前实现一致。改了协议就重新生成并提交，Kotlin 端同步更新。
#[test]
fn committed_vectors_match_implementation() {
    let committed: serde_json::Value = serde_json::from_str(include_str!("../vectors/v1.json")).expect("v1.json 不是合法 JSON");
    assert_eq!(committed, generate(), "vectors/v1.json 过期：运行 cargo run --example gen_vectors");
}
