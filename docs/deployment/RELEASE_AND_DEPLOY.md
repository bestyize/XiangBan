# 打包发布与更新线上服务

一条流水线：**触发打包 → 拿到 GitHub Release 里的产物 → 在服务器上更新对应服务**。打包入口是工作流 `BuildXiaoBanWallpaper`（只能手动或用接口触发，推送代码不会触发）。本页给人看，也给 AI 照着做；让本地 AI 通过 SSH 代做时，直接用 [发布与更新服务端提示词](../agent/发布与更新服务端提示词.md)。

## 1. 工作流能打什么

| 输入 | 默认 | 产出 |
| --- | --- | --- |
| `windows` | 开 | Windows x64 一键安装版，上传到客户端 Release `xiaoban-wallpaper-v<桌面版本>` |
| `macos` | 开 | macOS aarch64 dmg，同上 |
| `android` | 开 | 小伴手机 App（Compose）的 Android APK，同上。配了 `XIAOBAN_ANDROID_KEYSTORE_*` 密钥是发布签名的 `XiaoBan-<版本>-Android.apk`；没配就是可安装的 `…-Android-debug.apk`（未签名的 release 包装不上手机）。旧的“小伴壁纸”Android 包会在重跑时从 Release 里清掉 |
| `servers` | 空 | 服务端镜像，编号逗号分隔或 `all`，上传到**单独的** Release `server-<UTC 时间>-<提交前 7 位>`，只保留最近 5 个 |

服务编号和每个服务的端口、健康检查、部署栈、文档位置都在 [`infra/images/services.json`](../../infra/images/services.json)：

| 编号 | 服务 | 镜像 | 线上部署方式（以现状为准，见第 4 节） |
| --- | --- | --- | --- |
| `account` | 统一账号中心 | `xiangban-account` | Docker 栈 `infra/account` |
| `ai` | AI 网关 | `xiangban-ai` | 先确认线上有没有、怎么部署的 |
| `forum` | 论坛 | `xiangban-forum` | 香港机 systemd 服务 `server-forum`（用 bundle 替换文件） |
| `link` | 手机与桌面互联中转 | `xiangban-link` | Docker 栈 `infra/link` |
| `market` | 小伴市场 | `xiangban-market` | Docker 栈 `infra/market` |
| `resource` | 资源文件服务 | `xiangban-resource` | 先确认 |
| `wallpaper-collect` | 壁纸采集服务 | `xiangban-wallpaper-collect` | 先确认 |



每个服务在 Release 里有两个包：

- `<镜像>-<标签>.tar.gz`：`docker save` 的镜像，`docker load` 就能用。`<标签>` 是提交前 12 位。
- `<镜像>-<标签>-bundle.tar.gz`：二进制和前端静态资源，给不用 Docker 的主机直接替换文件。

另有 `SHA256SUMS`（所有包的校验和）和 `manifest.json`（每个服务的镜像标签、文件名、校验和、端口、健康检查路径、部署栈和文档）。**更新服务前先用 manifest.json 对表，不要凭记忆。**

## 2. 触发打包

只打服务端、不碰客户端（AI 最常用的形式）：

```sh
gh workflow run BuildXiaoBanWallpaper --ref main \
  -f windows=false -f macos=false -f android=false -f servers=link,forum
```

等它跑完：

```sh
sleep 20
run_id=$(gh run list --workflow=BuildXiaoBanWallpaper --limit 1 --json databaseId --jq '.[0].databaseId')
gh run watch "$run_id" --exit-status      # 失败时用 gh run view "$run_id" --log-failed 看原因
gh release list --limit 30 --json tagName --jq '[.[] | select(.tagName | startswith("server-"))][0].tagName'   # 最新的 server-… 就是这次的
```

注意：

- **选哪些服务看改了什么。** 看上次部署的提交（上一个 server-… Release 的 `manifest.json` 里有 `commit`）到现在改过哪些目录：`services/server-<x>/` 对应各自的服务；`libs/` 下的改动会影响所有依赖它的 Rust 服务（用 `grep -rn 'path = "../../libs' services/*/Cargo.toml` 查），拿不准就多选，不要漏。
- 在 `main` 上打包；分支上的提交没合入就不要部署到线上。
- 同样的输入重复触发会取消前一次还在跑的那个。
- 手机 App 和桌面版是客户端，不用 SSH 更新：打包成功后产物就在 `xiaoban-wallpaper-v<版本>` Release 里。

## 3. 拿到产物

```sh
tag=server-20261029-1530-abc1234           # 上一步看到的最新 server-… 标签
mkdir -p /tmp/release-$tag && cd /tmp/release-$tag
gh release download "$tag" -p 'manifest.json' -p 'SHA256SUMS' -p 'xiangban-link-*' -p 'xiangban-forum-*'
sha256sum --check --ignore-missing SHA256SUMS     # 必须全部 OK，否则停下
cat manifest.json                                  # 对一下镜像标签、提交
```

## 4. 在服务器上更新

**总原则**：一次只动一个服务；每个服务先备份、再换、再验证；失败就回滚并停下来告诉人，不要连着往下做。

### 顺序

账号中心 → AI 网关 → 论坛 → 互联中转 → 其他。账号中心是所有服务登录的根，更新它会重启并短暂中断全站登录，**先告诉人并等确认**；它的 `ACCOUNT_ISSUER` 不能变。

### 4.1 Docker 栈（`account`、`link`、`market`，以及确认是 Docker 的其他服务）

```sh
# 本机：把镜像传到服务器（服务器上的目录见各服务的部署文档，例如 /home/ubuntu/xiangban-link）
scp xiangban-link-<标签>.tar.gz <主机>:<部署目录>/

# 服务器：
cd <部署目录>
docker compose --env-file .env -f compose.yaml ps                       # 记下当前在跑的镜像标签（回滚用）
grep '_IMAGE_TAG' .env
cp .env .env.bak-$(date +%F-%H%M)
gunzip -c xiangban-link-<标签>.tar.gz | docker load
sed -i 's/^LINK_IMAGE_TAG=.*/LINK_IMAGE_TAG=<标签>/' .env               # 变量名见 services.json 的 tagVar
docker compose --env-file .env -f compose.yaml config --quiet
docker compose --env-file .env -f compose.yaml up -d --wait
curl -fsS http://127.0.0.1:<端口><健康检查路径>                         # 端口和路径见 manifest.json
docker compose --env-file .env -f compose.yaml logs --tail 50 <服务名>   # 日志里不能有 panic / error
```

有数据库迁移的服务（账号中心、论坛、AI、市场、互联中转等，启动时自动迁移），**更新前先备份数据库**（每个服务的部署文档里有 `pg_dump` 命令）。

回滚：把 `.env` 里的标签改回记下的旧值，`up -d --wait`；迁移是向前兼容追加的，回滚旧镜像一般可用，若新版本做了不兼容迁移要用更新前的备份恢复。

### 4.2 systemd 上的论坛（用 bundle 替换文件）

论坛现在是香港机 `/home/ubuntu/xiangban-forum`、systemd 服务 `server-forum`、监听 `127.0.0.1:55010`，库在 `wallpaper_postgres` 容器里。

```sh
# 服务器：
cd /home/ubuntu/xiangban-forum
systemctl status server-forum --no-pager | head -5                       # 先确认在跑
mkdir -p backup-$(date +%F-%H%M) && cp server-forum backup-*/ && cp -r web/dist backup-*/dist   # 备份当前二进制和前端（按实际路径）
docker exec wallpaper_postgres pg_dump -U postgres xiaban_forum | gzip > backup-$(date +%F-%H%M)/db.sql.gz
# 解包 bundle：里面是 server-forum 和 web/
mkdir -p /tmp/forum-new && tar -xzf xiangban-forum-<标签>-bundle.tar.gz -C /tmp/forum-new
sudo systemctl stop server-forum
cp /tmp/forum-new/server-forum ./server-forum && rm -rf web/dist && mkdir -p web && cp -r /tmp/forum-new/web web/dist
sudo systemctl start server-forum
curl -fsS http://127.0.0.1:55010/api/health
journalctl -u server-forum -n 50 --no-pager
```

以上路径来自论坛 README，AI 动手前要先 `ls` 核对（可执行文件、`web/dist` 实际在哪、`.env` 里 `FORUM_WEB_DIR`），不一致以服务器上的为准。回滚：停服务、把备份的二进制和 `dist` 放回去、启动。

镜像 `xiangban-forum` 也已经打好，将来要把论坛迁到 Docker 时直接可用（需要挂 `/data` 存上传文件，并配 `FORUM_DATABASE_URL`、账号中心地址等环境变量，见论坛 README）；迁移前不要在线上用它替换 systemd 版本。

### 4.3 现状不明的服务（`ai`、`resource`、`wallpaper-collect`）

**先只读摸清**：`docker ps`、`systemctl list-units --type=service | grep -i -E 'ai|resource|wallpaper|xiaban'`、`ss -ltnp`、各部署目录。确认它是 Docker 还是 systemd 之后，分别照 4.1 / 4.2 做。拿不准或发现和文档不一致，停下来问人，不要自己猜着改。

- `ai`（AI 网关）：同机常见端口冲突，镜像默认端口 55040，实际用的端口以服务器配置为准；它在 `/data` 里生成并保存 `master.key`，**更新时数据目录必须保持不变**，丢了会解不开已存的上游密钥。

## 5. 更新后的检查

1. 每个更新过的服务：健康检查返回正常、日志里没有 panic、`docker compose ps` 显示 healthy（或 `systemctl is-active`）。
2. 更新了账号中心、论坛、AI、互联中转之一，按 [手机互联上线清单](MOBILE_LINK_ROLLOUT.md) 里对应一节的“检查”做一遍。
3. 记录：更新了哪个服务、从哪个标签到哪个标签、备份放在哪、耗时。留给下一次回滚用。

## 6. 本地手工打包（没有 GitHub 时）

同一个脚本，和 CI 完全一样：

```sh
infra/images/build-image.sh link ./out            # 需要 cargo、node、docker；只想验证编译和装配：STAGE_ONLY=1
python3 infra/images/gen-dockerfiles.py --check   # 改了 services.json 之后重新生成 Dockerfile：去掉 --check
```

新增一个服务：在 `services.json` 加一项、运行 `gen-dockerfiles.py`、提交生成的 `services/<服务>/Dockerfile`。
