# agent-kernel

与产品无关的轻量 agent harness 内核（Rust）。宿主提供模型、存储和自己的工具，内核负责循环、流式、并行工具、取消、重试和事件。不依赖 tauri，也不含任何产品逻辑、固定路径或人设。

状态：0.3.0，已接入小伴桌面。内核循环、三种模型协议（chat completions、Responses、Claude Messages）、权限/上下文/输出落盘中间件、JSONL 存储、文件/网页/Shell、skills、记忆、子 Agent、MCP 与 stdio；支持工具 Schema 校验、结构化结果与图片。0.3 增加运行身份、可失败观察者、可选执行日志和进程管理。演进设计见 [框架审视](../../docs/architecture/agent-framework-review.md)，已实现架构图与使用约定见 [运行与进程基础](../../docs/architecture/agent-runtime.md)。

## 长期目标与运行监督（第一阶段）

可选的宿主目标调度无需污染 Agent Loop：新增无 Tauri 依赖的 `goals::GoalContract`、`GoalRecord`、`GoalStore`，保存目标、验收条件和目标运行状态；`recovery::reconcile`（`journal` feature）依据既有 `JsonlRunLog::inspect` 判断异常中断，以及是否存在已派发但结果未知的工具；`supervisor::SessionLease` 使用操作系统文件锁，在同机多个进程之间互斥一段会话的运行。

这是**安全的手动恢复底座**：目标的 `Completed` 必须由外部验证证据或用户确认产生，模型 `RunState::Completed` 仅意味着进入 `AwaitingVerification`。已派发但无工具结果的调用绝不自动重放。桌面端目前在聊天工作台提供创建、开始、暂停、核验与继续；没有独立后台守护调度器、自动质量门禁和跨设备的任务锁。完整约定见 [长期目标架构](../../docs/architecture/durable-goals.md)。

## 0.3 运行与进程

- 保留 `run` / `run_parts` / `retry_last`；新增 `run_parts_observed` / `retry_last_observed`，事件带独立 run_id、连续 sequence、时间戳和可选父 run。`RunSummary` 新增 run_id。运行状态与模型消息分开；completed 表示循环正常结束，不替代业务验收。
- `RunOptions.max_tool_calls` 按批次限制本轮工具请求数，超额批次全部拒绝并暂停。取消和权限检查后，ToolDispatched 派发意图事件允许宿主先完成记录；观察失败会阻止派发。旧事件入口不插入这一新事件。
- `ToolCtx.run_id()` 关联资源；`Tool::on_run_end` 在终止事件前清理，必须幂等，终止记录失败时可能重试清理。缺失历史结果会明确提示“动作可能已经发生”，不自动重放。
- 可选 `journal`：`JsonlRunLog` 保存独立的 v1 执行记录，支持 `read_after` 补读和 `inspect` 检查未完成工具。仅包含日志和单 run 写入约束；没有自动恢复、跨进程会话锁或授权恢复。
- 可选 `process`：宿主为工作区持有 `ProcessManager`，注入 process_start/read/write/stop。支持立即返回、日志游标、管道输入、超时、取消和受管进程清理；默认 run scope，keep_alive 显式保留到 session。宿主关闭会话和应用时主动清理。不含 PTY 或沙箱。
- `Subagent.observe_run` 提供带父 run 身份的子任务记录；临时子任务会释放 session 运行资源，对话保留仍由宿主决定。父子用量预算尚未共享。
- stdio hello 增加可选 capabilities。请求加 `observed: true` 切换到 run_event 帧，并可带 max_tool_calls；默认请求的 event/run_end 格式保持原样，协议版本仍为 v1。
- 独立 stdio 示例可通过 feature `process` 和 AGENT_ALLOW_PROCESSES=1 显式启用进程工具，仍由客户端批准，无 Tauri 依赖。

进程默认最多同时运行 8 个、保留 64 个记录、缓冲最近 256 KiB 输出；日志读取可聚合等待 10 秒，使用 next_cursor 避免重复。安装 RepeatGuard 时建议 `.ignore_tool("process_read")`，轮询仍受本轮调用预算限制。完整权限、生命周期、平台限制和示例见 [运行文档](../../docs/architecture/agent-runtime.md)。

## 0.2 接入与迁移

- `ToolSpec::new` 默认 `Risk::Exec`、串行。已核实的只读工具显式加 `.risk(Risk::Read)`，确认能并行的再加 `.parallel()`。风险声明不代替权限执行：宿主仍需安装 `Permission` 或自己的中间件。内置工具与桌面业务工具已迁移。
- 工具 Schema 在 `build` 时编译，调用前先验证参数为 JSON 对象并通过 Schema，再进入权限确认和执行。采用 `jsonschema` 0.33，关闭文件/HTTP 解析并显式拒绝外部引用；本地 `$defs`/`$ref`、类型、必填、枚举、边界、数组和组合规则由库校验。`format` 遵循库的草案默认规则；未知自定义关键字不视为安全策略。参数错误包含 JSON Pointer，不执行工具，不弹批准框。
- 原有 `Tool::execute -> Result<String, String>` 继续可用；框架改为调用 `execute_result -> ToolResult`，默认适配旧实现。返回图片或 JSON 的工具覆盖新入口，可用 `with_part`、`with_structured_content`。`output` 仍是可供中间件编辑/落盘的文本；附加文本会合并到它。结构化 JSON 受工具 `max_output` 字节限制，附加媒体合计最多 6 MiB；超过上限返回明确错误，不截断二进制或 JSON。错误发生时动作可能已经执行，不能盲目重试。
- `Message` 新增可选 `is_error`、`structured_content`、`context_summary`；`ToolEnd` 新增可选 `parts`、`structured_content`。工具错误写入历史后仍保留；Claude 使用 `is_error`，Chat Completions/Responses 使用错误文本标记。Claude 原生传图片结果；Responses 与 Chat Completions 在一组工具结果之后追加带调用标识的用户图片消息，避免部分兼容网关将工具结果中的图片数组转成含 Base64 的文本，导致上下文异常膨胀。是否能识图仍取决于模型，宿主按模型能力选择工具。
- `Turn` 新增 `tools`，中间件可看到本轮工具定义。`ContextTrim` 将其计入估算，可用 `.reserve_output(tokens)` 为输出和估算误差留余量；当前轮仍超限会在请求模型前返回错误。图片按保守基数和内嵌编码长度估算，不再固定按 800 token；估算不等于实际 tokenizer，不保证任意 Provider 都不会报上下文错误。
- `McpConfig` 默认只继承启动所需环境变量（PATH、HOME/USERPROFILE、系统路径、语言与临时目录等），通过 `.env(k, v)` 配置服务变量，或 `.inherit_env(k)` 显式放行已有变量。默认不继承模型、云服务等密钥；它不限制子进程读取磁盘。结果默认标记不可信。文本、图片、文本资源与资源链接可用，其他块明确报错；图片最多 4 MiB，单行 JSON 最大 8 MiB。图片与 `structuredContent` 会保留到模型请求和历史中。无字节内容的资源链接只展示 URI，不自动读取。
- `Subagent` 默认串行，需要并发委派时由宿主调用 `.parallel()` 并确保子任务可共享工作区。父子共享预算仍待实现。
- JSONL 与 stdio 的新增字段可选，线协议仍为 v1；Rust 结构体字面量和精确枚举匹配需补字段或使用构造器及 `..`。旧历史可读，旧版错误记录无法补出原本未保存的错误状态；降级程序可能忽略新元数据。`context_summary` 只供可信宿主写入，核心不从用户正文推断摘要，也不自动对其做压缩。

结果格式核对来源：[OpenAI function calling](https://developers.openai.com/api/docs/guides/function-calling)、[Claude tool results](https://platform.claude.com/docs/en/agents-and-tools/tool-use/handle-tool-calls)。Schema 实现与草案支持见 [jsonschema 0.33 文档](https://docs.rs/jsonschema/0.33.0/jsonschema/)。协议转换已做本地测试，真实兼容端点仍需分别验证。

结构化结果工具示例：

```rust
fn execute_result(&self, args: serde_json::Value, ctx: &ToolCtx) -> ToolResult {
    // 真实工具在这里执行一次；旧 execute 接口可提供同一操作的文本投影。
    ToolResult::ok("已生成截图")
        .with_part(Part::Image { url: "data:image/png;base64,...".into() })
        .with_structured_content(serde_json::json!({"width": 1280, "height": 720}))
}
```

## 接入点

| 接口 | 谁实现 | 作用 |
| --- | --- | --- |
| `ModelProvider` | 自带三种协议，见下；宿主可自写 | 流式调用模型，返回完整回复 |
| `Tool` | 宿主与内置工具 | 名称、JSON Schema、风险等级、超时、输出上限；`execute_result` 在独立线程运行，默认适配 `execute`；结束时 `on_run_end` 清理资源 |
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

三者共用 `ProviderProfile::new(base_url, api_key, model)`，换协议只换类型。其余配置：`extra_headers`（同名头会替换默认值，不会重复；网关要 `Authorization: Bearer` 时把 `api_key` 留空并在这里设置，也可设置 `anthropic-beta`）、`extra_body`（最后合并进请求体，如 `temperature`）、`supports_tools`、`max_output_tokens`、`max_request_bytes`、`stream_usage`（只对 chat completions 有效）。

- `max_output_tokens`：由宿主传入当前端点声明的模型能力，不维护内置模型名称/版本表。Responses 和 Chat Completions 在自动模式不附加输出上限；Anthropic Messages 必须提供 `max_tokens`，缺少服务商元数据时临时尝试 128000，遇到明确的上限错误会有界降低并在当前 Provider 生命周期内记忆。这个数字是协议回退值，**不是**对某个模型能力的断言。
- `model::ModelCapabilities`：以端点元数据为优先来源，缺少上下文或输出能力时保持未知。TokenBudget 独立计算上下文留存和预留空间；错误分类明确区分输入溢出与输出上限，避免把后者当成上下文压缩信号。模型目录实时查询和模型账号信息由宿主负责；内核不定期下载模型数据库，也不硬编码厂商型号。
- `max_request_bytes`：可选的序列化请求字节上限，默认不限制；按三种协议的实际 JSON（含工具定义、转义和图片）检查。超限时只在请求副本里逐步省略旧工具图片，保留最新工具图片结果和用户图片；仍放不下就明确报 `request_too_large`，不发送请求、不裁断二进制。
- Responses：无状态，每次发送完整 `input`，默认 `store: false`（可在 `extra_body` 覆盖）；函数工具显式发送 `strict: false`，因为该协议默认 true，会拒绝带可选参数的 schema。
- Claude：所有 system 消息合并进顶层 `system`；同一轮的多个工具结果合并成一条 user 消息，且排在文本之前；空白文本块会被丢弃（为避免被拒绝，文档里没找到这条的依据）；用量取累计值的最大值，`prompt_tokens` 含缓存读写。

**Responses 和 Claude 这两个协议已按官方资料核对过流式事件和请求字段（Anthropic 的流式与工具调用文档，OpenAI Python SDK 的类型定义），但只用本地假服务测过，没有连过真实端点**，各家兼容网关的差异也没验证。Responses 的 `phase` 字段没有支持：SDK 注释说较新的 codex 类模型在后续请求里应回传它，丢掉可能降低效果。Chat Completions 的 `reasoning_content`、Responses 的 `reasoning_summary_text.delta` 和最终摘要现在通过 `ReasoningDelta` 独立事件流式发送，并在助手消息 `reasoning` 字段中持久化，供宿主选择性展示；**只有服务商明确提供的可见文本才会进入此字段**。用于续传的 Responses reasoning 项仍存放在 `Message.provider_state`，不直接作为可见内容展示；状态按协议、端点和模型隔离，并计入上下文预算。自定义 `ModelProvider` 可以选择实现向后兼容的 `chat_with_reasoning` 来支持实时摘要。Claude 的 `thinking_delta` 作为可见推理流式发送；带签名的 `thinking` / `redacted_thinking` 块存入 `provider_state`（绑定端点、模型和产生它的请求前缀摘要），之后的请求只在前缀未变时原样放回该助手消息最前面。上下文被裁剪或压缩后前缀改变，这些块会被省略：只损失那段推理的延续，不会因“保留思考”的历史校验而被拒绝。思考与 effort 参数由宿主通过 `extra_body` 传入（当前模型用 `thinking: {type: "adaptive"}` 与 `output_config.effort`）。

```rust
let agent = Agent::builder(Arc::new(OpenAiCompat::new(profile)?), Arc::new(JsonlStore::new(dir)))
    .system_prompt("...")
    .tool(MyTool::new())
    .middleware(Permission::new(PermissionMode::Ask, my_approver))
    .build()?;
agent.run("session-1", "你好", &mut |event| { /* 转给 UI */ }, &Arc::new(AtomicBool::new(false)))?;
```

`Event` 可序列化为 `{"type": "text_delta", ...}`，协议版本见 `PROTOCOL_VERSION`。新观察入口用 RunEvent 包装原事件和独立运行状态；旧入口过滤新增派发事件，保持旧事件视图。

## 行为约定

- 同步加线程，不引入异步运行时。有 tokio 的宿主用 `spawn_blocking` 包一层。
- 同一个 Agent 实例内，同一会话同一时刻只允许一个 `run`，直到资源清理和终止事件结束才释放，否则返回 `SessionBusy`。多个实例/进程之间仍需宿主做会话互斥。
- 同一轮的工具调用：`parallel_safe` 的并发执行，`serial()` 的单独执行；结果按调用顺序回填。
- 工具不存在、参数不合法、超时、崩溃、被中间件阻止，都作为错误结果交还模型，不中断循环。
- 取消后历史里每个 `tool_call` 都有对应结果，会话可继续使用。
- 普通重试只发生在可重试错误（429、5xx、网络错误、流提前结束）且尚未流出任何文本时。明确的上下文超限另有最多两次恢复：逐步缩减请求中的旧工具结果/完整旧轮次，本次运行后续工具轮继续使用收紧后的预算；不改原始历史、不重放已执行工具。普通 400、已流出文本或无法再缩减时不恢复。
- 回答因长度被截断（`FinishReason::Length`）：纯文字在未自动续写、将返回时发出 `Event::Truncated`，`RunSummary::finish` 为 `Length`；安装 `ContinueGuard` 后可有界续写。截断回复若带工具调用，整批不执行（半截参数不可信），回填“未执行，拆小步骤、逐文件写入或局部编辑”的错误结果；无效参数换成 `{}`，保证历史合法。自动恢复只发 `Continuing { reason: "tool_call_length" }`，不把中间失败报告成最终截断。默认最多重试 3 次（共 4 次被截断后暂停），由 `AgentBuilder::max_truncated_tool_retries` 配置，0 表示第一次即暂停。新的成功工具调用会重置该预算；重复调用、失败、todo/ask_user/time_now 不重置。耗尽返回 `Paused("output_length_limit")`，已成功动作不重放，失败调用和结果仍成对保存；`max_turns` 继续兜底。
- 取消能打断等人的地方：`Approver` 用 `request.wait_for(&rx)`、`AskUser` 回调用 `ctx.wait_for(&rx)` 等待用户时，取消运行会让它们立即返回；取消后给出的批准会被丢弃，工具不执行，也不记入"总是允许"。不配合的回调（直接阻塞）会拖到它自己返回为止，stdio 桥已经处理好了。
- 工具超时靠协作式取消：超时后设置 `ToolCtx::is_cancelled`，不响应的工具线程会继续跑到结束，结果被丢弃。
- 宿主若启用 `panic = "abort"`，工具里的 panic 会直接终止进程。工具内部不要 panic。
- 已知限制：三种 Provider 都只处理 SSE 流，服务端忽略 `stream` 返回整块 JSON 时会报错；内核本身不会自动续写被截断的回答，只告知宿主，装了 `ContinueGuard` 才会续写。
- 模型回复不带工具调用即结束本次运行，这和主流 harness 一致；`before_stop` 是唯一的例外入口，`max_turns` 仍然兜底。

## 权限与注入防护

`Permission` 三种模式：`ReadOnly`（只放行 `Risk::Read`）、`Ask`、`Auto`；另有 `allow_tool`、`deny_tool`（所有模式下优先）。`Always` 只在本会话内生效；`Risk::Exec` 和 `always_ask` 的工具不记忆。

- 污染（taint）：工具声明 `untrusted_output`（网页、搜索结果、子 agent 的回答）后，该会话里所有非只读工具的自动放行（`Auto`、`allow_tool`、`Always`）都改为询问，`ApprovalRequest::tainted` 告知宿主。污染只存在内存里：重启后恢复的会话从未污染开始，`forget_session` 可清除。
- `always_ask`：任何模式都询问，不记忆（`ReadOnly` 下仍是阻止）。`skill_manage` 用它，因为 skill 会作为指令长期保留。
- `self_managed`：只改宿主自己的状态（它自己的窗口、外观、设置或它保存的内容），不碰用户的文件、其他应用和系统。`Ask` 下不询问，读过外部内容也不再因此逐步确认；`ReadOnly` 仍阻止，被 `danger_check` 标出时仍询问。只在该工具确实不改变用户数据时才标记，不要用它绕开用户文件之外的写入。

- `danger_check`（可选）：宿主给 `Permission` 装一个检查函数，参数是工具名和参数，返回“太危险”的原因。装了它，`Auto` 才真正只在被标出的调用上询问：污染和 `always_ask` 在 `Auto` 下不再强制询问，被标出的调用每次都问、不记忆，原因通过 `ApprovalRequest::danger`（stdio 协议里是 `danger` 字段）告知宿主。`Ask` 和 `ReadOnly` 的行为不变，只读工具不会被检查。不装它，`Auto` 保持上面更严格的行为。

这是纵深防御，不是隔离：已污染会话里用户点了同意，注入的内容照样能执行；`danger_check` 由宿主实现，通常是对命令行的启发式判断，拦不住藏在脚本里的命令。

## 其他中间件

`Arc<T>` 也实现了 `Middleware`，宿主可留一个句柄，例如运行中切换权限模式。建议顺序：`Permission`、`RepeatGuard`、`SpillOutput`、`ContextTrim`、`ContinueGuard`。

- `ContextTrim`：预算包含工具定义和 `reserve_output`。先把旧工具输出与结构化数据换成占位，仍超再按用户消息边界丢掉最旧的整轮；当前轮无法容纳则返回错误。不调用模型，只改本次请求，不改已存会话。token 是粗估值，预算要留余量。
- `SpillOutput`：大输出存成文件，只给模型头尾预览和路径。目录由宿主提供并清理，要让模型读到需把它加进文件工具的根目录。写失败时退回普通截断。
- `ContinueGuard`：模型没调工具就结束时，检测本次运行 `todo` 还有 `pending` 项、最后一句只宣布下一步（如冒号结尾、“让我／我先／let me…”）、空回复或长度截断，补合成用户消息续跑并发出 `Event::Continuing`。默认不限制整轮累计续跑次数，改为最多 3 次无新进展的续跑（`max_without_progress`）；首次成功执行的工具名 + 解析后的 JSON 参数组合恢复额度，参数键序不影响比较，失败、拒绝、重复调用及 `todo` / `ask_user` / `time_now` 不恢复。这是进展启发式，不保证工作正确；`max_turns` 仍限制总运行量，可用 `max_per_run` 另设累计上限（0 禁用）。
- `before_stop` 返回 `StopDecision::Finish` / `Continue(Nudge)` / `Pause(reason)`，第一个非 Finish 决策生效。需要续跑但额度已耗尽时返回 `HarnessError::Paused`（`no_progress` 或 `continuation_limit`），经 Error 事件通知宿主，不能当作 Done；最后一轮仍有工作则返回 MaxTurns。正常回答、询问用户、安全结束不会强制续跑；催过后模型明确完成或受阻也可结束，但再次空答或只宣布动作仍受保护。合成消息应在界面隐藏；宿主应对暂停提供原因和继续入口。
- `RepeatGuard`：同一工具、同样参数、同样结果连续出现时，从第 3 次起在结果后附提醒，第 6 次起直接拒绝执行（`warn_after`、`block_after` 可调）；换参数、换结果或新的用户消息都会重新计数。可用 `ignore_tool` 排除合法轮询。要放在 `SpillOutput` 前面，否则比较的是每次文件名都不同的预览。

## 内置工具

### 已有能力的按需发现

宿主用 `AgentBuilder::deferred_tool_arc(tool)` 注册低频工具，其余工具继续用 `tool` / `tool_arc`。没有延后加载工具时行为不变；有延后加载工具时，内核提供 `tool_search(query, limit?)` 和 `tool_load(names)`，并在每次模型请求的系统上下文中加入有体积预算的名称/用途目录。目录放不下的工具仍可检索；查找范围仅为当前 Agent 已注册的工具，不搜索商店，不安装软件，不发起连接或授权。

搜索命中或按准确名称加载后，下一次模型请求携带对应工具的原始函数定义；后续仍以真实工具名称调用，经原有参数校验、权限、hook、超时、取消与日志处理，不使用通用执行代理，不依赖某家服务商的 tool-search 协议。任务内保留加载状态；成功、失败、取消均清理，状态不跨会话共享。检索采用名称和描述的本地词项匹配，支持中文片段；没有语义向量检索，未命中时应尝试简短关键词或准确名称，不能直接断定能力不存在。

`tool_schemas()` 仍返回完整注册表，供兼容旧宿主及诊断；`initial_tool_schemas()` 返回冷启动可见定义，`tool_catalog()` 返回简短目录。首次开销估算需同时计入两者；运行时的 ContextTrim 在按需过滤后计入实际已加载定义。

已安装技能新增 `skill_find(query, page?, limit?)`：检索完整本地元数据，包括因 200 项列表上限或初始目录预算而未显示的技能；不搜索正文、不联网、不执行脚本。随后用 `load_skill(name)` 读取选中技能。现有 `skill_search` 是在线商店检索，二者职责不同。

针对性验证：`cargo test --locked --manifest-path libs/agent-kernel/Cargo.toml --features skills --test discovery --test skill_find`。

| 工具 | feature | 风险 | 说明 |
| --- | --- | --- | --- |
| `time_now` | 默认 | Read | 内核没有时区库，宿主用 `utc_offset_minutes` 传偏移，默认 UTC |
| `todo` | 默认 | Read | 任务清单整体替换，最多一项进行中；宿主用 `snapshot(session)` 读取 |
| `ask_user` | 默认 | Read | 宿主提供回调 `(会话, 问题, ctx)`，等用户时用 `ctx.wait_for(&rx)`，取消运行即可关掉弹窗 |
| `web_search` | 默认 | Network | 只定义 `SearchBackend`，结果标记为不可信 |
| `task` | 默认 | 宿主指定 | `Subagent`：默认串行，宿主可显式 `.parallel()`。宿主构建受限的子 `Agent`，父级只拿到最终回答。子 agent 的中间件要自己装（共用同一个 `Arc<Permission>` 才会受同样约束），用量不计入父级，深度由宿主构造时决定。每次调用用全新的会话 id（`<父会话>#<工具名>-<时间>-<进程>-<序号>`）；任务结束后用 `on_finish(id)` 清理状态，子 agent 不需要留档就用 `MemoryStore` |
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
| 默认 | 依赖 `serde`、`serde_json` 与关闭外部引用读取的 `jsonschema`；含假模型 `ScriptedProvider`，以及三种协议的请求构造和流解析（不含网络） |
| `openai` | `OpenAiCompat`、`OpenAiResponses`，引入阻塞版 `reqwest` |
| `native-tls` | `openai`、`anthropic` 加系统 TLS |
| `anthropic` | `AnthropicMessages`，引入阻塞版 `reqwest` |
| `mcp` | 轻量通用 MCP 客户端（JSON-RPC 2.0 over stdio），连接外部 MCP 工具服务（如 open-computer-use） |
| `process` | 可选受管进程会话；Unix 引入 libc，Windows 引入 windows-sys 的进程与 Job API，不引入网络或异步运行时 |
| `journal` | 可选 JSONL 执行日志，与 SessionStore 及模型上下文独立 |
| `skills` | 标准 YAML 元数据解析（可选 `serde_yaml_ng`）；支持注册中心锁文件提供的无头部技能元数据 |
| `fs` `memory` `web` `shell` `stdio` `mcp` | 见上；只有 `web`、`openai`、`anthropic` 引入 `reqwest`，其余无额外依赖 |

0.1 历史 macOS arm64 实测（本 crate 的发布配置）：启用全部特性并用上全部工具、中间件和 stdio 的示例程序为 1.63 MiB。0.2 增加 Schema 校验依赖后的体积未重新测量；Windows 与宿主已有 `reqwest` 时的增量未测。

## 验证

```sh
cargo test --manifest-path libs/agent-kernel/Cargo.toml
cargo test --manifest-path libs/agent-kernel/Cargo.toml --locked --all-features
```

`tests/` 是与产品无关的一致性测试，用 `ScriptedProvider` 和本地假 HTTP 服务，不依赖真实模型。只在 macOS 上运行过；`shell` 测试仅 Unix，Windows 上的文件、路径和进程处理未验证。新产品只能新增测试，不改这些。

0.2 第一阶段验证：`cargo test --manifest-path libs/agent-kernel/Cargo.toml --locked --offline --all-features`，226 项通过；无默认 feature 测试通过。MCP 假服务测试需要开发机器上的 `python3`，产品使用内核本身不因此依赖 Python。

0.3 执行基础验证（macOS arm64）：同一全 feature 命令 246 项通过；`--no-default-features` 146 项通过；无 Tauri 的 `stdio_agent` 参考宿主带 `native-tls,stdio,fs,process,journal` 编译通过。Windows x86_64 MSVC、Linux x86_64 的 `process,journal` 交叉编译检查通过，尚未运行对应平台进程测试。桌面聊天 Rust 115 项、前端聊天 116 项及 TypeScript 检查通过。未连接真实模型端点，未验收干净机器自动准备运行时或应用崩溃后恢复。

## MCP HTTP 与宿主连接器

启用 `mcp` 和 `native-tls`（或其他提供 reqwest 的网络 feature）后，可通过 `HttpMcpClient` 连接 Streamable HTTP 服务，支持 JSON / 响应 SSE、会话头、协议协商与工具分页。可选的 `HttpAuthorization` 由宿主提供动态令牌，HTTP 请求取得当前令牌并在 401 时触发一次刷新重试；内核不执行 OAuth 发现或存储凭据。旧式 HTTP+SSE 暂不支持，返回明确错误。连接器的保存、恢复、撤销和凭据管理由 `libs/agent-tools/connectors` 负责；不进入内核。

`McpClient::close` 可以主动终止仍被旧 Agent 工具引用的本地连接；HTTP 客户端同样支持关闭。宿主还应为外部工具设置名称空间、权限与断开后的调用门禁。
