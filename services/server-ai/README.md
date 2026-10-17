# server-ai：相伴 AI 网关与额度系统

状态：一期已实现（未部署）。为产品矩阵提供统一的 AI 调用入口：身份只认[账号中心](../server-account/README.md)，上游模型经 New API 网关调用，额度按「账号 × 产品」隔离。设计取舍见 [docs/architecture/ai-gateway.md](../../docs/architecture/ai-gateway.md)。

技术栈：Rust Axum 0.8 + PostgreSQL（SQLx），管理后台为 Vue 3 + Vite + TypeScript（`web/`），由本服务直接托管。默认端口 55010。

## 它做什么

- 用账号中心的 JWKS 离线验签，令牌 `aud` 决定用户属于哪个产品（`products.client_ids`）。
- 首次使用某产品的 AI 时，在 New API 里为该 `(账号, 产品)` 创建一个独立用户和一个无限额度令牌，并发放该产品的试用额度。令牌只存在本服务（AES-256-GCM 加密），客户端永远拿不到。
- 额度以「桶」记账（试用、月卡、加油包），每个桶有总额、剩余和到期时间，先扣最早过期的。每次变化都写只追加的流水。
- 每分钟与 New API 对账：钱包比预期少 = 消耗，按桶结算；比预期多 = 有人手工改过，记为异常并按账本对齐。推送一律用增减，不用覆盖。
- 客户端只认**档位**（如 `standard`、`premium`），真实模型只在管理后台配置，随时可换。响应、流式事件和错误信息里都不出现真实模型名。

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
| `AI_NEWAPI_ADMIN_TOKEN` | 空 | New API 访问令牌（`nap_` 前缀），只授予 `user:read`、`user:write`，不要用 root。未设置时无法开通用户、对账和转发 |
| `AI_NEWAPI_DEFAULT_RATIO` | `37.5` | 未配置倍率的模型在 New API 里的默认倍率，保存档位时用它判断是否漏配 |
| `AI_QUOTA_PER_UNIT` | `500000` | New API 里 1 美元对应的额度点数，仅用于后台展示 |
| `AI_MASTER_KEY_BASE64` / `AI_DATA_DIR` | 空 / `ai_data` | 主密钥（至少 32 字节）；未设置时首次启动在数据目录生成 `master.key`。**必须备份**，丢失后已保存的 New API Key 无法解密 |
| `AI_ADMIN_USER` / `AI_ADMIN_PASSWORD` | 空 | 仅在数据库里还没有管理员时用来创建初始管理员；首次登录后请改密码并移除 `AI_ADMIN_PASSWORD` |
| `AI_SYNC_INTERVAL_SECS` | `60` | 对账间隔，最小 5 |
| `AI_MAX_BODY_BYTES` | `8388608` | 转发请求体上限 |
| `AI_TRUST_PROXY` / `AI_CORS_ORIGINS` / `AI_WEB_DIR` | 见 `.env.example` | 同账号中心 |

## 接口

用户接口用账号中心的 Bearer 令牌；`aud` 不属于任何已启用产品时返回 403。

| 方法 路径 | 说明 |
| --- | --- |
| `POST /v1/chat/completions` | OpenAI 兼容。`model` 填档位名，省略则用默认档。支持流式。余额为 0 返回 402 `insufficient_quota`；档位要求付费而用户只有试用额度返回 403 `paid_required`；错误体为 OpenAI 格式 |
| `GET /v1/models` | 当前产品启用的档位名 |
| `GET /api/ai/tiers` | 档位的展示名、说明、对当前用户是否可用；不含真实模型名 |
| `GET /api/ai/balance` | 各有效额度桶、合计、付费部分、试用是否还可领取。只读，不会触发开通 |
| `GET /api/ai/ledger` | 分页流水（`before`、`limit`） |
| `GET /api/health` | 健康检查 |

管理接口 `/api/admin/*` 与账号中心同一套做法：独立管理员表、HS256 令牌（由主密钥派生，与用户令牌不通用）、连续错 5 次锁 15 分钟、没有默认账号、写操作入审计表。包含登录与改密码、概览、产品与档位增改删、New API 模型列表（标记倍率疑似未配置的模型）、用户额度查看、补发与扣回、对账异常、操作日志。

## 依赖的 New API 改动

本服务需要 New API 提供「管理员替普通用户创建令牌」的接口 `POST /api/user/:id/token`（上游没有）。补丁在 fork `bestyize/new-api` 的分支 `feat/admin-create-user-token`，基于线上运行的提交 `46dd6a6`。

## 上线前检查

- New API 的「新用户注册额度」必须为 0，否则每个新用户在网关里会多出一笔不在账本里的额度（对账会记为异常并对齐，但不该靠它兜底）。
- 给用到的每个模型在 New API 配置真实倍率。线上现状是多数模型仍是默认倍率。
- 用于本服务的 New API 管理员令牌只含 `user:read`、`user:write` 两个作用域。
- 备份主密钥。
- 账号中心对该产品的客户端 `aud` 要与产品的 `client_ids` 一致。

## 已知限制

- 同步有最长一个周期（默认 60 秒）的延迟：额度桶过期后到钱包被扣掉之间，用户可能多用一点；单次调用的实际花费可能略超余额（New API 先预扣再结算）。
- 兑换码、在线支付尚未实现（设计里的二期、四期）。
- 推送到 New API 成功但本地提交失败的极小窗口里，下一轮对账会按「钱包比预期多/少」处理，可能产生一条异常记录或一次误判的消耗。
