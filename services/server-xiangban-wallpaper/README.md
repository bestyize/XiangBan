# WallpaperCollect - 精品壁纸采集、管理与发布系统 🖼️

[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg?logo=rust)](https://www.rust-lang.org/)
[![Axum](https://img.shields.io/badge/Framework-Axum%200.8-blue.svg)](https://github.com/tokio-rs/axum)
[![Vue 3](https://img.shields.io/badge/Frontend-Vue%203%20%7C%20Vite%20%7C%20TS-42b883.svg?logo=vuedotjs)](https://vuejs.org/)
[![PostgreSQL](https://img.shields.io/badge/Database-PostgreSQL%20%7C%20In--Memory-336791.svg?logo=postgresql)](https://www.postgresql.org/)
[![License](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-green.svg)](LICENSE)

> 基于 **Rust (Axum)** + **Vue 3 (Vite + TypeScript)** 构建的高颜值、高性能壁纸与动态视频系统。  
> 采用 **「对内选品采集 / 自主上传 ➔ 人工审核入库 ➔ 对外权威发布」** 的精品自建库架构，彻底解决第三方图库质量与合规性无法把控的痛点。

## 小伴桌面官网与客户端服务

正式域名 `wallpaper.xiangban.xyz`。网站首页 `/` 展示小伴桌面介绍、分平台下载与公告，左侧猫脸品牌“小伴桌面”链接到首页；图片 `/images`、视频 `/videos` 保留独立导航。旧 `/desktop` 地址重定向到首页，旧图片分类 `/c/:id` 作为 `/images/c/:id` 的别名保留。后台 `/admin/apps` 支持 Windows/macOS/Android 安装包上传（EXE/MSI/DMG/PKG/APK/ZIP）、版本上架 / 下架和更新说明，官网与升级检测同步生效；`/admin/desktop` 配置官网介绍、网站源站和公告有效时间。站内包自动计算大小和 SHA-256，单包最多 512 MiB，草稿不公开，保存在独立的持久目录 `app-packages/`。

选中“小伴桌面”栏目时网站平滑切换为浅色，图片 / 视频栏目保持深色。官网使用独立维护的原有猫脸角色引擎，支持视线跟随、点击表情和桌面内拖动；Windows 风格演示包含图标、开始菜单、文件夹与任务栏，桌面显示区固定为 16:9 并响应缩放，背景为真实循环视频。视频入库先保存到服务器，缺少封面时截取首帧；运行环境需安装 FFmpeg（或配置 `FFMPEG_BIN`），本机开发工具安装见[桌面 API](docs/desktop-api.md)。

桌面客户端 `client/xiaoban-desktop-wallpaper` 直接使用本网站的公开图库、分类、更新和公告接口；资源按真实大小与 SHA-256 校验后缓存，支持离线使用。启动及每 30 分钟后台检查消息，可以手动刷新或关闭自动检查。接口、运行时目录与端到端验证详见[桌面 API](docs/desktop-api.md)。配置复用现有 `app_settings`，未新增历史数据迁移。

**来源链接优先。** 采集始终保留供应商的原始链接（`preview_url`、`regular_url`、`downloads`、`video`），本站保存的文件只是后备，其路径、SHA-256、大小记录在 `wallpapers.local_copy`（可空 JSONB，启动时 `ADD COLUMN IF NOT EXISTS`）。每个供应商按图片 / 视频各有策略 `provider.<id>.hotlink.image|video`（`source` 或 `copy`，后台“供应商密钥”页可改）：默认 Pexels / Unsplash / okslop / Bing 图片与视频都是 `source`，Pixabay 图片 `copy`、视频 `source`；自定义上传与 demo 始终 `copy`。Pixabay 图片默认 `copy` 是因为其 API 文档写明“permanent hotlinking of images (using Pixabay URLs in your app) is not allowed ... please download them to your server first. Videos may be embedded directly in your applications”，且 `webformatURL` 只有 24 小时有效。策略为 `source` 的条目采集时不下载素材；公开检索 / 详情 JSON 在有本站副本且与主链接不同时增加可选 `fallback`；`GET /api/v1/desktop/wallpapers/{id}?prefer=source` 返回 `source` 链接且不下载，客户端失败再回退到 `resource` 或不带 `prefer` 的接口。更早采集的视频已被改写成本地路径、丢失了原始链接，保持 `copy`。详见[桌面 API](docs/desktop-api.md)。

验证命令（本目录）：`npm --prefix web ci`、`npm --prefix web run build`、`cargo test --locked`、`cargo build --release --locked`。真实数据库集成测试设置 `TEST_DATABASE_URL`，每例使用独立临时 schema。Node.js 使用 24，Rust 使用 stable。本机运行见 [LOCAL_SETUP.md](LOCAL_SETUP.md)。

---

## 🏛️ 系统业务架构

```
 ┌─────────────────────────────────────────────────────────────┐
 │                      数据源与生产端                          │
 ├──────────────────────────────┬──────────────────────────────┤
 │  1. 外部供应商 (Unsplash等)   │   2. 站长 / 自定义创作者     │
 └──────────────┬───────────────┴──────────────┬───────────────┘
                │                              │
                ▼                              ▼
     【内部采集工作台】 (对内)       【后台管理 - 手动上架】 (对内)
     • 路由: /admin/collector        • 路由: /admin/upload
     • 聚合检索供应商内容            • 上传本地 4K 图片 / 循环短视频
     • 卡片高亮「已入库」状态        • 自动探测画幅分辨率与比例
     • 一键「采集入库 / 立即上架」   • 存入本地 uploads/ 静态托管
                │                              │
                └──────────────┬───────────────┘
                               ▼
            ┌──────────────────────────────────────┐
            │       PostgreSQL 权威数据库          │
            │  • status: published / pending / ... │
            │  • media_type: image / video         │
            │  • source_type: collected / custom   │
            └──────────────────┬───────────────────┘
                               │ (只对外提供 status = 'published')
                               ▼
                   【对外展示前台】 (公开访问)
                   • 路由: / (首页、分类、搜索、动态视频)
                   • 纯本地数据库分页检索，毫秒级响应
                   • 瀑布流缩略图秒开、全屏展厅级大图沉浸预览
                   • 100% 内容合规可控，支持全画幅与视频播放下载
```

---

## ✨ 核心特性

### 1. 公开展现门户 (Public Frontend)
- **纯本地权威数据源**：所有展示、搜索、分类浏览、详情查看均只读取数据库中 `status = 'published'` 的内容，内容 100% 可控、零合规风险、毫秒级响应。
- **瀑布流缩略图极速加载**：卡片智能优先加载轻量缩略图（350px~640px，几十 KB），相较原图节省 **85%~90%** 网络流量，滚动流畅不卡顿。
- **真实动态视频封面**：动态视频卡片采用视频真实第 0.001 秒首帧作为封面（杜绝无关作者头像），鼠标悬停即刻启动平滑微动效循环预览。
- **画廊级无遮挡大图预览**：
  - 严格保持纵横比自适应展示，四周设有充足的安全留白（Safe Padding）；
  - **彻底解决长图底部裁切**：即使是 9:16、9:19.5 等超长手机壁纸，也能完整自适应收纳于视口内，底边留有 48px 安全空间，四个圆角与底部画面 100% 完整可见；
  - 手机端默认避让底栏，轻触图片一键进入沉浸全屏模式。
- **极简分类切换**：点击顶部分类胶囊栏时平滑无缝加载该分类瀑布流，无突兀的大标题向下挤占页面。

### 2. 内部采集工作台 (Collector Workbench)
- **内部安全受控**：工作台挂载于 `/admin/collector`，需要管理员登录鉴权，不对外部访客公开。
- **保留高颜值交互**：复用流畅优雅的瀑布流卡片、多画幅过滤与全屏沉浸预览交互。
- **智能一键采集**：
  - 实时检测并批量标记当前页已入库壁纸（高亮显示绿色 **「✓ 已入库」** 徽章）；
  - 卡片右上角快捷点击 **「📥 采集」**，轻量弹窗可指定所属分类、选择是“立即上架发布”还是“存入待审核库”。

### 3. 后台管理与自主上架 (Admin Management)
- **图库内容中台 (`/admin/wallpapers`)**：
  - 全览库内所有壁纸与视频（支持按已上架、待审核、已下架、分类、媒体类型筛选）；
  - 支持快捷一键切换「上架 / 下架」状态、修改标题分类与标签、永久删除。
- **自主资源上架 (`/admin/upload`)**：
  - 支持拖拽上传本地原创或精选高清图片（JPG、PNG、WEBP）与动态视频（MP4、WebM）；
  - 自动通过浏览器底层媒体 API 探测文件真实宽度、高度与横竖屏画幅；
  - 支持为视频上传独立海报图，后端自动保存至 `uploads/` 目录并由静态文件服务器分发。
- **供应商多密钥轮换与代理池 (`/admin/keys` & `/admin/proxies`)**：
  - Unsplash / Pixabay / Pexels / okslop / Bing 多 API Key 轮询管理，自动熔断冷却；
  - 支持 HTTP / HTTPS / SOCKS5 出网代理池配置与一键测速。

---

## 📁 目录结构

```
WallpaperCollect/
├── Cargo.toml                # Rust 依赖（Axum 0.8, SQLx, Tokio, Tower 等）
├── .env.example              # 环境变量配置模板
├── docker-compose.yml        # PostgreSQL 一键启动配置
├── migrations/               # PostgreSQL 数据库建表与迁移脚本
│   ├── 20261008000000_init.sql    # wallpapers, favorites 表结构
│   └── 20261009000000_admin.sql   # admin_account, keys, proxies 表
├── uploads/                  # 本地自主上传图片与视频静态存储目录
├── web/                      # Vue 3 前端工程
│   ├── src/
│   │   ├── admin/            # 后台管理 API 与类型定义
│   │   ├── components/       # 卡片、栅格、大图预览 Viewer、样机预览组件
│   │   ├── views/
│   │   │   ├── admin/        # 采集工作台、图库管理、自主上传、密钥代理后台
│   │   │   └── BrowseView.vue# 对外展示前台（首页、分类、搜索、视频）
│   │   ├── router.ts         # 路由配置（前台公开路由与后台受保护路由）
│   │   └── utils.ts          # 缩略图选取、画幅比例与色彩算法
│   └── package.json
└── src/                      # Rust 后端源码
    ├── main.rs               # 服务入口与静态托管装配
    ├── config.rs             # 配置解析
    ├── db/                   # PostgreSQL 数据访问层 Repository
    ├── models/               # 数据模型（WallpaperItem, SearchQuery 等）
    ├── providers/            # 外部数据源适配层（Unsplash, Pixabay, Pexels, okslop, Bing, Demo）
    ├── admin/                # 管理后台鉴权与接口（图库、上传、采集、代理）
    ├── api/                  # 前台公开 API 路由与处理器
    ├── services/             # 供应商并发聚合器与后台同步器
    └── web/                  # 前端产物嵌入服务 (rust-embed)
```

---

## 🚀 快速启动

### 1. 克隆项目
```bash
git clone https://github.com/bestyize/WallpaperCollect.git
cd WallpaperCollect
```

### 2. 准备数据库与配置文件
启动 PostgreSQL（推荐通过 Docker 容器）：
```bash
docker compose up -d
```
复制环境变量配置：
```bash
cp .env.example .env
```
确认 `.env` 中的数据库连接串：
```env
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

# PostgreSQL 数据库（必须配置以开启图库管理和采集入库功能）
DATABASE_URL=postgres://postgres:password123@localhost:5432/wallpaper_collect

# 可选：外部供应商 API 密钥（首次启动后可在管理后台动态配置）
# UNSPLASH_ACCESS_KEY=your_key
# PIXABAY_API_KEY=your_key
# PEXELS_API_KEY=your_key
# OKSLOP_API_KEY=your_key
```

### okslop（AI 生成图片）

配置 `OKSLOP_API_KEY`（或在后台“供应商密钥”页添加）后，okslop 加入检索和采集；没有密钥时不参与。只有图片，没有视频。服务端用 `reqwest` 直接发 HTTP 请求，不依赖它的 SDK。

**已核对（2026-10-09，依据官方文档 okslop.com/developers/docs 与对线上接口的少量实测）**
- 基地址 `https://api.okslop.com/api/v1`（`https://okslop.com/api/v1` 返回同样内容）；认证 `Authorization: Bearer <key>`；响应格式与 Unsplash 一致。
- 搜索 `GET /search/photos?query=&page=&per_page=&orientation=`，列表 `GET /photos?page=&per_page=`，详情 `GET /photos/{id}`。
- **每页最多 10 条**（免费层；`per_page=24` 会被截成 10）。我们每页给 24 条，所以一页由连续的 3 个上游页拼成（`page_plan`），逐个请求、不并发，库见底时提前停止。
- **`orientation` 取 `landscape` / `portrait` / `square`**；传 `squarish` 会让接口 500，所以内部的 `squarish` 要发成 `square`。
- 搜索结果最多 200 条（`total_pages` 封顶）；列表 `/photos` 约 3.7 万张，可翻 3700 多页。
- 限制：免费层每分钟 30 次、每天 1000 次；Pro 100/10000；响应头带 `X-Ratelimit-Limit/Remaining/Reset`；超限返回 429（已有的密钥冷却逻辑会处理）。
- 图片：`urls.raw`/`full` 在 `okslop.com/img/<id>/original|full`，`regular`/`small`/`thumb` 在 `source.okslop.com`，全是 WebP，无需签名即可直接取。抽样 20 张长边都是 1024 像素以内（原图也不更大），**不适合直接当高清手机壁纸**。
- 文档说下载走 `links.download_location` 换 10 分钟有效的签名链接，无密钥返回 401；本实现不调用它（直链 `urls.*` 已可取），如 okslop 要求统计下载需另行接入。
- `is_design` 为 true 的是版式设计图（海报、徽标），不当壁纸，采集时过滤。

**条款（okslop.com/license 与 /developers/terms，2026-03-16 版）里与本站有关的几条**
- 可商用、免署名；不得训练 AI 模型；不得“不加实质修改地转卖”；不得用来复制或搭建竞争服务。
- API 条款：访问需注册并持有密钥；**密钥不得放进客户端代码**（本站只在服务端使用）；**不得系统性缓存、存储或下载图库的全部或相当一部分**；不得抓取；应用须在开发者门户登记，可能被审核；不得搭建“图库/素材市场类的竞争服务”。
- 对本站的含义：okslop 默认策略是 `source`（直链），我们自存的副本只在有人下载时才生成，不做批量；**不要对 okslop 开启定时自动同步或大批量采集**（定时同步对每个关键词逐页取，容易触碰“系统性缓存”和每日 1000 次的限额）。“壁纸聚合站”是否算“竞争服务”是个需要向 okslop（api@okslop.com）确认的判断，拿到密钥时顺便登记应用并问清。
- 请求被拒绝或限流时，沿用已有的密钥冷却。

### 3. 构建前端资源
```bash
cd web
npm install
npm run build
cd ..
```

### 4. 运行后端服务
```bash
cargo run --release
```
启动完成后访问：
- **公开展示前台**：[http://localhost:8080](http://localhost:8080)
- **管理后台 / 采集工作台**：[http://localhost:8080/admin](http://localhost:8080/admin)（默认账号：`admin` / `123456`，首次登录请在账户中修改）

---

## 📡 核心 API 接口概览

### 1. 公开前台接口（纯查已审核上架内容）
| 接口 | 方法 | 说明 |
|---|---|---|
| `/api/v1/wallpapers/search` | `GET` | 本地库壁纸检索（支持 `q`, `category`, `orientation`, `media_type`, `page`, `per_page`） |
| `/api/v1/wallpapers/curated` | `GET` | 本地已上架精选图片分页列表 |
| `/api/v1/wallpapers/videos` | `GET` | 本地已上架 4K / HD 动态视频分页列表 |
| `/api/v1/wallpapers/{provider}/{id}` | `GET` | 获取壁纸 / 视频详细信息 |
| `/api/v1/categories` | `GET` | 系统预设分类列表 |
| `/api/v1/favorites` | `GET` / `POST` | 用户收藏夹列表 / 添加收藏 |

### 2. 管理员私有接口（需管理员 Session Cookie 鉴权）
| 接口 | 方法 | 说明 |
|---|---|---|
| `/api/v1/admin/collector/search` | `GET` | 多源供应商外部壁纸实时检索（供采集工作台使用） |
| `/api/v1/admin/collector/import` | `POST` | 外部壁纸一键入库（可指定分类与 `published`/`pending` 状态） |
| `/api/v1/admin/collector/check-status` | `POST` | 批量检测壁纸 ID 是否已入库 |
| `/api/v1/admin/wallpapers` | `GET` | 管理端图库多维度筛选列表 |
| `/api/v1/admin/wallpapers/{id}/status` | `PUT` | 快速上下架切换 (`published` / `offline`) |
| `/api/v1/admin/wallpapers/{id}` | `PUT` / `DELETE` | 修改壁纸元信息 / 永久删除 |
| `/api/v1/admin/wallpapers/upload` | `POST` | 本地图片 / 视频二进制上传接口 |
| `/api/v1/admin/wallpapers/custom` | `POST` | 自定义壁纸 / 视频创建与上架发布 |

---

## 📄 开源许可证

本项目基于 [MIT](LICENSE) OR [Apache-2.0](LICENSE) 双协议开源。

网页角色核心位于 `web/src/companion`，初始从桌面客户端角色核心复制，不跨项目导入源码；交互适配在 `InteractiveXiaoban.vue`，演示桌面在 `WindowsDesktopDemo.vue`。角色和视频在离开该栏目时暂停。验证时检查深浅主题往返切换、桌面 / 手机 16:9、点击 / 拖动、开始菜单与文件夹，以及键盘访问。
