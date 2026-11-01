# 设计预览运行时

`support.js` 与 `editor.js` 是小伴独立实现的 DC HTML 与画布编辑协议；不打包 Anthropic 的实现。`react-bridge.js` 在同一个 opaque sandbox 内加载本地 React 组件。普通桌面运行和 Rust 构建不依赖 Node，直接使用已入库的 `vendor/react-bridge.js`。

修改 React 适配器或依赖后，在此目录执行：

```sh
npm ci
npm run build
```

提交源文件、package.json、package-lock.json、生成的 vendor/react-bridge.js 与 vendor/LICENSES.txt。构建脚本收集实际打包依赖的许可证；缺失许可证时失败。锁文件使用 npmmirror，受限网络可按根 AGENTS.md 使用官方源替换下载主机。

React 18.3.1 与 Sucrase 3.35.0 固定版本。本地模块通过受限 require 映射解析，提供 React/ReactDOM、项目内 JS/JSX/TS/TSX/JSON/CSS/媒体。没有 Node require、宿主 IPC 或外部网络权限；外部 npm 依赖须提供本地源码或预构建版本。编译、执行都在预览 iframe 中，错误显示在画板上。

验收矩阵见 [兼容性报告](../../../../docs/agent/open-claude-design-compatibility.md)。
