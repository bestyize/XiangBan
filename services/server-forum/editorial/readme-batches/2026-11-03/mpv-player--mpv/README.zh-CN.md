![mpv logo](https://raw.githubusercontent.com/mpv-player/mpv.io/master/source/images/mpv-logo-128.png)

# mpv


* [外部链接](#外部链接)
* [概览](#概览)
* [系统要求](#系统要求)
* [下载](#下载)
* [更新日志](#更新日志)
* [编译](#编译)
* [发布周期](#发布周期)
* [错误报告](#错误报告)
* [贡献](#贡献)
* [许可证](#许可证)
* [联系](#联系)


## 外部链接


* [Wiki](https://github.com/mpv-player/mpv/wiki)
* [用户脚本](https://github.com/mpv-player/mpv/wiki/User-Scripts)
* [FAQ][FAQ]
* [手册](https://mpv.io/manual/master/)


## 概览


**mpv** 是一款自由的命令行媒体播放器。“自由”指自由软件意义上的自由。它支持多种媒体文件格式、音视频编解码器和字幕类型。

另有[常见问题][FAQ]。

发布版本可在[版本列表][releases]中找到。

## 系统要求

- 不太古老的 Linux（通常只积极支持发行版的最新版本）、Windows 10 1607 或更高版本，或 macOS 10.15 或更高版本。
- 性能尚可的 CPU。如果 CPU 太慢无法实时解码视频，硬件解码可能有所帮助，但必须通过 `--hwdec` 选项明确启用。
- 性能不要太差的 GPU。mpv 并不专注于在嵌入式或集成 GPU 上实现低功耗播放（例如，默认甚至不启用硬件解码）。低功耗 GPU 可能出现画面撕裂、卡顿等问题；建议在这些 GPU 上使用 `--profile=fast` 以获得流畅播放。主要视频输出使用着色器进行视频渲染和缩放，而非 GPU 固定功能硬件。在 Windows 上，建议确保显卡驱动已更新。某些情况下，古老的备用视频输出方式可能有帮助（例如 Linux 上的 `--vo=xv`），但不推荐也不支持这种用法。

mpv 不会刻意破坏对旧硬件或已不受支持的旧操作系统的兼容，但开发时并未针对它们进行考虑，因此不保证兼容。如果能正常工作，请把它当作幸运的意外。

## 下载


半官方构建与第三方软件包请参阅 [mpv.io/installation](https://mpv.io/installation/)。

## 更新日志


没有完整的更新日志；播放器核心接口的更改列在[接口更新日志][interface-changes]中。

C API 的更改记录在[客户端 API 更新日志][api-changes]中。

[版本列表][releases]汇总了每个版本的大部分重要更改。

默认按键绑定的更改记录在 [restore-old-bindings.conf][restore-old-bindings] 中。

默认 OSC 绑定的更改记录在 [restore-osc-bindings.conf][restore-osc-bindings] 中。

## 编译


构建完整功能需要若干外部库的开发文件。构建 mpv 需要 [meson](https://mesonbuild.com/index.html)，可从发行版或 PyPI 获取。

创建构建目录后（例如 `meson setup build`），可以通过 `meson configure build` 查看全部构建选项，也可以直接查看 `meson_options.txt`。日志保存在构建目录内的 `meson-logs` 中。

示例：

    meson setup build
    meson compile -C build
    meson install -C build

如果构建环境中难以获得足够新版本的 libplacebo，meson 可以将 Git 检出作为子项目，方便编译 mpv。它将与 mpv 静态链接。例如：

    mkdir -p subprojects
    git clone https://code.videolan.org/videolan/libplacebo.git --depth=1 --recursive subprojects/libplacebo

必要依赖（列表不完整）：

- gcc 或 clang
- X 开发头文件（xlib、xrandr、xext、xscrnsaver、xpresent、libvdpau、libGL、GLX、EGL、xv 等）
- 音频输出开发头文件（libasound/ALSA、pulseaudio）
- FFmpeg 库（libavutil、libavcodec、libavformat、libswscale、libavfilter，以及 libswresample 或 libavresample）
- libplacebo
- zlib
- iconv（通常由系统 libc 提供）
- libass（OSD、OSC、文本字幕）
- Lua（可选；OSC 伪图形界面和 youtube-dl 集成需要）
- libjpeg（可选，仅用于截图）
- uchardet（可选，用于字幕字符集检测）
- Linux 硬件解码所需的 nvdec 和 vaapi 库（可选）

Libass 依赖（构建 libass 时）：

- gcc 或 clang；x86 和 x86_64 上需要 nasm
- fribidi、freetype、fontconfig 开发头文件（用于 libass）
- harfbuzz（正确渲染组合字符所需，尤其是 macOS 上的非英语文本以及任意平台上的阿拉伯文／印度文字）

FFmpeg 依赖（构建 FFmpeg 时）：

- gcc 或 clang；x86 和 x86_64 上需要 nasm
- OpenSSL 或 GnuTLS（编译 FFmpeg 时必须显式启用）
- 如果需要编码，则需要 libx264/libmp3lame/libfdk-aac（编译 FFmpeg 时必须显式启用）
- 原生 DASH 播放需要使用 --enable-libxml2 构建 FFmpeg（但存在安全影响，且 DASH 支持有许多错误）。
- AV1 解码支持需要 dav1d。
- 为在 Linux 上获得良好的 NVIDIA 支持，请确保已安装 nv-codec-headers 且 configure 能找到它。

大多数常见 Linux 发行版都提供以上库的合适版本。为了方便编译所有组件的最新 Git master，可使用单独提供的构建包装工具（[mpv-build][mpv-build]）；它先编译 FFmpeg 库和 libass，然后编译与它们静态链接的播放器。

如果要构建 Windows 二进制文件，请参阅 [Windows 编译][windows_compilation]。


## 发布周期

每年一到两次，项目从当前开发状态制作发行版并赋予 0.X.0 版本号。除出现安全问题外，不再对该发行版进行维护。

发布版本的目的是方便 Linux 发行版使用。出现错误时，也预期由 Linux 发行版自行应用补丁。

除最新发行版外，其他版本均不受支持且不维护。

更多信息请参阅[发布策略文档][release-policy]。

## 错误报告


请通过 GitHub 提供的 [issue 跟踪器][issue-tracker]提交错误报告或功能请求。请遵循模板说明，否则 issue 很可能被忽略或作为无效问题关闭。

问题可以在 [discussions][discussions] 或 IRC 中提出（参见下方[联系](#联系)）。

## 贡献


请阅读 [contribute.md][contribute.md]。

小修改可以直接通过 GitHub 提交 pull request。较大的修改请在开始前到 IRC 与我们讨论，以便双方之后更容易进行代码审查。

可以查看 [Wiki](https://github.com/mpv-player/mpv/wiki/Stuff-to-do) 或 [issue 跟踪器](https://github.com/mpv-player/mpv/issues?q=is%3Aopen+is%3Aissue+label%3Ameta%3Afeature-request)，了解可以贡献的内容。

## 许可证

默认采用 GPLv2“或更高版本”；使用 `-Dgpl=false` 时采用 LGPLv2.1“或更高版本”。请参阅[详情](https://github.com/mpv-player/mpv/blob/a687796d9d0efded1787d497167facd462b8f2ee/Copyright)

> 译注：上游 Copyright 文件进一步说明，设置 `-Dgpl=false` 本身并不产生 LGPL 许可授予，文档和部分非程序文件仍有各自许可；分发前请阅读上游许可详情。

## 历史

本软件基于 MPlayer 项目。在 mpv 项目出现之前，这套代码曾短暂以 mplayer2 项目的形式开发。详情请参阅[常见问题][FAQ]。

## 联系


大部分活动在 IRC 频道和 GitHub issue 跟踪器中进行。

- **GitHub issue 跟踪器**： [issue tracker][issue-tracker]（在此报告错误）
- **讨论**： [discussions][discussions]
- **用户 IRC 频道**： `#mpv`，位于 `irc.libera.chat`
- **开发者 IRC 频道**： `#mpv-devel`，位于 `irc.libera.chat`

[FAQ]: https://github.com/mpv-player/mpv/wiki/FAQ
[releases]: https://github.com/mpv-player/mpv/releases
[mpv-build]: https://github.com/mpv-player/mpv-build
[issue-tracker]:  https://github.com/mpv-player/mpv/issues
[discussions]: https://github.com/mpv-player/mpv/discussions
[release-policy]: https://github.com/mpv-player/mpv/blob/a687796d9d0efded1787d497167facd462b8f2ee/DOCS/release-policy.md
[windows_compilation]: https://github.com/mpv-player/mpv/blob/a687796d9d0efded1787d497167facd462b8f2ee/DOCS/compile-windows.md
[interface-changes]: https://github.com/mpv-player/mpv/blob/a687796d9d0efded1787d497167facd462b8f2ee/DOCS/interface-changes.rst
[api-changes]: https://github.com/mpv-player/mpv/blob/a687796d9d0efded1787d497167facd462b8f2ee/DOCS/client-api-changes.rst
[restore-old-bindings]: https://github.com/mpv-player/mpv/blob/a687796d9d0efded1787d497167facd462b8f2ee/etc/restore-old-bindings.conf
[restore-osc-bindings]: https://github.com/mpv-player/mpv/blob/a687796d9d0efded1787d497167facd462b8f2ee/etc/restore-osc-bindings.conf
[contribute.md]: https://github.com/mpv-player/mpv/blob/a687796d9d0efded1787d497167facd462b8f2ee/DOCS/contribute.md
