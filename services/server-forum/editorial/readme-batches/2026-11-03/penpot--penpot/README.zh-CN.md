> 译注：有关安全性、独占性等表述来自上游项目，尚未独立验证。原文 MCP 服务器介绍页及开发日志链接本次返回 404，可分别参阅 [Penpot 官方 MCP 文档](https://help.penpot.app/mcp/)和[官方博客](https://penpot.app/blog/)；Taiga 看板本次连接超时，原链接均保留。

<img width="100%" src="https://github.com/user-attachments/assets/da17b160-f289-436f-b140-972083a08602" />

[uri_license]: https://www.mozilla.org/en-US/MPL/2.0
[uri_license_image]: https://img.shields.io/badge/MPL-2.0-blue.svg

<p align="center">
  <a href="https://www.digitalpublicgoods.net/r/penpot" rel="nofollow">
    <img alt="Verified DPG" src="https://img.shields.io/badge/Verified-DPG-3333AB?logo=data:image/svg%2bxml;base64,PHN2ZyB3aWR0aD0iMzEiIGhlaWdodD0iMzMiIHZpZXdCb3g9IjAgMCAzMSAzMyIgZmlsbD0ibm9uZSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj4KPHBhdGggZD0iTTE0LjIwMDggMjEuMzY3OEwxMC4xNzM2IDE4LjAxMjRMMTEuNTIxOSAxNi40MDAzTDEzLjk5MjggMTguNDU5TDE5LjYyNjkgMTIuMjExMUwyMS4xOTA5IDEzLjYxNkwxNC4yMDA4IDIxLjM2NzhaTTI0LjYyNDEgOS4zNTEyN0wyNC44MDcxIDMuMDcyOTdMMTguODgxIDUuMTg2NjJMMTUuMzMxNCAtMi4zMzA4MmUtMDVMMTEuNzgyMSA1LjE4NjYyTDUuODU2MDEgMy4wNzI5N0w2LjAzOTA2IDkuMzUxMjdMMCAxMS4xMTc3TDMuODQ1MjEgMTYuMDg5NUwwIDIxLjA2MTJMNi4wMzkwNiAyMi44Mjc3TDUuODU2MDEgMjkuMTA2TDExLjc4MjEgMjYuOTkyM0wxNS4zMzE0IDMyLjE3OUwxOC44ODEgMjYuOTkyM0wyNC44MDcxIDI5LjEwNkwyNC42MjQxIDIyLjgyNzdMMzAuNjYzMSAyMS4wNjEyTDI2LjgxNzYgMTYuMDg5NUwzMC42NjMxIDExLjExNzdMMjQuNjI0MSA5LjM1MTI3WiIgZmlsbD0id2hpdGUiLz4KPC9zdmc+Cg==">
  </a>
  <a href="https://community.penpot.app" rel="nofollow">
    <img alt="Penpot Community" src="https://img.shields.io/discourse/posts?server=https%3A%2F%2Fcommunity.penpot.app">
  </a>
  <a href="https://tree.taiga.io/project/penpot/" rel="nofollow">
    <img alt="Managed with Taiga.io" src="https://img.shields.io/badge/managed%20with-TAIGA.io-709f14.svg">
  </a>
  <a href="https://gitpod.io/#https://github.com/penpot/penpot" rel="nofollow">
    <img alt="Gitpod ready-to-code" src="https://img.shields.io/badge/Gitpod-ready--to--code-blue?logo=gitpod">
  </a>
</p>

<p align="center">
  <a href="https://penpot.app/"><b>官网</b></a>  •
  <a href="https://help.penpot.app/user-guide/"><b>用户指南</b></a>  •
  <a href="https://penpot.app/learning-center"><b>学习中心</b></a>  •
  <a href="https://community.penpot.app/"><b>社区</b></a>
</p>
<p align="center">
  <a href="https://www.youtube.com/@Penpot"><b>Youtube</b></a>  •
  <a href="https://peertube.kaleidos.net/a/penpot_app/video-channels"><b>Peertube</b></a>  •
  <a href="https://www.linkedin.com/company/penpot/"><b>Linkedin</b></a>  •
  <a href="https://instagram.com/penpot.app"><b>Instagram</b></a>  •
  <a href="https://fosstodon.org/@penpot/"><b>Mastodon</b></a>  •
  <a href="https://bsky.app/profile/penpot.app"><b>Bluesky</b></a>  •
  <a href="https://twitter.com/penpotapp"><b>X</b></a>
</p>

[Penpot 视频](https://github.com/user-attachments/assets/7c67fd7c-04d3-4c9b-88ec-b6f5e23f8332)

Penpot 是面向大规模构建数字产品的团队的开源设计平台。

Penpot 的核心优势在于让你**完全掌控设计基础设施**。它基于开源构建并为[自托管](https://help.penpot.app/technical-guide/getting-started/)设计，让团队掌控设计环境，以满足严格的合规和治理要求。无论在**浏览器中使用还是部署到自己的服务器上**，Penpot 都**采用开放标准**，如 SVG、CSS、HTML 和 JSON。 

实时协作增强了这一基础，帮助团队扩展并让设计更贴近产品。开发者也能熟悉地使用 Penpot，因为设计以代码形式表达，便于直接转换并更快交付产品。 

原生[设计令牌](https://penpot.dev/collaboration/design-tokens)为设计与开发提供统一的信息来源，帮助保持一致性、改善协作并简化复杂设计系统的管理。

[MCP 服务器](https://penpot.app/penpot-mcp-server)进一步支持设计与代码之间的多向工作流。[开放 API](https://help.penpot.app/mcp/#quick-start)和插件系统使工作空间可编程，支持自动化、AI 驱动的工作流，以及与现有工具和系统集成。

借助 [CSS Grid 和 Flex 布局](https://help.penpot.app/user-guide/designing/flexible-layouts/)，团队可以设计从一开始就按真实代码方式运作的响应式界面。

这些功能共同使 Penpot 成为用于构建可扩展设计系统及完整集成产品开发流程的**全栈设计平台**。

如果你的组织正在扩展并需要额外支持，欢迎[联系我们](https://penpot.app/talk-to-us)。

## 目录 ##

- [为什么选择 Penpot](#为什么选择-penpot)
- [开始使用](#开始使用)
- [Penpot Enterprise](#penpot-enterprise)
- [社区](#社区)
- [贡献](#贡献)
- [资源](#资源)
- [许可证](#许可证)

## 为什么选择 Penpot ##

Penpot 通过基于代码的方法连接设计、代码与 AI 工作流，让开发者和 AI 可通过 MCP 服务器读取设计。这种方法借助设计令牌帮助团队交付实际设计的内容并大规模管理设计系统。作为自托管、开源且支持实时协作的平台，Penpot 提供灵活性、安全性和自主控制，避免供应商锁定。进一步了解[为什么选择 Penpot](https://penpot.app/why-penpot)。

### 插件系统 ###

[Penpot 插件](https://penpot.app/penpothub/plugins)让你扩展平台能力、灵活集成其他应用并设计自定义方案。

### 为开发者设计 ###

Penpot 同时面向设计师和开发者，旨在建立流畅的设计与代码流程。你可以选择实时协作，也可以独自工作。

### 检查模式 ###

使用现成代码，让工作流程更轻松快捷。检查选项卡可直接访问 SVG、CSS 和 HTML 代码。

### 集成 ###

Penpot 支持通过 webhook 和使用访问令牌调用的 API 与开发工具链[集成](https://penpot.app/integrations-api)。

### 构建设计系统：设计令牌、组件和变体 ###

Penpot 为关注代码的团队提供[设计系统](https://penpot.app/design/design-systems)：以原生设计令牌、组件和变体作为统一来源，在不同项目和平台构建可扩展、可复用且一致的 UI。

<img width="100%" alt="Penpot Design Systems" src="https://github.com/user-attachments/assets/cce75ad6-f783-473f-8803-da9eb8255fef">

## 开始使用 ##

项目称 Penpot 是唯一不受部署方式限制的设计与原型平台。你可以使用其 [SaaS](https://design.penpot.app)，也可以自行部署。

有关使用 Docker、Kubernetes、Elestio 等方式安装的信息，请参阅[官网](https://penpot.app/self-host)。

## Penpot Enterprise ##

[Penpot Enterprise](https://help.penpot.app/user-guide/account-teams/enterprise-plan/)是面向需要跨多个团队扩展设计工作，并需要高级治理、安全和管理功能的组织的付费方案。可以通过集中式**管理控制台**管理团队和访问权限，配置高级权限，并通过 **SSO 连接身份提供方**。该方案适用于云端和自托管环境，将企业控制能力与 Penpot 的开源基础和开放标准结合起来。

<img width="100%" height="1010" alt="2" src="https://github.com/user-attachments/assets/243e796e-a140-481a-b68f-b24be6a70e37" />

## 社区 ##

我们热爱开源软件社区，也热衷于贡献。如果你也一样，欢迎参与并[改进](https://community.penpot.app/c/help-us-improve-penpot/7) Penpot，分享设计、代码和想法！

想进一步参与？可以成为 [Penpot 大使](https://penpot.app/ambassador-program)，帮助发展所在地区的 Penpot 社区，同时为全球开放设计生态做贡献。

如果你需要帮助、有问题、想分享使用 Penpot 的经验、寻找灵感，或希望认识开发者与设计师，请[加入我们的社区](https://community.penpot.app/)！

分类包括：

- [向社区求助](https://community.penpot.app/c/ask-for-help-using-penpot/6)
- [故障排除](https://community.penpot.app/c/technical/8)
- [帮助改进 Penpot](https://community.penpot.app/c/help-us-improve-penpot/7)
- [活动与公告](https://community.penpot.app/c/announcements/5)
- [使用你的语言讨论 Penpot](https://community.penpot.app/c/penpot-in-your-language/12)
- [教育](https://community.penpot.app/c/education/28)

<img width="100%" alt="Pentpot Community" src="https://github.com/user-attachments/assets/4b2a4360-12b5-4994-bd45-641449f86c4e" />

### 行为准则 ###

任何以代码、社区或活动形式为 Penpot 做贡献的人都必须遵守[行为准则](https://help.penpot.app/contributing-guide/coc/)，共同维护积极、安全的环境。

### 贡献 ###

任何贡献都能帮助改进 Penpot。如何参与？

请选择参与方式：

- 创建并[分享对社区有帮助的库和模板](https://penpot.app/libraries-templates.html)。
- 邀请你的[团队加入](https://design.penpot.app/#/auth/register)。
- 给此仓库加星，并在社交媒体关注我们： [Mastodon](https://fosstodon.org/@penpot/), [Youtube](https://www.youtube.com/c/Penpot), [Instagram](https://instagram.com/penpot.app), [Linkedin](https://www.linkedin.com/company/penpotdesign),  [Peertube](https://peertube.kaleidos.net/a/penpot_app), [X](https://twitter.com/penpotapp) and [BlueSky](https://bsky.app/profile/penpot.app).
- 在[社区](https://community.penpot.app/)提问和回答问题、回应他人文章、发起讨论，并关注影响项目的决定。
- 根据[错误报告指南](https://help.penpot.app/contributing-guide/reporting-bugs/)或通过 [GitHub issues](https://github.com/penpot/penpot/issues)报告错误。
- 成为[翻译者](https://help.penpot.app/contributing-guide/translations)。
- 提供反馈：[发送邮件](mailto:support@penpot.app)。
- **贡献 Penpot 代码：** [观看视频](https://www.youtube.com/watch?v=TpN0osiY-8k)，Penpot 的 CIO 和开发者 Alejandro Alonso 会实际演示如何使用仓库并修改前后端。

有关如何贡献 Penpot 的更多信息，请参阅[贡献指南](https://help.penpot.app/contributing-guide/)。

<img width="100%" alt="Penpot hub" src="https://github.com/user-attachments/assets/0abc02f0-625c-45ab-ad81-4927bec7a055" />

## 资源 ##

你可以提问和回答问题、进行开放讨论，并关注影响项目的决定。

💾 [文档](https://help.penpot.app/technical-guide/)

🚀 [开始使用](https://help.penpot.app/technical-guide/getting-started/)

✏️ [教程](https://www.youtube.com/playlist?list=PLgcCPfOv5v54WpXhHmNO7T-YC7AE-SRsr)

🏘️ [架构](https://help.penpot.app/technical-guide/developer/architecture/)

📚 [开发日志](https://penpot.app/dev-diaries.html)

🧑‍🏫​ [UI 设计课程](https://penpot.app/courses/)


## 许可证 ##

```
This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
file, You can obtain one at http://mozilla.org/MPL/2.0/.

Copyright (c) KALEIDOS SUBSIDIARY SL
```
Penpot 是 Kaleidos 的[开源项目](https://kaleidos.net/)
