> 根据 [Jellyfin 固定版本 README](https://github.com/jellyfin/jellyfin/blob/5b385862fb97201d954a6ac7c9c745d791a738fa/README.md) 翻译；许可证见项目仓库。开发依赖和版本说明为上游该版本原文，使用时请核对最新文档。

<h1 align="center">Jellyfin</h1>
<h3 align="center">自由软件媒体系统</h3>

---

<p align="center">
<img alt="Logo 横幅" src="https://raw.githubusercontent.com/jellyfin/jellyfin-ux/master/branding/SVG/banner-logo-solid.svg?sanitize=true"/>
<br/><br/>
<a href="https://github.com/jellyfin/jellyfin"><img alt="GPL 2.0 许可证" src="https://img.shields.io/github/license/jellyfin/jellyfin.svg"/></a>
<a href="https://github.com/jellyfin/jellyfin/releases"><img alt="当前发行版" src="https://img.shields.io/github/release/jellyfin/jellyfin.svg"/></a>
<a href="https://translate.jellyfin.org/projects/jellyfin/jellyfin-core/?utm_source=widget"><img alt="翻译状态" src="https://translate.jellyfin.org/widgets/jellyfin/-/jellyfin-core/svg-badge.svg"/></a>
<a href="https://hub.docker.com/r/jellyfin/jellyfin"><img alt="Docker 拉取次数" src="https://img.shields.io/docker/pulls/jellyfin/jellyfin.svg"/></a>
<br/>
<a href="https://opencollective.com/jellyfin"><img alt="捐赠" src="https://img.shields.io/opencollective/all/jellyfin.svg?label=backers"/></a>
<a href="https://features.jellyfin.org"><img alt="提交功能请求" src="https://img.shields.io/badge/fider-vote%20on%20features-success.svg"/></a>
<a href="https://matrix.to/#/#jellyfinorg:matrix.org"><img alt="在 Matrix 聊天" src="https://img.shields.io/matrix/jellyfinorg:matrix.org.svg?logo=matrix"/></a>
<a href="https://github.com/jellyfin/jellyfin/releases.atom"><img alt="发行版 RSS" src="https://img.shields.io/badge/rss-releases-ffa500?logo=rss" /></a>
<a href="https://github.com/jellyfin/jellyfin/commits/master.atom"><img alt="Master 提交 RSS" src="https://img.shields.io/badge/rss-commits-ffa500?logo=rss" /></a>
</p>

---

Jellyfin 是自由软件媒体系统，让你自行掌控媒体的管理和串流。它是专有软件 Emby 和 Plex 的替代方案，通过多个应用将专用服务器上的媒体提供给终端设备。Jellyfin 派生自 Emby 3.5.2 版本，并移植到 .NET 平台以实现完整的跨平台支持。

这里没有附加条件、付费高级许可证或功能，也没有隐藏目的：只有一群想共同创造更好产品的人。我们欢迎任何有兴趣的人加入我们的探索！

详情请参阅[文档](https://jellyfin.org/docs/)。如需获取最新动态、寻求 Jellyfin 帮助并加入社区，请访问我们的[交流渠道](https://jellyfin.org/docs/general/getting-help)。项目介绍请参阅[关于页面](https://jellyfin.org/docs/general/about)。

<strong>想要开始使用？</strong><br/>
访问[下载页面](https://jellyfin.org/downloads)或[安装指南](https://jellyfin.org/docs/general/installation/)，再查看[快速入门](https://jellyfin.org/docs/general/quick-start)。你也可以[从源代码构建](https://jellyfin.org/docs/general/installation/source)。<br/>

<strong>遇到问题？</strong><br/>
在 GitHub 上[提交 Issue](https://jellyfin.org/docs/general/contributing/issues)。<br/>

<strong>想要贡献？</strong><br/>
通过[贡献指引](https://jellyfin.org/contribute)寻找可以帮忙的地方，然后阅读[贡献指南](https://jellyfin.org/docs/general/contributing/)和[社区标准](https://jellyfin.org/docs/general/community-standards)。<br/>

<strong>有新想法或改进建议？</strong><br/>
查看[功能请求中心](https://features.jellyfin.org/?view=most-wanted)。<br/>

<strong>没有你的语言？</strong><br/>
访问我们的 [Weblate](https://translate.jellyfin.org)，帮助翻译 Jellyfin 及其子项目。<br/>

<a href="https://translate.jellyfin.org/engage/jellyfin/?utm_source=widget"><img src="https://translate.jellyfin.org/widgets/jellyfin/-/jellyfin-web/multi-auto.svg" alt="详细翻译状态"/></a>

---

## Jellyfin 服务器

本仓库包含 Jellyfin 后端服务器的代码。它只是 Jellyfin GitHub [组织](https://github.com/jellyfin/)下的众多项目之一。如果想贡献，可以先查看[文档](https://jellyfin.org/docs/general/contributing/index.html)，了解有哪些工作可做。

## 服务器开发

以下说明帮助你建立本地开发环境，以便为本仓库做贡献。开始之前，请完整阅读[开发贡献指南](https://jellyfin.org/docs/general/contributing/development.html)。本项目支持所有主要操作系统，但 FreeBSD 仍不兼容。

### 前提条件

构建项目前，必须先安装 [.NET 10 SDK](https://dotnet.microsoft.com/download/dotnet)。

这里包含从命令行运行项目的说明，但若要在服务器运行时调试，还需安装 IDE。任何支持 .NET 6 开发的 IDE 都可以，例如较新版本的 [Visual Studio](https://visualstudio.microsoft.com/downloads/)（至少 2022）和 [Visual Studio Code](https://code.visualstudio.com/Download)。

还需要安装 [ffmpeg](https://github.com/jellyfin/jellyfin-ffmpeg)。

### 克隆仓库

安装依赖后，需要克隆一份本仓库到本地。如果只想从源代码运行服务器，可直接克隆；如果计划贡献代码，请先[创建自己的 fork](https://jellyfin.org/docs/general/contributing/development.html#set-up-your-copy-of-the-repo)。以下示例展示如何通过 HTTPS 直接克隆。

```bash
git clone https://github.com/jellyfin/jellyfin.git
```

### 安装 Web 客户端

除提供后端服务外，服务器默认配置为托管 [Web 客户端](https://github.com/jellyfin/jellyfin-web)所需的静态文件。运行服务器前需要获取 Web 客户端，因为这些文件未直接包含在本仓库中。

开发时建议通过额外配置[单独托管 Web 客户端](#hosting-the-web-client-separately)，这样可以跳过本步骤。

获取 Web 客户端文件有两种方式：

1. 按照 [jellyfin-web 仓库](https://github.com/jellyfin/jellyfin-web)中的说明从源代码构建。
2. 从现有服务器安装中获取预构建文件。例如 Windows 服务器安装的客户端文件位于 `C:\Program Files\Jellyfin\Server\jellyfin-web`。

### 运行服务器

以下说明帮助你通过命令行或喜欢的 IDE 启动并运行项目。

#### 使用 Visual Studio 运行

打开解决方案（`.slnx`）文件，然后按 `F5` 运行服务器。

#### 使用 Visual Studio Code 运行

首先使用 `Open Folder...` 选项在 Visual Studio Code 中打开仓库目录。

然后[安装工作区推荐扩展](https://code.visualstudio.com/docs/editor/extension-gallery#_recommended-extensions)。扩展建议分为“Workspace Recommendations”和“Other Recommendations”，只需要前者。

安装所需扩展后，按 `F5` 运行服务器。

#### 从命令行运行

使用 `dotnet run` 命令从命令行运行服务器。以下示例假设仓库克隆到名为 `jellyfin` 的目录（默认目录名），应适用于所有操作系统。

```bash
cd jellyfin                          # Move into the repository directory
dotnet run --project Jellyfin.Server --webdir /absolute/path/to/jellyfin-web/dist # Run the server startup project
```

另一种方式是构建项目，然后直接运行生成的可执行文件。直接运行时可以方便地添加命令行选项。添加 `--help` 标志可列出所有支持的命令行选项详情。

1. 构建项目

```bash
dotnet build                       # Build the project
cd Jellyfin.Server/bin/Debug/net10.0 # Change into the build output directory
```

2. 执行构建产物。在 Linux、Mac 等系统使用 `./jellyfin`，在 Windows 使用 `jellyfin.exe`。

#### 访问托管的 Web 客户端

如果服务器已配置为托管 Web 客户端并正在运行，默认可通过 `http://localhost:8096` 访问 Web 客户端。

API 文档可在 `http://localhost:8096/api-docs/swagger/index.html` 查看。

### 在 GitHub Codespaces 中运行

由于 Jellyfin 将在 GitHub 托管服务器的容器中运行，JF 需要以不同方式处理一些事情。

**注意：**根据所选配置（如果直接点击“create codespace”，将创建默认配置），在 VS Code 已打开时，加载全部扩展并准备环境可能需要 20–30 秒。请耐心等待，直到输出标签页显示 `Downloading .NET version(s) 7.0.15~x64 ...... Done!`。

**注意：**如果想从外部访问 JF 实例，例如通过另一台电脑的 Web 客户端，请记得将 VS Code 窗口下方的“ports”设置为公开。

**注意：**首次使用任何 WebUI 打开服务器实例时，会进入登录页面而不是设置页面。刷新一次登录页，应会重定向到设置页面。

可以选择两种配置：

#### 默认配置——开发用 Jellyfin 服务器

创建的容器具备运行和调试 Jellyfin 媒体服务器所需的一切，但不设置其他内容。每次创建新容器都需要重新完成整个设置。也没有预加载 ffmpeg、Web 客户端或媒体。使用 `.NET Launch (nowebclient)` 启动配置启动服务器。

> 请注意，由于没有 Web 客户端，你必须通过外部客户端连接，例如另一个运行 WebUI 的 codespace 容器。vuejs 无法开箱即用，因为它不支持设置步骤。

#### 带 ffmpeg 的开发用 Jellyfin 服务器

此配置在默认服务器上按此处描述的方式安装默认 ffmpeg6：https://jellyfin.org/docs/general/installation/linux#repository-manual

如果想安装特定版本的 ffmpeg，请遵循 `.devcontainer/Dev - Server Ffmpeg/install.ffmpeg.sh` 文件中的注释。

使用 `ghcs .NET Launch (nowebclient, ffmpeg)` 启动配置，运行启用 jellyfin-ffmpeg 的服务器。

### 运行测试

本仓库还包含单元测试，作为 Azure CI 流水线的一部分验证功能。可以通过以下几种方式运行：

1. 从命令行使用 `dotnet test` 运行测试。
2. 在 Visual Studio 中使用[测试资源管理器](https://docs.microsoft.com/en-us/visualstudio/test/run-unit-tests-with-test-explorer)运行测试。
3. 在 Visual Studio Code 中使用相应的 [CodeLens 注解](https://github.com/OmniSharp/omnisharp-vscode/wiki/How-to-run-and-debug-unit-tests)运行单个测试。

### 高级配置

以下部分介绍基于标准说明从源代码运行服务器的一些高级场景。

<a id="hosting-the-web-client-separately"></a>
#### 单独托管 Web 客户端

无需将前端 Web 客户端作为后端服务器的一部分托管。分别托管这两个组件可能对前端开发人员有用，他们可以在单独的 webpack 开发服务器中托管客户端，以缩短开发循环。具体说明参见 [jellyfin-web](https://github.com/jellyfin/jellyfin-web#getting-started) 仓库。

要指示服务器不托管 Web 内容，需要设置 `nowebclient` 配置标志。可以使用命令行开关 `--nowebclient` 或环境变量 `JELLYFIN_NOWEBCONTENT=true` 指定。

由于这是常见场景，Visual Studio 中还定义了单独的 `Jellyfin.Server (nowebcontent)` 启动配置文件，可从主工具栏的“Start Debugging”下拉菜单中选择。

**注意：**Web 客户端单独托管时无法运行设置向导。

---
<p align="center">
本项目得到以下支持：
<br/><br/>
<a href="https://www.jetbrains.com"><img src="https://gist.githubusercontent.com/anthonylavado/e8b2403deee9581e0b4cb8cd675af7db/raw/199ae22980ef5da64882ec2de3e8e5c03fe535b8/jetbrains.svg" height="50px" alt="JetBrains 标志"></a>
</p>
