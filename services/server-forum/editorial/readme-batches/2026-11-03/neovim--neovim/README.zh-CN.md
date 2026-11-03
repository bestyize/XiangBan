<h1 align="center">
  <img src="https://raw.githubusercontent.com/neovim/neovim.github.io/master/static/logos/neovim-logo-300x87.png" alt="Neovim">

  <a href="https://neovim.io/doc/">文档</a> |
  <a href="https://app.element.io/#/room/#neovim:matrix.org">聊天</a>
</h1>

[![Coverity Scan analysis](https://scan.coverity.com/projects/2227/badge.svg)](https://scan.coverity.com/projects/2227)
[![Packages](https://repology.org/badge/tiny-repos/neovim.svg)](https://repology.org/metapackage/neovim)
[![Debian CI](https://badges.debian.net/badges/debian/testing/neovim/version.svg)](https://buildd.debian.org/neovim)
[![Downloads](https://img.shields.io/github/downloads/neovim/neovim/total.svg?maxAge=2592001)](https://github.com/neovim/neovim/releases/)

Neovim 是一个致力于大幅重构 [Vim](https://www.vim.org/) 的项目，目标包括：

- 简化维护并鼓励[贡献](https://github.com/neovim/neovim/blob/6332ba64fe56f922c5cd2d93e4cfa23673966b6d/CONTRIBUTING.md)
- 让多位开发者分担工作
- 无需修改核心即可支持[高级用户界面][advanced UIs]
- 最大化[可扩展性](https://neovim.io/doc/user/api-ui-events.html#api-ui-events)

更多信息请参阅 wiki 的[介绍](https://github.com/neovim/neovim/wiki/Introduction)页面及[路线图][Roadmap]。

功能
--------

- 现代[图形界面](https://github.com/neovim/neovim/wiki/Related-projects#gui)
- [API 访问](https://github.com/neovim/neovim/wiki/Related-projects#api-clients)
  支持任意语言，包括 C/C++、C#、Clojure、D、Elixir、Go、Haskell、Java/Kotlin、
  JavaScript/Node.js、Julia、Lisp、Lua、Perl、Python、Racket、Ruby 和 Rust
- 内嵌、可脚本化的[终端模拟器](https://neovim.io/doc/user/terminal.html)
- 异步[作业控制](https://github.com/neovim/neovim/pull/2247)
- [共享数据（shada）](https://github.com/neovim/neovim/pull/2506)可在多个编辑器实例之间共享
- [XDG 基础目录](https://github.com/neovim/neovim/pull/3470)支持
- 兼容大多数 Vim 插件，包括 Ruby 和 Python 插件

完整列表请参阅 [`:help nvim-features`][nvim-features]；最新版本的重要变化请参阅 [`:help news`][nvim-news]！

从软件包安装
--------------------

Windows、macOS 和 Linux 的预构建软件包可在[发行版本](https://github.com/neovim/neovim/releases/)页面找到。

[托管软件包][Managed packages]可通过 [Homebrew]、[Debian]、[Ubuntu]、[Fedora]、[Arch Linux]、[Void Linux]、[Gentoo] 等渠道获取！

从源代码安装
-------------------

参阅 [BUILD.md](https://github.com/neovim/neovim/blob/6332ba64fe56f922c5cd2d93e4cfa23673966b6d/BUILD.md)及[支持的平台](https://neovim.io/doc/user/support.html#supported-platforms)了解详情。

构建基于 CMake，同时提供 Makefile 以方便使用。安装依赖后，运行以下命令。
```bash
make CMAKE_BUILD_TYPE=RelWithDebInfo
sudo make install
```

要安装到非默认位置：
```bash
make CMAKE_BUILD_TYPE=RelWithDebInfo CMAKE_INSTALL_PREFIX=/full/path/
make install
```

用于检查构建的 CMake 提示：

- `cmake --build build --target help`列出所有构建目标。
- `build/CMakeCache.txt` (or `cmake -LAH build/`)包含所有 CMake 变量的解析值。
- `build/compile_commands.json`显示每个编译单元的完整编译器调用。

从 Vim 迁移
--------------------

相关说明请参阅 [`:help nvim-from-vim`](https://neovim.io/doc/user/nvim.html#nvim-from-vim)。

项目结构
--------------

    ├─ cmake/           CMake utils
    ├─ cmake.config/    CMake defines
    ├─ cmake.deps/      subproject to fetch and build dependencies (optional)
    ├─ runtime/         plugins and docs
    ├─ src/nvim/        application source code (see src/nvim/README.md)
    │  ├─ api/          API subsystem
    │  ├─ eval/         Vimscript subsystem
    │  ├─ event/        event-loop subsystem
    │  ├─ generators/   code generation (pre-compilation)
    │  ├─ lib/          generic data structures
    │  ├─ lua/          Lua subsystem
    │  ├─ msgpack_rpc/  RPC subsystem
    │  ├─ os/           low-level platform code
    │  └─ tui/          built-in UI
    └─ test/            tests (see test/README.md)

许可证
-------

自 [b17d96][license-commit] 以来的 Neovim 贡献采用 Apache 2.0 许可证，但从 Vim 复制的贡献（由 `vim-patch` 标记识别）除外。详情请参阅 [LICENSE.txt](https://github.com/neovim/neovim/blob/6332ba64fe56f922c5cd2d93e4cfa23673966b6d/LICENSE.txt)。

[license-commit]: https://github.com/neovim/neovim/commit/b17d9691a24099c9210289f16afb1a498a89d803
[nvim-features]: https://neovim.io/doc/user/vim_diff.html#nvim-features
[nvim-news]: https://neovim.io/doc/user/news.html
[Roadmap]: https://neovim.io/roadmap/
[advanced UIs]: https://github.com/neovim/neovim/wiki/Related-projects#gui
[Managed packages]: https://github.com/neovim/neovim/blob/6332ba64fe56f922c5cd2d93e4cfa23673966b6d/INSTALL.md#install-from-package
[Debian]: https://packages.debian.org/testing/neovim
[Ubuntu]: https://packages.ubuntu.com/search?keywords=neovim
[Fedora]: https://packages.fedoraproject.org/pkgs/neovim/neovim/
[Arch Linux]: https://www.archlinux.org/packages/?q=neovim
[Void Linux]: https://voidlinux.org/packages/?arch=x86_64&q=neovim
[Gentoo]: https://packages.gentoo.org/packages/app-editors/neovim
[Homebrew]: https://formulae.brew.sh/formula/neovim

<!-- vim: set tw=80: -->
