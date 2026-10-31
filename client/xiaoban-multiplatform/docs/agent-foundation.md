# 手机 Agent 基础能力接入与分工

本轮以 Android 的独立创作为第一阶段：用户描述游戏 → Agent 写项目文件 → 校验安装 → 手机试玩 → 读取运行错误 → 修改 → 导出桌面兼容 ZIP。模型请求仍由手机通过 AI 网关发出，不要求电脑在线。iOS 未接入 Rust 内核、本地工作区或试玩宿主，继续使用已有降级路径。

## 模块分工

| 模块 | 负责内容 | 边界 |
| --- | --- | --- |
| `libs/agent-mobile` | Agent 循环、持久任务清单、结构化提问、操作确认、上下文裁剪、防重复与无进展暂停、有界重试 | 不持有密钥，不实现产品游戏逻辑，不代替 Android 后台任务调度 |
| `common/community-core` | 工作区接口与校验、文件工具、游戏清单与导出文件、聊天附件模型、JNI 事件桥接 | 不依赖 Compose；独立实现并兼容桌面资源协议 |
| `androidApp` | 私有文件存储、系统图片/文件选择、WebView 沙箱、存档、运行日志、系统文件保存 | 不向游戏暴露原生桥或手机文件系统；仅读取用户明确选择的附件 |
| `business/community-native` | 游戏列表、创作入口、试玩和导出操作、附件选择入口、提问与审批卡片 | 只组合能力和呈现状态；未实现平台显示明确提示 |
| `shell` | 注入平台能力与工作区 | 不放业务策略 |
| `xiaoban-desktop` | 原有游戏宿主、资源包导入导出与协议兼容 | 本轮只修正导入导出丢失 `orientation` 的问题 |
| `server-ai` | 鉴权、模型路由、额度与计费 | 本轮无改动；不在网关保存手机工作区或运行游戏 |

本轮并行任务为工作区与游戏包、Android 宿主、内核与附件三组；主任务负责界面整合和验证。工作分支 `codex/mobile-agent-foundation` 使用独立 worktree，其他进行中的桌面设置和服务端改动不纳入。

## 第一阶段已接入

- **工作区**：Android 应用私有的 `files/workspace/`。Agent 可列目录、读写、精确替换、搜索和删除文本；拒绝绝对路径、越界路径、符号链接及无效 UTF-8。单文件 1 MiB、工作区 32 MiB，最多 2048 个条目、8 层目录。工作区是本机作品空间，不随聊天删除而删除，也不进行云同步。
- **游戏创作**：`game_read_format`、`game_list`、`game_create`、`game_validate`，支持文件列表或草稿目录安装、显式覆盖与失败回滚。安装位置为 `games/<id>/`。游戏页能看到有效游戏和损坏包原因，提供新建、继续修改、试玩和导出入口。
- **试玩与诊断**：独立 Android Activity 中的脚本沙箱 iframe，使用固定 HTTPS 本地资源拦截，禁止外网、页面跳转和系统权限；提供 nonce 握手、成绩和存档协议。`game_runtime_read` 读取 `logs/<id>.txt`，最多 100 条、64 KiB。日志只表明实际观察到的事件，不代表已经通过玩法验收。
- **存档**：`saves/<id>.json`，最多 32 KiB。合并最新进度，原子落盘，退出和后台切换时刷新；错误会提示。存档和日志不进入导出包。
- **导出**：系统“另存为”保存 ZIP，最外层是 `manifest.json`，`schemaVersion:1`、`kind:game`。文本文件遵守桌面同一格式（最多 40 个、单个 1 MiB、合计 4 MiB）；横竖屏、封面和存档声明可由桌面导入。首阶段不支持 v2 二进制素材、桌面层游戏、手机 ZIP 导入或自动跨设备同步。
- **附件**：相册图片、文本文件选择。图片采样缩放为最长 1600 px 的 JPEG，处理方向；文本严格校验 UTF-8，最大 256 KiB。每次最多 3 个，总量最多 4 MiB。作为参考发送给所选模型，识图是否可用取决于模型；不会自动将图片安装为游戏素材。相机拍摄和资料库选择尚未接入。
- **任务机制**：持久 `todo`、`ask_user`（选择、自由输入、跳过）、防重复与无进展暂停。维持每轮 25 次模型调用上限，临时模型错误最多重试 2 次。任务清单在内核会话目录 `progress/`，删除会话同步清理。保存清单不等于进程被系统终止后自动续跑。

含 Rust `agent_mobile` 库的 Android 安装包才具备 Agent；本地未准备该库时仍降级纯云端聊天。`assembleDebug` 成功仅证明可打包，不保证包里已包含原生库。

## 后续阶段与依赖

| 阶段 | 工作 | 负责模块 | 验收条件 |
| --- | --- | --- | --- |
| 第二阶段 | 网页读取、搜索服务适配；独立记忆读写与检索 | `community-core`、`agent-mobile`，按需接已有服务 | 能引用实际网页与来源；跨聊天检索用户允许保存的记忆 |
| 第二阶段 | ZIP 导入、v2 图片/音频素材、项目文件浏览器、历史版本与恢复 | `community-core`、`androidApp`、原生 UI | 手机导入桌面作品并继续修改；素材与坏包校验一致 |
| 第三阶段 | 已安装技能发现与读取、适合移动端的远程连接器 | `agent-mobile`、`community-core`、原生 UI | 明确工具来源与连接授权，能够取消和撤销；不默认支持桌面 stdio 进程 |
| 第三阶段 | 智能摘要、运行中补充、任务中断恢复和外部验收 | 内核与宿主分别承担 | 不重复执行结果未知的动作，系统终止后按授权恢复并展示真实验收证据 |
| 单独评估 | 通用脚本执行、手机系统工具、跨 App 操作、iOS 原生接入 | 各平台宿主 | 明确 Android/iOS 能力和权限边界；不能直接复制桌面 Shell 或电脑控制引擎 |

## 验证

在 `client/xiaoban-multiplatform` 执行，使用 Android Studio JBR（JDK 17+）及已安装的 Android SDK：

```sh
./gradlew :common:community-core:jvmTest --tests '*MobileWorkspace*' --tests '*MobileGames*' --tests '*MobileAgentExtensions*' --tests '*MobileGameAgentTest*' --tests '*AgentRuntimeTest*' --tests '*AgentChatTest*'
./gradlew :androidApp:assembleDebug
node androidApp/scripts/test-mobile-game-host.cjs
```

JNI 测试要设置 `AGENT_MOBILE_LIB` 为实际编译的本机库，否则原有 native 测试会跳过。仓库根执行：

```sh
cargo test --locked --manifest-path libs/agent-mobile/Cargo.toml
cargo test --locked --manifest-path client/xiaoban-desktop/src-tauri/Cargo.toml --bin xiaoban-desktop resource_packs::tests::
```

自动化覆盖路径约束、限额、游戏清单、失败回滚、实际 JNI 桥接、图片/提问消息及宿主消息过滤。Android WebView 的触控、前后台、文件选择/保存和真实模型创作仍需设备验收；iOS 不在本轮验收范围。
