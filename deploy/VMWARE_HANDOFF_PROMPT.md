# 交给另一个 AI 的 VMware 部署提示词

请接手 XiangBan（下班了么）项目的本地 VMware 部署。目标是快速预览，并让同一 Wi-Fi 的手机访问。已有部署曾验证正常，请先检查当前状态，再按需更新，保留已有数据。

## 环境与访问

- Windows 项目目录：`C:\Users\sdjz0\Documents\Claude\XiangBan`。
- 项目技术栈：React/Vite 前端、Kotlin/Ktor 后端、PostgreSQL、Redis，使用 Docker Compose。
- VMware Ubuntu VM 的 VMX：`D:\vm\ubuntu_read\Ubuntu 64-bit.vmx`。
- 当前采用 NAT，VM 地址：`192.168.164.128`，SSH 端口：`22`，用户：`read`。
- 本机可用 SSH 密钥文件：`C:\Users\sdjz0\.ssh\codex_ubuntu_vm_ed25519`。直接使用文件，不输出密钥内容。
- SSH 示例：`ssh -i C:\Users\sdjz0\.ssh\codex_ubuntu_vm_ed25519 read@192.168.164.128`。
- VM 部署目录：`/home/read/xiaban/current`。
- 环境配置：`/home/read/xiaban/.env`，权限为 `600`，已有数据库密码和 JWT 密钥，应保留。
- Windows WLAN 地址曾为 `192.168.1.26`，手机入口为 `http://192.168.1.26:18080`。先核对当前 IP。
- 宿主机入口转发到 VM 的 `192.168.164.128:18080`。已有 Windows portproxy 与防火墙配置，不需要重复创建。

## 已有部署方式

VM 已安装 Docker Engine 和 Compose 插件。`read` 直接访问 Docker socket 曾被拒绝，可使用 sudo；需要凭证时再向用户询问。Windows 管理员权限需要 UAC，宿主机 PowerShell 并不是 VM 终端。

优先使用仓库内已有文件：`compose.yaml`、`server/Dockerfile.runtime`、`deploy/compose.vmware-prebuilt.yaml` 和 `deploy/DEPLOYMENT.md`。

为了减少 VM 下载和编译时间，之前在 Windows 项目根目录成功构建了后端安装产物：

```powershell
.\gradlew.bat :server:installDist -x test --offline --no-daemon
```

产物目录为 `server/build/install/server`。更新后端时，重新构建并将安装产物与需要的源码上传到 VM 对应目录；若依赖未缓存，按需去掉 `--offline`。前端改动需要重新构建 web 镜像。

在 VM 部署目录运行：

```sh
cd /home/read/xiaban/current
sudo docker compose --project-name xiaban --env-file /home/read/xiaban/.env -f compose.yaml -f deploy/compose.vmware-prebuilt.yaml up -d --build
```

当前局域网预览配置曾为：

```dotenv
XIABAN_DOMAIN=192.168.164.128
PUBLIC_BASE_URL=http://192.168.1.26:18080
WEB_BIND_ADDRESS=192.168.164.128
VITE_DATA_OVERSEAS=false
```

手机使用宿主机地址访问。如果改动分享链接或邮件链接，检查 `PUBLIC_BASE_URL` 是否也需要调整为手机可达的宿主机地址；该变量也参与前端构建，调整后按需重建。持久化卷是 `xiaban_postgres_data`、`xiaban_redis_data`。

## Windows 局域网转发

已有映射：`192.168.1.26:18080 → 192.168.164.128:18080`。

可用 `netsh interface portproxy show v4tov4` 查看。防火墙规则名称：`XiangBan Local Preview TCP 18080`，允许 TCP 18080 入站，LocalAddress 为 `192.168.1.26`，RemoteAddress 为 `LocalSubnet`，Profile 为 `Any`。

如果地址变化，更新对应转发和防火墙规则。只开放局域网所需入口。此前尝试桥接后 VM 未获得 IPv4，已恢复 NAT，请继续沿用 NAT 和宿主机转发。

## 下载与配置注意事项

Ubuntu 软件源已换为清华 TUNA，安全更新曾保留官方源；Docker CE 使用华为云 APT 镜像。VM IPv6 连接曾失败，可在 APT 下载时使用 `Acquire::ForceIPv4=true`。

Docker Hub 曾下载很慢；宿主机有 Meta Tunnel，尚未证实它是慢速原因。需要调整代理时说明影响并让用户操作。清华 Ubuntu APT 镜像不会直接加速 Docker Hub 镜像。

Windows 上传源码可能丢失执行位，已有 Dockerfile 对 `gradlew` 或安装脚本执行 chmod。保留用户原有的 Gradle 配置和其他未提交改动。

2026-10-03 已增加 `/admin` 管理后台和 `/login` 邮箱注册／登录入口，并部署到 VM。管理员可以在后台保存 SMTP 设置、发送测试邮件和修改密码。配置写入 PostgreSQL 的 `runtime_settings` 表并加密，保存后立即生效。尚未填写真实 SMTP 凭证时，用户请求验证码会返回 503，不会宣称邮件已发出。用户明确要求不接微信登录。

管理员用户名为 `admin`，初始凭证在 Windows 项目根目录的 `.env.admin-initial`（Git 忽略文件）中。不要输出该文件或将其上传到源码包。首次登录所用的 `ADMIN_BOOTSTRAP_PASSWORD` 和持久化配置加密密钥 `SETTINGS_ENCRYPTION_KEY` 在 VM 的 `.env` 中。管理员密码变更后以数据库哈希为准；不要覆盖或重新生成已有的加密密钥。数据库迁移 `V9__admin_settings.sql` 创建了管理后台所需表。

更新前已备份数据库与环境文件到 `/home/read/xiaban/backups/pre-email-admin-20261003.sql`、`pre-email-admin-20261003.env`，旧镜像保留为 `xiaban-server:before-email-admin-20261003` 和 `xiaban-web:before-email-admin-20261003`。

随后已部署侧边栏版后台与用户管理：`/admin/users` 支持服务端分页、搜索、账号类型与引导状态筛选、创建日期筛选、排序、详情、运营备注和当前页 CSV 导出；`/admin/audit` 展示后台操作日志。迁移 `V10__admin_user_management.sql` 创建备注和审计表及索引。账号创建时间与邮箱验证时间是不同字段；不要把全部游客账号统计成邮箱注册用户。更新前数据库备份为 `/home/read/xiaban/backups/pre-user-management-20261003.sql`，旧镜像标签为 `before-user-management-20261003`。

Nginx 必须使用 `proxy_set_header Host $http_host` 保留外部端口；使用 `$host` 会丢掉 `:18080`，导致浏览器发送的 Origin 被后台误判。后台接口测试应携带与访问地址一致的 Origin，不能只验证未携带 Origin 的命令行请求。

用户列表已增加编辑和删除入口。编辑支持昵称、小伴名字/性格、城市、时区、作息和发薪日；删除要求输入目标用户编号，API 同时校验 `confirmUserId`。编辑与删除均有操作日志；账号删除后，JWT 鉴权会检查账号存在性并拒绝旧令牌。更新前数据库备份为 `/home/read/xiaban/backups/pre-user-edit-delete-20261003.sql`，旧镜像标签为 `before-user-edit-delete-20261003`。用户明确取消了邮箱 TLS/默认端口调整，此次没有改动邮箱设置。

最新版本已补上首页与“我的”顶部的登录入口，API 返回数字 `userId` 和服务端确认的 `accountType`。昵称迁入 `user_profiles` 并按 NFKC／大小写统一后的唯一键判重。头像前端压缩、后端校验为不超过 64KB 的 JPEG，存于 `user_avatars`。`V11__identity_and_feature_data.sql` 同时建立 `game_sessions`、`afterwork_choices`、`afterwork_preferences`，均通过数字 `user_id` 关联账号；打卡继续使用独立的 `checkins` 表。游戏包脚本也有变更，更新部署时必须上传 `web/public/game-packs`，不能只上传 `web/src`。

旧版浏览器安排记录通过用户明确点击导入按钮迁移。验证码邮件已增加 HTML 与纯文本版本，验证码有效期仍为 600 秒。完整数据模型与登录状态说明见 `deploy/ACCOUNT_DATA_MODEL.md`。本次备份为 `/home/read/xiaban/backups/pre-identity-20261003.sql`，旧镜像标签为 `before-identity-20261003`。实际接口已验证昵称唯一性、头像大小与图片访问、游戏/安排的用户隔离，验证用的临时账号已删除。真实邮件收件尚未测试。

根据用户要求完成“游客数据仅存当前浏览器，不持久化数据库”的改造：未登录访问与退出登录绝不会新增数据库账号，历史未绑定邮箱的游客账号已通过 `V12__cleanup_guest_accounts.sql` 彻底清除；游客打卡、小伴档案设置、下班安排与小游戏对局记录均保存在浏览器 `localStorage` 中；“我的”、游戏广场及打卡结果页明确提示：“游客数据仅保存在当前浏览器；清除浏览器数据、使用无痕模式或换设备后无法找回。创建账号后可保存并跨设备使用。”新用户完成邮箱验证并指定唯一用户名后才正式在数据库创建用户记录与数字 ID。已部署并验证通过。

## 交付检查

检查容器状态，并从宿主机分别访问 VM 与宿主机转发入口。此前首页返回 HTTP 200，`/api/health` 返回：

```json
{"status":"ok","db":true,"redis":true}
```

交付时说明实际可用地址、更新内容和未配置的功能。宿主机健康检查通过不等于手机已经验证成功；手机访问仍需用户实际确认。
