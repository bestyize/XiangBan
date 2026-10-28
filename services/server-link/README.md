# server-link：手机与桌面互联的中转服务

状态：L1 已实现（2026-10-28），未部署。协议见 [设备互联协议](../../docs/agent/设备互联协议.md)，计划见 [实施计划_手机桌面互联](../../docs/product/实施计划_手机桌面互联.md)。

小伴手机和电脑上的小伴通常连不上对方，这个服务让两端各自主动连到这里，由它转发**端到端加密**的信封。它只认账号和配对关系，**看不到消息正文**，也不存任何消息历史。

技术栈：Rust Axum 0.8 + PostgreSQL（SQLx），默认端口 55030。

## 它做什么

- **设备与配对**：每台设备登记自己的公钥；桌面申请一次性配对码，手机扫码兑换，桌面验证手机的证明后确认。中转**不验证**证明（它不知道二维码里的一次性密钥），只原样转交，所以它无法冒充手机。
- **转发**：设备通过 WebSocket 连上来，中转按配对关系把信封转给对端；信封里的 `ct` 对中转是不透明的字节。
- **在线状态**：对端上线/下线实时通知；连接时补发当前状态。
- **离线**：对端不在线时，普通信封直接回 `undelivered`；只有发送方标了 `notify: true` 的才暂存（每个对端最多 50 条、24 小时），上线后按序补发。**指令类消息不要加 `notify`**，避免电脑上线后执行过时的指令。
- **限流与防滥用**：每连接令牌桶（消息数、字节数）、配对码兑换按账号限频、每码最多兑换次数、每账号设备数上限、单信封大小上限。

## 接口

认证：`Authorization: Bearer <账号访问令牌>`，离线验签（JWKS，EdDSA）；令牌的 `aud` 必须是 `LINK_ACCOUNT_CLIENT_IDS` 之一（默认 `xiaoban-desktop-wallpaper` 和 `xiaoban-android`）。WebSocket 也可以把令牌放在子协议里：`["xb-link", "xiaoban-token.<令牌>"]`。账号中心要先登记 `xiaoban-android` 这个客户端。

| 接口 | 说明 |
| --- | --- |
| `GET /api/link/health` | 健康检查与在线连接数 |
| `POST /api/link/devices` | 登记设备 `{kind, name, public_key}`；同一账号同一公钥幂等 |
| `GET /api/link/devices` | 名下设备与在线状态 |
| `DELETE /api/link/devices/{id}` | 撤销设备：解除所有配对、通知对端、断开连接 |
| `POST /api/link/pair-codes` | 桌面申请配对码 `{device_id}`（5 分钟有效，只存哈希，新码顶掉旧码） |
| `POST /api/link/pair` | 手机兑换 `{code, device_id, proof}`，返回桌面公钥，并向桌面推 `pair.request` |
| `POST /api/link/pairings/{id}/confirm` | 桌面确认（待确认请求超过配对码有效期则 410） |
| `POST /api/link/pairings/{id}/reject` | 桌面拒绝 |
| `GET /api/link/pairings?device_id=&status=active\|pending` | 配对列表；桌面取 pending 时带手机的证明 |
| `DELETE /api/link/pairings/{id}` | 解除配对，两端都收到 `unpaired` |
| `GET /api/link/ws?device_id=` | WebSocket，帧格式见协议文档 §4.1 |

错误体：`{ "code", "message" }`。码：`unauthorized`、`not_found`、`code_expired`（过期、不存在、不是这个账号的码，对外是同一个说法）、`code_used`、`too_many_attempts`、`rate_limited`、`too_many_devices`、`not_pending`、`pairing_expired`。

## 配置（环境变量）

| 变量 | 默认 | 说明 |
| --- | --- | --- |
| `LINK_HOST` / `LINK_PORT` | `0.0.0.0` / `55030` | 监听地址 |
| `LINK_DATABASE_URL` | `postgres://xiaban:xiaban@localhost:5432/xiaoban_link` | 数据库 |
| `LINK_ACCOUNT_ISSUER` / `LINK_ACCOUNT_JWKS_URL` | `https://account.xiangban.xyz` / 其下 `/.well-known/jwks.json` | 账号中心 |
| `LINK_ACCOUNT_CLIENT_IDS` | `xiaoban-desktop-wallpaper,xiaoban-android` | 接受的令牌受众，逗号分隔 |
| `LINK_CORS_ORIGINS` | 空（任意） | 跨域来源 |
| `LINK_PAIR_CODE_TTL_SECS` / `LINK_PAIR_CODE_MAX_REDEEMS` | `300` / `5` | 配对码有效期与最多兑换次数 |
| `LINK_PAIR_REDEEMS_PER_MINUTE` / `LINK_PAIR_CODES_PER_HOUR` | `10` / `20` | 兑换（每账号）与申请（每设备）限频 |
| `LINK_MAX_DEVICES_PER_USER` | `20` | 每账号设备数上限 |
| `LINK_NOTIFY_QUEUE_CAP` / `LINK_NOTIFY_TTL_SECS` | `50` / `86400` | 离线通知暂存 |
| `LINK_MSGS_PER_SEC` / `LINK_BYTES_PER_SEC` | `30` / `1048576` | 每连接限速 |

## 数据与隐私

三张表：`devices`（设备与公钥）、`pair_codes`（配对码哈希）、`pairings`（配对关系与手机的证明）。没有消息表；暂存的通知只在内存里，进程重启即丢。日志只记请求路径、状态码和连接事件，不记录信封内容。

## 已知限制

- 账号令牌被盗的人可以占用某台设备的连接（拿设备编号顶掉真连接）造成骚扰，但**看不到也伪造不了内容**；彻底解决需要设备用自己的密钥对连接做签名挑战，列入 v2。
- 单实例：在线状态与暂存在内存里。要水平扩展需要引入共享的发布订阅（Redis 等），目前规模不需要。
- 没有前向保密，见协议文档威胁模型。

## 验证

```sh
cargo test                                              # 限流与 hub 的单元测试
TEST_DATABASE_URL=postgres://… cargo test               # 加上真实 Postgres + 真实 HTTP/WebSocket 的集成测试（5 项）
```

集成测试覆盖：设备登记的校验与归属；完整配对（桌面验证手机的证明、别的账号兑换/确认被拒）；两端用 `libs/link-protocol` 派生密钥后经中转收发并解密；伪造发送方、未配对、超大信封、乱帧被拒；解除配对；配对码过期/用完/被顶掉；待确认请求在桌面离线时补发；离线暂存与上限；同设备新连接顶替旧连接；设备撤销；子协议认证；限流。
