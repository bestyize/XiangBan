# 设计资源与预览

状态：设计参考，静态文件用于讨论外观交互，不是客户端运行入口，不参与生产 Docker 构建。

实现设计时修改对应 client 工程并验证；不要直接用预览覆盖已有应用、鉴权或平台接入逻辑。

## 超级小伴社区网页版

[个人中心四页重设计稿（我的 / 个人主页 / 我的作品 / 通知 · PC + 移动 H5）](../../services/server-forum/docs/design-specs/prototypes/web-profile-suite.html) 对齐手机 App 方案 1 视觉规范与 `community-redesign` PC 端左导轨壳层，支持 PC + H5 同屏预览四页交互、本人/他人主页视角、内容包与方案筛选、未登录/空态、深浅色与 6 套主题色（仓库根运行 `python3 -m http.server 1450 --bind 127.0.0.1 --directory services/server-forum/docs/design-specs`）。[PC + H5 同屏预览](http://127.0.0.1:1450/prototypes/web-profile-suite.html) · [PC 端](http://127.0.0.1:1450/prototypes/web-profile-suite.html?device=desktop) · [手机 H5](http://127.0.0.1:1450/prototypes/web-profile-suite.html?device=mobile)

[社区全站 UI 重设计稿（活力社区风 · PC + 移动 H5）](../../services/server-forum/docs/design-specs/prototypes/community-redesign/index.html) 采用现代活力视觉社区风，覆盖 PC 端网页（左导轨 + 精简顶栏）与移动端 H5（390px 交互样机，右上角纯图标搜索与提醒不带背景，创意页卡片统一 `3:2` 封面），包含创意、资讯、精选、排行榜、导航、讨论、资源及作品详情评论，支持日夜切换与 6 套客户端主题色（仓库根运行 `python3 -m http.server 1449 --bind 127.0.0.1 --directory services/server-forum/docs/design-specs/prototypes/community-redesign`）。[PC + H5 同屏预览](http://127.0.0.1:1449/) · [PC 端网页](http://127.0.0.1:1449/?device=desktop) · [移动端 H5](http://127.0.0.1:1449/?device=mobile)

[创意页作品卡片](../../services/server-forum/docs/design-specs/README.md) 首张卡片直达微观展厅，后两张分别展示《小王子：星辰之旅》及「运输船：Saltline」的详情和打开方式。[本机在线预览](http://127.0.0.1:1448/)。

[创意作品详情页](../../services/server-forum/docs/design-specs/prototypes/ideas-cards/work-detail.html) 用「碎花垂耳兔」展示通栏详情、完整作品图和灰底的体验按钮。评论区沿用全站列表、入口和编辑弹层样式。预览沿用创意页作品卡片的服务。[电脑预览](http://127.0.0.1:1448/work-detail.html) · [手机预览](http://127.0.0.1:1448/work-detail.html?view=mobile)

[讨论卡片设计稿](../../client/xiaoban-multiplatform/docs/design-specs/prototypes/discussion-cards.html) 展示双端单列布局与带图定稿（方案 A 左文右图缩略图，无多图角标，底栏单行不换行并在窄屏自动隐藏标签；可切换方案 B 下方多图条与现行纯文字对照），混排「羊毛」带图帖与纯文字讨论帖，支持明暗切换。[桌面预览](http://127.0.0.1:1421/prototypes/discussion-cards.html) · [移动预览](http://127.0.0.1:1421/prototypes/discussion-cards.html?view=mobile)，启动方式见[手机设计索引](../../client/xiaoban-multiplatform/docs/design-specs/README.md)。

[发帖页设计稿（PC + 移动 H5）](../../client/xiaoban-multiplatform/docs/design-specs/prototypes/post-composer.html) 采用写作优先无框结构：PC 820px 居中弹窗提供级联板块选择、编辑/预览切换、单作品入口（V2 ZIP / HTML·HTM 自动转 V2 容器 + 关联创作）、内联附件积分与上拉权限设置；H5 窄屏采用 16px 防缩放底部抽屉，支持深浅色与 6 套主题色。[网页版预览](http://127.0.0.1:1421/prototypes/post-composer.html?view=web)（与手机共用 1421 端口）。

[更换头像设计稿（Android + 网页 PC / H5）](../../client/xiaoban-multiplatform/docs/design-specs/prototypes/avatar-editor.html) 只画更换头像弹窗：手机 App 与网页窄屏共用底部选图弹层和 390 方形裁剪页，网页 PC 用 820 居中弹窗并排预览四种尺寸；可模拟保存失败与重试，失败时旧头像不变、裁好的图保留。[本机预览](http://127.0.0.1:1421/prototypes/avatar-editor.html)（与手机共用 1421 端口）。

## 小伴桌面

[倒杯快乐 · 奶茶大师升级设计稿](../../client/xiaoban-desktop/docs/design-specs/pour-joy/README.md)：小店、可试玩调饮、风味手帐、通关分享小票；三道固定样题、灵感提示、可保存 PNG 与同题链接。[在线预览](https://pour-joy-design.bestyize.chatgpt.site)（私有，后续原址更新）。源文件已入仓库目录，正式游戏源码已升级为 36 关双端版，尚未重新打包安装。 V2：精简说明文案、高饱和液体、玻璃高光与立体彩色按钮，沿用原预览。

| 设计稿 | 内容 |
| --- | --- |
| [对话资产](../../client/xiaoban-desktop/docs/design-specs/prototypes/conversation-assets.html) | 右上角单图标入口，包含设计稿、生成图片、交付文件 / 文件夹；轻量浮层与常驻侧栏两版，可搜索、预览图片、展开目录、定位消息与导入资产。支持明暗、主题色、空状态和文件失效预览，稿中系统操作均为模拟。正式聊天页已接入轻量浮层、固定侧栏与本机文件 / 文件夹 / 设计源文件导入，普通导入保留原件；尚未重新打包。[在线预览](http://127.0.0.1:1420/docs/design-specs/prototypes/conversation-assets.html)，需在桌面项目运行 `npm run dev`；[可编辑源码](../../client/xiaoban-desktop/docs/design-specs/prototypes/conversation-assets.fragment.html) |
| [小伴 AI · 多模型配置](../../client/xiaoban-desktop/docs/design-specs/prototypes/ai-model-settings.html) | macOS 白与浅灰轻毛玻璃视觉提案：统一白色内容面、轻量额度概览、细条兑换区、无搜索框的灰阶列表与无边框兑换码输入框、无边框按钮与品牌色确认操作，“兑换”右侧新增可配置链接的“获取额度”入口，每行模型支持发送 `hi` 的检测交互演示；可对照现行界面，支持 4 / 24 / 80 个模型、明暗与窄窗口。已应用到产品独立窗口，支持真实模型检测与后台下发额度链接；同一预览加入第三方模型配置、完整设置页及左栏顶部自绘窗口控件，移除系统顶部标题栏；Windows 原生效果尚未实机验收。[本机在线预览](http://127.0.0.1:1420/docs/design-specs/prototypes/ai-model-settings.html)沿用原链接，需在桌面项目运行 `npm run dev`，不连接真实账户或模型服务 |
| [小伴桌面 UI 升级设计稿](../../client/xiaoban-desktop/docs/design-specs/ui-upgrade/README.md) | 39 个画板：主界面、设置、超级小伴、资料库、实时对话、小伴市场接入、输入框与各类状态卡片；[在线画布](https://claude.ai/artifact/GzhNp5y7ehkzSrWWExkqzg) |
| [工作空间检索与列表折叠](../../client/xiaoban-desktop/docs/design-specs/prototypes/workspace-search.html) | 标题、正文与工作区检索；限高滚动结果；项目和项目内会话默认各展示 6 项；点击结果自动展开并定位会话 |
| [长期目标弹窗](../../client/xiaoban-desktop/docs/design-specs/prototypes/long-term-goal.html) | 新建、执行中与待验收状态；折叠设置；浅色、深色及窄屏预览。遮罩仅用于演示内容，桌面端窗口形式按独立窗口规范 |

## 小伴手机 App

| 设计稿 | 内容 |
| --- | --- |
| [模型配置全链路](../../client/xiaoban-multiplatform/docs/design-specs/prototypes/model-config-redesign.html) | 模型配置主页、小伴 AI 官方配置、添加第三方服务、第三方服务商配置与对话模型+思考等级五屏重设计稿（待确认）：严格控色（90% 中性黑白灰，每屏至多 1 处主题色）；02 官方模型补齐思考等级胶囊与贴底弹层；04 协议改为独立选择行并自动联动预置 API 地址，高级设置新增「支持的思考等级」底部多选弹层（默认 低/中/高/极高/最高）；05 模型名严格左对齐，思考等级统一贴底 BottomSheet。支持 390/360px、深浅色与六套主题色。[本机预览](http://127.0.0.1:1421/prototypes/model-config-redesign.html) |
| [详情页分享面板](../../client/xiaoban-multiplatform/docs/design-specs/prototypes/detail-share-sheet.html) | 全站详情页（帖子 / 作品 / 开源 / 日报）统一分享面板重设计稿（待确认）：删掉大段重复正文、堆砌大胶囊与「举报」，改用单行预览条 + 单行可横滑图标轨（系统分享、复制链接、分享卡片、问问小伴、复制全文/发到电脑/复制仓库、浏览器打开，主态末尾放「删除」并支持单次内联确认）+ 通栏「取消」；「分享卡片」采用沉浸式磨砂舞台，卡片下悬浮纸色圆点与摘录/二维码开关，底部改为通透三图标操作栏（保存图片 / 发送给好友 / 复制图片）。支持现行对照、异常态、390/360px、深浅色与六套主题色。[本机预览](http://127.0.0.1:1421/prototypes/detail-share-sheet.html) |
| [我的作品 / 通知 / 模型配置](../../client/xiaoban-multiplatform/docs/design-specs/prototypes/mine-subpages-redesign.html) | 「我的」三大二级页统一通栏精修稿（待确认）：我的作品去掉顶部刷新条与卡片按钮堆砌，改用状态标签 + 右侧「···」底部菜单；通知移除 Tab 下双大灰按钮与重复刷新，将「全部已读」收进顶栏右上角并补齐头像与未读红点；模型配置更名并修复直角块割裂，直观标出「使用中」服务商与官方点数状态。支持弹层交互、空态/多服务商切换、390/360px、深浅色与六套主题色。[本机预览](http://127.0.0.1:1421/prototypes/mine-subpages-redesign.html) |
| [「我的」页与个人主页](../../client/xiaoban-multiplatform/docs/design-specs/prototypes/mine-redesign.html) | 手机「我的」页定稿（方案 1 通栏精修）与个人主页（本人视角 + 社区他人视角）三屏联动稿（待确认）：「我的」页移除「屏蔽的用户」、重复的「创作」与「关注与粉丝」及外露积分，「模型与点数」改名「模型配置」，右上角换为齿轮图标，快捷三宫格为「我的收藏 / 我的作品 / 通知」，下方仅留 2 组 4 项线性图标列表；个人主页统一通栏风格，配 68px 头像与 ID、四列等宽数据条（帖子/回复/关注/粉丝）、图文混排帖子卡、带头像的关注/粉丝列表，他人页「屏蔽」收进右上角「···」菜单。支持场景、未登录、空态、已屏蔽、390/360px、深浅色与六套主题色切换。[本机预览](http://127.0.0.1:1421/prototypes/mine-redesign.html) |
| [发帖页](../../client/xiaoban-multiplatform/docs/design-specs/prototypes/post-composer.html) | 手机端发帖页重设计（01 初始写作全交互 + 02 富内容完整态 + 03 选板块/作品/投票/封面/键盘五态）及网页版对照：紧凑单行板块与封面入口、正文下内联话题胶囊、每帖限 1 个可运行作品（V2 ZIP / `.html`·`.htm` 转 V2 / 关联创作）与普通附件区分、固定 48px 单行底栏、高对比投票芯片与紧耦合文字封面编辑。支持 390/360px、深浅色与六套主题色。[本机预览](http://127.0.0.1:1421/prototypes/post-composer.html?view=mobile) |
| [小伴 AI 首页](../../client/xiaoban-multiplatform/docs/design-specs/prototypes/ai-home.html) | 个人 Agent 连续主线三屏定稿（01 默认浏览态 + 02 键盘弹出态 + 03 点右上角「···」进入的个人 Agent 配置页）：极简顶栏 + 分离悬浮胶囊输入框 + 通栏白底配置列表，支持三态、390/360px、深浅色与六套主题色切换。[本机预览](http://127.0.0.1:1421/prototypes/ai-home.html) |
| [动态壁纸](../../client/xiaoban-multiplatform/docs/design-specs/prototypes/live-wallpapers.html) | Android 壁纸库、三种全屏预览和封面使用状态。原型模拟系统壁纸位置选择。[本机预览](http://127.0.0.1:1421/prototypes/live-wallpapers.html) |
| [评论组件](../../client/xiaoban-multiplatform/docs/design-specs/prototypes/comment-composer.html) | 键盘、表情与相册三种状态。点笑脸打开表情面板，选图后可预览和移除。支持单字或单表情发布、回复和深浅色预览，图稿使用小伴示例壁纸。[本机预览](http://127.0.0.1:1421/prototypes/comment-composer.html) |
| [MBTI · 向内看](../../client/xiaoban-multiplatform/docs/design-specs/prototypes/mbti-native.html) | 原生 iOS 质感的首页、答题、结果交互提案（第四版：珊瑚橙红＋暖白＋墨灰、暖色首页插画、紧凑间距、加大正文）；支持三页总览、单页预览、6 题演示、六章报告、16 型切换、深浅色和分享卡。结果与历史记录为示例，保存分享为模拟；源文件与素材在仓库内，已应用到内置 MBTI 1.3.0（正式版使用真实作答与存档）。[本机在线预览](http://127.0.0.1:1424/prototypes/mbti-native.html)，启动方法见手机设计索引，后续沿用同一地址。 |
| [轻启](../../client/xiaoban-multiplatform/docs/design-specs/prototypes/quick-launch.html) | Android 桌面快捷方式设计稿：收录中国区 App Store 免费榜前 100 的应用及常用应用，使用 256 × 256 WebP 图标；搜索筛选、Scheme 测试和添加到桌面均为模拟，候选 Scheme 待真机验证。[本机在线预览](http://127.0.0.1:1421/prototypes/quick-launch.html)。 |
| [社区作品分享面板](../../client/xiaoban-multiplatform/docs/design-specs/prototypes/community-work-share.html) | 作品封面预览、四个分享入口、复制反馈及独立海报预览。支持浅色、深色和窄屏，分享操作为演示。[在线预览（本机）](http://127.0.0.1:1421/prototypes/community-work-share.html) |
| [遥控器](../../client/xiaoban-multiplatform/docs/design-specs/prototypes/universal-remote.html) | 电视与空调小程序的可点按设计稿。包含我的遥控器、添加与试码、电视圆形四点方向键、双段音量与独立返回键，以及空调大字温度、风速和风向状态与四行全宽按键。品牌选择列出 19 个电视品牌和 21 个空调品牌，按 2026 年 9 月中国市场的公开份额排序，电视按整机出货、空调按全渠道零售额，其余单列。首页「添加」与标题同行靠右，使用 28px 高的白底、黑字和轻阴影。已保存的遥控器可点卡片右上角的铅笔修改备注名。子页标题为 16px，与两侧的返回箭头、竖三点同高。使用海湾蓝，支持深浅色及无红外演示。默认使用贴近右上角的无圆底竖三点，点击打开与 Android 宿主布局对齐的底部菜单，可切回旧版双按钮对照。[在线预览（本机）](http://127.0.0.1:1421/prototypes/universal-remote.html)，启动方法见手机设计索引。 |
| [万磁搜](../../client/xiaoban-multiplatform/docs/design-specs/prototypes/torrent-search.html) | 小伴协议 2 的独立工具应用设计稿：跨站搜索、来源筛选、磁力获取及失败状态；视觉参考 search.xiangban.xyz，使用示例数据和模拟交互。手机画板使用通用竖三点宿主样式。[在线预览（本机）](http://127.0.0.1:1421/prototypes/torrent-search.html)，启动方法见手机设计索引。 |
| [开源精选详情页](../../client/xiaoban-multiplatform/docs/design-specs/prototypes/open-source-detail.html) | 顶部项目卡片（含 Star 数与更新日期）、中文 / English README 语言切换、官方配图与可复制命令，支持手机 / 桌面、深浅色、图片放大、收藏、小红书风格评论 / 回复与贴边互动栏。[在线预览（本机）](http://127.0.0.1:1423/prototypes/open-source-detail.html)，启动方法见手机设计索引；未修改正式页面。 |
| [应用广场](../../client/xiaoban-multiplatform/docs/design-specs/prototypes/app-plaza.html) | 底栏「应用」：广场（精选游戏、精选工具、创作者精选、社区作品瀑布流）与我的作品；精选游戏用方形完整封面，方向标记放在图片下方，创作者精选并入社区作品「推荐」帖子流，万磁搜示例同时展示在精选工具和社区作品；MBTI 支持 16 型主题与角色切换，增加提交后的分析过渡、六章完整报告、图表与情境表格，以及独立的 3:4 分享卡（含测试日期）、图片/链接朋友圈分享示意和 16 个原创角色图鉴；官方游戏和工具轻点直接打开，长按查看详情；社区作品是普通帖子，「做一个我的版本」与改编关系；竖屏 / 横屏全屏、紧凑半透明宿主胶囊、两次返回退出；首页下拉打开最近用过与收藏。横屏与竖屏运行画板使用通用竖三点宿主菜单。[在线预览（本机）](http://127.0.0.1:1424/prototypes/app-plaza.html) |
| [手机创造中心](../../client/xiaoban-multiplatform/docs/design-specs/prototypes/create-center.html) | 游戏与桌面装扮统一入口，三画面对照：发现、我的作品、作品预览；我的作品支持二次确认删除；卡片移除大操作按钮，点击展开底部面板，集中修改、发送到电脑、分享到社区 / 其他渠道；发现页移除重复的新建按钮，仅我的作品页保留「＋」；交互均为模拟，支持明暗及 360 / 390px。[在线预览](http://127.0.0.1:1421/prototypes/create-center.html)。已接入 Android：游戏优先，卡片操作面板、社区发布与系统分享；电脑装扮通过遥控对话创作，作品直传暂用系统分享。 |
| [手机游戏中心 A / B](../../client/xiaoban-multiplatform/docs/design-specs/prototypes/game-center.html) | A 轻量游戏库与 B 沉浸游乐场；两版并排、390 / 360px、深浅色、我的游戏、详情、创作与启动交互。[在线预览](http://127.0.0.1:1421/prototypes/game-center.html)。已选 A 版接入手机正式页面，Android 使用只读官方游戏与真实最近游玩记录；实机效果待验收 |
| [社区与小伴AI 手机原型](../../client/xiaoban-multiplatform/docs/design-specs/mobile-prototype/README.md) | 70 个画板：社区、小伴 AI、游戏、我的与设置、分享、设备遥控、聊天状态卡片、实时语音与按住说话；[在线画布](https://claude.ai/artifact/BGtDJ7S7vGenqaY5MvyLLJ) |

两份 Claude Design 画布导出的设计稿由多个 `*.dc.html` 画板组成，需在各自目录运行 `python3 -m http.server` 后打开 `index.html` 总览；其余 HTML 设计稿可直接用浏览器打开，展示数据为模拟内容。设计稿源文件保留在对应项目的设计目录，避免多份副本产生差异。

完整设计规范与其他设计稿见[小伴桌面设计资源索引](../../client/xiaoban-desktop/docs/design-specs/README.md)。

设置页重设计（2026-10-31）：三组单色导航、底部账号、白色内容面与浅灰设置组，窗口控件固定在左栏顶部；深色模式增强正文、辅助文字和控件对比度。沿用[原在线预览](http://127.0.0.1:1420/docs/design-specs/prototypes/ai-model-settings.html)，默认展示新版设置，支持所有分类与明暗切换；前端代码已更新，本轮未重新打包原生应用。

设置侧栏细化：浅色背景跟随主页面的 `--ui-tint` 次要主题色；一级菜单使用无底色线性图标。账号区仅展示状态，不跳转、不响应点击；无头像时显示主题色 Sparkles 图标，替代猫脸占位图。沿用[原在线预览](http://127.0.0.1:1420/docs/design-specs/prototypes/ai-model-settings.html)。

模型配置 UI 升级：协议菜单保持单行，模型展开设置采用平整布局，蓝色选中态仅作用于标题行；第三方配置采用主题色连接侧栏、25px 标题、浅灰模型列表与主题色默认行；小伴 AI 统一标题、额度明细、兑换条和模型选择样式，保持同页布局及逐模型检测。沿用[原在线预览](http://127.0.0.1:1420/docs/design-specs/prototypes/ai-model-settings.html)，切换「小伴 AI · 新版 / 第三方模型 · 新版」查看，初版提案另保留为参考；代码已更新，尚未重新打包。

模型页精修：统一紧凑标题、添加入口和展开设置对齐，移除“思考摘要”控件；沿用[原在线预览](http://127.0.0.1:1420/docs/design-specs/prototypes/ai-model-settings.html)中的「第三方模型 · 新版」。

参考风格调整与小伴 AI 结构重排（2026-10-31）：配置页统一为轻主题色画布与白色分组卡片，设置页收小分类标题并移除侧栏重复标题。小伴 AI 将账户身份放在页头，余额、额度明细、兑换与获取额度合并为一张卡片，模型另设独立滚动列表，确认操作固定在底部。保留无描边控件、单页布局、模型检测与键盘选择；深色使用独立高对比表面。沿用[原在线预览](http://127.0.0.1:1420/docs/design-specs/prototypes/ai-model-settings.html)，默认查看「小伴 AI · 新版」。本轮更新了实际组件代码，通过类型检查及 63 项相关测试，核对了 360px 窄窗口、明暗与长列表；尚未重新打包原生应用。

小伴 AI 配置窗口默认宽度与最小宽度调整为 360px，高度仍为 620px；窗口尺寸单独记忆，避免继承第三方配置页的宽度。[原在线预览](http://127.0.0.1:1420/docs/design-specs/prototypes/ai-model-settings.html)同步默认显示 360px，仍可手动调整尺寸。

游戏库卡片重设计（待确认）：[设计稿源码](../../client/xiaoban-desktop/docs/design-specs/prototypes/game-library.html)，最少三列（宽窗口四列）、统一方形封面、无外框卡片、单行标题、低调横竖屏与存档图标，悬停/键盘聚焦展开“开始游戏”。支持明暗、主题色、窄窗口与自定义游戏示意，全部操作仅本地提示，不启动或删除真实游戏。[在线预览](http://127.0.0.1:1420/docs/design-specs/prototypes/game-library.html)。正式游戏页尚未修改。

特效页优化设计稿（待确认）：[设计稿源码](../../client/xiaoban-desktop/docs/design-specs/prototypes/effects-library.html)，保留三列特效库与右侧参数面板；采样代表性静帧、悬停播放，区分查看与启用，顶部关闭特效，参数在预览中调整后手动应用。支持明暗、窄窗口与模拟启停；所有交互仅修改页面内演示状态，不修改真实桌面。[在线预览](http://127.0.0.1:1420/docs/design-specs/prototypes/effects-library.html)。正式页面未改动。

组件与伙伴卡片设计稿（待确认）：[源码](../../client/xiaoban-desktop/docs/design-specs/prototypes/decor-library.html)，组件默认四列，放大真实组件预览，保留添加层级选择；伙伴采用无外框卡片、轻量实例数量、固定位置的管理/添加操作，主题导出收进更多菜单。支持页签、明暗、窄窗口、模拟添加与实例管理，均不改变真实桌面。[组件预览](http://127.0.0.1:1420/docs/design-specs/prototypes/decor-library.html?view=widgets) / [伙伴预览](http://127.0.0.1:1420/docs/design-specs/prototypes/decor-library.html?view=pets)。正式页面未修改。
