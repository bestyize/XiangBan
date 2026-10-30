# 小伴桌面 UI 升级设计稿

状态：设计参考。展示数据为模拟内容，不是客户端运行入口，不参与构建打包。与现行实现或专项规范不一致时，以[专项规范](../README.md)和代码为准。

- 在线画布：<https://claude.ai/artifact/GzhNp5y7ehkzSrWWExkqzg>（Claude Design 画布，可评论、可在画布里直接改）
- 内容：桌面主界面（新建任务、AI 插件、会话、装扮、提醒）、设置弹窗、超级小伴悬浮对话框、资料库、实时对话、AI 插件接入小伴市场、输入框的技能 / 专家 / 连接器 / 内置工具，以及任务卡、选择卡、权限菜单的现状与优化方案对比。
- 主题色：多数画板支持 6 套主题色（red / green / blue / purple / amber / graphite），在线画布的属性面板里切换；本地预览使用默认红色。

## 本地预览

画板引用了绝对路径 `/_blob/<id>` 的图片，需要以本目录为根目录启动一个静态服务，不能双击打开：

```bash
cd client/xiaoban-desktop/docs/design-specs/ui-upgrade
python3 -m http.server 8000
```

浏览器打开 `http://localhost:8000/` 查看全部画板（按画布分组，可调缩放），或打开单个 `*.dc.html` 查看全尺寸画板。

## 目录说明

| 文件 | 说明 |
| --- | --- |
| `index.html` | 本地总览页，读取 `canvas.json` 按分组渲染所有画板（仓库自加，不在画布里） |
| `canvas.json` | 画布索引：每个画板的标题、位置、尺寸，以及画布上的分组标题和便签 |
| `*.dc.html` | 每个画板的完整源码，一个文件一个画板；`WB-Market-*` 通过 `<dc-import>` 复用 `Market.dc.html` 的不同界面 |
| `support.js` | 画板渲染运行时，从画布导出，勿手改 |
| `_blob/` | 画板引用的图片（伙伴形象、壁纸缩略图等），文件名即画布资源 ID |

## 画布上的说明便签

**输入框规则**：技能为一次性，敲 `/` 或 `+ › 技能` 选择，最多 3 个，发送后清除；专家 / 专家团为会话级单选，放输入框右上角；连接器为会话级可多选，放专家右边，最多显示 3 个图标；内置工具（操作电脑 / 操作手机 / 设计画板）为一次性单选，从 `+ › 工具` 选择，chip 和技能并排在左侧，操作电脑首次使用要先启用；发送后消息上方留一行「技能 · 专家 · 连接器」；不新增入口，管理走 AI 插件页，找更多走小伴市场。

**小伴市场接入原则**：沿用现有「AI 插件」页，只在筛选行加来源切换；安装一律先审查（说明、文件清单、逐文件 SHA256、脚本预览），网页链接只能唤起确认窗口；按清单摘要锁定版本；市场不可用时退到 SkillHub；更新只提示不自动更新，旧版本保留为备份。

**语音入口分工**：录音按钮一次一句、不朗读、不自动发送；实时对话连续免提、可打断、每轮自动发送并朗读；按住快捷键说话松手即发。

## 与在线画布同步

本目录是画布 `project/` 下文件的副本，`*.dc.html` 与 `canvas.json` 保持原样未改。

- **画布 → 仓库**：在画布里改完后，让 AI 按上面的链接重新拉取 `project/` 下的文件和 `_blob` 图片，覆盖本目录后提交。
- **仓库 → 画布**：改了本目录的 `*.dc.html` 后，让 AI 把改动的文件发布回同一个画布链接；新增图片要先上传到画布资源库，再用返回的 `/_blob/<id>` 引用。
- 两边都改过时先拉取画布最新版再合并，避免覆盖别人在画布里的手动修改。

## 画板清单

### 小伴桌面 · 主界面（WorkBuddy 骨架 · 精致版 · 多主题色）

| 画板 | 文件 | 尺寸 |
| --- | --- | --- |
| ④ 新建任务 · 主题色可切换 | [WB-New.dc.html](WB-New.dc.html) | 1280×800 · 可交互 |
| ④ AI 插件 | [WB-Plugins.dc.html](WB-Plugins.dc.html) | 1280×800 · 可交互 |
| ④ 会话 · 头像菜单 | [WB-Chat.dc.html](WB-Chat.dc.html) | 1280×800 · 可交互 |
| ④ 装扮 | [WB-Studio.dc.html](WB-Studio.dc.html) | 1280×800 · 可交互 |
| ④ 装扮 / 特效 | [WB-Effects.dc.html](WB-Effects.dc.html) | 1280×800 · 可交互 |
| ④ 装扮 / 组件 | [WB-Widgets.dc.html](WB-Widgets.dc.html) | 1280×800 · 可交互 |
| ④ 装扮 / 伙伴 | [WB-Pets.dc.html](WB-Pets.dc.html) | 1280×800 · 可交互 |
| ④ 伙伴 · 设置弹窗 | [WB-Pets-Dialog.dc.html](WB-Pets-Dialog.dc.html) | 1280×800 · 可交互 |
| ④ 提醒 | [WB-Remind.dc.html](WB-Remind.dc.html) | 1280×800 · 可交互 |
| ④ 会话 · 选择模型（含思考强度） | [WB-Chat-Model.dc.html](WB-Chat-Model.dc.html) | 1280×800 · 可交互 |
| ④ 会话 · 任务执行（优化后的任务卡片） | [WB-Chat-Task.dc.html](WB-Chat-Task.dc.html) | 1280×800 |
| ④ 任务执行卡片 · 现状 vs 优化方案（含全部状态） | [WB-Task-Card.dc.html](WB-Task-Card.dc.html) | 1440×2480 |
| ④ 选择卡 · 现状 vs 优化方案（含各状态） | [WB-Ask-Card.dc.html](WB-Ask-Card.dc.html) | 1440×2310 |
| ④ 权限菜单 · 现状 vs 优化方案 | [WB-Perm-Menu.dc.html](WB-Perm-Menu.dc.html) | 1000×1215 |

### ④ 设置 · 头像菜单 → 设置（⌘,），弹窗式

| 画板 | 文件 | 尺寸 |
| --- | --- | --- |
| ④ 设置 · 通用 | [WB-Settings.dc.html](WB-Settings.dc.html) | 1280×800 · 可交互 |
| ④ 设置 · 外观与主题色 | [WB-Settings-Look.dc.html](WB-Settings-Look.dc.html) | 1280×800 · 可交互 |
| ④ 设置 · AI 模型 | [WB-Settings-AI.dc.html](WB-Settings-AI.dc.html) | 1280×800 · 可交互 |
| ④ 设置 · 超级小伴 | [WB-Settings-Assistant.dc.html](WB-Settings-Assistant.dc.html) | 1280×800 · 可交互 |
| ④ 设置 · 编辑供应商（DeepSeek） | [WB-Provider.dc.html](WB-Provider.dc.html) | 1280×800 · 可交互 |
| ④ 设置 · 添加自定义供应商 | [WB-Provider-Custom.dc.html](WB-Provider-Custom.dc.html) | 1280×800 · 可交互 |

### ④ 超级小伴 · 悬浮对话框（小球 → 胶囊 → 卡片 → 实时对话）

| 画板 | 文件 | 尺寸 |
| --- | --- | --- |
| ④ 超级小伴 · 小球状态与菜单 | [WB-Assistant-Ball.dc.html](WB-Assistant-Ball.dc.html) | 1280×800 · 可交互 |
| ④ 超级小伴 · 胶囊快问 | [WB-Assistant-Capsule.dc.html](WB-Assistant-Capsule.dc.html) | 1280×800 · 可交互 |
| ④ 超级小伴 · 卡片多轮对话 | [WB-Assistant-Card.dc.html](WB-Assistant-Card.dc.html) | 2240×800 · 可交互 |
| ④ 超级小伴 · 实时对话 | [WB-Assistant-Voice.dc.html](WB-Assistant-Voice.dc.html) | 1280×800 · 可交互 |

### ④ 资料库 · 我的文档 / 云文档 / 收藏 / 记忆（顶部 Tab）

| 画板 | 文件 | 尺寸 |
| --- | --- | --- |
| ④ 资料库 · 我的文档 | [WB-Library-Docs.dc.html](WB-Library-Docs.dc.html) | 1280×800 · 可交互 |
| ④ 资料库 · 云文档 | [WB-Library-Cloud.dc.html](WB-Library-Cloud.dc.html) | 1280×800 · 可交互 |
| ④ 资料库 · 收藏 | [WB-Library-Saved.dc.html](WB-Library-Saved.dc.html) | 1280×800 · 可交互 |
| ④ 资料库 · 记忆 | [WB-Library-Memory.dc.html](WB-Library-Memory.dc.html) | 1280×800 · 可交互 |

### ④ 会话 · 实时对话（输入框右侧按钮）

| 画板 | 文件 | 尺寸 |
| --- | --- | --- |
| ④ 会话 · 实时对话 | [WB-Chat-Voice.dc.html](WB-Chat-Voice.dc.html) | 1280×800 · 可交互 |

### ⑤ AI 插件 · 接入小伴市场（来源切换 / 审查安装 / 网页唤起 / 专家团 / 更新 / 连接 / 设置）

| 画板 | 文件 | 尺寸 |
| --- | --- | --- |
| ⑤ 插件 · 小伴市场 · 浏览与来源切换 | [Market.dc.html](Market.dc.html) | 1280×800 · 可交互 |
| ⑤ 插件 · 技能安装审查 | [WB-Market-Review.dc.html](WB-Market-Review.dc.html) | 1280×800 · 可交互 |
| ⑤ 插件 · 网页唤起确认（3 种状态） | [WB-Market-DeepLink.dc.html](WB-Market-DeepLink.dc.html) | 1280×800 · 可交互 |
| ⑤ 插件 · 专家团安装 | [WB-Market-Team.dc.html](WB-Market-Team.dc.html) | 1280×800 · 可交互 |
| ⑤ 插件 · 已安装与更新 | [WB-Market-Updates.dc.html](WB-Market-Updates.dc.html) | 1280×800 · 可交互 |
| ⑤ 插件 · 连接器连接 | [WB-Market-Connector.dc.html](WB-Market-Connector.dc.html) | 1280×800 · 可交互 |
| ⑤ 插件 · 插件来源设置 | [WB-Market-Settings.dc.html](WB-Market-Settings.dc.html) | 1280×800 · 可交互 |

### ⑥ 输入框 · 使用技能 / 专家 / 连接器（新建任务 · 会话 · 规范）

| 画板 | 文件 | 尺寸 |
| --- | --- | --- |
| ⑥ 新建任务 · 输入框：技能 / 专家 / 连接器 | [WB-New-Plugins.dc.html](WB-New-Plugins.dc.html) | 1280×800 · 可交互 |
| ⑥ 会话 · 输入框：技能 / 专家 / 连接器 | [WB-Chat-Plugins.dc.html](WB-Chat-Plugins.dc.html) | 1280×800 · 可交互 |
| ⑥ 输入框 · 构成 / 状态 / 选择器规范 | [WB-Compose-Spec.dc.html](WB-Compose-Spec.dc.html) | 1440×2080 |
