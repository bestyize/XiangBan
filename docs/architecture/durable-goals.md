# Persistent Goals + Durable Run Runtime

范围：`libs/agent-kernel/src/goals.rs`、`recovery.rs`、`supervisor.rs` 与桌面聊天工作台。2026-10-22。

## 目标

提供比单次聊天更稳定的长期目标：目标契约独立持久化，执行过程沿用现有 Journal，应用重启后能够识别上一次运行是否结束、是否可能存在结果不明的外部操作。**不以模型循环完成作为业务验收通过。**

当前实现包含持久进度、累计预算、用户指定命令验收和可选的启动恢复，不新增独立后台服务、Cron、LLM Judge 或未知动作重放器。聊天、停止与授权入口沿用原有流程。

## 结构与数据边界

- `GoalContract`：目标 outcome、verification、constraints、boundaries、stop_when，可选的 verification_command 与 auto_resume。字段由用户输入，作为下一轮 user 目标提示传递，绝不提升为系统级权限。
- `GoalRecord`：会话一对一存储 `chat/goals/<session>.json`，原子覆盖，独立于消息 JSONL；包含 `lastRunId`、`attempts`、状态及人工核验证据。
- `JsonlRunLog`：仍是运行事件事实来源。状态事件在记录成功后回写目标的运行标识与状态。模型返回 `Completed` 时目标只能转入 `AwaitingVerification`。
- `SessionLease`：OS 文件锁（`chat/leases/<session>.lock`）跨进程排他，贯穿聊天执行及会话删除；锁文件不删除，进程退出时锁自动释放。目标记录写操作使用单独的 `chat/goal-leases/`，防止不同桌面进程同时覆盖目标。
- `recovery::reconcile`：目标记录若仍是 Running、但当前及其他活跃进程都不持有会话锁，则查询上次 run trace，判定是否恢复终态、普通中断或不确定的已派发外部操作。
- `Todo::persistent`：计划、里程碑证据与 handoff 单独保存在 `chat/progress`；新回合开始刷新缓存。记录作为数据提供给模型，保存失败不报告成功，清除会话时一起删除。
- `ExecutionLimits`：交互默认无限轮数/工具次数，保留 45 分钟/100 万 reported tokens；可配置取消各上限。目标累计用量包含父子共享控制器读数，暂停/继续不清零。原“继续”按钮走同一目标入口。运行中压缩计入预算，目标结束后不额外调用摘要模型。
- `VerificationGuard`：只认可通过原权限链执行的精确用户命令的成功结果；支持 Shell，以及 process_start 后按进程 ID 等待退出码为 0 的长检查。修改或未知工具会使之前的验收过期。目标日志再次核对验收、未完成工具、未解决错误和里程碑，不依据模型自述完成。拒绝授权后暂停，不自动催促重试。

## 状态机

```text
Draft --明确启动--> Running
Running --RunState::Completed--> AwaitingVerification
Running --Paused/Failed/Cancelled--> Paused
Running --进程失联且无不确定外部动作--> Interrupted
Running --已派发外部动作但无持久结果 / 日志缺失--> NeedsReview

Interrupted/NeedsReview --用户记录现场核验--> Paused
Paused/AwaitingVerification --用户明确继续--> Running（全新回合）
AwaitingVerification --用户提供完成证据--> Completed
AwaitingVerification --指定命令有新鲜成功证据且里程碑关闭--> Completed
Interrupted --原契约允许自动恢复，且无未知动作--> Running（启动时的新回合）
```

注意：如果中断前已记录 `ToolDispatched`，但没有相应 `ToolEnd`，不应假设外部写操作未发生。用户必须先检查现场并记录结果；`chat_goal_start` 不重放旧工具调用，只创建新的带目标契约的用户回合，要求 Agent 从当前状态检查并继续。

## 控制台接口

| Tauri 命令 | 用途 |
| --- | --- |
| `chat_goal_get` | 查询目标，并在安全时自动对账旧状态 |
| `chat_goal_set` | 保存目标契约（无活跃任务） |
| `chat_goal_start` | 用户明确启动新的 Agent 回合 |
| `chat_goal_pause` | 停止当前运行、保存暂停 |
| `chat_goal_acknowledge` | 手动核验中断现场并记录证据 |
| `chat_goal_finish` | 人工按证据验收完成 |
| `chat_goal_delete` | 删除已停止的目标 |

全部限制为 `control` WebView，不接受壁纸内容调用，现有 Shell/文件权限中间件照常生效；完全信任不跳过目标的人工验收。

## 限制

1. **没有独立后台服务**：应用关闭时无目标执行。自动恢复仅在启动时扫描已明确授权的运行中目标；主动暂停、已完成和未知动作不会自动启动。
2. **验收覆盖有限**：可执行用户指定的 Shell 命令验收，也可人工确认；命令必须能代表预期检查。没有通用语义质量评审器，不保证通过一个命令就满足全部业务要求。
3. **不承诺 Exactly Once**：本地 OS 锁防止同时运行；它无法证明进程崩溃前的外部副作用有没有发生，也不是远程分布式锁。
4. **多进程条件**：会话租约只对使用本套 SessionLease 的入口有效；外部进程绕过 Kernel 直接修改文件不受限制。
5. **保存范围**：只记录目标契约、运行 ID 与验收说明；原消息及 RunEvent 仍在现有存储中，不会把私有运行历史上传。
6. **恢复不重放旧调用**：打开目标时对账；已授权且安全的普通中断可在启动时创建新回合，从现场检查继续。对账失败会报错，不修改不确定操作。自动恢复默认关闭，旧目标不会因升级获得新授权。

## 验收建议

```sh
cargo test --manifest-path libs/agent-kernel/Cargo.toml --features journal
cargo test --manifest-path client/xiaoban-desktop/src-tauri/Cargo.toml chat:: --locked
cd client/xiaoban-desktop
npm ci
npx vue-tsc --noEmit
npx vitest run src/ui/chat/ChatGoalPanel.test.ts src/ui/pages/ChatPage.test.ts
```

手动用两个小伴实例尝试同一会话并发，第二个应被租约拒绝；执行一项有可核查的外部写操作时强制终止应用，重启后必须进入 Interrupted / NeedsReview，不允许直接重放；完成普通回合后进入“等待验收”，提交证据后才能变成“已完成”。
