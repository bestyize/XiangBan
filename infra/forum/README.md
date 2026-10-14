# 超级小伴论坛部署栈

论坛服务说明见 [services/server-forum](../../services/server-forum/README.md)。这个目录是它的独立部署栈：自带 PostgreSQL，数据卷、备份和发布节奏都与主站 [infra/compose.yaml](../compose.yaml) 和账号中心 [infra/account](../account/compose.yaml) 分开。换机器、换环境（例如香港之外的独立环境）时直接整套搬走。

| 文件 | 说明 |
|------|------|
| [Dockerfile](Dockerfile) | 只含运行时的镜像；二进制与前端产物由 CI 构建后放入上下文 |
| [compose.yaml](compose.yaml) | 论坛 + PostgreSQL 两个服务；论坛只绑回环，入口是宿主机 Nginx |
| [.env.production.example](.env.production.example) | 服务器端环境变量模板，复制为 `.env` 后 `chmod 600` |
| [nginx.conf.example](nginx.conf.example) | 宿主机 Nginx 反代示例，含上传体积与限流 |

打包与部署由 [.github/workflows/forum-deploy.yml](../../.github/workflows/forum-deploy.yml) 完成。

## 工作流做了什么

`forum-deploy.yml` 只有手动触发一种方式（Actions → Forum deploy → Run workflow）：

| 输入 | 效果 |
|------|------|
| 不勾选 `deploy` | 只构建：前端产物、release 二进制、运行时镜像，导出镜像包与独立二进制包，产物保留 14 天 |
| 勾选 `deploy` | 先构建，再经 SSH 把镜像包和 compose 文件传到服务器，`docker load` 后 `compose up -d --wait`，首页探活失败则任务失败 |

`deploy` 任务限定 `main` 分支。镜像不经过镜像仓库，直接用 `docker save` 传给服务器，不需要仓库登录凭据。产物 `forum-binary-linux-x64-*` 是单独的二进制和 `dist`，不使用 Docker 时可以直接拿去用，运行方式见论坛 README。

## 首次部署（新服务器 / 新环境）

工作流的 `deploy` 需要在 GitHub 配好变量与密钥（见下），适合稳态更新。第一次把一套新环境跑起来时，按下面顺序：

1. **服务器准备**：装 Docker Engine 与 Compose 插件、Nginx；安全组开放 22、80、443，不要开放 55010 和数据库端口。确认 SSH 账号能免密 `sudo`（装软件用）。
2. **拿构建产物**：在本分支（或合并后的 main）手动跑一次工作流，不勾选 `deploy`，下载 `forum-build-<sha>` 产物，得到镜像包。
3. **上传镜像与栈**：把镜像包和 `compose.yaml` 放到服务器部署目录（如 `/home/ubuntu/xiangban-forum`），`gunzip -c 镜像包 | docker load`。
4. **写环境文件**：`cp .env.production.example .env`，填入 `FORUM_DB_PASSWORD`（`openssl rand -hex 24`）、`FORUM_PUBLIC_URL`、初始管理员与 `FORUM_ADMIN_JWT_SECRET`（`openssl rand -hex 24`），`chmod 600 .env`。
5. **启动**：在部署目录执行
   ```sh
   docker compose --env-file .env -f compose.yaml up -d --wait
   docker compose --env-file .env -f compose.yaml ps
   ```
   首次启动会自动建表并写入分类种子。
6. **Nginx 与证书**：参照 [nginx.conf.example](nginx.conf.example) 添加虚拟主机；域名 DNS 已指向服务器后，用 certbot 签发证书。没有域名可先用 IP + 80 端口联调，`FORUM_PUBLIC_URL` 设成 `http://IP`。
7. **打通登录**：浏览器登录是前端直连账号中心（跨域）。复用香港账号中心时，需要在它的 `ACCOUNT_CORS_ORIGINS` 里加上本论坛的 origin，并保证 `FORUM_ACCOUNT_ISSUER` 与账号中心的 `ACCOUNT_ISSUER` 完全一致（令牌 `iss` 校验严格相等，不能改成代理路径）。论坛自己的管理员后台不经过账号中心，不受影响。

之后更新版本：合并 main 后手动跑工作流并勾选 `deploy`，或在服务器上按「日常操作」手动更新。

## 一次性准备（让 deploy 任务可用）

GitHub 变量（Settings → Secrets and variables → Actions → Variables）：

| 名称 | 说明 |
|------|------|
| `FORUM_DEPLOY_HOST` | 服务器地址 |
| `FORUM_DEPLOY_USER` | SSH 用户，如 `ubuntu` |
| `FORUM_DEPLOY_PORT` | 可选，默认 22 |
| `FORUM_DEPLOY_PATH` | 部署目录绝对路径，如 `/home/ubuntu/xiangban-forum`，不要和主站的 `DEPLOY_PATH` 相同 |

GitHub 密钥（Secrets）：`FORUM_DEPLOY_SSH_KEY`（专用部署私钥）、`FORUM_DEPLOY_KNOWN_HOSTS`（`ssh-keyscan` 结果）。公钥放进服务器 SSH 账号的 `authorized_keys`。

## 日常操作

以下命令在服务器的 `FORUM_DEPLOY_PATH` 目录执行。

```sh
docker compose --env-file .env -f compose.yaml ps
docker compose --env-file .env -f compose.yaml logs -f forum
```

回滚到上一个版本（镜像标签是提交 SHA 的前 12 位，服务器上保留着之前加载过的镜像）：

```sh
docker images xiangban-forum
FORUM_IMAGE_TAG=<旧标签> docker compose --env-file .env -f compose.yaml up -d --wait
```

数据库表由服务启动时自动创建，新版本通常只会增加 `IF NOT EXISTS` 的表和索引；回滚到旧镜像不需要回退数据库。

备份数据库（Docker 卷本身不是备份），附件目录也要一起备份：

```sh
docker compose --env-file .env -f compose.yaml exec -T postgres pg_dump -U forum xiaban_forum | gzip > forum-$(date +%F).sql.gz
docker run --rm -v xiangban-forum_forum_uploads:/data -v "$PWD":/backup ubuntu:24.04 tar -czf /backup/forum-uploads-$(date +%F).tar.gz -C /data .
```

## 已知限制

- 每次部署会在服务器上多留一个约 100–200 MB 的镜像，需要定期手动清理不再用于回滚的旧标签（`docker image rm xiangban-forum:<旧标签>`）。
- 部署工作流没有备份步骤，也没有在升级前做数据库快照；重要数据先按上面的命令备份。
- Nginx 虚拟主机和证书是一次性的宿主机配置，不进工作流；换服务器要重配。
- 论坛没有独立健康接口，镜像用首页探活；前端产物损坏时也会表现为不健康。
