> 译注：中文全文由 Terra 对照[固定版本英文 README](https://github.com/calcom/cal.diy/blob/81ce067c1d0e16b21c39c2de98b9a43a92562cd1/README.md)翻译和审校，遵循上游 [MIT 许可证](https://github.com/calcom/cal.diy/blob/81ce067c1d0e16b21c39c2de98b9a43a92562cd1/LICENSE)。上游将本项目限定为建议个人、非生产用途；安装示例和默认账号仅为上游文档内容。Basecamp 官方入口的原文链接缺少协议，译文已补为已验证的 HTTPS 地址。代码和示例保留原文。

> [!WARNING]  
> 使用风险自负。Cal.diy 是 Cal.com 的开源社区版，面向希望自行托管 Cal.diy 实例的用户。上游强烈建议仅用于个人、非生产用途。请仔细审阅所有安装和配置步骤。自行托管需要高级服务器管理、数据库管理及敏感数据安全知识；只有能够承担这些责任时才继续。

> [!TIP]
> 对于商业及企业级排程基础设施，上游建议使用 Cal.com，而非 Cal.diy；可以使用其托管服务，或在此申请本地企业版访问： <a href="https://cal.com/sales">https://cal.com/sales</a>

<!-- PROJECT LOGO -->
<p align="center">
  <a href="https://github.com/calcom/cal.diy">
   <img src="https://user-images.githubusercontent.com/8019099/210054112-5955e812-a76e-4160-9ddd-58f2c72f1cce.png" alt="Logo">
  </a>

  <h3 align="center">Cal.diy</h3>

  <p align="center">
    社区驱动的开源预约排程平台。
    <br />
    <a href="https://github.com/calcom/cal.diy"><strong>GitHub</strong></a>
    <br />
    <a href="https://github.com/calcom/cal.diy/issues">问题</a>
    &middot;
    <a href="https://github.com/calcom/cal.diy/blob/81ce067c1d0e16b21c39c2de98b9a43a92562cd1/CONTRIBUTING.md">贡献</a>
  </p>
</p>

<p align="center">
   <a href="https://github.com/calcom/cal.diy/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-purple" alt="License"></a>
   <a href="https://github.com/calcom/cal.diy/stargazers"><img src="https://img.shields.io/github/stars/calcom/cal.diy" alt="Github Stars"></a>
   <a href="https://github.com/calcom/cal.diy/pulse"><img src="https://img.shields.io/github/commit-activity/m/calcom/cal.diy" alt="Commits-per-month"></a>
   <a href="https://hub.docker.com/r/calcom/cal.diy"><img src="https://img.shields.io/docker/pulls/calcom/cal.diy" alt="Docker Pulls"></a>
   <a href="https://github.com/calcom/cal.diy/issues?q=is:issue+is:open+label:%22%F0%9F%99%8B%F0%9F%8F%BB%E2%80%8D%E2%99%82%EF%B8%8Fhelp+wanted%22"><img src="https://img.shields.io/badge/Help%20Wanted-Contribute-blue"></a>
   <a href="https://contributor-covenant.org/version/1/4/code-of-conduct/"><img src="https://img.shields.io/badge/Contributor%20Covenant-1.4-purple" /></a>
</p>

<!-- ABOUT THE PROJECT -->

## 关于 Cal.diy

<img width="100%" alt="booking-screen" src="https://github.com/calcom/cal.diy/assets/8019099/407e727e-ff19-4ca4-bcae-049dca05cf02">

**Cal.diy** 是社区驱动的完全开源排程平台，基于 [Cal.com](https://cal.com) 分叉并移除了所有企业／商业代码。

Cal.diy **100% 采用 MIT 许可**，没有专有“企业版”功能。它面向希望完全掌控排程基础设施、且不依赖商业组件的个人和自托管用户。

### 与 Cal.com 有什么不同？

- **没有企业功能**——团队、组织、洞察、工作流、SSO/SAML 及其他企业版专属功能均已移除。
- **无需许可证密钥**——开箱即用，无需 Cal.com 账户或许可证。
- **100% 开源**——整个代码库均采用 MIT 许可，没有“开放核心”划分。
- **社区维护**——欢迎直接向本项目贡献（参阅 [CONTRIBUTING.md](https://github.com/calcom/cal.diy/blob/81ce067c1d0e16b21c39c2de98b9a43a92562cd1/CONTRIBUTING.md)）。

> **注意：** Cal.diy 是自托管项目，没有托管／管理版本，需要在自己的基础设施上运行。

### 使用的技术

- [Next.js](https://nextjs.org/)
- [tRPC](https://trpc.io/)
- [React.js](https://reactjs.org/)
- [Tailwind CSS](https://tailwindcss.com/)
- [Prisma.io](https://prisma.io/)
- [Daily.co](https://daily.co/)

<!-- GETTING STARTED -->

## 入门

按照以下步骤运行本地副本。

### 前提条件

运行 Cal.diy 需要：

- Node.js（版本：>=18.x）
- PostgreSQL（版本：>=13.x）
- Yarn（_推荐_）

> 如需启用集成，可能需要分别获取额外凭据。详情见下方[集成章节](#integrations)。

## 开发

### 设置

1. 克隆仓库（或在 https://github.com/calcom/cal.diy/fork 创建 fork）

   ```sh
   git clone https://github.com/calcom/cal.diy.git
   ```

   > 在 Windows 上，请使用管理员权限在 Git Bash 中运行以下命令：
   > `git clone -c core.symlinks=true https://github.com/calcom/cal.diy.git`

2. 进入项目目录

   ```sh
   cd cal.diy
   ```

3. 使用 yarn 安装软件包

   ```sh
   yarn
   ```

4. 设置 `.env` 文件

   - 将 `.env.example` 复制为 `.env`
   - 使用 `openssl rand -base64 32` 生成密钥，写入 `.env` 文件的 `NEXTAUTH_SECRET`。
   - 使用 `openssl rand -base64 24` 生成密钥，写入 `.env` 文件的 `CALENDSO_ENCRYPTION_KEY`。

 > **Windows 用户：** 将 `packages/prisma/.env` 符号链接替换为实际副本，以避免 Prisma 错误（`unexpected character / in variable name`）：
 >
 > ```sh
 > # Git Bash / WSL
 > rm packages/prisma/.env && cp .env packages/prisma/.env
 > ```

5. 设置 Node
   如果 Node 版本不符合文档所述项目要求，可使用“nvm”（Node Version Manager）切换到项目所需版本：

   ```sh
   nvm use
   ```

   可能需要先安装指定版本再使用：

   ```sh
   nvm install && nvm use
   ```

   可从[这里](https://github.com/nvm-sh/nvm)安装 nvm。

#### 使用 `yarn dx` 快速开始

> - **需要安装 Docker 和 Docker Compose**
> - 会启动本地 Postgres 实例并创建一些测试用户，凭据会输出到控制台

```sh
yarn dx
```

**创建的默认凭据：**

| 邮箱 | 密码 | 角色 |
|-------|----------|------|
| `free@example.com` | `free` | 免费用户 |
| `pro@example.com` | `pro` | Pro 用户 |
| `trial@example.com` | `trial` | 试用用户 |
| `admin@example.com` | `ADMINadmin2022!` | 管理员 |
| `onboarding@example.com` | `onboarding` | 尚未完成引导 |

可以使用任一凭据在 [http://localhost:3000](http://localhost:3000) 登录。

> **提示：** 要查看预置用户完整列表及详情，运行 `yarn db-studio` 并访问 [http://localhost:5555](http://localhost:5555)。

#### 开发提示

1. 在 shell 脚本中添加 `export NODE_OPTIONS="--max-old-space-size=16384"` 以提高 Node 进程内存上限，也可在运行应用前在终端执行。将 16384 替换为希望分配给 Node 进程的内存量。

2. 在 .env 文件中添加 `NEXT_PUBLIC_LOGGER_LEVEL={level}`，控制所有 tRPC 查询与 mutation 的日志详细程度。\
   其中 {level} 可以是以下值之一：

   `0`：silly \
   `1`：trace \
   `2`：debug \
   `3`：info \
   `4`：warn \
   `5`：error \
   `6`：fatal

   在 .env 文件中设置 `NEXT_PUBLIC_LOGGER_LEVEL={level}` 后，会启用该级别及更高级别的日志。具体如下：

   日志器会包含指定级别及更高级别的所有日志。例如： \

   - 设置 `NEXT_PUBLIC_LOGGER_LEVEL=2` 后，从级别 2（debug）开始记录，即记录 2（debug）、3（info）、4（warn）、5（error）和 6（fatal）。 \
   - 设置 `NEXT_PUBLIC_LOGGER_LEVEL=3` 后，从级别 3（info）开始记录，即记录 3（info）、4（warn）、5（error）和 6（fatal），忽略 2（debug）和 1（trace）。 \

```sh
echo 'NEXT_PUBLIC_LOGGER_LEVEL=3' >> .env
```

例如将日志级别设为 info。

#### Gitpod 设置

1. 点击下方按钮，在 Gitpod 中打开此项目。

2. 浏览器将打开配置完整的工作区，所有必要依赖均已安装。

[![Open in Gitpod](https://gitpod.io/button/open-in-gitpod.svg)](https://gitpod.io/#https://github.com/calcom/cal.diy)

#### 手动设置

1. 在 `.env` 文件中配置环境变量，将 `<user>`、`<pass>`、`<db-host>` 和 `<db-port>` 替换为对应值。

   ```
   DATABASE_URL='postgresql://<user>:<pass>@<db-host>:<db-port>'
   ```

   <details>
   <summary>如果不知道如何配置 DATABASE_URL，可按照以下步骤快速创建本地数据库</summary>

   1. 如果尚未安装，请在本地[下载](https://www.postgresql.org/download/)并安装 PostgreSQL。

   2. 执行 `createDB <DB name>` 创建本地数据库。

   3. 使用创建的数据库打开 psql shell：`psql -h localhost -U postgres -d <DB name>`。

   4. 在 psql shell 中执行 `\conninfo`，会得到以下信息。
      ![image](https://user-images.githubusercontent.com/39329182/236612291-51d87f69-6dc1-4a23-bf4d-1ca1754e0a35.png)

   5. 提取信息并写入 DATABASE_URL。URL 大致如下：
      `postgresql://postgres:postgres@localhost:5432/Your-DB-Name`. 端口可以配置，不一定是 5432。

   </details>

   如果不想创建本地数据库，也可考虑 railway.app、Northflank 或 render 等服务。
   

   - [使用 railway.app 设置 Postgres 数据库](https://docs.railway.app/guides/postgresql)
   - [使用 Northflank 设置 Postgres 数据库](https://northflank.com/guides/deploy-postgres-database-on-northflank)
   - [使用 render 设置 Postgres 数据库](https://render.com/docs/databases)

2. 将 `.env` 中的 `DATABASE_URL` 复制到 `.env.appStore`。

3. 使用 Prisma schema 设置数据库（位于 `packages/prisma/schema.prisma`）。

   在开发环境中运行：

   ```sh
   yarn workspace @calcom/prisma db-migrate
   ```

   在生产环境中运行：

   ```sh
   yarn workspace @calcom/prisma db-deploy
   ```

  **Windows/PowerShell 用户注意：** 如果数据库部署脚本报错 `Environment variable not found: DATABASE_DIRECT_URL`，可能是 Turbo 未能注入根目录 `.env` 变量。可在 PowerShell 中直接从 prisma 包目录执行命令，绕过此问题：

```powershell
cd packages/prisma
$env:DATABASE_URL="postgresql://postgres:YOUR_PASSWORD@localhost:5432/postgres"; $env:DATABASE_DIRECT_URL="postgresql://postgres:YOUR_PASSWORD@localhost:5432/postgres"
npx prisma db push
cd ../..
```
   
4. 运行 [mailhog](https://github.com/mailhog/MailHog)，查看开发期间发送的邮件。

   > **_注意：_** 当 `E2E_TEST_MAILHOG_ENABLED` 为 "1" 时必需。

   ```sh
   docker pull mailhog/mailhog
   docker run -d -p 8025:8025 -p 1025:1025 mailhog/mailhog
   ```

5. 运行（开发模式）

   ```sh
   yarn dev
   ```

#### 设置第一个用户

##### 方法 1

1. 打开 [Prisma Studio](https://prisma.io/studio)，查看或修改数据库内容：

   ```sh
   yarn db-studio
   ```

1. 点击 `User` 模型，添加新用户记录。
1. 填写 `email`、`username`、`password`，并将 `metadata` 设为空 `{}`（记得使用 [BCrypt](https://bcrypt-generator.com/) 加密密码），点击 `Save 1 Record` 创建第一个用户。
   > 新用户默认使用 `TRIAL` 计划。可根据需要在 `packages/prisma/schema.prisma` 文件中调整此行为。
1. 在浏览器打开 [http://localhost:3000](http://localhost:3000)，使用刚创建的第一个用户登录。

##### 方法 2

运行以下命令初始化本地数据库：

```sh
cd packages/prisma
yarn db-seed
```

以上命令会在本地数据库中填充示例用户。

### 端到端测试

务必将环境变量 `NEXTAUTH_URL` 设为正确值。如 `.env.example` 文档所述，在本地运行时该值应为 `http://localhost:3000`。

```sh
# In a terminal just run:
yarn test-e2e

# To open the last HTML report run:
yarn playwright show-report test-results/reports/playwright-html-report
```

#### 排查问题

##### 未安装端到端测试浏览器

运行 `npx playwright install` 下载测试浏览器，以解决执行 `yarn test-e2e` 时出现的以下错误：

```
Executable doesn't exist at /Users/alice/Library/Caches/ms-playwright/chromium-1048/chrome-mac/Chromium.app/Contents/MacOS/Chromium
```

### 从早期版本升级

1. 拉取当前版本：

   ```sh
   git pull
   ```

1. 检查依赖是否新增、更新或移除。

   ```sh
   yarn
   ```

1. 运行以下命令中的<b>一个</b>以应用数据库迁移：

   在开发环境中运行：

   ```sh
   yarn workspace @calcom/prisma db-migrate
   ```

   （某些情况下这可能清空开发数据库。）

   在生产环境中运行：

   ```sh
   yarn workspace @calcom/prisma db-deploy
   ```

1. 检查 `.env` 变量变更。

   ```sh
   yarn predev
   ```

1. 启动服务器。开发环境中运行：

   ```sh
   yarn dev
   ```

   生产构建可例如运行：

   ```sh
   yarn build
   yarn start
   ```

1. 开始使用新版本。

<!-- DEPLOYMENT -->

## 部署

### Docker

Docker 镜像位于 DockerHub：[https://hub.docker.com/r/calcom/cal.diy](https://hub.docker.com/r/calcom/cal.diy)。

**ARM 用户注意：** 拉取镜像时使用 {version}-arm 后缀，例如 `docker pull calcom/cal.diy:v5.6.19-arm`。

#### 要求

确保服务器／系统已安装 `docker` 和 `docker compose`。包括 Docker Desktop 和 Rancher Desktop 在内的大多数 Docker 工具都会安装这两者。

注意：根据 Docker 文档，不带连字符的 `docker compose` 现在是使用 docker-compose 的主要方式。

#### 使用 Docker Compose 运行 Cal.diy

1. 克隆仓库

   ```bash
   git clone --recursive https://github.com/calcom/cal.diy.git
   ```

2. 进入目录

   ```bash
   cd cal.diy
   ```

3. 准备配置：将 `.env.example` 重命名为 `.env`，然后更新 `.env`。

   ```bash
   cp .env.example .env
   ```

   大多数配置可保持原样；配置选项见下方[重要运行时变量](#important-run-time-variables)。

   **必需的密钥**

   启动前必须为 `NEXTAUTH_SECRET` 和 `CALENDSO_ENCRYPTION_KEY` 生成安全值。在生产环境使用默认的 `secret` 占位值存在安全风险。

   生成 `NEXTAUTH_SECRET`（cookie 加密密钥）：

   ```bash
   openssl rand -base64 32
   ```

   生成 `CALENDSO_ENCRYPTION_KEY`（AES256 要求 32 字节）：

   ```bash
   openssl rand -base64 24
   ```

   将这些值写入 `.env` 文件：

   ```env
   NEXTAUTH_SECRET=<your_generated_secret>
   CALENDSO_ENCRYPTION_KEY=<your_generated_key>
   ```

   **推送通知（VAPID 密钥）**
   如果看到类似错误：

   ```
   Error: No key set vapidDetails.publicKey
   ```

   这表示 Web Push 的环境变量缺失。
   必须生成并设置 `NEXT_PUBLIC_VAPID_PUBLIC_KEY` 和 `VAPID_PRIVATE_KEY`。

   使用以下命令生成：

   ```bash
   npx web-push generate-vapid-keys
   ```

   然后更新 `.env` 文件：

   ```env
   NEXT_PUBLIC_VAPID_PUBLIC_KEY=your_public_key_here
   VAPID_PRIVATE_KEY=your_private_key_here
   ```

   **不要**将真实密钥提交到 `.env.example`，只能放占位符。

   更新 .env 文件中的相应值，然后继续。

4. （可选）运行以下命令预先拉取镜像：

   ```bash
   docker compose pull
   ```

5. 通过 docker compose 启动 Cal.diy

   要运行完整组件（本地 Postgres 数据库、Cal.diy 网页应用及 Prisma Studio）：

   ```bash
   docker compose up -d
   ```

   要让 Cal.diy 网页应用和 Prisma Studio 使用远程数据库，请确保 DATABASE_URL 指向可用数据库，然后运行：

   ```bash
   docker compose up -d calcom studio
   ```

   如果只运行 Cal.diy 网页应用，请确保 DATABASE_URL 指向可用数据库，然后运行：

   ```bash
   docker compose up -d calcom
   ```

   **注意：要以前台附着模式调试，请从所需运行命令中移除 `-d`。**

6. 在浏览器打开 [http://localhost:3000](http://localhost:3000) 或你定义的 NEXT_PUBLIC_WEBAPP_URL。首次运行 Cal.diy 时会启动设置向导，创建第一个用户后即可使用。

   **首次设置注意（日历集成）：** 设置向导中“Connect your Calendar”步骤可能看似必填。如果暂时不想连接日历，可直接访问 `<NEXT_PUBLIC_WEBAPP_URL>/event-types` 仪表板跳过，之后可在 Settings > Integrations 页面添加日历集成。

#### 更新 Cal.diy

1. 停止 Cal.diy 服务栈

   ```bash
   docker compose down
   ```

2. 拉取最新更改

   ```bash
   docker compose pull
   ```

3. 按需更新环境变量。
4. 重启 Cal.diy 服务栈

   ```bash
   docker compose up -d
   ```

#### 使用 Docker 从源码构建

1. 克隆仓库

   ```bash
   git clone https://github.com/calcom/cal.diy.git
   ```

2. 进入目录

   ```bash
   cd cal.diy
   ```

3. 将 `.env.example` 重命名为 `.env`，然后更新 `.env`。

   配置选项见下方[构建时变量](#build-time-variables)。更新 .env 文件中的相应值，然后继续。

4. 构建 Cal.diy Docker 镜像：

   注意：由于应用配置要求，当前构建过程中需要可用的数据库。

   a) 如果数据库托管在别处，请在 .env 文件中配置 `DATABASE_URL` 并跳过下一步。

   b) 如果需要本地或临时数据库，请通过 docker compose 启动本地数据库。

   ```bash
   docker compose up -d database
   ```

5. 通过 docker compose 构建 Cal.diy（必须提供 DOCKER_BUILDKIT=0，才能在构建时使用网络桥接；未来将移除此要求）。

   ```bash
   DOCKER_BUILDKIT=0 docker compose build calcom
   ```

6. 通过 docker compose 启动 Cal.diy

   要运行完整组件（本地 Postgres 数据库、Cal.diy 网页应用及 Prisma Studio）：

   ```bash
   docker compose up -d
   ```

   要让 Cal.diy 网页应用和 Prisma Studio 使用远程数据库，请确保 DATABASE_URL 指向可用数据库，然后运行：

   ```bash
   docker compose up -d calcom studio
   ```

   如果只运行 Cal.diy 网页应用，请确保 DATABASE_URL 指向可用数据库，然后运行：

   ```bash
   docker compose up -d calcom
   ```

   **注意：要以前台附着模式调试，请从所需运行命令中移除 `-d`。**

7. 在浏览器打开 [http://localhost:3000](http://localhost:3000) 或定义的 NEXT_PUBLIC_WEBAPP_URL。首次运行时会启动设置向导，创建第一个用户后即可使用。

#### 配置

<a id="important-run-time-variables"></a>
##### 重要运行时变量

运行时也必须提供这些变量。

| 变量 | 说明 | 是否必需 | 默认值 |
| --- | --- | --- | --- |
| DATABASE_URL | 带凭据的数据库 URL；如使用连接池，该设置应指向连接池 | 必需 | `postgresql://unicorn_user:magical_password@database:5432/calendso` |
| NEXT_PUBLIC_WEBAPP_URL | 站点基础 URL。注意：如果与构建时的值不同，容器启动会略有延迟，以更新静态构建文件。 | 可选 | `http://localhost:3000` |
| NEXTAUTH_URL | 认证服务器的位置，默认为 Cal.diy Docker 实例本身。 | 可选 | `{NEXT_PUBLIC_WEBAPP_URL}/api/auth` |
| NEXTAUTH_SECRET | Cookie 加密密钥，必须与构建变量一致。生成方法： `openssl rand -base64 32` | 必需 | `secret` |
| CALENDSO_ENCRYPTION_KEY | 认证加密密钥（AES256 为 32 字节），必须与构建变量一致。生成方法： `openssl rand -base64 24` | 必需 | `secret` |

<a id="build-time-variables"></a>
##### 构建时变量

自行构建镜像时，必须在 Docker 构建时提供这些变量，可通过更新 .env 文件提供。目前如需更改这些变量，必须按照说明自行构建并发布镜像。

| 变量 | 说明 | 是否必需 | 默认值 |
| --- | --- | --- | --- |
| DATABASE_URL | 带凭据的数据库 URL；如使用连接池，该设置应指向连接池 | 必需 | `postgresql://unicorn_user:magical_password@database:5432/calendso` |
| MAX_OLD_SPACE_SIZE | Nodejs/NPM 构建选项所需 | 必需 | 4096 |
| NEXTAUTH_SECRET | Cookie 加密密钥 | 必需 | `secret` |
| CALENDSO_ENCRYPTION_KEY | 认证加密密钥 | 必需 | `secret` |
| NEXT_PUBLIC_WEBAPP_URL | 注入静态文件的基础 URL | 可选 | `http://localhost:3000` |
| NEXT_PUBLIC_WEBSITE_TERMS_URL | 条款网站的自定义 URL | 可选 | |
| NEXT_PUBLIC_WEBSITE_PRIVACY_POLICY_URL | 隐私政策网站的自定义 URL | 可选 | |
| CALCOM_TELEMETRY_DISABLED | 允许 Cal.diy 收集匿名使用数据（设为 `1` 禁用） | 可选 | |

#### 排查问题

##### SSL 边缘终止

如果在处理 SSL 证书的负载均衡器后运行，需要添加环境变量 `NODE_TLS_REJECT_UNAUTHORIZED=0` 防止请求被拒绝。只有了解其影响、并信任将流量导向服务的服务／负载均衡器时才这样做。（译注：该变量会关闭 TLS 证书验证。）

##### 提交更改失败：Invalid 'prisma.user.create()'

某些版本在 `metadata` 字段为空时可能无法创建用户。将字段值设为空 JSON 对象 `{}` 应可解决。`id` 字段会自动递增，也可以尝试将 `id` 留空。

##### CLIENT_FETCH_ERROR

如果遇到此错误，可能与服务器默认认证回调用 WEBAPP_URL 作为基础 URL 的方式有关。容器不一定能访问与本机相同的 DNS，因此需要配置为能解析到自身。可尝试设置 `NEXTAUTH_URL=http://localhost:3000/api/auth`，帮助后端回环到自身。

```
docker-calcom-1  | @calcom/web:start: [next-auth][error][CLIENT_FETCH_ERROR]
docker-calcom-1  | @calcom/web:start: https://next-auth.js.org/errors#client_fetch_error request to http://testing.localhost:3000/api/auth/session failed, reason: getaddrinfo ENOTFOUND testing.localhost {
docker-calcom-1  | @calcom/web:start:   error: {
docker-calcom-1  | @calcom/web:start:     message: 'request to http://testing.localhost:3000/api/auth/session failed, reason: getaddrinfo ENOTFOUND testing.localhost',
docker-calcom-1  | @calcom/web:start:     stack: 'FetchError: request to http://testing.localhost:3000/api/auth/session failed, reason: getaddrinfo ENOTFOUND testing.localhost\n' +
docker-calcom-1  | @calcom/web:start:       '    at ClientRequest.<anonymous> (/calcom/node_modules/next/dist/compiled/node-fetch/index.js:1:65756)\n' +
docker-calcom-1  | @calcom/web:start:       '    at ClientRequest.emit (node:events:513:28)\n' +
docker-calcom-1  | @calcom/web:start:       '    at ClientRequest.emit (node:domain:489:12)\n' +
docker-calcom-1  | @calcom/web:start:       '    at Socket.socketErrorListener (node:_http_client:494:9)\n' +
docker-calcom-1  | @calcom/web:start:       '    at Socket.emit (node:events:513:28)\n' +
docker-calcom-1  | @calcom/web:start:       '    at Socket.emit (node:domain:489:12)\n' +
docker-calcom-1  | @calcom/web:start:       '    at emitErrorNT (node:internal/streams/destroy:157:8)\n' +
docker-calcom-1  | @calcom/web:start:       '    at emitErrorCloseNT (node:internal/streams/destroy:122:3)\n' +
docker-calcom-1  | @calcom/web:start:       '    at processTicksAndRejections (node:internal/process/task_queues:83:21)',
docker-calcom-1  | @calcom/web:start:     name: 'FetchError'
docker-calcom-1  | @calcom/web:start:   },
docker-calcom-1  | @calcom/web:start:   url: 'http://testing.localhost:3000/api/auth/session',
docker-calcom-1  | @calcom/web:start:   message: 'request to http://testing.localhost:3000/api/auth/session failed, reason: getaddrinfo ENOTFOUND testing.localhost'
docker-calcom-1  | @calcom/web:start: }
```

### Railway

[![Deploy on Railway](https://railway.app/button.svg)](https://railway.app/new/template/cal)

可在 [Railway](https://railway.app) 部署 Cal.diy。Railway 团队还提供了在其平台部署的[详细博客文章](https://blog.railway.app/p/calendso)。

### Northflank

[![Deploy on Northflank](https://assets.northflank.com/deploy_to_northflank_smm_36700fb050.svg)](https://northflank.com/stacks/deploy-calcom)

可在 [Northflank](https://northflank.com) 部署 Cal.diy。Northflank 团队还提供了在其平台部署的[详细博客文章](https://northflank.com/guides/deploy-calcom-with-northflank)。

### Vercel

由于免费计划对无服务器函数数量的限制，目前使用 Vercel 部署此应用需要 Vercel Pro 计划。

[![Deploy with Vercel](https://vercel.com/button)](https://vercel.com/new/clone?repository-url=https%3A%2F%2Fgithub.com%2Fcalcom%2Fcal.diy&env=DATABASE_URL,NEXT_PUBLIC_WEBAPP_URL,NEXTAUTH_URL,NEXTAUTH_SECRET,CRON_API_KEY,CALENDSO_ENCRYPTION_KEY&envDescription=See%20all%20available%20env%20vars&envLink=https%3A%2F%2Fgithub.com%2Fcalcom%2Fcal.diy%2Fblob%2Fmain%2F.env.example&project-name=cal&repo-name=cal.diy&build-command=cd%20../..%20%26%26%20yarn%20build&root-directory=apps%2Fweb%2F)

### Render

[![Deploy to Render](https://render.com/images/deploy-to-render-button.svg)](https://render.com/deploy?repo=https://github.com/calcom/docker)

### Elestio

[![Deploy on Elestio](https://elest.io/images/logos/deploy-to-elestio-btn.png)](https://elest.io/open-source/cal.com)

<!-- LICENSE -->

## 许可

Cal.diy 完全开源，采用 [MIT 许可证](https://opensource.org/license/mit).

与 Cal.com 的“开放核心”模式不同，Cal.diy **没有商业／企业代码**，整个代码库均采用相同的开源许可证。

## 启用内容安全策略

- 设置环境变量 CSP_POLICY="non-strict"，启用 [Strict CSP](https://web.dev/strict-csp/)，但 `style-src` 中的 `unsafe-inline` 除外。如果实例有自定义修改，可能需要调整代码以兼容 CSP。目前严格 CSP 仅在登录页面启用；其他 SSR 页面以仅报告模式启用，以检测潜在问题；SSG 页面尚不支持。

<a id="integrations"></a>
## 集成

### 获取 Google API 凭据

1. 打开 [Google API Console](https://console.cloud.google.com/apis/dashboard)。如果 Google Cloud 订阅中没有项目，需先创建。然后在 Dashboard 面板选择 Enable APIS and Services。
2. 在搜索框输入 calendar，选择 Google Calendar API 搜索结果。
3. 启用所选 API。
4. 从侧栏进入 [OAuth consent screen](https://console.cloud.google.com/apis/credentials/consent)，选择应用类型（Internal 或 External），在第一页填写应用基本信息。
5. 在第二页 Scopes 选择 Add or Remove Scopes。搜索 Calendar.event，选择值为 `.../auth/calendar.events`、`.../auth/calendar.readonly` 的范围，然后选择 Update。
6. 在第三页（Test Users）添加将使用的 Google 账户。在向导最后一页确认信息正确，即可配置同意屏幕。
7. 从侧栏选择 [Credentials](https://console.cloud.google.com/apis/credentials)，然后选择 Create Credentials 和 OAuth Client ID。
8. 将 Application Type 设为 Web Application。
9. 在 Authorized redirect URI 下选择 Add URI，添加 `<Cal.diy URL>/api/integrations/googlecalendar/callback` 和 `<Cal.diy URL>/api/auth/callback/google`，将 Cal.diy URL 替换为应用实际运行的 URI。
10. 密钥创建后会返回 Credentials 页面。在 OAuth 2.0 Client IDs 下选择新生成的客户端 ID。
11. 选择 Download JSON。复制文件内容，将完整 JSON 字符串粘贴到 `.env` 中作为 `GOOGLE_API_CREDENTIALS` 的值。

#### _将 Google Calendar 添加到 Cal.diy App Store_

添加 Google 凭据后，可以将 Google Calendar 应用添加到 App Store。
可运行以下命令重新填充 App Store：

```
cd packages/prisma
yarn seed-app-store
```

要激活 Google Calendar 应用，还需完成几个步骤。确保已完成“获取 Google API 凭据”章节，然后：

1. 添加额外重定向 URL `<Cal.diy URL>/api/auth/callback/google`。
1. 在 “OAuth consent screen” 下点击 “PUBLISH APP”。

### 获取 Microsoft Graph 客户端 ID 和密钥

1. 打开 [Azure App Registration](https://portal.azure.com/#blade/Microsoft_AAD_IAM/ActiveDirectoryMenuBlade/RegisteredApps)，选择 New registration。
2. 为应用命名。
3. 将 **Who can use this application or access this API?** 设为 **Accounts in any organizational directory (Any Azure AD directory - Multitenant)**。
4. 将 **Web** 重定向 URI 设为 `<Cal.diy URL>/api/integrations/office365calendar/callback`，把 Cal.diy URL 替换为应用实际运行的 URI。
5. 将 **Application (client) ID** 用作 .env 中 **MS_GRAPH_CLIENT_ID** 的值。
6. 点击 **Certificates & secrets** 创建客户端密钥，将其值用作 **MS_GRAPH_CLIENT_SECRET**。

### 获取 Zoom 客户端 ID 和密钥

1. 打开 [Zoom Marketplace](https://marketplace.zoom.us/)，使用 Zoom 账户登录。
2. 在右上角点击 "Develop" => "Build App"。
3. 选择 "General App"，点击 "Create"。
4. 为应用命名。
5. 在 "Select how the app is managed" 选择 "User-managed app"。
6. 如果系统询问，请取消选择将应用发布到 Zoom App Marketplace。
7. 将 Client ID 和 Client Secret 复制到 `.env` 文件的 `ZOOM_CLIENT_ID` 和 `ZOOM_CLIENT_SECRET` 字段。
8. 将 "OAuth Information" 下的 "OAuth Redirect URL" 设为 `<Cal.diy URL>/api/integrations/zoomvideo/callback`，把 Cal.diy URL 替换为应用实际运行的 URI。
9. 将上述重定向 URL 加入允许列表并启用 "Subdomain check"，确认表单下方显示 "saved"。
10. 无需提供应用基本信息，点击 "Scopes" 然后点击 "+ Add Scopes"。在左侧：
    1. 点击 "Meeting" 类别，勾选范围 `meeting:write:meeting`。
    2. 点击 "User" 类别，勾选范围 `user:read:settings`。
11. 点击 "Done"。
12. 完成后即可在 Cal.diy 设置中添加 Zoom 集成。

### 获取 Daily API 凭据

1. 打开 [Daily.co](https://daily.co/) 并创建账户。
2. 在仪表板中进入 [developers](https://dashboard.daily.co/developers) 标签。
3. 复制 API 密钥。
4. 将 API 密钥粘贴到 `.env` 文件中的 `DAILY_API_KEY` 字段。
5. 如果使用 [Daily Scale Plan](https://daily.co/pricing)，将 `DAILY_SCALE_PLAN` 设为 `true`，以使用视频录制等功能。

### 获取 Basecamp 客户端 ID 和密钥

1. 访问 [37 Signals Integrations Dashboard](https://launchpad.37signals.com/integrations) 并登录。
2. 点击 Register one now 链接注册新应用。
3. 填写公司信息。
4. 选择 Basecamp 4 作为集成产品。
5. 将 OAuth Redirect URL 设为 `<Cal.diy URL>/api/integrations/basecamp3/callback`，把 Cal.diy URL 替换为应用实际运行的 URI。
6. 点击完成，将客户端 ID 和密钥复制到 `BASECAMP3_CLIENT_ID` 和 `BASECAMP3_CLIENT_SECRET` 字段。
7. 将环境变量 `BASECAMP3_CLIENT_SECRET` 设为 `{your_domain} ({support_email})`。

> 译注：上游第 6 和第 7 步对 `BASECAMP3_CLIENT_SECRET` 给出不同值，可能存在文档错误；此处保留原文，具体配置请核对上游代码或文档。

### 获取 HubSpot 客户端 ID 和密钥

1. 打开 [HubSpot Developer](https://developer.hubspot.com/) 并登录或创建账户。
2. 在开发者账户主页进入 "Manage apps"。
3. 点击右上角 "Create legacy app"，选择 public app。
4. 在 "App info" 标签中填写所需信息。
5. 进入 "Auth" 标签。
6. 将 Client ID 和 Client Secret 复制到 `.env` 文件中的 `HUBSPOT_CLIENT_ID` 和 `HUBSPOT_CLIENT_SECRET` 字段。
7. 将 OAuth Redirect URL 设为 `<Cal.diy URL>/api/integrations/hubspot/callback`，把 Cal.diy URL 替换为应用实际运行的 URI。
8. 在页面底部 "Scopes" 中，确保为 `crm.objects.contacts` 和 `crm.lists` 选择 "Read" 和 "Write"。
9. 点击页脚的 "Save" 按钮。
10. 完成后，Cal.diy 的预约可作为联系人的会议显示在 HubSpot 中。

### 获取 Webex 客户端 ID 和密钥

[参阅 Webex README](https://github.com/calcom/cal.diy/tree/81ce067c1d0e16b21c39c2de98b9a43a92562cd1/packages/app-store/webex/)

### 获取 ZohoCRM 客户端 ID 和密钥

1. 打开 [Zoho API Console](https://api-console.zoho.com/) 并登录或创建账户。
2. 在 API 控制台进入 "Applications"。
3. 点击右上角 "ADD CLIENT"，选择 "Server-based Applications"。
4. 在 "Client Details" 标签填写所需信息。
5. 进入 "Client Secret" 标签。
6. 将 Client ID 和 Client Secret 复制到 `.env` 中的 `ZOHOCRM_CLIENT_ID` 和 `ZOHOCRM_CLIENT_SECRET` 字段。
7. 将 OAuth Redirect URL 设为 `<Cal.diy URL>/api/integrations/zohocrm/callback`，把 Cal.diy URL 替换为应用实际运行的 URI。
8. 如果希望所有数据中心使用相同 OAuth 凭据，在 "Settings" 中勾选 "Multi-DC"。
9. 点击页脚的 "Save"／"UPDATE" 按钮。
10. 完成后即可在 Cal.diy 设置中添加 ZohoCRM 集成。

### 获取 Zoho Calendar 客户端 ID 和密钥

[按照以下步骤操作](https://github.com/calcom/cal.diy/tree/81ce067c1d0e16b21c39c2de98b9a43a92562cd1/packages/app-store/zohocalendar/)

### 获取 Zoho Bigin 客户端 ID 和密钥

[按照以下步骤操作](https://github.com/calcom/cal.diy/tree/81ce067c1d0e16b21c39c2de98b9a43a92562cd1/packages/app-store/zoho-bigin/)

### 获取 Pipedrive 客户端 ID 和密钥

[按照以下步骤操作](https://github.com/calcom/cal.diy/tree/81ce067c1d0e16b21c39c2de98b9a43a92562cd1/packages/app-store/pipedrive-crm/)

### 使用 Unkey 限流

Cal.diy 使用 [Unkey](https://unkey.com) 限流。这是可选功能，自托管不要求启用。

如果要启用限流：

1. 在 [unkey.com](https://unkey.com) 注册账户。
2. 创建具备以下权限的 Root key：
   `ratelimit.create_namespace` 和 `ratelimit.limit`
3. 将 root key 复制到 `.env` 文件的 `UNKEY_ROOT_KEY` 字段。

注意：如果不配置 Unkey，Cal.diy 仍会正常运行，但不会启用限流。

## 贡献

欢迎贡献！无论是修复拼写错误、改进文档还是开发新功能，你的帮助都会让 Cal.diy 变得更好。

> **重要：** Cal.diy 是社区分叉。向此仓库提交的贡献**不会**进入 Cal.com 的生产平台。详情参阅 [CONTRIBUTING.md](https://github.com/calcom/cal.diy/blob/81ce067c1d0e16b21c39c2de98b9a43a92562cd1/CONTRIBUTING.md)。

- 查阅[贡献指南](https://github.com/calcom/cal.diy/blob/81ce067c1d0e16b21c39c2de98b9a43a92562cd1/CONTRIBUTING.md)了解详细步骤。
- 请遵循编码标准和提交信息约定，以保持项目一致。

即使小改进也很重要，感谢你的帮助！

### 适合新手的问题

我们有一份 [help wanted](https://github.com/calcom/cal.diy/issues?q=is:issue+is:open+label:%22%F0%9F%99%8B%F0%9F%8F%BB%E2%80%8D%E2%99%82%EF%B8%8Fhelp+wanted%22)列表，包含范围相对有限的小功能和错误，适合入门、积累经验并熟悉贡献流程。

<!-- CONTRIBUTORS -->

### 贡献者

<a href="https://github.com/calcom/cal.diy/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=calcom/cal.diy" />
</a>

<!-- TRANSLATIONS -->

### 翻译

不会编程但仍想贡献？欢迎帮助将 Cal.diy 翻译成你的语言。

<!-- ACKNOWLEDGEMENTS -->

## 致谢

Cal.diy 建立在 [Cal.com](https://cal.com) 及原项目众多贡献者创建的基础之上。特别感谢：

- [Vercel](https://vercel.com/)
- [Next.js](https://nextjs.org/)
- [Day.js](https://day.js.org/)
- [Tailwind CSS](https://tailwindcss.com/)
- [Prisma](https://prisma.io/)
