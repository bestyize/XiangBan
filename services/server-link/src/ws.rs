//! 设备的 WebSocket 连接：认证、上线通知、消息转发、离线时只暂存通知。
//! 同一账号下，手机与电脑可以互发消息；中转按认证后的设备填写来源，转发的内容只做大小检查。
use crate::{
    auth::{ws_token, WS_PROTOCOL},
    error::AppError,
    frames,
    handlers::{counterparts, owned_device, Device},
    hub::Sent,
    ratelimit::Bucket,
    state::AppState,
};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    http::HeaderMap,
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{collections::HashSet, sync::Arc, time::Duration};
use tokio::sync::{mpsc, Notify};

/// 空闲超过这个时间没收到任何帧就断开（客户端每 25 秒发一次 ping）。
const IDLE_TIMEOUT: Duration = Duration::from_secs(75);
const OUTBOX: usize = 256;
const MAX_WS_MESSAGE: usize = 512 * 1024;
/// 一条消息正文（`data`）的最大字节数。
pub const MAX_DATA: usize = 256 * 1024;

#[derive(Deserialize)]
pub struct WsQuery {
    pub device_id: String,
}

#[derive(Deserialize)]
#[serde(tag = "op")]
enum ClientFrame {
    #[serde(rename = "send")]
    Send {
        to: String,
        data: String,
        #[serde(default)]
        notify: bool,
    },
    #[serde(rename = "ping")]
    Ping,
}

pub async fn connect(State(state): State<AppState>, headers: HeaderMap, Query(q): Query<WsQuery>, ws: WebSocketUpgrade) -> Result<Response, AppError> {
    let token = ws_token(&headers).ok_or_else(|| AppError::Unauthorized("请先登录".into()))?;
    let uid = state.verifier.verify(&token).await?;
    let device = owned_device(&state.pool, uid, &q.device_id, None).await?;
    Ok(ws.max_message_size(MAX_WS_MESSAGE).max_frame_size(MAX_WS_MESSAGE)
        .protocols([WS_PROTOCOL]).on_upgrade(move |socket| run(state, device, socket)))
}

async fn run(state: AppState, device: Device, socket: WebSocket) {
    let (mut sink, mut stream) = socket.split();
    let (tx, mut rx) = mpsc::channel::<String>(OUTBOX);
    let kick = Arc::new(Notify::new());
    // 先登记连接再查库：查库期间新登记的设备会通过 `add_pair` 找到这条连接，不会漏掉。
    let (conn_id, old) = state.hub.register(&device.id, tx.clone(), kick.clone(), HashSet::new());
    if let Some(old) = old {
        old.close_with(frames::error("replaced"));
    }
    let peers = counterparts(&state.pool, device.uid, &device.kind).await.unwrap_or_default();
    state.hub.extend_peers(&device.id, peers.iter().map(|p| p.id.clone()));
    let peer_ids: HashSet<String> = peers.iter().map(|p| p.id.clone()).collect();

    let writer = tokio::spawn(async move {
        while let Some(frame) = rx.recv().await {
            if sink.send(Message::Text(frame.into())).await.is_err() {
                break;
            }
        }
        let _ = sink.close().await;
    });

    let _ = tx.try_send(frames::ready(&device.id));
    let items: Vec<Value> = peers.iter().map(|p| json!({ "id": p.id, "kind": p.kind, "name": p.name, "online": state.hub.is_online(&p.id) })).collect();
    let _ = tx.try_send(frames::peers(&items));
    for peer in &peer_ids {
        state.hub.send(peer, frames::presence(&device.id, true));
    }
    for frame in state.hub.take_notify(&device.id) {
        let _ = tx.try_send(frame);
    }
    let _ = sqlx::query("UPDATE devices SET last_seen_at = now() WHERE id = $1").bind(&device.id).execute(&state.pool).await;

    let mut msgs = Bucket::new(state.config.msgs_per_sec);
    let mut bytes = Bucket::new(state.config.bytes_per_sec);
    loop {
        tokio::select! {
            _ = kick.notified() => break,
            next = tokio::time::timeout(IDLE_TIMEOUT, stream.next()) => {
                let Ok(Some(Ok(message))) = next else { break };
                match message {
                    Message::Text(text) => {
                        if !msgs.take(1.0) || !bytes.take(text.len() as f64) {
                            let _ = tx.try_send(frames::error("rate_limited"));
                            continue;
                        }
                        handle_text(&state, &device, &tx, text.as_str());
                    }
                    Message::Close(_) => break,
                    Message::Ping(_) | Message::Pong(_) => {}
                    Message::Binary(_) => { let _ = tx.try_send(frames::error("text_only")); }
                }
            }
        }
    }

    let peers_now = state.hub.peers_of(&device.id);
    if state.hub.unregister(&device.id, conn_id) {
        for peer in peers_now {
            state.hub.send(&peer, frames::presence(&device.id, false));
        }
        let _ = sqlx::query("UPDATE devices SET last_seen_at = now() WHERE id = $1").bind(&device.id).execute(&state.pool).await;
    }
    drop(tx);
    let _ = writer.await;
}

fn handle_text(state: &AppState, device: &Device, tx: &mpsc::Sender<String>, text: &str) {
    let frame: ClientFrame = match serde_json::from_str(text) {
        Ok(f) => f,
        Err(_) => {
            let _ = tx.try_send(frames::error("bad_frame"));
            return;
        }
    };
    match frame {
        ClientFrame::Ping => {
            let _ = tx.try_send(frames::pong());
        }
        ClientFrame::Send { to, data, notify } => {
            if data.len() > MAX_DATA || to.len() > 64 {
                let _ = tx.try_send(frames::error("payload_too_large"));
                return;
            }
            if !state.hub.is_paired(&device.id, &to) {
                let _ = tx.try_send(frames::error("unknown_peer"));
                return;
            }
            let out = frames::deliver(&device.id, &data);
            match state.hub.send(&to, out.clone()) {
                Sent::Delivered => {}
                Sent::Offline | Sent::Congested => {
                    if notify {
                        state.hub.store_notify(&to, out);
                    } else {
                        let _ = tx.try_send(frames::undelivered(&to, "offline"));
                    }
                }
            }
        }
    }
}
