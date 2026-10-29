# Agent 公共库

按能力独立维护 Rust crate，宿主选择接入，不包含产品业务策略。各库的 feature、边界和验证命令见对应 README。

| 目录 | 职责 |
| --- | --- |
| [agent-hooks](agent-hooks/README.md) | Claude Code hooks 配置与本机事件接收（实验） |
| [agent-bridge](agent-bridge/README.md) | 外部本地 Agent 适配层原型；会话启动尚未启用 |
| [agent-kernel](agent-kernel/README.md) | 模型协议、Agent 循环、工具、中间件、执行日志、受管进程与 MCP |
| [agent-learning](agent-learning/README.md) | 本地 SQLite + FTS5 任务结果、经验与评测记录 |
| [agent-proactivity](agent-proactivity/README.md) | 主动感知、授权、免打扰、注意力预算与反馈策略 |
| [voice-service](voice-service/README.md) | 本机 OpenAI 音频接口与 MCP 语音工具，后端通过 trait 注入（xiaoban-voice-service） |
| [community-packs](community-packs/README.md) | 社区内容包的链接解析、哈希校验下载与压缩包检查（xiaoban-community） |
| [link-client](link-client/README.md) | 桌面端互联客户端：同账号设备登记与发现、WebSocket 重连、文本收发（xiaoban-link-client） |
| [agent-tools](agent-tools/README.md) | 电脑控制、文件分析、技能发现与连接器扩展 |
