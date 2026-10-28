# 小伴桌面：桌宠、动态壁纸与 Agent

## Android 手机操作（基础与本地工作流）

对话按需发现 `phone_` 工具，可通过本机 ADB 操作 USB / 无线 Android 设备；应用包以包名保存 `skill.md + basic/ + patch/`，支持 ZIP 导入 / HTTPS 下载、版本化补丁及分层页面路由。工具及种子包在 [phone-use](../../libs/agent-tools/phone-use/README.md)，方案见 [Android 手机操作](../../docs/agent/android-phone-use.md)。数据放在用户聊天目录的 `phone-packs/`；不与普通 Skill 混用目录。

同一任务按设备复用非交互 ADB Shell；任务结束关闭连接。`phone_workflow_inspect` 审查操作包的结构化 YAML 工作流，`phone_workflow_run` 经确认后连续执行已知步骤，返回逐步耗时、通信次数及异常 checkpoint。内置小红书工作流分为“打开相册并交回模型选图”“从图片编辑进入正文、填写空字段并核对预览”，不包含发布。此优化仅经本地模拟测试，按用户要求暂未在小米17重测；uiautomator 仍逐次启动，截图/传图仍用独立命令，不代表生产延迟已达标。

需要本机已安装 Android Platform Tools（PATH 中的 adb，可用 `XIAOBAN_ADB` 指定路径）并由用户在手机授权。中文输入需要手机已安装且已选中 ADB Keyboard（Android 16 实测使用上游 v2.5-dev）。首轮通过聊天使用，没有独立手机管理页、无障碍接收端或 iOS 支持。小米17 Android16已验证小红书9.51.0的传图、选图和中文图文准备，以及消消乐1.162第4181关的两次交换；发布与通关尚未验收。消消乐仅有本地普通三消候选评分，没有完整棋盘识别器、特殊机关策略或通关保证。发布等动作必须遵循用户授权，结果不明先观察，不能盲目重试。

## 内置设计工具：open-claude-design

真实用户消息提到设计、画板、原型或准确 design- 编号时可发现设计工具；聊天顶部「设计稿」入口支持全局搜索、打开与新建。支持草稿立即落盘、逐页生成、完成校验与受阻恢复；一份作品支持多个 DC HTML 画板。聊天右侧画布支持拖动分屏、弹出独立窗口（复用、最大化、系统全屏、窗口位置记忆、回到分屏），支持以指针为中心的 Ctrl/⌘ 滚轮缩放、触控板捏合、Space/中键平移、选择/Play、属性与源码编辑、批注交给小伴修改、撤销重做和历史恢复。作品保存在「资料库 → 我的文档」，聊天与资料库读取同一份源数据；可导入 ZIP / HTML，导出 ZIP 源包或独立 HTML。

保存有版本检查和恢复草稿，保留 30 个历史版本，支持重命名、创建独立副本与历史恢复。聊天及排队消息固定发送时选中的设计稿，避免切换作品后改错项目。兼容 v3 boards/order 与旧版 artboards；当前只支持独立 DC HTML 运行时子集，React 和二进制素材导入尚不支持。详见 [模块说明](../../libs/agent-tools/open-claude-design/README.md) 与 [实施和验证记录](../../docs/agent/open-claude-design.md)。


## 社区内容包（xiaoban://community）

侧栏「社区」页只读浏览周刊和热门内容包，网页上的「在小伴中打开」通过 `xiaoban://community/apply/{kind}/{pack_id}?version=&sha256=` 唤起本应用。流程固定为：**下载 → SHA-256 与大小校验（主地址失败回退官方镜像）→ 压缩包结构检查 → 展示内容 → 用户确认 → 才应用**；链接带的 `sha256` 会锁定网页上看到的那一版，版本已更新则拒绝。首批支持 `effect`（沿用特效包校验，写入特效文件夹）、`reminder`（提醒规则）和 `task`（定时任务模板，会让小伴代为执行，应用后**默认不启用**，到提醒页确认后再打开）；其他类型先提示去网页查看。

下载、校验与压缩包检查在 [`libs/community-packs`](../../libs/community-packs/README.md)，可独立测试；`src-tauri/src/community.rs` 只负责暂存检查结果和落地，命令只允许设置面板调用（`community_*`）。论坛地址随应用内置（`https://chat.xiangban.xyz`），仅调试版可用 `XIAOBAN_COMMUNITY_URL` 指向本机。

验证：`npx vitest run src/ui/community`；`cargo test --manifest-path ../../libs/community-packs/Cargo.toml`。`src-tauri` 里的 Rust 改动（`community.rs`、`reminders::import_rules`、`main.rs` 的链接分流）需要在 Windows / macOS 上完整编译确认，Linux 云端环境缺少 GTK/WebKit 无法编译整个应用。

## AI 商店：SkillHub 专家包与 MCP

**小伴市场（默认来源）**：AI 插件页默认从自建的小伴市场获取专家、技能和连接器，筛选行右侧可切换到 SkillHub；市场连不上时自动先显示 SkillHub，已安装的插件不受影响。市场地址随应用内置（`src-tauri/src/chat/store_market.rs` 的 `DEFAULT_MARKET`，当前为 `https://market.xiangban.xyz`；仅调试版可用环境变量 `XIAOBAN_MARKET_URL` 指向本机）。安装前展示说明、文件清单、逐文件 SHA256 和脚本预览，并按清单摘要锁定版本；已安装的市场插件有新版本时提示更新，更新前展示文件差异，旧版本保留备份（`chat/skill-backups`）；专家团、技能套件一次审查、整组安装、失败整组回滚。网页上的「在小伴中安装」通过 `xiaoban://market/...` 唤起本应用，只弹出确认窗口，不会静默安装；协议关联由 `tauri-plugin-deep-link` 在打包安装版里注册（`tauri.conf.json` 的 `plugins.deep-link`）。「插件来源」设置里可关闭 SkillHub 补充来源和自动检查更新（保存在本机界面偏好中）。

验证：`npx vitest run src/ui/pages/AiStoreMarket.test.ts src/ui/ai-store/market.test.ts`；`cargo test --manifest-path libs/agent-tools/skill-discovery/Cargo.toml market`。

技能分类与在线技能目录优先通过设置中网站地址的 `/api/v1/ai/plugins`、`/api/v1/ai/plugins/categories` 读取，复用服务端 SkillHub API 代理与短期缓存。旧服务端明确返回 HTTP 404 时，回退到原有 SkillHub 直连流程；403、429、服务错误、网络故障或无效响应不会触发回退，空列表也保持服务端结果。每次查询都会优先尝试本站接口，服务端部署后自动切回，无需重启客户端。审查、安装和文件校验继续直接使用原有 SkillHub 流程，本地技能不受目录离线影响。官网 `/plugins` 也可浏览技能和复制安装指令，网页不会自动安装或执行代码。

目录兼容测试：在本目录执行 `cargo test --locked --manifest-path src-tauri/Cargo.toml chat::store::catalog::`，覆盖旧站纯文本 / HTML / JSON 404、正常空列表、权限与限流、异常响应和网络失败。

专家包和连接器目录读取 SkillHub 实际数据，支持搜索与分页；请求失败显示错误与重试。专家包按工作流审查、固定各依赖的发布者、版本和文件哈希，确认后事务安装，不运行脚本。已安装工作流可带入新聊天，移除时保留共用技能；WorkBuddy 专属注册与调度包暂不兼容。

MCP 连接器使用服务 README 中的标准配置，支持本机 stdio、远程 Streamable HTTP 与旧式 HTTP+SSE。市场插件审查并按摘要锁定后，可选择“安装并连接”：无需填写的远程服务自动试连，需要令牌时在当前安装窗口填写，需要标准 MCP OAuth 时直接打开浏览器授权（PKCE、本机回调与自动刷新）。本地命令展示后仍需点击“运行并连接”；运行环境需本机提供。连接器中心复用同一套表单和授权流程，不再要求额外勾选连接确认框；手工 MCP 配置的 JSON 仍在高级配置中。WorkBuddy 私有身份及服务专属登录仍需独立接入。初始化与工具发现成功才显示已连接。配置保存在本机，退出关闭连接，下次使用时恢复已启用配置；断开后旧任务中的工具也会失效。

对话支持 `connector_search` 搜索已安装服务及小伴市场，`connector_request` 展示最多三个连接卡片并等待用户安装或授权；凭据在配置窗口填写，不进入聊天或模型上下文。完成后返回当前可用工具及 schema，可通过 `connector_call` 在同一任务继续调用，无需重开聊天。外部工具调用保持逐次确认，断开后的旧工具仍会失效；不自动重放超时或结果未知的操作。针对性验证：`npx vitest run src/ui/ai-store/ConnectorConnection.test.ts src/ui/ai-store/ConnectorCenter.test.ts src/ui/ai-store/RegistryMarketConnector.test.ts src/ui/chat/ChatConnectorRequest.test.ts`。

运行时分别位于 [skill-discovery](../../libs/agent-tools/skill-discovery/README.md) 与 [connectors](../../libs/agent-tools/connectors/README.md)。针对性验证：`cargo test --manifest-path libs/agent-tools/skill-discovery/Cargo.toml`、`cargo test --manifest-path libs/agent-tools/connectors/Cargo.toml`（仓库根目录），以及本目录 `npx vitest run src/ui/pages/AiStorePage.test.ts src/ui/ai-store/RegistryCatalog.test.ts`。真实目录及临时安装联调：`cargo run --manifest-path libs/agent-tools/skill-discovery/Cargo.toml --example catalog-smoke -- tech-test-automation --install`，不执行包内脚本。

对话内 Skill 接入支持市场 slug 与包内名称不同的受管技能；名称映射保留原文和调用限制，加载时校验文件哈希。模型先通过 `skill_inspect` 获取摘要，再用 `skill_review_read` 分页审查清单、正文和相关脚本。安装有独立的下载总预算和可取消请求，回退下载最多并发 4 个文件；相同且完整的已安装版本支持安全重试。读取、运行统一支持 512 文件上限，大型文本引用通过 `skill_read` 分页查看。

“＋ → 专家”同时列出插件专家、专家包和市场工作流型专家；连接器列表同时列出已安装插件与手工配置的 MCP，未连接的项目保留并显示状态。所有专家从输入框或商店选择后都进入右上角的会话绑定，后续消息及重新打开会话时继续生效；工作流型专家以已验证的参考说明加载，普通技能仍位于左上角并只作用于当前消息。

Windows / macOS 桌面应用：一个可拖动、透明处点击穿透的悬浮小伴，也可以切换成位于桌面图标下方的动态壁纸。内置通用 Agent 聊天与 AI 商店；默认连接 `https://wallpaper.xiangban.xyz` 获取在线图片、动态视频、版本与公告。Windows 缺少运行组件时会联网补齐；组件与缓存可用后可离线运行。角色、场景、控制台与互动逻辑共用，桌面宿主分平台接入。

2.1.0（构建中，SHA-256 待出包后补）：界面与能力更新包括——桌面伙伴改为主题制，官方「花朵心语」「多肉点点」共七只，支持自定义主题、导入导出与每只独立取名/大小/心情/层级；桌面组件与特效、宠物、游戏统一为「官方 / 自定义」两栏，每个组件可选择“贴在桌面”或“始终置顶”，自定义组件单独归类；聊天回答下方可展开执行过程，支持粘贴图片与文件作为附件、选中文字复制；小伴 AI 面向用户只保留“日常 / 深入”两种使用模式，兼容端点可另设回答偏好；超级小伴新增紧凑工具栏、区域截图与标注、彩虹生命球，以及 macOS 截图权限的自助修复入口。

2026-10-20 **2.0.0 正式版（重新出包）**：Windows x64 一键安装包 `XiaoBan-Setup-2.0.0-Windows-x64.exe`（12,196,799 bytes；SHA-256 `eb5ec63491d8fba3a6a143ecaf6c39e817b4a4b66fc7430b3406585488e6a80c`），macOS aarch64 dmg `XiaoBanWallpaper-2.0.0-MacOS-aarch64.dmg`（19,700,610 bytes；SHA-256 `8722b71b1f55c62673154db8971aca9ac79a3cf9fc58b68c96c7c0daf7e4e794`，固定自签名证书、未公证）。已上架 `wallpaper.xiangban.xyz`（含客户端更新接口）与 GitHub Release `xiaoban-wallpaper-v2.0.0`，下载后 SHA-256 与构建产物一致。

旧原型包、临时 EXE 路径与逐包校验值不再作为运行入口维护。当前打包方式见下文；历史测试记录见 [功能验证清单](docs/feature-validation.md)，仅代表对应日期的基线。

角色核心在本工程的 `src/companion` 内维护，最初从网页纯角色代码原样复制，保留既有 SVG、发光、表情、弹簧参数和互动行为。允许独立演进，不再导入另一个客户端源码。`src/pet.ts` 适配 DOM 与桌面输入；暖光维持原 RobotStage 的尺寸与渐变。

## 设计规范与交互稿

超级小伴已采用月白 / 夜银两套配色与品牌标志小球（主题色圆底 + 白色 Sparkles），收起球使用待机动效；思考、文字回复及语音播放均可通过右侧方块停止。当前形态与品牌标志以 [超级小伴设计规范](docs/design-specs/super-assistant.md) 为准；[对话框设计稿](docs/design-specs/prototypes/assistant-conversation.html) 提供配色与回复交互参考，[旧生命球演示](docs/design-specs/prototypes/assistant-living-orb.html) 仅供历史追溯。

工程内的设计文档入口见 [设计规范目录](docs/design-specs/README.md)。超级小伴的 [悬浮按钮设计稿](docs/design-specs/prototypes/assistant-floating-tools.html) 与 [截图工具设计稿](docs/design-specs/prototypes/assistant-capture-overlay.html) 均可直接用浏览器打开；交互约定及正式代码对应关系见 [超级小伴设计规范](docs/design-specs/super-assistant.md)。

## 聊天输入框：技能 / 专家 / 连接器

新建任务与会话共用一个输入框：技能以 chip 放在文字上方左侧（一次性，最多 3 个，发送时写成 `/名称`）；专家 / 专家团和连接器放在右上角，整个对话内一直生效，连接器图标最多显示 3 个、其余折成 +N，点开逐个开关。规范见 [聊天输入框规范](docs/design-specs/chat-composer.md)。

## 回复中的消息与附件

聊天工作台在回复期间仍可截图、选择、粘贴或拖入附件。Enter 默认加入按会话持久保存的队列；输入框上方可编辑文字、移除附件、删除、拖动或上下移动排序，以及暂停或继续发送。发送按钮旁可选择“补充当前任务”（在模型请求或整批工具调用结束后接收）或“停止并发送”（确认当前运行结束后处理此条，其余队列暂停）。停止按钮始终独立，且不提交草稿。技能需要在新一轮构建时绑定，带斜杠技能的补充会进入下一轮队列。

停止、失败和应用重启后队列暂停，待用户检查后继续；入队消息以提交编号去重，写入历史后才确认接收。附件保存为会话副本，未接收的消息不进入模型上下文。会话的模型、专家或工作区变化时先暂停，点击继续后使用当前会话配置，权限在运行时重新检查。

## 长程任务执行

交互聊天默认不再设置固定模型轮数或工具调用次数。设置 → 安全与权限 → 任务执行可分别设置轮数、工具次数、时间和累计模型用量，全部默认不限制（与 Claude Code 一致），需要时再手动设置上限；停止、权限、重复调用和无进展检测继续生效。长期目标的活跃时间与用量跨继续累计，原“继续”按钮沿用目标预算；只读定时任务仍保留独立预算。

累计模型用量的统计方式参照 Claude Code 按费用估算预算的做法：每次模型调用的用量 = 未命中缓存的输入 + 命中提示词缓存的输入 ÷ 10 + 输出。长任务每一轮都会重新发送完整上下文，这部分绝大多数命中缓存，按全价计入会很快耗尽预算。缓存命中数读取服务商回传的字段：Anthropic 的 `cache_read_input_tokens`、OpenAI Chat 的 `prompt_tokens_details.cached_tokens`、DeepSeek 的 `prompt_cache_hit_tokens`、Responses 的 `input_tokens_details.cached_tokens`；服务商没有回传时按全价计入，与此前一致。与 Claude Code 的差别：它按各模型价格表折算成美元，这里没有价格表，所以输出不加权，Anthropic 的缓存写入也按普通输入计价；该数值是用量估算，不是账单。统计实现位于 `libs/agent-kernel/src/types.rs` 的 `Usage::budget_units`。

多个对话可以同时运行命令（shell 和后台进程不再互相占用），文件写入仍按文件互斥并校验内容是否变化。需要让几个对话改同一个项目又互不覆盖时，在聊天页的工作区菜单里选“在独立副本中工作”：工作区是 git 仓库（至少有一次提交）时，小伴会在数据目录的 `worktrees/` 里建一个 git worktree 和分支 `xiaoban/<编号>`，对话的所有读写、命令和撤销都在副本里进行，原文件夹不受影响。结束时选“合并回原文件夹”（原文件夹有未提交改动、不在分支上或有冲突时会拒绝并保持原样）、“保留副本”或“丢弃副本”。实现见 `src-tauri/src/chat/worktree.rs`。

小伴界面、装扮、记忆和设计稿工具按各自资源互斥，仅在执行调用时占用，不再锁住其他会话的普通文件任务。短暂占用显示“等待资源”，释放后自动执行原调用，等待可取消且计入运行时间预算；权限确认先于资源占用，撤销快照在取得资源后才记录。文件写入仍保留本轮文件所有权，电脑控制在本轮连续操作期间独占，避免两个会话交错点击和输入。同文件冲突、电脑独占或结果未确认的动作会明确暂停，执行详情包含资源与占用会话；不能安全确定范围的外部工具仍采取保守保护。

复杂任务的 `todo` 清单、里程碑证据和 `handoff` 交接说明独立保存在 `chat/progress`，重启后恢复，并在模型请求中按数据处理。长期目标的“自动验收与恢复”可选项支持指定验收命令：通过原有 Shell 权限执行，在最后一次修改后实际成功、无未完成里程碑或未解决错误时，才自动完成。没有命令时仍由用户验收；命令成功只证明用户选择的检查通过，不保证任务的全部语义质量。

启用“重新打开小伴后继续”后，下次启动会检查之前正在执行的目标，仅对日志完整且没有结果不明动作的安全中断启动新回合。主动暂停、缺失日志和不确定外部动作不自动继续；应用完全退出期间不执行。长命令的 `process_read` 可等待最多 60 秒，保持可取消，减少模型空轮询。

针对性验证：`cargo test --locked --manifest-path libs/agent-kernel/Cargo.toml --features journal --test long_tasks --test guards`、`cargo test --locked --manifest-path tools/agent-reliability-checks/Cargo.toml`，以及本目录的 `npx vitest run src/ui/chat/ChatGoalPanel.test.ts src/ui/components/AgentSafetyCard.test.ts src/ui/chat/chat-store.test.ts src/ui/chat/view.test.ts`。机制回归使用脚本模型与隔离临时目录，不代表真实模型长程任务成功率。

## 工作空间检索

工作空间标题右侧、新建工作区左侧的搜索图标可打开会话检索弹窗，也可按 Ctrl+K / Command+K。支持按标题、聊天正文或工作区名称搜索，按工作区筛选；结果按相关度和更新时间排序，列表限高滚动。没有关键词时显示最近更新的 8 个会话。点击结果会展开所属项目并定位会话。

侧栏默认最多显示 6 个项目，每个分组内默认最多显示 6 个会话，超过后可展开全部或收起；“最近任务”和“伙伴”作为固定分组保留。折叠只影响展示，检索仍覆盖全部已保存会话。检索在本机后台读取用户与小伴的消息正文，不调用模型，不检索隐藏推理、工具输出或附件文件的内部内容；运行中的会话只搜索已保存部分。读取失败会显示提示，连续输入产生的过期结果不会覆盖当前结果。

验证：`npx vitest run src/ui/ControlShell.test.ts src/ui/ControlShell.projects.test.ts src/ui/components/WorkspaceSearchDialog.test.ts`；原生运行 `cargo test --locked --manifest-path src-tauri/Cargo.toml --bin xiaoban-desktop-wallpaper chat::search::tests`。

## 在 Windows 上运行

需要 Node.js 22.12+（或 24）、Rust MSVC 工具链、Visual Studio C++ 构建工具和 WebView2 Runtime。在仓库根执行：

```powershell
cd client/xiaoban-desktop-wallpaper
npm ci
npm run desktop
```

生产构建（仅发布 NSIS 安装版，不再发布独立便携 EXE）：

```powershell
npm run desktop:windows:build
Get-ChildItem src-tauri/target/release/bundle/nsis/*-setup.exe
```

双击 NSIS 安装器后，首次安装只显示「一键安装」按钮和安装进度，不要求选择目录、组件或开机启动。默认安装到当前用户的 `%LOCALAPPDATA%\小伴`，不需要管理员权限，完成后自动打开小伴；开始菜单有「小伴」入口，可在 Windows「已安装的应用」正常卸载。安装器不主动创建桌面快捷方式。NSIS 自动检查及按需准备 WebView2，应用启动时仍执行原有运行时检查。支持 Windows 10/11，WebView2 最低主版本 109。

内部开发使用的 `npm run desktop:build` 仍可生成未打包的 exe 以运行原生 smoke test，但该文件不作为发行资产。

**从旧便携版迁移**：先从托盘彻底退出便携版，再运行一键安装器。既有配置、聊天、壁纸与游戏数据仍使用同一个用户数据目录，不复制也不清空。安装后启动时会把已开启的开机自启动项刷新为新安装路径；如用户在任务管理器手动禁用自启动，保持禁用。首次迁移后手动删除旧便携 exe，避免误启动旧版本。

### 首次运行组件准备

- 运行时可用：直接打开小伴，不显示准备提示、不下载资源；每次检测真实运行时，没有“安装过就永远跳过”的本地标记。
- 运行时缺失或低于最低版本：显示不依赖 WebView 的原生“正在加载必要资源，请稍后”窗口，下载微软官方 Evergreen 引导程序，验证有效 Microsoft Corporation 签名，然后以当前权限静默安装。默认不请求管理员权限。
- 安装后再次检测运行时，成功才创建控制台与角色窗口。安装器退出码为零不等于启动条件已满足。
- 下载、安装、权限或签名问题显示失败原因，允许重试或暂不启动；准备过程中可取消。下载超时 120 秒，安装等待最多 10 分钟，整体进程有超时与退出清理。
- 首次缺组件时需要联网，后续正常启动无需联网。引导程序和脚本放在唯一临时目录，完成或取消后清理；运行时安装到系统/用户组件位置，独立于小伴 EXE 的体积。
- 不支持的旧 Windows 提示升级系统，不尝试下载旧版运行时。macOS 12+ 使用系统 WKWebView，随系统维护，不下载或替换系统 WebKit；最低系统版本已写入 `.app` 配置。

微软官方说明：[运行时按需检测与安装](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/distribution)。正式 NSIS 安装器也会使用 `webviewInstallMode` 检测/安装 WebView2；原生入口保留运行时检查作为安装失败或组件损坏时的兜底。

单独浏览器预览：`npm run dev`，访问 `http://127.0.0.1:1420`。浏览器只能预览角色与场景，不能切换原生桌面模式。

## 在 macOS 上运行（Apple 芯片）

目标为 macOS 12+、Apple silicon arm64。构建需要 Node.js 24、Rust 和 Xcode Command Line Tools。无需安装网页依赖。在本目录执行：

```sh
npm ci
rustup target add aarch64-apple-darwin
npm run desktop
```

构建独立 `.app`：

```sh
npm run desktop:macos:build
open 'src-tauri/target/aarch64-apple-darwin/release/bundle/macos/小伴.app'
```

### DMG 首次安装指引

Release DMG 不再使用默认空白安装窗口。Finder 打开磁盘镜像后可看到浅色中文指引：把真实的「小伴.app」拖向「应用程序」别名；下方介绍首次启动可能遇到的 Gatekeeper 提示，以及在「系统设置 → 隐私与安全性」中由用户主动选择「仍要打开」的方法。

- 指引背景源文件：`src-tauri/dmg/render-background.swift`（使用 macOS 自带 AppKit 绘制；白/浅灰底、粉色点缀，无额外外部资源）。
- `src-tauri/tauri.macos.conf.json` 使用 `bundle.macOS.dmg` 设置 760×440 安装窗口、背景与 App/Applications 图标位置。安装说明不是交互控件；不得渲染伪造的两个图标，Finder 会在背景上叠加真实图标。
- `build.beforeBundleCommand` 每次 macOS 打包之前生成 `src-tauri/dmg/background.png`，已在 gitignore 中排除生成文件。可通过 `npm run test:macos:dmg` 检查配置与文案。
- 本机生成带引导的完整 DMG：`npx tauri build --target aarch64-apple-darwin --bundles app,dmg`；CI 也使用这条命令并检查背景 PNG 尺寸及 `hdiutil verify`。
- **重要**：这是安装说明，不是安全绕过。自签名仍未获得 Apple Developer ID 或公证；Gatekeeper 行为取决于系统版本、下载来源等情况。根据 Tauri 官方文档，部分 CI 环境生成的 DMG 图标位置可能不按设置应用，最终应在 macOS Finder 中打开 Release DMG 做人工验收。
- App 身份、现有自签名证书、Entitlements 和用户数据路径均保持不变。DMG 仅服务首次安装，应用内自动升级仍需另行实现。

应用使用系统 WKWebView，无需打包浏览器。macOS 配置自动读取 `src-tauri/tauri.macos.conf.json`，开启透明窗口并生成 `.app`；本地构建和 GitHub Release 均使用固定自签名证书（归档和密码存放在私有仓库中，构建机准备、安全风险与跨版本验收方法见 [.github/signing/README.md](../../.github/signing/README.md)），签名失败不回退 ad-hoc。`npm run desktop` 每次重编译后先生成并签名开发 App 再启动，开发数据仍独立；直接运行 `cargo run` 或 `npx tauri dev` 会绕过此启动器。尚未使用 Developer ID 签名或 Apple 公证。透明 WebView 目前使用 Tauri 的 macOS 私有透明接口，不以 Mac App Store 分发为目标。

macOS 悬浮窗口使用 AppKit 浮动层；壁纸使用 Core Graphics 的 desktop window level + 1，位于原桌面背景与桌面图标之间，并忽略鼠标事件以保留 Finder 的操作。角色视线使用 NSEvent 的鼠标逻辑点位置，经过 NSWindow/NSView 转换，处理坐标翻转和 Retina 缩放；壁纸点击只有命中桌面层时才触发，Finder 普通文件夹窗口和其他应用遮挡时不触发。角色视线与桌面命中检测不读取屏幕图像；超级小伴截图仅在用户主动点击时读取屏幕，macOS 会按需申请屏幕录制权限。

截图失败（包括 `could not create image from rect`）时，超级小伴会在提示中提供「去设置」，跳转到「设置 → 超级小伴 → 屏幕截图权限」。用户可先打开系统的屏幕与系统音频录制设置检查授权；对于更新导致的历史授权失效，可点击「修复截图权限」，经二次确认后调用固定命令 `/usr/bin/tccutil reset ScreenCapture xyz.xiaban.desktop-wallpaper`。该操作只撤销当前小伴标识对应的 ScreenCapture 记录，不会重置其他应用或其他类型权限；不会自动执行。成功后必须从托盘完全退出、重新打开应用并再次授权。执行失败时会提示在系统设置中手动移除旧记录。此兜底仅修复权限记录，不保证解决屏幕坐标或截图库错误。

配置 `CanJoinAllSpaces` 使角色与壁纸参与各桌面空间；AppKit 窗口操作都在主线程执行。macOS 14+ 会关闭 WKWebView 的后台调度限流以保留可见桌宠动画，旧系统的后台表现需要实机验证。

## 使用

“动态壁纸 → 在线壁纸”使用网站相同的搜索、分类和分页接口；图片与动态视频作为第二层类型筛选。点击卡片在当前页右侧查看预览与适配 / 速度设置，不跳回本地页；“保存到我的壁纸”实际下载并校验后可离线使用，不更换桌面（网站提供来源链接时优先直接下载并按真实文件头检查类型，失败再回退到网站副本，见协议文档）；“应用到桌面”独立执行，未保存时先完成下载。视频无图片封面时显示真实首帧，悬停静音播放。内置官方源站 `https://wallpaper.xiangban.xyz`，侧栏底部提供直达官网按钮；可在设置中开启或关闭自动检查，或手动同步版本与公告。原生后台在启动及每天检查一次，控制台关闭时也继续运行；消息在控制台和托盘提示中展示，关闭同一修订后不再重复提醒。修改后的公告和新版本重新提示。配置、消息与已读状态保存在用户目录，网络失败保留已有内容，手动同步显示失败原因。

网站对接协议见[网站接口文档](../../services/server-xiangban-wallpaper/docs/desktop-api.md)，本应用维护自身的类型与解析，不引用网站源码。原生联调测试 `scripts/verify-website.mjs` 使用隔离测试网站和桌面数据目录，验证真实图片 / 视频下载渲染、更新接口与公告已读状态。

在线壁纸目录等 JSON 元数据请求遇到短暂网关错误时，客户端会有限重试，改善首次访问时偶发的瞬时失败。媒体下载仍沿用 120 秒超时及 SHA-256 校验；重试策略不改变媒体传输路径。

桌面控制台使用 Vue 3.5.43、Naive UI 2.45.3、Vite 8 和 Lucide Vue Next。白色侧栏、红色主色（#FF2442）的布局分为聊天、壁纸库、桌面组件、桌面特效、桌面宠物、游戏空间、关怀提醒等主分区，设置入口固定在侧栏底部；外观可在“设置”里选跟随系统、浅色或深色。桌面宠物保留原 SVG、表情和互动；游戏页点击卡片直接开始游戏，Windows 桌面双击快捷入口在该页独立显示状态并配置。控制台使用 `#2563eb` 强调蓝、`#f5f7fb` 背景与 `#e5eaf1` 边线；游戏空间和关怀气泡采用协调的蓝白浅灰配色。桌面宠物和壁纸页预览见小伴截图（本地：`artifacts/ui/companion-blue.jpg`）及壁纸截图（本地：`artifacts/ui/wallpapers-blue.jpg`）。

账号登录采用每日激活续期：启动客户端、聚焦主窗口或超级小伴、macOS Dock 重新打开时异步检查，按本地自然日每天首次激活刷新一次。登录或成功刷新时间随会话保存在 `config/account.json`，同天重启不会重复刷新；旧版本没有时间字段时首次激活续期。与按需临期刷新共用并发锁；网络失败保留登录，5 分钟内不重复每日续期，下次激活再试，明确失效才退出。后台挂着不定时续期，超过刷新令牌有效期未成功续期仍需重新登录。桌面访问令牌 30 天需要账号服务部署对应配置后才生效。

游戏窗口默认不是全屏，带系统标题栏，可以用系统控制栏最大化或进入全屏。宿主不叠加返回或关闭按钮，游戏运行时返回/退出由游戏自身通过 `game.exit` 协议处理；宿主不抢占游戏的 Escape。退出游戏直接关闭窗口，不再回到游戏列表。系统关闭窗口仍先等待存档写入，保存失败保留游戏并显示恢复选项。

开关类偏好即时生效并反馈结果；壁纸在页面内预览，用户确认“应用到桌面”后才更换桌面。关怀提醒通过明确的保存动作提交编辑。控制台入口由 `src/main.ts` 动态载入 `src/control.ts`；`control.ts` 负责挂载 `src/ui/App.vue`，控制台页面、状态与组件分别位于 `src/ui/pages/`、`src/ui/control-store.ts` 和 `src/ui/components/`。它们与宠物窗口、游戏沙箱的入口分离。

设置页的“AI 模型”标签用来填写模型服务：接口协议（OpenAI Chat Completions、Responses 或 Anthropic Messages）、API 地址、可选的密钥和模型名、工作目录路径，以及“隐藏工具调用信息”、“启用电脑屏幕与应用控制（open-computer-use）”（开启后随应用附带的操作规则 `../../libs/agent-tools/computer-use/skills/open-computer-use/SKILL.md` 会写入助手的技能目录，关闭时移除；固定版本 1.1.2 的 Open Computer Use 原生引擎随安装包分端分架构提供，用户无需安装 Node.js/npx；构建时下载校验并只提取目标资源，macOS 辅助 App 裁剪架构并重新签名，Android 不包含桌面引擎。工具清单以 MCP 服务器返回的为准，不在提示词里手写；服务器没能启动时，原因会告诉模型，由它如实转述）等切换，地址下方会显示实际请求的完整地址。这些内容保存在用户数据目录的 `config/ai.json`，与 `preferences.json` 分开：偏好会广播给所有窗口，密钥只由原生端读取，网页端读回的只有“是否已保存”和末四位。文件没有加密，类 Unix 系统上仅当前用户可读写，Windows 上沿用目录权限，该情形未验证。只有控制台窗口能读写这些设置（`get_ai_config`、`save_ai_config`）。“AI 模型服务商”卡片网格里排第一的“小伴 AI”卡片对接 AI 网关 `https://ai.xiangban.xyz`（开发时可用环境变量 `XIAOBAN_AI_URL` 指向别处）：卡片和其他服务商一样有两个按钮：“配置”打开弹窗（登录后显示余额与到期时间、用兑换码充值、选择使用模式），“启用此服务商”直接把聊天切到预选的档位（未登录时提示先登录并打开弹窗）；正在使用时显示“正在使用中”（`src-tauri/src/ai_gateway.rs`）。壁纸网站后台“模型供应商”页顶部有“小伴 AI 入口”开关，关闭后目录里 `xiaobanAi.enabled` 为 false：桌面端下次刷新时隐藏这张卡片、拒绝启用，正在用它聊天的会提示改选其他服务商；目录里没有这个字段（旧站点）按开启处理。网关生效时 `active_provider` 为 `xiaoban-ai`，`ai.json` 里只存档位名，不存令牌；每次发送消息前由原生端取有效的账号访问令牌（过期前自动用刷新令牌更换），长对话里每次模型请求都会重新取，令牌不进入网页层。网关的错误（额度用完、需先兑换付费额度）会把网关写给用户的那句话显示在聊天里；未登录时发送会提示先登录。档位与真实模型的对应只在网关后台配置，客户端看不到。小伴 AI 面向用户只显示“日常”（默认，响应更快、额度更省）与“深入”（适合复杂任务，耗时和额度更多）两种使用模式，不提供额外思考强度设置；名称、说明、默认项与实际模型由 AI 后台维护，保留已有档位标识以兼容已安装版本。控制台侧栏首项为“聊天”入口，采用左侧历史对话列表、右侧 ChatGPT 风格对话区的两栏布局。列表每行右侧的“⋯”菜单提供重命名、移动和删除（正在回答的对话不能删除）；列表顶部可新建分组，分组可折叠、重命名和删除，删除分组不会删除里面的对话，只是让它们回到“未分组”。分组和自己起的对话名保存在 `chat/index.json`，索引损坏时会从对话文件重建，此时分组和自定义名称会丢失，对话内容不受影响。输入框旁的圆环显示这个对话下一次请求占用的上下文，点开是三行浮层：第一行是“已用 / 总上下文”和进度条（70% 变橙、90% 变红），第二行是上限滑块（16K 到 1M，默认 256K；拖动时数字实时变化，停下约 0.35 秒后保存到 `config/ai.json` 的 `contextTokens`，下一条消息起生效），第三行是“智能压缩”。每个对话各自保存上下文；超过上限时内核只在发给模型的那一份里先清掉较早的工具输出、再整轮丢掉最早的内容，保存的对话和界面上看到的始终是完整的。“智能压缩”让当前模型把较早的对话写成一份摘要（用的就是聊天正在用的模型，用小伴 AI 时会消耗额度），最近一轮问答原样保留；摘要作为一条带标记行的消息追加在对话文件末尾（`src-tauri/src/chat/compact.rs`），之后模型看到的是摘要加最近内容，界面里在该处显示可展开的“查看摘要”。回答期间不能压缩，压缩期间不能发送；摘要不比原文短时不会写入；一次摘要最多读入约 12 万 token 的对话记录。自动压缩：每次回答成功结束后，如果下一次请求的估算用量已达到上限的 90%，主机会在同一条线程里自动做一次同样的压缩，期间圆环按钮显示“正在压缩…”（事件 `chat-compacting`）；失败时提示一次原因，该对话 10 分钟内不再自动重试，手动压缩不受影响。自动压缩只在回答之后做，一次回答内部用量涨过上限时仍由内核按上述方式省略较早内容。用量估算包含文字、图片、工具定义、回复预留和 5% 余量。工具定义在每次构建 Agent 时按实际注册项校准，本次启动尚未注册时暂留 32K；图片按保守基数和编码长度估算，并非实际 tokenizer。请别把上限设得超过所用模型的窗口；有效的旧版较小设置原样保留。三种协议均限制实际请求体不超过 8 MiB，必要时省略旧工具截图并明确标注，仍放不下则提示减少本次图片或压缩会话。明确的上下文超限会在未流出回复时缩减请求内容并最多恢复两次，原始会话和已执行操作保持完整；普通参数错误不会盲目重试。小伴作为通用桌面 Agent，支持通过内置工具操作当前电脑（在指定工作目录内读取/写入/编辑文件、列出目录、搜索文本、运行 shell 命令、获取网页、维护任务规划清单、读取/写入持久记忆笔记与技能），并具备小伴桌面本身的控制能力（打开游戏空间玩小游戏、查看桌面状态、切换壁纸、开关粒子特效、按格式创作新的桌面特效包、放置与调整桌面组件、调整小伴外壳与表情、增删改查关怀提醒）。每个对话在自己的工作区（文件夹）里工作，入口在聊天输入框下方、权限模式旁边（`src/ui/components/ChatWorkspaceMenu.vue`、`src-tauri/src/chat/workspace.rs`）：菜单列出最近用过的文件夹（`chat/agent-settings.json` 的 `recentWorkspaces`，最多 8 个）、默认工作区、“选择文件夹…”和“在文件管理器中打开”。新对话默认用上次用过的文件夹；设置里的“默认工作区”只在第一次和较早的对话（升级前创建、没有保存文件夹的）里使用。还没发消息的对话直接切换；已有消息的对话会先问：默认“新开对话”，旧对话保持原来的文件夹，也可以选“在本对话里切换”，这时对话里会记一条“工作区已切换”的说明，提醒模型旧路径不再有效。回答进行中不能切换。选到用户主目录或磁盘根目录会提示范围太大，在里面删除文件前都会先问。危险命令检查、后台进程、撤销记录、项目规则和上下文估算都按对话各自的文件夹计算，两个对话可以同时在不同文件夹里工作；连接器（MCP）和定时任务仍在默认工作区运行。聊天支持“默认权限”（读操作自动，写文件、运行命令与系统调整前需确认）与“完全信任”两种模式，在聊天顶栏切换，每个对话各自保存；新对话沿用顶栏当时显示的模式。“完全信任”不再逐一询问，只在遇到高风险操作时才会停下来问（`src-tauri/src/chat/danger.rs`）：提权（sudo）、在工作目录之外递归或批量删除、格式化磁盘、关机重启、`git push --force` 与 `git reset --hard` 等会丢改动的 git 操作、下载后直接执行、读取 SSH 与云服务密钥等凭据、向外部发送数据或文件、修改 shell 配置与系统服务、删除数据库或云资源、发布到公共仓库；确认卡片会写明原因，且每次都问、不提供“这段对话都允许”。工作目录之内的操作（如 `rm -rf node_modules`）不打扰。这是对命令行的启发式判断，不是沙箱：脚本文件里藏的命令、`python -c` 之类的代码、混淆过的命令都识别不了。“完全信任”下，读过网页也不再强制逐步确认，也不再单独询问技能的改动；“默认权限”下这两条保持原样（污染后每步都问，技能改动每次都问）。用户亦可随时通过停止按钮中断回答。

动态壁纸支持图片、静音循环 H.264 MP4 与自包含 Canvas/WebGL ES module。可添加 HTTPS 订阅或导入 JSON 清单；下载后的内容校验 SHA-256 并缓存，离线可继续使用。小伴悬浮时可与通用壁纸并存，融入壁纸时叠在所选内容上；“小伴房间”恢复原先的房间场景或系统桌面。下发协议、SDK、限制和更新 JSON 示例见 [delivery-protocol.md](docs/delivery-protocol.md)，Schema 为 [壁纸](docs/wallpapers-v1.schema.json) 和 [更新](docs/updates-v1.schema.json)。

内置壁纸在 `public/wallpapers/builtin/`，目录 `catalog.json` 记录每个文件的大小和 SHA-256。除早期的晴山、晨潮、浮光和一池心灯外，新增六张程序绘制的 Canvas 壁纸，离线可用、不依赖任何外部素材：暮海、极光之夜、云海、山水（宣纸水墨）、糖霜微光（弥散光与玻璃球）、枫林暖阳。它们的共同做法是：静态层只在窗口尺寸或参数变化时画到离屏画布，每帧只绘制流动的部分，最后叠一层颗粒来增加质感；不使用 `ctx.filter`、`roundRect`、圆锥渐变等较新的画布接口，兼容 macOS 12 的 WebKit；参数只用滑块、取色器和开关，因为壁纸页的参数面板不渲染下拉选项。在没有 GPU 的无头浏览器里按 2872×1716 逐帧计时，六张约 12–50 毫秒一帧，不比原有的浮光重；真机帧率和耗电未测。新增或修改素材后运行 `node scripts/check-builtin-assets.mjs --write` 刷新校验值，`all_builtin_wallpapers_are_valid` 会校验整份目录。封面是用真实场景渲染出的 WebP，改动画面后需要重新生成。

### 已实现能力（PRD 第 1–3 项）

- **游戏空间**：设置页和托盘/菜单栏的“玩一玩”打开游戏空间页；点任意游戏卡片会打开一个只运行这一款游戏的普通窗口（默认不全屏，可用系统标题栏切换全屏），退出即关闭窗口。离线展示七款本地游戏包。资源协议沿用网页发布包的 manifest 与 v1 消息格式；桌面使用自己的宿主，iframe 仅授予脚本权限并校验消息来源、nonce、游戏 ID 和回合状态。桌面不接账户分享；网页专属分享和钱包入口由本地 adapter 隐藏。内置游戏只有水墨剑客保存进度，位于用户数据目录 `games/saves/ink-blade.json`，上限 32 KiB；用户自创游戏（“文件夹即游戏”，放在 `games/packs/<id>/`，由 `game-pack:` 协议提供，聊天里的 `game_create`/`game_delete` 可创建与删除）在 manifest 声明 `save: true` 时存为 `games/saves/user-<id>.json`，同样上限；读取或保存失败会保留原文件并提示处理。Windows 原生 smoke 已验证七包 ready、存档往返、Esc、IPC 隔离和桌面状态恢复。桌面空白双击入口仅 Windows 可用，默认关闭；只读低级鼠标钩子将事件交给异步队列并判断 Explorer 桌面列表空白区域，不吞鼠标输入。Windows 真实图标交互尚待人工验收。macOS 暂无可靠桌面空白双击判断，使用显式入口。
- **小伴显示**：小伴设置中可单独关闭角色显示，默认开启。关闭时仍可使用壁纸、图片特效与关怀提醒；提醒气泡独立显示在当前应用上方。
- **天气来源与定位**：关怀提醒的天气来源默认是 Open-Meteo（可在“提醒偏好”里改成自己的来源或关闭）。新建天气提醒时按网络位置预先填好城市：这一步会把公网 IP 发送给 ipwho.is（失败时用 api.ip.sb），只用来判断所在城市，随时可以搜索改成别的城市；来源关闭后不再定位。
- **应用图标**：红底圆角方块，叠放的图片卡片、爱心与白字“小伴”。源文件 `src-tauri/icons/icon-source.png`（1024×1024，四角透明，四周约 10% 留白），`icon.icns/.ico/.png` 由 `npx tauri icon src-tauri/icons/icon-source.png -o <临时目录>` 生成后只复制这三个文件，托盘用同一图标的 64×64 RGBA 原始像素 `icons/tray.rgba`（由生成的 `64x64.png` 解码得到）。原图只有 360×360，大尺寸图标是放大得到的，有更高分辨率的原图时应替换源文件后重新生成。
- **关怀提醒**：控制台提供关怀规则编辑，气泡由独立的 `mountCareBubble` 窗口展示；规则和调度在原生宿主运行，控制台隐藏时仍可触发。气泡显示在当前应用所在屏幕右上角，Windows 使用不抢焦点的置顶窗口，macOS 加入所有桌面空间与全屏辅助空间；桌宠隐藏不阻止提醒。下班/自定义按本地时间一次、每天或指定星期调度；休息规则只在平台活跃时间可读、未锁屏且空闲少于 60 秒时累计。提醒存于 `config/care-reminders.json`，支持预览、稍后、免打扰和游戏期间暂缓。语音播报默认关闭，开启后用声音设置里的 Edge TTS 音色朗读文案，需联网并向该语音服务发送文案；普通气泡 10 秒收起，语音气泡 30 秒收起。Windows care smoke 已检查配置持久化、气泡焦点、关闭/稍后、游戏延后与过期处理。系统通知目前不可用，应用完全退出后不承诺后台触发。天气来源默认是 Open-Meteo（个人非商业使用，可改成自有兼容 HTTPS API 或关闭）；live 天气查询尚未验收。
- **小伴感知本机状态**：原生宿主复用关怀提醒每 5 秒读取的空闲 / 锁屏 / 活跃时间，把结果交给 `src-tauri/src/presence.rs` 的状态机，向小伴窗口发出 `companion-presence` 事件（`welcome_back`、`first_today`、`late_night`），前端 `src/companion/presence.ts` 用现有表情回应，只改变表情，不弹窗、不发声、不显示文字。离开满 10 分钟后重新有操作打招呼，每天最多 3 次、两次间隔至少 30 分钟；本地日期的第一次活动问候一次（日期保存在 `config/presence.json`，读写失败按未问候处理）；00:00–05:00 累计活跃 20 分钟露出困倦表情，每晚一次。遵守关怀提醒的总开关与免打扰时段，小伴隐藏或游戏空间打开期间不触发，且不补发；锁屏或睡眠后恢复只按“离开后回来”处理一次，时钟回拨不会重复触发。不新增采集、不读取用户在做什么、不联网。仅 Windows / macOS 能读取活跃时间，其他平台没有此功能；睡眠恢复后若系统没有报告足够长的空闲时间，则不打招呼。锁屏 / 睡眠恢复尚未实机验证。
- **超级小伴**：控制台侧栏的“超级小伴”页（聊天页左下角也有入口）管理它。默认持续沿用上一次助手会话（包括重启后）；横排菜单的“新会话”用于主动切换并记住新会话，旧记录保留。助手回复和完整聊天页支持选中文字后右键复制。两种形态共用一个 `assistant` 窗口（`?scene=assistant`，`src/assistant/`，宿主在 `src-tauri/src/float_assistant.rs`）：屏幕边缘常驻的**悬浮球**（可在页面里关闭，点一下展开、按住拖动、右键打开设置页，位置会保存），以及展开后的**胶囊**，直接说话或打字提问，回答在胶囊上方浮出。全局快捷键（默认 Option+Space / Alt+Space，四选一）随时开合胶囊；它一直停在最上层（macOS 用“状态栏”窗口层级），只有再按快捷键、Esc 或工具栏的“收起小伴”才收起，收起后回到球（没有球就隐藏）。窗口三态 `Hidden/Ball/Capsule` 由宿主维护，页面只画；球与胶囊各记各的位置。设置页可调：启用、悬浮球、快捷键、重置位置、语音提问开关、呼出后先听还是先打字、朗读回答（只在语音提问时/总是/不读）。复用聊天、朗读、语音识别三套 store；命令权限按窗口标签收窄（助手只能提问/回答/朗读/识别），聊天事件用一次 `emit_filter` 同时发给设置窗口与助手窗口（不能各发一次：全局 `listen` 会收到两遍）。首次启动后提示一次快捷键。浅色/深色由主程序保管（`Settings.appearance`，命令 `set_appearance` 仅控制台可调），经 `settings-changed` 同步给超级小伴窗口：各窗口的 `localStorage` 互相隔离（真实 WebView 里实测，控制台写入后超级小伴读到的仍是空），所以不能靠 `storage` 事件同步；老用户本机已有的选择在第一次启动时由控制台一次性上报给主程序。
- **小伴外壳**：控制台“桌面宠物”页的“外壳”目前只保留盆栽一款（猫耳、兔耳、熊耳、哪吒都试做后移除，保存过这些的会回到盆栽），立即生效并保存（`Settings.companionSkin`，未知值回退为盆栽；命令 `set_companion_skin` 仅控制台窗口可调用）。外壳在 `src/companion/skins.ts` 里定义，只包含耳朵形状和配色，屏幕、符号位置和部件标记（`data-part`）与原来一致，所以表情、耳朵动作、点击命中和拖动不受影响。这是原型：三款外壳的观感只在浏览器里逐表情看过，没有在桌面上实机看过，也没有逐一核对 25 种表情在兔耳下的耳朵动作；兔耳外壳整体缩到 0.82 倍，给长耳朵留出空间；“盆栽”外壳不是机器人的头：机身是一个白瓷花盆（厚盆沿、盆口露出泥土和小石子、向下收口的盆身，没有托盘），表情屏嵌在盆身正中，一根茎从土中央长出，顶端分成两片叶子。两片叶子沿用耳朵的角度和发光来表达情绪，旋转中心在茎顶（`Skin.earPivot`），所以开心时竖起、难过时耷拉，都绕着茎动。这款在 `skins.ts` 里用 `art` 字段替换了机身轮廓、泥土和茎，其余外壳不用；整体缩到 0.76 倍，给植物留出空间。
- **桌面特效**：控制台的“桌面特效”页选一种特效——雨、雪、萤火虫、花瓣飘落、枫叶飘落、流星划过、繁星，或“无特效”——每种特效各自记住密度、速度、大小三个滑块，选择立即生效并保存。它是桌面上独立的一层透明、鼠标可穿过的窗口（窗口标签 `effects`，页面 `?scene=effects`），不属于任何壁纸：换壁纸、用系统桌面、壁纸是互动动画时都照常显示，也不会随壁纸保存。macOS 上它在动态壁纸窗口上方一级、仍在桌面图标之下；Windows 上它是置于所有应用窗口之下的顶层透明窗口。游戏空间打开期间隐藏，退出后恢复；全局“暂停桌面动画”和帧率同样适用。
- **特效包**：每种特效是一个 JSON 文件（“特效包”），描述有几层粒子、数量、大小、速度、风与摇摆、翻转、闪烁、颜色，甚至形状（SVG 路径），由统一的引擎读取并绘制，包里不能写代码。内置十二个在 `public/effects/`；自己的特效包有三种进法：在“桌面特效”页点“导入特效”选一个 JSON 文件（先检查再落盘，id 与已有特效重名时会先问是否替换，取消则不写入）、打开特效文件夹手动放进去再点“重新载入”，或者直接在聊天里让小伴写一个（`effect_create`），三种方式都不用重启。格式和限制见 [docs/effects.md](docs/effects.md)，示例见 [docs/effect-pack-example.json](docs/effect-pack-example.json)。引擎在 `src/effects/`（格式与检查 `types.ts`/`parse.ts`、模拟 `sim.ts`、绘制 `draw.ts`、画布 `layer.ts`），窗口与文件夹管理在 `src-tauri/src/effects.rs`、`effect_packs.rs`（导入、结构校验与落盘都在这里，`validate` 是唯一的检查点），选择的数据模型在 `effect_choice.rs`。旧版本保存在每张壁纸里的特效，在首次启动时迁移为这个全局选择（旧的“树叶”变成“枫叶”）。Windows 上的覆盖窗口和 effects smoke（`XIAOBAN_EFFECTS_SMOKE=1`，已改为验证覆盖窗口和全部内置特效）尚未在实机验证。

PRD 第 1–3 项已实现；2026-10-10 的 Windows 自动化检查记录不代表当前版本的完整验收。专注、玩具互动、场景收藏、昼夜变化和音乐联动（第 4–8 项）仍是规划。真实图标双击、天气 live 查询、网站服务联调、视觉与性能验收仍待完成；逐项结果见 [feature-validation.md](docs/feature-validation.md)。

“导入图片 / 视频”通过 Windows/macOS 原生文件选择器添加自定义壁纸，“我的导入”筛选自己的内容。支持 PNG/JPEG/WebP/GIF/BMP 图片（最多 64 MiB）与 MP4/M4V/WebM 视频（最多 1 GiB），视频推荐 H.264 MP4，其他编码由系统 WebView 决定。导入保存副本并按内容去重，移动原文件后仍能使用；可预览、应用、选择画面适配和调整视频速度，不需要制作 JSON 清单，不内置转码器。

Canvas 运行在独立的低权限壁纸窗口和 opaque-origin iframe 中，不能调用设置、自启动、退出或下载等桌面命令。控制页只展示封面，执行预览使用另一低权限窗口。视频缓存协议支持 byte ranges。更新功能目前为版本/说明检查和打开下载页；未配置更新源会显示未接入，不伪造最新状态，未实现无签名自动安装。

- 控制台切换“悬浮桌宠 / 动态壁纸”，调整 25 种表情、160–420px 尺寸、深夜/晨光背景、30/60 FPS，或暂停动画。
- 悬浮模式：点击角色触发反应，拖动角色移动窗口，右键角色打开控制台。角色实际 SVG 填充区域接收点击，暖光与空白处穿透。
- 壁纸模式：小伴在背景的桌面上，角色上方的桌面点击可以触发反应。应用不拦截桌面鼠标消息，桌面图标继续由 Explorer / Finder 处理。其他窗口遮挡角色时不触发壁纸点击反应。
- 角色视线的鼠标位置由平台原生层以约 30Hz 采样，即使离开角色窗口也能跟随；该采样不使用鼠标/键盘钩子，也不采集键盘输入或屏幕图像。若用户在 Windows 开启“桌面空白双击打开”，游戏入口另使用只读 `WH_MOUSE_LL` 钩子判断桌面点击，不吞输入；默认关闭。macOS 不安装该钩子。
- 40 秒无鼠标活动后困倦，90 秒后睡觉；这是原版逻辑，鼠标活动会唤醒。
- 关闭设置窗口或点“收起设置”只隐藏设置面板，小伴继续运行。Windows 左键点击托盘图标打开设置，右键显示菜单；macOS 点击菜单栏图标后选择“打开小伴设置”，也可以通过 Dock 唤起。
- 托盘/菜单栏和设置面板均有“开机自动启动”开关。首次正式运行默认开启；关闭后保存选择，下次启动不会自动重新开启。登录启动时恢复上次的悬浮/壁纸模式、表情、大小、背景、暂停和帧率，只显示小伴，不弹出设置面板。
- 再次打开程序会唤起已有实例的设置，不创建第二只小伴；“退出小伴”才退出全部窗口，恢复原本可见的壁纸。
- macOS 点击 Dock 中的小伴图标会显示、取消最小化并聚焦设置面板。壁纸模式中同样有效，不会切换角色模式或重建壁纸窗口。

## 桌面组件：日历、便签和纪念日

组件库新增「心事有处」（便签待办）和「好日将至」（倒数日／纪念日），与求签、时钟日历、天气一起沿用多实例、缩放、主题、桌面摆放和本地恢复机制。

- 日历显示农历、传统节日、所选日期详情以及官方「休／班」标识。客户端独立维护 `src/widgets/holidays.json`，目前收录国务院公布的 **2026 年**安排；其他年份明确提示未收录，不推测调休。数据来源链接写在 JSON 中，每年需要随客户端更新。月历扩大为 300×400 基准尺寸以容纳农历；点击月份回到今天。
- 便签可切换文字/待办，标题最多 40 字符，正文最多 1200 字符，最多 12 项待办（每项 120 字符）。文字和标题修改后点击「保存」，也可在正文中按 Ctrl/Cmd+Enter；增删、完成待办和切换视图会保存当前内容。保存失败保留草稿并提示重试。正文区域用于输入和选择文字，按住组件顶栏拖动；摆放模式会锁住内容操作。控制台选中便签后也能编辑。
- 倒数日支持一次性事件、每年重复与纪念日正计时。控制台选中组件后设置名称、日期与模式，再点击保存。年度 2 月 29 日在平年按 2 月 28 日纪念，保留原日期；纪念日正计时不支持年度重复。按本机日期计算天数，跨夏令时也按自然日计算；每 15 秒以及窗口重新可见时更新。
- 数据保存在现有 `config/widgets.json`，旧版三个组件无需迁移。原生窗口只可保存自己的便签/日期内容，不能传入其他组件 ID、样式或布局。布局读取/保存上限从 32 KiB 提高到 128 KiB，保持长度检查及保存失败不覆盖内存状态。新版布局含新类型，旧版客户端不支持读取，降级前应备份数据。

组件支持多屏摆放：拖动时使用原生鼠标位置确定目标显示器，保留抓取点比例；macOS 直接使用 AppKit 全局坐标，Windows 使用物理像素并补偿各屏 DPI。控制台可切换显示器预览，也可通过「所在显示器」直接移动选中组件。每个组件保存显示器标识和屏内相对位置；断开外接屏时显示在主屏，重新接回后恢复原位置（macOS 标识含屏幕名称与布局位置，重新排列或更改名称时可能回退到主屏，可重新选择）。显示器配置每 3 秒检查一次，仅在变化时重新摆放。此能力只扩展组件，不扩展主屏壁纸。

新闻和 A 股组件尚未实现，候选数据源、三大指数/自选股布局和刷新建议见 [资讯组件调研](docs/information-widgets-research.md)。

本轮针对性验证：`npx vue-tsc --noEmit`、`npx vitest run src/widgets`。新增 Rust 校验/权限回归用例，当前环境没有 Rust 工具链，未运行原生构建或 Windows/macOS 实机验收；需验证桌面输入焦点与文件持久化。浏览器预览环境缺少 Chromium，未完成截图审稿。

### 长输出恢复与游戏草稿

工具调用因输出超限而不完整时，内核拒绝整批执行，并自动要求拆分步骤；界面只显示重试状态，恢复成功后不遗留截断警告或“继续”按钮。无新工具进展时最多自动重试 3 次，再暂停供用户继续；纯文字达到上限时由 ContinueGuard 续写，最终截断才显示警告。既有成功操作不会重放。

`game_create` 兼容原有 `files`，新增与其二选一的 `source_dir`。较长游戏先在当前工作目录的专用子目录（如 `game-drafts/my-game/`）用 `write_file` 分文件创作、`edit_file` 局部修改，再传 `id/name/description/source_dir` 安装，避免重复传输整包源码。草稿保留供之后修改；它只接受工作目录内的普通 UTF-8 文本文件，拒绝越界、符号链接、manifest 与非支持类型，沿用 40 文件 / 单文件 1 MiB / 合计 4 MiB / 路径 4 层限制。校验通过后复用原子安装流程；无效草稿不会替换已安装游戏。保存成功不代表玩法或视觉已经实机验证。

自定义及第三方服务商使用 Responses / Anthropic 协议时，可在配置弹窗的“高级设置”单独设置单次输出上限（1024–131072 token）；留空优先读取模型目录可选的 `maxOutputTokens`（自动最多 32768），未知模型保守使用 8192，不根据模型名称猜测能力。显式设置仍受模型声明上限和上下文一半的约束，真实请求与上下文预留使用同一有效预算。更换模型、地址或协议时恢复自动；目录更新更换推荐模型时也会清除旧设置。小伴 AI 继续采用保守预算；Chat Completions 保留端点默认输出参数，暂不提供此控件。旧配置可直接读取。

针对性验证：`npx vitest run src/ui/chat/view.test.ts src/ui/chat/chat-store.test.ts src/ui/components/AiConfigCard.test.ts`、`npx vue-tsc --noEmit`；原生运行 `cargo test --manifest-path src-tauri/Cargo.toml --locked --bin xiaoban-desktop-wallpaper chat::`、相同命令筛选 `ai_config::` 与 `ai_catalogue::`。内核回归测试在仓库根运行 `cargo test --manifest-path libs/agent-kernel/Cargo.toml --locked --features openai,anthropic --test truncation --test guards --test anthropic --test responses --test openai_http`，使用脚本模型与本地假服务，不消耗生产模型额度。

## 验证

正式用户数据固定在 `~/.xiaobanwallpaper`。`config/preferences.json` 保存设置，`wallpapers/imports` 保存自定义媒体的持久副本，`wallpapers/local.json` 保存本地壁纸元数据，`wallpapers/cache` 保存订阅资源，`cache/webview` 是 Windows 浏览器运行数据。“关于小伴”提供打开目录按钮；首次迁移复制旧配置与缓存、保留原数据和备份，不覆盖已有新配置。完整子目录职责和迁移约束见 [user-data.md](docs/user-data.md)。设置通过临时文件替换保存。Windows 使用当前用户 `Run` 启动项 `XiaoBanDesktop`，兼容带空格/中文的 EXE 路径，并识别任务管理器中的禁用状态；macOS 使用 `~/Library/LaunchAgents/XiaoBanDesktop.plist`，随用户登录启动。便携 EXE 或 `.app` 移动后，手动打开一次可刷新已启用的启动路径；macOS 建议放到 Applications，避免下载目录或临时路径失效。登记失败会在设置中提示，不阻止桌宠运行。

开发预览不自动登记自启动，使用 `.xiaobanwallpaper/development` 独立数据根。原生 smoke test 使用独立应用标识、临时数据根与 `XiaoBanStartupSmoke-<测试标识>` 启动项；Rust 注册测试使用 `XiaoBanStartupTest-<PID>`。结束时仅清理自己的测试项，不修改正式启动项。两端媒体测试额外导入真实 PNG 与超过 16 MiB 的 MP4，删除测试源文件后验证播放、去重与重新加载；不读取用户图片或视频。

```powershell
npm run build
npm test
powershell -ExecutionPolicy Bypass -File .\scripts\smoke-test.ps1
cargo build --manifest-path src-tauri/Cargo.toml --locked
powershell -ExecutionPolicy Bypass -File .\scripts\smoke-runtime.ps1 -VerifyDownload
node scripts/verify-media.mjs
```

角色测试运行本工程的 12 个行为测试，独立于网页与后端业务目录。原生 smoke test 启动生产程序，在悬浮、壁纸、返回悬浮三个阶段检查窗口存在性、前端显示、背景父窗口类别、WS_CHILD 标记和旧窗口清理，输出 `artifacts/smoke-*.json`。Windows 的小伴房间与六种媒体壁纸用例同时检查实际客户区的物理尺寸和屏幕原点，要求与主显示器完全一致；壁纸窗口关闭系统阴影，避免无标题栏窗口的边框占用绘图区。桌面 parenting 使用 Explorer 的非公开 WorkerW 协议，需要在不同 Windows 版本上继续验证。

macOS 验证：

```sh
cargo test --manifest-path src-tauri/Cargo.toml --locked
npm test
npm run desktop:macos:build
npm run test:macos
```

四个坐标/层级测试覆盖上下原点、逻辑点、非零 bounds、窗口外鼠标和普通应用窗口遮挡判断。macOS 原生 smoke test 等待真实前端挂载握手，验证透明浮动窗口、壁纸层级、输入穿透、桌面空间配置、两次模式切换与旧窗口清理，输出 `artifacts/smoke-macos.json`。GitHub 工作流 `.github/workflows/xiaoban-desktop.yml` 在 Apple 芯片的 macOS 15 构建机上执行这些检查，并生成包含 `.app` 的 ZIP 和包体积记录。

Dock 回归通过 LaunchServices 真正重新打开已运行的 `.app`，分别验证设置面板最小化、面板隐藏、整个应用隐藏时都能恢复焦点，同时保留壁纸窗口与壁纸模式。测试同时核对壁纸原点和尺寸与主屏幕对齐。

Mac 实机还需检查：原角色的发光与连续表情、桌宠拖动、空白处点击穿透、壁纸上的角色点击、桌面图标操作、桌面空间切换和全屏应用行为。CI 的原生窗口属性验证不能代替这些视觉与操作检查。

## 当前范围与限制

新增功能规划见 [小伴桌面 PRD](../../docs/product/PRD_小伴桌面.md)。第 1–3 项已实现；早期 Windows 自动化结果见历史验证记录，第 4–8 项仍规划中。实际能力及平台限制以本文和代码为准。

启动流程另有四个 Rust 测试，覆盖已安装时不调用安装器、安装后重新检测、虚假成功退出码与失败重试。Windows CI 构建 NSIS 一键安装包；内部原生准备窗口的成功/失败/取消模拟仍需单独验证；本地 `-VerifyDownload` 还验证内嵌下载工具、进程清理及真实微软签名，不执行安装。真实缺组件的首次安装需在干净 Windows 虚拟机上验证。

- 提供 Windows 与 macOS 的原生接入；macOS 包面向 Apple 芯片，Intel / Universal 打包未纳入此原型。
- 壁纸只覆盖主显示器；未实现多显示器管理、视频转码或签名安装包。
- Windows 壁纸窗口作为 Explorer 背景 WorkerW 子窗口；macOS 壁纸使用 AppKit 桌面层窗口。未修改系统壁纸路径或偏好，退出后原壁纸恢复显示。
- 模式切换会重新挂载角色，当前的随机动画/睡眠进度会重置。
- Explorer 重启、显示器热插拔、macOS Stage Manager / 全屏空间等恢复策略需要进一步实机完善。Windows 背景父窗口异常会提示重新切换模式。
- 点击穿透通过原生采样与实时 SVG 轮廓判断切换，存在约一个采样周期的延迟。
- 这是还原路线验证原型，未对跨浏览器逐像素一致性作保证。

## 代码

- `src/companion`：本应用角色引擎、SVG 和测试；`src/pet.ts`：DOM 帧更新与 SVG 轮廓命中测试。
- `src/scene.css`：原小伴房间的程序背景；内置图片、视频和 Canvas 样例位于 public/wallpapers。
- `src-tauri/src/main.rs`：控制台、两种窗口生命周期、托盘、命令与诊断。
- `src-tauri/src/preferences.rs`：本应用设置、兼容旧字段、参数修复与文件保存；`src-tauri/src/startup.rs`：Windows 当前用户注册、macOS 登录启动与测试项清理。
- `src-tauri/src/storage.rs`：正式/开发/测试数据目录、旧数据迁移与备份；`src-tauri/src/local_media.rs`：原生选择器、媒体格式检查、流式复制、去重与本地目录元数据。持久导入副本与可重新下载的缓存不能混用。
- `src/main.ts` 按窗口场景动态载入入口；`src/control.ts` 仅负责控制台 Vue 应用挂载。`src/ui/ControlShell.vue`、`src/ui/pages/`、`src/ui/components/`、`src/ui/control-store.ts`：控制台布局、分区页面、展示组件与状态；`src/ui/wallpaper/`：壁纸页面状态逻辑；`src/ui/website-types.ts`：网站 API 类型；`src/ui/control.css` 和 `src/ui/theme.ts`：控制台主题；`src/style.css`：宠物、游戏和气泡等原生场景宿主样式；`src/wallpaper-media.ts`：视频与图片预览、首帧回退和可见卡片加载；`src-tauri/src/website.rs`：网站原生 HTTP、后台检查与公告已读状态；`src/content.ts` 与 `public/wallpapers/sandbox.html`：图片/视频与 Canvas 沙箱宿主；`src-tauri/src/library.rs`：目录、资源完整性、缓存协议和更新验证；`src-tauri/src/content.rs`：窗口切换、权限边界和交付命令。
- `src-tauri/src/runtime`：Windows 启动前检测、原生准备窗口、官方引导程序下载与签名验证。成功/失败/取消的 UI 模拟仅在 debug 构建中启用，不会安装组件；真实缺组件的首次安装还需干净虚拟机验证。
- `src-tauri/src/windows.rs`：WorkerW 接入、DPI 坐标转换、只读鼠标采样。
- `src-tauri/src/macos.rs`：AppKit 窗口层级、空间行为、鼠标逻辑点与桌面点击判断。
- `src-tauri/src/coordinates.rs`：坐标转换和桌面层判断的独立测试。

## 工具 hooks

可以让自己的命令在小伴调用工具前后运行，类似 Claude Code 的 hooks。在应用数据目录的 `chat/hooks.json` 中配置（与 `chat/sessions` 同级），每次工具调用都会重新读取，修改后立即生效：

```json
{ "hooks": [
  { "event": "before_tool", "tools": ["shell", "write_file"], "command": "python3 ~/check.py", "timeout_seconds": 10 },
  { "event": "after_tool", "tools": ["edit_file"], "command": "npx prettier --check ." }
] }
```

- 命令在工作目录中运行，调用信息以 JSON 从标准输入传入：`event`、`session`、`tool`、`arguments`；`after_tool` 另有 `output` 与 `is_error`。`tools` 省略或写 `"*"` 表示所有工具。
- `before_tool` 在权限确认之前运行：退出码 0 放行；其他退出码、超时（默认 10 秒，最长 120 秒）或无法启动都会拦下这次调用，标准错误输出作为原因告诉小伴。配置文件无法解析时，所有工具调用都会停下，便于发现错误。
- `after_tool` 退出码非 0 时，标准错误输出会附在工具结果后面交给小伴，例如格式检查没有通过。
- hooks 是你自己写的命令，会以你的身份运行。在 Windows 和“默认权限”下，小伴的 shell 命令也能改到这个文件，重要的安全检查不要只依赖 hooks。

## 大仓开发边界

本项目是桌面客户端，不承担网页、账户或后端业务。角色可以有自己的实现，修改角色行为时运行本地 npm test；不引用 client/xiaban-web 或主后端的私有文件。两种窗口模式共用本应用场景，Windows/macOS 接入分文件。新增能力按本次需求，当前无聊天/账号/云端依赖。

品牌插画使用 `public/xiaoban.svg`，由本工程原有猫脸角色 `robotMarkup(stillFrame("idle"))` 导出；网站维护一份相同的静态 SVG，不导入客户端源码。


## 模型的思考等级

点击聊天输入框右下角的模型按钮，悬停具体模型或小伴 AI 模式，在二级菜单选择思考等级。模型配置弹窗的“高级设置”也提供“思考等级”。默认不添加额外参数；Responses 使用 `reasoning.effort`，Chat Completions 使用 `reasoning_effort`。旧的 `faster/deeper` 配置仍对应 low/high，偏好用于下一次聊天、摘要与子任务请求。

等级按模型和协议识别，不限制官方域名，也不排除托管服务。已识别 GPT-6 系列的 low/medium/high/xhigh/max，以及 DeepSeek V4、Kimi K3、GLM 5.2/5.3 的 low/high/max。模型能力有差异，不把“支持思考”等同于“支持所有等级”。自定义或第三方模型使用别名时，可以在高级设置中手动声明 `reasoningEfforts`；`null` 或缺省为自动识别，空列表为禁用。切换模型或协议时清除本机声明，避免套用旧模型的能力。官方托管模型由壁纸网站管理后台逐模型声明，小伴 AI 由 `server-ai` 管理后台逐档位声明，只返回等级，不公开真实模型。网关需要更新服务端后才能返回此元数据；旧网关保持默认选项。

原生端再次校验选择；菜单只显示该模型支持的等级。Anthropic Messages 暂未接入可调等级，仍使用服务商默认。内核保留 Chat 的 `reasoning_content` 和 Responses 的 reasoning 项，用于工具调用后继续请求；状态随历史保存但不作为可见回答，并按模型、地址和协议隔离。

针对性验证：`npx vitest run src/ui/ai-config.test.ts src/ui/components/AiConfigCard.test.ts src/ui/components/chat-model-selection.test.ts`、`npx vue-tsc --noEmit`；`cargo test --manifest-path src-tauri/Cargo.toml --bin xiaoban-desktop-wallpaper ai_reasoning:: --locked` 与同命令筛选 `reply_preference`、`ai_gateway::`。协议测试使用本地假服务，未使用生产密钥调用真实模型。

## Edge TTS 语音朗读

设置中的“声音”页支持音色、语速、音量、音调、试听和自动朗读；默认晓晓中文女声，自动朗读默认关闭。可刷新在线音色列表；失败时保留常用音色。聊天完整回复旁提供朗读/停止按钮；自动朗读仅针对当前可见聊天的新完整回复，发送新消息、切换/删除当前会话或离开聊天页会停止，不补读历史记录。代码块和链接地址不进入朗读文本。

原生实现位于 `src-tauri/src/tts/`，参考后端 `services/xiaban-api/backend/src/main/kotlin/xyz/xiaban/server/tts` 的 GEC、浏览器握手、UTF-8 分段、SSML 和二进制帧协议；桌面直接连接 Edge，不经过小伴后端。仅控制台可调用 TTS 命令。合成返回 MP3 二进制，由 WebView 播放，取消时废弃过期请求并回收 Blob URL。声音配置位于 `config/tts.json`；音频仅在内存缓存（最多 4 项，总计 8 MiB），不保存合成文本或音频到磁盘。

需要联网，无需语音服务 Key。单次最多 10000 字、16 MiB 音频；语音失败保留文字对话。Edge 接口是非官方读出服务，握手或协议变化时会提示重试。针对性验证：`npx vitest run src/ui/speech/speech.test.ts src/ui/chat/speech-events.test.ts`、`npx vue-tsc --noEmit`；Rust 环境运行 `cargo test --manifest-path src-tauri/Cargo.toml tts:: --locked`，Windows/macOS 需实际验证合成、播放及取消。

## SenseVoice Small 本地语音识别

设置 → 声音 →「语音识别」是全局能力开关，默认关闭；聊天页麦克风仅控制当前会话的本次录音，不持久化。设置页开启只准备引擎；聊天页手动点击麦克风时也可首次启用识别能力。首次开启提示加载引擎，从魔搭社区下载 SenseVoice Small 的 sherpa-onnx INT8 导出（模型与词表共约 229 MiB），显示进度并校验固定大小及 SHA-256。模型不随安装包分发；与判停模型、本地推理引擎 `xiaoban-infer` 一起保存至用户数据根目录的 `inference/`（引擎说明见 `tools/xiaoban-infer/README.md`），开关保存于 `config/asr.json`。关闭时停止采集、丢弃未完成识别并释放引擎，保留完整模型，下次开启离线复用；下载中断、校验失败及权限错误可重试。

**后台预热**：应用启动 3 秒后，后台线程会检查语音模型，**所有用户**缺少就静默下载（无进度、无弹窗，失败按 20 秒 / 1 分钟 / 5 分钟 / 15 分钟退避重试）；语音开关开着的用户随后还会加载引擎，关着的只留文件、不占内存。状态通过 `asr-state` 事件同步给各窗口。用户在页面里主动开启时，后台下载会让路（`urgent` 标志），由页面带着进度重新下载。真实网络实测：空配置、语音关闭，约 30 秒下完 229 MB，SHA-256 与固定值一致，无 `.part` 残留。完整的 SHA-256 校验只在下载时（边写边算）或第一次启动时做一次，之后用“大小 + 修改时间”的 `<文件>.verified` 记录跳过：实测 239 MB 的整体校验约 9.5 秒（debug 构建），而加载引擎只要 0.45 秒、首次推理 0.08 秒，所以以前按下麦克风后的“好几秒”几乎全是重复校验。文件被改动或记录缺失时会重新校验。与真实引擎联调：`XIAOBAN_INFER_DIR=<引擎与模型目录> cargo test --bin xiaoban-desktop-wallpaper the_engine_serves_the_capability_it_was_started_for -- --ignored`。

下载源为 `https://modelscope.cn/models/pengzhendong/sherpa-onnx-sense-voice-zh-en-ja-ko-yue`，固定版本 `73eca47697f980daa3d16112404174b6b950b514`，由 `iic/SenseVoiceSmall` 导出。推理使用 `sherpa-onnx` Rust 静态库和 CPU；构建时需要下载对应平台的原生推理库（可通过 `SHERPA_ONNX_ARCHIVE_DIR` 或 `SHERPA_ONNX_LIB_DIR` 使用本地库），无需用户安装 Python。

只在用户手动开启本次录音、聊天页位于前台且模型已配置时申请麦克风权限。进入、切换、新建或返回会话均保持录音关闭；发送消息、离开聊天、窗口失焦或开始朗读会结束本次录音，之后不自动恢复。下载过程中切换会话可继续准备模型，完成后等待用户重新点击麦克风。语音在本机转成文字并追加到输入框，由用户确认后发送。离开聊天、切换会话、窗口失焦、发送消息或朗读回复时停止采集，避免误录和旧结果进入其他会话。采集使用 AudioWorklet，转为 16kHz 单声道，按短暂停顿分句（每段最多 20 秒）；音频仅驻留内存，不写文件、不上传语音服务。macOS 安装包包含麦克风用途说明。

悬浮助手在展开且开启语音对话时，朗读期间也会采集麦克风，用设备回声消除、降噪、持续语音门限和朗读文本比对共同过滤自身回声；短片段识别确认有新的话语后停止朗读，并保留整段插话继续识别。聆听时保守过滤低频嗡鸣、高频嘶声、底噪及短促点击。用户编辑、粘贴或输入法组字时继续朗读，同时暂停语音采集与自动发送以保护草稿；按回车或点击发送时停止当前朗读，再发送新问题。收起助手或关闭语音会停止采集。外放回声效果仍受设备及系统实现影响，需要在实际扬声器、耳机和麦克风组合上验收。

针对性验证：`npx vitest run src/assistant src/ui/asr src/ui/pages/ChatPage.test.ts src/ui/speech/speech.test.ts`、`npx vue-tsc --noEmit`。原生测试位于 `src-tauri/src/asr/engine.rs`；Windows/macOS 仍需实机确认麦克风授权、录音及系统音频设备兼容性。



### 自创资源导出与分享

自创 HTML 动画壁纸、桌面特效、HTML 小组件和通过小伴创作的文本文件游戏支持导出 zip 资源包。各页提供“导入资源包”，作品卡片或详情面板提供“导出资源包”“分享到论坛”。分享先选择保存位置，再打开超级小伴论坛，由用户填写图文说明、上传预览图或视频和资源包附件。

资源包是普通 zip：最外层有 `manifest.json`（`schemaVersion: 1`、`kind`）加作品文件，任何工具都能打开和制作，完整格式见 [docs/resource-pack-format.md](docs/resource-pack-format.md)。单包最多 32 MB，包含作品模板和源文件，不包含聊天、账号、桌面布局、便签内容或游戏存档。自包含作品可离线使用；代码自行引用的外部网址不会自动下载。游戏的文本文件沿用现有创作协议的类型和大小限制；图片、音频和字体放在 `assets/` 下，一并导出和导入（带资源的包 `schemaVersion` 为 2，最多 60 个、单个 4 MB、合计 16 MB，并核对文件头）。壁纸、组件和特效暂不支持带资源文件，包里出现多余文件会被明确拒绝。各页的“导入资源包”都按包里的 `kind` 放到对应位置。

游戏有两个运行层：独立游戏窗口（默认，可全屏）和桌面层。桌面层游戏在 manifest 里写 `layer: "desktop"`，由小伴在主显示器上放一个透明窗口、浮在其他应用窗口上方运行，窗口默认点击穿透，游戏用 `game.regions` 声明需要鼠标的矩形；按 `capabilities` 申请才会收到其他窗口的位置和大小（`windows`）或鼠标位置（`pointer`），不含窗口标题。托盘菜单的“停止桌面游戏”随时结束它。只支持 Windows 和 macOS；macOS 的窗口读取已在真机验证，Windows 部分尚未在真机运行。协议见 [docs/resource-pack-format.md](docs/resource-pack-format.md)，示例见 `docs/examples/desk-pal`。导入复用各资源原有的验证与存储流程，不自动应用壁纸、启用特效、放置组件或启动游戏；同名特效、组件与游戏拒绝覆盖，内容相同的壁纸去重。官网和论坛页面本轮未改动。

聊天里的小伴也能代办分享：内置技能 `xiaoban-community`（`src-tauri/skills/xiaoban-community/SKILL.md`，随程序更新）讲解官网与论坛的页面、板块和发帖规则；`community_share` 弹出保存对话框导出 zip 资源包，再打开论坛对应板块（壁纸、特效、组件到 `/c/xiaoban-wallpaper`，游戏到 `/c/xiaoban-game`），`community_open` 只能打开固定清单里的官网和论坛页面。登录和发帖仍由用户在论坛完成。



## 内置 Computer Use 引擎构建

运行 `npm run desktop`、`npm run desktop:macos:build` 或 `npm run desktop:windows:build` 时，Tauri 自动准备固定版本原生引擎。也可执行 `npm run computer-use:prepare -- --target aarch64-apple-darwin`，按实际目标切换 triple。Windows 包只包含对应架构的 EXE；macOS 包只包含裁剪后的目标架构 App 及所需资源；不将整个跨平台 npm 包塞入安装包。开发时资源来自 `src-tauri/resources/computer-use`，安装后从 Tauri 的资源目录解析。实际构建目录不提交 git。

针对性检查：`npm run test:computer-use`；工具包接口、校验规则及真实 MCP 握手命令见 [Computer Use 工具包](../../libs/agent-tools/computer-use/README.md)。macOS 主应用仍支持原有最低系统版本，但电脑控制引擎需要 macOS 14+；旧系统启用时给出明确错误。首次使用仍需系统权限，以系统显示的小伴或 Open Computer Use 授权条目为准。

Agent 的 `computer_use_diagnose` 可检查开关、内置路径、真实权限、稳定签名和当前引擎近期 TCC 日志中的旧签名冲突；`request_accessibility_permission` 启动原生授权引导并在 Finder 选中当前引擎。`computer_use_repair_permissions` 经每次确认后，仅重置固定引擎标识所选的旧无障碍或屏幕录制记录，保留已有效权限，再打开设置和重新授权入口；完全信任模式也必须确认。同标识独立安装版也受所选权限重置影响。最终系统授权仍由用户点击，复查后继续；不提供全局 TCC 重置。每次电脑操作先检查真实授权，不把 MCP 连接当成权限有效。`invalid arguments` 表示未执行工具，要按 schema 修正原调用；`list_apps` 最简参数 `{}`，不能加 `reason` 或 `command`，不因参数错误重置权限或转用 Python/AppKit/AppleScript。旧的 `System Events` 探测已移除；引擎仍是独立 App，相同证书不会自动合并系统身份。小伴截图仍有独立屏幕录制用途，归属以系统实际显示为准。

### 完全信任与可选命令沙箱

完全信任以当前用户权限读写工作区外文件和执行 shell / process_start，支持全局 npm / brew 操作；不会自动取得管理员或 macOS 隐私授权。默认权限继续要求外部路径的一次性授权。切换模式时按当前会话实时判定，专家团子会话继承主会话模式。

设置 → 安全与权限中的「隔离完全信任下的命令」默认关闭（旧设置没有 commandSandbox 字段时也关闭）。手动开启后，macOS 命令才限制写入工作区、临时目录和缓存；Windows 保留独立的实验性隔离开关。隔离只作用于命令，不改变文件工具权限。sandboxNetwork 只控制已隔离命令的联网，不控制写入目录。开关对下一次启动的命令生效，已运行的进程需要停止后重启。

提权、工作区外批量删除、危险 Git 操作、新技能安装和敏感文件访问等仍需确认。沙箱内 EPERM 可能由应用隔离造成，不能仅凭该错误认定系统权限不足，也不应直接建议 sudo。

权限一致性：完全信任下，已连接工具与已校验技能脚本遵循同一模式；MCP 显式声明 destructiveHint 的操作仍须确认。动态 connector_call 会固定实际工具并校验参数，再进行权限判断和资源占用。默认权限下，已明确记住的授权不会因为随后读取普通文件而失效，高风险操作、未知执行结果和撤销授权仍优先。切换完全信任后，普通待确认操作会重新判定并继续，高风险确认保留；取消不会触发继续。后台进程输入按内容检查，普通数据不再一律算高风险。主助手、专家团与只读调研助手共享敏感路径检查。
