# xiaoban-agent-bridge

状态（2026-10-28）：**Claude Code 路径已开放，Codex 路径仍关闭。**

- **Claude Code**：`start` / `resume` 可用，固定走最严格档（`--permission-mode default`，只放行 `Read,Glob,Grep`，其余工具在 `PermissionRequest` hook 上暂停，等人批准）。依据：真实 Claude Code 2.1.x 上的两项验证——① 批准前命令不执行、批准后执行（`XIAOBAN_REAL_CLAUDE=1 cargo test --lib real_claude`，另有经 `DelegationManager` 的同款测试）；② hook 服务不可达时工具被拒绝而不是放行（用桥接用的同一组启动参数实测）。**已知边界**：用户自己的 Claude 设置（用户级或项目级 `settings.json`）里若写了 `permissions.allow` 的规则（例如 `Bash(*)`），这些规则会先于 hook 生效，委派任务会按用户自己的规则放行；桥接不去改写用户的设置来源，因为那会同时丢掉用户在设置里配置的接口地址等环境变量。
- **委派（`DelegationManager`）**：Claude 任务走上面的桥接，审批出现在小伴界面和已配对的手机上；任务第一轮结束即完成。Codex 任务仍用 `codex exec --sandbox read-only`，只读，不能审批、不能写——**这条命令行参数未在真机核实**，参数不对时任务会失败并显示原因，不会放行。
- **Codex 的 `start` / `resume`**（app-server 路径）仍返回 `Unsupported`：`untrusted` 是否对每条命令和每次写入请求审批尚未证明。

边界：只启动用户本机安装的官方 Codex CLI / Claude Code；不读取、转存或借用它们的登录令牌、凭据文件，不重新分发它们的程序；外部 Agent 的完整会话由其自身保存，库不解析内部会话文件。不提供全自动档位，不自行提升权限。

当前可用：`availability` 查询程序版本，Codex `list_sessions` 通过 app-server 列表接口查询，Claude 列表仅为本库当前进程里记录的会话（由于启动入口停用，目前为空）。`cargo run --example probe` 仅查询版本和会话列表，不发起对话。在 Windows 通过 PATH 选择 `.exe`、`.cmd` 或 `.bat` 启动器。调用方可使用 `Codex::new(path)`、`ClaudeCode::new(path)` 指定路径，例如测试替身。

## 协议与权限（待实现及验证）

从本机 **codex-cli 0.159.2** 的 `codex app-server generate-json-schema` 导出并读取：逐行 JSON；`initialize` 后发送 `initialized`；`thread/start` 返回的会话 ID 位于 `result.thread.id`；`turn/start` 必填 `threadId`、`input`，文本输入为 `{"type":"text","text":"..."}`。命令和文件审批 `decision` 可取 `accept`、`acceptForSession`、`decline`、`cancel`；权限审批响应的必填字段为 `permissions`，可选 `scope` 取 `turn` 或 `session`；工具问答的 `answers` 按问题 ID 映射到 `{"answers":["..."]}`。更多版本兼容、旧协议审批、MCP elicitation 和多问题回答仍待确认。

原型中拟将 `Strict` 映射为 `approvalPolicy: untrusted`、`sandbox: read-only`，将 `AskForWrites` 映射为 `untrusted`、`workspace-write`。**这只是候选映射，不能证明题目要求的逐次审批，因此未启用。** Claude 候选模式为 `default`，只预允许 `Read,Glob,Grep`，并尝试使用仅本次传入的 HTTP `PreToolUse`、`PermissionRequest` 和 `Stop` hooks；其安全性、版本兼容性和 HTTP hook 故障语义未验证，同样未启用。原型中的 Claude 中断会终止并回收整个子进程，而不是保留进程继续会话。调用方不得把风险分类当成安全保证。

目标协议基于任务单所述的 Claude Code 2.1.312，环境中为 2.1.285。实现尚未使用真实 Claude 对话验证；hook 地址只监听 127.0.0.1，令牌随机生成，服务不记录请求正文。尚缺全面的异常、隔离和端到端测试，生产使用前必须补齐。错误类型不包含底层进程原始输出。

## 依赖许可与验证

直接依赖 `async-trait`、`serde`、`serde_json`、`thiserror`、`rand` 为 MIT / Apache-2.0 双许可；`tokio` 为 MIT。Windows 构建的传递依赖中 `zerocopy` 可选 BSD-2-Clause、Apache-2.0 或 MIT，`memchr` 可选 MIT，其余多数为 MIT 或 Apache-2.0；`unicode-ident` 的许可证为 `(MIT OR Apache-2.0) AND Unicode-3.0`；任务单的许可限制已放宽以允许 Unicode-3.0。跨平台传递依赖仍应按实际目标平台复核。

```sh
cargo test
cargo clippy --all-targets -- -D warnings
cargo run --example probe
```

测试使用假 Codex 和假 Claude 进程，覆盖会话事件映射、允许/拒绝、追加指令、恢复、中断、坏 JSON、异常退出、请求超时、会话隔离、环境变量清除、hook 令牌、请求体上限及未批准超时。hook 测试需要操作系统允许测试进程通过 127.0.0.1 通信；某些沙箱会拒绝本机连接。非回环接口连接测试仅在机器具备可用非回环 IPv4 接口时执行该连接探测。测试构建使用仅在 `cfg(test)` 下启用的内部路径覆盖停用保护，普通库构建不会解除保护。

真实 Claude Code 对话集成测试：未运行。本机发现 2.1.285，但没有设置任务单指定的 `XIAOBAN_REAL_CLAUDE=1`，普通库构建的真实会话入口仍未开放；测试构建包含受 `XIAOBAN_REAL_CLAUDE=1` 控制的真实测试，只有找到 CLI 时才尝试运行，并且只批准测试目录中精确的 `touch bridge-permission-test.txt` 命令。其他请求会拒绝。待验证：真实版本的 HTTP `PreToolUse` / `PermissionRequest` / `Stop` 请求与响应、hook 被禁用或故障时的失败行为、严格权限对全部工具的约束、stop 追加指令、终止与恢复、订阅登录、真实命令的批准及拒绝。真实 Codex 的权限和启动流程也尚未通过端到端验证。

