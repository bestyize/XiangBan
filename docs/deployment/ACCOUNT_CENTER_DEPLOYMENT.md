# 账号中心部署

服务说明见 [server-account](../../services/server-account/README.md)，独立部署栈见 [infra/account/compose.yaml](../../infra/account/compose.yaml)，环境模板见 [.env.production.example](../../infra/account/.env.production.example)。账号中心自带 PostgreSQL，数据卷、备份和发布节奏与主站分开。

当前仓库没有账号中心专用的 GitHub Actions 工作流。旧文档中的自动测试、打包产物下载和 Actions 部署入口已失效。按下面流程在本地 Linux 或构建机准备镜像，再上传服务器；服务器只加载镜像、启动服务。

## 构建镜像

[Dockerfile](../../services/server-account/Dockerfile) 只含运行时，要求构建上下文包含 `server-account` 二进制和 `web/` 静态资源。在 Ubuntu 24.04 x64 构建机的仓库根执行（构建平台须与服务器匹配；不要直接上传 Windows 二进制）：

```sh
npm --prefix services/server-account/web ci
npm --prefix services/server-account/web run build
cargo build --release --locked --manifest-path services/server-account/Cargo.toml

ACCOUNT_IMAGE_TAG=$(git rev-parse --short=12 HEAD)
staging=$(mktemp -d)
cp services/server-account/target/release/server-account "$staging/server-account"
cp -R services/server-account/web/dist "$staging/web"
docker build -f services/server-account/Dockerfile -t "xiangban-account:$ACCOUNT_IMAGE_TAG" "$staging"
docker save "xiangban-account:$ACCOUNT_IMAGE_TAG" | gzip > "account-$ACCOUNT_IMAGE_TAG.tar.gz"
```

保留镜像标签供升级和回滚。单独运行二进制时，将 `ACCOUNT_WEB_DIR` 指向随包静态目录；配置与验证命令见服务 README。

## 首次部署

服务器需要 Docker Engine、Compose 插件和宿主机 Nginx。将镜像包、`infra/account/compose.yaml` 和环境模板传到独立目录，例如 `/home/ubuntu/xiangban-account`；模板保存为 `.env`，权限设为 `600`。已有 `.env` 只按需补字段，不用模板覆盖。

先加载镜像，再用同一镜像生成生产密钥（将 `<镜像标签>` 替换成构建时的实际标签）：

```sh
gunzip -c account-<镜像标签>.tar.gz | docker load
docker run --rm xiangban-account:<镜像标签> gen-keys
```

把生成的 `ACCOUNT_JWT_PKCS8_BASE64`、`ACCOUNT_MASTER_KEY_BASE64` 写入 `.env`，并配置 `ACCOUNT_IMAGE_TAG`、`ACCOUNT_DB_PASSWORD`、`ACCOUNT_ISSUER`、初始管理员及腾讯云短信/邮件通道。数据库密码可用 `openssl rand -hex 24` 生成；签名密钥必须用服务的 `gen-keys`。先配齐再启动，无需故意让第一次部署失败。

`ACCOUNT_MASTER_KEY_BASE64` 用于解密邮箱和手机号，必须与数据库一起备份；更改 issuer 会使已有令牌失效。短信签名与模板需先审核通过，生产通道缺配置时服务会拒绝启动。

参照 [Nginx 模板](../../infra/account/nginx.conf.example) 配置域名与 HTTPS：覆盖 `X-Real-IP`、限制验证码和后台请求速率，服务继续只发布在 `127.0.0.1:55009`。安全组不开放服务和数据库端口。

在服务器部署目录执行：

```sh
docker compose --env-file .env -f compose.yaml config --quiet
docker compose --env-file .env -f compose.yaml up -d --wait
curl -fsS http://127.0.0.1:55009/api/health
```

## 升级、回滚与备份

升级前备份数据库和 `.env`，加载新镜像，更新 `.env` 的 `ACCOUNT_IMAGE_TAG`，再执行 `up -d --wait`。保留旧镜像标签。

```sh
docker compose --env-file .env -f compose.yaml ps
docker compose --env-file .env -f compose.yaml logs -f account
docker compose --env-file .env -f compose.yaml exec -T postgres pg_dump -U account xiaban_account | gzip > account-$(date +%F).sql.gz
```

表结构由服务启动时初始化，但旧数据库不保证兼容所有变更。历史上从 nickname 改为必填 username 的版本需要专门的数据迁移；当前文档不再提供基于“没有真实用户”假设的清表升级步骤。已有数据须先核对版本差异并制定迁移方案。

确认数据库兼容后，可选择已保留的旧镜像回滚：

```sh
ACCOUNT_IMAGE_TAG=<旧标签> docker compose --env-file .env -f compose.yaml up -d --wait
```

若表结构不兼容，需要配套数据库备份恢复。Docker 数据卷不能替代备份；没有工作流代为执行备份或数据库快照。
