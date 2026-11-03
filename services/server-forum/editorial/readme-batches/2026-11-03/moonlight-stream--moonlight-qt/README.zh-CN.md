# Moonlight PC

[Moonlight PC](https://moonlight-stream.org) 是 NVIDIA GameStream 和 [Sunshine](https://github.com/LizardByte/Sunshine) 的开源 PC 客户端。

Moonlight 也有适用于 [Android](https://github.com/moonlight-stream/moonlight-android) 和 [iOS](https://github.com/moonlight-stream/moonlight-ios) 的移动版本。

你可以在我们的 [Discord 服务器](https://moonlight-stream.org/discord)关注开发进展，并通过 [Weblate](https://hosted.weblate.org/projects/moonlight/moonlight-qt/)帮助将 Moonlight 翻译成你的语言。

 [![Build](https://img.shields.io/github/actions/workflow/status/moonlight-stream/moonlight-qt/build.yml?branch=master)](https://github.com/moonlight-stream/moonlight-qt/actions/workflows/build.yml?query=branch%3Amaster)
 [![Downloads](https://img.shields.io/github/downloads/moonlight-stream/moonlight-qt/total)](https://github.com/moonlight-stream/moonlight-qt/releases)
 [![Translation Status](https://hosted.weblate.org/widgets/moonlight/-/moonlight-qt/svg-badge.svg)](https://hosted.weblate.org/projects/moonlight/moonlight-qt/)

## 功能
 - Windows、Mac 和 Linux 上的硬件加速视频解码
 - 支持 H.264、HEVC 和 AV1 编解码器（AV1 需要 Sunshine 和受支持的主机 GPU）
 - 支持 YUV 4:4:4（仅限 Sunshine）
 - 支持 HDR 串流
 - 支持 7.1 环绕声音频
 - 支持 10 点多点触控（仅限 Sunshine）
 - 支持最多 16 名玩家使用带力反馈和体感控制的游戏手柄
 - 同时支持指针捕获（用于游戏）和直接鼠标控制（用于远程桌面）
 - 支持将 Alt+Tab 等全局键盘快捷键传递给主机

## 下载
- [Windows、macOS 和 Steam Link](https://github.com/moonlight-stream/moonlight-qt/releases)
- [Snap（适用于基于 Ubuntu 的 Linux 发行版）](https://snapcraft.io/moonlight)
- [Flatpak（适用于其他 Linux 发行版）](https://flathub.org/apps/details/com.moonlight_stream.Moonlight)
- [AppImage](https://github.com/moonlight-stream/moonlight-qt/releases)
- [Raspberry Pi 4 和 5](https://github.com/moonlight-stream/moonlight-docs/wiki/Installing-Moonlight-Qt-on-Raspberry-Pi-4)
- [通用 ARM 32 位和 64 位 Debian 软件包](https://github.com/moonlight-stream/moonlight-docs/wiki/Installing-Moonlight-Qt-on-ARM%E2%80%90based-Single-Board-Computers)（不适用于 Raspberry Pi）
- [实验性 RISC-V Debian 软件包](https://github.com/moonlight-stream/moonlight-docs/wiki/Installing-Moonlight-Qt-on-RISC%E2%80%90V-Single-Board-Computers)
- [NVIDIA Jetson 和 Nintendo Switch（Ubuntu L4T）](https://github.com/moonlight-stream/moonlight-docs/wiki/Installing-Moonlight-Qt-on-Linux4Tegra-(L4T)-Ubuntu)

### 每夜构建版
- [下载](https://nightly.link/moonlight-stream/moonlight-qt/workflows/build/master)

#### 特别感谢

[![Hosted By: Cloudsmith](https://img.shields.io/badge/OSS%20hosting%20by-cloudsmith-blue?logo=cloudsmith&style=flat-square)](https://cloudsmith.com)

Moonlight 的 Debian 和 L4T 软件包仓库由 [Cloudsmith](https://cloudsmith.com) 慷慨地免费托管。

## 构建

### Windows 构建要求
* Qt 6.11 SDK 或更高版本（较早版本可能可用，但官方不提供支持）
* [Visual Studio 2026](https://visualstudio.microsoft.com/downloads/)（社区版即可）
* 安装 Qt 时选择 **MSVC** 选项。不支持 MinGW。
* [7-Zip](https://www.7-zip.org/)（仅在为非开发 PC 构建安装程序时需要）
* Graphics Tools（仅在运行调试构建时需要）
  * 在 Windows 设置应用的“可选功能”页面安装“Graphics Tools”。
  * 或者运行 `dism /online /add-capability /capabilityname:Tools.Graphics.DirectX~~~~0.0.1.0` 并重启。

### macOS 构建要求
* Qt 6.11 SDK 或更高版本（较早版本可能可用，但官方不提供支持）
* Xcode 15 或更高版本（较早版本可能可用，但官方不提供支持）
* [create-dmg](https://github.com/sindresorhus/create-dmg)（仅在为非开发 Mac 构建 DMG 时需要）

### Linux/Unix 构建要求
* 推荐 Qt 6，但也支持 Qt 5.12 或更高版本（使用 Qt 5 时将 `qmake6` 替换为 `qmake`）。
* GCC 或 Clang
* FFmpeg 4.0 或更高版本
* 安装所需软件包：
  * Debian/Ubuntu：
    * 基本依赖： `libegl1-mesa-dev libgl1-mesa-dev libopus-dev libsdl2-dev libsdl2-ttf-dev libssl-dev libavcodec-dev libavformat-dev libswscale-dev libva-dev libvdpau-dev libxkbcommon-dev wayland-protocols libdrm-dev`
    * Qt 6（推荐）： `qt6-base-dev qt6-declarative-dev libqt6svg6-dev qt6-wayland qml6-module-qtquick-controls qml6-module-qtquick-templates qml6-module-qtquick-layouts qml6-module-qtqml-workerscript qml6-module-qtquick-window qml6-module-qtquick`
    * Qt 5： `qtbase5-dev qt5-qmake qtdeclarative5-dev qtquickcontrols2-5-dev qml-module-qtquick-controls2 qml-module-qtquick-layouts qml-module-qtquick-window2 qml-module-qtquick2 qtwayland5`
  * RedHat/Fedora（需要 RPM Fusion 仓库）：
    * 基本依赖： `openssl-devel SDL2-devel SDL2_ttf-devel ffmpeg-devel libva-devel libvdpau-devel opus-devel pulseaudio-libs-devel alsa-lib-devel libdrm-devel`
    * Qt 6（推荐）： `qt6-qtsvg-devel qt6-qtdeclarative-devel`
    * Qt 5： `qt5-qtsvg-devel qt5-qtquickcontrols2-devel`
* 构建 Vulkan 渲染器需要至少 v7.349.0 的 `libplacebo-dev`/`libplacebo-devel`，以及 FFmpeg 6.1 或更高版本。

### Steam Link 构建要求
* 已在构建系统上克隆 [Steam Link SDK](https://github.com/ValveSoftware/steamlink-sdk)
* 将 STEAMLINK_SDK_PATH 环境变量设置为 Steam Link SDK 路径

**Steam Link 硬件限制**  
面向 Steam Link 的 Moonlight 构建受该设备的硬件限制：
* 最大分辨率：**1080p（1920x1080）**
* 最大帧率：**60 FPS**
* 最大视频码率：**40 Mbps**
* 原版硬件**不支持 HDR 串流**

### Docker 容器

如果想使用 Docker 进行构建，请查看[此仓库](https://github.com/cgutman/moonlight-packaging)。其中包含适用于不同架构的标准容器，会处理依赖构建和额外链接。

### 构建设置步骤
1. 从 https://www.qt.io/download 安装最新版 Qt SDK（也可选择安装 Qt Creator IDE）。
    * 在 macOS 上可以通过 Homebrew 安装 Qt，但必须使用 `brew install qt --with-debug` 才能创建 Moonlight 的调试构建。
    * 也可以使用 Linux 发行版的软件包管理器安装 Qt SDK，只要软件包版本为 Qt 5.12 或更高即可。
    * 为 Steam Link 构建时无需此步骤，因为 Steam Link SDK 已包含 Qt 5.14。
2. 下载子模块和依赖项。
    * 在 `moonlight-qt/` 内运行 `git submodule update --init --recursive`。
    * 在 Windows 和 macOS 上，还必须分别运行 `setup-deps.ps1`（Windows）或 `setup-deps.py`（macOS）。
    * 每次从 Git 仓库拉取新更改后都要执行这些步骤。
3. 在 Qt Creator 中打开项目，或从命令行使用 qmake 构建。
    * 为非开发机器构建二进制文件，请使用 `scripts` 文件夹中的脚本。
        * Windows 构建使用 `scripts\build-arch.bat` 和 `scripts\generate-bundle.bat`。在 Qt 命令提示符中从仓库根目录执行这些脚本，确保 7-Zip 二进制目录位于 `%PATH%` 中。
        * macOS 构建使用 `scripts/generate-dmg.sh`。从仓库根目录执行脚本，并确保 Qt 的 `bin` 文件夹位于 `$PATH` 中。
        * Steam Link 构建从仓库根目录运行 `scripts/build-steamlink-app.sh`。
    * 在 macOS 或 Linux 上为开发使用而从命令行构建时，运行 `qmake6 moonlight-qt.pro`，然后运行 `make debug` 或 `make release`。
        * 最终二进制文件位于 `app/moonlight`。
    * 为单一用途设备创建嵌入式构建时，使用 `qmake6 "CONFIG+=embedded" moonlight-qt.pro` 并正常构建。
        * 此构建不包含窗口模式、Discord/帮助链接及其他不适用于嵌入式设备的功能。
        * 对于 GPU 性能较差的平台，添加 `"CONFIG+=gpuslow"`，优先使用直接 KMSDRM 渲染而非 GL/Vulkan 渲染器。直接 KMSDRM 渲染可使用专门的 YUV/RGB 转换和缩放硬件，而非使用速度较慢的 GPU 着色器。

## 参与贡献
1. Fork 我们的仓库
2. 编写代码
3. 提交 Pull Request

请查看我们的[网站](https://moonlight-stream.org)以获取项目链接和信息。
