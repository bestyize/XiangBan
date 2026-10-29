# server-link：手机与桌面互联的中转服务

状态：已实现（2026-10-29 改为同账号直连，去掉了配对与密钥），部署见 [LINK_DEPLOYMENT](../../docs/deployment/LINK_DEPLOYMENT.md)。协议见 [设备互联协议](../../docs/agent/设备互联协议.md)，计划见 [实施计划_手机桌面互联](../../docs/product/实施计划_手机桌面互联.md)。

小伴手机和电脑上的小伴通常连不上对方，这个服务让两端各自主动连到这里，由它转发文本消息。**用同一个小伴账号登录的手机和电脑自动互相看到对方**，不需要配对。它不存任何消息历史，也不加密消息（不是金融服务，安全边界放在账号上，见协议文档 §6）。

技术栈：Rust Axum 0.8 + PostgreSQL（SQLx），默认端口 55030。

## 它做什么

- **设备登记**：设备用账号令牌登记自己（`kind`: `desktop` / `phone`、名称、客户端随机生成的 `install_id`）；同一账号同一 `install_id` 幂等。
- **对端发现**：同一账号下、另一种类型的设备互为对端。连接后收到完整对端列表，之后新设备登记、改名、移除都会实时推送，不用重连。
- **转发**：设备通过 WebSocket 连上来，按对端关系互发文本；来源由中转按认证填写，不能伪造。
- **在线状态**：对端上线/下线实时通知。
- **离线**：对端不在线时，普通消息直接回 `undelivered`；只有发送方标了 `notify: true` 的才暂存（每个对端最多 50 条、24 小时），上线后按序补发。**指令类消息不要加 `notify`**，避免电脑上线后执行过时的指令。
- **限流与防滥用**：每连接令牌桶（消息数、字节数）、每账号设备数上限、单条消息 256 KiB 与单帧 512 KiB 上限。

## 接口

认证：`Authorization: Bearer <账号访问令牌>`，离线验签（JWKS，EdDSA）；令牌的 `aud` 必须是 `LINK_ACCOUNT_CLIENT_IDS` 之一（默认 `xiaoban-desktop-wallpaper` 和 `xiaoban-android`）。WebSocket 也可以把令牌放在子协议里：`["xb-link", "xiaoban-token.<令牌>"]`。账号中心要先登记 `xiaoban-android` 这个客户端。

| 接口 | 说明 |
| --- | --- |
| `GET /api/link/health` | 健康检查与在线连接数 |
| `POST /api/link/devices` | 登记设备 `{kind, name, install_id}` → `{device_id, created}` |
| `GET /api/link/devices` | 名下设备与在线状态 |
| `GET /api/link/peers?device_id=` | 这台设备的对端列表 |
| `DELETE /api/link/devices/{id}` | 移除设备：通知对端、向它推 `device_revoked` 并断开 |
| `GET /api/link/ws?device_id=` | WebSocket，帧格式见协议文档 §3 |

错误体：`{ "code", "message" }`。码：`unauthorized`、`not_found`、`bad_request`、`rate_limited`、`too_many_devices`。

## 配置（环境变量）

| 变量 | 默认 | 说明 |
| --- | --- | --- |
| `LINK_HOST` / `LINK_PORT` | `0.0.0.0` / `55030` | 监听地址 |
| `LINK_DATABASE_URL` | `postgres://xiaban:xiaban@localhost:5432/xiaoban_link` | 数据库 |
| `LINK_ACCOUNT_ISSUER` / `LINK_ACCOUNT_JWKS_URL` | `https://account.xiangban.xyz` / 其下 `/.well-known/jwks.json` | 账号中心 |
| `LINK_ACCOUNT_CLIENT_IDS` | `xiaoban-desktop-wallpaper,xiaoban-android` | 接受的令牌受众，逗号分隔 |
| `LINK_CORS_ORIGINS` | 空（任意） | 跨域来源 |
| `LINK_MAX_DEVICES_PER_USER` | `20` | 每账号设备数上限 |
| `LINK_NOTIFY_QUEUE_CAP` / `LINK_NOTIFY_TTL_SECS` | `50` / `86400` | 离线通知暂存 |
| `LINK_MSGS_PER_SEC` / `LINK_BYTES_PER_SEC` | `30` / `1048576` | 每连接限速 |

## 数据与隐私

一张在用的表 `devices`（设备编号、账号、类型、名称、`install_id`、移除时间）。没有消息表；暂存的通知只在内存里，进程重启即丢。日志只记请求路径、状态码和连接事件，不记录消息内容。

**从旧版（配对版）升级**：启动时自动给 `devices` 表加 `install_id` 列，旧版登记的设备（没有 `install_id`）被标记为已移除；旧表 `pair_codes`、`pairings` 不再使用，留在库里不动，确认无用后可手工删除。**旧版客户端与新版服务不兼容**，要一起更新。

## 已知限制

- 中转能看到消息内容（不加密），见协议文档 §6。
- 持账号令牌的人可以占用某台设备的连接（拿设备编号顶掉真连接）造成骚扰。
- 单实例：在线状态与暂存在内存里。要水平扩展需要引入共享的发布订阅（Redis 等），目前规模不需要。

## 验证

```sh
cargo test                                                       # 限流与 hub 的单元测试
TEST_DATABASE_URL=postgres://… cargo test --features testing     # 加上真实 Postgres + 真实 HTTP/WebSocket 的集成测试
```

集成测试覆盖：设备登记的校验与归属、幂等；同账号设备互相发现并收发；新设备登记 / 移除不用重连就能被对端看到；伪造来源、未知对端、超大消息、乱帧被拒；离线暂存与上限；同设备新连接顶替旧连接；设备移除；子协议认证；限流；乱帧与超大帧不影响其他连接。
