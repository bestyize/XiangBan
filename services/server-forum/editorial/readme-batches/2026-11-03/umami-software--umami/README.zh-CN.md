<p align="center">
  <img src="https://content.umami.is/website/images/umami-logo.png" alt="Umami Logo" width="100">
</p>

<h1 align="center">Umami</h1>

<p align="center">
  <i>Umami 是一个隐私优先的分析平台。它将流量、营销活动、用户行为、转化和收入集中在一起——无需 Cookie、无需监视，可自托管或使用云服务。</i>
</p>

<p align="center">
  <a href="https://github.com/umami-software/umami/releases"><img src="https://img.shields.io/github/release/umami-software/umami.svg" alt="GitHub Release" /></a>
  <a href="https://github.com/umami-software/umami/blob/559e3c607c23d2c72d9bd4f6481bda428255323c/LICENSE"><img src="https://img.shields.io/github/license/umami-software/umami.svg" alt="MIT License" /></a>
  <a href="https://github.com/umami-software/umami/actions"><img src="https://img.shields.io/github/actions/workflow/status/umami-software/umami/ci.yml" alt="Build Status" /></a>
  <a href="https://cloud.umami.is/share/LGazGOecbDtaIwDr/umami.is" style="text-decoration: none;"><img src="https://img.shields.io/badge/Try%20Demo%20Now-Click%20Here-brightgreen" alt="Umami Demo" /></a>
</p>

---

## 🚀 开始使用

详细入门指南请访问 [umami.is/docs](https://umami.is/docs/).

---

## 🛠 从源代码安装

### 要求

- 运行 Node.js 18.18 或更高版本的服务器。
- PostgreSQL 数据库 v12.14 或更高版本。

### 获取源代码并安装软件包

```bash
git clone https://github.com/umami-software/umami.git
cd umami
pnpm install
```

### 配置 Umami

创建包含以下内容的 `.env` 文件：

```bash
DATABASE_URL=connection-url
```

可选：设置 `API_URL` 以更改内部 UI API 调用使用的基础 URL。相对路径通过 `BASE_PATH` 提供服务；绝对 URL 通过本地 `/api` 路由代理。例如 `API_URL=/internal-api` 或 `API_URL=https://api.example.com/api`。

可选：将 `TWO_FACTOR_ENCRYPTION_KEY` 设置为 64 个字符的十六进制字符串以启用双因素认证，可使用 `openssl rand -hex 32` 生成。设置此密钥前，双因素认证不可用，也不能要求用户使用。

MCP 默认禁用。设置 `MCP_ENABLED=1` 启用 `/mcp` 端点，然后使用在“设置 → API 密钥”中生成的 API 密钥认证。

连接 URL 格式：

```bash
postgresql://username:mypassword@localhost:5432/mydb
```

### 构建应用

```bash
pnpm run build
```

首次安装时，构建步骤会在数据库中创建数据表，并创建用户名为 **admin**、密码为 **umami** 的登录用户。

### 启动应用

```bash
pnpm run start
```

默认情况下，应用将运行在 `http://localhost:3000`。你需要从 Web 服务器[代理](https://docs.nginx.com/nginx/admin-guide/web-server/reverse-proxy/)请求，或更改[端口](https://nextjs.org/docs/api-reference/cli#production)以直接提供应用服务。

---

## 🐳 使用 Docker 安装

Umami 提供 Docker 镜像和 Docker Compose 文件，方便部署。

Docker 镜像：

```bash
docker pull docker.umami.is/umami-software/umami:latest
```

Docker Compose（使用 PostgreSQL 数据库运行 Umami）：

```bash
docker compose up -d
```

---

## 🔄 获取更新

要获取最新功能，只需拉取更新、安装新依赖并重新构建：

```bash
git pull
pnpm install
pnpm build
```

要更新 Docker 镜像，只需拉取新镜像并重新构建：

```bash
docker compose pull
docker compose up --force-recreate -d
```

---

## 🛟 支持

<p align="center">
  <a href="https://github.com/umami-software/umami"><img src="https://img.shields.io/badge/GitHub--blue?style=social&logo=github" alt="GitHub" /></a>
  <a href="https://twitter.com/umami_software"><img src="https://img.shields.io/badge/Twitter--blue?style=social&logo=twitter" alt="Twitter" /></a>
  <a href="https://linkedin.com/company/umami-software"><img src="https://img.shields.io/badge/LinkedIn--blue?style=social&logo=linkedin" alt="LinkedIn" /></a>
  <a href="https://umami.is/discord"><img src="https://img.shields.io/badge/Discord--blue?style=social&logo=discord" alt="Discord" /></a>
</p>

[release-shield]: https://img.shields.io/github/release/umami-software/umami.svg
[releases-url]: https://github.com/umami-software/umami/releases
[license-shield]: https://img.shields.io/github/license/umami-software/umami.svg
[license-url]: https://github.com/umami-software/umami/blob/559e3c607c23d2c72d9bd4f6481bda428255323c/LICENSE
[build-shield]: https://img.shields.io/github/actions/workflow/status/umami-software/umami/ci.yml
[build-url]: https://github.com/umami-software/umami/actions
[github-shield]: https://img.shields.io/badge/GitHub--blue?style=social&logo=github
[github-url]: https://github.com/umami-software/umami
[twitter-shield]: https://img.shields.io/badge/Twitter--blue?style=social&logo=twitter
[twitter-url]: https://twitter.com/umami_software
[linkedin-shield]: https://img.shields.io/badge/LinkedIn--blue?style=social&logo=linkedin
[linkedin-url]: https://linkedin.com/company/umami-software
[discord-shield]: https://img.shields.io/badge/Discord--blue?style=social&logo=discord
[discord-url]: https://discord.com/invite/4dz4zcXYrQ
