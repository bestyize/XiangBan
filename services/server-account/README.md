# server-account：相伴统一账号中心

状态：已实现（v1）。只负责账号基本信息与登录凭证：邮箱/手机号验证码验证身份、注册时设置唯一且不可修改的用户名与密码、用户名/邮箱/手机号加密码登录、头像、令牌签发，并带一个管理后台（用户、应用、操作日志）。不存任何产品业务数据。设计取舍见 [docs/architecture/account-center.md](../../docs/architecture/account-center.md)。

技术栈：Rust Axum 0.8 + PostgreSQL（SQLx），Ed25519（EdDSA）签发 JWT，Argon2 存密码，腾讯云短信/邮件发送验证码；管理后台为 Vue 3 + Vite + TypeScript（`web/`），由本服务直接托管。默认端口 55009。

## 运行与验证

```sh
cp services/server-account/.env.example services/server-account/.env   # 按需修改
npm --prefix services/server-account/web ci
npm --prefix services/server-account/web run build                    # 生成 web/dist，不构建则只提供 API
cargo run --manifest-path services/server-account/Cargo.toml
cargo test --locked --manifest-path services/server-account/Cargo.toml
```

管理后台在 `http://localhost:55009/admin/`。前端开发可用 `npm --prefix services/server-account/web run dev`（端口 5175，`/api` 代理到 55009）。

数据库表由服务启动时自动创建。集成测试需要 `TEST_DATABASE_URL`（每个测试建独立 schema，结束删除，不碰已有数据）；未设置时集成测试跳过，只跑单元测试：

```sh
docker run -d --name account-test-pg -e POSTGRES_PASSWORD=test -e POSTGRES_DB=account_test -p 55432:5432 postgres:16-alpine
TEST_DATABASE_URL=postgres://postgres:test@localhost:55432/account_test cargo test --locked --manifest-path services/server-account/Cargo.toml
```

## 打包与部署

当前没有账号中心专用构建工作流。在本地 Ubuntu 24.04 x64 或匹配服务器的平台构建 release 二进制和管理后台，再使用只含运行时的 [Dockerfile](Dockerfile) 打包镜像并上传。独立部署栈在 [infra/account](../../infra/account/compose.yaml)，完整流程见 [docs/deployment/ACCOUNT_CENTER_DEPLOYMENT.md](../../docs/deployment/ACCOUNT_CENTER_DEPLOYMENT.md)。

生成生产密钥（二进制自带，不要用 openssl，它生成的 Ed25519 私钥格式服务读不了）：

```sh
cargo run --manifest-path services/server-account/Cargo.toml -- gen-keys
```

不用 Docker 时，一并上传构建好的 `server-account` 二进制和 `web/dist/`，配置环境变量并将 `ACCOUNT_WEB_DIR` 指向静态目录。仓库当前不提供上述产物的自动下载入口。

## 配置

| 变量 | 默认 | 说明 |
| --- | --- | --- |
| `ACCOUNT_DATABASE_URL` | `postgres://xiaban@localhost:5432/xiaban_account` | PostgreSQL 连接串，独立库 |
| `ACCOUNT_HOST` / `ACCOUNT_PORT` | `0.0.0.0` / `55009` | 监听地址 |
| `ACCOUNT_ISSUER` | `https://account.xiangban.xyz` | JWT `iss`，各服务校验时必须一致 |
| `ACCOUNT_DATA_DIR` | `account_data` | 首次启动自动生成 `jwt_ed25519.pk8`（签名私钥）与 `master.key`（权限 0600），必须持久化并备份 |
| `ACCOUNT_JWT_PKCS8_BASE64` / `ACCOUNT_MASTER_KEY_BASE64` | 空 | 容器部署时用环境变量代替上面两个文件，主密钥至少 32 字节 |
| `ACCOUNT_CLIENTS` | 内置 5 个产品 | 启动时登记的应用，`id:名称,id2:名称2`，只增不删 |
| `ACCOUNT_ACCESS_TTL_SECS` / `ACCOUNT_REFRESH_TTL_DAYS` | `1800` / `30` | 访问 / 刷新令牌有效期；桌面客户端 `xiaoban-desktop-wallpaper` 的访问令牌固定为 30 天，其他应用使用此配置 |
| `ACCOUNT_CODE_TTL_SECS` / `ACCOUNT_RESEND_COOLDOWN_SECS` | `300` / `60` | 验证码有效期与重发间隔 |
| `ACCOUNT_MAX_CODE_ATTEMPTS` | `5` | 单个验证码最多错误次数 |
| `ACCOUNT_DAILY_PER_IDENTIFIER` / `ACCOUNT_DAILY_PER_IP` | `10` / `30` | 每日发送上限，防短信被刷 |
| `ACCOUNT_TRUST_PROXY` | `false` | 为 true 时用反向代理设置的 `X-Real-IP` 作为来源 IP；Nginx 需 `proxy_set_header X-Real-IP $remote_addr;`，且不能让外部直连本服务 |
| `ACCOUNT_CORS_ORIGINS` | 空（允许任意来源） | 逗号分隔的允许来源。凭证只走 Authorization 头、不用 Cookie |
| `ACCOUNT_EMAIL_CLIENTS` | `xiaban-web,xiaban-api` | 令牌 `aud` 属于这些应用时，`GET /api/me` 额外返回已验证邮箱明文 `email`，供主后端按邮箱关联旧账号；其他应用只能看到掩码 |
| `ACCOUNT_ADMIN_USER` / `ACCOUNT_ADMIN_PASSWORD` | 空 | 仅在数据库里还没有任何管理员时用来创建初始管理员（密码至少 8 位）；已有管理员时被忽略，不会覆盖密码。首次登录后请在后台改密码并移除 `ACCOUNT_ADMIN_PASSWORD` |
| `ACCOUNT_ADMIN_TOKEN_TTL_SECS` | `28800` | 管理员登录有效期（8 小时） |
| `ACCOUNT_WEB_DIR` | `services/server-account/web/dist` | 管理后台静态文件目录，相对启动时的工作目录；容器里请设为绝对路径。目录不存在时只提供 API |
| `ACCOUNT_SENDER` | `console` | 初始发信通道。`console` 仅把验证码写入日志，只能用于开发；生产设为 `tencent`，或在后台「发信设置」改用 SMTP（后台保存的配置优先） |

`ACCOUNT_SENDER=tencent` 时：

| 变量 | 说明 |
| --- | --- |
| `TENCENT_SECRET_ID` / `TENCENT_SECRET_KEY` | 腾讯云 API 密钥，建议用只授予短信与邮件发送权限的子账号 |
| `TENCENT_SMS_SDK_APP_ID` / `TENCENT_SMS_SIGN_NAME` / `TENCENT_SMS_TEMPLATE_ID` | 短信应用、已审核签名与模板；模板按顺序接收两个参数：验证码、有效分钟数 |
| `TENCENT_SMS_REGION` | 默认 `ap-guangzhou` |
| `TENCENT_SES_FROM` / `TENCENT_SES_TEMPLATE_ID` / `TENCENT_SES_SUBJECT` / `TENCENT_SES_REGION` | 邮件发信地址、模板（变量 `code`、`minutes`）、主题；默认地域 `ap-hongkong` |

未配置某一通道时，该类型标识符的发送返回 400 提示暂未开通。

## 接口

成功返回 JSON；失败统一为 `{"error":true,"code":"...","message":"..."}`，限流类错误带 `retry_after` 与 `Retry-After` 头。`identifier` 含 `@` 视为邮箱（转小写），否则按中国大陆手机号处理（规范化为 `+86…`）。`client_id` 必须是已登记应用。

| 方法 路径 | 鉴权 | 说明 |
| --- | --- | --- |
| `POST /api/auth/code/send` | 无 | `{identifier, purpose?}`，purpose 为 `login`（默认）或 `reset_password`。重置密码时账号不存在也返回成功但不发送 |
| `POST /api/auth/code/login` | 无 | `{identifier, code, client_id, device_name?}`。已有账号：返回令牌、`is_new_user:false`、`user`。**新的邮箱 / 手机号不建号**，返回 `{registration_required:true, registration_token, expires_in}`（凭证 15 分钟有效），客户端带它去 `register` |
| `POST /api/auth/register` | 无 | `{registration_token, username, password, client_id, device_name?}`，一次事务里建号并写入用户名、密码和已验证标识，返回令牌与 `is_new_user:true`。错误码：`username_taken`（409，同一凭证可换名重试）、`identity_exists`（409）、`registration_expired`（401） |
| `GET /api/auth/methods` | 无 | `{email, phone}`：当前开放哪些**新用户注册**方式（默认只开邮箱）。客户端登录框据此收窄“验证码登录 / 注册”页的入口，取不到就按只开邮箱处理；密码登录、重置密码不受影响 |
| `GET /api/auth/username/available?username=` | 无 | `{available, message}`，格式不对或保留词也返回 200 并说明原因。每 IP 每分钟 60 次，只是提示，唯一性以 `register` 为准 |
| `POST /api/auth/password/login` | 无 | `{identifier, password, client_id, device_name?}`，`identifier` 可以是用户名、邮箱或手机号；账号不存在、没设密码、密码错误提示完全一致；连续错 5 次锁 15 分钟 |
| `POST /api/auth/password/reset` | 无 | `{identifier, code, new_password}`，成功后吊销该用户全部会话 |
| `POST /api/auth/refresh` | 无 | `{refresh_token, client_id}`，轮换刷新令牌 |
| `POST /api/auth/logout` | 无 | `{refresh_token}`，吊销该设备会话 |
| `POST /api/auth/logout-all` | Bearer | 吊销全部设备会话 |
| `GET /api/me` / `PATCH /api/me` | Bearer | 读取资料（`account_id` 数字字符串、从 13586 起，`username` 唯一，二者用户本人都不可修改）；`PATCH` 只能改 `avatar_url`（https，空串清除） |
| `POST /api/me/password` | Bearer | `{current_password, new_password}`，必须带当前密码；新密码不能和用户名相同 |
| `POST /api/me/identities/code` | Bearer | `{identifier}`，给待绑定的邮箱/手机号发验证码 |
| `POST /api/me/identities` | Bearer | `{identifier, code}`，绑定；每类最多一个，已被他人占用返回 409 |
| `GET /.well-known/jwks.json` | 无 | Ed25519 公钥（JWKS），业务服务离线验签用 |
| `GET /api/health` | 无 | 健康检查 |

令牌响应：`{access_token, refresh_token, token_type:"Bearer", expires_in}`。

### 管理接口（`/api/admin/*`）

管理员与普通用户是两套体系：`admins` 表独立，令牌用主密钥派生的 HS256 密钥签发（普通用户令牌是 EdDSA），互相不能通用，业务服务也拿不到验签或伪造管理员令牌所需的密钥。管理员令牌带 `token_version`，改密码后旧令牌立即失效；每次请求都会查库确认管理员仍存在。登录连续错 5 次锁 15 分钟。

| 方法 路径 | 说明 |
| --- | --- |
| `GET/PUT /api/admin/signup-methods` | Admin | 读取 / 修改上面的开关 `{email, phone}`，存在 `system_settings`，改动写入审计 `signup_methods.update`。保存时至少开放一种，且当前发信通道必须能发送对应验证码（手机号需腾讯云短信），否则 400 |
| `POST /api/admin/login` | `{username, password}`，返回 `{token, expires_in, admin}` |
| `GET /api/admin/me` | 当前管理员 |
| `POST /api/admin/password` | `{current_password, new_password}`，成功后所有管理员令牌失效 |
| `GET /api/admin/stats` | 用户总数/停用数/24 小时与 7 天新增/24 小时登录、有效会话数、24 小时验证码发送量及来源 IP 前 5 |
| `GET /api/admin/users` | 参数 `q`、`status`（active/disabled）、`page`、`page_size`（最大 100）。`q` 若是 UID 则精确匹配，若是完整邮箱/手机号则精确匹配已绑定标识，同时也按昵称模糊匹配。标识只返回掩码 |
| `GET /api/admin/users/{uid}` | 资料、状态、有效会话列表 |
| `POST /api/admin/users/{uid}/status` | `{status: "active"｜"disabled"}`；停用时立即吊销该用户全部会话 |
| `POST /api/admin/users/{uid}/revoke-sessions` | 下线该用户所有设备 |
| `GET /api/admin/clients` / `POST /api/admin/clients` | 应用列表（含有效会话数）/ 新增 `{client_id, name}` |
| `PATCH /api/admin/clients/{client_id}` | `{name?, enabled?}`；停用的应用不能登录也不能刷新令牌 |
| `GET /api/admin/audit` | 最近的操作日志，参数 `limit`（默认 50，最大 200） |
| `GET /api/admin/sender` / `PUT /api/admin/sender` | 读取 / 保存发信通道 `{provider: console｜smtp｜tencent, smtp?}`；SMTP 密码只返回 `has_password`，保存时留空表示沿用已存密码 |
| `POST /api/admin/sender/test` | `{recipient, smtp?}`，用表单里尚未保存的 SMTP 参数或当前通道发一封测试验证码邮件 |

后台「发信设置」保存的通道用主密钥加密存进 `system_settings`，立即热切换，重启后优先于 `ACCOUNT_SENDER`。SMTP 通道只发邮件，手机号验证码会返回 400。

停用用户、下线设备、应用增删改和管理员改密码都会写入 `admin_audit`。已发出的访问令牌无法撤回，停用后最长仍可在业务服务里再存活 `ACCOUNT_ACCESS_TTL_SECS`（默认 30 分钟）。

## 业务服务如何接入

1. 用 JWKS 公钥按 `kid` 验证 access token（EdDSA），校验 `iss` 等于账号中心的 `ACCOUNT_ISSUER`、`exp` 未过期，并检查 `aud` 是自己所属的 `client_id`。
2. 用 `sub`（全局 `uid`）作为自己业务用户表的外键，首次出现时懒创建业务用户行。
3. 缓存 JWKS（例如 1 小时），遇到未知 `kid` 时刷新一次。
4. 昵称、头像按需调用账号中心 `GET /api/me` 读取，不要自行复制存储为权威数据。

## 数据与安全

- 邮箱/手机号不存明文：`identifier_hash`（HMAC，用于查找与唯一约束）+ `identifier_enc`（AES-256-GCM，用于展示掩码）。两者的密钥由 `master.key` 派生；丢失主密钥将无法解密已有标识符，也无法再用哈希匹配登录，必须备份。
- 验证码只存 HMAC 哈希，一次性使用，用途（login / reset_password / bind）参与哈希，不可跨用途使用。
- 刷新令牌只存 SHA-256；轮换时旧令牌立即作废，已作废的令牌再次出现视为泄露，整个令牌族吊销。
- 限流存于 PostgreSQL（标识符冷却与日上限、IP 日上限），不依赖 Redis。检查与写入非原子，极端并发下可能略微超限。
- 签名私钥只在本服务；各业务服务只持有公钥，无共享密钥。

## 已知限制与后续

- 腾讯云短信/邮件的 TC3 签名与请求体按官方文档实现，仅有本地签名单测，尚未对真实账号联调；上线前需用真实签名模板验证。
- 仅支持中国大陆手机号；微信、QQ、Apple 等需另行接入，`identities.kind` 已可扩展。
- 暂无用户自助注销账号、更换已绑定邮箱/手机号的接口；管理后台也不能代用户换绑、删除用户或新增管理员（目前只有初始管理员，更多管理员需直接写 `admins` 表）。
- `verification_codes`、已作废的 `refresh_tokens` 和 `admin_audit` 没有自动清理，长期运行后需要定期清理旧数据。
- 管理后台没有登录 IP 限流，只有账号级锁定；公网暴露时建议在 Nginx 层限制 `/admin/` 与 `/api/admin/` 的来源或加访问限流。
- 暂无托管登录页与 OIDC 授权码 + PKCE 流程；v1 由各客户端直接调用上述接口。桌面与 Android 端若不想自带登录表单，再补授权端点。
- 密码修改（已登录状态下）不会吊销其他设备的会话，忘记密码重置才会。


## 后台创建用户与论坛编辑账号

用户页新增“创建用户”，管理接口为 `POST /api/admin/users`：`{request_id,username,password,avatar_url?}`。`request_id` 是客户端生成并在失败重试时保留的 UUID。沿用用户名和密码规则；用户名不区分大小写唯一，密码仍用 Argon2 存储，创建和审计在同一事务中完成。相同请求重试返回原 UID，不生成重复账号。账号可用用户名和密码正常登录，未绑定任何邮箱/手机号，可在登录后自行完成验证绑定。

给论坛后台使用的专用建号通道为 `POST /api/internal/forum/users`，请求体相同。设置 `ACCOUNT_FORUM_PROVISION_KEY`（至少 32 字节随机值）后，以同值 Bearer 凭据调用；论坛配置项为 `FORUM_ACCOUNT_PROVISION_KEY`。未配置默认关闭，该凭据只能创建普通用户，不能管理其他用户或签发用户会话。账号中心管理员令牌与论坛管理员令牌仍不通用。
