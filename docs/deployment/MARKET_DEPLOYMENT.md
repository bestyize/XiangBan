# 小伴市场部署

服务说明见 [services/server-xiaoban-market](../../services/server-xiaoban-market/README.md)，配置模板在 [infra/market](../../infra/market/compose.yaml)，测试、打包与部署由 [.github/workflows/market.yml](../../.github/workflows/market.yml) 完成。

小伴市场是独立部署栈：自带 PostgreSQL，数据卷、备份和发布节奏都与主站、账号中心分开。服务只绑定服务器回环地址 `127.0.0.1:55011`（容器内端口仍是 55010；宿主机 55010 在香港已被论坛占用，可用 `MARKET_HOST_PORT` 调整），对外由宿主机 Nginx 反代并终结 HTTPS。

> 状态：配置和工作流已按账号中心的做法写好，并在本地校验过 compose 配置与 YAML 语法；**镜像构建、容器冒烟和 SSH 部署还没有在真实环境里运行过**，第一次用请先只跑「不勾选 deploy」的流程，确认测试与打包都通过。

## 工作流做了什么

`market.yml` 只能手动运行（Actions 页选 “Market” → Run workflow），有三个任务：

| 任务 | 内容 |
| --- | --- |
| `verify` | 前端单元测试与构建；`cargo test`（含 PostgreSQL 端到端测试）；校验 compose 配置 |
| `package` | 在 `ubuntu-24.04` x64 上构建 release 二进制和前端，打成 `ubuntu:24.04` 运行时镜像；用真实 PostgreSQL 做冒烟（只读根文件系统、健康检查、前端页面、公开接口、管理员登录）；导出镜像包和独立二进制包，保留 14 天 |
| `deploy` | 仅当勾选 `deploy` 且分支是 main：经 SSH 把镜像包和 compose 文件传到服务器，`docker load` 后 `compose up -d --wait`，健康检查不通过则任务失败 |

镜像不经过镜像仓库，直接 `docker save` 传给服务器，不需要仓库登录凭据。按仓库规范，服务器上不做任何编译。

## 一次性准备

### 1. GitHub 配置

复用主站部署用的服务器连接配置，只多一个路径变量：

| 类型 | 名称 | 说明 |
| --- | --- | --- |
| Variable | `DEPLOY_HOST`、`DEPLOY_USER`、`DEPLOY_PORT`（可选） | 与主站相同 |
| Secret | `DEPLOY_SSH_KEY`、`DEPLOY_KNOWN_HOSTS` | 与主站相同 |
| Variable | `MARKET_DEPLOY_PATH` | 小伴市场在服务器上的目录，绝对路径，例如 `/home/ubuntu/xiangban-market`，不要和主站、账号中心的目录相同 |

### 2. 服务器环境文件

服务器需要 Docker Engine 和 Compose 插件：

```sh
ssh ubuntu@YOUR_SERVER "mkdir -p /home/ubuntu/xiangban-market"
scp infra/market/.env.production.example ubuntu@YOUR_SERVER:/home/ubuntu/xiangban-market/.env
ssh ubuntu@YOUR_SERVER "chmod 600 /home/ubuntu/xiangban-market/.env && nano /home/ubuntu/xiangban-market/.env"
```

必填：

- `MARKET_DB_PASSWORD`：只用字母数字，`openssl rand -hex 24`。
- `MARKET_JWT_SECRET`：`openssl rand -hex 32`。**上线后不要改**，改了会让所有后台登录失效。

初始管理员：设置 `MARKET_ADMIN_USER` 和 `MARKET_ADMIN_PASS`；留空 `MARKET_ADMIN_PASS` 时，首次启动会随机生成密码，**只在日志里出现一次**（`docker compose … logs market | grep 初始密码`）。首次登录后在后台「设置」改密码，并从 `.env` 删除 `MARKET_ADMIN_PASS`——它只在库里还没有管理员时生效，之后会被忽略。

### 3. Nginx 与证书

参照 [infra/market/nginx.conf.example](../../infra/market/nginx.conf.example) 添加虚拟主机，用 certbot 签发证书。重点：

- **`X-Forwarded-For` 必须用 `$remote_addr` 覆盖**，不能用 `$proxy_add_x_forwarded_for`：服务拿它的第一段给下载计数去重，后者会让客户端伪造的值排在最前面。
- 服务本身**没有登录限流**，示例里对后台登录限制为每 IP 每分钟 6 次，务必保留。
- 上传技能安装包需要放宽请求体（示例里为 14 MB）。
- 不需要公网访问后台时，用 `allow`/`deny` 把 `/admin` 与 `/api/v1/admin` 限定到办公网络或 VPN。
- 服务端口只发布在回环地址，不要把 `MARKET_BIND_ADDRESS` 改成 `0.0.0.0`；安全组只开 22、80、443。

### 4. 相伴账号登录（可选）

1. 在账号中心登记应用：给账号中心配置 `ACCOUNT_CLIENTS=…,xiaoban-market:小伴市场`（或在它的后台「应用」里新增）。
2. 在小伴市场的 `.env` 里取消注释并填写 `MARKET_ACCOUNT_ISSUER`（必须等于账号中心的 `ACCOUNT_ISSUER`）。
3. 重新部署后，本地管理员在后台「设置 → 相伴账号管理员」里绑定账号。详见服务 README 的「相伴账号登录」。

## 第一次部署

先在 Actions 里运行一次不勾选 `deploy` 的流程，确认 `verify` 与 `package` 都通过；再勾选 `deploy` 运行。第一次部署时服务会自动迁移数据库并写入种子数据（分类和六个采集来源）。

部署后到后台「采集来源」配置并采集；魔搭的三个来源需要先填接口，见服务 README 的「配置魔搭」。

## 日常操作

以下命令在服务器的 `MARKET_DEPLOY_PATH` 目录执行：

```sh
docker compose --env-file .env -f compose.yaml ps
docker compose --env-file .env -f compose.yaml logs -f market
```

回滚到上一个版本（镜像标签是提交 SHA 的前 12 位，服务器上保留着之前加载过的镜像）：

```sh
docker images xiangban-market
MARKET_IMAGE_TAG=<旧标签> docker compose --env-file .env -f compose.yaml up -d --wait
```

数据库迁移只增不改，一般回滚到旧镜像不需要回退数据库；但回退跨过了新增迁移的版本时，旧程序会忽略新表和新列。

## 备份

两样东西都要备份，缺一不可：

```sh
# 数据库
docker compose --env-file .env -f compose.yaml exec -T postgres pg_dump -U market xiaoban_market | gzip > market-$(date +%F).sql.gz
# 安装包（按内容哈希存放；没有它们，已上架的技能无法下载）
docker run --rm -v xiangban-market_market_data:/data -v "$PWD":/backup ubuntu:24.04 tar -czf /backup/market-data-$(date +%F).tar.gz -C /data .
```

`.env`（含 `MARKET_JWT_SECRET`）也要和备份一起离线保存。

## 已知限制

- 采集任务跑在服务进程内：重启时未完成的任务会被标记为失败；**不要同时运行多个实例**，否则各实例的定时调度会重复触发。
- 采集需要服务器能访问 SkillHub 和魔搭；compose 里 `market` 接入了可出网的 `edge` 网络，数据库所在的 `backend` 网络不能出网。
