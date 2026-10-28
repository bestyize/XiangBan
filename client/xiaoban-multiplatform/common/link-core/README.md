# :common:link-core

手机端与桌面互联的协议实现和客户端（Kotlin Multiplatform，不依赖 Compose）。规范见 [设备互联协议](../../../../docs/agent/设备互联协议.md)，Rust 参考实现和测试向量在 [`libs/link-protocol`](../../../../libs/link-protocol/README.md)，中转是 [`services/server-link`](../../../../services/server-link/README.md)。

| 文件 | 内容 |
| --- | --- |
| `Protocol.kt` | 配对密钥、配对证明、二维码、加密信封、补齐、重放防护；与 Rust 实现逐字节一致 |
| `PlatformCrypto.kt` / `jvmCommon/JcaCrypto.kt` | 密码学原语接口；Android 与 JVM 共用一份 `javax.crypto` 实现（`src/jvmCommon` 同时编进两个目标）；iOS 仅留桩 |
| `LinkState.kt` | 需要持久化的状态（设备身份、配对密钥、重放进度、启动计数）、`LinkStorage` 接口、事件类型 |
| `PhoneLink.kt` | 扫码配对（核对桌面公钥与二维码一致、用一次性密钥做证明、等桌面确认）、WebSocket 连接与指数退避重连、收发、重放防护、解除配对 |

## 验证

```sh
./gradlew :common:link-core:jvmTest
```

- **向量**（4 项）：读 `libs/link-protocol/vectors/v1.json`，ECDH、配对密钥、证明、二维码必须一致；解开向量里的每个信封，并用固定的 `ep` 和序号重新加密，结果必须与向量逐字节相同。也可用环境变量 `LINK_VECTORS` 指定向量文件。
- **协议规则**（7 项）：往返与方向绑定、篡改任一可见字段都失败、补齐隐藏长度、证明绑定身份与公钥、非法公钥被拒、二维码校验、重放防护。
- **跨语言互通**（`InteropTest`，默认跳过）：设置 `LINK_INTEROP_BIN` 指向 `libs/link-client` 的 `interop_desktop` 示例（`cargo build --example interop_desktop`）和 `TEST_DATABASE_URL` 后，Kotlin 手机端对真实 server-link 和 Rust 桌面端完成：扫码配对、双向收发（含中文与长消息）、重启后恢复并使用新会话、解除配对。

## 还没做

- Android 的 `LinkStorage`：私钥与配对密钥应经 Android Keystore 保护的密钥加密后再存（Keystore 的 EC 密钥在 API 24–30 上做不了 ECDH，所以私钥用软件密钥、由 Keystore 里的 AES 密钥包起来）。
- 界面：设备页与扫码、遥控会话页、审批卡片、离线降级（在 `:business:community-native` 或新模块里）。
- 本模块在云端只编译、运行了 JVM 目标；Android 目标与 iOS 桩没有编译过。
