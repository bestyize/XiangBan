# 小伴多端 App

状态：可运行原型，目前三端都只是把 https://xiaban.xyz 套进原生壳。Android 与 macOS 已运行验证，iOS 只验证到编译，Windows/Linux 桌面未构建（见下）。这个目录固定保留为未来 App 的开发位置。

移动端和桌面端分开实现：

- Android、iOS：Kotlin Multiplatform + Compose Multiplatform，Gradle 工程在本目录根（`settings.gradle.kts`）。
- 桌面：`desktopApp/`，Rust + Tauri 2 的独立工程，不在 Gradle 里，也不依赖 Kotlin 代码。

允许与其他客户端重复代码；不要强行套用网页 UI 或全局共享实现。可选择引用 services/xiaban-api/api-contracts，也可以维护自己的模型。

## 模块

| 模块 | 说明 |
| --- | --- |
| `:common:webview` | 通用网页外壳 `WebShell`：加载进度、失败重试、返回键、站内外链判定（站外链接交给系统浏览器）、按系统深浅色套用 `WebShellPalette`（系统栏边距与失败页）。基于 compose-webview-multiplatform，目标只有 Android 和 iOS。 |
| `:business:community` | 社区前台入口 `CommunityApp`：加载 https://chat.xiangban.xyz（周刊、资源、讨论、项目的只读浏览，分享先用网页上的“复制链接”），站内域名只有 chat.xiangban.xyz，配色取自论坛前端的品牌色与黑白灰。对应实施计划的 MA（网页壳）。 |
| `:business:xiaban` | xiaban.xyz 入口 `XiabanApp`：地址、站内域名、与网页一致的浅色/深色配色。 |
| `:shell` | 装配层，`App()` 目前渲染 `CommunityApp()`（原先是 `XiabanApp()`，该模块保留）。模板自带的 `Greeting`/`Platform` 示例与测试仍在，未使用。 |
| `:androidApp` `iosApp/` | Android、iOS 入口。 |
| `desktopApp/` | 桌面壳（Rust，Tauri 2）：`src/main.rs` 建窗口与外链处理，`src/links.rs` 是站内/站外/拦截的判定与单元测试，`ui/index.html` 是启动页（先测网络，通了再进站，不通显示「连不上相伴」和重试按钮）。 |

新增业务壳放 `business/`，可复用的能力放 `common/`，`shell` 只做装配。

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

页面本身由 xiaban.xyz 提供，三端都没有实现 App 桥接协议（`client/xiaban-web/src/lib/bridge.ts`），也没有注入 `window.XiabanBridge`，网页按普通浏览器运行；注入后网页会在首次会话前等待 App 的 token，实现桥接时一并处理。Android/iOS 的系统栏边距按系统深浅色填充，网页若让用户单独选择深浅色，两者不一致时会有色差。

桌面壳不给网页任何 Tauri 能力：`tauri.conf.json` 没有 capability，远程页面不能调用本地命令。站内只放行 `https://xiaban.xyz` 及其子域（默认端口），其他 http/https 链接、`mailto:`、`tel:` 交给系统打开，其余协议（`file:`、`javascript:`、`data:`、`blob:` 等）一律拦截。窗口数据（登录令牌所在的 localStorage）由系统 WebView 按应用标识 `xyz.xiangban.xiaoban` 持久化。

桌面曾试过 Compose 的 KCEF（Chromium），macOS 上内核目录结构与库的加载路径对不上，启动即原生崩溃，且要下载约 500MB，所以桌面改用 Rust。
