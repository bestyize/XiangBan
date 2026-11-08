# 应用广场与运行时协议 2：实施进度

规范见 [README.md](README.md)。本文记录实施进度，每完成一步就更新。「完成」只表示表里写明的验证已经做过，没写的就是没验证。

第一批已经合进 `main`（PR [#143](https://github.com/bestyize/XiangBan/pull/143)，合并提交 `7e7142b8`）。iOS 暂缓（2026-11-08 决定）。

第二批（精选工具、创作者精选、两个真实案例）已合入 `main`（合并提交 `c65beafa`）并推送到远端；线上论坛运行的 `684b5cf7` 现已进入远端主分支。

## 分工

| 工作 | 分支（worktree） | 执行 | 状态 |
| --- | --- | --- | --- |
| 规范、原型、给 AI 的规则 | `feat/app-plaza-runtime` | Claude Fable 5 | 已提交（`935174c9`）；下面三项已合入这个分支 |
| 手机端应用广场界面（Compose） | `feat/app-plaza-ui` | Codex gpt-6.1-sol | 已在真机上验证并合并（`56f1e86e`），审查时补了 2 个修复 |
| Android 运行时协议 2 小样 | `spike/android-runtime-v2` | Codex gpt-6.1-terra | 13 项全部在真机上通过，已合并（`2b192fe6`），审查时补了 2 个修复 |
| 服务端 `app` 类型与协议 2 字段 | `feat/forum-app-kind` | Claude Sonnet 5.5 | 已审查并合并（`dcad0aa3`），2026-11-08 已部署到香港机 |
| 审查、真机验证（小米 14）、合并 | — | Claude Fable 5 | 第一批完成 |
| 官方 MBTI 性格测评 | `feat/mbti-pack`，已合入 `fix/app-plaza-polish` | Codex gpt-6.1-sol | 浏览器里走过流程；手机上看过开始页和第一题 |
| 两个社区案例《墨渡》《茶签归位》 | `feat/community-cases`，已合入 `fix/app-plaza-polish` | Codex gpt-6.1-terra | 浏览器里各玩过；已登记到线上（编号 56 和 57）；手机上只从创作者精选点开过《墨渡》并进到游戏界面，《茶签归位》没在手机上打开过 |
| 创作者精选的服务端标记 | `feat/forum-featured`，已合入 `fix/app-plaza-polish` | Claude Sonnet 5.5 | 已审查，已部署 |
| 桌面端独立「应用」模块与 V2 容器 | `feat/desktop-app-runtime-v2` | — | 架构方案已敲定，待开工 |

## 第一批范围

**手机端广场界面**
- 底栏「应用」。
- 广场：最近用过、精选游戏、社区作品瀑布流。
- 我的作品。
- 社区作品帖子页，官方游戏详情。
- 去掉审核状态的显示。
- 小伴AI 首页下拉打开「最近用过 / 收藏」。
- 工具与测评、创作者精选要等服务端有数据，先只留位置。
- 只用协议 1 的运行能力。

**Android 运行时小样**
- 一个 Debug 专用页面，验证规范第 15 节的前提：
  - 每个作品一个 https 源，从本机缓存提供文件
  - 页面加载前注入 SDK，用新的消息通道收发消息
  - 标准存储能持久保存
  - ES modules、Worker、WebAssembly 能用
  - 摄像头和麦克风的权限流程
  - 用历史记录实现返回
  - `show` / `hide` 事件
- 交付一份验证报告：`spike-android.md`。

**服务端**
- 新增 `app` 类型；`game` 和 `app` 的上限提高到 100 MB。
- 校验协议 2 的清单。
- 新字段：协议版本、是否允许改编、改编来源、改编数、禁止联网。
- 改编作品不出现在主信息流里。
- 按 `X-Xiaoban-Runtime` 请求头过滤作品。
- 后台的禁止联网开关。
- 不做：`app.xiangban.xyz` 文件服务、域名封禁列表。

## 之后的批次

1. Android 正式宿主：内置测试包已验证协议分流、`xb` 基础接口、胶囊、相机、麦克风及网络限制；社区协议 2 包已接入，并用本地论坛在真机上验证。大包真机与生产发布待办。
2. `app.xiangban.xyz` 文件服务和 SDK 注入；通配证书；申请加入公共后缀列表（需要产品负责人操作）。
3. 手机 Agent 新工具：`app_*`、`lib_add`、`asset_image`、`app_publish`；工作区支持二进制文件。
4. 广场接入改编列表（精选工具和创作者精选已在 2026-11-08 下午接入，见记录）。
5. 网页端（已完成基础宿主代码与存储测试，待独立域名通配证书与线上基建）。
6. 桌面端应用模块与协议 2 独立容器（已确认技术方案）：
   - **架构解耦**：将「应用（App Plaza / 游戏 / 工具 / 测评）」与「壁纸 / 桌宠 / 特效」装饰体系彻底解耦。启动应用不暂停壁纸、不销毁桌宠，运行在独立原生沙箱窗口中；
   - **Tauri V2 容器**：实现专有 V2 Webview 宿主，注入标准全局 `window.xb`（对齐协议 2），配备统一右上角胶囊菜单与窗口控制栏；
   - **本地沙箱与存储**：使用 `apps/storage/<id>.json`（单应用 10 MB 上限）实现离线本地存储；彻底淘汰桌面旧版协议 1（`xiaban.game-pack`）；
   - **控制台界面重构**：侧边栏「游玩」升级为「应用」大厅，支持本地与社区 V2 作品一键运行；
   - **作品迁移**：内置 10 款游戏升级为 `runtimeProtocol: 2`，无缝拉通三端作品生态。

## 记录

- 2026-11-08：规范、原型和给 AI 的规则提交为基线（`935174c9`）。确认本机 Codex CLI 能调用 gpt-6.1-sol 和 gpt-6.1-terra，沙盒打开网络后能访问 Google 和 Maven 仓库。小米 14 通过 ADB 连接，WebView 156。三个任务已派出。
- 2026-11-08：服务端第一批完成。
  - 新增 `app` 类型；`game` 和 `app` 的上限提高到 100 MB。
  - 校验协议 2 的清单（`runtimeProtocol: 2`，`schemaVersion: 3`）。
  - 作品详情和列表返回新字段：`runtime_protocol`、`allow_remix`、`remix_of`、`remix_count`、`network_disabled`。
  - 改编作品不进主信息流，用 `?remix_of=<id>` 单独列出；原作删除后保留原作者署名。
  - 按 `X-Xiaoban-Runtime` 请求头过滤作品，旧版客户端看不到协议 2 的作品。
  - 后台接口 `POST /api/admin/packs/{id}/network`，可以禁止或恢复某个作品联网，会写审计记录。
  - 确认审核等级不影响 `game` 和 `app` 的列表、详情和下载。
  - 审查时我在隔离库里重跑了完整测试：209 个通过，5 个失败。这 5 个在基线 `1bb124b4` 上也失败（三个版块种子数据测试、`discover_tests::topics_carry_images_polls_and_help_stats`、`points::tests::purchases_are_atomic_idempotent_and_keep_read_permissions`），和本次改动无关。第一次跑时 cargo 用了旧的编译产物，把源码 touch 一下才编进新测试。
  - 遗留问题：
    - 没有 `create-app` 讨论版块，`app` 作品可能建不了讨论帖，还没测。
    - 文件服务 `services/server-resource` 自己的大小上限没改。
    - 改编作品没有校验 `kind` 必须和原作一致。
    - 下载接口不按 `X-Xiaoban-Runtime` 过滤。
    - 协议 2 的 `id` 仍然沿用 1～64 位的规则，规范写的是 2～64 位，两边要统一。
- 2026-11-08：Android 运行时小样完成，在小米 14 上验证（Android 16，系统 WebView 156），详细结果见 [spike-android.md](spike-android.md)。
  - 通过：
    - 安全上下文，源是 `https://spike1.app.xiangban.xyz`
    - localStorage、IndexedDB、OPFS；localStorage 在进程被强制停止后数据还在
    - ES modules、Worker、WebAssembly
    - 公网 fetch 能用，局域网 fetch 被拦截
    - 页面加载前注入 `xb`，安全区数值正确
    - `xb.storage` 3 MB 往返用时 373 毫秒，强制停止后数据还在
    - `hide` / `show` 事件
    - 用历史记录实现返回：逐层回退，回到首页后第一次提示、第二次退出
  - 发现的问题：Android 16 上 targetSdk 37 的 App 收不到 `onBackPressed`，已改为 `OnBackInvokedCallback`（`e4583354`）。现有的 `MobileGameActivity` 也要检查这个问题。
  - 还没测：摄像头和麦克风（需要人在手机上点系统权限弹窗）、100 MB 包、发往局域网的 WebSocket。
- 2026-11-08：广场界面完成。
  - Codex 沙盒写不了 worktree 的 git 元数据，没法提交，改由我按功能分成 6 个提交。
  - 编译通过；Codex 报告 community-core 的 326 个 JVM 测试全部通过。
  - 在小米 14 上看过广场首屏：底栏「应用」，精选游戏的「官方」标记在封面右下角。
  - 线上 `chat.xiangban.xyz/api/packs` 现在一个作品都没有，所以社区作品显示「还没有社区作品」。准备在本机起论坛并放入测试作品，用来看瀑布流和帖子页，但**本机磁盘满了**（数据卷只剩约 2.7 GB，Rust 链接时用完），停在这一步。我已删掉自己这次产生的 Android 中间编译产物和 Rust 增量缓存。
- 2026-11-08：第一批验证完成并合并。
  - **广场界面真机验证**：本机用 `feat/forum-app-kind` 起了论坛（单独的测试库 `forum_plaza_qa`），放入 6 个测试社区作品，用 `adb reverse` 让手机连本地论坛。小米 14 上看过并通过：
    - 社区作品两列瀑布流，竖屏和横屏封面比例都对，有作者圆形头像、玩过人数和点赞数，「推荐 / 最新 / 游戏 / 工具」分类可以切换。
    - 竖屏（像素钓鱼）和横屏（弹弹球）的帖子页：主按钮分别是「进入」和「横屏进入」，有「做一个我的版本」弹层，按返回键能关掉弹层。
    - 官方游戏详情弹层：收藏后变成「已收藏」。
    - 小伴AI 首页下拉打开「最近用过 / 我的收藏」，刚收藏的水墨剑客出现在里面；「去应用广场」能切到应用页。
  - 审查时修了 2 处：
    - 帖子页的评论标题重复出现了两次（`af8233f5`）。
    - 首页抽屉固定占屏幕 80%，下面空了一大块，改成按内容自适应（`1d022b6a`）。
  - **运行时小样的摄像头和麦克风**：一开始被拒绝。原因是 `PermissionRequest.origin` 结尾带斜杠，和作品源比较时不相等，改为比较协议和主机名后通过（`7624ffcc`）。测试时用 `pm grant` 临时授予系统权限，测完已经收回；系统权限弹窗本身没有经过。13 项全部通过。
  - **合并**：三个分支依次合入 `feat/app-plaza-runtime`，没有冲突。合并后的代码：`:androidApp:assembleDebug` 通过；community-core 的 JVM 测试 326 个全部通过；小样的单元测试 3 个通过。服务端代码和它的分支完全一样（diff 为 0 行），所以没有再跑一遍服务端测试。合并后的 APK 连线上服务时，广场能正常打开，没有崩溃；线上还没有社区作品，所以显示「还没有社区作品」。
  - 没有部署。测试期间改过的手机设置（USB 连接时保持常亮、ADB 端口转发）都已恢复。
- 2026-11-08：`feat/app-plaza-runtime` 推送到 GitHub（`origin/feat/app-plaza-runtime`，推送时最新提交是 `feb0e1cf`），还没合进 `main`，也没有开 PR。
- 2026-11-08：PR #143 合进 `main`，合并提交 `7e7142b8`。用合并提交的方式保留了分步提交的历史。删除了三个 worktree 和已合并的本地分支，远端的 `feat/app-plaza-runtime` 分支还保留着。服务端还没部署。
- 2026-11-08：服务端部署到香港机（`chat.xiangban.xyz`，systemd `server-forum`）。
  - 用的是从 `main` 的 `c4790cd0` 在本机交叉编译的二进制（sha256 前 16 位 `eacbf4017c10e15e`），服务器上的 `DEPLOYED_COMMIT` 已改成 `c4790cd0`。
  - 网页前端没有变化（从已部署的 `723ca5dd` 到现在，`web/` 下没有文件改动），所以没有重新发布前端。
  - 部署前做了备份：数据库 `backups/xiaban_forum-before-appplaza-20261107T204213Z.sql.gz`（58 MB，包含 224 张表，导出完整）；旧二进制 `backups/server-forum.before-appplaza-20261107T204213Z`（`8a4dddad…`）。
  - 数据库加了 6 个字段：`pack_versions.runtime_protocol`，以及 `packs` 表的 `allow_remix`、`remix_of_pack_id`、`remix_of_version`、`remix_of_author_uid`、`network_disabled`。都是追加的、带默认值的字段，旧二进制照样能跑，回滚只需要换回旧二进制，不用动数据库。`data_migrations` 没有新增记录（仍是 21 条）。
  - 冒烟测试：
    - `/`、`/sitemap.xml`、`/robots.txt`、`/api/discover/home`、`/api/categories`、`/api/topics` 都返回 200。
    - `?kind=app` 和带 `X-Xiaoban-Runtime: 1,2` 请求头的列表都能正常返回（现在都是 0 条）。
    - 后台禁止联网接口不带登录返回 401；用伪造令牌申请 `app` 上传凭证返回 401。
    - 线上唯一一个已发布的游戏（id 55）部署前后都是 404（作者被封禁或讨论帖不可见），没有变化。
    - 日志里没有新的报错。
  - **部署过程中有一次误操作**：上传新二进制后，我想用 `--help` 检查它能不能运行，但这个程序不处理命令行参数，会直接按正常方式启动。它用线上配置跑完了数据库迁移，加上了上面那 6 个字段，然后因为端口被旧服务占着，启动失败退出。这一步发生在数据库备份之后，加字段的效果和正式重启时完全一样，`data_migrations` 和后台审计记录都没有变化。但新代码在正式切换前已经连过一次线上数据库，后台任务也可能短暂启动过。以后检查二进制只看 `file` 和 `ldd`，不要直接运行它。
  - 还没做的：
    - 文件服务 `services/server-resource` 自己的大小上限没改，超过它上限的 `app` / `game` 包传不上去。
    - 网页社区试玩仍然只运行审核等级为 0 或 1 的作品，包大小上限也还是 5 MB。
    - 新版手机 App 还没有发布。
- 2026-11-08（第二批）：精选工具、创作者精选和两个真实案例。手机端已经能看到前两者，两个案例还差最后一步登记。
  - 手机端（分支 `fix/app-plaza-polish`，未推送）：
    - 对照设计稿修了四处：精选游戏卡片的文字不再被截断；去掉多余的「最近用过」标题；精选游戏右上角加了「全部」，点开是三列封面的全部页，可按竖屏、横屏筛选；首页下拉面板底部改成一条分隔线加居中的「去应用广场」。
    - 「工具与测评」改名「精选工具」（App、原型和设计稿说明一致），一行四个图标，「全部」进四列图标页。官方内置的 MBTI 放在这里，图标是衬线体的「格」字。
    - 「创作者精选」读取 `GET /api/packs?featured=true`，没有数据时整段不显示；「工具」筛选读取 `kind=app`。
  - MBTI 性格测评（Codex gpt-6.1-sol，在 `client/xiaoban-desktop/public/game-packs/mbti/`，`assets/game-packs/mbti/` 有完全一致的副本）：48 道原创题，四个维度各 12 题，两个选项的方向各占一半；16 种类型各有一段 176～191 字的解读，外加 3 条工作方式和 3 条盲点；每答一题就存进度。
    - 已验证：我用脚本核对了计分（全选前一极得 ESTJ、全选后一极得 INFP、没答完的维度不给类型）以及解读的字数和条数；在手机尺寸的浏览器里看过开始页并答了几题；在小米 14 上从「精选工具」点开，看到开始页和第一题。
    - 没有验证：手机上中途退出再继续、答完看结果（按你的要求没有逐项测）。桌面的两个资源检查脚本通过，但它们只检查桌面目录里登记的 9 款游戏，不包含 MBTI。
  - 两个案例（Codex gpt-6.1-terra，在 `services/server-forum/content/app-plaza-cases-20261108/`）：《墨渡》（创作者精选，竖屏，按住蓄力、松手跳到下一处落脚点，有风向和连击）和《茶签归位》（社区作品，竖屏，20 关）。我在手机尺寸的浏览器里各玩了一次：茶签归位用游戏自己的点击逻辑通过了第 2 关，墨渡按住松开后得分、落水后出现结算，控制台都没有报错；20 关用独立的求解程序验证了全部有解（最少 4～15 步）。手机上只确认了《墨渡》能从线上的创作者精选点开、进入游戏容器并看到开始页，没有真正玩一局；《茶签归位》没在手机上打开过。
  - 服务端：新增创作者精选标记（`packs.featured_at`、`?featured=true`、`POST /api/admin/packs/{id}/featured`，5 个提交）；管理员登记接口可以带 `author_uid`，让作品署名为某个已有用户（`f9bd372f`）。完整测试 211 个通过、5 个失败，是改动之前就有的那 5 个。
  - 部署（2026-11-08 09:33，香港机）：
    - 上线的二进制 sha256 前 16 位是 `7afe7a78bc6b8552`，由提交 `684b5cf7` 在本机交叉编译，服务器上的 `DEPLOYED_COMMIT` 写的是 `684b5cf7`。这个提交已随分支合入 `main` 并推送到 GitHub。
    - 部署前备份：数据库 `backups/xiaban_forum-before-plazacases-20261108T013043Z.sql.gz`（58 MB，导出完整）；旧二进制 `backups/server-forum.before-plazacases-20261108T013043Z`（`eacbf401…`）。新增的只有 `packs.featured_at` 一个可以为空的字段，回滚只需换回旧二进制。
    - 冒烟：首页、站点地图、发现页、分类、帖子、作品列表（含 `?featured=true` 和 `kind=game`）都返回 200，重启后日志没有报错；不带登录调用登记和精选接口返回 401。
    - 两个 ZIP 和两张封面已上传到资源机，哈希与本机一致；封面的公开地址返回 200，ZIP 不带签名返回 403（符合预期）。
  - 登记作品（已完成）：先用论坛 `.env` 里的管理员密码登录，返回了 401。那份密码只是创建账号时的初始值，账号 `admin` 在 11 月 5 日登录过，之后密码已被改过；我只试了一次就停下，没有绕过管理员验证去改数据库。改为在终端标签页里运行发布脚本（`--ask-password`），由你亲自输入用户名和密码，登记和设置精选一次完成：《墨渡》编号 56（创作者精选），《茶签归位》编号 57，作者都是「一瞬流光」，信任等级 1。线上核对：作品列表共 2 个，`?featured=true` 只有《墨渡》；两个作品用签名地址下载的哈希都与本机 ZIP 一致；封面公开地址返回 200，ZIP 不带签名返回 403。服务器 `/tmp` 里临时放的 `register.json` 和 `publish_remote.py` 已删除。脚本在 `services/server-forum/content/app-plaza-cases-20261108/scripts/`，可重复运行，已登记的不会重复登记。
  - 测试：手机端最后一次完整跑 community-core，326 个里有 1 个失败，是 `MobileAgentExtensionsTest`（里面有 3 秒超时，机器忙时偶发）。它在这个分支上单独跑 4 次失败过 1 次，在 `main` 上单独跑 2 次都通过，这批改动没有碰到智能体运行时。更早的一次整套里 `ChatHistoryTest` 也失败过 1 次，单独重跑通过。最后加「格」字图标之后，只重跑了 `AppPlazaTest` 并编译了 APK，没有再跑整套。
  - 待办：桌面端登记 MBTI（`src-tauri/src/games.rs` 的 `IDS` 现在只有 9 款，`catalog.json` 里没有 `mbti`，基准副本检查也要带上）；`server-resource` 的大小上限；发布新版手机 App；《茶签归位》和 MBTI 的完整一局在手机上还没玩过（按你的要求没有逐项测）。
- 2026-11-08：第二批（分支 `fix/app-plaza-polish`）合入 `main`（合并提交 `c65beafa`），并推送到 `origin/main`；远端保留了 `origin/fix/app-plaza-polish`，删除了已合并的本地分支。线上论坛部署的提交 `684b5cf7` 现已进入主分支。
- 2026-11-08：Android 正式宿主第一步在小米 14 真机通过（Android 16，WebView 156）。
  - 正常入口「应用 → 精选游戏 → 全部 → 协议 2 宿主测试 → 打开」。内置作品按清单的 `runtimeProtocol` 分流，字段缺失按协议 1 处理。协议 2 进入正式 `RuntimeV2Activity`，测试包资源只进入 Debug 构建；现有协议 1 继续走 `MobileGameActivity` 的离线 iframe。
  - 真机确认测试包源为 `https://local-runtime-v2-fixture.app.xiangban.xyz`，是安全上下文，首段页面脚本已能读到 `xb`。同源资源可读，外网请求被拦截。本步只加载内置资源，社区协议 2 下载与正式联网策略留待后续。
  - `xb.storage` 写入、读取、列键、删除和 4 MiB 字符串往返通过；进程强制停止后从广场重开仍能读回写入值。Home 切到后台再从任务切换器回来，收到 `hide` / `show`；展开和收起宿主菜单也收到对应事件。
  - 对照[应用广场设计稿](../../../client/xiaoban-multiplatform/docs/design-specs/prototypes/app-plaza.html)的 b7/b8 画板，核对竖屏胶囊与底部菜单、横屏胶囊与 440 dp 居中菜单。实机返回顺序为二级 → 一级 → 首页 → 展开菜单并提示「再返回一次也会退出」→ 退出；胶囊 ✕、菜单外点击收起和「重新开始」也通过。收藏、分享、反馈在测试菜单中保留设计稿位置，交互待接入。
  - 协议 1 回归：MBTI 从精选工具打开并恢复已有答题进度；《墨渡》（56）从广场打开后完成一次有效蓄力跳跃；《茶签归位》（57）打开第 1 关、移动一签并撤销。三者均确认在原 `allow-scripts` iframe 内运行。
  - 本地验证：`:androidApp:assembleDebug` 和 `:androidApp:compileReleaseKotlin` 通过；协议 2 路径、清单及小样请求测试共 6 项通过；community-core 的 `GameLibraryTest`、`MobileGamesTest`、`CommunityGameTrialTest`、`AppPlazaTest` 共 22 项通过。系统权限弹窗、后台断网开关、局域网 WebSocket 和 100 MB 社区包仍待专项验证。
- 2026-11-08：协议 2 媒体授权和网络策略通过小米 14 真机验证。
  - 相机、麦克风分别显示作品同意框，再显示 Android 系统权限弹窗。均点了「本次使用允许」，网页取得 `live` 视频轨与音频轨，使用时胶囊亮提示点。麦克风另测了在作品同意框点「不允许」，网页收到 `NotAllowedError`。作品同意按作品与能力保存，设置里的逐作品撤销入口待接入。
  - 协议 2 WebView 使用独立进程及独立数据目录，HTTPS/WSS 都经本机 CONNECT 代理。代理检查域名与解析结果，只连接已检查的公网地址。联网时公网 HTTPS fetch 与 WSS 握手通过；同一 Wi-Fi 下用手机直连探针证明局域网监听端口可达，作品对该端口的 HTTPS fetch 与 WSS 均失败，监听端没有收到额外连接。代理同时记录了局域网与 `127.0.0.1` WSS 的拒绝。WebRTC 仍按规范尽力限制，未验证其他 UDP 或系统路由的边界。
  - 用 Debug 测试包模拟 `network_disabled=true` 后从广场重开，包内文件仍返回 200，公网 HTTPS 与 WSS 均失败。真实论坛返回的字段将在社区包接入后走同一路径验证。本次没有修改线上作品或后台开关。
  - `:androidApp:assembleDebug` 和 `:androidApp:testDebugUnitTest --tests '*RuntimeV2*'` 通过，含 3 项新代理测试。
- 2026-11-08：Android 接入社区协议 2 包。
  - WebView 支持协议 2 的设备请求作品时带 `X-Xiaoban-Runtime: 1,2`。从作品详情打开时重新取上架状态、指定版本及 `network_disabled`。按版本清单分流，`app` 与 `game` 的协议 2 包进正式宿主，协议 1 游戏保留原入口与下载限制。缺失断网字段时按断网处理。
  - 下载请求不携带论坛账号令牌，禁止重定向，支持签名地址的 Range 续传。校验大小与 SHA-256 后流式解压到暂存目录，检查清单、资源路径、CRC、文件数和展开体积，再原子发布到私有缓存。Android 实现 ZIP 和展开合计各 100 MiB 的上限，移动网络超过 20 MiB 且无缓存时先确认。
  - 小米 14 通过 Debug 本地论坛预览从广场打开协议 2 工具，确认 `https://16002.app.xiangban.xyz` 安全上下文和首段脚本前注入。切换本地论坛响应的 `network_disabled` 再打开，同一 ZIP 缓存只下载一次，断网时包内文件和存档可读，公网 HTTPS 与 WSS 失败；解除断网后公网 HTTPS 恢复。菜单按设计稿显示社区作者和「举报」位置。
  - `:androidApp:assembleDebug`、`:androidApp:compileReleaseKotlin` 和 28 项针对性测试通过。100 MB 包、实际断点续传及移动网络确认框尚未真机验收；线上论坛与资源服务未改动，生产资源上传上限仍待配套。
- 2026-11-08：Android 的 MBTI 和 9 款内置游戏全部迁入协议 2。
  - Android 构建从共享玩法资源生成 10 个 schema 3 包，各有方形图标、16:9 封面和独立作品源。支持新宿主的 WebView 从广场默认打开协议 2 包；桌面和旧 WebView 继续使用协议 1 兼容包。
  - 内置作品通过包内适配代码直接调用 `xb.ready`、`xb.game`、`xb.exit` 和 `xb.storage`。MBTI 与《水墨剑客》通过 `xb.info.legacySave` 继承本机旧档，并写入各自的协议 2 存储。应用内的返回键与暂停行为随作品保留，运行态胶囊沿用设计稿，作品控件按实际安全区避让。
  - 小米 14 从「精选工具」及「精选游戏 → 全部」逐一打开 10 款作品，确认独立 HTTPS 源、协议 2 就绪、首屏按钮未压到胶囊，系统返回能退出。MBTI 仍显示已有答题进度，《水墨剑客》恢复原关卡进度，两份迁移后的存档与原数据一致。
  - Android Debug 构建、Release Kotlin 编译、11 项 Android/JVM 针对性测试、协议 1 宿主脚本和桌面资源校验通过。桌面协议、目录、魔方和《水墨剑客》存档相关测试共 15 项通过。
  - 补充胶囊避让。MBTI 的滚动区域固定在胶囊下方，结果页滚动时「返回首页」不会进入胶囊触控区。魔方标题栏和竖屏 HUD 复用安全区与胶囊计算出的上边距。浏览器模拟 320、360 CSS px 的竖屏及对应横屏尺寸，检查了十款首屏、MBTI 滚动和三款横屏动作游戏的对局、暂停、结算布局。Android Debug 构建通过。
  - 小米 14 的《水墨剑客》「烽火卷」选关页发现「江湖」按钮被胶囊挡住。选关标题栏现在按胶囊右边距和宽度留空，窄屏时难度选项换行。更新 Debug APK 后在同一选关页确认两个按钮的中心都可点击，最右按钮和胶囊之间相隔 8 CSS px。另用三组横屏尺寸和两侧安全区模拟复测。
  - Android 协议 2 内置游戏隐藏自带的左上角返回按钮，竖屏游戏将空出的列用于分数面板。《水墨剑客》选关页使用浏览器历史，系统返回先回卷首，再打开宿主菜单。小米 14 上验证了这两次返回，浏览器验证了开局后选关历史已消耗、各内置包的重复按钮均隐藏。Android Debug 构建通过。
  - 宿主胶囊缩至 76 × 32 dp，白底不透明度为 20%。应用广场设计稿的竖屏、横屏画板与安全区标示已同步。小米 14 在《水墨剑客》验证了 SDK 和 CSS 注入的胶囊尺寸，以及「···」打开菜单和「✕」退出。胶囊避让和游戏内返回规则已写入开发规范及作者指引。
  - 应用广场和「全部」列表里的官方游戏与工具改为轻点直接启动，长按打开带收藏和改编入口的详情。社区卡片继续进入帖子。应用广场设计稿已同步，Android Debug 构建通过。小米 14 实测 MBTI 和《水墨剑客》一次点击即进入协议 2，《水墨剑客》长按可查看详情，退出后回到原列表。
- 2026-11-08：Android 删除协议 1，手机上自己做的作品改走协议 2（未提交，由接手的会话完成；桌面端和论坛网页的协议 1 尚未处理）。
  - 起因：用户要求"删除 V1 支持"。上一位接手方（Codex）在迁移《墨渡》《茶签归位》、论坛网页试玩时因用量上限中断；这一段在其基础上继续，先让手机工作区作品支持协议 2，再删协议 1 的宿主。
  - 工作区作品：`MobileGameFormat` 改为协议 2 清单（schemaVersion 3、runtimeProtocol 2），必须有 `index.html`、`icon.svg`、`cover.svg`；kind 可以是 game 或 app，带 category、input、capabilities、version；文件上限 100 个、合计 8 MB，路径最多 6 层，允许 `.mjs`。id 不能以 `-` 结尾，因为它要做独立源的主机名。旧版（协议 1）作品在列表里提示"已不再支持，请让小伴按协议 2 重新生成"，仍可删除。
  - Agent 工具：`game_*` 改为 `app_read_format`、`app_list`、`app_validate`、`app_create`、`app_open`、`app_runtime_read`、`app_export`，创作规范换成协议 2 手机版；`libs/agent-mobile` 的覆盖确认跟着改名。`app_snapshot`、`lib_add`、`asset_*`、`app_publish`、`app_check_read` 仍待实现（规范第 11.3 节）。
  - 宿主：`RuntimeV2Activity` 直接打开 `games/<id>/`（独立源 `mine-<id>.app.xiangban.xyz`，联网默认开放，每次读取都拒绝符号链接），把加载、`xb.ready` 用时、脚本报错、控制台错误和成绩写进 `logs/<id>.txt`，供 `app_runtime_read` 读取。导出改为把协议 2 的 ZIP 存进下载目录的 Xiaoban 文件夹；发布页加入应用类型，游戏和应用上限均为 100 MB。
  - 已删除：`MobileGameActivity`、`MobileGameHost`、社区协议 1 压缩包读取器、构建里的协议 1 内置包、社区试玩里的协议 1 分支（只提示"旧版运行协议，手机已不再支持"）。协议 1 存档读取单独留在 `LegacySaves`，作品第一次以协议 2 打开时继承旧档。
  - 验证：community-core 的 MobileGames、GameLibrary、CommunityGameTrial、AppPlaza、ContentPublishing、CommunityCore 测试通过；`:androidApp:assembleDebug`、`:androidApp:compileReleaseKotlin`、`RuntimeV2*` 单元测试通过；agent-mobile 策略测试 4 项、server-forum 清单校验 5 项通过。小米 14 上用调试方式写入一件探针作品：从「我的作品」打开后进入 `RuntimeV2Activity`，日志依次记录 `host.open`、`game.ready`（55 ms）、`game.start`、`game.result`、`console.error` 和未捕获异常；胶囊 ✕ 正常退出；「保存作品包」在下载目录生成 ZIP，清单符合服务端协议 2 规则。内置作品回归：MBTI 仍显示已有答题进度，《水墨剑客》进入协议 2 且存储文件还在。
  - 没有验证：真正让手机 Agent 用 `app_*` 工具从头做一件作品（测试里的 Agent 流程需要本地原生库，只在有库时才跑）；工作区作品的相机、麦克风授权；其余 8 款内置游戏本轮没有重新逐个打开；分享作品包到系统分享面板。
  - 已知遗留：线上《墨渡》《茶签归位》仍是协议 1 包，手机端现在打不开，需要把本地已打好的协议 2 包发布成新版本（涉及生产论坛和资源机，需用户确认）；桌面端 `game_packs.rs`、`protocol.ts`、`check-game-assets.mjs` 和论坛 `game-sandbox` 仍是协议 1，桌面游戏空间与桌面层作品依赖它，要先确认桌面容器能给每个作品独立安全源才能删。
- 2026-11-08 23:36 CST：线上《墨渡》（56）和《茶签归位》（57）已原地替换为协议 2 的 1.1.0。
  - 做法：论坛没有"管理员给已有作品追加版本"的接口，作者接口追加版本又会降回未审并重新入队，所以按用户要求**直接替换版本 1**，不新增版本、不动审核状态、精选和署名。两个新 ZIP 先放进资源机 `data/packs/game/` 的新文件键（`ink-ferry-1.1.0.zip`、`tea-sort-1.1.0.zip`，资源机不允许覆盖旧键），再在一个事务里把 `pack_versions` 里这两行的 `label`、`sha256`、`size`、`manifest`、`file_key`、`runtime_protocol` 改成新值（`UPDATE 1` 两次）。生成 SQL 的脚本是 `scripts/replace_version_sql.py`，它会先核对 ZIP 与 `register.json` 的哈希、大小和清单，只允许这两个作品编号。
  - 回滚点：`/home/ubuntu/xiangban-forum/backups/v2-case-publish-20261108T073328Z/`（两行旧 `pack_versions`、`packs` 元数据、旧 ZIP）；旧 ZIP 在资源机原位仍在，回滚只需把两行改回旧值。服务端二进制和网页没有改动，`DEPLOYED_COMMIT` 仍是 `684b5cf7`，服务未重启。
  - 线上核对：带 `X-Xiaoban-Runtime: 1,2` 时两个作品都是 200、`versions` 只有一项（1.1.0、协议 2）；不带协议头（旧客户端）返回 404，创作者精选列表对旧客户端为空；下载地址能取到文件，大小和 SHA-256 与登记一致。
  - 小米 14：从广场点《墨渡》进入详情、"进入"，下载并缓存 `56-1-d79aab3c…` 后进入 `RuntimeV2Activity`；一局 12 分，结算页和"最高 12 分"正确，存储文件里 `{"best":12,"muted":false}` 说明旧最高分已继承。《茶签归位》同样下载缓存 `57-1-a9a8b599…`，选关页、第 1 关棋盘正常，"选关"按钮在胶囊下方；系统返回键一次后停在游戏内，没有退出。
  - 没有验证：《茶签归位》通关后的红印和旧存档继承（这台手机上没有它的旧存档）、横屏、断网后的表现、联网被禁用后的表现。
  - 代价：旧版 App（不带协议 2 能力）现在看不到这两个作品，也就是说线上只剩协议 2 的作品；网页版论坛试玩依赖作品域名，未配置前这两个作品在网页上不能试玩。
- 2026-11-08：确定小伴桌面端 V2「应用模块」技术路线图（方案已确认）。
  - **架构解耦**：彻底剥离桌面应用与壁纸/桌宠/特效等常驻装饰体系的强依赖。运行游戏、工具与测评时不挂起壁纸、不销毁桌宠，使用专有的 Tauri 独立沙箱窗口承载；
  - **协议拉通**：桌面端淘汰旧版协议 1（`xiaban.game-pack`），向协议 2 规范对齐。注入桌面版 `runtime-v2/bridge.js`，持久化存储使用本地独立沙箱文件（`apps/storage/<id>.json`），实现手机、网页、桌面三端共享统一的作品包生态；
  - **推进计划**：分三阶段实施：
    1. 第一阶段（容器基建）：在 `src-tauri` 中开发专有 V2 Webview 窗口与本地存储桥，实现 `bridge.js` 注入及胶囊控制栏；
    2. 第二阶段（界面重构）：控制台侧栏将原「游玩」升级为「应用」大厅，支持本地应用、精选测评与社区作品展示；
    3. 第三阶段（生态迁移）：将内置 10 款游戏升级为协议 2，并无缝拉通《墨渡》《茶签归位》等社区最新案例。
