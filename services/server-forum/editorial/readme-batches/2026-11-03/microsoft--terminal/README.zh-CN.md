> 根据 [Windows Terminal 固定版本 README](https://github.com/microsoft/terminal/blob/188822fa51dba7e11b64801483f4909db5fd020a/README.md) 翻译，仓库许可为 MIT。Windows 和开发工具要求以该版本原文为准；Microsoft 名称和标识可能受独立商标政策约束。

![Windows Terminal 项目徽标和品牌图](https://github.com/microsoft/terminal/assets/91625426/333ddc76-8ab2-4eb4-a8c0-4d7b953b1179)

[![Terminal 构建状态](https://dev.azure.com/shine-oss/terminal/_apis/build/status%2FTerminal%20CI?branchName=main)](https://dev.azure.com/shine-oss/terminal/_build/latest?definitionId=1&branchName=main)

# 欢迎来到 Windows Terminal、Console 和命令行仓库


<details>
<summary><strong>目录</strong></summary>

- [安装和运行 Windows Terminal](#installing-and-running-windows-terminal)
- [Microsoft Store［推荐］](#microsoft-store-recommended)
- [其他安装方法](#other-install-methods)
- [通过 GitHub](#via-github)
- [通过 Windows 包管理器 CLI（又称 winget）](#via-windows-package-manager-cli-aka-winget)
- [通过 Chocolatey（非官方）](#via-chocolatey-unofficial)
- [通过 Scoop（非官方）](#via-scoop-unofficial)
- [安装 Windows Terminal Canary](#installing-windows-terminal-canary)
- [Terminal 与 Console 概览](#terminal--console-overview)
- [Windows Terminal](#windows-terminal)
- [Windows 控制台宿主](#the-windows-console-host)
- [共享组件](#shared-components)
- [创建新的 Windows Terminal](#creating-the-new-windows-terminal)
- [资源](#resources)
- [常见问题](#faq)
- [我构建并运行了新的 Terminal，但它看起来与旧控制台一样](#i-built-and-ran-the-new-terminal-but-it-looks-just-like-the-old-console)
- [文档](#documentation)
- [参与贡献](#contributing)
- [与团队交流](#communicating-with-the-team)
- [开发者指南](#developer-guidance)
- [前提条件](#prerequisites)
- [构建代码](#building-the-code)
- [在 PowerShell 中构建](#building-in-powershell)
- [在 Cmd 中构建](#building-in-cmd)
- [运行与调试](#running--debugging)
- [编码指南](#coding-guidance)
- [行为准则](#code-of-conduct)

</details>

本仓库包含以下项目的源代码：

* [Windows Terminal](https://aka.ms/terminal)
* [Windows Terminal Preview](https://aka.ms/terminal-preview)
* Windows 控制台宿主（`conhost.exe`）
* 两个项目共享的组件
* [ColorTool](https://github.com/microsoft/terminal/tree/188822fa51dba7e11b64801483f4909db5fd020a/src/tools/ColorTool)
* 展示如何使用 Windows Console API 的[示例项目](https://github.com/microsoft/terminal/tree/188822fa51dba7e11b64801483f4909db5fd020a/samples)

相关仓库包括：

* [Windows Terminal 文档](https://learn.microsoft.com/windows/terminal)（[参与文档贡献的仓库](https://github.com/MicrosoftDocs/terminal)）
* [Console API 文档](https://github.com/MicrosoftDocs/Console-Docs)
* [Cascadia Code 字体](https://github.com/Microsoft/Cascadia-Code)

<a id="installing-and-running-windows-terminal"></a>
## 安装和运行 Windows Terminal

> [!NOTE]
> Windows Terminal 需要 Windows 10 2004（构建版本 19041）或更新版本。

<a id="microsoft-store-recommended"></a>
### Microsoft Store［推荐］

从 [Microsoft Store 安装 Windows Terminal][store-install-link]。发布新构建时，自动升级可让你始终使用最新版本。这是我们首选的方法。

<a id="other-install-methods"></a>
### 其他安装方法

<a id="via-github"></a>
#### 通过 GitHub

无法从 Microsoft Store 安装 Windows Terminal 的用户，可从本仓库的 [Releases 页面](https://github.com/microsoft/terminal/releases)手动下载发布版本。

从 **Assets** 部分下载 `Microsoft.WindowsTerminal_<versionNumber>.msixbundle` 文件。只需双击 `.msixbundle` 文件即可安装应用，安装程序应自动运行。如果因任何原因失败，可在 PowerShell 提示符下尝试以下命令：

```powershell
# NOTE: If you are using PowerShell 7+, please run
# Import-Module Appx -UseWindowsPowerShell
# before using Add-AppxPackage.

Add-AppxPackage Microsoft.WindowsTerminal_<versionNumber>.msixbundle
```

> [!NOTE]
> 手动安装 Terminal 时：
>
> * 可能需要安装 [VC++ v14 Desktop Framework Package](https://learn.microsoft.com/troubleshoot/cpp/c-runtime-packages-desktop-bridge#how-to-install-and-update-desktop-framework-packages)。通常只有在较旧版本的 Windows 10 上遇到缺少框架包的错误时才需要。
> * 新构建发布时 Terminal 不会自动更新，因此需要定期安装最新版本以获取所有最新修复和改进！

上述代码注释说明：使用 PowerShell 7+ 时，请先运行 `Import-Module Appx -UseWindowsPowerShell`，再使用 `Add-AppxPackage`。

<a id="via-windows-package-manager-cli-aka-winget"></a>
#### 通过 Windows 包管理器 CLI（又称 winget）

[winget](https://github.com/microsoft/winget-cli) 用户可以安装 `Microsoft.WindowsTerminal` 包，下载并安装最新 Terminal 发行版：

```powershell
winget install --id Microsoft.WindowsTerminal -e
```

> [!NOTE]
> WinGet [1.6.2631 或更高版本](https://github.com/microsoft/winget-cli/releases)支持依赖项。要安装 Terminal 1.18 或更高稳定版，请确保 WinGet 客户端已更新。

<a id="via-chocolatey-unofficial"></a>
#### 通过 Chocolatey（非官方）

[Chocolatey](https://chocolatey.org) 用户可以安装 `microsoft-windows-terminal` 包，下载并安装最新 Terminal 发行版：

```powershell
choco install microsoft-windows-terminal
```

使用 Chocolatey 升级 Windows Terminal：

```powershell
choco upgrade microsoft-windows-terminal
```

如果安装或升级包时遇到问题，请访问 [Windows Terminal 包页面](https://chocolatey.org/packages/microsoft-windows-terminal)，并遵循 [Chocolatey 问题分类流程](https://chocolatey.org/docs/package-triage-process)。

<a id="via-scoop-unofficial"></a>
#### 通过 Scoop（非官方）

[Scoop](https://scoop.sh) 用户可以安装 `windows-terminal` 包，下载并安装最新 Terminal 发行版：

```powershell
scoop bucket add extras
scoop install windows-terminal
```

使用 Scoop 更新 Windows Terminal：

```powershell
scoop update windows-terminal
```

如果安装或更新包时遇到问题，请在 Scoop Extras bucket 仓库的 [issues 页面](https://github.com/lukesampson/scoop-extras/issues)搜索或报告。

---

<a id="installing-windows-terminal-canary"></a>
## 安装 Windows Terminal Canary

Windows Terminal Canary 是 Windows Terminal 的 nightly 构建，包含 `main` 分支的最新代码，让你在功能进入 Windows Terminal Preview 之前试用。

Windows Terminal Canary 是我们最不稳定的版本，因此你可能在我们发现之前就遇到错误。

Windows Terminal Canary 提供应用安装程序版和便携 ZIP 版。

应用安装程序版支持自动更新。由于平台限制，该安装程序仅适用于 Windows 11。

便携 ZIP 版是便携应用，不会自动更新，也不会自动检查更新。它适用于 Windows 10（19041+）和 Windows 11。

| 发行方式 | 架构 | 链接 |
|---|:---:|---|
| 应用安装程序 | x64, arm64, x86 | [下载](https://aka.ms/terminal-canary-installer) |
| 便携 ZIP | x64 | [下载](https://aka.ms/terminal-canary-zip-x64) |
| 便携 ZIP | ARM64 | [下载](https://aka.ms/terminal-canary-zip-arm64) |
| 便携 ZIP | x86 | [下载](https://aka.ms/terminal-canary-zip-x86) |

_了解更多 [Windows Terminal 发行类型](https://learn.microsoft.com/windows/terminal/distributions)。_

---

<a id="terminal--console-overview"></a>
## Terminal 与 Console 概览

深入代码之前，请花几分钟阅读以下概览：

<a id="windows-terminal"></a>
### Windows Terminal

Windows Terminal 是面向命令行用户的全新、现代、功能丰富且高效的终端应用。它包含 Windows 命令行社区最常请求的许多功能，包括标签页、富文本、国际化、可配置性、主题与样式等。

Terminal 还需要达到我们的目标和衡量标准，确保它保持快速高效，不消耗大量内存或电力。

<a id="the-windows-console-host"></a>
### Windows 控制台宿主

Windows 控制台宿主 `conhost.exe` 是 Windows 原有的命令行用户体验。它还承载 Windows 的命令行基础设施、Windows Console API 服务器、输入引擎、渲染引擎、用户偏好设置等。本仓库的控制台宿主代码正是构建 Windows 自带 `conhost.exe` 的实际源代码。

自 2014 年接手 Windows 命令行以来，团队为 Console 添加了若干新功能，包括背景透明、按行选择、对 [ANSI / 虚拟终端序列](https://en.wikipedia.org/wiki/ANSI_escape_code)的支持、[24 位颜色](https://devblogs.microsoft.com/commandline/24-bit-color-in-the-windows-console/)、[伪控制台（“ConPTY”）](https://devblogs.microsoft.com/commandline/windows-command-line-introducing-the-windows-pseudo-console-conpty/)等。

然而，由于 Windows Console 的首要目标是保持向后兼容，我们无法添加社区和团队多年来期待的许多功能，包括标签页、Unicode 文本和 emoji。

这些限制促使我们创建新的 Windows Terminal。

> 要了解命令行总体的发展以及 Windows 命令行的发展，请阅读命令行团队博客中的[系列文章](https://devblogs.microsoft.com/commandline/windows-command-line-backgrounder/)。

<a id="shared-components"></a>
### 共享组件

在改造 Windows Console 时，我们大幅现代化了代码库：将逻辑实体清晰分成模块和类，引入若干关键扩展点，使用更安全高效的 [STL 容器](https://docs.microsoft.com/en-us/cpp/standard-library/stl-containers?view=vs-2022)替换几个旧的自研集合和容器，并通过 Microsoft 的 [Windows Implementation Libraries - WIL](https://github.com/Microsoft/wil) 简化代码、提高安全性。

此次改造使 Console 的若干关键组件可供 Windows 上的任何终端实现复用，包括基于 DirectWrite 的新文本布局和渲染引擎、能存储 UTF-16 和 UTF-8 的文本缓冲区、VT 解析器和输出器等。

<a id="creating-the-new-windows-terminal"></a>
### 创建新的 Windows Terminal

开始规划新的 Windows Terminal 应用时，我们探索并评估了多种方法和技术栈。最终决定继续投入 C++ 代码库最能满足目标，这让我们能在现有 Console 与新 Terminal 中复用上述若干现代化组件。此外，我们意识到这样可以将 Terminal 的大部分核心构建成可复用的 UI 控件，供其他人集成到自己的应用中。

这些工作的成果位于本仓库，并作为 Windows Terminal 应用提供；你可以从 Microsoft Store 或[直接从本仓库的发行版](https://github.com/microsoft/terminal/releases)下载。

---

<a id="resources"></a>
## 资源

以下资源可能有助于了解 Windows Terminal：

* [命令行博客](https://devblogs.microsoft.com/commandline)
* [命令行背景介绍系列博客](https://devblogs.microsoft.com/commandline/windows-command-line-backgrounder/)
* Windows Terminal 发布：[Terminal 宣传视频](https://www.youtube.com/watch?v=8gw0rXPMMPE&list=PLEHMQNlPj-Jzh9DkNpqipDGCZZuOwrQwR&index=2&t=0s)
* Windows Terminal 发布：[Build 2019 讲座](https://www.youtube.com/watch?v=KMudkRcwjCw)
* Run As Radio：[第 645 期——Richard Turner 谈 Windows Terminal](https://www.runasradio.com/Shows/Show/645)
* Azure DevOps Podcast：[第 54 期——Kayla Cinnamon 和 Rich Turner 谈 Windows Terminal 团队的 DevOps](http://azuredevopspodcast.clear-measure.com/kayla-cinnamon-and-rich-turner-on-devops-on-the-windows-terminal-team-episode-54)
* Microsoft Ignite 2019 讲座：[现代 Windows 命令行：Windows Terminal - BRK3321](https://myignite.techcommunity.microsoft.com/sessions/81329?source=sessions)

---

<a id="faq"></a>
## 常见问题

<a id="i-built-and-ran-the-new-terminal-but-it-looks-just-like-the-old-console"></a>
### 我构建并运行了新的 Terminal，但它看起来与旧控制台一样

原因：你在 Visual Studio 中启动了错误的解决方案。

解决方法：确保在 Visual Studio 中构建并部署 `CascadiaPackage` 项目。

> [!NOTE]
> `OpenConsole.exe` 只是本地构建的 `conhost.exe`，即承载 Windows 命令行基础设施的经典 Windows Console。Windows Terminal 使用 OpenConsole 通过 [ConPty](https://devblogs.microsoft.com/commandline/windows-command-line-introducing-the-windows-pseudo-console-conpty/)连接命令行应用并与其通信。

---

<a id="documentation"></a>
## 文档

项目全部文档位于 [aka.ms/terminal-docs](https://aka.ms/terminal-docs)。如果想为文档做贡献，请向 [Windows Terminal 文档仓库](https://github.com/MicrosoftDocs/terminal)提交拉取请求。

---

<a id="contributing"></a>
## 参与贡献

我们很高兴与你们这个出色的社区一起构建并改进 Windows Terminal！

***开始开发功能或修复问题之前***，请阅读并遵循[贡献者指南](https://github.com/microsoft/terminal/blob/188822fa51dba7e11b64801483f4909db5fd020a/CONTRIBUTING.md)，避免浪费或重复工作。

<a id="communicating-with-the-team"></a>
## 与团队交流

与团队交流最简单的方式是通过 GitHub issues。

欢迎提交新 issue、功能请求和建议，但**创建新 issue 前务必搜索类似的现有开放或已关闭 issue。**

如果想提出的问题（暂时）不值得新建 issue，请通过 Twitter 联系我们：

* Christopher Nguyen，产品经理：[@nguyen_dows](https://twitter.com/nguyen_dows)
* Dustin Howett，工程负责人：[@dhowett](https://twitter.com/DHowett)
* Mike Griese，高级开发者：[@zadjii@mastodon.social](https://mastodon.social/@zadjii)
* Carlos Zamora，开发者：[@cazamor_msft](https://twitter.com/cazamor_msft)
* Pankaj Bhojwani，开发者
* Leonard Hecker，开发者：[@LeonardHecker](https://twitter.com/LeonardHecker)

<a id="developer-guidance"></a>
## 开发者指南

<a id="prerequisites"></a>
## 前提条件

可以通过以下两种方式之一配置 Terminal 构建环境：

### 使用 WinGet 配置文件

克隆仓库后，可以使用 [WinGet 配置文件](https://learn.microsoft.com/en-us/windows/package-manager/configuration/#use-a-winget-configuration-file-to-configure-your-machine)设置环境。[默认配置文件](https://github.com/microsoft/terminal/blob/188822fa51dba7e11b64801483f4909db5fd020a/.config/configuration.winget)安装 Visual Studio 2026 Community 和其余必需工具。[.config](https://github.com/microsoft/terminal/tree/188822fa51dba7e11b64801483f4909db5fd020a/.config) 目录还提供 Visual Studio 2026 Enterprise 和 Professional 版本的另外两个配置文件。运行默认配置文件时，可以在资源管理器中双击文件，或运行以下命令：

```powershell
winget configure .config\configuration.winget
```

### 手动配置

* 运行 Windows Terminal 必须使用 Windows 10 2004（构建版本 >= 10.0.19041.0）或更高版本。
* 必须[在 Windows 设置应用中启用开发者模式](https://learn.microsoft.com/windows/uwp/get-started/enable-your-device-for-development)，才能在本地安装并运行 Windows Terminal。
* 必须安装 [PowerShell 7 或更高版本](https://github.com/PowerShell/PowerShell/releases/latest)。
* 必须安装 [Windows 11 (10.0.26100) SDK](https://developer.microsoft.com/windows/downloads/windows-sdk/)，版本为 10.0.26100.8249 或更高。
* 必须至少安装 [VS 2026](https://visualstudio.microsoft.com/downloads/) 18.6 版。
* 必须通过 VS 安装程序安装以下工作负载。注意：打开解决方案时将[提示自动安装缺少的组件](https://devblogs.microsoft.com/setup/configure-visual-studio-across-your-organization-with-vsconfig/)：
  * Desktop Development with C++
  * WinUI application development
* 构建测试项目必须安装 [.NET Framework 4.7.2 目标包](https://learn.microsoft.com/dotnet/framework/install/guide-for-developers#to-install-the-net-framework-developer-pack-or-targeting-pack)。

<a id="building-the-code"></a>
## 构建代码

可以在 Visual Studio 内构建 OpenConsole.slnx，也可以通过 **/tools** 目录中的一组便捷脚本和工具从命令行构建：

<a id="building-in-powershell"></a>
### 在 PowerShell 中构建

```powershell
Import-Module .\tools\OpenConsole.psm1
Set-MsBuildDevEnvironment
Invoke-OpenConsoleBuild
```

<a id="building-in-cmd"></a>
### 在 Cmd 中构建

```shell
.\tools\razzle.cmd
bcz
```

<a id="running--debugging"></a>
## 运行与调试

要在 VS 中调试 Windows Terminal，请在解决方案资源管理器中右键点击 `CascadiaPackage` 并打开属性。在 Debug 菜单中，将“Application process”和“Background task process”改为“Native Only”。

接下来应可按 <kbd>F5</kbd> 构建并调试 Terminal 项目。请确保选择“x64”或“x86”平台——Terminal 无法为“Any Cpu”构建（因为 Terminal 是 C++ 应用，而不是 C# 应用）。

> 👉 你**不能**直接通过运行 WindowsTerminal.exe 启动 Terminal。原因详见 [#926](https://github.com/microsoft/terminal/issues/926)、[#4043](https://github.com/microsoft/terminal/issues/4043)。

<a id="coding-guidance"></a>
### 编码指南

请查看以下关于编码实践的简短文档。

> 👉 如果发现这些文档缺少内容，欢迎为仓库任何位置的文档做贡献（或编写新文档）！

随着我们逐渐了解需要提供哪些内容才能让大家有效参与项目，这些文档仍在不断完善。

* [编码风格](https://github.com/microsoft/terminal/blob/188822fa51dba7e11b64801483f4909db5fd020a/doc/STYLE.md)
* [代码组织](https://github.com/microsoft/terminal/blob/188822fa51dba7e11b64801483f4909db5fd020a/doc/ORGANIZATION.md)
* [旧代码库中的异常](https://github.com/microsoft/terminal/blob/188822fa51dba7e11b64801483f4909db5fd020a/doc/EXCEPTIONS.md)
* [在 WIL 中与 Windows 交互时有用的智能指针和宏](https://github.com/microsoft/terminal/blob/188822fa51dba7e11b64801483f4909db5fd020a/doc/WIL.md)

---

<a id="code-of-conduct"></a>
## 行为准则

本项目采用 [Microsoft 开源行为准则][conduct-code]。更多信息请参阅[行为准则常见问题][conduct-FAQ]，或通过 [opencode@microsoft.com][conduct-email] 联系我们，提出其他问题或意见。

[conduct-code]: https://opensource.microsoft.com/codeofconduct/
[conduct-FAQ]: https://opensource.microsoft.com/codeofconduct/faq/
[conduct-email]: mailto:opencode@microsoft.com
[store-install-link]: https://aka.ms/terminal
