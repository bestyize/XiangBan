//! 设备的 WebSocket 连接：认证、上线通知、信封转发、离线时只暂存通知。
use crate::{
    auth::{ws_token, WS_PROTOCOL},
    error::AppError,
    frames,
    handlers::{owned_device, Device},
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
use std::{collections::HashSet, sync::Arc, time::Duration};
use tokio::sync::{mpsc, Notify};
use xiaoban_link_protocol::{envelope::Envelope, MAX_CIPHERTEXT, VERSION};

/// 空闲超过这个时间没收到任何帧就断开（客户端每 25 秒发一次 ping）。
const IDLE_TIMEOUT: Duration = Duration::from_secs(75);
const OUTBOX: usize = 256;
const MAX_WS_MESSAGE: usize = 512 * 1024;

#[derive(Deserialize)]
pub struct WsQuery {
    pub device_id: String,
}

#[derive(Deserialize)]
#[serde(tag = "op")]
enum ClientFrame {
    #[serde(rename = "send")]
    Send {
        env: Envelope,
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

async fn active_peers(state: &AppState, device: &str) -> HashSet<String> {
    sqlx::query_scalar::<_, String>(
        "SELECT CASE WHEN desktop_id = $1 THEN phone_id ELSE desktop_id END FROM pairings WHERE (desktop_id = $1 OR phone_id = $1) AND status = 'active'",
    )
    .bind(device)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .collect()
}

async fn run(state: AppState, device: Device, socket: WebSocket) {
    let (mut sink, mut stream) = socket.split();
    let (tx, mut rx) = mpsc::channel::<String>(OUTBOX);
    let kick = Arc::new(Notify::new());
    let peers = active_peers(&state, &device.id).await;
    let (conn_id, old) = state.hub.register(&device.id, tx.clone(), kick.clone(), peers.clone());
    if let Some(old) = old {
        old.close_with(frames::error("replaced"));
    }

    let writer = tokio::spawn(async move {
        while let Some(frame) = rx.recv().await {
            if sink.send(Message::Text(frame.into())).await.is_err() {
                break;
            }
        }
        let _ = sink.close().await;
    });

    let _ = tx.try_send(frames::ready(&device.id));
    for peer in &peers {
        let _ = tx.try_send(frames::presence(peer, state.hub.is_online(peer)));
        state.hub.send(peer, frames::presence(&device.id, true));
    }
    // 桌面刚上线：把还没处理的配对请求补发给它。
    if device.kind == "desktop" {
        let pending: Vec<(uuid::Uuid, String, String, String, String)> = sqlx::query_as(
            "SELECT p.id, d.id, d.name, d.public_key, p.phone_proof FROM pairings p JOIN devices d ON d.id = p.phone_id
             WHERE p.desktop_id = $1 AND p.status = 'pending' AND d.revoked_at IS NULL AND p.created_at > now() - make_interval(secs => $2)",
        )
        .bind(&device.id)
        .bind(state.config.pair_code_ttl.as_secs() as f64)
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default();
        for (id, phone_id, name, key, proof) in pending {
            let phone = serde_json::json!({ "id": phone_id, "kind": "phone", "name": name, "public_key": key });
            let _ = tx.try_send(frames::pair_request(&id.to_string(), &phone, &proof));
        }
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

/// 信封字段的粗检查：只看形状和大小，不碰密文。
fn envelope_ok(env: &Envelope) -> bool {
    env.v == VERSION
        && env.from.len() <= 64
        && env.to.len() <= 64
        && env.ep.len() <= 32
        && env.ct.len() <= MAX_CIPHERTEXT * 4 / 3 + 8
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
        ClientFrame::Send { env, notify } => {
            if !envelope_ok(&env) {
                let _ = tx.try_send(frames::error("payload_too_large"));
                return;
            }
            if env.from != device.id {
                let _ = tx.try_send(frames::error("bad_from"));
                return;
            }
            if !state.hub.is_paired(&device.id, &env.to) {
                let _ = tx.try_send(frames::error("not_paired"));
                return;
            }
            let out = frames::deliver(&env);
            match state.hub.send(&env.to, out.clone()) {
                Sent::Delivered => {}
                Sent::Offline | Sent::Congested => {
                    if notify {
                        state.hub.store_notify(&env.to, out);
                    } else {
                        let _ = tx.try_send(frames::undelivered(&env.to, env.seq, "offline"));
                    }
                }
            }
        }
    }
}
