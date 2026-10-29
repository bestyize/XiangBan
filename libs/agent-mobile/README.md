# agent-mobile

手机里的智能体内核：把 [`agent-kernel`](../agent-kernel) 编成 Android 的 `libagent_mobile.so`，由 Kotlin 通过 JNI 调用。

## 分工

- **这个库**：对话循环、调用工具、问用户同不同意、保存对话（JSONL）、上下文裁剪。
- **手机（Kotlin）**：网络请求（用 Ktor 发给 server-ai，带账号令牌）和所有工具（现在只有“设提醒”）。所以库里没有 TLS，也不用 OpenSSL，不带任何密钥。

## 协议

两边只传 JSON 行（UTF-8，一行一个对象），在 `agent_kernel::stdio` 的基础上加了几条，完整说明见 [`src/lib.rs`](src/lib.rs) 顶部：

| 方向 | 消息 | 作用 |
| --- | --- | --- |
| 内核 → 手机 | `model_request` | 请把这个（已是流式的）chat-completions 请求体发给 server-ai |
| 手机 → 内核 | `model_line` / `model_end` | 响应的每一行 / 响应结束（HTTP 状态、错误） |
| 内核 → 手机 | `tool_request` | 请执行这个手机侧工具 |
| 手机 → 内核 | `tool_result` | 工具的结果 |
| 内核 → 手机 | `approval_request` | 工具要改动东西，请问用户 |
| 手机 → 内核 | `approval`（`once` / `always` / `deny`） | 用户的回答 |
| 手机 → 内核 | `run` / `cancel` / `set_model` | 开始一轮 / 取消 / 换模型 |

除读取类（`risk: read`）工具外，所有工具每次都先问用户。

## JNI

类 `xyz.xiangban.xiaoban.common.community.AgentNative`（`common/community-core` 的 androidMain）：`nativeStart(config)`、`nativeSend(handle, line)`、`nativePoll(handle, timeoutMs)`、`nativeStop(handle)`。函数名和类名、包名绑在一起，改了要同步改 `src/jni_api.rs`。

## 测试

```sh
cd libs/agent-mobile && cargo test          # 6 项：用假的“手机”应答模型和工具，跑完整的一轮
cargo build                                  # 产出 target/debug/libagent_mobile.so
# 手机端 Kotlin 侧接真库的测试（JVM，不需要 Android）：
AGENT_MOBILE_LIB=$PWD/target/debug/libagent_mobile.so  # 然后在 client/xiaoban-multiplatform 里跑 :common:community-core:jvmTest
```

没设 `AGENT_MOBILE_LIB` 时 Kotlin 侧的内核测试会跳过（显示通过），别把“跳过”当成“通过”。

## 打包

GitHub 工作流 `BuildXiaoBanWallpaper` 的 Android 任务用 `cargo-ndk` 编出 arm64-v8a 的库（手机只支持 64 位 ARM），放进 `androidApp/src/main/jniLibs/`（已在 `.gitignore`）。本地没有这些库时，App 照常运行，只是没有智能体，退回纯云端聊天。
