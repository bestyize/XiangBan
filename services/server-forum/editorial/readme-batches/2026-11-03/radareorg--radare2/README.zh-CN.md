> 译注：本文由 Terra 对照[固定版本 README](https://github.com/radareorg/radare2/blob/70c2da061abbbca8fe51ea742c12b7e24483d8be/README.md)全文翻译；原文称项目以 LGPLv3 分发，插件可能采用不同许可证，许可依据与文本见 [COPYING.md](https://github.com/radareorg/radare2/blob/70c2da061abbbca8fe51ea742c12b7e24483d8be/COPYING.md)及[上游 LGPLv3 文本](https://github.com/radareorg/radare2/blob/70c2da061abbbca8fe51ea742c12b7e24483d8be/doc/licenses/LGPL-3.0-only.txt)。原文未说明是否允许使用后续版本。原文贡献章节锚点不存在，中文链接已指向同版本 CONTRIBUTING.md。代码示例保留原文。

<a href="https://radare.org/"><img border=0 src="https://raw.githubusercontent.com/radareorg/radare2/70c2da061abbbca8fe51ea742c12b7e24483d8be/doc/images/r2emoji.png" alt="screenshot" align="left" width="128px"></a>

## Radare2：面向 Unix 爱好者的自由逆向工程框架

[![Tests Status](https://github.com/radareorg/radare2/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/radareorg/radare2/actions/workflows/ci.yml?query=branch%3Amaster) [![build](https://github.com/radareorg/radare2/actions/workflows/build.yml/badge.svg?branch=master)](https://github.com/radareorg/radare2/actions/workflows/build.yml?query=branch%3Amaster) [![tcc](https://github.com/radareorg/radare2/actions/workflows/tcc.yml/badge.svg?branch=master)](https://github.com/radareorg/radare2/actions/workflows/tcc.yml)
[![CII Best Practices](https://bestpractices.coreinfrastructure.org/projects/741/badge)](https://bestpractices.coreinfrastructure.org/projects/741) [![Build Status](https://scan.coverity.com/projects/416/badge.svg)](https://scan.coverity.com/projects/416) [![Discord](https://badgen.net/discord/members/YBey7CR9jf)](https://discord.gg/YBey7CR9jf)

当前 git `master` 分支为 `6.2.5`，下一个版本将是 `6.2.6`。

### 简介

r2 是 radare 的完全重写版本，提供一系列库、工具和插件，便于执行逆向工程任务。项目以 LGPLv3 分发，各插件可能采用不同许可证（详情参见 `r2 -Lj`）。

**radare** 项目最初是一个面向取证的简单命令行十六进制编辑器。如今，r2 已成为功能完整的底层命令行工具，支持通过内置 JavaScript 解释器或 r2pipe 编写脚本。

r2 可以编辑本地硬盘文件、查看内核内存，并在本地或通过远程 gdb/windbg 服务器调试程序。r2 对多种架构的支持使你能够分析、模拟、调试、修改和反汇编二进制文件。

<p align="center">
<a href="https://www.radare.org/"><img src="https://raw.githubusercontent.com/radareorg/radare2/70c2da061abbbca8fe51ea742c12b7e24483d8be/doc/images/shot.png" alt="screenshot" align="center" border=0 width="600px"></a>
</p>

## 安装

下载最新[发布](https://github.com/radareorg/radare2/releases)的二进制文件。

推荐从 Git 仓库源码安装 radare2：

```sh
git clone https://github.com/radareorg/radare2
radare2/sys/install.sh
```

* 运行 `sys/install.sh` 进行默认的 acr+make+符号链接安装。
* 支持 meson/ninja（也可用 muon/samu）和 make 构建。

### Nix

radare2 在 [`dist/nix`](https://github.com/radareorg/radare2/blob/70c2da061abbbca8fe51ea742c12b7e24483d8be/dist/nix/README.md) 下提供 Nix 软件包。

### 从源码构建

* 可从 `git` 安装 r2，也可通过 `pip` 使用 `r2env` 安装。
* Windows 构建需要 meson，以及 msvc 或 mingw 编译器。
* 卸载当前构建的 r2：运行 `make uninstall`。
* 卸载系统中 **所有** r2 安装：运行 `sudo make purge`。

在 Windows 上使用 .bat 脚本和 msvc：

```bat
preconfigure.bat       REM setup python, meson, ninja
configure.bat          REM run meson b + vs project
make.bat               REM run ninja -C b
prefix\bin\radare2.exe
```

## 常用插件

使用 `r2pm` 工具可以浏览和安装许多使用 radare2 的插件和工具。

* [iaito](https://github.com/radareorg/iaito): 官方 Qt 图形界面
* [keystone](https://github.com/radareorg/radare2-extras/tree/master/keystone) 使用 Keystone 库组装指令
* [decai](https://github.com/radareorg/r2ai) 基于 AI 的反编译器
* [r2ai](https://github.com/radareorg/r2ai) 在 r2 内使用 Llama 在本机运行语言模型！
* [r2dec](https://github.com/wargio/r2dec-js): 用 JavaScript 编写、基于 r2 的反编译器，通过 `pdd` 命令使用
* [r2diaphora](https://github.com/FernandoDoming/r2diaphora): [Diaphora](https://github.com/joxeankoret/diaphora) 基于 radare2 的二进制差异比较引擎
* [r2flutter](https://github.com/trufae/r2flutter): 分析 Android 和 iOS 应用的 Dart/Flutter AOT 快照
* [r2frida](https://github.com/nowsecure/r2frida): frida IO 插件。通过 `r2 frida://0` 启动 r2 使用
* [r2ghidra](https://github.com/radareorg/r2ghidra): 独立原生 Ghidra 反编译器，通过 `pdg` 使用
* [r4ghidra](https://github.com/radareorg/r4ghidra): 在 Ghidra 中体验 radare
* [r2papi](https://github.com/radareorg/radare2-r2papi) 基于 r2pipe 的高级 API
* [r2pipe](https://github.com/radareorg/radare2-r2pipe) 使用任意编程语言为 radare2 编写脚本
* [r2poke](https://github.com/radareorg/radare2-extras/tree/master/r2poke) 集成 GNU/Poke，扩展二进制解析能力
* [goresym](https://github.com/hanemile/radare2-GoReSym): 将 GoReSym 符号导入为标志
* [r2yara](https://github.com/radareorg/r2yara) 从 r2 运行 Yara 或从 Yara 使用 r2 原语
* [radius2](https://github.com/radareorg/radius2): 基于 boolector 和 esil 的快速符号执行引擎
* [r2sarif](https://github.com/radareorg/r2sarif) 导入、扩展和导出 SARIF 文档
* [r2hermes](https://github.com/radareorg/r2hermes) React Native Hermes 字节码反汇编器和分析器
* [r2renef](https://github.com/Ahmeth4n/r2renef) Radare2 的 Renef IO 插件：动态 Android 插桩
* [r2unity](https://github.com/radareorg/r2unity): 从 radare2 检查 Unity IL2CPP 元数据和原生二进制文件
* [warrp](https://github.com/radareorg/warrp) 支持 Binary Ninja WARP 格式的原生 radare2 插件

## 使用

以下是使用 r2 的入门步骤；详情请阅读手册或查找教程。

```sh
$ r2 /bin/ls   # open file in read-only
> aaa          # analyse the program (r2 -A)
> afl          # list all functions (try aflt, aflm)
> px 32        # print 32 byte hexdump current block
> s sym.main   # seek to main (using flag name)
> f~foo        # filter flags matching 'foo' (internal |grep)
> iS;is        # list sections and symbols (rabin2 -Ss)
> pdf; agf     # disassembly and ascii-art function graph
> oo+;w hello  # reopen in read-write and write a string
> ?*~...       # interactive filter in all command help
> q            # quit
```

r2 默认包含许多插件，也可使用 [r2pm](https://github.com/radareorg/radare2-pm) 包管理器扩展其功能。

```sh
r2pm -s <word>  # search packages matching a word
r2pm -Uci <pkg> # update database and clean install a package
r2pm -u <pkg>   # uninstall the given package
r2pm -l <pkg>   # list installed packages
```

## 资源

* [Official Book](https://book.rada.re): 阅读 r2 用法
* [COMMUNITY.md](https://github.com/radareorg/radare2/blob/70c2da061abbbca8fe51ea742c12b7e24483d8be/COMMUNITY.md): 社区参与及宽松的行为指南
* [CONTRIBUTING.md](https://github.com/radareorg/radare2/blob/70c2da061abbbca8fe51ea742c12b7e24483d8be/CONTRIBUTING.md): 问题报告和贡献信息。另请参阅[贡献指南](https://github.com/radareorg/radare2/blob/70c2da061abbbca8fe51ea742c12b7e24483d8be/CONTRIBUTING.md)
* [DEVELOPERS.md](https://github.com/radareorg/radare2/blob/70c2da061abbbca8fe51ea742c12b7e24483d8be/DEVELOPERS.md): r2 开发指南
* [SECURITY.md](https://github.com/radareorg/radare2/blob/70c2da061abbbca8fe51ea742c12b7e24483d8be/SECURITY.md): 报告漏洞的说明
* [USAGE.md](https://github.com/radareorg/radare2/blob/70c2da061abbbca8fe51ea742c12b7e24483d8be/USAGE.md): 一些示例命令
* [INSTALL.md](https://github.com/radareorg/radare2/blob/70c2da061abbbca8fe51ea742c12b7e24483d8be/INSTALL.md): 使用 make 或 meson 安装的说明

## 文档

可观看 [r2con](https://rada.re/con) 的 [YouTube 演讲](https://www.youtube.com/c/r2con)了解更多 r2 内容。此外还有许多博客、演示文稿和 [radare2 官方手册](https://book.rada.re)；也欢迎加入官方聊天渠道提出问题或反馈。

## 社区

* 网站： [https://www.radare.org/](https://www.radare.org/)
* Discord：[服务器](https://discord.gg/YBey7CR9jf)
* Mastodon: [@radareorg](https://infosec.exchange/@radareorg)
* Telegram：[主](https://t.me/radare)和[旁支](https://t.me/radare_side)频道
* [irc.libera.chat](https://libera.chat): `#radare`, `#radare_side`
* [Matrix](https://matrix.to/#/#radare:matrix.org): `#radare:matrix.org`

# 支持的平台

## 操作系统

Windows（x86/x64/arm64 自 XP 起支持）、 Linux, Darwin, GNU/Hurd, Apple 的 {Mac,i,iPad,watch}OS、
Android, Wasmer, [Dragonfly, Net, Free, Open] BSD, Z/OS, QNX, SerenityOS, Solaris, AIX,
Haiku, Vinix, FirefoxOS.

## 架构

i386, x86-64, Alpha, ARM, AVR, BPF, sBPF, MIPS, PowerPC, SPARC, RISC-V, SH, m68k,
S390, XCore, CR16, HPPA, ARC, Blackfin, Z80, H8/300, V810, PDP11, m680x, V850,
CRIS, XAP (CSR), PIC, LM32, 8051, 6502, i4004, i8080, Propeller, EVM, OR1K
Tricore, CHIP-8, LH5801, T8200, GameBoy, SNES, SPC700, MSP430, Xtensa, xcore,
NIOS II, Java, Dalvik, Pickle, WebAssembly, MSIL, EBC, TMS320 (c54x, c55x,
c55+, c64x), Hexagon, Brainfuck, Malbolge, whitespace, DCPU16, LANAI, lm32,
MSIL, InfernoDis, UXN, Cosmac, PythonBytecode, Sharp sm5xx MCPus, FreeScale QIQa,
MCORE, mcs96, RSP, SuperH-4, VAX, KVX, Am29000, LOONGARCH, JDH8, s390x, STM8.

## 文件格式

ELF, Mach-O, Fatmach-O, PE, PE+, MZ, COFF, XCOFF, OMF, TE, XBE, SEP64, BIOS/UEFI, BFLT, Pebble apps,
Apple Dyldcache、Kernelcache、MCLF trustlet、DEX、ART、Java class、Android 启动镜像、Plan9 可执行文件、Amiga HUNK，
ZIMG, MBN/SBL bootloader, ELF coredump, MDMP (Windows minidump), PDP11, XTAC, CGC, DotNet,
WASM（WebAssembly 二进制）、Commodore VICE 模拟器、 DOL, QNX, WAD, OFF, TIC-80,
GB/GBA、PSX、PRG（C64）、Apple Classic PEF、NDS、z64 和 N3DS
...

## 文件系统

在用户空间挂载多种文件系统，例如 NTFS、FAT、HFS+、EXT、APFS、BeosFS、BFS、MINIX、ReiserFS、SFS、ubifs、ufs、jfs、ISO9660、UDF、xfs。

## 软件包状态

* [![Snap package](https://snapcraft.io/radare2/badge.svg)](https://snapcraft.io/radare2)
* [![Termux package](https://repology.org/badge/version-for-repo/termux/radare2.svg)](https://repology.org/project/radare2/versions)
* [![Alpine Linux 3.23 package](https://repology.org/badge/version-for-repo/alpine_3_23/radare2.svg)](https://repology.org/project/radare2/versions)  [![Alpine Linux 3.22 package](https://repology.org/badge/version-for-repo/alpine_3_22/radare2.svg)](https://repology.org/project/radare2/versions) 
* [![Arch package](https://repology.org/badge/version-for-repo/arch/radare2.svg)](https://repology.org/project/radare2/versions)
* [![Fedora 42](https://repology.org/badge/version-for-repo/fedora_42/radare2.svg)](https://repology.org/project/radare2/versions) [![Fedora 41](https://repology.org/badge/version-for-repo/fedora_41/radare2.svg)](https://repology.org/project/radare2/versions)
* [![Homebrew package](https://repology.org/badge/version-for-repo/homebrew/radare2.svg)](https://repology.org/project/radare2/versions) [![MacPorts package](https://repology.org/badge/version-for-repo/macports/radare2.svg)](https://repology.org/project/radare2/versions)
* [![Haiku Ports](https://repology.org/badge/version-for-repo/haikuports_master/radare2.svg)](https://repology.org/project/radare2/versions) [![Void Linux](https://repology.org/badge/version-for-repo/void_x86_64/radare2.svg)](https://repology.org/project/radare2/versions)
