# 互联中转服务（server-link）部署

服务说明见 [server-link](../../services/server-link/README.md)，独立部署栈见 [infra/link/compose.yaml](../../infra/link/compose.yaml)，环境模板见 [.env.production.example](../../infra/link/.env.production.example)，Nginx 示例见 [nginx.conf.example](../../infra/link/nginx.conf.example)。中转自带 PostgreSQL，数据卷、备份和发布节奏与账号中心、主站分开。让 AI 代做部署时，直接用 [server-link部署提示词](../agent/server-link部署提示词.md)。

中转只存设备、公钥和配对关系，**不存消息**；暂存的离线通知只在内存里，重启即丢。因此它是单实例服务，升级时已连接的设备会掉线并自动重连（桌面和手机都有退避重连）。

## 先决条件

- 账号中心已部署，并且是**包含 `xiaoban-android` 客户端的版本**（该客户端在启动时登记，默认已内置；老版本需要升级，或在 `ACCOUNT_CLIENTS` 里补上 `xiaoban-android:小伴手机应用`）。中转用账号中心的 JWKS 离线验签，`LINK_ACCOUNT_ISSUER` 必须与账号中心的 `ACCOUNT_ISSUER` 完全一致。
- 一个域名（桌面端默认 `link.xiangban.xyz`，路径前缀 `/api/link`），指向服务器并签好证书。换域名要同步改桌面端的默认值（`phone_link.rs` 的 `DEFAULT_RELAY_URL`）。
- 服务器有 Docker Engine、Compose 插件和宿主机 Nginx。

## 构建镜像

和账号中心一样，镜像只含运行时，二进制在 Ubuntu 24.04 x64 上编译。可以用 GitHub Actions 的 “Server Link Tests” 工作流（勾选 package）得到 `xiangban-link-<标签>.tar.gz` 产物，也可以手工：

```sh
cargo build --release --locked --manifest-path services/server-link/Cargo.toml
LINK_IMAGE_TAG=$(git rev-parse --short=12 HEAD)
staging=$(mktemp -d)
cp services/server-link/target/release/server-link "$staging/server-link"
docker build -f services/server-link/Dockerfile -t "xiangban-link:$LINK_IMAGE_TAG" "$staging"
docker save "xiangban-link:$LINK_IMAGE_TAG" | gzip > "link-$LINK_IMAGE_TAG.tar.gz"
```

## 首次部署

在服务器上建独立目录（例如 `/home/ubuntu/xiangban-link`），放入镜像包、`infra/link/compose.yaml`，把环境模板保存为 `.env`（权限 `600`），填好 `LINK_IMAGE_TAG` 和 `LINK_DB_PASSWORD`（`openssl rand -hex 24`）。

```sh
gunzip -c link-<标签>.tar.gz | docker load
docker compose --env-file .env -f compose.yaml config --quiet
docker compose --env-file .env -f compose.yaml up -d --wait
curl -fsS http://127.0.0.1:55030/api/link/health
```

按 [Nginx 示例](../../infra/link/nginx.conf.example) 配域名与 HTTPS。要点：`/api/link/ws` 必须带 `Upgrade` / `Connection` 头并把读超时放到 1 小时以上、关闭缓冲；服务只发布在 `127.0.0.1:55030`，安全组不开放它和数据库端口。

## 验证

1. `curl -fsS https://<域名>/api/link/health` 返回 JSON。
2. 不带令牌访问 `GET /api/link/devices` 返回 401（`unauthorized`）。
3. 用一个测试账号的访问令牌（`aud` 为 `xiaoban-desktop-wallpaper` 或 `xiaoban-android`）调用 `POST /api/link/devices`（`{"kind":"desktop","name":"部署自检","public_key":"<32 字节随机数的 base64>"}`）应返回设备编号；再 `DELETE /api/link/devices/<编号>` 清理。
4. 用同一令牌通过 WebSocket 子协议 `["xb-link","xiaoban-token.<令牌>"]` 连 `wss://<域名>/api/link/ws?device_id=<编号>`，应在 10 秒内收到状态帧，连接保持一小时以上不被 Nginx 断开。
5. 最终以真机为准：桌面端“设置 → 连接手机”出二维码，手机扫码配对成功，手机发一条消息电脑能收到。

## 升级、回滚与备份

升级前备份数据库和 `.env`，加载新镜像，更新 `.env` 的 `LINK_IMAGE_TAG`，再执行 `up -d --wait`；保留旧镜像标签以便回滚。表结构由服务启动时按迁移创建。

```sh
docker compose --env-file .env -f compose.yaml logs -f link
docker compose --env-file .env -f compose.yaml exec -T postgres pg_dump -U link xiaoban_link | gzip > link-$(date +%F).sql.gz
```

数据库丢失的后果有限：所有设备需要重新登记并重新配对，不会泄露消息内容。
