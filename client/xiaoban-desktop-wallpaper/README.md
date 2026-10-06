# 小伴桌宠 / 动态壁纸原型

Windows 原型：一个可拖动、透明处点击穿透的悬浮小伴，也可以切换成嵌入桌面图标下方的动态壁纸。无聊天、账户、服务器连接或云端依赖。

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

## 使用

- 控制台切换“悬浮桌宠 / 动态壁纸”，调整 25 种表情、160–420px 尺寸、深夜/晨光背景、30/60 FPS，或暂停动画。
- 悬浮模式：点击角色触发反应，拖动角色移动窗口，右键角色打开控制台。角色实际 SVG 填充区域接收点击，暖光与空白处穿透。
- 壁纸模式：小伴在背景的桌面上，角色上方的桌面点击可以触发反应。应用不拦截桌面鼠标消息，桌面图标继续由 Explorer 处理。其他窗口遮挡角色时不触发壁纸点击反应。
- 鼠标位置由 Windows 原生层以约 30Hz 采样，即使离开角色窗口也能跟随。无鼠标/键盘钩子、键盘采集或屏幕录制。
- 40 秒无鼠标活动后困倦，90 秒后睡觉；这是原版逻辑，鼠标活动会唤醒。
- 收起控制台后通过任务栏或托盘菜单打开。关闭控制台或点“退出小伴”会退出全部窗口，恢复原本可见的壁纸。

## 验证

```powershell
npm run build
npm test
powershell -ExecutionPolicy Bypass -File .\scripts\smoke-test.ps1
```

角色测试运行原项目的 13 个行为测试。原生 smoke test 启动生产程序，在悬浮、壁纸、返回悬浮三个阶段检查窗口存在性、前端显示、背景父窗口类别、WS_CHILD 标记和旧窗口清理，输出 `artifacts/smoke-*.json`。桌面 parenting 使用 Explorer 的非公开 WorkerW 协议，需要在不同 Windows 版本上继续验证。

## 当前范围与限制

- 原生壁纸接入仅实现 Windows；macOS 的 AppKit 接入待后续实现，不代表已支持 macOS。
- 壁纸只覆盖主显示器；未实现多显示器管理、开机自启、设置持久化、背景素材导入或签名安装包。
- 壁纸窗口确实作为 Explorer 背景 WorkerW 的子窗口运行，未修改系统壁纸路径或注册表，退出后原壁纸恢复显示。
- 模式切换会重新挂载角色，当前的随机动画/睡眠进度会重置。
- Explorer 重启、显示器热插拔等恢复策略需要进一步完善。背景父窗口异常会提示重新切换模式。
- 点击穿透通过原生采样与实时 SVG 轮廓判断切换，存在约一个采样周期的延迟。
- 这是还原路线验证原型，未对跨浏览器逐像素一致性作保证。

## 代码

- `src/pet.ts`：复用现有角色引擎，DOM 帧更新、真实 SVG 轮廓命中测试。
- `src/scene.css`：程序绘制的轻量动态背景，无视频资源。
- `src-tauri/src/main.rs`：控制台、两种窗口生命周期、托盘、命令与诊断。
- `src-tauri/src/windows.rs`：WorkerW 接入、DPI 坐标转换、只读鼠标采样。
