# uv

<a href="https://pypi.python.org/pypi/uv"><img src="https://img.shields.io/pypi/v/uv.svg" alt="Latest PyPI version" /></a>
<a href="https://pypi.python.org/pypi/uv"><img src="https://img.shields.io/pypi/pyversions/uv.svg" alt="Supported Python versions" /></a>
<a href="https://discord.gg/astral-sh"><img src="https://img.shields.io/badge/Discord-%235865F2.svg?logo=discord&logoColor=white" alt="Discord" /></a>

一个用 Rust 编写的高速 Python 包与项目管理器。

<p align="center">
  <picture align="center">
    <source media="(prefers-color-scheme: dark)" srcset="https://github.com/astral-sh/uv/assets/1309177/03aa9163-1c79-4a87-a31d-7a9311ed9310">
    <source media="(prefers-color-scheme: light)" srcset="https://github.com/astral-sh/uv/assets/1309177/629e59c0-9c6e-4013-9ad4-adb2bcf5080d">
    <img alt="Shows a bar chart with benchmark results." src="https://github.com/astral-sh/uv/assets/1309177/629e59c0-9c6e-4013-9ad4-adb2bcf5080d">
  </picture>
</p>

<p align="center">
  <i>使用预热缓存安装 <a href="https://trio.readthedocs.io/">Trio</a> 的依赖。</i>
</p>

## 亮点

- 一个工具替代 `pip`、`pip-tools`、`pipx`、`poetry`、`pyenv`、`twine`、`virtualenv` 等。
- 比 `pip` [快 10–100 倍](https://github.com/astral-sh/uv/blob/main/BENCHMARKS.md)。
- 提供[全面的项目管理](#项目)，支持[通用锁文件](https://docs.astral.sh/uv/concepts/projects/layout#the-lockfile)。
- [运行脚本](#脚本)，支持[内联依赖元数据](https://docs.astral.sh/uv/guides/scripts#declaring-script-dependencies)。
- [安装和管理](#python-版本) Python 版本。
- [运行和安装](#工具)以 Python 包形式发布的工具。
- 包含[pip 兼容接口](#pip-接口)，在熟悉的命令行界面中提升性能。
- 支持 Cargo 风格的[工作空间](https://docs.astral.sh/uv/concepts/projects/workspaces)，适用于可扩展项目。
- 节省磁盘空间，使用[全局缓存](https://docs.astral.sh/uv/concepts/cache)对依赖进行去重。
- 无需 Rust 或 Python，可通过 `curl` 或 `pip` 安装。
- 支持 macOS、Linux 和 Windows。

uv 由 [Astral](https://astral.sh) 支持，Astral 也是 [Ruff](https://github.com/astral-sh/ruff) 和 [ty](https://github.com/astral-sh/ty) 的创建者。

## 安装

使用独立安装程序安装 uv：

```bash
# On macOS and Linux.
curl -LsSf https://astral.sh/uv/install.sh | sh
```

```bash
# On Windows.
powershell -ExecutionPolicy ByPass -c "irm https://astral.sh/uv/install.ps1 | iex"
```

或者，从 [PyPI](https://pypi.org/project/uv/) 安装：

```bash
# With pip.
pip install uv
```

```bash
# Or pipx.
pipx install uv
```

如果通过独立安装程序安装，uv 可以自行更新到最新版本：

```bash
uv self update
```

详细信息及其他安装方法请参阅[安装文档](https://docs.astral.sh/uv/getting-started/installation/)。

## 文档

uv 的文档位于 [docs.astral.sh/uv](https://docs.astral.sh/uv)。

此外，可以使用 `uv help` 查看命令行参考文档。

## 功能

### 项目

uv 管理项目依赖和环境，支持锁文件、工作空间等，类似于 `rye` 或 `poetry`：

```console
$ uv init example
Initialized project `example` at `/home/user/example`

$ cd example

$ uv add ruff
Creating virtual environment at: .venv
Resolved 2 packages in 170ms
   Built example @ file:///home/user/example
Prepared 2 packages in 627ms
Installed 2 packages in 1ms
 + example==0.1.0 (from file:///home/user/example)
 + ruff==0.5.0

$ uv run ruff check
All checks passed!

$ uv lock
Resolved 2 packages in 0.33ms

$ uv sync
Resolved 2 packages in 0.70ms
Checked 1 package in 0.02ms
```

入门请参阅[项目文档](https://docs.astral.sh/uv/guides/projects/)。

uv 还支持构建和发布项目，即使这些项目不是由 uv 管理的。更多信息请参阅[发布指南](https://docs.astral.sh/uv/guides/publish/)。

### 脚本

uv 管理单文件脚本的依赖和环境。

创建新脚本并添加声明其依赖的内联元数据：

```console
$ echo 'import requests; print(requests.get("https://astral.sh"))' > example.py

$ uv add --script example.py requests
Updated `example.py`
```

然后在隔离的虚拟环境中运行脚本：

```console
$ uv run example.py
Reading inline script metadata from: example.py
Installed 5 packages in 12ms
<Response [200]>
```

入门请参阅[脚本文档](https://docs.astral.sh/uv/guides/scripts/)。

### 工具

uv 执行和安装 Python 包提供的命令行工具，类似于 `pipx`。

使用 `uvx`（`uv tool run` 的别名）在临时环境中运行工具：

```console
$ uvx pycowsay 'hello world!'
Resolved 1 package in 167ms
Installed 1 package in 9ms
 + pycowsay==0.0.0.2
  """

  ------------
< hello world! >
  ------------
   \   ^__^
    \  (oo)\_______
       (__)\       )\/\
           ||----w |
           ||     ||
```

使用 `uv tool install` 安装工具：

```console
$ uv tool install ruff
Resolved 1 package in 6ms
Installed 1 package in 2ms
 + ruff==0.5.0
Installed 1 executable: ruff

$ ruff --version
ruff 0.5.0
```

入门请参阅[工具文档](https://docs.astral.sh/uv/guides/tools/)。

### Python 版本

uv 安装 Python，并支持快速切换版本。

安装多个 Python 版本：

```console
$ uv python install 3.12 3.13 3.14
Installed 3 versions in 972ms
 + cpython-3.12.12-macos-aarch64-none (python3.12)
 + cpython-3.13.9-macos-aarch64-none (python3.13)
 + cpython-3.14.0-macos-aarch64-none (python3.14)

```

按需下载 Python 版本：

```console
$ uv venv --python 3.12.0
Using Python 3.12.0
Creating virtual environment at: .venv
Activate with: source .venv/bin/activate

$ uv run --python pypy@3.8 -- python --version
Python 3.8.16 (a9dbdca6fc3286b0addd2240f11d97d8e8de187a, Dec 29 2022, 11:45:30)
[PyPy 7.3.11 with GCC Apple LLVM 13.1.6 (clang-1316.0.21.2.5)] on darwin
Type "help", "copyright", "credits" or "license" for more information.
>>>>
```

在当前目录中使用特定 Python 版本：

```console
$ uv python pin 3.11
Pinned `.python-version` to `3.11`
```

入门请参阅 [Python 安装文档](https://docs.astral.sh/uv/guides/install-python/)。

### pip 接口

uv 为常用的 `pip`、`pip-tools` 和 `virtualenv` 命令提供可直接替换的接口。

uv 通过依赖版本覆盖、与平台无关的解析、可复现解析及替代解析策略等高级功能扩展这些接口。

通过 `uv pip` 接口，无需改变现有工作流即可迁移到 uv，并体验 10–100 倍的速度提升。

将依赖要求编译为与平台无关的 requirements 文件：

```console
$ uv pip compile requirements.in \
   --universal \
   --output-file requirements.txt
Resolved 43 packages in 12ms
```

创建虚拟环境：

```console
$ uv venv
Using Python 3.12.3
Creating virtual environment at: .venv
Activate with: source .venv/bin/activate
```

安装已锁定的依赖：

```console
$ uv pip sync requirements.txt
Resolved 43 packages in 11ms
Installed 43 packages in 208ms
 + babel==2.15.0
 + black==24.4.2
 + certifi==2024.7.4
 ...
```

入门请参阅 [pip 接口文档](https://docs.astral.sh/uv/pip/index/)。

## 参与贡献

我们热衷于支持各种经验水平的贡献者，欢迎你参与项目。入门请参阅[贡献指南](https://github.com/astral-sh/uv?tab=contributing-ov-file#contributing)。

## FAQ

#### uv 如何发音？

发音为 “you - vee” ([`/juː viː/`](https://en.wikipedia.org/wiki/Help:IPA/English#Key))

#### uv 应如何书写？

请直接写作“uv”。详情请参阅[风格指南](https://github.com/astral-sh/uv/blob/3db723cf104c685b9f2f31a26834b7d417db652f/STYLE.md#styling-uv)。

#### uv 支持哪些平台？

请参阅 uv 的[平台支持](https://docs.astral.sh/uv/reference/platforms/)文档。

#### uv 可以用于生产环境吗？

可以。uv 是稳定的，并已广泛用于生产环境。详情请参阅 uv 的[版本策略](https://docs.astral.sh/uv/reference/versioning/)文档。

## 致谢

uv 的依赖解析器在底层使用 [PubGrub](https://github.com/pubgrub-rs/pubgrub)。我们感谢 PubGrub 维护者，尤其是 [Jacob Finkelman](https://github.com/Eh2406) 的支持。

uv 的 Git 实现基于 [Cargo](https://github.com/rust-lang/cargo)。

uv 的一些优化受到 [pnpm](https://pnpm.io/)、[Orogene](https://github.com/orogene/orogene) 和 [Bun](https://github.com/oven-sh/bun) 出色工作的启发。我们还从 Nathaniel J. Smith 的 [Posy](https://github.com/njsmith/posy) 学到很多，并改造了其 [trampoline](https://github.com/njsmith/posy/tree/main/src/trampolines/windows-trampolines/posy-trampoline) 以支持 Windows。

## 许可证

uv 可由用户任选以下一种许可证：

- Apache License, Version 2.0， ([LICENSE-APACHE](https://github.com/astral-sh/uv/blob/3db723cf104c685b9f2f31a26834b7d417db652f/LICENSE-APACHE) or
  <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT 许可证 ([LICENSE-MIT](https://github.com/astral-sh/uv/blob/3db723cf104c685b9f2f31a26834b7d417db652f/LICENSE-MIT) or <https://opensource.org/licenses/MIT>)



除非你明确另行声明，否则你有意提交以纳入 uv 的任何贡献（按 Apache-2.0 许可证定义）都将如上采用双重许可，不附加其他条款或条件。

<div align="center">
  <a target="_blank" href="https://astral.sh" style="background:none">
    <img src="https://raw.githubusercontent.com/astral-sh/uv/3db723cf104c685b9f2f31a26834b7d417db652f/assets/svg/Astral.svg" alt="Made by Astral">
  </a>
</div>
