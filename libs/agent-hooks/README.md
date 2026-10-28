# xiaoban-agent-hooks

独立的 Claude Code hooks 配置规划器、本机接收端及静默转发程序。**当前为实验实现，真实 Claude 集成和跨平台行为仍待验证，不应直接安装到生产用户配置。**

## 用法

构建 `cargo build --release` 后，提供绝对路径 `xiaoban-hook`（Windows 为 `xiaoban-hook.exe`）和作用域：

```rust,no_run
use xiaoban_agent_hooks::claude_settings::{Scope, InstallSpec, plan_install, apply};
use std::path::PathBuf;
let scope = Scope::Project(PathBuf::from("/path/to/project"));
let plan = plan_install(&scope, &InstallSpec::new(PathBuf::from("/path/to/xiaoban-hook")))?;
println!("before: {}\nafter: {}", plan.before, plan.after); // 用户审阅后
apply(&plan)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

卸载使用 `plan_uninstall` 和 `apply`；`status` 检查现有处理器与路径，`restore_latest_backup` 恢复最近备份。`cargo run --example demo` 启动接收端并打印事件；真实应用应该保管端点令牌、订阅事件、在收到 `PermissionRequest` 时由用户决定后调用 `respond`，然后在退出时调用 `shutdown`。演示会打印事件内容，避免在敏感终端运行。

默认事件：PermissionRequest、Notification、Stop、StopFailure、SubagentStop、UserPromptSubmit、SessionStart、SessionEnd。权限请求的接收端/转发/Claude 超时分别为 120/125/130 秒；其他设置默认 30 秒、转发总期限 2 秒。`queue_instruction` 按会话排队，只有 `stop_hook_active=false` 的 Stop 才取走一条。权限默认无意见，不自动允许。

## 所有权和文件

仅将 `type=command`、文件名去扩展名为 `xiaoban-hook` 且 `args` 第一项为字符串的处理器视为本项目所有；安装追加独立匹配组，并允许更新自己的路径；卸载只删除符合规则的处理器。安装创建的匹配组位置及容器归属保存在 `settings.json.xiaoban-state`。如果用户在安装后移动或修改这些结构，卸载会保守保留无法确认的空结构。若存在语义上与卸载结果相同的备份，则采用该备份的原字节以恢复格式；否则按原缩进和键顺序格式化。修改前生成同目录 UTC 时间戳备份，保留最近五份；计划过期则拒绝应用。JSONC 和非法 JSON 不会修改。

不存在的设置文件会新建。卸载不删除设置文件；外部修改和备份轮换可能使原字节恢复不可用。配置文件重写使用同目录临时文件、刷盘、原子替换与有限重试；不改变真实配置的测试必须指定临时项目目录或 `CLAUDE_CONFIG_DIR`。

端点默认在 Windows `%APPDATA%/xiaoban`、macOS `~/Library/Application Support/xiaoban`、Linux `$XDG_DATA_HOME/xiaoban` 或 `~/.local/share/xiaoban`，也可用 `XIAOBAN_HOOK_ENDPOINT` 指向测试文件。Unix 文件权限为 0600；Windows 在写入令牌前为临时文件移除继承 ACL，只授予当前用户 SID 完全控制，ACL 设置失败则拒绝启动。Windows ACL 还需要在多用户、不同文件系统和受限策略环境验证。 `shutdown` 只删除仍属于自己的端点文件；异常退出可能留下过期文件。只支持同时有一个活动端点文件的服务实例。

## 文档核查和版本

[Claude Code hooks 文档](https://code.claude.com/docs/en/hooks)确认 `args` 为直接进程调用，通知类型字段为 `notification_type`，PermissionRequest 支持 allow/deny，Stop 支持 block/reason。权限提示也可能在无交互环境被 Claude 自行拒绝；网络沙箱提示不一定触发 PermissionRequest。任务单称 HTTP 类型收不到 SessionStart 是给定的实测观察，官方文档并未在所查页面保证此行为，本项目没有复现；这里统一使用 command。任务单的字段示例仅覆盖部分事件，未知字段保存在 `extra`。本机检查到 Claude Code 2.1.285；没有启动或登录真实 Claude 测试，因此不声称验证了该版本的行为。

## Codex：待确认，不安装

本机 Codex CLI 0.159.2 的 `features list` 显示 hooks 为 stable；临时目录生成的 app-server schema 中，`hooks/list` 接收可选 `cwds` 并返回 `data`，`hook/started` 和 `hook/completed` 含 `threadId`、可空 `turnId`、`run`；信任状态包括 managed、untrusted、trusted、modified。[OpenAI 插件文档](https://developers.openai.com/plugins/build/plugins)说明插件 hooks 需单独审阅和信任。用户配置位置为 `~/.codex/config.toml`，`CODEX_HOME` 可改变根目录。

待确认：当前版本用户级和项目级 hooks 的完整配置键和合并语义、`notify` 的参数和确切触发时机、每种事件与输入输出的可用性、hook 信任批准与更新流程、Codex 载荷映射及跨平台命令行为。由于尚未完整核实，`codex::plan_install_codex` 返回 Unsupported，不写任何 Codex 配置。

## 依赖与验证限制

直接依赖：serde、serde_json、tokio、axum、hyper、hyper-util、rand、chrono、thiserror、tempfile；均可选 MIT 或 Apache-2.0 许可（其中部分仅 MIT）；测试依赖 tempfile、tracing、tracing-subscriber。任务允许 Unicode-3.0：传递依赖 `unicode-ident` 使用 `(MIT OR Apache-2.0) AND Unicode-3.0`；其他目标平台的完整传递依赖尚未审计。

自动测试覆盖临时配置的共存、格式、路径更新、幂等、备份、还原、非法 JSON、卸载与 MCP 夹具；覆盖令牌、Host、请求体与读取超时、权限挂起及三种决定、并发会话、Stop 队列、事件广播、未知字段、全 TRACE 日志不含令牌或正文，以及转发的拒绝连接、无响应、输入上限和阻塞输入。Windows 测试检查端点 ACL 无继承项和存在完全控制项；Unix 测试检查 0600，但本次仅在 Windows 运行。`detect_coexisting` 仍只扫描用户 settings 和 `~/.claude.json` 的 JSON 文本，不能覆盖所有 MCP 注册来源，可能误报。

Windows 本机 release 版、端点文件缺失时的 20 次进程启动实测：第一次 66.83 ms，中位数 13.79 ms，范围 12.21–66.83 ms（含创建进程）。第一次没有达到小于 20 ms 的冷启动目标；数据不是跨设备保证。

### 真实 Claude Code 测试（默认未运行）

`tests/real_claude.rs` 是忽略的可选测试。需要可执行的 Claude Code、允许网络和已配置的 `ANTHROPIC_API_KEY`。显式设置 `XIAOBAN_REAL_CLAUDE=1` 并运行 `cargo test --test real_claude -- --ignored --nocapture`；如 `claude` 不在 PATH，可指定 `XIAOBAN_CLAUDE_BIN`。这会调用真实模型并可能产生费用。测试使用临时 `HOME`、`USERPROFILE`、`CLAUDE_CONFIG_DIR`、项目和端点，不写入真实用户配置；仅通过显式 API key 认证，不复用 OAuth 登录。

真实环境仍待验证：临时设置是否被该 Claude 版本加载；权限请求、批准后首个文件创建；首次 Stop 是否注入排队指令，以及继续执行后第二个文件和第二次 Stop；Windows/macOS/Linux、终端与桌面会话行为。自动化尚未覆盖 Windows 多用户隔离及受限文件系统、不同平台的冷启动、真实日志环境和 Claude Desktop。
