> 本文根据 [HedgeDoc 项目 README](https://github.com/hedgedoc/hedgedoc/blob/f2ff89befccc2686b2bdf266af604b7de5a0c619/README.md) 翻译，并修改相对链接；原文作者为 [HedgeDoc 贡献者](https://github.com/hedgedoc/hedgedoc/blob/f2ff89befccc2686b2bdf266af604b7de5a0c619/AUTHORS)。译文按 [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/) 许可提供。HedgeDoc 标志另有使用条款，见下文。

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/hedgedoc/hedgedoc/f2ff89befccc2686b2bdf266af604b7de5a0c619/docs/content/images/hedgedoc_logo_white.svg">
    <source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/hedgedoc/hedgedoc/f2ff89befccc2686b2bdf266af604b7de5a0c619/docs/content/images/hedgedoc_logo_black.svg">
    <img alt="Hedgedoc Logo" src="https://raw.githubusercontent.com/hedgedoc/hedgedoc/f2ff89befccc2686b2bdf266af604b7de5a0c619/docs/content/images/hedgedoc_logo_white.svg">
  </picture>
</p>


[![#HedgeDoc on matrix.org][matrix.org-image]][matrix.org-url]
[![version][github-version-badge]][github-release-page]
[![POEditor][poeditor-image]][poeditor-url]
[![Mastodon][social-mastodon-image]][social-mastodon]
![REUSE Compliance Check][reuse-workflow-badge]
[![codecov][codecov-badge]][codecov-url]

HedgeDoc 可以让你实时协作创建 Markdown 笔记。

## 开始使用
- ℹ️ 在[我们的网站](https://hedgedoc.org)了解 HedgeDoc 及项目历史
- 🧪 通过[演示实例][hedgedoc-demo]试用 HedgeDoc。查看[功能页面][hedgedoc-demo-features]！
- 💽 使用[安装指南](https://docs.hedgedoc.org/setup/getting-started/)自行安装 HedgeDoc
- ❓ 有问题？加入我们的 [Matrix 聊天][matrix.org-url]或[社区论坛][hedgedoc-community]
- 💬 订阅[发布动态][github-release-feed]以获取最新信息

## 项目状态

HedgeDoc 1.x 已经稳定并在世界各地使用，但随着时间推移，代码库[逐渐增长](https://hedgedoc.org/history/)，使添加新功能变得困难。  
我们目前正在开发 HedgeDoc 2，即对 HedgeDoc 的完全重写。请注意：

- 此分支包含最新开发代码，尚未实现所有功能。**如果你正在寻找 1.x 源代码，请查看 [master 分支](https://github.com/hedgedoc/hedgedoc/tree/master)。**
- 1.x 版本只进行维护。我们不再接受此版本的功能请求或 pull request；如果某个错误在 2.0 中不再存在，我们可能会关闭非关键错误报告。
- HedgeDoc 2 将拆分为两个组件：后端和前端。两者都在本仓库中。

## 开发

有关设置本地开发环境的信息，请参阅[开发者文档](https://docs.hedgedoc.dev/how-to/develop/setup/)。

## HedgeDoc 2 Alpha

对 HedgeDoc 2 的新外观和使用体验感到好奇吗？我们在 [hedgedoc.dev](https://hedgedoc.dev) 提供了 alpha 演示。

如果想在自己的设备上试用，请访问 [HedgeDoc 2 文档](https://docs.hedgedoc.dev)。但请注意，这些内容可能随时间变化。

## 贡献

欢迎贡献！  
请查看[贡献文档](https://github.com/hedgedoc/hedgedoc/blob/f2ff89befccc2686b2bdf266af604b7de5a0c619/CONTRIBUTING.md)，了解如何提供帮助。如果想为 HedgeDoc 2 作贡献，请加入我们的[开发聊天][matrix.org-dev-url]。

# 许可证

采用 AGPLv3 许可。贡献者名单请参阅 [AUTHORS](https://github.com/hedgedoc/hedgedoc/blob/f2ff89befccc2686b2bdf266af604b7de5a0c619/AUTHORS)。

此许可证不包含 HedgeDoc 标志；其使用条款见 [GitHub 仓库](https://github.com/hedgedoc/hedgedoc-logo)。

[matrix.org-image]: https://img.shields.io/matrix/hedgedoc:matrix.org?logo=matrix&server_fqdn=matrix.org

[matrix.org-url]: https://chat.hedgedoc.org

[matrix.org-dev-url]: https://chat.hedgedoc.org/dev

[github-version-badge]: https://img.shields.io/github/release/hedgedoc/hedgedoc.svg

[github-release-page]: https://github.com/hedgedoc/hedgedoc/releases

[github-release-feed]: https://github.com/hedgedoc/hedgedoc/releases.atom

[github-issue-tracker]: https://github.com/hedgedoc/hedgedoc/issues/

[poeditor-image]: https://img.shields.io/badge/POEditor-translate-blue.svg

[poeditor-url]: https://poeditor.com/join/project/1OpGjF2Jir

[hedgedoc-demo]: https://hedgedoc.org/demo/

[hedgedoc-demo-features]: https://demo.hedgedoc.org/features

[hedgedoc-community]: https://community.hedgedoc.org

[hedgedoc-community-calls]: https://community.hedgedoc.org/t/codimd-community-call/19

[social-mastodon]: https://social.hedgedoc.org/mastodon

[social-mastodon-image]: https://img.shields.io/mastodon/follow/109259563190314667?domain=https%3A%2F%2Ffosstodon.org&style=social

[reuse-workflow-badge]: https://github.com/hedgedoc/hedgedoc/workflows/REUSE%20Compliance%20Check/badge.svg

[nestjs-workflow-badge]: https://github.com/hedgedoc/hedgedoc/workflows/Nest.JS%20CI/badge.svg

[codecov-badge]: https://codecov.io/gh/hedgedoc/hedgedoc/branch/develop/graph/badge.svg?token=pdaRF4qjNQ

[codecov-url]: https://codecov.io/gh/hedgedoc/hedgedoc
