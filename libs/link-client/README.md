# xiaoban-link-client

小伴桌面端的互联客户端。协议见 [设备互联协议](../../docs/agent/设备互联协议.md)，密码学在 [link-protocol](../link-protocol/README.md)，中转是 [server-link](../../services/server-link/README.md)。不含界面，不依赖 Tauri。

宿主（桌面应用）提供三样东西：

- **令牌**（`TokenFn`）：取当前账号访问令牌，每次请求、每次重连都会调；刷新由宿主负责。
- **存储**（`store::LinkStore`）：保存设备私钥、各对端的配对密钥、重放防护进度、启动计数。内置 `FileStore`（JSON，Unix 上权限 0600）；**正式版应实现 `LinkStore`，把私钥和配对密钥放进系统钥匙串**。
- **事件**（`LinkEvent` 流）：连接状态、对端上下线、配对结果、收到的明文消息、没送达、被丢弃的信封、被顶替、设备被撤销。

```rust
let (link, mut events) = Link::start(LinkConfig::new("https://link.xiangban.xyz/api/link", "办公室电脑"), store, token).await?;
let offer = link.begin_pairing().await?;          // offer.qr_text 画成二维码
// 手机扫码 → 本库自动验证配对证明并确认 → 收到 LinkEvent::Paired
link.send(&phone_id, b"...", false)?;             // 加密后经中转发给手机
while let Some(e) = events.recv().await { /* Message { peer, data } 等 */ }
```

## 它负责什么

- **设备身份**：首次启动生成 P-256 密钥并向中转登记；中转撤销设备后自动清除本地配对并重新登记。
- **配对**：`begin_pairing` 申请配对码并打开配对窗口，生成二维码内容；手机兑换后，**用二维码里的一次性密钥验证它的证明**，通过才向中转确认并派生配对密钥；证明不对、窗口已关闭的请求一律拒绝。假请求不会烧掉窗口，真正的手机随后仍能配对。
- **连接**：WebSocket 长连接，25 秒心跳，断线指数退避加抖动自动重连（初始/上限可配），拿不到令牌时持续重试；被同设备的新连接顶替时停止重连并报告。
- **加解密**：每个对端一个发送方，会话标识 `ep` 由持久化的启动计数生成；收到的信封先解密再做重放检查，重放和解密失败的以 `Dropped` 事件报告，不当作消息。
- **持久化**：重放防护进度每秒落盘一次（以及断开和关闭时），**崩溃时最多丢最近一秒的进度**，此窗口内的信封理论上可被重放；这是为了不在每个流式增量上写盘而做的取舍。
- **解除配对**：`unpair` 通知中转，对端收到 `unpaired`；对端先解除时本地同步清除。

## 验证

```sh
TEST_DATABASE_URL=postgres://… cargo test
```

6 项端到端集成测试，用真实的 server-link（真实 Postgres 与 WebSocket，通过它的 `testing` 特性拉起）、本库的桌面端、以及测试里按协议手写的手机端：配对后双向收发并保持顺序；错误证明与已关闭窗口被拒、真手机随后仍可配对；重启后配对关系恢复、旧信封重放被拒、新会话被手机接受；桌面离线时的通知在上线后补发而指令不补发；两端各自解除配对；拿不到令牌时持续重试直到成功。没有 `TEST_DATABASE_URL` 时跳过。

桌面应用（`src-tauri`）的接入层还没写：需要把事件映射到现有会话和审批（见互联计划 §3.6），并在 Windows / macOS 上编译验证。
