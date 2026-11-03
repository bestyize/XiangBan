> 本文根据 [Cataclysm: Dark Days Ahead 项目 README](https://github.com/CleverRaven/Cataclysm-DDA/blob/57cf429e845fec8ae0405ec0f4dca5921c9c6d79/README.md) 翻译并修改相对链接；原文由项目贡献者创作，译文采用 [CC BY-SA 3.0](https://creativecommons.org/licenses/by-sa/3.0/) 许可。

# Cataclysm: Dark Days Ahead

Cataclysm: Dark Days Ahead 是一款以后末日世界为背景的回合制生存游戏。虽然有人称它为“僵尸游戏”，但 Cataclysm 的内容远不止于此。在严酷、持续存在且程序生成的世界中挣扎求生。从已经消亡的文明遗迹中搜寻食物、装备，或是幸运的话找到一辆油箱加满的车，赶紧逃离险境。与各种强大的怪物战斗或逃脱：从僵尸、巨型昆虫到杀人机器人，还有更加奇异致命的东西，以及和你一样觊觎你所拥有物品的其他人……

<p align="center">
    <img src="https://raw.githubusercontent.com/CleverRaven/Cataclysm-DDA/57cf429e845fec8ae0405ec0f4dca5921c9c6d79/data/screenshots/ultica-showcase-sep-2021.png" alt="图块集：Ultica">
</p>

## 下载

**发布版本** - [稳定版](https://cataclysmdda.org/releases/) | [实验版](https://cataclysmdda.org/experimental/)

**源代码** - 可以下载 [.zip 压缩包](https://github.com/CleverRaven/Cataclysm-DDA/archive/master.zip)，也可以克隆我们的 [GitHub 仓库](https://github.com/CleverRaven/Cataclysm-DDA/)。

[![General build matrix](https://github.com/CleverRaven/Cataclysm-DDA/actions/workflows/matrix.yml/badge.svg)](https://github.com/CleverRaven/Cataclysm-DDA/actions/workflows/matrix.yml)
[![Coverage Status](https://coveralls.io/repos/github/CleverRaven/Cataclysm-DDA/badge.svg?branch=master)](https://coveralls.io/github/CleverRaven/Cataclysm-DDA?branch=master)
[![Open Source Helpers](https://www.codetriage.com/cleverraven/cataclysm-dda/badges/users.svg)](https://www.codetriage.com/cleverraven/cataclysm-dda)
[![Commit Activity](https://img.shields.io/github/commit-activity/m/CleverRaven/Cataclysm-DDA)](https://github.com/CleverRaven/Cataclysm-DDA/graphs/contributors)
[![TODOs](https://badgen.net/https/api.tickgit.com/badgen/github.com/CleverRaven/Cataclysm-DDA)](https://www.tickgit.com/browse?repo=github.com/CleverRaven/Cataclysm-DDA)

### 打包状态

#### Arch Linux

Ncurses 版和图块版可从[社区仓库](https://www.archlinux.org/packages/?q=cataclysm-dda)获取。

```sh
sudo pacman -S cataclysm-dda
sudo pacman -S cataclysm-dda-tiles
```

#### Fedora

Ncurses 版和图块版可从[官方仓库](https://src.fedoraproject.org/rpms/cataclysm-dda)获取。

```sh
sudo dnf install cataclysm-dda
```

#### Debian / Ubuntu

Ncurses 版和图块版可从[官方仓库](https://tracker.debian.org/pkg/cataclysm-dda)获取。

```sh
sudo apt install cataclysm-dda-curses cataclysm-dda-sdl
```

#### Flatpak

从 [Flathub](https://flathub.org/apps/org.cataclysmdda.CataclysmDDA) 下载。

## 编译

请阅读 [COMPILING.md](https://github.com/CleverRaven/Cataclysm-DDA/blob/57cf429e845fec8ae0405ec0f4dca5921c9c6d79/doc/c++/COMPILING.md)：其中涵盖一般信息，以及 Linux、OS X、Windows 和 BSD 更具体的构建方法。请参阅 [COMPILER_SUPPORT.md](https://github.com/CleverRaven/Cataclysm-DDA/blob/57cf429e845fec8ae0405ec0f4dca5921c9c6d79/doc/c++/COMPILER_SUPPORT.md)了解支持的编译器。你也可以随时在 [doc/](https://github.com/CleverRaven/Cataclysm-DDA/tree/57cf429e845fec8ae0405ec0f4dca5921c9c6d79/doc) 中查找更多信息。

我们还提供以下构建指南：
* 使用 `MSYS2` 在 Windows 上构建：[COMPILING-MSYS.md](https://github.com/CleverRaven/Cataclysm-DDA/blob/57cf429e845fec8ae0405ec0f4dca5921c9c6d79/doc/c++/COMPILING-MSYS.md)
* 使用 `vcpkg` 在 Windows 上构建：[COMPILING-VS-VCPKG.md](https://github.com/CleverRaven/Cataclysm-DDA/blob/57cf429e845fec8ae0405ec0f4dca5921c9c6d79/doc/c++/COMPILING-VS-VCPKG.md)
* 使用 `cmake` 构建：[COMPILING-CMAKE.md](https://github.com/CleverRaven/Cataclysm-DDA/blob/57cf429e845fec8ae0405ec0f4dca5921c9c6d79/doc/c++/COMPILING-CMAKE.md)（*非官方指南*）

## 参与贡献

Cataclysm: Dark Days Ahead 是超过 1000 名志愿者根据知识共享署名-相同方式共享 3.0 许可证贡献的成果。游戏代码和内容可为任何目的自由使用、修改和重新分发。详情见 https://creativecommons.org/licenses/by-sa/3.0/。
项目附带的部分代码不属于本项目，采用不同的软件许可证发布；这些文件各自带有许可声明。

详情请参阅 [CONTRIBUTING.md](https://github.com/CleverRaven/Cataclysm-DDA/blob/57cf429e845fec8ae0405ec0f4dca5921c9c6d79/CONTRIBUTING.md)。

特别感谢贡献者，包括但不限于以下人员：
<a href="https://github.com/cleverraven/cataclysm-dda/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=cleverraven/cataclysm-dda" />
</a>

Made with [contrib.rocks](https://contrib.rocks).

## 社区

论坛：
https://discourse.cataclysmdda.org

GitHub 仓库：
https://github.com/CleverRaven/Cataclysm-DDA

IRC：
[Libera Chat](https://libera.chat) 上的 `#CataclysmDDA`，https://web.libera.chat/#CataclysmDDA

官方 Discord：
https://discord.gg/jFEc7Yp

## 常见问题

#### 有教程吗？

有，你可以在主菜单的 **Special** 菜单中找到教程（请注意，由于代码发生大量变化，教程可能无法运行）。也可以在游戏中通过 `?` 键访问文档。

#### 如何修改按键绑定？

按 `?` 键，然后按 `1` 查看完整按键命令列表。按 `+` 添加按键绑定，用对应的字母键 `a-w` 选择动作，再按希望分配给该动作的键。

#### 如何开始一个新世界？

主菜单中的 **World** 会为你生成一个新世界。选择 **Create World**。

#### 我发现了错误，该怎么办？

请在[我们的 GitHub 页面](https://github.com/CleverRaven/Cataclysm-DDA/issues/)使用[错误报告模板](https://github.com/CleverRaven/Cataclysm-DDA/issues/new?template=bug_report.yaml)提交 issue。如果无法提交，请发送电子邮件至 `kevin.granade@gmail.com`。

#### 我想提出建议，该怎么办？

请在[我们的 GitHub 页面](https://github.com/CleverRaven/Cataclysm-DDA/issues/)使用[功能请求模板](https://github.com/CleverRaven/Cataclysm-DDA/issues/new?template=feature_request.yaml)提交 issue。
