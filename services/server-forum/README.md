# server-forum：超级小伴论坛

状态：一期（网页版）已实现，2026-10-11 部署在香港机 chat.xiangban.xyz。面向开发者和 AI 用户的社区：技术交流、资源分享、福利羊毛（羊毛帖的能领到什么、领取条件、有效期、活动页都是选填，到期自动下架）、随便唠唠和站务板块。福利羊毛不作为主推，排在技术交流和资源分享之后。站点名「超级小伴」，计划域名 `chat.xiangban.xyz`。产品设计见 [docs/product/PRD_论坛.md](../../docs/product/PRD_论坛.md)。

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

## 配置

见 [.env.example](.env.example)。关键项：

| 变量 | 默认 | 说明 |
| --- | --- | --- |
| `FORUM_DATABASE_URL` | `postgres://xiaban@localhost:5432/xiaban_forum` | 独立库 |
| `FORUM_PUBLIC_URL` | `https://chat.xiangban.xyz` | SEO 页面的 canonical 和 sitemap |
| `FORUM_ACCOUNT_ISSUER` | `https://account.xiangban.xyz` | 必须和账号中心 `ACCOUNT_ISSUER` 完全一致 |
| `FORUM_ACCOUNT_CLIENT_ID` | `forum-web` | 令牌 `aud` 必须是它，需要先在账号中心后台登记 |
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

前台路由：`/` 首页、`/deals` 福利羊毛（顶部一行分类标签，可按最新、热门、快过期排序；已到期的默认不显示）、`/c/{slug}` 分类、`/t/{id}` 帖子、`/new` 发帖、`/tag/{name}`、`/search`、`/u/{uid}`、`/settings`（收藏、昵称、等级说明）、`/rules` `/terms` `/privacy`，后台在 `/admin`。

服务端对 `/`、`/deals`、`/t/{id}`、`/c/{slug}`、`/tag/{name}` 直接输出带标题、摘要、正文和 JSON-LD 的 HTML（填进 `#app`，Vue 挂载时替换），另有 `/sitemap.xml`、`/robots.txt`。实现在 `src/seo.rs`，依赖 `web/index.html` 保留 `<title>`、`<meta name="description">` 和空的 `<div id="app"></div>`。

前台接口（`/api`，写操作要带账号中心访问令牌）：

| 接口 | 说明 |
| --- | --- |
| `GET /config` | 站点名、账号中心地址、client_id |
| `GET /categories` `/tags` | 分类树、热门标签 |
| `POST /session` | 登录后调用：同步昵称头像、记访问、检查自动升级，返回本站资料和每日上限 |
| `GET /topics` `/deals` `/search` | 列表；参数见 `models::ListFilter` |
| `GET/POST /topics`、`GET/PATCH/DELETE /topics/{id}` | 帖子；羊毛帖带 `deal` 字段 |
| `GET/POST /topics/{id}/posts`、`PATCH/DELETE /posts/{id}` | 回复 |
| | `PUT/DELETE /topics/{id}/like`、`/posts/{id}/like`、`/topics/{id}/bookmark` | 点赞、收藏 |
| `PUT/DELETE /topics/{id}/pin` | 置顶、取消置顶（仅守护者 TL3） |
| `POST /reports` | 举报帖子或回复 |
| `POST /markdown/preview` | 发帖预览 |
| `POST /images`、`GET /images/{id}` | 正文插图：上传（multipart，字段 `file`）、按 id 公开访问 |
| `POST /attachments`、`GET/DELETE /attachments/{id}` | 上传附件（multipart，字段 `file`）、下载、撤销还没挂到帖子上的上传 |
| `GET /users/{uid}`、`/users/{uid}/topics`、`/users/{uid}/posts`、`/me/bookmarks` | 用户主页、我的收藏 |

管理接口在 `/api/admin/*`（`src/handlers/admin.rs`）：管理员独立建表，HS256 令牌带 `token_version`，改密码后旧令牌失效，连续输错 5 次锁 15 分钟；包括概览、举报处理、帖子与回复管理、用户等级和禁言封禁、分类维护、操作日志，所有写操作都记审计。

## 社区内容层（内容包、方案、项目、周刊）

接口都在 `/api` 下，实现见 `src/handlers/content.rs`，纯逻辑（体积上限、清单检查、签名）见 `src/content.rs`。

- 内容包类型与体积上限：effect/scheme/reminder/routine/task/goal ≤ 1MB，widget/game/skill/workflow ≤ 8MB，expert ≤ 1MB，wallpaper ≤ 32MB。信任分级 T0 官方、T1 已审核、T2 用户未审；T2 只有 effect、scheme、reminder 返回 `one_click: true`，其余类型需要桌面端二次确认。
- 前台：`GET /packs`（`kind`、`q`、`tag`、`author`、`sort=new|hot`、分页）、`GET /packs/{id}`（含版本与清单）、`GET /packs/{id}/download?version=`（返回签名地址、`sha256`、`size`、官方镜像 `mirrors`；客户端每个来源都要校验 sha256）、`POST /packs/{id}/applied`（应用计数，匿名可调）、`GET/POST /schemes`、`GET /projects[/{slug}]`、`GET /weekly[/{week}]`、`GET /share/{kind}/{id}`（`pack|topic|scheme|project|weekly`，返回落地页地址与 `xiaoban://community/...` 深链接）。
- 用户发布流程（文件不经过本服务）：
  1. `POST /packs/upload-ticket {kind,size,sha256}`（需登录）→ `{ticket, upload_url, file_key}`；
  2. 客户端把文件和 `ticket` 直传 `upload_url`（资源机校验凭证、大小、哈希）；
  3. 资源机落盘并重算哈希后，回调 `POST /api/internal/pack-uploaded`，请求体 `{file_key,size,sha256,manifest}`（`manifest` 是资源机从压缩包根目录读出的 `manifest.json`，内容包必带，封面不带；本服务会按上传类型再校验一遍），头 `X-Signature` 为请求体的 HMAC-SHA256（十六进制，密钥同 `FORUM_RESOURCE_SECRET`）；
  4. `POST /packs {kind,title,summary,tags,file_key,cover_key?}` 发布，清单以资源机回调里的为准，客户端不再自报。封面同样先申请 `kind:"cover"` 的凭证（≤ 2MB，键为 `covers/{id}`，公开可读，不签名），`cover_key` 必须是自己上传并已确认的封面。只认已被资源机确认、属于该用户、没用过的上传；发布后是 T2 并进审核队列。`POST /packs/{id}/versions {file_key,changelog}` 发新版本（id 必须一致，已审核的会降回未审并重新入队）。
- 资源机侧约定：下载地址 `{base}/{file_key}?exp=&sig=`，`sig = hex(HMAC-SHA256(secret, "GET\n{file_key}\n{exp}"))`；上传凭证是 `base64url(JSON声明).hex(HMAC-SHA256(secret, "PUT\n{base64url部分}"))`，声明含 `uid, kind, size, sha256, file_key, exp`。`src/content.rs` 里的 `verify_download`、`verify_upload_ticket`、`sign_callback` 是资源机实现的参考，有单元测试。
- 后台（管理员令牌）：`GET/POST /admin/packs`（POST 是官方直接登记 T0/T1，文件已由运维放进资源机，可带魔搭等 https 镜像）、`POST /admin/packs/{id}/review`（`approve|reject|remove|restore`）、`GET /admin/moderation`（待审队列）、`PUT /admin/projects/{slug}`、`PUT /admin/weekly/{week}`（草稿不对外，周刊条目只存引用，下架的内容包读取时自动隐去）。
- 集成测试：`TEST_DATABASE_URL=postgres://… cargo test content_tests` 覆盖凭证、回调、发布、版本、审核、下载签名、配额、周刊和分享。

## 业务规则

- 正文是 Markdown，服务端用 pulldown-cmark 渲染、ammonia 清洗后存 `body_html`；外链加 `nofollow ugc`。新人（TL0）的链接和图片会被去掉，只留文字。
- 内置标签：二级分类可带一组内置标签（`categories.preset_tags`，追加列，已部署的库启动时自动补上）。发帖选了分类后标签栏下面出现"常用标签"，点一下加上；板块页顶部用同一组标签筛选（`GET /topics?category=&tag=`）。标签和用户自己输入的标签是同一种写法（小写、空白换成连字符，由 `topics::normalize_tag` 统一），后台分类表单可以改。种子里只给资源分享的四个二级板块配了标签。
- 技术交流板块：AI 大模型、AI 提效、AI 硬件、AI 创意。随便唠唠下有「亏了吗」（板块说明和站规写明不荐股、不拉群、不带单）和「梦想和人生」。AI 大模型的板块说明和站规都写明：自建中转站的技术教程可以发，运营、出售、推广中转服务仍然禁止。
- 板块调整的迁移：已部署的库里旧的二级板块由 `db::apply_board_migration` 在启动时一次性并入新板块，做完记进 `data_migrations` 表，不再重复，所以后台之后改过甚至重新建过同名 slug 的分类不会被动到。资源分享（`share-boards-v1`）、技术交流（`tech-boards-v1`、`tech-creative-v1`）、随便唠唠（`casual-loss-v1`、`casual-dream-v1`）、小伴产品（`xiaoban-boards-v1`，`create_parent` 新增一级分类，插在福利羊毛的子树后面）各自一条；每条只写入自己列出的板块（名称、说明、标签按种子覆盖，已有分类的名称和说明不动），新板块插在种子里前一个板块的后面，其后的分类整体顺延一位，后台改过的排序保持相对顺序；技术交流里 AI 应用开发、前端·后端·移动端 → AI 提效，服务器与运维、模型部署 → AI 大模型。新库种子直接是最新结构并把这些迁移记为已完成。
- 发帖等级：分类上有 `min_post_level`（最低发帖等级）和 `admin_only`（仅管理员可发）两个开关，实际需要的等级由 `topics::required_post_level` 统一算：`admin_only` 的分类要守护者（TL3），其余看 `min_post_level`。管理后台账号和论坛用户是两套体系，后台账号发不了帖，所以「仅管理员」指的就是守护者（站务人员）；站务下的公告、规则、举报处理公示因此由守护者发布。发帖和编辑帖子时换分类都用这一个函数检查（以前编辑时可以绕过发帖等级，已修）；留在原分类编辑总是允许，所以被降级的作者仍能改自己发的公告。前端对应 `lib/format.ts` 的 `requiredPostLevel`。
- 小伴产品板块：站方推广自家产品的官方板块，二级板块是小伴壁纸、小伴游戏、小伴App，各自带内置标签（更新日志、下载、使用教程等）。小伴App 和一级分类 `min_post_level = 3`，只有守护者能发帖；小伴壁纸、小伴游戏 `min_post_level = 0`，所有人（含新人）都能发，用来分享用户自己做的 `.xiaoban` 资源包（新人每天最多 2 帖，可上传附件，链接和图片仍显示为文字），已部署的库由 `xiaoban-share-boards-open-v1` 迁移一次性放开。所有人可以回复。发帖页和板块页对等级不够的用户隐藏这些板块和按钮。以后加产品：在 `CATEGORIES` 里往 `xiaoban` 下加一行二级板块（同样 `min_post_level = 3`），再加一条 `BoardMigration { parent: "xiaoban", create_parent: false, boards: &[新板块 slug], moves: &[] }`，需要标签就在 `PRESET_TAGS` 加一项。
- 资源分享板块：精品软件、电影资源、音乐资源、学习资料，只收合法来源的内容（板块说明和站规里写明，禁止盗版下载）。已部署的库里旧的四个二级板块（开源项目、工具软件、提示词·工作流、自制作品）由上面的迁移并入：开源项目、工具软件、自制作品里的帖子归到精品软件，提示词·工作流归到学习资料；
- 羊毛帖：能领到什么、领取条件、有效期、活动页全部选填，什么都不填也能发；没有投票和状态标记，写了有效期的到期后自动不再出现在福利羊毛、首页和分类列表（用户主页、搜索、标签页、产品页的历史仍保留）。每 10 分钟重算最近 7 天的热度。
- 信任等级与阅读权限：用户分 TL0 新人、TL1 成员、TL2 老用户、TL3 守护者。每个帖子有 `min_read_level`（0 所有人含游客，1 成员以上，2 老用户以上，3 守护者），发帖人只能设到不超过自己的等级，编辑时可以保持或调低，不能调高到超过自己；不传按公开。读者按登录态确定（游客是 0 级），作者总能读自己的帖子。受限帖子在列表、搜索、标签页、用户主页（帖子、回复摘要、统计）、收藏夹和搜索引擎页面里对没权限的人完全不可见，直接打开返回 403 并说明需要什么等级；回复列表、回复、点赞、收藏、举报同样检查。分类、产品、标签上的帖子数只统计公开帖。`sitemap.xml` 不收录受限帖，受限帖的搜索引擎页面按不存在处理（404 + noindex）。后台能看到所有帖子及其阅读权限。读者身份在 `models::Reader`，`ListFilter.reader` 用 `serde(skip)` 防止被查询参数伪造，默认值是游客，所以忘记设置的查询只会看到公开内容。
- 帖子附件：发帖和编辑页的「附件」一栏，先逐个上传（`POST /attachments`），再用 `attachment_ids` 随发帖或编辑挂到帖子上，一个帖子最多 5 个。单文件上限看信任等级：成员及以上（TL1-TL2）5 MB，守护者（TL3，也就是站务人员，后台账号发不了帖）30 MB，新人不能上传。文件落在 `FORUM_UPLOAD_DIR/{id}`，库表 `attachments` 只记文件名和大小；上传后一天内没挂到帖子上的由定时任务清理，编辑时移除的附件立即删文件。下载一律按 `application/octet-stream` + `attachment` 输出，下载权限和读帖子一致（阅读等级不够返回 403，帖子被隐藏或删除返回 404，没挂上的只有上传者能取），所以站内不会直接打开用户传的 HTML。阅读等级受限的帖子前端用带令牌的 fetch 下载，公开帖子是普通链接。只有主楼支持附件，回复不支持；搜索引擎页面不列附件。部署时 Nginx 的 `client_max_body_size` 要调到 31m 以上，否则上传会被代理直接 413。
- 置顶：守护者（TL3）在帖子页的操作栏能看到「置顶 / 取消置顶」，只有正常状态的帖子可以置顶；管理后台 `/admin` 的帖子管理里原来就有同样的开关，两边改的是同一个 `topics.pinned`。置顶的帖子在所属板块和首页的列表里排在最前面（不管按哪种方式排序），列表里带「置顶」标记；搜索、标签页和用户主页不受影响。前台守护者的操作同样记进审计日志，操作人显示为「守护者:昵称」。
- 正文插图：编辑器工具栏的「插图」按钮，也可以直接粘贴截图或把图片拖进输入框（发帖、编辑、回复都一样）。上传后得到站内地址 `/api/images/{id}`，自动插进 Markdown 的 `![](…)`。只收 PNG、JPEG、GIF、WebP，服务端按文件头判断真实类型（不信文件名和客户端类型），不收 SVG；每张不超过 `FORUM_UPLOAD_MAX_MB_IMAGE`，成员及以上才能传（新人的图片本来就会被渲染成纯文字），每人每天最多 50 张。文件放在 `FORUM_UPLOAD_DIR/img-{id}`，库表 `images`。图片地址靠不可猜的 id 公开访问，不看帖子的阅读等级，响应带 `nosniff` 和沙箱 CSP，长期缓存。上传一天后帖子和回复的 Markdown 里都搜不到它的 id 的，定时任务每小时清理一次。Nginx 要给 `/api/images` 单独放宽 `client_max_body_size`（6m 以上）。后续接腾讯云 COS 时，只需要换这两个接口的存储。
- 活动链接不允许带 `ref`、`aff`、`invite` 等个人推广参数。
- 只做站内功能：没有系统推送、邮件提醒，也没有匿名发帖（按产品决定）。

## 已知限制与后续

- 数据库兼容：阅读权限是追加的列（`ALTER TABLE topics ADD COLUMN IF NOT EXISTS min_read_level ... DEFAULT 0`），已部署的库启动时自动补上，旧帖子都按公开处理。
- 二期（PRD 里的 M2 起）未做：站内通知和关键词订阅、TL2 自动升级、屏蔽用户、敏感词、图片上传到腾讯云 COS、赞助位。积分规则见上面的运行与验证；正文插图已经走本机磁盘（见业务规则），COS 是二期的存储替换。
- 账号中心还没有用户自助注销，设置页提示发邮件处理。
- SEO 页面只渲染第一页（30-50 条），sitemap 每次请求实时查询，没有进程内缓存。
- 部署：香港机 `/home/ubuntu/xiangban-forum`，systemd 服务 `server-forum`，监听 127.0.0.1:55010，Nginx 反代 chat.xiangban.xyz（Let's Encrypt），库 `xiaban_forum` 在 `wallpaper_postgres` 容器里，独立角色 `forum`。账号中心已登记 `forum-web` 并在 `ACCOUNT_CORS_ORIGINS` 加入 `https://chat.xiangban.xyz`。初始管理员密码在服务器 `.env` 的 `FORUM_ADMIN_PASSWORD`，登录 /admin 改密码后删掉它。更新时本地交叉编译后替换二进制和 `web/dist`，再重启服务（index.html 启动时读一次）。

## 站方图文创作与批量发帖

后台新增 `/admin/editor-accounts`（编辑账号与本地令牌）和 `/admin/editor`（批量发帖）。统一账号中心后台的用户页也支持“创建用户”。站方账号用用户名和密码正常登录，不会被自动标为拥有已验证邮箱/手机号，后续可自行绑定。

- **创建新编辑账号**：账号中心设置 `ACCOUNT_FORUM_PROVISION_KEY`、论坛设置相同的 `FORUM_ACCOUNT_PROVISION_KEY`（至少 32 字节随机值，例如本机 `openssl rand -hex 32` 生成）。两项都是服务端环境变量，不写进前端或本地 Skill。账号中心与论坛均需更新并重启。论坛通过受限的 `/api/internal/forum/users` 建号；该凭据不能管理已有用户或获取用户会话。
- **绑定已有账号**：用户先正常登录过论坛，再在编辑账号页填 UID 绑定；须为正常状态且至少 TL1。绑定后论坛展示名作为编辑资料保留，不被登录同步覆盖。官方板块仍要求 TL3，批量发布重新核对作者状态和板块权限。
- **导入**：每篇 `.md` 带 YAML 元信息 `id/title/category/tags`，图片放在该文件旁的 `images/` 并用相对路径引用。`category` 使用真实 slug。一个 ZIP 可含 1–50 篇；单个 `.md` 用于纯文字稿。ZIP 不超过 100 MB，解压后不超过 80 MB、最多 500 个内容文件。每篇最多 20 张图片、共 20 MB；后台上传适配默认每张 5 MB 的限制。浏览器解包，逐篇调用同一导入接口；可选择作者、覆盖板块、更新已有待发布稿。部分失败不回滚成功项。
- **待发布稿**：导入不产生前台帖子。稿件与图片随数据库/上传目录一起持久化；图片清理任务会保留稿件引用的图片，待发布图片公开接口返回 404。编辑预览通过鉴权接口返回内嵌配图，使用论坛同一 Markdown 渲染器。保存后才更新预览。
- **发布与去重**：同作者下稿件 `id` 唯一；默认拒绝重复导入，显式覆盖只允许更新待发布稿。逐篇发布在事务中写正文、标签、分类扩展、图片公开状态和记录，再返回正式链接；重试相同稿件只返回已有帖子。已发布稿不能通过再次导入覆盖，走论坛原有编辑入口。发布的是公开帖，暂不提供付费附件、阅读等级、定时发布及羊毛扩展字段；羊毛稿可正常发文，额外信息走现有编辑入口。
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
