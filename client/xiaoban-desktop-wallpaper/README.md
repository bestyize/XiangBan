# 小伴桌宠 / 动态壁纸原型

Windows / macOS 原型：一个可拖动、透明处点击穿透的悬浮小伴，也可以切换成位于桌面图标下方的动态壁纸。无聊天、账户、服务器连接或云端依赖。角色、场景、控制台与互动逻辑共用，桌面宿主分平台接入。

本机生产构建实测：Windows x64 EXE 为 3,597,312 字节（约 3.60MB），内嵌前端与两套程序背景，不包含系统 WebView2。原生切换测试已在主屏 2880×1800、175% 缩放下通过，确认壁纸为 WorkerW 子窗口；验证报告位于本地 `artifacts/smoke-*.json`。本地试用文件为 `artifacts/XiaoBan-Windows-x64.exe`，ZIP 包为 `artifacts/XiaoBan-Windows-x64.zip`。

角色直接导入仓库 `web/src/robot` 中的 SVG 渲染器、Animator 和 Mind，保留原来的渐变、发光、连续表情、呼吸、眨眼、耳朵抖动、鼠标跟随、随机点击反应和自动睡眠。`src/pet.ts` 仅适配 DOM 与桌面输入；未复制或重新设计角色。暖光使用原 RobotStage 的尺寸与渐变。

## 在 Windows 上运行

需要 Node.js 22.12+（或 24）、Rust MSVC 工具链、Visual Studio C++ 构建工具和 WebView2 Runtime。已有仓库安装依赖的情况下：

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

这个 `.exe` 内嵌全部前端与背景资源，运行不需要 Node、Vite 或本地服务器。它依赖系统 WebView2，未把浏览器运行时打进包。当前配置不生成安装器；WebView2 未安装的机器应先安装运行时。

单独浏览器预览：`npm run dev`，访问 `http://127.0.0.1:1420`。浏览器只能预览角色与场景，不能切换原生桌面模式。

## 在 macOS 上运行（Apple 芯片）

目标为 macOS 12+、Apple silicon arm64。构建需要 Node.js 24、Rust 和 Xcode Command Line Tools。先安装仓库 `web` 目录的 npm 依赖以运行共用角色测试，然后在本目录执行：

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

- 控制台切换“悬浮桌宠 / 动态壁纸”，调整 25 种表情、160–420px 尺寸、深夜/晨光背景、30/60 FPS，或暂停动画。
- 悬浮模式：点击角色触发反应，拖动角色移动窗口，右键角色打开控制台。角色实际 SVG 填充区域接收点击，暖光与空白处穿透。
- 壁纸模式：小伴在背景的桌面上，角色上方的桌面点击可以触发反应。应用不拦截桌面鼠标消息，桌面图标继续由 Explorer / Finder 处理。其他窗口遮挡角色时不触发壁纸点击反应。
- 鼠标位置由平台原生层以约 30Hz 采样，即使离开角色窗口也能跟随。无鼠标/键盘钩子、键盘采集或屏幕录制。
- 40 秒无鼠标活动后困倦，90 秒后睡觉；这是原版逻辑，鼠标活动会唤醒。
- 收起控制台后通过任务栏、Dock 或托盘/菜单栏菜单打开。关闭控制台或点“退出小伴”会退出全部窗口，恢复原本可见的壁纸。
- macOS 点击 Dock 中的小伴图标会显示、取消最小化并聚焦设置面板。壁纸模式中同样有效，不会切换角色模式或重建壁纸窗口。

## 验证

```powershell
npm run build
npm test
powershell -ExecutionPolicy Bypass -File .\scripts\smoke-test.ps1
```

角色测试运行原项目的 13 个行为测试。原生 smoke test 启动生产程序，在悬浮、壁纸、返回悬浮三个阶段检查窗口存在性、前端显示、背景父窗口类别、WS_CHILD 标记和旧窗口清理，输出 `artifacts/smoke-*.json`。桌面 parenting 使用 Explorer 的非公开 WorkerW 协议，需要在不同 Windows 版本上继续验证。

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

- 提供 Windows 与 macOS 的原生接入；macOS 包面向 Apple 芯片，Intel / Universal 打包未纳入此原型。
- 壁纸只覆盖主显示器；未实现多显示器管理、开机自启、设置持久化、背景素材导入或签名安装包。
- Windows 壁纸窗口作为 Explorer 背景 WorkerW 子窗口；macOS 壁纸使用 AppKit 桌面层窗口。未修改系统壁纸路径或偏好，退出后原壁纸恢复显示。
- 模式切换会重新挂载角色，当前的随机动画/睡眠进度会重置。
- Explorer 重启、显示器热插拔、macOS Stage Manager / 全屏空间等恢复策略需要进一步实机完善。Windows 背景父窗口异常会提示重新切换模式。
- 点击穿透通过原生采样与实时 SVG 轮廓判断切换，存在约一个采样周期的延迟。
- 这是还原路线验证原型，未对跨浏览器逐像素一致性作保证。

## 代码

- `src/pet.ts`：复用现有角色引擎，DOM 帧更新、真实 SVG 轮廓命中测试。
- `src/scene.css`：程序绘制的轻量动态背景，无视频资源。
- `src-tauri/src/main.rs`：控制台、两种窗口生命周期、托盘、命令与诊断。
- `src-tauri/src/windows.rs`：WorkerW 接入、DPI 坐标转换、只读鼠标采样。
- `src-tauri/src/macos.rs`：AppKit 窗口层级、空间行为、鼠标逻辑点与桌面点击判断。
- `src-tauri/src/coordinates.rs`：坐标转换和桌面层判断的独立测试。
