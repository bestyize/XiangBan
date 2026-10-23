# server-ai：相伴 AI 网关与额度系统

状态：一期、二期（兑换码）已实现，未部署。为产品矩阵提供统一的 AI 调用入口：身份只认[账号中心](../server-account/README.md)，上游模型经 New API 网关调用，额度按「账号 × 产品」隔离。设计取舍见 [docs/architecture/ai-gateway.md](../../docs/architecture/ai-gateway.md)。

技术栈：Rust Axum 0.8 + PostgreSQL（SQLx），管理后台为 Vue 3 + Vite + TypeScript（`web/`），由本服务直接托管。默认端口 55010。

## 它做什么

- 用账号中心的 JWKS 离线验签，令牌 `aud` 决定用户属于哪个产品（`products.client_ids`）。
- 首次使用某产品的 AI 时，在 New API 里为该 `(账号, 产品)` 创建一个独立用户和一个无限额度令牌，并发放该产品的试用额度。令牌只存在本服务（AES-256-GCM 加密），客户端永远拿不到。
- 额度以「桶」记账（试用、月卡、加油包），每个桶有总额、剩余和到期时间，先扣最早过期的。每次变化都写只追加的流水。
- 每分钟与 New API 对账：钱包比预期少 = 消耗，按桶结算；比预期多 = 有人手工改过，记为异常并按账本对齐。推送一律用增减，不用覆盖。
- 客户端只认**档位**（如 `standard`、`premium`），真实模型只在管理后台配置，随时可换。响应、流式事件和错误信息里都不出现真实模型名。小伴桌面向用户显示“日常／深入”两种使用模式，日常为默认；模式名称、说明、排序与模型通过后台维护，不增加用户侧思考强度控件。线上沿用 `standrad`（历史拼写）与 `premium` 标识，不能仅为改展示名称而重建档位。

## 语音合成

`POST /v1/audio/speech`：小伴桌面按句朗读时调用。请求体 `{ "input": "一句话", "voice"?: "音色", "speed"?: 1.0 }`，返回 `audio/mpeg`。

- 真实模型由 `AI_SPEECH_MODEL` 指定（New API 里 OpenAI 兼容的语音渠道），客户端不传也看不到模型名；可选音色由 `AI_SPEECH_VOICES` 列出，第一个为默认，请求其他音色会被拒绝。
- 鉴权、额度检查与对话接口相同，用量随 New API 钱包对账结算。单次最多 1000 字，返回音频不超过 8 MiB。
- 未配置模型时返回 503 `speech_unavailable`，客户端会自动改用系统朗读。

## 联网搜索与网页抓取

`POST /v1/search` 和 `POST /v1/fetch`：给小伴的联网搜索用。**对外格式以 DeepSeek 的 Search / Fetch 为准**，客户端只认这一种；后面接格式不同的服务商时，在 `src/search.rs` 的 `Provider` 里加一个分支，把它的请求和结果改写成这个格式。响应里不出现服务商名字，密钥只在服务端。

- 搜索请求：`{ "query": "1–128 字，可带 site: 等语法", "count"?: 1–50（默认 10）, "need_content"?: false, "ori_question"?: "用户原本的问题" }`。
- 搜索返回：`{ "results": [{ "title", "url", "snippet", "summary"?, "date"（可为 null）, "content"?, "rerank_score"?（0–3，≥1.5 为高相关）, "authority"?（0–4，≥3 为权威） }] }`。没找到时 `results` 为空数组，不算错误。没有 http(s) 网址的条目会被丢掉，单条正文最多 8000 字。
- 抓取请求：`{ "url": "http(s) 网址，最长 8192 字符，不能带账号密码" }`；返回 `{ "url", "title", "content"（Markdown，最多 60000 字）, "truncated" }`。
- 服务商按 `AI_SEARCH_PROVIDERS` 的顺序尝试：前一家超时、5xx、限流、密钥失效或返回格式不对就换下一家；400/422（查询本身不被接受）不换，直接返回 400。都没配时返回 503 `search_unavailable`，都失败时返回 502。
- 鉴权、额度检查与对话接口相同。**只有搜到结果的搜索和成功的抓取才计费**：直接从该用户的 New API 钱包扣 `AI_SEARCH_COST` / `AI_FETCH_COST` 点，下一次对账照常按消耗结算进额度桶和流水。扣费失败只记日志，不影响已经返回的结果。

## 运行与验证

```sh
cp services/server-ai/.env.example services/server-ai/.env   # 按需修改
cargo run --manifest-path services/server-ai/Cargo.toml -- gen-key   # 生成主密钥，写入 AI_MASTER_KEY_BASE64
npm --prefix services/server-ai/web ci
npm --prefix services/server-ai/web run build                 # 生成 web/dist，不构建则只提供 API
cargo run --manifest-path services/server-ai/Cargo.toml
cargo test --locked --manifest-path services/server-ai/Cargo.toml
```

管理后台在 `http://localhost:55010/admin/`。前端开发可用 `npm --prefix services/server-ai/web run dev`（端口 5176，`/api` 代理到 55010）。

数据库表由服务启动时自动创建。集成测试需要 `TEST_DATABASE_URL`（每个测试建独立 schema，结束删除；New API 用进程内的假网关代替，它会校验请求格式）；未设置时集成测试跳过，只跑单元测试。

## 配置

| 变量 | 默认 | 说明 |
| --- | --- | --- |
| `AI_DATABASE_URL` | `postgres://xiaban@localhost:5432/xiaban_ai` | PostgreSQL 连接串，独立库 |
| `AI_HOST` / `AI_PORT` | `127.0.0.1` / `55010` | 监听地址，生产经 Nginx 反代，不要直接对公网开放 |
| `AI_ACCOUNT_ISSUER` | `https://account.xiangban.xyz` | 账号中心令牌的 `iss`，必须一致 |
| `AI_ACCOUNT_JWKS_URL` | `{issuer}/.well-known/jwks.json` | 公钥地址 |
| `AI_NEWAPI_BASE_URL` | `http://127.0.0.1:3000` | New API 地址（回环）。本服务访问它时忽略系统代理环境变量 |
| `AI_NEWAPI_ADMIN_TOKEN` | 空 | **可选，一般不用设**。New API 管理令牌推荐直接在后台「网关连接」页填写（加密存数据库，立即生效，无需重启）；后台保存过的令牌优先于这个变量。令牌是 `nap_` 前缀的访问令牌，只授予「用户」的查看和编辑（`user:read`、`user:write`），不要用 root |
| `AI_NEWAPI_DEFAULT_RATIO` | `37.5` | 未配置倍率的模型在 New API 里的默认倍率，保存档位时用它判断是否漏配 |
| `AI_QUOTA_PER_UNIT` | `500000` | New API 里 1 美元对应的额度点数，仅用于后台展示 |
| `AI_MASTER_KEY_BASE64` / `AI_DATA_DIR` | 空 / `ai_data` | 主密钥（至少 32 字节）；未设置时首次启动在数据目录生成 `master.key`。**必须备份**，丢失后已保存的 New API Key 无法解密 |
| `AI_ADMIN_USER` / `AI_ADMIN_PASSWORD` | 空 | 仅在数据库里还没有管理员时用来创建初始管理员；首次登录后请改密码并移除 `AI_ADMIN_PASSWORD` |
| `AI_SYNC_INTERVAL_SECS` | `60` | 对账间隔，最小 5 |
| `AI_MAX_BODY_BYTES` | `8388608` | 转发请求体上限 |
| `AI_TRUST_PROXY` / `AI_CORS_ORIGINS` / `AI_WEB_DIR` | 见 `.env.example` | 同账号中心 |
| `AI_SEARCH_PROVIDERS` | `deepseek` | 联网搜索的服务商，逗号分隔，按顺序尝试；没配密钥的跳过 |
| `AI_DEEPSEEK_API_KEY` / `AI_DEEPSEEK_BASE_URL` | 空 / `https://api.deepseek.com` | DeepSeek Search / Fetch 的密钥与地址；不填密钥则不启用。密钥可以填多个（逗号分隔）：请求轮流使用以分摊限流，某个密钥失效、限流或超时就换下一个，全部不行再换下一家服务商 |
| `AI_SEARCH_COST` / `AI_FETCH_COST` | `500` / `500` | 一次有结果的搜索、一次成功的抓取扣多少 New API 额度点数 |

## 接口

用户接口用账号中心的 Bearer 令牌；`aud` 不属于任何已启用产品时返回 403。

| 方法 路径 | 说明 |
| --- | --- |
| `POST /v1/chat/completions` | OpenAI 兼容。`model` 填档位名，省略则用默认档。支持流式。余额为 0 返回 402 `insufficient_quota`；档位要求付费而用户只有试用额度返回 403 `paid_required`；错误体为 OpenAI 格式 |
| `POST /v1/responses` | OpenAI Responses 协议，规则同上（档位、余额、流式、错误格式）。`response.model` 也会回写成档位名 |
| `POST /v1/messages` | Anthropic Messages 协议。密钥放 `x-api-key`（Anthropic SDK 默认）或 `Authorization: Bearer` 都行，值是账号中心的登录令牌；只透传 `anthropic-version`、`anthropic-beta`；`message_start` 里的 `message.model` 回写成档位名；错误体是 Anthropic 格式（`{"type":"error","error":{"type":"billing_error",…}}`）。这三个协议能否真正跑通，还取决于档位对应的模型在 New API 里有没有渠道，以及 New API 的协议转换 |
| `POST /v1/search`、`POST /v1/fetch` | 联网搜索与网页抓取，格式以 DeepSeek 为准，见上文「联网搜索与网页抓取」 |
| `GET /v1/models` | 当前产品启用的档位名 |
| `GET /api/ai/tiers` | 档位的展示名、说明、对当前用户是否可用；不含真实模型名 |
| `GET /api/ai/balance` | 各有效额度桶、合计、付费部分、试用是否还可领取。只读，不会触发开通 |
| `GET /api/ai/ledger` | 分页流水（`before`、`limit`）。每条带 `title`（如“兑换码入账”），不含管理员备注和操作人 |
| `POST /api/ai/redeem` | `{code}`，兑换到令牌 `aud` 所属产品。错误码：`code_invalid`（格式或校验位不对、系统里没有）、`code_used`、`code_void`、`code_wrong_product`、`too_many_attempts`（429） |
| `GET /api/health` | 健康检查 |

管理接口 `/api/admin/*` 与账号中心同一套做法：独立管理员表、HS256 令牌（由主密钥派生，与用户令牌不通用）、连续错 5 次锁 15 分钟、没有默认账号、写操作入审计表。包含登录与改密码、概览、网关连接（`GET /api/admin/settings`、`PUT /api/admin/settings/newapi` 先向网关验证再保存、`POST /api/admin/settings/newapi/test`；令牌只写不读，只返回来源和尾号）、产品与档位增改删、New API 模型列表（标记倍率疑似未配置的模型）、用户额度查看、补发与扣回、对账异常、操作日志，以及兑换码：`POST /api/admin/batches` 生成批次、`GET /api/admin/batches` 批次列表、`POST /api/admin/batches/{id}/void` 作废批次里未兑换的码、`POST /api/admin/codes/lookup` 查单个码（用 POST，码不进 URL 和访问日志）、`POST /api/admin/codes/{id}/void` 作废单个码。

公开档位列表通过 `reasoning_efforts` 返回允许的思考等级，不返回真实模型。管理后台可逐档位选择自动识别、手动声明或不支持；数据库 `product_tiers.reasoning_efforts` 为 nullable `TEXT[]`，缺省自动识别，空列表禁用。Responses 的 `reasoning.effort`、Chat Completions 的 `reasoning_effort` 与 Messages 的 `output_config.effort` 在转发前按档位校验，非法选择返回 `unsupported_reasoning_effort`。省略参数保持上游默认；网关不会替不支持的模型模拟等级。

上下文超限返回稳定错误码 `context_length_exceeded`；请求体超过本服务或上游限制时返回 `request_too_large`（HTTP 413）。错误说明提示压缩会话、减少图片或检查协议，保留三种协议各自的错误形状，不回传上游模型、渠道或完整错误内容。客户端可按错误码恢复，普通参数错误仍为 `bad_request`。

## 兑换码

- 格式：16 个字符，字母表去掉 0/O/1/I，前 15 个随机、最后 1 个是校验位，显示成 `XXXX-XXXX-XXXX-XXXX`。校验位能发现任何单个字符写错和绝大多数相邻字符颠倒，输错时不用查库就能拦下。输入时容忍小写、空格、各种连字符、全角字符和复制带来的零宽字符。
- 存储：数据库只存带密钥（由主密钥派生）的 HMAC，拿到数据库也看不到可用的码，也不能离线穷举。**明文只在生成批次的那一次响应里返回**，之后无法找回；后台的弹窗强制先下载或复制再关闭。
- 一批 = 一个产品 + 一种类型（加油包/月卡）+ 一个面额 + 一个有效期，最多 2000 个；对应淘宝上的一个商品。有效期从用户兑换那天开始算，不是从生成那天。
- 兑换：先验码、再开通账号（需要访问 New API，不能持有数据库锁），最后在一个事务里锁住这个码、标记已兑换、生成额度桶。并发兑换同一个码只会成功一次。推送到 New API 失败不影响结果，后台对账会补上。
- 防乱猜：只有“用户输错了”才计失败（码不对、已用过、已作废、产品不对），成功不占额度；同一账号 15 分钟内错 8 次、同一来源 IP 错 60 次就暂时锁住，锁住期间连正确的码也不受理。限流在进程内，多实例时各自计数。
- 退款：没兑换的码在后台“查询一个码”里作废；已兑换的码在同一处“扣回这份额度”（沿用额度桶扣回，已经用掉的不退）。

## 依赖的 New API 改动

本服务需要 New API 提供「管理员替普通用户创建令牌」的接口 `POST /api/user/:id/token`（上游没有）。补丁在 fork `bestyize/new-api` 的分支 `feat/admin-create-user-token`，基于线上运行的提交 `46dd6a6`。

## 上线前检查

- New API 的「新用户注册额度」必须为 0，否则每个新用户在网关里会多出一笔不在账本里的额度（对账会记为异常并对齐，但不该靠它兜底）。
- 给用到的每个模型在 New API 配置真实倍率。线上现状是多数模型仍是默认倍率。
- 登录后台「网关连接」页填 New API 管理令牌，只含 `user:read`、`user:write` 两个作用域。令牌不写进 `.env`。
- 首次登录后台后改掉初始管理员密码，并从 `.env` 里删掉 `AI_ADMIN_PASSWORD`。
- 备份主密钥。
- 账号中心对该产品的客户端 `aud` 要与产品的 `client_ids` 一致。

## 已知限制

- 同步有最长一个周期（默认 60 秒）的延迟：额度桶过期后到钱包被扣掉之间，用户可能多用一点；单次调用的实际花费可能略超余额（New API 先预扣再结算）。
- 在线支付尚未实现（设计里的四期）。
- 集成测试在断言失败时会跳过清理，留下 `ai_test_*` 的 schema；本地跑红过之后记得清一下。
- 推送到 New API 成功但本地提交失败的极小窗口里，下一轮对账会按「钱包比预期多/少」处理，可能产生一条异常记录或一次误判的消耗。
