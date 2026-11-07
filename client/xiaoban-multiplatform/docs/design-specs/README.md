# 小伴手机 App 设计稿

| 设计稿 | 内容与预览 |
| --- | --- |
| [开源精选详情页](prototypes/open-source-detail.html) | 独立图文详情原型：顶部项目卡片（含 Star 数与更新日期）、默认中文 README / English 语言切换、上游英文正文与中文译文、官方配图、目录及安装命令复制、图片放大、收藏、点赞、小红书风格评论与回复、贴边互动栏、分享、深浅色与手机 / 桌面预览。正文使用 Excalidraw 采编样稿；仅原型交互，未接生产接口。[在线预览（本机）](http://127.0.0.1:1423/prototypes/open-source-detail.html) |
| [应用广场](prototypes/app-plaza.html) | 底栏「创造」改为「应用」：广场（最近用过、每期小伴选、官方游戏、工具与测评、创作者精选、社区作品瀑布流）与我的作品；社区作品就是普通帖子（帖子页带评论、点赞、收藏、做一个我的版本），官方应用用底部弹层；启动过渡、竖屏 / 横屏全屏运行，右上角「··· \| ✕」宿主胶囊，返回一次展开菜单、再次退出，可显示安全区；小伴AI 首页下拉打开最近用过和收藏、问候语推荐、对话里递上应用卡片。配色、底栏、字号按 `docs/product/手机端设计规范.md`；仅原型交互，未接接口。预览：`python3 -m http.server 1424 --bind 127.0.0.1 --directory client/xiaoban-multiplatform/docs/design-specs`，打开 [本机链接](http://127.0.0.1:1424/prototypes/app-plaza.html)。 |
| [创造中心](prototypes/create-center.html) | 统一游戏、动画壁纸、小组件、特效、伙伴的手机入口；发现 / 我的作品 / 作品预览三画面，我的作品支持二次确认删除；卡片点击展开底部操作面板：预览 / 游玩、继续修改、发送到电脑、分享到社区及其他渠道；发现页保留创作提示入口，我的作品页保留右上角新建；支持分类、创作输入、360 / 390px 与明暗模式。发现页游戏优先；已接入 Android，桌面装扮走电脑创作，直传暂用系统分享作品包。[在线预览](http://127.0.0.1:1421/prototypes/create-center.html) |
| [游戏中心 A / B](prototypes/game-center.html) | A 轻量游戏库与 B 沉浸游乐场并排比较；390 / 360px、明暗模式、玩法筛选、我的游戏 / 空状态、游戏详情、创作与启动过渡。[在线预览](http://127.0.0.1:1421/prototypes/game-center.html) |
| [社区与小伴 AI 原型](mobile-prototype/README.md) | 现有多画板设计与在线画布说明。 |

游戏中心保留 A / B 对比稿，已选择 A 版并接入 Compose 正式页面；Android 官方游戏与运行能力说明见项目 README。稿内最近游玩和示例作品仍为演示内容；所有操作仅影响预览页面。封面资源位于 `prototypes/game-center-assets/`，复用小伴桌面内置游戏 SVG。

预览：在仓库根运行 `python3 -m http.server 1421 --bind 127.0.0.1 --directory client/xiaoban-multiplatform/docs/design-specs`，访问上方链接。后续迭代沿用同一链接。

开源精选详情页预览：在仓库根运行 `python3 -m http.server 1423 --bind 127.0.0.1 --directory client/xiaoban-multiplatform/docs/design-specs`。素材见 `prototypes/open-source-detail-assets/`，后续迭代沿用同一预览地址。
