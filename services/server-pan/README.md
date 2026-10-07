# PanDown 极简网络云盘系统

> 专为个人及小微团队设计的极简、轻量、高可用自托管网络云盘系统。
> 后端采用高性能、高并发与内存安全的 **Rust (Axum + SQLite WAL)**，前端采用 **React 19 + TypeScript + Tailwind CSS**，深度融合 **Apple HIG 纯白明快与 macOS 经典红黄绿蓝纯色质感**。

---

## 📑 目录

- [核心功能特性](#-核心功能特性)
- [技术栈](#-技术栈)
- [架构与分包设计](#️-架构与分包设计)
- [快速启动](#-快速启动)
- [部署说明](#-部署说明)
- [使用说明](#-使用说明)
- [完整 API 接口文档](#-完整-api-接口文档)
- [数据存储结构](#-数据存储结构)
- [配置项说明](#️-配置项说明)
- [常见问题 FAQ](#-常见问题-faq)
- [深度调研文档](#-深度调研文档)

---

## 🌟 核心功能特性

- 🔐 **开箱即用账号系统**
  - 内置初始管理员账号：`admin`，默认密码：`123456`（登录界面内置一键填入快捷卡片）
  - 采用 SHA-256 + 32 位随机盐（Salt）加盐哈希，防彩虹表碰撞
  - JWT 无状态鉴权，支持自主修改密码
- 📁 **树形虚拟文件与文件夹管理**
  - 多层级嵌套文件夹浏览与面包屑（Breadcrumbs）一键逐层回溯
  - 拖拽与批量文件上传（支持拖拽区域与全屏 Dropzone 捕捉）
  - 文件重命名、同名文件自动防冲突重命名（`file (1).txt`）
  - 文件夹递归级联删除与物理文件安全清理（防目录遍历安全路径沙箱）
  - 全局关键词模糊搜索
- ⬇️ **高性能流式下载与多媒体预览**
  - 基于 Tokio 异步流式 I/O，大文件边读边发不占内存
  - 规范标准 RFC 5987 编码 `Content-Disposition`，完美支持中文文件名
  - 支持图片、音视频播放、PDF 阅读、文本与代码在线预览与一键复制
- 🔗 **强大的外链分享生态（支持免密与带密码提取码）**
  - **单文件与整个文件夹均可分享**：访客端可在分享的文件夹内无缝穿透浏览子目录与文件
  - **访问凭证与提取码保护**：可选公开直链，或配置自定义 / 随机生成的提取码保护
  - **有效时效控制**：支持 1 天、7 天、30 天或永久有效
  - **独立访客提取端**：访客无需注册登录，输入密码校验后签发时效凭证
  - **分享管理中心**：实时统计分享的浏览量与下载量，支持随时一键撤销
- 🎨 **Apple 极简亮色设计语言（明快活泼 · 纯色为主）**
  - 纯净透亮底色（`#FFFFFF` 与浅灰 `#F5F5F7`）
  - 经典 macOS Traffic Light 指示点（🔴 红 `#FF5F56`、🟡 黄 `#FFBD2E`、🟢 绿 `#27C93F`）
  - 经典系统纯色点缀：蓝（`#007AFF` 交互）、绿（`#34C759` 成功）、黄（`#FF9500` 加密分享）、红（`#FF3B30` 警示删除）
  - 列表视图与大图标网格视图一键切换

---

## 🛠️ 技术栈

| 层次 | 技术选型 |
| :--- | :--- |
| **后端框架** | Rust 1.80+ · Axum 0.8 · Tokio 异步运行时 |
| **数据库** | SQLite（rusqlite，WAL 模式，零外部依赖） |
| **鉴权** | JWT（jsonwebtoken）+ SHA-256 加盐哈希 |
| **文件传输** | Tokio 异步流式 I/O（`tokio_util::io::ReaderStream`） |
| **前端框架** | React 19 · TypeScript 5 · Vite 6 |
| **样式方案** | Tailwind CSS 4 · Lucide 图标 · Apple HIG 设计语言 |
| **构建产物** | 后端单二进制 + 前端静态资源（后端自动托管） |

---

## 🏗️ 架构与分包设计

系统遵循现代整洁架构（Clean Architecture）分层与高内聚低耦合原则：

```text
PanDown/
├── docs/                        # 调研报告与架构设计文档
│   └── RESEARCH_AND_DESIGN.md   # 市场主流云盘产品形态与技术栈深度调研
├── backend/                     # Rust Axum 后端服务
│   ├── Cargo.toml               # 后端依赖配置
│   └── src/
│       ├── main.rs              # 入口、路由编排、CORS、Gzip、前端静态托管
│       ├── config.rs            # 配置加载与命令行参数解析
│       ├── error.rs             # 统一强类型 AppError 与 HTTP 状态码映射
│       ├── db/                  # 数据库层（SQLite WAL 模式高并发支持）
│       │   ├── mod.rs           # 连接池封装与事务
│       │   └── schema.rs        # DDL 建表与默认 admin 账号自动播种
│       ├── models/              # 数据传输对象（User, File, Share, Stats）
│       ├── storage/             # 物理存储驱动（UUID 物理映射与路径安全防穿透）
│       ├── middleware/          # JWT 身份提取器与认证守卫
│       ├── services/            # 领域业务服务（AuthService, FileService, ShareService）
│       └── handlers/            # RESTful HTTP 控制器
└── frontend/                    # React 19 + TypeScript + Vite 前端
    ├── src/
    │   ├── App.tsx              # 单页路由守卫与公开分享视图分流
    │   ├── api.ts               # 统一 API 客户端与工具函数
    │   ├── types.ts             # TypeScript 类型定义
    │   ├── context/             # AuthContext 登录认证上下文
    │   ├── components/          # 苹果风格高质感组件库
    │   └── pages/               # 页面（Login, Drive, PublicShare）
    └── dist/                    # 前端生产打包静态资源（由后端自动单端口托管）
```

**请求处理链路：**

```text
浏览器 → [CORS / Gzip 压缩] → [JWT 鉴权提取器] → Handler 控制器
       → Service 业务服务 → [SQLite 元数据 + 磁盘物理存储]
```

---

## 🚀 快速启动

### 前置要求

| 依赖 | 版本 | 说明 |
| :--- | :--- | :--- |
| Rust | 1.80+ | 后端编译运行 |
| Node.js | 18+ | 仅前端构建/开发需要 |

### 方式一：一体化启动（推荐，单端口运行）

前端构建产物已编译至 `frontend/dist`，后端运行时会自动挂载前端静态资源：

```bash
# 1. 构建前端（首次或前端代码变更后执行）
cd frontend
npm install
npm run build

# 2. 启动后端服务（同时托管前端，默认监听 55005 端口）
cd ../backend
cargo run --release
```

在浏览器中打开：👉 **http://127.0.0.1:55005**

- 默认管理员账号：`admin`
- 默认登录密码：`123456`

### 方式二：开发模式（前端热更新 + 后端 API）

适合调试前端界面，享受 Vite 极速热更新：

```bash
# 终端 1：启动 Rust 后端
cd backend
cargo run

# 终端 2：启动前端 Vite 开发服务器
cd frontend
npm install
npm run dev
```

打开控制台输出的地址（如 `http://localhost:5173`），Vite 已配置自动反向代理 `/api` 至后端。
若后端使用非默认端口，可指定代理目标：

```bash
PANDOWN_BACKEND_PORT=9000 npm run dev
```

---

## 📦 部署说明

### 1. 生产构建

```bash
# 构建前端静态资源
cd frontend && npm install && npm run build

# 编译后端优化版二进制
cd ../backend && cargo build --release
```

产物：
- 后端可执行文件：`backend/target/release/pandown-backend`（约 6–8 MB，无运行时依赖）
- 前端静态资源：`frontend/dist/`

> ⚠️ **重要**：后端通过相对路径查找 `frontend/dist` 与 `pandown_data`。
> 请在 **项目根目录** 下运行二进制，或使用 `-d` 指定数据目录。

### 2. 首次启动

```bash
# 在项目根目录运行（会自动查找 frontend/dist 与创建 pandown_data）
./backend/target/release/pandown-backend --port 55005
```

首次启动会自动：创建 `pandown_data/` 数据目录、初始化 SQLite 数据库、播种默认管理员账号。

### 3. 后台常驻运行

使用 `nohup`：

```bash
nohup ./backend/target/release/pandown-backend --port 55005 > pandown.log 2>&1 &
```

使用 `systemd`（Linux 推荐），创建 `/etc/systemd/system/pandown.service`：

```ini
[Unit]
Description=PanDown Cloud Drive
After=network.target

[Service]
Type=simple
User=pandown
WorkingDirectory=/opt/pandown
ExecStart=/opt/pandown/backend/target/release/pandown-backend --port 55005 --data-dir /opt/pandown/data
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now pandown
sudo systemctl status pandown
```

### 4. 反向代理（可选，Nginx）

如需绑定域名 / 配置 HTTPS，可前置 Nginx：

```nginx
server {
    listen 80;
    server_name drive.example.com;

    client_max_body_size 10G;   # 允许上传大文件（默认 1M 会拒绝大文件）

    location / {
        proxy_pass http://127.0.0.1:55005;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_request_buffering off;   # 流式上传/下载，避免缓冲整个文件
        proxy_read_timeout 3600s;
    }
}
```

### 5. 生产安全建议

| 项目 | 建议 |
| :--- | :--- |
| **JWT 密钥** | 通过环境变量 `JWT_SECRET` 设置强随机值（默认值仅供开发） |
| **默认密码** | 首次登录后立即通过「修改密码」更换 `admin` 密码 |
| **监听地址** | 反向代理场景下用 `--host 127.0.0.1` 仅监听本机 |
| **数据备份** | 定期备份整个 `pandown_data/` 目录（含数据库与物理文件） |
| **防火墙** | 仅暴露反向代理端口，不直接暴露后端端口 |

---

## 📖 使用说明

### 登录

访问站点首页，输入 `admin` / `123456`。登录页提供「一键填入」按钮快速填充默认账号。

### 文件管理

| 操作 | 方式 |
| :--- | :--- |
| **上传文件** | 点击「上传文件」按钮选择，或将文件直接拖到页面任意位置 |
| **新建文件夹** | 点击「新建文件夹」，输入名称 |
| **进入文件夹** | 单击文件夹行 / 卡片 |
| **返回上级** | 点击面包屑上级路径 |
| **重命名** | 行/卡片操作区的 ✏️ 按钮 |
| **删除** | 行/卡片操作区的 🗑️ 按钮（文件夹会递归删除全部内容，有二次确认） |
| **下载** | 行/卡片操作区的 ⬇️ 按钮 |
| **预览** | 行/卡片操作区的 👁️ 按钮（支持图片/音视频/PDF/文本代码） |
| **搜索** | 顶部搜索框输入关键词，实时模糊匹配 |
| **切换视图** | 工具栏右侧「列表 / 网格」图标 |

### 分享文件 / 文件夹

1. 在文件行或卡片上点击 **分享（↗️）** 按钮，打开分享弹窗
2. 配置分享选项：
   - **提取码保护**：勾选后输入自定义提取码，或点「随机生成」
   - **有效期**：永久 / 1 天 / 7 天 / 30 天
3. 点击「创建分享」，生成分享链接与提取码
4. 点击「复制分享信息」将链接和提取码一并复制到剪贴板

**访客提取流程**：访客打开链接 → 若需提取码则输入 → 校验通过后浏览 / 下载。
访客端支持文件夹内穿透浏览与单文件下载。

### 分享管理

点击顶部「我的分享」按钮，可查看所有已创建的分享：链接、提取码、有效期、浏览量与下载量统计，并支持复制链接、新窗口打开、取消分享。

### 修改密码

点击右上角用户名 → 「修改密码」，输入原密码与新密码（至少 6 位）。

---

## 📋 完整 API 接口文档

**通用约定：**
- Base URL：`http://<host>:<port>/api`
- 认证方式：除公开接口外，均需携带 JWT 凭证，支持两种方式：
  - 请求头：`Authorization: Bearer <JWT_TOKEN>`（推荐，用于常规 API 调用）
  - 查询参数：`?token=<JWT_TOKEN>`（用于浏览器直链下载 / 预览等无法设置请求头的场景）
- 请求/响应体：除文件上传与流式下载外，均为 `application/json`
- 时间戳格式：Unix 秒级时间戳（整数）
- 错误响应统一格式：
  ```json
  { "error": "错误描述", "status": 400 }
  ```

### 一、认证模块 `/api/auth`

#### 1. 登录

```http
POST /api/auth/login
Content-Type: application/json
```

**请求体：**
```json
{
  "username": "admin",
  "password": "123456"
}
```

**响应 `200`：**
```json
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGci...",
  "user": {
    "id": 1,
    "username": "admin",
    "is_admin": true,
    "created_at": 1791381351
  }
}
```

**错误：** `401` 用户名或密码错误

#### 2. 获取当前用户信息

```http
GET /api/auth/me
Authorization: Bearer <token>
```

**响应 `200`：** `UserDto` 对象（同上 `user` 字段）

#### 3. 修改密码

```http
POST /api/auth/change-password
Authorization: Bearer <token>
Content-Type: application/json
```

**请求体：**
```json
{ "old_password": "123456", "new_password": "newpass123" }
```

**响应 `200`：** `{ "message": "密码修改成功" }`
**错误：** `400` 原密码不正确 / 新密码长度不能少于 6 位

---

### 二、文件模块 `/api/files`

> 所有接口均需认证。

#### 1. 获取目录列表

```http
GET /api/files/list?folder_id={uuid}
```

`folder_id` 可选，缺省或为空表示根目录。

**响应 `200`：**
```json
{
  "current_folder": { /* FileItem，根目录时为 null */ },
  "breadcrumbs": [
    { "id": null, "name": "全部文件" },
    { "id": "465b670d-...", "name": "设计稿与原型" }
  ],
  "items": [ /* FileItem 数组，文件夹在前 */ ]
}
```

**FileItem 结构：**
```json
{
  "id": "9fd19abe-5bab-4486-8871-98d5dbc47510",
  "user_id": 1,
  "parent_id": "465b670d-...",
  "name": "welcome.txt",
  "size": 74,
  "mime_type": "text/plain",
  "is_dir": false,
  "storage_path": "d4c9c7f0-....txt",
  "created_at": 1791381521,
  "updated_at": 1791381521
}
```

#### 2. 新建文件夹

```http
POST /api/files/mkdir
Content-Type: application/json
```

**请求体：** `{ "name": "新文件夹", "parent_id": null }`
**响应 `200`：** 新建的 `FileItem`
**错误：** `400` 名称为空 / 含非法字符 / 同级同名；`404` 父文件夹不存在

#### 3. 上传文件（支持多文件）

```http
POST /api/files/upload?folder_id={uuid}
Content-Type: multipart/form-data
```

`folder_id` 可选（根目录）。表单字段名为 `files`，可重复以批量上传。

```bash
curl -X POST "http://127.0.0.1:55005/api/files/upload?folder_id=$FOLDER_ID" \
  -H "Authorization: Bearer $TOKEN" \
  -F "files=@./a.txt" -F "files=@./b.png"
```

**响应 `200`：** 已录入的 `FileItem` 数组

#### 4. 重命名

```http
POST /api/files/rename/{id}
Content-Type: application/json
```

**请求体：** `{ "new_name": "新名字.txt" }`
**响应 `200`：** 更新后的 `FileItem`

#### 5. 删除文件 / 文件夹

```http
DELETE /api/files/{id}
```

文件夹将递归删除其全部子孙内容。**响应 `200`：** `{ "message": "删除成功" }`

#### 6. 搜索文件

```http
GET /api/files/search?q={关键词}
```

**响应 `200`：** 匹配的 `FileItem` 数组（最多返回 100 条）

#### 7. 下载文件

```http
GET /api/files/download/{id}
Authorization: Bearer <token>          # 方式一：请求头
GET /api/files/download/{id}?token=<JWT>   # 方式二：查询参数
```

**响应 `200`：** 二进制流，附 `Content-Type`、`Content-Length` 与 RFC 5987 编码的 `Content-Disposition`。

> **关于两种传参方式**：浏览器的 `<a href>` 直链下载、`<img src>` 等标签无法携带自定义 `Authorization` 请求头，因此鉴权中间件同时支持 `?token=<JWT>` 查询参数回退。前端下载即采用该方式，从而保证大文件走浏览器原生流式下载（不会像 `fetch + Blob` 那样把整个文件读入内存）。

#### 8. 预览文件（内联）

```http
GET /api/files/preview/{id}
Authorization: Bearer <token>          # 或 ?token=<JWT>
```

**响应 `200`：** 二进制流，`Content-Disposition: inline`，供浏览器直接渲染图片 / 播放音视频 / 阅读 PDF。

---

### 三、分享管理模块 `/api/shares`

> 需认证，管理当前用户创建的分享。

#### 1. 创建分享

```http
POST /api/shares/create
Content-Type: application/json
```

**请求体：**
```json
{
  "file_id": "9fd19abe-...",
  "password": "8888",      // 可选，null 表示免密公开
  "note": "项目终稿说明",   // 可选，null 表示不附带说明
  "expire_days": 7         // 可选，null 表示永久；可为 1/7/30
}
```

**响应 `200`：**
```json
{
  "code": "6LbVuU",
  "share_url": "/share/6LbVuU",
  "has_password": true,
  "password": "8888",
  "note": "项目终稿说明",
  "expires_at": 1791986321
}
```

#### 2. 我的分享列表

```http
GET /api/shares/my
```

**响应 `200`：** `ShareDto` 数组
```json
[
  {
    "code": "6LbVuU",
    "file_id": "9fd19abe-...",
    "file_name": "welcome.txt",
    "is_dir": false,
    "file_size": 74,
    "has_password": true,
    "password": "8888",
    "note": "项目终稿说明",
    "expires_at": 1791986321,
    "is_expired": false,
    "view_count": 1,
    "download_count": 1,
    "created_at": 1791381521
  }
]
```

#### 3. 编辑分享信息

```http
PATCH /api/shares/{code}
Authorization: Bearer <token>
Content-Type: application/json
```

**请求体（部分更新，未传字段不作修改）：**
```json
{
  "note": "更新后的说明（传空串可清空）",      // 可选
  "password": "9999",                       // 可选，传空串取消密码
  "expire_days": 30                         // 可选，传 0 改为永久有效
}
```

**响应 `200`：** `{ "message": "分享已更新" }`

#### 4. 取消分享

```http
DELETE /api/shares/{code}
```

**响应 `200`：** `{ "message": "分享已取消" }`

---

### 四、公开访问模块 `/api/public`

> **无需认证**，供访客提取分享内容。

#### 1. 获取分享基础信息

```http
GET /api/public/shares/{code}/info
```

**响应 `200`：**
```json
{
  "code": "6LbVuU",
  "target_name": "welcome.txt",
  "is_dir": false,
  "file_size": 74,
  "has_password": true,
  "note": "项目终稿说明",
  "is_expired": false,
  "expires_at": 1791986321,
  "creator_name": "admin",
  "created_at": 1791381521
}
```

#### 2. 校验提取码，换取访客凭证

```http
POST /api/public/shares/{code}/verify
Content-Type: application/json
```

**请求体：** `{ "password": "8888" }`（免密分享可传 `null`）
**响应 `200`：** `{ "share_token": "eyJ0eXAiOiJKV1Qi..." }`
**错误：** `401` 提取码错误；`400` 分享已过期

#### 3. 获取分享内容（支持文件夹穿透）

```http
GET /api/public/shares/{code}/content?sub_folder_id={uuid}
X-Share-Token: <share_token>
```

- `sub_folder_id` 可选：浏览分享文件夹内的子目录
- 访客凭证通过请求头 `X-Share-Token` 传递（也支持 `?token=` 查询参数）

**响应 `200`：**
```json
{
  "target_item": { /* FileItem 分享根节点 */ },
  "current_folder": { /* FileItem 当前目录，单文件分享时为 null */ },
  "breadcrumbs": [ /* 以分享根目录为起点的面包屑 */ ],
  "items": [ /* 当前目录子项，单文件分享时为该项本身 */ ]
}
```
**错误：** `401` 需要提取码 / 凭证过期；`403` 无权访问该目录；`404` 分享不存在

#### 4. 下载分享文件

```http
GET /api/public/shares/{code}/download/{file_id}?token={share_token}
```

**响应 `200`：** 二进制流（附件下载）。

#### 5. 预览分享文件（内联）

```http
GET /api/public/shares/{code}/preview/{file_id}?token={share_token}
```

**响应 `200`：** 二进制流（`Content-Disposition: inline`）。

---

### 五、系统模块 `/api/system`

#### 1. 存储统计

```http
GET /api/system/stats
Authorization: Bearer <token>
```

**响应 `200`：**
```json
{
  "total_files": 12,
  "total_folders": 3,
  "used_bytes": 1073741824,
  "total_shares": 5
}
```

---

### 六、请求示例汇总（curl）

```bash
BASE=http://127.0.0.1:55005/api

# 登录并取出 Token
TOKEN=$(curl -s -X POST $BASE/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"123456"}' \
  | grep -o '"token":"[^"]*' | cut -d'"' -f4)

# 新建文件夹
curl -s -X POST $BASE/files/mkdir \
  -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
  -d '{"name":"资料"}'

# 上传文件
curl -s -X POST "$BASE/files/upload" \
  -H "Authorization: Bearer $TOKEN" \
  -F "files=@./report.pdf"

# 列表
curl -s "$BASE/files/list" -H "Authorization: Bearer $TOKEN"

# 创建带提取码的分享
curl -s -X POST $BASE/shares/create \
  -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
  -d '{"file_id":"<FILE_ID>","password":"8888","expire_days":7}'

# 访客校验提取码
curl -s -X POST $BASE/public/shares/<CODE>/verify \
  -H "Content-Type: application/json" -d '{"password":"8888"}'
```

---

## 🗂️ 数据存储结构

所有数据默认位于项目根目录的 `pandown_data/`（可通过 `--data-dir` 修改）：

```text
pandown_data/
├── pandown.db          # SQLite 数据库（元数据：用户、文件树、分享）
├── pandown.db-wal      # WAL 预写日志（高并发读写）
├── pandown.db-shm      # 共享内存索引文件
└── storage/            # 物理文件存储目录
    ├── d4c9c7f0-363e-47bb-b907-2c54c980a6f7.txt   # 以 UUID 命名的物理文件
    └── ...
```

**设计要点：**
- **元数据与物理存储解耦**：数据库维护逻辑目录树，磁盘以 UUID 命名物理文件，杜绝重名冲突与文件名注入。
- **文件夹不占物理空间**：文件夹仅为数据库中的逻辑节点，重命名/移动为 O(1) 指针操作。
- **备份方式**：直接复制整个 `pandown_data/` 目录即可（建议停止服务后复制，或同时复制 `-wal`/`-shm` 文件）。

---

## ⚙️ 配置项说明

### 命令行参数

| 参数 | 说明 | 默认值 |
| :--- | :--- | :--- |
| `-p, --port <PORT>` | 监听端口 | `55005` |
| `-H, --host <HOST>` | 监听地址 | `0.0.0.0` |
| `-d, --data-dir <DIR>` | 数据目录（数据库 + 物理存储） | `pandown_data` |
| `-h, --help` | 打印帮助 | - |

### 环境变量

优先级：**命令行参数 > 环境变量 > 默认值**

| 变量 | 说明 | 默认值 |
| :--- | :--- | :--- |
| `PORT` | 监听端口 | `55005` |
| `HOST` | 监听地址 | `0.0.0.0` |
| `DATA_DIR` | 数据目录 | `pandown_data` |
| `JWT_SECRET` | JWT 签名密钥（**生产环境务必修改**） | 内置默认值 |
| `RUST_LOG` | 日志级别（如 `pandown_backend=debug`） | `pandown_backend=info` |

### 启动示例

```bash
# 默认启动
./pandown-backend

# 指定端口
./pandown-backend --port 9000

# 指定端口与数据目录
./pandown-backend -p 9000 -d /var/lib/pandown

# 生产环境（自定义 JWT 密钥，仅监听本机）
JWT_SECRET="$(openssl rand -hex 32)" ./pandown-backend --host 127.0.0.1 --port 55005

# 查看帮助
./pandown-backend --help
```

若端口被占用，会给出友好提示：
```
Error: 无法绑定端口 0.0.0.0:55005 —— Address already in use (os error 48)
（该端口可能已被占用，可用 --port <端口> 指定其它端口）
```

---

## ❓ 常见问题 FAQ

**Q：忘记管理员密码怎么办？**
A：删除 `pandown_data/pandown.db*` 三个文件后重启服务，会重新播种 `admin` / `123456`。⚠️ 这会清空所有元数据（物理文件仍残留在 `storage/`，但已无法通过界面访问）。

**Q：上传大文件失败 / Nginx 报 413？**
A：Nginx 默认限制 1MB。请在 `server` 块设置 `client_max_body_size 10G;`，并配置 `proxy_request_buffering off;` 启用流式传输。

**Q：如何更换端口？**
A：`./pandown-backend --port 9000`，或设置环境变量 `PORT=9000`。

**Q：前端页面显示「前端正在构建中」？**
A：说明后端未找到 `frontend/dist`。请在项目根目录运行二进制，或先执行 `cd frontend && npm run build`。

**Q：下载时中文文件名乱码？**
A：系统已按 RFC 5987 同时输出 `filename` 与 `filename*=UTF-8''...`，可兼容所有现代浏览器。

**Q：支持多用户注册吗？**
A：当前版本为单管理员模式（默认仅 `admin`），数据库结构已预留 `users` 表与 `is_admin` 字段，可平滑扩展多用户。

**Q：数据如何迁移？**
A：复制整个 `pandown_data/` 目录到目标机器，用 `--data-dir` 指向它即可。

---

## 📄 深度调研文档

详细的市面网络云盘产品形态分析（百度网盘、阿里云盘、Google Drive、Dropbox、iCloud Drive、Nextcloud、Seafile、Filebrowser、AList、Cloudreve）及各技术栈选型对比，详见：

👉 [docs/RESEARCH_AND_DESIGN.md](docs/RESEARCH_AND_DESIGN.md)
