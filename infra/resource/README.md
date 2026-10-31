# 社区资源服务部署

2026-11-01 已接入香港服务器，与论坛使用独立进程和数据目录。公开入口为 `https://chat.xiangban.xyz/files`，Nginx 转发到 `127.0.0.1:55020`。

- 运行目录：`/home/ubuntu/xiangban-resource`，文件目录：`data/`。
- systemd 单元：本目录 `server-resource.service`。
- Nginx 片段：`nginx-files.conf`，放入现有 `chat.xiangban.xyz` HTTPS server；尾部斜线用于去掉 `/files/` 前缀。
- 资源服务 `.env` 和论坛 `.env` 中共享签名密钥，权限 600；密钥不进入仓库。
- 论坛 `FORUM_RESOURCE_BASE_URL=https://chat.xiangban.xyz/files`；资源回调为 `http://127.0.0.1:55010/api/internal/pack-uploaded`。

## 发布与检查

在开发机按 `services/server-resource/README.md` 构建 Linux x86_64 二进制，再上传替换并重启 `server-resource`，不在云端编译。`GET /files/health` 应返回 `{"ok":true}`。内容包通过论坛下载接口获取短期签名地址；封面使用公开 `/files/covers/…` URL。

官方资源按不可变对象键写入 `data/packs/`、`data/covers/`，确认 ZIP 结构、文件大小和 SHA-256 后，通过论坛 `POST /api/admin/packs` 登记。每个发布包再从公开下载入口取回核对哈希，并检查 Range 和未签名请求拒绝。普通用户仍使用上传凭证、资源直传和签名回调流程。

部署前备份位于论坛主机 `/home/ubuntu/xiangban-forum/backups/community-content-20261031/`，含数据库、论坛配置与 Nginx 配置；目录权限 700。配置备份包含密钥，只在受保护的服务器目录保留。
