# 小伴游戏厅（`game.xiaban.xyz`）香港机静态部署说明

`client/xiaoban-games-h5` 为纯前端静态站点（Vue 3 + Vite + PWA 离线缓存 + 8 款官方游戏静态包，合计约 4 MB），无需后端进程、无需数据库、无需 Docker 容器；由香港云服务器宿主机 Nginx 直接静态托管并终结 HTTPS。

> 严格遵守云服务器部署规范：**一律在本地完成游戏同步、测试与构建**，只把 `dist/` 静态产物同步至服务器。

---

## 首次上线（3 步完成）

### 第 1 步：配置 DNS 解析
在 `xiaban.xyz` 域名解析控制台添加一条 `A` 记录：
- 主机记录：`game`
- 记录类型：`A`
- 记录值：`<香港云服务器公网 IP>`

### 第 2 步：在香港机配置 Nginx 与 HTTPS 证书
登录香港云服务器执行：

```sh
sudo mkdir -p /var/www/game.xiaban.xyz /var/www/certbot
```

先放置最小 80 端口配置申请 Let's Encrypt 证书，再替换为本目录 [nginx.conf.example](nginx.conf.example)：

```sh
# 1. 临时 80 配置（用于 certbot 校验）
sudo tee /etc/nginx/conf.d/game.xiaban.xyz.conf >/dev/null <<'EOF'
server {
    listen 80;
    listen [::]:80;
    server_name game.xiaban.xyz;
    root /var/www/game.xiaban.xyz;
    index index.html;
    location /.well-known/acme-challenge/ { root /var/www/certbot; }
    location / { try_files $uri $uri/ /index.html; }
}
EOF
sudo nginx -t && sudo systemctl reload nginx

# 2. 签发 HTTPS 证书
sudo certbot certonly --webroot -w /var/www/certbot -d game.xiaban.xyz

# 3. 替换为完整 HTTPS 配置（nginx.conf.example）并重载
sudo nginx -t && sudo systemctl reload nginx
```

### 第 3 步：本地一键构建并同步静态产物
在仓库根目录执行（替换为你的香港机 SSH 地址）：

```sh
HK_HOST=ubuntu@<香港机IP> ./infra/game/deploy.sh
```

脚本会自动完成：`sync-games` → `npm test` → `npm run build` → `rsync --delete` 同步 `dist/` 到 `/var/www/game.xiaban.xyz/`。后续每次更新游戏包或界面，只需重新运行这一条命令即可零停机发版。
