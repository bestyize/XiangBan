# 任务单 A：外部 Agent 适配层（`xiaoban-agent-bridge`）

这份任务单**自包含**：不需要读仓库其余部分。产出是一个独立的 Rust 库，放在 `libs/agent-bridge/`，用**统一接口**驱动两种命令行编程 Agent：OpenAI 的 **Codex CLI** 和 Anthropic 的 **Claude Code**。库不含界面、不依赖 Tauri，全部功能可以用假的 Agent 进程完整测试。

## 1. 背景

“小伴”是一个桌面助手应用。我们要让它能当指挥台：启动并监督用户**本机已安装**的 Claude Code、Codex，把进度、权限请求交给用户（电脑或手机），并能在对话里追加指令。这个库就是“和 Agent 说话”的那一层，上层（小伴的对话、手机遥控、语音）以后调用它。

红线（必须遵守，写进 README）：
- 只拉起用户本机安装的**官方可执行文件**；**不读取、不转存、不借用**它们的登录令牌或凭据文件；不重新分发它们的程序。
- 默认用最严格的权限；库不得自行提升权限，权限请求一律交给调用方决定。
- 外部 Agent 的完整会话记录由它们自己保存；本库**不解析**它们的内部会话文件（格式是内部的、会变）。

## 2. 交付物

```
libs/agent-bridge/
  Cargo.toml          # 包名 xiaoban-agent-bridge，edition 2021，不依赖 tauri
  README.md           # 用法、两个适配器的协议要点、权限映射、已知限制、验证方式
  src/
    lib.rs
    model.rs          # §3 的统一类型与 trait
    codex/            # Codex 适配器：app-server 客户端
      mod.rs  protocol.rs  process.rs
    claude/           # Claude Code 适配器：无界面 stream-json + 本机 hook 接收
      mod.rs  stream.rs  hooks.rs  process.rs
    process.rs        # 子进程管理（启动、终止、清理环境变量、超时）
  tests/
    fake_codex/       # 假的 app-server（脚本或小二进制），按 §4 的协议应答
    fake_claude/      # 假的 claude（按录制的 stream-json 夹具输出，并向 hook 地址发事件）
    fixtures/         # 夹具（§4、§5 里的样例整理成文件）
    *.rs              # 集成测试
  examples/
    probe.rs          # 探测本机：装了哪个、版本、能不能列会话（不发起对话）
```

依赖许可：允许 MIT、Apache-2.0、BSD-2/3-Clause、ISC、Zlib、BSL-1.0、CC0-1.0、**Unicode-3.0**（含 `(MIT OR Apache-2.0) AND Unicode-3.0` 这类组合）。其中 Unicode-3.0 是宽松的数据/字符表许可，`unicode-ident`（经 `proc-macro2`、`syn`、`serde_derive` 进入几乎所有用 serde 的 Rust 项目）就带它，**不可避免，仓库里现有的库也都包含**，不要为此换掉 serde 或自己造轮子。不允许 GPL / AGPL / LGPL / MPL / SSPL 等；遇到这些先停下来在交付说明里提出，不要自行决定。README 里列出直接依赖及其许可。不写 `unsafe`。公共 API 写文档注释。

## 3. 统一接口（签名可微调，语义不变）

```rust
pub enum AgentKind { Codex, ClaudeCode }

pub struct Availability { pub installed: bool, pub path: Option<PathBuf>, pub version: Option<String>, pub notes: Vec<String> }

/// 权限档位，由调用方指定，默认 Strict。
pub enum PermissionProfile {
    Strict,        // 只读；任何写文件、执行命令都要调用方批准
    AskForWrites,  // 读自由；写文件和命令要批准
    // 不提供“全自动”档位。
}

pub struct StartRequest { pub cwd: PathBuf, pub prompt: String, pub profile: PermissionProfile, pub model: Option<String>, pub name: Option<String> }
pub struct SessionInfo { pub id: String, pub title: Option<String>, pub cwd: Option<PathBuf>, pub updated_at: Option<i64>, pub kind: AgentKind }

pub enum ApprovalKind { Command, FileChange, Permission, Question }
pub enum Risk { Low, High }
pub struct Approval { pub id: String, pub kind: ApprovalKind, pub title: String, pub detail: String, pub risk: Risk, pub options: Vec<ApprovalOption> }
pub enum Decision { Allow, AllowForSession, Deny, Answer(String) }   // Answer 用于 Question

pub enum AgentEvent {
    SessionStarted { session: String },
    Status { busy: bool },
    TextDelta { text: String },                 // 流式文字
    Message { text: String },                   // 一段完整的 Agent 回复
    ToolStarted { id: String, name: String, summary: String },
    ToolFinished { id: String, ok: bool, preview: String },   // preview 截断到 2000 字符
    FileChanged { path: String },
    ApprovalRequested(Approval),
    ApprovalResolved { id: String },            // 已被任一方处理
    TurnCompleted { final_text: Option<String>, usage: Option<Usage> },
    Error { code: String, message: String },
    Closed,
}

#[async_trait]
pub trait ExternalAgent: Send + Sync {
    fn kind(&self) -> AgentKind;
    async fn availability(&self) -> Availability;
    async fn list_sessions(&self, limit: usize) -> Result<Vec<SessionInfo>, BridgeError>;
    async fn start(&self, req: StartRequest) -> Result<Box<dyn Session>, BridgeError>;
    async fn resume(&self, id: &str, cwd: &Path, profile: PermissionProfile) -> Result<Box<dyn Session>, BridgeError>;
}

#[async_trait]
pub trait Session: Send {
    fn id(&self) -> &str;
    /// 事件流，只能取一次。
    fn events(&mut self) -> tokio::sync::mpsc::Receiver<AgentEvent>;
    async fn send(&self, text: &str) -> Result<(), BridgeError>;           // 空闲时开始新一轮
    async fn steer(&self, text: &str) -> Result<(), BridgeError>;          // 正在做事时追加指令；不支持则返回 Unsupported
    async fn interrupt(&self) -> Result<(), BridgeError>;
    async fn answer(&self, approval_id: &str, decision: Decision) -> Result<(), BridgeError>;
    async fn close(self: Box<Self>) -> Result<(), BridgeError>;            // 终止进程并清理
}
```

`BridgeError` 至少区分：未安装、未登录、协议错误、进程退出、超时、不支持、调用方参数错误。错误信息里**不能含令牌、完整环境变量或用户文件内容**。

## 4. Codex 适配器（`codex app-server`）

### 4.1 传输与握手（已在 Codex CLI 0.165.0 上实测）

- 启动：`codex app-server`（默认 stdio）。**每行一个 JSON，没有 `"jsonrpc"` 字段**。
- 客户端请求：`{"id":1,"method":"initialize","params":{"clientInfo":{"name":"…","title":"…","version":"…"}}}`，响应：
  `{"id":1,"result":{"userAgent":"…/0.165.0 (…)","codexHome":"~/.codex","platformFamily":"unix","platformOs":"linux"}}`。
  之后客户端发通知 `{"method":"initialized"}`。**未登录也能握手和列会话。**
- 服务端启动后会主动推通知，例如 `{"method":"configWarning","params":{"summary":"…","details":null}}`、`{"method":"remoteControl/status/changed","params":{…}}`；未知通知一律忽略。
- 列会话：`{"id":2,"method":"thread/list","params":{"limit":5}}` → `{"id":2,"result":{"data":[…],"readStates":{},"nextCursor":null,"backwardsCursor":null}}`。

### 4.2 要用到的方法与通知（名称来自 Codex 自己导出的协议）

- 客户端请求：`thread/start`（参数含 `cwd`、`approvalPolicy`、`sandbox`、`model`、`ephemeral`、`config` 等）、`thread/resume {threadId}`、`thread/list`、`thread/read`、`thread/name/set`、`turn/start {threadId, input:[{"type":"text","text":"…"}], …}`（`threadId`、`input` 必填）、`turn/steer {threadId, expectedTurnId, input}`、`turn/interrupt {threadId, turnId}`。
- 服务端通知：`thread/started`、`thread/status/changed`、`turn/started`、`turn/completed`、`item/started`、`item/completed`、`item/agentMessage/delta`、`item/commandExecution/outputDelta`、`item/fileChange/patchUpdated`、`turn/diff/updated`、`turn/plan/updated`、`thread/tokenUsage/updated`、`serverRequest/resolved`、`error`、`warning`、`hook/started`、`hook/completed`。
- **服务端请求（带 `id`，客户端必须回应，否则 Agent 一直等）**：`item/commandExecution/requestApproval`、`item/fileChange/requestApproval`、`item/permissions/requestApproval`、`item/tool/requestUserInput`、`mcpServer/elicitation/request`，以及旧版的 `applyPatchApproval`、`execCommandApproval`；还有 `item/tool/call`、`account/chatgptAuthTokens/refresh`、`attestation/generate`（后三个本库一律回“不支持”的错误响应，**不要代为刷新或提供任何令牌**）。

### 4.3 精确的字段以 Codex 自己导出的协议为准（必须先做）

不要凭记忆写字段名。先运行：

```sh
codex app-server generate-json-schema --out ./schema      # JSON Schema
codex app-server generate-ts --out ./ts                    # TypeScript 类型（可选，更易读）
```

在 `schema/` 里读 `ClientRequest.json`、`ServerRequest.json`、`ServerNotification.json`、`ThreadStartParams`、`TurnStartParams`、`CommandExecutionRequestApprovalResponse.json`（及其他 `*RequestApprovalResponse.json`）等，按它们写类型。把你读到的关键字段和审批响应的取值写进 `README.md`。**如果你的环境没有 Codex CLI**，用 `npm i @openai/codex` 安装后运行上面的命令（安装包会带上对应平台的可执行文件）；仍然拿不到就在交付说明里把 §4 的所有字段列为“待确认”，并让协议层用宽松的 `serde_json::Value` 加薄封装，便于以后改。

### 4.4 行为要求

- 用 `PermissionProfile` 映射到 `approvalPolicy` 和 `sandbox`（Strict ≈ 只读沙箱 + 所有操作请求批准；AskForWrites ≈ 工作区可写 + 命令请求批准）。映射写进 README。
- 服务端请求 → `AgentEvent::ApprovalRequested`；调用方 `answer` 后回应对应 `id`；Agent 侧或其他客户端先处理了（`serverRequest/resolved`）→ `ApprovalResolved`。
- `risk`：删除/覆盖文件、执行包含网络或包管理器安装的命令、写工作目录之外 → `High`，其余 `Low`；规则写成可测试的纯函数。
- 子进程退出、协议行不是合法 JSON、请求超时（默认 60 秒，`turn/start` 之后的事件不设超时）都要转成明确的 `Error`/`Closed`，不能挂住或 panic。
- 启动子进程时**清掉**环境变量 `CLAUDE_CODE_SESSION_ID`、`CLAUDE_CODE_CHILD_SESSION`、`CLAUDE_CODE_REMOTE_SESSION_ID`；其余环境原样继承。
- `close` 要终止子进程（先礼后兵：关 stdin → 等 2 秒 → kill），并确保没有僵尸进程。

## 5. Claude Code 适配器（无界面 stream-json + 本机 hook）

### 5.1 基本调用（已在 Claude Code 2.1.312 上实测）

```sh
claude -p "<提示>" --output-format stream-json --verbose [--include-partial-messages]
       [--input-format stream-json]          # 单进程多轮：往 stdin 逐行写用户消息
       [--resume <session_id>]               # 可在任意目录按编号恢复
       [--permission-mode <模式>] [--allowedTools "…"] [--name <名>] [--settings '<JSON 或文件>']
```

- 输入行格式（`--input-format stream-json`）：`{"type":"user","message":{"role":"user","content":[{"type":"text","text":"…"}]}}`，每行一个，每轮得到一个 `result` 事件。
- 输出每行一个 JSON，`type` 取值：`system`（`subtype`: `init`、`hook_started`、`hook_response`、`status`、`task_summary`、`post_turn_summary`、`api_retry`）、`stream_event`（带 `--include-partial-messages` 时的文字增量）、`assistant`、`user`（工具结果）、`rate_limit_event`、`result`（`subtype:"success"` 等；字段含 `session_id`、`result`、`is_error`、`num_turns`、`total_cost_usd`、`permission_denials`、`usage`）。未知 `type` 一律忽略。
- `system/init` 含 `session_id`、`model`、`tools`、`mcp_servers`、`permissionMode`、`capabilities`（用于功能探测，不要比较版本号）。
- 会话恢复：`claude -p --resume <id> "…"` 在**另一个进程**里仍接得上，`session_id` 不变。会话记录在 `~/.claude/projects/<目录>/<id>.jsonl`，格式是内部的，**不要解析**。
- **不要加 `--bare`**：它会跳过 hooks，且不读订阅登录（只认 API 密钥）。

### 5.2 权限请求与向运行中的会话注入指令：hooks（已实测）

无界面模式下，需要权限的操作会触发 `PermissionRequest` hook（只读命令如 `echo` 被自动放行，不触发）。本库在**本机回环地址**起一个小 HTTP 接收端，通过 `--settings '<JSON>'` 给**这一次**启动的会话注入 hooks（不改用户的任何设置文件；已实测：`claude -p … --settings '<JSON>'` 里的 `http` 类型 hook 会被调用，`PermissionRequest` 返回 allow 后工具真的执行）：

```json
{"hooks":{
  "PermissionRequest":[{"hooks":[{"type":"http","url":"http://127.0.0.1:<端口>/h/<随机令牌>/PermissionRequest","timeout":600}]}],
  "Stop":[{"hooks":[{"type":"http","url":"http://127.0.0.1:<端口>/h/<随机令牌>/Stop","timeout":30}]}],
  "Notification":[…同样…], "SessionEnd":[…同样…]
}}
```

已实测的行为（请按这些写，并在 README 里注明版本）：
- `http` 类型的 hook 收不到 `SessionStart`（只有 `command` 类型会触发）；要 `SessionStart` 就用 `command` 类型，本库不需要它。
- 事件载荷（JSON）例：`PermissionRequest` → `{"session_id","transcript_path","cwd","permission_mode","hook_event_name":"PermissionRequest","tool_name":"Bash","tool_input":{"command":"touch p0-hook-test.txt","description":"…"},"permission_suggestions":[…]}`；`Stop` → `{"session_id","hook_event_name":"Stop","stop_hook_active":false,"last_assistant_message":"完成","background_tasks":[],…}`。
- **回应权限请求**：HTTP 响应体 `{"hookSpecificOutput":{"hookEventName":"PermissionRequest","decision":{"behavior":"allow"}}}` 后，被拦的工具真的会执行；`"deny"` 同理。接收端要**挂起这个 HTTP 请求直到调用方通过 `answer` 给出决定**；超时（`timeout` 的一半）仍无决定就返回 `{}`（交还给 Claude Code 自己的默认流程，等于未批准），**绝不默认允许**。
- **注入新指令**：`Stop` hook 返回 `{"decision":"block","reason":"<新指令>"}`，会话会带着这条指令继续做事（实测通过）。实现 `steer`：把指令排队，下一次 `Stop` 时返回 block；`stop_hook_active` 为真时**一律放行**（返回 `{}`），避免死循环。若 Agent 当前空闲（没有进行中的一轮）则 `steer` 等价于 `send`。
- 令牌放在 URL 路径里；接收端只绑 `127.0.0.1`，路径令牌不对返回 404，请求体上限 1 MiB，日志里不记录令牌和载荷正文。

### 5.3 行为要求

- `PermissionProfile` 映射到 `--permission-mode` 和 `--allowedTools`（Strict ≈ 只读工具；AskForWrites ≈ 读工具免批准，其余走 `PermissionRequest`）。映射写进 README。
- 把 stream-json 事件转成 `AgentEvent`：`stream_event` 的文字增量 → `TextDelta`；`assistant` 里的 `text` → `Message`，`tool_use` → `ToolStarted`；`user` 里的 `tool_result` → `ToolFinished`；`result` → `TurnCompleted`（`is_error` 为真则另发 `Error`）。
- `list_sessions`：Claude Code 没有稳定的“列会话”机器接口；**本库不解析会话文件**，所以这里返回本库自己启动过的会话（内存里记录，可选持久化到调用方给的路径），并在文档里说明限制。
- 同样清掉三个 `CLAUDE_CODE_*SESSION*` 环境变量；`close` 的终止流程同 Codex。

## 6. 测试要求（`cargo test` 一条命令跑完，不需要真实的 Codex 或 Claude Code）

- **假 Codex**：一个按 §4.1/§4.2 应答的小程序（或脚本），能脚本化地：握手、返回会话列表、响应 `thread/start`、`turn/start` 后依次推送若干通知并发出一个 `requestApproval`，等客户端回应后继续、最后 `turn/completed`；也能模拟：乱序通知、坏 JSON 行、中途退出、不回应请求。
- **假 Claude**：一个按夹具输出 stream-json 的小程序；同时能读取 `--settings` 里的 hook 地址，在需要权限的时刻 POST `PermissionRequest` 并按响应决定“执行”还是“被拒绝”，在结束时 POST `Stop` 并按响应决定是否继续（输出第二段事件）。
- 覆盖：两种适配器各自的启动 → 事件映射 → 审批允许/拒绝 → 完成；`steer`（Claude 经 Stop 注入、Codex 经 `turn/steer`）；`interrupt`；`resume`；子进程异常退出与超时；权限请求超时**不被默认允许**；`stop_hook_active` 为真时不再拦截；hook 接收端的路径令牌错误返回 404、非回环地址无法连接、超大请求体被拒；环境变量被清除（假进程回显环境）；`risk` 判定函数的表驱动测试；`PermissionProfile` 的映射；并发多个会话互不串事件。
- **可选的真实测试**：设置环境变量 `XIAOBAN_REAL_CODEX=1` / `XIAOBAN_REAL_CLAUDE=1` 且本机已安装并登录时才运行：Codex 做握手加 `thread/list`（不消耗额度）；Claude 做一个会触发权限请求的小对话（让它 `touch` 一个临时文件，批准后文件应出现）。默认全部跳过。

## 7. 验收标准（逐条自检并在交付说明里回答）

1. `cargo test` 全绿，贴原始输出；没有执行环境就明确写“未运行”。
2. `cargo clippy --all-targets -- -D warnings` 无警告。
3. `examples/probe.rs`：在装了其中之一的机器上输出版本；都没装时输出“未安装”而不是崩溃。
4. §4.3 的协议导出结果已读过，README 里写明了审批响应的实际取值和 `thread/start` 返回里会话编号的位置；若无法获得，列为“待确认”。
5. 没有读取、写入或打印任何令牌、凭据文件；日志里没有请求正文。
6. 只改了 `libs/agent-bridge/` 和 `libs/README.md`（加一行索引）。没有新建 GitHub Actions 工作流（仓库规定所有工作流只能手动触发）。

## 8. 明确不要做的

- 不要实现界面、手机遥控、语音；不要接 Tauri。
- 不要为了“省事”默认批准任何权限请求，也不要提供“全自动”档位。
- 不要解析 Claude Code 或 Codex 的内部会话文件（`~/.claude/projects/*.jsonl`、`~/.codex/sessions/*`）。
- 不确定就查或标“待确认”，不要编造字段。

## 9. 交付说明格式

请回答：新增了哪些文件；每条验收标准的结果（带原始输出）；做了哪些取舍；哪些细节是“待确认”；哪些没有测试到。
