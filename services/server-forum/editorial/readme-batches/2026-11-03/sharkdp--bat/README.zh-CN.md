> 译注：本文基于固定提交的 [官方中文 README](https://github.com/sharkdp/bat/blob/67e2ec24e0284694c10d1e25166fc97aad15845e/doc/README-zh.md)，由 Terra 对照[英文原文](https://github.com/sharkdp/bat/blob/67e2ec24e0284694c10d1e25166fc97aad15845e/README.md)补译和修正。上游中文有 6 个失效相对链接，已改为固定提交中的正确目标；Funtoo、Void、Termux 安装内容为同提交中文独有，予以保留。代码示例按英文原文保留。项目可任选 [MIT](https://github.com/sharkdp/bat/blob/67e2ec24e0284694c10d1e25166fc97aad15845e/LICENSE-MIT) 或 [Apache-2.0](https://github.com/sharkdp/bat/blob/67e2ec24e0284694c10d1e25166fc97aad15845e/LICENSE-APACHE)。

<p align="center">
  <img src="https://raw.githubusercontent.com/sharkdp/bat/67e2ec24e0284694c10d1e25166fc97aad15845e/doc/logo-header.svg" alt="bat - a cat clone with wings"><br>
  <a href="https://github.com/sharkdp/bat/actions?query=workflow%3ACICD"><img src="https://github.com/sharkdp/bat/workflows/CICD/badge.svg" alt="Build Status"></a>
  <img src="https://img.shields.io/crates/l/bat.svg" alt="license">
  <a href="https://crates.io/crates/bat"><img src="https://img.shields.io/crates/v/bat.svg?colorB=319e8c" alt="Version info"></a><br>
  类似 <i>cat(1)</i>，但带有 git 集成和语法高亮.
</p>

<p align="center">
  <a href="#语法高亮">主要功能</a> •
  <a href="#如何使用">使用方法</a> •
  <a href="#安装">安装</a> •
  <a href="#自定义">自定义</a> •
  <a href="#项目目标和替代方案">项目目标和替代方案</a><br>
  [<a href="https://github.com/sharkdp/bat/blob/67e2ec24e0284694c10d1e25166fc97aad15845e/README.md">English</a>]
  [中文]
  [<a href="https://github.com/sharkdp/bat/blob/67e2ec24e0284694c10d1e25166fc97aad15845e/doc/README-ja.md">日本語</a>]
  [<a href="https://github.com/sharkdp/bat/blob/67e2ec24e0284694c10d1e25166fc97aad15845e/doc/README-ko.md">한국어</a>]
  [<a href="https://github.com/sharkdp/bat/blob/67e2ec24e0284694c10d1e25166fc97aad15845e/doc/README-ru.md">Русский</a>]
</p>

### 语法高亮

`bat` 对大部分编程语言和标记语言提供语法高亮：

![Syntax highlighting example](https://imgur.com/rGsdnDe.png)

### Git 集成

`bat` 能从 git 中获取文件的修改并展示在边栏（见下图）：

![Git integration example](https://i.imgur.com/2lSW4RE.png)

### 不可打印(non-printable)字符可视化

添加`-A`/`--show-all`参数可以文件文件中的不可打印字符:

![Non-printable character example](https://i.imgur.com/WndGp9H.png)

### 自动分页

`bat`会在一般情况下将大于屏幕可显示范围的内容输出到分页器(pager, e.g. `less`)。

你可以在命令行或配置文件中添加 `--paging=never` 参数来使`bat`不使用分页器（就像`cat`一样）。如果你想要用为`cat`使用`bat`别名，可以在 shell 配置文件（shell configuration）中添加`alias cat='bat --paging=never'`。

#### 文件连接

即使设置了分页器，仍可以使用 `bat` 连接文件 :wink:。当 `bat` 检测到非交互式终端（例如将输出通过管道传给其他进程或写入文件），它会作为 `cat` 的直接替代品，退回到打印纯文件内容，且不受 `--pager` 选项的值影响。

## 如何使用

在终端中查看一个文件

```bash
bat README.md
```

一次性展示多个文件

```bash
bat src/*.rs
```

从`stdin`读入流，自动为内容添加语法高亮（前提是输入内容的语言可以被正确识别，通常根据内容第一行的 shebang 标记，形如`#!/bin/sh`）

```bash
curl -s https://sh.rustup.rs | bat
```

显式指定`stdin`输入的语言

```bash
yaml2json .travis.yml | json_pp | bat -l json
```

显示不可打印字符

```bash
bat -A /etc/hosts
```

与`cat`的兼容性

```bash
bat > note.md  # quickly create a new file

bat header.md content.md footer.md > document.md

bat -n main.rs  # show line numbers (only)

bat f - g  # output 'f', then stdin, then 'g'.
```

### 第三方工具交互

#### `fzf`

你可以使用 `bat` 作为 [`fzf`](https://github.com/junegunn/fzf) 的预览器。使用 `--color=always` 强制彩色输出，也可用 `--line-range` 缩短读取长文件所需的时间。

```bash
fzf --preview "bat --color=always --style=numbers --line-range=:500 {}"
```

更多信息请参阅[`fzf`的说明](https://github.com/junegunn/fzf#preview-window)。

#### `find` 或 `fd`

你可以使用`find`的`-exec`选项来用`bat`预览搜索结果：

```bash
find … -exec bat {} +
```

亦或者在使用 [`fd`](https://github.com/sharkdp/fd) 时添加`-X`/`--exec-batch`选项：

```bash
fd … -X bat
```

#### `ripgrep`

`bat` 可以使用 [`batgrep`](https://github.com/eth-p/bat-extras/blob/master/doc/batgrep.md) 显示 [`ripgrep`](https://github.com/BurntSushi/ripgrep) 的搜索结果。

```bash
batgrep needle src/
```

#### `tail -f`

当与`tail -f`一起使用，`bat`可以持续监视文件内容并为其添加语法高亮。

```bash
tail -f /var/log/pacman.log | bat --paging=never -l log
```

注意：这项功能需要在关闭分页时使用，同时要手动指定输入的内容语法（通过`-l log`）。

#### `git`

可以结合 `bat` 和 `git show`，以正确的语法高亮查看文件的旧版本：

```bash
git show v0.6.0:src/main.rs | bat -l rs
```

#### `git diff`

可以结合 `bat` 和 `git diff`，以正确的语法高亮查看代码变更附近的行：

```bash
batdiff() {
    git diff --name-only --relative --diff-filter=d -z | xargs -0 bat --diff
}
```

该功能也作为一个独立工具提供，你可以在[`bat-extras`](https://github.com/eth-p/bat-extras)中找到`batdiff`。

如果需要更多 Git 和 diff 操作支持，请参阅 [`delta`](https://github.com/dandavison/delta)。

#### `xclip`

行号和 Git 修改标记会妨碍复制文件内容。可以用 `-p`／`--plain` 调用 `bat`，也可以直接将输出通过管道传给 `xclip`：

```bash
bat main.cpp | xclip
```

`bat`会检测输出是否是管道重定向来决定是否使用纯文本输出。

#### `man`

通过设置 `MANPAGER` 环境变量，可以将 `bat` 用作 `man` 的彩色分页器：

```bash
export MANPAGER="bat -plman"
man 2 select
```

（在某些较旧的 Debian 或 Ubuntu 版本上，可执行文件名为 `batcat` 而非 `bat`。）

如果希望将其封装为新命令，也可以使用 [`batman`](https://github.com/eth-p/bat-extras/blob/master/doc/batman.md)。

注意，[Manpage 语法](https://github.com/sharkdp/bat/blob/67e2ec24e0284694c10d1e25166fc97aad15845e/assets/syntaxes/02_Extra/Manpage.sublime-syntax)在本仓库开发，仍需改进。

> 译注：同一提交的官方中文候选保留了旧版提示：Mandoc 的 `man` 实现可能[无法直接使用该方案](https://github.com/sharkdp/bat/issues/1145)；可使用 `batman` 或将 shell 脚本包装为 shebang 可执行文件再让 `MANPAGER` 指向它。英文版当前未列此提示，适用性待核实。

#### `prettier` / `shfmt` / `rustfmt`

[`prettybat`](https://github.com/eth-p/bat-extras/blob/master/doc/prettybat.md) 脚本是一个包装器，可格式化代码并用 `bat` 输出。

#### 高亮 `--help` 消息

可以使用 `bat` 为帮助文本着色：`$ cp --help | bat -plhelp`

也可以使用包装函数：

```bash
# in your .bashrc/.zshrc/*rc
alias bathelp='bat --plain --language=help'
help() {
    "$@" --help 2>&1 | bathelp
}
```

然后可以使用 `$ help cp` 或 `$ help git commit`。

使用 `zsh` 时，也可以通过全局别名完全覆盖 `-h` 和 `--help`：

```bash
alias -g -- -h='-h 2>&1 | bat --language=help --style=plain'
alias -g -- --help='--help 2>&1 | bat --language=help --style=plain'
```

对于 `fish`，可以使用缩写：

```fish
abbr -a --position anywhere -- --help '--help | bat -plhelp'
abbr -a --position anywhere -- -h '-h | bat -plhelp'
```

这样仍可以使用 `cp --help`，但能看到彩色帮助页面。

> [!TIP]
> 日后要删除这些缩写，可运行：
> ```fish
> abbr -e -- --help
> abbr -e -- -h
> ```
> 缩写名之前必须有 `--`，因为 `--help` 和 `-h` 以短横线开头，否则会被解释为 `abbr` 自身的选项。

请注意，有时 `-h` 并不是 `--help` 的缩写（例如 `ls`）。需要将 `-h` 作为命令参数时，可在参数前加上 `\`（例如 `ls \-h`）以避开上述别名。

请在[此仓库](https://github.com/victor-gp/cmd-help-sublime-syntax)报告帮助语法相关问题。

## 安装

[![Packaging status](https://repology.org/badge/vertical-allrepos/bat-cat.svg?columns=3&exclude_unsupported=1)](https://repology.org/project/bat-cat/versions)

### Ubuntu (使用 `apt`)

*... 以及其他基于 Debian的发行版.*

`bat` 要求的版本： [Ubuntu 自 20.04 起 ("Focal")](https://packages.ubuntu.com/search?keywords=bat&exact=1) 和 [Debian 自 2021 年 8 月起 (Debian 11 - "Bullseye")](https://packages.debian.org/bullseye/bat).

当你的发行版满足条件那么直接在终端运执行：

```bash
sudo apt install bat
```

重要：在较旧的 Ubuntu 或 Debian 版本上可执行文件可能名为 `batcat`，较新版本通常为 `bat`。安装后如果 `bat --version` 不可用，请尝试 `batcat --version`。如果你通过这种方法安装 `bat`，请留意你所安装的可执行文件是否为 `batcat`（由[其他包的可执行文件名冲突](https://github.com/sharkdp/bat/issues/982)造成）。你可以创建一个`bat -> batcat`的符号链接(symlink)或别名来避免因为可执行文件不同带来的问题并与其他发行版保持一致性。

``` bash
mkdir -p ~/.local/bin
ln -s /usr/bin/batcat ~/.local/bin/bat
```

或者设置别名：

```bash
alias bat="batcat"
```

### Ubuntu (使用`.deb`包)

*... 以及其他基于 Debian的发行版.*

如果你无法使用上一种方法安装，或需要用最新版的`bat`，你可以从[release 页面](https://github.com/sharkdp/bat/releases)下载最新的`.deb`包并通过下述方法安装：

```bash
sudo dpkg -i bat_0.18.3_amd64.deb  # adapt version number and architecture
```

### Alpine Linux

你可以用下面下列命令从官方源中安装[`bat 包`](https://pkgs.alpinelinux.org/packages?name=bat)：

```bash
apk add bat
```

### Arch Linux

你可以用下面下列命令从官方源中安装[`bat`包](https://www.archlinux.org/packages/extra/x86_64/bat/)：

```bash
pacman -S bat
```

### Fedora

你可以使用下列命令从官方仓库安装 [`bat` 包](https://packages.fedoraproject.org/pkgs/rust-bat/bat/)：

```bash
dnf install bat
```

### Funtoo Linux

你可以从 dev-kit 中安装[`bat` 包](https://github.com/funtoo/dev-kit/tree/1.4-release/sys-apps/bat)：

```bash
emerge sys-apps/bat
```

### Gentoo Linux

你可以使用下列命令从官方源中安装 [`bat` 包](https://packages.gentoo.org/packages/sys-apps/bat)：


```bash
emerge sys-apps/bat
```

### Void Linux

你可以用 xbps-install 安装`bat`：

```bash
xbps-install -S bat
```

### Termux:

你可以用 pkg 安装`bat：

```bash
pkg install bat
```

### FreeBSD

你可以用 pkg 来安装一份预编译的[`bat` 包](https://www.freshports.org/textproc/bat)：

```bash
pkg install bat
```

或从 FreeBSD ports 自己编译一份：

```bash
cd /usr/ports/textproc/bat
make install
```

### OpenBSD

你可以用 [`pkg_add(1)`](https://man.openbsd.org/pkg_add.1)安装`bat`包

```bash
pkg_add bat
```

### 通过 nix

你可以用[nix 包管理器](https://nixos.org/nix)安装`bat`：

```bash
nix-env -i bat
```

### openSUSE

你可以用 zypper 安装`bat`：

```bash
zypper install bat
```

### 通过 snap

目前还没有推荐的 snap 包可用。可以使用其他现存的包但不会受到官方支持且可能会遇到[问题](https://github.com/sharkdp/bat/issues/1519)。

### macOS (或 Linux) 通过 Homebrew

你可以用 [Homebrew on MacOS](https://formulae.brew.sh/formula/bat) 或者 [Homebrew on Linux](https://formulae.brew.sh/formula-linux/bat) 安装`bat`：

```bash
brew install bat
```

### macOS 通过 MacPorts

或用 [MacPorts](https://ports.macports.org/port/bat/summary) 安装`bat`：

```bash
port install bat
```

### Windows

在 Windows 上具有多种安装`bat`的方法。若你已完成安装，记得看看 ["在 Windows 上使用`bat`"](#在-windows-中使用-bat) 。

#### 前置条件

你必须已安装 [Visual C++ Redistributable](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist) 包。

#### 使用 WinGet

可以使用 [WinGet](https://learn.microsoft.com/en-us/windows/package-manager/winget) 安装 `bat`：

```bash
winget install sharkdp.bat
```

#### 使用 Chocolatey

你可以用[Chocolatey](https://chocolatey.org/packages/Bat) 安装`bat`：

```bash
choco install bat
```

#### 使用 Scoop

你可以用 [scoop](https://scoop.sh/) 安装`bat`：

```bash
scoop install bat
```

#### 使用预编译二进制版本

直接从 [Release 发布页](https://github.com/sharkdp/bat/releases) 下载已经编译好的二进制包，前提是你安装了 [Visual C++ Redistributable](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist) 包。

### 使用二进制版本

在 [Release 发布页](https://github.com/sharkdp/bat/releases) 中可以找到为多种架构构建的`bat`版本和静态编译的二进制文件（文件名带有`musl`）。

### 从源码编译

如果想从源码构建 `bat`，需要 Rust 1.79.0 或更新版本，然后可用 `cargo` 构建：

#### 从本地源码

```bash
cargo install --path . --locked
```

> [!NOTE]
> 上面的 `--path .` 指定源码目录，并非 `bat` 的安装位置。详情参阅 [`cargo install` 文档](https://doc.rust-lang.org/cargo/commands/cargo-install.html)。

#### 从 `crates.io`

```bash
cargo install --locked bat
```

这两种方式均不能自动安装 man 页面或 shell 补全等附加文件。从本地源码安装时，`cargo` 会生成这些文件，它们应位于 cargo 目标目录下的 `build` 中。

还可以运行以下命令获取 shell 补全：

```bash
bat --completion <shell>
# see --help for supported shells
```

## 自定义

### 语法高亮主题

使用 `bat --list-themes` 列出所有可用的语法高亮主题。默认情况下，`bat` 对深色和浅色分别使用 `Monokai Extended` 和 `Monokai Extended Light`。如需选择 `TwoDark`，可使用 `--theme=TwoDark` 或将环境变量 `BAT_THEME` 设为 `TwoDark`。在 shell 启动文件中使用 `export BAT_THEME="TwoDark"` 可永久设置；也可使用 [配置文件](#配置文件)。

如果想在自定义文件上预览不同主题，可运行以下命令（需要 [`fzf`](https://github.com/junegunn/fzf)）：

```bash
bat --list-themes | fzf --preview="bat --theme={} --color=always /path/to/file"
```

`bat` 根据终端背景色自动选择合适主题。可通过 `--theme-dark` / `--theme-light` 或 `BAT_THEME_DARK` / `BAT_THEME_LIGHT` 自定义，在经常切换深浅模式时尤其有用。也可以按照下方“添加主题”章节使用自定义主题。

### 8-bit 主题

即使终端支持真彩色，`bat` 的三个主题仍使用 [8-bit 颜色](https://en.wikipedia.org/wiki/ANSI_escape_code#Colors)：

- `ansi` 在任何终端上效果都不错。它使用 3-bit 颜色：黑、红、绿、黄、蓝、品红、青和白。
- `base16` 为 [base16](https://github.com/tinted-theming/home) 终端主题设计，按 [base16 样式指南](https://github.com/tinted-theming/home/blob/main/styling.md)使用 4-bit 颜色（3-bit 颜色及其亮色版本）。
- `base16-256` 为 [tinted-shell](https://github.com/tinted-theming/tinted-shell) 设计，用编号 16 至 21 的 8-bit 颜色替换某些亮色。如果只是有 256 色终端但没有使用 tinted-shell，**不要**因此使用此主题。

虽然这些主题限制更多，它们相较真彩色主题有三个优点：兼容性最好（某些终端工具不支持超过 3-bit 的颜色）；能适应终端主题变化，包括已打印的输出；与其他终端软件在视觉上更协调。

### 输出样式

使用 `--style` 控制 `bat` 输出外观。例如 `--style=numbers,changes` 只显示 Git 修改和行号，不显示网格和文件标题。可设置 `BAT_STYLE` 环境变量使更改永久生效，也可使用[配置文件](#配置文件)。

默认启用 `changes`、`grid`、`header-filename`、`numbers` 和 `snip`。

| 预定义样式 | 说明 |
|-------|-------------|
| `default` | 启用上述推荐组件。 |
| `full` | 启用全部可用组件。 |
| `auto` | 与 `default` 相同，输出通过管道传递时除外。 |
| `plain` | 禁用全部可用组件。 |

| 组件 | 说明 |
|-----------|-------------|
| `changes` | 显示 Git 修改标记。 |
| `header` | `header-filename` 的别名。 |
| `header-filename` | 在内容之前显示文件名。 |
| `header-filesize` | 在内容之前显示文件大小。 |
| `grid` | 使用垂直／水平线分隔侧栏、标题与内容。 |
| `rule` | 使用水平线分隔文件。 |
| `numbers` | 在侧栏显示行号。 |
| `snip` | 在不同的行范围之间绘制分隔线。 |

> [!TIP]
> 如果在配置文件指定了默认样式，可在单次运行时用 `--style` 修改显示的组件。在组件前加 `+` 或 `-` 可将其加入当前样式或从中移除。
>
> 例如，配置中有 `--style=full,-snip` 时，可运行 `--style=-grid,+snip` 移除网格并重新添加 `snip`。如要完全覆盖样式，可使用 `--style=numbers` 只显示行号。

### 装饰元素

默认情况下，`bat` 仅在输出到交互式终端时显示行号、文件标题、网格边框等装饰。可以使用 `--decorations` 选项控制此行为。`--decorations=always` 即使将输出通过管道传给其他命令也显示装饰，`--decorations=never` 则完全禁用。可选值为 `auto`（默认）、`never` 和 `always`。

还有 `--force-colorization` 选项，它是 `--decorations=always --color=always` 的别名，适合将 `bat` 的输出通过管道传给其他程序时保留颜色和装饰。

### 添加新的语言和语法

当现有的`bat`不支持某个语言或语法时你可以自己添加。

`bat`使用 [`syntect`](https://github.com/trishume/syntect/) 库来支持语法高亮，该库使用 [Sublime Text `.sublime-syntax` 语法文件](https://www.sublimetext.com/docs/3/syntax.html)和主题。而后者中的大部分可以在 [Package Control](https://packagecontrol.io/) 找到。

当你找到一份语法文件，按照下列方法：

1. 创建包含语法描述文件的目录：
   
   ```bash
   mkdir -p "$(bat --config-dir)/syntaxes"
   cd "$(bat --config-dir)/syntaxes"

   # Put new '.sublime-syntax' language definition files
   # in this folder (or its subdirectories), for example:
   git clone https://github.com/tellnobody1/sublime-purescript-syntax
   ```

2. 调用下面指令把文件转换为二进制缓存：
   
   ```bash
   bat cache --build
   ```

3. 最后用`bat --list-languages`来检查新的语法是否被成功导入。如果想要回滚到最初状态，执行：
   
   ```bash
   bat cache --clear
   ```

4. 如果你觉得`bat`有必要自带该语法支持，请在阅读[指导](https://github.com/sharkdp/bat/blob/67e2ec24e0284694c10d1e25166fc97aad15845e/doc/assets.md)后向仓库提交 [Syntax Request](https://github.com/sharkdp/bat/issues/new?labels=syntax-request&template=syntax_request.md)。

### 添加主题

> [!NOTE]
> 自定义主题必须使用 [`.tmTheme` 文件](https://www.sublimetext.com/docs/color_schemes_tmtheme.html)，目前不支持较新的 `.sublime-color-scheme` 文件。

类似添加语法支持，第一步也是创建一个带有语法高亮的目录

```bash
mkdir -p "$(bat --config-dir)/themes"
cd "$(bat --config-dir)/themes"

# Download a theme in '.tmTheme' format, for example:
git clone https://github.com/greggb/sublime-snazzy

# Update the binary cache
bat cache --build
```

然后用`bat --list-themes`检查添加是否成功。

### 添加或修改文件关联

你可以用`--map-syntax`参数添加或修改文件名模板。它需要一个类似`pattern:syntax`的参数来指定，其中`pattern`是 glob 文件匹配模板，`syntax`则是支持的语法的完整名（使用`bat --list-languages`来查看获取一份清单）。

注意：方便起见，你可能需要把参数添加到配置文件，而不是每次都在命令行中传递该参数。

以下展示了把“INI”关联到具有`.conf`扩展名的文件

```bash
--map-syntax='*.conf:INI'
```

把`.ignore`文件与“Git Ignore”关联

```bash
--map-syntax='.ignore:Git Ignore'
```

把`/etc/apache2`内的`.conf`文件关联到“Apache Conf”语法（`bat`已默认绑定）

```bash
--map-syntax='/etc/apache2/**/*.conf:Apache Conf'
```

### 使用自定义分页器

`bat` 使用 `PAGER` 环境变量指定的分页器；未设置时默认使用 `less`。也可用 `--pager=builtin` 或将 `BAT_PAGER` 设置为 `builtin` 使用 bat 内置分页器。

如要使用其他分页器，可修改 `PAGER`，或设置 `BAT_PAGER` 覆盖 `PAGER` 的值。

> [!NOTE]
> 如果 `PAGER` 为 `more` 或 `most`，`bat` 会静默改用 `less` 以确保颜色支持。

要向分页器传递命令行参数，也可以通过 `PAGER`／`BAT_PAGER` 设置：

```bash
export BAT_PAGER="less -RFK"
```

除了环境变量，也可以使用 `bat` 的[配置文件](#配置文件)配置分页器（`--pager` 选项）。

### 使用 `less` 作为分页器

使用 `less` 作为分页器时，`bat` 会自动向 `less` 传递额外选项，改善使用体验。具体包括 `-R`/`--RAW-CONTROL-CHARS`、`-F`/`--quit-if-one-screen`、`-K`/`--quit-on-intr`，并在某些条件下添加 `-X`/`--no-init` 和／或 `-S`/`--chop-long-lines`。

> [!IMPORTANT]
> 以下情况不会添加这些选项：
> - 分页器名称不是 `less`。
> - `--pager` 参数包含任何命令行参数（例如 `--pager="less -R"`）。
> - `BAT_PAGER` 环境变量包含任何命令行参数（例如 `export BAT_PAGER="less -R"`）。
>
> 以下情况不会添加 `--quit-if-one-screen`：
> - 使用 `--paging=always` 参数。
> - `BAT_PAGING` 环境变量设为 `always`。

`-R`/`--RAW-CONTROL-CHARS` 用于正确解释 ANSI 颜色。

`-F`/`--quit-if-one-screen` 指示 `less` 在输出高度小于终端高度时立即退出。对小文件来说很方便，因为不必按 `q` 退出分页器。

`-K`/`--quit-on-intr` 指示 `less` 收到中断信号后立即退出，以确保收到 SIGINT 时 `less` 和 `bat` 一起退出。

对低于 530 的 `less` 版本（Windows 上低于 558），会添加 `-X`/`--no-init` 来修复 `-F`/`--quit-if-one-screen` 功能的缺陷。但这也会导致 `less` 的鼠标滚轮不可用。如果想在旧版 `less` 上启用滚轮且不介意失去单屏即退出功能，可以通过 `--pager` 或 `BAT_PAGER` 将分页器设为 `less -R`。对于 `less` 530 或更新版本，开箱即用。

使用 `bat` 的 `-S`/`--chop-long-lines` 选项时会添加 `-S`/`--chop-long-lines`，告诉 `less` 截断超过终端宽度的行。

### 缩进

`bat` 使用四个空格宽的制表符，而不受分页器影响，同时也可以用`--tabs`参数来自定义。

注意：通过其他方法针对分页器的制表符设置不会生效（例如通过`bat`的`--pager`参数传递或`less`使用的`LESS`环境变量）。因为在输出提交给分页器之前，内容中的制表符就已经被`bat`替换为了特定长度的空格以避免由于边栏导致的缩进问题。你可以用给`bat`传递`--tabs=0`参数来取消该设定并让分页器自己处理制表符。

### 暗色模式

如果使用 **macOS** 的暗色模式，可以配置 `bat` 根据操作系统主题使用不同主题。下面的代码在暗色模式使用 `default`，在亮色模式使用 `GitHub`。

```bash
alias cat="bat --theme auto:system --theme-dark default --theme-light GitHub"
```

**GNOME** 也提供暗色模式，它影响 `org.gnome.desktop.interface color-scheme` 设置。下面的代码使用该设置实现相同功能。

```bash
# .bashrc
sys_color_scheme_is_dark() {
    condition=$(gsettings get org.gnome.desktop.interface color-scheme)
    condition=$(echo "$condition" | tr -d "[:space:]'")
    if [ $condition == "prefer-dark" ]; then
        return 0
    else
        return 1
    fi
}

bat_alias_wrapper() {
    #get color scheme
    sys_color_scheme_is_dark
    if [[ $? -eq 0 ]]; then
        # bat command with dark color scheme
        bat --theme=default "$@"
    else
        # bat command with light color scheme
        bat --theme=GitHub "$@"
    fi
}
alias cat='bat_alias_wrapper'
```

## 配置文件

`bat` 也可以通过配置文件自定义。文件位置取决于操作系统，运行以下命令可查看本机的默认路径：

```bash
bat --config-file
```

也可以用 `BAT_CONFIG_PATH` 或 `BAT_CONFIG_DIR` 环境变量分别指定非默认的配置文件或配置目录：

```bash
export BAT_CONFIG_PATH="/path/to/bat/bat.conf"
export BAT_CONFIG_DIR="/path/to/bat"
```

使用`--generate-config-file`参数调用`bat`会在指定位置生成一份默认的`bat`配置文件：

```bash
bat --generate-config-file
```

系统级配置文件位于 Linux 和 macOS 的 `/etc/bat/config`，以及 Windows 的 `C:\ProgramData\bat\config`。存在系统配置文件时，用户配置内容会追加在其后。

### 格式

配置文件其实是一份按行分割的命令行参数列表。你可以用`bat --help`来查看所有可用的参数和适用的值。配置文件中`#`打头的行会被视为注释而不生效。

以下是一份示例：

```bash
# Set the theme to "TwoDark"
--theme="TwoDark"

# Show line numbers, Git modifications and file header (but no grid)
--style="numbers,changes,header"

# Use italic text on the terminal (not supported on all terminals)
--italic-text=always

# Use C++ syntax for Arduino .ino files
--map-syntax "*.ino:C++"
```

## 在 Windows 中使用 `bat`

`bat` 在 Windows 上开箱即用，除了部分功能需要额外配置。

### 前置条件

你需要先安装 [Visual C++ Redistributable](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist) 包。

### 分页

Windows 只有一个提供有限功能的分页器，你可以从[这里下载](http://www.greenwoodsoftware.com/less/download.html)或用 [Chocolatey 安装](https://chocolatey.org/packages/Less) Windows 版本的`less`。第一种方法需要你把它所在目录加入`PATH`环境变量或[定义分页器变量](#使用自定义分页器)

### 色彩

Windows 10 从 [v1511](https://en.wikipedia.org/wiki/Windows_10_version_history#Version_1511_(November_Update)) 开始，`conhost.exe`（命令提示符）和 PowerShell 原生支持颜色，较新版本的 bash 也支持。在更早的 Windows 版本上，可以使用包含 [ConEmu](https://conemu.github.io/) 的 [Cmder](http://cmder.app/)。

**注意：** 旧版 `less` 无法在 Windows 上正确解释颜色。为解决此问题，可在安装 Git 时将可选的 Unix 工具加入 PATH。如果没有安装其他分页器，可通过 `--paging=never` 或将 `BAT_PAGER` 设为空字符串完全禁用分页。

### Cygwin

Windows 上的`bat`原生不支持 Cygwin' unix-style 路径（`/cygdrive/*`）。当传递一个绝对 cygwin 路径作为参数值时，`bat`会产生`The system cannot find the path specified. (os error 3)`的错误。你可以`.bash_profile`文件中添加以下函数来解决这个问题。

```bash
bat() {
    local index
    local args=("$@")
    for index in $(seq 0 ${#args[@]}) ; do
        case "${args[index]}" in
        -*) continue;;
        *)  [ -e "${args[index]}" ] && args[index]="$(cygpath --windows "${args[index]}")";;
        esac
    done
    command bat "${args[@]}"
}
```

## 疑难解答

### 输出内容含糊不清

如果输入文件包含颜色代码、其他 ANSI 转义序列或控制字符，`bat` 的语法高亮和文本换行可能出现问题，导致输出乱码。

如果你的 `bat` 版本支持 `--strip-ansi=auto`，可使用它在语法高亮前移除这些序列。也可以向 `bat` 传递 `--color=never --wrap=never`，同时禁用语法高亮和换行。

对于不可信输入，`--sanitize=auto|always|never` 还会将可影响终端的控制字节和 Unicode 双向／零宽格式字符替换为 Unicode 替换字符。它同时以相同值启用 `--strip-ansi`。

> [!NOTE]
> 当语法为纯文本时，`--strip-ansi` 的 `auto` 选项不会移除转义序列。

### 终端与色彩

`bat`会区分支持 truecolor 和不支持 truecolor 的终端。但是大部分语法高亮主题都是用了没有为 8-bit 色彩支持的颜色，因此强烈推荐使用一个支持 24-bit 色彩的终端（`terminator`，`konsole`，`iTerm2`...），或使用一个 [8-bit 主题](#8-bit-主题)来限制一些颜色。查看[这篇文章](https://gist.github.com/XVilka/8346728)了解更多支持 truecolor 的终端。你需要定义`COLORTERM`变量为`truecolor`或`24bit`来确保`bat`能够识别终端的对颜色的支持，否则会使用 8 bit 模式。

### 行号和边框很难看清

可以尝试其他主题（用 `bat --list-themes` 列出），`OneHalfDark` 和 `OneHalfLight` 的网格与行号颜色较亮。

### 文件编码

`bat`原生支持 UTF-8 和 UTF-16。其他编码通常无法自动检测，因此可能需要先转换为 UTF-8。

这里展示了使用`iconv`来把 Latin-1(ISO-8859-1) 编码的 PHP 文件转换到 UTF-8：

``` bash
iconv -f ISO-8859-1 -t UTF-8 my-file.php | bat
```

注意： 当`bat`无法识别语言时你可能会需要`-l`/`--language`参数。

## 开发

```bash
# Recursive clone to retrieve all submodules
git clone --recursive https://github.com/sharkdp/bat

# Build (debug version)
cd bat
cargo build --bins

# Run unit tests and integration tests
cargo test

# Install (release version)
cargo install --path . --locked

# Build a bat binary with modified syntaxes and themes
bash assets/create.sh
cargo install --path . --locked --force
```

如果你想构建一个使用 `bat` 美化打印功能的应用程序，请查看 [API 文档](https://docs.rs/bat/)。请注意，当你依赖 `bat` 作为库时，必须使用 `regex-onig` 或 `regex-fancy` 作为特性。

## 贡献指南

请查看 [`CONTRIBUTING.md`](https://github.com/sharkdp/bat/blob/67e2ec24e0284694c10d1e25166fc97aad15845e/CONTRIBUTING.md) 指南。

## 维护者

- [sharkdp](https://github.com/sharkdp)
- [eth-p](https://github.com/eth-p)
- [keith-hall](https://github.com/keith-hall)
- [Enselic](https://github.com/Enselic)

## 安全漏洞

请参阅 [SECURITY.md](https://github.com/sharkdp/bat/blob/67e2ec24e0284694c10d1e25166fc97aad15845e/SECURITY.md)。

如果你想报告 `bat` 中的漏洞，请通过邮件联系 [David Peter](https://david-peter.de/)。

## 项目目标和替代方案

`bat` 试图实现以下目标：

- 提供美观的高级语法高亮
- 与 Git 集成以显示文件修改
- 成为 (POSIX) `cat` 的替代品
- 提供用户友好的命令行界面

如果你在寻找类似的程序，有很多替代方案。请参阅[本文档](https://github.com/sharkdp/bat/blob/67e2ec24e0284694c10d1e25166fc97aad15845e/doc/alternatives.md)进行比较。

## 许可证

版权所有 (c) 2018-2025 [bat-developers](https://github.com/sharkdp/bat)。

`bat` 可根据 MIT 许可证或 Apache 许可证 2.0 的条款使用，任选其一。

有关许可证的详细信息，请参阅 [LICENSE-APACHE](https://github.com/sharkdp/bat/blob/67e2ec24e0284694c10d1e25166fc97aad15845e/LICENSE-APACHE) 和 [LICENSE-MIT](https://github.com/sharkdp/bat/blob/67e2ec24e0284694c10d1e25166fc97aad15845e/LICENSE-MIT) 文件。
