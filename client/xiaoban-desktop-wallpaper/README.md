# 小伴桌宠 / 动态壁纸原型

Windows / macOS 桌面应用：一个可拖动、透明处点击穿透的悬浮小伴，也可以切换成位于桌面图标下方的动态壁纸。无聊天或账户；默认连接 `https://wallpaper.xiangban.xyz` 获取在线图片、动态视频、版本与公告。Windows 缺少运行组件时会联网补齐；组件与缓存可用后可离线运行。角色、场景、控制台与互动逻辑共用，桌面宿主分平台接入。

2026-10-10 最新 Windows x64 网络恢复修复包：XiaoBan-UI-NetworkFix-Windows-x64.exe（本地：`artifacts/XiaoBan-UI-NetworkFix-Windows-x64.exe`）（6,017,024 bytes，约 5.74 MiB；SHA-256 `AC52792F39CAF2B239867E0B4AD84EB07C90A632A785F46332F0B0E528E58454`）。若先前版本仍在托盘运行，请退出旧实例再启动新版；单实例行为可能会唤起旧进程。此前 UI 版和 P0 版作为历史包保留：UI 版（本地：`artifacts/XiaoBan-UI-Windows-x64.exe`）（6,014,464 bytes；SHA-256 `7EF780BABBA7F2D545EDBF56D7018D3F80E1E58E65CE0DD0CDE1A388B2420278`）、P0 版（本地：`artifacts/XiaoBan-P0-Windows-x64.exe`）（5,814,272 bytes；SHA-256 `71AC7CFDB01BC65A2949400D047503F3B86D2F58739D542ABD1A452B3E8D04D0`）。2026-10-10 Rust 测试 60 项通过；本轮 UI 前端测试此前 51 tests / 11 files 通过，本次网络修复未修改前端且未重跑。自动化及人工验收状态见 [功能验证清单](docs/feature-validation.md)。

历史原型构建记录：早期 Windows x64 EXE 约 4.80MB，原生切换测试曾在主屏 2880×1800、175% 缩放下通过。该体积和验证报告只描述早期原型，不代表本轮构建或人工验收结果。

角色核心在本工程的 `src/companion` 内维护，最初从网页纯角色代码原样复制，保留既有 SVG、发光、表情、弹簧参数和互动行为。允许独立演进，不再导入另一个客户端源码。`src/pet.ts` 适配 DOM 与桌面输入；暖光维持原 RobotStage 的尺寸与渐变。

## 在 Windows 上运行

需要 Node.js 22.12+（或 24）、Rust MSVC 工具链、Visual Studio C++ 构建工具和 WebView2 Runtime。在仓库根执行：

```powershell
cd client/xiaoban-desktop-wallpaper
npm ci
npm run desktop
```

生产构建与启动：

```powershell
npm run desktop:build
.\src-tauri\target\release\xiaoban-desktop-wallpaper.exe
```

这个 `.exe` 内嵌全部前端与背景资源，运行不需要 Node、Vite 或本地服务器。它依赖系统 WebView2，未把浏览器运行时打进包。当前配置不生成安装器；便携 EXE 本身在打开界面前检测运行时，支持 Windows 10/11，WebView2 最低主版本 109。

### 首次运行组件准备

- 运行时可用：直接打开小伴，不显示准备提示、不下载资源；每次检测真实运行时，没有“安装过就永远跳过”的本地标记。
- 运行时缺失或低于最低版本：显示不依赖 WebView 的原生“正在加载必要资源，请稍后”窗口，下载微软官方 Evergreen 引导程序，验证有效 Microsoft Corporation 签名，然后以当前权限静默安装。默认不请求管理员权限。
- 安装后再次检测运行时，成功才创建控制台与角色窗口。安装器退出码为零不等于启动条件已满足。
- 下载、安装、权限或签名问题显示失败原因，允许重试或暂不启动；准备过程中可取消。下载超时 120 秒，安装等待最多 10 分钟，整体进程有超时与退出清理。
- 首次缺组件时需要联网，后续正常启动无需联网。引导程序和脚本放在唯一临时目录，完成或取消后清理；运行时安装到系统/用户组件位置，独立于小伴 EXE 的体积。
- 不支持的旧 Windows 提示升级系统，不尝试下载旧版运行时。macOS 12+ 使用系统 WKWebView，随系统维护，不下载或替换系统 WebKit；最低系统版本已写入 `.app` 配置。

微软官方说明：[运行时按需检测与安装](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/distribution)。设置中的 `webviewInstallMode` 仅对未来安装包有效；便携 EXE 的准备流程在原生入口中实现。

单独浏览器预览：`npm run dev`，访问 `http://127.0.0.1:1420`。浏览器只能预览角色与场景，不能切换原生桌面模式。

## 在 macOS 上运行（Apple 芯片）

目标为 macOS 12+、Apple silicon arm64。构建需要 Node.js 24、Rust 和 Xcode Command Line Tools。无需安装网页依赖。在本目录执行：

```sh
npm ci
rustup target add aarch64-apple-darwin
npm run desktop
```

构建独立 `.app`：

```sh
npm run desktop:macos:build
open 'src-tauri/target/aarch64-apple-darwin/release/bundle/macos/小伴.app'
```

应用使用系统 WKWebView，无需打包浏览器。macOS 配置自动读取 `src-tauri/tauri.macos.conf.json`，开启透明窗口并生成 `.app`，使用本地 ad-hoc 签名；尚未使用 Developer ID 签名或 Apple 公证。透明 WebView 目前使用 Tauri 的 macOS 私有透明接口，不以 Mac App Store 分发为目标。

macOS 悬浮窗口使用 AppKit 浮动层；壁纸使用 Core Graphics 的 desktop window level + 1，位于原桌面背景与桌面图标之间，并忽略鼠标事件以保留 Finder 的操作。角色视线使用 NSEvent 的鼠标逻辑点位置，经过 NSWindow/NSView 转换，处理坐标翻转和 Retina 缩放；壁纸点击只有命中桌面层时才触发，Finder 普通文件夹窗口和其他应用遮挡时不触发。没有安装事件钩子或读取屏幕图像，不主动申请辅助功能和屏幕录制权限。

配置 `CanJoinAllSpaces` 使角色与壁纸参与各桌面空间；AppKit 窗口操作都在主线程执行。macOS 14+ 会关闭 WKWebView 的后台调度限流以保留可见桌宠动画，旧系统的后台表现需要实机验证。

## 使用

“动态壁纸 → 在线壁纸”使用网站相同的搜索、分类和分页接口；图片与动态视频作为第二层类型筛选。点击卡片在当前页右侧查看预览与适配 / 速度设置，不跳回本地页；“保存到我的壁纸”实际下载并校验后可离线使用，不更换桌面（网站提供来源链接时优先直接下载并按真实文件头检查类型，失败再回退到网站副本，见协议文档）；“应用到桌面”独立执行，未保存时先完成下载。视频无图片封面时显示真实首帧，悬停静音播放。内置官方源站 `https://wallpaper.xiangban.xyz`，侧栏底部提供直达官网按钮；可在设置中开启或关闭自动检查，或手动同步版本与公告。原生后台在启动及每天检查一次，控制台关闭时也继续运行；消息在控制台和托盘提示中展示，关闭同一修订后不再重复提醒。修改后的公告和新版本重新提示。配置、消息与已读状态保存在用户目录，网络失败保留已有内容，手动同步显示失败原因。

网站对接协议见[网站接口文档](../../services/server-xiangban-wallpaper/docs/desktop-api.md)，本应用维护自身的类型与解析，不引用网站源码。原生联调测试 `scripts/verify-website.mjs` 使用隔离测试网站和桌面数据目录，验证真实图片 / 视频下载渲染、更新接口与公告已读状态。

在线壁纸目录等 JSON 元数据请求遇到短暂网关错误时，客户端会有限重试，改善首次访问时偶发的瞬时失败。媒体下载仍沿用 120 秒超时及 SHA-256 校验；重试策略不改变媒体传输路径。

桌面控制台使用 Vue 3.5.43、Naive UI 2.45.3、Vite 8 和 Lucide Vue Next。白色侧栏、红色主色（#FF2442）的布局分为壁纸库、桌面特效、桌面小伴、游戏空间、关怀提醒五个主分区，设置入口固定在侧栏底部；外观可在“设置”里选跟随系统、浅色或深色。桌面小伴保留原 SVG、表情和互动；游戏页提供明确的“进入游戏空间”入口，Windows 桌面双击快捷入口在该页独立显示状态并配置。控制台使用 `#2563eb` 强调蓝、`#f5f7fb` 背景与 `#e5eaf1` 边线；游戏空间和关怀气泡采用协调的蓝白浅灰配色。桌面小伴和壁纸页预览见小伴截图（本地：`artifacts/ui/companion-blue.jpg`）及壁纸截图（本地：`artifacts/ui/wallpapers-blue.jpg`）。

开关类偏好即时生效并反馈结果；壁纸在页面内预览，用户确认“应用到桌面”后才更换桌面。关怀提醒通过明确的保存动作提交编辑。控制台入口由 `src/main.ts` 动态载入 `src/control.ts`；`control.ts` 负责挂载 `src/ui/App.vue`，控制台页面、状态与组件分别位于 `src/ui/pages/`、`src/ui/control-store.ts` 和 `src/ui/components/`。它们与宠物窗口、游戏沙箱的入口分离。

动态壁纸支持图片、静音循环 H.264 MP4 与自包含 Canvas/WebGL ES module。可添加 HTTPS 订阅或导入 JSON 清单；下载后的内容校验 SHA-256 并缓存，离线可继续使用。小伴悬浮时可与通用壁纸并存，融入壁纸时叠在所选内容上；“小伴房间”恢复原先的房间场景或系统桌面。下发协议、SDK、限制和更新 JSON 示例见 [delivery-protocol.md](docs/delivery-protocol.md)，Schema 为 [壁纸](docs/wallpapers-v1.schema.json) 和 [更新](docs/updates-v1.schema.json)。

### 本轮已实现能力（1–3，尚未发布）

- **游戏空间**：设置页和托盘/菜单栏的“玩一玩”打开主屏沉浸窗口，离线展示七款本地游戏包。资源协议沿用网页发布包的 manifest 与 v1 消息格式；桌面使用自己的宿主，iframe 仅授予脚本权限并校验消息来源、nonce、游戏 ID 和回合状态。桌面不接账户分享；网页专属分享和钱包入口由本地 adapter 隐藏。只有水墨剑客保存进度，位于用户数据目录 `games/saves/ink-blade.json`，上限 32 KiB；读取或保存失败会保留原文件并提示处理。Windows 原生 smoke 已验证七包 ready、存档往返、Esc、IPC 隔离和桌面状态恢复。桌面空白双击入口仅 Windows 可用，默认关闭；只读低级鼠标钩子将事件交给异步队列并判断 Explorer 桌面列表空白区域，不吞鼠标输入。Windows 真实图标交互尚待人工验收。macOS 暂无可靠桌面空白双击判断，使用显式入口。
- **小伴显示**：小伴设置中可单独关闭角色显示，默认开启。关闭时仍可使用壁纸与图片特效；关怀气泡因要求小伴可见而延后，重新开启后可继续展示有效期内的提醒。
- **天气来源与定位**：关怀提醒的天气来源默认是 Open-Meteo（可在“提醒偏好”里改成自己的来源或关闭）。新建天气提醒时按网络位置预先填好城市：这一步会把公网 IP 发送给 ipwho.is（失败时用 api.ip.sb），只用来判断所在城市，随时可以搜索改成别的城市；来源关闭后不再定位。
- **应用图标**：红底圆角方块，叠放的图片卡片、爱心与白字“小伴”。源文件 `src-tauri/icons/icon-source.png`（1024×1024，四角透明，四周约 10% 留白），`icon.icns/.ico/.png` 由 `npx tauri icon src-tauri/icons/icon-source.png -o <临时目录>` 生成后只复制这三个文件，托盘用同一图标的 64×64 RGBA 原始像素 `icons/tray.rgba`（由生成的 `64x64.png` 解码得到）。原图只有 360×360，大尺寸图标是放大得到的，有更高分辨率的原图时应替换源文件后重新生成。
- **关怀提醒**：控制台提供关怀规则编辑，气泡由独立的 `mountCareBubble` 窗口展示；规则和调度在原生宿主运行，控制台隐藏时仍可触发。下班/自定义按本地时间一次、每天或指定星期调度；休息规则只在平台活跃时间可读、未锁屏且空闲少于 60 秒时累计。提醒存于 `config/care-reminders.json`，支持预览、稍后、免打扰和游戏期间暂缓。Windows care smoke 已检查配置持久化、气泡焦点、关闭/稍后、游戏延后与过期处理。系统通知目前不可用；提醒依赖小伴可见，应用完全退出后不承诺后台触发。天气来源默认是 Open-Meteo（个人非商业使用，可改成自有兼容 HTTPS API 或关闭）；live 天气查询尚未验收。
- **小伴感知本机状态**：原生宿主复用关怀提醒每 5 秒读取的空闲 / 锁屏 / 活跃时间，把结果交给 `src-tauri/src/presence.rs` 的状态机，向小伴窗口发出 `companion-presence` 事件（`welcome_back`、`first_today`、`late_night`），前端 `src/companion/presence.ts` 用现有表情回应，只改变表情，不弹窗、不发声、不显示文字。离开满 10 分钟后重新有操作打招呼，每天最多 3 次、两次间隔至少 30 分钟；本地日期的第一次活动问候一次（日期保存在 `config/presence.json`，读写失败按未问候处理）；00:00–05:00 累计活跃 20 分钟露出困倦表情，每晚一次。遵守关怀提醒的总开关与免打扰时段，小伴隐藏或游戏空间打开期间不触发，且不补发；锁屏或睡眠后恢复只按“离开后回来”处理一次，时钟回拨不会重复触发。不新增采集、不读取用户在做什么、不联网。仅 Windows / macOS 能读取活跃时间，其他平台没有此功能；睡眠恢复后若系统没有报告足够长的空闲时间，则不打招呼。锁屏 / 睡眠恢复尚未实机验证。
- **小伴外壳**：控制台“桌面小伴”页的“外壳”可在猫耳、兔耳、盆栽之间切换（熊耳试过后移除，保存过熊耳的会回到猫耳），立即生效并保存（`Settings.companionSkin`，未知值回退为猫耳；命令 `set_companion_skin` 仅控制台窗口可调用）。外壳在 `src/companion/skins.ts` 里定义，只包含耳朵形状和配色，屏幕、符号位置和部件标记（`data-part`）与原来一致，所以表情、耳朵动作、点击命中和拖动不受影响。这是原型：三款外壳的观感只在浏览器里逐表情看过，没有在桌面上实机看过，也没有逐一核对 25 种表情在兔耳下的耳朵动作；兔耳外壳整体缩到 0.82 倍，给长耳朵留出空间；“盆栽”外壳不是机器人的头：机身是一个白瓷花盆（厚盆沿、盆口露出泥土和小石子、向下收口的盆身，没有托盘），表情屏嵌在盆身正中，一根茎从土中央长出，顶端分成两片叶子。两片叶子沿用耳朵的角度和发光来表达情绪，旋转中心在茎顶（`Skin.earPivot`），所以开心时竖起、难过时耷拉，都绕着茎动。这款在 `skins.ts` 里用 `art` 字段替换了机身轮廓、泥土和茎，其余外壳不用；整体缩到 0.76 倍，给植物留出空间。
- **桌面特效**：控制台的“桌面特效”页选一种特效——雨、雪、萤火虫、花瓣飘落、枫叶飘落、流星划过、繁星，或“无特效”——每种特效各自记住密度、速度、大小三个滑块，选择立即生效并保存。它是桌面上独立的一层透明、鼠标可穿过的窗口（窗口标签 `effects`，页面 `?scene=effects`），不属于任何壁纸：换壁纸、用系统桌面、壁纸是互动动画时都照常显示，也不会随壁纸保存。macOS 上它在动态壁纸窗口上方一级、仍在桌面图标之下；Windows 上它是置于所有应用窗口之下的顶层透明窗口。游戏空间打开期间隐藏，退出后恢复；全局“暂停桌面动画”和帧率同样适用。
- **特效包**：每种特效是一个 JSON 文件（“特效包”），描述有几层粒子、数量、大小、速度、风与摇摆、翻转、闪烁、颜色，甚至形状（SVG 路径），由统一的引擎读取并绘制，包里不能写代码。内置十二个在 `public/effects/`，自己的放进用户数据目录的 `effects/` 文件夹后在页面点“重新载入”，不用重启。格式和限制见 [docs/effects.md](docs/effects.md)，示例见 [docs/effect-pack-example.json](docs/effect-pack-example.json)。引擎在 `src/effects/`（格式与检查 `types.ts`/`parse.ts`、模拟 `sim.ts`、绘制 `draw.ts`、画布 `layer.ts`），窗口与文件夹管理在 `src-tauri/src/effects.rs`、`effect_packs.rs`，选择的数据模型在 `effect_choice.rs`。旧版本保存在每张壁纸里的特效，在首次启动时迁移为这个全局选择（旧的“树叶”变成“枫叶”）。Windows 上的覆盖窗口和 effects smoke（`XIAOBAN_EFFECTS_SMOKE=1`，已改为验证覆盖窗口和全部内置特效）尚未在实机验证。

PRD 第 1–3 项已实现并通过 Windows 自动化检查，尚未发布。专注、玩具互动、场景收藏、昼夜变化和音乐联动（第 4–8 项）仍是规划。真实图标双击、天气 live 查询、网站服务联调、视觉与性能验收仍待完成；逐项结果见 [feature-validation.md](docs/feature-validation.md)。

“导入图片 / 视频”通过 Windows/macOS 原生文件选择器添加自定义壁纸，“我的导入”筛选自己的内容。支持 PNG/JPEG/WebP/GIF/BMP 图片（最多 64 MiB）与 MP4/M4V/WebM 视频（最多 1 GiB），视频推荐 H.264 MP4，其他编码由系统 WebView 决定。导入保存副本并按内容去重，移动原文件后仍能使用；可预览、应用、选择画面适配和调整视频速度，不需要制作 JSON 清单，不内置转码器。

Canvas 运行在独立的低权限壁纸窗口和 opaque-origin iframe 中，不能调用设置、自启动、退出或下载等桌面命令。控制页只展示封面，执行预览使用另一低权限窗口。视频缓存协议支持 byte ranges。更新功能目前为版本/说明检查和打开下载页；未配置更新源会显示未接入，不伪造最新状态，未实现无签名自动安装。

- 控制台切换“悬浮桌宠 / 动态壁纸”，调整 25 种表情、160–420px 尺寸、深夜/晨光背景、30/60 FPS，或暂停动画。
- 悬浮模式：点击角色触发反应，拖动角色移动窗口，右键角色打开控制台。角色实际 SVG 填充区域接收点击，暖光与空白处穿透。
- 壁纸模式：小伴在背景的桌面上，角色上方的桌面点击可以触发反应。应用不拦截桌面鼠标消息，桌面图标继续由 Explorer / Finder 处理。其他窗口遮挡角色时不触发壁纸点击反应。
- 角色视线的鼠标位置由平台原生层以约 30Hz 采样，即使离开角色窗口也能跟随；该采样不使用鼠标/键盘钩子，也不采集键盘输入或屏幕图像。若用户在 Windows 开启“桌面空白双击打开”，游戏入口另使用只读 `WH_MOUSE_LL` 钩子判断桌面点击，不吞输入；默认关闭。macOS 不安装该钩子。
- 40 秒无鼠标活动后困倦，90 秒后睡觉；这是原版逻辑，鼠标活动会唤醒。
- 关闭设置窗口或点“收起设置”只隐藏设置面板，小伴继续运行。Windows 左键点击托盘图标打开设置，右键显示菜单；macOS 点击菜单栏图标后选择“打开小伴设置”，也可以通过 Dock 唤起。
- 托盘/菜单栏和设置面板均有“开机自动启动”开关。首次正式运行默认开启；关闭后保存选择，下次启动不会自动重新开启。登录启动时恢复上次的悬浮/壁纸模式、表情、大小、背景、暂停和帧率，只显示小伴，不弹出设置面板。
- 再次打开程序会唤起已有实例的设置，不创建第二只小伴；“退出小伴”才退出全部窗口，恢复原本可见的壁纸。
- macOS 点击 Dock 中的小伴图标会显示、取消最小化并聚焦设置面板。壁纸模式中同样有效，不会切换角色模式或重建壁纸窗口。

## 桌面组件：日历、便签和纪念日

组件库新增「心事有处」（便签待办）和「好日将至」（倒数日／纪念日），与求签、时钟日历、天气一起沿用多实例、缩放、主题、桌面摆放和本地恢复机制。

- 日历显示农历、传统节日、所选日期详情以及官方「休／班」标识。客户端独立维护 `src/widgets/holidays.json`，目前收录国务院公布的 **2026 年**安排；其他年份明确提示未收录，不推测调休。数据来源链接写在 JSON 中，每年需要随客户端更新。月历扩大为 300×400 基准尺寸以容纳农历；点击月份回到今天。
- 便签可切换文字/待办，标题最多 40 字符，正文最多 1200 字符，最多 12 项待办（每项 120 字符）。文字和标题修改后点击「保存」，也可在正文中按 Ctrl/Cmd+Enter；增删、完成待办和切换视图会保存当前内容。保存失败保留草稿并提示重试。正文区域用于输入和选择文字，按住组件顶栏拖动；摆放模式会锁住内容操作。控制台选中便签后也能编辑。
- 倒数日支持一次性事件、每年重复与纪念日正计时。控制台选中组件后设置名称、日期与模式，再点击保存。年度 2 月 29 日在平年按 2 月 28 日纪念，保留原日期；纪念日正计时不支持年度重复。按本机日期计算天数，跨夏令时也按自然日计算；每 15 秒以及窗口重新可见时更新。
- 数据保存在现有 `config/widgets.json`，旧版三个组件无需迁移。原生窗口只可保存自己的便签/日期内容，不能传入其他组件 ID、样式或布局。布局读取/保存上限从 32 KiB 提高到 128 KiB，保持长度检查及保存失败不覆盖内存状态。新版布局含新类型，旧版客户端不支持读取，降级前应备份数据。

组件支持多屏摆放：拖动时使用原生鼠标位置确定目标显示器，保留抓取点比例；macOS 直接使用 AppKit 全局坐标，Windows 使用物理像素并补偿各屏 DPI。控制台可切换显示器预览，也可通过「所在显示器」直接移动选中组件。每个组件保存显示器标识和屏内相对位置；断开外接屏时显示在主屏，重新接回后恢复原位置（macOS 标识含屏幕名称与布局位置，重新排列或更改名称时可能回退到主屏，可重新选择）。显示器配置每 3 秒检查一次，仅在变化时重新摆放。此能力只扩展组件，不扩展主屏壁纸。

新闻和 A 股组件尚未实现，候选数据源、三大指数/自选股布局和刷新建议见 [资讯组件调研](docs/information-widgets-research.md)。

本轮针对性验证：`npx vue-tsc --noEmit`、`npx vitest run src/widgets`。新增 Rust 校验/权限回归用例，当前环境没有 Rust 工具链，未运行原生构建或 Windows/macOS 实机验收；需验证桌面输入焦点与文件持久化。浏览器预览环境缺少 Chromium，未完成截图审稿。

## 验证

正式用户数据固定在 `~/.xiaobanwallpaper`。`config/preferences.json` 保存设置，`wallpapers/imports` 保存自定义媒体的持久副本，`wallpapers/local.json` 保存本地壁纸元数据，`wallpapers/cache` 保存订阅资源，`cache/webview` 是 Windows 浏览器运行数据。“关于小伴”提供打开目录按钮；首次迁移复制旧配置与缓存、保留原数据和备份，不覆盖已有新配置。完整子目录职责和迁移约束见 [user-data.md](docs/user-data.md)。设置通过临时文件替换保存。Windows 使用当前用户 `Run` 启动项 `XiaoBanDesktop`，兼容带空格/中文的 EXE 路径，并识别任务管理器中的禁用状态；macOS 使用 `~/Library/LaunchAgents/XiaoBanDesktop.plist`，随用户登录启动。便携 EXE 或 `.app` 移动后，手动打开一次可刷新已启用的启动路径；macOS 建议放到 Applications，避免下载目录或临时路径失效。登记失败会在设置中提示，不阻止桌宠运行。

开发预览不自动登记自启动，使用 `.xiaobanwallpaper/development` 独立数据根。原生 smoke test 使用独立应用标识、临时数据根与 `XiaoBanStartupSmoke-<测试标识>` 启动项；Rust 注册测试使用 `XiaoBanStartupTest-<PID>`。结束时仅清理自己的测试项，不修改正式启动项。两端媒体测试额外导入真实 PNG 与超过 16 MiB 的 MP4，删除测试源文件后验证播放、去重与重新加载；不读取用户图片或视频。

```powershell
npm run build
npm test
powershell -ExecutionPolicy Bypass -File .\scripts\smoke-test.ps1
cargo build --manifest-path src-tauri/Cargo.toml --locked
powershell -ExecutionPolicy Bypass -File .\scripts\smoke-runtime.ps1 -VerifyDownload
node scripts/verify-media.mjs
```

角色测试运行本工程的 12 个行为测试，独立于网页与后端业务目录。原生 smoke test 启动生产程序，在悬浮、壁纸、返回悬浮三个阶段检查窗口存在性、前端显示、背景父窗口类别、WS_CHILD 标记和旧窗口清理，输出 `artifacts/smoke-*.json`。Windows 的小伴房间与六种媒体壁纸用例同时检查实际客户区的物理尺寸和屏幕原点，要求与主显示器完全一致；壁纸窗口关闭系统阴影，避免无标题栏窗口的边框占用绘图区。桌面 parenting 使用 Explorer 的非公开 WorkerW 协议，需要在不同 Windows 版本上继续验证。

macOS 验证：

```sh
cargo test --manifest-path src-tauri/Cargo.toml --locked
npm test
npm run desktop:macos:build
npm run test:macos
```

四个坐标/层级测试覆盖上下原点、逻辑点、非零 bounds、窗口外鼠标和普通应用窗口遮挡判断。macOS 原生 smoke test 等待真实前端挂载握手，验证透明浮动窗口、壁纸层级、输入穿透、桌面空间配置、两次模式切换与旧窗口清理，输出 `artifacts/smoke-macos.json`。GitHub 工作流 `.github/workflows/xiaoban-desktop.yml` 在 Apple 芯片的 macOS 15 构建机上执行这些检查，并生成包含 `.app` 的 ZIP 和包体积记录。

Dock 回归通过 LaunchServices 真正重新打开已运行的 `.app`，分别验证设置面板最小化、面板隐藏、整个应用隐藏时都能恢复焦点，同时保留壁纸窗口与壁纸模式。测试同时核对壁纸原点和尺寸与主屏幕对齐。

Mac 实机还需检查：原角色的发光与连续表情、桌宠拖动、空白处点击穿透、壁纸上的角色点击、桌面图标操作、桌面空间切换和全屏应用行为。CI 的原生窗口属性验证不能代替这些视觉与操作检查。

## 当前范围与限制

新增功能规划见 [小伴桌面 PRD](../../docs/product/PRD_小伴桌面.md)。本轮第 1–3 项已实现并通过 Windows 自动化检查，尚未发布；第 4–8 项仍规划中。实际能力及平台限制以本文和代码为准。

启动流程另有四个 Rust 测试，覆盖已安装时不调用安装器、安装后重新检测、虚假成功退出码与失败重试。Windows CI 构建便携包并执行原生准备窗口成功/失败/取消模拟；本地 `-VerifyDownload` 还验证内嵌下载工具、进程清理及真实微软签名，不执行安装。真实缺组件的首次安装需在干净 Windows 虚拟机上验证。

- 提供 Windows 与 macOS 的原生接入；macOS 包面向 Apple 芯片，Intel / Universal 打包未纳入此原型。
- 壁纸只覆盖主显示器；未实现多显示器管理、视频转码或签名安装包。
- Windows 壁纸窗口作为 Explorer 背景 WorkerW 子窗口；macOS 壁纸使用 AppKit 桌面层窗口。未修改系统壁纸路径或偏好，退出后原壁纸恢复显示。
- 模式切换会重新挂载角色，当前的随机动画/睡眠进度会重置。
- Explorer 重启、显示器热插拔、macOS Stage Manager / 全屏空间等恢复策略需要进一步实机完善。Windows 背景父窗口异常会提示重新切换模式。
- 点击穿透通过原生采样与实时 SVG 轮廓判断切换，存在约一个采样周期的延迟。
- 这是还原路线验证原型，未对跨浏览器逐像素一致性作保证。

## 代码

- `src/companion`：本应用角色引擎、SVG 和测试；`src/pet.ts`：DOM 帧更新与 SVG 轮廓命中测试。
- `src/scene.css`：原小伴房间的程序背景；内置图片、视频和 Canvas 样例位于 public/wallpapers。
- `src-tauri/src/main.rs`：控制台、两种窗口生命周期、托盘、命令与诊断。
- `src-tauri/src/preferences.rs`：本应用设置、兼容旧字段、参数修复与文件保存；`src-tauri/src/startup.rs`：Windows 当前用户注册、macOS 登录启动与测试项清理。
- `src-tauri/src/storage.rs`：正式/开发/测试数据目录、旧数据迁移与备份；`src-tauri/src/local_media.rs`：原生选择器、媒体格式检查、流式复制、去重与本地目录元数据。持久导入副本与可重新下载的缓存不能混用。
- `src/main.ts` 按窗口场景动态载入入口；`src/control.ts` 仅负责控制台 Vue 应用挂载。`src/ui/ControlShell.vue`、`src/ui/pages/`、`src/ui/components/`、`src/ui/control-store.ts`：控制台布局、分区页面、展示组件与状态；`src/ui/wallpaper/`：壁纸页面状态逻辑；`src/ui/website-types.ts`：网站 API 类型；`src/ui/control.css` 和 `src/ui/theme.ts`：控制台主题；`src/style.css`：宠物、游戏和气泡等原生场景宿主样式；`src/wallpaper-media.ts`：视频与图片预览、首帧回退和可见卡片加载；`src-tauri/src/website.rs`：网站原生 HTTP、后台检查与公告已读状态；`src/content.ts` 与 `public/wallpapers/sandbox.html`：图片/视频与 Canvas 沙箱宿主；`src-tauri/src/library.rs`：目录、资源完整性、缓存协议和更新验证；`src-tauri/src/content.rs`：窗口切换、权限边界和交付命令。
- `src-tauri/src/runtime`：Windows 启动前检测、原生准备窗口、官方引导程序下载与签名验证。成功/失败/取消的 UI 模拟仅在 debug 构建中启用，不会安装组件；真实缺组件的首次安装还需干净虚拟机验证。
- `src-tauri/src/windows.rs`：WorkerW 接入、DPI 坐标转换、只读鼠标采样。
- `src-tauri/src/macos.rs`：AppKit 窗口层级、空间行为、鼠标逻辑点与桌面点击判断。
- `src-tauri/src/coordinates.rs`：坐标转换和桌面层判断的独立测试。

## 大仓开发边界

本项目是桌面客户端，不承担网页、账户或后端业务。角色可以有自己的实现，修改角色行为时运行本地 npm test；不引用 client/xiaban-web 或主后端的私有文件。两种窗口模式共用本应用场景，Windows/macOS 接入分文件。新增能力按本次需求，当前无聊天/账号/云端依赖。

品牌插画使用 `public/xiaoban.svg`，由本工程原有猫脸角色 `robotMarkup(stillFrame("idle"))` 导出；网站维护一份相同的静态 SVG，不导入客户端源码。

