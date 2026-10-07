# 小伴桌宠 / 动态壁纸原型

Windows / macOS 原型：一个可拖动、透明处点击穿透的悬浮小伴，也可以切换成位于桌面图标下方的动态壁纸。无聊天、账户或业务服务器连接。Windows 缺少运行组件时会联网补齐；组件可用后可离线运行。角色、场景、控制台与互动逻辑共用，桌面宿主分平台接入。

本机生产构建实测：Windows x64 EXE 约 4.60MB，内嵌前端与两套程序背景，不包含系统 WebView2。原生切换测试已在主屏 2880×1800、175% 缩放下通过，确认壁纸为 WorkerW 子窗口；验证报告位于本地 `artifacts/smoke-*.json`。本地试用文件为 `artifacts/XiaoBan-Windows-x64.exe`，ZIP 包为 `artifacts/XiaoBan-Windows-x64.zip`。

角色核心在本工程的 `src/companion` 内维护，最初从网页纯角色代码原样复制，保留既有 SVG、发光、表情、弹簧参数和互动行为。允许独立演进，不再导入另一个客户端源码。`src/pet.ts` 适配 DOM 与桌面输入；暖光维持原 RobotStage 的尺寸与渐变。

## 在 Windows 上运行

需要 Node.js 22.12+（或 24）、Rust MSVC 工具链、Visual Studio C++ 构建工具和 WebView2 Runtime。在仓库根执行：

```powershell
cd client/xiaoban-desktop-wallpaper
npm ci
npm run desktop
```

生产构建与启动：

```powershell
npm run desktop:build
.\src-tauri\target\release\xiaoban-desktop-wallpaper.exe
```

这个 `.exe` 内嵌全部前端与背景资源，运行不需要 Node、Vite 或本地服务器。它依赖系统 WebView2，未把浏览器运行时打进包。当前配置不生成安装器；便携 EXE 本身在打开界面前检测运行时，支持 Windows 10/11，WebView2 最低主版本 109。

### 首次运行组件准备

- 运行时可用：直接打开小伴，不显示准备提示、不下载资源；每次检测真实运行时，没有“安装过就永远跳过”的本地标记。
- 运行时缺失或低于最低版本：显示不依赖 WebView 的原生“正在加载必要资源，请稍后”窗口，下载微软官方 Evergreen 引导程序，验证有效 Microsoft Corporation 签名，然后以当前权限静默安装。默认不请求管理员权限。
- 安装后再次检测运行时，成功才创建控制台与角色窗口。安装器退出码为零不等于启动条件已满足。
- 下载、安装、权限或签名问题显示失败原因，允许重试或暂不启动；准备过程中可取消。下载超时 120 秒，安装等待最多 10 分钟，整体进程有超时与退出清理。
- 首次缺组件时需要联网，后续正常启动无需联网。引导程序和脚本放在唯一临时目录，完成或取消后清理；运行时安装到系统/用户组件位置，独立于小伴 EXE 的体积。
- 不支持的旧 Windows 提示升级系统，不尝试下载旧版运行时。macOS 12+ 使用系统 WKWebView，随系统维护，不下载或替换系统 WebKit；最低系统版本已写入 `.app` 配置。

微软官方说明：[运行时按需检测与安装](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/distribution)。设置中的 `webviewInstallMode` 仅对未来安装包有效；便携 EXE 的准备流程在原生入口中实现。

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

应用使用系统 WKWebView，无需打包浏览器。macOS 配置自动读取 `src-tauri/tauri.macos.conf.json`，开启透明窗口并生成 `.app`，使用本地 ad-hoc 签名；尚未使用 Developer ID 签名或 Apple 公证。透明 WebView 目前使用 Tauri 的 macOS 私有透明接口，不以 Mac App Store 分发为目标。

macOS 悬浮窗口使用 AppKit 浮动层；壁纸使用 Core Graphics 的 desktop window level + 1，位于原桌面背景与桌面图标之间，并忽略鼠标事件以保留 Finder 的操作。角色视线使用 NSEvent 的鼠标逻辑点位置，经过 NSWindow/NSView 转换，处理坐标翻转和 Retina 缩放；壁纸点击只有命中桌面层时才触发，Finder 普通文件夹窗口和其他应用遮挡时不触发。没有安装事件钩子或读取屏幕图像，不主动申请辅助功能和屏幕录制权限。

配置 `CanJoinAllSpaces` 使角色与壁纸参与各桌面空间；AppKit 窗口操作都在主线程执行。macOS 14+ 会关闭 WKWebView 的后台调度限流以保留可见桌宠动画，旧系统的后台表现需要实机验证。

## 使用

设置界面为奶白/暖粉色的左侧导航布局，分为“小伴设置”“动态壁纸”“关于小伴”三页。小伴页保留原 SVG、表情和互动，壁纸页管理内置与订阅内容，关于页展示真实程序版本并检查已配置的更新服务。

动态壁纸支持图片、静音循环 H.264 MP4 与自包含 Canvas/WebGL ES module。可添加 HTTPS 订阅或导入 JSON 清单；下载后的内容校验 SHA-256 并缓存，离线可继续使用。小伴悬浮时可与通用壁纸并存，融入壁纸时叠在所选内容上；“小伴房间”恢复原先的房间场景或系统桌面。下发协议、SDK、限制和更新 JSON 示例见 [delivery-protocol.md](docs/delivery-protocol.md)，Schema 为 [壁纸](docs/wallpapers-v1.schema.json) 和 [更新](docs/updates-v1.schema.json)。

Canvas 运行在独立的低权限壁纸窗口和 opaque-origin iframe 中，不能调用设置、自启动、退出或下载等桌面命令。控制页只展示封面，执行预览使用另一低权限窗口。视频缓存协议支持 byte ranges。更新功能目前为版本/说明检查和打开下载页；未配置更新源会显示未接入，不伪造最新状态，未实现无签名自动安装。

- 控制台切换“悬浮桌宠 / 动态壁纸”，调整 25 种表情、160–420px 尺寸、深夜/晨光背景、30/60 FPS，或暂停动画。
- 悬浮模式：点击角色触发反应，拖动角色移动窗口，右键角色打开控制台。角色实际 SVG 填充区域接收点击，暖光与空白处穿透。
- 壁纸模式：小伴在背景的桌面上，角色上方的桌面点击可以触发反应。应用不拦截桌面鼠标消息，桌面图标继续由 Explorer / Finder 处理。其他窗口遮挡角色时不触发壁纸点击反应。
- 鼠标位置由平台原生层以约 30Hz 采样，即使离开角色窗口也能跟随。无鼠标/键盘钩子、键盘采集或屏幕录制。
- 40 秒无鼠标活动后困倦，90 秒后睡觉；这是原版逻辑，鼠标活动会唤醒。
- 关闭设置窗口或点“收起设置”只隐藏设置面板，小伴继续运行。Windows 左键点击托盘图标打开设置，右键显示菜单；macOS 点击菜单栏图标后选择“打开小伴设置”，也可以通过 Dock 唤起。
- 托盘/菜单栏和设置面板均有“开机自动启动”开关。首次正式运行默认开启；关闭后保存选择，下次启动不会自动重新开启。登录启动时恢复上次的悬浮/壁纸模式、表情、大小、背景、暂停和帧率，只显示小伴，不弹出设置面板。
- 再次打开程序会唤起已有实例的设置，不创建第二只小伴；“退出小伴”才退出全部窗口，恢复原本可见的壁纸。
- macOS 点击 Dock 中的小伴图标会显示、取消最小化并聚焦设置面板。壁纸模式中同样有效，不会切换角色模式或重建壁纸窗口。

## 验证

设置写入当前用户配置目录的 `preferences.json`，通过临时文件替换保存。Windows 使用当前用户 `Run` 启动项 `XiaoBanDesktop`，兼容带空格/中文的 EXE 路径，并识别任务管理器中的禁用状态；macOS 使用 `~/Library/LaunchAgents/XiaoBanDesktop.plist`，随用户登录启动。便携 EXE 或 `.app` 移动后，手动打开一次可刷新已启用的启动路径；macOS 建议放到 Applications，避免下载目录或临时路径失效。登记失败会在设置中提示，不阻止桌宠运行。

开发预览不自动登记自启动，使用独立配置目录。原生 smoke test 使用独立应用标识、独立配置与 `XiaoBanStartupSmoke-<测试标识>` 启动项；Rust 注册测试使用 `XiaoBanStartupTest-<PID>`。结束时仅清理自己的测试项，不修改正式启动项。

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

启动流程另有四个 Rust 测试，覆盖已安装时不调用安装器、安装后重新检测、虚假成功退出码与失败重试。Windows CI 构建便携包并执行原生准备窗口成功/失败/取消模拟；本地 `-VerifyDownload` 还验证内嵌下载工具、进程清理及真实微软签名，不执行安装。真实缺组件的首次安装需在干净 Windows 虚拟机上验证。

- 提供 Windows 与 macOS 的原生接入；macOS 包面向 Apple 芯片，Intel / Universal 打包未纳入此原型。
- 壁纸只覆盖主显示器；未实现多显示器管理、本地原始媒体文件导入或签名安装包。
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
- `src/control.ts`：三页设置界面；`src/content.ts` 与 `public/wallpapers/sandbox.html`：图片/视频与 Canvas 沙箱宿主；`src-tauri/src/library.rs`：目录、资源完整性、缓存协议和更新验证；`src-tauri/src/content.rs`：窗口切换、权限边界和交付命令。
- `src-tauri/src/runtime`：Windows 启动前检测、原生准备窗口、官方引导程序下载与签名验证。成功/失败/取消的 UI 模拟仅在 debug 构建中启用，不会安装组件；真实缺组件的首次安装还需干净虚拟机验证。
- `src-tauri/src/windows.rs`：WorkerW 接入、DPI 坐标转换、只读鼠标采样。
- `src-tauri/src/macos.rs`：AppKit 窗口层级、空间行为、鼠标逻辑点与桌面点击判断。
- `src-tauri/src/coordinates.rs`：坐标转换和桌面层判断的独立测试。

## 大仓开发边界

本项目是桌面客户端，不承担网页、账户或后端业务。角色可以有自己的实现，修改角色行为时运行本地 npm test；不引用 client/xiaban-web 或主后端的私有文件。两种窗口模式共用本应用场景，Windows/macOS 接入分文件。新增能力按本次需求，当前无聊天/账号/云端依赖。
