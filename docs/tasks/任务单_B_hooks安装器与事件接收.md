# 任务单 B：hooks 安装器与事件接收（`xiaoban-agent-hooks`）

这份任务单**自包含**：不需要读仓库其余部分。产出是一个独立的 Rust 库加一个很小的转发程序，放在 `libs/agent-hooks/`。它让“小伴”能在**不改变用户使用习惯**的前提下，观察用户在终端或桌面版里自己运行的 Claude Code 会话（任务做完了、在等权限、需要输入），并能在下一个 hook 边界把一条新指令送进正在运行的会话。

## 1. 背景与红线

用户可能已经装了别的 hooks 或语音工具（例如 mcp-voice-hooks、VoiceMode），**不能破坏他们的配置**。必须做到：
- 只**追加**自己的条目，**绝不覆盖或重排**用户已有的内容；
- 改动前**备份**；
- 卸载**只删除自己加的条目**，并且只清理因此变空、且是我们创建的结构；
- 重复安装是幂等的；
- 任何失败都不能让用户的 Claude Code 变得不可用（转发程序必须快速、不抛错、不阻塞）。

## 2. 交付物

```
libs/agent-hooks/
  Cargo.toml            # 包名 xiaoban-agent-hooks；保留 JSON 键顺序（serde_json 的 preserve_order）
  README.md             # 用法、所有权规则、已验证的行为与版本、已知限制
  src/
    lib.rs
    claude_settings.rs  # §3 设置文件的计划/应用/卸载/检测
    receiver.rs         # §4 本机 HTTP 接收端
    events.rs           # §4 事件类型
    endpoint.rs         # §5 转发程序与接收端之间的“端点文件”
    codex.rs            # §6 Codex 部分（先核实再做）
    bin/xiaoban-hook.rs # §5 转发程序
  tests/                # 临时目录里的文件测试、接收端测试、转发程序端到端测试
  examples/demo.rs      # 起接收端，打印收到的事件
```

只用 MIT / Apache-2.0 / BSD 依赖，README 列出。不写 `unsafe`。公共 API 写文档注释。

## 3. Claude Code 设置文件编辑

### 3.1 已知格式

Claude Code 的设置文件是 JSON：用户级 `~/.claude/settings.json`（若设置了环境变量 `CLAUDE_CONFIG_DIR` 则在该目录下），项目级 `<项目>/.claude/settings.json` 和 `<项目>/.claude/settings.local.json`。hooks 结构：

```json
{
  "hooks": {
    "<事件名>": [
      { "matcher": "<可省略>", "hooks": [
          { "type": "command", "command": "/path/to/exe", "args": ["--flag"], "timeout": 30 },
          { "type": "http", "url": "http://127.0.0.1:1234/x", "timeout": 30 }
      ]}
    ]
  }
}
```

要点：事件 → 匹配组数组 → 处理器数组；处理器类型有 `command`、`http`、`mcp_tool`、`prompt`、`agent`；`command` 带 `args` 时不经 shell 执行（跨平台更稳）；处理器可有 `timeout`、`statusMessage`、`if`、`once`。**已实测**：`http` 类型收不到 `SessionStart`，`command` 类型能收到全部事件；`PermissionRequest` 的 hook 返回 `{"hookSpecificOutput":{"hookEventName":"PermissionRequest","decision":{"behavior":"allow"}}}` 会让被拦的工具执行；`Stop` 的 hook 返回 `{"decision":"block","reason":"…"}` 会让会话带着 `reason` 里的新指令继续做事。官方文档 https://code.claude.com/docs/en/hooks 有完整的事件列表和载荷，写代码前请核对一遍，把和上面不一致的地方写进 README。

### 3.2 所有权规则（核心）

我们加的处理器必须能被**可靠地识别**，且不依赖端口（端口会变）。规定：处理器是 `type:"command"`，`command` 的文件名（不含扩展名）等于 `xiaoban-hook`，`args` 的第一项是事件名。凡符合这条的就是“我们的”，其余一律不动。其他工具的条目（包括与我们监听同一事件的）原样保留，顺序不变。

### 3.3 要实现的函数（签名可微调）

```rust
pub enum Scope { User, Project(PathBuf), ProjectLocal(PathBuf) }
pub struct InstallSpec { pub forwarder: PathBuf, pub events: Vec<String>, pub timeout_secs: u32 }  // 默认事件见 §3.4
pub struct Plan { pub path: PathBuf, pub before: String, pub after: String, pub changes: Vec<Change> }  // 供界面展示 diff

pub fn plan_install(scope: &Scope, spec: &InstallSpec) -> Result<Plan, HooksError>;
pub fn plan_uninstall(scope: &Scope) -> Result<Plan, HooksError>;
pub fn apply(plan: &Plan) -> Result<Applied, HooksError>;       // 先备份再原子写入（临时文件加改名）
pub fn status(scope: &Scope) -> Result<Status, HooksError>;      // 已装哪些事件、转发程序路径是否仍存在、是否有别的工具的 hooks
pub fn detect_coexisting() -> Vec<Coexisting>;                    // 检测 mcp-voice-hooks / voicemode 等（读设置文件里的条目和已注册的 MCP 服务名，只读）
pub fn restore_latest_backup(scope: &Scope) -> Result<(), HooksError>;
```

### 3.4 默认安装的事件

`PermissionRequest`、`Notification`、`Stop`、`StopFailure`、`SubagentStop`、`UserPromptSubmit`、`SessionStart`、`SessionEnd`。（`PreToolUse`/`PostToolUse` 噪声太大，默认不装，可通过 `InstallSpec.events` 指定。）

### 3.5 编辑要求

- 保持文件原有的缩进风格（2 空格或 4 空格或制表符）和键顺序；文件不存在则新建，仅含我们的条目。
- 文件内容不是合法 JSON（包括带注释的 JSONC）时**拒绝修改**并返回清晰的错误，**不要尝试修复**。
- 备份文件名 `settings.json.xiaoban-backup-<UTC 时间戳>`，放在同目录；保留最近 5 份。
- 写入用“写临时文件 → 刷盘 → 改名”，Windows 上改名失败要有重试与回退。
- `hooks` 键里除事件数组外还可能有其他键（例如 `disableAllHooks` 是顶层键）；遇到不认识的结构原样保留。
- 路径含空格、中文、Windows 反斜杠都要正确处理（`command` 用绝对路径，`args` 不经 shell）。

## 4. 本机 HTTP 接收端

```rust
pub struct HookServer { /* 绑定 127.0.0.1:随机端口，随机令牌 */ }
impl HookServer {
    pub async fn start(config: ReceiverConfig) -> io::Result<HookServer>;
    pub fn endpoint(&self) -> Endpoint;                       // 端口、令牌
    pub fn events(&self) -> broadcast::Receiver<HookEvent>;  // 收到的所有事件
    pub async fn shutdown(self);
    // 需要回应的两类事件：
    pub fn respond(&self, event_id: EventId, decision: HookDecision);   // 对 PermissionRequest
    pub fn queue_instruction(&self, session_id: &str, text: String);    // 下一次 Stop 时把它塞进去（一次性）
}
pub enum HookDecision { Allow, Deny { message: Option<String> }, NoOpinion }
```

路由：`POST /h/<令牌>/<事件名>`，请求体是 Claude Code 发来的事件 JSON（见 §4.1）。响应：
- `PermissionRequest`：**挂起请求**直到 `respond` 被调用；`Allow` → `{"hookSpecificOutput":{"hookEventName":"PermissionRequest","decision":{"behavior":"allow"}}}`，`Deny` → `behavior:"deny"`（带 `message`），`NoOpinion` → `{}`。超过 `ReceiverConfig.permission_timeout`（默认 120 秒，必须小于安装时写的 hook `timeout`）仍无决定 → 返回 `{}`（交还给 Claude Code 自己的提示流程），**绝不默认允许**。
- `Stop`：若该会话有排队的指令且 `stop_hook_active` 为假 → 返回 `{"decision":"block","reason":"<指令>"}` 并清除该指令；否则 `{}`。**`stop_hook_active` 为真时一律返回 `{}`**，防止死循环。
- 其他事件：立即返回 `{}`。

安全：只绑 `127.0.0.1`；路径令牌不对返回 404；`Host` 头必须是 `127.0.0.1`/`localhost`（防 DNS 重绑定）；请求体上限 1 MiB；单连接读超时 10 秒；日志里不记录令牌和载荷正文。

### 4.1 事件载荷（实测样例）

公共字段：`session_id`、`transcript_path`、`cwd`、`hook_event_name`、`permission_mode`、`prompt_id`。

- `PermissionRequest`：`{"session_id":"…","cwd":"…","permission_mode":"default","hook_event_name":"PermissionRequest","tool_name":"Bash","tool_input":{"command":"touch p0-hook-test.txt","description":"Create empty test file"},"permission_suggestions":[{"type":"addDirectories","directories":["…"],"destination":"…"}]}`
- `Stop`：`{"session_id":"…","hook_event_name":"Stop","stop_hook_active":false,"last_assistant_message":"完成","background_tasks":[],"session_crons":[]}`
- `UserPromptSubmit`：多 `"prompt":"…"`。`PreToolUse`/`PostToolUse`：多 `tool_name`、`tool_input`、`tool_use_id`，`PostToolUse` 另有 `tool_response`。`SessionEnd`：多 `"reason":"other"`。`Notification` 的类型（`permission_prompt`、`idle_prompt`、`agent_needs_input` 等）在 `matcher`/载荷里，请对照官方文档确认字段名。

把它们整理成强类型的 `HookEvent`（未知字段保留在 `extra: serde_json::Value` 里，保证向前兼容）。

## 5. 转发程序 `xiaoban-hook`

被 Claude Code 当作 `command` 类型的 hook 调用：`xiaoban-hook <事件名>`，从标准输入读事件 JSON。

- 从“端点文件”读取接收端的端口和令牌：`<用户数据目录>/xiaoban/hook-endpoint.json`（`{"port":…,"token":"…","pid":…}`；用户数据目录：Windows `%APPDATA%`，macOS `~/Library/Application Support`，Linux `$XDG_DATA_HOME` 或 `~/.local/share`；可被环境变量 `XIAOBAN_HOOK_ENDPOINT` 覆盖，便于测试）。端点文件由接收端启动时写入、关闭时删除（权限 0600）。
- 把事件 `POST` 到 `http://127.0.0.1:<端口>/h/<令牌>/<事件名>`，把响应体原样打印到标准输出。
- **永不阻塞、永不报错**：端点文件不存在、端口连不上、接收端无响应（连接 500 毫秒、普通事件总共 2 秒；`PermissionRequest` 总共等到 `--wait <秒>`，默认 110 秒）→ 静默退出，退出码 0，标准输出为空（相当于“没有意见”）。绝不向标准错误打印会干扰 Claude Code 的内容。
- 冷启动要快：不要引入异步运行时，用阻塞的最小 HTTP 客户端（可以直接用标准库的 `TcpStream` 手写一个 HTTP/1.1 POST），目标 < 20 毫秒。
- 只读标准输入最多 1 MiB。

## 6. Codex 部分（先核实再做，可作为第二阶段）

Codex CLI 也有“生命周期钩子”和“任务完成通知”，但**我们没有核实过它们的配置格式**，不要凭记忆写。先做：

1. 安装 `npm i @openai/codex`（或用本机已有的），运行 `codex --help`、`codex features`、`codex doctor`、`codex app-server generate-json-schema --out ./schema`，在 `schema/` 里找 `hooks/list` 的请求/响应类型和 `hook/started`、`hook/completed` 通知的类型；查官方文档 https://developers.openai.com/codex（若无法访问就说明）。
2. 弄清并写进 README：配置文件位置（`~/.codex/config.toml`，环境变量 `CODEX_HOME` 可改）；hooks 与 `notify` 各自的配置键、载荷字段、触发时机、信任/批准机制（本机有“hook trust”之类的概念）。
3. 若能确认，用 `toml_edit` 实现与 §3 同样的规则（追加、保留注释和顺序、备份、只删自己的条目、检测共存），并提供 `plan_install_codex` 等对应函数；转发程序按 Codex 的载荷增加对应的解析。
4. 若无法确认，**不要实现**，在 README 和交付说明里列出“待确认”清单，并让 `codex.rs` 只保留一个返回 `Unsupported` 的占位。

## 7. 测试要求（`cargo test` 一条命令跑完，不需要真实的 Claude Code）

- **设置文件**：空目录新建；已有无关设置（保持缩进和键顺序）；已有同事件的别家 hooks（共存，顺序不变）；幂等重装；更新转发程序路径；卸载后字节级恢复到安装前（对没有别家条目的文件）、有别家条目时只少了我们的；非法 JSON / JSONC 拒绝修改且文件不变；备份生成与保留 5 份；`restore_latest_backup`；路径含空格/中文；CLAUDE_CONFIG_DIR 生效；`status` 在转发程序被删除后报告“失效”；`detect_coexisting` 能识别夹具里的 mcp-voice-hooks 与 voicemode 条目。
- **接收端**：令牌错误 404、`Host` 非法被拒、超大请求体被拒；`PermissionRequest` 挂起直到 `respond`（Allow/Deny/NoOpinion 三种响应体）；超时返回 `{}` 而不是允许；并发多个会话互不串；`queue_instruction` 只生效一次、只对指定会话、`stop_hook_active` 为真时不拦；事件广播；日志不含令牌和正文（用收集日志的 subscriber 断言）。
- **转发程序端到端**：构建 `xiaoban-hook`，用环境变量指向测试接收端，管道输入夹具事件，断言标准输出与接收端的决定一致；端点文件缺失/端口关闭/接收端超时都返回退出码 0 且标准输出为空；耗时断言（连不上时 < 700 毫秒返回）。
- **可选真实测试**：设置 `XIAOBAN_REAL_CLAUDE=1` 且本机已装已登录 Claude Code 时：用 `--settings` 或临时的项目级设置装上 hooks，跑一个需要权限的小任务（`touch` 一个临时文件），断言接收端收到 `PermissionRequest` 和 `Stop`，批准后文件出现；再测 `queue_instruction`：第一次 `Stop` 后会话带着注入的指令继续做事。默认跳过。

## 8. 验收标准

1. `cargo test` 全绿并贴原始输出；没有执行环境就明确写“未运行”。
2. `cargo clippy --all-targets -- -D warnings` 无警告。
3. 所有权规则、备份与卸载行为在 README 里有清楚的说明和示例。
4. `examples/demo.rs` 能起接收端并打印收到的事件。
5. Codex 部分：要么按 §6 核实并实现，要么列出明确的“待确认”清单。
6. 只改了 `libs/agent-hooks/` 和 `libs/README.md`（加一行索引）；没有新建 GitHub Actions 工作流（仓库规定所有工作流只能手动触发）。

## 9. 明确不要做的

- 不要修改用户的真实 `~/.claude` 或 `~/.codex`——测试一律在临时目录里做（用环境变量或参数指定配置目录）。
- 不要实现语音、界面、Tauri 接入。
- 不要解析 Claude Code 的会话记录文件。
- 不确定就查或标“待确认”，不要编造字段。

## 10. 交付说明格式

请回答：新增了哪些文件；每条验收标准的结果（带原始输出）；做了哪些取舍；哪些细节是“待确认”；哪些没有测试到。
