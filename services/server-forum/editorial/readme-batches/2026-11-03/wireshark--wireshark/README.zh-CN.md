基本信息
-------------------

Wireshark 是一款适用于 Linux、macOS、*BSD 和其他 Unix 及类 Unix 操作系统以及 Windows 的网络流量分析器，也称“嗅探器”。它使用图形用户界面库 Qt，并使用 libpcap 和 npcap 作为数据包捕获与过滤库。

Wireshark 发行包还包含 TShark，这是一款面向行的嗅探器（类似 Sun 的 snoop 或 tcpdump），使用与 Wireshark 相同的协议解析、捕获文件读写和数据包过滤代码；还包含 editcap，可读取捕获文件并将其中的数据包写出，可能采用不同的捕获文件格式，也可能从捕获中移除部分数据包。

Wireshark 官方网站是 https://www.wireshark.org。

最新发行版可在 https://www.wireshark.org/download 子目录找到。

安装
------------

Wireshark 项目定期在以下平台上构建和测试：

  - Linux（Ubuntu）
  - Microsoft Windows
  - macOS / {Mac} OS X

官方安装包适用于 Microsoft Windows 和 macOS。

许多流行的操作系统和 Linux 发行版提供标准或附加软件包，包括 Debian、Ubuntu、Fedora、CentOS、RHEL、Arch、Gentoo、openSUSE、FreeBSD、DragonFly BSD、NetBSD 和 OpenBSD。

此外，还可通过 pkgsrc、OpenCSW、Homebrew 和 MacPorts 等多种第三方打包系统获取。

它应当可以在其他类 Unix 系统上运行，不会遇到太多问题。

在某些情况下，当前版本的 Wireshark 可能不支持你的操作系统。例如 Windows XP 受 Wireshark 1.10 及更早版本支持。在其他情况下，Wireshark 标准软件包可能只是版本较旧，例如 Solaris 和 HP-UX。

构建 Wireshark 需要 Python 3。构建文档（包括手册页）需要 AsciiDoctor。生成部分源代码需要 Perl 和 flex。

因此，在缺少这些工具的系统上，必须安装 Python 3、AsciiDoctor 和 GNU“flex”（普通“lex”不能使用）；可能还需要安装 Perl。

完整安装说明见 INSTALL 文件，以及 https://www.wireshark.org/docs/wsdg_html_chunked/ 上的开发者指南。

针对特定操作系统的安装说明，请另见相应的 README._OS_ 文件。

使用
-----

要从网络捕获数据包，你需要将 dumpcap 程序设置为 set-UID root，或者在系统采用此方式时具有访问 `/dev` 下相应条目的权限（BSD 衍生系统，以及支持 DLPI 的 Solaris、HP-UX 等系统通常属于这一类）。虽然可能有人想把 Wireshark 和 TShark 可执行文件设为 setuid root 或直接以 root 运行它们，请不要这样做。捕获过程已隔离到 dumpcap；这个简单程序包含安全漏洞的可能性更低，因此以 root 运行更安全。

有关各命令行选项和界面功能的说明，请查阅手册页。

多种文件类型
-------------------

Wireshark 可以读取多种文件类型中的数据包。支持的文件格式列表请参阅 Wireshark 手册页或 Wireshark 用户指南。

如果编译 Wireshark 时有相应压缩库，它可以透明地读取这些文件的压缩版本。目前支持的压缩格式包括：

- GZIP
- LZ4
- ZSTD

GZIP 和 LZ4（使用独立块时，默认即如此）支持快速随机定位，大文件上的图形界面性能因此更好。可在编译时通过传入对应的 cmake 选项禁用任一压缩格式：`cmake -DENABLE_ZLIB=OFF`、`cmake -DENABLE_LZ4=OFF` 或 `cmake -DENABLE_ZSTD=OFF`。

尽管 Wireshark 可以读取 AIX iptrace 文件，但 AIX 的 iptrace 数据包跟踪命令文档很少。`iptrace` 命令会启动一个守护进程，必须终止它才能停止跟踪。实验表明，向该守护进程发送 HUP 信号会使其正常关闭，并将完整数据包写入跟踪文件。如果文件末尾保存了不完整的数据包，Wireshark 在读取时会发出警告，但仍能读取其他所有数据包。如果发生这种情况，请通过 wireshark-dev@wireshark.org 告知 Wireshark 开发者；如果跟踪文件较小且不含敏感数据，请务必随信发送一份副本。

对 Lucent/Ascend 产品的支持仅限于 MAX 和 Pipeline 系列产品生成的调试跟踪输出。Wireshark 可以读取 `wandsession`、`wandisplay`、`wannext` 和 `wdd` 命令的输出。

Wireshark 也可以读取东芝“Compact Router”系列 ISDN 路由器（TR-600 和 TR-650）的转储跟踪输出。可以通过 telnet 连接路由器并使用 `snoop dump` 启动转储会话。

Wireshark 还可以读取 CoSine L2 调试输出。要获取该输出，先进入 diags 模式，然后在 layer-2 类别下使用 `create-pkt-log-profile` 和 `apply-pkt-lozg-profile` 命令。有关这些命令的更多用法，请通过 `layer-2 create ?` 或 `layer-2 apply ?` 查看帮助。

要在 Wireshark 中使用 Lucent/Ascend、东芝和 CoSine 的跟踪信息，必须将跟踪输出捕获到磁盘文件。跟踪发生在路由器内部，路由器无法替你将跟踪保存到文件。在 Unix 下，简单方法是运行 `telnet <ascend> | tee <outfile>`。或者，如果系统安装了“script”命令，可以将包括 telnet 在内的 shell 会话保存到文件。例如，记录到名为 tracefile.out 的文件：

~~~
$ script tracefile.out
Script started on <date/time>
$ telnet router
..... do your trace, then exit from the router's telnet session.
$ exit
Script done on <date/time>
~~~

名称解析
---------------

解码 IPv4 和 IPv6 数据包时，Wireshark 会尝试使用反向名称解析功能。

如果希望在使用 Wireshark 时关闭名称解析，请以 `-n` 选项启动，关闭所有名称解析（包括将 MAC 地址和 TCP/UDP/SMTP 端口号解析为名称）；或者使用 `-N mt` 选项，关闭所有网络层地址（IPv4、IPv6、IPX）的名称解析。

你可以通过编辑菜单中的 Preferences 项打开首选项对话框，选择“Name resolution”，关闭相应名称解析选项并点击“OK”，将其设为默认设置。

SNMP
----

Wireshark 可以对 SNMP 数据包进行一些基本解码；也可以使用 libsmi 库读取 MIB 文件，并利用其中信息以更友好的方式显示 OID 和变量绑定值，从而进行更复杂的解码。CMake 会自动确定系统中是否有 libsmi 库。如果已安装 libsmi 但_不希望_ Wireshark 使用它，可以运行 cmake 并添加 `-DENABLE_SMI=OFF` 选项。

如何报告错误
-------------------

Wireshark 不断开发，因此使用时可能遇到错误。请在 https://gitlab.com/wireshark/wireshark/-/issues 报告。

> 译注：该上游问题入口在本次采集时返回 HTTP 404；尚未确认替代的官方报告入口。请确保报告包含：

1. 对于 Wireshark 错误，提供帮助菜单中“About Wireshark”的完整构建信息或 `wireshark -v` 的输出；对于 TShark 错误，提供 `tshark -v` 的输出；
2. 如果错误发生在 Linux 上，提供使用的 Linux 发行版及其版本；
3. 如果从命令行运行 Wireshark 或运行 TShark，提供启动命令，以及导致错误出现的操作步骤。

如果错误由特定跟踪文件触发，请务必在错误描述中附上该文件。如果跟踪文件含有敏感信息（例如密码），请不要发送。

如果 Wireshark 因“segmentation violation”“bus error”“abort”或其他产生 UNIX core 转储文件的错误而退出，且你安装了调试器，就能给开发者提供很大帮助。可以使用调试器（此例为“gdb”）、Wireshark 二进制文件和生成的 core 文件获取堆栈跟踪。以下是使用 gdb 的“backtrace”命令的示例：

~~~
$ gdb wireshark core
(gdb) backtrace
..... prints the stack trace
(gdb) quit
$
~~~

在某些平台（例如 BSD 系统）上，core 转储文件可能命名为“wireshark.core”而不是“core”。如果是 TShark 而不是 Wireshark 产生了 core 转储，请在调试器中使用“tshark”作为第一个参数；转储文件可能命名为“tshark.core”。

许可证
-------

Wireshark 根据 GNU GPLv2 分发。完整许可证文本见 COPYING 文件。如有疑问，以完整文本作为具有法律约束力的部分。这些说明只是为了帮助不熟悉 GPLv2 的人理解。

使用没有限制。以源代码或二进制形式分发则有限制。

Wireshark 的大多数部分采用“GPL version 2 or later”许可。某些文件采用与 GPLv2 兼容的其他许可证。

值得注意的例外是，随 Wireshark 源码分发的一些工具采用本身不直接兼容 GPLv2 的其他许可证。项目说明这可以接受，因为只有工具本身按此方式许可，工具输出不被视为衍生作品，因此可以采用适合 Wireshark 使用的许可。以下是这些工具的不完整列表：
 - pidl 工具（tools/pidl）采用 GPLv3+ 许可。

Wireshark 的部分内容可以作为库构建和分发。这些部分仍受 GPL 而不是宽通用公共许可证（LGPL）或其他许可证约束。

如果将 Wireshark 的全部或部分内容集成到自己的应用中，并选择发布或发行该应用，那么组合后的作品必须按照 GPLv2 条款发布。

免责声明
----------

本产品没有任何明示或暗示的保证。使用风险由你自行承担。

Gerald Combs <gerald@wireshark.org>

Gilbert Ramirez <gram@alumni.rice.edu>

Guy Harris <gharris@sonic.net>
