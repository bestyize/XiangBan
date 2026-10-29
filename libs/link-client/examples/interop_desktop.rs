//! 跨语言互通测试用的桌面端：起一个真实的 server-link（需要 TEST_DATABASE_URL），再起本库的桌面端。
//! 标准输出第一行是 `READY {json}`，里面有服务地址和手机端令牌，供另一种语言实现的手机端使用。
//! 行为：对端上线后给它发一句 "hello from desktop"；收到的每条消息回 "ECHO:" 加大写后的内容。
//! 用法：TEST_DATABASE_URL=postgres://… cargo run --example interop_desktop
use server_link::testing::TestServer;
use std::{sync::Arc, time::Duration};
use uuid::Uuid;
use xiaoban_link_client::{static_token, store::MemoryStore, Link, LinkConfig, LinkEvent};

#[tokio::main]
async fn main() {
    let url = std::env::var("TEST_DATABASE_URL").expect("需要 TEST_DATABASE_URL");
    let srv = TestServer::start(&url).await;
    let uid = Uuid::new_v4();
    let store = Arc::new(MemoryStore::new());
    let (link, mut events) = Link::start(LinkConfig::new(&srv.base, "互通测试电脑"), store, static_token(srv.desktop_token(uid))).await.unwrap();
    // 等设备登记好再报告。
    for _ in 0..100 {
        if link.device_id().is_some() { break; }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    println!("READY {}", serde_json::json!({ "base": srv.base, "ws_base": srv.ws_base, "phone_token": srv.phone_token(uid), "desktop_id": link.device_id() }));

    let deadline = tokio::time::sleep(Duration::from_secs(180));
    tokio::pin!(deadline);
    loop {
        tokio::select! {
            _ = &mut deadline => break,
            e = events.recv() => {
                let Some(e) = e else { break };
                match e {
                    LinkEvent::PeerAdded(peer) => println!("EVENT peer {}", peer.id),
                    LinkEvent::PeerOnline { peer, online: true } => {
                        println!("EVENT online {peer}");
                        let _ = link.send(&peer, "hello from desktop".as_bytes(), false);
                    }
                    LinkEvent::Message { peer, data } => {
                        let text = String::from_utf8_lossy(&data).to_string();
                        println!("EVENT message {text}");
                        let _ = link.send(&peer, format!("ECHO:{}", text.to_uppercase()).as_bytes(), false);
                    }
                    other => println!("EVENT {other:?}"),
                }
            }
        }
    }
    link.shutdown();
}
