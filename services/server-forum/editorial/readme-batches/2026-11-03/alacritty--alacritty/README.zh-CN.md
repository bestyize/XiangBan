<p align="center">
    <img width="200" alt="Alacritty Logo" src="https://raw.githubusercontent.com/alacritty/alacritty/aecf0f600fa840a415b55da5cf34fbeaec39ac32/extra/logo/compat/alacritty-term%2Bscanlines.png">
</p>

<h1 align="center">Alacritty - 快速、跨平台的 OpenGL 终端模拟器</h1>

<p align="center">
  <img alt="Alacritty - 快速、跨平台的 OpenGL 终端模拟器"
       src="https://raw.githubusercontent.com/alacritty/alacritty/aecf0f600fa840a415b55da5cf34fbeaec39ac32/extra/promo/alacritty-readme.png">
</p>

## 简介

Alacritty 是一个现代终端模拟器，具有合理的默认设置，同时允许广泛的[配置](#配置)。它通过与其他应用集成，而不是重新实现其功能，提供灵活的[功能](https://github.com/alacritty/alacritty/blob/aecf0f600fa840a415b55da5cf34fbeaec39ac32/docs/features.md)并保持高性能。目前支持的平台包括 BSD、Linux、macOS 和 Windows。

该软件被认为处于 **beta** 阶段；仍有部分功能缺失和错误需要修复，但许多人已经将它作为日常工具使用。

预编译二进制文件可从 [GitHub 发行页面](https://github.com/alacritty/alacritty/releases)获取。

如果你有问题或需要快速帮助，可以加入 libera.chat 上的 [`#alacritty`]。

[`#alacritty`]: https://web.libera.chat/gamja/?channels=#alacritty

## 功能

可在[此处](https://github.com/alacritty/alacritty/blob/aecf0f600fa840a415b55da5cf34fbeaec39ac32/docs/features.md)查看 Alacritty 的功能概览。

## 更多信息

- [Announcing Alacritty, a GPU-Accelerated Terminal Emulator](https://jwilm.io/blog/announcing-alacritty/) 2017 年 1 月 6 日
- [A talk about Alacritty at the Rust Meetup January 2017](https://www.youtube.com/watch?v=qHOdYO3WUTk) 2017 年 1 月 19 日
- [Alacritty Lands Scrollback, Publishes Benchmarks](https://jwilm.io/blog/alacritty-lands-scrollback/) 2018 年 9 月 17 日

## 安装

Alacritty 可以通过 Linux、BSD、macOS 和 Windows 上的多种包管理器安装。

macOS 和 Windows 的预构建二进制文件也可从 [GitHub 发行页面](https://github.com/alacritty/alacritty/releases)下载。

其他用户可以在[此处](https://github.com/alacritty/alacritty/blob/aecf0f600fa840a415b55da5cf34fbeaec39ac32/INSTALL.md)查看详细的 Alacritty 安装说明。

### 系统要求

- 至少需要 OpenGL ES 2.0
- [Windows] 支持 ConPTY（Windows 10 版本 1809 或更高版本）

## 配置

可通过 `man 5
alacritty` 查看 Alacritty 配置文档；如果没有安装 man 页面，也可以查看[网站][the website]。

[the website]: https://alacritty.org/config-alacritty.html

Alacritty 不会自动创建配置文件，但会在以下位置查找：

1. `$XDG_CONFIG_HOME/alacritty/alacritty.toml`
2. `$XDG_CONFIG_HOME/alacritty.toml`
3. `$HOME/.config/alacritty/alacritty.toml`
4. `$HOME/.alacritty.toml`
5. `/etc/alacritty/alacritty.toml`

在 Windows 上，会在以下位置查找配置文件：

* `%APPDATA%\alacritty\alacritty.toml`

## 参与贡献

Alacritty 的贡献指南位于 [`CONTRIBUTING.md`](https://github.com/alacritty/alacritty/blob/aecf0f600fa840a415b55da5cf34fbeaec39ac32/CONTRIBUTING.md) 文件中。

## FAQ

**_它真的是最快的终端模拟器吗？_**

终端模拟器的基准测试很复杂。Alacritty 使用 [vtebench](https://github.com/alacritty/vtebench) 量化终端模拟器的吞吐量，并在该测试中持续取得优于其他产品的成绩。如果你发现并非如此的例子，请报告问题。

延迟、帧率和帧一致性等其他方面更难量化。有些终端模拟器也会有意减速以节省资源，这可能是部分用户更喜欢的方式。

如果你对 Alacritty 的性能或易用性有疑问，衡量终端模拟器的最佳方法始终是使用**你自己的**具体场景进行测试。

**_为什么没有实现某个功能？_**

Alacritty 有许多功能，但并不包含其他每个终端中的所有功能。原因可能有很多，有时只是某个功能不适合 Alacritty。因此，你不会在其中找到标签页或分屏（这些最好交给窗口管理器或[终端复用器][tmux]），也不会找到图形界面配置编辑器等便利功能。

[tmux]: https://github.com/tmux/tmux

## 许可证

Alacritty 按照 [Apache License, Version 2.0] 发布。

[Apache License, Version 2.0]: https://github.com/alacritty/alacritty/blob/aecf0f600fa840a415b55da5cf34fbeaec39ac32/LICENSE-APACHE
