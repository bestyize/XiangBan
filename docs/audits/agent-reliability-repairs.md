# Agent 可靠性审计整改记录

原修复基线：`28125dd0257db8dc4197a8e4a66c8e8853249dfe`。接手后沿用 `codex/agent-reliability-repairs` 的实现，并与最新主分支三方合并。对应原审计 F01–F15；本次不修改发布工作流，不自动触发构建。

## 整改对应关系

| 审计项 | 本次实现 | 回归证据 |
| --- | --- | --- |
| F01 授权/提问刷新后丢失 | Broker 保存完整待答载荷及修订号；会话快照和变化事件返回权威状态；客户端拒绝旧修订 | Broker 快照测试、聊天 Store 重建及乱序事件测试 |
| F02 超时后副作用未停 | 给工具 250ms 确认停止；无确认/崩溃记录 `outcome_unknown`，暂停后续派发并保留资源租约。stdio 发送取消通知，直到实际响应或进程停止才释放；HTTP 关闭 socket 不冒充远端停止，保留占用并阻止自动重连洗掉未知状态。状态提示核对远端后断开重连。复核标记持久化，完全信任/历史授权不能跳过后续修改确认 | `reliability_control`、`tools_exec`、`permission`、连接器测试 |
| F03 静默后台子进程残留 | Shell、Skill、MCP 共用 ChildTree；普通退出、取消及启动失败都释放进程树；Windows 使用已有受管进程实现的 Job Object，不能挂接时拒绝启动 | Linux 重定向静默子进程测试；Windows 分支待原生验证 |
| F04 跨任务共享资源冲突 | 桌面/未知执行域保守持有整轮租约；文件工具声明解析后的路径，按文件隔离，读取和不同文件仍可并行；子任务共享父租约。文件读回版本摘要，写入可传 expected_sha256，编辑检查读写期间的变化。撤销取得资源租约并核对结束态哈希/Git tree；Git 临时索引使用唯一身份 | 真实文件双写、租约冲突、只读/不同文件并行、撤销冲突测试已通过 |
| F05 初始化取消及锁等待 | MCP 初始化总时限 45s，stdio 取消关闭进程，HTTP 取消释放请求；连接器网络初始化移出状态锁，以代次拒绝过期安装。共享电脑控制正在初始化时立即降级，不把另一个会话堵在锁上；状态工具报告不可用能力；宿主 RAII 清理覆盖构建/执行 panic | stdio/HTTP 初始化取消、连接器包测试通过；Tauri 整包未构建 |
| F06 异常终态仍派发 | filtered/unknown 终态整批拒绝；ToolCalls 终态没有实际调用也拒绝普通完成；持久化拒绝结果并暂停 | `reliability_admission`、协议与终态测试 |
| F07 循环完成误作目标完成 | in_progress 仍算未完成，提醒后不能自动忽略 TODO；定时空答/异常终态/未解决工具错误不返回成功报告。同工具、同参数后续成功可解除未解决错误，仍保留失败证据；不同目标不能替代。交互界面明确提示未验收，文件验收绑定实际运行工作区 | `guards`、任务结果、聊天 Store/View 测试；定时宿主待原生验收 |
| F08 压缩丢约束及错误状态 | 原始用户请求独立保留，超预算明确报错；摘要保留 call_id、错误和结构化状态；拒绝截断/过滤/超长摘要，不提交半份摘要 | `context_spill`、压缩模块测试 |
| F09 信任污染重启丢失 | 在外部内容执行前落盘并 sync；恢复历史授权前恢复信任状态；旧工具历史保守处理；损坏状态要求复核 | `permission`，含重启及完全信任模式复核 |
| F10 HTTP 取消遗留 socket | 模型传输采用同步入口内的可取消 async 请求，无被遗弃的阻塞读线程；保留首包/空闲/总超时及 SSE 大小限制。MCP OAuth 阻塞式令牌刷新移到异步运行时之外，避免刷新时 panic | socket EOF、三协议 HTTP、OAuth 刷新和恢复测试 |
| F11 进程终止及 Drop 卡住 | 删除依赖 dash kill 语法的路径；共用受管进程所有权；MCP reader 只在有界等待后 join 已完成线程 | 进程、MCP 与子进程故障测试 |
| F12 MCP 目录截断 | stdio 分页至结束，限制 20 页/500 工具，拒绝重复名称、游标及无效目录 | 双页目录、重复游标测试 |
| F13 重复工具标识执行两次 | 派发前校验整个批次的非空唯一 ID，异常时不执行任何一项，并规范拒绝记录中的 ID，避免污染后续协议历史 | `reliability_admission` |
| F14 工具网络未用手动代理 | 宿主统一注入可信 Rust HTTP 策略，WebFetch、SkillHub、HTTP MCP、OAuth 使用同一策略；direct 明确关闭代理 | WebFetch 录制代理及 direct 覆盖测试；SkillHub/OAuth 原生包待构建 |
| F15 运行预算和无进展 | 交互默认 45min/100万 reported tokens，定时 10min/20万，仍保留调用数/轮数预算；父子共享，压缩计量；连续 6 个无新成功结果的工具轮暂停；变化的观测结果计为进展 | 总时限中断、交替失败、子任务、续跑测试 |

## 实际验证

接手后在 Linux 使用实际 Cargo 依赖完成编译和测试；没有沿用原先“直接 rustc 链接缓存”的结果代替本轮验证：

- Kernel 单元及集成测试：347 项通过，包括取消、进程后代、协议终态、MCP、资源租约、文件版本和预算。
- Broker、压缩、运行日志、Git 快照、撤销及任务结果模块：47 项通过，使用 `tools/agent-reliability-checks` 编译真实产品模块，无复制实现。
- 连接器与 OAuth：10 项通过；Skill 包：30 项通过；电脑控制封装：6 项通过，2 项真实运行时测试因未准备原生引擎而明确跳过。
- 与最新主分支合并后的聊天 Store/View：96 项通过；相关 TypeScript 检查通过。
- 合计 536 项通过；Kernel 的 Windows GNU 目标 `mcp shell process` 类型检查通过。修改的 Rust 文件通过语法解析。

这是组件、协议和核心进程类型验证，不是 Tauri 整包或全平台实机验收。没有构建 Windows/macOS 安装包、调用真实付费模型或操作用户桌面。首次完整回归发现的旧断言（允许异常终态、压缩时删除原始用户请求、遗漏 skill_find）已按新契约更新并复跑。

依赖齐备后的针对性验收命令（在仓库根执行）：

```sh
cargo test --locked --manifest-path libs/agent-kernel/Cargo.toml --features 'openai anthropic web mcp shell process journal memory fs skills stdio'
cargo test --locked --manifest-path tools/agent-reliability-checks/Cargo.toml
cargo test --manifest-path libs/agent-tools/connectors/Cargo.toml
cargo test --manifest-path libs/agent-tools/skill-discovery/Cargo.toml
cargo test --locked --manifest-path client/xiaoban-desktop-wallpaper/src-tauri/Cargo.toml chat::
cd client/xiaoban-desktop-wallpaper
npx vitest run src/ui/chat/chat-store.test.ts src/ui/chat/view.test.ts
```

Windows/macOS 还需人工验证：Job/进程树、辅助功能/截图权限被撤销、原生窗口重建、授权中停止、MCP 重启、共享文件撤销冲突。只使用隔离文件和模拟服务，不发送真实消息或执行生产破坏性命令。

## 明确的边界和取舍

- 桌面和无法声明更小执行域的修改采用整轮租约；文件按路径隔离；只读工具、网页读取和不同文件不因该租约全局停摆。繁忙任务明确暂停。Kernel 旧入口兼容，宿主通过 `RunOptions.control` 启用；不声称提供跨进程/跨设备资源锁。
- expected_sha256 拒绝过期输入，进程内路径锁避免文件工具相互丢更新；它不是对任意外部进程的文件系统事务，外部程序仍可能在最后检查与原子替换之间修改文件。
- Rust 任意工具线程和已发送到远端的操作不能安全强杀或自动回滚。无确认结果会暂停、保留租约并要求核对，不会声称动作未发生；新的修改需重新确认。长期非合作工具应迁入可终止的独立进程。
- reported token 是响应边界上的软预算；服务商未报告的用量、单个在途响应和真实货币费用不能由此精确限制。时间、调用数及协议请求上限提供额外边界。
- 保留原始用户请求可能使过长会话无法继续压缩；此时明确要求缩小范围，不静默丢弃限制。通用语义验收仍需任务领域的验证器；模型回复/工具成功不会自动成为业务成功。
- 旧撤销记录缺少结束态证据时拒绝覆盖。配置的代理是可信网络中继；不把代理设置传播给不可信 WebView 或任意 Shell 脚本。
- 账号凭据刷新、系统目录标签等宿主准备工作仍受其既有网络/系统调用边界约束；此次已验证的是模型及 MCP 传输取消，未宣称所有原生系统调用可瞬时取消。
