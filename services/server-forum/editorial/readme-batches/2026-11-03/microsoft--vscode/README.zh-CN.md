# Visual Studio Code - 开源版（“Code - OSS”）
[![Feature Requests](https://img.shields.io/github/issues/microsoft/vscode/feature-request.svg)](https://github.com/microsoft/vscode/issues?q=is%3Aopen+is%3Aissue+label%3Afeature-request+sort%3Areactions-%2B1-desc)
[![Bugs](https://img.shields.io/github/issues/microsoft/vscode/bug.svg)](https://github.com/microsoft/vscode/issues?utf8=✓&q=is%3Aissue+is%3Aopen+label%3Abug)

## 本仓库

这个仓库（“`Code - OSS`”）是我们（微软）与社区共同开发 [Visual Studio Code](https://code.visualstudio.com) 产品的地方。我们不仅在这里处理代码和问题，还发布[路线图](https://github.com/microsoft/vscode/wiki/Roadmap)、[每月迭代计划](https://github.com/microsoft/vscode/wiki/Iteration-Plans)和[收尾计划](https://github.com/microsoft/vscode/wiki/Running-the-Endgame)。这些源代码依据标准 [MIT 许可证](https://github.com/microsoft/vscode/blob/main/LICENSE.txt)向所有人开放。

## Visual Studio Code

<p align="center">
  <img alt="VS Code in action" src="https://github.com/user-attachments/assets/56af271c-949d-454c-a3ea-16188c063414">
</p>

[Visual Studio Code](https://code.visualstudio.com) 是基于 `Code - OSS` 仓库、包含微软特定定制的发行版，依据传统的[微软产品许可证](https://code.visualstudio.com/License/)发布。

[Visual Studio Code](https://code.visualstudio.com) 结合了代码编辑器的简洁性以及开发者在编辑、构建和调试核心流程中所需的功能。它提供全面的代码编辑、导航和理解支持，以及轻量级调试、丰富的扩展机制和与现有工具的轻量集成。

Visual Studio Code 每月更新，带来新功能和错误修复。你可以从 [Visual Studio Code 网站](https://code.visualstudio.com/Download)下载 Windows、macOS 和 Linux 版本。要每天获取最新版本，请安装 [Insiders 版本](https://code.visualstudio.com/insiders)。

## 参与贡献

参与本项目有许多方式，例如：

* [提交错误和功能请求](https://github.com/microsoft/vscode/issues)，并在相关更改提交后协助我们验证
* 审查[源代码更改](https://github.com/microsoft/vscode/pulls)
* 审阅[文档](https://github.com/microsoft/vscode-docs)，并通过 Pull Request 修正拼写错误或添加新内容。

如果你有兴趣修复问题并直接为代码库贡献，请参阅[如何贡献](https://github.com/microsoft/vscode/wiki/How-to-Contribute)，其中涵盖：

* [如何从源代码构建和运行](https://github.com/microsoft/vscode/wiki/How-to-Contribute)
* [开发工作流，包括调试和运行测试](https://github.com/microsoft/vscode/wiki/How-to-Contribute#debugging)
* [编码指南](https://github.com/microsoft/vscode/wiki/Coding-Guidelines)
* [提交 Pull Request](https://github.com/microsoft/vscode/wiki/How-to-Contribute#pull-requests)
* [寻找可参与的问题](https://github.com/microsoft/vscode/wiki/How-to-Contribute#where-to-contribute)
* [参与翻译](https://aka.ms/vscodeloc)

## 反馈

* 在 [Stack Overflow](https://stackoverflow.com/questions/tagged/vscode) 提问
* [请求新功能](https://github.com/microsoft/vscode/blob/18c52617ca765445751c66ffb61137d9101823cc/CONTRIBUTING.md)
* 为[热门功能请求](https://github.com/microsoft/vscode/issues?q=is%3Aopen+is%3Aissue+label%3Afeature-request+sort%3Areactions-%2B1-desc)投票
* [提交问题](https://github.com/microsoft/vscode/issues)
* 通过 [GitHub Discussions](https://github.com/microsoft/vscode-discussions/discussions)或 [Slack](https://aka.ms/vscode-dev-community) 联系扩展作者社区
* 关注 [@code](https://x.com/code)，告诉我们你的想法！

请参阅我们的 [wiki](https://github.com/microsoft/vscode/wiki/Feedback-Channels)，了解各渠道以及其他社区渠道的信息。

## 相关项目

VS Code 的许多核心组件和扩展都位于 GitHub 上的独立仓库中。例如，[node 调试适配器](https://github.com/microsoft/vscode-node-debug)和 [mono 调试适配器](https://github.com/microsoft/vscode-mono-debug)位于不同仓库。完整列表请访问我们 [wiki](https://github.com/microsoft/vscode/wiki) 中的[相关项目](https://github.com/microsoft/vscode/wiki/Related-Projects)页面。

## 内置扩展

VS Code 包含一组位于 [extensions](https://github.com/microsoft/vscode/blob/18c52617ca765445751c66ffb61137d9101823cc/extensions) 文件夹中的内置扩展，包括多种语言的语法和代码片段。为某种语言提供丰富支持（行内建议、转到定义）的扩展带有 `language-features` 后缀。例如，`json` 扩展为 `JSON` 提供语法着色，`json-language-features` 扩展则为 `JSON` 提供丰富的语言支持。

## 开发容器

本仓库包含一个 Visual Studio Code Dev Containers / GitHub Codespaces 开发容器。

* 对于 [Dev Containers](https://aka.ms/vscode-remote/download/containers)，使用 **Dev Containers: Clone Repository in Container Volume...** 命令；该命令会创建 Docker 卷，以改善 macOS 和 Windows 上的磁盘 I/O。
  * 如果你已安装 VS Code 和 Docker，也可以点击[此处](https://vscode.dev/redirect?url=vscode://ms-vscode-remote.remote-containers/cloneInVolume?url=https://github.com/microsoft/vscode)开始。VS Code 会在需要时自动安装 Dev Containers 扩展，将源代码克隆到容器卷，并启动一个开发容器供使用。

* 对于 Codespaces，请在 VS Code 中安装 [GitHub Codespaces](https://marketplace.visualstudio.com/items?itemName=GitHub.codespaces) 扩展，并使用 **Codespaces: Create New Codespace** 命令。

运行完整构建时，Docker / Codespace 应至少具有 **4 个核心和 6 GB 内存（推荐 8 GB）**。更多信息请参阅[开发容器 README](https://github.com/microsoft/vscode/blob/18c52617ca765445751c66ffb61137d9101823cc/.devcontainer/README.md)。

## 行为准则

本项目采用了[微软开源行为准则](https://opensource.microsoft.com/codeofconduct/)。更多信息请参阅[行为准则常见问题](https://opensource.microsoft.com/codeofconduct/faq/)，或通过 [opencode@microsoft.com](mailto:opencode@microsoft.com) 联系我们，提出其他问题或意见。

## 许可证

版权所有 (c) Microsoft Corporation。保留所有权利。

依据 [MIT](https://github.com/microsoft/vscode/blob/18c52617ca765445751c66ffb61137d9101823cc/LICENSE.txt)许可证授权。
