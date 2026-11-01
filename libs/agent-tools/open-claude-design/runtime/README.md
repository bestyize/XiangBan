# 设计预览运行时

`support.js` 与 `editor.js` 是小伴独立实现的 DC HTML 与画布编辑协议；不打包 Anthropic 的实现。`react-bridge.js` 在同一个 opaque sandbox 内加载本地 React 组件。普通桌面运行和 Rust 构建不依赖 Node，直接使用已入库的 `vendor/react-bridge.js`。

`capture.js` 在同一 sandbox 中接收带 token/requestId 的截图请求，以 html-to-image 1.11.13 生成 PNG；过滤编辑轮廓和批注覆盖层，保留原型当前状态。单边最多 8192 CSS 像素、输出最多 16,777,216 像素，支持 1×/2×。桌面宿主校验尺寸后用 jsPDF 4.2.1 生成每画板一页的栅格 PDF，再通过现有原生保存对话框落盘。截图不新增宿主 IPC、网络或子 iframe 权限。

修改 React 适配器、截图器或依赖后，在此目录执行：

```sh
npm ci
npm run build
```

提交源文件、package.json、package-lock.json、生成的 vendor/react-bridge.js、vendor/capture.js 与 vendor/LICENSES.txt。构建脚本收集实际打包依赖的许可证；缺失许可证时失败。锁文件使用 npmmirror，受限网络可按根 AGENTS.md 使用官方源替换下载主机。

浏览器会暂停离屏 iframe 的 requestAnimationFrame。构建插件将 html-to-image 已解码图片后的这一处等待改为 setTimeout，使未滚动到的画板可批量截图；升级依赖时若原代码不匹配，构建直接失败，必须复核。自有截图入口也不等待动画帧。使用 SVG foreignObject 的截图仍受浏览器绘制能力影响；WebGL、视频、跨域素材和特定滤镜没有全面保真保证。PDF 当前为 96dpi 白底栅格图，不含可选择文字、矢量路径或 Flow 自动分页。

React 18.3.1 与 Sucrase 3.35.0 固定版本。本地模块通过受限 require 映射解析，提供 React/ReactDOM、项目内 JS/JSX/TS/TSX/JSON/CSS/媒体。没有 Node require、宿主 IPC 或外部网络权限；外部 npm 依赖须提供本地源码或预构建版本。编译、执行都在预览 iframe 中，错误显示在画板上。

验收矩阵见 [兼容性报告](../../../../docs/agent/open-claude-design-compatibility.md)。
