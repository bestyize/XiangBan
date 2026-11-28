# 小伴产品矩阵总站（`xiaban.xyz`）部署说明

`client/xiaban-portal` 为纯静态前端站点（Vue 3 + Vite），展示用 `.webp` 截图统一托管在资源机 `https://res.xiangban.xyz/img/xiaban-portal/`，HTML/JS/CSS 产物由香港云服务器宿主机 Nginx 托管。

> 严格遵守云服务器部署规范：**一律在本地完成测试与构建**，仅上传静态产物与 WebP 图片。

## 一键构建并发布

```sh
HK_HOST=ubuntu@43.155.99.113 \
RES_HOST=ubuntu@124.220.64.203 \
SSH_KEY=~/.ssh/id_aihehua \
./infra/portal/deploy.sh
```

## 首次 HTTPS 证书配置

确认 `xiaban.xyz` 的 DNS `A` 记录已指向香港机 `43.155.99.113` 后，在香港机执行：

```sh
sudo certbot certonly --webroot -w /var/www/certbot -d xiaban.xyz
# 将 nginx.conf.example 写入 /etc/nginx/conf.d/xiaban.xyz.conf 后重载：
sudo nginx -t && sudo systemctl reload nginx
```
