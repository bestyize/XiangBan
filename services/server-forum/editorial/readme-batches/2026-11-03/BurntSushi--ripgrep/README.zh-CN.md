> 根据 [ripgrep 固定版本 README](https://github.com/BurntSushi/ripgrep/blob/91f5d22ef48bde459d90a8c7dbeab416e97d0f2b/README.md) 翻译。基准数据为上游在文中所述测试条件下的结果，未经本次独立复测。项目可选择 MIT 或 Unlicense。

ripgrep (rg)
------------
ripgrep 是按行搜索的工具，会递归搜索当前目录中与正则表达式匹配的内容。默认情况下，ripgrep 遵循 gitignore 规则，并自动跳过隐藏文件、目录和二进制文件。（要默认禁用所有自动过滤，请使用 `rg -uuu`。）ripgrep 对 Windows、macOS 和 Linux 提供一流支持，[每个发行版](https://github.com/BurntSushi/ripgrep/releases)均提供二进制下载。ripgrep 与 The Silver Searcher、ack 和 grep 等流行搜索工具类似。

[![Build status](https://github.com/BurntSushi/ripgrep/workflows/ci/badge.svg)](https://github.com/BurntSushi/ripgrep/actions)
[![Crates.io](https://img.shields.io/crates/v/ripgrep.svg)](https://crates.io/crates/ripgrep)
[![Packaging status](https://repology.org/badge/tiny-repos/ripgrep.svg)](https://repology.org/project/ripgrep/badges)

采用 MIT 或 [UNLICENSE](https://unlicense.org) 双重许可。

### 更新日志

发行历史请参阅 [CHANGELOG](https://github.com/BurntSushi/ripgrep/blob/91f5d22ef48bde459d90a8c7dbeab416e97d0f2b/CHANGELOG.md)。

### 文档快速链接

* [安装](#installation)
* [用户指南](https://github.com/BurntSushi/ripgrep/blob/91f5d22ef48bde459d90a8c7dbeab416e97d0f2b/GUIDE.md)
* [常见问题](https://github.com/BurntSushi/ripgrep/blob/91f5d22ef48bde459d90a8c7dbeab416e97d0f2b/FAQ.md)
* [正则语法](https://docs.rs/regex/1/regex/#syntax)
* [配置文件](https://github.com/BurntSushi/ripgrep/blob/91f5d22ef48bde459d90a8c7dbeab416e97d0f2b/GUIDE.md#configuration-file)
* [Shell 补全](https://github.com/BurntSushi/ripgrep/blob/91f5d22ef48bde459d90a8c7dbeab416e97d0f2b/FAQ.md#complete)
* [构建](#building)
* [翻译](#translations)

### 搜索结果截图

[![A screenshot of a sample search with ripgrep](https://burntsushi.net/stuff/ripgrep1.png)](https://burntsushi.net/stuff/ripgrep1.png)

### 工具比较示例

此示例在完整的 [Linux 内核源代码树](https://github.com/BurntSushi/linux)（运行 `make defconfig && make -j8` 后）中搜索 `[A-Z]+_SUSPEND`，且所有匹配必须是完整单词。计时在配备 Intel i9-12900K 5.2 GHz 的系统上采集。

请记住，单次基准测试永远不够！详尽的比较、更多基准和分析请参阅我的 [ripgrep 博客文章](https://blog.burntsushi.net/ripgrep/)。

| 工具 | 命令 | 行数 | 时间 |
| ---- | ------- | ---------- | ---- |
| ripgrep (Unicode) | `rg -n -w '[A-Z]+_SUSPEND'` | 536 | **0.082s** (1.00x) |
| [hypergrep](https://github.com/p-ranav/hypergrep) | `hgrep -n -w '[A-Z]+_SUSPEND'` | 536 | 0.167s (2.04x) |
| [git grep](https://www.kernel.org/pub/software/scm/git/docs/git-grep.html) | `git grep -P -n -w '[A-Z]+_SUSPEND'` | 536 | 0.273s (3.34x) |
| [The Silver Searcher](https://github.com/ggreer/the_silver_searcher) | `ag -w '[A-Z]+_SUSPEND'` | 534 | 0.443s (5.43x) |
| [ugrep](https://github.com/Genivia/ugrep) | `ugrep -r --ignore-files --no-hidden -I -w '[A-Z]+_SUSPEND'` | 536 | 0.639s (7.82x) |
| [git grep](https://www.kernel.org/pub/software/scm/git/docs/git-grep.html) | `LC_ALL=C git grep -E -n -w '[A-Z]+_SUSPEND'` | 536 | 0.727s (8.91x) |
| [git grep (Unicode)](https://www.kernel.org/pub/software/scm/git/docs/git-grep.html) | `LC_ALL=en_US.UTF-8 git grep -E -n -w '[A-Z]+_SUSPEND'` | 536 | 2.670s (32.70x) |
| [ack](https://github.com/beyondgrep/ack3) | `ack -w '[A-Z]+_SUSPEND'` | 2677 | 2.935s (35.94x) |

下面是使用与上面相同的语料进行的另一个基准测试：忽略 gitignore 文件，改用白名单搜索。每条命令使用的参数确保它们执行等价的工作：

| 工具 | 命令 | 行数 | 时间 |
| ---- | ------- | ---------- | ---- |
| ripgrep | `rg -uuu -tc -n -w '[A-Z]+_SUSPEND'` | 447 | **0.063s** (1.00x) |
| [ugrep](https://github.com/Genivia/ugrep) | `ugrep -r -n --include='*.c' --include='*.h' -w '[A-Z]+_SUSPEND'` | 447 | 0.607s (9.62x) |
| [GNU grep](https://www.gnu.org/software/grep/) | `grep -E -r -n --include='*.c' --include='*.h' -w '[A-Z]+_SUSPEND'` | 447 | 0.674s (10.69x) |

接下来搜索单个大文件。以下直接比较 ripgrep、ugrep 和 GNU grep 在内存缓存文件（约 13GB，解压后的 [`OpenSubtitles.raw.en.gz`](https://object.pouta.csc.fi/OPUS-OpenSubtitles/v2018/mono/en.txt.gz)）上的表现：

| 工具 | 命令 | 行数 | 时间 |
| ---- | ------- | ---------- | ---- |
| ripgrep (Unicode) | `rg -w 'Sherlock [A-Z]\w+'` | 7882 | **1.042s** (1.00x) |
| [ugrep](https://github.com/Genivia/ugrep) | `ugrep -w 'Sherlock [A-Z]\w+'` | 7882 | 1.339s (1.28x) |
| [GNU grep (Unicode)](https://www.gnu.org/software/grep/) | `LC_ALL=en_US.UTF-8 egrep -w 'Sherlock [A-Z]\w+'` | 7882 | 6.577s (6.31x) |

在上述基准中，传入用于显示行号的 `-n` 参数后，ripgrep 的时间增加到 `1.664s`，GNU grep 增加到 `9.484s`。ugrep 的时间不受是否使用 `-n` 的影响。

不过要注意性能断崖：

| 工具 | 命令 | 行数 | 时间 |
| ---- | ------- | ---------- | ---- |
| ripgrep (Unicode) | `rg -w '[A-Z]\w+ Sherlock [A-Z]\w+'` | 485 | **1.053s** (1.00x) |
| [GNU grep (Unicode)](https://www.gnu.org/software/grep/) | `LC_ALL=en_US.UTF-8 grep -E -w '[A-Z]\w+ Sherlock [A-Z]\w+'` | 485 | 6.234s (5.92x) |
| [ugrep](https://github.com/Genivia/ugrep) | `ugrep -w '[A-Z]\w+ Sherlock [A-Z]\w+'` | 485 | 28.973s (27.51x) |

在大文件中搜索无法进行任何字面量优化的模式时，各工具的性能都可能急剧下降：

| 工具 | 命令 | 行数 | 时间 |
| ---- | ------- | ---------- | ---- |
| ripgrep | `rg '[A-Za-z]{30}'` | 6749 | **15.569s** (1.00x) |
| [ugrep](https://github.com/Genivia/ugrep) | `ugrep -E '[A-Za-z]{30}'` | 6749 | 21.857s (1.40x) |
| [GNU grep](https://www.gnu.org/software/grep/) | `LC_ALL=C grep -E '[A-Za-z]{30}'` | 6749 | 32.409s (2.08x) |
| [GNU grep (Unicode)](https://www.gnu.org/software/grep/) | `LC_ALL=en_US.UTF-8 grep -E '[A-Za-z]{30}'` | 6795 | 8m30s (32.74x) |

最后，大量匹配也往往既会降低性能，又会缩小工具之间的差异（因为总体而言，性能主要取决于处理匹配结果的速度，而不是检测匹配所用的算法）：

| 工具 | 命令 | 行数 | 时间 |
| ---- | ------- | ---------- | ---- |
| ripgrep | `rg the` | 83499915 | **6.948s** (1.00x) |
| [ugrep](https://github.com/Genivia/ugrep) | `ugrep the` | 83499915 | 11.721s (1.69x) |
| [GNU grep](https://www.gnu.org/software/grep/) | `LC_ALL=C grep the` | 83499915 | 15.217s (2.19x) |

### 为什么使用 ripgrep？

* 它包含其他搜索工具的大多数功能且通常更快，因此可以替代许多使用场景。（关于 ripgrep 是否真正能替代 grep，详见[常见问题](https://github.com/BurntSushi/ripgrep/blob/91f5d22ef48bde459d90a8c7dbeab416e97d0f2b/FAQ.md#posix4ever)。）
* 与其他专门用于代码搜索的工具一样，ripgrep 默认[递归搜索](https://github.com/BurntSushi/ripgrep/blob/91f5d22ef48bde459d90a8c7dbeab416e97d0f2b/GUIDE.md#recursive-search)并进行[自动过滤](https://github.com/BurntSushi/ripgrep/blob/91f5d22ef48bde459d90a8c7dbeab416e97d0f2b/GUIDE.md#automatic-filtering)。也就是说，ripgrep 不搜索 `.gitignore`、`.ignore`、`.rgignore` 文件忽略的文件，不搜索隐藏文件，也不搜索二进制文件。可以用 `rg -uuu` 禁用自动过滤。
* ripgrep 可以[搜索特定类型的文件](https://github.com/BurntSushi/ripgrep/blob/91f5d22ef48bde459d90a8c7dbeab416e97d0f2b/GUIDE.md#manual-filtering-file-types)。例如，`rg -tpy foo` 将搜索限制在 Python 文件中，`rg -Tjs foo` 则排除 JavaScript 文件。可以通过自定义匹配规则为 ripgrep 添加新的文件类型。
* ripgrep 支持 `grep` 的许多功能，如显示搜索结果上下文、搜索多个模式、用颜色突出匹配内容，以及完整的 Unicode 支持。与 GNU grep 不同，ripgrep 在支持 Unicode（始终启用）的同时仍保持快速。
* ripgrep 可选用 PCRE2 正则引擎，从而支持默认正则引擎不支持的环视和反向引用。可以用 `-P/--pcre2`（始终使用 PCRE2）或 `--auto-hybrid-regex`（仅在需要时使用 PCRE2）开启，也可以用 `--engine (default|pcre2|auto)` 语法。
* ripgrep [初步支持替换](https://github.com/BurntSushi/ripgrep/blob/91f5d22ef48bde459d90a8c7dbeab416e97d0f2b/GUIDE.md#replacements)，可以根据匹配内容改写输出。
* ripgrep 支持[搜索非 UTF-8 编码的文本文件](https://github.com/BurntSushi/ripgrep/blob/91f5d22ef48bde459d90a8c7dbeab416e97d0f2b/GUIDE.md#file-encoding)，如 UTF-16、latin-1、GBK、EUC-JP、Shift_JIS 等。（它对 UTF-16 提供部分自动检测；其他编码必须用 `-E/--encoding` 明确指定。）
* ripgrep 使用 `-z/--search-zip` 标志搜索采用常见格式压缩的文件，包括 brotli、bzip2、gzip、lz4、lzma、xz 和 zstandard。
* ripgrep 支持[任意输入预处理过滤器](https://github.com/BurntSushi/ripgrep/blob/91f5d22ef48bde459d90a8c7dbeab416e97d0f2b/GUIDE.md#preprocessor)，可用于 PDF 文本提取、对支持较少的格式解压、解密、自动检测编码等。
* ripgrep 可通过[配置文件](https://github.com/BurntSushi/ripgrep/blob/91f5d22ef48bde459d90a8c7dbeab416e97d0f2b/GUIDE.md#configuration-file)配置。

换句话说，如果你喜欢速度、默认过滤、更少的错误和 Unicode 支持，可以使用 ripgrep。

### 为什么不使用 ripgrep？

尽管最初不想将所有可能的功能都加入 ripgrep，但随着时间推移，它已经支持其他文件搜索工具中的大多数功能，包括跨多行搜索，以及可选的 PCRE2 支持，后者提供环视和反向引用。

目前不使用 ripgrep 的主要原因可能包括以下一项或多项：

* 你需要可移植且无处不在的工具。虽然 ripgrep 可在 Windows、macOS 和 Linux 上运行，但并不普及，也不符合 POSIX 等标准。此时最好的工具是经典的 grep。
* 你依赖其他工具中存在、但 ripgrep 没有且本 README 未列出的某项功能（或错误）。
* 某种性能边界场景下 ripgrep 表现不佳，而其他工具表现良好。（请提交错误报告！）
* 无法在你的机器上安装 ripgrep，或它不适用于你的平台。（请提交错误报告！）

### 它真的比其他所有工具都快吗？

通常是的。我的[博客](https://blog.burntsushi.net/ripgrep/)提供了大量基准测试及逐项详细分析。

概括来说，ripgrep 快的原因是：

* 它建立在 [Rust 正则表达式引擎](https://github.com/rust-lang/regex)之上。该引擎使用有限自动机、SIMD 和积极的字面量优化实现快速搜索。（可通过 `-P/--pcre2` 选择使用 PCRE2。）
* Rust 正则库将 UTF-8 解码直接构建进确定性有限自动机引擎，从而在完整支持 Unicode 的同时维持性能。
* 它支持内存映射搜索，也支持使用中间缓冲区进行增量搜索。前者更适合单个文件，后者更适合大目录。ripgrep 会自动为你选择最佳策略。
* 它使用 [`RegexSet`](https://docs.rs/regex/1/regex/struct.RegexSet.html) 应用 `.gitignore` 文件中的忽略模式，因此可以同时用多个 glob 模式匹配同一个路径。
* 它使用无锁并行递归目录迭代器，得益于 [`crossbeam`](https://docs.rs/crossbeam) 和 [`ignore`](https://docs.rs/ignore)。

### 功能比较

[ack](https://beyondgrep.com/) 作者 Andy Lester 发布了一个出色的表格，比较 ack、ag、git-grep、GNU grep 和 ripgrep 的功能：https://beyondgrep.com/feature-comparison/

注意 ripgrep 最近增加了一些重要功能，尚未出现在 Andy 的表格中，包括但不限于配置文件、passthru、搜索压缩文件、多行搜索及通过 PCRE2 可选启用的高级正则支持。

### 在线试用

如果想在安装前试用 ripgrep，可以使用非官方[在线试用](https://codapi.org/ripgrep/)和[交互式教程](https://codapi.org/try/ripgrep/)。

如对此有疑问，请在[教程仓库](https://github.com/nalgeon/tryxinyminutes)提交 issue。

<a id="installation"></a>
### 安装

ripgrep 的二进制文件名为 `rg`。

**[Windows、macOS 和 Linux 可下载 ripgrep 预编译二进制归档。](https://github.com/BurntSushi/ripgrep/releases)** Linux 和 Windows 二进制文件为静态可执行文件。未在下方明确提及的平台用户建议下载其中一个归档。

如果使用 **macOS Homebrew** 或 **Linuxbrew**，可以从 homebrew-core 安装 ripgrep：

```
$ brew install ripgrep
```

如果使用 **MacPorts**，可以从[官方 ports](https://www.macports.org/ports.php?by=name&substr=ripgrep) 安装 ripgrep：

```
$ sudo port install ripgrep
```

如果使用 **Windows Chocolatey**，可以从[官方仓库](https://chocolatey.org/packages/ripgrep)安装 ripgrep：

```
$ choco install ripgrep
```

如果使用 **Windows Scoop**，可以从[官方 bucket](https://github.com/ScoopInstaller/Main/blob/master/bucket/ripgrep.json) 安装 ripgrep：

```
$ scoop install ripgrep
```

如果使用 **Windows Winget**，可以从 [winget-pkgs](https://github.com/microsoft/winget-pkgs/tree/master/manifests/b/BurntSushi/ripgrep) 仓库安装 ripgrep：

```
$ winget install BurntSushi.ripgrep.MSVC
```

如果使用 **Arch Linux**，可以从官方仓库安装 ripgrep：

```
$ sudo pacman -S ripgrep
```

如果使用 **Gentoo**，可以从[官方仓库](https://packages.gentoo.org/packages/sys-apps/ripgrep)安装 ripgrep：

```
$ sudo emerge sys-apps/ripgrep
```

如果使用 **Fedora**，可以从官方仓库安装 ripgrep。

```
$ sudo dnf install ripgrep
```

如果使用 **openSUSE**，从 15.1 起 **openSUSE Tumbleweed** 和 **openSUSE Leap** 均包含 ripgrep。

```
$ sudo zypper install ripgrep
```

如果使用 **CentOS Stream 10**，可以从 [EPEL](https://docs.fedoraproject.org/en-US/epel/getting-started/) 仓库安装 ripgrep：

```
$ sudo dnf config-manager --set-enabled crb
$ sudo dnf install https://dl.fedoraproject.org/pub/epel/epel-release-latest-10.noarch.rpm
$ sudo dnf install ripgrep
```

如果使用 **Red Hat 10**，可以从 [EPEL](https://docs.fedoraproject.org/en-US/epel/getting-started/) 仓库安装 ripgrep：

```
$ sudo subscription-manager repos --enable codeready-builder-for-rhel-10-$(arch)-rpms
$ sudo dnf install https://dl.fedoraproject.org/pub/epel/epel-release-latest-10.noarch.rpm
$ sudo dnf install ripgrep
```

如果使用 **Rocky Linux 10**，可以从 [EPEL](https://docs.fedoraproject.org/en-US/epel/getting-started/) 仓库安装 ripgrep：

```
$ sudo dnf install https://dl.fedoraproject.org/pub/epel/epel-release-latest-10.noarch.rpm
$ sudo dnf install ripgrep
```

如果使用 **Nix**，可以从 [nixpkgs](https://github.com/NixOS/nixpkgs/blob/master/pkgs/by-name/ri/ripgrep/package.nix) 安装 ripgrep：

```
$ nix-env --install ripgrep
```

如果使用 **Flox**，可以按以下方式安装 ripgrep：

```
$ flox install ripgrep
```

如果使用 **Guix**，可以从官方软件包集合安装 ripgrep：

```
$ guix install ripgrep
```

如果使用 **Debian**（或 **Ubuntu** 等 Debian 衍生发行版），可以使用每个 [ripgrep 发行版](https://github.com/BurntSushi/ripgrep/releases)提供的二进制 `.deb` 文件安装。

```
$ curl -LO https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep_14.1.1-1_amd64.deb
$ sudo dpkg -i ripgrep_14.1.1-1_amd64.deb
```

如果使用 Debian 稳定版，ripgrep 由 [Debian 官方维护](https://tracker.debian.org/pkg/rust-ripgrep)，但其版本可能比上一步的 `deb` 包旧。

```
$ sudo apt-get install ripgrep
```

如果使用 **Ubuntu Cosmic (18.10)** 或更新版本，可以使用与 Debian 相同的软件包[安装](https://launchpad.net/ubuntu/+source/rust-ripgrep) ripgrep：

```
$ sudo apt-get install ripgrep
```

（注意：Ubuntu 上也有多种 ripgrep snap 包，但它们似乎都无法正常工作，并产生许多我不知道如何修复、也没有时间修复的奇怪错误报告。因此不再推荐这种安装方式。）

如果使用 **ALT**，可以从[官方仓库](https://packages.altlinux.org/en/search?name=ripgrep)安装 ripgrep：

```
$ sudo apt-get install ripgrep
```

如果使用 **FreeBSD**，可以从[官方 ports](https://www.freshports.org/textproc/ripgrep/) 安装 ripgrep：

```
$ sudo pkg install ripgrep
```

如果使用 **OpenBSD**，可以从[官方 ports](https://openports.se/textproc/ripgrep) 安装 ripgrep：

```
$ doas pkg_add ripgrep
```

如果使用 **NetBSD**，可以从 [pkgsrc](https://pkgsrc.se/textproc/ripgrep) 安装 ripgrep：

```
$ sudo pkgin install ripgrep
```

如果使用 **Haiku x86_64**，可以从[官方 ports](https://github.com/haikuports/haikuports/tree/master/sys-apps/ripgrep) 安装 ripgrep：

```
$ sudo pkgman install ripgrep
```

如果使用 **Haiku x86_gcc2**，可以通过 x86 次要架构构建，从与 Haiku x86_64 相同的 port 安装 ripgrep：

```
$ sudo pkgman install ripgrep_x86
```

如果使用 **Void Linux**，可以从[官方仓库](https://voidlinux.org/packages/?arch=x86_64&q=ripgrep)安装 ripgrep：

```
$ sudo xbps-install -Syv ripgrep
```

如果你是 **Rust 程序员**，可以用 `cargo` 安装 ripgrep。

* 注意 ripgrep 支持的最低 Rust 版本为 **1.96.0**，但它也可能适用于更旧版本。
* 注意二进制文件可能比预期大，因为它包含调试符号。这是有意为之。要移除调试符号并缩小文件，请对二进制文件运行 `strip`。

```
$ cargo install ripgrep
```

也可以使用 [`cargo binstall`](https://github.com/cargo-bins/cargo-binstall) 直接从 GitHub 安装 ripgrep 二进制文件：

```
$ cargo binstall ripgrep
```

<a id="building"></a>
### 构建

ripgrep 使用 Rust 编写，因此需要[安装 Rust](https://www.rust-lang.org/) 才能编译。ripgrep 可使用 Rust 1.96.0（稳定版）或更高版本编译；通常 ripgrep 跟随最新稳定版 Rust 编译器。

构建 ripgrep：

```
$ git clone https://github.com/BurntSushi/ripgrep
$ cd ripgrep
$ cargo build --release
$ ./target/release/rg --version
0.1.3
```

**注意：**过去使用 Rust nightly 编译器时，ripgrep 支持 `simd-accel` Cargo 特性。这仅有利于 UTF-16 转码。由于它需要不稳定功能，这种构建方式容易损坏，因此已移除支持。如需对 UTF-16 转码进行 SIMD 优化，需要请求 [`encoding_rs`](https://github.com/hsivonen/encoding_rs) 项目使用稳定 API。

最后，可以通过启用 `pcre2` 特性，在构建 ripgrep 时加入可选的 PCRE2 支持：

```
$ cargo build --release --features 'pcre2'
```

启用 PCRE2 特性可使用稳定版 Rust 编译器，并会尝试通过 `pkg-config` 自动查找并链接系统的 PCRE2 库。如果没有该库，ripgrep 会使用系统 C 编译器从源代码构建 PCRE2，并将其静态链接到最终可执行文件。即使系统中有 PCRE2 库，也可以通过使用 MUSL 目标构建 ripgrep 或设置 `PCRE2_SYS_STATIC=1` 强制静态链接。

要在 Linux 上使用 MUSL 目标构建 ripgrep，首先在系统中安装 MUSL 库（请咨询你友好的本地包管理器）。然后只需为 Rust 工具链添加 MUSL 支持并重新构建 ripgrep，即可得到完全静态的可执行文件：

```
$ rustup target add x86_64-unknown-linux-musl
$ cargo build --release --target x86_64-unknown-linux-musl
```

上面的 `--features` 标志仍按预期工作。如果要使用 MUSL 和 PCRE2 构建静态可执行文件，则需要安装 `musl-gcc`；根据 Linux 发行版不同，它可能与实际 MUSL 库位于不同的软件包。

### 运行测试

ripgrep 经过相对充分的测试，包括单元测试和集成测试。要运行完整测试集，请在仓库根目录使用：

```
$ cargo test --all
```



### 相关工具

* [delta](https://github.com/dandavison/delta) 是支持 `rg --json` 输出格式的语法高亮分页程序。只需运行 `rg --json pattern | delta` 即可使用。详情参见 [delta 手册中的 grep 章节](https://dandavison.github.io/delta/grep.html)。

### 漏洞报告

如需报告安全漏洞，请[联系 Andrew Gallant](https://blog.burntsushi.net/about/)。如果希望发送加密消息，联系页面提供我的电子邮箱和 PGP 公钥。

<a id="translations"></a>
### 翻译

以下是已知的 ripgrep 文档翻译列表。这些翻译由非官方维护，可能不是最新的。

* [中文](https://github.com/chinanf-boy/ripgrep-zh#%E6%9B%B4%E6%96%B0-)
* [西班牙语](https://github.com/UltiRequiem/traducciones/tree/master/ripgrep)
