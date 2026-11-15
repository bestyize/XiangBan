# server-resource：小伴资源机

社区内容包（zip）、封面图与已核验官方工具安装包的存放、直传校验与签名下载。部署在独立的 200 Mbps 机器上，主机（`server-forum`）只签发地址与凭证，不经手文件内容。无数据库，文件按对象键存在磁盘上。

## 接口

| 方法与路径 | 说明 |
| --- | --- |
| `PUT /upload` | 直传。头 `X-Upload-Ticket` 带论坛签发的上传凭证，请求体是文件原始字节。 |
| `GET /{file_key}?exp=&sig=` | 下载内容包，签名有效且未过期才返回；支持 `Range` 断点续传。 |
| `GET /img/{md5}.{ext}` | 新封面图，公开，不签名；按文件头确定扩展名。旧 `covers/` 键仍可读取。 |
| `POST /internal/tool-mirrors` | 论坛HMAC签名的内部镜像请求；固定官方来源，流式写入后回读SHA-256，返回绑定nonce/资产的签名回执。 |
| `GET/HEAD /tool-mirrors/{sha}/{filename}?exp=&sig=` | 已核验安装包的短期签名下载，支持Range；是否已发布由论坛入口校验。 |
| `GET /health` | 健康检查。 |

## 上传流程

1. 客户端向论坛申请凭证（`POST /api/packs/upload-ticket`），得到 `ticket` 与 `upload_url`。
2. 客户端 `PUT upload_url`，头 `X-Upload-Ticket: <ticket>`。资源机检查凭证、大小和 SHA-256，边接收边计算 MD5（超过声明大小立即中止，不进内存）。
3. 封面只收 PNG、JPEG、WebP（看文件头，不收 SVG 和 GIF）。内容包必须是 zip：条目 ≤ 2000、解压后合计 ≤ 128MB、路径不能含 `..` 或绝对路径、不允许符号链接、根目录必须有合法 JSON 的 `manifest.json`（≤ 64KB）。
   桌面 V2 的壁纸、特效、组件和伙伴 ZIP 还要通过入口文件和目录结构检查。资源机从特效和伙伴的入口 JSON 读取作品 ID 和名称，再随签名回调发送给论坛。下载时提供上传的原始 ZIP。
4. 校验通过后创建不可变对象：内容包为 `files/{md5}.zip`，封面为 `img/{md5}.{ext}`；上传凭证的文件键通过硬链接指向同一内容。相同 MD5 的现有对象必须同时匹配 SHA-256 和大小。回调论坛 `RESOURCE_CALLBACK_URL`，请求体包含 `file_key,object_key,size,sha256,manifest`，头 `X-Signature` 为请求体的 HMAC-SHA256。回调失败时移除该凭证的链接并返回错误；共享对象保留，避免影响并发或已使用的相同内容。
5. 同一个凭证键不能覆盖；同时上传数受 `RESOURCE_MAX_UPLOADS` 限制，超出返回 503。

## 签名约定

与 `server-forum/src/content.rs` 一致，两边各有一份实现，靠同一个测试向量防止漂移：

- 下载：`sig = hex(HMAC-SHA256(secret, "GET\n{file_key}\n{exp}"))`。
- 上传凭证：`base64url(JSON声明).hex(HMAC-SHA256(secret, "PUT\n{base64url部分}"))`，声明含 `uid, kind, size, sha256, file_key, exp`。
- 回调：对请求体字节做 HMAC-SHA256。

## 配置

见 `.env.example`。`RESOURCE_SECRET` 必填且不少于 32 位，必须与论坛的 `FORUM_RESOURCE_SECRET` 一致；否则服务拒绝启动。

## 部署要点

- Nginx 反代到 127.0.0.1:55020，`/upload` 关闭请求缓冲；生产入口和大小限制见 [`infra/resource/nginx-res.conf`](../../infra/resource/nginx-res.conf)。
- 数据目录 `RESOURCE_DATA_DIR` 需要持久化；`files/`、`img/`、`packs/`、`covers/`、`tools/` 下的已确认对象需要异地备份。官方公开文件和播放器在独立目录，也需单独备份，见部署说明。
- 带宽到 70% 时告警；下载地址有效期默认 10 分钟，限制被转发盗链的窗口。

## 运维脚本（`scripts/`）

- `backup.sh`：把 `files/`、`img/`、`packs/`、`covers/`、`tools/*.bin`、`forum-archive/`、`forum-content/` 追加式同步到备份机并维护 `SHA256SUMS` 清单；`backup.sh verify` 在备份端核对。`BACKUP_TARGET` 可以是 `user@host:/path` 或本机目录。`test-backup.sh` 测试备份、恢复和损坏检测。
- `bandwidth_alert.py`：读网卡出口流量，占 `LINK_MBPS`（默认 200）的比例连续 3 次采样超过 70% 才告警，发 `ALERT_WEBHOOK`，30 分钟冷却，回落发恢复通知。`python3 -m unittest test_bandwidth_alert`。
- `mirror_modelscope.py`：把官方内容包镜像到魔搭并登记到论坛（`POST /api/admin/packs/{id}/mirrors`，按版本锁定）：从资源机下载并核对 sha256，用 `modelscope upload` 上传，再从魔搭下载回来核对，通过才登记。`--dry-run` 只做下载与校验。魔搭上传这一步需要 `modelscope` 命令和账号令牌，云端环境没有验证过。

## 验证

```bash
cd services/server-resource
cargo test
cd scripts && ./test-backup.sh && python3 -m unittest discover -p 'test_*.py'
```

测试覆盖：凭证伪造、大小与哈希不符、非 zip、缺清单、路径穿越、封面类型、论坛拒绝或不可达时撤销文件、签名下载的过期与错键、Range。

官方工具镜像的接入、HMAC协议、限额和反向代理超时见 [tool-mirrors.md](docs/tool-mirrors.md)。镜像请求在资源机处理，论坛不转发大文件；每文件128MiB、目录1GiB/128对象，OS锁串行写入。资源机先升级，论坛再开启镜像入口。
