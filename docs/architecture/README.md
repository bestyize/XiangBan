# 架构文档

实现状态以各项目 README 和代码为准；规划与阶段性验证不能作为当前版本的验收结果。

| 范围 | 文档 |
| --- | --- |
| 仓库边界与目录迁移 | [monorepo.md](monorepo.md) |
| 统一账号与 AI 网关 | [account-center.md](account-center.md)、[ai-gateway.md](ai-gateway.md) |
| Agent 运行与进程 | [agent-runtime.md](agent-runtime.md) |
| 长期目标与持久运行 | [durable-goals.md](durable-goals.md) |
| 聊天附件与文件分析 | [agent-file-tools.md](agent-file-tools.md) |
| 专家团运行时 | [team-runtime.md](team-runtime.md) |
| 连接器插件格式与接入边界 | [connector-plugins.md](connector-plugins.md) |

以下保留为设计决策、演进规划或历史修复记录，阅读时结合文内基线：

- [WorkBuddy 对齐设计与分期](workbuddy-alignment.md)
- [Agent 框架审视与演进设计](agent-framework-review.md)
- [聊天协议修复记录](chat-protocol-repair-plan.md)
- [Agent 可靠性审计整改记录](../audits/agent-reliability-repairs.md)
- [Agent 演进规划](../agent/README.md)

连接器临时工作包报告已合并到连接器规范、库 README 和市场 README；不再维护个人 worktree、提交阻塞和会话交接说明。历史版本可通过 Git 查看。
