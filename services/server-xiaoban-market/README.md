# server-xiaoban-market

小伴市场：专家、技能、连接器（MCP）的**自建目录服务**。对外提供公开网页和 API（分类、关键词模糊搜索、下载安装、更新检测），对内提供**采集工作台**，把 SkillHub、魔搭社区的内容采集到暂存区，审核后入库。

技术栈：Rust（Axum 0.8 + SQLx）+ PostgreSQL + Vue 3 + Vite。

## 实现状态

| 部分 | 状态 |
| --- | --- |
| 数据库、公开目录 API、下载与更新检测 | 已实现，有端到端测试 |
| 采集：SkillHub（技能 / 专家 / 连接器） | 已实现；接口形状取自客户端与壁纸站已验证的用法，**未对真实线上接口做过联调** |
| 采集：魔搭社区（MCP / 技能 / 智能体） | **通用 JSON 适配器已实现，但三个来源的接口地址与字段映射尚未配置**。开发环境访问不到 modelscope.cn，真实接口只能由能访问的人在后台「试采」后填写 |
| 采集工作台后端 API（来源、试采、任务、暂存区、入库、内容库、分类、版本上传） | 已实现 |
| 前端：公开网页（专家 / 技能 / 连接器、搜索、详情弹窗）与后台工作台（总览、采集来源与试采、采集任务、暂存区、内容库、分类、设置） | 已实现（Vue 3 + Vite + TypeScript），在 `web/` |
| 部署栈（`infra/`）、定时备份 | 未实现 |

## 目录

```text
migrations/     PostgreSQL 迁移（启动时自动执行）
web/            Vue 3 + Vite 前端：公开网页在 `/`，后台工作台在 `/admin`
src/
  upstream/     上游适配器：skillhub.rs、generic.rs（配置驱动的 JSON 适配器）
  crawler.rs    采集任务：每来源同时只跑一个，支持取消、重试退避、定时、自动入库
  importer.rs   暂存项入库：二次抓取详情 / 安装包、校验、分类解析、事务写入
  package.rs    技能安装包校验与清单（限制与桌面客户端一致）
  handlers/     public.rs（公开 API）、admin.rs（后台 API）
tests/api.rs    真实 PostgreSQL + 本机模拟上游的端到端测试
```

## 运行与验证

```sh
cp .env.example .env     # 填 DATABASE_URL 等
cargo run                # 默认 http://0.0.0.0:55010，首次启动自动迁移并建种子数据

npm --prefix web ci && npm --prefix web run build   # 前端构建到 web/dist，由后端静态托管
npm --prefix web test    # 前端单元测试
# 前端开发：先 cargo run，再 npm --prefix web run dev（http://localhost:5175，/api 代理到 55010）

cargo test --lib         # 单元测试，不需要数据库
# 端到端测试需要一个有 CREATEDB 权限的账号；每个用例会建一个临时库并在结束时删除
TEST_DATABASE_URL=postgres://market:market@127.0.0.1:5432/postgres cargo test
```

首次启动会创建管理员；没设 `MARKET_ADMIN_PASS` 时，随机密码**只在日志里出现一次**。

按仓库规范，服务器上不要现场编译：在本地构建好二进制和前端静态资源再上传。

## 前端

公开网页和后台是同一个单页应用（`web/`），按已确认的设计稿实现：

- **公开网页** `/experts` `/skills` `/connectors`：分类、排序、关键词搜索（命中高亮；精确结果少时补「相近匹配」）、分页；点击条目以弹窗展示详情（`/skills/{namespace}/{slug}`，可直接分享链接）。弹窗标题区固定，内容区滚动。
- **后台** `/admin`：登录后可用。采集来源页有结构化配置表单（SkillHub / 通用 JSON 各一套）、字段映射编辑和「试采预览」；暂存区支持批量入库 / 忽略、与已入库内容对比；采集任务页每 2 秒刷新进度和日志。
- 网页版**不显示**「已安装 / 召唤 / 我的专家」这类依赖桌面端本地状态的按钮；详情弹窗的主按钮通过深链 `xiaoban://market/{open|install|connect}/{skills|experts|connectors}/{namespace}/{slug}?version=&sha256=` 唤起小伴桌面。**桌面客户端目前没有注册这个协议**，接入时需要对齐。
- 管理员令牌保存在浏览器 `localStorage`；任何接口返回 401 都会回到登录页。
- 暂未做：专家团、套件、收藏 / 评分、深色模式。

## 数据流

```text
上游目录 ──采集任务──▶ 暂存区(staged_items) ──人工审核/自动入库──▶ 内容库(items + versions) ──▶ 公开 API / 网页
            │ 抓取列表页，规范化，按内容哈希判断 新增 / 有更新 / 无变化
            └ 入库时才二次抓取详情、下载并校验安装包
```

- 暂存区保留上游原文和规范化结果。上游内容变化时，已入库的暂存项回到「待处理」，在工作台里显示为「有更新」；已忽略的保持忽略。
- 重复入库只更新内容字段，**不会改动管理员已设置的状态、精选和分类**。来源设置了「自动入库」时，也只对新条目套用该状态。
- 入库时管理员选的分类会被记住（`category_aliases`），同名上游分类下次自动归入。

## 采集来源

`sources` 表一行一个上游板块，种子数据已建好六个，定时间隔默认 0（仅手动），**不配置不会访问外网**：

| key | 类型 | 适配器 |
| --- | --- | --- |
| `skillhub-skills` / `skillhub-experts` / `skillhub-connectors` | 技能 / 专家 / 连接器 | `skillhub` |
| `modelscope-mcp` / `modelscope-skills` / `modelscope-agents` | 连接器 / 技能 / 专家（魔搭的智能体归入专家） | `generic_json` |

SkillHub 可在 `config.api_key` 填团队密钥；密钥只发给 SkillHub 的 API 域名，下载被重定向到腾讯 COS 时不会带上。

### 配置魔搭（generic_json）

在后台「采集来源」里打开浏览器开发者工具的网络面板，找到魔搭页面加载列表时的请求，把地址、请求体和响应结构填进配置，再点「试采一页」，根据每个字段是否命中调整映射：

```json
{
  "origin": "modelscope",
  "url": "https://<列表接口地址>",
  "method": "POST",
  "body": { "page": "{page}", "page_size": "{page_size}", "keyword": "{keyword}" },
  "items_path": "/data/items",
  "total_path": "/data/total",
  "page_size": 20,
  "page_start": 1,
  "mapping": { "upstream_id": "/id", "name": "/name", "summary": "/desc", "tags": "/tags", "category": "/cat" },
  "config_path": "/config",
  "origin_url_template": "https://www.modelscope.cn/mcp/servers/{upstream_id}"
}
```

- `body` / `query` 里可用 `{page}` `{page_size}` `{keyword}` `{category}`；整串恰好是 `{page}` 或 `{page_size}` 时输出数字。
- `mapping` 的值是相对每个条目的 JSON 指针；数组表示取第一个非空值；以 `=` 开头是字面量（如 `"=official"`）。必填 `name`，缺 `upstream_id` 时用 slug 或名称。
- `config_path` 只用于连接器：指向条目里的 MCP 配置（对象或 JSON 字符串），解析后只展示，不执行。
- 地址必须是 HTTPS，且不能指向回环 / 内网。配置里的密钥（`api_key`、`headers` 中含 token / key / authorization / cookie 的值）读取时显示为 `***`，保存时回传 `***` 表示不修改。

## 安装包规则

与桌面客户端 `libs/agent-tools/skill-discovery` 对齐，上架时就拒绝客户端会拒绝的包：最多 512 个文件，解压后 12 MiB，单文件 4 MiB，`SKILL.md` 100 KiB；拒绝路径穿越、绝对路径、符号链接、Windows 保留名、大小写重复路径；允许一个顶层包装目录；根目录必须有 `SKILL.md`。

清单摘要 `manifestSha256` 是**本服务自有算法**：按路径排序后，对每行 `"<文件 sha256>  <路径>\n"` 求 SHA-256。它和 SkillHub 的 `manifest.sha256` 算法不同，桌面客户端接入时需要对齐。包文件按内容哈希存放在 `MARKET_DATA_DIR/packages/`，删除条目不会删除包文件（可能被其它版本共用）。

SHA-256 只保证内容与清单一致，**不代表安全**；脚本不在沙箱中运行。

## API

公开接口（不需要登录）在 `/api/v1/market`，`{kind}` 取 `skills` / `experts` / `connectors`：

| 接口 | 说明 |
| --- | --- |
| `GET /{kind}?keyword&category&source&sortBy&page&pageSize` | 列表 + 搜索。`sortBy`: `relevance`（默认）/ `hot` / `new`；每页默认 24，最多 50。精确结果少于 5 条时，第一页额外返回 `near`（相近匹配） |
| `GET /categories?kind=` | 分类及已上架数量 |
| `GET /suggest?kind=&q=` | 搜索联想 |
| `GET /{kind}/{namespace}/{slug}` | 详情：描述、示例提问、元数据、版本、最新版文件清单 |
| `GET /{kind}/{namespace}/{slug}/versions/{version}/download` | 下载 ZIP；支持 Range；响应头 `x-content-sha256`；同一来源同一天同一版本只计一次下载 |
| `POST /updates` | 客户端上报已安装的 `[{kind,namespace,slug,version}]`（≤200），返回哪些有更新 |

关键词按空格拆成最多 5 个词，每个词做不区分大小写的子串匹配（中文可用），`%` `_` `\` 会被转义。热度 `heat = 下载量 + 收藏量 × 5`。

后台接口在 `/api/v1/admin`，除 `POST /login` 外都需要 `Authorization: Bearer <token>`：

| 分组 | 接口 |
| --- | --- |
| 账号 | `POST /login`，`GET /me`，`PUT /account`，`GET /stats` |
| 来源 | `GET /sources`，`PUT /sources/{id}`，`POST /sources/{id}/probe`（试采，不写库），`POST /sources/{id}/jobs` |
| 任务 | `GET /jobs`，`GET /jobs/{id}?after=<日志ID>`，`POST /jobs/{id}/cancel` |
| 暂存区 | `GET /staged`，`GET /staged/{id}`，`POST /staged/import`，`POST /staged/ignore`，`POST /staged/restore` |
| 内容库 | `GET/POST /items`，`GET/PUT/DELETE /items/{id}`，`POST /items/{id}/status`，`POST /items/{id}/versions`（multipart：`version`、`changelog`、`file`），`PUT /versions/{id}`（撤回 / 恢复） |
| 分类 | `GET/POST /categories`，`PUT/DELETE /categories/{id}` |

采集任务参数（`POST /sources/{id}/jobs` 的请求体，下划线命名）：`keyword`、`category`、`source`、`sort_by`、`max_pages`（缺省取来源配置，默认 5，最多 200）。

## 相伴账号登录

后台默认只有本地管理员（`MARKET_ADMIN_USER` / `MARKET_ADMIN_PASS`）。设置 `MARKET_ACCOUNT_ISSUER` 后，登录页多出「相伴账号」登录，复用统一账号中心（`services/server-account`）的账号：

1. 在账号中心登记应用：给账号中心配置 `ACCOUNT_CLIENTS=…,xiaoban-market:小伴市场`（或在它的后台「应用」里新增），`client_id` 要和 `MARKET_ACCOUNT_CLIENT_ID` 一致。
2. 给小伴市场配置 `MARKET_ACCOUNT_ISSUER=https://account.xiangban.xyz`（必须等于账号中心的 `ACCOUNT_ISSUER`）。可选：`MARKET_ACCOUNT_JWKS_URL`（默认 `<issuer>/.well-known/jwks.json`）、`MARKET_ACCOUNT_URL`（浏览器访问账号中心 API 的地址，默认同 issuer）、`MARKET_ACCOUNT_CLIENT_ID`（默认 `xiaoban-market`）。账号中心需允许小伴市场的来源跨域（`ACCOUNT_CORS_ORIGINS`，留空即不限制）。
3. 用本地管理员登录，在「设置 → 相伴账号管理员」里填入要授权的账号和它的密码，点「绑定为管理员」。之后这个账号就可以用相伴账号登录。

工作方式：浏览器用账号 + 密码直接向账号中心换访问令牌（密码不经过小伴市场）；小伴市场用 JWKS 公钥离线验签，要求 `iss` 匹配、`aud` 等于本应用的 `client_id`，并且账号 ID 已被绑定才签发后台令牌（8 小时）。**解绑后，该账号已签发的后台令牌在下一次请求就会失效**；本地管理员始终可用，不依赖账号中心。账号中心不可达时，相伴账号登录会提示稍后重试，本地登录不受影响。

## 已知限制

- 后台登录没有限流，部署时请放在反向代理后并限制来源。
- 采集任务在服务进程内运行；重启时未完成的任务会被标记为失败，不会续跑。多实例部署会让定时任务重复触发（各实例都会启动调度器）。
- 上游地址只做静态校验（HTTPS、非回环 / 内网字面量），不防 DNS 指向内网的情况；来源配置只有管理员能改。
- 未实现收藏、评分、评论和投稿审核。「专家团」「套件」等设计稿里提到的类型暂未建模。
