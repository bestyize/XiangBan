# agent-kernel

与产品无关的轻量 agent harness 内核（Rust）。宿主提供模型、存储和自己的工具，内核负责循环、流式、并行工具、取消、重试和事件。不依赖 tauri，也不含任何产品逻辑、固定路径或人设。

状态：里程碑 3 已实现。内核循环、三种模型协议（chat completions、Responses、Claude Messages）、权限/上下文/输出落盘中间件、JSONL 存储、内置工具（时间、任务清单、提问、文件读写、网页、搜索接口、shell、子 agent）、skills 与记忆笔记、JSON 行协议（stdio）。尚未接入任何产品。

## 接入点

| 接口 | 谁实现 | 作用 |
| --- | --- | --- |
| `ModelProvider` | 自带三种协议，见下；宿主可自写 | 流式调用模型，返回完整回复 |
| `Tool` | 宿主与内置工具 | 名称、JSON Schema、风险等级、超时、输出上限；`execute` 在独立线程运行 |
| `SessionStore` | 自带 `MemoryStore`、`JsonlStore` | 只负责 `load` 与 `append` |
| `Middleware` | 宿主与内置中间件 | `before_model`、`after_model`、`before_tool`（可阻止，拿到 `ToolCtx`，含会话 id 和取消状态）、`after_tool`、`before_stop`（模型不调工具、本轮将结束时调用，返回 `Nudge` 可让运行继续） |
| `Approver` | 宿主 | 需要确认时弹窗，返回 `Once`、`Always` 或 `Deny`；等用户时用 `request.wait_for(&rx)`，取消运行才能关掉弹窗 |
| `SearchBackend` | 宿主 | `web_search` 背后的搜索服务，内核不绑定任何一家 |
| `prompt_section` | 宿主与 skills、记忆 | 追加到系统提示的片段，每次调用模型前重新生成 |

## 模型协议

| 类型 | 协议 | feature | `base_url` 写法 | 鉴权 |
| --- | --- | --- | --- | --- |
| `OpenAiCompat` | `/chat/completions` | `openai` | 含 `/v1`，如 `https://api.openai.com/v1` | `Authorization: Bearer` |
| `OpenAiResponses` | `/responses` | `openai` | 同上 | 同上 |
| `AnthropicMessages` | `/v1/messages` | `anthropic` | 不含 `/v1`，如 `https://api.anthropic.com`；已带 `/v1` 的也能用 | `x-api-key` 加 `anthropic-version: 2023-06-01` |

三者共用 `ProviderProfile::new(base_url, api_key, model)`，换协议只换类型。其余配置：`extra_headers`（同名头会替换默认值，不会重复；网关要 `Authorization: Bearer` 时把 `api_key` 留空并在这里设置，也可设置 `anthropic-beta`）、`extra_body`（最后合并进请求体，如 `temperature`）、`supports_tools`、`max_output_tokens`、`stream_usage`（只对 chat completions 有效）。

- `max_output_tokens`：Claude 必须给出，未设置时用 8192；Responses 对应 `max_output_tokens`；chat completions 不发送，各家参数名不一致（`max_tokens` 或 `max_completion_tokens`），请放进 `extra_body`。
- Responses：无状态，每次发送完整 `input`，默认 `store: false`（可在 `extra_body` 覆盖）；函数工具显式发送 `strict: false`，因为该协议默认 true，会拒绝带可选参数的 schema。
- Claude：所有 system 消息合并进顶层 `system`；同一轮的多个工具结果合并成一条 user 消息，且排在文本之前；空白文本块会被丢弃（为避免被拒绝，文档里没找到这条的依据）；用量取累计值的最大值，`prompt_tokens` 含缓存读写。

**Responses 和 Claude 这两个协议已按官方资料核对过流式事件和请求字段（Anthropic 的流式与工具调用文档，OpenAI Python SDK 的类型定义），但只用本地假服务测过，没有连过真实端点**，各家兼容网关的差异也没验证。Responses 的 `phase` 字段没有支持：SDK 注释说较新的 codex 类模型在后续请求里应回传它，丢掉可能降低效果。思考内容（Claude 的 thinking、Responses 的 reasoning）不会带回下一轮，所以带工具循环时不要通过 `extra_body` 开启扩展思考。

```rust
let agent = Agent::builder(Arc::new(OpenAiCompat::new(profile)?), Arc::new(JsonlStore::new(dir)))
    .system_prompt("...")
    .tool(MyTool::new())
    .middleware(Permission::new(PermissionMode::Ask, my_approver))
    .build()?;
agent.run("session-1", "你好", &mut |event| { /* 转给 UI */ }, &Arc::new(AtomicBool::new(false)))?;
```

`Event` 可序列化为 `{"type": "text_delta", ...}`，协议版本见 `PROTOCOL_VERSION`。

## 行为约定

- 同步加线程，不引入异步运行时。有 tokio 的宿主用 `spawn_blocking` 包一层。
- 同一会话同一时刻只允许一个 `run`，否则返回 `SessionBusy`。
- 同一轮的工具调用：`parallel_safe` 的并发执行，`serial()` 的单独执行；结果按调用顺序回填。
- 工具不存在、参数不合法、超时、崩溃、被中间件阻止，都作为错误结果交还模型，不中断循环。
- 取消后历史里每个 `tool_call` 都有对应结果，会话可继续使用。
- 重试只发生在可重试错误（429、5xx、网络错误、流提前结束）且尚未流出任何文本时。
- 回答因长度被截断（`FinishReason::Length`，三种协议各自的 `length`、`max_tokens`、`max_output_tokens` 等都归到这里）：发出 `Event::Truncated`，`RunSummary::finish` 为 `Length`。截断的回答里若带工具调用，一个都不执行（半截参数不可信），改为回给模型一条"未执行，请缩短回复或拆小步骤"的错误结果让它重试；历史里解析不了的参数会被换成 `{}`，保证之后仍是合法 JSON。模型反复被截断时由 `max_turns` 兜底。
- 取消能打断等人的地方：`Approver` 用 `request.wait_for(&rx)`、`AskUser` 回调用 `ctx.wait_for(&rx)` 等待用户时，取消运行会让它们立即返回；取消后给出的批准会被丢弃，工具不执行，也不记入"总是允许"。不配合的回调（直接阻塞）会拖到它自己返回为止，stdio 桥已经处理好了。
- 工具超时靠协作式取消：超时后设置 `ToolCtx::is_cancelled`，不响应的工具线程会继续跑到结束，结果被丢弃。
- 宿主若启用 `panic = "abort"`，工具里的 panic 会直接终止进程。工具内部不要 panic。
- 已知限制：三种 Provider 都只处理 SSE 流，服务端忽略 `stream` 返回整块 JSON 时会报错；内核本身不会自动续写被截断的回答，只告知宿主，装了 `ContinueGuard` 才会续写。
- 模型回复不带工具调用即结束本次运行，这和主流 harness 一致；`before_stop` 是唯一的例外入口，`max_turns` 仍然兜底。

## 权限与注入防护

`Permission` 三种模式：`ReadOnly`（只放行 `Risk::Read`）、`Ask`、`Auto`；另有 `allow_tool`、`deny_tool`（所有模式下优先）。`Always` 只在本会话内生效；`Risk::Exec` 和 `always_ask` 的工具不记忆。

- 污染（taint）：工具声明 `untrusted_output`（网页、搜索结果、子 agent 的回答）后，该会话里所有非只读工具的自动放行（`Auto`、`allow_tool`、`Always`）都改为询问，`ApprovalRequest::tainted` 告知宿主。污染只存在内存里：重启后恢复的会话从未污染开始，`forget_session` 可清除。
- `always_ask`：任何模式都询问，不记忆（`ReadOnly` 下仍是阻止）。`skill_manage` 用它，因为 skill 会作为指令长期保留。

- `danger_check`（可选）：宿主给 `Permission` 装一个检查函数，参数是工具名和参数，返回“太危险”的原因。装了它，`Auto` 才真正只在被标出的调用上询问：污染和 `always_ask` 在 `Auto` 下不再强制询问，被标出的调用每次都问、不记忆，原因通过 `ApprovalRequest::danger`（stdio 协议里是 `danger` 字段）告知宿主。`Ask` 和 `ReadOnly` 的行为不变，只读工具不会被检查。不装它，`Auto` 保持上面更严格的行为。

这是纵深防御，不是隔离：已污染会话里用户点了同意，注入的内容照样能执行；`danger_check` 由宿主实现，通常是对命令行的启发式判断，拦不住藏在脚本里的命令。

## 其他中间件

`Arc<T>` 也实现了 `Middleware`，宿主可留一个句柄，例如运行中切换权限模式。建议顺序：`Permission`、`RepeatGuard`、`SpillOutput`、`ContextTrim`、`ContinueGuard`。

- `ContextTrim`：超出 token 预算时，先把旧工具输出换成占位，仍超再按用户消息边界丢掉最旧的整轮。不调用模型，只改本次请求，不改已存会话。token 是粗估值，预算要留余量。
- `SpillOutput`：大输出存成文件，只给模型头尾预览和路径。目录由宿主提供并清理，要让模型读到需把它加进文件工具的根目录。写失败时退回普通截断。
- `ContinueGuard`：模型没调工具就结束时，判断是不是活没干完，是就补一条合成的用户消息（`Message::synthetic` 为 true）让它接着做，并发出 `Event::Continuing { reason }`。四种情形：本次运行里 `todo` 最新清单还有 `pending` 项（`unfinished_todos`；只剩一项 `in_progress` 视为忘了勾，不算）；回复最后一句以冒号结尾或在说“让我／我先／let me…”（`announced_action`）；回复为空（`empty`）；纯文字回答被长度截断（`length`，此时不再发 `Truncated`，续写用完才发）。最后一句是在问用户（问号、“吗”“呢”）时不催；催过一次模型仍不调工具就接受结束（续写截断除外）；每次运行最多催 3 次（`max_per_run`），最后一轮不催。判断是启发式的，误判时模型只需回一句话结束。宿主装了它，就要在界面上隐藏合成消息。
- `RepeatGuard`：同一工具、同样参数、同样结果连续出现时，从第 3 次起在结果后附提醒，第 6 次起直接拒绝执行（`warn_after`、`block_after` 可调）；换参数、换结果或新的用户消息都会重新计数。要放在 `SpillOutput` 前面，否则比较的是每次文件名都不同的预览。

## 内置工具

| 工具 | feature | 风险 | 说明 |
| --- | --- | --- | --- |
| `time_now` | 默认 | Read | 内核没有时区库，宿主用 `utc_offset_minutes` 传偏移，默认 UTC |
| `todo` | 默认 | Read | 任务清单整体替换，最多一项进行中；宿主用 `snapshot(session)` 读取 |
| `ask_user` | 默认 | Read | 宿主提供回调 `(会话, 问题, ctx)`，等用户时用 `ctx.wait_for(&rx)`，取消运行即可关掉弹窗 |
| `web_search` | 默认 | Network | 只定义 `SearchBackend`，结果标记为不可信 |
| `task` | 默认 | 宿主指定 | `Subagent`：宿主构建受限的子 `Agent`，父级只拿到最终回答。子 agent 的中间件要自己装（共用同一个 `Arc<Permission>` 才会受同样约束），用量不计入父级，深度由宿主构造时决定。每次调用用全新的会话 id（`<父会话>#<工具名>-<时间>-<进程>-<序号>`），所以即使子 agent 用持久化存储，重启后也不会读到上一个任务的历史；任务结束后用 `on_finish(id)` 清理该会话的状态（如 `Permission::forget_session`、`Todo::clear_session`、删除存储），子 agent 不需要留档就用 `MemoryStore` |
| `read_file` `list_dir` `grep` | `fs` | Read | 限定在宿主给的根目录内，符号链接逃逸被拒绝；`grep` 是字面量匹配 |
| `write_file` `edit_file` | `fs` | Write | 同样限定根目录；拒绝 `..`，悬空符号链接不可写穿；原子写入，保留权限；`edit_file` 要求 `old_string` 唯一 |
| `web_fetch` | `web` | Network | 只取公网 http/https；每一跳（含重定向）都解析主机并拒绝私有和保留地址，连接固定到检查过的 IP；可设域名白名单；https 需要 `native-tls` |
| `shell` | `shell` | Exec | 不是沙箱。环境变量清空（宿主的密钥不外泄）、stdin 关闭、输出有上限、超时或取消时杀整个进程组。Windows 路径未运行验证 |
| `load_skill` `skill_manage` | `skills` | Read、Write | 见下 |
| `memory_read` `memory_write` | `memory` | Read、Write | 每条笔记一个 Markdown 文件，最多 100 条、每条 20 KB，不跟随符号链接 |

Skills：`<根>/<name>/SKILL.md`，开头 `---` 块含 `name`、`description`，提示里只放名称和描述，正文由 `load_skill` 按需读取。多个根时靠前的根优先，符号链接目录被忽略。`skill_manage`（只在配置了可写根时提供）可 create、patch、delete：只能动自己写入的（`origin: agent`）且在可写根里的 skill，用户手写的和随包的只读；修改前备份到 `.history/`（不自动清理）；删除只删 `SKILL.md`，目录里有别的文件则保留。

## 存储

`JsonlStore`：每个会话一个文件，每行一条带版本号的记录。会话 id 转义后做文件名（含大写字母，因为 macOS、Windows 默认不区分大小写），加 `s-` 前缀避开 `con`、`nul`。崩溃留下的半行读取时忽略、下次追加时删除。Unix 下仅所有者可读写。

上次崩在“模型已请求工具、结果还没存”之间时，下一次 `run` 会为缺失的结果补上错误记录；只处理历史末尾这一种情形。

## stdio 协议（feature `stdio`）

非 Rust 宿主用 `StdioBridge`：一行一个 JSON。`bridge.approver()` 交给 `Permission`，`bridge.ask_handler()` 交给 `AskUser`，再 `bridge.serve(agent, input)`。完整消息见 `src/stdio.rs` 开头；参考宿主见 `examples/stdio_agent.rs`（用环境变量配置端点，`AGENT_PROTOCOL` 选择 `chat`、`responses` 或 `anthropic`）。

- 客户端发：`run`、`cancel`、`approval`、`answer`。服务端发：`hello`、`event`、`run_end`（含 `finish`：`stop`、`length`、`content_filter`、`other`）、`approval_request`、`question`、`error`。
- 多个 `run` 可并行（会话和 run id 各自互斥）；取消会同时放行该会话挂起的确认和提问。
- 输入结束后，挂起的确认按拒绝、提问按放弃处理，运行中的任务跑完再返回。
- 单行最长 8 MiB。输出管道断开只在读下一行时才发现，宿主应两端一起关。

## 特性与体积

| feature | 作用 |
| --- | --- |
| 默认 | 只依赖 `serde`、`serde_json`；含假模型 `ScriptedProvider`，以及三种协议的请求构造和流解析（不含网络） |
| `openai` | `OpenAiCompat`、`OpenAiResponses`，引入阻塞版 `reqwest` |
| `native-tls` | `openai`、`anthropic` 加系统 TLS |
| `anthropic` | `AnthropicMessages`，引入阻塞版 `reqwest` |
| `mcp` | 轻量通用 MCP 客户端（JSON-RPC 2.0 over stdio），连接外部 MCP 工具服务（如 open-computer-use） |
| `fs` `skills` `memory` `web` `shell` `stdio` `mcp` | 见上；只有 `web`、`openai`、`anthropic` 引入 `reqwest`，其余无额外依赖 |

macOS arm64 实测（本 crate 的发布配置）：启用全部特性并用上全部工具、中间件和 stdio 的示例程序为 1.63 MiB。Windows 与宿主已有 `reqwest` 时的增量未测。

## 验证

```sh
cargo test --manifest-path libs/agent-kernel/Cargo.toml
cargo test --manifest-path libs/agent-kernel/Cargo.toml --features native-tls,fs,skills,memory,web,shell,stdio,mcp
```

`tests/` 是与产品无关的一致性测试，用 `ScriptedProvider` 和本地假 HTTP 服务，不依赖真实模型。只在 macOS 上运行过；`shell` 测试仅 Unix，Windows 上的文件、路径和进程处理未验证。新产品只能新增测试，不改这些。
