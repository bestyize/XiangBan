<p align="center">
  <img src="https://raw.githubusercontent.com/rustdesk/rustdesk/beeeef14f0a61bd622aacf042b4396bdeaed9b61/res/logo-header.svg" alt="RustDesk - Your remote desktop"><br>
  <a href="#基本构建步骤">编译</a> •
  <a href="#使用-docker-构建">Docker</a> •
  <a href="#文件结构">结构</a> •
  <a href="#截图">截图</a><br>
  [<a href="https://github.com/rustdesk/rustdesk/blob/beeeef14f0a61bd622aacf042b4396bdeaed9b61/docs/README-UA.md">Українська</a>] | [<a href="https://github.com/rustdesk/rustdesk/blob/beeeef14f0a61bd622aacf042b4396bdeaed9b61/docs/README-CS.md">česky</a>] | [<a href="https://github.com/rustdesk/rustdesk/blob/beeeef14f0a61bd622aacf042b4396bdeaed9b61/docs/README-ZH.md">中文</a>] | [<a href="https://github.com/rustdesk/rustdesk/blob/beeeef14f0a61bd622aacf042b4396bdeaed9b61/docs/README-HU.md">Magyar</a>] | [<a href="https://github.com/rustdesk/rustdesk/blob/beeeef14f0a61bd622aacf042b4396bdeaed9b61/docs/README-ES.md">Español</a>] | [<a href="https://github.com/rustdesk/rustdesk/blob/beeeef14f0a61bd622aacf042b4396bdeaed9b61/docs/README-FA.md">فارسی</a>] | [<a href="https://github.com/rustdesk/rustdesk/blob/beeeef14f0a61bd622aacf042b4396bdeaed9b61/docs/README-FR.md">Français</a>] | [<a href="https://github.com/rustdesk/rustdesk/blob/beeeef14f0a61bd622aacf042b4396bdeaed9b61/docs/README-DE.md">Deutsch</a>] | [<a href="https://github.com/rustdesk/rustdesk/blob/beeeef14f0a61bd622aacf042b4396bdeaed9b61/docs/README-PL.md">Polski</a>] | [<a href="https://github.com/rustdesk/rustdesk/blob/beeeef14f0a61bd622aacf042b4396bdeaed9b61/docs/README-ID.md">Indonesian</a>] | [<a href="https://github.com/rustdesk/rustdesk/blob/beeeef14f0a61bd622aacf042b4396bdeaed9b61/docs/README-FI.md">Suomi</a>] | [<a href="https://github.com/rustdesk/rustdesk/blob/beeeef14f0a61bd622aacf042b4396bdeaed9b61/docs/README-ML.md">മലയാളം</a>] | [<a href="https://github.com/rustdesk/rustdesk/blob/beeeef14f0a61bd622aacf042b4396bdeaed9b61/docs/README-JP.md">日本語</a>] | [<a href="https://github.com/rustdesk/rustdesk/blob/beeeef14f0a61bd622aacf042b4396bdeaed9b61/docs/README-NL.md">Nederlands</a>] | [<a href="https://github.com/rustdesk/rustdesk/blob/beeeef14f0a61bd622aacf042b4396bdeaed9b61/docs/README-IT.md">Italiano</a>] | [<a href="https://github.com/rustdesk/rustdesk/blob/beeeef14f0a61bd622aacf042b4396bdeaed9b61/docs/README-RU.md">Русский</a>] | [<a href="https://github.com/rustdesk/rustdesk/blob/beeeef14f0a61bd622aacf042b4396bdeaed9b61/docs/README-PTPT.md">Português (Portugal)</a>] | [<a href="https://github.com/rustdesk/rustdesk/blob/beeeef14f0a61bd622aacf042b4396bdeaed9b61/docs/README-PTBR.md">Português (Brasil)</a>] | [<a href="https://github.com/rustdesk/rustdesk/blob/beeeef14f0a61bd622aacf042b4396bdeaed9b61/docs/README-EO.md">Esperanto</a>] | [<a href="https://github.com/rustdesk/rustdesk/blob/beeeef14f0a61bd622aacf042b4396bdeaed9b61/docs/README-KR.md">한국어</a>] | [<a href="https://github.com/rustdesk/rustdesk/blob/beeeef14f0a61bd622aacf042b4396bdeaed9b61/docs/README-AR.md">العربي</a>] | [<a href="https://github.com/rustdesk/rustdesk/blob/beeeef14f0a61bd622aacf042b4396bdeaed9b61/docs/README-VN.md">Tiếng Việt</a>] | [<a href="https://github.com/rustdesk/rustdesk/blob/beeeef14f0a61bd622aacf042b4396bdeaed9b61/docs/README-DA.md">Dansk</a>] | [<a href="https://github.com/rustdesk/rustdesk/blob/beeeef14f0a61bd622aacf042b4396bdeaed9b61/docs/README-GR.md">Ελληνικά</a>] | [<a href="https://github.com/rustdesk/rustdesk/blob/beeeef14f0a61bd622aacf042b4396bdeaed9b61/docs/README-TR.md">Türkçe</a>] | [<a href="https://github.com/rustdesk/rustdesk/blob/beeeef14f0a61bd622aacf042b4396bdeaed9b61/docs/README-NO.md">Norsk</a>] | [<a href="https://github.com/rustdesk/rustdesk/blob/beeeef14f0a61bd622aacf042b4396bdeaed9b61/docs/README-RO.md">Română</a>]<br>
  <b>我们需要你的帮助，将这份 README、 <a href="https://github.com/rustdesk/rustdesk/tree/master/src/lang">RustDesk UI</a>和 <a href="https://github.com/rustdesk/doc.rustdesk.com">RustDesk 文档</a>翻译成你的母语</b>
</p>

> [!Caution]
> **滥用免责声明：** <br>
> RustDesk 的开发人员不纵容或支持任何不道德或非法的软件使用行为。滥用行为，例如未经授权的访问、控制或侵犯隐私，严格违反我们的准则。作者对应用程序的任何滥用行为概不负责。


与我们交流： [Discord](https://discord.gg/nDceKgxnkV) | [Twitter](https://twitter.com/rustdesk) | [Reddit](https://www.reddit.com/r/rustdesk) | [YouTube](https://www.youtube.com/@rustdesk)

[![RustDesk Server Pro](https://img.shields.io/badge/RustDesk%20Server%20Pro-Advanced%20Features-blue)](https://rustdesk.com/pricing.html)

又一个使用 Rust 编写的远程桌面方案，开箱即用，无需配置。项目表示用户可以完全控制自己的数据，无需担心安全问题。你可以使用我们的注册／中继服务器，[自行搭建](https://rustdesk.com/server)，或[编写自己的注册／中继服务器](https://github.com/rustdesk/rustdesk-server-demo)。

![RustDesk 界面](https://user-images.githubusercontent.com/71636191/171661982-430285f0-2e12-4b1d-9957-4a58e375304d.png)

RustDesk 欢迎所有人参与贡献。入门请参阅 [CONTRIBUTING.md](https://github.com/rustdesk/rustdesk/blob/beeeef14f0a61bd622aacf042b4396bdeaed9b61/docs/CONTRIBUTING.md)。

[**FAQ**](https://github.com/rustdesk/rustdesk/wiki/FAQ)

[**二进制下载**](https://github.com/rustdesk/rustdesk/releases)

[**每日构建**](https://github.com/rustdesk/rustdesk/releases/tag/nightly)

[<img src="https://f-droid.org/badge/get-it-on.png"
    alt="Get it on F-Droid"
    height="80">](https://f-droid.org/en/packages/com.carriez.flutter_hbb)
[<img src="https://flathub.org/api/badge?svg&locale=en"
    alt="Get it on Flathub"
    height="80">](https://flathub.org/apps/com.rustdesk.RustDesk)

## 依赖

桌面版本使用 Flutter 或 Sciter（已弃用）作为图形界面。本教程仅适用于 Sciter，因为它更简单且更易于上手。构建 Flutter 版本请查看我们的 [CI](https://github.com/rustdesk/rustdesk/blob/master/.github/workflows/flutter-build.yml)。

请自行下载 Sciter 动态库。

[Windows](https://raw.githubusercontent.com/c-smile/sciter-sdk/master/bin.win/x64/sciter.dll) |
[Linux](https://raw.githubusercontent.com/c-smile/sciter-sdk/master/bin.lnx/x64/libsciter-gtk.so) |
[macOS](https://raw.githubusercontent.com/c-smile/sciter-sdk/master/bin.osx/libsciter.dylib)

## 基本构建步骤

- 准备 Rust 开发环境和 C++ 构建环境

- 安装 [vcpkg](https://github.com/microsoft/vcpkg)，并正确设置 `VCPKG_ROOT` 环境变量

  - Windows: vcpkg install libvpx:x64-windows-static libyuv:x64-windows-static opus:x64-windows-static aom:x64-windows-static
  - Linux/macOS: vcpkg install libvpx libyuv opus aom

- 运行 `cargo run`

## [构建](https://rustdesk.com/docs/en/dev/build/)

## 在 Linux 上构建

### Ubuntu 18 (Debian 10)

```sh
sudo apt install -y zip g++ gcc git curl wget nasm yasm libgtk-3-dev clang libxcb-randr0-dev libxdo-dev \
        libxfixes-dev libxcb-shape0-dev libxcb-xfixes0-dev libasound2-dev libpulse-dev cmake make \
        libclang-dev ninja-build libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev
```

### openSUSE Tumbleweed

```sh
sudo zypper install gcc-c++ git curl wget nasm yasm gcc gtk3-devel clang libxcb-devel libXfixes-devel cmake alsa-lib-devel gstreamer-devel gstreamer-plugins-base-devel xdotool-devel
```

### Fedora 28 (CentOS 8)

```sh
sudo yum -y install gcc-c++ git curl wget nasm yasm gcc gtk3-devel clang libxcb-devel libxdo-devel libXfixes-devel pulseaudio-libs-devel cmake alsa-lib-devel gstreamer1-devel gstreamer1-plugins-base-devel
```

### Arch (Manjaro)

```sh
sudo pacman -Syu --needed unzip git cmake gcc curl wget yasm nasm zip make pkg-config clang gtk3 xdotool libxcb libxfixes alsa-lib pipewire
```

### 安装 vcpkg

```sh
git clone https://github.com/microsoft/vcpkg
cd vcpkg
git checkout 2023.04.15
cd ..
vcpkg/bootstrap-vcpkg.sh
export VCPKG_ROOT=$HOME/vcpkg
vcpkg/vcpkg install libvpx libyuv opus aom
```

### 修复 libvpx（Fedora）

```sh
cd vcpkg/buildtrees/libvpx/src
cd *
./configure
sed -i 's/CFLAGS+=-I/CFLAGS+=-fPIC -I/g' Makefile
sed -i 's/CXXFLAGS+=-I/CXXFLAGS+=-fPIC -I/g' Makefile
make
cp libvpx.a $HOME/vcpkg/installed/x64-linux/lib/
cd
```

### 构建

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
git clone --recurse-submodules https://github.com/rustdesk/rustdesk
cd rustdesk
mkdir -p target/debug
wget https://raw.githubusercontent.com/c-smile/sciter-sdk/master/bin.lnx/x64/libsciter-gtk.so
mv libsciter-gtk.so target/debug
VCPKG_ROOT=$HOME/vcpkg cargo run
```

## 使用 Docker 构建

首先克隆仓库并构建 Docker 容器：

```sh
git clone https://github.com/rustdesk/rustdesk
cd rustdesk
git submodule update --init --recursive
docker build -t "rustdesk-builder" .
```

然后，每次需要构建应用时，运行以下命令：

```sh
docker run --rm -it -v $PWD:/home/user/rustdesk -v rustdesk-git-cache:/home/user/.cargo/git -v rustdesk-registry-cache:/home/user/.cargo/registry -e PUID="$(id -u)" -e PGID="$(id -g)" rustdesk-builder
```

请注意，首次构建时依赖尚未缓存，可能需要较长时间；后续构建会更快。如果需要为构建命令指定其他参数，可以将其放在命令末尾的 `<OPTIONAL-ARGS>` 位置。例如，要构建经过优化的发布版本，可以在上述命令后加上 `--release`。生成的可执行文件位于系统的 target 文件夹中，可使用以下命令运行：

```sh
target/debug/rustdesk
```

如果运行的是发布版可执行文件：

```sh
target/release/rustdesk
```

请确保在 RustDesk 仓库根目录运行这些命令，否则应用可能找不到所需资源。另请注意，此方法目前不支持 `install` 或 `run` 等其他 cargo 子命令，因为它们会在容器内而不是宿主机上安装或运行程序。

## 文件结构

- **[libs/hbb_common](https://github.com/rustdesk/rustdesk/tree/master/libs/hbb_common)**：视频编解码器、配置、TCP/UDP 封装，以及与服务器共享的其他实用函数
- **[libs/base](https://github.com/rustdesk/rustdesk/tree/master/libs/base)**：protobuf、文件传输的文件系统函数、仅供本应用使用的键盘与平台代码
- **[libs/scrap](https://github.com/rustdesk/rustdesk/tree/master/libs/scrap)**：屏幕采集
- **[libs/enigo](https://github.com/rustdesk/rustdesk/tree/master/libs/enigo)**：特定平台的键盘／鼠标控制
- **[libs/clipboard](https://github.com/rustdesk/rustdesk/tree/master/libs/clipboard)**：Windows、Linux 和 macOS 的文件复制粘贴实现。
- **[src/ui](https://github.com/rustdesk/rustdesk/tree/master/src/ui)**：旧版 Sciter 界面（已弃用）
- **[src/server](https://github.com/rustdesk/rustdesk/tree/master/src/server)**：音频／剪贴板／输入／视频服务和网络连接
- **[src/client.rs](https://github.com/rustdesk/rustdesk/tree/master/src/client.rs)**：启动对等连接
- **[src/rendezvous_mediator.rs](https://github.com/rustdesk/rustdesk/tree/master/src/rendezvous_mediator.rs)**：与 [rustdesk-server](https://github.com/rustdesk/rustdesk-server) 通信，等待远程直连（TCP 打洞）或中继连接
- **[src/platform](https://github.com/rustdesk/rustdesk/tree/master/src/platform)**：平台相关代码
- **[flutter](https://github.com/rustdesk/rustdesk/tree/master/flutter)**：桌面端和移动端的 Flutter 代码

## 截图

![连接管理器](https://github.com/rustdesk/rustdesk/assets/28412477/db82d4e7-c4bc-4823-8e6f-6af7eadf7651)

![连接到 Windows 电脑](https://github.com/rustdesk/rustdesk/assets/28412477/9baa91e9-3362-4d06-aa1a-7518edcbd7ea)

![文件传输](https://github.com/rustdesk/rustdesk/assets/28412477/39511ad3-aa9a-4f8c-8947-1cce286a46ad)

![TCP 隧道](https://github.com/rustdesk/rustdesk/assets/28412477/78e8708f-e87e-4570-8373-1360033ea6c5)

