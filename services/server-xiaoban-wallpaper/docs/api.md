# HTTP 与 JSON 协议

本服务与 [桌面壁纸 v1 协议](../../../client/xiaoban-desktop-wallpaper/docs/delivery-protocol.md) 兼容。所有接口同源，错误返回 `{ "message": "原因" }`。API 不接受图片、视频 multipart 上传，不提供媒体代理。

## 公开接口

| 接口 | 返回 |
| --- | --- |
| GET /api/health | 服务存活状态 |
| GET /api/catalog | schemaVersion / revision / items；仅已发布壁纸，最多 100 项、1 MiB |
| GET /api/site | 产品名称、公开目录和已发布应用版本；无版本时 release 为 null |
| GET /api/updates?currentVersion=0.1.0&target=windows-x86_64 | 更新清单；无更新时 HTTP 204 |

目录 `revision` 为公开内容的 SHA-256；改草稿不影响公开目录。图片和视频 URL 保持提供方原地址，客户端下载外部媒体，不经过本服务。

## 简化导入格式

以下为可保存的草稿。示例域名仅用于说明，替换成自己的 HTTPS 下载地址。

```json
[
  {
    "id": "afternoon-sea",
    "title": "午后的海",
    "kind": "video",
    "downloadUrl": "https://YOUR-CDN.example/afternoon.mp4",
    "version": "1.0.0",
    "description": "安静的海面，陪你慢慢工作。",
    "author": "小伴"
  },
  {
    "title": "日落的片刻",
    "kind": "image",
    "downloadUrl": "https://YOUR-CDN.example/sunset.png",
    "coverUrl": "https://YOUR-CDN.example/sunset-cover.jpg"
  }
]
```

接受单条对象、数组和桌面 v1 的 `{schemaVersion:1,revision,items:[...]}`。简化记录支持 `url` 作为 `downloadUrl` 别名、`type` 作为 `kind` 别名，以及 `mime`、`sha256`、`bytes`、`coverMime`、`coverSha256`、`coverBytes`。缺少 ID 时生成并保存 `wall-<UUID>`，缺少版本时为 `1.0.0`。需要重复导入更新已有内容时请提供相同 ID；单独的 `.`、`..` 不能用作管理接口 ID。

导入默认草稿；正文只有地址也可保存。发布前需补齐入口、封面和附属资源的真实 SHA-256 与字节数。可手填，或在后台点击“探测”：有大小与时间上限的流式 GET 请求计算完整原始字节哈希，不写入媒体文件或数据库 BLOB。Canvas 脚本不执行，只记录资源与 SDK 参数。

完整格式直接使用桌面 Wallpaper 定义：`id/version/title/description/author/kind/resource/cover/assets/controls`。`resource` 是 `{url,sha256,bytes,mime}`。本地客户端保留的 `companion`、`builtin-`、`local-` ID 和 `file:`、`local://`、`builtin://` URL 均不能发布。

## 管理员接口

| 接口 | 请求/行为 |
| --- | --- |
| POST /api/admin/login | username / password，设置 HttpOnly Cookie 并返回 csrfToken |
| GET /api/admin/me | 当前账号、csrfToken、是否仍用初始密码 |
| POST /api/admin/logout | 撤销当前会话 |
| PUT /api/admin/account | username / currentPassword / newPassword；新密码空串则仅改用户名，撤销全部旧会话 |
| GET /api/admin/wallpapers | 全部草稿、发布记录及 revision |
| POST /api/admin/wallpapers | definition / published / sortOrder；创建壁纸 |
| PUT /api/admin/wallpapers/:id | 上述字段 + revision；更新时检查版本冲突 |
| DELETE /api/admin/wallpapers/:id | revision；只删除元数据 |
| POST /api/admin/wallpapers/import | data / overwrite；整批事务，覆盖同 ID 时转为草稿 |
| POST /api/admin/wallpapers/:id/publish | revision / published；上下架 |
| POST /api/admin/wallpapers/:id/verify | revision；流式探测并更新校验值 |
| GET /api/admin/release | 应用发布草稿、公开快照与 revision |
| PUT /api/admin/release | definition / revision；保存更新清单草稿 |
| POST /api/admin/release/publish | revision；发布到下载页面与更新接口 |

所有管理修改（除登录）需要 Cookie 会话和 `X-CSRF-Token`。浏览器 Origin 与 `PUBLIC_ORIGIN` 必须匹配。账号修改会撤销所有旧会话；数据库已有账号时，服务重启不再次创建或重置初始管理员。登录按实际连接 IP 限流，不信任未配置的 X-Forwarded-For。

创建/更新发布记录使用服务端协议验证。发布数量与清单体积在 PostgreSQL 事务内检查，通过服务级 advisory lock 避免并发越限；冲突或不合法记录整批回滚。`revision` 过期返回 409，不覆盖其他编辑。已发布资源的内容校验值改变时需提高壁纸版本；在线资源与记录不符时探测失败，先下架调整后再探测。

探测仅允许公网目标，解析全部 DNS 地址并逐个校验，每次重定向重新校验并固定连接到已验证的 IP，禁用环境代理。最多 4 次重定向，连接超时 10 秒、单张壁纸整体探测最多 120 秒；下载地址应直接返回 HTTP 200 原文件，不是网页或压缩传输结果。入口和附属资源合计最多 256 MiB，封面与每个 JS 最多 2 MiB。debug 测试可显式启用环回；release 构建不能开启该例外。

应用发布清单沿用桌面更新 v1：至少一个平台安装包，平台键为 `windows-x86_64`、`darwin-aarch64`、`darwin-x86_64`，包体不超过 60 MiB，校验字段必须齐全。不执行无签名自动更新，不存储安装包。没有已发布安装包时产品页诚实显示“安装包准备中”。
