# AI订阅测评 QuotaReset（`aireset.xiaban.xyz`）香港机静态部署说明

`client/aireset-web` 是纯静态站点（零依赖，四个中英双语入口页 `/`、`/benchmark/`、`/en/`、`/en/benchmark/`，无后端、无数据库、无 Docker 容器），由香港云服务器宿主机 Nginx 直接托管并终结 HTTPS。

> 严格遵守云服务器部署规范：**一律在本地完成构建与自检**，只把 `dist/` 静态产物同步到服务器，服务器上不装 Node、不跑浏览器。

## 机器与域名

| 项 | 值 |
|---|---|
| 服务器 | 香港 `vpn1`（腾讯云），`ubuntu@43.155.99.113`，密钥 `~/.ssh/id_aihehua` |
| 域名 | `aireset.xiaban.xyz`（`xiaban.xyz` 的 DNS 托管在 DNSPod，与 `game` / `people` 同属香港机） |
| 站点目录 | `/var/www/aireset.xiaban.xyz` |
| Nginx 配置 | `/etc/nginx/conf.d/aireset.xiaban.xyz.conf` |
| 证书 | Let's Encrypt（certbot webroot：`/var/www/certbot`） |

登录必须显式写用户名并锁定密钥（`-i ~/.ssh/id_aihehua -o IdentitiesOnly=yes`），避免 fail2ban 误封。

## 首次上线

### 第 1 步：本地构建并同步静态产物

```sh
HK_HOST=ubuntu@43.155.99.113 SSH_KEY=~/.ssh/id_aihehua ./infra/aireset/deploy.sh
```

脚本会在本地执行 `npm test`（构建 + 无头浏览器预渲染 + 自检），再 `rsync --delete` 同步 `dist/`。

### 第 2 步：放置 HTTP 配置（先让站点可访问）

把 [nginx.http.conf.example](nginx.http.conf.example) 放到 `/etc/nginx/conf.d/aireset.xiaban.xyz.conf`，然后 `sudo nginx -t && sudo systemctl reload nginx`。未解析时可在本机用 `curl --resolve aireset.xiaban.xyz:80:43.155.99.113 http://aireset.xiaban.xyz/` 验证。

### 第 3 步：DNS 解析

在 `xiaban.xyz` 的 DNSPod 解析里添加（当前已存在，指向香港机）：

| 主机记录 | 类型 | 记录值 |
|---|---|---|
| `aireset` | `A` | `43.155.99.113` |

### 第 4 步：签发证书并切 HTTPS

DNS 生效后（在服务器上 `dig +short aireset.xiaban.xyz` 能返回 `43.155.99.113`）：

```sh
sudo certbot certonly --webroot -w /var/www/certbot -d aireset.xiaban.xyz
# 再把 nginx.conf.example 放到 /etc/nginx/conf.d/aireset.xiaban.xyz.conf
sudo nginx -t && sudo systemctl reload nginx
```

## 后续更新

重新运行第 1 步那一条命令即可，无需停机。

## 上线后给搜索引擎

站点已带 `robots.txt`、`sitemap.xml`、每页独立 `title/description/canonical` 与 JSON-LD，并且表格内容已预渲染进静态 HTML。HTTPS 生效后建议在 Google Search Console 添加域名属性并提交 `https://aireset.xiaban.xyz/sitemap.xml`；再从小伴官网或社区页脚挂一条到本站的链接，能明显加快收录。
