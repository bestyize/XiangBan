# 小伴手机 App 设计稿

| 设计稿 | 内容与预览 |
| --- | --- |
| [创造中心](prototypes/create-center.html) | 统一游戏、动画壁纸、小组件、特效、伙伴的手机入口；发现 / 我的作品 / 作品预览三画面，我的作品支持二次确认删除；卡片点击展开底部操作面板：预览 / 游玩、继续修改、发送到电脑、分享到社区及其他渠道；发现页保留创作提示入口，我的作品页保留右上角新建；支持分类、创作输入、360 / 390px 与明暗模式。发现页游戏优先；已接入 Android，桌面装扮走电脑创作，直传暂用系统分享作品包。[在线预览](http://127.0.0.1:1421/prototypes/create-center.html) |
| [游戏中心 A / B](prototypes/game-center.html) | A 轻量游戏库与 B 沉浸游乐场并排比较；390 / 360px、明暗模式、玩法筛选、我的游戏 / 空状态、游戏详情、创作与启动过渡。[在线预览](http://127.0.0.1:1421/prototypes/game-center.html) |
| [社区与小伴 AI 原型](mobile-prototype/README.md) | 现有多画板设计与在线画布说明。 |

游戏中心保留 A / B 对比稿，已选择 A 版并接入 Compose 正式页面；Android 官方游戏与运行能力说明见项目 README。稿内最近游玩和示例作品仍为演示内容；所有操作仅影响预览页面。封面资源位于 `prototypes/game-center-assets/`，复用小伴桌面内置游戏 SVG。

预览：在仓库根运行 `python3 -m http.server 1421 --bind 127.0.0.1 --directory client/xiaoban-multiplatform/docs/design-specs`，访问上方链接。后续迭代沿用同一链接。
