# 小伴桌面官网设计预览（wallpaper.xiangban.xyz）

状态：设计参考，静态文件用于讨论外观与交互，不参与生产构建。确认后再落地到 `services/server-xiaoban/web`。

## 功能页 & 下载页 UI 重设计稿（PC + 移动 H5）

[功能页 & 下载页 UI 重设计稿](prototypes/features-download-redesign.html) 去除原浅绿色系，默认采用克制耐看的**极简暖白 · 墨黑**视觉体系（`#fafaf9` 暖白底 + `#18181b` 墨黑主色 + 柔灰展台与深曜石黑推荐卡），并在评审栏支持一键切换**暖纸 · 珊瑚红**（呼应小伴品牌图标）与**晴白 · 冰川蓝**两套备选主题，重点解决现行 `/features` 与 `/download` 页面的视觉与层级问题：

- **功能页 `/features`**：升级全新「双星相伴 · 日落珊瑚」品牌图标；首屏改用「左文案 + 右六大能力速览 Bento」并配吸顶全 6 模块导航；`01 AI 工作助手` 支持切换最新 v3.1.8 新建任务台与任务拆解执行视图；`03 个性桌面` 升级为可点按切换的「动态壁纸 / 14 种桌面特效 / 桌面伙伴」最新 v3.1.8 实机画廊；`05 精选应用` 换用最新 v3.1.8 11 款应用广场截图；新增 `06 手机版小伴` 三联手机实机展示（连续对话主线 / 双星实时语音通话 / 作品交付卡）。
- **下载页 `/download`**：合并重复的双重大标题，新增高质感深色「当前设备推荐主卡」（桌面端展示最新 v3.1.8 控制台实机界面，切换 Android 时自动呈现最新手机双屏实机样机），用带 OS 图标与体量信息的实体下载按钮取代裸文字链接；提供精致更新说明弹窗、一键复制 SHA-256、柔灰公告横幅、三列卡片化使用指南与系统要求速览条，并覆盖加载骨架屏与网络异常重试状态。
- **新旧痛点对照**：支持一键切换「现行版 vs 新版对照」和「PC 桌面 / 移动 H5 (390px)」双端预览，全部最新实机图存放于 `prototypes/assets/`。

从仓库根目录启动本机预览：

```sh
python3 -m http.server 1452 --bind 127.0.0.1 --directory services/server-xiaoban
```

- [功能页 PC 预览](http://127.0.0.1:1452/docs/design-specs/prototypes/features-download-redesign.html?page=features&device=desktop) · [下载页 PC 预览](http://127.0.0.1:1452/docs/design-specs/prototypes/features-download-redesign.html?page=download&device=desktop) · [移动 H5 预览](http://127.0.0.1:1452/docs/design-specs/prototypes/features-download-redesign.html?page=download&device=mobile) · [新旧痛点对照](http://127.0.0.1:1452/docs/design-specs/prototypes/features-download-redesign.html?page=compare)
