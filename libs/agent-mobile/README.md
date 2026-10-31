# agent-mobile

手机里的智能体内核：把 [`agent-kernel`](../agent-kernel) 编成 Android 的 `libagent_mobile.so`，由 Kotlin 通过 JNI 调用。

## 分工

- **这个库**：对话循环、工具调度、操作审批、运行中提问、持久任务清单、保存对话（JSONL）、上下文裁剪，以及重复调用和无进展保护。
- **手机（Kotlin）**：网络请求（用 Ktor 发给 server-ai，带账号令牌）、平台工具、审批与提问界面、附件选择及作品管理。所以库里没有 TLS，也不用 OpenSSL，不带任何密钥；实际平台工具由启动配置的 `tools` 决定。

内置 `time_now`、`todo`、`ask_user`。启动配置可传 `utc_offset_minutes`（例如中国时间为 480），不传仍用 UTC，以兼容旧宿主。`time_now` 不自带时区数据库，手机应在启动时传当前偏移。

`todo` 的计划、证据与交接说明单独保存在 `sessions_dir/progress/`，新运行和重启后恢复，不依赖聊天上下文是否已被裁剪；不同会话互相隔离。旧会话无需迁移。宿主应只为同一个 `sessions_dir` 创建一个 Engine，并在删除会话时同时清理它的聊天记录和进度记录。恢复进度只供参考，不代表用户新增授权、独立验收或自动重放动作。

每次运行最多 25 次模型调用。`RepeatGuard` 在同参数同结果连续出现第 3 次时提示，第 6 次起阻止执行；`ContinueGuard` 继续被截断、空答、只宣布操作或仍有未完成任务的回答，连续 3 次无进展或累计 8 次续跑提示后暂停，返回失败的 `run_end` 和暂停原因，宿主应显示原因并让用户继续。临时网络错误、429 和 5xx 最多重试 2 次（首次请求之外）；取消仍会立即中断等待。没有接入后台目标恢复、独立任务验收或跨进程会话锁。

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
| 内核 → 手机 | `question`（`id`、`session`、`question`、`options`） | `ask_user` 等待用户选择或补充信息 |
| 手机 → 内核 | `answer`（`id`、`answer` 字符串或 `null`） | 用户回答或关闭问题；取消会自动释放等待 |
| 手机 → 内核 | `run` / `cancel` / `set_model` | 开始一轮 / 取消 / 换模型 |

除读取类（`risk: read`）工具外，所有工具每次都先问用户。

`run` 可以保持原来的 `input` 字符串，也可以传非空 `parts` 数组，两者只能选一个，无需变更 JNI：

```json
{"type":"run","id":"r1","session":"s1","parts":[{"type":"text","text":"参考这个画面设计游戏"},{"type":"image","url":"data:image/png;base64,..."}]}
```

图片 URL 会按 OpenAI chat-completions 的 `image_url` 发送给宿主模型请求并保存到会话。宿主负责选择与缩放图片、限制大小和确认模型支持视觉；stdio 每条消息限 8 MiB。内核能传输图片不代表手机已实现附件界面或游戏试玩。

## JNI

类 `xyz.xiangban.xiaoban.common.community.AgentNative`（`common/community-core` 的 androidMain）：`nativeStart(config)`、`nativeSend(handle, line)`、`nativePoll(handle, timeoutMs)`、`nativeStop(handle)`。函数名和类名、包名绑在一起，改了要同步改 `src/jni_api.rs`。

## 测试

```sh
cargo test --locked --manifest-path libs/agent-mobile/Cargo.toml
cd libs/agent-mobile
cargo build                                  # Linux 为 .so，macOS 为 .dylib
# 手机端 Kotlin 侧接真库的测试（JVM，不需要 Android）：
export AGENT_MOBILE_LIB=$PWD/target/debug/libagent_mobile.dylib  # macOS；Linux 改成 .so
# 然后在 client/xiaoban-multiplatform 里跑 :common:community-core:jvmTest
```

没设 `AGENT_MOBILE_LIB` 时 Kotlin 侧的内核测试会跳过（显示通过），别把“跳过”当成“通过”。

Rust 测试用假的手机应答模型和工具，覆盖完整审批执行、拒绝、取消、持久任务恢复与会话隔离、重复工具拦截、空答暂停后继续、提问回答/关闭/取消、有界重试、图片消息和本地时间偏移；不需要真实模型、Android 设备或网络。

## 打包

GitHub 工作流 `BuildXiaoBanWallpaper` 的 Android 任务用 `cargo-ndk` 编出 arm64-v8a 的库（手机只支持 64 位 ARM），放进 `androidApp/src/main/jniLibs/`（已在 `.gitignore`）。本地没有这些库时，App 照常运行，只是没有智能体，退回纯云端聊天。
