# 小伴桌面网站与管理服务

独立 Rust（Axum）+ PostgreSQL 元数据服务，配套 Vue 3 + Vite 产品介绍网站和管理后台。默认端口 55006，不接入主 API、账号体系、根 Gradle 或网盘服务，不存储、代理或上传壁纸媒体文件。

## 职责与入口

- `backend`：管理员登录、账号修改、壁纸 JSON 元数据、上下架、流式校验、公开桌面清单与应用发布信息。
- `frontend`：奶白/暖粉色产品介绍页、壁纸展示、下载入口和 `/admin` 管理后台。
- `docs`：JSON 输入格式与接口说明。
- `compose.yaml`、`Dockerfile`：独立部署，PostgreSQL 数据卷只保存元数据。

首次管理员为 `admin / 123456`，登录后可修改用户名和密码；修改会让旧会话失效。初始化只发生一次，重启不重置账号。

公开网站为 `/`，管理入口为 `/admin`。壁纸可新建、编辑、排序、上下架、删除元数据和批量 JSON 导入/导出；流式探测可以补齐大小与 SHA-256。产品发布页管理外部安装包与版本，公开订阅接口 `/api/catalog`、更新接口 `/api/updates` 与桌面协议兼容。输入样例与完整接口见 [docs/api.md](docs/api.md)。

## 开发与验证

```sh
npm --prefix services/server-xiaoban-wallpaper/frontend ci
cargo test --manifest-path services/server-xiaoban-wallpaper/backend/Cargo.toml
npm --prefix services/server-xiaoban-wallpaper/frontend run build
cargo run --manifest-path services/server-xiaoban-wallpaper/backend/Cargo.toml
```

需要 Rust、Node.js 24 与独立 PostgreSQL 数据库，启动前设置 `DATABASE_URL`。后端会托管前端 `dist`，默认本地地址 `http://127.0.0.1:55006`。前端开发服务器端口 5176，代理 `/api` 到 55006。

Rust 需 1.88+，PostgreSQL 17+。Windows PowerShell 用 `$env:DATABASE_URL='postgresql://用户:密码@127.0.0.1:5432/xiaoban_wallpaper'`；数据库必须提前创建，表结构会自动迁移。前端使用 `npm run dev` 时将后端 `PUBLIC_ORIGIN` 设为 `http://127.0.0.1:5176`。不设置 `.env` 自动读取，环境变量由 Shell 或部署环境注入。

集成测试必须显式配置 `XIAOBAN_TEST_DATABASE_URL`，数据库名以 `xiaoban_wallpaper_test` 开头。每个测试使用唯一 schema 并清理自己的 schema；缺少测试库时直接失败，不把跳过算作通过。测试覆盖登录与 CSRF、账号改密及重启、批量导入回滚、并发发布上限、元数据乐观锁、流式探测与网络边界、更新版本语义。

## 独立部署

在本目录复制 `.env.example` 到 `.env`，填写数据库密码与网站源地址，执行：

```sh
docker compose --env-file .env up -d --build
```

网站与 API 共用 55006 端口。容器只绑定宿主 `127.0.0.1`；正式域名通过已有 HTTPS 反向代理接入，设置准确的 `PUBLIC_ORIGIN=https://你的域名` 与 `SECURE_COOKIE=true`，不带末尾斜杠。Compose 的数据库密码需使用 URL 安全字符；托管数据库可直接给后端配置完整 `DATABASE_URL`（自行 URL 编码密码）。数据库不暴露宿主端口，不与主 API 共用库或迁移。

持久化只发生在 PostgreSQL：管理员 Argon2id 哈希、已哈希会话标识、壁纸 JSON 和应用发布 JSON。备份使用本服务数据库的 `pg_dump`，无需备份媒体目录。网页内置角色/品牌 SVG 是静态产品界面资产，壁纸资源与安装包均是外部 URL。本机开发数据库与运行日志仅位于忽略的测试目录，不纳入仓库。

独立工作流 `.github/workflows/xiaoban-wallpaper-server.yml` 在 PostgreSQL 环境运行真实集成测试、构建 Rust/Vue、验证 Compose 并构建独立镜像。现有腾讯云主 API 部署工作流不增加本服务。

## AI 开发边界

服务只持久化管理员哈希、会话、壁纸和发布 JSON 元数据。禁止增加媒体上传、媒体落盘缓存或媒体下载代理。资源校验按有限大小、有限时间流式读取；限制外部目标，不能访问内网、环回或云元数据地址（测试环境明确允许环回）。发布的清单须与桌面 v1 协议兼容，最多 100 张、1 MiB；草稿允许缺少 SHA-256 和大小，发布前必须补齐。产品介绍页面只展示真实已有能力，未配置安装包时不伪造下载。

密码采用 Argon2id 随机盐哈希；会话使用 HttpOnly Cookie，修改请求验证 CSRF。正式部署设置 HTTPS 的 `PUBLIC_ORIGIN` 和 `SECURE_COOKIE=true`。独立 CI 与构建只覆盖本服务，不增加腾讯云现有主 API 部署的构建项目。
