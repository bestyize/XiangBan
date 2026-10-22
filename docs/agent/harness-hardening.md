# 小伴桌面 Agent 加固：差距分析、已修复项与后续路线

**编制日期：** 2026-10-22
**分支：** `fix/agent-harness-hardening-20261022`（已合并到 main）、`feat/agent-harness-stage2-20261022`（第二批）
**范围：** `client/xiaoban-desktop-wallpaper` 的聊天 Agent（`src-tauri/src/chat/`、`src/ui/chat/`）与 `libs/agent-kernel`

本文记录一次对照 Claude Code、Codex、Hermes Agent 等 harness 的审查：发现的问题、本分支已经修复的部分、验证方式，以及还没做的部分和建议做法。标注 **[已修复]** 的条目有对应提交和测试；标注 **[待建]** 的条目尚未实现，不能对外宣称。

## 1. 与成熟 harness 的差距

| 维度 | 审查时的现状 | 成熟 harness 的做法 | 本分支之后 |
|---|---|---|---|
| 执行隔离 | 无沙箱；“完全信任”只靠关键词规则（`danger.rs`）把关 | Codex 用系统沙箱（Seatbelt / Landlock）限制写入和网络；Claude Code 有沙箱模式；Hermes 可把终端放进 Docker/SSH | 规则已加固 **[已修复]**；macOS 沙箱 **[已实现，待真机验证]**；Windows **[待建]** |
| 权限粒度 | 全局一个模式；执行类工具“总是允许”不可用 | 按会话；按命令前缀记住授权 | 按会话 **[已修复]**；按命令种类记住 **[已修复]** |
| 上下文压缩 | 只在一轮回答结束后压缩 | 回答进行中也会自动压缩 | **[已修复]** |
| Prompt 缓存 | Claude 协议未发 `cache_control` | 缓存系统提示、工具和历史 | **[已修复]** |
| 后台进程 | 内核有 `ProcessManager`，桌面端未接入（运行文档写着已接入） | 后台 shell、轮询输出 | **[已修复]** |
| 编辑回滚 | 无 | Claude Code 检查点；Codex 基于 git 撤销 | 文件工具改动可整轮撤销 **[已修复]** |
| 子 Agent | 内核有 `Subagent`，桌面端未用 | 常规能力 | 只读调研子 Agent **[已实现]** |
| 网页搜索 | 只有 `web_fetch` | 自带搜索 | **[待建]**（需搜索后端） |
| 项目指令与 hooks | 只有 skills 与记忆 | 读取 AGENTS.md / CLAUDE.md，工具前后 hooks | 项目指令 **[已实现]**；hooks **[待建]** |
| Claude 思考模式 | thinking 块不在轮次间保留，带工具时不能开 | 支持 | **[待建]** |

## 2. 已修复的问题

### 2.1 安全

1. **权限模式全局共享，多个对话互相串改**（`db9d6ed`）。所有对话共用一个 `Permission`，`chat_load_session` 只是打开查看一个对话就会改写它。“询问”模式的对话正在运行时，切去查看一个“完全信任”的对话，前者后续的写入和联网就不再询问。现在 `Permission` 支持按会话设置模式（`set_session_mode` / `mode_for`），桌面端只在 `chat_send` 时为本对话设置。
2. **危险命令检测可绕过**（`7287b01`）。实测以下命令此前都不需要确认，现在都会询问：`cd / && rm -rf *`、`ln -s / root && rm -rf root/`、工作目录内指向外部的符号链接、`base64 -d | sh`、`$(echo rm) -rf`、`$CMD -rf /`、`git -c alias.x='push --force' x`、`python -c` / `node -e` 等内联脚本里的删除或执行、`> ~/.bashrc`、`mv … /dev/null`，以及 gh、git-credentials、浏览器登录数据等凭据路径。日常命令保持不问。它仍是启发式规则，不是沙箱，见 4.1。
3. **任何窗口都能调用对话命令**（`89f208c`）。Tauri 应用命令默认对所有窗口开放，而壁纸、组件、宠物、游戏窗口会加载外部或由助手生成的内容。16 个此前没有窗口检查的对话命令（发消息、切换完全信任、批准确认等）现在只接受 `control` 和 `assistant` 窗口。

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

合并 #93（长期目标）时，顺带修正了 main 上的三处编译错误：`File::try_lock` 的错误类型、`pub use` 重新导出 Tauri 命令、`app.state()` 临时值的借用。

## 3. 验证情况

- `libs/agent-kernel`：`cargo test --all-features` 新增测试全部通过。以下 5 个测试在 main 上本来就失败，本分支没有新增失败：`output_limit_errors_reduce_only_the_generation_cap`、`a_skill_created_during_a_run_appears_in_the_next_model_call`、`load_returns_the_body_without_frontmatter_and_names_alternatives_on_miss`，以及 shell 的两个进程清理测试。`cargo check` 对 `x86_64-pc-windows-msvc` 与 `aarch64-apple-darwin` 都通过。
- 桌面端 Rust：在 Linux 上编译需要给 `startup.rs` 加临时桩（应用只支持 Windows/macOS），桩没有提交。聊天模块 143 个测试通过；全量唯一失败是 `ai_reasoning::tests::explicit_alias_capabilities_can_enable_disable_or_restrict_levels`，本分支未改动该模块。
- 前端：`src/ui/chat` 与 `ChatPage` 共 268 个测试通过。全量里 ControlShell、AiConfigCard、assistant-page 的 5 个失败在 main 上同样存在；`pet.test.ts` 的动画测试在 main 上也时好时坏。`vue-tsc` 的 13 个类型错误在 main 上同样存在。
- **尚未在真机验证**：Windows 的引号与环境变量、macOS 的 PATH 读取、`npx` 启动 MCP 连接器、Claude 端点接受 `cache_control`。建议用手动触发的 Actions（仅 `workflow_dispatch`）或真机冒烟。

在受限网络里装前端依赖时，`package-lock.json` 指向 `registry.npmmirror.com`，若该源不可达，可临时改走官方源而不改 lock 文件：

```sh
npm ci --registry=https://registry.npmjs.org/ --replace-registry-host=registry.npmmirror.com
```

## 4. 后续路线

### 4.1 系统沙箱：macOS 已实现，Windows [待建]

关键词规则挡不住 `python script.py` 这类由模型先写脚本再执行的路径，“完全信任”需要真正的隔离作为前提：

- macOS：已实现（见 2.5 第 18 条），尚需真机验证；后续可在设置里提供“完全信任时关闭网络”的选项（`SandboxPolicy.network`）。
- Windows：受限令牌或 AppContainer 加现有的 Job Object；先只限制写入范围。
- 沙箱不可用（系统版本过低、启动失败）时，“完全信任”自动降级为“询问”并在界面说明。
- 验收：沙箱内 `rm -rf ~`、写 `~/.zshrc`、`curl -d` 均失败；`npm test`、`cargo build` 正常。

### 4.2 命令权限的进一步收敛 [待建]

- 用 Tauri AppManifest 按窗口声明可用命令，替代当前在命令里检查窗口标签。需要列出每个窗口实际调用的命令并做真机回归。
- 记住的命令种类目前只在内存里、按会话保存；需要时可持久化，并在设置里列出与撤销。

### 4.3 能力补齐 [待建]

- **子 Agent**：只读调研已实现；子任务的用量尚未计入父任务的预算。
- **网页搜索**：`web_search` 需要宿主提供 `SearchBackend`，建议走 server-ai 网关统一计费与限流。
- **hooks**：工具调用前后执行用户配置的命令（类似 Claude Code 的 hooks）。
- **Claude 思考模式**：内核保留并回传 thinking 块后，才能在带工具的循环里开启扩展思考。
- **Agent 复用**：现在每条消息都会重建 Agent；开启电脑控制时，每条消息还会重启一次 MCP 子进程。可改为按配置缓存，配置变化时再重建。
- **检查点的覆盖范围**：可在工作目录是 git 仓库时，额外用 `git stash create` 记录 shell 改动。

### 4.4 需要补的边界测试

- 两个对话同时运行、各自弹出确认，再分别切换权限模式。
- 任务进行中修改设置里的工作目录或模型，以及模型 key 过期。
- 回答中途杀掉进程后重启：执行日志里有未完成的工具时，提示“动作可能已发生”。
- 磁盘写满时的会话文件、索引与检查点写入。
- 工作目录设为用户主目录或磁盘根目录时的危险命令判断。
- 路径含空格、中文，Windows 长路径，网络盘。
- 系统休眠唤醒后，进行中的流式请求与确认框。
