# 小伴桌面与网站对接

正式源站：`https://wallpaper.xiangban.xyz`。客户端“关于小伴 → 壁纸网站”可以修改网站源站；本机联调使用 `http://localhost:8080`。前台入口 `/desktop`，后台入口 `/admin/desktop`。后台配置复用现有 PostgreSQL `app_settings`，不新增历史数据迁移。

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

下载 URL 必填；真实 `sha256` 和 `bytes` 可选。客户端只在浏览器打开下载链接，未实现自动下载、覆盖程序或安装；因此不为仅用于跳转的链接伪造校验值。旧订阅更新清单中的 `mime`、`sha256`、`bytes` 仍可解析。

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

检索列表直接使用网站现有 snake_case 字段和 `has_more` 分页语义。客户端选中列表中的 `id` 后请求单项清单，其格式遵循[客户端资源协议](../../../client/xiaoban-desktop-wallpaper/docs/delivery-protocol.md)。

服务端首次准备资源时，流式读取本地 `/uploads` 或公开 HTTPS 源，最多 256 MiB；计算真实 SHA-256 与字节数后保存不可变副本，后续复用。外部源限制公开网络地址，逐跳校验并固定 DNS 解析结果，不允许访问内部网络。清单包含绝对资源 URL，源站来自后台“网站地址”。部署后须确保这里与实际源站一致。

图片支持 PNG/JPEG/WebP/AVIF/GIF/BMP，视频支持 MP4/WebM；跨平台推荐 JPEG/PNG/WebP 与 H.264 MP4。解码能力由客户端系统决定，失败会报告错误并保留正在使用的壁纸。清单中不下发脚本或本地路径，客户端下载完成并通过大小、SHA-256 校验后才应用；随后可离线播放。

`uploads/` 包含用户上传媒体和桌面不可变副本，需要作为持久目录备份，不加入 Git。源文件发生变化应上传为新 URL；对相同 URL 的外部媒体按首次选中时的快照交付。安装包本身可存放在自己的对象存储或下载服务器，后台配置其链接。

## 管理接口

`GET /api/v1/admin/desktop` 读取配置，`PUT /api/v1/admin/desktop` 保存配置，沿用管理员 Session Cookie 与同源检查。完整配置包含 `siteUrl`、`introduction`、`releases`、`announcements`。版本号、平台、URL、公告 ID、时间区间和大小限制由后端校验。单次配置最多 100 个版本 / 100 条公告、总量不超过 900 KiB。

## 验证

在网站目录先执行 `npm --prefix web ci`、`npm --prefix web run build`、`cargo test --locked`。设置独立 `TEST_DATABASE_URL` 后，数据库测试为每例创建并清理临时 schema，覆盖后台鉴权、配置持久化、版本选择、公告过滤、上下架与资源实际校验。

原生端到端测试需要临时网站数据库 / schema，不应指向正式数据库。启动该网站后设置 `XIAOBAN_WEBSITE_TEST_URL=http://127.0.0.1:8087`，在仓库根运行：

```powershell
node client/xiaoban-desktop-wallpaper/scripts/verify-website.mjs client/xiaoban-desktop-wallpaper/src-tauri/target/release/xiaoban-desktop-wallpaper.exe
```

脚本使用测试网站默认管理员，写入测试版本 / 公告，上传测试图片 / 视频，通过隔离桌面实例实际检索、下载、渲染和检查消息。客户端使用独立数据目录和测试自启动项，不读取或修改正式桌面设置；报告位于客户端 `artifacts/website-smoke-*.json`。测试网站数据随后整体清理。
