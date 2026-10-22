# 桌面控制台 UI 设计规范

> 实际桌面上的悬浮小组件采用独立的 [Soft Living UI 桌面小组件设计规范](desktop-widgets-design.md)；这里的视觉令牌主要面向应用控制台，不应直接套用到悬浮小组件。

> 设计文档总入口见 [设计规范目录](design-specs/README.md)。超级小伴悬浮输入框、横排工具栏和截图蒙层采用独立的 [超级小伴设计规范与 HTML 交互稿](design-specs/super-assistant.md)，调整这些界面时先查看对应设计稿。

## 技术结构与入口

控制台使用 Vue 3.5.43、Naive UI 2.45.3、Vite 8 和 Lucide Vue Next。表单、按钮、提示、标签和加载状态优先使用 Naive UI；Lucide 图标用于辅助识别，不能取代文字标签。

`src/main.ts` 在需要控制台时动态载入 `src/control.ts`；`control.ts` 只负责 `mountControl()`，创建 Vue 应用并挂载 `src/ui/App.vue`。页面由 `src/ui/ControlShell.vue` 按当前分区加载，`src/ui/pages/` 保存分区页面，`src/ui/components/` 保存可复用展示组件，`src/ui/wallpaper/` 保存壁纸页面状态逻辑，`src/ui/control-store.ts` 管理控制台状态，`src/ui/control.css` 和 `src/ui/theme.ts` 定义控制台外观。`src/style.css` 仅服务宠物、游戏和关怀气泡等原生场景宿主；游戏和气泡使用协调的蓝白浅灰色彩。

控制台仅随控制台入口加载，不进入宠物窗口与 Canvas 游戏沙箱。宠物渲染由独立角色代码负责，游戏内容在隔离的 iframe 沙箱中运行；控制台页面组件不会作为它们的依赖。

## 导航与分区

控制台使用六个清晰分区：壁纸库、桌面特效、桌面宠物、游戏空间、关怀提醒，以及底部的设置。主内容区域一次展示一个分区；设置入口固定在侧栏底部，避免与日常内容混淆。侧栏的导航文案使用用户任务语言，图标只做辅助识别。

| 分区 | 主要职责 | 关键动作 |
| --- | --- | --- |
| 壁纸库 | 浏览、预览、保存和应用真实壁纸 | 选择内容并预览，明确执行应用 |
| 桌面特效 | 为整个桌面叠加雨、雪、萤火虫、花瓣、树叶或流星，与壁纸无关 | 点选场景立即生效，调整强度和速度；选“无特效”关闭 |
| 桌面宠物 | 调整小伴可见状态和桌面表现 | 即时切换并反馈保存结果 |
| 游戏空间 | 展示本地游戏资料并直接开玩 | 点击游戏卡片打开只运行这款游戏的普通窗口（默认不全屏），退出即回到原来的界面 |
| 关怀提醒 | 管理提醒内容与展示时间 | 编辑后保存，再展示生效状态 |
| 设置 | 管理应用级偏好与维护项 | 保存偏好、查看应用状态 |

## 交互约定

- 即时设置在用户操作后立刻生效，并在失败时显示原因、恢复已保存值。开关旁始终提供清晰标题、状态或简短说明；平台不支持时展示“当前平台不可用”，禁用控件并说明可用的显式入口。
- 预览与保存分开。壁纸预览应标出预览状态，只有用户执行应用后才改变桌面内容；需要持久保存的编辑使用明确的保存动作，并显示结果。
- 游戏卡片只呈现包内 manifest 提供的名称、说明与封面。卡片不伪装成单款游戏入口；点击卡片直接开始这一款游戏，不经过中间的游戏列表。当前只有水墨剑客提供本机进度存档，界面只说明这项真实能力，不展示虚构在线状态、玩家人数或统计。
- 浏览器预览可以展示游戏资料，但禁用依赖原生桌面能力的游戏启动。
- 网络或本地资源失败时，在对应内容附近显示轻量错误与可理解的恢复方式；不要用技术实现细节替代用户说明。

## 视觉规范

- 色彩以中性灰白为底，配合用户可选的 6 套主题色（默认小伴红）。具体令牌、深色模式与工具类见文末「方案四令牌与工具类」；Naive UI 主题在 `src/ui/theme.ts` 中随主题色与深浅色生成，控制台 CSS 变量由 `src/ui/design-tokens.ts` 写入。错误和警告使用 Naive UI 语义色。
- 间距基于 8 px：常用 8、16、24 px；卡片内边距通常 16–24 px，区块间距 16–24 px。卡片圆角约 12 px，阴影轻，避免厚重边框。
- 主操作使用 Naive UI primary 按钮；开关、标签、提示和骨架屏采用 Naive UI 对应组件。Lucide 图标用于导航和操作提示，不用图标替代文字标签。
- 桌面卡片网格在宽屏采用三列，中等宽度两列，窄屏一列。封面使用资源自身比例与裁切，不以占位图冒充真实封面。
- 页面遵循通用 `.ui-page`、`.page-header`、`.page-subtitle`、`.surface`、`.section-heading`、`.setting-row`、`.muted` 结构类；组件特有样式限制在局部作用域。

## 验证状态

2026-10-10 UI 重构后，前端全量测试 51 tests / 11 files、Rust 55 tests、Windows x64 桌面生产构建均已通过。桌面宠物页和壁纸库视觉参考：companion-blue.jpg（本地：`../artifacts/ui/companion-blue.jpg`）、wallpapers-blue.jpg（本地：`../artifacts/ui/wallpapers-blue.jpg`）。

## 方案四令牌与工具类

2026-10 起控制台采用「方案四」外壳：左侧 264 px 侧栏（底色 `--ui-tint`），右侧白色内容面板浮在侧栏底色上（`margin:8px 8px 8px 0`、圆角 12、细描边 + 轻阴影）。设置改为居中弹窗（940×660），由账号菜单或 `store.openSettings(tab)` 打开。

**主题色**：`src/ui/appearance.ts` 导出 `accents`（`red` 小伴红 / `green` 薄荷绿 / `blue` 海湾蓝 / `purple` 暮紫 / `amber` 琥珀 / `graphite` 石墨，各含 `light`、`dark` 两套色值）、`accent`（当前 id，localStorage `xiaoban.control.accent` 持久化）与 `setAccent(id)`。每套主题色有 6 个角色，映射为 CSS 变量：

| 变量 | 用途 |
| --- | --- |
| `--ui-btn` / `--ui-btn-hi` | 主按钮底色 / 渐变高光 |
| `--ui-soft` / `--ui-ink` | 选中浅底 / 选中与链接文字 |
| `--ui-tint` | 侧栏与窗口底色（= `--ui-app-bg`） |
| `--ui-dot` | 待确认、未读等标记点 |

**中性色**：`--ui-text-primary/secondary/tertiary/disabled`、`--ui-surface`（内容面板）、`--ui-surface-alt`/`--ui-control`（灰底控件）、`--ui-surface-sunken`（弹窗内容底）、`--ui-surface-hover`、`--ui-surface-raised`（浮层）、`--ui-border`、`--ui-border-strong`、`--ui-line`（分隔线）、`--ui-success/warning/danger`，阴影 `--ui-shadow-panel/card/menu/dialog/btn/btn-secondary`，字体 `--ui-font-family`、`--ui-font-mono`。深色模式下面板为 #1C1C1E → #232325 → #2C2C2E，主题色整体提亮。旧变量继续可用：`--ui-primary*` 指向主题色，`--ui-sidebar-bg` 指向 tint，`--ui-accent` 仍是成功绿。组件里只用变量，不要写死主题色。

**全局工具类**（`src/ui/control.css`）：

- 按钮：`.xb-btn-primary`（渐变主按钮）、`.xb-btn`（白底细描边）、`.is-sm`/`.is-lg` 尺寸，`.xb-btn-ghost`、`.xb-icon-btn`（30×30，`.is-sm` 24×24）、`.xb-link`；危险操作加 `.is-danger`（`.xb-btn.is-danger` 红字次按钮）。
- 容器：`.xb-page`（内容区内边距 22px 28px）、`.xb-page-head` + `.xb-page-title` + `.xb-page-actions`、`.xb-card`（`.is-hover`、`.is-selected`）、`.xb-empty`（+ `.xb-empty-icon`）、`.xb-scroll`。
- 标签与分段：`.xb-tabs-lg`（页面顶部大标签 20px/600，未选 #A1A1A6，选中项加 `aria-selected="true"`）、`.xb-seg`（分段控件，选中按钮加 `.is-active` 或 aria-pressed/selected/checked）。
- 浮层菜单：`.xb-menu`、`.xb-menu-item`（`.is-active`/`aria-checked` 为 soft 底，`.is-danger` 红字）、`.xb-menu-hint`、`.xb-menu-check`、`.xb-menu-sep`、`.xb-menu-label`。
- 设置分组：`.xb-group-title`、`.xb-group`、`.xb-row`、`.xb-row-text`（内含 `<b>` 标题与 `<span>` 说明）。
- 小件：`.xb-dot`、`.xb-badge`、`.xb-chip`、`.xb-ok`、`.xb-kbd`、`.xb-mark`（渐变品牌方块）、`.xb-search`。

**路由**：`ControlPage` 为 `new-task`、`chat`、`ai-store`、`library`、`studio`、`games`、`care`（以及 `interactive`）。旧 id 仍可传给 `navigate`：`wallpapers`/`effects`/`widgets`/`companion` 打开装扮对应标签（`store.studioTab`：`wallpapers`/`effects`/`widgets`/`pets`），`settings`/`assistant` 打开设置弹窗。装扮内页可用 `<Teleport defer to="#studio-actions">` 把按钮放到标签行右侧。

**会话管理**：侧栏空间树的会话行悬停出现 ⋯（右键同一菜单）：重命名、移动到工作区（`chat.moveToWorkspace`，可选已有工作区或另选文件夹）、删除（菜单内二次确认）。工作区取代了旧的“分组”：不再有分组界面，宿主里已有的分组数据原样保留、不显示。页面统一自带 `.xb-page` 内边距，`.control-workspace` 本身不加内边距。

