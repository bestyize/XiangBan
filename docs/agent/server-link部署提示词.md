# server-link 部署提示词

本文只覆盖 server-link；手机互联涉及账号中心、AI 网关、论坛、中转四个服务，整体部署用 [手机互联整体部署提示词](手机互联整体部署提示词.md)。

把下面“提示词”整段交给负责部署的 AI（需要它能 SSH 到服务器、能用 Docker 和 Nginx）。它不需要读完整个仓库，所需信息都在提示词和文中链接的文档里。部署前请你准备好：服务器 SSH 访问、域名解析、一个能登录小伴账号的测试账号（只给令牌，不给密码）。

## 提示词

```text
你是一名运维工程师，任务是把“小伴”项目的互联中转服务 server-link 部署到生产服务器。仓库是 bestyize/XiangBan（main 分支），先读这三份文件再动手：
- docs/deployment/LINK_DEPLOYMENT.md（部署步骤与验证清单，以它为准）
- services/server-link/README.md（接口、环境变量、隐私与限制）
- infra/link/compose.yaml、infra/link/.env.production.example、infra/link/nginx.conf.example、services/server-link/Dockerfile

背景：server-link 让手机和电脑上的小伴通过它转发端到端加密的消息。它只存设备、公钥和配对关系，不存消息正文；用账号中心的 JWKS 离线验签。技术栈是 Rust + PostgreSQL，端口 55030，路径前缀 /api/link，包含 WebSocket（/api/link/ws）。

要求与约束：
1. 独立部署栈：新建目录（例如 /home/ubuntu/xiangban-link），自带 PostgreSQL，不改动也不依赖账号中心、主站、市场等现有栈和它们的数据卷。动手前先看清服务器上已有的容器、Nginx 配置、端口占用，不要覆盖已有文件；有疑问先停下来问我。
2. 镜像：Dockerfile 只含运行时，二进制必须在 Ubuntu 24.04 x64 上编译（glibc 版本要匹配）。优先使用 GitHub Actions 工作流 “BuildXiaoBanWallpaper”（勾选 `package_server_link`，可取消 `build_release`）产出的 xiangban-link-<标签>.tar.gz；没有的话按 LINK_DEPLOYMENT.md 在构建机手工构建。不要把 Windows 或 macOS 编译的二进制传上去。
3. 配置：.env 权限 600，LINK_DB_PASSWORD 用 openssl rand -hex 24 生成；LINK_ACCOUNT_ISSUER 必须与线上账号中心的 ACCOUNT_ISSUER 完全一致（先去账号中心的 .env 或 /.well-known/jwks.json 所在域名核对）。任何密钥、令牌都不要打印到日志、聊天记录或提交到仓库。
4. 先决条件检查：线上账号中心必须登记了客户端 xiaoban-android（新版本默认内置；若线上是旧版本，需要升级账号中心，或在其 .env 的 ACCOUNT_CLIENTS 里补上 xiaoban-android:小伴手机应用 后重启）。这一步会重启账号中心，动手前告诉我并等我确认。
5. Nginx：使用 infra/link/nginx.conf.example，域名 link.xiangban.xyz（若不同请告诉我，因为桌面端默认值写死了这个域名）。重点确认：/api/link/ws 带 Upgrade/Connection 头、proxy_read_timeout 至少 3600s、关闭缓冲；服务只监听 127.0.0.1:55030，安全组/防火墙不对外开放 55030 和数据库端口。证书用 certbot，改完先 nginx -t 再 reload。
6. 安全：容器保持 read_only、cap_drop ALL、no-new-privileges（compose 里已有，别删）；数据库网络保持 internal。不要开启任何调试端口。

验证（全部通过才算完成，逐条把实际命令和结果贴给我，令牌之类敏感信息打码）：
1. docker compose ps 显示 postgres 与 link 均 healthy；curl 127.0.0.1:55030/api/link/health 返回 JSON。
2. curl https://link.xiangban.xyz/api/link/health 返回 JSON；不带令牌 GET /api/link/devices 返回 401。
3. 向我索取一个测试账号的访问令牌（aud 为 xiaoban-android 或 xiaoban-desktop-wallpaper），然后：POST /api/link/devices 登记一台名为“部署自检”的设备（public_key 用 32 字节随机数的 base64）应成功；用同一令牌按 WebSocket 子协议 ["xb-link","xiaoban-token.<令牌>"] 连 wss://link.xiangban.xyz/api/link/ws?device_id=<刚登记的设备编号>，应在 10 秒内收到状态帧并保持连接超过 5 分钟；最后 DELETE 这台设备清理现场。
4. 重启 link 容器后 health 恢复，数据库数据仍在。
5. 做一次 pg_dump 备份并确认文件非空，把备份命令写进部署目录的 README，说明回滚到旧镜像标签的步骤。

交付：部署目录路径、镜像标签、已修改的 Nginx 文件路径、备份与回滚方法，以及你发现的任何与文档不一致之处（直接指出文档哪里要改，不要自己改仓库）。不要做提示词之外的事情，尤其不要重启或改动与本任务无关的服务。
```

## 部署完成之后

1. 桌面端和手机端的默认地址是 `https://link.xiangban.xyz/api/link`；域名不同要先改默认值再发版。
2. 在 GitHub Actions 里运行 “BuildXiaoBanWallpaper”（勾选 `package_android_app`，可取消 `build_release`），下载 `xiaoban-android-debug-apk` 装到手机，桌面端“设置 → 连接手机”扫码配对，按 [LINK_DEPLOYMENT.md](../deployment/LINK_DEPLOYMENT.md) 第 5 条做一次真机完整场景，并把结果记到实施计划的 L3 一节。
