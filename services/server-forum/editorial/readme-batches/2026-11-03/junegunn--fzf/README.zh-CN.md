> 根据 [fzf 固定版本 README](https://github.com/junegunn/fzf/blob/99bfd5aa0f66133d7dcfbf18f8044eb765f04649/README.md) 翻译。代码中的自然语言注释已翻译，命令及参数保留；性能表述为上游声明，未独立测量。

<div align="center">
  <img src="https://raw.githubusercontent.com/junegunn/i/master/fzf-color.png" alt="fzf - a command-line fuzzy finder">
  <a href="https://github.com/junegunn/fzf/actions"><img src="https://github.com/junegunn/fzf/actions/workflows/linux.yml/badge.svg?branch=master" alt="Build Status"></a>
  <a href="http://github.com/junegunn/fzf/releases"><img src="https://img.shields.io/github/v/tag/junegunn/fzf" alt="Version"></a>
  <a href="https://github.com/junegunn/fzf?tab=MIT-1-ov-file#readme"><img src="https://img.shields.io/github/license/junegunn/fzf" alt="License"></a>
  <a href="https://github.com/junegunn/fzf/graphs/contributors"><img src="https://img.shields.io/github/contributors/junegunn/fzf" alt="Contributors"></a>
  <a href="https://github.com/sponsors/junegunn"><img src="https://img.shields.io/github/sponsors/junegunn" alt="Sponsors"></a>
  <a href="https://github.com/junegunn/fzf/stargazers"><img src="https://img.shields.io/github/stars/junegunn/fzf?style=flat" alt="Stars"></a>
</div>

---

<kbd align="center">
  <br/>
  <a href="https://commitgoods.com/collections/fzf?utm_source=github&amp;utm_medium=referral"><img src="https://junegunn.github.io/fzf/images/fzf-mugs.jpg" width="80%" alt="fzf merch"></a>
  <br/>
  <br/>
  表达你对 fzf 的喜爱——T 恤、马克杯和贴纸现已推出！
  <br/>
  <br/>
  <a href="https://commitgoods.com/collections/fzf?utm_source=github&amp;utm_medium=referral"><picture><source media="(prefers-color-scheme: dark)" srcset="https://cdn.shopify.com/s/files/1/0101/8752/files/commitgoods-badge-stacked-navy-dark.svg"><img src="https://cdn.shopify.com/s/files/1/0101/8752/files/commitgoods-badge-stacked-navy.svg" height="28" alt="Official merch on commitgoods.com"></picture></a>
  <br/>
  <br/>
</kbd>

---

fzf 是通用命令行模糊查找器和交互式终端工具包。

<img src="https://raw.githubusercontent.com/junegunn/i/master/fzf-preview.png" width=640>

无论是选择文件、浏览命令历史、预览数据、通过模糊匹配浏览复杂数据集，还是创建自定义菜单和工作流程，fzf 都提供基础构件，将 shell 脚本变成功能丰富的终端应用。

特点
----------

- **便携** // 以单个二进制文件分发，便于安装
- **快速** // 经过优化，可在毫秒内处理数百万条目
- **可编程** // 事件驱动架构，可构建自定义终端界面和工作流程
- **开箱即用** // 提供 Bash、Zsh、Fish、Nushell、Vim 和 Neovim 集成

目录
-----------------

<!-- vim-markdown-toc GFM -->

* [安装](#installation)
    * [使用 Homebrew](#using-homebrew)
    * [使用 Mise](#using-mise)
    * [Linux 软件包](#linux-packages)
    * [Windows 软件包](#windows-packages)
    * [使用 git](#using-git)
    * [二进制发行版](#binary-releases)
    * [设置 shell 集成](#setting-up-shell-integration)
    * [Vim/Neovim 插件](#vimneovim-plugin)
* [升级 fzf](#upgrading-fzf)
* [构建 fzf](#building-fzf)
* [用法](#usage)
    * [使用查找器](#using-the-finder)
    * [显示模式](#display-modes)
        * [`--height` 模式](#--height-mode)
        * [`--popup` 模式](#--popup-mode)
    * [搜索语法](#search-syntax)
    * [环境变量](#environment-variables)
    * [自定义外观](#customizing-the-look)
    * [选项](#options)
    * [演示](#demo)
* [示例](#examples)
* [命令行快捷键](#key-bindings-for-command-line)
* [模糊补全](#fuzzy-completion)
    * [文件和目录](#files-and-directories)
    * [进程 ID](#process-ids)
    * [主机名](#host-names)
    * [环境变量 / 别名](#environment-variables--aliases)
    * [自定义 bash 和 zsh 的模糊补全](#customizing-fuzzy-completion-for-bash-and-zsh)
        * [自定义补全的 fzf 选项](#customizing-fzf-options-for-completion)
        * [自定义路径和目录的补全来源](#customizing-completion-source-for-paths-and-directories)
        * [支持的命令（bash）](#supported-commands-bash)
        * [自定义模糊补全](#custom-fuzzy-completion)
    * [fish 的模糊补全](#fuzzy-completion-for-fish)
    * [Nushell 的模糊补全](#fuzzy-completion-for-nushell)
* [Vim 插件](#vim-plugin)
* [高级主题](#advanced-topics)
    * [为不同类型输入自定义](#customizing-for-different-types-of-input)
    * [性能](#performance)
    * [执行外部程序](#executing-external-programs)
    * [变成另一个进程](#turning-into-a-different-process)
    * [重新加载候选列表](#reloading-the-candidate-list)
        * [1. 按 CTRL-R 更新进程列表](#1-update-the-list-of-processes-by-pressing-ctrl-r)
        * [2. 按 CTRL-D 或 CTRL-F 切换数据源](#2-switch-between-sources-by-pressing-ctrl-d-or-ctrl-f)
        * [3. 交互式 ripgrep 集成](#3-interactive-ripgrep-integration)
    * [预览窗口](#preview-window)
    * [预览图像](#previewing-an-image)
* [技巧](#tips)
    * [遵循 `.gitignore`](#respecting-gitignore)
    * [Fish shell](#fish-shell)
    * [fzf Theme Playground](#fzf-theme-playground)
* [相关项目](#related-projects)
* [许可证](#license)
* [周边](#goods)
* [赞助者 :heart:](#sponsors-heart)

<!-- vim-markdown-toc -->

<a id="installation"></a>
安装
------------

<a id="using-homebrew"></a>
### 使用 Homebrew

可以使用 [Homebrew](https://brew.sh/) 在 macOS 或 Linux 上安装 fzf。

```sh
brew install fzf
```

> [!IMPORTANT]
> 设置 shell 集成（快捷键和模糊补全）请参阅[以下说明](#setting-up-shell-integration)。

fzf 也可[通过 MacPorts][portfile] 安装：`sudo port install fzf`

[portfile]: https://github.com/macports/macports-ports/blob/master/sysutils/fzf/Portfile

<a id="using-mise"></a>
### 使用 Mise

可以使用 [mise](https://github.com/jdx/mise) 安装 fzf。

```sh
mise use -g fzf@latest
```

<a id="linux-packages"></a>
### Linux 软件包

| 包管理器 | Linux 发行版 | 命令 |
| --------------- | ----------------------- | ---------------------------------- |
| APK             | Alpine Linux            | `sudo apk add fzf`                 |
| APT             | Debian 9+/Ubuntu 19.10+ | `sudo apt install fzf`             |
| Conda           |                         | `conda install -c conda-forge fzf` |
| DNF             | Fedora                  | `sudo dnf install fzf`             |
| Nix             | NixOS, etc.             | `nix-env -iA nixpkgs.fzf`          |
| Pacman          | Arch Linux              | `sudo pacman -S fzf`               |
| pkg             | FreeBSD                 | `pkg install fzf`                  |
| pkgin           | NetBSD                  | `pkgin install fzf`                |
| pkg_add         | OpenBSD                 | `pkg_add fzf`                      |
| Portage         | Gentoo                  | `emerge --ask app-shells/fzf`      |
| Spack           |                         | `spack install fzf`                |
| XBPS            | Void Linux              | `sudo xbps-install -S fzf`         |
| Zypper          | openSUSE                | `sudo zypper install fzf`          |

> [!IMPORTANT]
> 设置 shell 集成（快捷键和模糊补全）请参阅[以下说明](#setting-up-shell-integration)。

[![Packaging status](https://repology.org/badge/vertical-allrepos/fzf.svg?columns=3)](https://repology.org/project/fzf/versions)

<a id="windows-packages"></a>
### Windows 软件包

Windows 上可以通过 [Chocolatey][choco]、[Scoop][scoop]、[Winget][winget] 和 [MSYS2][msys2] 安装 fzf：

| 包管理器 | 命令 |
| --------------- | ------------------------------------- |
| Chocolatey      | `choco install fzf`                   |
| Scoop           | `scoop install fzf`                   |
| Winget          | `winget install fzf`                  |
| MSYS2 (pacman)  | `pacman -S $MINGW_PACKAGE_PREFIX-fzf` |

[choco]: https://chocolatey.org/packages/fzf
[scoop]: https://github.com/ScoopInstaller/Main/blob/master/bucket/fzf.json
[winget]: https://github.com/microsoft/winget-pkgs/tree/master/manifests/j/junegunn/fzf
[msys2]: https://packages.msys2.org/base/mingw-w64-fzf

<a id="using-git"></a>
### 使用 git

也可以将此仓库用 `git clone` 克隆到任意目录，再运行 [install](https://github.com/junegunn/fzf/blob/master/install) 脚本。

```sh
git clone --depth 1 https://github.com/junegunn/fzf.git ~/.fzf
~/.fzf/install
```

安装脚本会向 shell 配置文件添加内容，以修改 `$PATH` 并设置 shell 集成。

<a id="binary-releases"></a>
### 二进制发行版

可以从发行页面下载官方 fzf 二进制文件。

* https://github.com/junegunn/fzf/releases

<a id="setting-up-shell-integration"></a>
### 设置 shell 集成

将以下行添加到 shell 配置文件。

* bash
  ```sh
  # 设置 fzf 快捷键和模糊补全
  eval "$(fzf --bash)"
  ```
* zsh
  ```sh
  # 设置 fzf 快捷键和模糊补全
  source <(fzf --zsh)
  ```
* fish
  ```fish
  # 设置 fzf 快捷键
  fzf --fish | source
  ```
* Nushell——Nushell 不支持通过管道传入 `source`，因此安装脚本会在 autoload 目录生成文件。如果没有使用安装脚本，可以手动设置：
  ```nu
  # 生成集成脚本
  mkdir ($nu.default-config-dir | path join "autoload")
  fzf --nushell | save -f ($nu.default-config-dir | path join "autoload" "_fzf_integration.nu")
  ```

> [!NOTE]
> `--bash`、`--zsh`、`--fish` 和 `--nushell` 选项只在较新的 fzf 版本中提供。如果使用旧版本或需要更细粒度的控制，可以加载 [/shell](https://github.com/junegunn/fzf/tree/99bfd5aa0f66133d7dcfbf18f8044eb765f04649/shell) 目录中的各个脚本文件。文件位置可能因包管理器而异，详情请参阅包文档（例如 `apt show fzf`）。

> [!TIP]
> 加载脚本时将对应的 `*_COMMAND` 变量设为空字符串，可以禁用 CTRL-T、CTRL-R 或 ALT-C 绑定。例如禁用 CTRL-R 和 ALT-C：
>
> * bash: `FZF_CTRL_R_COMMAND= FZF_ALT_C_COMMAND= eval "$(fzf --bash)"`
> * zsh: `FZF_CTRL_R_COMMAND= FZF_ALT_C_COMMAND= source <(fzf --zsh)`
> * fish: `fzf --fish | FZF_CTRL_R_COMMAND= FZF_ALT_C_COMMAND= source`
> * nushell: add to your `env.nu`:
>   `$env.FZF_CTRL_R_COMMAND = ""; $env.FZF_ALT_C_COMMAND = ""`
>
> 加载脚本之后再设置变量不会生效。

<a id="vimneovim-plugin"></a>
### Vim/Neovim 插件

如果使用 [vim-plug](https://github.com/junegunn/vim-plug)，请将以下内容添加到 Vim 配置文件：

```vim
Plug 'junegunn/fzf', { 'do': { -> fzf#install() } }
Plug 'junegunn/fzf.vim'
```

* `junegunn/fzf` 提供基本库函数
    * `fzf#install()` 确保你拥有最新二进制文件
* `junegunn/fzf.vim` 是[独立项目](https://github.com/junegunn/fzf.vim)，提供多种实用命令

有关 Vim 集成的更多信息，请参阅 [README-VIM.md](https://github.com/junegunn/fzf/blob/99bfd5aa0f66133d7dcfbf18f8044eb765f04649/README-VIM.md)。

> [!TIP]
> 如果使用 Neovim 且偏好基于 Lua 的插件，可以看看 [fzf-lua](https://github.com/ibhagwan/fzf-lua)。

<a id="upgrading-fzf"></a>
升级 fzf
-------------

fzf 正在积极开发，你可能需要定期升级。请根据使用的安装方式遵循以下说明。

- git: `cd ~/.fzf && git pull && ./install`
- brew: `brew update; brew upgrade fzf`
- macports: `sudo port upgrade fzf`
- chocolatey: `choco upgrade fzf`
- vim-plug: `:PlugUpdate fzf`

<a id="building-fzf"></a>
构建 fzf
------------

参见 [BUILD.md](https://github.com/junegunn/fzf/blob/99bfd5aa0f66133d7dcfbf18f8044eb765f04649/BUILD.md)。

<a id="usage"></a>
用法
-----

fzf 将启动交互式查找器，从标准输入读取列表，并将选中项写入标准输出。

```sh
find * -type f | fzf > selected
```

没有标准输入管道时，fzf 会遍历当前目录下的文件系统以获取文件列表。

```sh
vim $(fzf)
```

> [!NOTE]
> 可以覆盖默认行为：
> * 将 `$FZF_DEFAULT_COMMAND` 设置为生成所需列表的命令；
> * 或者在 `$FZF_DEFAULT_OPTS` 中设置 `--walker`、`--walker-root` 和 `--walker-skip` 选项。

> [!WARNING]
> 更稳健的方案是使用 `xargs`，但上面的例子更容易理解：
> ```sh
> fzf --print0 | xargs -0 -o vim
> ```

> [!TIP]
> fzf 也能够将自身变成另一个进程。
>
> ```sh
> fzf --bind 'enter:become(vim {})'
> ```
>
> *详情参见[变成另一个进程](#turning-into-a-different-process)。*

<a id="using-the-finder"></a>
### 使用查找器

- 使用 `CTRL-K` / `CTRL-J`（或 `CTRL-P` / `CTRL-N`）上下移动光标。
- 按 `Enter` 选择条目，按 `CTRL-C` / `CTRL-G` / `ESC` 退出。
- 在多选模式（`-m`）下，使用 `TAB` 和 `Shift-TAB` 标记多个条目。
- 支持 Emacs 风格快捷键。
- 鼠标支持滚动、点击和双击；多选模式还支持 Shift 点击和 Shift 滚动。

<a id="display-modes"></a>
### 显示模式

fzf 默认全屏运行，也提供其他显示模式。

<a id="--height-mode"></a>
#### `--height` 模式

使用 `--height HEIGHT[%]` 时，fzf 会在光标下方以指定高度启动。

```sh
fzf --height 40%
```

`reverse` 布局和 `--border` 与此选项很搭配。

```sh
fzf --height 40% --layout reverse --border
```

在高度前加 `~` 表示设置最大高度。

```sh
# 使用尽可能少的行显示列表
seq 3 | fzf --height ~100%
seq 3000 | fzf --height ~100%
```

高度值可以是负数。

```sh
# 屏幕高度减 3
fzf --height -3
```

<a id="--popup-mode"></a>
#### `--popup` 模式

使用 `--popup` 选项时，fzf 会在弹出窗口中启动（需要 tmux 3.3+ 或 Zellij 0.44+）。

```sh
# --popup [center|top|bottom|left|right][,SIZE[%]][,SIZE[%][,border-native]]

fzf --popup center         # 居中，宽高均为 50%
fzf --popup 80%            # 居中，宽高均为 80%
fzf --popup 100%,50%       # 居中，宽度 100%，高度 50%
fzf --popup left,40%       # 居左，宽度 40%
fzf --popup left,40%,90%   # 居左，宽度 40%，高度 90%
fzf --popup top,40%        # 居上，高度 40%
fzf --popup bottom,80%,40% # 居下，宽度 80%，高度 40%
```

不在 tmux 或 Zellij 中时，`--popup` 会被静默忽略。

> [!NOTE]
> 如果使用不支持弹出窗口的旧版 tmux，或想在普通 tmux 窗格中打开 fzf，请参阅 [fzf-tmux](https://github.com/junegunn/fzf/blob/99bfd5aa0f66133d7dcfbf18f8044eb765f04649/bin/fzf-tmux) 脚本。

> [!TIP]
> 可以将这些选项加入 `$FZF_DEFAULT_OPTS`，使其默认生效。例如：
>
> ```sh
> # 在 tmux 或 Zellij 中以弹出窗口打开，否则使用 --height 模式
> export FZF_DEFAULT_OPTS='--height 40% --popup bottom,40% --layout reverse --border top'
> ```

<a id="search-syntax"></a>
### 搜索语法

除非另有指定，fzf 以“扩展搜索模式”启动，可输入由空格分隔的多个搜索词，例如 `^music .mp3$ sbtrkt !fire`。

| 词项 | 匹配类型 | 说明 |
| --------- | --------------------------------------  | ------------------------------------------   |
| `sbtrkt` | 模糊匹配 | 与 `sbtrkt` 匹配的条目 |
| `'wild` | 精确匹配（加引号） | 包含 `wild` 的条目 |
| `'wild'` | 单词边界精确匹配（两端加引号） | 在单词边界包含 `wild` 的条目 |
| `^music` | 前缀精确匹配 | 以 `music` 开头的条目 |
| `.mp3$` | 后缀精确匹配 | 以 `.mp3` 结尾的条目 |
| `!fire` | 反向精确匹配 | 不包含 `fire` 的条目 |
| `!^music` | 反向前缀精确匹配 | 不以 `music` 开头的条目 |
| `!.mp3$` | 反向后缀精确匹配 | 不以 `.mp3` 结尾的条目 |

如果不偏好模糊匹配，也不想为每个词加引号，请使用 `-e` 或 `--exact` 启动 fzf。注意设置 `--exact` 后，`'` 前缀会对搜索词“取消引用”。

单独的竖线字符作为 OR 运算符。例如以下查询匹配以 `core` 开头且以 `go`、`rb` 或 `py` 之一结尾的条目。

```
^core go$ | rb$ | py$
```

<a id="environment-variables"></a>
### 环境变量

- `FZF_DEFAULT_COMMAND`
    - 输入为 tty 时使用的默认命令
    - 例如 `export FZF_DEFAULT_COMMAND='fd --type f'`
- `FZF_DEFAULT_OPTS`
    - 默认选项
    - 例如 `export FZF_DEFAULT_OPTS="--layout=reverse --inline-info"`
- `FZF_DEFAULT_OPTS_FILE`
    - 如果偏好在文件中管理默认选项，请将此变量设为该文件的位置
    - 例如 `export FZF_DEFAULT_OPTS_FILE=~/.fzfrc`

> [!WARNING]
> 由于需求略有不同，shell 集成不使用 `FZF_DEFAULT_COMMAND`。
>
> * `CTRL-T` 运行 `$FZF_CTRL_T_COMMAND` 获取文件和目录列表
> * `ALT-C` 运行 `$FZF_ALT_C_COMMAND` 获取目录列表
> * `vim ~/**<tab>` 运行 `fzf_compgen_path()`，以前缀（`~/`）作为第一个参数
> * `cd foo**<tab>` 运行 `fzf_compgen_dir()`，以前缀（`foo`）作为第一个参数
>
> 可用选项将在本文后面介绍。

<a id="customizing-the-look"></a>
### 自定义外观

fzf 的用户界面提供大量配置选项，可以完全自定义。要快速设置，可以使用 `--style` 选择 `default`、`full` 或 `minimal` 样式预设。

```sh
fzf --style full \
    --preview 'fzf-preview.sh {}' --bind 'focus:transform-header:file --brief {}'
```

| 预设 | 截图 |
| :---      | :---                                                                                   |
| `default` | <img src="https://raw.githubusercontent.com/junegunn/i/master/fzf-style-default.png"/> |
| `full`    | <img src="https://raw.githubusercontent.com/junegunn/i/master/fzf-style-full.png"/>    |
| `minimal` | <img src="https://raw.githubusercontent.com/junegunn/i/master/fzf-style-minimal.png"/> |

以下是基于 `full` 预设的示例：

<img src="https://raw.githubusercontent.com/junegunn/i/master/fzf-4-borders.png"/>

<details>

```sh
git ls-files | fzf --style full \
    --border --padding 1,2 \
    --border-label ' Demo ' --input-label ' Input ' --header-label ' File Type ' \
    --preview 'fzf-preview.sh {}' \
    --bind 'result:transform-list-label:
        if [[ -z $FZF_QUERY ]]; then
          echo " $FZF_MATCH_COUNT items "
        else
          echo " $FZF_MATCH_COUNT matches for [$FZF_QUERY] "
        fi
        ' \
    --bind 'focus:transform-preview-label:[[ -n {} ]] && printf " Previewing [%s] " {}' \
    --bind 'focus:+transform-header:file --brief {} || echo "No file selected"' \
    --bind 'ctrl-r:change-list-label( Reloading the list )+reload(sleep 2; git ls-files)' \
    --color 'border:#aaaaaa,label:#cccccc' \
    --color 'preview-border:#9999cc,preview-label:#ccccff' \
    --color 'list-border:#669966,list-label:#99cc99' \
    --color 'input-border:#996666,input-label:#ffcccc' \
    --color 'header-border:#6699cc,header-label:#99ccff'
```

</details>

<a id="options"></a>
### 选项

完整选项列表请参阅 man 手册（`fzf --man` 或 `man fzf`）。

<a id="demo"></a>
### 演示
如果喜欢通过视频学习，可以观看 [@samoshkin](https://github.com/samoshkin) 的屏幕录像，探索 `fzf` 功能。

<a title="fzf - command-line fuzzy finder" href="https://www.youtube.com/watch?v=qgG5Jhi_Els">
  <img src="https://i.imgur.com/vtG8olE.png" width="640">
</a>

<a id="examples"></a>
示例
--------

* [示例 Wiki 页面](https://github.com/junegunn/fzf/wiki/examples)
    * *声明：此页面的示例由社区维护，未经过全面测试*
* [fzf 高级示例](https://github.com/junegunn/fzf/blob/master/ADVANCED.md)

<a id="key-bindings-for-command-line"></a>
命令行快捷键
-----------------------------

通过[设置 shell 集成](#setting-up-shell-integration)，可以在 bash、zsh、fish 和 Nushell 中使用以下快捷键。

- `CTRL-T`——将选中的文件和目录粘贴到命令行
    - 使用 `--walker file,dir,follow,hidden` 选项生成列表
        - 可以将 `FZF_CTRL_T_COMMAND` 设置为生成所需列表的自定义命令，以覆盖此行为
        - 也可以在 `FZF_CTRL_T_OPTS` 中设置 `--walker*` 选项
    - 设置 `FZF_CTRL_T_OPTS` 向 fzf 传递额外选项
      ```sh
      # 使用 bat 预览文件内容（https://github.com/sharkdp/bat）
      export FZF_CTRL_T_OPTS="
        --walker-skip .git,node_modules,target
        --preview 'bat -n --color=always {}'
        --bind 'ctrl-/:change-preview-window(down|hidden|)'"
      ```
    - 加载脚本时将 `FZF_CTRL_T_COMMAND` 设为空字符串可禁用该功能
- `CTRL-R`——将选中的历史命令粘贴到命令行。
    - 使用 `TAB` 选择多个命令。
    - 要按时间顺序查看命令，请再次按 `CTRL-R`，切换按相关性排序。
    - 按 `ALT-R` 切换“原始”模式，可查看匹配项周围的条目。在该模式下，`CTRL-N` 和 `CTRL-P` 仅在匹配项之间移动。
    - 按 `CTRL-/` 或 `ALT-/` 切换自动换行
    - 按 `SHIFT-DELETE` 删除选中的命令（bash 和 fish）
    - 仅限 Fish shell：
      - 按 `ALT-ENTER` 重新格式化并插入选中的命令
      - 按 `ALT-T` 循环切换命令前缀（时间戳、日期/时间、无）
    - 设置 `FZF_CTRL_R_OPTS` 向 fzf 传递额外选项
      ```sh
      # 使用 CTRL-Y 通过 pbcopy 将命令复制到剪贴板
      export FZF_CTRL_R_OPTS="
        --bind 'ctrl-y:execute-silent(echo -n {2..} | pbcopy)+abort'
        --color header:italic
        --header 'Press CTRL-Y to copy command into clipboard'"
      ```
      ```fish
      # Fish shell：将日期/时间设为默认前缀
      set -gx FZF_CTRL_R_OPTS "--with-nth 1,3.. --bind 'alt-t:change-with-nth(2..|3..|1,3..)'"

      # 或者默认不显示前缀
      set -gx FZF_CTRL_R_OPTS "--with-nth 3.. --bind 'alt-t:change-with-nth(2..|1,3..|3..)'"
      ```
    - 加载脚本时将 `FZF_CTRL_R_COMMAND` 设为空字符串可禁用该功能
    - 尚不支持通过非空 `FZF_CTRL_R_COMMAND` 自定义覆盖，使用时会发出警告
- `ALT-C`——通过 cd 进入选中的目录
    - 使用 `--walker dir,follow,hidden` 选项生成列表
    - 设置 `FZF_ALT_C_COMMAND` 以覆盖默认命令
        - 也可以在 `FZF_ALT_C_OPTS` 中设置 `--walker-*` 选项
    - 设置 `FZF_ALT_C_OPTS` 向 fzf 传递额外选项
      ```sh
      # 在预览窗口中显示树形结构
      export FZF_ALT_C_OPTS="
        --walker-skip .git,node_modules,target
        --preview 'tree -C {}'"
      ```
    - 加载脚本时将 `FZF_ALT_C_COMMAND` 设为空字符串可禁用该功能

这些绑定的显示模式可分别通过 `FZF_{CTRL_T,CTRL_R,ALT_C}_OPTS` 配置，或通过 `FZF_DEFAULT_OPTS` 全局配置（例如 `FZF_CTRL_R_OPTS='--popup bottom,60% --height 60% --border top'`）。

更多技巧见 [Wiki 页面](https://github.com/junegunn/fzf/wiki/Configuring-shell-key-bindings)。

<a id="fuzzy-completion"></a>
模糊补全
----------------

Shell 集成还为 bash、zsh、fish 和 Nushell 提供模糊补全。

<a id="files-and-directories"></a>
### 文件和目录

如果光标前的词以触发序列结尾，即可触发文件和目录的模糊补全；默认触发序列为 `**`。

- `COMMAND [DIRECTORY/][FUZZY_PATTERN]**<TAB>`

```sh
# 当前目录下的文件
# - 可以使用 TAB 键选择多个条目
vim **<TAB>

# 父目录下的文件
vim ../**<TAB>

# 父目录下与 `fzf` 匹配的文件
vim ../fzf**<TAB>

# 主目录下的文件
vim ~/**<TAB>


# 当前目录下的目录（单选）
cd **<TAB>

# ~/github 下与 `fzf` 匹配的目录
cd ~/github/fzf**<TAB>
```

<a id="process-ids"></a>
### 进程 ID

kill 命令支持进程 ID 的模糊补全。

```sh
# 可以使用 <TAB> 或 <Shift-TAB> 键选择多个进程
kill -9 **<TAB>
```

<a id="host-names"></a>
### 主机名

ssh 命令支持主机名模糊补全，名称取自 /etc/hosts 和 ~/.ssh/config。

```sh
ssh **<TAB>
```

<a id="environment-variables--aliases"></a>
### 环境变量 / 别名

```sh
# bash and zsh
unset **<TAB>
export **<TAB>
unalias **<TAB>

# fish
set <SHIFT-TAB>
```

<a id="customizing-fuzzy-completion-for-bash-and-zsh"></a>
### 自定义 bash 和 zsh 的模糊补全

<a id="customizing-fzf-options-for-completion"></a>
#### 自定义补全的 fzf 选项

```sh
# 使用 ~~ 作为触发序列，替代默认的 **
export FZF_COMPLETION_TRIGGER='~~'

# fzf 命令选项
export FZF_COMPLETION_OPTS='--border --info=inline'

# 路径补全选项（例如 vim **<TAB>）
export FZF_COMPLETION_PATH_OPTS='--walker file,dir,follow,hidden'

# 目录补全选项（例如 cd **<TAB>）
export FZF_COMPLETION_DIR_OPTS='--walker dir,follow'

# 通过 _fzf_comprun 函数高级自定义 fzf 选项
# - 函数的第一个参数是命令名称。
# - 请确保将其余参数（$@）传给 fzf。
_fzf_comprun() {
  local command=$1
  shift

  case "$command" in
    cd)           fzf --preview 'tree -C {} | head -200'   "$@" ;;
    export|unset) fzf --preview "eval 'echo \$'{}"         "$@" ;;
    ssh)          fzf --preview 'dig {}'                   "$@" ;;
    *)            fzf --preview 'bat -n --color=always {}' "$@" ;;
  esac
}
```

<a id="customizing-completion-source-for-paths-and-directories"></a>
#### 自定义路径和目录的补全来源

```sh
# 使用 fd（https://github.com/sharkdp/fd）列出候选路径。
# - 函数的第一个参数（$1）是开始遍历的基础路径
# - 详情参阅源代码（completion.{bash,zsh}）。
_fzf_compgen_path() {
  fd --hidden --follow --exclude ".git" . "$1"
}

# 使用 fd 生成目录补全列表
_fzf_compgen_dir() {
  fd --type d --hidden --follow --exclude ".git" . "$1"
}
```

<a id="supported-commands-bash"></a>
#### 支持的命令（bash）

在 bash 中，模糊补全只为预定义的一组命令启用（运行 `complete | grep _fzf` 查看列表）。也可以使用 `_fzf_setup_completion` 辅助函数为其他命令启用。

```sh
# usage: _fzf_setup_completion path|dir|var|alias|host COMMANDS...
_fzf_setup_completion path ag git kubectl
_fzf_setup_completion dir tree
```

<a id="custom-fuzzy-completion"></a>
#### 自定义模糊补全

_**（自定义补全 API 处于实验阶段，可能发生变化）**_

对于名为 _"COMMAND"_ 的命令，使用 `_fzf_complete` 辅助函数定义 `_fzf_complete_COMMAND` 函数。

```sh
# 为 "doge" 命令自定义模糊补全
#   例如 doge **<TAB>
_fzf_complete_doge() {
  _fzf_complete --multi --reverse --prompt="doge> " -- "$@" < <(
    echo very
    echo wow
    echo such
    echo doge
  )
}
```

- `--` 之前的参数是 fzf 的选项。
- `--` 之后原样传入原始补全参数（`"$@"`）。
- 然后编写一组生成补全候选项的命令，并通过进程替换（`< <(...)`）将输出传给函数。

zsh 会根据命名约定自动找到该函数；在 bash 中，必须使用 `complete` 命令手动将函数与命令关联。

```sh
[ -n "$BASH" ] && complete -F _fzf_complete_doge -o default -o bashdefault doge
```

如果需要对 fzf 输出进行后处理，请按如下方式定义 `_fzf_complete_COMMAND_post`。

```sh
_fzf_complete_foo() {
  _fzf_complete --multi --reverse --header-lines=3 -- "$@" < <(
    ls -al
  )
}

_fzf_complete_foo_post() {
  awk '{print $NF}'
}

[ -n "$BASH" ] && complete -F _fzf_complete_foo -o default -o bashdefault foo
```

<a id="fuzzy-completion-for-fish"></a>
### fish 的模糊补全

fish 的模糊补全与 bash 和 zsh 的区别在于：

- 它不需要 `**` 之类的触发序列，而是通过 `Shift-TAB` 激活（替代原生分页器搜索模式）；`TAB` 保留 fish 的原生补全行为。
- 它依靠 fish 的原生补全系统填充候选列表，而不是递归遍历文件系统。如需递归搜索，请使用 `CTRL-T` 绑定。
- 如果当前命令行词项是通配符模式，它会在通配符展开后的路径列表中搜索，而不是像原生行为那样将所有结果插入命令行。由于使用 shell 展开，结果数量有限制。
- 仅支持配置变量 `FZF_COMPLETION_OPTS` 和 `FZF_EXPANSION_OPS`。
- 自定义补全函数使用的函数名为 `fzf_complete`，只接受 fzf 选项作为参数；也可不重定向输入直接调用，在显示原生补全结果时仅修改 fzf 选项。为兼容其他 shell，还提供 `_fzf_complete` 函数，可在 fzf 选项后接受命令行参数 ` -- $argv`。

对于 fish 补全未覆盖的命令，最好创建普通 fish 补全函数（同时适用于 `TAB` 和 `Shift-TAB`）；只在需要为特定命令修改 fzf 选项，或希望 `Shift-TAB` 给出不同结果时，才创建 fzf 补全函数：

```fish
# 自定义 git 补全
function _fzf_complete_git
  # 为所有 git 补全显示含当前分支的标题文本
  set -lx -- FZF_COMPLETION_OPTS --header="'"(git branch --show-current 2>/dev/null)"'"

  # 参数少于 3 个或正在补全选项时不作其他更改
  if not set -q argv[3]; or string match -q -- '-*' $argv[-1]
    fzf_complete
    return
  end

  # 检查子命令
  switch $argv[2]
    case checkout diff log show
      # 为子命令 checkout、diff、log、show 设置预览并显示所有分支和提交
      begin
        git branch --all --format='%(refname:short)'
        git log --all --oneline --color=always
      end | fzf_complete --no-multi --ansi --accept-nth=1 --query=$argv[-1] --preview='git show --color=always {1}'

    case add rm mv
      # 仅为子命令 add、rm、mv 设置预览
      # fish 补全列表中的特殊字符已转义，因此必须使用 r 标志。
      fzf_complete --preview="git diff --color=always {r1}"

    case '*'
      # 其他子命令不作更改
      fzf_complete
  end
end
```

与 bash 和 zsh 类似，可以定义名为 `_fzf_complete_COMMAND_post` 或 `_fzf_post_complete_COMMAND` 的函数，在 fzf 输出插入命令行前进行处理：

```fish
function _fzf_complete_foo
  ls -sh --zero --color=always | fzf_complete --read0 --print0 --ansi --no-multi-line --header-lines=1
end

function _fzf_post_complete_foo
  while read -lz result
    string escape -n -- $result | string trim -l -c '\ ' | string split -m 1 -f 2 ' '
  end
end
```

<a id="fuzzy-completion-for-nushell"></a>
### Nushell 的模糊补全

Nushell 的模糊补全通过[外部补全器](https://www.nushell.sh/cookbook/external_completers.html)机制工作，与 bash 和 zsh 有一些区别：

- 需要 Nushell 0.116.1 或更高版本。补全在线编辑器线程运行，以便 fzf 接管终端；因此未出现 `**` 触发符时，会在线调用先前配置的外部补全器（例如 Carapace）。
- 脚本会包装加载时配置的外部补全器，因此请在其他外部补全器设置之后加载。上文的自动加载设置在 `config.nu` 之后运行，已满足此要求。
- 在 Nushell >= 0.103.0 中，内置命令（如 `cd`、`ls`）不再调用外部补全器。使用 `**<TAB>` 的模糊补全只适用于外部命令。
- 可以在 `config.nu` 中通过 `$env.FZF_COMPLETERS` 记录定义自定义补全器。每个条目都是接收前缀和命令片段的闭包，并返回候选字符串列表，或返回带自定义 fzf 选项的记录 `{ candidates: [...], opts: [...] }`：
  ```nu
  $env.FZF_COMPLETERS = {
      pacman: {|prefix, spans|
          let sub = $spans | skip 1 | first
          let candidates = (if ($sub =~ "-[SF]") { ^pacman -Slq | lines
          } else if ($sub =~ "-[QR]") { ^pacman -Qq | lines
          } else { [] })
          { candidates: $candidates, opts: ["--preview", "pacman -Si {}"] }
      }
  }
  ```
  更多示例参见 [shell/completion-examples.nu](https://github.com/junegunn/fzf/blob/99bfd5aa0f66133d7dcfbf18f8044eb765f04649/shell/completion-examples.nu)。
- 支持以下环境变量：
  `FZF_COMPLETION_TRIGGER`, `FZF_COMPLETION_OPTS`,
  `FZF_COMPLETION_PATH_OPTS`, `FZF_COMPLETION_DIR_OPTS`,
  `FZF_COMPLETION_DIR_COMMANDS`.

<a id="vim-plugin"></a>
Vim 插件
----------

See [README-VIM.md](https://github.com/junegunn/fzf/blob/99bfd5aa0f66133d7dcfbf18f8044eb765f04649/README-VIM.md).

<a id="advanced-topics"></a>
高级主题
---------------

<a id="customizing-for-different-types-of-input"></a>
### 为不同类型输入自定义

由于 fzf 是通用文本过滤器，其算法旨在对任何输入“一般都能”良好工作。但确实没有真正适用于所有情况的方案，你可能希望根据输入类型调整算法和部分设置。为简化此过程，fzf 为一些常见输入类型提供一组“方案”。

| 方案 | 说明 |
| :---               | :---                                                                                |
| `--scheme=default` | 适用于各种输入的通用方案 |
| `--scheme=path` | 适用于文件路径 |
| `--scheme=history` | 适用于命令历史或任何时间顺序很重要的输入 |

（详情参见 `fzf --man`）

<a id="performance"></a>
### 性能

fzf 很快，大多数场景下性能应该不是问题。不过，你可能需要了解会影响性能的选项。

- `--ansi` 让 fzf 提取并解析输入中的 ANSI 颜色代码，会减慢初始扫描，因此不建议加入 `$FZF_DEFAULT_OPTS`。
- `--nth` 会使 fzf 变慢，因为它必须对每行进行分词。
- `--delimiter` 应优先使用普通字符串，而不是正则表达式分隔符。
- `--with-nth` 会使 fzf 变慢，因为它必须对每行分词并重新组装。

<a id="executing-external-programs"></a>
### 执行外部程序

可以设置快捷键，在不离开 fzf 的情况下启动外部进程（`execute`、`execute-silent`）。

```bash
# 按 F1 使用 less 打开文件而不离开 fzf
# 按 CTRL-Y 将该行复制到剪贴板并退出 fzf（需要 pbcopy）
fzf --bind 'f1:execute(less -f {}),ctrl-y:execute-silent(echo {} | pbcopy)+abort'
```

详情参阅 man 手册的 *KEY/EVENT BINDINGS* 章节。

<a id="turning-into-a-different-process"></a>
### 变成另一个进程

`become(...)` 与上文的 `execute(...)`/`execute-silent(...)` 类似，但它不会在命令完成后返回 fzf，而是将 fzf 变成该命令的新进程。

```sh
fzf --bind 'enter:become(vim {})'
```

与看似等价的命令替换 `vim "$(fzf)"` 相比，此方法有几个优点：

* 使用 <kbd>CTRL-C</kbd> 终止 fzf 时，Vim 不会打开空文件
* 对空结果按 <kbd>ENTER</kbd> 时，Vim 不会打开空文件
* 即使选择项含空格，也能处理多选
  ```sh
  fzf --multi --bind 'enter:become(vim {+})'
  ```

客观地说，用 `fzf --print0 | xargs -0 -o vim` 替代 `vim "$(fzf)"` 也能解决上述所有问题。不过，`become(...)` 在不同场景下仍有额外好处。

* 无需包装脚本即可设置多个绑定，以不同方式处理结果
  ```sh
  fzf --bind 'enter:become(vim {}),ctrl-e:become(emacs {})'
  ```
  * 以前必须使用 `--expect=ctrl-e` 并检查 fzf 输出的第一行
* 可以使用 fzf 的字段索引表达式轻松构造后续命令
  ```sh
  # 在 Vim 中打开文件并跳转到该行
  git grep --line-number . |
      fzf --delimiter : --nth 3.. --bind 'enter:become(vim {1} +{2})'
  ```

<a id="reloading-the-candidate-list"></a>
### 重新加载候选列表

将 `reload` 操作绑定到按键或事件，可以让 fzf 动态重新加载候选列表。详情见 https://github.com/junegunn/fzf/issues/1750。

<a id="1-update-the-list-of-processes-by-pressing-ctrl-r"></a>
#### 1. 按 CTRL-R 更新进程列表

```sh
ps -ef |
  fzf --bind 'ctrl-r:reload(ps -ef)' \
      --header 'Press CTRL-R to reload' --header-lines=1 \
      --height=50% --layout=reverse
```

<a id="2-switch-between-sources-by-pressing-ctrl-d-or-ctrl-f"></a>
#### 2. 按 CTRL-D 或 CTRL-F 切换数据源

```sh
FZF_DEFAULT_COMMAND='find . -type f' \
  fzf --bind 'ctrl-d:reload(find . -type d),ctrl-f:reload(eval "$FZF_DEFAULT_COMMAND")' \
      --height=50% --layout=reverse
```

<a id="3-interactive-ripgrep-integration"></a>
#### 3. 交互式 ripgrep 集成

以下示例将 fzf 用作 ripgrep 的选择界面。我们把 `reload` 操作绑定到 `change` 事件，因此每次在 fzf 输入时，ripgrep 进程都会使用占位表达式 `{q}` 表示的更新后查询字符串重新启动。注意我们使用 `--disabled` 选项，避免 fzf 进行二次过滤。

```sh
: | rg_prefix='rg --column --line-number --no-heading --color=always --smart-case' \
    fzf --bind 'start:reload:$rg_prefix ""' \
        --bind 'change:reload:$rg_prefix {q} || true' \
        --bind 'enter:become(vim {1} +{2})' \
        --ansi --disabled \
        --height=50% --layout=reverse
```

如果 ripgrep 找不到匹配项，会以非零状态退出，fzf 会发出警告。为抑制警告，我们在命令中加入 `|| true`，使其始终以 0 退出。

更复杂的示例参见[“将 fzf 用作交互式 Ripgrep 启动器”](https://github.com/junegunn/fzf/blob/master/ADVANCED.md#using-fzf-as-interactive-ripgrep-launcher)。

<a id="preview-window"></a>
### 预览窗口

设置 `--preview` 选项后，fzf 会自动以当前行为参数启动外部进程，并在分屏窗口中显示结果。它使用 `$SHELL -c COMMAND` 通过你的 `$SHELL` 执行命令。可以使用鼠标或自定义快捷键滚动窗口。

```bash
# {} 替换为当前焦点行的单引号字符串
fzf --preview 'cat {}'
```

预览窗口支持 ANSI 颜色，因此可以使用任何对文件内容进行语法高亮的程序，例如 [Bat](https://github.com/sharkdp/bat) 或 [Highlight](https://gitlab.com/saalen/highlight)：

```bash
fzf --preview 'bat --color=always {}' --preview-window '~3'
```

可以使用 `--preview-window` 自定义预览窗口的大小、位置和边框，使用 `--color` 自定义前景色和背景色。例如：

```bash
fzf --height 40% --layout reverse --info inline --border \
    --preview 'file {}' --preview-window up,1,border-horizontal \
    --bind 'ctrl-/:change-preview-window(50%|hidden|)' \
    --color 'fg:#bbccdd,fg+:#ddeeff,bg:#334455,preview-bg:#223344,border:#778899'
```

完整选项列表请参阅 man 手册（`man fzf`）。

更多高级示例请见[这里](https://github.com/junegunn/fzf/blob/master/ADVANCED.md)。

> [!WARNING]
> 由于 fzf 是通用文本过滤器而非文件查找器，**不建议将 `--preview` 选项加入 `$FZF_DEFAULT_OPTS`**。
>
> ```sh
> # *********************
> # ** 不要这样做！ **
> # *********************
> export FZF_DEFAULT_OPTS='--preview "bat --style=numbers --color=always --line-range :500 {}"'
>
> # 除文件列表外，bat 无法处理其他输入
> ps -ef | fzf
> seq 100 | fzf
> history | fzf
> ```

<a id="previewing-an-image"></a>
### 预览图像

fzf 可以使用以下任一协议在预览窗口中显示图像：

* [Kitty 图形协议](https://sw.kovidgoyal.net/kitty/graphics-protocol/)
* [iTerm2 内联图像协议](https://iterm2.com/documentation-images.html)
* [Sixel](https://en.wikipedia.org/wiki/Sixel)

更多信息请参阅 [bin/fzf-preview.sh](https://github.com/junegunn/fzf/blob/99bfd5aa0f66133d7dcfbf18f8044eb765f04649/bin/fzf-preview.sh) 脚本。

```sh
fzf --preview 'fzf-preview.sh {}'
```

<a id="tips"></a>
技巧
----

<a id="respecting-gitignore"></a>
### 遵循 `.gitignore`

可以使用 [fd](https://github.com/sharkdp/fd)、[ripgrep](https://github.com/BurntSushi/ripgrep) 或 [the silver searcher](https://github.com/ggreer/the_silver_searcher) 遍历文件系统，同时遵循 `.gitignore`。

```sh
# 将 fd 的输出传给 fzf
fd --type f --strip-cwd-prefix | fzf

# 将 fd 设为 fzf 的默认来源
export FZF_DEFAULT_COMMAND='fd --type f --strip-cwd-prefix'

# 现在 fzf（不使用管道时）将使用 fd 命令生成列表
fzf

# 要将该命令也应用于 CTRL-T
export FZF_CTRL_T_COMMAND="$FZF_DEFAULT_COMMAND"
```

如果希望命令跟随符号链接且不排除隐藏文件，请使用以下命令：

```sh
export FZF_DEFAULT_COMMAND='fd --type f --strip-cwd-prefix --hidden --follow --exclude .git'
```

<a id="fish-shell"></a>
### Fish shell

与 bash 和 zsh 不同，fish 的 `CTRL-T` 快捷键会将命令行最后一个词项作为递归搜索的根目录。例如，在以下命令行末尾按 `CTRL-T`：

```sh
ls /var/
```

将列出 `/var/` 下的所有文件和目录。

使用自定义 `FZF_CTRL_T_COMMAND` 时，可使用未展开的 `$dir` 变量利用此功能。最后一个词项不是有效目录时，`$dir` 默认为 `.`。例如：

```sh
set -g FZF_CTRL_T_COMMAND "command find -L \$dir -type f 2> /dev/null | sed '1d; s#^\./##'"
```

<a id="fzf-theme-playground"></a>
### fzf Theme Playground

[Vitor Mello](https://github.com/vitormv) 创建的 [fzf Theme Playground](https://vitormv.github.io/fzf-themes/) 是一个可交互创建 fzf 主题的网页。

<a id="related-projects"></a>
相关项目
----------------

https://github.com/junegunn/fzf/wiki/Related-projects

<a id="license"></a>
[许可证](https://github.com/junegunn/fzf/blob/99bfd5aa0f66133d7dcfbf18f8044eb765f04649/LICENSE)
------------------

MIT 许可证（MIT）

Copyright (c) 2013-2026 Junegunn Choi

<a id="goods"></a>
周边
-----

可在此获取 fzf T 恤、马克杯和贴纸：[commitgoods.com/collections/fzf](https://commitgoods.com/collections/fzf?utm_source=github&utm_medium=referral)

<a id="sponsors-heart"></a>
赞助者 :heart:
----------------

我要感谢本项目的所有赞助者，正是你们让我能够持续改进 fzf。

如果想赞助本项目，请访问 https://github.com/sponsors/junegunn。
