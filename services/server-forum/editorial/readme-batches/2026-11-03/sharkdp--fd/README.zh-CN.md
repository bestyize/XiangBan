> 根据 [fd 固定版本 README](https://github.com/sharkdp/fd/blob/cc4e40fa4e32d61733cd72926e233a3fb6c8d56d/README.md) 翻译。基准结果为上游在特定机器上的测试，未独立复测；删除文件示例含安全提示，请先核对搜索结果。

# fd

[![CICD](https://github.com/sharkdp/fd/actions/workflows/CICD.yml/badge.svg)](https://github.com/sharkdp/fd/actions/workflows/CICD.yml)
[![Version info](https://img.shields.io/crates/v/fd-find.svg)](https://crates.io/crates/fd-find)
[[中文](https://github.com/cha0ran/fd-zh)]
[[한국어](https://github.com/spearkkk/fd-kor)]

`fd` 是一个在文件系统中查找条目的程序。它是 [`find`](https://www.gnu.org/software/findutils/) 的简单、快速、易用的替代品。虽然它不打算支持 `find` 的所有强大功能，但为大多数使用场景提供了合理且带有设计取舍的默认设置。

[安装](#installation) • [使用方法](#how-to-use) • [故障排查](#troubleshooting)

## 功能

* 直观的语法：使用 `fd PATTERN`，而非 `find -iname '*PATTERN*'`。
* 支持正则表达式（默认）和 glob 模式。
* 并行遍历目录，因此[速度很快](#benchmark)。
* 使用颜色突出不同文件类型（与 `ls` 相同）。
* 支持[并行执行命令](#command-execution)。
* 智能大小写：默认不区分大小写；如果模式包含大写字符，则切换为区分大小写[\*](http://vimdoc.sourceforge.net/htmldoc/options.html#'smartcase')。
* 默认忽略隐藏目录和文件。
* 默认忽略 `.gitignore` 中的模式。
* 命令名比 `find` 短 *50%*[\*](https://github.com/ggreer/the_silver_searcher) :-)。

## 演示

![Demo](https://raw.githubusercontent.com/sharkdp/fd/cc4e40fa4e32d61733cd72926e233a3fb6c8d56d/doc/screencast.svg)

<a id="how-to-use"></a>
## 使用方法

首先，可以运行 [`fd -h`](#command-line-options) 查看简明帮助，或运行 `fd --help` 查看更详细的版本，以了解所有可用命令行选项。

### 简单搜索

*fd* 用于查找文件系统中的条目。最基本的搜索是运行 *fd* 并传入一个参数：搜索模式。例如，假设你想找到以前写的一个脚本（名称包含 `netflix`）：
``` bash
> fd netfl
Software/python/imdb-ratings/netflix-details.py
```
像这样只传一个参数时，*fd* 会递归搜索当前目录中所有*包含*模式 `netfl` 的条目。

### 正则表达式搜索

搜索模式按正则表达式处理。这里搜索以 `x` 开头、以 `rc` 结尾的条目：
``` bash
> cd /etc
> fd '^x.*rc$'
X11/xinit/xinitrc
X11/xinit/xserverrc
```

`fd` 使用的正则表达式语法[记录于此](https://docs.rs/regex/latest/regex/#syntax)。

### 指定根目录

要搜索指定目录，可以将目录作为 *fd* 的第二个参数：
``` bash
> fd passwd /etc
/etc/default/passwd
/etc/pam.d/passwd
/etc/passwd
```

### 递归列出所有文件

*fd* 可以不带参数调用。这对于快速递归查看当前目录中的所有条目很有用（类似 `ls -R`）：
``` bash
> cd fd/tests
> fd
testenv
testenv/mod.rs
tests.rs
```

要使用此功能列出指定目录中的所有文件，需要使用 `.` 或 `^` 等匹配所有内容的模式：
``` bash
> fd . fd/tests/
testenv
testenv/mod.rs
tests.rs
```

### 按特定文件扩展名搜索

我们经常需要某一类型的所有文件，可用 `-e`（或 `--extension`）选项。这里搜索 fd 仓库中的所有 Markdown 文件：
``` bash
> cd fd
> fd -e md
CONTRIBUTING.md
README.md
```

`-e` 选项可以与搜索模式结合使用：
``` bash
> fd -e rs mod
src/fshelper/mod.rs
src/lscolors/mod.rs
tests/testenv/mod.rs
```

### 按特定文件名搜索

要查找与给定搜索模式完全匹配的文件，请使用 `-g`（或 `--glob`）选项：
``` bash
> fd -g libc.so /usr
/usr/lib32/libc.so
/usr/lib/libc.so
```

### 隐藏和忽略的文件
默认情况下，*fd* 不搜索隐藏目录，也不在结果中显示隐藏文件。可以使用 `-H`（或 `--hidden`）关闭这一行为：
``` bash
> fd pre-commit
> fd -H pre-commit
.git/hooks/pre-commit.sample
```

如果当前目录是 Git 仓库或包含 Git 仓库，*fd* 不会搜索或显示与 `.gitignore` 模式匹配的目录和文件。可以使用 `-I`（或 `--no-ignore`）关闭这一行为：
``` bash
> fd num_cpu
> fd -I num_cpu
target/debug/deps/libnum_cpus-f5ce7ef99006aa05.rlib
```

要真正搜索*所有*文件和目录，只需组合显示隐藏和忽略内容的选项（`-HI`），或使用 `-u`/`--unrestricted`。

### 匹配完整路径
默认情况下，*fd* 仅匹配每个文件的文件名。使用 `--full-path` 或 `-p` 选项可以匹配完整路径。

```bash
> fd -u -p -g '**/.git/config'
> fd -p '.*/lesson-\d+/[a-z]+.(jpg|png)'
```

<a id="command-execution"></a>
### 执行命令

除了显示搜索结果，你通常还希望对结果*执行操作*。`fd` 提供两种方式执行外部命令：

* `-x`/`--exec` 选项为*每个搜索结果*并行运行外部命令。
* `-X`/`--exec-batch` 选项只启动一次外部命令，并将*所有搜索结果作为参数*。

#### 示例

递归查找所有 zip 压缩包并解压：
``` bash
fd -e zip -x unzip
```
如果有 `file1.zip` 和 `backup/file2.zip` 两个文件，将分别执行 `unzip file1.zip` 和 `unzip backup/file2.zip`。如果文件发现得足够快，两个 `unzip` 进程会并行运行。

查找所有 `*.h` 和 `*.cpp` 文件，并使用 `clang-format -i` 原地自动格式化：
``` bash
fd -e h -e cpp -x clang-format -i
```
注意可以将 `clang-format` 的 `-i` 选项作为单独参数传入，因此我们把 `-x` 放在最后。

`-x` 后的任何位置参数都属于命令模板，而不是 `fd` 本身。如果还要传入模式或搜索路径，请将 `-x` 放在最后：
``` bash
fd pattern path -x echo
```

查找所有 `test_*.py` 文件并在喜欢的编辑器中打开：
``` bash
fd -g 'test_*.py' -X vim
```
注意这里使用大写 `-X` 以打开单个 `vim` 实例。如果找到 `test_basic.py` 和 `lib/test_advanced.py`，将运行 `vim test_basic.py lib/test_advanced.py`。

要查看文件权限、所有者、大小等详情，可以让 `fd` 对结果运行 `ls`：
``` bash
fd … -X ls -lhd --color=always
```
由于这种模式很有用，`fd` 提供了快捷方式：使用 `-l`/`--list-details` 执行同样的 `ls`，例如 `fd … -l`。

结合 [ripgrep](https://github.com/BurntSushi/ripgrep/)（`rg`）搜索某类文件内容时，`-X` 也很有用，例如搜索所有 C++ 源文件：
```bash
fd -e cpp -e cxx -e h -e hpp -X rg 'std::cout'
```

将所有 `*.jpg` 文件转换为 `*.png` 文件：
``` bash
fd -e jpg -x convert {} {.}.png
```
这里 `{}` 是搜索结果的占位符；`{.}` 类似，但不含文件扩展名。占位符语法详见下文。

使用 `-x` 从并行线程运行的命令，其终端输出不会交错或乱码，因此可以使用 `fd -x` 对处理许多文件的任务进行基本的并行化。例如，计算目录中每个文件的校验和：
``` bash
fd -tf -x md5sum > file_checksums.txt
```

#### 占位符语法

`-x` 和 `-X` 接受一系列参数作为*命令模板*（而非单个字符串）。如果要在命令模板后继续向 `fd` 添加选项，可以用 `\;` 终止模板。

例如，`fd -x echo \; pattern path` 将 `pattern path` 视为 `fd` 参数，而不是传给 `echo`。实际使用中，写成 `fd pattern path -x echo` 往往更清晰。

生成命令的语法类似 [GNU Parallel](https://www.gnu.org/software/parallel/)：

- `{}`：替换为搜索结果路径的占位符（`documents/images/party.jpg`）。
- `{.}`：类似 `{}`，但去掉文件扩展名（`documents/images/party`）。
- `{/}`：替换为搜索结果基本文件名的占位符（`party.jpg`）。
- `{//}`：所找到路径的父目录（`documents/images`）。
- `{/.}`：去掉扩展名的基本文件名（`party`）。

如果未包含占位符，*fd* 会自动在末尾添加 `{}`。

#### 并行与串行执行

对于 `-x`/`--exec`，可以通过 `-j`/`--threads` 选项控制并行任务数。使用 `--threads=1` 串行执行。

### 排除特定文件或目录

有时需要忽略特定子目录中的搜索结果。例如，可能想搜索所有隐藏文件和目录（`-H`），但排除 `.git` 目录中的所有匹配。可以使用 `-E`（或 `--exclude`）选项，并将任意 glob 模式作为参数：
``` bash
> fd -H -E .git …
```

也可用它跳过挂载目录：
``` bash
> fd -E /mnt/external-drive …
```

……或跳过某些文件类型：
``` bash
> fd -E '*.bak' …
```

要永久保留此类排除模式，可以创建 `.fdignore` 文件。它与 `.gitignore` 文件类似，但仅供 `fd` 使用。例如：
``` bash
> cat ~/.fdignore
/mnt/external-drive
*.bak
```

> [!NOTE]
> `fd` 也支持 `rg` 或 `ag` 等其他程序使用的 `.ignore` 文件。

如果希望 `fd` 在全局忽略这些模式，可以将它们放入 `fd` 的全局忽略文件。macOS 或 Linux 上通常位于 `~/.config/fd/ignore`，Windows 上通常位于 `%APPDATA%\fd\ignore`。

你可能希望在 `fd/ignore` 文件中加入 `.git/`，这样使用 `--hidden` 时就不会输出 `.git` 目录及其内容。

### 删除文件

可以使用 `fd` 删除与搜索模式匹配的所有文件和目录。如果只想删除文件，可以使用 `--exec-batch`/`-X` 调用 `rm`。例如，要递归删除所有 `.DS_Store` 文件，请运行：
``` bash
> fd -H '^\.DS_Store$' -tf -X rm
```
如果不确定，请始终先不带 `-X rm` 调用 `fd`。也可以使用 `rm` 的交互选项：
``` bash
> fd -H '^\.DS_Store$' -tf -X rm -i
```

如果还想删除某一类目录，可以使用相同的方法，但必须使用 `rm` 的 `--recursive`/`-r` 标志来删除目录。

> [!NOTE]
> 某些情况下使用 `fd … -X rm -r` 可能造成竞态条件：如果路径形如 `…/foo/bar/foo/…`，并且想删除所有名为 `foo` 的目录，外层 `foo` 可能先被删除，导致 `rm` 调用出现（无害的）“'foo/bar/foo': No such file or directory”错误。

<a id="command-line-options"></a>
### 命令行选项

以下是 `fd -h` 的输出。要查看完整选项及更详细的帮助文本，请使用 `fd --help`。

```
Usage: fd [OPTIONS] [pattern [path]...]

Arguments:
  [pattern]  the search pattern (a regular expression, unless '--glob' is used; optional)
  [path]...  the root directories for the filesystem search (optional)

Options:
  -H, --hidden                     Search hidden files and directories
  -I, --no-ignore                  Do not respect .(git|fd)ignore files
  -s, --case-sensitive             Case-sensitive search (default: smart case)
  -i, --ignore-case                Case-insensitive search (default: smart case)
  -g, --glob                       Glob-based search (default: regular expression)
  -a, --absolute-path              Show absolute instead of relative paths
  -l, --list-details               Use a long listing format with file metadata
  -L, --follow                     Follow symbolic links
  -p, --full-path                  Search full abs. path (default: filename only)
  -d, --max-depth <depth>          Set maximum search depth (default: none)
  -E, --exclude <glob>             Exclude entries that match the given glob pattern
  -t, --type <filetype>            Filter by type: file (f), directory (d/dir), symlink (l),
                                   executable (x), empty (e), socket (s), pipe (p), char-device
                                   (c), block-device (b)
  -e, --extension <ext>            Filter by extension
  -S, --size <size>                Limit results based on the size of files
      --changed-within <date|dur>  Filter by file modification time (newer than)
      --changed-before <date|dur>  Filter by file modification time (older than)
  -o, --owner <user:group>         Filter by owning user and/or group
      --format <fmt>               Print results according to template
  -x, --exec <cmd>...              Execute a command for each search result
  -X, --exec-batch <cmd>...        Execute a command with all search results at once
  -c, --color <when>               When to use colors [default: auto] [possible values: auto,
                                   always, never]
      --hyperlink[=<when>]         Add hyperlinks to output paths [default: never] [possible
                                   values: auto, always, never]
      --ignore-contain <name>      Ignore directories containing the named entry
  -h, --help                       Print help (see more with '--help')
  -V, --version                    Print version
```

上述帮助中的参数与选项说明：

- `[pattern]`：搜索模式；除非使用 `--glob`，否则为正则表达式，可省略。
- `[path]...`：文件系统搜索的根目录，可省略。
- `-H, --hidden`：搜索隐藏文件和目录。
- `-I, --no-ignore`：不遵循 .(git|fd)ignore 文件。
- `-s, --case-sensitive`：区分大小写搜索（默认智能大小写）。
- `-i, --ignore-case`：不区分大小写搜索（默认智能大小写）。
- `-g, --glob`：使用 glob 模式搜索（默认为正则表达式）。
- `-a, --absolute-path`：显示绝对路径而非相对路径。
- `-l, --list-details`：以包含文件元数据的长列表格式显示。
- `-L, --follow`：跟随符号链接。
- `-p, --full-path`：搜索完整绝对路径（默认仅文件名）。
- `-d, --max-depth <depth>`：设置最大搜索深度（默认不限制）。
- `-E, --exclude <glob>`：排除匹配给定 glob 模式的条目。
- `-t, --type <filetype>`：按类型过滤：文件 `f`、目录 `d/dir`、符号链接 `l`、可执行文件 `x`、空文件 `e`、套接字 `s`、管道 `p`、字符设备 `c`、块设备 `b`。
- `-e, --extension <ext>`：按扩展名过滤。
- `-S, --size <size>`：根据文件大小限制结果。
- `--changed-within <date|dur>`：按文件修改时间过滤，仅保留较新的文件。
- `--changed-before <date|dur>`：按文件修改时间过滤，仅保留较旧的文件。
- `-o, --owner <user:group>`：按所有用户和/或组过滤。
- `--format <fmt>`：根据模板输出结果。
- `-x, --exec <cmd>...`：对每个搜索结果执行命令。
- `-X, --exec-batch <cmd>...`：将所有结果一次性传给命令执行。
- `-c, --color <when>`：何时使用颜色；默认 `auto`，可选 `auto`、`always`、`never`。
- `--hyperlink[=<when>]`：为输出路径添加超链接；默认 `never`，可选 `auto`、`always`、`never`。
- `--ignore-contain <name>`：忽略包含指定名称条目的目录。
- `-h, --help`：输出帮助（使用 `--help` 查看更多）。
- `-V, --version`：输出版本。


选项也可以放在模式或路径之后。

<a id="benchmark"></a>
## 基准测试

下面在我的主目录中搜索以 `[0-9].jpg` 结尾的文件。该目录包含约 750,000 个子目录和约 400 万个文件。我使用 [hyperfine](https://github.com/sharkdp/hyperfine) 计算平均值并进行统计分析。以下基准测试使用“热”磁盘缓存（预填充缓存；“冷”磁盘缓存的结果显示相同趋势）。

先从 `find` 开始：
```
Benchmark 1: find ~ -iregex '.*[0-9]\.jpg$'
  Time (mean ± σ):     19.922 s ±  0.109 s
  Range (min … max):   19.765 s … 20.065 s
```

如果不必进行正则表达式搜索，`find` 会快得多：
```
Benchmark 2: find ~ -iname '*[0-9].jpg'
  Time (mean ± σ):     11.226 s ±  0.104 s
  Range (min … max):   11.119 s … 11.466 s
```

现在对 `fd` 做相同测试。注意 `fd` 默认执行正则搜索。为了公平比较，这里需要 `-u`/`--unrestricted` 选项，否则 `fd` 不必遍历隐藏目录和忽略路径（见下文）：
```
Benchmark 3: fd -u '[0-9]\.jpg$' ~
  Time (mean ± σ):     854.8 ms ±  10.0 ms
  Range (min … max):   839.2 ms … 868.9 ms
```
在这个特定示例中，`fd` 比 `find -iregex` 快约 **23 倍**，比 `find -iname` 快约 **13 倍**。顺便一提，两种工具找到了完全相同的 546 个文件 :smile:。

**注意**：这是在*某一台特定机器*上的*某一个特定*基准。尽管我们进行了很多不同的测试并发现结果一致，你的结果可能不同！我们鼓励大家自行尝试。所有必需脚本见[此仓库](https://github.com/sharkdp/fd-benchmarks)。

关于 *fd* 的速度，很大一部分功劳归于 [ripgrep](https://github.com/BurntSushi/ripgrep) 也使用的 `regex` 和 `ignore` crate（不妨看看！）。

<a id="troubleshooting"></a>
## 故障排查

### `fd` 找不到我的文件！

请记住，`fd` 默认忽略隐藏目录和文件，也忽略 `.gitignore` 文件中的模式。要确保找到所有可能的文件，请始终使用 `-u`/`--unrestricted`（或使用 `-HI` 启用隐藏和忽略的文件）：
``` bash
> fd -u …
```

还要记住，`fd` 默认只根据文件名搜索，不会将模式与完整路径比较。要根据完整路径搜索（类似 `find` 的 `-path` 选项），需要使用 `--full-path`（或 `-p`）。

### 彩色输出

`fd` 可以像 `ls` 一样按扩展名给文件着色。为此必须设置环境变量 [`LS_COLORS`](https://linux.die.net/man/5/dir_colors)。通常由 `dircolors` 命令设置该变量，它提供方便的配置格式来定义不同文件格式的颜色。大多数发行版应已设置 `LS_COLORS`。如果使用 Windows，或者希望寻找更完整或更丰富的替代方案，请参见[这里](https://github.com/sharkdp/vivid)、[这里](https://github.com/seebi/dircolors-solarized)或[这里](https://github.com/trapd00r/LS_COLORS)。

`fd` 也遵守 [`NO_COLOR`](https://no-color.org/) 环境变量。

### `fd` 似乎没有正确解释我的正则表达式

许多正则特殊字符（如 `[]`、`^`、`$` 等）在 shell 中也是特殊字符。如有疑问，请务必用单引号包围正则表达式：

``` bash
> fd '^[A-Z][0-9]+$'
```

如果模式以连字符开头，必须添加 `--` 表示命令行选项结束，否则模式会被解释为命令行选项。也可以使用只包含连字符的字符类：

``` bash
> fd -- '-pattern'
> fd '[-]pattern'
```

### `alias` 或 shell 函数显示“Command not found”

Shell 的 `alias` 和 shell 函数不能用于通过 `fd -x` 或 `fd -X` 执行命令。在 `zsh` 中可使用 `alias -g myalias="…"` 将别名设为全局；在 `bash` 中，可使用 `export -f my_function` 将函数提供给子进程，但仍需调用 `fd -x bash -c 'my_function "$1"' bash`。其他场景或 shell 可使用（临时）shell 脚本。

### `-x`/`-X` 中的占位符

根据所用 shell 的不同，可能需要给占位符（`{}`、`{/}`、`{//}`、`{.}`、`{/.}`）加引号，防止 shell 在 `fd` 之前解释它们。

### 预加载其他内存分配器时 `fd` 崩溃

发布的 Linux 二进制文件通过 `use-jemalloc` 特性使用 jemalloc，该特性对部分目标默认启用。编译时包含 jemalloc 又预加载另一分配器不受支持，例如：

``` bash
> LD_PRELOAD=/usr/bin/mimalloc fd
```

这会使两个分配器对内存分配的所有权产生分歧，导致 `fd` 崩溃。`fd` 无法检测并报告该问题，因为预加载库已经响应了它的第一次内存分配。若要使用预加载的分配器，请在不启用 jemalloc 的情况下构建：

``` bash
> cargo build --release --no-default-features
```

## 与其他程序集成

### 将 fd 与 `fzf` 配合使用

可以使用 *fd* 为命令行模糊查找器 [fzf](https://github.com/junegunn/fzf) 生成输入：
``` bash
export FZF_DEFAULT_COMMAND='fd --type file'
export FZF_CTRL_T_COMMAND="$FZF_DEFAULT_COMMAND"
```

然后可在终端输入 `vim <Ctrl-T>`，打开 fzf 并搜索 fd 的结果。

也可以跟随符号链接并包含隐藏文件，但排除 `.git` 目录：
``` bash
export FZF_DEFAULT_COMMAND='fd --type file --follow --hidden --exclude .git'
```

甚至可以通过以下设置在 fzf 内使用 fd 的彩色输出：
``` bash
export FZF_DEFAULT_COMMAND="fd --type file --color=always"
export FZF_DEFAULT_OPTS="--ansi"
```

更多详情参见 fzf README 的[技巧部分](https://github.com/junegunn/fzf#tips)。

### 将 fd 与 `rofi` 配合使用

[*rofi*](https://github.com/davatorium/rofi) 是图形化启动菜单应用，可从*标准输入*读取内容创建菜单。将 `fd` 输出通过管道传给 `rofi` 的 `-dmenu` 模式，可以创建支持模糊搜索的文件和目录列表。

#### 示例

在 `$HOME` 目录下创建一个不区分大小写、可搜索和多选的 *PDF* 文件列表，并用配置的 PDF 查看器打开所选文件。要列出所有文件类型，请去掉 `-e pdf` 参数。

``` bash
fd --type f -e pdf . $HOME | rofi -keep-right -dmenu -i -p FILES -multi-select | xargs -I {} xdg-open {}
```

要修改 rofi 显示的列表，请为 `fd` 命令添加参数。要修改 rofi 的搜索行为，请为 `rofi` 命令添加参数。

### 将 fd 与 `emacs` 配合使用

Emacs 软件包 [find-file-in-project](https://github.com/technomancy/find-file-in-project) 可以使用 *fd* 查找文件。

安装 `find-file-in-project` 后，将 `(setq ffip-use-rust-fd t)` 添加到 `~/.emacs` 或 `~/.emacs.d/init.el`。

在 Emacs 中运行 `M-x find-file-in-project-by-selected` 查找匹配文件。也可运行 `M-x find-file-in-project` 列出所有文件并选择。

### 将输出打印为树形结构

要将 `fd` 的输出格式化为文件树，可以将 `tree` 命令与 `--fromfile` 一起使用：
```bash
❯ fd | tree --fromfile
```

这可能比单独运行 `tree` 更有用，因为 `tree` 默认不忽略任何文件，也不像 `fd` 一样提供丰富选项来控制输出内容：
```bash
❯ fd --extension rs | tree --fromfile
.
├── build.rs
└── src
    ├── app.rs
    └── error.rs
```

在 bash 及类似 shell 中可以创建别名：
```bash
❯ alias as-tree='tree --fromfile'
```

### 将 fd 与 `xargs` 或 `parallel` 配合使用

注意，`fd` 通过 `-x`/`--exec` 和 `-X`/`--exec-batch` 内置了[命令执行](#command-execution)功能。如果更喜欢，也可以与 `xargs` 结合使用：
``` bash
> fd -0 -e rs | xargs -0 wc -l
```
这里 `-0` 选项让 *fd* 使用 NULL 字符而非换行符分隔搜索结果。同样，`xargs` 的 `-0` 选项让它按此方式读取输入。

<a id="installation"></a>
## 安装

[![Packaging status](https://repology.org/badge/vertical-allrepos/fd-find.svg)](https://repology.org/project/fd-find/versions)

### Ubuntu

*……以及其他基于 Debian 的 Linux 发行版。*

如果使用 Ubuntu 19.04（Disco Dingo）或更新版本，可以安装[官方维护的软件包](https://packages.ubuntu.com/fd-find)：
```
apt install fd-find
```
注意二进制程序名为 `fdfind`，因为名称 `fd` 已被另一个软件包占用。建议安装后执行 `ln -s $(which fdfind) ~/.local/bin/fd` 创建指向 `fd` 的链接，以便按本文档方式使用。请确保 `$HOME/.local/bin` 位于 `$PATH` 中。

如果使用较旧版本的 Ubuntu，可以从[发行页面](https://github.com/sharkdp/fd/releases)下载最新 `.deb` 包并安装：
``` bash
dpkg -i fd_9.0.0_amd64.deb # adapt version number and architecture
```
示例注释提示：请调整版本号和架构。

注意，本项目发行页面上的 .deb 包仍将可执行文件命名为 `fd`。

### Debian

如果使用 Debian Buster 或更新版本，可以安装[官方维护的 Debian 软件包](https://tracker.debian.org/pkg/rust-fd-find)：
```
apt-get install fd-find
```
注意二进制程序名为 `fdfind`，因为名称 `fd` 已被另一个软件包占用。建议安装后执行 `ln -s $(which fdfind) ~/.local/bin/fd` 创建指向 `fd` 的链接，以便按本文档方式使用。请确保 `$HOME/.local/bin` 位于 `$PATH` 中。

注意，本项目发行页面上的 .deb 包仍将可执行文件命名为 `fd`。

### Fedora

从 Fedora 28 起，可以从官方软件源安装 `fd`：
``` bash
dnf install fd-find
```

### Alpine Linux

如果启用了相应仓库，可以从官方源安装 [fd 软件包](https://pkgs.alpinelinux.org/packages?name=fd)：
```
apk add fd
```

### Arch Linux

可以从官方仓库安装 [fd 软件包](https://www.archlinux.org/packages/extra/x86_64/fd/)：
```
pacman -S fd
```
也可以从 [AUR](https://aur.archlinux.org/packages/fd-git) 安装 fd。

### Gentoo Linux

可以使用官方仓库的 [fd ebuild](https://packages.gentoo.org/packages/sys-apps/fd)：
```
emerge -av fd
```

### openSUSE Linux

可以从官方仓库安装 [fd 软件包](https://software.opensuse.org/package/fd)：
```
zypper in fd
```

### Void Linux

可以通过 xbps-install 安装 `fd`：
```
xbps-install -S fd
```

### ALT Linux

可以从官方仓库安装 [fd 软件包](https://packages.altlinux.org/en/sisyphus/srpms/fd/)：
```
apt-get install fd
```

### Solus

可以从官方仓库安装 [fd 软件包](https://github.com/getsolus/packages/tree/main/packages/f/fd)：
```
eopkg install fd
```

### RedHat Enterprise Linux (RHEL) 8/9/10、Almalinux 8/9/10、EuroLinux 8/9 或 Rocky Linux 8/9/10

可以从 Fedora Copr 安装 [`fd` 软件包](https://copr.fedorainfracloud.org/coprs/tkbcopr/fd/)。

```bash
dnf copr enable tkbcopr/fd
dnf install fd
```

EPEL8/9 仓库还提供名为 `fd-find` 的另一版本，使用[较慢的](https://github.com/sharkdp/fd/pull/481#issuecomment-534494592) malloc [替代 jemalloc](https://bugzilla.redhat.com/show_bug.cgi?id=2216193#c1)。

### macOS

可以使用 [Homebrew](https://formulae.brew.sh/formula/fd) 安装 `fd`：
```
brew install fd
```

……或者使用 MacPorts：
```
port install fd
```

### Windows

可以从[发行页面](https://github.com/sharkdp/fd/releases)下载预构建二进制文件。

也可以通过 [Scoop](http://scoop.sh) 安装 `fd`：
```
scoop install fd
```

或者通过 [Chocolatey](https://chocolatey.org)：
```
choco install fd
```

或者通过 [Winget](https://learn.microsoft.com/en-us/windows/package-manager/)：
```
winget install sharkdp.fd
```

### GuixOS

可以从官方仓库安装 [fd 软件包](https://guix.gnu.org/en/packages/fd-8.1.1/)：
```
guix install fd
```

### Mise

可以使用 [mise](https://github.com/jdx/mise)，通过类似以下命令安装 `fd`：
```
mise use -g fd@latest
```

### NixOS / 通过 Nix

可以使用 [Nix 包管理器](https://nixos.org/nix/)安装 `fd`：
```
nix-env -i fd
```

### 通过 Flox

可以使用 [Flox](https://flox.dev) 将 `fd` 安装到 Flox 环境：
```
flox install fd
```

### FreeBSD

可以从官方仓库安装 [fd-find 软件包](https://www.freshports.org/sysutils/fd)：
```
pkg install fd-find
```

### 从 npm 安装

在 Linux 和 macOS 上，可以安装 [fd-find](https://npm.im/fd-find) 软件包：

```
npm install -g fd-find
```

### 从源代码安装

可以通过 Rust 包管理器 [cargo](https://github.com/rust-lang/cargo) 安装 *fd*：
```
cargo install fd-find
```
注意需要 Rust *1.90.0* 或更高版本。

构建时还需要 `make`。

### 从二进制文件安装

[发行页面](https://github.com/sharkdp/fd/releases)包含适用于 Linux、macOS 和 Windows 的预编译二进制文件。也提供静态链接二进制文件：请查找文件名中包含 `musl` 的归档。

## 开发

以下命令依次克隆、构建、运行单元及集成测试并安装。

```bash
git clone https://github.com/sharkdp/fd

# Build
cd fd
cargo build

# Run unit tests and integration tests
cargo test

# Install
cargo install --path .
```

### 补全

#### 从发行归档获取

预构建补全文件包含在 [Releases 页面](https://github.com/sharkdp/fd/releases)上的发行归档（`.tar.gz`/`.zip`）的 `autocomplete` 目录中。使用补全的方法如下：

- **bash**：在 `~/.bashrc` 中加载 `fd.bash`，或将其放入会自动加载的目录。
- **zsh**：将 `_fd` 移到 `fpath` 中的目录（例如 `~/.zfunc`）。
- **fish**：将 `fd.fish` 复制到 `~/.config/fish/completions/`。
- **powershell**：从一个[配置文件脚本](https://learn.microsoft.com/en-us/powershell/scripting/learn/shell/creating-profiles?view=powershell-7.5)加载 `_fd.ps1`。

#### 从 fd 生成

也可以直接使用 `fd --gen-completions <shell>` 生成补全：
下面依次展示 Bash、Zsh（请确保 `~/.zfunc` 在 `fpath` 中）、Fish 和 PowerShell 的补全生成命令。

```bash
# Bash
fd --gen-completions bash > ~/.local/share/bash-completion/completions/fd

# Zsh (ensure ~/.zfunc is in your fpath)
fd --gen-completions zsh > ~/.zfunc/_fd

# Fish
fd --gen-completions fish > ~/.config/fish/completions/fd.fish

# PowerShell
fd --gen-completions powershell >> $PROFILE
```

## 维护者

- [sharkdp](https://github.com/sharkdp)
- [tmccombs](https://github.com/tmccombs)
- [tavianator](https://github.com/tavianator)

## 许可

`fd` 按 MIT License 和 Apache License 2.0 两种许可的条款分发。

许可详情参见 [LICENSE-APACHE](https://github.com/sharkdp/fd/blob/cc4e40fa4e32d61733cd72926e233a3fb6c8d56d/LICENSE-APACHE) 和 [LICENSE-MIT](https://github.com/sharkdp/fd/blob/cc4e40fa4e32d61733cd72926e233a3fb6c8d56d/LICENSE-MIT) 文件。
