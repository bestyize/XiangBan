<p align="center">
  <a href="https://strapi.io/#gh-light-mode-only">
    <img src="https://raw.githubusercontent.com/strapi/strapi/73032435344ce40b4d756745dd5cbbbe24039911/packages/core/admin/admin/src/assets/images/logo-strapi-2022.svg" width="318px" alt="Strapi logo" />
  </a>
  <a href="https://strapi.io/#gh-dark-mode-only">
    <img src="https://raw.githubusercontent.com/strapi/strapi/73032435344ce40b4d756745dd5cbbbe24039911/packages/core/admin/admin/src/assets/images/logo-strapi-2022.svg" width="318px" alt="Strapi logo" />
  </a>
</p>

<h3 align="center">开源无头 CMS，自托管或使用云端，由你掌控。</h3>
<p align="center">开源无头 CMS，100% 使用 JavaScript/TypeScript，灵活且可完全自定义。</p>
<p align="center"><a href="https://docs.strapi.io">文档</a> · <a href="https://strapi.io/cloud">Strapi Cloud</a> · <a href="https://feedback.strapi.io">路线图</a> · <a href="https://discord.strapi.io">Discord</a> · <a href="https://github.com/strapi/strapi/discussions">讨论</a></p>

<p align="center">
  <a href="https://www.npmjs.org/package/@strapi/strapi">
    <img src="https://img.shields.io/npm/v/@strapi/strapi/latest.svg" alt="NPM Version" />
  </a>
  <a href="https://github.com/strapi/strapi/actions/workflows/tests.yml">
    <img src="https://github.com/strapi/strapi/actions/workflows/tests.yml/badge.svg?branch=main" alt="Tests" />
  </a>
  <a href="https://discord.strapi.io">
    <img src="https://img.shields.io/discord/811989166782021633?label=Discord" alt="Strapi on Discord" />
  </a>
  <a href="https://github.com/strapi/strapi/actions/workflows/nightly.yml">
    <img src="https://github.com/strapi/strapi/actions/workflows/nightly.yml/badge.svg" alt="Strapi Nightly Release Build Status" />
  </a>
</p>

<br>

Strapi 是一个开源、自托管的无头 CMS，帮助开发者快速构建内容 API，同时为内容创建者提供友好的编辑界面。定义内容模型后，Strapi 会生成完整 API，可供前端、移动应用或物联网设备调用。

- 使用[**内容类型构建器**](https://docs.strapi.io/cms/features/content-type-builder)可视化设计内容结构，无需编写代码
- 为每种内容类型自动生成 [**REST**](https://docs.strapi.io/cms/api/rest) 和 [**GraphQL**](https://docs.strapi.io/cms/plugins/graphql) API
- 开箱即用的细粒度[**角色与权限**](https://docs.strapi.io/cms/features/users-permissions)
- 内置[**媒体库**](https://docs.strapi.io/cms/features/media-library)、[**国际化（i18n）**](https://docs.strapi.io/cms/features/internationalization)和[**草稿与发布**](https://docs.strapi.io/cms/features/draft-and-publish)
- 全面支持 **TypeScript**，提供灵活的数据库选项（SQLite、PostgreSQL、MySQL、MariaDB）
- 可扩展的[**插件系统**](https://docs.strapi.io/cms/plugins-development/developing-plugins)和可自定义的管理界面

> 查看全部功能： **[strapi.io/features](https://strapi.io/features)**

> **Strapi AI** —— 使用 Strapi 内置 AI 层自动处理内容建模、媒体替代文本和翻译。[了解更多](https://strapi.io/ai)

### Strapi 如何处理请求

每个传入请求都会经过分层后端架构：**Routes → Middlewares → Controllers → Services**。

<p align="center">
  <img src="https://docs.strapi.io/img/assets/backend-customization/diagram-routes.png" alt="Strapi backend request flow: Routes, Middlewares, Controllers, and Services" />
</p>

> 有关后端自定义的更多信息，请参阅[官方文档](https://docs.strapi.io/cms/backend-customization)。

## 开始使用

<a href="https://docs.strapi.io/developer-docs/latest/getting-started/quick-start.html" target="_blank">阅读入门教程</a>，或按照以下步骤操作：

### ⏳ 安装

使用以下 **Quickstart** 命令快速创建一个新的 Strapi 项目：

```bash
npx create-strapi@latest my-project
```

此命令会生成一个包含默认功能的新项目（身份验证、权限、内容管理、内容类型构建器和文件上传）。

> 完整安装选项（包括 TypeScript、`--quickstart` 等）请参阅 **[CLI 安装文档](https://docs.strapi.io/cms/installation/cli#creating-a-strapi-project)**

### 系统要求

> 硬件与软件要求（操作系统、Node.js、数据库）请参阅 **[要求文档](https://docs.strapi.io/cms/deployment#hardware-and-software-requirements)**

## Docker

Strapi 不提供官方 Docker 镜像，因此需要基于自己的项目构建。可以通过社区 CLI 工具快速开始：

```bash
npx @strapi-community/dockerize@latest
```

该工具会生成适合项目的 `Dockerfile` 和 `docker-compose.yml`。更多详情请参阅 [strapi-tool-dockerize](https://github.com/strapi-community/strapi-tool-dockerize) 仓库。

> Dockerfile、Docker Compose 示例和生产构建请参阅 **[Docker 安装文档](https://docs.strapi.io/cms/installation/docker)**

## 部署到 Strapi Cloud

从本地进入生产环境的一种快速方式。Strapi Cloud 是官方托管平台，提供零 DevOps、内置数据库、媒体库和 CDN。

> [**立即部署**](https://cloud.strapi.io)

## LaunchPad

结合 Strapi 和 Next.js 的官方演示模板，帮助你快速上手。

> [**LaunchPad template**](https://github.com/strapi/LaunchPad)

### 试用在线演示

访问带有示例数据的[托管 Strapi 项目](https://strapi.io/demo)，了解其内部工作方式——[在线试用](https://strapi.io/demo)或[拉取到本地](https://github.com/strapi/LaunchPad)。

## 仓库

| 仓库                                                          | 说明                                   |
| ------------------------------------------------------------------- | --------------------------------------------- |
| [**strapi/strapi**](https://github.com/strapi/strapi)               | 核心单体仓库，即 CMS 本身                 |
| [**strapi/design-system**](https://github.com/strapi/design-system) | Strapi 设计系统，React 组件库 |
| [**strapi/LaunchPad**](https://github.com/strapi/LaunchPad)         | 演示应用：Strapi + Next.js                    |

## 贡献与社区

Strapi 由社区共同构建并开源。如需帮助、提供反馈或参与：

- **[Discord](https://discord.strapi.io)**：与社区和 Strapi 团队实时讨论
- **[GitHub Discussions](https://github.com/strapi/strapi/discussions)**：提问、分享项目并获取反馈
- **[GitHub](https://github.com/strapi/strapi)**：报告错误和参与贡献
- **[贡献指南](https://contributor.strapi.io/)**：如何贡献代码、文档和翻译
- **[社区内容](https://github.com/strapi/community-content)**：展示、教程、入门项目和模板
- **[反馈](https://feedback.strapi.io)**：路线图和功能请求
- **[Twitter](https://twitter.com/strapijs)**：快速获取消息
- **[YouTube 频道](https://www.youtube.com/strapi)**：通过视频教程学习

一般帮助请参阅 [Strapi 官方文档](https://docs.strapi.io)。初次接触开源？请查看[如何参与开源贡献](https://opensource.guide/)。

## 迁移

按照文档中的[迁移指南](https://docs.strapi.io/developer-docs/latest/update-migration-guides/migration-guides.html)保持项目更新。

## 路线图

查看[路线图](https://feedback.strapi.io)，了解已发布和即将推出的功能。你也可以提供意见并为特定功能投票。

## 文档

Strapi 文档请参阅专用[仓库](https://github.com/strapi/documentation)，或在线查看：

- [CMS 文档](https://docs.strapi.io/cms/intro)
- [云端文档](https://docs.strapi.io/cloud/intro)

## 安全

如果发现安全问题，请负责任地报告。有关披露流程和联系信息，请参阅[安全政策](https://github.com/strapi/strapi/blob/main/SECURITY.md)。

## 关于 Strapi

- **[公司手册](https://handbook.strapi.io)**：我们的价值观、工作方式等
- **[关于我们](https://strapi.io/about-us)**：Strapi 的故事

[联系我们](https://strapi.io/contact)

## 支持 Strapi

如果 Strapi 对你有用，欢迎点星支持。这比你想象的更有帮助！

## 感谢所有贡献者

感谢所有贡献代码、报告问题并帮助塑造 Strapi 的人。

[![Strapi contributors](https://contrib.rocks/image?repo=strapi/strapi&max=400&columns=20)](https://github.com/strapi/strapi/graphs/contributors)

## 许可证

许可信息请参阅 [LICENSE](https://github.com/strapi/strapi/blob/73032435344ce40b4d756745dd5cbbbe24039911/LICENSE) 文件。

---

[官网](https://strapi.io) · [文档](https://docs.strapi.io) · [博客](https://strapi.io/blog) · [Twitter](https://twitter.com/strapijs) · [LinkedIn](https://www.linkedin.com/company/strapi/)
