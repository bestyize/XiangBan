# OpenTTD

## 目录

- 1.0) [关于](#10-关于)
    - 1.1) [下载 OpenTTD](#11-下载-openttd)
    - 1.2) [OpenTTD 游戏手册](#12-openttd-游戏手册)
    - 1.3) [支持的平台](#13-支持的平台)
    - 1.4) [安装和运行 OpenTTD](#14-安装和运行-openttd)
    - 1.5) [附加内容 / 模组](#15-附加内容--模组)
    - 1.6) [OpenTTD 目录](#16-openttd-目录)
    - 1.7) [编译 OpenTTD](#17-编译-openttd)
- 2.0) [联系与社区](#20-联系与社区)
    - 2.1) [多人游戏](#21-多人游戏)
    - 2.2) [为 OpenTTD 作贡献](#22-为-openttd-作贡献)
    - 2.3) [报告错误](#23-报告错误)
    - 2.4) [翻译](#24-翻译)
- 3.0) [许可](#30-许可)
- 4.0) [致谢](#40-致谢)

## 1.0) 关于

OpenTTD 是一款运输模拟游戏，基于 Chris Sawyer 编写的热门游戏 Transport Tycoon Deluxe。它尽可能模仿原版，同时通过新功能加以扩展。

OpenTTD 采用 GNU 通用公共许可证 2.0 版，但包含一些采用其他许可的第三方软件。详情见下方[许可](#30-许可)部分。

## 1.1) 下载 OpenTTD

可从 [OpenTTD 官方网站](https://www.openttd.org/)下载。

“稳定版”和“每夜构建版”均可下载：

- 大多数人应选择“稳定版”，因为它经过更充分的测试。
- “每夜构建版”包含最新更改和功能，但有时可能不太可靠。

OpenTTD 也可在 [Steam](https://store.steampowered.com/app/1536610/OpenTTD/)、[GOG.com](https://www.gog.com/game/openttd) 和 [Microsoft Store](https://www.microsoft.com/p/openttd-official/9ncjg5rvrr1c) 免费获取。在某些平台上，也可通过操作系统包管理器或类似服务获取。

## 1.2) OpenTTD 游戏手册

OpenTTD 有一个[社区维护的 Wiki](https://wiki.openttd.org/)，包含游戏手册和提示。

## 1.3) 支持的平台

OpenTTD 已移植到多个平台和操作系统。

目前支持的平台：

- Linux（SDL，支持 OpenGL 和非 OpenGL）
- macOS（通用版本）（Cocoa）
- Windows（Win32 GDI / OpenGL）

其他平台（尤其是各种 BSD 系统）也可能运行，但我们不主动测试或维护它们。

### 1.3.1) 旧平台支持

平台、语言和编译器都会变化。只要有人愿意维护，我们就会继续支持旧平台；但如果这会使项目无法跟上语言和编译器功能的发展，则不在此限。

我们保证 OpenTTD 的每个修订版本都能加载所有较旧修订版本的存档（损坏的存档除外）。如果发现无法加载的存档，请报告错误。

## 1.4) 安装和运行 OpenTTD

OpenTTD 通常易于安装；如需更多帮助，Wiki 中有[安装指南](https://wiki.openttd.org/en/Manual/Installation)。

OpenTTD 需要一些额外的图像和声音文件才能运行。在某些平台上，如有需要，这些文件会在安装过程中下载；在另一些平台上，需要参阅[安装指南](https://wiki.openttd.org/en/Manual/Installation)。

### 1.4.1) 免费图像和声音文件

免费数据文件分为图像 OpenGFX、音效 OpenSFX 和音乐 OpenMSX，可从以下位置获取：

- [OpenGFX](https://www.openttd.org/downloads/opengfx-releases/latest)
- [OpenSFX](https://www.openttd.org/downloads/opensfx-releases/latest)
- [OpenMSX](https://www.openttd.org/downloads/openmsx-releases/latest)

请遵循这些软件包自带 README 中的安装步骤。Windows 安装程序可选择下载并安装这些软件包。

### 1.4.2) 原版 Transport Tycoon Deluxe 图像和声音文件

如果想使用原版 Transport Tycoon Deluxe 数据文件游玩，需要将 CD-ROM 中的数据文件复制到 baseset/ 目录。不论文件来自 DOS 版还是 Windows 版 Transport Tycoon Deluxe 均可。Windows 安装程序可以选择复制这些文件。

需要复制以下文件：
- sample.cat
- trg1r.grf 或 TRG1.GRF
- trgcr.grf 或 TRGC.GRF
- trghr.grf 或 TRGH.GRF
- trgir.grf 或 TRGI.GRF
- trgtr.grf 或 TRGT.GRF

### 1.4.3) 原版 Transport Tycoon Deluxe 音乐

如果想使用 Transport Tycoon Deluxe 音乐，请将原版游戏中对应的文件复制到 baseset 文件夹。
- Windows 版 TTD：gm/ 文件夹内所有文件（gm_tt00.gm 至 gm_tt21.gm）
- DOS 版 TTD：GM.CAT 文件
- 原版 Transport Tycoon：GM.CAT 文件，但要重命名为 GM-TTO.CAT

## 1.5) 附加内容 / 模组

OpenTTD 提供多种附加内容，可通过不同方式改变玩法。

大多数附加内容可通过 OpenTTD 主菜单的“Check Online Content”按钮下载。

也可以手动安装附加内容，但过程更复杂；[OpenTTD Wiki](https://wiki.openttd.org/) 或 [OpenTTD 目录结构指南](https://github.com/OpenTTD/OpenTTD/blob/e87721aac6f405af3301fa3c1b825f99a4eec0b9/docs/directory_structure.md)可能提供帮助。

### 1.5.1) 社交集成

OpenTTD 可以加载插件，与 Steam、Discord 等社交平台集成。

要启用此类集成，必须下载对应平台的插件并将其放在 `social_integration` 文件夹。

可在 [OpenTTD 网站](https://www.openttd.org)的 Downloads 下查看可用插件。

### 1.6) OpenTTD 目录

OpenTTD 使用自己的目录结构存储游戏数据、附加内容等。

更多信息请参阅[目录结构指南](https://github.com/OpenTTD/OpenTTD/blob/e87721aac6f405af3301fa3c1b825f99a4eec0b9/docs/directory_structure.md)。

### 1.7) 编译 OpenTTD

如要从源代码编译 OpenTTD，请参阅 [COMPILING.md](https://github.com/OpenTTD/OpenTTD/blob/e87721aac6f405af3301fa3c1b825f99a4eec0b9/COMPILING.md)。

## 2.0) 联系与社区

“官方”渠道：

- [OpenTTD 网站](https://www.openttd.org)
- [OpenTTD 官方 Discord](https://discord.gg/openttd)
- 在 irc.oftc.net 上使用 #openttd 进行 IRC 聊天（[有关 IRC 频道的更多信息](https://wiki.openttd.org/en/Development/IRC%20channel)）
- [GitHub 上的 OpenTTD](https://github.com/OpenTTD/)：代码仓库及问题报告
- [forum.openttd.org](https://forum.openttd.org/)：讨论 OpenTTD 和相关游戏的主要社区论坛
- [OpenTTD Wiki](https://wiki.openttd.org/)：社区维护的 Wiki，包括游戏指南、游戏机制详解、附加内容（模组）使用方法等

“非官方”渠道：

- OpenTTD Wiki 的[社区列表页面](https://wiki.openttd.org/en/Community/Community)列出了 OpenTTD 社区，其中包括一些非英语社区。

### 2.1) 多人游戏

可以与其他人合作或竞争游玩 OpenTTD。

详情请参阅[多人游戏文档](https://github.com/OpenTTD/OpenTTD/blob/e87721aac6f405af3301fa3c1b825f99a4eec0b9/docs/multiplayer.md)。

### 2.2) 为 OpenTTD 作贡献

我们欢迎贡献者。更多信息见 [CONTRIBUTING.md](https://github.com/OpenTTD/OpenTTD/blob/e87721aac6f405af3301fa3c1b825f99a4eec0b9/CONTRIBUTING.md)。

### 2.3) 报告错误

良好的错误报告非常有帮助。我们提供了[错误报告指南](https://github.com/OpenTTD/OpenTTD/blob/e87721aac6f405af3301fa3c1b825f99a4eec0b9/CONTRIBUTING.md#bug-reports)。

多人游戏不同步问题的调试和报告比较复杂（需要一定的软件开发技能）。说明见[调试和报告不同步问题](https://github.com/OpenTTD/OpenTTD/blob/e87721aac6f405af3301fa3c1b825f99a4eec0b9/docs/debugging_desyncs.md)。

### 2.4) 翻译

OpenTTD 已被翻译成多种语言。翻译通过[在线翻译工具](https://translator.openttd.org)添加和更新。

## 3.0) 许可

OpenTTD 采用 GNU 通用公共许可证 2.0 版。完整许可文本见文件 [COPYING.md](https://github.com/OpenTTD/OpenTTD/blob/e87721aac6f405af3301fa3c1b825f99a4eec0b9/COPYING.md)。除下述例外外，该许可证适用于本发行包中的所有文件。

`src/3rdparty/squirrel` 中的 squirrel 实现采用 Zlib 许可。完整许可文本见 `src/3rdparty/squirrel/COPYRIGHT`。

`src/3rdparty/md5` 中的 md5 实现采用 Zlib 许可。完整许可文本见 `src/3rdparty/md5` 中源文件的注释。

`src/3rdparty/fmt` 中的 fmt 实现采用 MIT 许可。完整许可文本见 `src/3rdparty/fmt/LICENSE.rst`。

`src/3rdparty/nlohmann` 中的 nlohmann json 实现采用 MIT 许可。完整许可文本见 `src/3rdparty/nlohmann/LICENSE.MIT`。

`src/3rdparty/opengl` 中的 OpenGL API 采用 MIT 许可。完整许可文本见 `src/3rdparty/opengl/khrplatform.h`。

`src/3rdparty/catch2` 中的 catch2 实现采用 Boost Software License 1.0 版。完整许可文本见 `src/3rdparty/catch2/LICENSE.txt`。

`src/3rdparty/icu` 中的 icu scriptrun 实现采用 Unicode 许可。完整许可文本见 `src/3rdparty/icu/LICENSE`。

`src/3rdparty/monocypher` 中的 monocypher 实现采用 BSD 2-Clause 和 CC-0 许可。完整许可文本见 `src/3rdparty/monocypher/LICENSE.md`。

`src/3rdparty/openttd_social_integration_api` 中的 OpenTTD Social Integration API 采用 MIT 许可。完整许可文本见 `src/3rdparty/openttd_social_integration_api/LICENSE`。

`cmake/3rdparty/llvm/CheckAtomic.cmake` 中的原子数据类型支持检测采用 Apache 2.0 许可。完整许可文本见 `cmake/3rdparty/llvm/LICENSE.txt`。

## 4.0) 致谢

参见 [CREDITS.md](https://github.com/OpenTTD/OpenTTD/blob/e87721aac6f405af3301fa3c1b825f99a4eec0b9/CREDITS.md)。
