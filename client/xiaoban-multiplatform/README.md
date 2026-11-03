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
| `:business:community-native` | 社区、云端与遥控小伴原生界面（Compose），按设计稿[社区与小伴AI 手机原型](docs/design-specs/mobile-prototype/README.md)搭建：悬浮胶囊导航（小伴AI、游戏、发布、社区、我的）与发布弹层；小伴AI 首页和对话页（模型选择、额度与兑换）；社区按[新稿](docs/design/社区与小伴AI%20手机原型.html)改版：首页（推荐、精选、排行榜、追踪、导航）、讨论（创作、学习和教程、休闲，帖子带配图和投票）、资源（AI 工具、资料、影音娱乐、求助悬赏），以及模型天梯、横屏的“智能 vs 价格”散点图、工具详情、求助帖（悬赏、采纳）和内容包、帖子详情；发现页内容来自 server-forum 的 `/api/discover/*`，目前由后台录入，带“示例”标记的是示例数据；我的、设置、登录与完成注册；我的电脑、遥控会话列表、遥控对话页（工具进度、审批卡片、高危二次确认、离线与异常提示）。基础部件（图标、顶栏、按钮、输入框、弹层）在 `Kit.kt`，图标路径由脚本从设计稿生成在 `Icons.kt`。平台能力通过 `PlatformActions` 注入，令牌存储通过 `TokenStorage` 注入。尚未接入的页面（私人资料库等）见 docs 里的进度说明。 |
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

Android 冷启动后延迟约 2 秒，在后台静默下载本地语音组件及 SenseVoice、Silero 和 Smart Turn 模型（模型约 250 MB，原生组件约 60 MB）；不弹下载询问、不阻塞首页。Application 进程级下载器与声音设置页共用任务和进度，Activity 重建或手动重试不会重复写入；已有文件复用，旧用户只补齐当前版本组件。断网保留临时文件，按 15 秒、60 秒和 5 分钟有限退避；进程结束后下次冷启动续传。设置页可查看进度或重试。

客户端资源只从魔搭下载。SenseVoice 优先使用上游 Sherpa 适配发布仓库 `pengzhendong/sherpa-onnx-sense-voice-zh-en-ja-ko-yue` 的固定提交（已核对大小及 SHA-256），再用小伴镜像；不把转换版称为阿里原始官方模型。Silero、Smart Turn 与当前兼容运行库使用 `bestyize/xiaoban-asr` 镜像。当前未找到可验证且匹配固定版本的微软/Sherpa 官方魔搭原生库分发；以后若新增官方源，核对版本、ABI 和完整哈希后再优先登记，不在用户手机上动态探测任意来源。

资源未齐时保留电脑/云端语音及原有停顿判定，不弹下载询问阻断通话。模型在手机运行，聊天请求仍发到所选服务。Edge 朗读可用时会预备少量简短接话，失败不阻塞对话。

安卓构建保留 `onnxruntime-android` 的 Java 接口，四个语音 `.so` 从 APK 排除。`prepareVoiceRuntime` 使用 Python 3.11+、`patchelf` 准备固定版本的预编译库，不编译 Sherpa/ORT；Sherpa 使用独立 SONAME 避免版本冲突。构建输出为 `androidApp/build/voice-runtime/assets/voice-runtime.json`（随 APK 打包）和 `downloads/*.so`（独立分发）。公开文件名只保留原始库名，不添加 voice、版本或哈希前缀；大小和 SHA-256 保留在清单中。客户端下载到私有缓存，在加载前校验，再按依赖顺序加载；下载失败可取消或续传。

分发方式沿用桌面端：把 `downloads/*.so` 原名上传到魔搭 `bestyize/xiaoban-asr` 的仓库根目录，文件为 `libonnxruntime.so`、`libonnxruntime4j_jni.so`、`libort_sherpa.so`、`libsherpa-onnx-jni.so`。手动 APK 打包工作流通过官方 `modelscope-hub==0.6.1` SDK 同步缺少或不一致的组件；配置仓库 Secret `MODELSCOPE_API_TOKEN`（该魔搭仓库的写权限）。也可事先手动上传；已有文件匹配时跳过上传，无需写入授权。

`publish-voice-runtime.py` 在构建 APK 前核对本地完整哈希及魔搭公开元数据，并记录已验证文件的 Git 提交到 `published-revisions.json`。随后生成的 APK 下载地址固定到对应提交，同名更新不影响旧客户端；保留魔搭仓库的 Git 历史即可，无需在文件名中暴露版本。缺文件、缺写入授权或无法核验提交时停止 APK 发布；客户端清单不含 GitHub 下载源。

组件准备有本地增量检查和 Actions 缓存；Agent Rust 库仍在 APK 内，仍用 Cargo 缓存增量编译。ORT Android 的加载调用由构建时的局部字节码转换接入私有目录加载器，其余推理接口保持上游版本。

针对性验证：`./gradlew :common:community-core:jvmTest --tests '*SpeechModelsTest*' --tests '*DownloadedSpeechRuntimeTest*' :androidApp:assembleDebug`，涵盖冷启动延迟、静默失败退避、后台与设置共用下载、镜像续传、取消、缺库后补下载、依赖加载顺序、校验失败与加载重试。真机上的语音识别、判停、外放回声及打断仍需设备验收。

## 社区网页对齐（2026-11-03）

原生社区同步当前网页的内容结构与卡片规则，使用 Compose 原生布局，保留站内详情、评论、分页和返回栈：

- 推荐：小伴前沿从 `/api/discover/frontier` 读取，16:9 横幅将单行标题、来源与时间覆盖在封面上，左下分页点与手势翻页，不提供暂停或箭头按钮；详情与留言使用独立前沿接口。日报使用文字与右侧小图的阅读卡；移除推荐页的精选创作模块。
- 精选：仍使用首页的精选入口及第二子 TAB 热榜，资源不重复提供精选；模板封面里的项目名与图标不在卡片底部重复。卡片不外露收藏或安装说明，收藏沿用详情入口。目录由后端提供，接口未部署时显示加载失败与重试。
- 导航：大模型 / Harness / Skill / API，筛选规则与网页一致；图标、厂商、简介、类型采用双列信息卡，不再重复分类标题、描述和数量。新版 `catalog_version >= 2` 尊重已发布条目、顺序与删除；旧接口按审核快照补齐 66 个入口并保留编辑，v1 删除项不会复活。快照归手机端维护，手动运行 `python3 client/xiaoban-multiplatform/tools/sync-navigation-catalog.py` 从论坛审核目录刷新，Gradle 不跨项目依赖论坛源码。
- 讨论与资源：阅读帖为紧凑配图卡，休闲与影音按真实图片比例展示瀑布流，封面显示真实浏览和回复数；无图时保留文字，不制造海报。作品默认 16:9，壁纸筛选按真实比例限制在 4:5 至 16:9；能力类作品用横向目录卡。补齐图文分享、工具导读和资料导读入口及站内分页；学习筛选按正式栏目查询。全部讨论提供最新回复、新帖、热门、精选与关注。

Android 调试包可通过 `adb reverse tcp:5290 tcp:5290` 连接本地论坛预览，再运行 `adb shell am force-stop xyz.xiangban.xiaoban` 和 `adb shell am start -n xyz.xiangban.xiaoban/.MainActivity --es forum_preview_base http://127.0.0.1:5290`。入口只在 Debug 接受 `127.0.0.1` 的 1024–65535 端口，不持久保存；普通冷启动恢复正式服务。明文访问许可仅在 Debug 的 loopback 域生效，Release 保持正式地址与 HTTPS。

针对性验证：`:androidApp:assembleDebug`、community-core JVM 的 NavigationDirectoryTest、FrontierTest、CommunityFeedAlignmentTest、DiscoverApiTest 与 OpenSourceTest。小米 14（23127PN0CC）已覆盖安装调试包并验证前沿横幅、日报阅读卡、Harness 编程筛选和创作图文分享。通过本地论坛预览验证精选封面、项目详情、返回及热榜切换；热榜使用本地测试数据，未进行生产写入。iOS 未实机验收。当前线上精选目录 `/api/open-source` 尚未部署，需配套发布论坛服务后才能在正式地址使用。

## 开发与验证

精选（2026-11-03，本地实现）：首页精选提供项目分类与标签，资源不重复提供精选子栏目；原开源热榜合并为第二个子 TAB「热榜」，保留旧导航状态与往期榜单；关键词搜索复用顶栏入口，收藏操作仅在项目详情显示。支持 Logo 搭配截图、实物照片或明确标注示意的 16:9 项目封面、专题卡、项目中文导读、官方资料链接和原有仓库评论。历史开源帖子保留「社区分享」入口。目录由 `GET /api/open-source` 返回，与网页读取同一已发布目录（首次发布前使用 `services/server-forum/editorial/open-source.json`），封面配置由精选专属规则解析；此规则不适用于其他社区板块。后台保存与发布流程见论坛 README，依赖论坛服务部署该接口；加载失败显示重试，不以示例数据替代。收藏保存在设备偏好，与开源热榜沿用同一仓库键，未实现账号同步。验证：`:business:community-native:compileAndroidMain`、`:common:community-core:jvmTest --tests '*OpenSourceTest*'`；未做 Android/iOS 实机验收。

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

## 对话工作空间

小伴 AI 左侧栏支持创建、重命名、删除和折叠工作空间；长按对话可移动到工作空间或未分组。删除空间保留全部对话，继续聊天保留分组。Android 随本机历史按账号保存，iOS 沿用内存存储；不提供电脑目录绑定或跨设备同步。新对话默认未分组。

针对性验证：`:common:community-core:jvmTest --tests '*ChatHistoryTest*'` 与 `:business:community-native:compileAndroidMain`，以及 `:androidApp:assembleDebug` 均通过。2026-11-01 在小米 14（23127PN0CC）覆盖安装调试版，验证创建空间、长按移动对话、重启后保留分组、折叠空间以及删除空间后对话回到未分组；测试空间已清理。重命名交互与 iOS 仍需实机验收。

侧栏支持横向拖动收起，位置与遮罩随手指变化，松手按距离与速度回弹或关闭；已在小米 14 验证短拖回弹与长拖关闭，Android 调试包构建通过。关闭时仍通过菜单按钮打开。

## 聊天模型思考等级

对话输入栏模型按钮采用与桌面相同的 Lucide Brain 图标。模型选择列表右侧可打开思考等级菜单，仅展示网关声明的等级；选择等级同时选中该模型，默认不传 reasoning_effort。普通流式聊天与手机 Agent 均携带所选等级。模型和思考等级随对话持久化，切换对话恢复各自选择，旧记录采用默认值。

验证：community-core JVM 测试及 community-native Android 编译通过，新增 ReasoningTest 覆盖请求参数与默认值清除；本轮尚未安装到手机验收。

## 手机 Agent 可靠性对齐（2026-11-02）

手机本地 Agent 默认不再受 25 轮和累计 8 次续跑提示限制，保留重复调用与连续无进展暂停。聊天页锁图标可按对话切换默认/完全信任、查看和撤销授权；删除文件与覆盖已有游戏仍每次确认。右上角任务设置提供上下文及可选执行预算，模型、思考等级、权限与预算随对话保存，旧记录采用安全默认值。切换账号先停止并保存原账号任务，再切换存储目录。

本地工具取消会传到 Kotlin 协程；结果不明时保留复核状态，由用户核对后继续。重试使用内核原记录，已有输出或操作时转为检查状态后继续。手机与纯云聊天都按真实 JSON 大小控制请求，保留当前提问；上下文提供有界历史摘录和原文回读。取消 120/180 秒总流式截止，保留连接及无数据超时。聊天保存失败会保留待写内容并显示“重试保存”，不再删除原文件后静默换名。

Android 缺原生库或 iOS 当前入口会明确显示纯云聊天；原生与宿主需配套更新。持久 todo 继续可用，系统终止后的后台目标恢复、手机通用 Shell 等仍按原阶段边界。

针对性验证：`cargo test --locked --manifest-path libs/agent-mobile/Cargo.toml`（仓库根）；community-core 的 MobileReliabilityTest、MobileCancellationTest、ChatHistoryTest、AgentChatTest、AgentRuntimeTest 和 ReasoningTest；JNI 测试需设置 AGENT_MOBILE_LIB 指向本次编译的原生库，避免跳过。
