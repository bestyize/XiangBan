<h1 align="center">
  <img src="https://raw.githubusercontent.com/Warzone2100/warzone2100/253e370c2af78228a822df6afa82b6261dc94656/icons/warzone2100.large.png" alt="Warzone 2100">
  <br />
  Warzone 2100
</h1>

Warzone 2100 是<b>免费开源的 3D 即时战略游戏</b>，包含剧情驱动的单人战役、在线多人游戏、离线遭遇战等。

# 亮点

- 重制的单人战役，包括不断扩展的持久基地、外出任务等
- 最多 10 人在线多人对战，支持自由混战或团队作战
- 与 AI 对手进行本地遭遇战，提供持续重玩空间
- 庞大的科技树，包含 400 多项不同技术
- 灵活的单位设计系统，可定制单位并采用多种战术
- 跨平台多人游戏
- 支持多种图形后端（OpenGL、OpenGL ES、Vulkan）
- 完全免费且开源

更多信息请见 [Warzone 2100 项目主页](https://wz2100.net/)。

# 安装

**请访问 https://wz2100.net 获取适用于 Windows、macOS 和 Linux 的最新稳定版。**

> Ubuntu 视频说明：为获取游戏中的重要信息，请手动下载视频。假设游戏安装在标准的 `~/.local/share/` 文件夹，请使用以下命令：
> ```shell
> mkdir ~/.local/share/warzone2100
> wget https://github.com/Warzone2100/wz-sequences/releases/download/v3/standard-quality-en-sequences.wz -O ~/.local/share/warzone2100/sequences.wz
> ```

# 起源

Warzone 2100 最初由 Pumpkin Studios 开发，于 1999 年作为一款开创性的创新 3D 即时战略游戏发布。

2004 年，Eidos 与 Pumpkin Studios 合作，决定以 GNU GPL 条款发布游戏源代码，随后也发布了剩余音乐和视频文件。

此后社区一直以“Warzone 2100 Project”名义开发、维护和改进该游戏。

# 游戏现状

自 2004 年 12 月 6 日 Warzone 2100 源代码开放以来，所有专有技术均已由开源替代品取代，并在保留原版优秀之处的同时进行了大量改进和新增内容。

开发在 GitHub 上继续，欢迎错误报告与贡献！

# 最新开发构建

[![Windows Build Status](https://img.shields.io/github/actions/workflow/status/Warzone2100/warzone2100/CI_windows.yml?branch=master&label=Windows&logo=windows)](https://github.com/Warzone2100/warzone2100/actions?query=workflow%3AWindows+branch%3Amaster+event%3Apush)
 [![macOS Build Status](https://img.shields.io/github/actions/workflow/status/Warzone2100/warzone2100/CI_macos.yml?branch=master&label=macOS&logo=apple)](https://github.com/Warzone2100/warzone2100/actions?query=workflow%3AmacOS+branch%3Amaster+event%3Apush)
 [![Ubuntu Build Status](https://img.shields.io/github/actions/workflow/status/Warzone2100/warzone2100/CI_ubuntu.yml?branch=master&label=Ubuntu&logo=ubuntu&logoColor=FFFFFF)](https://github.com/Warzone2100/warzone2100/actions?query=workflow%3AUbuntu+branch%3Amaster+event%3Apush)
 [![Fedora Build Status](https://img.shields.io/github/actions/workflow/status/Warzone2100/warzone2100/CI_fedora.yml?branch=master&label=Fedora&logo=fedora&logoColor=FFFFFF)](https://github.com/Warzone2100/warzone2100/actions?query=workflow%3AFedora+branch%3Amaster+event%3Apush)
 [![WebAssembly Build Status](https://img.shields.io/github/actions/workflow/status/Warzone2100/warzone2100/CI_emscripten.yml?branch=master&label=WebASM&logo=webassembly&logoColor=FFFFFF)](https://github.com/Warzone2100/warzone2100/actions?query=workflow%3AEmscripten+branch%3Amaster+event%3Apush)
 [![FreeBSD Build Status](https://img.shields.io/cirrus/github/Warzone2100/warzone2100/master?label=FreeBSD&logo=FreeBSD)](https://cirrus-ci.com/github/Warzone2100/warzone2100/master)
 [![Packaging status](https://repology.org/badge/tiny-repos/warzone2100.svg)](https://repology.org/project/warzone2100/versions)

### Windows

获取最新 Windows 开发构建的方法：
1. 查看[最新成功的 Windows 构建](https://github.com/Warzone2100/warzone2100/actions?query=workflow%3AWindows+branch%3Amaster+event%3Apush+is%3Asuccess)。
2. 在表格或列表中选择最近一次工作流运行，页面应显示该次运行的 **Artifacts** 列表。
3. 下载 `warzone2100_win_installer` 构建产物。
> 注意：目前下载构建产物需要免费 GitHub 账号。

### macOS

获取最新 macOS 开发构建的方法：
1. 查看[最新成功的 macOS 构建](https://github.com/Warzone2100/warzone2100/actions?query=workflow%3AmacOS+branch%3Amaster+event%3Apush+is%3Asuccess)。
2. 在表格或列表中选择最近一次工作流运行，页面应显示 **Artifacts** 列表。
3. 下载 `warzone2100_macOS_universal` 或 `warzone2100_macOS_universal_novideos`，取决于是否需要完整应用包。
> 注意：目前下载构建产物需要免费 GitHub 账号。

### Ubuntu

获取最新 Ubuntu 开发构建的方法：
1. 查看[最新成功的 Ubuntu 构建](https://github.com/Warzone2100/warzone2100/actions?query=workflow%3AUbuntu+branch%3Amaster+event%3Apush+is%3Asuccess)。
2. 在表格或列表中选择最近一次工作流运行，页面应显示 **Artifacts** 列表。
3. 下载适合的 `warzone2100_ubuntu<version>_amd64_deb` 构建产物。
   - Ubuntu 22.04：`warzone2100_ubuntu22.04_amd64_deb`
   - Ubuntu 24.04：`warzone2100_ubuntu24.04_amd64_deb`
> 注意：目前下载构建产物需要免费 GitHub 账号。
4. 将下载的 .zip 内容（`warzone2100_ubuntu<version>_amd64.deb`）解压到桌面。
5. 在终端执行以下命令：
```shell
cd ~/Desktop
sudo apt install ./warzone2100_ubuntu<version>_amd64.deb
```
6. 下载游戏中重要信息所需的视频，详情见“视频”部分。假设游戏安装在标准 `~/.local/share/` 文件夹，请使用以下命令（更新 `warzone2100-<version>`）：
```shell
wget https://github.com/Warzone2100/wz-sequences/releases/download/v3/standard-quality-en-sequences.wz -O ~/.local/share/warzone2100-<version>/sequences.wz
```

### Linux（从源代码构建）

克隆本 Git 仓库，并按[如何构建](#如何构建)中的说明构建。

> 开发构建是当前开发状态、最新成功构建提交的快照。欢迎帮助测试，但应将其视为尚在进行中的工作。

### 视频

可以从[这里](https://github.com/Warzone2100/wz-sequences/releases/tag/v3)或[这里](https://sourceforge.net/projects/warzone2100/files/warzone2100/Videos/)下载视频。需要将下载文件重命名为 `sequences.wz`，并如上所述放入 Warzone 2100 目录。`.wz` 文件实际上是更名后的 `.zip`，如想查看内容可重命名并解压。

# 报告错误

游戏仍有错误。如果遇到问题，请通过 GitHub 错误跟踪器（https://github.com/Warzone2100/warzone2100/issues）报告。为了更快修复错误，请遵循以下规则：

1. 游戏崩溃时可以保存内存转储，请在报告错误时上传。（Linux 文件位于 /tmp/warzone2100.gdmp，Windows 位于 /Program Files/Warzone 2100/warzone2100.RPT，macOS 可在崩溃错误消息中点击“Details”。）自行生成的回溯也同样有用。
2. 尽可能详细说明崩溃或错误发生前的操作。
3. 尝试复现错误，并在报告中描述过程。
4. 也可以上传存档。存档由一个文件和一个文件夹组成，两者均以存档名称命名（例如 MySaveGame.gam 和 MySaveGame 文件夹）。
5. 错误报告并非提交后就不用管了：你可能遗漏信息或文件，提交后请继续关注报告，也可以启用评论邮件通知。

# 配置

Warzone 2100 使用用户主目录下的专用子目录保存配置、存档和某些其他内容。你也可以将自定义地图和模组放入该目录，以便游戏找到。目录位置因操作系统而异。

> [!TIP]
> 查找配置目录的简便方法：
> 1. 启动 Warzone 2100
> 2. 点击“Options”
> 3. 点击左下角的小型“Open Configuration Directory”链接

### 配置文件

配置文件名为 `config`，包含多个选项。部分选项可通过命令行或游戏内菜单修改，其他选项只能手动编辑文件。

如果操作有误，可以删除旧配置文件并重启 Warzone 2100，游戏会用默认值重新生成配置文件。

# 命令行选项

Warzone 2100 可以带不同选项和参数启动。要查看选项列表，请使用 --help 启动游戏。

注意：这些选项都使用两个短横线（--），而不是一个（-）。如果选项带参数，必须使用 `=` 分隔选项和参数，空格不起作用。

注意：一些选项在配置文件中有对应条目，设置会在后续启动之间保留。

# 通过互联网进行多人游戏

有两种方式通过互联网开始多人游戏：使用主机 IP 或使用大厅服务器。请确保能够通过 TCP 端口 2100 和 9999 通信。端口转发只需在路由器上转发端口 2100。

可以选择通过 Lobby 或 IP 连接：

* 选择 IP 时，Warzone 2100 会询问主机 IP 地址并尝试连接。
* 选择 Lobby 时，只要配置文件中的 lobby-server-address 未被更改，Warzone 2100 就会连接到大厅服务器。

之后会看到可供选择的游戏列表。

游戏开始前，可以按住鼠标右键并左键点击不需要的玩家，将其移出游戏。

当你主持游戏时，它会自动出现在大厅服务器列表中。如果不希望列出游戏，应将配置中的 `masterserver_name=lobby.wz2100.net` 改为无效值，例如 `nomasterserverplease`。

之后若想查看大厅服务器上的游戏，可以在提示输入主机 IP 时输入 `lobby.wz2100.net`，或将配置恢复。必须重启 Warzone 2100 才能使配置更改生效。

# 作弊码

Warzone 2100 内置许多作弊码：
* [Warzone 2100 作弊码](https://github.com/Warzone2100/warzone2100/blob/253e370c2af78228a822df6afa82b6261dc94656/doc/Cheats.md)

# 模组制作信息

Warzone 2100 的 AI、地图和战役可以使用 JavaScript 编写脚本。

更多信息：
* [脚本](https://github.com/Warzone2100/warzone2100/blob/253e370c2af78228a822df6afa82b6261dc94656/doc/Scripting.md)
* [模型格式](https://github.com/Warzone2100/warzone2100/blob/253e370c2af78228a822df6afa82b6261dc94656/doc/PIE.md)
* [动画](https://github.com/Warzone2100/warzone2100/blob/253e370c2af78228a822df6afa82b6261dc94656/doc/Animation.md)

# 如何构建

### 获取源代码

要正确构建游戏，请使用以下方式之一：
- 下载包含所有源代码和修订信息的发布版 `tar.xz`。

  _或者_

- 克隆 Git 仓库：
  ```shell
  git clone https://github.com/Warzone2100/warzone2100.git
  cd warzone2100
  git fetch --tags
  git submodule update --init --recursive
  ```
  > 注意：必须初始化子模块。

**不要**使用 GitHub 的“Download Zip”选项，因为它**不包含子模块**或基于 Git 的自动修订信息。

### Linux

* 前提条件
   * 编译工具（例如 CMake、GCC/G++/Clang、ninja-build）
   * 归档工具（例如 zip、p7zip）
   * 各种库：
      * [SDL](https://www.libsdl.org) ≥ 3.2.12 _(强烈推荐： ≥ 3.2.26)_
      * [PhysicsFS](https://icculus.org/physfs/) ≥ 2.1 _(强烈推荐： ≥ 3.0.2)_
      * [libpng](https://www.libpng.org/pub/png/libpng.html) ≥ 1.2
      * [libjpeg-turbo](https://github.com/libjpeg-turbo/libjpeg-turbo) ≥ 2.1
      * [libtheora](https://theora.org)
      * [libvorbis](https://xiph.org/vorbis)
      * [libogg](https://xiph.org/ogg/)
      * [libvpx](https://www.webmproject.org/code/)
      * [opus](https://github.com/xiph/opus)
      * [Freetype](https://www.freetype.org/) _(强烈推荐： ≥ 2.10.4)_
      * [Harfbuzz](https://github.com/harfbuzz/harfbuzz) ≥ 1.0 _(强烈推荐： ≥ 3.3.0)_
      * [fribidi](https://github.com/fribidi/fribidi)
      * [OpenAL-Soft](https://openal-soft.org)
      * [libcurl](https://curl.haxx.se/libcurl/) _(强烈推荐： ≥ 7.58.0)_
      * [libsodium](https://github.com/jedisct1/libsodium) ≥ 1.0.16
      * [SQLite](https://www.sqlite.org/index.html) ≥ 3.14
      * [libzip](https://github.com/nih-at/libzip) _(强烈推荐： ≥ 1.10.1)_
      * [libprotobuf](https://github.com/protocolbuffers/protobuf) （如编译时启用 GNS 支持）
   * 语言支持：[Gettext](https://www.gnu.org/software/gettext/)
   * 生成文档：[Asciidoctor](https://asciidoctor.org) ≥ 1.5.3
   * 构建 Vulkan 支持：完整的 [Vulkan SDK](https://vulkan.lunarg.com/sdk/home)（_强烈推荐：≥ 1.2.148.1_）
* **安装前提条件：**
   * 使用 `get-dependencies_linux.sh`：
      1. 指定脚本支持的一种 Linux 发行版：(`ubuntu`, `fedora`, `alpine`, `archlinux`, `opensuse-tumbleweed`, `gentoo`, `debian`, `raspberrypios`)（_必需_）
      2. 指定模式：(`build-all`（默认）, `build-dependencies`)（_可选_）

      例如：
      ```shell
      sudo ./get-dependencies_linux.sh ubuntu build-dependencies
      ```
* **从命令行构建：**
   1. 从 warzone2100 仓库的_父目录_开始，创建一个[源代码目录之外的](https://cmake.org/cmake/help/book/mastering-cmake/chapter/Getting%20Started.html#directory-structure)构建目录：
      ```shell
      mkdir build
      ```
   2. 切换到 `build` 目录：
      ```shell
      cd build
      ```
   3. 运行 CMake 配置以生成构建文件：
      ```shell
      cmake -DCMAKE_BUILD_TYPE=RelWithDebInfo -DCMAKE_INSTALL_PREFIX:PATH=~/wz/install -GNinja ../warzone2100
      ```
      > - [按需修改 `CMAKE_INSTALL_PREFIX` 参数](https://cmake.org/cmake/help/latest/variable/CMAKE_INSTALL_PREFIX.html)，以配置基本安装路径。
      > - 末尾的 `../warzone2100` 应指向 warzone2100 仓库目录。此示例假定仓库和构建目录是同级目录，且仓库克隆到名为 `warzone2100` 的目录。
   4. 运行 CMake 构建：
      ```shell
      cmake --build . --target install
      ```

### 使用 MSVC 的 Windows 构建

参阅 [platforms/windows/README-build.md](https://github.com/Warzone2100/warzone2100/blob/253e370c2af78228a822df6afa82b6261dc94656/platforms/windows/README-build.md)。

### macOS
参阅 [platforms/macos/README.md](https://github.com/Warzone2100/warzone2100/blob/253e370c2af78228a822df6afa82b6261dc94656/platforms/macos/README.md)。

# 许可

Warzone 2100 是自由软件；你可以根据自由软件基金会发布的 GNU 通用公共许可证第 2 版或由你选择的任何更高版本，重新分发和/或修改它。

[![SPDX-License-Identifier: GPL-2.0-or-later](https://img.shields.io/static/v1?label=SPDX-License-Identifier&message=GPL-2.0-or-later&color=blue&logo=open-source-initiative&logoColor=white&logoWidth=10&style=flat-square)](https://github.com/Warzone2100/warzone2100/blob/253e370c2af78228a822df6afa82b6261dc94656/COPYING)

更多信息：[COPYING.README](https://github.com/Warzone2100/warzone2100/blob/253e370c2af78228a822df6afa82b6261dc94656/COPYING.README)、[COPYING.NONGPL](https://github.com/Warzone2100/warzone2100/blob/253e370c2af78228a822df6afa82b6261dc94656/COPYING.NONGPL)。

# 特别感谢

Windows 免费代码签名由 [SignPath.io](https://signpath.io) 提供，证书由 [SignPath Foundation](https://signpath.org/) 提供。
