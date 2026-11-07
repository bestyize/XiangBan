# 应用广场与运行时协议 2：实施进度

规范见 [README.md](README.md)。本文记录实施进度，每完成一步就更新。「完成」只表示表里写明的验证已经做过，没写的就是没验证。

基线分支：`feat/app-plaza-runtime`。iOS 暂缓（2026-11-08 决定）。

## 分工

| 工作 | 分支（worktree） | 执行 | 状态 |
| --- | --- | --- | --- |
| 规范、原型、给 AI 的规则 | `feat/app-plaza-runtime` | Claude Fable 5 | 已提交（`935174c9`） |
| 手机端应用广场界面（Compose） | `feat/app-plaza-ui`（`.claude/worktrees/plaza-ui`） | Codex gpt-6.1-sol | 进行中 |
| Android 运行时协议 2 小样 | `spike/android-runtime-v2`（`.claude/worktrees/runtime-spike`） | Codex gpt-6.1-terra | 进行中 |
| 服务端 `app` 类型与协议 2 字段 | `feat/forum-app-kind`（`.claude/worktrees/server-v2`） | Claude Sonnet 5.5 | 进行中 |
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
