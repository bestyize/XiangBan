<a href="https://www.navidrome.org"><img src="https://raw.githubusercontent.com/navidrome/navidrome/563c5c72a16f7128dd2c31e46d7438af5e6c1399/resources/logo-192x192.png" alt="Navidrome logo" title="navidrome" align="right" height="60px" /></a>

# Navidrome Music Server &nbsp;[![Tweet](https://img.shields.io/twitter/url/http/shields.io.svg?style=social)](https://twitter.com/intent/tweet?text=Tired%20of%20paying%20for%20music%20subscriptions%2C%20and%20not%20finding%20what%20you%20really%20like%3F%20Roll%20your%20own%20streaming%20service%21&url=https://navidrome.org&via=navidrome)

[![Last Release](https://img.shields.io/github/v/release/navidrome/navidrome?logo=github&label=latest&style=flat-square)](https://github.com/navidrome/navidrome/releases)
[![Build](https://img.shields.io/github/actions/workflow/status/navidrome/navidrome/pipeline.yml?branch=master&logo=github&style=flat-square)](https://nightly.link/navidrome/navidrome/workflows/pipeline/master)
[![Downloads](https://img.shields.io/github/downloads/navidrome/navidrome/total?logo=github&style=flat-square)](https://github.com/navidrome/navidrome/releases/latest)
[![Docker Pulls](https://img.shields.io/endpoint?url=https%3A%2F%2Fwww.navidrome.org%2Fbadges%2Fcontainer-downloads.json&logo=docker&label=pulls&style=flat-square)](https://hub.docker.com/r/navidrome/navidrome)
[![Dev Chat](https://img.shields.io/discord/671335427726114836?logo=discord&label=discord&style=flat-square)](https://discord.gg/xh7j7yF)
[![Subreddit](https://img.shields.io/reddit/subreddit-subscribers/navidrome?logo=reddit&label=/r/navidrome&style=flat-square)](https://www.reddit.com/r/navidrome/)
[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-v2.0-ff69b4.svg?style=flat-square)](https://github.com/navidrome/navidrome/blob/563c5c72a16f7128dd2c31e46d7438af5e6c1399/CODE_OF_CONDUCT.md)
[![Gurubase](https://img.shields.io/badge/Gurubase-Ask%20Navidrome%20Guru-006BFF?style=flat-square)](https://gurubase.io/g/navidrome)

Navidrome 是一款开源的基于 Web 的音乐收藏服务器和流媒体服务，让你可以从任何浏览器或移动设备收听自己的音乐收藏。它就像你自己的 Spotify！

**注意**：`master` 分支在开发期间可能不稳定，甚至无法正常工作。为了获取稳定的二进制版本，请使用[发布版本](https://github.com/navidrome/navidrome/releases)，而不是 `master` 分支。

## [查看在线演示！](https://www.navidrome.org/demo/)

__欢迎各种反馈！__如果你需要或希望增加某项功能、发现错误，或想到任何改进 Navidrome 的方法，请提交 [GitHub issue](https://github.com/navidrome/navidrome/issues) 或在我们的 [Subreddit](https://www.reddit.com/r/navidrome/) 参与讨论。如果你希望以其他方式为项目作贡献（[界面或后端开发](https://www.navidrome.org/docs/developers/)、[翻译](https://www.navidrome.org/docs/developers/translations/)、[主题](https://www.navidrome.org/docs/developers/creating-themes)），请加入我们的 [Discord 服务器](https://discord.gg/xh7j7yF)聊天。

## 安装

请参阅[项目网站](https://www.navidrome.org/docs/installation/)上的说明。

## 云托管

多家云托管服务商与我们合作，提供[官方支持的云托管方案](https://www.navidrome.org/docs/installation/managed)。如果你注册其中任何一家服务商，其收入的一部分将用于资助 Navidrome 开发，而不会向你额外收费。

<a href="https://www.pikapods.com/pods?run=navidrome"><img src="https://raw.githubusercontent.com/navidrome/navidrome/563c5c72a16f7128dd2c31e46d7438af5e6c1399/resources/hosting/pikapods.svg" alt="Run on PikaPods" width="200" height="40"></a>
<br>
<a href="https://zenith.hosting/host/navidrome?ref=navidrome"><img src="https://raw.githubusercontent.com/navidrome/navidrome/563c5c72a16f7128dd2c31e46d7438af5e6c1399/resources/hosting/zenith.svg" alt="Deploy with Zenith" width="200" height="40"></a>
<br>
<a href="https://store.elfhosted.com/product/navidrome/"><img src="https://raw.githubusercontent.com/navidrome/navidrome/563c5c72a16f7128dd2c31e46d7438af5e6c1399/resources/hosting/elfhosted.svg" alt="Deploy on ElfHosted" width="200" height="40"></a>






## 功能

- 处理非常**庞大的音乐收藏**
- 流式播放几乎**任何音频格式**
- 读取并使用你精心整理的所有**元数据**
- 良好支持**合辑**（多位艺术家的专辑）和**套装**（多碟专辑）
- **多用户**：每个用户拥有自己的播放次数、播放列表、收藏等
- **资源占用很低**
- **跨平台**：可在 macOS、Linux 和 Windows 上运行，也提供 **Docker** 镜像
- 提供所有主要平台的即用型二进制文件，包括 **Raspberry Pi**
- 自动**监视音乐库**变化，导入新文件并重新加载新元数据
- 支持附属的 .ttml、.yaml/.yml Lyricsfile、.elrc、.lrc、.srt、.txt 歌词文件，以及嵌入的 TTML、Enhanced LRC、LRC、SRT 和纯文本标签（通过 `lyricspriority`）
- 基于 [Material UI](https://material-ui.com) 的现代响应式 **Web 界面**，**可更换主题**
- 与所有 Subsonic/Madsonic/Airsonic [客户端](https://www.navidrome.org/docs/overview/#apps)**兼容**
- 即时**转码**，可按用户或播放器设置；**支持 Opus 编码**
- 已翻译为**多种语言**

## 翻译

Navidrome 使用 [POEditor](https://poeditor.com/) 进行翻译，我们一直在寻找[更多贡献者](https://www.navidrome.org/docs/developers/translations/)。

<a href="https://poeditor.com/"> 
<img height="32" src="https://github.com/user-attachments/assets/c19b1d2b-01e1-4682-a007-12356c42147c">
</a>

## 文档

所有文档都可以在项目网站找到：https://www.navidrome.org/docs。以下是一些有用的直接链接：

- [概览](https://www.navidrome.org/docs/overview/)
- [安装](https://www.navidrome.org/docs/installation/)
  - [Docker](https://www.navidrome.org/docs/installation/docker/)
  - [二进制文件](https://www.navidrome.org/docs/installation/pre-built-binaries/)
  - [从源代码构建](https://www.navidrome.org/docs/installation/build-from-source/)
- [开发](https://www.navidrome.org/docs/developers/)
- [Subsonic API 兼容性](https://www.navidrome.org/docs/developers/subsonic-api/)

## 截图

<p align="left">
    <img height="550" src="https://raw.githubusercontent.com/navidrome/navidrome/563c5c72a16f7128dd2c31e46d7438af5e6c1399/.github/screenshots/ss-mobile-login.png">
    <img height="550" src="https://raw.githubusercontent.com/navidrome/navidrome/563c5c72a16f7128dd2c31e46d7438af5e6c1399/.github/screenshots/ss-mobile-player.png">
    <img height="550" src="https://raw.githubusercontent.com/navidrome/navidrome/563c5c72a16f7128dd2c31e46d7438af5e6c1399/.github/screenshots/ss-mobile-album-view.png">
    <img width="550" src="https://raw.githubusercontent.com/navidrome/navidrome/563c5c72a16f7128dd2c31e46d7438af5e6c1399/.github/screenshots/ss-desktop-player.png">
</p>
