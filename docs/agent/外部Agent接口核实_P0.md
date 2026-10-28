# 外部 Agent 接口核实（语音编程 P0）

状态：P0 第一部分（2026-10-28）。对应 [目标与实施计划_语音编程](../product/目标与实施计划_语音编程.md) 的 P0。这里只写**已经核实过**的事，并标明核实方式；没核实的单列在最后，不要当成结论用。

## 1. 核实方式与范围

| 对象 | 版本 | 方式 | 范围 |
| --- | --- | --- | --- |
| Claude Code | 2.1.312 | 在云端沙箱用 npm 装官方包，读 `--help`；读官方文档的 hooks、无界面、会话、Remote Control 页；用 `tools/p0-verify` 的脚本跑真实对话（见 §4b） | Linux |
| Codex CLI | 0.165.0 | 同上；另用 `codex app-server generate-json-schema` 生成协议定义并统计方法；用脚本握手并列会话（见 §4b） | Linux；**未登录，没发起过真实对话**（`turn/start` 等需登录的部分未实测） |
| Happy（slopus/happy） | 当前主分支 | 读仓库首页与搜索结果 | 只看到概述，没读源码；协议、加密细节未核实 |
| Claude 桌面版、ChatGPT 桌面版 | — | **未核实** | 云端沙箱装不了桌面应用，快捷键与无障碍树要在 Windows 和 macOS 真机上验证（见 §6） |

Codex 官方文档站（developers.openai.com）在沙箱里被网络策略拦住，所以 Codex 的信息来自**实际装出来的程序**（`--help`、协议定义、握手实测），比文档更可靠，但没有文字说明，语义要实测。

## 2. Claude Code

### 2.1 Hooks（观察层的基础）

- 配置位置：`~/.claude/settings.json`（全局）、项目 `.claude/settings.json` 与 `.claude/settings.local.json`、托管策略、插件。结构：事件 → matcher 组 → handler；handler 类型有 `command`、`http`、`mcp_tool`、`prompt`、`agent`。**`http` 类型会把事件 POST 到指定地址**，小伴可以直接起一个本机地址接收，不用写脚本。
- 所有事件的公共输入字段：`session_id`、`transcript_path`、`cwd`、`hook_event_name`、`permission_mode`，以及 `prompt_id` 等。
- 事件与我们要的场景对应：

| 场景 | 事件 |
| --- | --- |
| 一轮结束 | `Stop`（带 `last_assistant_message`）；出错结束为 `StopFailure` |
| 等待权限 | `PermissionRequest`（可由 hook 直接允许或拒绝）；`Notification` 的类型 `permission_prompt` |
| 等待用户输入 | `Notification` 的类型 `idle_prompt`、`agent_needs_input`、`elicitation_dialog` |
| 子任务 | `SubagentStop`、`TaskCompleted` |
| 会话开始/结束 | `SessionStart`、`SessionEnd` |

- `Notification` 类 hook 不能阻塞；`PermissionRequest` 可以返回决定，这是**小伴把权限请求转到手机或语音确认**的正规入口。
- 注意：`claude -p` 默认会加载项目里的 hooks 和 MCP；`--bare` 会跳过它们（且不读 OAuth 登录，只认 API 密钥）。小伴自己拉起会话时要选择是否带 `--bare`，并只加载小伴自己的配置。

### 2.2 无界面运行与会话接续（接续层的基础）

- `claude -p "…" --output-format stream-json --verbose --include-partial-messages`：逐行 JSON 事件；`system/init` 事件给出模型、工具、MCP 状态和**能力列表**（用来做功能探测，不比较版本号）。
- `--input-format stream-json`：从标准输入持续送入用户消息，实现**一个进程里多轮对话**；`--replay-user-messages` 把用户消息回显以便确认。
- 接续：`--resume <session-id>`（可在任意目录，会在本机所有项目里按编号查找）、`--continue`、`--fork-session`；`-p --resume` 可以拿到结构化结果。**用 `-p` 或 SDK 建的会话默认不出现在选择器里，但可按编号恢复。**
- 会话记录：JSONL，`~/.claude/projects/<项目>/<session-id>.jsonl`，`<项目>` 是工作目录路径把非字母数字替换为 `-`。官方明确说**记录格式是内部的、随版本变化**，不要直接解析；应该用 hooks 的 `transcript_path` 和脚本接口。结论：**观察层不解析 JSONL，只用 hooks + 无界面输出**。
- 同时在两个终端恢复同一会话而不分叉，消息会交错写进同一份记录——所以接续层必须检测占用（官方也有后台会话 `--bg`、`claude attach/agents/stop` 这套管理命令）。
- 权限：`--permission-mode`（`manual`、`acceptEdits`、`auto`、`dontAsk`、`plan`、`bypassPermissions` 等）、`--allowedTools`、`--permission-prompt-tool`（由一个 MCP 工具回答权限请求）、`--permission-prompts none`。
- 其他有用的：`--name` 给会话起名；`--mcp-config` 注入 MCP 服务（**小伴的语音工具就这样接进去**）；`--append-system-prompt`；退出码与 SIGINT/SIGTERM 的行为文档里写得很细。
- 还有官方的 **Agent SDK**（TypeScript、Python），带工具审批回调和结构化消息；若不想直接解析 stream-json，可以用它，但会引入 Node 或 Python 运行环境。

### 2.3 官方的远程控制（对我们的定位有影响）

- `claude --remote-control`：把本机会话接到 claude.ai/code 和 Claude 手机应用，手机与电脑可同时发消息、同步子任务进度。
- 要求：Pro、Max、Team、Enterprise 订阅；**不支持 API 密钥**；Team/Enterprise 需管理员开启。数据经 Anthropic 云端。
- 也有云会话（`--cloud`）和后台会话（`--bg`）。

## 3. Codex

### 3.1 命令行能力（装出来的 0.165.0）

- 子命令：`exec`（非交互）、`resume`、`fork`、`queue`（给已有会话排队一条消息，可指定远程地址）、`agents`（浏览共享本地服务上的所有会话）、`app-server`、`remote-control`（`start`、`stop`、`pair` 生成短时配对码）、`mcp`、`plugin`、`doctor`、`update`、`login`、`sandbox`、`cloud` 等。
- `codex exec --json`：事件以 JSONL 输出；`-o` 把最后一条消息写入文件；`--output-schema` 约束最终回答格式；`-C` 指定工作根；`--worktree` 在新的 git 工作树里跑；`-s read-only | workspace-write | danger-full-access`；`--ephemeral` 不落盘会话；`-c key=value` 覆盖配置。
- `codex exec resume [会话编号或名字] [提示]`、`--last`；`codex exec fork`。

### 3.2 `codex app-server`（Codex 的首选集成面）

这是 Codex 官方客户端（编辑器扩展、桌面应用）用的协议，标注为 experimental，但能直接导出类型：`generate-ts`、`generate-json-schema`。它是 JSON-RPC，既有 daemon 模式（`app-server daemon`），也可用 `app-server proxy` 通过标准输入输出接入本机控制套接字。从协议定义里统计到：

- **客户端请求 106 个**，和我们相关的：`thread/start`、`thread/resume`、`thread/list`、`thread/read`、`thread/fork`、`thread/name/set`、`thread/archive`、`thread/inject_items`、`turn/start`、`turn/steer`（对正在跑的一轮追加引导）、`turn/interrupt`、`review/start`、`model/list`、`hooks/list`、`mcpServer/tool/call`、`config/read`、`config/value/write`，以及账号、技能、插件、文件系统等。
- **服务端请求 10 个**（需要客户端回答）：`item/commandExecution/requestApproval`、`item/fileChange/requestApproval`、`item/permissions/requestApproval`、`item/tool/requestUserInput`、`mcpServer/elicitation/request`、`item/tool/call` 等——这就是**权限与提问**的接口，语音确认、手机审批都挂在这里。
- **服务端通知 85 个**：`thread/started`、`thread/status/changed`、`turn/started`、`turn/completed`、`item/started`、`item/completed`、`item/agentMessage/delta`（流式文字）、`item/commandExecution/outputDelta`、`turn/diff/updated`、`turn/plan/updated`、`thread/tokenUsage/updated`、`hook/started`、`hook/completed`、`remoteControl/status/changed` 等。
- 值得注意：协议里已有 **`thread/realtime/*`**（实时音频、转写增量）。也就是说 Codex 自身在做实时语音，必须在“差异点”里把它当竞品。
- 结论：**Codex 不必走“读会话文件”那条路，直接作为 app-server 的一个客户端**，就同时拿到会话列表、事件流、权限请求和打断/引导，并且会话与官方客户端共享。

### 3.3 Codex 的远程控制

- `codex remote-control start | stop | pair`：把 app-server daemon 以“远程控制”方式启动，并生成短时配对码；`--remote ws://… | wss://… | unix://…` 让客户端连远程服务，并可用环境变量传 bearer 令牌。官方的远程方案在 0.165.0 里已经能看到，但仍标注 experimental，实际行为和账号要求未核实。

## 4. Happy / Happier 的启示

已知（仓库首页、应用商店说明，源码未读）：
- 组成：手机/网页客户端（Expo）、命令行包装 `happy claude` / `happy codex`、中转服务；数据在设备间端到端加密；有权限请求的推送；MIT 许可。Happier 是它的分支，加了 Gemini、OpenCode 等提供方，并支持自托管。
- 做法是**让用户改用包装命令启动**，这点和我们“外挂、会话共用”的目标不同。

借鉴：端到端加密中转、扫码配对、权限推送、多提供方适配。许可是 MIT，可以参考设计，若复制代码需保留版权声明；我们不开源，所以**不直接搬代码，只参考设计**，并在引入任何代码前再核对许可。

## 4b. 实测结果（Linux 沙箱，用真实登录跑脚本，2026-10-28）

脚本在 `tools/p0-verify/`，样例在其 `samples/`。以下都是实际跑出来的，不是文档摘抄：

| 项 | 结果 |
| --- | --- |
| Codex app-server 传输 | 默认 `stdio://`，**逐行 JSON**（一行一个消息，请求带 `id` 和 `method`，**没有 `jsonrpc` 字段**）；`initialize` 传 `clientInfo` 即可，未登录也能握手；之后发 `initialized` 通知 |
| Codex `thread/list` | 未登录可调用，返回 `data`、`readStates`、`nextCursor`、`backwardsCursor` |
| Codex 启动时的通知 | 会主动推 `configWarning`、`remoteControl/status/changed` |
| Claude hooks，`http` 类型 | 能收到 `UserPromptSubmit`、`PreToolUse`、`PermissionRequest`、`PostToolUse`、`Stop`、`SessionEnd`；**收不到 `SessionStart`**（该事件只有 `command` 类型会触发） |
| Claude hooks，`command` 类型（`args` 形式，不经 shell） | 上述事件加 `SessionStart` 全部收到 |
| `PermissionRequest` 的回应 | 需要权限的 Bash 命令（`touch` 文件）触发；hook 返回 `{"hookSpecificOutput":{"hookEventName":"PermissionRequest","decision":{"behavior":"allow"}}}` 后工具**真的执行了**（http 与 command 两种都验证）。只读命令（`echo`）会被自动放行，**不会**触发 `PermissionRequest` |
| `Stop` 载荷 | 带 `last_assistant_message`（本轮最后的回答文字）、`stop_hook_active`、`background_tasks`——播报“做完了，结果是……”所需的信息一个事件里就有 |
| 无界面恢复 | `claude -p … --output-format json` 返回 `session_id`；`claude -p --resume <id>` 在**另一个进程**里仍记得内容，且仍是同一个 `session_id` |
| 会话文件位置 | `~/.claude/projects/<工作目录把非字母数字换成 ->/<session_id>.jsonl`，与文档一致 |
| 单进程多轮 | `--input-format stream-json --output-format stream-json --verbose`，向标准输入逐行写 `{"type":"user","message":{"role":"user","content":[{"type":"text","text":"…"}]}}`，每轮得到一个 `result` 事件；第二轮约 5 秒（含模型耗时） |
| `system/init` 事件 | 含 `session_id`、`model`、`tools`、`mcp_servers`、`permissionMode`、`capabilities`（功能探测用）等 |
| **hooks 注入新指令** | `Stop` hook 返回 `{"decision":"block","reason":"…"}` 后，会话**带着这条新指令继续做事**（实测：reason 里要求 `touch` 一个文件，文件真的被创建，最终回复按新指令给出）。这是 mcp-voice-hooks 一类工具“把话送进正在工作的会话”的机制，已在 `-p` 模式验证；交互模式按文档行为应一致，仍需真机确认。注意要在第二次 `Stop` 放行（`stop_hook_active` 为真时不再拦），否则会死循环 |
| 环境陷阱 | 如果从 Claude Code 里面运行脚本，子进程会继承 `CLAUDE_CODE_SESSION_ID` 之类变量而复用同一个会话编号；小伴拉起子进程时必须清掉这类变量 |

仍需真机验证：Windows 与 macOS 的同类行为、桌面版应用的快捷键与无障碍、终端里手动 `claude --resume` 看到完整历史。

## 5. 对计划的影响

1. **官方远程控制已出现**（Claude Code 的 Remote Control、Codex 的 remote-control），“手机遥控电脑上的编程 Agent”不再是稀缺能力，不能拿它当主卖点。
2. **仍然没被官方覆盖的差异**：
   - Claude 的 Remote Control **不支持 API 密钥、依赖订阅和 Anthropic 云端**。很多国内用户走第三方接口或网关，官方方案用不了；小伴自己的中转可以覆盖这部分。
   - 两家的语音仍是“听写”或自家应用内的实时对话，不是**跨 Agent、中文、主动播报、口头确认**的统一助手。
   - **一键装好环境、代配代理与兼容接口**不在官方的范围里。
3. **Codex 的适配器改用 app-server**，不再按会话文件设计观察层；Claude Code 用 hooks（优先 `http` 类型）+ stream-json。两边都不解析内部会话文件。
4. **语音确认的挂点明确了**：Claude 的 `PermissionRequest` hook / `--permission-prompt-tool`，Codex 的三类 `requestApproval` 服务端请求。
5. 应把 P3 的“外部 Agent 适配层”接口按两边的共同点设计：会话列表、开始一轮、追加引导、打断、权限请求与回答、事件流。

## 6. 还没核实的（需要真机或登录）

| 事项 | 为什么没核实 | 谁来做 |
| --- | --- | --- |
| Codex 真实对话的 app-server 事件样例（`turn/start` 之后的流、三类权限请求） | 沙箱里没有 Codex 登录 | 在你的电脑上登录后跑 `02-codex-appserver.mjs --chat` |
| Windows 上两个 CLI 的行为（路径、`.cmd` 垫片、PowerShell 与 hooks 的命令写法） | 沙箱是 Linux | 在 Windows 真机上验证 |
| Claude 桌面版、ChatGPT 桌面版：全局快捷键、快速输入、无障碍树是否暴露输入框、粘贴并回车是否可行 | 无法安装桌面应用 | 在 Windows 与 macOS 真机上验证，我写验证脚本和记录表 |
| Codex 的 `notify` 配置（载荷字段）、hooks 配置格式与信任机制 | 官方文档站被拦；程序里能看到 `hooks/list` 与 `hook/started`，但没有字段说明 | 读程序生成的配置模板与实测 |
| Codex remote-control 的账号与网络要求 | 同上 | 真机实测 |
| Happy 的加密协议与会话同步方式 | 只读了概述 | 需要时再读源码，注意许可 |
| 三项技术验证（hooks 回调、无界面恢复后终端接续、悬浮条送字进桌面版） | 需要登录与桌面环境 | 真机 |
