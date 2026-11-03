> 译注：文中的性能、安全性和“首个”等表述来自上游项目，尚未独立验证。原文两个一键部署短链及 G2、npm 网页本次检查返回 HTTP 403；可参阅 [Payload 官方部署文档](https://payloadcms.com/docs/production/deployment)及 [npm 官方注册表](https://registry.npmjs.org/payload)。原链接保留，一键部署可用性未确认。

<a href="https://payloadcms.com"><img width="100%" src="https://l4wlsi8vxy8hre4v.public.blob.vercel-storage.com/github-banner-new-logo.jpg" alt="Payload headless CMS Admin panel built with React" /></a>
<br />
<br />

<p align="left">
  <a href="https://github.com/payloadcms/payload/actions"><img alt="GitHub Workflow Status" src="https://img.shields.io/github/actions/workflow/status/payloadcms/payload/main.yml?style=flat-square"></a>
  &nbsp;
  <a href="https://discord.gg/payload"><img alt="Discord" src="https://img.shields.io/discord/967097582721572934?label=Discord&color=7289da&style=flat-square" /></a>
  &nbsp;
  <a href="https://www.npmjs.com/package/payload"><img alt="npm" src="https://img.shields.io/npm/dw/payload?style=flat-square" /></a>
  &nbsp;
  <a href="https://github.com/payloadcms/payload/graphs/contributors"><img alt="npm" src="https://img.shields.io/github/contributors-anon/payloadcms/payload?color=yellow&style=flat-square" /></a>
  &nbsp;
  <a href="https://www.npmjs.com/package/payload"><img alt="npm" src="https://img.shields.io/npm/v/payload?style=flat-square" /></a>
  &nbsp;
  <a href="https://twitter.com/payloadcms"><img src="https://img.shields.io/badge/follow-payloadcms-1DA1F2?logo=twitter&style=flat-square" alt="Payload Twitter" /></a>
</p>
<hr/>
<h4>
<a target="_blank" href="https://payloadcms.com/docs/getting-started/what-is-payload" rel="dofollow"><strong>浏览文档</strong></a>&nbsp;·&nbsp;<a target="_blank" href="https://payloadcms.com/community-help" rel="dofollow"><strong>社区帮助</strong></a>&nbsp;·&nbsp;<a target="_blank" href="https://github.com/payloadcms/payload/discussions/1539" rel="dofollow"><strong>路线图</strong></a>&nbsp;·&nbsp;<a target="_blank" href="https://www.g2.com/products/payload-cms/reviews#reviews" rel="dofollow"><strong>查看 G2 评价</strong></a>
</h4>
<hr/>

> [!IMPORTANT]
> 给仓库加星或关注它，以便跟踪项目进展。

Payload 自称是首个可直接安装到现有 `/app` 文件夹的 Next.js 原生 CMS，并称其开启了无头 CMS 的新时代。

<h3>相比普通 CMS 的优势</h3>
<ul>
   <li>既是应用框架，也是无头 CMS</li>
  <li>可部署在任何地方，包括在 Vercel 免费使用无服务器部署</li>
  <li>如有需要，可将前后端合并到同一个 <code>/app</code> 文件夹</li>
  <li>无需再注册一个 SaaS——Payload 是开源的</li>
  <li>在 React Server Components 中查询数据库</li>
  <li>管理界面与后端均可完全扩展</li>
  <li>不受供应商锁定</li>
  <li>不再需要接触古老的 WP 代码</li>
  <li>更快构建，不再遇到障碍</li>
</ul>

## 快速开始

开始使用 Payload 前，请确保已具备所有[所需软件](https://payloadcms.com/docs/getting-started/installation).

```text
pnpx create-payload-app@latest
```

**如果你是 Payload 新手，应从网站模板开始**（`pnpx create-payload-app@latest -t website`）。它展示了如何完成_各种操作_，包括自定义富文本块、按需重新验证、实时预览等。模板包含使用 Tailwind 构建的前端，所有内容都位于同一个 `/app` 文件夹中。

## 一键部署选项

可以通过 Vercel 和 Cloudflare 一键无服务器部署 Payload，获得所需组件而不必处理底层配置。

### 部署到 Cloudflare

完整独立——一键部署 Payload，并使用 **Workers**、用于上传的 **R2** 和用于全球复制数据库的 **D1**。

[![Deploy to Cloudflare](https://deploy.workers.cloudflare.com/button)](https://dub.sh/payload-cloudflare)

### 部署到 Vercel

在 Vercel 上一体化部署——一键部署 Payload，包含 **Next.js** 前端、**Neon** 数据库和用于媒体存储的 **Vercel Blob**。

[![Deploy with Vercel](https://vercel.com/button)](https://dub.sh/payload-vercel)

## 一键模板

使用现成模板快速开始下一个项目。这些模板是旨在帮助你快速进入市场的**可用于生产的端到端解决方案**。可以构建**网站**、**电商商店**、**博客**或**作品集**，并配有使用 **React Server Components** 和 **Tailwind** 构建的现代前端。

#### 🌐 [网站](https://github.com/payloadcms/payload/tree/be95e1a17acfa9979bd6c9d34565708a3c833e68/templates/website)

#### 🛍️ [电商](https://github.com/payloadcms/payload/tree/be95e1a17acfa9979bd6c9d34565708a3c833e68/templates/ecommerce) 🎉 _**新增**_ 🎉

我们持续向[**模板目录**](https://github.com/payloadcms/payload/tree/be95e1a17acfa9979bd6c9d34565708a3c833e68/templates).
添加更多模板。如果维护自己的模板，请为 GitHub 仓库添加 `payload-template` 主题，以便其他人发现。

**🔗 探索更多：**

- [官方模板](https://github.com/payloadcms/payload/tree/be95e1a17acfa9979bd6c9d34565708a3c833e68/templates)
- [社区模板](https://github.com/topics/payload-template)

## ✨ Payload 功能

- 完全免费且开源
- Next.js 原生，旨在运行于_你的_ `/app` 文件夹中
- 使用服务端组件扩展 Payload UI
- 在服务端组件中直接查询数据库，无需 REST / GraphQL
- 完全使用 TypeScript，为数据自动生成类型
- [开箱即用的认证](https://payloadcms.com/docs/authentication/overview)
- [版本与草稿](https://payloadcms.com/docs/versions/overview)
- [本地化](https://payloadcms.com/docs/configuration/localization)
- [基于区块的布局构建器](https://payloadcms.com/docs/fields/blocks)
- [可定制的 React 管理界面](https://payloadcms.com/docs/admin/overview)
- [Lexical 富文本编辑器](https://payloadcms.com/docs/fields/rich-text)
- [条件字段逻辑](https://payloadcms.com/docs/fields/overview#conditional-logic)
- 极细粒度的[访问控制](https://payloadcms.com/docs/access-control/overview)
- [文档和字段级钩子](https://payloadcms.com/docs/hooks/overview)覆盖 Payload 提供的每项操作
- 速度极快的 API
- 借助 HTTP-only Cookie、CSRF 防护等机制提供高安全性

<a target="_blank" href="https://github.com/payloadcms/payload/discussions"><strong>请求功能</strong></a>

## 🗒️ 文档

请查看 [Payload 网站](https://payloadcms.com/docs/getting-started/what-is-payload)，了解 Payload 所提供功能的深入文档。

从 v2 迁移到 v3？请参阅 [3.0 迁移指南](https://github.com/payloadcms/payload/blob/be95e1a17acfa9979bd6c9d34565708a3c833e68/docs/migration-guide/overview.mdx)了解如何操作。

## 🙋 贡献

如果想为此仓库做贡献，请遵循 [contributing.md](https://github.com/payloadcms/payload/blob/be95e1a17acfa9979bd6c9d34565708a3c833e68/CONTRIBUTING.md).

## 📚 示例

[示例目录](https://github.com/payloadcms/payload/tree/be95e1a17acfa9979bd6c9d34565708a3c833e68/examples)是学习通过不同方式配置 Payload 的资源；我们的博客和社交媒体中也有示例。

如果想运行示例，可以使用 `create-payload-app` 从示例创建项目：

```sh
npx create-payload-app --example example_name
```

可以在以下位置查看更多示例：

- [示例目录](https://github.com/payloadcms/payload/tree/be95e1a17acfa9979bd6c9d34565708a3c833e68/examples)
- [Payload 博客](https://payloadcms.com/blog)
- [Payload YouTube](https://www.youtube.com/@payloadcms)

## 🔌 插件

Payload 高度可扩展，允许安装或分发增加或移除功能的插件。既有官方支持的插件，也有社区支持的插件。如果维护自己的插件，可以考虑为 GitHub 仓库添加 `payload-plugin` 主题，方便他人发现。

- [官方插件](https://github.com/orgs/payloadcms/repositories?q=topic%3Apayload-plugin)
- [社区插件](https://github.com/topics/payload-plugin)

## 🚨 需要帮助？

GitHub Discussions 和 Discord 服务器中有大量讨论和资源。如果遇到困难，可能已经有人解决过类似问题。:point_down:

- [GitHub Discussions](https://github.com/payloadcms/payload/discussions)
- [GitHub 问题](https://github.com/payloadcms/payload/issues)
- [Discord](https://t.co/30APlsQUPB)
- [社区帮助](https://payloadcms.com/community-help)

## ⭐ 喜欢我们的工作？请给我们一颗星

## 👏 感谢所有贡献者

<img align="left" src="https://contributors-img.web.app/image?repo=payloadcms/payload"/>
