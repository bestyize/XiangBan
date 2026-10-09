# server-xiaoban-appstore

小伴优质应用商店（Xiaoban AppStore）—— 只收录最优质的软件，传承腾讯柠檬精选实验室（Lemon Lab）的纯净美学与公信力，专为极客、开发者与数字创作者量身打造的精选软件发现平台。

## 核心设计与创新

- **视觉风格**：对标腾讯柠檬实验室（https://lemon.qq.com/lab/），极简白底结合柠檬暖黄（`#FFA805`），高清圆角图标、精细微交互浮动、带引号的灵魂编辑推荐语，绝无广告弹窗。
- **8 大特色创新分类**：
  1. `ai-agents`（🤖 AI 新纪元）：大模型与智能体时代的生产力底座与高质感客户端（Claude Desktop, CodeX, CCSwitch, Ollama 等）。
  2. `modern-classics`（🎨 经典重塑）：向工业巨兽致敬，现代技术重构、极度轻量秒开的免全家桶工具（PhotoCraft, Patchy, Typora 等）。
  3. `github-gems`（🐙 开源之光）：千锤百炼、代码透明且无商业绑架的独立开发者瑰宝（IINA, Maccy, LocalSend 等）。
  4. `day-one`（🚀 开荒必备）：新电脑开机第 1 天必装的底座基建（Raycast, 腾讯柠檬, Shottr 等）。
  5. `developer`（⚡️ 极客工坊）：程序员与黑客的雕花工具（Ghostty, OrbStack 等）。
  6. `minimalism`（🍃 数字减负）：单点极致、无弹窗、内存以 MB 计的清爽平替（Ice, Velja 等）。
  7. `focus-flow`（✍️ 心流创作）：本地优先的双链知识图谱与沉浸写作（Obsidian 等）。
  8. `life-aesthetics`（🎮 生活美学）：高颜值动态壁纸与治愈系数字陪伴（相伴动态桌面等）。
- **品质与基因多维过滤**：原生极速（Native）、GitHub 开源、安全认证、纯净无广告、本地优先（Local-First 数据主权）。
- **极客命令行友好**：每款应用提供 `brew install --cask xxx` 或 `winget install xxx` 一键复制。
- **经典管理后台**：
  - **经典布局**：左侧深色导航菜单（固定 240px）+ 右侧高效工作台；
  - **核心操作**：支持**一键上架 / 下架**（Switch 极速切换），即时控制前台可见性；
  - **全维配置**：应用增删改查、多平台支持勾选、优缺点客观分析、痛点平替标签、GitHub 仓库与 Star 统计录入。

## 技术栈

- **后端**：Rust (Axum 0.8 + Tokio + SQLx + PostgreSQL + Argon2/JWT)
- **前端**：Vue 3 + Vite 6 + TypeScript + Pinia + Vue Router + Lucide 图标
- **数据库**：PostgreSQL 17（开箱自动建表与种子数据填充）

## 开发与验证

### 1. 启动后端 (Rust)

```sh
# 依赖本地 PostgreSQL 运行在 5432 端口（默认库 xiaban）
cargo run --manifest-path services/server-xiaoban-appstore/Cargo.toml
```

服务默认监听 `http://127.0.0.1:55008`：
- 前台单页：`http://127.0.0.1:55008/`
- 后台入口：`http://127.0.0.1:55008/admin`
- 默认管理员账号：`admin`
- 默认管理员密码：`admin123`

### 2. 启动前端开发服务器 (Vite)

```sh
npm --prefix services/server-xiaoban-appstore/web ci
npm --prefix services/server-xiaoban-appstore/web run dev
```

前端开发服务器将运行在 `http://localhost:5174`，自动代理 `/api` 到后端 55008 端口。

### 3. 构建前端产物

```sh
npm --prefix services/server-xiaoban-appstore/web run build
```

产物输出至 `services/server-xiaoban-appstore/web/dist`，Rust 后端会自动托管该目录下的静态网页和 SPA 回退。

## 环境变量配置

可创建 `.env` 文件覆盖以下默认配置：

| 变量名 | 默认值 | 说明 |
| --- | --- | --- |
| `APPSTORE_PORT` | `55008` | 服务监听端口 |
| `DATABASE_URL` | `postgres://xiaban@localhost:5432/xiaban` | PostgreSQL 数据库连接串 |
| `APPSTORE_JWT_SECRET` | 内置密钥 | 管理员登录 JWT 签名密钥 |
| `APPSTORE_ADMIN_USER` | `admin` | 首次初始化默认管理员用户名 |
| `APPSTORE_ADMIN_PASS` | `admin123` | 首次初始化默认管理员密码 |
| `APPSTORE_WEB_DIR` | `services/server-xiaoban-appstore/web/dist` | 前端静态页面产物路径 |
