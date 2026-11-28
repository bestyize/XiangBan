# 设计资源与预览

状态：设计参考，静态文件用于讨论外观交互，不是客户端运行入口，不参与生产 Docker 构建。

实现设计时修改对应 client / services 工程并验证；不要直接用预览覆盖已有应用、鉴权或平台接入逻辑。

## 总体设计稿导航网页（推荐入口）

[小伴全端设计稿总导航](index.html) 统一收录 **手机版、网页版、桌面版、小游戏与小程序** 四大分类共 45 个设计稿模块/状态入口（38 份独立原型，其中社区详情页按作品/帖子/开源三态拆出、桌面 UI 升级画布按 5 大模块拆出直达；已剔除被替代旧稿、冗余静态 PDF/ZIP 归档及已淘汰老样式对照），支持关键词检索、模块与落地状态筛选、快捷预设跳转、深浅色切换与内置工作台即时预览。

在仓库根目录一键启动统一预览服务（自动映射桌面 UI 升级画布的 `/_blob/` 图片资源）：

```sh
python3 docs/design-preview/serve.py --open
```

访问 [http://127.0.0.1:1455/](http://127.0.0.1:1455/) 即可浏览全部设计稿。若需预览 4 份依赖桌面真实 Vue/TS 组件的 Vite 设计稿（`ai-model-settings.html`、`decor-library.html`、`game-library.html`、`effects-library.html`），另开终端运行：

```sh
npm --prefix client/xiaoban-desktop run dev
```

---

## 一、小伴手机 App（手机版）

完整说明见[手机设计索引](../../client/xiaoban-multiplatform/docs/design-specs/README.md)。

| 设计稿 | 模块 | 状态 | 内容 |
| --- | --- | --- | --- |
| [小伴手机版 · 独立移动端下载落地页](../../client/xiaoban-multiplatform/docs/design-specs/prototypes/mobile-download-h5.html) | 分发与拉新 | 已合入 | 面向扫码/社交分享/H5 引流的独立移动端下载落地页 6 画板交互原型，已合入官网 `/app`（别名 `/mobile`、`MobileAppDownload.vue`）：首屏 3 秒转化与 4 场景实机切换、端侧创造工具/游戏与连续对话生图、墨黑官方游乐场、实用百宝箱 2×2 实装图标矩阵、原生社区与我的作品空间、下载后安装指引抽屉、微信拦截遮罩、iOS 网页生态导流及 PC 真实二维码扫码直下（[本地预览](http://127.0.0.1:1455/client/xiaoban-multiplatform/docs/design-specs/prototypes/mobile-download-h5.html)） |
| [应用广场视觉重设计](../../client/xiaoban-multiplatform/docs/design-specs/prototypes/app-plaza-redesign.html) | 应用与广场 | 已合入 | 零额外封面三方案对比（定稿方案 A · 精装榜单流，已合入手机与网页）：1:1 图标三行横滑游戏卡 × 2×2 工具矩阵 × 净化 3:2 社区双列卡 |
| [实时语音通话页](../../client/xiaoban-multiplatform/docs/design-specs/prototypes/voice-call-redesign.html) | 核心 AI 与对话 | 已合入 | 同步「双星相伴 · 日落珊瑚」AppIcon，88dp 同心呼吸光球（回答态上收至 64dp 释放 224dp 渐隐歌词区），思考/聆听/回答三态与六套主题色 |
| [小伴 AI 聊天窗口与全卡片体系](../../client/xiaoban-multiplatform/docs/design-specs/prototypes/ai-chat-ui.html) | 核心 AI 与对话 | 已合入 | 通栏流式排版、无底星标、思考/完成/过程三合一摘要卡，统一 16px 白底细框卡片语法覆盖交付/提醒/生图/代码块/文件变更与贴底审批/提问/方案确认卡 |
| [小伴 AI 首页](../../client/xiaoban-multiplatform/docs/design-specs/prototypes/ai-home.html) | 核心 AI 与对话 | 已合入 | 个人 Agent 连续主线 + 并行任务三屏定稿（默认态、键盘态、个人 Agent 配置页），极简顶栏与分离悬浮胶囊输入框 |
| [「我的」页与个人主页](../../client/xiaoban-multiplatform/docs/design-specs/prototypes/mine-redesign.html) | 个人与账号 | 已合入 | 「我的」页方案 1 通栏精修与个人主页（本人 / 他人视角）三屏联动，三宫格快捷入口与四列数据条 |
| [我的作品 / 通知](../../client/xiaoban-multiplatform/docs/design-specs/prototypes/mine-subpages-redesign.html) | 个人与账号 | 已合入 | 统一收口本机草稿与社区已发布（草稿虚线轻描边、全状态删除二次确认区分下架与永久删除）；通知顶栏右上角收口「全部已读」 |
| [模型配置全链路](../../client/xiaoban-multiplatform/docs/design-specs/prototypes/model-config-redesign.html) | 设置与模型 | 已合入 | 主页、小伴 AI 官方配置、添加第三方服务、第三方服务商配置与对话模型+思考等级五屏重设计，贴底思考等级 BottomSheet |
| [详情页分享面板](../../client/xiaoban-multiplatform/docs/design-specs/prototypes/detail-share-sheet.html) | 社区与互动 | 已合入 | 全站详情页统一分享面板：单行预览条 + 可横滑图标轨（主态末尾删除单次内联确认）+ 沉浸式磨砂分享卡片舞台 |
| [分享回流弹窗](../../client/xiaoban-multiplatform/docs/design-specs/prototypes/share-link-landing.html) | 社区与互动 | 已合入 | 外部分享链接拉起 App 后的居中预览确认卡，覆盖有封面/无封面、骨架屏加载、电脑端内容琥珀提示及已下架/网络失败态 |
| [基础组件库与通用弹层规范（手机 + 社区网页）](../../client/xiaoban-multiplatform/docs/design-specs/prototypes/ui-kit-overlays.html) | 系统与弹层 | 已合入 | 手机 `:common:ui` 与社区前台 `src/ui/` 分层组件规范：弹层（24px 居中卡、20px 顶圆角 BottomSheet、ActionSheet、Toast、PC↔H5 响应式对照）与通用控件（按钮/输入/开关/标签/通栏列表/空态）分开建设，支持浅深色、6 套主题色与 390/360px 预览 |
| [统一弹层](../../client/xiaoban-multiplatform/docs/design-specs/prototypes/mobile-dialogs.html) | 系统与弹层 | 已合入 | 八个手机高频场景统一 24px 圆角居中卡片弹层（评论删除、附件购买、求助关闭、作品投稿、屏蔽、举报、工具环境移除、App 更新） |
| [系统通知栏与通知设置](../../client/xiaoban-multiplatform/docs/design-specs/prototypes/mobile-notifications.html) | 系统与弹层 | 已合入 | Android 14 / iOS 18 系统通知栏与锁屏聚合、顶部横幅与锁屏隐私态、24px 权限引导弹层、通栏「设置 → 通知与提醒」页、AI 任务进度/灵动岛实时活动、审批/快捷回复/交付与异常通知及 5 通知渠道规划 |
| [评论组件](../../client/xiaoban-multiplatform/docs/design-specs/prototypes/comment-composer.html) | 社区与互动 | 已合入 | 评论列表、吸底入口与键盘/表情/相册三态输入面板，支持单字/单表情发布、回复与图片预览 |
| [发帖页（手机 + 网页双端）](../../client/xiaoban-multiplatform/docs/design-specs/prototypes/post-composer.html) | 社区与互动 | 已合入 | 写作优先无框结构：手机端三态 + 网页 PC 820px 级联弹窗与 H5 16px 抽屉，每帖限 1 个可运行作品（V2 ZIP / HTML 自动转 V2 / 关联创作） |
| [更换头像（手机 + 网页双端）](../../client/xiaoban-multiplatform/docs/design-specs/prototypes/avatar-editor.html) | 个人与账号 | 已合入 | 手机 App 与网页窄屏共用底部选图弹层 + 390 方形裁剪页；网页 PC 采用 820px 居中弹窗并排预览 96/48/32/24 四尺寸 |

---

## 二、网页版（小伴官网 & 超级小伴社区）

完整说明见[官网设计索引](../../services/server-xiaoban/docs/design-specs/README.md)与[社区设计索引](../../services/server-forum/docs/design-specs/README.md)。上方「发帖页」「更换头像」亦覆盖社区网页端。

| 设计稿 | 站点 | 状态 | 内容 |
| --- | --- | --- | --- |
| [小伴产品门户 · 产品矩阵](../../services/server-xiaoban/docs/design-specs/prototypes/xiaban-matrix-portal.html) | xiaban.xyz | 已合入 | v2.1 在首页产品列表和页脚末尾增加「AI 订阅测评」，归入「网站」筛选。封面取自 AI 额度监控原型。[本地预览](http://127.0.0.1:1455/services/server-xiaoban/docs/design-specs/prototypes/xiaban-matrix-portal.html) |
| [小伴桌面独立下载页](../../client/xiaban-portal/docs/design-specs/prototypes/desktop-download-order.html) | xiaban.xyz/desktop | 已合入 | 首屏平台选项按 Windows、macOS 的顺序展示。「让桌面活起来」增加「龙鱼巡游」案例，链接到小伴社区微观展厅的展品详情。预览需运行门户 Vite。[浏览器预览](http://127.0.0.1:5179/docs/design-specs/prototypes/desktop-download-order.html#desktop-case) |
| [小伴桌面官网 · 功能页 & 下载页](../../services/server-xiaoban/docs/design-specs/prototypes/features-download-redesign.html) | wallpaper.xiangban.xyz | 已合入 | v2.3 重设计：极简暖白 · 墨黑与日落珊瑚点缀，首屏 8 场景双端实机展台、电脑与手机核心能力 Bento、当前设备推荐下载卡与更新日志弹窗 |
| [小伴官网 · 管理后台全模块重设计](../../services/server-xiaoban/docs/design-specs/prototypes/admin-console-redesign.html) | wallpaper.xiangban.xyz/admin | 已合入 | v2.0 规范对齐版：方案四浮起白面板壳层、无描边浅底白卡控件、6 套官方主题色 × 明暗模式、说明收进悬停提示、标准 `xb-menu` / `⋯` 菜单，覆盖运营总览、壁纸图库、快采工作台、自定义上架、四端 App 发版、Android 公告实时预览、AI 模型目录与图源密钥（已合入 `services/server-xiaoban/web` 管理后台） |
| [超级小伴社区 · 主站浏览（PC + H5）](../../services/server-forum/docs/design-specs/prototypes/community-redesign/index.html) | chat.xiangban.xyz | 已合入 | 现代活力社区风主站，覆盖 PC 左导轨壳层与 390px 移动 H5，包含首页五栏目、讨论、资源、应用与小伴 AI |
| [社区详情页（作品 / 帖子 / 开源三态 · PC + H5）](../../services/server-forum/docs/design-specs/prototypes/community-redesign/index.html) | chat.xiangban.xyz | 已合入 | 总导航按三类详情与身份视角拆出独立入口：创意作品（作者视角）、讨论帖子（读者视角）、开源精选（管理员视角） |
| [社区个人中心四页套件（PC + H5）](../../services/server-forum/docs/design-specs/prototypes/web-profile-suite.html) | chat.xiangban.xyz | 部分合入 | 1:1 对齐手机 App 方案 1 通栏精修，独立覆盖「我的 / 个人主页 / 我的作品 / 通知」四页（我的/通知已合入，我的作品草稿与居中删除确认待补齐） |
| [社区详情页分享面板（PC + H5）](../../services/server-forum/docs/design-specs/prototypes/web-detail-share.html) | chat.xiangban.xyz | 已合入 | 纯黑白灰中性配色、仅分享图保留品牌主题色：H5 1:1 对齐手机 App，PC 端 716px 居中双栏弹窗与内联删除确认。可运行作品支持从分享面板和 V2 标题栏打开客户端。[作品分享](http://127.0.0.1:1450/prototypes/web-detail-share.html?scene=work&work=whale-girl) · [V2 运行页](http://127.0.0.1:1450/prototypes/web-detail-share.html?scene=work&work=whale-girl&view=run) |
| [社区管理后台 UI 重设计](../../services/server-forum/docs/design-specs/prototypes/admin-redesign.html) | chat.xiangban.xyz/admin | 已合入 | 现代极简运营工作台重设计：将 23 个平铺导航收口为 4 大域 × 12 入口（100% 兼容旧路由），覆盖工作台待办行动中心、主从双栏审核中心、侧滑抽屉编辑与 Diff 核对、精选项目四步发布流、采编健康闭环、⌘K 快捷跳转、日夜模式与 6 套主题色 |
| [AI 网关管理后台 · 官方设计规范版重设计](../../services/server-ai/docs/design-specs/prototypes/gateway-admin-redesign.html) | server-ai `/admin/` | 已合入 | 严格遵循小伴全端统一设计规范：无色偏纯中性画布（浅 `#FAFAFA/#FFFFFF/#F2F2F2`、深 `#111111/#1C1C1E/#2C2C2E`）、90/8/2 配色占比、6 套官方主题色（默认小伴红）+ 日夜切换、中性底+圆点状态标签、16px 零阴影静态卡片、24px 弹窗与 0 原生表单控件；已落地至 `services/server-ai/web`（[本地预览](http://127.0.0.1:1455/services/server-ai/docs/design-specs/prototypes/gateway-admin-redesign.html)） |
| [相伴账号中心管理后台 · 极简控制台重设计](../../services/server-account/docs/design-specs/prototypes/account-admin-redesign.html) | server-account `/admin/` | 已合入 | 对齐小伴暖白极简 + 墨黑中性基调 + 双星日落珊瑚徽标（浅/深色切换、`⌘K` 快捷跳转与用户检索、URL Hash 同步与移动端抽屉导航），重构登录页、概览与验证码来源 IP 监控、用户管理与详情抽屉、接入应用、发信通道（SMTP/腾讯云/阿里云 PNVS）与注册开关、结构化时间线操作日志并全面替换原生确认/提示弹窗；已落地至 `services/server-account/web`（[本地预览](http://127.0.0.1:1455/services/server-account/docs/design-specs/prototypes/account-admin-redesign.html)） |
| [AI订阅测评 QuotaReset · 御三家重置雷达与订阅额度实测站（中英双语 · PC + 移动端）](prototypes/ai-quota-monitor.html) | 独立监控站点 | 已合入 | 已落地为中英双语静态站 [client/aireset-web](../../client/aireset-web/README.md)（`aireset.xiaban.xyz`，支持 `/`、`/benchmark/`、`/en/`、`/en/benchmark/`，部署见 [infra/aireset](../../infra/aireset/README.md)）。覆盖 ChatGPT/Codex、反重力（Antigravity）、Claude 御三家全局重置监控（类 `tibo.cc`）与订阅额度等效 API 实测对比（类 `modeldial.com`）：顶部双 Tab 分离（重置雷达 / 订阅额度实测）、中英文一键切换、三平台点击卡片直筛统一历史重置日志、30 天三轨同步重置时间轴、平均月/每周/五小时可切 10 档订阅实测与多模型拆解、小伴社区交流群（`chat.xiangban.xyz/contact`）与小伴产品门户（`xiaban.xyz`）直达、深浅色切换及内置自适应/PC 1280px/手机 390px 预览切换（[本地预览](http://127.0.0.1:1455/docs/design-preview/prototypes/ai-quota-monitor.html)） |

---

## 三、小伴桌面（桌面版）

完整规范见[小伴桌面设计资源索引](../../client/xiaoban-desktop/docs/design-specs/README.md)。

| 设计稿 | 模块 | 状态 | 内容 |
| --- | --- | --- | --- |
| [小伴桌面 UI 升级画布（5 模块拆达）](../../client/xiaoban-desktop/docs/design-specs/ui-upgrade/README.md) | 主界面 / 超级小伴 / 资料库 / 市场 / 输入框 | 已合入 | Claude Design 39 画板导出，总览页默认隐藏 12 张已替代/历史对比画板，总导航按 5 个独立模块（8+4+4+8+3 = 27 张有效画板）直达；[在线画布](https://claude.ai/artifact/GzhNp5y7ehkzSrWWExkqzg) |
| [设置页 & 小伴 AI / 第三方多模型配置](../../client/xiaoban-desktop/docs/design-specs/prototypes/ai-model-settings.html) | 设置与模型 | 已合入 | 直接挂载真实桌面独立窗口组件（360px 小伴 AI 额度与逐模型 `hi` 检测、第三方服务商配置、完整设置页）；需运行桌面 Vite `npm run dev` |
| [对话资产](../../client/xiaoban-desktop/docs/design-specs/prototypes/conversation-assets.html) | 核心 AI 与对话 | 已合入 | 右上角单图标收纳设计稿、生成图片、交付文件与文件夹，支持轻量浮层与固定侧栏切换；[可编辑源码](../../client/xiaoban-desktop/docs/design-specs/prototypes/conversation-assets.fragment.html) |
| [工作空间检索与列表折叠](../../client/xiaoban-desktop/docs/design-specs/prototypes/workspace-search.html) | 核心 AI 与对话 | 已合入 | 标题、正文与工作区检索，项目与项目内会话默认各折叠展示 6 项，点击结果自动展开定位 |
| [长期目标弹窗](../../client/xiaoban-desktop/docs/design-specs/prototypes/long-term-goal.html) | 核心 AI 与对话 | 已合入 | 聊天页长期目标的新建、执行中与待验收三态及折叠设置预览 |
| [超级小伴 · 区域截图工具](../../client/xiaoban-desktop/docs/design-specs/prototypes/assistant-capture-overlay.html) | 超级小伴 | 已合入 | 压暗蒙层、框选微调、标注工具栏与回填对话交互基准 |
| [组件与伙伴卡片](../../client/xiaoban-desktop/docs/design-specs/prototypes/decor-library.html) | 装扮与桌面空间 | 已合入 | 组件默认四列放大真实预览，伙伴无外框卡片与轻量实例计数；需运行桌面 Vite `npm run dev` |
| [游戏库卡片重设计](../../client/xiaoban-desktop/docs/design-specs/prototypes/game-library.html) | 装扮与桌面空间 | 待合入 | 3–4 列统一方形封面无外框卡片、悬停展开「开始游戏」；正式游戏页尚未改造；需运行桌面 Vite `npm run dev` |
| [特效页优化设计稿](../../client/xiaoban-desktop/docs/design-specs/prototypes/effects-library.html) | 装扮与桌面空间 | 待合入 | 三列特效库与右侧参数面板，静帧采样、悬停播放、预览调参后手动应用；正式特效页尚未改造；需运行桌面 Vite `npm run dev` |
| [AI 插件页卡片与独立窗口重设计（定稿方案 A）](../../client/xiaoban-desktop/docs/design-specs/prototypes/ai-plugins-redesign.html) | 核心 AI 与对话 | 已合入 | 定稿方案 A 精装目录卡（克制主题色、2 行简介、低饱和分类图标、宽屏 3 列、整卡可点），覆盖专家 / 技能 / 连接器三页与 5 类安装/更新/连接器独立窗口；纯静态可直开 |
| [关怀提醒与定时任务重设计](../../client/xiaoban-desktop/docs/design-specs/prototypes/care-reminders-redesign.html) | 关怀与效率 | 已合入 | V2 单栏专注流重构（已合入 `CarePage.vue`、`care-bubble.ts` 与 `reminders.rs`）：移除右侧常驻设置栏，改为按条绑定提醒方式（行内直切响铃/朗读/常驻 + 编辑窗 3 格卡），全局免打扰与默认偏好收纳进页头浮层；新增一句话快建栏、下一条提醒焦点卡、定时任务结果卡片流与 4 态悬浮气泡（[本地预览](http://127.0.0.1:1455/client/xiaoban-desktop/docs/design-specs/prototypes/care-reminders-redesign.html)） |
| [内置小组件精修与组件页排版重设计](../../client/xiaoban-desktop/docs/design-specs/prototypes/widgets-redesign.html) | 装扮与桌面空间 | 已合入 | 8 款内置小组件矢量精修（暖白 / 深色 / 柔雾玻璃三材质、标题栏避让设置按钮、天气 6 态 + 3 日预报、待办快输、日程 Flex 徽章、股票昨收线与悬停读数、木鱼连击多气泡、月历农历 + 休/班、纪念日自定义短句）与组件页 Bento 跨列画廊、轻量层级菜单及统一主题设置浮窗；纯静态可直开（[本地预览](http://127.0.0.1:1455/client/xiaoban-desktop/docs/design-specs/prototypes/widgets-redesign.html)） |
| [伙伴页卡片与独立配置窗口设计稿](../../client/xiaoban-desktop/docs/design-specs/prototypes/companion-redesign.html) | 装扮与桌面空间 | 已合入 | 保留原版 7 只官方伙伴真实形象并严格克制主题色（全页仅主按钮与开关用主题色）：纯净中性浅画框与中性柔光展台、底栏紧凑中性「+ 添加 / 管理」与轻量层级浮层、在桌面上两行清晰实例卡（表情/多屏角标 + 行内层级快切）、卡片 8 态解剖展板及伙伴设置/主动陪伴原生独立窗口（已合入 `CompanionPage.vue`、`PetSettingsDialog.vue`、`PetProactivityPanel.vue`、`PetConfigWindow.vue` 与 Rust `configuration_window`）；纯静态可直开（[本地预览](http://127.0.0.1:1455/client/xiaoban-desktop/docs/design-specs/prototypes/companion-redesign.html)） |
| [伙伴双态气泡 UI 重设计稿](../../client/xiaoban-desktop/docs/design-specs/prototypes/companion-bubbles-redesign.html) | 装扮与桌面空间 | 已合入 | 定稿暖感对话泡泡（暖瓷白 / 暖炭表面、22px 圆角、全圆胶囊操作、无底部倒计时条与左右/底右指向尾尖），覆盖桌面伙伴主动问候气泡与桌面/输入框点击对话卡片：内置 4 套壁纸与 4 只伙伴的桌面实机试玩、输入框小伴宽窄窗场景与 9 态解剖展板；已合入 `GreetingBubble.vue`、`CompanionReplyCard.vue`、`ChatComposerPet.vue` 与 Rust `proactivity.rs`（[本地预览](http://127.0.0.1:1455/client/xiaoban-desktop/docs/design-specs/prototypes/companion-bubbles-redesign.html)） |

---

## 四、小游戏与小程序（协议 2 与应用广场）

| 设计稿 | 模块 | 状态 | 内容 |
| --- | --- | --- | --- |
| [暂停 / 存档 / 翻页退出统一规范](../../client/xiaoban-multiplatform/docs/design-specs/prototypes/game-pause-save-exit.html) | 宿主运行规范 | 已合入 | 覆盖无大厅直开《方块爆破》、有大厅多层《水墨剑客》与多页翻页小程序：32px 自适应单入口宿主胶囊、残局恢复与侧滑逐层回退 |
| [倒杯快乐 · 奶茶大师升级 V2](../../client/xiaoban-desktop/docs/design-specs/pour-joy/README.md) | 休闲小游戏 | 已合入 | 小店、可试玩调饮台、风味手帐与通关分享小票（正式版 `water-sort` v2.1.0 已升级至 45 关 + 3 教学关）；[在线预览](https://pour-joy-design.bestyize.chatgpt.site) |
| [MBTI · 向内看](../../client/xiaoban-multiplatform/docs/design-specs/prototypes/mbti-native.html) | 实用小程序 | 已合入 | 原生 iOS 质感第四版（珊瑚橙红 + 暖白 + 墨灰），首页、6 题缩略流程、六章报告、16 型角色与分享卡（已合入内置 MBTI 1.3.0） |
| [人格贩卖机 · 趣味与人格测评 H5](../../client/xiaoban-multiplatform/docs/design-specs/prototypes/personality-tests-h5.html) | 独立 H5 / 拉新测评 | 设计评审中 | 移动端 H5 优先 6 画板可交互设计稿（支持总览/单机试玩、390/360px、深浅色、系统分享与真实 PNG 海报导出），覆盖 5+1 首发话题与 6 套专属视觉海报皮肤（真实面具 SBTI 风、双色主导色、动物塑、避雷说明书、精神电量体检单、16 型速测）、零门槛双人合盘与多维灵魂档案；配套方案见 [PRD_趣味与人格测评H5.md](../product/PRD_趣味与人格测评H5.md)（[本地预览](http://127.0.0.1:1455/client/xiaoban-multiplatform/docs/design-specs/prototypes/personality-tests-h5.html)） |
| [小伴游戏厅 · game.xiaban.xyz 移动端游戏站](../../client/xiaoban-multiplatform/docs/design-specs/prototypes/xiaoban-games-h5.html) | 独立 H5 / 游戏合集 | 设计评审中 | 香港纯静态移动端游戏站 8+1 画板可交互设计稿（支持总览/单机试玩/真机全屏、390/360px、暖瓷浅色/掌机深色与 6 套主题色）：精装掌机展厅风，收录小伴现役 8+1 款官方游戏，覆盖「继续上次」断点续玩、今日主打大片轮播、单手竖屏筛选与随机摇号、卡带预览抽屉、竖屏沉浸运行与透明三点宿主菜单、横屏旋转引导与右侧窄抽屉、免登录战绩档案、通栏精修「我的」页、统一相伴账号 4 态登录注册弹窗及分享直达页（[本地预览](http://127.0.0.1:1455/client/xiaoban-multiplatform/docs/design-specs/prototypes/xiaoban-games-h5.html)） |
| [轻启](../../client/xiaoban-multiplatform/docs/design-specs/prototypes/quick-launch.html) | 实用小程序 | 已合入 | Android 桌面快捷方式工具原型，256×256 WebP 图标网格、搜索筛选与添加到桌面交互 |
| [遥控器](../../client/xiaoban-multiplatform/docs/design-specs/prototypes/universal-remote.html) | 实用小程序 | 已合入 | 协议 2 电视与空调红外工具，按市场份额排序的 19 电视 / 21 空调品牌搜索试码、四点方向盘、大字温控与无圆底竖三点宿主菜单 |
| [万磁搜](../../client/xiaoban-multiplatform/docs/design-specs/prototypes/torrent-search.html) | 实用小程序 | 已合入 | 协议 2 独立搜索工具设计稿，覆盖首屏、跨站搜索结果、来源筛选、失败态与磁力操作弹层（已合入 `wancisou` v1.0.2） |
| [收盘别问](../../client/xiaoban-multiplatform/docs/design-specs/prototypes/shoupan-biewen.html) | 休闲小游戏 | 概念试玩 | 390×800 竖屏 H5 单页，100 个真实创业板交易日、可横滑 K 线、提前收手结算与带二维码的 PNG 成绩海报（尚未合入内置应用包） |
