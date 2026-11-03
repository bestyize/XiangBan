

<p align="center">
  <a href="https://openrct2.io">
    <img src="https://raw.githubusercontent.com/OpenRCT2/OpenRCT2/5df376f311e03a90ad162334d145cc1fbc0e671b/resources/logo/icon_x128.png" style="width: 128px;" alt="OpenRCT2 logo"/>
  </a>
</p>

<h1 align="center">OpenRCT2</h1>

<h3 align="center">《过山车大亨 2》的开源重新实现。这是一款模拟游乐园管理的建造与经营游戏。</h3>

---

![Still from the v0.5.0 title sequence](https://github.com/user-attachments/assets/fa893cc8-1484-4751-94be-4ead00a6c8f9)


---

### 下载
| 最新发布版本                                                                                                       | 最新开发构建 |
|----------------------------------------------------------------------------------------------------------------------|--------------------------|
| [![OpenRCT2.io](https://img.shields.io/github/v/release/OpenRCT2/OpenRCT2.svg?color=green)](https://openrct2.io/download/release/latest) | [![OpenRCT2.io](https://img.shields.io/github/last-commit/OpenRCT2/OpenRCT2/develop?color=green)](https://openrct2.io/download/develop/latest) |

---

### 聊天

聊天在 Discord 进行。如果还没有 Discord 账号，需要先创建一个。

如果想帮助*制作*游戏，请加入开发者频道。

如果需要帮助、想与开发者交流或只是想了解最新动态，请加入你所使用语言的非开发者频道。

如果想帮助将游戏翻译成你的语言，请访问本地化频道。

| 语言 | 非开发者 | 开发者 | 本地化 | 素材替换 |
| -------- | ------------- | --------- | ------------ | ----------------- |
| English | [![Discord](https://img.shields.io/badge/discord-%23openrct2--talk-blue.svg)](https://discord.gg/ZXZd8D8) </br> [![Discord](https://img.shields.io/badge/discord-%23help-blue.svg)](https://discord.gg/vJABqGGTEt) | [![Discord](https://img.shields.io/badge/discord-%23development-yellowgreen.svg)](https://discord.gg/fsEwSWs) | [![Discord](https://img.shields.io/badge/discord-%23localisation-green.svg)](https://discord.gg/sxnrvX9) | [![Discord](https://img.shields.io/badge/discord-%23open--graphics-b00b69.svg)](https://discord.gg/aM2Pchscnp) </br> [![Discord](https://img.shields.io/badge/discord-%23open--sound--and--music-b00b69.svg)](https://discord.gg/tuz3QBBWJf)
| Nederlands | [![Discord](https://img.shields.io/badge/discord-%23nederlands-orange.svg)](https://discord.gg/cQYSXzW) | | |

---

# 目录
- 1 - [介绍](#1-介绍)
- 2 - [下载游戏（预构建版）](#2-下载游戏预构建版)
- 3 - [构建游戏](#3-构建游戏)
- 4 - [参与贡献](#4-参与贡献)
  - 4.1 - [错误修复](#41-错误修复)
  - 4.2 - [新功能](#42-新功能)
  - 4.3 - [翻译](#43-翻译)
  - 4.4 - [图形](#44-图形)
  - 4.5 - [音频](#45-音频)
  - 4.6 - [场景](#46-场景)
- 5 - [政策](#5-政策)
  - 5.1 - [行为准则](#51-行为准则)
  - 5.2 - [代码签名政策](#52-代码签名政策)
  - 5.3 - [隐私政策](#53-隐私政策)
- 6 - [许可证](#6-许可证)
- 7 - [更多信息](#7-更多信息)
- 8 - [赞助商](#8-赞助商)

---

# 1. 介绍

**OpenRCT2** 是《过山车大亨 2》（RCT2）的开源重新实现。游戏围绕建造和维护包含游乐设施、商店及其他设施的游乐园展开。玩家需要努力盈利、保持良好的公园声誉，同时让游客满意。OpenRCT2 支持场景和沙盒两种玩法：场景要求玩家在规定时间内完成特定目标，而沙盒允许玩家更灵活地建造公园，并可选择不受限制或资金约束。

《过山车大亨 2》最初由 Chris Sawyer 使用 x86 汇编编写，是《过山车大亨》的续作。引擎基于更早的游戏 Transport Tycoon；该游戏也有对应的开源项目 [OpenTTD](https://openttd.org)。OpenRCT2 尝试提供 RCT2 的全部内容，并增加许多改进和功能，包括支持现代平台、改进界面、改善游客与员工 AI、更多编辑工具、提高限制上限和合作多人游戏。它还重新引入了《过山车大亨》中存在但《过山车大亨 2》中没有的机制，例如游戏内山地工具、“尽情游玩”目标、发射式过山车（不穿过车站）及工具栏上的若干按钮。

---

# 2. 下载游戏（预构建版）

OpenRCT2 需要《过山车大亨 2》的原始文件才能游玩。可以从 [Steam](https://store.steampowered.com/app/285330/RollerCoaster_Tycoon_2_Triple_Thrill_Pack/) 或 [GOG.com](https://www.gog.com/game/rollercoaster_tycoon_2) 购买。如果拥有原版《过山车大亨》及扩展包，也可以[让 OpenRCT2 指向这些文件](https://github.com/OpenRCT2/OpenRCT2/wiki/Loading-RCT1-scenarios-and-data)，以游玩原版场景。

[我们的网站](https://openrct2.io/download)提供 `master` 和 `develop` 分支最新版本的便携版和安装程序。还提供适用于 Windows、macOS 和 Linux 的[启动器](https://openrct2.io/download/launcher)，可自动更新游戏构建，确保你始终拥有最新版本。

对于大多数 Linux 发行版，除启动器外，我们推荐使用[最新 Flatpak 版本](https://flathub.org/apps/details/io.openrct2.OpenRCT2)。从 Flathub 下载时，无论使用哪个 Linux 发行版，都能获得最新更新。

一些 Linux 发行版提供原生软件包：
* Arch Linux：[openrct2](https://archlinux.org/packages/extra/x86_64/openrct2/) 最新发布版（`extra` 仓库），或 [openrct2-git](https://aur.archlinux.org/packages/openrct2-git)（AUR）
* Gentoo（主 portage 树）：[games-simulation/openrct2](https://packages.gentoo.org/packages/games-simulation/openrct2)
* NixOS：[openrct2](https://github.com/NixOS/nixpkgs/blob/master/pkgs/by-name/op/openrct2/package.nix)
* openSUSE OBS：[games/openrct2](https://software.opensuse.org/download.html?project=games&package=openrct2)
* Ubuntu PPA（每夜构建版）：[`develop` 分支](https://launchpad.net/~openrct2/+archive/ubuntu/nightly)

一些 *BSD 操作系统提供原生软件包：
* FreeBSD：[games/openrct2](https://www.freshports.org/games/openrct2)

---

# 3. 构建游戏
- [在 Linux 上构建 OpenRCT2](https://github.com/OpenRCT2/OpenRCT2/wiki/Building-OpenRCT2-on-Linux)
- [在 macOS 上使用 CMake 构建 OpenRCT2](https://github.com/OpenRCT2/OpenRCT2/wiki/Building-OpenRCT2-on-macOS-using-CMake)
- [在 Windows 上构建 OpenRCT2](https://github.com/OpenRCT2/OpenRCT2/wiki/Building-OpenRCT2-on-Windows)
- [在 Windows Subsystem for Linux 上构建 OpenRCT2](https://github.com/OpenRCT2/OpenRCT2/wiki/Building-OpenRCT2-on-Windows-Subsystem-for-Linux)
- [在 MSYS2 MinGW 上构建 OpenRCT2](https://github.com/OpenRCT2/OpenRCT2/wiki/Building-OpenRCT2-on-MSYS2-MinGW)

---

# 4. 参与贡献

OpenRCT2 使用 [gitflow 工作流](https://www.atlassian.com/git/tutorials/comparing-workflows#gitflow-workflow)。实现新功能或修复错误时，请从 ```develop``` 分支创建分支并向其提交 pull request。```master``` 只包含带标签的发布版本，绝不应从它创建分支。

更多信息请阅读我们的[贡献指南](https://github.com/OpenRCT2/OpenRCT2/blob/5df376f311e03a90ad162334d145cc1fbc0e671b/CONTRIBUTING.md)。

## 4.1 错误修复

可在[问题跟踪器](https://github.com/OpenRCT2/OpenRCT2/issues)找到错误列表。欢迎处理任何错误，并向 develop 分支提交修复 pull request。在 issue 中说明你打算修复某个错误，可以避免其他人同时尝试。

## 4.2 新功能

开始开发新功能前，请先与 OpenRCT2 团队交流。我们可能已有相关计划，或有理由不采用你想开发的内容。提前联系可以让我们帮助你，也能避免浪费时间。可通过 Discord 与我们交流，链接见本页顶部。

## 4.3 翻译

可以通过编辑 ```data/language``` 目录中的语言文件，将游戏翻译成其他语言。请加入 [Discord 的 #localisation 频道](https://discordapp.com/invite/sxnrvX9)讨论，并向 [OpenRCT2/Localisation](https://github.com/OpenRCT2/Localisation) 提交 pull request。

## 4.4 图形

访问 [OpenGraphics 项目](https://github.com/OpenRCT2/OpenGraphics)，帮助创建新的游戏图形。我们需要 3D 建模师！

## 4.5 音频

你可以帮助为游戏创作音乐和音效。查看 [OpenMusic](https://github.com/OpenRCT2/OpenMusic) 仓库，并访问 [Discord 的 #open-sound-and-music 频道](https://discord.gg/9y8WbcX)了解更多信息。

## 4.6 场景

时机成熟时，我们也希望随游戏分发更多场景，因此需要有才华的场景制作者！请查看 [OpenScenarios 仓库](https://github.com/PFCKrutonium/OpenRCT2-OpenScenarios)。

---

# 5. 政策

## 5.1 行为准则

我们有一份适用于所有 OpenRCT2 项目的[行为准则](https://github.com/OpenRCT2/OpenRCT2/blob/5df376f311e03a90ad162334d145cc1fbc0e671b/CODE_OF_CONDUCT.md)，请阅读。

## 5.2 代码签名政策

我们使用 SignPath Foundation 提供的数字证书为发布版本签名。

免费代码签名由 [SignPath.io](https://about.signpath.io/) 提供，证书由 [SignPath Foundation](https://signpath.org/) 提供。

只有[开发团队](https://github.com/OpenRCT2/OpenRCT2/blob/5df376f311e03a90ad162334d145cc1fbc0e671b/contributors.md#development-team)成员可以发布签名版本。

## 5.3 隐私政策

更多信息请参阅 [PRIVACY.md](https://github.com/OpenRCT2/OpenRCT2/blob/5df376f311e03a90ad162334d145cc1fbc0e671b/PRIVACY.md)。

---

# 6. 许可证

**OpenRCT2** 采用 GNU 通用公共许可证第 3 版，或由你选择的任何更高版本。详情请参阅 [`licence.txt`](https://github.com/OpenRCT2/OpenRCT2/blob/5df376f311e03a90ad162334d145cc1fbc0e671b/licence.txt)。

---

# 7. 更多信息
- [GitHub](https://github.com/OpenRCT2/OpenRCT2)
- [OpenRCT2.io](https://openrct2.io)
- [Facebook](https://www.facebook.com/OpenRCT2)
- [RCT subreddit](https://www.reddit.com/r/rct/)
- [OpenRCT2 subreddit](https://www.reddit.com/r/openrct2/)
- OpenRCT2 插件
    - [插件目录（非官方）](https://openrct2plugins.org)
    - [插件开发文档](https://github.com/OpenRCT2/OpenRCT2/blob/5df376f311e03a90ad162334d145cc1fbc0e671b/distribution/scripting/scripting.md)

## 类似项目

| [OpenLoco](https://github.com/OpenLoco/OpenLoco) | [OpenTTD](https://github.com/OpenTTD/OpenTTD) | [openage](https://github.com/SFTtech/openage) | [OpenRA](https://github.com/OpenRA/OpenRA) |
|:------------------------------------------------:|:----------------------------------------------------------------------------------------------------------:|:-------------------------------------------------------------------------------------------------------:|:-------------------------------------------------------------------------------------------------------------:|
| [![icon_x128](https://user-images.githubusercontent.com/604665/53047651-2c533c00-3493-11e9-911a-1a3540fc1156.png)](https://github.com/OpenLoco/OpenLoco) | [![](https://github.com/OpenTTD/OpenTTD/raw/850d05d24d4768c81d97765204ef2a487dd4972c/media/openttd.128.png)](https://github.com/OpenTTD/OpenTTD) | [![](https://user-images.githubusercontent.com/550290/36507534-4693f354-175a-11e8-93a7-faa0481474fb.png)](https://github.com/SFTtech/openage) | [![](https://raw.githubusercontent.com/OpenRA/OpenRA/bleed/packaging/artwork/ra_128x128.png)](https://github.com/OpenRA/OpenRA) |
| Chris Sawyer 的 Locomotion | Transport Tycoon Deluxe | Age of Empires 2 | Red Alert |

# 8. 赞助商

慷慨允许我们使用其产品的公司：

| [JetBrains](https://www.jetbrains.com/)                                                                                                        | [Backtrace](https://backtrace.io/)                                                                                                        | [SignPath](https://signpath.org/)                                                                                  |
|------------------------------------------------------------------------------------------------------------------------------------------------|-------------------------------------------------------------------------------------------------------------------------------------------|--------------------------------------------------------------------------------------------------------------------|
| [![jetbrains](https://github.com/user-attachments/assets/0d1cf25e-706d-4e3a-96ee-157fbf2cf0c0)](https://www.jetbrains.com/) | [![backtrace](https://user-images.githubusercontent.com/550290/47113259-d0647680-d258-11e8-97c3-1a2c6bde6d11.png)](https://backtrace.io/) | [![Image](https://github.com/user-attachments/assets/2b5679e0-76a4-4ae7-bb37-a6a507a53466)](https://signpath.org/) |
| CLion 及其他产品                                                                                                                       | Minidump 上传和检查                                                                                                           | 免费代码签名由 [SignPath.io](https://about.signpath.io/), 证书由 [SignPath Foundation](https://signpath.org/).                                                                                                       |
