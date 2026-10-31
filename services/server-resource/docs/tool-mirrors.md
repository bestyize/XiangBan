# 官方工具安装包镜像（E2）

镜像是传输元信息，不是发布渠道。管理员从论坛的工具草稿选择一个真实 GitHub Release 安装包，资源机下载并校验、写入后再次打开文件回读 SHA-256，论坛验证签名回执并重新核对草稿版本后才登记。整个流程不修改草稿、不发布内容；只有同一 asset_id / SHA-256 / 大小已出现在当前线上工具中，公开读接口才添加镜像下载地址。官方下载始终保留。

## 接入入口

资源机：

1. `src/main.rs` 注册 `mod tool_mirrors;`。
2. `build_router` 在挂载 state 前 `.merge(tool_mirrors::routes())`。新增 `POST /internal/tool-mirrors`、`GET/HEAD /tool-mirrors/{sha}/{filename}`。不改旧 zip 上传与内容包回调。
3. 保持 `RESOURCE_SECRET` 与论坛 `FORUM_RESOURCE_SECRET` 相同，至少 32 字节。使用原 `RESOURCE_DATA_DIR`，数据写到 `tools/`，请纳入备份。

论坛：

1. `src/main.rs` 注册 `mod tool_mirrors;`；迁移时调用 `tool_mirrors::init(&pool)`（创建独立 `tool_mirror_registry`）。
2. 在 `/api/admin` 路由合并 `tool_mirrors::admin_routes()`；在 `/api` 路由合并 `tool_mirrors::public_routes()`。
3. 对已发布工具的列表与详情，在返回前调用 `tool_mirrors::decorate_tool(&pool, &config.public_url, &key, &mut data).await?`。该函数对非 GitHub 工具直接跳过，并独立读取当前线上内容、核对引用，避免未审核候选被误装饰。
4. 管理台独立组件为 `ToolMirrorsPanel.vue`，建议菜单 key `tool-mirrors`；API 在 `api/toolMirrors.ts`。不需要改动采集面板。

建议先升级资源机，再升级论坛。双方 Nginx 对镜像 POST 路径配置 `proxy_read_timeout 210s`，论坛等待资源机期间不搬运二进制字节。资源机内部 POST 的请求体限制为 8 KiB，无需放宽大文件上传限制。业务内部最长三分钟，论坛请求最长 195 秒。

## 协议

`POST /internal/tool-mirrors` 请求体：

```json
{"nonce":"UUID","repository":"owner/repo","version":"v1","filename":"repo-windows-x64.exe","repo_id":1,"asset_id":2,"source_url":"https://github.com/owner/repo/releases/download/v1/repo-windows-x64.exe","size":4,"sha256":"64位小写十六进制","exp":0}
```

- `X-Tool-Signature = hex(HMAC-SHA256(secret, "POST\n/internal/tool-mirrors\n" + 原始请求字节))`。
- `exp` 在当前时间至未来 300 秒内，论坛默认签发 240 秒。
- 资源机按 repository / version / filename 重新构造官方 GitHub URL，比对完整地址，不接受任意源 URL、URL 凭据、查询串或路径穿越。请求不含 GitHub Token。
- 下载使用新的无鉴权客户端，禁自动重定向；逐跳只允许 `github.com`、`release-assets.githubusercontent.com`、`objects.githubusercontent.com` 的 HTTPS 默认端口，最多三跳，禁止内网/IP URL。数据不解压、不执行。

成功响应包括 `nonce,repo_id,asset_id,filename,sha256,size,file_key,readback_verified,verified_at`。

- `X-Tool-Receipt-Signature = hex(HMAC-SHA256(secret, "TOOL-MIRROR-RECEIPT\n" + 原始响应字节))`。
- 论坛只向配置的资源地址请求，禁止跳转，限制回执 8 KiB，核对签名和全部绑定字段。
- 成功文件为 `tools/{sha256}.bin`。写临时文件、流校验、fsync、原子硬链接、重新开文件回读校验，全部成功才回执。同哈希对象重用前仍回读。

公开 GET 先核对当前已发布工具与登记记录，资源可用才签发十分钟以内的资源下载 URL；资源不可用则跳转回当前已发布且校验过的官方 GitHub URL。资源下载复用 `GET\n{file_key}\n{exp}` 签名约定，逻辑键为 `tool-mirrors/{sha}/{filename}`，支持 Range，固定附件和 `nosniff`。

## 配额与失败

- 每文件 128 MiB；工具镜像目录总计 1 GiB / 128 个对象。
- 写入前获取 OS 文件排他锁，同机多进程也串行；进程退出自动释放锁。
- 满额停止新增，不自动删除已验证对象。重试清理中断留下的专属 `.part` 文件；哈希不符的临时文件不进入正式对象。
- 上游失败、超时、哈希/大小错误、磁盘/配额失败、回读不符、回执签名错误或草稿版本冲突，均不新增/更新登记。失败不清空已有有效登记。
- 资源机完成而论坛版本冲突时，磁盘可能保留一个有界、不可公开引用的内容寻址对象；再次执行可复用。后续配额维护由运维结合引用处理。

## 验证

```sh
cargo test --locked --manifest-path services/server-resource/Cargo.toml tool_mirrors
TEST_DATABASE_URL='<隔离测试库>' cargo test --locked --manifest-path services/server-forum/Cargo.toml tool_mirrors
npm --prefix services/server-forum/web run build
```

论坛数据库测试创建独立 schema，结束后删除；没有 `TEST_DATABASE_URL` 时明确跳过。测试仅使用本地 mock、4 字节样本和临时目录，不下载真实大安装包。覆盖签名域与固定向量、nonce/资产/哈希/大小绑定、目录与跳转限制、并发配额锁、写入回读、Range、失败不登记、草稿不公开、发布引用校验、下架及官方下载回退。
