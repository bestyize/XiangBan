# :common:link-core

手机端与桌面互联的客户端和应用控制器（Kotlin Multiplatform，不依赖 Compose）。规范见 [设备互联协议](../../../../docs/agent/设备互联协议.md)，Rust 对应实现是 [`libs/link-client`](../../../../libs/link-client/README.md)，中转是 [`services/server-link`](../../../../services/server-link/README.md)。

用同一个小伴账号登录，手机就自动看到这个账号下开启了“允许手机遥控”的电脑：没有配对、没有二维码、没有密钥。

| 文件 | 内容 |
| --- | --- |
| `LinkState.kt` | 需要持久化的状态（设备编号与随机 `install_id`，没有密钥）、`LinkStorage` 接口、`PeerSummary`、事件类型 |
| `PhoneLink.kt` | 用账号令牌登记手机、WebSocket 连接与指数退避重连、同账号电脑列表（实时更新）、收发文本、设备被移除后自动重新登记 |
| `RemoteChat.kt` | 遥控会话控制器：会话列表、流式回复、工具进度、审批卡片，状态给界面用 |
| `androidMain/SharedPrefsLinkStorage.kt` | Android 上的 `LinkStorage`（应用私有 SharedPreferences） |

## 验证

```sh
./gradlew :common:link-core:jvmTest
```

- **遥控控制器**（`RemoteChatTest`，3 项）：会话列表、快照分页、流式文本、工具进度、审批请求与处理。
- **跨语言互通**（`InteropTest`，默认跳过）：设置 `LINK_INTEROP_BIN` 指向 `libs/link-client` 的 `interop_desktop` 示例（`cargo build --example interop_desktop`）和 `TEST_DATABASE_URL` 后，Kotlin 手机端对真实 server-link 和 Rust 桌面端完成：自动发现电脑、双向收发（含中文与长消息）、重启后仍是同一台设备、从账号里移除电脑。

## 还没做

- 界面按设计稿重做（在 `:business:community-native`）。
- 本模块在云端只编译、运行了 JVM 目标；Android 目标与 iOS 没有编译过（iOS 暂不支持互联）。
