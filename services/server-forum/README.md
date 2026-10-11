# server-forum：超级小伴论坛

状态：一期（网页版）已实现，2026-10-11 部署在香港机 chat.xiangban.xyz。面向开发者和 AI 用户的社区：技术交流、资源分享、福利羊毛（羊毛帖的产品、能领到什么、领取条件、有效期、活动页都是选填，到期自动下架）、随便唠唠和站务板块。福利羊毛不作为主推，排在技术交流和资源分享之后。站点名「超级小伴」，计划域名 `chat.xiangban.xyz`。产品设计见 [docs/product/PRD_论坛.md](../../docs/product/PRD_论坛.md)。

技术栈：Rust Axum 0.8 + PostgreSQL（SQLx），前端 Vue 3 + Vite + TypeScript（`web/`），由本服务直接托管。登录走[统一账号中心](../server-account/README.md)，本站不存密码，用 JWKS 公钥离线验签。默认端口 55010。

## 运行与验证

```sh
cp services/server-forum/.env.example services/server-forum/.env   # 按需修改
npm --prefix services/server-forum/web ci
npm --prefix services/server-forum/web run build                  # 生成 web/dist；不构建也能跑，只是没有前端
cargo run --manifest-path services/server-forum/Cargo.toml
cargo test --manifest-path services/server-forum/Cargo.toml
```

数据库表、分类和产品种子在启动时自动创建（种子只在表为空时写入，后台改过的不会被覆盖）。数据库有 `pg_trgm` 扩展时会给标题建三元组索引，没有就退化为普通 ILIKE。

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
| `FORUM_DAILY_TOPICS` / `FORUM_DAILY_POSTS` | `0,5,20,50` / `10,60,200,500` | 按信任等级 0-3 的每日发帖、回复上限 |

## 页面与接口

前台路由：`/` 首页、`/deals` 福利羊毛（顶部一行分类标签，可按最新、热门、快过期排序；已到期的默认不显示）、`/c/{slug}` 分类、`/t/{id}` 帖子、`/new` 发帖、`/p/{slug}` 产品、`/products`、`/tag/{name}`、`/search`、`/u/{uid}`、`/settings`（收藏、昵称、等级说明）、`/rules` `/terms` `/privacy`，后台在 `/admin`。

服务端对 `/`、`/deals`、`/t/{id}`、`/c/{slug}`、`/p/{slug}`、`/products`、`/tag/{name}` 直接输出带标题、摘要、正文和 JSON-LD 的 HTML（填进 `#app`，Vue 挂载时替换），另有 `/sitemap.xml`、`/robots.txt`。实现在 `src/seo.rs`，依赖 `web/index.html` 保留 `<title>`、`<meta name="description">` 和空的 `<div id="app"></div>`。

前台接口（`/api`，写操作要带账号中心访问令牌）：

| 接口 | 说明 |
| --- | --- |
| `GET /config` | 站点名、账号中心地址、client_id |
| `GET /categories` `/products` `/products/{slug}` `/tags` | 分类树、产品库、产品页、热门标签 |
| `POST /session` | 登录后调用：同步昵称头像、记访问、检查自动升级，返回本站资料和每日上限 |
| `GET /topics` `/deals` `/search` | 列表；参数见 `models::ListFilter` |
| `GET/POST /topics`、`GET/PATCH/DELETE /topics/{id}` | 帖子；羊毛帖带 `deal` 字段 |
| `GET/POST /topics/{id}/posts`、`PATCH/DELETE /posts/{id}` | 回复 |
| | `PUT/DELETE /topics/{id}/like`、`/posts/{id}/like`、`/topics/{id}/bookmark` | 点赞、收藏 |
| `POST /reports` | 举报帖子或回复 |
| `POST /markdown/preview` | 发帖预览 |
| `GET /users/{uid}`、`/users/{uid}/topics`、`/users/{uid}/posts`、`/me/bookmarks` | 用户主页、我的收藏 |

管理接口在 `/api/admin/*`（`src/handlers/admin.rs`）：管理员独立建表，HS256 令牌带 `token_version`，改密码后旧令牌失效，连续输错 5 次锁 15 分钟；包括概览、举报处理、帖子与回复管理、用户等级和禁言封禁、分类与产品库维护、操作日志，所有写操作都记审计。

## 业务规则

- 正文是 Markdown，服务端用 pulldown-cmark 渲染、ammonia 清洗后存 `body_html`；外链加 `nofollow ugc`。新人（TL0）的链接和图片会被去掉，只留文字。
- 羊毛帖：产品、能领到什么、领取条件、有效期、活动页全部选填，什么都不填也能发；没有投票和状态标记，写了有效期的到期后自动不再出现在福利羊毛、首页和分类列表（用户主页、搜索、标签页、产品页的历史仍保留）。每 10 分钟重算最近 7 天的热度。
- 活动链接不允许带 `ref`、`aff`、`invite` 等个人推广参数。
- 只做站内功能：没有系统推送、邮件提醒，也没有匿名发帖（按产品决定）。

## 已知限制与后续

- 二期（PRD 里的 M2 起）未做：站内通知和关键词订阅、积分规则、TL2 自动升级、屏蔽用户、敏感词、图片上传到腾讯云 COS、赞助位。现在帖子里的图片只能用外链。
- 账号中心还没有用户自助注销，设置页提示发邮件处理。
- SEO 页面只渲染第一页（30-50 条），sitemap 每次请求实时查询，没有进程内缓存。
- 部署：香港机 `/home/ubuntu/xiangban-forum`，systemd 服务 `server-forum`，监听 127.0.0.1:55010，Nginx 反代 chat.xiangban.xyz（Let's Encrypt），库 `xiaban_forum` 在 `wallpaper_postgres` 容器里，独立角色 `forum`。账号中心已登记 `forum-web` 并在 `ACCOUNT_CORS_ORIGINS` 加入 `https://chat.xiangban.xyz`。初始管理员密码在服务器 `.env` 的 `FORUM_ADMIN_PASSWORD`，登录 /admin 改密码后删掉它。更新时本地交叉编译后替换二进制和 `web/dist`，再重启服务（index.html 启动时读一次）。
