# 设计资源与预览

状态：设计参考，静态文件用于讨论外观交互，不是客户端运行入口，不参与生产 Docker 构建。

实现设计时修改对应 client 工程并验证；不要直接用预览覆盖已有应用、鉴权或平台接入逻辑。

## 小伴桌面

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
| [开源精选详情页](../../client/xiaoban-multiplatform/docs/design-specs/prototypes/open-source-detail.html) | 顶部项目卡片（含 Star 数与更新日期）、中文 / English README 语言切换、官方配图与可复制命令，支持手机 / 桌面、深浅色、图片放大、收藏、小红书风格评论 / 回复与贴边互动栏。[在线预览（本机）](http://127.0.0.1:1423/prototypes/open-source-detail.html)，启动方法见手机设计索引；未修改正式页面。 |
| [应用广场](../../client/xiaoban-multiplatform/docs/design-specs/prototypes/app-plaza.html) | 底栏「应用」：广场（精选游戏、精选工具、创作者精选、社区作品瀑布流）与我的作品；精选游戏用方形完整封面，方向标记放在图片下方，创作者精选用紧凑横滑卡片；MBTI 支持 16 型主题与角色切换，增加提交后的分析过渡、六章完整报告、图表与情境表格，以及独立的 3:4 分享卡（含测试日期）、图片/链接朋友圈分享示意和 16 个原创角色图鉴；官方游戏和工具轻点直接打开，长按查看详情；社区作品是普通帖子，「做一个我的版本」与改编关系；竖屏 / 横屏全屏、紧凑半透明宿主胶囊、两次返回退出；首页下拉打开最近用过与收藏。[在线预览（本机）](http://127.0.0.1:1424/prototypes/app-plaza.html) |
| [手机创造中心](../../client/xiaoban-multiplatform/docs/design-specs/prototypes/create-center.html) | 游戏与桌面装扮统一入口，三画面对照：发现、我的作品、作品预览；我的作品支持二次确认删除；卡片移除大操作按钮，点击展开底部面板，集中修改、发送到电脑、分享到社区 / 其他渠道；发现页移除重复的新建按钮，仅我的作品页保留「＋」；交互均为模拟，支持明暗及 360 / 390px。[在线预览](http://127.0.0.1:1421/prototypes/create-center.html)。已接入 Android：游戏优先，卡片操作面板、社区发布与系统分享；电脑装扮通过遥控对话创作，作品直传暂用系统分享。 |
| [手机游戏中心 A / B](../../client/xiaoban-multiplatform/docs/design-specs/prototypes/game-center.html) | A 轻量游戏库与 B 沉浸游乐场；两版并排、390 / 360px、深浅色、我的游戏、详情、创作与启动交互。[在线预览](http://127.0.0.1:1421/prototypes/game-center.html)。已选 A 版接入手机正式页面，Android 使用只读官方游戏与真实最近游玩记录；实机效果待验收 |
| [社区与小伴AI 手机原型](../../client/xiaoban-multiplatform/docs/design-specs/mobile-prototype/README.md) | 70 个画板：社区、小伴 AI、游戏、我的与设置、分享、设备遥控、聊天状态卡片、实时语音与按住说话；[在线画布](https://claude.ai/artifact/BGtDJ7S7vGenqaY5MvyLLJ) |

两份 Claude Design 画布导出的设计稿由多个 `*.dc.html` 画板组成，需在各自目录运行 `python3 -m http.server` 后打开 `index.html` 总览；其余单个 HTML 设计稿均自包含，可直接用浏览器打开，展示数据为模拟内容。设计稿源文件统一保留在客户端设计目录，避免多份副本产生差异。

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
