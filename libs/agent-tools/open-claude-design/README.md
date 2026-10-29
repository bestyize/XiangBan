# open-claude-design

小伴的内置设计工具，独立于模型厂商和外部 CLI。依赖 agent-kernel，内核不反向依赖本模块。桌面宿主已接入，当前为首版功能实现。

## 使用

真实用户消息提到设计、画板、原型或 design- 正式编号时，工具可被发现；聊天顶部固定「设计稿」入口可以搜索全部作品。例如“做一份音乐播放器设计稿，包含首页、播放页和设置页”。模型通过工具先读取格式，再创建作品；创建重试使用相同 request_id。已有作品可继续修改，不必每次重复关键词。

聊天页交付卡片打开右侧画布，分屏支持拖动和键盘调整，也可以弹出独立原生窗口（同项目复用、最大化、F11/系统全屏、记忆位置与大小、回到分屏）。两种模式共用源数据与视图状态，AI 修改广播同步。Ctrl/⌘+滚轮与捏合以指针为中心缩放（10%–800%），普通滚轮/Shift 滚轮平移，Space/中键拖动，0 适应全部、1 恢复 100%。编辑态 iframe 转发画布手势；Play 保留页面滚动。资料库「我的文档」也可打开、分组、重命名、删除和导入设计源包。一个项目只有一条文档，多个画板共用同一份源数据；从另一会话继续时关联原项目，不创建副本。

## 模块边界

- `src/lib.rs`：工具、真实用户关键词上下文、项目校验、版本检查、创建去重、会话关联、历史与恢复草稿。
- `src/archive.rs`：有大小限制的 ZIP 源包导入导出、独立 HTML 预览导出。
- `runtime/support.js`：独立实现的 DC HTML 兼容运行时子集，不使用 Anthropic 的 support.js。
- `client/xiaoban-desktop/src-tauri/src/design.rs`：主窗口命令、原生文件选择、专用预览响应。
- `client/xiaoban-desktop/src/ui/design/`：多画板画布、选择/抓手/Play、属性、源码、批注、撤销重做、历史和冲突处理。

生命周期：create 立即保存 draft（允许缺少页面），逐页 save 为 generating，complete 检查源文件与本地引用后变为 ready，failed 保留受阻现场。缺失画板显示占位；Agent 停止前对本轮未完成写入最多续行 4 次、无进展最多 2 次，避免假交付与无限重试。每条聊天/队列消息固定提交时的设计稿编号，跨会话打开同源，copy 创建独立副本，delete 要求最新 revision。

项目主文件：`<data root>/data-library/files/<design-id>/project.json`，包含 canvas 与源文件；历史保留最近 30 个完整版本，失败草稿单独保存为 draft.json。canonical 项目原子替换，保存必须携带读到的版本。删除与保存使用同一写锁，过期保存不能复活已删除项目。资料库索引在源项目落盘后更新，下次读取可修复中断的索引更新。

预览放在无 allow-same-origin 的 sandbox iframe 内，并由独立响应设置 CSP；控制页的脚本策略不放宽。父窗口检查 frame 来源与消息 token，主窗口可以管理作品；独立设计窗口只能读写当前项目及画布视图，不能访问其他项目或聊天、系统命令。内容不能使用桌面 IPC、网络 fetch 或本机路径。

## 格式边界

创作质量要求见 [CRAFT.md](CRAFT.md)（风格方向与 tokens、真实内容、组件状态、交付前自查），随 format 动作一并返回。

详见 [FORMAT.md](FORMAT.md)：支持 v3 boards/order 与旧版 artboards 索引，保留未知索引字段和原始 DC 程序。选择文字或样式时修改源模板，不用渲染后的 DOM 覆盖程序；动态文字通过批注修改。

支持 x-dc、helmet、表达式、sc-if/sc-for、DCLogic 状态和事件、参数默认值、画板间相对链接。图片使用内嵌 data URL，CSS 使用内联样式；本地 CSS/SVG/JS 等文本可在源包中保留，但不会作为外部资源加载。

当前不是 Anthropic 专有运行时的完整替代：React、其他专有组件/编辑器扩展和二进制素材 ZIP 暂不支持；不保证任意 Claude Design 项目无损运行。保存源包不会转换原程序，独立 HTML 使用本模块运行时。sandbox/CSP 提供权限隔离，不提供独立进程或无限循环中断能力。

限制：100 个画板、300 个源文本文件（自动生成的 support.js 不计入，含运行时总计不超过 400 个文件）、2 MiB/文件、16 MiB/项目、最多 64 个附加会话。

## 验证

仓库根目录：

```sh
cargo test --locked --manifest-path libs/agent-tools/open-claude-design/Cargo.toml
```

桌面客户端目录：

```sh
npx vitest run src/ui/design src/ui/library/library.test.ts src/ui/pages/ChatPage.test.ts
npm run build
```

验证范围与剩余平台验收见 [实施说明](../../../docs/agent/open-claude-design.md)。
