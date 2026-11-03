# VLC media player

**VLC** 是自由开源的**媒体播放器**和**多媒体引擎**，专注于**播放一切**和**随处运行**。

**VLC** 可以播放大多数多媒体文件、光盘、流和设备内容，也能够将流转换、编码、**推流**并处理为多种格式。

世界各地的许多人在各种平台上为不同用途使用 VLC。

**VLC 引擎**可嵌入第三方应用，称为 *libVLC*。

**VLC** 是 [VideoLAN 项目](https://videolan.org)的一部分，由志愿者社区开发和支持。

VideoLAN 项目始于[巴黎中央理工学院](https://www.centralesupelec.fr/)，该校于 2001 年 2 月将 VLC 重新以 GPLv2 许可证发布。自那以后，VLC 已被下载**数十亿**次。

## 许可证

**VLC** 采用 GPLv2 *（或更高版本）*许可。*由于依赖项的许可证，在某些平台上实际上为 GPLv3。*

**libVLC** 引擎采用 LGPLv2 *（或更高版本）*许可。\
这允许将引擎嵌入第三方应用，同时允许这些应用采用其他许可证。

# 平台

VLC 适用于以下平台：
- [Windows] *（从 Windows 7 开始，包括 UWP 平台及所有 Windows 10 版本）*
- [macOS] *（10.10 及更高版本）*
- [GNU/Linux] 及相关系统
- [BSD] 及相关系统
- [Android] *（4.2 及更高版本）*，包括 Android TV 和 Android Auto
- [iOS] *（9 及更高版本）*，包括 AppleTV 和 iPadOS
- Haiku、OS/2 和其他少数系统。

[Windows]: https://www.videolan.org/vlc/download-windows.html
[macOS]: https://www.videolan.org/vlc/download-macosx.html
[GNU/Linux]: https://www.videolan.org/vlc/#download
[BSD]: https://www.videolan.org/vlc/download-freebsd.html
[Android]: https://www.videolan.org/vlc/download-android.html
[iOS]: https://www.videolan.org/vlc/download-ios.html

由于资源有限，并非所有平台都能得到同等程度的维护。

**注意**：[Android 应用](https://code.videolan.org/videolan/vlc-android/)和 [iOS 应用](https://code.videolan.org/videolan/vlc-ios/)位于与主仓库不同的仓库中。

# 贡献与社区

**VLC** 由社区成员维护，VideoLAN 不向他们支付报酬。\
社区由希望这个开源项目繁荣发展的开发者、协助者、维护者、设计师和作者组成。

VLC 的主要开发语言是 C，但本仓库也包含大量 C++、Obj-C、汇编和 Rust。

与 VLC 相关的其他仓库使用的语言包括 Kotlin/Java（[Android](https://code.videolan.org/videolan/vlc-android/)）、Swift（[iOS](https://code.videolan.org/videolan/vlc-ios/)）和 C#（[libVLCSharp](https://code.videolan.org/videolan/libvlcsharp/)）。

我们需要以下方面的帮助：
- 编码
- 为 Windows、macOS 和 Linux 发行版打包
- 编写技术文档
- 设计
- 支持
- 社区管理和沟通。

欢迎贡献 :)

我们在 IRC 上。你可以在 *[Libera.chat]* 的 **#videolan** 频道找到我们。

[Libera.chat]: https://libera.chat

## 贡献

现在通过我们的 [GitLab 仓库](https://code.videolan.org/videolan/vlc/)中的 Merge Request 提交贡献。

合并 Merge Request 前应解决 CI 和讨论中的问题。

# libVLC

**libVLC** 是适用于第三方应用和框架的可嵌入引擎。

它运行在与 VLC 相同的平台上（*有时支持更多平台*），可以提供多媒体文件和流的播放、推流和转换功能。

**libVLC** 提供多种其他语言的绑定，例如 C++、Python 和 C#。

# 支持

## 链接

一些可能有帮助的链接：

- [VLC 网站](https://www.videolan.org/vlc/)
- [支持](https://www.videolan.org/support/)
- [论坛](https://forum.videolan.org/)
- [Wiki](https://wiki.videolan.org/)
- [开发者专区](https://wiki.videolan.org/Developers_Corner)
- [VLC 开发指南](https://wiki.videolan.org/Hacker_Guide)
- [错误跟踪器](https://code.videolan.org/videolan/vlc/-/issues)
- [VideoLAN 网站](https://www.videolan.org/)

## 源代码目录说明
```
ABOUT-NLS          - Notes on the Free Translation Project.
AUTHORS            - VLC authors.
COPYING            - The GPL license.
COPYING.LIB        - The LGPL license.
INSTALL            - Installation and building instructions.
NEWS               - Important modifications between the releases.
README             - Project summary.
THANKS             - VLC contributors.

bin/               - VLC binaries.
bindings/          - libVLC bindings to other languages.
compat/            - compatibility library for operating systems missing
                     essential functionalities.
contrib/           - Facilities for retrieving external libraries and building
                     them for systems that don't have the right versions.
doc/               - Miscellaneous documentation.
extras/analyser    - Code analyser and editor specific files.
extras/buildsystem - Different build system specific files.
extras/misc        - Files that don't fit in the other extras/ categories.
extras/package     - VLC packaging specific files such as spec files.
extras/tools/      - Facilities for retrieving external building tools needed
                     for systems that don't have the right versions.
include/           - Header files.
lib/               - libVLC source code.
modules/           - VLC plugins and modules. Most of the code is here.
po/                - VLC translations.
share/             - Common resource files.
src/               - libvlccore source code.
test/              - Testing system.
```
