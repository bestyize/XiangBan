# open-claude-design

小伴的内置设计工具，独立于模型厂商和外部 CLI。依赖 agent-kernel，内核不反向依赖本模块。桌面宿主已接入，当前以复刻 Claude Design 的格式与编辑交互为目标；[逐项验收与剩余差距](../../../docs/agent/open-claude-design-compatibility.md)。

## 使用

设计工具始终可通过工具目录或搜索发现，完整参数按需加载；真实用户提到设计、画板、原型或 design- 正式编号，以及紧接这些任务的简短“继续”请求，会附加设计工作台提示。聊天顶部固定「设计稿」入口可以搜索全部作品。例如“做一份音乐播放器设计稿，包含首页、播放页和设置页”。模型通过工具先读取格式，再创建作品；创建重试使用相同 request_id。已有作品可继续修改，不必每次重复关键词。

聊天页交付卡片打开右侧画布，分屏支持拖动和键盘调整，也可以弹出独立原生窗口（同项目复用、最大化、F11/系统全屏、记忆位置与大小、回到分屏）。两种模式共用源数据与视图状态，AI 修改广播同步。Ctrl/⌘+滚轮与捏合以指针为中心缩放（10%–800%），普通滚轮/Shift 滚轮平移，Space/中键拖动，0 适应全部、1 恢复 100%。编辑态 iframe 转发画布手势；Play 保留页面滚动。资料库「我的文档」也可打开、分组、重命名、删除和导入设计源包。一个项目只有一条文档，多个画板共用同一份源数据；从另一会话继续时关联原项目，不创建副本。

## 模块边界

- `src/lib.rs`：工具、设计任务上下文、项目校验、版本检查、创建去重、会话关联、历史与恢复草稿。
- `src/archive.rs`：ZIP 源包导入导出、独立 HTML 预览导出。
- `src/html_import.rs`：官方单 HTML 数据岛解包，恢复画板、组件与嵌入资源；不执行导出器脚本。
- `src/file_store.rs`：原件保留、按内容摘要存储文件、历史共用资源与按需读取素材。
- `runtime/capture.js` 与 `src/rendered_export.rs`：隔离预览截图及 PNG/PDF 落盘前校验；桌面支持 PNG 1×/2×、单画板及按位置排序的全部画板栅格 PDF。
- `runtime/support.js`：独立实现的 DC HTML 兼容运行时子集，不使用 Anthropic 的 support.js。
- `client/xiaoban-desktop/src-tauri/src/design.rs`：主窗口命令、原生文件选择、专用预览响应。
- `client/xiaoban-desktop/src/ui/design/`：多画板画布、选择/抓手/Play、属性、源码、批注、撤销重做、历史和冲突处理。

生命周期：create 立即保存 draft（允许缺少页面），逐页 save 为 generating，complete 检查源文件与本地引用后变为 ready，failed 保留受阻现场。缺失画板显示占位；Agent 停止前对本轮未完成写入最多续行 4 次、无进展最多 2 次，避免假交付与无限重试。每条聊天/队列消息固定提交时的设计稿编号，跨会话打开同源，copy 创建独立副本，delete 要求最新 revision。

项目主文件：`<data root>/data-library/files/<design-id>/project.json`，包含 canvas 与文件摘要索引（storageVersion 2）；源文件和二进制素材保存在 `objects/<sha256>`。历史保留最近 30 个版本清单，失败草稿单独保存为 draft.json，共用未变化的文件对象。旧版内嵌文件项目仍可读取，下次保存升级；对外 Project 接口仍返回完整 files。导入原件由 original.json 引用独立对象，编辑不会覆盖，菜单「导入原件」可逐字节导出；当前未清理历史淘汰后不再引用的对象。canonical 项目原子替换，保存必须携带读到的版本。删除与保存使用同一写锁，过期保存不能复活已删除项目。资料库索引在源项目落盘后更新，下次读取可修复中断的索引更新。

预览放在无 allow-same-origin 的 sandbox iframe 内，并由独立响应设置 CSP；控制页的脚本策略不放宽。父窗口检查 frame 来源与消息 token，主窗口可以管理作品；独立设计窗口只能读写当前项目及画布视图，不能访问其他项目或聊天、系统命令。内容不能使用桌面 IPC、任意外部网络 fetch 或本机路径；专用设计素材协议允许按需读取图片/字体/媒体。

## 格式边界

创作质量要求见 [CRAFT.md](CRAFT.md)（风格方向与 tokens、真实内容、组件状态、交付前自查），随 format 动作一并返回。

详见 [FORMAT.md](FORMAT.md)：支持 v3 boards/order 与旧版 artboards 索引，保留未知索引字段和原始 DC 程序。选择文字或样式时修改源模板，不用渲染后的 DOM 覆盖程序；动态文字通过批注修改。

支持 x-dc、helmet、表达式、sc-if/sc-for、DCLogic 状态和事件、参数默认值、画板间相对链接。支持原始 `_blob` 图片及常见媒体/字体的 ZIP 导入与字节保真导出；接口使用 data URL，磁盘按原始二进制独立存储，原生预览按需加载素材并解析项目内相对/根路径。CSS（含相对素材与 @import）和普通本地 JS 在预览中内联；原始文件保持不变。

已支持按名称的 dc-import、本地 x-import React/JSX/TSX、组件状态/生命周期与画布便签。React 适配器的构建和开源许可证见 [runtime](runtime/README.md)。远程 CDN、任意 npm/Node 环境、原生 module script 和未验证的专有扩展尚未完整兼容；PDF 矢量/Flow 分页、PPTX、云端协作等差距见验收报告，不保证任意 Claude Design 项目无损运行。保存源包不会转换原程序，独立 HTML 使用本模块运行时。sandbox/CSP 提供权限隔离，不提供独立进程或无限循环中断能力。

官方单 HTML 可导入并展开为多个 `.dc.html`；保留导入原件，按导出顺序安排画板，界面列出缺失引用。HTML 中没有的原画布坐标、分组、批注及源文件名映射不能恢复。官方图片 ZIP 和 PDF 不包含可编辑设计源码；图片 ZIP 会给出明确提示。完整源包仍用 canvas.json + 页面 + 素材 ZIP 导入，与官方图片 ZIP 不同。

已取消单文件 2 MiB、项目/导入包 16 MiB 限制。保留 100 个画板、300 个源文件（包含素材；生成运行时不计入，含运行时最多 405 个文件）、最多 64 个附加会话，以及路径/嵌套校验。解包和对外 Project 仍使用内存，不承诺任意体积可用；PNG/PDF 渲染导出的像素及文件限制独立保留。

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
