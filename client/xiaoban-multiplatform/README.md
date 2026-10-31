# 小伴多端 App

状态：可运行原型。Android/iOS 的移动端入口使用 Compose 原生社区、AI 与桌面互联界面；桌面 Tauri 壳仍指向已停用的 xiaban.xyz。Android 与 macOS 的历史验证、iOS 编译和 Windows/Linux 桌面构建范围见下文，不代表当前版本均已实机验收。

移动端和桌面端分开实现：

- Android、iOS：Kotlin Multiplatform + Compose Multiplatform，Gradle 工程在本目录根（`settings.gradle.kts`）。
- 桌面：`desktopApp/`，Rust + Tauri 2 的独立工程，不在 Gradle 里，也不依赖 Kotlin 代码。

允许与其他客户端重复代码；不要强行套用网页 UI 或全局共享实现。各客户端维护自己的协议模型。

## 模块

| 模块 | 说明 |
| --- | --- |
| `:common:webview` | 通用网页外壳 `WebShell`：加载进度、失败重试、返回键、站内外链判定（站外链接交给系统浏览器）、按系统深浅色套用 `WebShellPalette`（系统栏边距与失败页）。基于 compose-webview-multiplatform，目标只有 Android 和 iOS。 |
| `:business:community` | 社区网页壳（MA，已不被 `:shell` 装配，保留作回退）`CommunityApp`：加载 https://chat.xiangban.xyz（周刊、资源、讨论、项目的只读浏览，分享先用网页上的“复制链接”），站内域名只有 chat.xiangban.xyz，配色取自论坛前端的品牌色与黑白灰。对应实施计划的 MA（网页壳）。 |
| `:common:community-core` | 社区、账号与 AI 网关的网络层和状态逻辑（不依赖 Compose）：`ForumApi`、`AccountApi`（与网页同一组接口）、`AiGatewayApi`（对接 `server-ai` 的档位、余额、兑换码与 `/v1/chat/completions` 流式对话）、`SessionManager`（令牌存储、临近过期自动刷新且并发共用一次、仅服务端明确拒绝才登出）、`FeedController`/`DiscussController`/`CloudChatController`/`Loader`、`DesignTokens`（与论坛前端同一套配色圆角）。能在 JVM 上直接跑单元测试：`./gradlew :common:community-core:jvmTest`。账号中心要先登记客户端 `xiaoban-android`。 |
| `:common:link-core` | 与桌面互联的协议、客户端与应用控制器（`PhoneLink`、`RemoteChatController`、`SharedPrefsLinkStorage`）：同账号自动发现电脑、经中转收发、会话快照与流式增量、待办审批卡片、断线重连；有对真实 server-link 与 Rust 桌面端的跨语言互通测试。`./gradlew :common:link-core:jvmTest`。详见模块 README。 |
| `:business:community-native` | 社区、云端与遥控小伴原生界面（Compose），按设计稿[社区与小伴AI 手机原型](docs/design-specs/mobile-prototype/README.md)搭建：悬浮胶囊导航（小伴AI、游戏、发布、社区、我的）与发布弹层；小伴AI 首页和对话页（模型选择、额度与兑换）；社区按[新稿](docs/design/社区与小伴AI%20手机原型.html)改版：首页（推荐、开源热榜、排行榜、追踪、导航）、讨论（创作、学习和教程、休闲，帖子带配图和投票）、资源（AI 工具、资料、影音娱乐、求助悬赏、开源精选），以及模型天梯、横屏的“智能 vs 价格”散点图、工具详情、求助帖（悬赏、采纳）和内容包、帖子详情；发现页内容来自 server-forum 的 `/api/discover/*`，目前由后台录入，带“示例”标记的是示例数据；我的、设置、登录与完成注册；我的电脑、遥控会话列表、遥控对话页（工具进度、审批卡片、高危二次确认、离线与异常提示）。基础部件（图标、顶栏、按钮、输入框、弹层）在 `Kit.kt`，图标路径由脚本从设计稿生成在 `Icons.kt`。平台能力通过 `PlatformActions` 注入，令牌存储通过 `TokenStorage` 注入。尚未接入的页面（私人资料库等）见 docs 里的进度说明。 |
| `:business:xiaban` | xiaban.xyz 入口 `XiabanApp`：地址、站内域名、与网页一致的浅色/深色配色。 |
| `:shell` | 装配层，`App(storage, platform, linkStorage)` 渲染 `CommunityNativeApp`；Android 入口传入 `SharedPrefsTokenStorage`、`AndroidPlatformActions` 与 `SharedPrefsLinkStorage`，iOS 暂用内存存储。模板自带的 `Greeting`/`Platform` 示例与测试仍在，未使用。 |
| `:androidApp` `iosApp/` | Android、iOS 入口。 |
| `desktopApp/` | 桌面壳（Rust，Tauri 2）：`src/main.rs` 建窗口与外链处理，`src/links.rs` 是站内/站外/拦截的判定与单元测试，`ui/index.html` 是启动页（先测网络，通了再进站，不通显示「连不上相伴」和重试按钮）。 |

新增业务壳放 `business/`，可复用的能力放 `common/`，`shell` 只做装配。

## 手机 Agent 与游戏创作

Android 在内置 `agent_mobile` 原生库时支持私有文本工作区、文件读写/编辑/搜索、游戏草稿安装与校验、游戏列表、独立沙箱试玩、运行错误读取和桌面兼容 ZIP 导出。聊天可以选择相册图片或 UTF-8 文本文件作为模型参考；Agent 支持持久任务清单、结构化提问、防重复和无进展暂停。原生库缺失时保持纯云端聊天降级。

第一阶段只支持普通离线文本游戏包（v1、`layer: space`）；ZIP 导入、v2 二进制素材、通用 Shell、技能/连接器及系统终止后自动续跑尚未接入。iOS 尚无对应原生实现。模块分工、限制、后续阶段和针对性验证见 [手机 Agent 基础能力接入](docs/agent-foundation.md)。

## 游戏中心 A 版（2026-10-31）

手机版游戏页接入[轻量游戏库设计](docs/design-specs/prototypes/game-center.html)：双列封面、玩法筛选、最近游玩、我的作品、详情及创作弹层。Android 构建从桌面 `public/game-packs/` 打包 9 款只读官方游戏，仍使用离线沙箱与原有消息/存档协议；不写入用户创作工作区，不放宽自定义游戏保留 ID、网络或权限限制。官方排序优先暮色滑翔、水墨剑客、雅韵五子棋与 3D 全阶魔方。

最近游玩只在沙箱收到有效 `game.ready` 后持久保存，读取失败或仅点击启动不会产生记录。自创游戏优先显示本地 `cover.svg`，缺少或解码失败时显示名称封面；试玩、修改、ZIP 导出沿用原平台能力。iOS 仍未接入游戏运行和本地创作，显示平台不可用状态。

针对性验证：`:common:community-core:jvmTest --tests '*GameLibraryTest*' --tests '*MobileGamesTest*'`、`:business:community-native:compileAndroidMain`、`:androidApp:compileDebugKotlin` 与 `:androidApp:mergeDebugAssets`。本轮未安装到手机，触控、横竖屏和各机型 WebView 的实际效果仍需实机验收。

## 创造中心（2026-11-01）

手机底部「游戏」升级为「创造」，保留原导航存储标识。发现页优先展示全部官方游戏，其后是壁纸、小组件、特效、伙伴的创作参考；分类在两页通用。发现页仅保留「你想创造什么？」入口，我的作品页用右上角「＋」新建。作品卡片不铺操作按钮，点击打开分组底部菜单。我的作品提供二次确认删除，清理本机作品、存档和运行记录；不影响已导出或发布的副本，当前创作任务进行中时禁用删除。

本机游戏可直接游玩、继续修改，或把校验后的 ZIP、标题和简介带入社区发布页（仍需用户确认发布权利并主动提交）。其他渠道支持 Android 系统分享作品卡片 PNG、作品 ZIP 和文字介绍；私有存档、对话与运行日志不进入 ZIP。发送到电脑使用系统分享 / 保存 ZIP 后在电脑导入，不伪造直传成功状态。

桌面装扮参考图明确标记为创作灵感；选择同账号在线电脑后，将想法预填到新的遥控对话，由用户发送。手机没有装扮运行宿主，也没有装扮作品同步协议，所以电脑创作成果仍保存在电脑，手机「我的作品」目前只列真实本机游戏。iOS 尚无游戏打包、图片分享和本地创作能力。

设计稿沿用 [创造中心](docs/design-specs/prototypes/create-center.html)。验证：Android `:androidApp:assembleDebug` 通过，已有游戏格式 / 官方目录 8 项 JVM 测试通过；小米 17 确认游戏优先、作品操作菜单、发布页预填真实 ZIP，以及 ZIP / PNG 系统分享面板。未提交社区发布或向外部接收方发送；电脑创作链路仅接入已有遥控对话，未替用户执行生成。导航草稿换行编码增加到 `RouteTest`，该 UI 模块当前只有 iOS 测试目标，本轮未运行。

## 安卓实时语音

手机通话沿用桌面的级联方式：本地 Silero 检测人声、Smart Turn v3 判断是否说完、SenseVoice 转文字，再请求聊天模型并按句朗读。说话期间显示本地识别的中间字幕；可在小伴回答时开口打断，本地识别可用时会过滤语气词和朗读回声，下一轮带上已播放内容的说明。通话不会因安静自动挂断，用户可随时静音或结束；后台任务仍按原有权限运行。

首次使用可下载 SenseVoice、Silero 和 Smart Turn（共约 250 MB），已下载的文件会复用；缺少 Smart Turn 或加载失败时回退到原有识别和停顿判定。模型在手机运行，聊天请求仍发到所选服务；无法使用本地识别时按原有顺序尝试电脑或云端识别。Edge 朗读可用时会预备少量简短接话，失败不阻塞对话。

安卓构建使用 `onnxruntime-android` 运行 Smart Turn；sherpa-onnx 使用独立命名的运行库，避免两个版本的原生符号冲突。手动打包工作流在准备 sherpa 库时使用 `patchelf` 设置依赖和 SONAME。针对性验证：`./gradlew :common:community-core:jvmTest --tests '*VoiceAlignmentTest*' --tests '*VoiceActivityTest*' --tests '*VoiceCallTest*' --tests '*SpeechModelsTest*' :common:link-core:jvmTest --tests '*RemoteChatTest*' :androidApp:assembleDebug`；外放回声、打断和手机性能仍需实际设备验收。

## 开发与验证

Android 与 iOS 在本目录运行。`local.properties` 的 `sdk.dir` 为空时设置 `ANDROID_HOME`（macOS 通常是 `~/Library/Android/sdk`）。

```sh
export ANDROID_HOME=$HOME/Library/Android/sdk
./gradlew :common:webview:testAndroidHostTest             # 站内外链判定的单元测试
./gradlew :androidApp:assembleDebug                       # Android 调试包
./gradlew :shell:linkDebugFrameworkIosSimulatorArm64      # iOS 共享框架
```

桌面需要 Rust stable，在 `desktopApp/` 运行：

```sh
cargo test                       # 链接判定的单元测试
cargo run                        # 开发运行
cargo build --release --features custom-protocol   # 发布二进制（内嵌启动页）
```

打 `.app` / `.dmg` / `.msi` 用 Tauri CLI（`cargo install tauri-cli --version "^2" --locked`，或已有的 npm `@tauri-apps/cli`），在 `desktopApp/` 运行 `cargo tauri build`。macOS 的 `.app` 约 4 MB，使用系统 WebView。图标由 iOS 的 1024 图标生成（`tauri icon`）。

## 已验证与未验证

- Android：模拟器（Android 35）上验证过页面加载、登录弹窗、网页直连账号中心（CORS 与 localStorage 正常）、软键盘、返回键。真机装包需要在手机上允许 USB 安装。
- iOS：共享框架能链接。本机没有 iOS 模拟器运行时，Xcode 工程未构建，运行时未验证。
- 桌面（macOS）：开发运行和发布版 `.app` 都打开过，页面加载、深色主题正常；启动页的离线失败态与重试按钮在浏览器里验证过。Windows 要 WebView2（安装包会引导下载），Linux 要 webkit2gtk，两者都没构建过。
- 桌面未验证：网页里「保存图片」「下载」这类依赖浏览器下载的功能（系统 WebView 默认不一定处理下载，没有专门实现）；游戏 iframe 在站内导航规则下的表现；浅色系统外观。

## 边界

页面本身由 xiaban.xyz 提供，三端都没有实现 App 桥接协议，也没有注入 `window.XiabanBridge`，网页按普通浏览器运行；注入后网页会在首次会话前等待 App 的 token，实现桥接时一并处理。Android/iOS 的系统栏边距按系统深浅色填充，网页若让用户单独选择深浅色，两者不一致时会有色差。

桌面壳不给网页任何 Tauri 能力：`tauri.conf.json` 没有 capability，远程页面不能调用本地命令。站内只放行 `https://xiaban.xyz` 及其子域（默认端口），其他 http/https 链接、`mailto:`、`tel:` 交给系统打开，其余协议（`file:`、`javascript:`、`data:`、`blob:` 等）一律拦截。窗口数据（登录令牌所在的 localStorage）由系统 WebView 按应用标识 `xyz.xiangban.xiaoban` 持久化。

桌面曾试过 Compose 的 KCEF（Chromium），macOS 上内核目录结构与库的加载路径对不上，启动即原生崩溃，且要下载约 500MB，所以桌面改用 Rust。

## 社区本轮补齐（2026-10-31）

AI 日报在首页和往期列表显示图文封面，点击进入站内文章与留言页；支持回复、点赞和删除自己的留言，来源链接保留在文章末尾。站内正文由论坛编辑审核后提供，旧的纯链接条目会提示正文暂缺。需要部署对应版本的 server-forum 才能使用详情和留言接口。

`community-core` 新增独立的模型榜、上传、内容发布、社交与挑战协议模块；`community-native` 接入能力榜/调用量榜、精确Token用量、来源时间与开源榜实际观测天数、投票/图文和帖子附件、能力清单/方案详情、我的作品与zip/新版本发布、通知/帖子收藏/关注/屏蔽/举报、官方挑战与本人作品投稿。调用量表示OpenRouter平台Token用量，不是请求次数或能力评分；未审核数据不公开。

上传流程使用论坛凭证直传资源机，不把论坛Bearer发给资源地址，禁重定向；只读校验最多五次，资源未配置或超时保留表单。能力权限/提示词仅展示清单真实字段，缺失明确标注。挑战点赞复用真实讨论帖接口，不生成独立投票数字。

针对性JVM测试与Android模块编译已验证，系统选择器/分享/键盘/返回栈、完整资源机链路和iOS仍需实机验收。正文图片、帖子/回复附件与回复作品卡已接；作品/能力包和方案支持收藏、私密举报，收藏按帖子/作品/方案分组。Android 提供真实系统分享、海报 PNG/二维码保存与安全深链；iOS 仅复制链接/网页回退，不宣称系统分享或海报已验收。账号方案备份为默认关闭的本人只读入口，开启后可读备份和回收站，不自动下载、应用或分享私有内容。具体状态见[社区验收记录](../../docs/product/验收记录_社区全功能_2026-10-31.md)。

第二轮回复编辑仅对本人且服务器状态允许的回复开放，读取原文/source保留作品固定版本和原附件价格；保存失败保留输入，取消只清理本次新增待关联附件。集成验证：分享7、收藏/回复12、方案备份4、既有上传12项 JVM 测试通过，`:androidApp:assembleDebug` 成功；未操作真机。

## 创作详情互动（2026-11-01）

社区创作中的视觉作品、能力与方案共用底栏：评论、点赞、收藏、分享。评论复用帖子编辑器和评论区，支持评论点赞及本人回复编辑；已下架/待审作品不开放互动。分享仍走真实系统分享与落地页，电脑使用链接保留为能力页的次级入口。列表有真实互动时显示赞/评论数，无数据时不加装饰数字。

Android 构建与 `:common:community-core:jvmTest --tests '*WorkDiscussionTest*' --tests '*ContentSocialReplyTest*'` 验证协议、登录状态、服务端错误、兼容字段与评论点赞路由。依赖论坛新增的 `/content-social/{kind}/{id}/discussion` 接口；不能只更新 APK 而保持旧后端。iOS 未实机验证。

## 社区游戏试玩与收藏（2026-11-01）

Android 的社区游戏作品详情、帖子中的本站游戏链接卡片和回复作品卡支持「试玩」，无需放入「我的作品」。每次启动重新检查作品公开状态与指定版本，资源 ZIP 通过大小、SHA-256、清单和展开限制后进入原有离线沙箱；帖子回复的固定版本不自动替换为最新版。第一版支持 v1、space、无系统能力的离线文本游戏包；普通 ZIP 附件不会直接执行，iOS 显示暂不支持。

游戏 ZIP 存于独立 cache 目录，最多 8 包、单包 5 MiB，清理缓存不影响收藏、存档与创作工作区。存档/运行日志按社区作品 ID 与版本隔离在独立 files 目录，避免同名游戏覆盖自己创作的游戏。试玩页的窄原生工具栏提供退出、重新开始和收藏，并为游戏保留不被覆盖的显示区域；退出回到原页面。旧的纯 HTML 游戏在资源加载成功且没有加载错误后记录游玩，协议游戏沿用 game.ready。

「我的收藏」新增游戏分类及本机「最近玩过」，最近记录最多 12 项，试玩成功才加入。游戏收藏仍是已有作品收藏，跨设备同步；最近试玩不自动收藏。服务端 `/api/me/content-bookmarks?kind=game` 在分页前筛选游戏，返回对象仍为 kind=pack。不新增一级「我的游戏」，不自动收录为个人创作，不提供离线保存承诺；打开缓存仍需确认公开状态。

验证：`:common:community-core:jvmTest --tests '*CommunityGameTrialTest*' --tests '*MobileGamesTest*' --tests '*GameLibraryTest*'`（17 项）与 `:androidApp:assembleDebug` 通过。社区游戏真机验证见 [试玩验收记录](../../docs/product/community-game-trial-2026-11-01.md)。
