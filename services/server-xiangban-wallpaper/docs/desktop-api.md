# 小伴桌面与网站对接

正式源站：`https://wallpaper.xiangban.xyz`。客户端“关于小伴 → 壁纸网站”可以修改网站源站；本机联调使用 `http://localhost:8080`。介绍页首页入口 `/`（旧 `/desktop` 重定向），图片入口 `/images`、视频入口 `/videos`，后台入口 `/admin/desktop`。后台配置复用现有 PostgreSQL `app_settings`，不新增历史数据迁移。

## 公开接口

| GET 接口 | 用途 |
| --- | --- |
| `/api/v1/desktop/product` | 产品介绍、各平台当前下载版本和网站公告 |
| `/api/v1/desktop/updates?currentVersion=0.1.0&target=windows-x86_64` | 当前平台的版本检测；没有更高版本返回 204，有更新返回 200 |
| `/api/v1/desktop/announcements` | 客户端有效公告 |
| `/api/v1/wallpapers/search?q=&category=&media_type=image&page=1&per_page=24` | 与网页相同的公开图库检索、分类和分页；`media_type=video` 获取动态壁纸 |
| `/api/v1/categories` | 与网页相同的分类 |
| `/api/v1/desktop/wallpapers/{id}` | 单项交付清单，供客户端下载、校验、缓存与应用 |

桌面专用接口使用 `Cache-Control: no-store`。未知平台或非法版本号返回 400，未上架 / 已下架的单项内容返回 404；数据库不可用返回 503，不假装已是最新版本。

### 版本和下载

平台值：`windows-x86_64`、`darwin-aarch64`、`darwin-x86_64`。管理员只填写实际提供安装包的平台。草稿、未来发布日期和稳定客户端的预发布版本不参与选择。按 SemVer 为每个平台分别选择最新发布，支持多个平台不同发布进度。

200 响应：

```json
{
  "schemaVersion": 1,
  "version": "1.2.0",
  "notes": "体验优化。",
  "publishedAt": "2026-10-09T00:00:00Z",
  "platforms": {
    "windows-x86_64": { "url": "https://wallpaper.xiangban.xyz/downloads/XiaoBan.zip" }
  }
}
```

下载 URL 必填；在后台 App 上架上传的安装包自动记录真实 `sha256` 和 `bytes`，仍支持外部 HTTPS 下载链接。外部链接的校验信息可选。客户端只在浏览器打开下载链接，未实现自动下载、覆盖程序或安装；因此不为仅用于跳转的链接伪造校验值。旧订阅更新清单中的 `mime`、`sha256`、`bytes` 仍可解析。

### 公告

```json
{
  "schemaVersion": 1,
  "items": [{ "id": "welcome", "title": "欢迎", "body": "欢迎使用小伴桌面。", "revision": "64位小写SHA256" }]
}
```

公告由后台配置纯文本标题、正文、发布状态、网站 / 桌面展示位置以及可选开始和结束时间。`revision` 由服务端根据配置内容计算；客户端持久化 `(id, revision)` 已读状态，相同修订不重复提醒。内容修改后重新提示；下架 / 过期公告不再出现在成功刷新后的列表中。`update` 为客户端保留的消息 ID。

客户端在启动时和运行期间每 30 分钟检查版本与公告，可以关闭自动检查。消息通过原生层发给控制台，并更新托盘提示；关闭控制台仍继续检查，重开后能看到消息。支持手动检查。网络失败保留已有壁纸和上次消息，手动同步显示错误，不将连接失败当成“没有更新”。

### 壁纸交付

检索列表直接使用网站现有 snake_case 字段和 `has_more` 分页语义。客户端执行保存或应用时根据列表中的 `id` 请求单项清单，其格式遵循[客户端资源协议](../../../client/xiaoban-desktop-wallpaper/docs/delivery-protocol.md)。

视频在管理员采集 / 自定义创建、重新上架与后台采集时先流式保存到服务器 `uploads/`；没有图片封面时用 FFmpeg 生成真实首帧 JPEG，再将本地视频与封面链接入库。处理失败时不将视频发布为可用内容。已有记录在首次下载请求时按需补齐，不执行全库迁移。图片在首次准备时生成不可变副本。素材最多 256 MiB；计算真实 SHA-256 与字节数后保存不可变副本，后续复用。外部源限制公开网络地址，逐跳校验并固定 DNS 解析结果，不允许访问内部网络。清单包含绝对资源 URL，网站自有内容寻址资源由客户端按已配置的网站源站解析（保留路径、SHA-256 与大小），避免本机地址配置后下载仍指向默认正式域名；外部 CDN 资源地址不改变。

图片支持 PNG/JPEG/WebP/AVIF/GIF/BMP，视频支持 MP4/WebM；跨平台推荐 JPEG/PNG/WebP 与 H.264 MP4。解码能力由客户端系统决定，失败会报告错误并保留正在使用的壁纸。清单中不下发脚本或本地路径，客户端下载完成并通过大小、SHA-256 校验后才应用；随后可离线播放。

`uploads/` 包含用户上传媒体和桌面不可变副本，需要作为持久目录备份，不加入 Git。源文件发生变化应上传为新 URL；对相同 URL 的外部媒体按首次选中时的快照交付。安装包可以在后台 `/admin/apps` 上传到网站服务器，也可以使用自己的对象存储 / 下载服务器链接。站内安装包保存在 `app-packages/`，该目录不作为静态目录公开，需持久化并备份；通过专用下载接口按版本发布状态开放。

## 视频首帧运行依赖

视频入库需要 `ffmpeg` 可执行程序。部署环境安装 FFmpeg，并通过 PATH 或 `FFMPEG_BIN` 指定；CI 显式安装 FFmpeg。本机可在网站目录执行 `npm install --prefix target/media-tools --no-save ffmpeg-static`，服务优先使用这个被 Git 忽略的开发工具目录。视频首帧处理有 30 秒超时与临时文件清理。`uploads/` 必须持久化。

## 管理接口

`GET /api/v1/admin/desktop` 读取配置，`PUT /api/v1/admin/desktop` 保存配置，沿用管理员 Session Cookie 与同源检查。完整配置包含 `siteUrl`、`introduction`、`releases`、`announcements`。版本号、平台、URL、公告 ID、时间区间和大小限制由后端校验。单次配置最多 100 个版本 / 100 条公告、总量不超过 900 KiB。

## 验证

在网站目录先执行 `npm --prefix web ci`、`npm --prefix web run build`、`cargo test --locked`。设置独立 `TEST_DATABASE_URL` 后，数据库测试为每例创建并清理临时 schema，覆盖后台鉴权、配置持久化、版本选择、公告过滤、上下架与资源实际校验。

原生端到端测试需要临时网站数据库 / schema，不应指向正式数据库。启动该网站后设置 `XIAOBAN_WEBSITE_TEST_URL=http://127.0.0.1:8087`，在仓库根运行：

```powershell
node client/xiaoban-desktop-wallpaper/scripts/verify-website.mjs client/xiaoban-desktop-wallpaper/src-tauri/target/release/xiaoban-desktop-wallpaper.exe
```

脚本使用测试网站默认管理员，写入测试版本 / 公告，上传测试图片 / 视频，通过隔离桌面实例实际检索、下载、渲染和检查消息。客户端使用独立数据目录和测试自启动项，不读取或修改正式桌面设置；报告位于客户端 `artifacts/website-smoke-*.json`。测试网站数据随后整体清理。

客户端第一层为“在线壁纸 / 我的壁纸”，第二层为图片 / 视频（本地也支持动画）。在线保存会实际下载、校验并留在当前页面，应用是独立按钮。官网首页 `/` 使用同一猫脸 SVG 与实际 MP4 演示，可切换悬浮 / 融入壁纸并播放 / 暂停。

## App 上架管理

后台 `/admin/apps` 管理小伴桌面 Windows / macOS 版本，官网和公告设置仍在 `/admin/desktop`。

1. 新建版本，填写 SemVer、更新说明和发布时间。
2. 每个平台上传一个安装包或填写外部下载链接。Windows 支持 EXE/MSI/ZIP，macOS 支持 DMG/PKG/ZIP；单包最多 512 MiB，上传有进度与取消按钮。
3. 保存草稿后安装包不开放下载；保存并上架后官网与客户端版本接口同步生效。未来发布时间为定时上架，下架关闭站内下载。
4. 已有版本可编辑、下架或删除版本记录。删除版本不直接删除安装包文件，避免影响其他引用或正在进行的下载。

管理接口：`POST /api/v1/admin/desktop/packages`（Multipart，`target` 在 `file` 之前）、`GET /api/v1/admin/desktop/releases`、`PUT /api/v1/admin/desktop/releases/{version}`、`DELETE /api/v1/admin/desktop/releases/{version}`。复用管理员会话与同源校验。上传流式计算字节数与 SHA-256，文件名与平台格式由后端验证，使用服务器生成的 ID 保存，不执行安装包。

公开下载：`GET /api/v1/desktop/downloads/{id}`，只允许当前已发布且到达发布时间的版本引用的文件，返回附件下载与 UTF-8 文件名，支持 HTTP Range。草稿、下架或不存在的安装包返回 404。站内下载 URL 按实际请求的网站域名 / 本机端口生成，外部链接保持原地址。

`PATCH /api/v1/admin/desktop` 只保存网站地址、介绍和公告；版本管理按单版本更新。配置更新使用数据库事务与行锁，避免不同页面编辑时覆盖其他配置。旧的完整配置 PUT 接口保留兼容。未新增数据迁移，包元数据复用 `app_settings`。
