# 应用广场与运行时协议 2：实施进度

规范见 [README.md](README.md)。本文记录实施进度，每完成一步就更新。「完成」只表示表里写明的验证已经做过，没写的就是没验证。

基线分支：`feat/app-plaza-runtime`。iOS 暂缓（2026-11-08 决定）。

## 分工

| 工作 | 分支（worktree） | 执行 | 状态 |
| --- | --- | --- | --- |
| 规范、原型、给 AI 的规则 | `feat/app-plaza-runtime` | Claude Fable 5 | 已提交（`935174c9`） |
| 手机端应用广场界面（Compose） | `feat/app-plaza-ui`（`.claude/worktrees/plaza-ui`） | Codex gpt-6.1-sol | 已完成，6 个提交（`3c6a1f2c`…`e16fb885`）；真机看过广场首屏，社区作品、帖子页还没在真机上看过 |
| Android 运行时协议 2 小样 | `spike/android-runtime-v2`（`.claude/worktrees/runtime-spike`） | Codex gpt-6.1-terra | 已完成并在真机上验证，5 个提交（`a9d3fd73`…`e4583354`）；摄像头和麦克风待人工点同意 |
| 服务端 `app` 类型与协议 2 字段 | `feat/forum-app-kind`（`.claude/worktrees/server-v2`） | Claude Sonnet 5.5 | 已完成并通过审查，7 个提交（`53b1f89d`…`383b4256`），未合并、未部署 |
| 审查、真机验证（小米 14）、合并 | — | Claude Fable 5 | 等上面三项完成 |

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

1. Android 正式宿主：以小样为基础改造运行页，实现 `xb` 第一批接口、胶囊、权限同意框和网络限制。
2. `app.xiangban.xyz` 文件服务和 SDK 注入；通配证书；申请加入公共后缀列表（需要产品负责人操作）。
3. 手机 Agent 新工具：`app_*`、`lib_add`、`asset_image`、`app_publish`；工作区支持二进制文件。
4. 广场接入工具与测评、创作者精选、改编列表。
5. 网页和桌面按规范第 9 节跟进。

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
