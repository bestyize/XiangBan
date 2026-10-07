# 网络云盘产品形态、技术栈深度调研与 PanDown 系统架构设计

---

## 第一部分：市面网络云盘产品形态与生态调研

网络云盘（Cloud Storage / Network Attached Storage Drive）历经二十余年演进，已经从单纯的“网络磁盘映射”蜕变为集**存储、同步、协作、内容流媒体化、冷热分层、加密隐私**于一体的复杂分布式系统。当前市场上的主流网盘产品形态大致可以分为两大流派：**公有商业级消费云盘** 与 **私有自托管开源云盘**。

### 1. 公有商业级消费云盘形态分析

| 产品代表 | 核心定位与形态特点 | 典型技术特征 | 痛点 / 局限性 |
| :--- | :--- | :--- | :--- |
| **百度网盘** | 国内资源共享与海量冷备存储枢纽；强依赖提取码分享生态。 | 客户端 P2P CDN 互助加速（P2SP）、全网 MD5/SHA256 秒传机制、服务端离线下载集群、AI 智能相册与人脸索引。 | 非会员严重 QOS 限速；文件合规审查与敏感词拦截；客户端臃肿、广告频繁。 |
| **阿里云盘** | 基于阿里云 OSS 基础设施的个人敏捷网盘；主打“不限速”与开发者挂载 API。 | OSS 底层分块并发、直传客户端架构；智能相册识别；开放 OpenAPI 与第三方挂载 Token 机制。 | 近期对第三方挂载流量施加限制；VIP 体系转向容量与视频转码高清度变现。 |
| **Google Drive** | 办公协同与 Workspace 深度融合驱动的文件中枢。 | Google Docs/Sheets 在线富文本并发协同引擎（OT/CRDT）、全局全文检索（结合 OCR 与深度语义理解）、严格的多级权限控制（Viewer/Commenter/Editor）。 | 缺乏中国大陆直接网络接入；大文件分享易触达“下载配额已用尽”限制；纯网盘属性弱于协同属性。 |
| **Dropbox** | 全球文件同步体验的标杆；跨平台无缝差分同步。 | **Sync Engine**（基于 Rust 重写）、块级别（Block-level）差分二进制增量同步算法、自研 Magic Pocket 冷存储硬件与存储引擎。 | 免费容量小（2GB 起）；付费阶梯昂贵；更偏重于企业文件同步与协作，而非大文件公网分享传播。 |
| **iCloud Drive** | 深度植入 Apple 生态的无感虚拟文件系统。 | 与 macOS APFS 文件系统深度联动、浅拷贝文件占位符（Dataless Files/Files on Demand）、端到端加密（Advanced Data Protection）。 | 跨平台（Windows/Web/Android）体验极其孱弱；缺乏现代化的外链与密码公开分享机制。 |

---

### 2. 私有自建 / 开源云盘方案技术栈与形态比对

自托管（Self-Hosted）网盘是个人开发者、小微团队及私有 NAS 用户的核心诉求。当前开源社区主流形态如下：

```
                    ┌───────────────────────────────┐
                    │     开源自建网盘生态形态图谱     │
                    └──────────────┬────────────────┘
                                   │
      ┌────────────────────────────┼────────────────────────────┐
      ▼                            ▼                            ▼
【全功能套件型】              【块级增量同步型】            【轻量开箱即用型】
Nextcloud / ownCloud           Seafile                    Filebrowser / Cloudreve
• PHP + Vue                  • C/Python + React         • Go + Vue / React
• 插件市场/日历/办公套件      • Git式分块快照存储         • 单二进制 / 多后端支持
• 功能极全面，但臃肿缓慢      • 同步极致可靠，挂载较深    • 快速敏捷，但扩展有限
                                   │
                                   ▼
                      【多存储聚合挂载型】: AList
                      • Go + Solid.js/React
                      • 虚拟文件系统抽象层 / 统一驱动代理
```

#### 各主流开源方案技术栈拆解：

1. **Nextcloud / ownCloud**
   - **技术栈**：PHP + Symfony/Vue + MySQL/PostgreSQL + Redis。
   - **特点**：生态最大、应用商店（App Store）插件数百款，支持协同办公、日历通讯录、在线协作。
   - **短板**：架构厚重，资源开销极大，PHP 在高并发大文件 I/O 场景下开销高，运维调优繁琐。
2. **Seafile**
   - **技术栈**：核心同步引擎为 C（ccnet/seafile-server），Web 接口为 Python (Django) + React。
   - **特点**：不直接在底层存储原生文件，而是将文件切分成 1MB~3MB 的 Block 并以快照 commit 形式存储（类似 Git 对象库）。版本历史控制与断点恢复极其稳定。
   - **短板**：底层存储非原生文件名与目录结构，一旦数据库丢失难以直接从磁盘打捞；二开复杂度高。
3. **Filebrowser**
   - **技术栈**：Go + Vue3 + BadgerDB/SQLite。
   - **特点**：极简单二进制文件；直接基于宿主机本地文件目录映射；即装即用。
   - **短板**：缺乏多租户高级外链管理、缺乏现代化的分享体系（带密码/过期/统计）。
4. **Cloudreve**
   - **技术栈**：Go + React + SQLite/MySQL。
   - **特点**：多存储策略支持（本地磁盘、七牛、又拍、S3、OneDrive），支持客户端直传与离线下载。
   - **短板**：对中轻量场景依然需要单独的前后端构建与配置文件管理。
5. **AList**
   - **技术栈**：Go + Solid.js / React + SQLite。
   - **特点**：聚合挂载利器，屏蔽几十种云存储协议差异，提供统一 Web 视图和 WebDAV 服务。

---

### 3. 技术栈演进规律与 Rust 的独特优势

在云盘的核心业务中，系统的性能瓶颈通常集中于：
1. **高并发高吞吐的文件 I/O**：上传时流式写入磁盘，下载时基于 HTTP Range 的分片零拷贝流式传输；
2. **内存控制与安全性**：避免处理百兆或数吉字节文件时发生缓冲溢出或爆内存；
3. **单二进制极简交付**：不需要复杂的运行时环境（如 Python、PHP 或 JRE），降低部署与运维成本。

**为什么选择 Rust 构建新一代轻量网盘？**
- **内存安全与零成本抽象**：Rust 借用检查器消除数据竞争和悬垂指针，长时间运行服务不泄露内存；
- **现代异步运行时（Tokio + Axum）**：基于 epoll/kqueue 的极速异步事件循环，天生适合数千并发连接的流式大文件传输；
- **单静态可执行文件分发**：结合嵌入式数据库（SQLite WAL 模式）与前端静态资源打包，编译后仅单个可执行文件，极简无依赖部署。

---

## 第二部分：PanDown 系统架构设计

PanDown 旨在打造一个**架构清晰易懂、开箱即用、性能强悍、视觉极佳**的现代化个人/团队网盘。

### 1. 系统分层架构（Clean Architecture）

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    前端表现层 (Presentation Layer)                        │
│    Apple HIG 风格 (纯白高光基调 + 经典活力红/黄/绿/蓝/紫 状态点缀)       │
│    React 19 + TypeScript + Tailwind CSS + Lucide Icons + Vite           │
│  [文件管理]     [拖拽多文件上传]     [音视频/文档预览]     [外链提取与分享]  │
└────────────────────────────────────┬────────────────────────────────────┘
                                     │ HTTP RESTful / Streaming API
┌────────────────────────────────────▼────────────────────────────────────┐
│                    传输适配层 (Transport Layer - Axum)                   │
│   • JWT Auth Extractor 中间件     • CORS 跨域与 Compression Gzip 压缩    │
│   • Multipart/Stream 异步解析器   • Range Header 字节范围流式分片器     │
└────────────────────────────────────┬────────────────────────────────────┘
                                     │
┌────────────────────────────────────▼────────────────────────────────────┐
│                    业务领域层 (Domain & Service Layer)                   │
│   ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐      │
│   │   AuthService    │  │   FileService    │  │   ShareService   │      │
│   │ 密码加盐校验/Token│  │ 虚拟目录/CRUD/搜索│  │ 提取码/时效/权限 │      │
│   └──────────────────┘  └──────────────────┘  └──────────────────┘      │
└────────────────────────────────────┬────────────────────────────────────┘
                                     │
┌────────────────────────────────────▼────────────────────────────────────┐
│                    基础设施与存储层 (Infrastructure Layer)               │
│   ┌──────────────────────────────────┐  ┌─────────────────────────────┐ │
│   │        SQLite 引擎 (WAL 模式)    │  │    LocalAsyncStorage 驱动   │ │
│   │   (零外部依赖，极速原子事务)     │  │ (Tokio Async FS + 安全路径) │ │
│   └──────────────────────────────────┘  └─────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────┘
```

### 2. 关键业务机制设计

#### (1) 账号与认证机制
- **默认管理员**：系统初次启动时，若数据库为空，自动初始化管理员账号：
  - 用户名：`admin`
  - 默认密码：`123456`（采用 SHA-256 + 独立高强度随机盐（Salt）哈希存储）
- **无状态鉴权**：登录成功签发 JWT Token，携带 `user_id` 与 `username`，前端统一通过 `Authorization: Bearer <token>` 传递，支持安全登出与密码重置。

#### (2) 树形文件系统与物理隔离
- **元数据与物理存储解耦**：
  - 数据库维护逻辑层级目录树（包含 `id`, `parent_id`, `name`, `is_dir`, `size`, `mime_type` 等）；
  - 真实磁盘物理存储位于 `pandown_data/storage/` 目录下，文件以 UUID 进行唯一命名物理隔离，防止文件重名冲突和底层文件名注入攻击；
  - 文件夹只作为元数据节点存在，无需在物理磁盘创建实体空目录，重命名与移动操作只需变更关系型指针，性能达到 $O(1)$。

#### (3) 分享体系（带密码 / 免密码 / 有效期控制）
- **分享模型设计**：
  - 支持分享单个文件，也支持分享整个**文件夹**（访客端可无缝在分享文件夹内穿透浏览子文件/子目录并按需下载）；
  - **访问密码**：可选配置。不填即为免密公开分享；填写则生成带提取码的保护外链；
  - **有效期**：支持自定义有效时长（如 1小时、1天、7天、永久），超时自动失效；
  - **访问凭证（Share Token）**：当访客输入正确提取码后，服务端签发带有时效签名的轻量访问 Token，避免每次点击子文件都需重复输入密码。

#### (4) UI 规范：Apple 纯白极简与原色活力设计
遵循 Apple Human Interface Guidelines（HIG）的现代亮色美学：
- **基底色彩**：
  - 纯白背景：`#FFFFFF` 与次级背景浅冷灰 `#F5F5F7` / `#F8FAFC`，营造通透呼吸感；
  - 细腻描边：`border border-slate-200/80`，避免生硬黑线；
  - 柔和光影：`box-shadow: 0 4px 20px -2px rgba(0, 0, 0, 0.05)`。
- **macOS / iOS 经典纯色点缀**：
  - 🔴 **Apple Red** (`#FF3B30` / `#FF5F56`)：用于删除、危险警告、窗体关闭红点；
  - 🟡 **Apple Yellow** (`#FF9500` / `#FFBD2E`)：用于加密外链、私密状态、收藏、窗体最小化黄点；
  - 🟢 **Apple Green** (`#34C759` / `#27C93F`)：用于公开分享、上传成功、系统就绪、窗体全屏绿点；
  - 🔵 **Apple Blue** (`#007AFF`)：品牌主色彩、主操作按钮、选中聚焦、面包屑高亮；
  - 🟣 **Apple Purple** (`#AF52DE`)：多媒体视听文件徽标点缀。

---

## 第三部分：数据库模型设计 (SQLite Schema)

```sql
-- 1. 用户表
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    salt TEXT NOT NULL,
    is_admin INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL
);

-- 2. 文件元数据表
CREATE TABLE IF NOT EXISTS files (
    id TEXT PRIMARY KEY,               -- 逻辑唯一UUID
    user_id INTEGER NOT NULL,          -- 归属用户
    parent_id TEXT,                    -- 父文件夹UUID，根目录为NULL
    name TEXT NOT NULL,                -- 逻辑显示文件名
    size INTEGER NOT NULL DEFAULT 0,   -- 字节大小
    mime_type TEXT NOT NULL,           -- 文件MIME类型
    is_dir INTEGER NOT NULL DEFAULT 0, -- 1为文件夹，0为文件
    storage_path TEXT,                 -- 物理磁盘相对路径（目录为NULL）
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY(parent_id) REFERENCES files(id) ON DELETE CASCADE
);

-- 3. 分享表
CREATE TABLE IF NOT EXISTS shares (
    code TEXT PRIMARY KEY,             -- 友好的短码 (例如 6-8 位随机字母数字)
    user_id INTEGER NOT NULL,          -- 创建者用户ID
    file_id TEXT NOT NULL,             -- 分享的目标文件或文件夹UUID
    password TEXT,                     -- 访问密码/提取码（为NULL时表示免密公开）
    expires_at INTEGER,                -- 过期时间戳（为NULL时表示永久有效）
    view_count INTEGER NOT NULL DEFAULT 0,     -- 浏览次数
    download_count INTEGER NOT NULL DEFAULT 0, -- 下载次数
    created_at INTEGER NOT NULL,
    FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY(file_id) REFERENCES files(id) ON DELETE CASCADE
);
```

---

## 第四部分：API 接口设计契约

### 1. 认证接口
- `POST /api/auth/login`：登录接口（支持用户名和密码，返回 JWT Token 及用户信息）
- `GET  /api/auth/me`：获取当前登录用户信息
- `POST /api/auth/change-password`：修改密码

### 2. 文件操作接口
- `GET  /api/files/list?folder_id={uuid}`：获取指定目录下的文件和文件夹列表与路径面包屑
- `POST /api/files/mkdir`：新建文件夹
- `POST /api/files/upload`：多文件上传（Multipart Form-Data，支持保留逻辑文件名与文件大小）
- `DELETE /api/files/{id}`：删除文件或递归删除文件夹
- `POST /api/files/rename`：重命名文件或文件夹
- `GET  /api/files/download/{id}`：下载单个文件（支持 Content-Disposition 与 Range 头）
- `GET  /api/files/preview/{id}`：内联预览文件（返回原样 MIME Type，供浏览器展示图片/PDF/视频/文本）
- `GET  /api/files/search?q={query}`：关键词全文模糊搜索
- `GET  /api/system/stats`：存储统计（已用容量、文件数、文件夹数、系统环境）

### 3. 分享接口
- `POST /api/shares/create`：创建文件或文件夹分享（可传 `file_id`, `password` (可选), `expire_days` (可选)）
- `GET  /api/shares/my`：查看当前用户的所有分享列表
- `DELETE /api/shares/{code}`：取消/删除指定分享
- `GET  /api/public/shares/{code}/info`：公开获取分享基础信息（判断是否需要密码、目标名称、类型、是否已失效）
- `POST /api/public/shares/{code}/verify`：校验分享提取码，换取访问凭证
- `GET  /api/public/shares/{code}/content?sub_folder_id={id}`：免密或通过验证后，获取分享内容树（支持文件夹内穿透浏览）
- `GET  /api/public/shares/{code}/download/{file_id}`：访客直接下载分享的文件
- `GET  /api/public/shares/{code}/preview/{file_id}`：访客预览分享的文件
