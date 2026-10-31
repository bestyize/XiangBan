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
| `:business:community-native` | 社区、云端与遥控小伴原生界面（Compose），按设计稿[社区与小伴AI 手机原型](docs/design-specs/mobile-prototype/README.md)搭建：悬浮胶囊导航（小伴AI、游戏、发布、社区、我的）与发布弹层；小伴AI 首页和对话页（模型选择、额度与兑换）；社区按[新稿](docs/design/社区与小伴AI%20手机原型.html)改版：首页（推荐、开源热榜、排行榜、追踪、导航）、讨论（创作、学习和教程、休闲，帖子带配图和投票）、资源（AI 工具、资料、影音娱乐、求助悬赏、开源精选），以及模型天梯、横屏的“智能 vs 价格”散点图、工具详情、求助帖（悬赏、采纳）和内容包、帖子详情；发现页内容来自 server-forum 的 `/api/discover/*`，目前由后台录入，带“示例”标记的是示例数据；我的、设置、登录与完成注册；我的电脑、遥控会话列表、遥控对话页（工具进度、审批卡片、高危二次确认、离线与异常提示）。基础部件（图标、顶栏、按钮、输入框、弹层）在 `Kit.kt`，图标路径由脚本从设计稿生成在 `Icons.kt`。平台能力通过 `PlatformActions` 注入，令牌存储通过 `TokenStorage` 注入。尚未接入的页面（游戏、资料库、分享海报等）见 docs 里的进度说明。 |
| `:business:xiaban` | xiaban.xyz 入口 `XiabanApp`：地址、站内域名、与网页一致的浅色/深色配色。 |
| `:shell` | 装配层，`App(storage, platform, linkStorage)` 渲染 `CommunityNativeApp`；Android 入口传入 `SharedPrefsTokenStorage`、`AndroidPlatformActions` 与 `SharedPrefsLinkStorage`，iOS 暂用内存存储。模板自带的 `Greeting`/`Platform` 示例与测试仍在，未使用。 |
| `:androidApp` `iosApp/` | Android、iOS 入口。 |
| `desktopApp/` | 桌面壳（Rust，Tauri 2）：`src/main.rs` 建窗口与外链处理，`src/links.rs` 是站内/站外/拦截的判定与单元测试，`ui/index.html` 是启动页（先测网络，通了再进站，不通显示「连不上相伴」和重试按钮）。 |

新增业务壳放 `business/`，可复用的能力放 `common/`，`shell` 只做装配。

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
