# 小伴桌面官网设计预览（wallpaper.xiangban.xyz）

状态：设计参考（已合入 `services/server-xiaoban/web` 的 `ProductFeatures.vue` 与 `ProductDownload.vue`），静态文件不参与生产构建。全端统一入口见[小伴设计稿总导航](../../../../docs/design-preview/index.html)。

## 功能页 & 下载页 UI 重设计稿（v2.3 · PC + 移动 H5）

[功能页 & 下载页 UI 重设计稿](prototypes/features-download-redesign.html) 采用极简暖白 · 墨黑与「双星相伴 · 日落珊瑚」品牌点缀，覆盖 `/features` 与 `/download` 完整 PC 与移动 H5 体验：

- **功能页 `/features`**：首屏居中主视觉配 8 场景双端实机展台（控电脑 & 多模型、外部 Agent & 语音 MCP、专家市场 & 技能、3D 龙鱼 & 14 特效、8 款实时桌面组件、手机本地造作品、端侧语音 & 控电脑、原生社区 & 自由模型），下方分设电脑端旗舰能力 Bento 与手机端核心能力展示。
- **下载页 `/download`**：当前设备推荐主卡、带 OS 图标与体量的实体下载按钮、更新说明弹窗、一键复制 SHA-256、三步上手指南与加载骨架屏 / 网络异常状态模拟。

通过全端总导航一键预览（`python3 docs/design-preview/serve.py --open`），或单独启动本目录静态服务：

```sh
python3 -m http.server 1452 --bind 127.0.0.1 --directory services/server-xiaoban
```

- [功能页 PC 预览](http://127.0.0.1:1452/docs/design-specs/prototypes/features-download-redesign.html?page=features&device=desktop) · [下载页 PC 预览](http://127.0.0.1:1452/docs/design-specs/prototypes/features-download-redesign.html?page=download&device=desktop) · [功能页 H5 预览](http://127.0.0.1:1452/docs/design-specs/prototypes/features-download-redesign.html?page=features&device=mobile) · [下载页 H5 预览](http://127.0.0.1:1452/docs/design-specs/prototypes/features-download-redesign.html?page=download&device=mobile)

---

## 管理后台全模块 UI 重设计稿（v2.0 规范对齐版 · 6 套主题色 × 明暗）

[小伴官网管理后台重设计稿](prototypes/admin-console-redesign.html) 严格对齐小伴「方案四外壳（左 `--ui-tint` 侧栏 + 右浮起白色圆角内容面板）」「浅底白卡无描边控件」「描述不直接展示、统一放悬停提示」「标准 `xb-menu` / `⋯` 更多菜单」及 6 套官方主题色 × 明暗模式：

- **重构信息架构（IA）**：新增「运营总览」默认首页（待审壁纸队列、AI 目录、图源与代理状态），侧栏按「总览 / 内容与壁纸 / 产品与分发 / 基础设施」语义分组，支持 `⌘K` 快捷跳转。
- **无描边控件与标准浮层**：彻底清除原生 `<select>` 与灰色细描边，全部下拉菜单与卡片/行次要操作统一复用 `.xb-menu` 与 `⋯` 菜单；弹窗 24px 圆角、抽屉 20px 圆角，阴影仅给浮层。
- **高频业务提效**：壁纸图库画廊/列表双视图与底部浮动批量栏；采集工作台一键快采预设条；自定义上架桌面/手机预览；App 四平台矩阵（含 OS 矢量图标与 SHA-256 复制）；官网公告支持右侧 Android 弹窗实时预览；AI 模型弹窗采用紧凑单行模型编辑。

- [运营总览（浅色·小伴红）](http://127.0.0.1:1452/docs/design-specs/prototypes/admin-console-redesign.html?page=overview&theme=light&accent=red) · [壁纸图库（深色）](http://127.0.0.1:1452/docs/design-specs/prototypes/admin-console-redesign.html?page=wallpapers&theme=dark) · [采集工作台](http://127.0.0.1:1452/docs/design-specs/prototypes/admin-console-redesign.html?page=collector) · [官网与公告](http://127.0.0.1:1452/docs/design-specs/prototypes/admin-console-redesign.html?page=desktop) · [登录页](http://127.0.0.1:1452/docs/design-specs/prototypes/admin-console-redesign.html?mode=login)
