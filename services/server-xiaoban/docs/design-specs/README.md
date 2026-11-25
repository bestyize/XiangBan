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
