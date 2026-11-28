# 小伴手机 App 与协议 2 小程序设计稿

全端统一浏览入口见[小伴设计稿总导航](../../../../docs/design-preview/index.html)（仓库根运行 `python3 docs/design-preview/serve.py --open`）。

## 手机 App 核心页面

| 设计稿 | 状态 | 内容与预览 |
| --- | --- | --- |
| [小伴手机版 · 独立移动端下载落地页](prototypes/mobile-download-h5.html) | 已合入 | 面向扫码、社交分享与 H5 生态引流的独立下载落地页 6 画板交互原型，已合入小伴官网 `/app`（别名 `/mobile`，`services/server-xiaoban/web/src/views/MobileAppDownload.vue`）：覆盖 Android 默认转化页、微信/QQ 拦截引导遮罩、iOS 网页生态导流态、下载后 20px 安装指引与 SHA-256 抽屉及 PC 宽屏真实二维码扫码直下态。[本机预览](http://127.0.0.1:1455/client/xiaoban-multiplatform/docs/design-specs/prototypes/mobile-download-h5.html) |
| [应用广场视觉重设计](prototypes/app-plaza-redesign.html) | 已合入 | 零额外封面三方案对比（定稿方案 A · 精装榜单流，已合入手机与网页）：游戏与工具仅复用 1:1 图标，社区沿用 3:2 封面，最近用过收口至顶栏「最近」+ 下拉抽屉，支持浅/深色、390/360px 与六套主题色。[本机预览](http://127.0.0.1:1421/prototypes/app-plaza-redesign.html) |
| [实时语音通话页](prototypes/voice-call-redesign.html) | 已合入 | 同步最新「双星相伴 · 日落珊瑚」AppIcon，88dp 同心呼吸光球（回答态上收至 64dp 释放 224dp 渐隐歌词字幕区），思考/聆听/回答三态与 390/360px、六套主题色。[本机预览](http://127.0.0.1:1421/prototypes/voice-call-redesign.html) |
| [小伴 AI 聊天窗口视觉优化](prototypes/ai-chat-ui.html) | 已合入 | 通栏流式阅读排版与无底星标；思考/任务完成/执行过程三合一摘要卡；统一 16px 白底细框卡片语法覆盖作品交付、提醒、生图、代码块、文件变更撤销及贴底审批/提问/方案确认与续作卡，四屏覆盖全场景。[本机预览](http://127.0.0.1:1421/prototypes/ai-chat-ui.html) |
| [小伴 AI 首页](prototypes/ai-home.html) | 已合入 | 个人 Agent 连续主线 + 并行任务三屏定稿（默认浏览态、键盘弹出态、个人 Agent 配置页），极简三段顶栏与分离悬浮胶囊输入框。[本机预览](http://127.0.0.1:1421/prototypes/ai-home.html) |
| [「我的」页与个人主页](prototypes/mine-redesign.html) | 已合入 | 手机「我的」页定稿（方案 1 通栏精修）与个人主页（本人视角 + 他人视角）三屏联动：三宫格（收藏/作品/通知）与两组线性列表、68px 头像与四列数据条、他人屏蔽收进右上角菜单。[本机预览](http://127.0.0.1:1421/prototypes/mine-redesign.html) |
| [我的作品 / 通知](prototypes/mine-subpages-redesign.html) | 已合入 | 我的作品统一收口「本机草稿 + 社区已发布」（草稿虚线轻描边、全状态删除二次确认区分下架与永久删除）；通知将「全部已读」收进顶栏右上角并补齐头像与未读红点。[本机预览](http://127.0.0.1:1421/prototypes/mine-subpages-redesign.html) |
| [模型配置全链路](prototypes/model-config-redesign.html) | 已合入 | 模型配置主页、小伴 AI 官方配置、添加第三方服务、第三方服务商配置及对话模型+思考等级五屏统一重设计，严格控色与贴底思考等级 BottomSheet。[本机预览](http://127.0.0.1:1421/prototypes/model-config-redesign.html) |
| [详情页分享面板](prototypes/detail-share-sheet.html) | 已合入 | 全站详情页统一分享面板：单行预览条 + 单行可横滑图标轨（主态末尾删除单次内联确认）+ 沉浸式磨砂分享卡片舞台（极简白 / 暖纸笺 / 墨夜）。[本机预览](http://127.0.0.1:1421/prototypes/detail-share-sheet.html) |
| [分享回流弹窗](prototypes/share-link-landing.html) | 已合入 | 外部分享链接拉起 App 后的居中预览确认卡，覆盖有封面/无封面横排卡、骨架屏加载、电脑端内容琥珀提示及已下架/网络失败空态。[本机预览](http://127.0.0.1:1421/prototypes/share-link-landing.html) |
| [基础组件库与通用弹层规范（手机 + 社区网页）](prototypes/ui-kit-overlays.html) | 已合入 | 手机 `:common:ui` 与社区前台 `src/ui/` 分层规范：24px 居中卡、20px 顶圆角 BottomSheet、ActionSheet、Toast、通用控件与模块边界（业务特例保留在业务层），支持深浅色、六套主题色与 390/360px 预览。[本机预览](http://127.0.0.1:1421/prototypes/ui-kit-overlays.html) |
| [统一弹层](prototypes/mobile-dialogs.html) | 已合入 | 八个手机高频弹窗场景（评论删除、附件购买、求助关闭、作品投稿、屏蔽、举报、工具环境移除与 App 更新），统一 24px 圆角居中卡片。[八场景总览](prototypes/mobile-dialog-cards-preview.png) · [本机预览](http://127.0.0.1:1421/prototypes/mobile-dialogs.html) |
| [系统通知栏与通知设置](prototypes/mobile-notifications.html) | 已合入 | Android 14 / iOS 18 系统通知栏与锁屏聚合、顶部横幅与隐私隐藏态、24px 权限引导弹层、通栏「设置 → 通知与提醒」页、AI 任务执行进度/灵动岛实时活动、方案确认/高危审批/快捷回复/交付大图与中断续作、24dp 单色双星图标及 5 通知渠道规范。[本机预览](http://127.0.0.1:1421/prototypes/mobile-notifications.html) |
| [评论组件](prototypes/comment-composer.html) | 已合入 | 评论列表、吸底入口与键盘/表情/相册三态面板，支持单字/单表情发布、回复、展开编辑与图片预览。[本机预览](http://127.0.0.1:1421/prototypes/comment-composer.html) |
| [发帖页（手机 + 网页双端）](prototypes/post-composer.html) | 已合入 | 写作优先无框结构：手机端三态 + 网页 PC 820px 级联弹窗与 H5 16px 抽屉，每帖限 1 个可运行作品（V2 ZIP / `.html`·`.htm` 自动转 V2 / 关联创作）。[手机预览](http://127.0.0.1:1421/prototypes/post-composer.html) · [网页预览](http://127.0.0.1:1421/prototypes/post-composer.html?view=web) |
| [更换头像（手机 + 网页双端）](prototypes/avatar-editor.html) | 已合入 | 手机 App 与网页窄屏共用底部选图弹层 + 390 方形裁剪页；网页 PC 820px 居中弹窗并排预览四种尺寸，支持保存失败重试模拟。[本机预览](http://127.0.0.1:1421/prototypes/avatar-editor.html) |

## 小游戏与小程序（协议 2）

| 设计稿 | 状态 | 内容与预览 |
| --- | --- | --- |
| [小游戏与小程序 暂停 / 存档 / 翻页退出](prototypes/game-pause-save-exit.html) | 已合入 | 覆盖无大厅直开《方块爆破》、有大厅多层《水墨剑客》与多页翻页小程序：32px 自适应单入口融合胶囊、残局自动恢复、侧滑逐层回退与「重新加载」语义规范。[本机预览](http://127.0.0.1:1421/prototypes/game-pause-save-exit.html) |
| [MBTI · 向内看](prototypes/mbti-native.html) | 已合入 | 原生 iOS 质感第四版（珊瑚橙红 + 暖白 + 墨灰）：首页、6 题缩略流程、六章报告、16 型角色与分享卡，已合入内置 MBTI 1.3.0。[本机预览](http://127.0.0.1:1421/prototypes/mbti-native.html) |
| [人格贩卖机 · 趣味与人格测评 H5](prototypes/personality-tests-h5.html) | 设计评审中 | 移动端 H5 一期纯免费拉新设计稿（6 画板总览 + 单机试玩 + 390/360px + 深浅色）：发现首页、单手自动切题答题流、6 大专属视觉皮肤结果页（真实面具 SBTI 风 / 双色主导色 / 动物塑 / 避雷说明书 / 精神电量体检单 / 16 型速测）、零门槛双人合盘、系统分享与 9:16 / 3:4 高清 PNG 海报导出弹层及多维灵魂档案。[本机预览](http://127.0.0.1:1455/client/xiaoban-multiplatform/docs/design-specs/prototypes/personality-tests-h5.html) |
| [小伴游戏厅 · game.xiaban.xyz 移动端游戏站](prototypes/xiaoban-games-h5.html) | 设计评审中 | 香港纯静态移动端游戏合集站设计稿（8+1 画板总览 + 单机试玩 + 真机全屏 + 390/360px + 暖瓷浅色/掌机深色 + 6 套官方主题色）：精装掌机展厅风，收录小伴现役 8+1 款官方游戏，覆盖「继续上次」断点续玩、今日主打大片轮播、单手竖屏筛选与随机摇号、卡带预览抽屉、竖屏沉浸运行与透明三点宿主菜单、横屏旋转引导与右侧窄抽屉、免登录战绩档案、通栏精修「我的」页、统一相伴账号 4 态登录注册弹窗及分享直达页。[本机预览](http://127.0.0.1:1455/client/xiaoban-multiplatform/docs/design-specs/prototypes/xiaoban-games-h5.html) |
| [轻启](prototypes/quick-launch.html) | 已合入 | Android 桌面快捷方式设计稿，256×256 WebP 图标网格、搜索筛选与添加到桌面演示，已合入内置 `quick-launch` 包。[本机预览](http://127.0.0.1:1421/prototypes/quick-launch.html) |
| [遥控器](prototypes/universal-remote.html) | 已合入 | 协议 2 电视与空调红外工具：按市场份额排序的 19 电视 / 21 空调品牌试码、四点方向盘、大字温控与无圆底竖三点宿主菜单，已合入内置 `remote-control` 包。[本机预览](http://127.0.0.1:1421/prototypes/universal-remote.html) |
| [动态壁纸 · 雨窗天光与画廊重设计](prototypes/live-wallpapers-redesign.html) | 设计评审中 | 协议 2 动态壁纸小程序重设计（6 画板总览 + 单机试玩 + 全屏壁纸 + 390/360px + 深浅色 + 6 套主题色）：画廊首页与「雨窗天光」WebGL 实时雨滴折射/触控拭雾壁纸，含暮雨/晴岚/梅雨/霓虹夜/微雪 5 套天光氛围、桌面/锁屏模拟预览与双槽位切换。[本机预览](http://127.0.0.1:1455/client/xiaoban-multiplatform/docs/design-specs/prototypes/live-wallpapers-redesign.html) |
| [万磁搜](prototypes/torrent-search.html) | 已合入（V2，`wancisou` v2.0.0） | V2 改版已完整合入独立应用包：暖中性纸面与顶部绯红柔光弧线、深绯墨 Hero 卡与 4 格场景入口、可点击 11 站点矩阵、两色语义结果卡、绯红浅底一键直复制、双段式 BTIH 磁力与带定位副标来源 BottomSheet、新版 16:9 封面预览（独立应用包 `assets/app-packs/wancisou/` v2.0.0）。[本机预览](http://127.0.0.1:1421/prototypes/torrent-search.html) · [线上应用](https://res.xiangban.xyz/builtin-games/wancisou/index.html) |
| [收盘别问](prototypes/shoupan-biewen.html) | 概念试玩 | 390×800 竖屏 H5 单页，100 个真实创业板交易日、可横滑真实日 K、提前收手结算与带二维码 PNG 海报（尚未合入内置应用包）。[本机预览](http://127.0.0.1:1421/prototypes/shoupan-biewen.html) |

单独预览本目录：在仓库根运行 `python3 -m http.server 1421 --bind 127.0.0.1 --directory client/xiaoban-multiplatform/docs/design-specs`，或直接通过总导航 `python3 docs/design-preview/serve.py --open` 访问。
