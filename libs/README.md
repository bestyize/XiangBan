# Agent 公共库

按能力独立维护 Rust crate，宿主选择接入，不包含产品业务策略。各库的 feature、边界和验证命令见对应 README。

| 目录 | 职责 |
| --- | --- |
| [agent-kernel](agent-kernel/README.md) | 模型协议、Agent 循环、工具、中间件、执行日志、受管进程与 MCP |
| [agent-learning](agent-learning/README.md) | 本地 SQLite + FTS5 任务结果、经验与评测记录 |
| [agent-proactivity](agent-proactivity/README.md) | 主动感知、授权、免打扰、注意力预算与反馈策略 |
| [voice-service](voice-service/README.md) | 本机 OpenAI 音频接口与 MCP 语音工具，后端通过 trait 注入（xiaoban-voice-service） |
| [community-packs](community-packs/README.md) | 社区内容包的链接解析、哈希校验下载与压缩包检查（xiaoban-community） |
| [link-protocol](link-protocol/README.md) | 手机与桌面互联协议的参考实现：配对、密钥派生、端到端加密信封、重放防护，附测试向量（xiaoban-link-protocol） |
| [link-client](link-client/README.md) | 桌面端互联客户端：设备身份、配对、WebSocket 重连、端到端加密收发、重放防护（xiaoban-link-client） |
| [agent-tools](agent-tools/README.md) | 电脑控制、文件分析、技能发现与连接器扩展 |
