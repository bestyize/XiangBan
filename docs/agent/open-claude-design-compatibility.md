# Claude Design 复刻与兼容验收

更新：2026-11-01。目标按用户要求设为 **复刻 Claude Design：格式、运行行为、编辑交互和交付流程都要对照**。本文件是持续验收清单，不把“能打开 HTML”或“样例全部通过”等同于全面复刻。

## 依据

- [Anthropic 产品介绍](https://www.anthropic.com/news/claude-design-anthropic-labs)：产品范围、设计系统、分享和导出。
- [官方使用与键盘交互说明](https://support.claude.com/en/articles/14604416-get-started-with-claude-design)：画布、批注、图层、焦点和快捷键行为。
- [官方设计系统说明](https://support.claude.com/en/articles/14604397-set-up-your-design-system-in-claude-design)：组件/令牌来源、组织发布与同步。
- [实际参考画布](https://claude.ai/artifact/GzhNp5y7ehkzSrWWExkqzg)及仓库中的 `client/xiaoban-desktop/docs/design-specs/ui-upgrade/`：39 个 DC HTML、13 个二进制 `_blob` 文件、原始 `canvas.json` 与导出的 `support.js`。参考稿未修改。

2026-11-01 已通过用户登录的 Edge 打开同一画布，实际查看选择、Properties / Code / Tweaks、图层、缩放、播放、绘图和导出菜单；没有修改在线原稿内容。在线版本同样包含 39 个画板。以下区分已实测行为与待实现项目，不声称完成逐像素或全部手势对照。原始导出只用于观察格式与回归，不复制 Anthropic 运行时代码。React/ReactDOM/Sucrase 来自开源 npm 包，版本、构建方法和许可证随仓库保存。

## 已实现与验证

| 对照项 | 调研发现的差异 | 当前实现与证据 |
| --- | --- | --- |
| 实际源包导入 | 原来只接受文本，13 个 `_blob` 图片会拒绝导入 | 二进制内容以有类型校验的 data URL 存储；ZIP 导出恢复原始字节。真实整包往返覆盖 39 个画板与所有资源，未知索引字段及原始 support.js 保留 |
| 组件按名称引用 | 真实稿使用 `dc-import name="Market"`，原来只识别路径 | 同时支持 name/component 和 src/file、相对目录、根目录回退；缺失组件在完成校验中报告 |
| 模板语法 | 对象字面量、箭头函数、多行表达式含花括号时解析失败 | 平衡分隔符解析；支持 style 对象/数组、组件 kebab-case 参数、dc-props、sc-helmet、自闭合 import 标签 |
| 嵌套组件交互 | 每次重绘重建组件，状态丢失；插槽复制丢事件；子组件样式被清理 | 按组件位置保留独立实例，支持 mount/update/unmount、setState 回调、forceUpdate；插槽按调用方作用域渲染并保留事件；样式与滚动位置随重绘保留 |
| 原型表单 | 原生 change 与 React 风格 onChange 的触发时机不同，textarea value 与布尔属性不完整 | 文本 onChange 映射 input；textarea、readonly 等属性同步；编辑态不执行原型操作。真实市场审查页在浏览器中勾选后启用“确认安装” |
| 样式与资源 | 本地 CSS 相对图片路径、动态图片地址和更多伪类未处理 | CSS @import/媒体条件、图片与字体 URL 解析；表达式图片地址解析；style-hover/focus/active 等生成 CSS 伪类规则 |
| React 组件 | x-import 不渲染，也没有明确提示 | 隔离 iframe 内置 React 18.3.1 + ReactDOM + Sucrase；本地 JS/JSX/TS/TSX、相对模块、JSON/CSS/媒体依赖、Hooks、错误边界和卸载；不依赖桌面安装 Node，不访问宿主 require |
| 中文预览 | 大型注入脚本把 charset 挤到浏览器编码探测范围外 | 生成预览将 UTF-8 声明放在前部；真实浏览器复查中文与导入组件正常 |
| 独立 HTML | 每个画板重复打包全部资源和运行时 | 导出文件共用资源字典与运行时，实例化时构造隔离 iframe；39 画板参考包导出小于 8 MiB |
| 原始画布便签/分组标题 | notes 只保留数据、不显示 | 渲染 title/maxW/maxH、负坐标；支持选择、拖动、宽度调整、编辑、复制、删除、撤销重做，并保留未知字段 |
| 属性与键盘焦点 | 属性面板展开时仍显示浮动工具栏；关闭后焦点未返回；清空选择后方向键仍移动首个画板 | 属性打开隐藏工具栏；关闭返回属性按钮；工具栏方向键/Home/End/Escape；清空选择后不移动或删除无关画板 |
| 官方面板布局 | Edge 实测：关闭属性时没有常驻图层栏，图层位于右侧；Tweaks 独立展示参数 | 图层移入右侧 Properties / Code 顶部，关闭面板一起隐藏；画板标题增加 Tweaks 入口，参数修改写回 data-props，支持撤销 |
| Code 元素编辑 | 官方 Code 是所选元素的 CSS/属性编辑，选中画板时仍显示画板属性 | 新增元素 CSS、HTML 属性编辑与版本保护，保持模板表达式、事件、节点标识；保留整份源文件入口。当前 HTML 属性使用 JSON 独立输入，尚未达到官方统一编辑器外观/语法 |
| 缩放菜单 | Edge 实测固定 25/50/75/100/150/200%，菜单含布局参考线、撤销重做 | 相同档位与菜单动作、键盘上下/Home/End/Escape；只改视图，不写作品版本 |
| 属性颜色 | 原始 computed rgb 直接传入 color input，浏览器不能正常显示 | 转换为有效 hex；有可解析的不透明背景时显示文本对比度，透明/未知背景不猜测 |
| 已有编辑操作 | 需以官方交互文档为指标继续验证 | 现有选择、多选、框选、拖动/缩放、对齐/分布、图层树、复制/粘贴/编组、Alt+方向键移动、画板键盘操作、批注/历史等保留并纳入针对性测试 |

React 的标准生命周期实现参考 [React createRoot](https://react.dev/reference/react-dom/client/createRoot)；本地 TSX 转换使用 [Sucrase](https://github.com/alangpierce/sucrase)。这不表示可运行任意 npm/Node 工程。

## 尚未满足“全面复刻”的项目

| 项目 | 当前差距 / 后续验收要求 |
| --- | --- |
| 官方编辑器完整外观与手势 | 已完成 Edge 的面板/菜单对照与部分修正。浮动顶栏、统一层级树、可拖动面板宽度、精确选择轮廓、多画板播放窗口、完整页面管理仍有差异；触控板、多屏及 Windows/macOS 原生窗口仍需打包应用实测 |
| 手绘与形状 | Edge 确认 Pencil / Pen、Rectangle / Oval / Arrow / Line。当前支持画板内文字/矩形/椭圆插入及便签，尚无官方自由画布上的全部绘制手势、钢笔路径和箭头编辑 |
| 画板高级属性 | Edge 确认 Print mode Flow / Fixed、Letter / A4、自动高度、背景、分角圆角与 Layout guide。当前宽高/预设可编辑，这些官方高级属性尚未完整实现 |
| 动态内容直接编辑 | 循环生成内容、导入组件内部和 React 输出仍通过批注改源程序；尚无官方式逐实例覆盖和所有动态层的直接编辑 |
| 任意模块环境 | 远程 CDN/包下载、任意 npm 依赖、Node 内建模块、原生 ES module script、特定专有扩展仍不等价。外部依赖必须随源包提供，缺失时报告错误 |
| 动画与高级模板 | 原始运行时中可观察到的 data-keyframes-* 等扩展尚无完整样例验收；不能据此声明支持全部动画、WebGL 或高级组件 |
| PNG、PDF、PPTX、Google Slides | Edge 画板菜单实测 PNG 1×/2×、单画板 PDF、位置排序的全部画板 PDF、HTML ZIP；官方文档另有 PPTX/Google Slides。本地导出仍为 ZIP、独立 HTML、Markdown，尚未实现其余格式 |
| 云端共享与多人协作 | 无 Claude 账号/组织权限、多人实时协作及其云端分享服务；本地多窗口同源与版本冲突保护不能替代这些能力 |
| 组织设计系统与连接器 | 已有本地 tokens、@dsCard、组件引用；组织发布、从 Figma/代码库自动抽取、design-sync 双向同步和第三方发布尚未复刻 |
| 完整数据兼容 | 目前基准为已入库的 v3 与现有 legacy 测试；新版本私有字段保留但不保证理解所有语义。严格保持项目/文件/画板上限，不承诺任意体积源包 |

**结论：已补齐可验证的一批格式与交互差距；尚不能标记“全面兼容”或“完整复刻完成”。** 后续必须逐条关闭上述差距，并记录对照环境、可复现操作与实际结果。

## 复刻验收指标

- **格式往返**：参考源包 39/39 画板、13/13 二进制资源，无源文件字节损失；未知索引字段保留。扩大样本后按每个源包单独计数。
- **运行行为**：相同初始参数、点击/输入序列得到相同状态；真实市场审查页的确认勾选和安装按钮联动已在本地浏览器验证。不能用“无异常”代替行为对照。
- **编辑交互**：逐项覆盖选择/退出、图层导航、属性/Code/Tweaks、便签、缩放、播放、绘图、撤销重做；每项记录官方动作、本地动作、结果及剩余差异。
- **外观**：相同画板、参数、缩放和视口截图比较。当前只完成可视复查，尚未建立像素差阈值，不报告“视觉还原百分比”。
- **交付**：每个导出格式均需验证文件可打开、资源完整、尺寸/分页/画板顺序正确；未实现的格式不得计为通过。
- **完成门槛**：已发现差距全部关闭，并补原生应用端到端验证，才能标记“全面复刻”。现阶段不以通过测试数量冒充整体兼容率。

## 验证与复现

仓库根目录：

```sh
cargo test --locked --manifest-path libs/agent-tools/open-claude-design/Cargo.toml --lib
```

桌面目录：

```sh
npx vitest run src/ui/design src/ui/library/library.test.ts src/ui/pages/ChatPage.test.ts
npx vue-tsc --noEmit
```

真实源包回归在 `archive.rs` 中，运行时/交互回归在 `compatibility.test.ts`、`runtime.test.ts`、`canvas-notes.test.ts`、`ElementCode.test.ts`、`DesignWorkbench.test.ts`。渲染参考稿用于浏览器复查：

```sh
cargo run --locked --manifest-path libs/agent-tools/open-claude-design/Cargo.toml \
  --example render_reference -- /tmp/ocd-reference-preview
python3 -m http.server 4177 --bind 127.0.0.1 --directory /tmp/ocd-reference-preview
```

打开 `http://127.0.0.1:4177/index.html` 检查本模块导出的独立 HTML。该命令不会修改参考稿或真实用户作品。浏览器工作台验收使用模拟存储；原生文件对话框、Tauri 协议与真实模型全流程不能用它替代。本轮未运行全桌面打包、未推送、未触发工作流。

本轮结果：完整桌面相关回归此前 214 项通过；Edge 对照后变更相关回归 97 项通过，TypeScript 检查通过；Rust 23 项通过。随后新增的 Code 草稿保护 2 项、多目录共享运行时 1 项定向回归均通过（累计本轮变更相关前端 99 项、Rust 24 项）。工作台浏览器确认右侧 Tweaks 和无常驻左图层栏，独立 HTML 浏览器确认 39 画板文件的中文/图片渲染。
