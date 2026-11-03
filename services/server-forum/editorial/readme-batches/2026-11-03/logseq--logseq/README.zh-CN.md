> 依据 [Logseq 固定版本 README](https://github.com/logseq/logseq/blob/e7e9c97c7fcb9c95d1bea156124a12bb20eea053/README.md) 翻译；仓库许可为 AGPL-3.0。上游数据库版、移动端和同步服务的测试阶段说明对应此版本，使用前请核对最新状态。

<p align="center"><a href="https://logseq.com"><img src="https://user-images.githubusercontent.com/25513724/220608753-f33db466-af72-4611-b603-411440c15ed0.png?sanatize=true" height="173" alt="Logseq 标志"/></a></p>
<h1 align="center">Logseq</h1>
<h4 align="center">以隐私为先的开源知识管理与协作平台</h4>
<div align="center"><a href="https://logseq.com">首页</a> | <a href="https://blog.logseq.com/">博客</a> | <a href="https://docs.logseq.com/">文档</a> | <a href="https://logseq.io/p/NX4mc_ggEV">路线图</a></div>
<br></br>
<p align="center"><a href="https://github.com/logseq/logseq/releases/latest/"><img src="https://img.shields.io/badge/Download_Logseq-100000?style=for-the-badge&logo=flatpak&logoColor=white&labelColor=002b36&color=85c8c8" alt="下载 Logseq"/></a></p>
<p align="center"><a href="https://discuss.logseq.com"><img src="https://img.shields.io/badge/forum-Logseq-blue.svg?&color=%2385c8c8&logo=discourse&style=for-the-badge" alt="论坛"></a> <a href="https://discord.gg/KpN4eHY"><img src="https://img.shields.io/discord/725182569297215569?color=%2385c8c8&label=Discord&logo=discord&style=for-the-badge" alt="Discord 聊天"></a> <a href="https://twitter.com/intent/follow?screen_name=logseq"><img src="https://img.shields.io/badge/twitter-%40logseq-blue.svg?&color=%2385c8c8&logo=twitter&style=for-the-badge" alt="关注 Twitter"></a></p>
<p align="center"><a href="https://github.com/logseq/logseq/graphs/contributors"><img src="https://img.shields.io/github/contributors/logseq/logseq?color=%2385c8c8&style=for-the-badge" alt="贡献者"/></a> <a href="#-backers"><img src="https://img.shields.io/opencollective/backers/logseq?color=%2385c8c8&style=for-the-badge" alt="支持者"/></a> <a href="#-sponsors"><img src="https://img.shields.io/opencollective/sponsors/logseq?color=%2385c8c8&style=for-the-badge" alt="赞助者"/></a> <a href="https://github.com/logseq/logseq/blob/master/LICENSE.md"><img src="https://img.shields.io/github/license/logseq/logseq?color=%2385c8c8&style=for-the-badge" alt="许可"/></a> <a href="https://github.com/logseq/logseq/releases"><img src="https://img.shields.io/github/v/release/logseq/logseq?color=%2385c8c8&style=for-the-badge" alt="最新发行版"/></a></p>

## 目录

* [🚀 数据库版本](#-database-version)
* [🤔 为什么选择 Logseq？](#-why-logseq)
* [👀 如何使用？](#-how-can-i-use-it)
* [📚 了解更多](#-learn-more)
* [🫶 支持 Logseq 开发](#-support-logseq-development)
* [💡 功能请求](#-feature-requests)
* [🔌 插件 API](#-plugin-api)
* [🌟 为 Logseq 做贡献](#-contributing-to-logseq)
  * [🛠️ 设置开发环境](#setting-up-a-development-environment)
* [✨ 灵感来源](#-inspiration)
* [🙏 致谢](#-thank-you)

<a id="-database-version"></a>
## 🚀 数据库版本

Logseq 的数据库版本（DB 版本）引入了 DB 图谱。[此页面](https://github.com/logseq/docs/blob/master/db-version.md)概述了 DB 图谱的主要功能。如果你是现有用户，请[查看 DB 版本带来的变化](https://github.com/logseq/docs/blob/master/db-version-changes.md)。DB 版本有自己的新移动应用（iOS 已有，Android 即将推出）！如需参加移动应用 alpha 测试，请[填写这份简短表格](https://forms.gle/nfefJv51jUuULbFB9)。DB 版本还有一种新的同步方式 RTC（实时协作）！你可以用它在多台设备之间同步图谱或与他人协作。如需参加 RTC alpha 测试，请[填写此表](https://forms.gle/YSyF4WfKPSDuwyjH6)。

DB 版本处于 beta 阶段，而新移动应用和 RTC 处于 alpha 阶段。这意味着**可能发生数据丢失**，因此建议使用[自动备份](https://github.com/logseq/docs/blob/master/db-version.md#automated-backup)或[定期备份 SQLite 数据库](https://github.com/logseq/docs/blob/master/db-version.md#graph-export)。建议创建专用的测试图谱，选择一个对你不重要的项目。

开始使用 DB 版本：

* 试用最新网页版：访问 https://app.logseq.com/。
* 试用最新桌面版：访问 https://github.com/logseq/logseq/releases/tag/nightly，下载适用于操作系统的构建产物。
* 从源代码构建以试用最新版：
    * 使用 `test/db` 获取稳定发行版。错误较少、更新较慢，更新频率为数天或数周。
    * 使用 `master` 获取开发中的最新更新。预计错误更多、变化更快，更新频率为数小时或数天。
* 报告错误请提交到 https://github.com/logseq/db-test/issues。
* 功能或增强请求请提交到 Discord 的 `#db-feedback` 频道。
* 讨论：
    * 一般话题：Discord 的 `#db-chat` 频道。
    * 同步与 RTC：Discord 的 `#sync-test` 频道。
    * 移动端：Discord 的 `#mobile-test` 频道。

<a id="-why-logseq"></a>
## 🤔 为什么选择 Logseq？

[Logseq](https://logseq.com) 是**知识管理**和**协作**平台，注重**隐私**、**持久性**与[**用户控制权**](https://www.gnu.org/philosophy/free-sw.en.html)。Logseq 提供一系列用于**知识管理**、**协作**、**PDF 批注**和**任务管理**的强大工具，支持包括 **Markdown** 和 **Org-mode** 在内的多种文件格式，并提供**多种功能**来整理和组织笔记。

除了核心功能，Logseq 还有不断壮大的**插件**与**主题**生态，支持多种工作流及**自定义**选项。**移动应用**也可使用桌面应用的大多数功能。无论你是学生、专业人士，还是重视清晰有序地管理想法和笔记的人，Logseq 都是希望提高效率和简化工作流程的人的一个优秀选择。

![logseq-demo](https://user-images.githubusercontent.com/25513724/221387376-4dc419c2-0d0a-460c-a920-2d211e78b456.gif)

<a href="https://github.com/logseq/logseq/releases/latest/"><img src="https://img.shields.io/badge/Download_Logseq-100000?style=for-the-badge&logo=flatpak&logoColor=white&labelColor=002b36&color=85c8c8" align="right" alt="下载 Logseq"/></a>

<a id="-how-can-i-use-it"></a>
## 👀 如何使用？

开始使用 Logseq，只需以下简单步骤：

1. [下载](https://github.com/logseq/logseq/releases/latest)最新版本的 Logseq。
2. 在设备上安装并启动 Logseq。
3. 开始写作 ✍️

完成！现在你可以使用 Logseq 简化工作流程、管理项目并跟进目标。玩得开心！🎉

**Linux 用户**：使用自动安装脚本以获得最佳体验。示例依次为下载安装脚本、安装指定版本，以及无需 root 的当前用户安装：

   ```bash
   # Download and run the installer
   curl -fsSL https://raw.githubusercontent.com/logseq/logseq/master/scripts/install-linux.sh | bash

   # Or install a specific version
   curl -fsSL https://raw.githubusercontent.com/logseq/logseq/master/scripts/install-linux.sh | bash -s -- 0.10.14

   # For user-specific installation (no root required)
   curl -fsSL https://raw.githubusercontent.com/logseq/logseq/master/scripts/install-linux.sh | bash -s -- --user
   ```

<a id="-learn-more"></a>
## 📚 了解更多

* 网站：[logseq.com](https://logseq.com)
* 文档：[docs.logseq.com](https://docs.logseq.com)
  * 常见问题页面：[Logseq 文档：FAQ](https://docs.logseq.com/#/page/faq)
* 博客：[blog.logseq.com](https://blog.logseq.com)
  * 最新动态请访问我们的[关于页面](https://blog.logseq.com/about)。
* 论坛：[discuss.logseq.com](https://discuss.logseq.com)——我们在此回答问题、讨论工作流程并分享技巧。
  * 论坛常见问题板块：[Logseq 论坛：FAQ](https://discuss.logseq.com/c/faq/6)
* [Awesome Logseq](https://github.com/logseq/awesome-logseq)——社区创建的精彩 Logseq 扩展和资源 <3
* Twitter：[@Logseq](https://twitter.com/logseq)
* Discord：[https://discord.com/invite/KpN4eHY](https://discord.com/invite/KpN4eHY)
  * [中文 Discord](https://discord.gg/xYqcrXWymg)

<a id="-support-logseq-development"></a>
## 🫶 支持 Logseq 开发

如果你觉得 Logseq 有用，并希望帮助项目持续发展，请考虑在 [Open Collective](https://opencollective.com/logseq) 支持贡献者。你的支持能让贡献者知道他们的努力受到认可，并鼓励他们继续出色的工作。每一份贡献，无论大小，都能帮助我们持续改进 Logseq。

<a id="-feature-requests"></a>
## 💡 功能请求

我们重视你对改进 Logseq 和让它更实用的意见。如果你有任何想法或功能请求，请在 [Logseq 论坛的功能请求](https://discuss.logseq.com/new-topic?category=feature-requests)板块分享。

你的反馈帮助我们理解用户需求，并确定对你最重要的功能的优先级。感谢你花时间分享想法。

感谢支持，期待听到你的创意！

<a id="-plugin-api"></a>
## 🔌 插件 API

Logseq 提供插件 API，让开发者创建自定义插件并扩展 Logseq 的功能。插件 API 文档位于 [plugins-doc.logseq.com](https://plugins-doc.logseq.com/)，其中包含开始插件开发所需的一切。

我们重视你对改进文档的反馈和建议，如有意见或问题请随时联系我们。你的意见帮助我们为用户和开发者提供更好的体验。

感谢使用 Logseq，期待看到你通过插件 API 创造的作品！

<a id="-contributing-to-logseq"></a>
## 🌟 为 Logseq 做贡献

开始为 Logseq 贡献之前，请阅读 [CONTRIBUTING.md](https://github.com/logseq/logseq/blob/e7e9c97c7fcb9c95d1bea156124a12bb20eea053/CONTRIBUTING.md)。你可以[贡献代码](https://github.com/logseq/logseq/blob/master/CONTRIBUTING.md#code-contributions)，也可以[通过其他方式贡献](https://github.com/logseq/logseq/blob/master/CONTRIBUTING.md#-how-can-i-help)。我们欢迎所有贡献，无论大小，并感谢你花时间帮助改进 Logseq。期待你的贡献 🚀

<a id="setting-up-a-development-environment"></a>
### 🛠️ 设置开发环境

如需为 Logseq Web 或桌面应用设置开发环境，macOS/Linux 用户请参阅 [Develop Logseq](https://github.com/logseq/logseq/blob/e7e9c97c7fcb9c95d1bea156124a12bb20eea053/docs/develop-logseq.md) 指南，Windows 用户请参阅 [Develop Logseq on Windows](https://github.com/logseq/logseq/blob/e7e9c97c7fcb9c95d1bea156124a12bb20eea053/docs/develop-logseq-on-windows.md) 指南。

除了这些指南，还可以在 [docs/](https://github.com/logseq/logseq/tree/e7e9c97c7fcb9c95d1bea156124a12bb20eea053/docs/) 文件夹中找到其他实用资源，如[翻译贡献指南](https://github.com/logseq/logseq/blob/e7e9c97c7fcb9c95d1bea156124a12bb20eea053/docs/contributing-to-translations.md)、[Docker Web 应用指南](https://github.com/logseq/logseq/blob/e7e9c97c7fcb9c95d1bea156124a12bb20eea053/docs/docker-web-app-guide.md)和[移动端开发指南](https://github.com/logseq/logseq/blob/e7e9c97c7fcb9c95d1bea156124a12bb20eea053/docs/develop-logseq-on-mobile.md)。

### 🧰 Logseq CLI（Node）

Logseq CLI 文档维护在 `docs/cli/logseq-cli.md`。

<a id="-inspiration"></a>
## ✨ 灵感来源

Logseq 的灵感来自若干独特的工具和项目，包括 [Roam Research](https://roamresearch.com/)、[Org Mode](https://orgmode.org/)、[TiddlyWiki](https://tiddlywiki.com/)、[Workflowy](https://workflowy.com/) 和 [Cuekeeper](https://github.com/talex5/cuekeeper)。

我们非常感谢这些项目的开发者和创作者，希望 Logseq 能继续在他们的创新想法上发展，并让更多人能够使用这些想法。

感谢所有给予我们灵感的人，期待看到 Logseq 社区用这一工具创造的作品！

以下项目也使 Logseq 的实现成为可能：

* [Clojure & ClojureScript](https://clojure.org/)——动态、函数式、通用编程语言。
* [DataScript](https://github.com/tonsky/datascript)——面向 Clojure、ClojureScript 和 JS 的不可变数据库和 Datalog 查询引擎。
* [OCaml](https://ocaml.org/) 与 [Angstrom](https://github.com/inhabitedtype/angstrom)——用于文档解析器 [mldoc](https://github.com/logseq/mldoc)。
* [isomorphic-git](https://isomorphic-git.org/)——适用于 NodeJS 和 Web 浏览器的纯 JavaScript Git 实现。
* [SCI](https://github.com/borkdude/sci)——小型 Clojure 解释器。

<a id="-thank-you"></a>
# 🙏 致谢

我们衷心感谢 [Open Collective](https://opencollective.com/logseq) 的**赞助者**、**支持者**和**贡献者**。你们的支持和贡献让我们能够继续开发并改进 Logseq。感谢你们成为社区的一员，帮助我们尽可能做好 Logseq！

<a id="-sponsors"></a>
## 💎 赞助者

<p align="center"><a href="https://opencollective.com/logseq#sponsor">[成为赞助者]</a></p>
<p align="center"><a href="https://opencollective.com/logseq" alt="Open Collective 赞助者"><img src="https://opencollective.com/logseq/tiers/sponsors.svg?avatarHeight=42&width=600"/></a></p>

## 🌟 贡献者

<p align="center"><a href="https://github.com/logseq/logseq/graphs/contributors"><img src="https://contrib.rocks/image?repo=logseq/logseq&max=300&columns=14" width="600"/></a></p>

<p align="center"><a href="https://jetbrains.com" alt="JetBrains"><img src="https://raw.githubusercontent.com/logseq/logseq/e7e9c97c7fcb9c95d1bea156124a12bb20eea053/docs/assets/jetbrains.svg"/></a></p>

<p align="center"><a href="https://www.producthunt.com/posts/logseq?utm_source=badge-review&utm_medium=badge&utm_souce=badge-logseq#discussion-body" target="_blank"><img src="https://api.producthunt.com/widgets/embed-image/v1/review.svg?post_id=298158&theme=dark" align="center" alt="Logseq - 私密数字花园 | Product Hunt" style="width: 250px; height: 54px;" width="250" height="54"/></a></p>
