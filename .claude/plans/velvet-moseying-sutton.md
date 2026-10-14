# 自定义桌面小组件动态加载架构与 widget_create 工具实现方案

## Context
小伴桌面目前内置了 5 种桌面小组件（求签、日历、天气、便签、倒数日）。这些小组件全部在前端 `WidgetSurface.vue` 中以 `v-if` 硬编码分发，并在前后端做了严格的白名单校验。
为了让桌面小伴 Agent 能够根据用户需求现场创作并实时展示各种各样个性化的小组件（如番茄钟、每日诗词金句、摸鱼倒计时、健康喝水打卡、极简时钟、股票看板等），需要打通“单文件自包含 HTML 小组件”的动态加载机制，并为 Agent 注入 `widget_create` 工具。

## 架构设计

### 1. 存储与分层（参考特效包 `effects/` 设计）
- **小组件定义库（Templates/Packs）**：
  - 存放目录：`~/.xiaobanwallpaper/custom_widgets/<id>.json`
  - 单个 JSON 文件定义一个组件模板，自包含元数据与完整 HTML：
    ```json
    {
      "schemaVersion": 1,
      "id": "pomodoro-timer",
      "name": "番茄工作钟",
      "description": "25分钟专注与5分钟休息交替的时钟",
      "width": 260,
      "height": 220,
      "theme": "glass",
      "html": "<!DOCTYPE html><html><head><style>...</style></head><body>...<script>...</script></body></html>"
    }
    ```
  - 单文件限制：最大 128 KB，UTF-8 编码，原子落盘（`.tmp` -> rename）。
- **桌面摆放实例（Instances）**：
  - 维持现有 `~/.xiaobanwallpaper/config/widgets.json` 的布局管理，新增 `kind: "custom"` 以及 `customId: "pomodoro-timer"`。

### 2. 安全与渲染隔离（沙箱 IFrame）
- 渲染层：在每个独立原生桌面窗口的 `WidgetSurface.vue` 中渲染。
- 当 `widget.kind === 'custom'` 时，采用 `<iframe :srcdoc="html" sandbox="allow-scripts" class="custom-widget-frame">`：
  - `sandbox="allow-scripts"`：允许小组件内的 JavaScript 自行运行（计时器、Canvas、UI 交互、存储 `localStorage` 等）。
  - **无 `allow-top-navigation` / 跨源阻断**：无法访问父页面窗口，无法调用宿主 Tauri IPC，杜绝任意代码执行风险。
  - 外层容器继续保留小伴统一的：
    - 毛玻璃（glass）/ 纸张（paper）/ 透明（transparent）背景主题
    - 鼠标拖拽移动与位置自动记忆
    - 整体等比缩放（scale: 0.75 ~ 1.5）
    - 标题栏与删除操作

---

## 核心修改模块与文件清单

### 1. 后端：自定义小组件模板管理 (`src-tauri/src/custom_widgets.rs`) [新增]
- 实现 `CustomWidgetPack` 结构体及 `validate(&Value)` 结构与安全检查：
  - `id` 校验：小写字母、数字、`-`、`_`，不能与内置 5 个 ID 重名。
  - `name`：最多 24 个字符。
  - `width` / `height`：范围 100~800 像素，默认 280 × 240。
  - `html`：必须包含非空的 HTML 字符串，大小不超过 128 KB。
- 实现 `load_all(directory)`、`find(directory, id)`、`save(directory, pack, overwrite)`、`remove(directory, id)` 等文件操作与原子保存。

### 2. 后端：窗口与布局调度升级 (`src-tauri/src/widgets.rs`)
- 在 `Widget` 结构体中添加可选字段：
  ```rust
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub custom_id: Option<String>,
  ```
- 更新 `validate(&Layout)`：允许 `widget.kind == "custom"`，并校验 `custom_id` 存在且模板有效。
- 更新 `base_size(kind, custom_id)`：对 `custom` 动态查询对应模板的宽高，默认回退到 280×230。
- 更新 `widget_payload()`：将关联的自定义小组件模板信息（`name`, `html`, `width`, `height`）打包进 payload 发给 Webview 窗口。
- 添加 Tauri 命令供前端与测试使用：
  - `list_custom_widgets`
  - `create_custom_widget`
  - `remove_custom_widget`

### 3. Agent 工具注入 (`src-tauri/src/chat/desktop_tools.rs`)
- `Desktop` trait 增加小组件创作方法：
  ```rust
  fn custom_widgets(&self) -> Result<Value, String>;
  fn create_custom_widget(&self, pack: &Value, place_now: bool, overwrite: bool) -> Result<Value, String>;
  ```
- 注册新工具 `widget_create`：
  - 工具描述详细说明单文件 HTML 的编写规范、尺寸建议与内置 CSS 变量。
  - 支持参数：`id`、`name`、`html`、`description`（可选）、`width`（可选）、`height`（可选）、`theme`（可选，默认 glass）、`place_now`（默认 true，创作完成后直接放置到桌面上）、`overwrite`（是否覆盖已有同名组件）。
  - 若 `place_now: true`，成功写入模板后直接调用 `widget_add` 将该组件放置在桌面合理空闲位置。

### 4. 前端：模型与渲染适配 (`src/widgets/`)
- `src/widgets/model.ts`：
  - `WidgetKind` 加入 `'custom'`。
  - `DesktopWidget` 加入 `customId?: string` 和 `custom?: { name: string; html: string; width: number; height: number }`。
  - `baseSize()` 函数支持读取自定义小组件的 `width` / `height`。
  - `validateLayout()` 放行 `kind === 'custom'`。
- `src/widgets/WidgetSurface.vue`：
  - 添加 `<template v-else-if="widget.kind === 'custom'">` 分支。
  - 使用 `<iframe :srcdoc="customHtml" sandbox="allow-scripts" class="custom-widget-frame" />`。
  - 保证拖拽把手与外框无缝配合。

---

## 验证方案
1. **单元测试验证**：
   - 为 `custom_widgets.rs` 编写 Rust 单元测试，覆盖模板结构验证、合法/非法 ID、超大 HTML 拦截、原子写入与删除。
   - 为 `src/widgets/model.test.ts` 添加 `kind: 'custom'` 验证用例，确保布局校验与解析稳定。
   - 运行前端单元测试：`npm test`（或 vitest）。
2. **Agent 工具测试**：
   - 编写 `desktop_tools.rs` 中针对 `widget_create` 的单元测试，验证模板落盘与 `place_now` 联动逻辑。
3. **功能端到端验证**：
   - 启动桌面端，通过 Agent 或调试命令调用 `widget_create` 创建一个真实的“极简时钟”小组件并放置到桌面，验证透明背景、样式渲染与 JS 运行。
