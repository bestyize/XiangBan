> 本文由小伴根据 [Audacity 项目 README](https://github.com/audacity/audacity/blob/9f09f496ddb402cf56d6f383905ed796820c61d3/README.md) 翻译并改写部分相对链接；原文作者为 Audacity 项目贡献者，文档采用 [CC BY 3.0](https://creativecommons.org/licenses/by/3.0/)。

# Audacity

[![Coverage](https://s3.us-east-1.amazonaws.com/extensions.musescore.org/test/code_coverage/au_coverage_badge.svg)](https://github.com/audacity/audacity/actions/workflows/au4_check_unit_tests.yml)

[**Audacity**](https://www.audacityteam.org) 是一款易于使用的多轨音频编辑和录音软件，适用于 Windows、macOS、GNU/Linux 和其他操作系统。更多信息请见 https://www.audacityteam.org。

## 本仓库目前正在进行重大的结构调整。

我们目前正在开发 Audacity 4，这意味着将引入全新的界面并进行大量重构。因此，目前的 `master` 分支对新贡献者来说并不特别友好。仍然可以向 Audacity 3.x 提交补丁；如果选择这样做，请确保从 `audacity3` 分支创建分支。Audacity 3.x 的构建说明见[此处](https://github.com/audacity/audacity/blob/release-3.7.0/BUILDING.md)；Audacity 4 的构建说明见[此处](https://github.com/audacity/audacity/blob/9f09f496ddb402cf56d6f383905ed796820c61d3/BUILDING.md)。

你可以通过 [YouTube](https://youtube.com/@audacity)、[Discord](https://discord.gg/audacity) 和[我们的博客](https://audacityteam.org/blog)了解最新进展。

## 许可证

Audacity 是采用 GPLv3 许可的开源软件。大多数代码文件采用 GPLv2-or-later，显著的例外包括 `/au3/lib-src`（其中包含第三方库）以及与 VST3 相关的代码。除非另有说明，文档采用 CC BY 3.0 许可。详情见[许可证文件](https://github.com/audacity/audacity/blob/9f09f496ddb402cf56d6f383905ed796820c61d3/LICENSE.txt)。
