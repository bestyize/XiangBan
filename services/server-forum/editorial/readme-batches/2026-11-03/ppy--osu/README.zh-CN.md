<p align="center">
  <img width="500" alt="osu! logo" src="https://raw.githubusercontent.com/ppy/osu/fc9d1d936a3b886d6f8eab9c819174f3580796b9/assets/lazer.png">
</p>

# osu!

[![Build status](https://github.com/ppy/osu/actions/workflows/ci.yml/badge.svg?branch=master&event=push)](https://github.com/ppy/osu/actions/workflows/ci.yml)
[![GitHub release](https://img.shields.io/github/release/ppy/osu.svg)](https://github.com/ppy/osu/releases/latest)
[![CodeFactor](https://www.codefactor.io/repository/github/ppy/osu/badge)](https://www.codefactor.io/repository/github/ppy/osu)
[![dev chat](https://discordapp.com/api/guilds/188630481301012481/widget.png?style=shield)](https://discord.gg/ppy)
[![Crowdin](https://d322cqt584bo4o.cloudfront.net/osu-web/localized.svg)](https://crowdin.com/project/osu-web)

一款不靠付费取胜的节奏游戏。轻轻*点击*，节奏近在眼前！

这是 [osu!](https://osu.ppy.sh) 游戏客户端未来、也是最终的迭代，标志着开放时代的开始！目前以发布代号“*lazer*”为人所知和发布，寓意比尖端还锋利。

## 状态

本项目持续开发，但我们尽力保持稳定。鼓励玩家在稳定版 *osu!* 客户端旁安装一个发布版本。项目将继续演进，直至大多数用户最终更喜欢它而不是之前的“osu!stable”版本。

以下资源可作为参与和了解项目的起点：

- [osu! 官方网站](https://osu.ppy.sh/home/changelog/lazer)提供详细的版本更新日志。
- 你可以进一步了解我们的[项目管理](https://github.com/ppy/osu/wiki/Project-management)方法。
- 跟踪我们当前[改进游戏的工作](https://github.com/orgs/ppy/projects/7/views/6)。

## 运行 osu!

如果只是想试玩游戏，可以获取适合自己平台的最新发布版本：

### 最新发布版本：

| [Windows 10+ (x64)](https://github.com/ppy/osu/releases/latest/download/install.exe) | macOS 12+ ([Intel](https://github.com/ppy/osu/releases/latest/download/osu.app.Intel.zip), [Apple Silicon](https://github.com/ppy/osu/releases/latest/download/osu.app.Apple.Silicon.zip)) | [Linux (x64)](https://github.com/ppy/osu/releases/latest/download/osu.AppImage) | [iOS 13.4+](https://osu.ppy.sh/home/testflight) | [Android 7+](https://github.com/ppy/osu/releases/latest/download/sh.ppy.osulazer.apk) |
|--------------------------------------------------------------------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------| ------------- | ------------- | ------------- |

通常也可以从 [osu! 网站](https://osu.ppy.sh/home/download)下载适合当前设备的版本。

如果你的平台不受支持或未在上面列出，仍可能通过以下说明运行发布版本或手动构建。

**iOS/iPadOS 用户注意**：iOS TestFlight 链接很快就会满员（Apple 硬性限制为 10,000 名用户）。我们偶尔会重置名额，请不要询问此事。请定期回来查看是否重置，或在 Twitter 关注 [peppy](https://twitter.com/ppy) 获取公告。我们的目标是尽快将游戏上架移动应用商店，以摆脱这一限制。

## 开发自定义规则集

osu! 允许用户创建称为“规则集”的玩法变体。开发规则集可以利用 osu! 的谱面库、游戏引擎及整体用户体验创造新的玩法。我们在[这里](https://github.com/ppy/osu/tree/fc9d1d936a3b886d6f8eab9c819174f3580796b9/Templates)提供了一些模板作为入门。

你可以在[自定义规则集目录](https://github.com/ppy/osu/discussions/13096)中查看示例。

## 开发 osu!

### 前提条件

请确保满足以下条件：

- 已安装正确 [.NET SDK](https://dotnet.microsoft.com/download) 的桌面平台。

处理代码库时，我们建议使用支持智能代码补全和语法高亮的 IDE，例如最新版 [Visual Studio](https://visualstudio.microsoft.com/vs/)、[JetBrains Rider](https://www.jetbrains.com/rider/)，或安装了 [EditorConfig](https://marketplace.visualstudio.com/items?itemName=EditorConfig.EditorConfig) 和 [C# Dev Kit](https://marketplace.visualstudio.com/items?itemName=ms-dotnettools.csdevkit) 插件的 [Visual Studio Code](https://code.visualstudio.com/)。

### 下载源代码

克隆仓库：

```shell
git clone https://github.com/ppy/osu
cd osu
```

要将源代码更新至最新提交，请在 `osu` 目录中运行：

```shell
git pull
```

### 构建

#### 使用 IDE

应通过某个平台专用的 `.slnf` 文件加载解决方案，而非主 `.sln` 文件。这样可以减少依赖并隐藏不关心的平台。可用的 `.slnf` 文件包括：

- `osu.Desktop.slnf`（最常用）
- `osu.Android.slnf`
- `osu.iOS.slnf`

项目包含上述推荐 IDE 的运行配置。请使用 IDE 提供的 Build/Run 功能来启动。测试或构建新组件时，强烈建议使用 `osu! (Tests)` 项目或配置。更多信息见[下文](#参与贡献)。

为移动平台构建时，如果之前尚未执行，可能需要运行 `sudo dotnet workload restore`。这将安装完成构建所需的 Android/iOS 工具。

#### 使用命令行

也可以通过一条命令从命令行构建并运行 *osu!*：

```shell
dotnet run --project osu.Desktop
```

本地进行任何性能测试时，请务必在构建命令中添加 `-c Release`，因为默认 `Debug` 配置的运行开销可能很大（尤其是在按下文所述使用本地框架修改进行测试时）。

如果构建失败，请尝试使用 `dotnet restore` 恢复 NuGet 包。

### 测试资源或框架修改

有时可能需要交叉测试 [osu-resources](https://github.com/ppy/osu-resources) 或 [osu-framework](https://github.com/ppy/osu-framework) 中的修改。可以使用随附命令快速实现：

Windows：

```ps
UseLocalFramework.ps1
UseLocalResources.ps1
```

macOS / Linux：

```ps
UseLocalFramework.sh
UseLocalResources.sh
```

请注意，这些命令假定你已将相关项目检出到相邻目录中：

```
|- osu            // this repository
|- osu-framework
|- osu-resources
```

### 代码分析

提交代码前请运行代码格式化工具。可以在命令行运行 `dotnet format`，或使用 IDE 的 `Format code` 命令。

我们采用了一些跨平台、集成于编译器的分析器。在 IDE 或命令行编辑、构建时，它们可以像编译器本身一样提供警告。

JetBrains ReSharper InspectCode 也用于更广泛的规则集。可以从 PowerShell 运行 `.\InspectCode.ps1`。也可以安装 ReSharper 或使用 Rider，在所选 IDE 中获得内联支持。

## 参与贡献

对项目作贡献的两种主要方式是报告问题和提交 pull request。请参阅[贡献指南](https://github.com/ppy/osu/blob/fc9d1d936a3b886d6f8eab9c819174f3580796b9/CONTRIBUTING.md)，了解最有效的帮助方式。

如果希望帮助本地化，请前往 [crowdin](https://crowdin.com/project/osu-web)。

我们的团队相信**人类贡献**。任何由 AI/LLM 创建、记录或辅助的贡献——无论是问题报告还是 pull request——通常都会被**直接关闭并锁定，不再进一步讨论**。

## 许可证

*osu!* 的代码和框架采用 [MIT 许可证](https://opensource.org/licenses/MIT)。更多信息请参阅[许可证文件](https://github.com/ppy/osu/blob/fc9d1d936a3b886d6f8eab9c819174f3580796b9/LICENCE)。[简而言之](https://tldrlegal.com/license/mit-license)，只要在软件或源代码的任何副本中包含原始版权和许可证声明，你就可以自由使用。

请注意，这**不涵盖**在软件、资源、广告或宣传中使用“osu!”或“ppy”品牌，因为它们受商标法保护。

另请注意，游戏资源采用单独许可证。详情请参阅 [ppy/osu-resources](https://github.com/ppy/osu-resources) 仓库。
