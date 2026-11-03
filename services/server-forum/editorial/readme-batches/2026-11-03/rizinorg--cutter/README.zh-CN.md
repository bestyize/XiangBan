<img width="150" height="150" align="left" style="float: left; margin: 0 10px 0 0;" alt="Cutter 标志" src="https://raw.githubusercontent.com/rizinorg/cutter/1fbee2e0d688eb12110e4301c993aa5a04a6a29e/src/img/cutter.svg?sanitize=true">

# Cutter

Cutter 是一个由 [rizin](https://github.com/rizinorg/rizin) 驱动的免费开源逆向工程平台。它旨在提供先进、可定制的逆向工程平台，同时兼顾用户体验。Cutter 由逆向工程师为逆向工程师创建。

[![Cutter CI](https://github.com/rizinorg/cutter/workflows/Cutter%20CI/badge.svg)](https://github.com/rizinorg/cutter/actions?query=workflow%3A%22Cutter+CI%22)
[![Build status](https://ci.appveyor.com/api/projects/status/tn7kttv55b8wf799/branch/dev?svg=true)](https://ci.appveyor.com/project/rizinorg/cutter/branch/dev)

![截图](https://raw.githubusercontent.com/rizinorg/cutter/1fbee2e0d688eb12110e4301c993aa5a04a6a29e/docs/source/images/screenshot.png)

## 在 [cutter.re](https://cutter.re) 了解更多。

## 获取 Cutter
### 下载

Cutter 的主要平台（Linux、macOS、Windows）发布版二进制文件可以从 [GitHub Releases](https://github.com/rizinorg/cutter/releases) 下载。

- **Linux**：如果你的发行版提供此软件，请在包管理器中查找 `cutter` 包（或 `cutter-re` / `rz-cutter`）。如果没有，我们为一些常见发行版在 [OBS](https://openbuildservice.org/) 上设置了仓库。请访问 [https://software.opensuse.org/package/cutter-re](https://software.opensuse.org/download/package?package=cutter-re&project=home%3ARizinOrg) 并按照说明操作。否则，可以从发布版本下载 `.AppImage` 文件，赋予其可执行权限并如下运行，或使用 [AppImageLauncher](https://github.com/TheAssassin/AppImageLauncher)。

    `chmod +x Cutter*.AppImage; ./Cutter*.AppImage`
- **macOS**：下载 `.dmg` 文件或使用 [Homebrew Cask](https://github.com/Homebrew/homebrew-cask)：

    `brew install --cask cutter`
- **Windows**：下载 `.zip` 压缩包，或使用 [Chocolatey](https://chocolatey.org) 或 [Scoop](https://scoop.sh/)：

    `choco install cutter`

    先运行 `scoop bucket add extras`，再运行 `scoop install cutter`

### 从源代码构建

要从源代码构建 Cutter，请查看[构建文档](https://cutter.re/docs/building.html)。

### Docker 镜像

要使用预构建的 `Dockerfile` 部署 *cutter*，可以使用[提供的配置](https://github.com/rizinorg/cutter/tree/1fbee2e0d688eb12110e4301c993aa5a04a6a29e/docker)。对应的 `README.md` 也包含快速开始使用 Docker 镜像的说明。

## 文档

### [用户指南](https://cutter.re/docs/user-docs.html)

### [贡献指南](https://cutter.re/docs/contributing.html)

### [开发者文档](https://cutter.re/docs/contributing/code.html)

## 插件
Cutter 同时支持 Python 和原生 C++ 插件。

我们的社区为 Cutter 构建了许多插件和实用脚本，例如 [Ghidra 反编译器](https://github.com/rizinorg/rz-ghidra)的原生集成，以及用于可视化 DynamoRIO 代码覆盖率的插件。你可以在下面找到 Cutter 插件列表，也欢迎将自己的 Cutter 插件和脚本加入列表。

**[官方和社区插件](https://github.com/rizinorg/cutter-plugins)**

**[插件开发指南](https://cutter.re/docs/plugins.html)**

## 获取帮助

请通过以下渠道向 Cutter 开发者和社区求助：

- **Telegram：** https://t.me/cutter_re
- **Mattermost：** https://im.rizin.re
- **IRC：** https://web.libera.chat/ 上的 #cutter
- **Twitter：** [@cutter_re](https://twitter.com/cutter_re)
