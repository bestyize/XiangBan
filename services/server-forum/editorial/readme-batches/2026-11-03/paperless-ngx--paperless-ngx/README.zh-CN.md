> 译注：原文官方演示及文档网站在本次检查中返回 HTTP 403。可查看同一版本的 GitHub [文档首页](https://github.com/paperless-ngx/paperless-ngx/blob/cd58cffd89e2dd2cd5098f18da6ec659509c2a15/docs/index.md)、[安装指南](https://github.com/paperless-ngx/paperless-ngx/blob/cd58cffd89e2dd2cd5098f18da6ec659509c2a15/docs/setup.md)和[开发指南](https://github.com/paperless-ngx/paperless-ngx/blob/cd58cffd89e2dd2cd5098f18da6ec659509c2a15/docs/development.md)；演示站暂未确认可用，原链接保留。

[![ci](https://github.com/paperless-ngx/paperless-ngx/workflows/ci/badge.svg)](https://github.com/paperless-ngx/paperless-ngx/actions)
[![Crowdin](https://badges.crowdin.net/paperless-ngx/localized.svg)](https://crowdin.com/project/paperless-ngx)
[![Documentation Status](https://img.shields.io/github/deployments/paperless-ngx/paperless-ngx/github-pages?label=docs)](https://docs.paperless-ngx.com)
[![codecov](https://codecov.io/gh/paperless-ngx/paperless-ngx/branch/main/graph/badge.svg?token=VK6OUPJ3TY)](https://codecov.io/gh/paperless-ngx/paperless-ngx)
[![Chat on Matrix](https://matrix.to/img/matrix-badge.svg)](https://matrix.to/#/%23paperlessngx%3Amatrix.org)
[![demo](https://cronitor.io/badges/ve7ItY/production/W5E_B9jkelG9ZbDiNHUPQEVH3MY.svg)](https://demo.paperless-ngx.com)

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/paperless-ngx/paperless-ngx/cd58cffd89e2dd2cd5098f18da6ec659509c2a15/docs/assets/logo_full_white.png" width="50%">
    <source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/paperless-ngx/paperless-ngx/cd58cffd89e2dd2cd5098f18da6ec659509c2a15/docs/assets/logo_full_black.png" width="50%">
    <img src="https://raw.githubusercontent.com/paperless-ngx/paperless-ngx/cd58cffd89e2dd2cd5098f18da6ec659509c2a15/docs/assets/logo_full_black.png" width="50%">
  </picture>
</p>

<!-- omit in toc -->

# Paperless-ngx

Paperless-ngx 是一个文档管理系统，可将纸质文档转换为可搜索的在线档案，让你保存更少的纸张。

Paperless-ngx 是最初的 [Paperless](https://github.com/the-paperless-project/paperless)和 [Paperless-ng](https://github.com/jonaswinkler/paperless-ng)项目的官方继任者，旨在让一个团队共同承担推进和支持项目的责任。[欢迎考虑加入我们！](#社区支持)

感谢 [Fyra Stack](https://fyrastack.com) 慷慨提供支持，演示地址为 [demo.paperless-ngx.com](https://demo.paperless-ngx.com)，登录名和密码均为 `demo`。_注意：演示内容会频繁重置，请勿上传机密信息。_

- [功能](#功能)
- [开始使用](#开始使用)
- [贡献](#贡献)
  - [社区支持](#社区支持)
  - [翻译](#翻译)
  - [功能请求](#功能请求)
  - [错误](#错误)
- [相关项目](#相关项目)
- [重要说明](#重要说明)

<p align="right">演示托管由以下机构提供：<br/>
  <a href="https://fyrastack.com">
    <img src="https://raw.githubusercontent.com/paperless-ngx/paperless-ngx/cd58cffd89e2dd2cd5098f18da6ec659509c2a15/docs/assets/sponsors/fyra-stack.svg" alt="" width="28px" align="absmiddle"> Fyra Stack
  </a>
</p>

# 功能

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/paperless-ngx/paperless-ngx/cd58cffd89e2dd2cd5098f18da6ec659509c2a15/docs/assets/screenshots/documents-smallcards-dark.png">
  <source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/paperless-ngx/paperless-ngx/cd58cffd89e2dd2cd5098f18da6ec659509c2a15/docs/assets/screenshots/documents-smallcards.png">
  <img src="https://raw.githubusercontent.com/paperless-ngx/paperless-ngx/cd58cffd89e2dd2cd5098f18da6ec659509c2a15/docs/assets/screenshots/documents-smallcards.png">
</picture>

完整的[功能列表](https://docs.paperless-ngx.com/#features)和[截图](https://docs.paperless-ngx.com/#screenshots)可在[文档](https://docs.paperless-ngx.com/)中找到。

# 开始使用

部署 paperless 最简单的方式是使用 `docker compose`。[`/docker/compose` 目录](https://github.com/paperless-ngx/paperless-ngx/tree/cd58cffd89e2dd2cd5098f18da6ec659509c2a15/docker/compose)中的文件已配置为从 GitHub 容器注册表拉取镜像。

如果希望立即开始，可以使用安装脚本配置 `docker compose` 环境：

```bash
bash -c "$(curl -L https://raw.githubusercontent.com/paperless-ngx/paperless-ngx/main/install-paperless-ngx.sh)"
```

更多细节和其他安装方式的逐步指南请参阅[文档](https://docs.paperless-ngx.com/setup/#installation)。

从 Paperless-ng 迁移很容易，只需替换为新的 Docker 镜像！详情请参阅[迁移文档](https://docs.paperless-ngx.com/setup/#migrating-to-paperless-ngx)。

<!-- omit in toc -->

### 文档

Paperless-ngx 文档位于 [https://docs.paperless-ngx.com](https://docs.paperless-ngx.com/)。

# 贡献

如果想为项目做贡献，欢迎参与！我们欢迎错误修复、功能增强、视觉修复等。如果想实现大型改动，请先发起讨论！[文档](https://docs.paperless-ngx.com/development/)提供了入门的基本信息。

## 社区支持

欢迎有兴趣继续参与 paperless-ngx 的人通过 GitHub 和 [Matrix 聊天室](https://matrix.to/#/#paperless:matrix.org) 联系我们。如果想持续为项目做贡献，有多个[团队](https://github.com/orgs/paperless-ngx/people)（前端、CI/CD 等）需要帮助，请与我们联系！

## 翻译

Paperless-ngx 提供多种语言版本，翻译工作通过 Crowdin 协调。如果想帮助将 paperless-ngx 翻译成你的语言，请访问 https://crowdin.com/project/paperless-ngx，谢谢！更多详情参见 [CONTRIBUTING.md](https://github.com/paperless-ngx/paperless-ngx/blob/cd58cffd89e2dd2cd5098f18da6ec659509c2a15/CONTRIBUTING.md#translating-paperless-ngx)。

## 功能请求

可以通过 [GitHub Discussions](https://github.com/paperless-ngx/paperless-ngx/discussions/categories/feature-requests)提交功能请求；你可以搜索已有想法、添加自己的想法并为关注的内容投票。

## 错误

发现错误请[创建 issue](https://github.com/paperless-ngx/paperless-ngx/issues)；如有问题请[发起讨论](https://github.com/paperless-ngx/paperless-ngx/discussions)。

# 相关项目

请参阅 [Wiki](https://github.com/paperless-ngx/paperless-ngx/wiki/Related-Projects)，查看由用户维护的相关项目及兼容 Paperless-ngx 的软件列表。

# 重要说明

> 文档扫描仪通常用于扫描敏感文件，例如社会保险号码、税务记录和发票等。**绝不能在不受信任的主机上运行 Paperless-ngx**，因为信息以未加密的明文存储。项目不保证安全性（但我们确实会努力！），使用应用的风险由你自行承担。
> **运行 Paperless-ngx 最安全的方式是使用自己家中的本地服务器，并做好备份**。
