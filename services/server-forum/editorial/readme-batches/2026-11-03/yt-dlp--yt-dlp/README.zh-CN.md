<a id="readme"></a>

> 中文译文依据 [yt-dlp 固定版本 README](https://github.com/yt-dlp/yt-dlp/blob/b77c9304e1c4220cca450c583491904daf8455af/README.md) 整理，项目许可见 [Unlicense](https://github.com/yt-dlp/yt-dlp/blob/b77c9304e1c4220cca450c583491904daf8455af/LICENSE)。命令示例中的自然语言注释已翻译，命令和参数保持原样；原文可供对照。上游 README 中的版本、特性和支持情况以此版本为准，使用时请核对最新文档。

<!-- MANPAGE: BEGIN EXCLUDED SECTION -->
<div align="center">

[![YT-DLP](https://raw.githubusercontent.com/yt-dlp/yt-dlp/b77c9304e1c4220cca450c583491904daf8455af/.github/banner.svg)](#readme)

[![Release version](https://img.shields.io/github/v/release/yt-dlp/yt-dlp?color=brightgreen&label=Latest&style=for-the-badge)](#installation "Installation")
[![Python Version](https://img.shields.io/python/required-version-toml?tomlFilePath=https%3A%2F%2Fraw.githubusercontent.com%2Fyt-dlp%2Fyt-dlp%2Frefs%2Fheads%2Fmaster%2Fpyproject.toml&style=for-the-badge)](https://github.com/yt-dlp/yt-dlp/blob/b77c9304e1c4220cca450c583491904daf8455af/pyproject.toml "Python Version")
[![PyPI](https://img.shields.io/badge/-PyPI-blue.svg?logo=pypi&labelColor=555555&style=for-the-badge)](https://pypi.org/project/yt-dlp "PyPI")
[![Discord](https://img.shields.io/badge/Discord-%235865F2.svg?&logo=discord&logoColor=white&style=for-the-badge)](https://discord.gg/H5MNcFW63r "Discord")
[![License: Unlicense](https://img.shields.io/badge/-Unlicense-red.svg?style=for-the-badge)](https://github.com/yt-dlp/yt-dlp/blob/b77c9304e1c4220cca450c583491904daf8455af/LICENSE "License")
[![Commits](https://img.shields.io/github/commit-activity/m/yt-dlp/yt-dlp?label=commits&style=for-the-badge)](https://github.com/yt-dlp/yt-dlp/commits "Commit History")

</div>
<!-- MANPAGE: END EXCLUDED SECTION -->

yt-dlp 是一款功能丰富的命令行音视频下载器，支持[数千个网站](https://github.com/yt-dlp/yt-dlp/blob/b77c9304e1c4220cca450c583491904daf8455af/supportedsites.md)。项目是基于现已停止维护的 [youtube-dlc](https://github.com/blackjack4494/yt-dlc) 的 [youtube-dl](https://github.com/ytdl-org/youtube-dl) 分叉。

<!-- MANPAGE: MOVE "USAGE AND OPTIONS" SECTION HERE -->

<!-- MANPAGE: BEGIN EXCLUDED SECTION -->
* [安装](#installation)
    * [详细说明](https://github.com/yt-dlp/yt-dlp/wiki/Installation)
    * [发布文件](#release-files)
    * [更新](#update)
    * [依赖](#dependencies)
    * [编译](#compile)
* [用法与选项](#usage-and-options)
    * [通用选项](#general-options)
    * [网络选项](#network-options)
    * [地理限制](#geo-restriction)
    * [视频选择](#video-selection)
    * [下载选项](#download-options)
    * [文件系统选项](#filesystem-options)
    * [缩略图选项](#thumbnail-options)
    * [互联网快捷方式选项](#internet-shortcut-options)
    * [输出详细程度和模拟选项](#verbosity-and-simulation-options)
    * [变通选项](#workarounds)
    * [视频格式选项](#video-format-options)
    * [字幕选项](#subtitle-options)
    * [认证选项](#authentication-options)
    * [后处理选项](#post-processing-options)
    * [SponsorBlock 选项](#sponsorblock-options)
    * [提取器选项](#extractor-options)
    * [预设别名](#preset-aliases)
* [配置](#configuration)
    * [配置文件编码](#configuration-file-encoding)
    * [使用 netrc 认证](#authentication-with-netrc)
    * [环境变量说明](#notes-about-environment-variables)
* [输出模板](#output-template)
    * [输出模板示例](#output-template-examples)
* [格式选择](#format-selection)
    * [过滤格式](#filtering-formats)
    * [格式排序](#sorting-formats)
    * [格式选择示例](#format-selection-examples)
* [修改元数据](#modifying-metadata)
    * [修改元数据示例](#modifying-metadata-examples)
* [提取器参数](#extractor-arguments)
* [插件](#plugins)
    * [安装插件](#installing-plugins)
    * [开发插件](#developing-plugins)
* [嵌入 YT-DLP](#embedding-yt-dlp)
    * [嵌入示例](#embedding-examples)
* [与 youtube-dl 的变化](#changes-from-youtube-dl)
    * [新功能](#new-features)
    * [默认行为的区别](#differences-in-default-behavior)
    * [已弃用的选项](#deprecated-options)
* [参与贡献](https://github.com/yt-dlp/yt-dlp/blob/b77c9304e1c4220cca450c583491904daf8455af/CONTRIBUTING.md#contributing-to-yt-dlp)
    * [提交 Issue](https://github.com/yt-dlp/yt-dlp/blob/b77c9304e1c4220cca450c583491904daf8455af/CONTRIBUTING.md#opening-an-issue)
    * [开发者指南](https://github.com/yt-dlp/yt-dlp/blob/b77c9304e1c4220cca450c583491904daf8455af/CONTRIBUTING.md#developer-instructions)
* [WIKI](https://github.com/yt-dlp/yt-dlp/wiki)
    * [常见问题](https://github.com/yt-dlp/yt-dlp/wiki/FAQ)
<!-- MANPAGE: END EXCLUDED SECTION -->

<a id="installation"></a>
# 安装

<!-- MANPAGE: BEGIN EXCLUDED SECTION -->
[![Windows](https://img.shields.io/badge/-Windows_x64-blue.svg?style=for-the-badge&logo=windows)](https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe)
[![Unix](https://img.shields.io/badge/-Linux/BSD-red.svg?style=for-the-badge&logo=linux)](https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp)
[![MacOS](https://img.shields.io/badge/-MacOS-lightblue.svg?style=for-the-badge&logo=apple)](https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_macos)
[![PyPI](https://img.shields.io/badge/-PyPI-blue.svg?logo=pypi&labelColor=555555&style=for-the-badge)](https://pypi.org/project/yt-dlp)
[![Source Tarball](https://img.shields.io/badge/-Source_tar-green.svg?style=for-the-badge)](https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.tar.gz)
[![Other variants](https://img.shields.io/badge/-Other-grey.svg?style=for-the-badge)](#release-files)
[![All versions](https://img.shields.io/badge/-All_Versions-lightgrey.svg?style=for-the-badge)](https://github.com/yt-dlp/yt-dlp/releases)
<!-- MANPAGE: END EXCLUDED SECTION -->

可以使用[二进制文件](#发布文件)、[pip](https://pypi.org/project/yt-dlp)或第三方包管理器安装 yt-dlp。详细说明请参阅 [Wiki](https://github.com/yt-dlp/yt-dlp/wiki/Installation)。

<a id="release-files"></a>
## 发布文件

#### 推荐

文件|说明
:---|:---
[yt-dlp](https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp)|跨平台 [zipimport](https://docs.python.org/3/library/zipimport.html) 二进制文件。需要 Python（推荐用于 **Linux/BSD**）
[yt-dlp.exe](https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe)|Windows (Win10+) 独立 x64 二进制文件（推荐用于 **Windows**）
[yt-dlp_macos](https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_macos)|通用 MacOS (10.15+) 独立可执行文件（推荐用于 **MacOS**）

#### 其他选择

文件|说明
:---|:---
[yt-dlp_linux](https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_linux)|Linux (glibc 2.17+) 独立 x86_64 二进制文件
[yt-dlp_linux.zip](https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_linux.zip)|未打包的 Linux (glibc 2.17+) x86_64 可执行文件（不自动更新）
[yt-dlp_linux_aarch64](https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_linux_aarch64)|Linux (glibc 2.17+) 独立 aarch64 二进制文件
[yt-dlp_linux_aarch64.zip](https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_linux_aarch64.zip)|未打包的 Linux (glibc 2.17+) aarch64 可执行文件（不自动更新）
[yt-dlp_linux_armv7l.zip](https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_linux_armv7l.zip)|未打包的 Linux (glibc 2.31+) armv7l 可执行文件（不自动更新）
[yt-dlp_musllinux](https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_musllinux)|Linux (musl 1.2+) 独立 x86_64 二进制文件
[yt-dlp_musllinux.zip](https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_musllinux.zip)|未打包的 Linux (musl 1.2+) x86_64 可执行文件（不自动更新）
[yt-dlp_musllinux_aarch64](https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_musllinux_aarch64)|Linux (musl 1.2+) 独立 aarch64 二进制文件
[yt-dlp_musllinux_aarch64.zip](https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_musllinux_aarch64.zip)|未打包的 Linux (musl 1.2+) aarch64 可执行文件（不自动更新）
[yt-dlp_x86.exe](https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_x86.exe)|Windows (Win10+) 独立 x86（32 位）二进制文件
[yt-dlp_win_x86.zip](https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_win_x86.zip)|未打包的 Windows (Win10+) x86（32 位）可执行文件（不自动更新）
[yt-dlp_arm64.exe](https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_arm64.exe)|Windows (Win10+) 独立 ARM64 二进制文件
[yt-dlp_win_arm64.zip](https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_win_arm64.zip)|未打包的 Windows (Win10+) ARM64 可执行文件（不自动更新）
[yt-dlp_win.zip](https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_win.zip)|未打包的 Windows (Win10+) x64 可执行文件（不自动更新）
[yt-dlp_macos.zip](https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_macos.zip)|未打包的 MacOS (10.15+) 可执行文件（不自动更新）

#### 其他

文件|说明
:---|:---
[yt-dlp.tar.gz](https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.tar.gz)|源代码 tarball
[SHA2-512SUMS](https://github.com/yt-dlp/yt-dlp/releases/latest/download/SHA2-512SUMS)|GNU 格式 SHA512 校验值
[SHA2-512SUMS.sig](https://github.com/yt-dlp/yt-dlp/releases/latest/download/SHA2-512SUMS.sig)|SHA512 校验值的 GPG 签名文件
[SHA2-256SUMS](https://github.com/yt-dlp/yt-dlp/releases/latest/download/SHA2-256SUMS)|GNU 格式 SHA256 校验值
[SHA2-256SUMS.sig](https://github.com/yt-dlp/yt-dlp/releases/latest/download/SHA2-256SUMS.sig)|SHA256 校验值的 GPG 签名文件

可用于验证 GPG 签名的公钥[在此获取](https://github.com/yt-dlp/yt-dlp/blob/b77c9304e1c4220cca450c583491904daf8455af/public.key)。使用示例：
```
curl -L https://github.com/yt-dlp/yt-dlp/raw/master/public.key | gpg --import
gpg --verify SHA2-256SUMS.sig SHA2-256SUMS
gpg --verify SHA2-512SUMS.sig SHA2-512SUMS
```

#### 许可

虽然 yt-dlp 采用 [Unlicense](https://github.com/yt-dlp/yt-dlp/blob/b77c9304e1c4220cca450c583491904daf8455af/LICENSE)，但许多发布文件包含来自其他项目、采用不同许可证的代码。

尤其是，使用 PyInstaller 打包的可执行文件包含 GPLv3+ 许可的代码，因此组合后的作品采用 [GPLv3+](https://www.gnu.org/licenses/gpl-3.0.html) 许可。

zipimport Unix 可执行文件（`yt-dlp`）和发布版 tarball（`yt-dlp.tar.gz`）包含来自 [`meriyah`](https://github.com/meriyah/meriyah) 的 [ISC](https://github.com/meriyah/meriyah/blob/main/LICENSE.md) 许可代码，以及来自 [`astring`](https://github.com/davidbonnet/astring) 的 [MIT](https://github.com/davidbonnet/astring/blob/main/LICENSE) 许可代码。

详情见 [THIRD_PARTY_LICENSES.txt](https://github.com/yt-dlp/yt-dlp/blob/b77c9304e1c4220cca450c583491904daf8455af/THIRD_PARTY_LICENSES.txt)。Git 仓库、PyPI 源代码发行包和 PyPI 构建发行包（wheel）只包含按 [Unlicense](https://github.com/yt-dlp/yt-dlp/blob/b77c9304e1c4220cca450c583491904daf8455af/LICENSE) 许可的代码。

**注意**：man 手册页、shell 补全文件等包含在[源代码 tarball](https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.tar.gz)中。

<a id="update"></a>
## 更新

如果使用[发布版二进制文件](#发布文件)，可以使用 `yt-dlp -U` 更新。

如果[使用 pip 安装](https://github.com/yt-dlp/yt-dlp/wiki/Installation#with-pip)，只需重新运行最初安装程序时使用的相同命令。

其他第三方包管理器请参阅 [Wiki](https://github.com/yt-dlp/yt-dlp/wiki/Installation#third-party-package-managers)或其文档。

<a id="update-channels"></a>

目前二进制文件有三个发布渠道：`stable`、`nightly` 和 `master`。

* `stable` 是默认渠道，通常按月发布。虽然因许多更改已由 `nightly` 或 `master` 渠道用户测试而名为 `stable`，但最新 `stable` 版本往往会“过时”，容易因外部变化而失效（例如网站更改自身内容导致 yt-dlp 无法工作）。
* `nightly` 渠道在代码库有更改的日期，于 UTC 午夜前不久发布版本，作为项目开发状态的快照；这是**推荐普通 yt-dlp 用户使用的渠道**。可从 [yt-dlp/yt-dlp-nightly-builds](https://github.com/yt-dlp/yt-dlp-nightly-builds/releases)或 `yt-dlp` PyPI 包的开发版本获取（可用 pip 的 `--pre` 标志安装）。
* `master` 渠道在每次推送到 master 分支后发布“金丝雀”版本。它始终提供最新修复和功能，但可能存在错误或回归。可从 [yt-dlp/yt-dlp-master-builds](https://github.com/yt-dlp/yt-dlp-master-builds/releases)获取。

使用 `--update`/`-U` 时，发布版二进制文件只更新当前渠道。`--update-to CHANNEL` 可在有新版本时切换到其他渠道。`--update-to [CHANNEL@]TAG` 也可升级或降级到渠道中的特定标签。

还可以使用 `--update-to <repository>`（`<owner>/<repository>`）更新到完全不同仓库中的渠道。但要谨慎选择目标仓库：来自不同仓库的二进制文件不会经过验证。

示例：

* `yt-dlp --update-to master`：切换到 `master` 渠道并更新至最新发布版
* `yt-dlp --update-to stable@2023.07.06`：升级或降级到 `stable` 渠道的 `2023.07.06` 标签
* `yt-dlp --update-to 2023.10.07`：如果当前渠道存在 `2023.10.07` 标签，则升级或降级到该标签
* `yt-dlp --update-to example/yt-dlp@2023.09.24`：升级或降级到 `example/yt-dlp` 仓库的 `2023.09.24` 标签

**重要**：任何在 `stable` 版本遇到问题的用户，应在提交错误报告前安装或更新到 `nightly` 版本：
```
# 从稳定版可执行文件或二进制文件更新到 nightly：
yt-dlp --update-to nightly

# 使用 pip 安装 nightly：
python -m pip install -U --pre "yt-dlp[default]"
```

运行超过 90 天前发布的 yt-dlp 版本时，会显示建议更新至最新版本的警告。可在命令或配置文件中添加 `--no-update` 隐藏此警告。

<a id="dependencies"></a>
## 依赖

支持 Python 3.11+（CPython 和 PyPy）。其他版本和实现可能正常工作，也可能不能。

<!-- Python 3.5+ uses VC++14 and it is already embedded in the binary created
<!x-- https://www.microsoft.com/en-us/download/details.aspx?id=26999 --x>
在 Windows 上，运行 yt-dlp 还需要 [Microsoft Visual C++ 2010 SP1 Redistributable Package (x86)](https://download.microsoft.com/download/1/6/5/165255E7-1014-4D0A-B094-B6A430A6BFFC/vcredist_x86.exe)。你可能已经安装；如果可执行文件因缺少 `MSVCR100.dll` 报错，则需要手动安装。
-->

虽然其他依赖均为可选，但强烈推荐安装 `ffmpeg`、`ffprobe`、`yt-dlp-ejs` 和受支持的 JavaScript 运行时或引擎。

### 强烈推荐

* [**ffmpeg** 和 **ffprobe**](https://www.ffmpeg.org)——[合并分离的视频与音频文件](#格式选择)以及多种[后处理](#后处理选项)任务所必需。许可证[取决于构建方式](https://www.ffmpeg.org/legal.html)。

    因为 ffmpeg 非常重要，我们在 [yt-dlp/FFmpeg-Builds](https://github.com/yt-dlp/FFmpeg-Builds) 提供自己的构建。过去这些构建曾应用补丁以修复 yt-dlp 用户的常见问题，但目前与上游 ffmpeg 等同。详情见 [README](https://github.com/yt-dlp/FFmpeg-Builds#patches-applied)。

    **重要**：需要的是 ffmpeg *二进制文件*，**不是**[同名 Python 包](https://pypi.org/project/ffmpeg)。

* [**yt-dlp-ejs**](https://github.com/yt-dlp/ejs)——完整支持 YouTube 所必需。采用 [Unlicense](https://github.com/yt-dlp/ejs/blob/main/LICENSE)，并捆绑 [MIT](https://github.com/davidbonnet/astring/blob/main/LICENSE) 和 [ISC](https://github.com/meriyah/meriyah/blob/main/LICENSE.md) 组件。

    运行 yt-dlp-ejs 还需要 JavaScript 运行时或引擎，例如 [**deno**](https://deno.land)（推荐）、[**node.js**](https://nodejs.org)、[**bun**](https://bun.sh) 或 [**QuickJS**](https://bellard.org/quickjs/)。参阅 [Wiki](https://github.com/yt-dlp/yt-dlp/wiki/EJS)。

### 网络

* [**certifi**](https://github.com/certifi/python-certifi)\*——提供 Mozilla 根证书包，采用 [MPLv2](https://github.com/certifi/python-certifi/blob/master/LICENSE) 许可。
* [**brotli**](https://github.com/google/brotli)\* 或 [**brotlicffi**](https://github.com/python-hyper/brotlicffi)——支持 [Brotli](https://en.wikipedia.org/wiki/Brotli) 内容编码。两者均采用 MIT 许可 <sup>[1](https://github.com/google/brotli/blob/master/LICENSE) [2](https://github.com/python-hyper/brotlicffi/blob/master/LICENSE)</sup>。
* [**websockets**](https://github.com/aaugustin/websockets)\*——用于通过 WebSocket 下载，采用 [BSD-3-Clause](https://github.com/aaugustin/websockets/blob/main/LICENSE)。
* [**requests**](https://github.com/psf/requests)\*——HTTP 库，提供 HTTPS 代理和持久连接支持，采用 [Apache-2.0](https://github.com/psf/requests/blob/main/LICENSE)。

#### 模拟浏览器请求

以下组件提供模拟浏览器请求的支持。某些采用 TLS 指纹识别的网站可能需要此功能。

* [**curl_cffi**](https://github.com/lexiforest/curl_cffi)（推荐）——[curl-impersonate](https://github.com/lexiforest/curl-impersonate) 的 Python 绑定，提供 Chrome、Edge 和 Safari 模拟目标，采用 [MIT](https://github.com/lexiforest/curl_cffi/blob/main/LICENSE)。
  * 可以通过 `curl-cffi` extra 安装，例如 `pip install "yt-dlp[default,curl-cffi]"`。
  * 目前包含在大多数构建中，*不包括* `yt-dlp`（Unix zipimport 二进制文件）和 `yt-dlp_x86`（Windows 32 位）。

### 元数据

* [**mutagen**](https://github.com/quodlibet/mutagen)\*——用于在某些格式中实现 `--embed-thumbnail`，采用 [GPLv2+](https://github.com/quodlibet/mutagen/blob/master/COPYING)。
* [**AtomicParsley**](https://github.com/wez/atomicparsley)——在 `mutagen`/`ffmpeg` 无法处理时，为 `mp4`/`m4a` 文件提供 `--embed-thumbnail`，采用 [GPLv2+](https://github.com/wez/atomicparsley/blob/master/COPYING)。
* [**xattr**](https://github.com/xattr/xattr)、[**pyxattr**](https://github.com/iustin/pyxattr) 或 [**setfattr**](http://savannah.nongnu.org/projects/attr)——用于在 **Mac** 和 **BSD** 上写入 xattr 元数据（`--xattrs`），分别采用 [MIT](https://github.com/xattr/xattr/blob/master/LICENSE.txt)、[LGPL2.1](https://github.com/iustin/pyxattr/blob/master/COPYING) 和 [GPLv2+](http://git.savannah.nongnu.org/cgit/attr.git/tree/doc/COPYING) 许可。

### 其他

* [**pycryptodomex**](https://github.com/Legrandin/pycryptodome)\*——用于解密 AES-128 HLS 流及其他各种数据，采用 [BSD-2-Clause](https://github.com/Legrandin/pycryptodome/blob/master/LICENSE.rst)。
* [**phantomjs**](https://github.com/ariya/phantomjs)——某些需要运行 JavaScript 的提取器使用；YouTube 已不再使用，近期将弃用，采用 [BSD-3-Clause](https://github.com/ariya/phantomjs/blob/master/LICENSE.BSD)。
* [**secretstorage**](https://github.com/mitya57/secretstorage)\*——在 **Linux** 上解密基于 **Chromium** 的浏览器 Cookie 时，供 `--cookies-from-browser` 访问 **Gnome** 密钥环，采用 [BSD-3-Clause](https://github.com/mitya57/secretstorage/blob/master/LICENSE)。
* 想通过 `--downloader` 使用的任何外部下载器。

### 已弃用

* [**rtmpdump**](http://rtmpdump.mplayerhq.hu)——用于下载 `rtmp` 流。也可通过 `--downloader ffmpeg` 使用 ffmpeg，采用 [GPLv2+](http://rtmpdump.mplayerhq.hu)。

使用或重新分发依赖时，必须同意其各自的许可条款。

独立发布版二进制文件随附 Python 解释器以及标记 **\*** 的软件包。

如果正在尝试的任务缺少必要依赖，yt-dlp 会发出警告。当前可用的所有依赖列于 `--verbose` 输出开头。

<a id="compile"></a>
## 编译

### 独立 PyInstaller 构建

要构建独立可执行文件，必须具备 Python 和 `pyinstaller`（如有需要，还包括 yt-dlp 的[可选依赖](#依赖)）。生成的可执行文件将使用与 Python 相同的 CPU 架构。

可以运行以下命令：

```
python devscripts/install_deps.py --include-group pyinstaller
python devscripts/make_lazy_extractors.py
python -m bundle.pyinstaller
```

在某些系统上，可能需要使用 `py` 或 `python3` 而不是 `python`。

`python -m bundle.pyinstaller` 接受可传给 `pyinstaller` 的任何参数，例如 `--onefile/-F` 或 `--onedir/-D`，更多说明见[这里](https://pyinstaller.org/en/stable/usage.html#what-to-generate)。

**注意**：低于 4.4 的 PyInstaller 版本[不支持](https://github.com/pyinstaller/pyinstaller#requirements-and-tested-platforms)不使用虚拟环境时从 Windows 商店安装的 Python。

**重要**：直接运行 `pyinstaller` **而不是**使用 `python -m bundle.pyinstaller`，官方**不提供支持**。这样做可能正常，也可能不正常。

### 平台无关二进制文件（UNIX）

需要构建工具 `python`（3.11+）、`zip`、`make`（GNU）、`pandoc`\* 和 `pytest`\*。

安装后只需运行 `make`。

也可以运行 `make yt-dlp`，只编译二进制文件而不更新任何附加文件。（此时不需要标记 **\*** 的构建工具。）

### 相关脚本

* **`devscripts/install_deps.py`**——安装 yt-dlp 的依赖。
* **`devscripts/update-version.py`**——根据当前日期更新版本号。
* **`devscripts/set-variant.py`**——设置可执行文件的构建变体。
* **`devscripts/make_changelog.py`**——使用简短提交信息创建 Markdown 更新日志，并更新 `CONTRIBUTORS` 文件。
* **`devscripts/make_lazy_extractors.py`**——创建惰性提取器。在构建二进制文件（任何变体）前运行可改善启动性能。将环境变量 `YTDLP_NO_LAZY_EXTRACTORS` 设置为非空值，可强制禁用惰性提取器加载。

注意：更多信息请查看各脚本的 `--help`。

### Fork 项目

如果在 GitHub 上 fork 本项目，可以运行自己 fork 的[构建工作流](https://github.com/yt-dlp/yt-dlp/blob/b77c9304e1c4220cca450c583491904daf8455af/.github/workflows/build.yml)，自动将所选版本构建为产物；也可以运行[发布工作流](https://github.com/yt-dlp/yt-dlp/blob/b77c9304e1c4220cca450c583491904daf8455af/.github/workflows/release.yml)，或启用[每夜工作流](https://github.com/yt-dlp/yt-dlp/blob/b77c9304e1c4220cca450c583491904daf8455af/.github/workflows/release-nightly.yml)，创建完整的（预）发布版本。

<a id="usage-and-options"></a>
# 用法与选项

    yt-dlp [OPTIONS] [--] URL [URL...]

提示：使用 `CTRL`+`F`（或 `Command`+`F`）按关键词搜索。

<a id="general-options"></a>
## 通用选项

- `-h, --help`：输出此帮助文本并退出。
- `--version`：输出程序版本并退出。
- `-U, --update`：将程序更新到最新版本。
- `--no-update`：不检查更新（默认）。
- `--update-to [CHANNEL]@[TAG]`：升级或降级到特定版本。CHANNEL 也可以是仓库；省略时 CHANNEL 和 TAG 分别默认为 `stable` 和 `latest`。详情见“更新”。支持的渠道：stable、nightly、master。
- `-i, --ignore-errors`：忽略下载和后处理错误。即使后处理失败，下载也会被视为成功。
- `--no-abort-on-error`：下载出错时继续处理下一个视频，例如跳过播放列表中不可用的视频（默认）。
- `--abort-on-error`：发生错误时停止下载后续视频（别名：`--no-ignore-errors`）。
- `--list-extractors`：列出所有支持的提取器并退出。
- `--extractor-descriptions`：输出所有支持的提取器的说明并退出。
- `--use-extractors NAMES`：指定要使用的提取器名称，以逗号分隔。也可以使用正则表达式、`all`、`default` 和 `end`（结束 URL 匹配），例如 `--ies "holodex.*,end,youtube"`。名称前加 `-` 可排除，例如 `--ies default,-generic`。使用 `--list-extractors` 查看名称列表。（别名：`--ies`）
- `--default-search PREFIX`：为不完整的 URL 使用此前缀。例如 `gvsearch2:python` 会在 Google Videos 上搜索 `python` 并下载两个视频。使用 `auto` 让 yt-dlp 猜测（使用 `auto_warning` 在猜测时发出警告）。`error` 只抛出错误。默认值 `fixup_error` 会修复损坏的 URL；如果无法修复则报错，而不是进行搜索。
- `--ignore-config`：除 `--config-locations` 指定的文件外，不再加载配置文件。为保持向后兼容，如果系统配置文件中出现此选项，则不加载用户配置。（别名：`--no-config`）
- `--no-config-locations`：不加载任何自定义配置文件（默认）。在配置文件内部使用时，忽略当前文件此前定义的所有 `--config-locations`。
- `--config-locations PATH`：主配置文件的位置，可以是配置文件路径或其所在目录（`-` 表示标准输入）。可多次使用，也可在其他配置文件中使用。
- `--plugin-dirs DIR`：指定额外的插件搜索目录。可多次使用以添加多个目录。使用 `default` 搜索默认插件目录（默认）。
- `--no-plugin-dirs`：清除要搜索的插件目录，包括默认目录和此前通过 `--plugin-dirs` 提供的目录。
- `--js-runtimes RUNTIME[:PATH]`：启用额外的 JavaScript 运行时，并可选择指定其位置（二进制文件路径或所在目录）。可多次使用以启用多个运行时。支持的运行时按优先级从高到低为 deno、node、quickjs、bun；默认仅启用 `deno`。将使用已启用且可用的最高优先级运行时。如需在 `deno` 可用时使用较低优先级的运行时，需要先传入 `--no-js-runtimes`，再启用其他运行时。
- `--no-js-runtimes`：清除要启用的 JavaScript 运行时，包括默认值及此前通过 `--js-runtimes` 提供的运行时。
- `--remote-components COMPONENT`：允许 yt-dlp 在需要时获取的远程组件。如果使用官方可执行文件或已安装所需版本的 yt-dlp-ejs 包，目前无需此选项。可多次使用以允许多个组件。支持的值：`ejs:npm`（来自 npm 的外部 JavaScript 组件）、`ejs:github`（来自 yt-dlp-ejs GitHub 的外部 JavaScript 组件）。默认不允许任何远程组件。
- `--no-remote-components`：禁止获取所有远程组件，包括之前通过 `--remote-components` 或默认值允许的组件。
- `--flat-playlist`：不提取播放列表的 URL 结果条目；部分条目元数据可能缺失，也可能跳过下载。
- `--no-flat-playlist`：完整提取播放列表中的视频（默认）。
- `--live-from-start`：从头下载直播。目前为实验性功能，仅支持 YouTube、Twitch、TVer 和 mellow-fan。
- `--no-live-from-start`：从当前时间开始下载直播（默认）。
- `--wait-for-video MIN[-MAX]`：等待计划中的直播可用。传入重试间等待的最小秒数（或范围）。
- `--no-wait-for-video`：不等待计划中的直播（默认）。
- `--mark-watched`：将视频标记为已观看（即使使用 `--simulate` 也是如此）。
- `--no-mark-watched`：不将视频标记为已观看（默认）。
- `--color [STREAM:]POLICY`：是否在输出中使用颜色代码；可选地以 STREAM（`stdout` 或 `stderr`）作为前缀，将设置应用于相应输出。可选 `always`、`auto`（默认）、`never` 或 `no_color`（使用无颜色的终端序列）。使用 `auto-tty` 或 `no_color-tty` 可仅依据终端支持情况决定。可多次使用。
- `--compat-options OPTS`：通过撤销 yt-dlp 的某些更改，帮助保持与 youtube-dl 或 youtube-dlc 配置兼容。详情见“默认行为差异”。
- `--alias ALIASES OPTIONS`：为选项字符串创建别名。除非别名以短横线 `-` 开头，否则会添加 `--` 前缀。参数按 Python 字符串格式化迷你语言解析。例如，`--alias get-audio,-X "-S aext:{0},abr -x --audio-format {0}"` 创建接受参数的 `--get-audio` 和 `-X`，并展开为 `-S aext:ARG0,abr -x --audio-format ARG0`。所有已定义别名都列在 `--help` 输出中。别名选项可以触发其他别名，因此应避免定义递归选项。作为安全措施，每个别名最多可触发 100 次。此选项可多次使用。
- `-t, --preset-alias PRESET`：应用一组预定义选项，例如 `--preset-alias mp3`。可用预设：mp3、aac、mp4、mkv、sleep。更多信息见末尾“预设别名”部分。此选项可多次使用。

<a id="network-options"></a>
## 网络选项

- `--proxy URL`：使用指定的 HTTP/HTTPS/SOCKS 代理。要启用 SOCKS 代理，请指定正确的 scheme，例如 `socks5://user:pass@127.0.0.1:1080/`。传入空字符串（`--proxy ""`）以直接连接。
- `--socket-timeout SECONDS`：放弃前等待的时间，单位为秒。
- `--source-address IP`：要绑定的客户端 IP 地址。
- `--impersonate CLIENT[:OS]`：发起请求时要模拟的客户端，例如 `chrome`、`chrome-110`、`chrome:windows-10`。传入 `--impersonate=""` 可模拟任意客户端。请注意，强制所有请求使用模拟可能影响下载速度和稳定性。
- `--list-impersonate-targets`：列出可模拟的客户端。
- `-4, --force-ipv4`：所有连接使用 IPv4。
- `-6, --force-ipv6`：所有连接使用 IPv6。
- `--enable-file-urls`：启用 `file://` URL。出于安全原因，默认禁用。

<a id="geo-restriction"></a>
## 地理限制

- `--geo-verification-proxy URL`：使用此代理为某些有地理限制的网站验证 IP 地址。实际下载使用 `--proxy` 指定的默认代理（如果未指定则不使用代理）。
- `--xff VALUE`：如何伪造 `X-Forwarded-For` HTTP 标头以尝试绕过地理限制。可选 `default`（仅在已知有用时）、`never`、CIDR 格式的 IP 地址块，或两位 ISO 3166-2 国家代码。

<a id="video-selection"></a>
## 视频选择

- `-I, --playlist-items ITEM_SPEC`：要下载条目的 `playlist_index`，以逗号分隔。可用 `[START]:[STOP][:STEP]` 指定范围；为向后兼容，也支持 `START-STOP`。负索引从右侧计数，负 STEP 表示倒序下载。例如对长度为 15 的播放列表使用 `-I 1:3,7,-5::2`，将下载索引为 1、2、3、7、11、13、15 的条目。
- `--min-filesize SIZE`：文件小于 SIZE 时中止下载，例如 `50k` 或 `44.6M`。
- `--max-filesize SIZE`：文件大于 SIZE 时中止下载，例如 `50k` 或 `44.6M`。
- `--date DATE`：仅下载指定日期上传的视频。日期可以是 `YYYYMMDD` 或 `[now|today|yesterday][-N[day|week|month|year]]` 格式。例如 `--date today-2weeks` 仅下载两周前同一天上传的视频。
- `--datebefore DATE`：仅下载指定日期或之前上传的视频，接受与 `--date` 相同的日期格式。
- `--dateafter DATE`：仅下载指定日期或之后上传的视频，接受与 `--date` 相同的日期格式。
- `--match-filters FILTER`：通用视频过滤器。任何“输出模板”字段都可使用“过滤格式”中定义的运算符与数字或字符串比较。也可以只指定字段以匹配该字段存在的情况；使用 `!field` 检查字段不存在，使用 `&` 检查多个条件。需要时使用 `\` 转义 `&` 或引号。多次使用时，只要满足其中一个条件即匹配。例如 `--match-filters !is_live --match-filters "like_count>?100 & description~='(?i)\bcats \& dogs\b'"` 只匹配非直播视频，或者点赞数大于 100（或点赞字段不可用）且描述包含短语 `cats & dogs`（不区分大小写）的视频。使用 `--match-filters -` 可交互式询问是否下载每个视频。
- `--no-match-filters`：不使用任何 `--match-filters`（默认）。
- `--break-match-filters FILTER`：与 `--match-filters` 相同，但视频被拒绝时停止下载过程。
- `--no-break-match-filters`：不使用任何 `--break-match-filters`（默认）。
- `--no-playlist`：如果 URL 同时指向视频和播放列表，只下载视频。
- `--yes-playlist`：如果 URL 同时指向视频和播放列表，下载播放列表。
- `--age-limit YEARS`：仅下载适合给定年龄的视频。
- `--download-archive FILE`：只下载归档文件中未列出的视频，并将所有已下载视频的 ID 记录到该文件。
- `--no-download-archive`：不使用归档文件（默认）。
- `--max-downloads NUMBER`：下载 NUMBER 个文件后中止。
- `--break-on-existing`：遇到 `--download-archive` 所指定归档中已有的文件时停止下载过程。
- `--no-break-on-existing`：遇到归档中已有的文件时不停止下载过程（默认）。
- `--break-per-input`：使 `--max-downloads`、`--break-on-existing`、`--break-match-filters` 和自动编号针对每个输入 URL 重置。
- `--no-break-per-input`：`--break-on-existing` 等选项会终止整个下载队列。
- `--skip-playlist-after-errors N`：允许失败的次数；达到后跳过播放列表的剩余部分。

<a id="download-options"></a>
## 下载选项

- `-N, --concurrent-fragments N`：dash/hlsnative 视频应并发下载的分片数（默认 1）。
- `-r, --limit-rate RATE`：最大下载速率，单位为每秒字节数，例如 `50K` 或 `4.2M`。
- `--throttled-rate RATE`：最低下载速率，单位为每秒字节数；低于该值时认为发生限速，并重新提取视频数据，例如 `100K`。
- `-R, --retries RETRIES`：重试次数（默认 10）或 `infinite`。
- `--file-access-retries RETRIES`：文件访问错误时的重试次数（默认 3）或 `infinite`。
- `--fragment-retries RETRIES`：分片重试次数（默认 10）或 `infinite`（适用于 DASH、hlsnative 和 ISM）。
- `--retry-sleep [TYPE:]EXPR`：重试之间休眠的秒数，可选地以前缀指定适用的重试类型（`http`（默认）、`fragment`、`file_access`、`extractor`）。EXPR 可以是数字、`linear=START[:END[:STEP=1]]` 或 `exp=START[:END[:BASE=2]]`。可多次使用以设置不同类型的休眠时间，例如 `--retry-sleep linear=1::2 --retry-sleep fragment:exp=1:20`。
- `--skip-unavailable-fragments`：DASH、hlsnative 和 ISM 下载时跳过不可用分片（默认；别名：`--no-abort-on-unavailable-fragments`）。
- `--abort-on-unavailable-fragments`：分片不可用时中止下载（别名：`--no-skip-unavailable-fragments`）。
- `--keep-fragments`：下载完成后在磁盘保留已下载分片。
- `--no-keep-fragments`：下载完成后删除已下载分片（默认）。
- `--buffer-size SIZE`：下载缓冲区大小，例如 `1024` 或 `16K`（默认 `1024`）。
- `--resize-buffer`：从 `--buffer-size` 的初始值自动调整缓冲区大小（默认）。
- `--no-resize-buffer`：不自动调整缓冲区大小。
- `--http-chunk-size SIZE`：分块 HTTP 下载的块大小，例如 `10485760` 或 `10M`（默认禁用）。可能有助于绕过 Web 服务器施加的带宽限制（实验性）。
- `--playlist-random`：随机顺序下载播放列表视频。
- `--lazy-playlist`：在收到播放列表条目时处理。此选项会禁用 `n_entries`、`--playlist-random` 和 `--playlist-reverse`。
- `--no-lazy-playlist`：仅在整个播放列表解析完成后处理其中视频（默认）。
- `--hls-use-mpegts`：HLS 视频使用 mpegts 容器，使某些播放器可以边下载边播放，并降低下载中断时文件损坏的可能性。直播流默认启用。
- `--no-hls-use-mpegts`：HLS 视频不使用 mpegts 容器。下载非直播流时为默认值。
- `--download-sections REGEX`：仅下载匹配正则表达式的章节。`*` 前缀表示时间范围而不是章节；负时间戳从末尾计算。`*from-url` 可用于下载从 URL 提取的 `start_time` 与 `end_time` 之间的内容。需要 ffmpeg。可多次使用以下载多个片段，例如 `--download-sections "*10:15-inf" --download-sections "intro"`。
- `--downloader [PROTO:]NAME`：使用的外部下载器名称或路径，可选地以协议（`http`、`ftp`、`m3u8`、`dash`、`rtmp`）作为前缀，指定适用协议。目前支持 `native`、`aria2c`、`axel`、`curl`、`ffmpeg`、`httpie`、`wget`。可多次使用，为不同协议设置不同下载器。例如 `--downloader aria2c --downloader "dash,m3u8:native"` 将为 HTTP/FTP 使用 aria2c，为 DASH/M3U8 使用内置下载器。（别名：`--external-downloader`）
- `--downloader-args NAME:ARGS`：向外部下载器传递参数。用冒号 `:` 分隔下载器名称和参数。对于 ffmpeg，可使用与 `--postprocessor-args` 相同的语法将参数传到不同位置。可多次使用，为不同下载器提供不同参数。（别名：`--external-downloader-args`）

<a id="filesystem-options"></a>
## 文件系统选项

- `-a, --batch-file FILE`：包含待下载 URL 的文件（`-` 表示标准输入），每行一个 URL。以 `#`、`;` 或 `]` 开头的行被视为注释并忽略。
- `--no-batch-file`：不从批处理文件读取 URL（默认）。
- `-P, --paths [TYPES:]PATH`：文件下载路径。用冒号 `:` 分隔文件类型和路径，支持与 `--output` 相同的所有 TYPES。此外，还可指定 `home`（默认）和 `temp` 路径。所有中间文件先下载到 temp 路径，下载完成后最终文件再移到 home 路径。如果 `--output` 是绝对路径，此选项被忽略。
- `-o, --output [TYPES:]TEMPLATE`：输出文件名模板，详情见“输出模板”。
- `--output-na-placeholder TEXT`：`--output` 中不可用字段的占位符（默认 `NA`）。
- `--restrict-filenames`：将文件名限制为 ASCII 字符，并避免使用 `&` 和空格。
- `--no-restrict-filenames`：文件名允许 Unicode 字符、`&` 和空格（默认）。
- `--windows-filenames`：强制文件名与 Windows 兼容。
- `--no-windows-filenames`：仅对文件名进行最少的清理。
- `--trim-filenames LENGTH`：将文件名长度（不含扩展名）限制为指定字符数。
- `-w, --no-overwrites`：不覆盖任何文件。
- `--force-overwrites`：覆盖所有视频和元数据文件。此选项包含 `--no-continue`。
- `--no-force-overwrites`：不覆盖视频，但覆盖相关文件（默认）。
- `-c, --continue`：继续下载部分已下载文件或分片（默认）。
- `--no-continue`：不继续下载部分已下载分片。如果文件未分片，则从头重新下载整个文件。
- `--part`：使用 `.part` 文件，而不是直接写入输出文件（默认）。
- `--no-part`：不使用 `.part` 文件，直接写入输出文件。
- `--mtime`：使用 Last-modified 标头设置文件修改时间。
- `--no-mtime`：不使用 Last-modified 标头设置文件修改时间（默认）。
- `--write-description`：将视频描述写入 `.description` 文件。
- `--no-write-description`：不写入视频描述（默认）。
- `--write-info-json`：将视频元数据写入 `.info.json` 文件（其中可能包含个人信息）。
- `--no-write-info-json`：不写入视频元数据（默认）。
- `--write-playlist-metafiles`：使用 `--write-info-json`、`--write-description` 等时，除视频元数据外还写入播放列表元数据（默认）。
- `--no-write-playlist-metafiles`：使用 `--write-info-json`、`--write-description` 等时不写入播放列表元数据。
- `--clean-info-json`：从 infojson 移除文件名等部分内部元数据（默认）。
- `--no-clean-info-json`：将所有字段写入 infojson。
- `--write-comments`：获取视频评论并放入 infojson。如果已知提取速度很快，即使没有此选项也会获取评论。（别名：`--get-comments`）
- `--no-write-comments`：除非已知提取速度很快，否则不获取视频评论。（别名：`--no-get-comments`）
- `--load-info-json FILE`：包含视频信息的 JSON 文件（由 `--write-info-json` 选项创建）。
- `--cookies FILE`：采用 Netscape 格式、用于读取 Cookie 并写入 Cookie jar 的文件。
- `--no-cookies`：不从文件读取 Cookie，也不将 Cookie 写入文件（默认）。
- `--cookies-from-browser BROWSER[+KEYRING][:PROFILE][::CONTAINER]`：要从中加载 Cookie 的浏览器名称。目前支持 brave、chrome、chromium、edge、firefox、opera、safari、vivaldi、whale。也可以使用相应分隔符指定用于在 Linux 上解密 Chromium Cookie 的 KEYRING、加载 Cookie 的 PROFILE 名称或路径，以及 CONTAINER 名称（Firefox；`none` 表示无容器）。默认使用最近访问的配置文件中的所有容器。目前支持的密钥环：basictext、gnomekeyring、kwallet、kwallet5、kwallet6。
- `--no-cookies-from-browser`：不从浏览器加载 Cookie（默认）。
- `--cache-dir DIR`：yt-dlp 在文件系统中永久保存部分下载信息（例如客户端 ID 和签名）的位置。默认 `${XDG_CACHE_HOME}/yt-dlp`。
- `--no-cache-dir`：禁用文件系统缓存。
- `--rm-cache-dir`：删除所有文件系统缓存文件。

<a id="thumbnail-options"></a>
## 缩略图选项

- `--write-thumbnail`：将缩略图写入磁盘。
- `--no-write-thumbnail`：不将缩略图写入磁盘（默认）。
- `--write-all-thumbnails`：将所有缩略图格式写入磁盘。
- `--list-thumbnails`：列出各视频可用缩略图。除非使用 `--no-simulate`，否则执行模拟。

<a id="internet-shortcut-options"></a>
## 互联网快捷方式选项

- `--write-link`：根据当前平台写入互联网快捷方式文件（`.url`、`.webloc` 或 `.desktop`）。操作系统可能缓存 URL。
- `--write-url-link`：写入 Windows `.url` 互联网快捷方式。操作系统根据文件路径缓存 URL。
- `--write-webloc-link`：写入 macOS `.webloc` 互联网快捷方式。
- `--write-desktop-link`：写入 Linux `.desktop` 互联网快捷方式。

<a id="verbosity-and-simulation-options"></a>
## 输出详细程度和模拟选项

- `-q, --quiet`：启用安静模式。与 `--verbose` 同时使用时，将日志输出到 stderr。
- `--no-quiet`：禁用安静模式（默认）。
- `--no-warnings`：忽略警告。
- `-s, --simulate`：不下载视频，也不向磁盘写入任何内容。
- `--no-simulate`：即使使用打印或列出选项，也下载视频。
- `--ignore-no-formats-error`：忽略“No video formats”错误。即使视频实际上无法下载，也可用于提取元数据（实验性）。
- `--no-ignore-no-formats-error`：找不到可下载视频格式时抛出错误（默认）。
- `--skip-download`：不下载视频，但写入所有相关文件（别名：`--no-download`）。
- `-O, --print [WHEN:]TEMPLATE`：要输出到屏幕的字段名或输出模板；可选地以前缀指定输出时机，并用 `:` 分隔。WHEN 支持的值与 `--use-postprocessor` 相同（默认 `video`）。隐含 `--quiet`；除非使用 `--no-simulate` 或 WHEN 的后续阶段，否则也隐含 `--simulate`。可多次使用。
- `--print-to-file [WHEN:]TEMPLATE FILE`：将给定模板追加到文件。WHEN 和 TEMPLATE 的值与 `--print` 相同。FILE 使用与输出模板相同的语法。可多次使用。
- `-j, --dump-json`：安静模式，但为每个视频输出 JSON 信息。除非使用 `--no-simulate`，否则执行模拟。可用键的说明见“输出模板”。
- `-J, --dump-single-json`：安静模式，但为每个传入的 URL 或 infojson 输出 JSON 信息。除非使用 `--no-simulate`，否则执行模拟。如果 URL 指向播放列表，则整份播放列表信息输出为一行。
- `--force-write-archive`：只要没有错误，即使使用 `-s` 或其他模拟选项，也强制写入下载归档条目（别名：`--force-download-archive`）。
- `--newline`：将进度条输出为新行。
- `--no-progress`：不输出进度条。
- `--progress`：即使处于安静模式也显示进度条。
- `--console-title`：在控制台标题栏显示进度。
- `--progress-template [TYPES:]TEMPLATE`：进度输出模板，可选地以 `download:`（默认）、`download-title:`（控制台标题）、`postprocess:` 或 `postprocess-title:` 之一为前缀。视频字段可通过 `info` 键访问，进度属性可通过 `progress` 键访问。例如 `--console-title --progress-template "download-title:%(info.id)s-%(progress.eta)s"`。
- `--progress-delta SECONDS`：两次进度输出之间的时间（默认 0）。
- `-v, --verbose`：输出各种调试信息。
- `--dump-pages`：输出以 base64 编码的已下载页面，用于调试问题（输出非常详细）。
- `--write-pages`：将下载的中间页面写入当前目录下的文件，用于调试问题。
- `--print-traffic`：显示发送和读取的 HTTP 流量。

<a id="workarounds"></a>
## 变通选项

- `--encoding ENCODING`：强制使用指定编码（实验性）。
- `--legacy-server-connect`：明确允许与不支持 RFC 5746 安全重新协商的服务器建立 HTTPS 连接。
- `--no-check-certificates`：关闭 HTTPS 证书验证。
- `--prefer-insecure`：使用未加密连接获取视频信息。
- `--add-headers FIELD:VALUE`：指定自定义 HTTP 标头及其值，用冒号 `:` 分隔。可多次使用。
- `--bidi-workaround`：兼容缺少双向文本支持的终端。需要 PATH 中有 `bidiv` 或 `fribidi` 可执行文件。
- `--sleep-requests SECONDS`：数据提取期间两次请求之间休眠的秒数。
- `--sleep-interval SECONDS`：每次下载前休眠的秒数。与 `--max-sleep-interval` 一起使用时，这是最短休眠时间（别名：`--min-sleep-interval`）。
- `--max-sleep-interval SECONDS`：最长休眠秒数，只能与 `--min-sleep-interval` 一起使用。
- `--sleep-subtitles SECONDS`：每次下载字幕前休眠的秒数。

<a id="video-format-options"></a>
## 视频格式选项

- `-f, --format FORMAT`：视频格式代码，详情见“格式选择”。
- `-S, --format-sort SORTORDER`：按指定字段排序格式，详情见“格式排序”。
- `--format-sort-reset`：忽略先前用户指定的排序顺序，重置为默认值。
- `--format-sort-force`：强制用户指定的排序顺序优先于所有字段，详情见“格式排序”（别名：`--S-force`）。
- `--no-format-sort-force`：部分字段优先于用户指定的排序顺序（默认）。
- `--video-multistreams`：允许将多个视频流合并到一个文件。
- `--no-video-multistreams`：每个输出文件只下载一个视频流（默认）。
- `--audio-multistreams`：允许将多个音频流合并到一个文件。
- `--no-audio-multistreams`：每个输出文件只下载一个音频流（默认）。
- `--prefer-free-formats`：同等质量下优先选择自由容器的视频格式，而非非自由容器。配合 `-S ext` 使用，可不考虑质量而严格优先选择自由容器。
- `--no-prefer-free-formats`：不对自由容器给予特殊优先级（默认）。
- `--check-formats`：确保只从实际可下载的格式中选择。
- `--check-all-formats`：检查所有格式是否实际可下载。
- `--no-check-formats`：不检查格式是否实际可下载。
- `-F, --list-formats`：列出各视频的可用格式。除非使用 `--no-simulate`，否则执行模拟。
- `--merge-output-format FORMAT`：合并格式时可使用的容器，以 `/` 分隔，例如 `mp4/mkv`。无需合并时忽略。（当前支持：avi、flv、mkv、mov、mp4、webm）

<a id="subtitle-options"></a>
## 字幕选项

- `--write-subs`：写入字幕文件。
- `--no-write-subs`：不写入字幕文件（默认）。
- `--write-auto-subs`：写入自动生成的字幕文件（别名：`--write-automatic-subs`）。
- `--no-write-auto-subs`：不写入自动生成的字幕（默认；别名：`--no-write-automatic-subs`）。
- `--list-subs`：列出各视频的可用字幕。除非使用 `--no-simulate`，否则执行模拟。
- `--sub-format FORMAT`：字幕格式；接受以 `/` 分隔的格式优先顺序，例如 `srt` 或 `ass/srt/best`。
- `--sub-langs LANGS`：要下载的字幕语言（可以是正则表达式）或 `all`，用逗号分隔，例如 `--sub-langs "en.*,ja"`（其中 `en.*` 是匹配 `en` 后跟零个或多个任意字符的正则表达式）。可以在语言代码前加 `-`，将其从请求语言中排除，例如 `--sub-langs all,-live_chat`。使用 `--list-subs` 查看可用语言标签。

<a id="authentication-options"></a>
## 认证选项

- `-u, --username USERNAME`：使用此账号 ID 登录。
- `-p, --password PASSWORD`：账号密码。省略时 yt-dlp 会交互式询问。
- `-2, --twofactor TWOFACTOR`：双因素认证码。
- `-n, --netrc`：使用 `.netrc` 认证数据。
- `--netrc-location PATH`：`.netrc` 认证数据的位置，可以是文件路径或所在目录。默认 `~/.netrc`。
- `--netrc-cmd NETRC_CMD`：执行以获取提取器凭据的命令。
- `--video-password PASSWORD`：视频专用密码。
- `--ap-mso MSO`：Adobe Pass 多系统运营商（电视服务提供商）标识符，使用 `--ap-list-mso` 查看可用运营商。
- `--ap-username USERNAME`：多系统运营商账号登录名。
- `--ap-password PASSWORD`：多系统运营商账号密码。省略时 yt-dlp 会交互式询问。
- `--ap-list-mso`：列出所有支持的多系统运营商。
- `--client-certificate CERTFILE`：PEM 格式客户端证书文件路径，可包含私钥。
- `--client-certificate-key KEYFILE`：客户端证书的私钥文件路径。
- `--client-certificate-password PASSWORD`：客户端证书私钥的密码（如果已加密）。如果密钥已加密且未提供密码，yt-dlp 会交互式询问。

<a id="post-processing-options"></a>
## 后处理选项

- `-x, --extract-audio`：将视频文件转换为纯音频文件（需要 ffmpeg 和 ffprobe）。
- `--audio-format FORMAT`：使用 `-x` 时转换音频的格式。目前支持 `best`（默认）、`aac`、`alac`、`flac`、`m4a`、`mp3`、`opus`、`vorbis`、`wav`。可使用与 `--remux-video` 类似的语法指定多条规则。
- `--audio-quality QUALITY`：使用 `-x` 转换音频时指定 ffmpeg 音频质量。VBR 可输入 0（最佳）至 10（最差）的值，也可以指定码率，例如 `128K`（默认 5）。
- `--remux-video FORMAT`：必要时将视频重新封装到另一容器。目前支持 avi、flv、gif、mkv、mov、mp4、webm、aac、aiff、alac、flac、m4a、mka、mp3、ogg、opus、vorbis、wav。如果目标容器不支持视频或音频编解码器，重新封装会失败。可以指定多条规则，例如 `aac>m4a/mov>mp4/mkv` 会将 aac 封装为 m4a、mov 封装为 mp4，其余封装为 mkv。
- `--recode-video FORMAT`：必要时将视频重新编码为另一格式。语法和支持的格式与 `--remux-video` 相同。
- `--postprocessor-args NAME:ARGS`：向后处理器传递参数。用冒号 `:` 分隔后处理器或可执行文件名称与参数，以将参数传给指定对象。支持的后处理器（PP）为 Merger、ModifyChapters、SplitChapters、ExtractAudio、VideoRemuxer、VideoConvertor、Metadata、EmbedSubtitle、EmbedThumbnail、SubtitlesConvertor、ThumbnailsConvertor、FixupStretched、FixupM4a、FixupM3u8、FixupTimestamp、FixupDuration；支持的可执行文件为 AtomicParsley、FFmpeg 和 FFprobe。也可以指定 `PP+EXE:ARGS`，仅在指定后处理器使用该可执行文件时向它传递参数。此外，对 ffmpeg/ffprobe 可在前缀后添加 `_i`/`_o`，并可再跟数字，以在指定输入/输出文件前传参，例如 `--ppa "Merger+ffmpeg_i1:-v quiet"`。可多次使用，为不同后处理器指定不同参数。（别名：`--ppa`）
- `-k, --keep-video`：后处理完成后在磁盘保留中间视频文件。
- `--no-keep-video`：后处理后删除中间视频文件（默认）。
- `--post-overwrites`：覆盖后处理后的文件（默认）。
- `--no-post-overwrites`：不覆盖后处理后的文件。
- `--embed-subs`：将字幕嵌入视频（仅适用于 mp4、webm、mkv 视频）。
- `--no-embed-subs`：不嵌入字幕（默认）。
- `--embed-thumbnail`：将缩略图作为封面嵌入视频。
- `--no-embed-thumbnail`：不嵌入缩略图（默认）。
- `--embed-metadata`：将元数据嵌入视频文件。如果存在章节和 infojson，除非使用 `--no-embed-chapters`/`--no-embed-info-json`，否则也嵌入它们。（别名：`--add-metadata`）
- `--no-embed-metadata`：不向文件添加元数据（默认；别名：`--no-add-metadata`）。
- `--embed-chapters`：向视频文件添加章节标记（别名：`--add-chapters`）。
- `--no-embed-chapters`：不添加章节标记（默认；别名：`--no-add-chapters`）。
- `--embed-info-json`：将 infojson 作为附件嵌入 mkv/mka 视频文件。
- `--no-embed-info-json`：不将 infojson 作为附件嵌入视频文件。
- `--parse-metadata [WHEN:]FROM:TO`：从其他字段解析标题、艺术家等额外元数据；详情见“修改元数据”。WHEN 支持的值与 `--use-postprocessor` 相同（默认 `pre_process`）。
- `--replace-in-metadata [WHEN:]FIELDS REGEX REPLACE`：使用给定正则表达式替换元数据字段中的文本。可多次使用。WHEN 支持的值与 `--use-postprocessor` 相同（默认 `pre_process`）。
- `--xattrs`：将元数据写入视频文件的 xattrs（使用 Dublin Core 和 XDG 标准）。
- `--concat-playlist POLICY`：连接播放列表中的视频。可选 `never`、`always` 或 `multi_video`（默认；仅当视频组成一个节目时）。所有视频文件必须有相同的编解码器和流数量才能连接。可在 `--paths` 和 `--output` 中使用 `pl_video:` 前缀设置连接文件的输出文件名，详情见“输出模板”。
- `--fixup POLICY`：自动修复文件的已知问题。可选 `never`（不做任何事）、`warn`（仅发出警告）、`detect_or_warn`（默认；能修复则修复，否则警告）、`force`（即使文件已存在也尝试修复）。
- `--ffmpeg-location PATH`：ffmpeg 二进制文件位置，可以是文件路径或其所在目录。
- `--exec [WHEN:]CMD`：执行命令，可选地以执行时机作为前缀并用 `:` 分隔。WHEN 支持的值与 `--use-postprocessor` 相同（默认 `after_move`）。可以使用与输出模板相同的语法将任意字段作为命令参数；但出于安全原因，仅允许 `i`/`d`（有符号十进制整数）、`f`（浮点十进制）和 `q`（shell 引号转义）转换。如果未传入字段，`%(filepath,_filename|)q` 会附加到命令末尾。此选项可多次使用。
- `--no-exec`：移除之前定义的所有 `--exec`。
- `--convert-subs FORMAT`：将字幕转换为其他格式（当前支持 ass、lrc、srt、vtt）。使用 `--convert-subs none` 禁用转换（默认；别名：`--convert-subtitles`）。
- `--convert-thumbnails FORMAT`：将缩略图转换为其他格式（当前支持 jpg、png、webp）。可使用与 `--remux-video` 类似的语法指定多条规则。使用 `--convert-thumbnails none` 禁用转换（默认）。
- `--split-chapters`：根据内部章节将视频拆分为多个文件。可在 `--paths` 和 `--output` 中使用 `chapter:` 前缀设置拆分文件的输出文件名，详情见“输出模板”。
- `--no-split-chapters`：不按章节拆分视频（默认）。
- `--remove-chapters REGEX`：移除标题匹配给定正则表达式的章节。语法与 `--download-sections` 相同，可多次使用。
- `--no-remove-chapters`：不从文件移除任何章节（默认）。
- `--force-keyframes-at-cuts`：下载、拆分或移除片段时，在切点强制插入关键帧。因为需要重新编码，速度较慢，但生成的视频在切点附近可能有更少的伪影。
- `--no-force-keyframes-at-cuts`：裁切或拆分时不在章节附近强制插入关键帧（默认）。
- `--use-postprocessor NAME[:ARGS]`：要启用的插件后处理器名称（区分大小写），以及可选的传入参数，用冒号 `:` 分隔。ARGS 是以分号 `;` 分隔的 `NAME=VALUE` 列表。`when` 参数决定调用时机，可选 `pre_process`（视频提取后）、`after_filter`（视频通过过滤后）、`video`（`--format` 后、`--print`/`--output` 前）、`before_dl`（每个视频下载前）、`post_process`（每个视频下载后；默认）、`after_move`（视频文件移动到最终位置后）、`after_video`（某视频的所有格式下载并处理后）或 `playlist`（播放列表结束时）。可多次使用以添加不同后处理器。

<a id="sponsorblock-options"></a>
## SponsorBlock 选项

使用 [SponsorBlock API](https://sponsor.ajay.app) 为下载的 YouTube 视频中的各种片段（赞助、开头等）创建章节，或将它们移除。

- `--sponsorblock-mark CATS`：要创建章节的 SponsorBlock 类别，以逗号分隔。可用类别为 `sponsor`、`intro`、`outro`、`selfpromo`、`preview`、`filler`、`interaction`、`music_offtopic`、`hook`、`poi_highlight`、`chapter`、`all` 和 `default`（等于 `all`）。类别前加 `-` 可排除。类别说明见 [1]。例如 `--sponsorblock-mark all,-preview`。
  [1] https://wiki.sponsor.ajay.app/w/Segment_Categories
- `--sponsorblock-remove CATS`：要从视频文件中移除的 SponsorBlock 类别，以逗号分隔。如果某类别同时出现在标记和移除中，移除优先。语法和可用类别与 `--sponsorblock-mark` 相同，但 `default` 指 `all,-filler`，且不能使用 `poi_highlight`、`chapter`。
- `--sponsorblock-chapter-title TEMPLATE`：由 `--sponsorblock-mark` 创建的 SponsorBlock 章节标题输出模板。仅可使用 `start_time`、`end_time`、`category`、`categories`、`name`、`category_names` 字段。默认 `"[SponsorBlock]: %(category_names)l"`。
- `--no-sponsorblock`：同时禁用 `--sponsorblock-mark` 和 `--sponsorblock-remove`。
- `--sponsorblock-api URL`：SponsorBlock API 位置，默认为 https://sponsor.ajay.app。

<a id="extractor-options"></a>
## 提取器选项

- `--extractor-retries RETRIES`：已知提取器错误的重试次数（默认 3）或 `infinite`。
- `--allow-dynamic-mpd`：处理动态 DASH 清单（默认；别名：`--no-ignore-dynamic-mpd`）。
- `--ignore-dynamic-mpd`：不处理动态 DASH 清单（别名：`--no-allow-dynamic-mpd`）。
- `--hls-split-discontinuity`：在广告间断等不连续处将 HLS 播放列表拆分为不同格式。
- `--no-hls-split-discontinuity`：不在广告间断等不连续处将 HLS 播放列表拆分为不同格式（默认）。
- `--extractor-args IE_KEY:ARGS`：向 IE_KEY 提取器传递 ARGS 参数。详情见“提取器参数”。可多次使用，为不同提取器传参。

<a id="preset-aliases"></a>
## 预设别名

为方便使用而预定义的别名。请注意，未来版本的 yt-dlp 可能增加或调整预设，但不会更改或移除已有预设名称。

    -t mp3                          -f 'ba[acodec^=mp3]/ba/b' -x --audio-format
                                    mp3

    -t aac                          -f
                                    'ba[acodec^=aac]/ba[acodec^=mp4a.40.]/ba/b'
                                    -x --audio-format aac

    -t mp4                          --merge-output-format mp4 --remux-video mp4
                                    -S vcodec:h264,lang,quality,res,fps,hdr:12,a
                                    codec:aac

    -t mkv                          --merge-output-format mkv --remux-video mkv

    -t sleep                        --sleep-subtitles 5 --sleep-requests 0.75
                                    --sleep-interval 10 --max-sleep-interval 20

<a id="configuration"></a>
# 配置

可将任何支持的命令行选项放入配置文件，以配置 yt-dlp。配置从以下位置加载：

1. **主配置**：
    * `--config-locations` 指定的文件
1. **便携配置**：（推荐用于便携安装）
    * 使用二进制文件时，与二进制文件同目录的 `yt-dlp.conf`
    * 从源代码运行时，`yt_dlp` 父目录中的 `yt-dlp.conf`
1. **主目录配置**：
    * `-P` 指定的 home 路径中的 `yt-dlp.conf`
    * 如果没有指定 `-P`，则搜索当前目录
1. **用户配置**：
    * `${XDG_CONFIG_HOME}/yt-dlp.conf`
    * `${XDG_CONFIG_HOME}/yt-dlp/config`（Linux/macOS 上推荐）
    * `${XDG_CONFIG_HOME}/yt-dlp/config.txt`
    * `${APPDATA}/yt-dlp.conf`
    * `${APPDATA}/yt-dlp/config`（Windows 上推荐）
    * `${APPDATA}/yt-dlp/config.txt`
    * `~/yt-dlp.conf`
    * `~/yt-dlp.conf.txt`
    * `~/.yt-dlp/config`
    * `~/.yt-dlp/config.txt`

    另见：[环境变量说明](#环境变量说明)
1. **系统配置**：
    * `/etc/yt-dlp.conf`
    * `/etc/yt-dlp/config`
    * `/etc/yt-dlp/config.txt`

例如，使用以下配置文件时，yt-dlp 将始终提取音频、复制 mtime、使用代理，并将所有视频保存在主目录的 `YouTube` 目录下：
```
# 以 # 开头的行是注释

# 始终提取音频
-x

# 复制修改时间
--mtime

# 使用此代理
--proxy 127.0.0.1:3128

# 将所有视频保存到主目录下的 YouTube 目录
-o ~/YouTube/%(title)s.%(ext)s
```

**注意**：配置文件中的选项与常规命令行调用中的选项或开关完全相同，因此 `-` 或 `--` 后**不能有空白字符**，例如使用 `-o` 或 `--proxy`，而不是 `- o` 或 `-- proxy`。必要时也必须像 UNIX shell 一样加引号。

如果希望某次 yt-dlp 运行禁用所有配置文件，可以使用 `--ignore-config`。如果任何配置文件中出现 `--ignore-config`，之后的配置均不会加载。例如，便携配置文件中包含该选项会阻止加载主目录、用户和系统配置。此外，为保持向后兼容，如果系统配置文件中出现 `--ignore-config`，则不加载用户配置。

<a id="configuration-file-encoding"></a>
### 配置文件编码

配置文件若有 UTF BOM，则按 BOM 解码；否则按系统区域设置的编码解码。

如果希望使用其他编码，请在文件开头添加 `# coding: ENCODING`（例如 `# coding: shift-jis`）。在这之前不能有任何字符，包括空格或 BOM。

<a id="authentication-with-netrc"></a>
### 使用 netrc 认证

对于支持认证的提取器（通过 `--username` 和 `--password` 提供登录名和密码），可以配置自动凭据存储，避免每次运行 yt-dlp 时都将凭据作为命令行参数传入，也避免在 shell 命令历史中留下明文密码。可以使用 [`.netrc` 文件](https://stackoverflow.com/tags/.netrc/info)按提取器配置。需要在 `--netrc-location` 创建 `.netrc` 文件，并将读写权限限制为仅自己可用：
```
touch ${HOME}/.netrc
chmod a-rwx,u+rw ${HOME}/.netrc
```
之后可按以下格式添加提取器凭据，其中 *extractor* 是提取器名称的小写形式：
```
machine <extractor> login <username> password <password>
```
例如：
```
machine youtube login myaccount@gmail.com password my_youtube_password
machine twitch login my_twitch_account_name password my_twitch_password
```
要启用 `.netrc` 文件认证，应向 yt-dlp 传入 `--netrc`，或将其放入[配置文件](#配置)。

`.netrc` 文件的默认位置为 `~`（见下文）。

`.netrc` 文件的缺点是以明文存储密码。另一种方式是配置自定义 shell 命令为提取器提供凭据：传入 `--netrc-cmd` 参数，该命令应以 netrc 格式输出凭据，成功时返回 `0`，其他值视为错误。命令中的 `{}` 会被替换为提取器名称，以便选择正确提取器的凭据。

例如，使用存储为 `.authinfo.gpg` 的加密 `.netrc` 文件：
```
yt-dlp --netrc-cmd 'gpg --decrypt ~/.authinfo.gpg' 'https://www.youtube.com/watch?v=YE7VzlLtp-4'
```

<a id="notes-about-environment-variables"></a>
### 环境变量说明

* UNIX 上环境变量通常写作 `${VARIABLE}`/`$VARIABLE`，Windows 上写作 `%VARIABLE%`；但本文档始终写作 `${VARIABLE}`。
* yt-dlp 也允许在 Windows 上对路径类选项使用 UNIX 风格变量，例如 `--output`、`--config-locations`。
* 如果未设置，`${XDG_CONFIG_HOME}` 默认 `~/.config`，`${XDG_CACHE_HOME}` 默认 `~/.cache`。
* 在 Windows 上，如果存在 `${HOME}`，则 `~` 指向它；否则指向 `${USERPROFILE}` 或 `${HOMEDRIVE}${HOMEPATH}`。
* 在 Windows 上，`${USERPROFILE}` 通常指向 `C:\Users\<user name>`，`${APPDATA}` 指向 `${USERPROFILE}\AppData\Roaming`。

<a id="output-template"></a>
# 输出模板

`-o` 选项用于指定输出文件名模板，`-P` 选项用于指定各类文件的保存路径。

**简要说明：**[直接查看示例](#输出模板示例)。

`-o` 的最简单用法是在下载单个文件时不设置任何模板参数，例如 `yt-dlp -o funny_video.flv "https://some/video"`（_不建议_像这样硬编码文件扩展名，这可能导致某些后处理失败）。

模板也可以包含在下载每个视频时替换的特殊序列。特殊序列可按照 [Python 字符串格式化操作](https://docs.python.org/3/library/stdtypes.html#printf-style-string-formatting)进行格式化，例如 `%(NAME)s` 或 `%(NAME)05d`。具体而言，这是百分号、括号中的名称以及后面的格式化操作。

字段名本身（括号内的部分）也可采用一些特殊格式：

1. **对象遍历**：可使用点号 `.` 分隔符遍历元数据中的字典和列表，例如 `%(tags.0)s`、`%(subtitles.en.-1.ext)s`。可以用冒号 `:` 进行 Python 切片，例如 `%(id.3:7)s`、`%(id.6:2:-1)s`、`%(formats.:.format_id)s`。花括号 `{}` 可用于构建仅包含特定键的字典，例如 `%(formats.:.{format_id,height})#j`。空字段名 `%()s` 指整个 infodict，例如 `%(.{id,title})s`。请注意，通过此方法可用的字段并非都列于下方；使用 `-j` 查看这些字段。
1. **算术运算**：可使用 `+`、`-` 和 `*` 对数字字段进行简单算术运算，例如 `%(playlist_index+10)03d`、`%(n_entries+1-playlist_index)d`。
1. **日期/时间格式化**：在字段名后用 `>` 分隔并指定 [strftime 格式](https://docs.python.org/3/library/datetime.html#strftime-and-strptime-format-codes)，即可格式化日期/时间字段，例如 `%(duration>%H-%M-%S)s`、`%(upload_date>%Y-%m-%d)s`、`%(epoch-3600>%H-%M-%S)s`。
1. **备选字段**：可用 `,` 分隔指定备选字段，例如 `%(release_date>%Y,upload_date>%Y|Unknown)s`。
1. **替换值**：可使用 `&` 分隔符，按照 [`str.format` 迷你语言](https://docs.python.org/3/library/string.html#format-specification-mini-language)指定替换值。如果字段*非空*，将使用替换值而不是字段实际内容。此操作在考虑备选字段之后进行，因此只要*任一*备选字段非空，就会使用替换值。例如 `%(chapters&has chapters|no chapters)s`、`%(title&TITLE={:>20}|NO TITLE)s`。
1. **默认值**：可用 `|` 分隔符为字段为空时指定字面默认值，这会覆盖 `--output-na-placeholder`。例如 `%(uploader|Unknown)s`。
1. **更多转换**：除普通格式类型 `diouxXeEfFgGcrs` 外，yt-dlp 还支持转换为 `B`（字节）、`j`（JSON；`#` 标志用于美化输出，`+` 用于 Unicode）、`h`（HTML 转义）、`l`（逗号分隔列表；`#` 标志改为以 `\n` 换行分隔）、`q`（为终端加引号的字符串；`#` 标志将列表拆为不同参数）、`D`（添加十进制后缀，例如 10M；`#` 标志使用 1024 作为因数）和 `S`（清理为文件名；`#` 标志使用受限形式）。
1. **Unicode 规范化**：格式类型 `U` 可用于 NFC [Unicode 规范化](https://docs.python.org/3/library/unicodedata.html#unicodedata.normalize)。备用形式标志 `#` 将规范化改为 NFD，转换标志 `+` 可用于 NFKC/NFKD 兼容等价规范化。例如 `%(title)+.100U` 是 NFKC。

字段的通用语法为：
```
%(name[.keys][addition][>strf][,alternate][&replacement][|default])[flags][width][.precision][length]type
```

此外，可以为不同元数据文件分别设置输出模板，而不是使用通用输出模板：指定文件类型和模板，并用冒号 `:` 分隔。支持的文件类型为 `subtitle`、`thumbnail`、`description`、`annotation`（已弃用）、`infojson`、`link`、`pl_thumbnail`、`pl_description`、`pl_infojson`、`chapter`、`pl_video`。例如 `-o "%(title)s.%(ext)s" -o "thumbnail:%(title)s/%(title)s.%(ext)s"` 会把缩略图放在与视频同名的文件夹中。如果模板为空，则不写入该类文件。例如 `--write-thumbnail -o "thumbnail:"` 只为播放列表写入缩略图，不为视频写入。

<a id="outtmpl-postprocess-note"></a>

**注意**：由于合并等后处理，实际输出文件名可能不同。使用 `--print after_move:filepath` 获取所有后处理完成后的名称。

可用字段如下：

- `id` (string)：视频标识符
- `title` (string)：视频标题
- `fulltitle` (string)：忽略直播时间戳和通用标题的视频标题
- `ext` (string)：视频文件扩展名
- `alt_title` (string)：视频的次要标题
- `description` (string)：视频描述
- `display_id` (string)：视频的另一种标识符
- `uploader` (string)：视频上传者全名
- `uploader_id` (string)：视频上传者昵称或 ID
- `uploader_url` (string)：视频上传者个人资料 URL
- `license` (string)：视频采用的许可证名称
- `creators` (list)：视频创作者
- `creator` (string)：视频创作者，以逗号分隔
- `timestamp` (numeric)：视频变为可用时刻的 UNIX 时间戳
- `upload_date` (string)：视频上传日期（UTC，YYYYMMDD）
- `release_timestamp` (numeric)：视频发布时刻的 UNIX 时间戳
- `release_date` (string)：视频发布的 UTC 日期（YYYYMMDD）
- `release_year` (numeric)：视频或专辑发布年份（YYYY）
- `modified_timestamp` (numeric)：视频最后修改时刻的 UNIX 时间戳
- `modified_date` (string)：视频最后修改的 UTC 日期（YYYYMMDD）
- `channel` (string)：上传视频的频道全名
- `channel_id` (string)：频道 ID
- `channel_url` (string)：频道 URL
- `channel_follower_count` (numeric)：频道关注者数量
- `channel_is_verified` (boolean)：频道是否在平台上获得认证
- `location` (string)：拍摄视频的实际地点
- `duration` (numeric)：视频时长，单位为秒
- `duration_string` (string)：视频时长（HH:mm:ss）
- `view_count` (numeric)：平台上观看过视频的用户数
- `concurrent_view_count` (numeric)：平台上当前正在观看视频的用户数
- `like_count` (numeric)：视频正面评价数量
- `dislike_count` (numeric)：视频负面评价数量
- `repost_count` (numeric)：视频转发次数
- `average_rating` (numeric)：用户给出的平均评分，量表取决于网页
- `comment_count` (numeric)：视频评论数（某些提取器只在最后下载评论，因此不能使用此字段）
- `save_count` (numeric)：视频被保存或收藏的次数
- `age_limit` (numeric)：视频年龄限制（岁）
- `live_status` (string)：`not_live`、`is_live`、`is_upcoming`、`was_live`、`post_live` 之一（`post_live` 表示曾直播但点播视频尚未处理）
- `is_live` (boolean)：此视频是否为直播流，而非固定时长视频
- `was_live` (boolean)：此视频最初是否为直播流
- `playable_in_embed` (string)：此视频是否允许在其他网站的嵌入式播放器中播放
- `availability` (string)：视频是 `private`、`premium_only`、`subscriber_only`、`needs_auth`、`unlisted` 还是 `public`
- `media_type` (string)：网站分类的媒体类型，例如 `episode`、`clip`、`trailer`
- `start_time` (numeric)：URL 指定的开始播放时间，单位为秒
- `end_time` (numeric)：URL 指定的结束播放时间，单位为秒
- `extractor` (string)：提取器名称
- `extractor_key` (string)：提取器键名
- `epoch` (numeric)：信息提取完成时的 Unix epoch 时间
- `autonumber` (numeric)：每次下载递增的数字，从 `--autonumber-start` 开始，前导零补齐至 5 位
- `video_autonumber` (numeric)：每个视频递增的数字
- `n_entries` (numeric)：播放列表中提取出的条目总数
- `playlist_id` (string)：包含该视频的播放列表标识符
- `playlist_title` (string)：包含该视频的播放列表名称
- `playlist` (string)：可用时为 `playlist_title`，否则为 `playlist_id`
- `playlist_count` (numeric)：播放列表中的条目总数。如果未提取整个播放列表，可能未知
- `playlist_index` (numeric)：视频在播放列表中的索引，按最终索引位数在前面补零
- `playlist_autonumber` (numeric)：视频在播放列表下载队列中的位置，按播放列表总长度位数在前面补零
- `playlist_uploader` (string)：播放列表上传者全名
- `playlist_uploader_id` (string)：播放列表上传者昵称或 ID
- `playlist_channel` (string)：上传播放列表的频道显示名称
- `playlist_channel_id` (string)：上传播放列表的频道标识符
- `playlist_webpage_url` (string)：播放列表网页 URL
- `webpage_url` (string)：视频网页 URL；将其传给 yt-dlp 应再次产生相同结果
- `webpage_url_basename` (string)：网页 URL 的 basename
- `webpage_url_domain` (string)：网页 URL 的域名
- `original_url` (string)：用户提供的 URL（播放列表条目则与 `webpage_url` 相同）
- `categories` (list)：视频所属类别列表
- `tags` (list)：视频标签列表
- `cast` (list)：演员列表

[过滤格式](#过滤格式)中的所有字段也可使用。

对于属于某个逻辑章节或片段的视频，可使用：

- `chapter` (string)：视频所属章节的名称或标题
- `chapter_number` (numeric)：视频所属章节的编号
- `chapter_id` (string)：视频所属章节的 ID

对于某个系列或节目的剧集视频，可使用：

- `series` (string)：视频剧集所属系列或节目标题
- `series_id` (string)：视频剧集所属系列或节目的 ID
- `season` (string)：视频剧集所属季的标题
- `season_number` (numeric)：视频剧集所属季的编号
- `season_id` (string)：视频剧集所属季的 ID
- `episode` (string)：视频剧集标题
- `episode_number` (numeric)：视频剧集在一季中的编号
- `episode_id` (string)：视频剧集 ID

对于曲目或音乐专辑的一部分，可使用：

- `track` (string)：曲目标题
- `track_number` (numeric)：曲目在专辑或唱片中的编号
- `track_id` (string)：曲目 ID
- `artists` (list)：曲目艺术家
- `artist` (string)：曲目艺术家，以逗号分隔
- `genres` (list)：曲目类型
- `genre` (string)：曲目类型，以逗号分隔
- `composers` (list)：作品作曲者
- `composer` (string)：作品作曲者，以逗号分隔
- `album` (string)：曲目所属专辑标题
- `album_type` (string)：专辑类型
- `album_artists` (list)：专辑中的所有艺术家
- `album_artist` (string)：专辑中的所有艺术家，以逗号分隔
- `disc_number` (numeric)：曲目所属唱片或其他物理介质的编号

仅在使用 `--download-sections`，以及对具有内部章节的视频使用 `--split-chapters` 时的 `chapter:` 前缀中可用：

- `section_title` (string)：章节标题
- `section_number` (numeric)：章节在文件中的编号
- `section_start` (numeric)：章节开始时间，单位为秒
- `section_end` (numeric)：章节结束时间，单位为秒

仅用于 `--print` 时可用：

- `urls` (string)：所有请求格式的 URL，每行一个
- `filename` (string)：视频文件名。请注意[实际文件名可能不同](#outtmpl-postprocess-note)
- `formats_table` (table)：`--list-formats` 打印的视频格式表
- `thumbnails_table` (table)：`--list-thumbnails` 打印的缩略图格式表
- `subtitles_table` (table)：`--list-subs` 打印的字幕格式表
- `automatic_captions_table` (table)：`--list-subs` 打印的自动字幕格式表

仅在视频下载后（`post_process`/`after_move`）可用：

- `filepath`：下载的视频文件实际路径

仅在 `--sponsorblock-chapter-title` 中可用：

- `start_time` (numeric)：章节开始时间，单位为秒
- `end_time` (numeric)：章节结束时间，单位为秒
- `categories` (list)：章节所属 [SponsorBlock 类别](https://wiki.sponsor.ajay.app/w/Types#Category)
- `category` (string)：章节所属最小的 SponsorBlock 类别
- `category_names` (list)：类别的友好名称
- `name` (string)：最小类别的友好名称
- `type` (string)：章节的 [SponsorBlock 动作类型](https://wiki.sponsor.ajay.app/w/Types#Action_Type)

输出模板中引用上述任一序列时，它会被与序列名称对应的实际值替换。例如，对 `-o %(title)s-%(id)s.%(ext)s`，如果 mp4 视频标题为 `yt-dlp test video`、ID 为 `YE7VzlLtp-4`，则会在当前目录创建 `yt-dlp test video-YE7VzlLtp-4.mp4` 文件。

**注意**：不能保证所有序列都存在，因为它们取决于特定提取器获得的元数据。缺失的序列将替换为 `--output-na-placeholder` 提供的占位值（默认 `NA`）。

**提示**：查看 `-j` 输出，确定特定 URL 可用的字段。

对于数字序列，可以使用[数字相关格式](https://docs.python.org/3/library/stdtypes.html#printf-style-string-formatting)。例如 `%(view_count)05d` 会产生将观看次数用前导零补齐至 5 个字符的字符串，如 `00042`。

输出模板也可以包含任意层级路径，例如 `-o "%(playlist)s/%(playlist_index)s - %(title)s.%(ext)s"`，这会将各视频下载到对应路径模板的目录中。缺失目录会自动创建。

要在输出模板中使用字面百分号，请使用 `%%`。输出到标准输出请使用 `-o -`。

当前默认模板为 `%(title)s [%(id)s].%(ext)s`。

有时你不希望出现“中”、空格或 `&` 等特殊字符，例如将下载文件名传输到 Windows 系统，或经由不支持 8 位字符的通道传输时。此时添加 `--restrict-filenames` 标志可获得较短的标题。

<a id="output-template-examples"></a>
#### 输出模板示例

```bash
$ yt-dlp --print filename -o "test video.%(ext)s" ptd1NN40vMw
test video.webm    # Literal name with correct extension

$ yt-dlp --print filename -o "%(title)s.%(ext)s" ptd1NN40vMw
To'y!🤯😂🤦🏻‍♂️.webm    # All kinds of weird characters

$ yt-dlp --print filename -o "%(title)s.%(ext)s" ptd1NN40vMw --restrict-filenames
To_y.webm    # Restricted file name

# 将 YouTube 播放列表视频下载到单独目录，并按播放列表中的顺序编号
$ yt-dlp -o "%(playlist)s/%(playlist_index)s - %(title)s.%(ext)s" "https://www.youtube.com/playlist?list=PLwiyx1dc3P2JR9N8gQaQN_BCvlSlap7re"

# 将 YouTube 播放列表视频按上传年份下载到不同目录
$ yt-dlp -o "%(upload_date>%Y)s/%(title)s.%(ext)s" "https://www.youtube.com/playlist?list=PLwiyx1dc3P2JR9N8gQaQN_BCvlSlap7re"

# 仅在播放列表索引存在时，在其前面加上 " - " 分隔符
$ yt-dlp -o "%(playlist_index&{} - |)s%(title)s.%(ext)s" YE7VzlLtp-4 "https://www.youtube.com/user/TheLinuxFoundation/playlists"

# 下载 YouTube 频道或用户的所有播放列表，每个列表放在独立目录：
$ yt-dlp -o "%(uploader)s/%(playlist)s/%(playlist_index)s - %(title)s.%(ext)s" "https://www.youtube.com/user/TheLinuxFoundation/playlists"

# 下载 Udemy 课程，将每章放在主目录的 MyVideos 下的独立目录中
$ yt-dlp -u user -p password -P "~/MyVideos" -o "%(playlist)s/%(chapter_number)s - %(chapter)s/%(title)s.%(ext)s" "https://www.udemy.com/java-tutorial"

# 下载整季剧集，在 C:/MyVideos 下按剧集和季分别建目录
$ yt-dlp -P "C:/MyVideos" -o "%(series)s/%(season_number)s - %(season)s/%(episode_number)s - %(episode)s.%(ext)s" "https://videomore.ru/kino_v_detalayah/5_sezon/367617"

# 将视频下载为 "C:\MyVideos\uploader\title.ext"，字幕下载为 "C:\MyVideos\subs\uploader\title.ext"
# 将所有临时文件放入 "C:\MyVideos\tmp"
$ yt-dlp -P "C:/MyVideos" -P "temp:tmp" -P "subtitle:subs" -o "%(uploader)s/%(title)s.%(ext)s" YE7VzlLtp-4 --write-subs

# 将视频下载为 "C:\MyVideos\uploader\title.ext"，字幕下载为 "C:\MyVideos\uploader\subs\title.ext"
$ yt-dlp -P "C:/MyVideos" -o "%(uploader)s/%(title)s.%(ext)s" -o "subtitle:%(uploader)s/subs/%(title)s.%(ext)s" YE7VzlLtp-4 --write-subs

# 将正在下载的视频流式输出到标准输出
$ yt-dlp -o - YE7VzlLtp-4
```

<a id="format-selection"></a>
# 格式选择

默认情况下，如果**不**传任何选项，yt-dlp 会尝试下载可用的最佳质量。这通常等同于 `-f bestvideo*+bestaudio/best`。但如果启用了多音频流（`--audio-multistreams`），默认格式会变为 `-f bestvideo+bestaudio/best`。同样，如果 ffmpeg 不可用，或使用 yt-dlp 将内容流式输出到 `stdout`（`-o -`），默认值变为 `-f best/bestvideo+bestaudio`。

**弃用警告**：最新版本的 yt-dlp 可以使用 ffmpeg 同时将多种格式流式输出到标准输出。因此，未来版本中此处的默认值将与普通下载类似，设为 `-f bv*+ba/b`。如要保留 `-f b/bv+ba` 设置，建议在配置选项中明确指定。

格式选择的通用语法是 `-f FORMAT`（或 `--format FORMAT`），其中 `FORMAT` 是*选择器表达式*，即描述希望下载的一种或多种格式的表达式。

**简要说明：**[直接查看示例](#格式选择示例)。

最简单的情况是请求特定格式，例如 `-f 22` 可下载格式代码为 22 的格式。可以用 `--list-formats` 或 `-F` 获取特定视频可用的格式代码列表。注意这些格式代码由各提取器决定。

还可以使用文件扩展名（目前支持 `3gp`、`aac`、`flv`、`m4a`、`mp3`、`mp4`、`ogg`、`wav`、`webm`），下载作为单个文件提供的指定扩展名最佳质量格式。例如 `-f webm` 会下载作为单个文件提供、扩展名为 `webm` 的最佳质量格式。

可以使用 `-f -` 为*每个视频*交互式提供格式选择器。

也可以使用特殊名称选择特定边缘情况的格式：

- `all`：分别选择**所有格式**。
- `mergeall`：选择并**合并所有格式**（必须与 `--audio-multistreams`、`--video-multistreams` 或两者一起使用）。
- `b*`、`best*`：选择**包含视频、音频或两者**的最佳质量格式（即 `vcodec!=none or acodec!=none`）。
- `b`、`best`：选择**同时包含视频和音频**的最佳质量格式，等同于 `best*[vcodec!=none][acodec!=none]`。
- `bv`、`bestvideo`：选择最佳质量的**纯视频**格式，等同于 `best*[acodec=none]`。
- `bv*`、`bestvideo*`：选择**包含视频**的最佳质量格式，也可能包含音频，等同于 `best*[vcodec!=none]`。
- `ba`、`bestaudio`：选择最佳质量的**纯音频**格式，等同于 `best*[vcodec=none]`。
- `ba*`、`bestaudio*`：选择**包含音频**的最佳质量格式，也可能包含视频，等同于 `best*[acodec!=none]`（[不要使用！](https://github.com/yt-dlp/yt-dlp/issues/979#issuecomment-919629354)）。
- `w*`、`worst*`：选择包含视频或音频的最差质量格式。
- `w`、`worst`：选择同时包含视频和音频的最差质量格式，等同于 `worst*[vcodec!=none][acodec!=none]`。
- `wv`、`worstvideo`：选择最差质量的纯视频格式，等同于 `worst*[acodec=none]`。
- `wv*`、`worstvideo*`：选择包含视频的最差质量格式，也可能包含音频，等同于 `worst*[vcodec!=none]`。
- `wa`、`worstaudio`：选择最差质量的纯音频格式，等同于 `worst*[vcodec=none]`。
- `wa*`、`worstaudio*`：选择包含音频的最差质量格式，也可能包含视频，等同于 `worst*[acodec!=none]`。

例如，可使用 `-f worstvideo` 下载最差质量的纯视频格式。不过建议不要使用 `worst` 及相关选项：当格式选择器为 `worst` 时，会选择各方面都最差的格式。大多数时候实际需要的是文件最小的视频。因此通常最好使用 `-S +size`，或者更严格地使用 `-S +size,+br,+res,+fps`，而不是 `-f worst`。详情见[格式排序](#格式排序)。

可用 `best<type>.<n>` 选择某种类型中第 n 好的格式。例如 `best.2` 选择第二好的合并格式；`bv*.3` 选择第三好的包含视频流的格式。

如果要下载多个视频，而它们的可用格式不同，可使用斜杠指定优先顺序。左侧格式优先：例如 `-f 22/17/18` 会先尝试格式 22，不可用时尝试 17，再不行则尝试 18；都不可用时报告没有合适的可下载格式。

如要下载同一视频的多个格式，可使用逗号分隔，例如 `-f 22,17,18` 会下载这三种可用格式。还可以结合优先级，例如 `-f 136/137/mp4/bestvideo,140/m4a/bestaudio`。

可用 `-f <format1>+<format2>+...` 将多个格式的视频和音频合并到一个文件中（需要安装 ffmpeg）。例如 `-f bestvideo+bestaudio` 会下载最佳纯视频和最佳纯音频格式，并用 ffmpeg 合并。

**弃用警告**：由于下述行为复杂且违反直觉，未来将移除这种行为并默认启用多流；届时会增加新运算符，将格式限制为单个音频/视频流。

除非使用 `--video-multistreams`，否则除第一个以外的所有含视频流格式都会被忽略。同样，除非使用 `--audio-multistreams`，否则除第一个以外的所有含音频流格式都会被忽略。例如 `-f bestvideo+best+bestaudio --video-multistreams --audio-multistreams` 会下载并合并给定的 3 种格式，结果文件有 2 个视频流和 2 个音频流。但 `-f bestvideo+best+bestaudio --no-video-multistreams` 只会下载并合并 `bestvideo` 和 `bestaudio`：因为已选择另一个包含视频流的格式 `bestvideo`，所以忽略 `best`。因此格式顺序很重要。`-f best+bestaudio --no-audio-multistreams` 只下载 `best`，而 `-f bestaudio+best --no-audio-multistreams` 会忽略 `best`，只下载 `bestaudio`。

<a id="filtering-formats"></a>
## 过滤格式

也可通过在方括号中放置条件来过滤视频格式，例如 `-f "best[height=720]"`（也可用 `-f "[filesize>10M]"`，因为不带选择器的过滤器会被解释为 `best`）。

以下数字元数据字段可用于 `<`、`<=`、`>`、`>=`、`=`（等于）、`!=`（不等于）比较：

- `filesize`：字节数（如预先已知）
- `filesize_approx`：估计字节数
- `width`：视频宽度（如已知）
- `height`：视频高度（如已知）
- `aspect_ratio`：视频宽高比（如已知）
- `tbr`：音视频平均总码率，单位 [kbps](## "1000 bits/sec")
- `abr`：平均音频码率，单位 [kbps](## "1000 bits/sec")
- `vbr`：平均视频码率，单位 [kbps](## "1000 bits/sec")
- `asr`：音频采样率，单位赫兹
- `fps`：帧率
- `audio_channels`：音频声道数
- `stretched_ratio`：视频像素非正方形时的 `width:height`

以下字符串元数据字段可使用 `=`（等于）、`^=`（以……开头）、`$=`（以……结尾）、`*=`（包含）、`~=`（匹配正则表达式）比较：

- `url`：视频 URL
- `ext`：文件扩展名
- `acodec`：使用的音频编解码器名称
- `vcodec`：使用的视频编解码器名称
- `container`：容器格式名称
- `protocol`：实际下载使用的协议，小写（`http`、`https`、`rtmp`、`rtmpe`、`f4m`、`ism`、`http_dash_segments`、`m3u8` 或 `m3u8_native`）
- `language`：语言代码
- `dynamic_range`：视频动态范围
- `format_id`：格式的简短描述
- `format`：供人阅读的格式描述
- `format_note`：格式的附加信息
- `resolution`：宽度和高度的文本描述

任何字符串比较前可加否定符 `!`，得到相反比较，例如 `!*=`（不包含）。如果字符串比较值包含空格或 `._-` 之外的特殊字符，需要用单引号或双引号括起。

**注意**：上述元数据字段均不保证存在，因为这完全取决于特定提取器获得的元数据，即网站提供的元数据。提取器提供的任何其他字段也可用于过滤。

值未知的格式会被排除，除非在运算符后加问号 `?`。可以组合格式过滤器，因此 `-f "bv[height<=?720][tbr>500]"` 选择最高 720p 的视频（或高度未知的视频），且码率大于 500 kbps。也可将过滤器与 `all` 一起使用以下载所有满足条件的格式，例如 `-f "all[vcodec=none]"` 选择所有纯音频格式。

格式选择器也可用圆括号分组，例如 `-f "(mp4,webm)[height<480]"` 会下载高度低于 480 的最佳预合并 mp4 和 webm 格式。

<a id="sorting-formats"></a>
## 格式排序

可以使用 `-S`（`--format-sort`）改变判定 `best` 的标准，通用格式为 `--format-sort field1,field2...`。

可用字段：

- `hasvid`：优先选择有视频流的格式
- `hasaud`：优先选择有音频流的格式
- `ie_pref`：格式优先级
- `lang`：提取器确定的语言优先级（例如原语言优先于音频描述）
- `quality`：格式质量
- `source`：来源优先级
- `proto`：下载协议（`https`/`ftps` > `http`/`ftp` > `m3u8_native`/`m3u8` > `http_dash_segments` > `websocket_frag` > `f4f`/`f4m`）
- `vcodec`：视频编解码器（`av01` > `vp9.2` > `vp9` > `h265` > `h264` > `vp8` > `h263` > `theora` > 其他）
- `acodec`：音频编解码器（`flac`/`alac` > `wav`/`aiff` > `opus` > `vorbis` > `aac` > `mp4a` > `mp3` > `ac4` > `eac3` > `ac3` > `dts` > 其他）
- `codec`：等同于 `vcodec,acodec`
- `vext`：视频扩展名（`mp4` > `mov` > `webm` > `flv` > 其他）。使用 `--prefer-free-formats` 时优先 `webm`
- `aext`：音频扩展名（`m4a` > `aac` > `mp3` > `ogg` > `opus` > `webm` > 其他）。使用 `--prefer-free-formats` 时顺序变为 `ogg` > `opus` > `webm` > `mp3` > `m4a` > `aac`
- `ext`：等同于 `vext,aext`
- `filesize`：确切文件大小（如预先已知）
- `fs_approx`：估计文件大小
- `size`：有确切文件大小时使用该值，否则使用估计值
- `height`：视频高度
- `width`：视频宽度
- `res`：视频分辨率，按较小的维度计算
- `fps`：视频帧率
- `hdr`：视频动态范围（`DV` > `HDR12` > `HDR10+` > `HDR10` > `HLG` > `SDR`）
- `channels`：音频声道数
- `tbr`：平均总码率，单位 [kbps](## "1000 bits/sec")
- `vbr`：平均视频码率，单位 [kbps](## "1000 bits/sec")
- `abr`：平均音频码率，单位 [kbps](## "1000 bits/sec")
- `br`：平均码率，单位 [kbps](## "1000 bits/sec")，即 `tbr`/`vbr`/`abr`
- `asr`：音频采样率，单位 Hz

**弃用警告**：其中许多字段有目前未写入文档的别名，未来版本可能移除。建议只使用文档中列出的字段名。

除非另有说明，所有字段按降序排列。要反转顺序，可在字段前加 `+`；例如 `+res` 优先选择分辨率最小的格式。还可在字段后用 `:` 分隔附加首选值。例如 `res:720` 优先选择较大但不超过 720p 的视频；若没有低于 720p 的视频，则选择最小的视频。对于 `codec` 和 `ext`，可以提供两个首选值，第一个用于视频，第二个用于音频。例如 `+codec:avc:m4a`（等同于 `+vcodec:avc,+acodec:m4a`）将视频编解码器优先级设为 `h264` > `h265` > `vp9` > `vp9.2` > `av01` > `vp8` > `h263` > `theora`，音频编解码器优先级设为 `mp4a` > `aac` > `vorbis` > `opus` > `mp3` > `ac3` > `dts`。也可使用 `~` 分隔，使排序优先选择最接近给定值的格式，例如 `filesize~1G` 优先选择大小最接近 1 GiB 的格式。

`hasvid` 和 `ie_pref` 在排序中始终具有最高优先级，与用户定义顺序无关。可用 `--format-sort-force` 改变这一行为。除此之外，默认顺序为 `lang,quality,res,fps,hdr:12,vcodec,channels,acodec,size,br,asr,proto,ext,hasaud,source,id`。提取器可以覆盖此默认顺序，但不能覆盖用户提供的顺序。

请注意，hdr 的默认值是 `hdr:12`，即不优先选择 Dolby Vision。这是因为 DV 格式尚未与大多数设备完全兼容。未来可能改变。

如果格式选择器为 `worst`，则选择排序后的最后一项，即各方面都最差的格式。大多数时候实际需要的是文件最小的视频，因此通常最好使用 `-f best -S +size,+br,+res,+fps`。

如果多次使用 `-S`/`--format-sort`，每个后续排序参数会置于之前参数前，重复字段只保留最高优先级项。例如 `-S proto -S res` 等同于 `-S res,proto`，`-S res:720,fps -S vcodec,res:1080` 等同于 `-S vcodec,res:1080,fps`。可使用 `--format-sort-reset` 忽略之前传入的 `-S`/`--format-sort` 参数并重置默认顺序。

**提示**：可使用 `-v -F` 查看格式的排序方式（从最差到最佳）。

<a id="format-selection-examples"></a>
## 格式选择示例

```bash
# 下载并合并最佳纯视频格式和最佳纯音频格式，
# 如果没有纯视频格式，则下载最佳合并格式
$ yt-dlp -f "bv+ba/b"

# 下载包含视频的最佳格式，
# 如果其中没有音频流，则与最佳纯音频格式合并
$ yt-dlp -f "bv*+ba/b"

# 同上
$ yt-dlp

# 下载最佳纯视频和最佳纯音频格式，但不合并
# 此时应使用输出模板，因为
# 默认情况下 bestvideo 和 bestaudio 会有相同的文件名。
$ yt-dlp -f "bv,ba" -o "%(title)s.f%(format_id)s.%(ext)s"

# 下载并合并包含视频流的最佳格式，
# 以及所有纯音频格式，合并为一个文件
$ yt-dlp -f "bv*+mergeall[vcodec=none]" --audio-multistreams

# 下载并合并包含视频流的最佳格式，
# 以及最佳的 2 个纯音频格式，合并为一个文件
$ yt-dlp -f "bv*+ba+ba.2" --audio-multistreams


# 以下示例展示旧的格式选择方法（不使用 -S）
# 以及如何使用 -S 获得类似但（通常）更好的结果

# 下载可用的最差视频（旧方法）
$ yt-dlp -f "wv*+wa/w"

# 下载分辨率最小的最佳可用视频
$ yt-dlp -S "+res"

# 下载可用的最小视频
$ yt-dlp -S "+size,+br"



# 下载最佳可用 mp4 视频；没有 mp4 时下载最佳视频
$ yt-dlp -f "bv*[ext=mp4]+ba[ext=m4a]/b[ext=mp4] / bv*+ba/b"

# 下载使用最佳扩展名的最佳视频
# （视频：mp4 > mov > webm > flv；音频：m4a > aac > mp3 ...）
$ yt-dlp -S "ext"



# 下载不高于 480p 的最佳可用视频，
# 若没有低于 480p 的视频则下载最差视频
$ yt-dlp -f "bv*[height<=480]+ba/b[height<=480] / wv*+ba/w"

# 下载高度最大但不高于 480p 的最佳可用视频，
# 若没有低于 480p 的视频则下载分辨率最小的最佳视频
$ yt-dlp -S "height:480"

# 下载分辨率最大但不高于 480p 的最佳可用视频，
# 若没有低于 480p 的视频则下载分辨率最小的最佳视频
# 分辨率按较小的尺寸确定。
# 因此这对竖屏视频也适用
$ yt-dlp -S "res:480"



# 下载不大于 50 MB 的最佳视频（同时包含音频），
# 若没有小于 50 MB 的视频则下载最差视频（同时包含音频）
$ yt-dlp -f "b[filesize<50M] / w"

# 下载不大于 50 MB 的最大视频（同时包含音频），
# 若没有小于 50 MB 的视频则下载最小视频（同时包含音频）
$ yt-dlp -f "b" -S "filesize:50M"

# 下载大小最接近 50 MB 的最佳视频（同时包含音频）
$ yt-dlp -f "b" -S "filesize~50M"



# 通过 HTTP/HTTPS 协议的直链下载最佳可用视频，
# 若没有此类视频则用任意协议下载最佳可用视频
$ yt-dlp -f "(bv*+ba/b)[protocol^=http][protocol!*=dash] / (bv*+ba/b)"

# 通过最佳协议下载最佳可用视频
# （https/ftps > http/ftp > m3u8_native > m3u8 > http_dash_segments ...）
$ yt-dlp -S "proto"



# 下载使用 h264 或 h265 编解码器的最佳视频，
# 若没有此类视频则下载最佳视频
$ yt-dlp -f "(bv*[vcodec~='^((he|a)vc|h26[45])']+ba) / (bv*+ba/b)"

# 下载使用不优于 h264 的最佳编解码器的最佳视频，
# 若没有此类视频则下载使用最差编解码器的最佳视频
$ yt-dlp -S "codec:h264"

# 下载使用不差于 h264 的最差编解码器的最佳视频，
# 若没有此类视频则下载使用最佳编解码器的最佳视频
$ yt-dlp -S "+codec:h264"



# 更复杂的示例

# 下载不高于 720p 的最佳视频，优先选择帧率大于 30 的视频，
# 若没有此类视频则下载最差视频（仍优先选择帧率大于 30）
$ yt-dlp -f "((bv*[fps>30]/bv*)[height<=720]/(wv*[fps>30]/wv*)) + ba / (b[fps>30]/b)[height<=720]/(w[fps>30]/w)"

# 下载分辨率最大但不高于 720p 的视频，
# 若没有此类视频则下载可用的最小分辨率视频，
# 相同分辨率的格式优先选择较高帧率
$ yt-dlp -S "res:720,fps"



# 下载分辨率最小但不低于 480p 的视频，
# 若没有此类视频则下载可用的最大分辨率视频，
# 相同分辨率时优先选择更好的编解码器，再选择更高的总比特率
$ yt-dlp -S "+res:480,codec,br"
```

<a id="modifying-metadata"></a>
# 修改元数据

提取器获取的元数据可以使用 `--parse-metadata` 和 `--replace-in-metadata` 修改。

`--replace-in-metadata FIELDS REGEX REPLACE` 使用 [Python 正则表达式](https://docs.python.org/3/library/re.html#regular-expression-syntax)替换任意元数据字段中的文本。高级用法可在替换字符串中使用[反向引用](https://docs.python.org/3/library/re.html?highlight=backreferences#re.sub)。

`--parse-metadata FROM:TO` 的通用语法是提供要从中提取数据的字段名或[输出模板](#输出模板)，以及解释该数据所用的格式，两者用冒号 `:` 分隔。`TO` 可以是带命名捕获组的 [Python 正则表达式](https://docs.python.org/3/library/re.html#regular-expression-syntax)、单个字段名，或类似[输出模板](#输出模板)的语法（仅支持 `%(field)s` 格式）。该选项可多次使用，以解析和修改多个字段。

请注意，这些选项保留相对顺序，因此可以替换已解析字段，也可以反过来操作。以这种方式创建的任何字段都可用于[输出模板](#输出模板)，并且会影响使用 `--embed-metadata` 时添加到媒体文件的元数据。

此选项还有几种特殊用途：

* 可以根据当前下载视频的元数据下载额外 URL：将 `additional_urls` 字段设为要下载的 URL。例如 `--parse-metadata "description:(?P<additional_urls>https?://www\.vimeo\.com/\d+)"` 会下载描述中发现的第一个 Vimeo 视频。
* 可以用它更改嵌入媒体文件的元数据：设置带 `meta_` 前缀的相应字段值。例如，设置到 `meta_description` 的任何值都会添加到文件的 `description` 字段，可用来设置不同的“description”和“synopsis”。要修改单个流的元数据，请使用 `meta<n>_` 前缀（例如 `meta1_language`）。设置到 `meta_` 字段的任何值都会覆盖所有默认值。

**注意**：元数据修改发生在格式选择、提取后处理及其他后处理操作之前。这些步骤可能添加或更改某些字段，从而覆盖你的修改。

供参考，yt-dlp 默认将以下字段添加到文件元数据：

元数据字段                 | 来源
:--------------------------|:------------------------------------------------
`title`                    | `track` 或 `title`
`date`                     | `upload_date`
`description`,  `synopsis` | `description`
`purl`, `comment`          | `webpage_url`
`track`                    | `track_number`
`artist`                   | `artist`, `artists`, `creator`, `creators`, `uploader` 或 `uploader_id`
`composer`                 | `composer` 或 `composers`
`genre`                    | `genre`, `genres`, `categories` 或 `tags`
`album`                    | `album` 或 `series`
`album_artist`             | `album_artist` 或 `album_artists`
`disc`                     | `disc_number`
`show`                     | `series`
`season_number`            | `season_number`
`episode_id`               | `episode` 或 `episode_id`
`episode_sort`             | `episode_number`
每个流的 `language`        | 格式的 `language`


**注意**：文件格式可能不支持其中某些字段。

<a id="modifying-metadata-examples"></a>
## 修改元数据示例

```bash
# 将标题解释为 "Artist - Title"
$ yt-dlp --parse-metadata "title:%(artist)s - %(title)s"

# 正则表达式示例
$ yt-dlp --parse-metadata "description:Artist - (?P<artist>.+)"

# 将 episode 字段复制到 title 字段（FROM 和 TO 均为单个字段）
$ yt-dlp --parse-metadata "episode:title"

# 将标题设为 "Series name S01E05"
$ yt-dlp --parse-metadata "%(series)s S%(season_number)02dE%(episode_number)02d:%(title)s"

# 在视频元数据的 "artist" 字段中优先使用 uploader
$ yt-dlp --parse-metadata "%(uploader|)s:%(meta_artist)s" --embed-metadata

# 使用 description 而非 webpage_url 设置视频元数据的 "comment" 字段，
# 并正确处理多行
$ yt-dlp --parse-metadata "description:(?s)(?P<meta_comment>.+)" --embed-metadata

# 不在视频元数据中设置 "synopsis"
$ yt-dlp --parse-metadata ":(?P<meta_synopsis>)"

# 将 infojson 的 "formats" 字段设为空字符串以移除它
$ yt-dlp --parse-metadata "video::(?P<formats>)" --write-info-json

# 将 title 和 uploader 中的所有空格和 "_" 替换为 `-`
$ yt-dlp --replace-in-metadata "title,uploader" "[ _]" "-"

```

<a id="extractor-arguments"></a>
# 提取器参数

某些提取器接受通过 `--extractor-args KEY:ARGS` 传入的额外参数。`ARGS` 是以 `;`（分号）分隔的 `ARG=VAL1,VAL2` 字符串。例如 `--extractor-args "youtube:player-client=tv,mweb;formats=incomplete" --extractor-args "twitter:api=syndication"`。

注意：在 CLI 中，`ARG` 可使用 `-` 代替 `_`；例如 `youtube:player-client"` 变为 `youtube:player_client"`。

以下提取器使用此功能：

#### youtube
* `lang`：优先使用此语言代码（区分大小写）的翻译元数据（`title`、`description` 等）。默认优先视频主要语言的元数据，回退到 `en` 翻译。支持的内容语言代码列表见 [youtube/_base.py](https://github.com/yt-dlp/yt-dlp/blob/415b4c9f955b1a0391204bd24a7132590e7b3bdb/yt_dlp/extractor/youtube/_base.py#L402-L409)。
* `skip`：指定 `hls`、`dash` 或 `translated_subs` 中一个或多个，分别跳过 m3u8 清单、dash 清单和[自动翻译字幕](https://github.com/yt-dlp/yt-dlp/issues/4090#issuecomment-1158102032)的提取。
* `player_client`：用于提取视频数据的客户端。目前可用 `web`、`web_safari`、`web_embedded`、`web_music`、`web_creator`、`mweb`、`ios`、`visionos`、`android`、`android_vr`、`tv`、`tv_downgraded`、`tv_samsung` 和 `tv_simply`。默认使用 `tv_samsung,visionos,web`。如果没有 JavaScript 运行时或引擎，则省略 `web` 和 `tv_samsung`。向 yt-dlp 传入已登录 Cookie 时，免费账号使用 `web_embedded,tv_samsung,web`，高级账号使用 `web_creator,tv_samsung,web`。使用已登录 Cookie 时，`music.youtube.com` URL 会加入 `web_music` 客户端。年龄限制视频会加入 `web_embedded`，但它仅在某些情况下能成功绕过年龄限制（例如视频可嵌入时）。如果 `android_vr` 或 `visionos` 无法访问视频，可能加入 `web_embedded` 作为回退。如果账号需要年龄验证，年龄限制视频会加入 `web_creator`。`web_creator` 和 `web_music` 等客户端的格式需要 `po_token` 才能下载；`web_creator` 等客户端仅在认证后工作。并非所有客户端都支持通过 Cookie 认证。可用 `default` 表示默认客户端，或用 `all` 表示全部客户端（不推荐）。客户端名前加 `-` 可将其排除，例如 `youtube:player_client=default,-web`。
* `player_skip`：跳过稳健提取通常需要的一些网络请求。可选一个或多个：`configs`（跳过客户端配置）、`webpage`（跳过初始网页）、`js`（跳过 JS 播放器）、`initial_data`（跳过初始数据/下一集请求）。这些选项可能减少请求或避开某些限速，但也可能导致格式或元数据缺失等问题。详情见 [#860](https://github.com/yt-dlp/yt-dlp/pull/860) 和 [#12826](https://github.com/yt-dlp/yt-dlp/issues/12826)。
* `webpage_skip`：跳过嵌入网页数据的提取。可选 `player_response`、`initial_data` 中一个或两个。这些选项用于测试，不会跳过任何网络请求。默认都不跳过；但若 `player_js_version` 使用 `actual` 以外的值，则隐含 `webpage_skip=player_response`。
* `webpage_client`：视频网页请求所用客户端，`web`（默认）或 `web_safari`。
* `player_params`：播放器请求所用 YouTube 播放器参数，会覆盖 yt-dlp 设置的任何默认值。
* `player_js_variant`：用于 n/sig 解密的播放器 JavaScript 变体。已知变体：`main`、`tcc`、`tce`、`es5`、`es6`、`es6_tcc`、`es6_tce`、`tv`、`tv_es6`、`phone`、`house`。默认 `main`，其余用于调试。可使用 `actual` 采用网站指定值。
* `player_js_version`：用于 n/sig 解密的播放器 JavaScript 版本，格式为 `signature_timestamp@hash`（例如 `20348@0004de42`）。默认使用网站指定值，也可用 `actual` 选择。使用其他值会隐含 `webpage_skip=player_response`。
* `comment_sort`：`top` 或 `new`（默认），选择评论排序模式（在 YouTube 端）。
* `max_comments`：限制获取的评论数量。逗号分隔整数列表，依次表示 `max-comments,max-parents,max-replies,max-replies-per-thread,max-depth`。默认 `all,all,all,all,all`。
    * `max-depth` 为 `1` 时，无论 `max-replies` 或 `max-replies-per-thread` 如何设置，所有回复都会丢弃。
    * 例如 `all,all,1000,10,2` 最多获取总计 1000 条回复、每个线程最多 10 条回复，深度仅 2 层（顶层评论及其直接回复）。`1000,all,100` 最多获取 1000 条评论和总计 100 条回复。
* `formats`：更改返回的格式类型。`dashy`（将 HTTP 转换为 DASH）、`duplicate`（内容相同但 URL 或协议不同；包含 `dashy`）、`incomplete`（无法完整下载——直播和直播后 DASH、直播后 m3u8，以及未使用 `--live-from-start` 的直播自适应 HTTPS）、`missing_pot`（包含需要但缺少 PO Token 的格式）。
* `innertube_host`：所有 API 请求使用的 Innertube API 主机，例如 `studio.youtube.com`、`youtubei.googleapis.com`。从一个子域导出的 Cookie 不适用于其他子域。
* `innertube_key`：所有 API 请求使用的 Innertube API 密钥。默认不使用 API 密钥。
* `raise_incomplete_data`：`Incomplete Data Received` 会抛出错误，而非报告警告。
* `data_sync_id`：覆盖 Innertube API 请求所用账号 Data Sync ID。如果使用账号并设置 `youtube:player_skip=webpage,configs` 或 `youtubetab:skip=webpage`，可能需要此项。
* `visitor_data`：覆盖 Innertube API 请求所用 Visitor Data。应与 `player_skip=webpage,configs` 一起使用，且不使用 Cookie。注意：使用不当可能产生负面影响。如需浏览器会话，应传入 Cookie（其中包含 Visitor ID）。
* `po_token`：使用的 Proof of Origin（PO）Token，逗号分隔，格式为 `CLIENT.CONTEXT+PO_TOKEN`，例如 `youtube:po_token=web.gvs+XXX,web.player=XXX,web_safari.gvs+YYY`。上下文可为 `gvs`（Google Video Server URL）、`player`（Innertube 播放器请求）或 `subs`（字幕）。
* `pot_trace`：启用获取 PO Token 的调试日志。`true` 或 `false`（默认）。
* `fetch_pot`：从提供方获取 PO Token 的策略。可选 `always`（无论客户端是否在给定上下文中需要都尝试获取）、`never`（从不获取）或 `auto`（默认；仅在客户端需要时获取）。
* `jsc_trace`：启用获取 JS Challenge 的调试日志。`true` 或 `false`（默认）。
* `use_ad_playback_context`：跳过前贴片广告以消除下载前的强制等待。向 yt-dlp 传入高级账号 Cookie 时**不要**使用，否则会失去高级格式。仅对 `mweb` 和 `web_music` 播放器客户端有效。`true` 或 `false`（默认）。

#### youtube-ejs
* `jitless`：以无 JIT 模式运行支持的 JavaScript 引擎，支持 `deno`、`node` 和 `bun`。以性能和速度为代价提高安全性。请注意，`node` 和 `bun` 仍被认为不安全。`true` 或 `false`（默认）。

#### youtubepot-webpo
* `bind_to_visitor_id`：缓存 WebPO Token 时是否使用 Visitor ID 而非 Visitor Data。`true`（默认）或 `false`。

#### youtubetab（YouTube 播放列表、频道、信息流等）
* `skip`：`webpage`（跳过初始网页下载）或 `authcheck`（未下载初始网页时允许下载需要认证的播放列表）中的一个或多个。后者可能引发非预期行为，详情见 [#1122](https://github.com/yt-dlp/yt-dlp/pull/1122)。
* `approximate_date`：在 flat-playlist 中提取近似 `upload_date` 和 `timestamp`，可能使基于日期的过滤器略有偏差。

#### generic
* `fragment_query`：未提供值时，将 mpd/m3u8 清单 URL 的查询参数透传给分片；否则应用 `fragment_query=VALUE` 给定的查询字符串。若流包含 HLS AES-128 密钥，查询参数也会传给密钥 URI，除非传入 `key_query` 提取器参数，或通过 `hls_key` 提供外部密钥 URI。不适用于 ffmpeg。
* `variant_query`：未提供值时，将主 m3u8 URL 查询参数透传给其变体播放列表 URL；否则应用 `variant_query=VALUE` 给定的查询字符串。
* `key_query`：未提供值时，将主 m3u8 URL 查询参数透传给 HLS AES-128 解密密钥 URI；否则应用 `key_query=VALUE` 给定的查询字符串。如果通过 `hls_key` 提取器参数提供密钥 URI，此项无效。不适用于 ffmpeg。
* `hls_key`：HLS AES-128 密钥 URI *或*密钥（十六进制），以及可选 IV（十六进制），格式为 `(URI|KEY)[,IV]`，例如 `generic:hls_key=ABCDEF1234567980,0xFEDCBA0987654321`。传入任意这些值会强制使用内置 HLS 下载器，并覆盖 m3u8 播放列表中找到的相应值。
* `is_live`：跳过直播 HLS 检测，手动设置 `live_status`。值为 `false` 时设为 `not_live`；任何其他值或无值时设为 `is_live`。
* `impersonate`：初始网页请求中尝试模拟的目标，例如 `generic:impersonate=safari,chrome-110`。使用 `generic:impersonate` 模拟任意可用目标，使用 `generic:impersonate=false` 禁用模拟（默认）。

#### vikichannel
* `video_types`：要下载的视频类型，从 `episodes`、`movies`、`clips`、`trailers` 中选择一个或多个。

#### youtubewebarchive
* `check_all`：以更多请求为代价尝试检查更多内容，可选 `thumbnails`、`captures` 中一个或多个。

#### gamejolt
* `comment_sort`：选择 GameJolt 端评论排序模式：`hot`（默认）、`you`（需要 Cookie）、`top`、`new`。

#### hotstar
* `res`：要忽略的分辨率，从 `sd`、`hd`、`fhd` 中选择一个或多个。
* `vcodec`：要忽略的视频编解码器，从 `h264`、`h265`、`dvh265` 中选择一个或多个。
* `dr`：要忽略的动态范围，从 `sdr`、`hdr10`、`dv` 中选择一个或多个。

#### instagram
* `app_id`：API 请求中使用的 `X-IG-App-ID` 标头值，可以是实际 ID 数字、`ios` 或 `web`（默认）。

#### niconicochannelplus
* `max_comments`：要提取的最大评论数，默认 `120`。

#### tiktok
* `api_hostname`：移动 API 调用所用主机名，例如 `api22-normal-c-alisg.tiktokv.com`。
* `app_name`：移动 API 调用所用默认应用名称，例如 `trill`。
* `app_version`：移动 API 调用所用默认应用版本，应与 `manifest_app_version` 一起设置，例如 `34.1.2`。
* `manifest_app_version`：移动 API 调用所用默认数字应用版本，例如 `2023401020`。
* `aid`：移动 API 调用所用默认应用 ID，例如 `1180`。
* `app_info`：使用一个或多个应用信息字符串启用移动 API 提取，格式为 `<iid>/[app_name]/[app_version]/[manifest_app_version]/[aid]`，其中 `iid` 是唯一的应用安装 ID。`iid` 是唯一必需值；其他值及其 `/` 分隔符均可省略，例如 `tiktok:app_info=1234567890123456789` 或 `tiktok:app_info=123,456/trill///1180,789//34.0.1/340001`。
* `device_id`：使用真实设备 ID 启用移动 API 提取。默认是随机的 19 位字符串。

#### rokfinchannel
* `tab`：要下载的选项卡，可选 `new`、`top`、`videos`、`podcasts`、`streams`、`stacks`。

#### twitter
* `api`：选择 `graphql`（默认）、`legacy` 或 `syndication` 作为推文提取 API。登录时无效。

#### stacommu, wrestleuniverse
* `device_id`：网站分配的 UUID，用于对付费直播内容实施设备数量限制。可在浏览器本地存储中找到。

#### twitch
* `client_id`：GraphQL 请求发送的客户端 ID，例如 `twitch:client_id=kimne78kx3ncx6brgo4mv6wki5h1ko`。

#### nhkradirulive (NHK らじる★らじる LIVE)
* `area`：要提取的地区版本。有效值：`sapporo`、`sendai`、`tokyo`、`nagoya`、`osaka`、`hiroshima`、`matsuyama`、`fukuoka`。默认 `tokyo`。

#### nflplusreplay
* `type`：要提取的比赛回放类型。有效值：`full_game`、`full_game_spanish`、`condensed_game`、`all_22`。可使用默认值 `all` 提取全部可用回放类型。

#### jiocinema
* `refresh_token`：当使用 `token` 作为用户名、浏览器本地存储中的 `accessToken` 作为密码登录时，可传入本地存储中的 `refreshToken` UUID 以延长登录会话有效期。

#### jiosaavn
* `bitrate`：请求的音频码率，从 `16`、`32`、`64`、`128`、`320` 中选择一个或多个。默认 `128,320`。

#### afreecatvlive
* `cdn`：获取流 URL 的 API 调用所用一个或多个 CDN ID，例如 `gcp_cdn`、`gs_cdn_pc_app`、`gs_cdn_mobile_web`、`gs_cdn_pc_web`。

#### soundcloud
* `formats`：向 API 请求的格式，格式应为 `{protocol}_{codec}`，例如 `hls_opus,http_aac`。`*` 是通配符，可用如 `*_mp3`，也可单独使用以请求所有格式。已知协议包括 `http`、`hls`、`hls-aes`；已知编解码器包括 `aac`、`opus`、`mp3`。原始 `download` 格式始终被提取。默认 `http_aac,hls_aac,http_opus,hls_opus,http_mp3,hls_mp3`。

#### orfon (orf:on)
* `prefer_segments_playlist`：可用时，优先选择节目片段的播放列表而不是单个完整视频。如果需要各个片段，请使用 `--concat-playlist never --extractor-args "orfon:prefer_segments_playlist"`。

#### bilibili
* `prefer_multi_flv`：对于仍提供旧格式的老视频，优先提取 flv 而非 mp4 格式。

#### sonylivseries
* `sort_order`：系列提取的剧集排序顺序：`asc`（升序，最旧优先）或 `desc`（降序，最新优先）。默认 `asc`。

#### streaks
* `api_key`：用于 `X-Streaks-Api-Key` 标头的 API 密钥。

#### tver
* `backend`：提取所用后端 API：`streaks`（默认）或 `brightcove`（已弃用）。

#### vimeo
* `client`：提取视频数据所用客户端，可用 `android` 和 `web`，只能使用一个。默认使用 `web`，它仅在有账号 Cookie 或登录凭据时工作。`android` 仅在有之前缓存的 OAuth Token 时工作。
* `original_format_policy`：何时尝试提取原始格式的策略：`always`、`never` 或 `auto`。默认 `auto` 策略只在 Vimeo 公开视频可下载时发起额外请求，以避免超过 web 客户端的 API 速率限制。

#### adn
* `profile_id`：用于下载视频的高级账号配置文件数字 ID（默认 `1`）。

#### zan
* `split_angles`：将多视角流拆成独立视角格式。下载期间强制重新编码视频流，需要 ffmpeg。`true` 或 `false`（默认）。

**注意**：这些选项未来可能在不考虑向后兼容的情况下更改或移除。

<!-- MANPAGE: MOVE "INSTALLATION" SECTION HERE -->

<a id="plugins"></a>
# 插件

请注意，即使没有调用，**所有**插件也都会被导入，并且**不会检查**插件代码。**使用插件的风险由你承担，且只应使用你信任的代码！**

插件的 `<type>` 可以是 `extractor` 或 `postprocessor`。
- 提取器插件无需通过 CLI 启用；输入 URL 适合时会自动调用。
- 提取器插件优先于内置提取器。
- 后处理器插件可使用 `--use-postprocessor NAME` 调用。

插件从命名空间包 `yt_dlp_plugins.extractor` 和 `yt_dlp_plugins.postprocessor` 加载。

换言之，磁盘上的文件结构大致如下：

        yt_dlp_plugins/
            extractor/
                myplugin.py
            postprocessor/
                myplugin.py

yt-dlp 会在许多位置（见下文）查找这些 `yt_dlp_plugins` 命名空间文件夹，并从**所有**位置加载插件。将环境变量 `YTDLP_NO_PLUGINS` 设置为非空值，可完全禁用插件加载。

部分已知插件见 [Wiki](https://github.com/yt-dlp/yt-dlp/wiki/Plugins)。

<a id="installing-plugins"></a>
## 安装插件

可以通过多种方式在不同位置安装插件。

1. **配置目录**：包含 `yt_dlp_plugins` 命名空间文件夹的插件包可以放入以下标准[配置位置](#配置)：
    * **用户插件**
      * `${XDG_CONFIG_HOME}/yt-dlp/plugins/<package name>/yt_dlp_plugins/`（Linux/macOS 上推荐）
      * `${XDG_CONFIG_HOME}/yt-dlp-plugins/<package name>/yt_dlp_plugins/`
      * `${APPDATA}/yt-dlp/plugins/<package name>/yt_dlp_plugins/`（Windows 上推荐）
      * `${APPDATA}/yt-dlp-plugins/<package name>/yt_dlp_plugins/`
      * `~/.yt-dlp/plugins/<package name>/yt_dlp_plugins/`
      * `~/yt-dlp-plugins/<package name>/yt_dlp_plugins/`
    * **系统插件**
      * `/etc/yt-dlp/plugins/<package name>/yt_dlp_plugins/`
      * `/etc/yt-dlp-plugins/<package name>/yt_dlp_plugins/`
2. **可执行文件位置**：插件包同样可安装在可执行文件位置下的 `yt-dlp-plugins` 目录中（推荐用于便携安装）：
    * 二进制文件：`<root-dir>/yt-dlp.exe` 所在位置，`<root-dir>/yt-dlp-plugins/<package name>/yt_dlp_plugins/`
    * 源代码：`<root-dir>/yt_dlp/__main__.py` 所在位置，`<root-dir>/yt-dlp-plugins/<package name>/yt_dlp_plugins/`
3. **pip 和 `PYTHONPATH` 中的其他位置**
    * 可使用 `pip` 安装和管理插件包。示例见 [yt-dlp-sample-plugins](https://github.com/yt-dlp/yt-dlp-sample-plugins)。
      * 注意：通过 pip 安装的不同插件包中的插件文件必须具有唯一文件名。
    * 会在 `PYTHONPATH` 中的所有路径搜索 `yt_dlp_plugins` 命名空间文件夹。
      * 注意：这不适用于 PyInstaller 构建。

根目录中包含 `yt_dlp_plugins` 命名空间文件夹的 `.zip`、`.egg` 和 `.whl` 归档也支持作为插件包。

* 例如 `${XDG_CONFIG_HOME}/yt-dlp/plugins/mypluginpkg.zip`，其中 `mypluginpkg.zip` 包含 `yt_dlp_plugins/<type>/myplugin.py`。

使用 `--verbose` 运行 yt-dlp，可检查插件是否已加载。

<a id="developing-plugins"></a>
## 开发插件

模板插件包见 [yt-dlp-sample-plugins](https://github.com/yt-dlp/yt-dlp-sample-plugins) 仓库；插件开发指南见 Wiki 的[插件开发](https://github.com/yt-dlp/yt-dlp/wiki/Plugin-Development)部分。

每个文件中名称以 `IE`/`PP` 结尾的所有公共类分别作为提取器/后处理器导入。此过程遵循下划线前缀（例如 `_MyBasePluginIE` 是私有类）和 `__all__`。同样可以在模块名前加下划线以排除模块（例如 `_myplugin.py`）。

要用现有提取器的子类替换它，请设置 `plugin_name` 类关键字参数（例如 `class MyPluginIE(ABuiltInIE, plugin_name='myplugin')` 会用 `MyPluginIE` 替换 `ABuiltInIE`）。由于提取器替换了父类，应使用上述方法之一将子类设为私有，以避免其被单独导入。

如果你是插件作者，请为仓库添加 [yt-dlp-plugins](https://github.com/topics/yt-dlp-plugins) 主题，以便被发现。

有关如何编写和测试提取器，请参阅[开发者说明](https://github.com/yt-dlp/yt-dlp/blob/b77c9304e1c4220cca450c583491904daf8455af/CONTRIBUTING.md#developer-instructions)。

<a id="embedding-yt-dlp"></a>
# 嵌入 YT-DLP

yt-dlp 尽力成为良好的命令行程序，因此应可从任何编程语言调用。

程序应避免解析常规 stdout，因为其内容可能在未来版本中改变。应改用 `-J`、`--print`、`--progress-template`、`--exec` 等选项，创建可以可靠重现和解析的控制台输出。

在 Python 程序中，可以以更强大的方式嵌入 yt-dlp，例如：

```python
from yt_dlp import YoutubeDL

URLS = ['https://www.youtube.com/watch?v=YE7VzlLtp-4']
with YoutubeDL() as ydl:
    ydl.download(URLS)
```

你很可能需要使用各种选项。可用选项列表请查看 [`yt_dlp/YoutubeDL.py`](https://github.com/yt-dlp/yt-dlp/blob/b77c9304e1c4220cca450c583491904daf8455af/yt_dlp/YoutubeDL.py#L183)，或在 Python shell 中运行 `help(yt_dlp.YoutubeDL)`。如果已熟悉 CLI，可以使用 [`devscripts/cli_to_api.py`](https://github.com/yt-dlp/yt-dlp/blob/b77c9304e1c4220cca450c583491904daf8455af/devscripts/cli_to_api.py)将任何 CLI 开关转换为 `YoutubeDL` 参数。

**提示**：如果正将代码从 youtube-dl 迁移到 yt-dlp，一个重要注意点是：我们不保证 `YoutubeDL.extract_info` 的返回值可 JSON 序列化，甚至不保证它是字典。它会表现得像字典；如果要确保得到可序列化字典，请如[下方示例](#提取信息)所示通过 `YoutubeDL.sanitize_info` 处理。

<a id="embedding-examples"></a>
## 嵌入示例

#### 提取信息

```python
import json
import yt_dlp

URL = 'https://www.youtube.com/watch?v=YE7VzlLtp-4'

# ℹ️ 可用选项和公共函数列表见 help(yt_dlp.YoutubeDL)
ydl_opts = {}
with yt_dlp.YoutubeDL(ydl_opts) as ydl:
    info = ydl.extract_info(URL, download=False)

    # ℹ️ ydl.sanitize_info 使 info 可 JSON 序列化
    print(json.dumps(ydl.sanitize_info(info)))
```

#### 使用 info-json 下载

```python
import yt_dlp

INFO_FILE = 'path/to/video.info.json'

with yt_dlp.YoutubeDL() as ydl:
    error_code = ydl.download_with_info_file(INFO_FILE)

print('Some videos failed to download' if error_code
      else 'All videos successfully downloaded')
```

#### 提取音频

```python
import yt_dlp

URLS = ['https://www.youtube.com/watch?v=YE7VzlLtp-4']

ydl_opts = {
    'format': 'm4a/bestaudio/best',
    # ℹ️ 可用后处理器及其参数列表见 help(yt_dlp.postprocessor)
    'postprocessors': [{  # Extract audio using ffmpeg
        'key': 'FFmpegExtractAudio',
        'preferredcodec': 'm4a',
    }]
}

with yt_dlp.YoutubeDL(ydl_opts) as ydl:
    error_code = ydl.download(URLS)
```

#### 过滤视频

```python
import yt_dlp

URLS = ['https://www.youtube.com/watch?v=YE7VzlLtp-4']

def longer_than_a_minute(info, *, incomplete):
    """Download only videos longer than a minute (or with unknown duration)"""
    duration = info.get('duration')
    if duration and duration < 60:
        return 'The video is too short'

ydl_opts = {
    'match_filter': longer_than_a_minute,
}

with yt_dlp.YoutubeDL(ydl_opts) as ydl:
    error_code = ydl.download(URLS)
```

#### 添加日志记录器和进度钩子

```python
import yt_dlp

URLS = ['https://www.youtube.com/watch?v=YE7VzlLtp-4']

class MyLogger:
    def debug(self, msg):
        # 为了兼容 youtube-dl，debug 和 info 都会传给 debug
        # 可以通过前缀 '[debug] ' 区分它们
        if msg.startswith('[debug] '):
            pass
        else:
            self.info(msg)

    def info(self, msg):
        pass

    def warning(self, msg):
        pass

    def error(self, msg):
        print(msg)


# ℹ️ 参见 help(yt_dlp.YoutubeDL) 中的 "progress_hooks"
def my_hook(d):
    if d['status'] == 'finished':
        print('Done downloading, now post-processing ...')


ydl_opts = {
    'logger': MyLogger(),
    'progress_hooks': [my_hook],
}

with yt_dlp.YoutubeDL(ydl_opts) as ydl:
    ydl.download(URLS)
```

#### 添加自定义 PostProcessor

```python
import yt_dlp

URLS = ['https://www.youtube.com/watch?v=YE7VzlLtp-4']

# ℹ️ 参见 help(yt_dlp.postprocessor.PostProcessor)
class MyCustomPP(yt_dlp.postprocessor.PostProcessor):
    def run(self, info):
        self.to_screen('Doing stuff')
        return [], info


with yt_dlp.YoutubeDL() as ydl:
    # ℹ️ "when" 可以取 yt_dlp.utils.POSTPROCESS_WHEN 中的任意值
    ydl.add_post_processor(MyCustomPP(), when='pre_process')
    ydl.download(URLS)
```

#### 使用自定义格式选择器

```python
import yt_dlp

URLS = ['https://www.youtube.com/watch?v=YE7VzlLtp-4']

def format_selector(ctx):
    """ Select the best video and the best audio that won't result in an mkv.
    NOTE: This is just an example and does not handle all cases """

    # formats 已按从最差到最佳排序
    formats = ctx.get('formats')[::-1]

    # acodec='none' 表示没有音频
    best_video = next(f for f in formats
                      if f['vcodec'] != 'none' and f['acodec'] == 'none')

    # 查找兼容的音频扩展名
    audio_ext = {'mp4': 'm4a', 'webm': 'webm'}[best_video['ext']]
    # vcodec='none' 表示没有视频
    best_audio = next(f for f in formats if (
        f['acodec'] != 'none' and f['vcodec'] == 'none' and f['ext'] == audio_ext))

    # 这些是合并格式所需的最少字段
    yield {
        'format_id': f'{best_video["format_id"]}+{best_audio["format_id"]}',
        'ext': best_video['ext'],
        'requested_formats': [best_video, best_audio],
        # 必须是用 + 分隔的协议列表
        'protocol': f'{best_video["protocol"]}+{best_audio["protocol"]}'
    }


ydl_opts = {
    'format': format_selector,
}

with yt_dlp.YoutubeDL(ydl_opts) as ydl:
    ydl.download(URLS)
```

<a id="changes-from-youtube-dl"></a>
# 与 YOUTUBE-DL 的变化

<a id="new-features"></a>
### 新功能

* 从 [**yt-dlc@f9401f2**](https://github.com/blackjack4494/yt-dlc/commit/f9401f2a91987068139c5f757b12fc711d4c0cee) 派生，并与 [**youtube-dl@a08f2b7**](https://github.com/ytdl-org/youtube-dl/commit/a08f2b7e4567cdc50c0614ee0a4ffdff49b8b6e6) 合并（[例外](https://github.com/yt-dlp/yt-dlp/issues/21)）。
* **[SponsorBlock 集成](#sponsorblock-options)**：可以利用 [SponsorBlock](https://sponsor.ajay.app) API 标记或移除 YouTube 视频中的赞助片段。
* **[格式排序](#sorting-formats)**：默认排序改为优先选择更高分辨率和更好的编解码器，而非单纯选择更大比特率。还可用 `-S` 指定排序顺序，比仅用 `--format` 更容易选择格式（[示例](#format-selection-examples)）。
* **合并 animelover1984/youtube-dl**：获得 [animelover1984/youtube-dl](https://github.com/animelover1984/youtube-dl) 的大多数功能与改进，包括 `--write-comments`、`BiliBiliSearch`、`BilibiliChannel`、在 mp4/ogg/opus 中嵌入缩略图、播放列表 infojson 等。详见 [#31](https://github.com/yt-dlp/yt-dlp/pull/31)。
* **YouTube 改进**：
    * 支持 Clips、Stories（`ytstories:<channel UCID>`）、搜索（包括过滤器）**\***、YouTube Music 搜索、指定频道内搜索、搜索前缀（`ytsearch:`）**\***、Mixes 和动态（`:ytfav`、`:ytwatchlater`、`:ytsubs`、`:ythistory`、`:ytrec`、`:ytnotif`）。
    * 修复[基于 n-sig 的限速](https://github.com/ytdl-org/youtube-dl/issues/29326) **\***。
    * 使用 `--live-from-start` 从头下载直播（*实验性*）。
    * 频道 URL 下载频道上传的全部内容，包括 Shorts 和直播。
* **从浏览器获取 Cookie**：使用 `--cookies-from-browser BROWSER[+KEYRING][:PROFILE][::CONTAINER]` 可自动从主要浏览器提取 Cookie。
* **下载时间范围**：使用 `--download-sections` 按时间戳或章节下载视频的一部分。
* **按章节拆分视频**：使用 `--split-chapters` 按章节拆成多个文件。
* **多线程分片下载**：并行下载 m3u8/mpd 视频的多个分片。使用 `--concurrent-fragments`（`-N`）设置线程数。
* **新增和修复的提取器**：新增了许多提取器并修复了许多现有提取器。参见[更新日志](https://github.com/yt-dlp/yt-dlp/blob/b77c9304e1c4220cca450c583491904daf8455af/Changelog.md)或[支持的网站列表](https://github.com/yt-dlp/yt-dlp/blob/b77c9304e1c4220cca450c583491904daf8455af/supportedsites.md)。
* **新的 MSO**：Philo、Spectrum、SlingTV、Cablevision、RCN 等。
* **从清单提取字幕**：可以从流媒体清单中提取字幕，详见 [commit/be6202f](https://github.com/yt-dlp/yt-dlp/commit/be6202f12b97858b9d716e608394b51065d0419f)。
* **多路径和输出模板**：可针对不同类型的文件指定不同的[输出模板](#output-template)和下载路径，也可用 `--paths`（`-P`）指定下载中间文件的临时路径。
* **便携配置**：自动从主目录和根目录加载配置文件，详见[配置](#configuration)。
* **输出模板改进**：输出模板现在支持日期时间格式化、数值偏移、对象遍历等，详见[输出模板](#output-template)。还可借助 `--parse-metadata` 和 `--replace-in-metadata` 执行更高级的操作。
* **其他新选项**：新增 `--alias`、`--print`、`--concat-playlist`、`--wait-for-video`、`--retry-sleep`、`--sleep-requests`、`--convert-thumbnails`、`--force-download-archive`、`--force-overwrites`、`--break-match-filters` 等选项。
* **改进**：`--format`/`--match-filters` 中的正则表达式和其他运算符、多个 `--postprocessor-args` 和 `--downloader-args`、更快的归档检查、更多[格式选择选项](#format-selection)、合并多个视频/音频、多个 `--config-locations`、不同阶段的 `--exec` 等。
* **插件**：可以从外部文件加载提取器和后处理器，详见[插件](#plugins)。
* **自更新**：可用 `yt-dlp -U` 更新发行版，必要时使用 `--update-to` 降级。
* **自动构建**：可通过 `--update-to nightly` 和 `--update-to master` 使用 [nightly/master 构建](#update-channels)。

完整变化列表参见[更新日志](https://github.com/yt-dlp/yt-dlp/blob/b77c9304e1c4220cca450c583491904daf8455af/Changelog.md)或[提交记录](https://github.com/yt-dlp/yt-dlp/commits)。标有 **\*** 的功能已反向移植到 youtube-dl。

<a id="differences-in-default-behavior"></a>
### 默认行为的区别

部分 yt-dlp 默认选项与 youtube-dl 和 youtube-dlc 不同：

* yt-dlp 仅支持 [Python 3.11+](#dependencies)，随着版本[结束支持](https://devguide.python.org/versions/#python-release-cycle)还会移除对更多版本的支持；[youtube-dl 仍支持 Python 2.6+ 和 3.2+](https://github.com/ytdl-org/youtube-dl/issues/30568#issue-1118238743)。
* 某些选项不再有效，详见[已移除选项](#removed)。
* 不支持用 `avconv` 替代 `ffmpeg`。
* yt-dlp 配置文件的位置与 youtube-dl 略有不同，正确位置参见[配置](#configuration)。
* 默认[输出模板](#output-template)是 `%(title)s [%(id)s].%(ext)s`。这个改动没有特别的理由：它在 yt-dlp 公开之前就已修改，现在没有计划改回 `%(title)s-%(id)s.%(ext)s`。你可以改用 `--compat-options filename`。
* 默认[格式排序](#sorting-formats)不同于 youtube-dl，优先选择更高分辨率和更好的编解码器，而不是更高比特率。可用 `--format-sort` 自定义顺序，或使用 `--compat-options format-sort` 采用 youtube-dl 的排序。旧版 yt-dlp 因兼容性更广而优先选择 VP9；可以用 `--compat-options prefer-vp9-sort` 恢复该偏好。这两个兼容选项不能同时使用。
* 默认格式选择器是 `bv*+ba/b`。如果找到比最佳纯视频格式更好的视频音频合并格式，会优先选择前者。使用 `-f bv+ba/b` 或 `--compat-options format-spec` 可恢复原行为。
* 与 youtube-dlc 不同，yt-dlp 默认不允许将多个音频或视频流合并到一个文件（因为这与 `-f bv*+ba` 冲突）。需要时必须用 `--audio-multistreams` 和 `--video-multistreams` 开启，也可用 `--compat-options multistreams` 同时开启两者。
* 默认启用 `--no-abort-on-error`。可用 `--abort-on-error` 或 `--compat-options abort-on-error` 在出错时中止。
* 写入缩略图、描述或 infojson 等元数据文件时，也会为播放列表写入相同信息（如果可用）。使用 `--no-write-playlist-metafiles` 或 `--compat-options no-playlist-metafiles` 可关闭。
* `--add-metadata` 除写入元数据外，在与 `--write-info-json` 一起使用时还会将 `infojson` 附加到 `mkv` 文件。可用 `--no-embed-info-json` 或 `--compat-options no-attach-info-json` 恢复原行为。
* 使用 `--add-metadata` 时，部分元数据嵌入的字段与 youtube-dl 不同，最明显的是 `comment` 包含 `webpage_url`、`synopsis` 包含 `description`。可以[用 `--parse-metadata`](#modifying-metadata) 按需修改，或用 `--compat-options embed-metadata` 恢复原行为。
* `playlist_index` 与 `--playlist-reverse`、`--playlist-items` 等选项一起使用时行为不同，详见 [#302](https://github.com/yt-dlp/yt-dlp/issues/302)。可用 `--compat-options playlist-index` 保持以前的行为。
* `-F` 以新格式列出输出；用 `--compat-options list-formats` 恢复。
* 直播聊天（如可用）被视为字幕。用 `--sub-langs all,-live_chat` 下载除直播聊天之外的所有字幕，也可用 `--compat-options no-live-chat` 禁止下载直播聊天或弹幕。
* YouTube 频道 URL 下载频道所有上传内容。只下载特定标签页的视频时，请传入该标签页的 URL；如果频道没有请求的标签页，将报错。`/live` URL 没有直播时也会报错，而非静默下载整个频道。可以用 `--compat-options no-youtube-channel-redirect` 恢复这些重定向行为。
* YouTube 播放列表也会列出不可用的视频，使用 `--compat-options no-youtube-unavailable-videos` 可移除。
* 从 YouTube 提取的上传日期使用 UTC。
* 如果使用 `ffmpeg` 下载，在可能时会将格式下载和合并放在一步完成；用 `--compat-options no-direct-merge` 恢复原行为。
* 在可行时使用 mutagen 将缩略图嵌入 `mp4`，可用 `--compat-options embed-thumbnail-atomicparsley` 强制使用 AtomicParsley。
* 默认从 infojson 移除文件名等一些内部元数据。可用 `--no-clean-infojson` 或 `--compat-options no-clean-infojson` 恢复。
* 同时使用 `--embed-subs` 和 `--write-subs` 时，字幕既写入磁盘也嵌入媒体文件。只用 `--embed-subs` 可嵌入字幕并自动删除单独文件，详见 [#630（评论）](https://github.com/yt-dlp/yt-dlp/issues/630#issuecomment-893659460)。可用 `--compat-options no-keep-subs` 恢复原行为。
* 如已安装 `certifi`，将使用它提供 SSL 根证书。若要使用系统证书（例如自签名证书），请用 `--compat-options no-certifi`。
* yt-dlp 对文件名无效字符的清理方式与 youtube-dl 不同，也更智能。可用 `--compat-options filename-sanitization` 恢复 youtube-dl 行为。
* （目前未实现）~~yt-dlp 尽可能将外部下载器的输出解析成标准进度输出。可以使用 `--compat-options no-external-downloader-progress` 原样获取下载器输出。~~

* yt-dlp 2021.09.01 至 2022.11.11（含）版本会对嵌套播放列表应用 `--match-filters`。这是 [8f18ac](https://github.com/yt-dlp/yt-dlp/commit/8f18aca8717bb0dd49054555af8d386e5eda3a88) 的意外副作用，已在 [d7b460](https://github.com/yt-dlp/yt-dlp/commit/d7b460d0e5fc710950582baed2e3fc616ed98a80) 修复。用 `--compat-options playlist-match-filter` 恢复旧行为。
* yt-dlp 2021.11.10 至 2023.06.21（含）版本会为分片或清单格式估计 `filesize_approx`。为方便起见在 [f2fe69](https://github.com/yt-dlp/yt-dlp/commit/f2fe69c7b0d208bdb1f6292b4ae92bc1e1a7444a) 加入，但由于估计值可能极不准确而在 [0dff8e](https://github.com/yt-dlp/yt-dlp/commit/0dff8e4d1e6e9fb938f4256ea9af7d81f42fd54f) 撤销。用 `--compat-options manifest-filesize-approx` 继续提取估计值。
* yt-dlp 使用 `requests` 等现代 HTTP 客户端后端。使用 `--compat-options prefer-legacy-http-handler` 可优先使用旧 HTTP 处理器（`urllib`）处理标准 HTTP 请求。
* 已移除 `swfinterp`、`casefold` 子模块。
* 传入 `--simulate`（或以 `download=False` 调用 `extract_info`）不再改变默认格式选择。详见 [#9843](https://github.com/yt-dlp/yt-dlp/issues/9843)。
* yt-dlp 默认不再将服务器修改时间应用于下载文件。使用 `--mtime` 或 `--compat-options mtime-by-default` 可恢复旧行为。

为方便使用，提供以下兼容选项别名：

* `--compat-options all`：使用所有兼容选项（**不要使用！**）
* `--compat-options youtube-dl`：等同于 `--compat-options all,-multistreams,-playlist-match-filter,-manifest-filesize-approx,-allow-unsafe-ext,-prefer-vp9-sort,-allow-unsafe-exec-expansion`
* `--compat-options youtube-dlc`：等同于 `--compat-options all,-no-live-chat,-no-youtube-channel-redirect,-playlist-match-filter,-manifest-filesize-approx,-allow-unsafe-ext,-prefer-vp9-sort,-allow-unsafe-exec-expansion`
* `--compat-options 2021`：等同于 `--compat-options 2022,no-certifi,filename-sanitization`
* `--compat-options 2022`：等同于 `--compat-options 2023,playlist-match-filter,no-external-downloader-progress,prefer-legacy-http-handler,manifest-filesize-approx`
* `--compat-options 2023`：等同于 `--compat-options 2024,prefer-vp9-sort`
* `--compat-options 2024`：等同于 `--compat-options 2025,mtime-by-default`
* `--compat-options 2025`：目前没有作用。用它启用未来的所有兼容选项。

使用某个按年份命名的兼容选项别名会将 yt-dlp 的默认行为固定到该公历年*年末*的行为。

以下兼容选项会恢复安全补丁之前存在漏洞的行为：

* `--compat-options allow-unsafe-ext`：允许下载任意扩展名的文件（包括不安全的扩展名）（[GHSA-79w7-vh3h-8g4j](https://github.com/yt-dlp/yt-dlp/security/advisories/GHSA-79w7-vh3h-8g4j)）。

    > :warning: 仅当有效文件因扩展名被识别为少见而遭拒绝时使用。
    >
    > **此选项可能导致远程代码执行！**请考虑改为[提交 issue](https://github.com/yt-dlp/yt-dlp/issues/new/choose)！

* `--compat-options allow-unsafe-exec-expansion`：`--exec` 选项允许在命令中使用输出模板语法；但出于安全原因，可用的转换仅限 `i`/`d`（有符号十进制整数）、`f`（十进制浮点数）和 `q`（经 shell 引号处理）。yt-dlp 2021.04.11 至 2026.03.17（含）版本未施加这一限制。此选项会撤销该限制。

    > :warning: **此选项可能导致远程代码执行！**请考虑在 exec 命令模板中对所有字符串值使用 `%()q` 转换。

<a id="deprecated-options"></a>
### 已弃用的选项

以下列出所有已弃用选项，以及目前实现相同效果的替代方式。

#### 近乎重复的选项
这些选项与新选项几乎相同，但仍有一些区别，因此并非完全重复。

    -j, --dump-json                  --print "%()j"
    -F, --list-formats               --print formats_table
    --list-thumbnails                --print thumbnails_table --print playlist:thumbnails_table
    --list-subs                      --print automatic_captions_table --print subtitles_table

#### 重复的选项
虽然这些选项重复，但由于使用方便，预计仍会有人使用。

    --get-description                --print description
    --get-duration                   --print duration_string
    --get-filename                   --print filename
    --get-format                     --print format
    --get-id                         --print id
    --get-thumbnail                  --print thumbnail
    -e, --get-title                  --print title
    -g, --get-url                    --print urls
    --match-title REGEX              --match-filters "title ~= (?i)REGEX"
    --reject-title REGEX             --match-filters "title !~= (?i)REGEX"
    --min-views COUNT                --match-filters "view_count >=? COUNT"
    --max-views COUNT                --match-filters "view_count <=? COUNT"
    --break-on-reject                Use --break-match-filters
    --user-agent UA                  --add-headers "User-Agent:UA"
    --referer URL                    --add-headers "Referer:URL"
    --playlist-start NUMBER          -I NUMBER:
    --playlist-end NUMBER            -I :NUMBER
    --playlist-reverse               -I ::-1
    --no-playlist-reverse            Default
    --no-colors                      --color no_color

#### 不推荐
这些选项仍然有效，但由于有其他实现相同效果的方法，不推荐使用。

    --force-generic-extractor        --ies generic,default
    --exec-before-download CMD       --exec "before_dl:CMD"
    --no-exec-before-download        --no-exec
    --all-formats                    -f all
    --all-subs                       --sub-langs all --write-subs
    --print-json                     -j --no-simulate
    --autonumber-size NUMBER         Use string formatting, e.g. %(autonumber)03d
    --autonumber-start NUMBER        Use internal field formatting like %(autonumber+NUMBER)s
    --id                             -o "%(id)s.%(ext)s"
    --metadata-from-title FORMAT     --parse-metadata "%(title)s:FORMAT"
    --hls-prefer-native              --downloader "m3u8:native"
    --hls-prefer-ffmpeg              --downloader "m3u8:ffmpeg"
    --list-formats-old               --compat-options list-formats (Alias: --no-list-formats-as-table)
    --list-formats-as-table          --compat-options -list-formats [Default]
    --geo-bypass                     --xff "default"
    --no-geo-bypass                  --xff "never"
    --geo-bypass-country CODE        --xff CODE
    --geo-bypass-ip-block IP_BLOCK   --xff IP_BLOCK

#### 开发者选项
这些选项并非供最终用户使用。

    --test                           Download only part of video for testing extractors
    --load-pages                     Load pages dumped by --write-pages
    --allow-unplayable-formats       List unplayable formats also
    --no-allow-unplayable-formats    Default

#### 旧别名
以下别名因各种原因不再列入文档。

    --clean-infojson                 --clean-info-json
    --force-write-download-archive   --force-write-archive
    --no-clean-infojson              --no-clean-info-json
    --no-split-tracks                --no-split-chapters
    --no-write-srt                   --no-write-subs
    --prefer-unsecure                --prefer-insecure
    --rate-limit RATE                --limit-rate RATE
    --split-tracks                   --split-chapters
    --srt-lang LANGS                 --sub-langs LANGS
    --trim-file-names LENGTH         --trim-filenames LENGTH
    --write-srt                      --write-subs
    --yes-overwrites                 --force-overwrites

#### Sponskrub 选项
已移除对 [SponSkrub](https://github.com/faissaloo/SponSkrub) 的支持，改用 `--sponsorblock` 选项。

    --sponskrub                      --sponsorblock-mark all
    --no-sponskrub                   --no-sponsorblock
    --sponskrub-cut                  --sponsorblock-remove all
    --no-sponskrub-cut               --sponsorblock-remove -all
    --sponskrub-force                Not applicable
    --no-sponskrub-force             Not applicable
    --sponskrub-location             Not applicable
    --sponskrub-args                 Not applicable

#### 不再支持
这些选项可能不再按预期工作。

    --prefer-avconv                  avconv is not officially supported by yt-dlp (Alias: --no-prefer-ffmpeg)
    --prefer-ffmpeg                  Default (Alias: --no-prefer-avconv)
    -C, --call-home                  Not implemented
    --no-call-home                   Default
    --include-ads                    No longer supported
    --no-include-ads                 Default
    --write-annotations              No supported site has annotations now
    --no-write-annotations           Default
    --cn-verification-proxy URL      Removed alias for --geo-verification-proxy URL
    --youtube-skip-dash-manifest     Removed alias for --extractor-args "youtube:skip=dash" (Alias: --no-youtube-include-dash-manifest)
    --youtube-skip-hls-manifest      Removed alias for --extractor-args "youtube:skip=hls" (Alias: --no-youtube-include-hls-manifest)
    --youtube-include-dash-manifest  Default (Alias: --no-youtube-skip-dash-manifest)
    --youtube-include-hls-manifest   Default (Alias: --no-youtube-skip-hls-manifest)
    --youtube-print-sig-code         Removed testing functionality
    --dump-user-agent                No longer supported
    --xattr-set-filesize             No longer supported
    --compat-options seperate-video-versions  No longer needed
    --compat-options no-youtube-prefer-utc-upload-date  No longer supported

<a id="removed"></a>
#### 已移除
这些别名已被移除，以后可能重新用于其他用途。请相应更新脚本和配置文件。

    -A, --auto-number                -o "%(autonumber)s-%(id)s.%(ext)s"
    -t, -l, --title, --literal       -o "%(title)s-%(id)s.%(ext)s"
    --avconv-location                --ffmpeg-location
    --dump-headers                   --print-traffic
    --dump-intermediate-pages        --dump-pages



表中自然语言说明：`Use --break-match-filters` 表示使用 `--break-match-filters`；`Default` 表示默认行为；`Use string formatting, e.g. %(autonumber)03d` 表示使用字符串格式化，例如 `%(autonumber)03d`；`Use internal field formatting like %(autonumber+NUMBER)s` 表示使用内部字段格式化，例如 `%(autonumber+NUMBER)s`；`Alias` 表示别名；`Download only part of video for testing extractors` 表示仅下载视频的一部分以测试提取器；`Load pages dumped by --write-pages` 表示加载由 `--write-pages` 转储的页面；`List unplayable formats also` 表示同时列出不可播放的格式；`Not applicable` 表示不适用；`avconv is not officially supported by yt-dlp` 表示 yt-dlp 不正式支持 avconv；`Not implemented` 表示未实现；`No longer supported` 表示不再支持；`No supported site has annotations now` 表示目前没有受支持的网站提供注释；`Removed alias for` 表示已移除的别名；`Removed testing functionality` 表示已移除的测试功能；`No longer needed` 表示不再需要。

# 参与贡献
请参阅 [CONTRIBUTING.md](https://github.com/yt-dlp/yt-dlp/blob/b77c9304e1c4220cca450c583491904daf8455af/CONTRIBUTING.md#contributing-to-yt-dlp)，了解如何[提交 Issue](https://github.com/yt-dlp/yt-dlp/blob/b77c9304e1c4220cca450c583491904daf8455af/CONTRIBUTING.md#opening-an-issue) 和[为项目贡献代码](https://github.com/yt-dlp/yt-dlp/blob/b77c9304e1c4220cca450c583491904daf8455af/CONTRIBUTING.md#developer-instructions)。

# WIKI
更多信息请参阅 [Wiki](https://github.com/yt-dlp/yt-dlp/wiki)。
