# server-forum：超级小伴论坛

## 账号 ID 与 AI 额度入口

「我的」从账号中心 `/api/me` 读取并展示数字字符串 `account_id`，不会展示内部 UUID；换号和退出会清除展示 ID，过期请求不能覆盖当前账号。「AI 额度」通过 `/ai?panel=account` 直接展开现有额度、兑换和模型配置面板。

验证：在 `web/` 执行 `npm run typecheck`、`node --experimental-strip-types --test tests/session.test.ts tests/ai.test.ts tests/aiCredits.test.ts`。

状态：一期（网页版）已实现并部署。超级小伴社区 `chat.xiangban.xyz` 以发现、讨论和资源组织内容；讨论包含创作、学习和教程、休闲，资源包含 AI 工具、资料、影音娱乐、求助悬赏，另有站务。产品设计见 [docs/product/PRD_论坛.md](../../docs/product/PRD_论坛.md)。

栏目清单位于 [`editorial/channels.json`](editorial/channels.json)，前台映射在 [`web/src/lib/communityStructure.ts`](web/src/lib/communityStructure.ts)。启动迁移会将旧技术、资源和小伴产品分类改为当前栏目，保留分类 ID、历史帖子与权限；旧 slug 查询兼容，旧分类页面跳转至新地址。下文关于旧迁移的记录用于解释历史实现，不代表当前分类仍使用旧名称。创作和方案页面同时呈现真实作品与图文导读，资源工具和资料同时呈现真实目录与导读，彼此独立计数。小伴前沿和日报使用发现页文章通道。常驻采编的来源、预算、发布验证与部署见 [`editorial/README.md`](editorial/README.md)。

技术栈：Rust Axum 0.8 + PostgreSQL（SQLx），前端 Vue 3 + Vite + TypeScript（`web/`），由本服务直接托管。登录走[统一账号中心](../server-account/README.md)，本站不存密码，用 JWKS 公钥离线验签。默认端口 55010。

## 运行与验证

积分系统已实现：新加入论坛 +50、每日登录访问 +1、签到 +1，当月累计签到 7／20 天分别额外 +3／+5，有效邀请 +30（每天最多 3 位）、同帖首次加精 +10。积分与信任等级独立，入口为顶部积分余额与 `/points`，提供签到、邀请和账本。已有论坛用户不补发注册奖励。

附件可设置 0～10 分；购买确认、扣款、作者入账和购买记录在同一事务内完成，同一附件重复下载免费，作者自己下载免费，付费不绕过阅读权限。资源分享下新增“资源求助”，支持 0～1000 分悬赏、7／14／30 天期限、采纳发放、关闭和到期退款。后台任务约每 10 分钟处理到期退款；已采纳回答保留原文。

数据库回归测试需要 `TEST_DATABASE_URL` 指向隔离测试库：`cargo test --locked --manifest-path services/server-forum/Cargo.toml points::tests`。测试使用独立 schema，不改生产数据。

```sh
cp services/server-forum/.env.example services/server-forum/.env   # 按需修改
npm --prefix services/server-forum/web ci
npm --prefix services/server-forum/web run build                  # 生成 web/dist；不构建也能跑，只是没有前端
cargo run --manifest-path services/server-forum/Cargo.toml
cargo test --manifest-path services/server-forum/Cargo.toml
```

数据库表和分类种子在启动时自动创建（种子只在表为空时写入，后台改过的不会被覆盖）。数据库有 `pg_trgm` 扩展时会给标题建三元组索引，没有就退化为普通 ILIKE。

前端开发：`npm --prefix services/server-forum/web run dev`，默认端口 5185，`/api` 代理到 55010；可用 `FORUM_WEB_PORT`、`FORUM_API` 覆盖。

本地联调登录需要同时跑一个账号中心：`ACCOUNT_ISSUER=http://localhost:55009`、`ACCOUNT_SENDER=console`（验证码写进日志）、`ACCOUNT_CLIENTS` 里包含 `forum-web`、`ACCOUNT_CORS_ORIGINS` 包含前端开发地址；论坛这边设 `FORUM_ACCOUNT_ISSUER=http://localhost:55009`。想跳过新人限制，可以设 `FORUM_TL1_DAYS=0 FORUM_TL1_READS=0`。

## 移动优先网页版（2026-11-01，本地实现）

范围及验收见 [实施计划](../../docs/product/实施计划_小伴网页版_2026-11-01.md)。网页浏览社区、体验公开作品、进行基础文字对话；生成/修改作品引导官网下载客户端。Android 仅承诺已支持的游戏创作，iOS 不宣称支持本地创作。桌面社区通过独立原生窗口访问本站，账号在网页独立登录，不注入桌面令牌。

- 创造发现页与小米 14 对齐：9 款官方内置游戏按手机顺序展示，其后是柔软山丘壁纸灵感、9 款官方小组件（含鹈鹕骑车）、12 款官方特效及 7 位官方伙伴。分类为全部/游戏/壁纸/小组件/特效/伙伴。卡片打开预览与游戏试玩，只有「你想创造什么？」触发客户端下载提示；「我的作品」通往账号已发布作品，不冒充手机本机文件同步。社区作品继续在社区讨论的创作栏目展示。
- `npm run dev/build` 的前置脚本从仓库 `client/xiaoban-desktop/public/game-packs/` 同步这 9 款官方资源到忽略提交的 `web/public/builtin-games/`，构建产物包含它们；构建需完整本仓库。官方游戏无需社区 API 或独立试玩域名，在不允许同源访问的 iframe 内运行，以来源和 nonce 校验就绪/退出消息，本次试玩不持久保存进度。`npm run sync:builtins` 同时同步桌面小组件的纯视觉实现与完整伙伴 SVG；特效复用桌面粒子引擎和官方数据。小组件天气/行情明确标记为演示数据，特效只在详情可见时播放，伙伴预览保留完整花盆与表情，详情按需加载客户端原有动画引擎，支持点击/键盘表情反馈、自然眨眼和视线跟随；离屏、后台及关闭详情时停止绘制。网页预览不会安装桌面装扮。
- AI 默认连接 `https://ai.xiangban.xyz`，构建时可设 `VITE_AI_BASE_URL`（HTTPS；本地联调允许 localhost HTTP）。论坛令牌的 `aud=forum-web` 须与桌面、手机一起关联到同一个已启用的 AI 产品，使同一账号共享试用、额度、兑换与流水；如果配置了 `AI_CORS_ORIGINS`，还须包含本站实际 origin。前端不包含服务商密钥。对话历史按账号保存在当前浏览器，最多 30 对话、每个 100 消息，不代表跨端同步。
- 游戏试玩仅接受公开 T0/T1、当前指定版本且完整性检查通过的离线 v1 游戏包。构建时配置 `VITE_GAME_SANDBOX_URL` 为独立 HTTPS origin，部署 `web/game-sandbox/`，并让资源服务允许网页跨域读取签名下载 URL。未配置时如实显示不可用。独立域名、CSP、内部 sandbox 和父子消息校验详见 [试玩宿主说明](web/game-sandbox/README.md)。不向作品提供账号令牌或桌面能力。
- 本地只读联调可设 `FORUM_API=https://chat.xiangban.xyz`；试玩同时需独立本地宿主及 `VITE_GAME_RESOURCE_ORIGIN=https://chat.xiangban.xyz`，后者仅在 DEV 生效，用于匹配 `/files/` 代理。生产构建不启用开发代理。
- 前端验收：`npm --prefix services/server-forum/web run build`；行为测试：在 `web/` 执行 `node --experimental-strip-types --test tests/*.test.ts`。

## 网页图片与 GIF 生成（本地实现，尚未部署）

`/ai` 登录后点击「生图」，输入描述、尺寸和数量即可使用独立图片服务；网关开放 `gif` 格式后可选 GIF 动图，设置帧数和每帧时长。静态图每次 1–4 张，GIF 每次 1 张、2–16 帧、20–2000 毫秒/帧。切换面板不取消进行中的请求，关闭页面或超时不自动重发；结果不明时提示先核对，保留已返回的部分图片。

GIF 直接播放原动画，下载保留原格式。图片按账号存在当前浏览器 IndexedDB，最多最近 10 张且不超过 96MB，不写入文字历史或发送给聊天模型，不代表跨端同步；存储受限时保留当前结果并提示及时下载。换号/退出取消旧请求并清除旧账号界面。单图 16MB；GIF 先检查帧结构、帧数和总像素，远程图片下载不携带令牌或 Cookie、不跟随跳转。

验证：`node --experimental-strip-types --test tests/aiImages.test.ts tests/ai.test.ts`、`npm run typecheck`、`npm exec vite -- build`（在 `web/` 执行）。浏览器通过本地模拟网关验证 GIF 请求、原文件下载、刷新恢复和账号隔离，未调用真实模型。标准 `npm run build` 的既有小组件同步脚本有依赖遗漏，本轮直接验证现有源码打包，不改动该同步流程。

## 社区卡片呈现（2026-11-02）

学习教程、资源求助保持单列左文右图；站务为无装饰封面的文字列表。工具与资料导读使用16:9上图下文，普通卡片在手机上双列，按实际内容区宽度在840px起三列、1120px起四列；工具目录和能力作品使用左图标右说明，同样在宽屏增加列数。壁纸作品为手机双列、宽屏三列瀑布流；休闲的全部及各子栏目同样使用手机双列、宽屏三列上图下文瀑布流，无配图的帖子显示标题与摘要。影视和音乐使用各屏幕双列瀑布流；文章预览宽高比限制在2:3到16:9，壁纸预览限制在4:5到16:9，详情保留完整原图。没有方案实景截图时仍展示真实文字与作品引用，不生成伪截图。影音娱乐的全部、影视、音乐筛选与讨论分类共用胶囊标签组件。

卡片标题统一14px、400字重；影视标题最多3行，其他卡片最多2行，作者、标签、时间等辅助文字12px。文章详情标题与分组标题保留各自层级。规则集中于 `web/src/lib/cardPresentation.ts` 和 `web/src/styles/community-cards.css`。瀑布流使用保留DOM顺序的网格，图片加载和容器变化由ResizeObserver重排，未支持时退回普通双列。图片继续使用已有WebP资源，懒加载并预留封面比例，不修改文章或权限。

首页“小伴前沿”使用16:9焦点轮播，宽屏旁列展示两条推荐，手机完整显示单张焦点图，不使用横向滚动条。圆点、键盘方向键和水平滑动可切换；每6秒自动轮播，手动切换后暂停，悬停、聚焦、后台或减少动态效果偏好下暂停自动切换。封面标题最多一行并省略溢出文字，来源与日期在右下角同一行，圆点位于左下角；不设置独立文字区或播放/暂停、左右切换按钮；所有跳转仍指向原前沿文章。

验证：`node --experimental-strip-types --test web/tests/cardPresentation.test.ts`（从本目录执行），完整前端构建仍使用上文命令。

## 配置

见 [.env.example](.env.example)。关键项：

| 变量 | 默认 | 说明 |
| --- | --- | --- |
| `FORUM_DATABASE_URL` | `postgres://xiaban@localhost:5432/xiaban_forum` | 独立库 |
| `FORUM_PUBLIC_URL` | `https://chat.xiangban.xyz` | SEO 页面的 canonical 和 sitemap |
| `FORUM_ACCOUNT_ISSUER` | `https://account.xiangban.xyz` | 必须和账号中心 `ACCOUNT_ISSUER` 完全一致 |
| `FORUM_ACCOUNT_CLIENT_ID` | `forum-web` | 网页的令牌 `aud`，需要先在账号中心后台登记；也是 `/api/meta` 告诉前端用的 client_id |
| `FORUM_ACCOUNT_EXTRA_CLIENT_IDS` | `xiaoban-android,xiaoban-desktop-wallpaper` | 除网页外接受手机与桌面令牌 `aud`（逗号分隔）；显式设置会替换默认值，需同时保留这两个客户端 ID |
| `FORUM_ADMIN_USER` / `FORUM_ADMIN_PASSWORD` | 无 | 没有管理员时创建初始管理员 |
| `FORUM_ADMIN_JWT_SECRET` | 随机 | 管理员令牌密钥，生产必须设置（≥32 位） |
| `FORUM_WEB_DIR` | `services/server-forum/web/dist` | 前端产物；index.html 启动时读一次，换前端后要重启 |
| `FORUM_TL1_DAYS` / `FORUM_TL1_READS` | 3 / 30 | 新人自动升级为成员的门槛 |
| `FORUM_DAILY_TOPICS` / `FORUM_DAILY_POSTS` | `2,5,20,50` / `10,60,200,500` | 按信任等级 0-3 的每日发帖、回复上限。新人（TL0）能发帖的只有最低发帖等级为 0 的板块，即小伴壁纸、小伴游戏 |
| `FORUM_UPLOAD_DIR` | `services/server-forum/data/uploads` | 帖子附件存放目录，需要持久化和备份 |
| `FORUM_UPLOAD_MAX_MB_USER` / `FORUM_UPLOAD_MAX_MB_ADMIN` | 5 / 30 | 单个附件上限（MB）：成员及以上 / 守护者 |
| `FORUM_UPLOAD_MAX_MB_IMAGE` | 5 | 正文插图上限（MB），所有人一样 |
| `FORUM_RESOURCE_BASE_URL` | 空 | 资源机对外地址（内容包、封面）。为空时下载与上传凭证接口返回 503 |
| `FORUM_RESOURCE_SECRET` | 随机 | 与资源机共用的 HMAC 密钥（≥32 位），签下载地址与上传凭证、校验回调；生产必须设置，且两台机器一致 |
| `FORUM_DOWNLOAD_URL_TTL_SECS` / `FORUM_UPLOAD_TICKET_TTL_SECS` | 600 / 1800 | 下载地址与上传凭证的有效期 |
| `FORUM_DAILY_PACK_PUBLISHES` | 5 | 普通用户每天最多发布（含已申请凭证未使用）的内容包数 |

## 页面与接口

「小伴前沿」提供 `/frontier` 列表和详情、首页已发布文章入口、发现页 `frontier` 草稿类型与隔离的评论表，沿用发现页人工审核发布。`POST /api/discover-editor/frontier/{key}/assets` 使用允许 `frontier` 的采集令牌接收 JSON `data_base64`，限制每图 5 MiB、每篇 20 张；存入数据库并审计，只有已发布文章引用的图片通过公开接口读取。Nginx 对该前缀需设置 `client_max_body_size 7m`。采集和图文生成可由 `tools/claude-skills/xiaoban-frontier` 使用 server-ai 完成，运行需运营账号和受限采集令牌；日报关联和 SEO 暂未实现。

前台路由：`/` 社区发现（推荐、开源热榜、排行榜、追踪、导航）、`/discuss` 讨论、`/resources` 资源、`/daily[/{key}]` 日报、`/tools/{key}` 工具详情；`/create` 官方内置创造体验、`/ai` 小伴 AI、`/me` 我的。手机底栏与 PC 侧栏统一四个模块，发布入口提供发帖与上传已有作品。保留 `/packs/{id}`、`/schemes`、`/weekly/{week}`、`/projects/{slug}`、帖子、分类、搜索、收藏、积分、设置及 `/admin`。旧首页社区作品类型链接转到 `/discuss?tab=create&filter=…`，周刊/项目旧 Tab 继续有效。

服务端对 `/`、`/t/{id}`、`/c/{slug}`、`/tag/{name}` 直接输出带标题、摘要、正文和 JSON-LD 的 HTML（填进 `#app`，Vue 挂载时替换），另有 `/sitemap.xml`、`/robots.txt`。实现在 `src/seo.rs`，依赖 `web/index.html` 保留 `<title>`、`<meta name="description">` 和空的 `<div id="app"></div>`。

前台接口（`/api`，写操作要带账号中心访问令牌）：

| 接口 | 说明 |
| --- | --- |
| `GET /config` | 站点名、账号中心地址、client_id |
| `GET /categories` `/tags` | 分类树、热门标签 |
| `POST /session` | 登录后调用：同步昵称头像、记访问、检查自动升级，返回本站资料和每日上限 |
| `GET /topics` `/search` | 列表；参数见 `models::ListFilter` |
| `GET/POST /topics`、`GET/PATCH/DELETE /topics/{id}` | 帖子 |
| `GET/POST /topics/{id}/posts`、`PATCH/DELETE /posts/{id}` | 回复 |
| | `PUT/DELETE /topics/{id}/like`、`/posts/{id}/like`、`/topics/{id}/bookmark` | 点赞、收藏 |
| `PUT/DELETE /topics/{id}/pin` | 置顶、取消置顶（仅守护者 TL3） |
| `POST /reports` | 举报帖子或回复 |
| `POST /markdown/preview` | 发帖预览 |
| `POST /images`、`GET /images/{id}` | 正文插图：上传（multipart，字段 `file`）、按 id 公开访问 |
| `POST /attachments`、`GET/DELETE /attachments/{id}` | 上传附件（multipart，字段 `file`）、下载、撤销还没挂到帖子上的上传 |
| `GET /users/{uid}`、`/users/{uid}/topics`、`/users/{uid}/posts`、`/me/bookmarks` | 用户主页、我的收藏 |

休闲一级板块下的帖子支持省略 `title`（或传空串），填写时最多 80 个 Unicode 字符；创建、编辑及移动分类均按目标分类校验，其他板块仍要求 4～80 字。列表/详情的 `category.parent_slug` 标识所属一级板块，分享信息在标题为空时使用正文摘要。详情 `body_html` 保留全部图片，列表 `images` 仍只提供前三张缩略图。

管理接口在 `/api/admin/*`（`src/handlers/admin.rs`）：管理员独立建表，HS256 令牌带 `token_version`，改密码后旧令牌失效，连续输错 5 次锁 15 分钟；包括概览、举报处理、帖子与回复管理、用户等级和禁言封禁、分类维护、操作日志，所有写操作都记审计。

网页版外观支持客户端同款小伴红、薄荷绿、海湾蓝、暮紫、琥珀、石墨六套配色，在「我的 → 外观」、PC 外观入口及账户设置中选择。主题色与日夜模式独立保存于当前浏览器，跨标签同步；夜间保留提亮后的中深灰底色，强调文字根据网页背景调整对比度。

排行榜概览 `/?tab=rankings` 按手机版顺序展示 AA 智能指数 Top 10、智能 vs 价格散点图、输出速度 Top 5、调用量 Top 5。`/models` 与调用量榜使用可展开的条形排行，支持快照实际提供的能力指标及 UTC 历史日榜。散点图沿用输入/输出 3:1 加权报价、对数价格轴、前沿与中位数优势区，支持厂商筛选、模型定位及全屏搜索。厂商图标来自手机本地资源（附原许可），不请求外部图片。相关验证：`node --experimental-strip-types --test tests/modelRankings.test.ts tests/discover.test.ts tests/navigation.test.ts`（在 web 目录运行）。

### AI 导航（2026-11-03，本地实现）

导航主分类按用途分为大模型（原厂）、Harness（AI 助手与智能体）、Skill（技能平台与官方技能库）、API（模型调用服务）。首批目录 `editorial/navigation.json` 为 v2，共 66 个入口：18 家模型原厂、25 个助手与 Agent、5 个技能平台/官方技能库、18 个 API 服务。同一厂商按不同目的链接到不同入口，例如 Anthropic 原厂、Claude Desktop、Claude Code 和 Claude Platform；产品入口不混进原厂分类。大模型按国产/海外筛选，Harness 按综合助手/编程 Agent/个人智能体筛选，Skill 按技能市场/官方技能库筛选，API 按原厂 API/聚合与推理/云平台筛选。筛选与主分类记录在 `/?tab=nav&nav_category=...&nav_filter=...`，刷新、返回和分享保留状态，换主分类自动清空筛选。

卡片提供 Logo、名称、所属厂商、一句话用途和类型标签；宽屏补充域名。按实际内容区宽度使用两、三、四列，保持社区主题、边框与字号规范，不使用封面图或卡片阴影。Hermes Agent 使用项目官网 favicon，其他新增图标延用同版本 Lobe Icons；来源与许可在 `web/public/vendor-logos/README.md`。导航仅链接官方入口，不承诺地区可用性或服务方案。

服务启动执行一次性 `navigation-catalog-20261103-v2` 迁移。缺失条目按目录补齐；未分类的历史内置入口按用途迁移，保留编辑后的名称、URL、状态、排序和自定义说明，只有仍与原始 v1 默认值一致的名称/URL 才更新为新入口。已经显式分类的编辑不覆盖；已完成 v1 的目录不会在 v2 升级中恢复已删除的旧入口。完成 v2 后，管理员删除的条目不会在后续启动时恢复。后台“发现页 → 导航”可维护分类、入口类型、地区、用途说明和图标品牌名，服务端校验元数据组合。

`/api/discover/nav` 保留 `groups` 协议并返回 `catalog_version:2`，四个分组按主分类固定顺序排列，手机可继续读取；更新的网页以 v2 已发布数据为准，允许主动清空。旧接口版本低于 2 时，网页用同一清单做分类兼容与补齐；v1 中被管理员删除的旧条目仍保留删除结果。按分类内 key/地址去重，原厂官网与 API 控制台可以作为不同目的入口共存，接口加载错误如实呈现。

验证：网页 `npm run build`、`node --experimental-strip-types --test tests/navigationDirectory.test.ts`；隔离库 `TEST_DATABASE_URL=postgres://… cargo test --manifest-path services/server-forum/Cargo.toml navigation_catalog` 检查字段校验、分类迁移、编辑/下架/删除保留。本轮未部署线上。

### 资源 · 开源软件

软件目录由 `editorial/open-source-software.json` 指定 GitHub 仓库及发行仓库。论坛进程分散执行首次采集，目录超过 400 条时匿名首轮分散到约 72 小时，每四分钟最多开始一次。后续按上次成功采集结果分档：至少 1,000 Star 且最近 30 天有推送，每 2 天刷新。至少 1,000 Star 但近期无推送，每 7 天刷新。其余每 14 天刷新。采集队列保存在 PostgreSQL，多实例共用锁和请求间隔，GitHub 限流时按返回的重试时间延后队列，失败时保留上次成功快照。可选的 `FORUM_SOFTWARE_GITHUB_TOKEN` 仅在服务端使用。此调度不依赖 GitHub Actions。

公开列表和详情读取同一份 Star、推送时间、最新正式版和最近 100 个 Release 的快照。近期热门使用距当前快照最接近 7 天的 Star 快照计算，接受相隔 6 至 8 个 UTC 日期的快照，缺少任一端时不计算。首次同步前，列表使用收录时的资料，详情提供 GitHub 官方发布页。网页安装包下载通过浏览器请求 ipwho.is 查询访问者 IP 国家代码：CN 使用第三方 gh-proxy.com 加速并同时提供 GitHub 官方下载，其余地区或查询失败时使用官方链接。查询会向 ipwho.is 暴露访问者 IP；免费接口有按站点共享的请求限额，超限时回退官方链接。全部历史可在官方发布页查看。

### 精选（2026-11-03，本地实现）

精选仅在首页提供入口，资源页不再重复设置精选子 TAB（旧 `/resources?tab=open` 跳转首页精选）；原「开源热榜」合并为第二个子 TAB「热榜」，旧 `/?tab=repos` 携带往期期数跳转新入口；`/open-source` 跳转首页精选。推荐、热榜与 11 个用途分类支持跨分类收录、标签筛选，项目卡片与专题卡链接独立导读页；搜索统一进入顶栏，收藏操作仅在项目详情显示。网页列表以项目 Logo（缺失时使用名称首字母）、名称、仓库标识、GitHub About 中文译文和标签为主，不重复推荐标题。仅 `web/src/lib/openSourceCardMedia.json` 中已核对来源的真实界面、作品和实物图保留大图；仓库首页截图和流程示意不再用于公开卡片，专题卡保留。历史图文保留在「社区分享」。

**板块边界：以下模板、自动匹配、素材库与发布规则仅用于精选的 GitHub 项目和专题。** 不接入资讯、前沿、讨论、作品、休闲等板块的卡片渲染器；其他板块继续使用各自的内容与封面规范。可以复用图片格式检查等底层能力，不能直接继承精选的版式或匹配策略。

精选封面规则位于 `editorial/open-source-cover-rules.json`：统一 1280×720，提供界面截图、实物照片、流程示意、命令示例、内容示意五个模板，以及均衡、错位、舒展三种构图。自动匹配按素材身份、分类、标签选择，仓库身份稳定决定缺省配色和构图；支持手动选择、品牌色、照片焦点、横向 Logo 裁切和三行示意内容。封面编辑与导出工具没有图片时退回明确标注的示意；公开网页卡片没有已核实配图时使用紧凑排版，详情保留上游原图。不依赖模型生图；浏览器与服务端共享清单及协议案例，手机读取服务端解析后的配置。网页管理台可导出 PNG；外部素材受 CORS 限制时，先采集为站内素材或本地上传。

2026-11-06 网页卡片更新：精选 168 个项目与热榜现有 7 期的 31 个项目（去重共 195 个）已核对 GitHub About；193 个有原文并完成中文翻译，`raspberrypi/pico-sdk` 和 `pingdotgg/t3code` 未填写 About，明确显示暂无简介。原文、译文、许可元数据、目录 revision 与热榜历史入口分别保存在 `editorial/open-source-about-20261106.json`、`editorial/hot-repo-about-20261106.json`。前端共用 `web/src/lib/githubAboutTranslations.json` 和 `githubProjectAbout.ts`：精选列表和详情共用译文；热榜首页露出卡、完整列表和详情按已核对的原文匹配译文，后续原文变化时回退到接口的中文翻译或原文，避免套用过期译文。热榜详情可展开 About 原文，保留 README、评论和收藏；增量按本期口径展示。资源页软件卡片沿用原有数据。这里记录的是本地实现与核验，尚未发布生产。

2026-11-03 内容方向更新：精选详情采用中文 README / English 切换，项目卡片与阅读交互见已入库原型，具体采编与全量改稿要求见 [README 采编规范](editorial/OPEN_SOURCE_README_WORKFLOW.md)。下文是现有正式接口状态，双语正文方案尚不能视为已接入或部署。

管理入口 `/admin/open-source`，操作顺序为获取 GitHub 资料或手工新增 → 编辑导读与素材 → 保存草稿 → 校验预览 → 发布此版本。GitHub 导入只读取公开仓库元数据和 README 图片候选，过滤徽章，不把作者头像当作项目 Logo，不会自动保存或发布；编辑需核对图片身份与许可。上传支持将 SVG/ICO 在浏览器转成 PNG，服务端仅接收 ≤5 MiB 的 PNG/JPEG/WebP；远程采集逐次检查重定向与公共网络地址。素材按项目归属保存，未被已发布目录引用前仅管理员可预览。

`open_source_catalog` 保存独立草稿、线上快照及版本，`open_source_assets` 保存素材及来源。保存和发布使用事务锁及 `expected_revision` 防止覆盖并发编辑，审计记录保留管理员与版本；只有已保存并通过校验的版本能从管理界面发布。`editorial/open-source.json` 是首版 12 项目、2 专题的随版本目录，首次发布前作为服务端初始内容，发布后以数据库快照为准，允许有意发布空目录。网页 GET `/api/open-source` 后读取线上快照（旧服务无接口时保留随版本目录），手机读取同一接口。精选发布不写入 `discover_items`，不改变其他板块的数据或运营流程。

管理 API 均要求管理员令牌：`GET/PUT /api/admin/open-source` 读取/保存草稿，`POST .../preview` 校验，`POST .../publish` 发布指定已保存版本，`POST .../import-github` 获取资料，`POST .../assets` 上传，`POST .../assets/fetch` 采集，`GET .../assets/{id}` 私有预览。公开素材入口 `/api/open-source/assets/{id}` 仅提供线上目录引用的文件。种子图片来源见 `web/public/open-source-media/README.md`。公开资料导读明确未经本站实测，不伪造 Star 或使用体验；项目评论继续使用原有账号和内容审核规则；收藏沿用设备本地存储，与热榜共用仓库身份。

验证：网页 `npm run build`；在 `web/` 执行 `node --experimental-strip-types --test tests/openSource.test.ts tests/navigation.test.ts tests/cardPresentation.test.ts`。服务端用隔离库运行 `TEST_DATABASE_URL=postgres://… cargo test --manifest-path services/server-forum/Cargo.toml open_source`，覆盖权限、草稿隔离、并发版本、空目录、素材可见性/归属、其他板块隔离和封面协议。手机编译与协议测试见手机项目 README。本轮仅本地实现与验证，未部署生产环境。

### PC 网页布局（2026-11-02）

公开社区页面在 1024px 起使用独立的 PC 样式：200px 导航栏、与正文对齐的顶部导航和搜索入口，内容区最大 1320px。社区首页的前沿图文、日报、精选作品分清主次，讨论和资源统一卡片与间距，正文保持阅读宽度。排行榜、开源榜、追踪和导航按实际内容宽度切换布局；原先内容容器最大 1000px，却使用 1100px 容器断点，导致宽屏布局无法生效，本轮将其改为 PC 范围内的 960px 内容断点。手机与后台不使用这套桌面样式。 网页窄屏底栏另与手机版对齐：通栏固定、56px 主体加安全区、四个等宽图标文字入口，选中项深色加粗，中间为 44×32px 的品牌色发布按钮；聊天输入区随新底栏高度避让。

验证：网页生产构建（含 Vue/TypeScript 检查）与 navigation/discover/appearance 9 项测试通过；云浏览器使用生产公开数据的只读快照检查首页、讨论、资源、排行榜和日夜主题。预览未连接登录、发布和其他写操作；本轮没有部署生产网站。

## 社区内容层（内容包、方案、项目、周刊）

接口都在 `/api` 下，实现见 `src/handlers/content.rs`，纯逻辑（体积上限、清单检查、签名）见 `src/content.rs`。

- 内容包类型与体积上限：effect/reminder/routine/task/goal ≤ 1MB，widget/skill/workflow ≤ 8MB，game/app ≤ 100MB，expert ≤ 1MB，wallpaper ≤ 32MB。信任分级 T0 官方、T1 已审核、T2 用户未审；T2 只有 effect、scheme、reminder 返回 `one_click: true`，其余类型需要桌面端二次确认。
- 前台：`GET /packs`（`kind`、`q`、`tag`、`author`、`sort=new|hot`、分页）、`GET /packs/{id}`（含版本与清单）、`GET /packs/{id}/download?version=`（返回签名地址、`sha256`、`size`、官方镜像 `mirrors`；客户端每个来源都要校验 sha256）、`POST /packs/{id}/applied`（应用计数，匿名可调）、`GET/POST /schemes`、`GET /schemes/{id}`、`GET /me/schemes`、`POST /schemes/{id}/remove`、`GET /projects[/{slug}]`、`GET /weekly[/{week}]`、`GET /share/{kind}/{id}`（`pack|topic|scheme|project|weekly`，返回落地页地址与 `xiaoban://community/...` 深链接）。
- 用户发布流程（文件不经过本服务）：
  1. `POST /packs/upload-ticket {kind,size,sha256}`（需登录）→ `{ticket, upload_url, file_key}`；
  2. 客户端把文件和 `ticket` 直传 `upload_url`（资源机校验凭证、大小、哈希）；
  3. 资源机落盘并重算哈希后，回调 `POST /api/internal/pack-uploaded`，请求体 `{file_key,size,sha256,manifest}`（`manifest` 是资源机从压缩包根目录读出的 `manifest.json`，内容包必带，封面不带；本服务会按上传类型再校验一遍），头 `X-Signature` 为请求体的 HMAC-SHA256（十六进制，密钥同 `FORUM_RESOURCE_SECRET`）；
  4. 可用 `GET /packs/upload-status?file_key=` 查询自己的验证状态（其他用户和不存在的 key 均返回 404）；验证完成后 `POST /packs {kind,title,summary,tags,file_key,cover_key?}` 发布，清单以资源机回调里的为准，客户端不再自报。封面同样先申请 `kind:"cover"` 的凭证（≤ 2MB，键为 `covers/{id}`，公开可读，不签名），`cover_key` 必须是自己上传并已确认的封面。只认已被资源机确认、属于该用户、没用过的上传；发布后是 T2 并进审核队列。`POST /packs/{id}/versions {file_key,changelog}` 发新版本（id 必须一致，已审核的会降回未审并重新入队）。
- 资源机侧约定：下载地址 `{base}/{file_key}?exp=&sig=`，`sig = hex(HMAC-SHA256(secret, "GET\n{file_key}\n{exp}"))`；上传凭证是 `base64url(JSON声明).hex(HMAC-SHA256(secret, "PUT\n{base64url部分}"))`，声明含 `uid, kind, size, sha256, file_key, exp`。`src/content.rs` 里的 `verify_download`、`verify_upload_ticket`、`sign_callback` 是资源机实现的参考，有单元测试。
- 我的作品：`GET /me/packs`（自己发布的全部内容包，含下架、被拒与是否在审核中）、`POST /packs/{id}/remove`（作者自己下架）。发布时必须带 `rights_ack: true`（权利声明）。网页入口是 `/publish`（上传 zip 与封面、填信息、勾选声明；`?pack=编号` 发新版本）和 `/me/works`。
- 后台（管理员令牌）：`GET/POST /admin/packs`（POST 是官方直接登记 T0/T1，文件已由运维放进资源机，可带魔搭等 https 镜像）、`POST /admin/packs/{id}/mirrors`（给某个版本补官方镜像，同名替换，最多 5 个；镜像地址不需要被信任，客户端对每个来源都校验 sha256）、`POST /admin/packs/{id}/review`（`approve|reject|remove|restore`）、`GET /admin/moderation`（待审队列）、`PUT /admin/projects/{slug}`、`PUT /admin/weekly/{week}`（草稿不对外，周刊条目只存引用，下架的内容包读取时自动隐去）。
- 集成测试：`TEST_DATABASE_URL=postgres://… cargo test content_tests` 覆盖凭证、回调、发布、版本、审核、下载签名、配额、周刊和分享。

### 应用广场后端（运行时协议 2，2026-11-08，本地实现，尚未部署）

规范见 [docs/architecture/app-runtime/README.md](../../docs/architecture/app-runtime/README.md)。本轮只做内容包后端，不含 `app.xiangban.xyz` 文件服务、域名封禁和断点续传。

- 新增内容类型 `app`，与 `game` 一样不是纯数据；`game` 和 `app` 的体积上限提高到 100MB（凭证、官方登记都按此校验），其他类型不变。收藏筛选和埋点维度同步认 `app`。
- 清单带 `runtimeProtocol: 2` 时按规范第 3.3 节校验（`schemaVersion` 为 3、`kind` 为 game/app、名称 ≤ 24 字、简介 1～80 字、`category`/`orientation`/`input`/`capabilities` 取值、`icon`/`cover` 必填、`windows`/`pointer` 只给 `layer: "desktop"`）；没有该字段的按协议 1 原规则，不变。协议号记在 `pack_versions.runtime_protocol`（默认 1）。
- `packs` 新增 `allow_remix`（默认 true）、`remix_of_pack_id`、`remix_of_version`、`remix_of_author_uid`（创建时一并记下原作者，原作品被删后仍署名）、`network_disabled`。列表和详情返回 `runtime_protocol`（当前版本）、`allow_remix`、`remix_of`（`{pack_id, version, author}` 或 null）、`remix_count`（已上架的改编数）、`network_disabled`。
- `POST /packs` 可带 `allow_remix`、`remix_of_pack_id`、`remix_of_version`（不传取原作品当前版本）。原作品不存在、未上架、版本不存在返回 400，不允许改编返回 403。
- `GET /packs` 默认不含改编作品，`remix_of=<pack_id>` 时只返回改编自该作品的作品。
- 请求头 `X-Xiaoban-Runtime`（如 `1,2`，缺失或读不出按 `1`）：列表过滤掉当前版本协议不在集合内的作品，详情对这类作品返回 404。下载接口不看这个头。
- `POST /admin/packs/{id}/network {disabled, note?}`：管理员令牌，开关“禁止联网”并写审计，不影响审核状态。
- 创作者精选：`packs.featured_at`（空表示未精选）。列表和详情返回 `featured`（布尔）和 `featured_at`；`GET /packs?featured=true` 只列出被精选的作品，按 `featured_at DESC, id DESC` 排序（覆盖 `sort`），可与 `kind`、`X-Xiaoban-Runtime` 等筛选叠加，不传或 `false` 不过滤。`POST /admin/packs/{id}/featured {featured, note?}`：管理员令牌，只允许已上架（`published`）的作品（否则 400，未知作品 404），设置时间取当前时间（重复精选会刷新时间），写审计 `pack.featured`。与信任等级无关，不影响可见性。
- 可见性不看信任等级：列表、详情、下载对 game/app 都不按 `trust_level` 过滤，T2 未审作品同样可见可下载。
- 验证：`TEST_DATABASE_URL=… cargo test --manifest-path services/server-forum/Cargo.toml content::`（清单单元测试）和 `app_plaza`（体积、发布、改编、协议过滤、后台开关的集成测试）。全量测试里有 5 个与本改动无关的既有失败（`db::tests` 三个板块种子断言、`discover_tests::topics_carry_images_polls_and_help_stats`、`points::tests::purchases_are_atomic_...`），改动前同样失败。

## 创作作品互动（2026-11-01）

作品与方案共用论坛 `topics` / `posts` / `likes`，评论沿用正文审核、附件、编辑、举报、每日配额与回复通知。`GET /api/content-social/{pack|scheme}/{id}/discussion` 允许游客读取真实讨论编号、赞/评论数及本人状态，不创建数据；首次主动互动由登录用户 `POST` 同一路径幂等建立关联，然后使用既有帖子回复和点赞接口。旧作品已有 `topic_id` 时直接复用；导入作品没有作者时由明确标注的“社区作品讨论”身份承载，不把首位评论者当作作者。

关联表 `content_discussions` 随启动追加。自动创建的讨论不重复出现在普通帖子列表；作品下架、作者封禁或讨论被隐藏后，直接访问原帖子/回复/点赞入口也会校验作品可见性。作品点赞及计数在事务内更新，重复点赞不重复计数；同一读者对同一作品的赞只提醒作者一次，评论通知可返回作品。内容包列表增加兼容字段 `like_count` / `reply_count`，不把应用次数当点赞。

针对性验证：`TEST_DATABASE_URL=… cargo test --locked --manifest-path services/server-forum/Cargo.toml content_social`；测试使用隔离 schema，覆盖游客读取不写入、首次互动并发、真实评论/点赞/通知、旧讨论复用、导入作品、方案、锁定/下架/禁言及原收藏举报。

## 手机端社区的发现页

手机端社区（首页五个栏目、AI 工具、资料、求助悬赏）用的接口，实现在 `src/discover.rs`（内容校验与示例种子）、`src/handlers/discover.rs`（公开读取与兼容管理接口）、`src/handlers/discover_editor.rs`（编辑与审核）。公开内容存在 `discover_items`（`kind` + `key` + JSON `data`），编辑副本存在独立的 `discover_drafts`；保存副本不会改变线上内容。写入时按类型校验、只保留已知字段，草稿及内容日期未到的条目不对外。

- 前台：`GET /discover/home`（头图、当天日报、精选创作、热榜前三，一次返回）、`/discover/daily`（往期日报，分页）、`/discover/repos[?issue=]` 与 `/discover/repos/issues`（开源热榜及往期）、`/discover/models`（模型榜，分档阈值 `tiers` 由后端给）、`/discover/tracker`（额度重置、模型上新、限时免费，进度和剩余天数服务端算）、`/discover/nav`、`/discover/docs`、`GET /tools[/{key}]`（AI 工具与按系统的安装包、更新记录）、`GET /help/stats`（进行中、本周已解决、悬赏积分合计，全站值）、`PUT /topics/{id}/poll/vote {option}`（每人一票）。
- 帖子：发帖可带 `poll: {options: [2～6 项], days?}`；列表与详情返回 `images`（正文里前 3 张站内图）、`poll`（票数、总数、自己投的项）、`view_count`；`GET /topics?category=a,b` 可以同时指定几个分类。
- 兼容后台接口：`GET /admin/discover/{kind}`、`PUT /admin/discover/{kind}/{key}`（`{data, sort_order?, status?, published_at?, sample?}`）、`DELETE`，仍允许管理员直接编辑线上内容。新管理台统一走下述草稿流程。类型：`banner news nav doc vendor model repo_issue quota launch promo tool hotword`，管理台已提供热门搜索编辑。`sample: true` 的条目手机端会标「示例数据」；第一次启动会写入一批示例（模型榜、热榜、日报等，导航和资料是真实官方入口），录入真实内容后可删除示例。
- 已实现服务端自动采集与管理台 `/admin/collection`：Artificial Analysis 能力榜、OpenRouter 每日 Token 用量榜、GitHub 候选仓库每日快照、官方 RSS 日报、GitHub Releases 工具版本。各来源独立开关，默认停用；第三方 key 在后台验证后用独立主密钥加密入库，只返回配置状态与尾号。`FORUM_COLLECTION_MASTER_KEY` 是部署用主密钥，不是第三方 API key。
- 能力榜、调用量榜和开源热榜每天北京时间 12:00 采集，验证通过后原子自动发布，无需人工审核；公开接口始终读取数据库快照，页面刷新不会请求原厂。资讯与工具版本仍生成草稿，管理员核对后整批发布。失败保留上一版；人工编辑与定时稿不被覆盖。来源配置、运行、批次预览/审核接口在 `/api/admin/collection`；采集令牌不能进入该管理入口。
- OpenRouter 每天北京时间 12:00 采集最近三个完整 UTC 日的 Token 用量快照，公开读接口 `/discover/model-usage[?date=]` 与 `/discover/model-usage/dates`，精确整数以字符串传输。能力榜 v2 为 `/discover/models/v2`，缺失指标为 null；旧模型接口兼容保留。
- 开源热榜只记录明确配置的最多 30 个仓库；第一天是基线，满八个连续日边界才能形成完整七日净增，缺采样标明观测天数。日报仅采 OpenAI/DeepMind/Google Research 固定官方 RSS 标题、来源、链接和时间，北京时间每天最多十个新候选，不抓正文。
- 工具来源最多十个 GitHub 仓库，只收正式版本与可识别主程序资产。官方摘要优先，缺摘要才在后台有界流式下载核验；128 MiB/文件、累计 256 MiB/次，不执行文件。未识别架构与超限资产提示人工维护。模型上新、限免、额度重置、导航与资料继续人工编辑，不推断全局实时额度。
- 集成测试：`TEST_DATABASE_URL=postgres://… cargo test --locked --manifest-path services/server-forum/Cargo.toml discover`。未设置 `TEST_DATABASE_URL` 会跳过数据库测试；测试使用独立 schema，覆盖原有接口、草稿隔离、批量事务、作用域与撤销、并发版本冲突、审核和定时发布。

### 发现页编辑与发布

管理台「发现页」提供编辑与审核、线上内容、批量导入、采集令牌四个入口，JSON 校验预览显示服务端规范化的内容，并可对照线上内容；手机页面的最终排版仍在客户端负责。日报支持 `summary`（摘要）和 `body`（Markdown 正文，可含图片），来源链接作为参考。官方 RSS 采集仍只保存元数据，不复制来源正文；编辑者须自行补充、核实有权发布的站内正文并审核。历史纯链接条目会明确显示正文暂缺，不会伪造文章。

日报详情为 `GET /api/discover/daily/{key}`；`/api/discover/daily/{key}/comments` 支持公开分页阅读和登录评论，评论可回复、点赞和由作者删除。只允许对已发布文章操作，写入沿用账号状态、屏蔽与敏感词检查，并限制频率；命中复核词时要求修改后再发布。数据库在启动时创建评论表。

下表所有管理路径前缀为 `/api/admin/discover-workflow`。采集工具使用 `/api/discover-editor` 前缀，只允许前四项；其他操作要求管理员令牌。

| 方法和路径 | 说明 |
| --- | --- |
| `GET /drafts?kind=&status=&page=` | 列出副本（每页 50 条）；状态 `draft`、`scheduled`、`published` |
| `GET /drafts/{kind}/{key}`、`PUT /drafts/{kind}/{key}` | 读取副本及当前线上内容、保存草稿；PUT 体见下例 |
| `POST /drafts/import` | `{items:[{kind,key,...草稿字段}]}`，每批 1–100 条，可跨类型；校验、权限或版本冲突导致整批回滚 |
| `PUT /preview/{kind}` | `{data,sample?}`，只校验并返回规范化预览，不写库 |
| `POST /drafts/{kind}/{key}/review` | `{expected_revision,publish_after?}`；人工终审后立即发布，或指定未来时间 |
| `POST /drafts/{kind}/{key}/cancel` | `{expected_revision}`，取消定时计划并退回待审 |
| `POST /drafts/{kind}/{key}/discard` | `{expected_revision}`，丢弃编辑副本，线上内容保留 |
| `GET/POST /tokens`、`DELETE /tokens/{id}` | 生成、查看元数据、撤销发现页采集令牌；POST 为 `{name,kinds:[...],days}` |

草稿请求示例（新建 `expected_revision: 0`，更新必须用读取到的 `revision`）：

```json
{
  "data": {"title": "官方公告标题", "source": "官方", "url": "https://example.com/announcement"},
  "expected_revision": 0,
  "sort_order": 0,
  "sample": false,
  "source_name": "官方公告",
  "source_url": "https://example.com/announcement"
}
```

`published_at` 可选，表示内容日期；不填时新条目取发布时刻、已有条目沿用线上日期。`publish_after` 是审核后的实际发布时间，两者独立。调度器每 30 秒检查一次，到期执行已终审版本；重启后继续执行，多实例和重试不会重复发布。重新保存会撤销此前审核与定时计划，版本号不会因丢弃重建而复用。线上内容在保存草稿之后变化会阻止发布；旧管理接口改动会立即撤销定时计划，其他修改导致的冲突在到期时退回待审并记录原因。管理员须对照最新线上内容合并后重新保存、审核。

采集令牌按 `kinds` 限定读写范围，强制只写草稿，禁止标为示例，不能发布、删除或操作其他管理接口。有效期 1–365 天，明文只在生成时返回，数据库仅保存元数据；修改 `FORUM_ADMIN_JWT_SECRET` 会使所有采集令牌失效。所有草稿、终审、发布、撤销和凭据管理写入审计，审计不记录令牌或内容正文。

## 合规项：敏感词与待审队列

- 敏感词由管理员在后台维护（`/admin/words`），**不内置任何词**。词表在发帖、编辑、回复、发布内容包与新版本、发布方案时检查：命中“拦截”的直接 400，且不告诉用户是哪个词；命中“复核”的先发布并进入待审队列。匹配前先归一化（转小写、去掉空白和标点），所以“违 禁-词”也命中“违禁词”；每个词 2～40 个字。实现见 `src/moderation.rs`。
- 待审队列（`/admin/moderation`）汇集用户新发布的内容包和命中复核词的帖子、回复、方案；内容包与方案可直接通过或拒绝，帖子和回复要到帖子管理里隐藏，再标记“已处理”（`POST /admin/moderation/{id}/resolve`）。

## 屏蔽用户

`PUT/DELETE /users/{uid}/block`、`GET /me/blocks`。屏蔽后互相取消关注；被屏蔽者的帖子不再出现在屏蔽者的列表、搜索和关注流里，回复折叠（接口返回 `blocked: true` 且不带正文）；对方看不出自己被屏蔽（接口不报错、其他人看到的不变）。个人主页返回 `blocked`，设置页列出屏蔽的人并可取消。通知（见下节）也会跳过被屏蔽者带来的事件。

## 站内通知

`GET /me/notifications?page=&unread=`、`GET /me/notifications/unread`（未读数）、`POST /me/notifications/read`（带 `{ids:[…]}` 标记指定几条，不带请求体全部已读）。事件：别人回复了你的帖子（`reply`）、回复了你的楼层（`reply_to`，和 `reply` 不重复发）、正文里 `@用户名` 提到了你（`mention`，一条最多提 5 人，发帖和回复都算，编辑不重复通知）、你的回答被采纳（`accepted`）、你的内容包审核通过或被拒（`pack_approved`/`pack_rejected`，下架与恢复不通知）。通知在业务事务里写入；自己对自己不通知；屏蔽了对方的人不收到对方带来的通知。标题和摘要是写入时的快照；已读超过 90 天的由定时任务清理。只做站内，没有邮件和系统推送。前台顶栏有铃铛和未读数（登录后每分钟刷新），`/notifications` 是列表页，打开即标记已读。

## 匿名埋点

`POST /events`（匿名，`{device, events:[{name, dim?, count?}]}`，成功一律 204，不合格的事件静默丢弃）。依据 PRD §8.3：设备标识由客户端随机生成，服务端只存加盐 SHA-256（`FORUM_ANALYTICS_SALT`，生产请设置）；事件名和维度（内容类型或来源）都来自白名单（`src/analytics.rs`），不收账号、内容、搜索词、文件名；只存“某设备某天某事件几次”（表 `events_daily`），不保存行为流。后台「数据」页（`GET /admin/metrics`）看每日活跃设备、事件次数与独立设备数、次日留存。网页端在设置页可一键关闭（同时删除本地设备标识），浏览器开启“不跟踪”也不上报；桌面端在社区页关闭。隐私政策已披露。事件：看首页/周刊/项目/内容包/讨论、搜索、点“在小伴中打开”、下载、复制链接、发布开始与完成，以及桌面端的社区页打开、应用确认、成功、取消、方案复刻。暂未做 IP 限流，量大后再加。

## 分享海报

内容包详情、周刊和帖子页都有“生成海报”：在浏览器里用 canvas 画一张竖版图（封面或渐变底、类型标签、标题、摘要、品牌和指向该页的二维码），不经过服务器，可保存或复制链接，手机上长按图片也能保存。二维码用 `web/src/lib/vendor/qrcode.js`（qrcode-generator 1.4.4，MIT，仅把出口改成 ES 模块），已用 OpenCV 解码核对内容一致。`npm run test:poster` 覆盖折行与二维码矩阵。封面图跨域读取需要资源机允许跨域（已默认放开）。

## 业务规则

- 正文是 Markdown，服务端用 pulldown-cmark 渲染、ammonia 清洗后存 `body_html`；外链加 `nofollow ugc`。新人（TL0）的链接和图片会被去掉，只留文字。
- 内置标签：二级分类可带一组内置标签（`categories.preset_tags`，追加列，已部署的库启动时自动补上）。发帖选了分类后标签栏下面出现"常用标签"，点一下加上；板块页顶部用同一组标签筛选（`GET /topics?category=&tag=`）。标签和用户自己输入的标签是同一种写法（小写、空白换成连字符，由 `topics::normalize_tag` 统一），后台分类表单可以改。种子里只给资源分享的四个二级板块配了标签。
- 技术交流板块：AI 大模型、AI 提效、AI 硬件、AI 创意。随便唠唠下有「亏了吗」（板块说明和站规写明不荐股、不拉群、不带单）和「梦想和人生」。AI 大模型的板块说明和站规都写明：自建中转站的技术教程可以发，运营、出售、推广中转服务仍然禁止。
- 板块调整的迁移：已部署的库里旧的二级板块由 `db::apply_board_migration` 在启动时一次性并入新板块，做完记进 `data_migrations` 表，不再重复，所以后台之后改过甚至重新建过同名 slug 的分类不会被动到。资源分享（`share-boards-v1`）、技术交流（`tech-boards-v1`、`tech-creative-v1`）、随便唠唠（`casual-loss-v1`、`casual-dream-v1`）、小伴产品（`xiaoban-boards-v1`，`create_parent` 新增一级分类，插在资源分享的子树后面）各自一条；每条只写入自己列出的板块（名称、说明、标签按种子覆盖，已有分类的名称和说明不动），新板块插在种子里前一个板块的后面，其后的分类整体顺延一位，后台改过的排序保持相对顺序；技术交流里 AI 应用开发、前端·后端·移动端 → AI 提效，服务器与运维、模型部署 → AI 大模型。新库种子直接是最新结构并把这些迁移记为已完成。
- 发帖等级：分类上有 `min_post_level`（最低发帖等级）和 `admin_only`（仅管理员可发）两个开关，实际需要的等级由 `topics::required_post_level` 统一算：`admin_only` 的分类要守护者（TL3），其余看 `min_post_level`。管理后台账号和论坛用户是两套体系，后台账号发不了帖，所以「仅管理员」指的就是守护者（站务人员）；站务下的公告、规则、举报处理公示因此由守护者发布。发帖和编辑帖子时换分类都用这一个函数检查（以前编辑时可以绕过发帖等级，已修）；留在原分类编辑总是允许，所以被降级的作者仍能改自己发的公告。前端对应 `lib/format.ts` 的 `requiredPostLevel`。
- 小伴产品板块：站方推广自家产品的官方板块，二级板块是小伴壁纸、小伴游戏、小伴App，各自带内置标签（更新日志、下载、使用教程等）。小伴App 和一级分类 `min_post_level = 3`，只有守护者能发帖；小伴壁纸、小伴游戏 `min_post_level = 0`，所有人（含新人）都能发，用来分享用户自己做的 `.xiaoban` 资源包（新人每天最多 2 帖，可上传附件，链接和图片仍显示为文字），已部署的库由 `xiaoban-share-boards-open-v1` 迁移一次性放开。所有人可以回复。发帖页和板块页对等级不够的用户隐藏这些板块和按钮。以后加产品：在 `CATEGORIES` 里往 `xiaoban` 下加一行二级板块（同样 `min_post_level = 3`），再加一条 `BoardMigration { parent: "xiaoban", create_parent: false, boards: &[新板块 slug], moves: &[] }`，需要标签就在 `PRESET_TAGS` 加一项。
- 资源分享板块：精品软件、电影资源、音乐资源、学习资料，只收合法来源的内容（板块说明和站规里写明，禁止盗版下载）。已部署的库里旧的四个二级板块（开源项目、工具软件、提示词·工作流、自制作品）由上面的迁移并入：开源项目、工具软件、自制作品里的帖子归到精品软件，提示词·工作流归到学习资料；
- 信任等级与阅读权限：用户分 TL0 新人、TL1 成员、TL2 老用户、TL3 守护者。每个帖子有 `min_read_level`（0 所有人含游客，1 成员以上，2 老用户以上，3 守护者），发帖人只能设到不超过自己的等级，编辑时可以保持或调低，不能调高到超过自己；不传按公开。读者按登录态确定（游客是 0 级），作者总能读自己的帖子。受限帖子在列表、搜索、标签页、用户主页（帖子、回复摘要、统计）、收藏夹和搜索引擎页面里对没权限的人完全不可见，直接打开返回 403 并说明需要什么等级；回复列表、回复、点赞、收藏、举报同样检查。分类、产品、标签上的帖子数只统计公开帖。`sitemap.xml` 不收录受限帖，受限帖的搜索引擎页面按不存在处理（404 + noindex）。后台能看到所有帖子及其阅读权限。读者身份在 `models::Reader`，`ListFilter.reader` 用 `serde(skip)` 防止被查询参数伪造，默认值是游客，所以忘记设置的查询只会看到公开内容。
- 帖子附件：发帖和编辑页的「附件」一栏，先逐个上传（`POST /attachments`），再用 `attachment_ids` 随发帖或编辑挂到帖子上，一个帖子最多 5 个。单文件上限看信任等级：成员及以上（TL1-TL2）5 MB，守护者（TL3，也就是站务人员，后台账号发不了帖）30 MB，新人不能上传。文件落在 `FORUM_UPLOAD_DIR/{id}`，库表 `attachments` 只记文件名和大小；上传后一天内没挂到帖子上的由定时任务清理，编辑时移除的附件立即删文件。下载一律按 `application/octet-stream` + `attachment` 输出，下载权限和读帖子一致（阅读等级不够返回 403，帖子被隐藏或删除返回 404，没挂上的只有上传者能取），所以站内不会直接打开用户传的 HTML。阅读等级受限的帖子前端用带令牌的 fetch 下载，公开帖子是普通链接。只有主楼支持附件，回复不支持；搜索引擎页面不列附件。部署时 Nginx 的 `client_max_body_size` 要调到 31m 以上，否则上传会被代理直接 413。
- 置顶：守护者（TL3）在帖子页的操作栏能看到「置顶 / 取消置顶」，只有正常状态的帖子可以置顶；管理后台 `/admin` 的帖子管理里原来就有同样的开关，两边改的是同一个 `topics.pinned`。置顶的帖子在所属板块和首页的列表里排在最前面（不管按哪种方式排序），列表里带「置顶」标记；搜索、标签页和用户主页不受影响。前台守护者的操作同样记进审计日志，操作人显示为「守护者:昵称」。
- 正文插图：编辑器工具栏的「插图」按钮，也可以直接粘贴截图或把图片拖进输入框（发帖、编辑、回复都一样）。上传后得到站内地址 `/api/images/{id}`，自动插进 Markdown 的 `![](…)`。只收 PNG、JPEG、GIF、WebP，服务端按文件头判断真实类型（不信文件名和客户端类型），不收 SVG；每张不超过 `FORUM_UPLOAD_MAX_MB_IMAGE`，成员及以上才能传（新人的图片本来就会被渲染成纯文字），每人每天最多 50 张。文件放在 `FORUM_UPLOAD_DIR/img-{id}`，库表 `images`。图片地址靠不可猜的 id 公开访问，不看帖子的阅读等级，响应带 `nosniff` 和沙箱 CSP，长期缓存。上传一天后帖子和回复的 Markdown 里都搜不到它的 id 的，定时任务每小时清理一次。Nginx 要给 `/api/images` 单独放宽 `client_max_body_size`（6m 以上）。后续接腾讯云 COS 时，只需要换这两个接口的存储。
- 活动链接不允许带 `ref`、`aff`、`invite` 等个人推广参数。
- 只做站内功能：没有系统推送、邮件提醒，也没有匿名发帖（按产品决定）。

## 已知限制与后续

- 数据库兼容：阅读权限是追加的列（`ALTER TABLE topics ADD COLUMN IF NOT EXISTS min_read_level ... DEFAULT 0`），已部署的库启动时自动补上，旧帖子都按公开处理。
- 二期（PRD 里的 M2 起）未做：关键词订阅、TL2 自动升级、图片上传到腾讯云 COS、赞助位。积分规则见上面的运行与验证；正文插图已经走本机磁盘（见业务规则），COS 是二期的存储替换。
- 账号中心还没有用户自助注销，设置页提示发邮件处理。
- SEO 页面只渲染第一页（30-50 条），sitemap 每次请求实时查询，没有进程内缓存。
- 部署：香港机 `/home/ubuntu/xiangban-forum`，systemd 服务 `server-forum`，监听 127.0.0.1:55010，Nginx 反代 chat.xiangban.xyz（Let's Encrypt），库 `xiaban_forum` 在 `wallpaper_postgres` 容器里，独立角色 `forum`。账号中心已登记 `forum-web` 并在 `ACCOUNT_CORS_ORIGINS` 加入 `https://chat.xiangban.xyz`。初始管理员密码在服务器 `.env` 的 `FORUM_ADMIN_PASSWORD`，登录 /admin 改密码后删掉它。更新时本地交叉编译后替换二进制和 `web/dist`，再重启服务（index.html 启动时读一次）。手机应用的令牌（`aud=xiaoban-android`）由 `FORUM_ACCOUNT_EXTRA_CLIENT_IDS` 接受（默认已含，账号中心要登记该客户端），见 [手机互联上线清单](../../docs/deployment/MOBILE_LINK_ROLLOUT.md)。

## 站方图文创作与批量发帖

后台新增 `/admin/editor-accounts`（编辑账号与本地令牌）和 `/admin/editor`（批量发帖）。统一账号中心后台的用户页也支持“创建用户”。站方账号用用户名和密码正常登录，不会被自动标为拥有已验证邮箱/手机号，后续可自行绑定。

- **创建新编辑账号**：账号中心设置 `ACCOUNT_FORUM_PROVISION_KEY`、论坛设置相同的 `FORUM_ACCOUNT_PROVISION_KEY`（至少 32 字节随机值，例如本机 `openssl rand -hex 32` 生成）。两项都是服务端环境变量，不写进前端或本地 Skill。账号中心与论坛均需更新并重启。论坛通过受限的 `/api/internal/forum/users` 建号；该凭据不能管理已有用户或获取用户会话。
- **绑定已有账号**：用户先正常登录过论坛，再在编辑账号页填 UID 绑定；须为正常状态且至少 TL1。绑定后论坛展示名作为编辑资料保留，不被登录同步覆盖。官方板块仍要求 TL3，批量发布重新核对作者状态和板块权限。
- **导入**：每篇 `.md` 带 YAML 元信息 `id/title/category/tags`，图片放在该文件旁的 `images/` 并用相对路径引用。`category` 使用真实 slug。一个 ZIP 可含 1–50 篇；单个 `.md` 用于纯文字稿。ZIP 不超过 100 MB，解压后不超过 80 MB、最多 500 个内容文件。每篇最多 20 张图片、共 20 MB；后台上传适配默认每张 5 MB 的限制。浏览器解包，逐篇调用同一导入接口；可选择作者、覆盖板块、更新已有待发布稿。部分失败不回滚成功项。
- **待发布稿**：导入不产生前台帖子。稿件与图片随数据库/上传目录一起持久化；图片清理任务会保留稿件引用的图片，待发布图片公开接口返回 404。编辑预览通过鉴权接口返回内嵌配图，使用论坛同一 Markdown 渲染器。保存后才更新预览。
- **发布与去重**：同作者下稿件 `id` 唯一；默认拒绝重复导入，显式覆盖只允许更新待发布稿。逐篇发布在事务中写正文、标签、分类扩展、图片公开状态和记录，再返回正式链接；重试相同稿件只返回已有帖子。已发布稿不能通过再次导入覆盖，走论坛原有编辑入口。发布的是公开帖，暂不提供付费附件、阅读等级、定时发布等字段，额外信息走现有编辑入口。
- **帖子封面**：发布时从正文提取前 3 张站内图片供列表展示；采编稿需在正文提供可识别的站内图片。升级时只对站方已发布且列表配图为空的旧稿执行一次补全，不覆盖已有配图。
- **权限与审计**：站方专用通道不受普通用户每日发帖/图片次数限制，但保留作者禁言封禁、最低等级和板块权限检查，所有导入/发布写后台审计。不会因导入或发布额外发注册、签到积分，不制造点赞或浏览量。
- **创作令牌**：后台生成，仅显示一次，固定绑定一个作者；默认只允许导入，可选直接发布，有效期 1–365 天，可撤销。令牌不能访问管理接口，不存明文；签名使用 `FORUM_ADMIN_JWT_SECRET`，生产必须稳定配置，改密钥会让全部创作令牌失效。

本地 Claude Skill 位于 [`tools/claude-skills/forum-post-studio`](../../tools/claude-skills/forum-post-studio)，将完整目录复制到 `~/.claude/skills/`，按其中引用说明安装依赖，设置 `FORUM_EDITOR_TOKEN`，即可选题、创作、打包、导入并按授权发布。`FORUM_URL` 默认 `https://chat.xiangban.xyz`。帖子目录只放 Markdown 和图片，报告放目录外。

API：

| 路径 | 凭据与用途 |
| --- | --- |
| `GET/POST /api/admin/editor/authors` | 管理员：列出/创建编辑账号 |
| `POST /api/admin/editor/authors/bind` | 管理员：绑定已有论坛用户 |
| `GET/POST /api/admin/editor/tokens`、`DELETE /api/admin/editor/tokens/{id}` | 管理员：生成、查看元数据、撤销本地令牌 |
| `GET /api/editor/context` | 管理员或创作令牌：板块、作者及权限 |
| `GET/POST /api/editor/drafts` | 分页列表/导入结构化稿件（浏览器和 CLI 负责 ZIP/YAML 解析） |
| `GET/DELETE /api/editor/drafts/{id}` | 读取带配图的预览/删除未发布稿 |
| `POST /api/editor/drafts/{id}/publish` | 发布；限定令牌需拥有发布权限 |

结构化导入包含 `author_uid/external_id/title/category/body/tags/images`，其中 `images` 是 `{path,data_base64}` 列表（仅用于传输，正式 Markdown 仍是站内 URL）。更新需传 `overwrite:true`，后台编辑还带 `expected_updated_at` 防止覆盖并发修改。单请求上限 30 MiB。生产 Nginx 应给 `/api/editor/drafts` 放宽 `client_max_body_size 31m`，否则图文包可能被代理拒绝；其他上传接口限制不变。

验证：`npm --prefix services/server-forum/web run test:post-bundle`、`python -m unittest discover -s tools/claude-skills/forum-post-studio/scripts -p 'test_*.py'`。数据库回归：设置隔离的 `TEST_DATABASE_URL` 后执行 `cargo test --locked --manifest-path services/server-forum/Cargo.toml handlers::editor`，涵盖作者隔离、令牌撤销、待发布图片私有性、图片保留及重复发布；账号创建回归在账号服务 `admin_and_scoped_service_create_real_users_without_fake_verified_identities` 测试中。

### 自动采编执行器

独立运行的采编 sidecar 见 [`editorial/README.md`](editorial/README.md)。队列、官方来源、事实审校、配图、幂等发布、封面回读和费用停止阈值均由服务进程处理，不依赖 Codex 定时聊天。当前只启用已配置官方来源的部分教程栏目，其余栏目及作品/目录适配器仍待接入，不能视为全部栏目已覆盖。运行 `worker.py audit` 按公开入口核对数量，既有条目数不等于质量验收数。

自动发布请求可携带 `If-Match: "<draft.updated_at>"`，在事务锁内检查已审校版本，发现并发修改返回 409；旧调用省略该头仍兼容，已经发布的稿件重试仍返回原链接。

发现页采集令牌新增可选的 `publish_kinds`（缺省为空，旧令牌仍只写草稿）。当前仅支持单独授权 `frontier`，后台令牌页展示权限并允许撤销。`POST /api/discover-editor/drafts/frontier/{key}/publish {expected_revision}` 在事务内检查精确版本、线上基线、1200 字正文、来源字段和封面属于本篇，使用真实当前发布时间；操作记录标明 `automation:`，不冒充人工终审。其他发现类型仍沿用管理员审核。

### 精选与官方挑战

管理台 `/admin/community-editorial` 支持人工选择首页精选作品、排序和挑战草稿的保存/终审发布/撤回/关闭。精选、参赛作品、内容包详情/分享及方案引用联动作者封禁、作品下架与关联讨论阅读权限；不可用引用只返回 id/版本/available=false，不返回旧标题和路径。

公开 `GET /api/challenges[/{key}]`；登录后 `POST /api/challenges/{key}/entries {pack_id}` 仅能提交自己的已发布作品，按服务端开始/截止时间判定，重复投稿幂等。人工精选与顺序更新记审计，点赞沿用作品原讨论帖的真实计数，不创造挑战票数或积分。

方案详情、列表与作者下架已补齐，网页 `/schemes`、`/schemes/{id}`、`/schemes/new` 可浏览和组装 1～20 个已发布作品的固定版本；`/me/works?tab=schemes` 管理自己的方案。分享链接匿名可读，下架后不可访问。方案只存社区对象引用，不上传私人资料、Agent 记忆或密钥。

本次实施、本地验证及未完成的生产/真机验收见 [社区验收记录](../../docs/product/验收记录_社区全功能_2026-10-31.md)。未配置真实第三方凭据的来源不能记为线上已接通。

工具镜像管理台 `/admin/tool-mirrors` 从已采集草稿按精确版本选择资产，资源机完成签名回执与回读核验后登记；登记不改变草稿或发布状态。公开 `/api/discover/tool-mirrors/{key}/{asset_id}/{sha}` 只接受已发布且发布时间已到的精确资产，资源故障回退官方来源。接入及反向代理超时见[镜像协议](../server-resource/docs/tool-mirrors.md)。

### 社区第二轮补齐（2026-10-31）

作品/能力包和方案支持独立收藏、私密举报及管理员下架联动；求助与普通回复可附最多3个公开作品/方案卡、5个受保护附件，旧客户端省略字段不会清空已有内容。下架引用仅显示不可用占位，隐藏回复附件也无法下载或购买。见 [收藏举报](docs/content-social.md)、[回复扩展](docs/reply-content.md)。

账号方案备份为本人独立对象，显式保存、版本冲突保护、软删除与回收站，和公开方案分离。桌面/手机默认关闭云端入口；备份范围与客户端限制见 [私有方案备份](docs/scheme-backups.md)。显式设置 `FORUM_ACCOUNT_EXTRA_CLIENT_IDS` 时需保留 Android 与桌面客户端 ID。

后台新增“栏目运营”和“作品举报”：栏目数字与采集新鲜度来自数据库，外链检查仅记录结果，不修改内容；定时检查默认关闭，最多每24小时运行，每次60条、4并发、120秒总时限，候选窗口和网络边界见 [外链与运营](docs/link-health-operations.md)。内部服务调度不使用 GitHub Actions。

### 站方方案发布

`POST /api/admin/schemes` 使用管理员凭据，传入已绑定的活跃站方 `author_uid`、`title`、`summary` 和 `items: [{pack_id, version}]`。复用用户方案的公开作品、版本、去重和日限额校验；发布与审计记录在同一事务中完成。手动模板的 manifest 可标记 `manualOnly: true`，网页详情提供下载和使用说明，不展示桌面安装入口。

### 游戏收藏筛选（2026-11-01）

`GET /api/me/content-bookmarks?kind=game&page=1` 筛选当前账号收藏的公开游戏作品，筛选先于分页和 total 统计；响应条目的 `kind` 继续为 `pack`，收藏/取消仍使用 `/content-social/pack/{id}/bookmark`。兼容原有 pack/scheme 查询，不增加收藏表或改变权限。针对性验证：`TEST_DATABASE_URL=... cargo test --locked --manifest-path services/server-forum/Cargo.toml game_bookmarks_filter`，覆盖登录限制、真实总数、分页、下架排除和原作品查询兼容。

## 资讯订阅（开发中，默认关闭）

账号私有订阅、周期任务及期刊档案，桌面/手机/网页统一管理入口。配置、计费边界、投递不确定状态和验证范围见 [资讯订阅](docs/information-subscriptions.md)。未部署，不读取收件箱，不接收用户本地 API Key。

开源精选正式详情页现支持默认中文 README / English 切换、真实项目卡片、来源及许可证、现有评论与浏览器收藏。逐项目发布产物位于 `web/public/open-source-readmes/`；缺少已审稿时仅提供官方原文入口。生成、校验及增量发布流程见 [README 采编规范](editorial/OPEN_SOURCE_README_WORKFLOW.md#正式网页发布路径2026-11-03-接入)。

## 社区分类与发帖归属（2026-11-06）

社区页面按首页、创意、讨论、资源组织。首页的精选、排行榜、追踪、导航属于独立目录或数据模块，不作为帖子分类。数据库分类的 `section`（ideas/discuss/resources/site）、`status`（active/archived，后台显示启用/停用）和 `posting_enabled` 是浏览、发帖及自动采编共同使用的策略；后台修改名称、归属或状态后，新打开的页面读取同一分类接口。一级分类修改归属同步子分类；停用一级分类对其全部子分类生效。讨论页提供站务入口。

资源页的开源软件是目录，开源讨论是帖子分类；影视、求助悬赏读取各自帖子。发帖按当前板块预选，停用或禁止发帖的分类不能新发、迁入或通过采编发布；原作者可编辑自己现有文章。旧分类记录 AI 工具、资料、音乐、片单乐评及产品使用已移除，关联帖子和稿件重归现行分类，文章删除状态保持不变，不保留这些旧分类入口。迁移只执行一次，之后不会覆盖后台设置。

验证：前端 `npm run typecheck`、`node --experimental-strip-types --test tests/communityStructure.test.ts tests/navigation.test.ts`；隔离 PostgreSQL 中运行 `cargo test --locked taxonomy::tests`；采编 `python3 -m unittest discover -s editorial -p 'test_*.py'`。部署前在本地构建 Linux 二进制和网页资源。
