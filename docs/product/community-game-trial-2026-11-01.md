# 社区游戏试玩验收 · 2026-11-01

代码分支：`codex/community-game-trial`，功能提交 `caff7108`，基于创作互动分支 `86391e6e`。本轮未合并或推送远端主分支。

## 行为

- Android 社区游戏详情、帖子正文中的本站游戏作品链接和回复作品卡提供试玩；回复固定版本不会被替换为最新版。
- 点击后检查公开状态、版本、资源大小和 SHA-256，再验证 ZIP 路径、数量、展开体积、UTF-8 和清单。运行复用离线 WebView 沙箱，游戏不取得账号凭证、文件访问或原生桥。
- 独立窄工具栏提供退出试玩、重新开始、收藏；为游戏保留独立显示区域，不覆盖游戏内按钮。退出返回原页面。
- ZIP 缓存最多 8 包、单包不超过 5 MiB；不放入创作工作区。社区存档/日志与本机创作分开，按作品编号和版本隔离。缓存清理不影响存档、收藏和最近记录。
- 收藏复用已有作品收藏；「我的收藏」新增游戏筛选及本机最近玩过。最近记录最多 12 项，成功加载后才写入，不自动收藏。纯 HTML 游戏兼容无宿主握手的加载成功事件。
- 后端 `kind=game` 仅是收藏查询筛选，响应身份仍为 `pack`，筛选先于分页和 total 统计。无数据库结构变更。

## 验证

- JVM 17 项通过：CommunityGameTrialTest 7、MobileGamesTest 7、GameLibraryTest 3。包括完整性、版本/可见性、路径和重复项、二进制拒绝、展开限制、历史去重、链接来源、收藏分页协议。
- 后端 `game_bookmarks_filter_before_pagination_and_keep_pack_identity` 通过，使用本机独立测试实例和 schema，覆盖未登录、下架排除、真实总数、翻页及 pack 查询兼容。测试 PostgreSQL 已停止。
- Android `:androidApp:assembleDebug` 通过，最终 APK 已覆盖安装到小米 14（23127PN0CC）；未操作另一台小米 17。
- 真机从「社区 → 讨论 → 创作 → 接住星光」进入详情，点击试玩成功加载真实资源包 #55 v1。重开、开始、暂停、退出返回均可操作；返回页保留标题和试玩入口。
- 真机「我的 → 我的收藏 → 游戏」成功读取线上收藏数据；「最近玩过」显示接住星光，并有再次试玩入口。两次启动后仅一份 ZIP 缓存，创作工作区不存在 starlight-catcher 目录。
- 未在生产环境发帖、评论、点赞或改变用户收藏。收藏写操作由既有接口和针对性测试覆盖，真机核对读取和入口；没有伪造互动数据。
- 本机截图位于 `/tmp/xb-community-review/`：`trial-work-final.png`、`trial-toolbar-final.png`、`trial-returned-final.png`、`trial-game-collection.png`、`trial-recent-final.png`。

## 部署

后端在本机构建 x86_64 Linux 产物后上传香港论坛服务器，未在云端编译。替换前确认运行二进制仍为上一轮互动版本，保留回退文件于 `/home/ubuntu/xiangban-forum/backups/game-trial-caff7108/server-forum`。

新二进制 SHA-256：`821505c6d5901111d4468e53d163d52d0f08710e8df03443d7cfa22dc1071ed0`。重启后 systemd 为 active，内外网 `/api/health` 均返回 `{"ok":true}`。

## 当前范围

支持 v1、layer=space、无系统能力的离线文本游戏。普通帖子 ZIP 附件不自动认定为游戏；v2 二进制包、桌面层和系统权限包保持明确失败，不降级为联网网页运行。iOS 尚无游戏沙箱，入口明确提示暂不支持。

不新增一级「我的游戏」，不提供离线保存入口；即便命中缓存，启动仍需在线确认作品公开状态。协议游戏保留现有存档协议，纯 HTML 游戏不伪造存档能力。本文不代表网页端、iOS 或桌面端新增试玩实现。
