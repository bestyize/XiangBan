# 小伴桌面 Agent 加固：差距分析、已修复项与后续路线

**编制日期：** 2026-10-22
**分支：** `fix/agent-harness-hardening-20261022`、`feat/agent-harness-stage2-20261022`（均已合并到 main）、`feat/agent-harness-stage3-20261022`（第三批，均已合并到 main）、`feat/agent-harness-stage4-20261022`（第四批）
**范围：** `client/xiaoban-desktop-wallpaper` 的聊天 Agent（`src-tauri/src/chat/`、`src/ui/chat/`）与 `libs/agent-kernel`

本文记录一次对照 Claude Code、Codex、Hermes Agent 等 harness 的审查：发现的问题、本分支已经修复的部分、验证方式，以及还没做的部分和建议做法。标注 **[已修复]** 的条目有对应提交和测试；标注 **[待建]** 的条目尚未实现，不能对外宣称。

## 1. 与成熟 harness 的差距

| 维度 | 审查时的现状 | 成熟 harness 的做法 | 本分支之后 |
|---|---|---|---|
| 执行隔离 | 无沙箱；“完全信任”只靠关键词规则（`danger.rs`）把关 | Codex 用系统沙箱（Seatbelt / Landlock）限制写入和网络；Claude Code 有沙箱模式；Hermes 可把终端放进 Docker/SSH | 规则已加固 **[已修复]**；macOS 沙箱 **[已实现，待真机验证]**；Windows 低完整性沙箱 **[实验性，默认关闭]** |
| 权限粒度 | 全局一个模式；执行类工具“总是允许”不可用 | 按会话；按命令前缀记住授权 | 按会话 **[已修复]**；按命令种类记住 **[已修复]** |
| 上下文压缩 | 只在一轮回答结束后压缩 | 回答进行中也会自动压缩 | **[已修复]** |
| Prompt 缓存 | Claude 协议未发 `cache_control` | 缓存系统提示、工具和历史 | **[已修复]** |
| 后台进程 | 内核有 `ProcessManager`，桌面端未接入（运行文档写着已接入） | 后台 shell、轮询输出 | **[已修复]** |
| 编辑回滚 | 无 | Claude Code 检查点；Codex 基于 git 撤销 | 整轮撤销 **[已实现]**：文件工具改动全部覆盖；git 仓库内命令的改动也能恢复 |
| 子 Agent | 内核有 `Subagent`，桌面端未用 | 常规能力 | 只读调研子 Agent **[已实现]** |
| 网页搜索 | 只有 `web_fetch` | 自带搜索 | **[待建]**（需搜索后端） |
| 项目指令与 hooks | 只有 skills 与记忆 | 读取 AGENTS.md / CLAUDE.md，工具前后 hooks | 项目指令、hooks **[已实现]**，可在设置中编辑 |
| Claude 思考模式 | thinking 块不在轮次间保留，带工具时不能开 | 支持 | **[已实现]**，前缀改变时不回传 |

## 2. 已修复的问题

### 2.1 安全

1. **权限模式全局共享，多个对话互相串改**（`db9d6ed`）。所有对话共用一个 `Permission`，`chat_load_session` 只是打开查看一个对话就会改写它。“询问”模式的对话正在运行时，切去查看一个“完全信任”的对话，前者后续的写入和联网就不再询问。现在 `Permission` 支持按会话设置模式（`set_session_mode` / `mode_for`），桌面端只在 `chat_send` 时为本对话设置。
2. **危险命令检测可绕过**（`7287b01`）。实测以下命令此前都不需要确认，现在都会询问：`cd / && rm -rf *`、`ln -s / root && rm -rf root/`、工作目录内指向外部的符号链接、`base64 -d | sh`、`$(echo rm) -rf`、`$CMD -rf /`、`git -c alias.x='push --force' x`、`python -c` / `node -e` 等内联脚本里的删除或执行、`> ~/.bashrc`、`mv … /dev/null`，以及 gh、git-credentials、浏览器登录数据等凭据路径。日常命令保持不问。它仍是启发式规则，不是沙箱，见 4.1。
3. **对话命令的窗口检查**（`89f208c`）。在 16 个对话命令内部加了窗口检查，只接受 `control` 和 `assistant`。**更正：** 最初审查时认为“任何窗口都能调用对话命令”，这是误判。`main.rs` 的命令分发处早已有按窗口标签的白名单（`content::permitted`、`float_assistant::permitted` 等），`chat_*` 本来只开放给控制台，超级小伴只能用其中 8 个。命令内部的检查只是多一层防护；之后新增的撤销、安全设置等命令同样受这层白名单约束，已有回归测试固定。

### 2.2 稳定性

4. **`panic = "abort"` 让任何 panic 都关掉整个应用**（`eae2630`）。改为 unwind，并把宿主中约 400 处 `lock().unwrap()` 换成容忍锁中毒的写法，一次 panic 不再连锁拖垮其他命令。
5. **工具超时后仍可能在后台执行**（`b59f2e6`）。超时结果现在提示“动作可能已经发生”，要求模型先检查实际状态再重试，避免副作用执行两次。
6. **确认或回答发送失败时卡片消失，任务卡住**（`b0386d3`）。失败时把卡片放回原处，可以再次作答。

### 2.3 Windows / macOS 可用性

7. **Windows 上 shell 命令里的双引号被转坏**（`2c068ed`）。改用 `cmd /D /S /C` 加 `raw_arg`；shell 不再为每条命令弹出控制台窗口。
8. **子进程环境缺少必要变量**（`2c068ed`）。Windows 补上 `APPDATA`、`LOCALAPPDATA`、`ProgramFiles` 等（仍不含任何密钥）；MCP 的裸命令按 `PATH` + `PATHEXT` 解析，标准 `npx` 配置能找到 `npx.cmd`。macOS 从程序坞启动时读取一次登录 shell 的 `PATH`（3 秒超时），并补 Homebrew 等常见目录。三处重复的环境白名单合并为 `agent-kernel` 的 `child_env` 模块。
9. **CRLF 文件无法做多行编辑**（`18dba87`）。`read_file` 展示的行不带 `\r`，整份文件是 CRLF 时，`edit_file` 现在按 LF 匹配并写回 CRLF。

### 2.4 新能力

10. **Prompt 缓存**（`9221cd3`）：`ProviderProfile.prompt_cache`，在工具末尾、系统提示和最新消息上打 `cache_control`；桌面端在 anthropic 协议下开启。
11. **后台进程工具**（`ddcc7bd`）：`process_start/read/write/stop`，按工作目录缓存管理器；删除对话、切换工作目录、退出应用时清理。`process_start` 共用危险命令检查。
12. **回答进行中的自动压缩**（`8c9e8ef`）：`MidRunCompact` 中间件在请求达到上限 85% 时摘要较早部分，保留最新约 35% 的工作、系统提示和用户问题；只影响发给模型的请求。
13. **按命令种类记住授权**（`443ea02`）：批准 `npm test` 时可选“这段对话里 npm test 都允许”，不会放行 `npm publish`。带分隔符、管道、重定向、替换、环境变量前缀的命令，以及解释器、删除/移动/联网/提权类程序，每次都问。
14. **撤销一轮回答的文件改动**（`17541f3`）：文件工具第一次改动某个文件前备份原文件；回答结束后显示“这轮回答修改了 N 个文件”，两步确认后恢复。每个对话保留最近 20 轮。只覆盖文件工具，shell 和后台进程改动的文件不在其中。
15. 桌宠与进程工具补上中文名称（`6d642e9`）。

### 2.5 第二批（`feat/agent-harness-stage2-20261022`）

16. **项目说明**：工作目录根下的 `AGENTS.md`、`CLAUDE.md`、`.claude/CLAUDE.md` 加入系统提示，每次调用模型前重新读取；单个文件最多 16 KB、合计 24 KB；不跟随符号链接。提示中写明这些内容只是工作约定，不能扩大权限或改变确认流程。
17. **只读调研子 Agent**（`research` 工具）：子助手只有 `read_file` / `list_dir` / `grep`，只把结论带回对话；不会弹确认框，可以并行。
18. **macOS 沙箱**：“完全信任”下的 shell 与后台进程运行在 Seatbelt 里，只能写工作目录、临时目录和软件包缓存，读不到 `~/.ssh` 等密钥目录；网络默认开放。要求沙箱但系统不支持时拒绝执行，不会退回无保护运行。用户明确允许的高风险命令在沙箱外执行一次。询问模式和 Windows 行为不变。
19. **GBK/GB18030 文件**：`read_file`、`grep` 能正确显示和搜索；`edit_file` 编辑后按原编码写回。

### 2.6 第三批（`feat/agent-harness-stage3-20261022`）

20. **Claude 扩展思考**：内核解析带签名的 `thinking` / `redacted_thinking` 块，实时显示推理摘要，并在后续请求中原样回传。回传只在请求前缀（系统提示、工具与历史）与产生它时完全一致时进行：Claude 的“保留思考”会校验前缀，而上下文裁剪和回答中途压缩都会改历史，无条件回传会让 2026-08-31 之后创建的账号收到 400。桌面端对支持 adaptive thinking 的 Claude 模型发送 `thinking: {type: "adaptive", display: "summarized"}`，选了等级时加 `output_config.effort`。已确认请求不发送 2026-10-21 起会被拒绝的 `temperature` / `top_p` / `top_k`。
21. **工具 hooks**：`chat/hooks.json` 配置调用前后的命令，调用前的 hook 失败即拦截，调用后的 hook 把问题反馈给模型。用法见桌面端 README“工具 hooks”。
22. **子任务运行记录**：research 子 Agent 的运行写入父对话的执行日志，带 `parent_run_id`。

### 2.7 第四批（`feat/agent-harness-stage4-20261022`）

23. **设置“安全与权限”**：沙箱联网开关（macOS）、hooks 编辑与校验、已记住的授权列表与撤销。“这段对话都允许”现在保存到 `chat/agent-settings.json`，重启后仍有效（内核 `Permission::grant` / `revoke`）。
24. **电脑控制引擎复用**：computer-use MCP 进程跨消息复用，连接断开才重启。
25. **git 检查点**：工作目录在 git 仓库时，一轮回答第一次运行命令前，用临时索引做工作文件快照。撤销时恢复命令的改动，不碰用户的索引、分支和 stash；被忽略的文件不在其中。
26. **命令的窗口白名单**：确认 `main.rs` 已有按窗口过滤（见 2.1 第 3 条的更正），补回归测试。
27. **每轮用量**：回答下方显示这一轮的输入与输出 token，含调研子任务。
28. **Windows 低完整性沙箱（实验性，默认关闭）**：宿主可执行文件充当启动器，用低完整性令牌启动 `cmd`；开启时把工作目录标记为低完整性。只限制写入，不限制读取与联网。已在 Linux 上对 Windows 目标做类型检查，尚未在真机运行。

### 2.8 第五批（`feat/agent-scheduled-tasks-20261022`）

29. **上下文圆环的工具开销**：第一次回答前按实测值估算（上次启动保存在 `chat/tool-tokens.json`），没有实测值时用接近真实大小的默认值，测试保证默认值不偏离太多。
30. **多平台测试工作流**：`.github/workflows/desktop-agent-platform-tests.yml` 只能手动运行，在 Windows 与 macOS 上跑内核和桌面端测试，其中 `tests/os_sandbox.rs` 检查 Seatbelt 沙箱、Windows 低完整性沙箱和 cmd 引号。顺带去掉 `agent-chat-tests.yml` 的自动触发。
31. **定时 Agent 任务**：关怀提醒新增“定时任务”类型。到点后小伴只用只读工具执行（读文件、列目录、搜索、时间、网页），没有需要确认的操作；每条规则的运行追加到它自己的会话，气泡显示答案开头，结果最多保留 12 小时。
32. **精简创作类工具定义**：游戏、小组件、特效的长篇规范移到 `game_read_format`、`widget_read_format`、`effect_read_format`，创作时才读取。工具列表本身固定，不影响提示缓存。内置工具定义由约 9,600 token 降到约 7,900；测试限制单个工具定义不超过 600 token。

### 2.9 第六批（`feat/agent-cleanup-20261022`）

33. **已有失败测试清零**：内核 5 个、桌面端 1 个、前端 7 个，vue-tsc 13 个类型错误。其中两处是真实问题：Debian/Ubuntu 上 `sh` 是 dash，`kill -KILL -- -pid` 会失败，命令留下的后台进程一直没被清理（改用 `killpg`）；“max_tokens 必须大于 0”被误判为超过输出上限。
34. **AI 创建定时任务**：`care_set` 支持 `kind=agent`，添加或修改都要用户确认，完全信任模式也一样。
35. **定时任务上次运行**：关怀页显示每条定时任务上次运行的时间、是否完成、token 用量和累计次数。
36. **小组件保存前检查**：`-webkit-app-region`、直接调用 Tauri、给 html/body 画不透明背景时，说明原因并不保存。
37. **子任务计入调用预算**：内核 `ToolBudget` 由一轮回答和它启动的子 Agent 共享。
38. **非 git 目录的命令撤销**：私有快照仓库放在 `chat/snapshots`，工作目录里不加 `.git`，依赖与构建输出不纳入。
39. **中断的回答**：打开对话时，若最近一轮没有正常结束，提示一次，并列出可能已经执行的工具。
40. **过宽的工作目录**：工作目录是主目录或磁盘根目录时，其中的删除不再视为“在工作目录内”。

合并 #93（长期目标）时，顺带修正了 main 上的三处编译错误：`File::try_lock` 的错误类型、`pub use` 重新导出 Tauri 命令、`app.state()` 临时值的借用。

## 3. 验证情况

- `libs/agent-kernel`：`cargo test --all-features` 312 个测试全部通过；`cargo check` 对 `x86_64-pc-windows-msvc` 与 `aarch64-apple-darwin` 都通过。
- 桌面端 Rust：在 Linux 上编译需要给 `startup.rs` 加临时桩（应用只支持 Windows/macOS），桩没有提交。466 个测试全部通过；`cargo check --target x86_64-pc-windows-gnu` 通过。
- 前端：95 个文件 875 个测试全部通过，`vue-tsc` 没有类型错误。
- **尚未在真机验证**：Windows 的引号与环境变量、macOS 的 PATH 读取、`npx` 启动 MCP 连接器、Claude 端点接受 `cache_control`。建议用手动触发的 Actions（仅 `workflow_dispatch`）或真机冒烟。

在受限网络里装前端依赖时，`package-lock.json` 指向 `registry.npmmirror.com`，若该源不可达，可临时改走官方源而不改 lock 文件：

```sh
npm ci --registry=https://registry.npmjs.org/ --replace-registry-host=registry.npmmirror.com
```

## 4. 后续路线

### 4.1 系统沙箱：macOS 已实现，Windows 实验性

关键词规则挡不住 `python script.py` 这类由模型先写脚本再执行的路径，“完全信任”需要真正的隔离作为前提：

- macOS：已实现（见 2.5 第 18 条），联网开关已在设置中提供，尚需真机验证。
- Windows：低完整性方案已实现并默认关闭（见 2.7 第 28 条）。真机验证通过后再考虑默认开启；若要限制读取，需要改用 AppContainer。
- 沙箱不可用（系统版本过低、启动失败）时，“完全信任”自动降级为“询问”并在界面说明。
- 验收：沙箱内 `rm -rf ~`、写 `~/.zshrc`、`curl -d` 均失败；`npm test`、`cargo build` 正常。

### 4.2 命令权限的进一步收敛 [已完成]

- 按窗口限定可调用的命令：`main.rs` 的命令分发处按窗口标签过滤（见 2.7 第 26 条），有回归测试。
- 记住的授权持久化，并在设置里列出、撤销（见 2.7 第 23 条）。

### 4.3 能力补齐

- **子 Agent** [已完成]：子任务的工具调用计入父任务的调用预算（见 2.9 第 37 条）。
- **检查点的覆盖范围** [已完成]：不在 git 仓库的工作目录用私有仓库做快照（见 2.9 第 38 条）。超过 2 万个文件或 512 MB 的目录、用户主目录和磁盘根目录仍不覆盖。
- **网页搜索** [待定]：需要选定方案。现有的搜索是 xiaban-api 后台配置的 SearXNG；走 server-ai 网关要新增带鉴权、限流和计费的搜索接口并部署，桌面端再接 `SearchBackend`。
- **Agent 复用**：电脑控制进程已复用；Agent 本身仍按消息重建（开销小，换来配置与工作目录变化即时生效）。

### 4.4 边界测试

已补：

- 两个对话同时等待确认，其中一个在等待时切换权限模式：各自收到自己的答复，模式只影响自己（内核 `tests/permission.rs`）。
- 回答中途程序退出后重开对话：提示这轮中断，并列出已经开始执行、可能已生效的工具（见 2.9 第 39 条）。
- 磁盘写满：保存不了问题时不发给模型；工具调用记录不下来时不执行该工具，已保存的内容仍可读（`tests/store_jsonl.rs`）。检查点写入失败不影响任务。
- 工作目录是用户主目录、`/Users`、磁盘根目录时，`rm -rf *` 等删除一律需要确认（见 2.9 第 40 条）。
- 路径含空格和中文：文件工具与命令在这样的工作目录里都正常（`tests/fs_write.rs`、`tests/shell.rs`，Windows 在 `tests/os_sandbox.rs`）。
- 模型 key 失效：报错转成“认证失败，请修改配置或切换模型”（`model-errors.ts`）。
- 任务进行中修改工作目录或模型：Agent 按消息构建，进行中的回答不受影响，下一条消息生效。

仍需真机验证：Windows 长路径与网络盘；系统休眠唤醒后进行中的流式请求与确认框。
