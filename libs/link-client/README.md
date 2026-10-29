# xiaoban-link-client

小伴桌面端的互联客户端。协议见 [设备互联协议](../../docs/agent/设备互联协议.md)，中转是 [server-link](../../services/server-link/README.md)。不含界面，不依赖 Tauri。

用同一个账号登录的手机和电脑自动互相发现：本库用账号令牌向中转登记这台设备，保持一条 WebSocket 连接，收到同账号下对端设备的列表与上下线通知，并收发文本消息。**没有配对、没有密钥。**

宿主（桌面应用）提供三样东西：

- **令牌**（`TokenFn`）：取当前账号访问令牌，每次请求、每次重连都会调；刷新由宿主负责；没登录返回 `None`。
- **存储**（`store::LinkStore`）：只存这台设备的编号和随机的 `install_id`，没有任何密钥。内置 `FileStore`（JSON）和 `MemoryStore`。
- **事件**（`LinkEvent` 流）：连接状态、对端出现 / 消失 / 上下线、收到的消息、没送达、被顶替、设备被移除。

```rust
let (link, mut events) = Link::start(LinkConfig::new("https://link.xiangban.xyz/api/link", "办公室电脑"), store, token).await?;
link.send(&phone_id, b"...", false)?;             // 经中转发给手机
while let Some(e) = events.recv().await { /* PeerAdded / Message { peer, data } 等 */ }
```

## 它负责什么

- **设备登记**：首次启动生成随机 `install_id` 并登记；重复登记拿回同一个设备编号。设备在中转被移除（或换了账号）后，自动清除本地编号并以新的 `install_id` 重新登记。登记串行化，启动后立刻发消息也只会登记一台。
- **连接**：WebSocket 长连接，25 秒心跳，断线指数退避加抖动自动重连；拿不到令牌时持续重试；被同设备的新连接顶替时停止重连并报告。
- **对端表**：维护同账号下对端设备的名称与在线状态，`peers()` 随时可读；新对端登记不用重连就能看到。
- **发送**：`send` 只接受已知对端和 UTF-8 文本，进入队列即返回；没送达会收到 `Undelivered`。`notify = true` 的消息在对端离线时由中转暂存。

## 验证

```sh
TEST_DATABASE_URL=postgres://… cargo test
```

6 项端到端集成测试，用真实的 server-link（真实 Postgres 与 WebSocket，通过它的 `testing` 特性拉起）和两个本库的客户端（一个当电脑、一个当手机）：同账号互相发现并双向收发（中文、6 万字大消息、连发 50 条保持顺序、来源由中转填写）；不同账号互相看不到；对端离线时通知在上线后补发而指令不补发；移除设备后它收到通知并重新登记成新设备；拿不到令牌时持续重试且只登记一台设备；状态文件重启后仍是同一台设备、旧版本的多余字段被忽略。没有 `TEST_DATABASE_URL` 时跳过。

跨语言互通示例 `examples/interop_desktop.rs`：起真实 server-link 和桌面端，供 Kotlin 手机端的 `InteropTest` 连接（见 `:common:link-core` 的 README）。
