# 专家团运行时（第 3 期）

对齐 WorkBuddy 的 `expertType: team`。依据：本机 WorkBuddy 自带的 `skill-expert-manager/references/team-spec.md`（协作铁律、红线、SOP 并行/串行）和它的专家模式工具清单（`Agent`、`Defer(TeamCreate)`、`Defer(TeamDelete)`、`SendMessage`）。

## 做法

主理人（团里的 `leadAgent`）所在的对话拿到四个工具，名字和用法与 WorkBuddy 一致：

| 工具 | 作用 |
|---|---|
| `TeamCreate {team_name, description?}` | 为这一轮回答建团队，返回成员名单和 agent id。只有主理人有这个工具；不先建团队就调度会被拒绝 |
| `Agent {name, subagent_type, prompt}` | 把一项任务交给成员，返回成员的产出。`name` 和 `subagent_type` 都填成员的 agent id（不是中文名）。**同一条回复里调用多个 `Agent`，它们同时工作**（SOP 的并行阶段） |
| `SendMessage {to, message}` | 给已经工作过的成员追加要求或回答它的提问，成员接着自己的上下文做 |
| `TeamDelete` | 解散团队，释放成员的会话 |

成员是独立的 agent：

- 人设来自它在插件里的 `agents/<id>.md`；工具、权限与主对话一样（文件、命令、联网、技能），agent md 里写的 `tools` 一律忽略。
- 成员之间不能直接通信：成员只有一个 `SendMessage`，且只能发给主理人（`to: lead`）。需要跨成员的信息由主理人转交。
- 成员的产出回到主理人手里：成员用 `SendMessage` 发来的内容；没发就取它最后一段回答。
- 权限检查是同一个 `Permission`：成员的会话是 `<对话>#team-<成员>-…`，沿用主对话的模式、"这段对话都允许"、污染状态，确认卡片出现在主对话里（内核 `Permission::children_inherit`）。

主理人的系统提示词 = 它自己的人设 + 团队章程（协作铁律 4 条、红线 5 条、工具用法、成员名单），章程在 `chat/team.rs` 的 `charter`。

## 和 WorkBuddy 的差异（以及为什么）

- 团队只活在一轮回答里：下一条用户消息需要时主理人再 `TeamCreate`，成员的上下文不跨用户消息保留。原因是桌面端每条消息重建一次 agent；对话历史里主理人的汇总足够接续。
- `Agent` 是同步的：调用返回时成员已经完成。WorkBuddy 的成员异步汇报；效果相同（主理人拿到成员产出后继续），并行靠"同一条回复里多个 `Agent` 调用"。
- 没有 A2A（跨产品的 agent 通信），也不做成员之间的私聊。

## 代码

- 内核：`libs/agent-kernel/src/builtin/team.rs`（`Team`、`TeamMember`、`Outbox`），测试 `tests/team.rs`（8 项：建队与汇报、并行、顺序约束、成员只能发给主理人、主理人追加消息、团队随回答结束而释放、成员失败、事件观察）。
- 权限继承：`middleware/permission.rs` 的 `children_inherit` / `root_session`，测试在 `tests/permission.rs`。
- 桌面端：`chat/team.rs`（读团队、章程、成员 agent、工具），`chat/mod.rs` 的 `build_agent` 在对话绑定专家团时把团队工具作为常驻工具接入，章程接在人设后面。测试用真实的工具组装成员，脚本化模型走完"建团队 → 调度成员 → 成员回传 → 主理人收尾"。
