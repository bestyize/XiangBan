# Gitea

[![](https://github.com/go-gitea/gitea/actions/workflows/release-nightly.yml/badge.svg?branch=main)](https://github.com/go-gitea/gitea/actions/workflows/release-nightly.yml?query=branch%3Amain "Release Nightly")
[![](https://img.shields.io/discord/322538954119184384.svg?logo=discord&logoColor=white&label=Discord&color=5865F2)](https://discord.gg/Gitea "Join the Discord chat at https://discord.gg/Gitea")
[![](https://pkg.go.dev/badge/gitea.dev?status.svg)](https://pkg.go.dev/gitea.dev "GoDoc")
[![](https://img.shields.io/github/release/go-gitea/gitea.svg)](https://github.com/go-gitea/gitea/releases/latest "GitHub release")
[![](https://www.codetriage.com/go-gitea/gitea/badges/users.svg)](https://www.codetriage.com/go-gitea/gitea "Help Contribute to Open Source")
[![](https://opencollective.com/gitea/tiers/backers/badge.svg?label=backers&color=brightgreen)](https://opencollective.com/gitea "Become a backer/sponsor of gitea")
[![](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT "License: MIT")
[![](https://badges.crowdin.net/gitea/localized.svg)](https://translate.gitea.com "Crowdin")

[English](https://github.com/go-gitea/gitea/blob/12b20c7c9962e0be75912b290a0c5778799b0ad2/README.md) | [繁體中文](https://github.com/go-gitea/gitea/blob/12b20c7c9962e0be75912b290a0c5778799b0ad2/README.zh-tw.md)

## 目的

Gitea 的目标是提供最简单、最快捷、省心的方式来建立一套自托管的一体化软件开发服务，包括 Git 托管、代码管理、代码审查、问题追踪、项目看板、Wiki、团队协作、软件包注册表以及可以复用 GitHub Actions 的 CI/CD。

Gitea 使用 Go 编写，因此可以在 Go 支持的 **所有** 平台和架构上运行，包括 Linux、macOS、FreeBSD/OpenBSD 和 Windows，以及 x86、amd64、ARM、RISC-V 64 和 PowerPC 架构。

在线演示可以访问 [demo.gitea.com](https://demo.gitea.com)。

要访问免费的 Gitea 服务（仓库数量有限），可以访问 [gitea.com](https://gitea.com/user/login)。

要在 Gitea Cloud 上快速部署自己的专用 Gitea 实例，可以在 [cloud.gitea.com](https://cloud.gitea.com) 开始免费试用；也可以使用[官方镜像](https://hub.docker.com/r/gitea/gitea)通过容器（Docker、Podman 等）部署到自己的服务器。

## 文件

您可以在我们的官方 [文件网站](https://docs.gitea.com/) 上找到全面的文件。

它包括安装、管理、使用、开发、贡献指南等，帮助您快速入门并有效地探索所有功能。

如果您有任何建议或想要贡献，可以访问 [文件仓库](https://gitea.com/gitea/docs)

## 构建

请参阅 [docs/build-setup.md](https://github.com/go-gitea/gitea/blob/12b20c7c9962e0be75912b290a0c5778799b0ad2/docs/build-setup.md) 了解前置条件，并参阅 [docs/development.md](https://github.com/go-gitea/gitea/blob/12b20c7c9962e0be75912b290a0c5778799b0ad2/docs/development.md) 了解如何配置本地开发环境、执行 lint 和测试。

如果要从源代码构建或制作发行包，请参阅 [docs/build-source.md](https://github.com/go-gitea/gitea/blob/12b20c7c9962e0be75912b290a0c5778799b0ad2/docs/build-source.md)。

构建完成后，运行 `./gitea web` 启动服务器，或运行 `./gitea help` 查看全部可用命令。

## 贡献

预期的工作流程是：Fork -> Patch -> Push -> Pull Request

> [!NOTE]
>
> 1. **在开始处理 Pull Request 之前，必须阅读[贡献者指南](https://github.com/go-gitea/gitea/blob/12b20c7c9962e0be75912b290a0c5778799b0ad2/CONTRIBUTING.md)。**
> 2. 不熟悉代码库？[开发指南](https://github.com/go-gitea/gitea/blob/12b20c7c9962e0be75912b290a0c5778799b0ad2/docs/development.md)介绍了本地环境配置和从源代码构建。
> 3. 如果发现项目漏洞，请私下发送邮件至 **security@gitea.io**。谢谢！

## 翻译

[![Crowdin](https://badges.crowdin.net/gitea/localized.svg)](https://translate.gitea.com)

翻译通过 [Crowdin](https://translate.gitea.com) 进行。如果想翻译成新的语言，请请求 Crowdin 项目中的一位管理员添加该语言。

也可以创建 issue 请求添加语言，或在 Discord 的 #translation 频道询问。如果需要上下文或发现翻译问题，可以在字符串上留言或在 Discord 上询问。

更多信息请参阅[贡献指南的翻译章节](https://github.com/go-gitea/gitea/blob/12b20c7c9962e0be75912b290a0c5778799b0ad2/CONTRIBUTING.md#translation)。

## 官方和第三方项目

我们提供了一个官方的 [go-sdk](https://gitea.com/gitea/go-sdk)，一个名为 [gt](https://gitea.com/gitea/cli) 的 CLI 工具和一个 Gitea Action 的 [action runner](https://gitea.com/gitea/runner)。

我们在 [gitea/awesome-gitea](https://gitea.com/gitea/awesome-gitea) 维护了一个 Gitea 相关项目的列表，您可以在那里发现更多的第三方项目，包括 SDK、插件、主题等。

## 通讯

[![](https://img.shields.io/discord/322538954119184384.svg?logo=discord&logoColor=white&label=Discord&color=5865F2)](https://discord.gg/Gitea "Join the Discord chat at https://discord.gg/Gitea")

如果您有任何文件未涵盖的问题，可以在我们的 [Discord 服务器](https://discord.gg/Gitea) 上与我们联系，或者在 [discourse 论坛](https://forum.gitea.com/) 上创建帖子。

## 作者

- [维护者](https://github.com/orgs/go-gitea/people)
- [贡献者](https://github.com/go-gitea/gitea/graphs/contributors)
- [翻译者](https://github.com/go-gitea/gitea/blob/12b20c7c9962e0be75912b290a0c5778799b0ad2/options/locale/TRANSLATORS)

## 支持者

感谢所有支持者！ 🙏 [[成为支持者](https://opencollective.com/gitea#backer)]

<a href="https://opencollective.com/gitea#backers" target="_blank"><img src="https://opencollective.com/gitea/backers.svg?width=890"></a>

## 赞助商

通过成为赞助商来支持这个项目。您的标志将显示在这里，并带有链接到您的网站。 [[成为赞助商](https://opencollective.com/gitea#sponsor)]

<a href="https://opencollective.com/gitea/sponsor/0/website" target="_blank"><img src="https://opencollective.com/gitea/sponsor/0/avatar.svg"></a>
<a href="https://opencollective.com/gitea/sponsor/1/website" target="_blank"><img src="https://opencollective.com/gitea/sponsor/1/avatar.svg"></a>
<a href="https://opencollective.com/gitea/sponsor/2/website" target="_blank"><img src="https://opencollective.com/gitea/sponsor/2/avatar.svg"></a>
<a href="https://opencollective.com/gitea/sponsor/3/website" target="_blank"><img src="https://opencollective.com/gitea/sponsor/3/avatar.svg"></a>
<a href="https://opencollective.com/gitea/sponsor/4/website" target="_blank"><img src="https://opencollective.com/gitea/sponsor/4/avatar.svg"></a>
<a href="https://opencollective.com/gitea/sponsor/5/website" target="_blank"><img src="https://opencollective.com/gitea/sponsor/5/avatar.svg"></a>
<a href="https://opencollective.com/gitea/sponsor/6/website" target="_blank"><img src="https://opencollective.com/gitea/sponsor/6/avatar.svg"></a>
<a href="https://opencollective.com/gitea/sponsor/7/website" target="_blank"><img src="https://opencollective.com/gitea/sponsor/7/avatar.svg"></a>
<a href="https://opencollective.com/gitea/sponsor/8/website" target="_blank"><img src="https://opencollective.com/gitea/sponsor/8/avatar.svg"></a>
<a href="https://opencollective.com/gitea/sponsor/9/website" target="_blank"><img src="https://opencollective.com/gitea/sponsor/9/avatar.svg"></a>

## 常见问题

**Gitea 怎么发音？**

Gitea 的发音是 [/ɡɪ’ti:/](https://youtu.be/EM71-2uDAoY)，如同 “gi-tea”，g 是硬音。

**如何配置 Gitea？**

动态配置项可以在管理面板的配置部分修改。

静态配置项可以编辑 `app.ini` 文件并重启实例。更多信息请参阅 [app.example.ini](https://github.com/go-gitea/gitea/blob/main/custom/conf/app.example.ini) 或[配置文档](https://docs.gitea.com/administration/config-cheat-sheet)。

**在哪里可以找到安全补丁？**

请查看[发布说明](https://github.com/go-gitea/gitea/releases)和[安全公告](https://github.com/go-gitea/gitea/security/advisories)。

（更多问题请参阅[常见问题文档](https://docs.gitea.com/help/faq)。）

## 许可证

这个项目是根据 MIT 许可证授权的。
请参阅 [LICENSE](https://github.com/go-gitea/gitea/blob/main/LICENSE) 文件以获取完整的许可证文本。

## 进一步信息

<details>
<summary>寻找界面概述？查看这里！</summary>

### 登录/注册页面

![Login](https://dl.gitea.com/screenshots/login.png)
![Register](https://dl.gitea.com/screenshots/register.png)

### 用户仪表板

![Home](https://dl.gitea.com/screenshots/home.png)
![Issues](https://dl.gitea.com/screenshots/issues.png)
![Pull Requests](https://dl.gitea.com/screenshots/pull_requests.png)
![Milestones](https://dl.gitea.com/screenshots/milestones.png)

### 用户资料

![Profile](https://dl.gitea.com/screenshots/user_profile.png)

### 探索

![Repos](https://dl.gitea.com/screenshots/explore_repos.png)
![Users](https://dl.gitea.com/screenshots/explore_users.png)
![Orgs](https://dl.gitea.com/screenshots/explore_orgs.png)

### 仓库

![Home](https://dl.gitea.com/screenshots/repo_home.png)
![Commits](https://dl.gitea.com/screenshots/repo_commits.png)
![Branches](https://dl.gitea.com/screenshots/repo_branches.png)
![Labels](https://dl.gitea.com/screenshots/repo_labels.png)
![Milestones](https://dl.gitea.com/screenshots/repo_milestones.png)
![Releases](https://dl.gitea.com/screenshots/repo_releases.png)
![Tags](https://dl.gitea.com/screenshots/repo_tags.png)

#### 仓库问题

![List](https://dl.gitea.com/screenshots/repo_issues.png)
![Issue](https://dl.gitea.com/screenshots/repo_issue.png)

#### 仓库拉取请求

![List](https://dl.gitea.com/screenshots/repo_pull_requests.png)
![Pull Request](https://dl.gitea.com/screenshots/repo_pull_request.png)
![File](https://dl.gitea.com/screenshots/repo_pull_request_file.png)
![Commits](https://dl.gitea.com/screenshots/repo_pull_request_commits.png)

#### 仓库操作

![List](https://dl.gitea.com/screenshots/repo_actions.png)
![Details](https://dl.gitea.com/screenshots/repo_actions_run.png)

#### 仓库活动

![Activity](https://dl.gitea.com/screenshots/repo_activity.png)
![Contributors](https://dl.gitea.com/screenshots/repo_contributors.png)
![Code Frequency](https://dl.gitea.com/screenshots/repo_code_frequency.png)
![Recent Commits](https://dl.gitea.com/screenshots/repo_recent_commits.png)

### 组织

![Home](https://dl.gitea.com/screenshots/org_home.png)

</details>
