# 账号中心部署

服务说明见 [services/server-account](../../services/server-account/README.md)，配置模板在 [infra/account](../../infra/account/compose.yaml)，打包与部署由 [.github/workflows/account-center.yml](../../.github/workflows/account-center.yml) 完成。

账号中心是独立部署栈：自带 PostgreSQL，数据卷、备份和发布节奏都与主站 `infra/compose.yaml` 分开。服务只绑定服务器回环地址 `127.0.0.1:55009`，对外由宿主机 Nginx 反代并终结 HTTPS。

## 工作流做了什么

`account-center.yml` 有三个任务：

| 任务 | 触发 | 内容 |
| --- | --- | --- |
| `verify` | PR、推送 main、手动 | 前端构建，`cargo test`（含 PostgreSQL 集成测试），校验 compose 配置，`check-layout.py` |
| `package` | 推送 main、手动 | 在 `ubuntu-24.04` x64 上构建 release 二进制和管理后台，打成 `ubuntu:24.04` 运行时镜像，用真实 PostgreSQL 做冒烟（健康检查、后台页面、JWKS、管理员登录，且以只读根文件系统运行），导出镜像与二进制两个产物，保留 14 天 |
| `deploy` | 仅手动，勾选 `deploy`，且在 main 分支 | 经 SSH 把镜像包和 compose 文件传到服务器，`docker load` 后 `compose up -d --wait`，健康检查不通过则任务失败 |

推送 main 只会测试和打包，不会部署。要部署：在 Actions 里选 “Account center”，Run workflow，分支选 main，勾选 `deploy`。

镜像不经过镜像仓库，直接用 `docker save` 传给服务器，所以不需要仓库登录凭据。产物 `account-binary-linux-x64` 里是单独的二进制和 `dist`，不使用 Docker 时可以直接拿去用，运行方式见服务 README。

## 一次性准备

### 1. GitHub 配置

复用主站部署用的服务器连接配置，只多一个路径变量：

| 类型 | 名称 | 说明 |
| --- | --- | --- |
| Variable | `DEPLOY_HOST`、`DEPLOY_USER`、`DEPLOY_PORT`（可选） | 与主站相同 |
| Secret | `DEPLOY_SSH_KEY`、`DEPLOY_KNOWN_HOSTS` | 与主站相同 |
| Variable | `ACCOUNT_DEPLOY_PATH` | 账号中心在服务器上的目录，绝对路径，例如 `/home/ubuntu/xiangban-account`，不要和主站的 `DEPLOY_PATH` 相同 |

### 2. 服务器环境文件

服务器需要 Docker Engine 和 Compose 插件。把模板传上去并填写：

```sh
ssh ubuntu@YOUR_SERVER "mkdir -p /home/ubuntu/xiangban-account"
scp infra/account/.env.production.example ubuntu@YOUR_SERVER:/home/ubuntu/xiangban-account/.env
ssh ubuntu@YOUR_SERVER "chmod 600 /home/ubuntu/xiangban-account/.env && nano /home/ubuntu/xiangban-account/.env"
```

需要填：`ACCOUNT_DB_PASSWORD`（只用字母数字，`openssl rand -hex 24`）、`ACCOUNT_ISSUER`（对外域名，带 `https://`）、初始管理员、腾讯云密钥与短信/邮件模板。

两个密钥要在第一次部署之后、镜像已加载的服务器上生成，因为必须用服务自带的 `gen-keys`（OpenSSL 生成的 Ed25519 私钥是 PKCS#8 v1，服务读不了）：

```sh
docker run --rm xiangban-account:latest gen-keys
```

把输出的两行 `ACCOUNT_JWT_PKCS8_BASE64=`、`ACCOUNT_MASTER_KEY_BASE64=` 粘贴进 `.env`。所以第一次部署会在 `up` 时因缺少这两个变量失败，这是预期的：镜像此时已经加载好，生成密钥后再运行一次部署即可。

> `ACCOUNT_MASTER_KEY_BASE64` 丢失后，库里加密的邮箱和手机号无法解密，所有用户都无法再用原标识登录。`.env` 要和数据库备份一起离线保存。`ACCOUNT_ISSUER` 上线后不要改，会让全部已发令牌失效。

### 3. Nginx 与证书

参照 [infra/account/nginx.conf.example](../../infra/account/nginx.conf.example) 添加虚拟主机，用 certbot 签发证书。重点：

- 必须用 `proxy_set_header X-Real-IP $remote_addr` 覆盖客户端自带的请求头，服务按它做验证码的 IP 限额。
- 发验证码、管理后台、其余接口分别限流；不需要公网访问后台时，用 `allow`/`deny` 限定来源。
- 服务端口只发布在回环地址，不要把 `ACCOUNT_BIND_ADDRESS` 改成 `0.0.0.0`。

安全组只需开放 22、80、443，不要开放 55009 和数据库端口。

### 4. 腾讯云短信与邮件

短信签名和模板需要先通过审核，这是上线前耗时最长的一步。模板按顺序接收两个参数：验证码、有效分钟数。邮件模板使用变量 `code` 与 `minutes`。通道配置缺失时服务会启动失败，不会退化成把验证码打进日志。

## 日常操作

以下命令在服务器的 `ACCOUNT_DEPLOY_PATH` 目录执行。

```sh
docker compose --env-file .env -f compose.yaml ps
docker compose --env-file .env -f compose.yaml logs -f account
```

回滚到上一个版本（镜像标签是提交 SHA 的前 12 位，服务器上保留着之前加载过的镜像）：

```sh
docker images xiangban-account
ACCOUNT_IMAGE_TAG=<旧标签> docker compose --env-file .env -f compose.yaml up -d --wait
```

数据库表由服务启动时自动创建，新版本只会增加 `IF NOT EXISTS` 的表和索引；回滚到旧镜像不需要回退数据库。

备份数据库（Docker 卷本身不是备份）：

```sh
docker compose --env-file .env -f compose.yaml exec -T postgres pg_dump -U account xiaban_account | gzip > account-$(date +%F).sql.gz
```

## 已知限制

- 每次部署会在服务器上多留一个约 100 MB 的镜像，需要定期手动清理不再用于回滚的旧标签（`docker image rm xiangban-account:<旧标签>`）。
- 部署工作流没有备份步骤，也没有在升级前做数据库快照。
- 工作流已写好但没有在 GitHub 上实际运行过；本地只验证了 compose 配置、镜像结构和容器启动。首次使用请先看一遍 `verify` 与 `package` 的结果，再勾选 `deploy`。
