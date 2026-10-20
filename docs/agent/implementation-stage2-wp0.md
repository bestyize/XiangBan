# 第二阶段 WP0：桌面执行轨迹接线

> 代码阶段性变更记录：当前只完成了技术 Run 记录的接入，没有完成用户任务结果验证。`RunState::Completed` 代表 Agent 循环正常结束，不能解释为业务任务成功。

## 已实现

- 桌面聊天改用 Kernel 现有的 `run_parts_observed` / `retry_last_observed`，无需新建第二套执行引擎。
- 每次运行生成独立的 `run_id`，沿用 `RunEvent` 的连续事件序列，并限制一轮最多 256 次工具请求。
- 使用 Kernel 的 `JsonlRunLog` 把事件保存在用户数据目录 `chat/runs/<session_id>/run-<run_id>.jsonl`，原始会话消息仍用现有 JSONL。
- 执行事件先持久化再传递给前端，日志失败会中止后续工具派发，防止不可靠的运行轨迹继续被用于学习。
- 现有 `chat-event` 结构保持兼容，在外层新增 `runId`、`sequence`、`timestampMs`；新增 `chat-run-state`，不把内部 `ToolDispatched` 当作旧的 UI 事件。
- 新增仅控制台窗口可调用的 Tauri 命令 `chat_run_history`，通过会话 ID、Run ID、事件游标按页读取轨迹和未完成工具检查信息。
- 删除会话同时删除对应执行日志；写入前遮蔽当前模型 API Key。

## 数据与安全边界

Journal 中仍可能有用户对话、工具参数与工具输出。它是本地敏感诊断数据，不是匿名遥测；不得默认上传或跨用户共享。SQLite 尚未接入，本阶段没有创建经验库或改动原始会话格式。

对于崩溃后状态为 Running 或派发后没有 ToolEnd 的工具，只能显示“不确定”，**不能根据轨迹自动重复执行**。

## 后续工作

1. 通过外部验证器或用户确认产生独立的 TaskOutcome（成功、失败、部分成功、取消、未知）。
2. 使用 SQLite + FTS5 建立索引与经验存储，并添加迁移、保留、清理策略，而不是重复写入全部运行流。
3. 实现基于真实任务的经验提炼和检索，对比启用/停用经验后的任务质量。

## 测试状况

增加了运行轨迹持久化、终态、原聊天事件兼容、API Key 遮蔽的代码测试。
当前执行环境没有 Rust 工具链，**未运行 Cargo 编译与测试**；合并前应在构建机或 CI 执行 `cargo test --manifest-path client/xiaoban-desktop-wallpaper/src-tauri/Cargo.toml`，并在 macOS 与 Windows 做功能回归。
