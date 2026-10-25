# Agent 公共库

按能力独立维护 Rust crate，宿主选择接入，不包含产品业务策略。各库的 feature、边界和验证命令见对应 README。

| 目录 | 职责 |
| --- | --- |
| [agent-kernel](agent-kernel/README.md) | 模型协议、Agent 循环、工具、中间件、执行日志、受管进程与 MCP |
| [agent-learning](agent-learning/README.md) | 本地 SQLite + FTS5 任务结果、经验与评测记录 |
| [agent-proactivity](agent-proactivity/README.md) | 主动感知、授权、免打扰、注意力预算与反馈策略 |
| [agent-tools](agent-tools/README.md) | 电脑控制、文件分析、技能发现与连接器扩展 |
