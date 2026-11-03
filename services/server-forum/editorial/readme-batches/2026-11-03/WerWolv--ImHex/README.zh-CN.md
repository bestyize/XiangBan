> 根据 [ImHex 固定版本 README](https://github.com/WerWolv/ImHex/blob/34483cb90527cd71c92049ef3b1b2cc27936a1d4/README.md) 翻译。功能与系统要求是上游该版本表述；主项目 GPLv2-only，指定插件接口目录另用 LGPLv2.1，详情见文末许可说明。

<a href="https://imhex.werwolv.net">
  <h1 align="center">
    <picture>
      <img height="300px" style="margin: 0; padding: 0" src="https://raw.githubusercontent.com/WerWolv/ImHex/34483cb90527cd71c92049ef3b1b2cc27936a1d4/resources/dist/common/logo/ImHexLogoSVGBG.svg">
    </picture>
  </h1>
</a>

<p align="center">
    面向逆向工程师、程序员以及凌晨 3 点工作时仍珍惜视网膜的人们的十六进制编辑器。
    <br>
    <a href="https://itinerarium.github.io/phoneme-synthesis/?w=/'ˈɪmhɛks/"><strong>/ˈɪmhɛks/</strong></a>
</p>
<p align="center">
  <a title="'Build' workflow Status" href="https://github.com/WerWolv/ImHex/actions?query=workflow%3ABuild"><img alt="'Build' workflow Status" src="https://img.shields.io/github/actions/workflow/status/WerWolv/ImHex/build.yml?longCache=true&style=for-the-badge&label=Build&logoColor=fff&logo=GitHub%20Actions&branch=master"></a>
  <a title="Discord Server" href="https://discord.gg/X63jZ36xBY"><img alt="Discord Server" src="https://img.shields.io/discord/789833418631675954?label=Discord&logo=Discord&logoColor=fff&style=for-the-badge"></a>
  <a title="Total Downloads" href="https://github.com/WerWolv/ImHex/releases/latest"><img alt="Total Downloads" src="https://img.shields.io/github/downloads/WerWolv/ImHex/total?longCache=true&style=for-the-badge&label=Downloads&logoColor=fff&logo=GitHub"></a>
  <a title="Code Quality" href="https://www.codefactor.io/repository/github/werwolv/imhex"><img alt="Code Quality" src="https://img.shields.io/codefactor/grade/github/WerWolv/ImHex?longCache=true&style=for-the-badge&label=Code%20Quality&logoColor=fff&logo=CodeFactor&branch=master"></a>
  <a title="Translation" href="https://weblate.werwolv.net/projects/imhex/"><img alt="Translation" src="https://img.shields.io/weblate/progress/imhex?logo=weblate&logoColor=%23FFFFFF&server=https%3A%2F%2Fweblate.werwolv.net&style=for-the-badge"></a>
  <a title="Plugins" href="https://github.com/WerWolv/ImHex/blob/master/PLUGINS.md"><img alt="Plugins" src="https://img.shields.io/badge/Plugins-Supported-brightgreen?logo=stackedit&logoColor=%23FFFFFF&style=for-the-badge"></a>
</p>

<p align="center">
  <a title="Download the latest version of ImHex" href="https://imhex.download"><img alt="Download the latest version of ImHex!" src="https://raw.githubusercontent.com/WerWolv/ImHex/34483cb90527cd71c92049ef3b1b2cc27936a1d4/resources/dist/common/get_release_banner.png"></a>
  <a title="Download the latest nightly pre-release version of ImHex" href="https://imhex.download/#nightly"><img alt="Download the latest nightly pre-release version of ImHex" src="https://raw.githubusercontent.com/WerWolv/ImHex/34483cb90527cd71c92049ef3b1b2cc27936a1d4/resources/dist/common/get_nightly_banner.png"></a>
  <a title="Use the Web version of ImHex right in your browser!" href="https://web.imhex.werwolv.net"><img alt="Use the Web version of ImHex right in your browser!" src="https://raw.githubusercontent.com/WerWolv/ImHex/34483cb90527cd71c92049ef3b1b2cc27936a1d4/resources/dist/common/try_online_banner.png"></a>
  <a title="Read the documentation of ImHex!" href="https://docs.werwolv.net"><img alt="Read the documentation of ImHex!" src="https://raw.githubusercontent.com/WerWolv/ImHex/34483cb90527cd71c92049ef3b1b2cc27936a1d4/resources/dist/common/read_docs_banner.png"></a>
</p>

## 支持

如果你喜欢我的工作，请考虑通过 GitHub Sponsors 或 Ko-Fi 支持我。非常感谢！
如果你在工作场所使用 ImHex，请通过电子邮件 [imhex@werwolv.net](mailto:imhex@werwolv.net) 或 Discord 联系我，讨论赞助以及如何改进 ImHex 以满足你的使用场景！

<p align="center">
<a href="https://github.com/sponsors/WerWolv"><img src="https://werwolv.net/assets/github_banner.png" alt="GitHub donate button" /></a>
<a href="https://ko-fi.com/WerWolv"><img src="https://werwolv.net/assets/kofi_banner.png" alt="Ko-Fi donate button" /></a>
</p>

### 重要赞助者
|                                                                                                     |                                                                                   |
|:---------------------------------------------------------------------------------------------------:|-----------------------------------------------------------------------------------|
| [![JetBrains 标志](https://avatars.githubusercontent.com/u/878437?s=48)](https://www.jetbrains.com) | JetBrains，为开发提供免费的 All Products Pack 许可证 |
| [![SignPath 标志](https://avatars.githubusercontent.com/u/34448643?s=48)](https://signpath.io/) | SignPath，为 Windows 构建提供免费的代码签名证书 |
| [![AWS 标志](https://avatars.githubusercontent.com/u/2232217?s=48)](https://aws.amazon.com) | Amazon，为 CI 提供免费的 AWS 云额度 |

也想出现在这里？请通过 [imhex@werwolv.net](mailto:imhex@werwolv.net) 联系我们！

## 截图
![十六进制编辑器、模式和数据信息](https://github.com/user-attachments/assets/902a7c4c-410d-490f-999e-14c856fec027)
![书签、数据信息、查找视图和数据处理器](https://github.com/user-attachments/assets/58eefa1f-31c9-4bb8-a1c1-8cdd8ddbd29f)

<details>
<summary><strong>更多截图</strong></summary>

![数据处理器解密数据并将其显示为图像](https://github.com/WerWolv/ImHex/assets/10835354/d0623081-3094-4840-a8a8-647b38724db8)
![用模式语言编写的 STL 解析器可视化 3D 模型](https://github.com/WerWolv/ImHex/assets/10835354/62cbcd18-1c3f-4dd6-a877-2bf2bf4bb2a5)
![数据信息视图展示文件的各种统计信息](https://github.com/WerWolv/ImHex/assets/10835354/d4706c01-c258-45c9-80b8-fe7a10d5a1de)

</details>

## 功能

<details>
  <summary><strong>功能丰富的十六进制视图</strong></summary>

  - 字节修补
  - 补丁管理
  - 无限撤销/重做
  - “将字节复制为……”
    - 字节
    - 十六进制字符串
    - C、C++、C#、Rust、Python、Java 和 JavaScript 数组
    - ASCII 艺术十六进制视图
    - 独立 HTML div
  - 简单的字符串和十六进制搜索
  - 从起点、终点和当前光标位置跳转
  - 彩色高亮
    - 可配置的前景高亮规则
    - 使用模式、查找结果和书签进行背景高亮
  - 以多种类型的列表显示数据
    - 十六进制整数（8、16、32、64 位）
    - 有符号和无符号十进制整数（8、16、32、64 位）
    - 浮点数（16、32、64 位）
    - RGBA8 颜色
    - HexII
    - 二进制
  - 将数据解码为 ASCII 和自定义编码
    - 内置支持 UTF-8、UTF-16、ShiftJIS、大多数 Windows 编码等
  - 分页数据视图
</details>
<details>
  <summary><strong>用于解析并高亮文件内容的自定义类 C++ 模式语言</strong></summary>
  
  - 根据 MIME 类型和魔数自动加载
  - 支持数组、指针、结构体、联合体、枚举、位域、命名空间、小端和大端、条件语句等！
  - 实用错误信息、语法高亮和错误标记
  - 支持可视化多种类型的数据
    - 图像
    - 音频
    - 3D 模型
    - 坐标
    - 时间戳
</details>
<details>
  <summary><strong>主题支持</strong></summary>

  - 深夜使用时不会灼伤你的视网膜
    - 默认深色模式，也提供浅色模式
  - 通过可共享主题文件自定义所有界面元素的颜色和样式
  - 支持自定义字体
</details>
<details>
  <summary><strong>导入和导出数据</strong></summary>
  
  - Base64 文件
  - IPS 和 IPS32 补丁
  - Markdown 报告
  - 各种编程语言的二进制数组
</details>
<details>
  <summary><strong>数据检查器</strong></summary>

  - 将数据解释为多种类型，支持字节序、十进制、十六进制、八进制及位反转
    - 无符号和有符号整数（8、16、24、32、48、64 位）
    - 浮点数（16、32、64 位）
    - 有符号和无符号 LEB128
    - ASCII、宽字符及 UTF-8 字符和字符串
    - time32_t、time64_t、DOS 日期和时间
    - GUID
    - RGBA8 和 RGB65 颜色
  - 通过检查器复制和修改字节
  - 通过模式语言添加新数据类型
  - 支持隐藏未使用的行
</details>
<details>
  <summary><strong>基于节点的数据预处理器</strong></summary>

  - 在十六进制编辑器显示数据前修改、解密和解码
  - 修改数据而不触及底层源
  - 支持添加自定义节点
</details>
<details>
  <summary><strong>从多种不同数据源加载数据</strong></summary>

  - 本地文件
    - 支持快速高效加载大型文件
  - 原始磁盘
    - 从原始磁盘和分区加载数据
  - GDB 服务器
    - 通过 GDB 访问运行中进程或嵌入式设备的内存
  - Intel Hex 和 Motorola SREC 数据
  - Base64 编码数据
  - UDP 数据包
    - 支持显示通过 UDP 接收的原始数据
  - 进程内存
    - 检查运行中进程的整个地址空间
  - 通过 SSH 和 SFTP 访问远程文件
    - 支持使用 SSH 和 SFTP 从远程服务器加载文件
</details>
<details>
  <summary><strong>数据搜索</strong></summary>
  
  - 支持搜索整个文件或仅搜索选区
  - 字符串提取
    - 可指定最小长度和字符集（小写、大写、数字、符号）
    - 可指定编码（ASCII、UTF-8、UTF-16 大端和小端）
  - 序列搜索
    - 搜索字节或字符序列
    - 可忽略字符大小写
  - 正则表达式搜索
    - 使用正则表达式搜索字符串
  - 二进制模式
    - 搜索字节序列，可选择使用通配符
  - 数值搜索
    - 搜索有符号/无符号整数和浮点数
    - 搜索数值范围
    - 可指定大小和字节序
    - 可忽略未对齐的值
</details>
<details>
  <summary><strong>数据哈希支持</strong></summary>

  - 提供多种算法
    - 支持自定义初始值和多项式的 CRC8、CRC16 和 CRC32
      - 提供多种默认多项式
    - MD5
    - SHA-1, SHA-224, SHA-256, SHA-384, SHA-512
    - Adler32
    - AP
    - BKDR
    - Bernstein, Bernstein1
    - DEK, DJB, ELF, FNV1, FNV1a, JS, PJW, RS, SDBM
    - OneAtTime, Rotating, ShiftAndXor, SuperFast
    - Murmur2_32, MurmurHash3_x86_32, MurmurHash3_x86_128, MurmurHash3_x64_128
    - SipHash64, SipHash128
    - XXHash32, XXHash64
    - Tiger, Tiger2
    - Blake2B, Blake2S
  - 对已加载数据的指定区域计算哈希
  - 对任意字符串计算哈希
</details>
<details>
  <summary><strong>差异比较支持</strong></summary>

  - 比较不同数据源的数据
  - 差异高亮
  - 差异表格视图
</details>
<details>
  <summary><strong>集成反汇编器</strong></summary>
  
  - 支持 Capstone 支持的所有架构
    - ARM32 (ARM, Thumb, Cortex-M, AArch32)
    - ARM64
    - MIPS (MIPS32, MIPS64, MIPS32R6, Micro)
    - x86 (16-bit, 32-bit, 64-bit)
    - PowerPC (32-bit, 64-bit)
    - SPARC
    - IBM SystemZ
    - xCORE
    - M68K
    - TMS320C64X
    - M680X
    - Ethereum
    - RISC-V
    - WebAssembly
    - MOS65XX
    - Berkeley Packet Filter
  - 支持为自己的架构编写自定义反汇编器
</details>
<details>
  <summary><strong>书签</strong></summary>

  - 支持自定义名称和颜色的书签
  - 在十六进制编辑器中高亮书签区域
  - 跳转到书签
  - 在新标签页打开书签内容
  - 为书签添加注释
</details>
<details>
  <summary><strong>功能丰富的数据分析与可视化工具</strong></summary>

  - 基于文件魔数的解析器和 MIME 类型数据库
  - 字节类型分布图
  - 熵图
  - 最高熵和平均熵
  - 检测加密或压缩文件
  - 双字节及分层分布图
</details>
<details>
  <summary><strong>YARA 规则支持</strong></summary>

  - 使用官方 YARA 规则扫描文件中的漏洞
  - 在十六进制编辑器中高亮匹配项
  - 跳转到匹配项
  - 同时应用多条规则
</details>
<details>
  <summary><strong>实用工具</strong></summary>

  - 基于 LLVM 的 Itanium、MSVC、Rust 和 D-Lang 符号解码器
  - ASCII 表
  - 正则替换工具
  - 数学表达式求值器（计算器）
  - 绘图计算器
  - 支持多种格式的十六进制颜色选择器
  - 进制转换器
  - 字节交换器
  - UNIX 权限计算器
  - 维基百科术语定义查找器
  - 文件工具
    - 文件拆分器
    - 文件合并器
    - 文件粉碎器
  - IEEE754 浮点数可视化工具
  - 不变量乘法除法计算器
  - TCP 客户端/服务器
  - 欧几里得算法计算器
  - HTTP 请求
</details>
<details>
  <summary><strong>内置内容更新器</strong></summary>

  - 直接在 ImHex 内下载数据库中找到的所有文件
    - 解码各种文件格式的模式文件
    - 模式语言库
    - 用于文件类型检测的魔数文件
    - 自定义数据处理节点
    - 自定义编码
    - 自定义主题
    - Yara 规则
</details>
<details>
  <summary><strong>现代化界面</strong></summary>

  - 支持多个工作区
  - 支持自定义布局
  - 可拆卸窗口
</details>
<details>
  <summary><strong>易于上手</strong></summary>

  - 支持多种语言
  - 适合初学者的简化模式
  - 详尽文档
  - [数据库](https://github.com/WerWolv/ImHex-Patterns)提供许多示例文件
  - 通过成就引导你了解 ImHex 功能
  - 交互式教程
</details>

## 模式语言

模式语言是专为 ImHex 开发的完全自定义编程语言。
它允许你使用类 C 语法定义结构和数据类型，然后用于解析和高亮文件内容。

- 源代码：[链接](https://github.com/WerWolv/PatternLanguage/)
- 文档：[链接](https://docs.werwolv.net/pattern-language)

## 数据库

要查找格式模式、库、魔数和常量文件，请查看 [ImHex-Patterns](https://github.com/WerWolv/ImHex-Patterns) 仓库。

**欢迎在那里提交你自己的文件！**

## 系统要求

使用 ImHex 需要满足以下最低系统要求。

> [!IMPORTANT]
> ImHex 通常需要支持 OpenGL 3.0 的 GPU。
> 也有软件渲染、不需要 GPU 的发行版（带 `-NoGPU` 后缀），但它们可能比 GPU 加速版本慢得多。
> 
> 如果可能，请让 ImHex 使用系统的独立 GPU，而非集成 GPU。
> ImHex 通常也能在集成 GPU 上正常运行，但已知 Windows 上某些 Intel HD GPU 驱动会造成图形伪影。

- **操作系统**：
  - **Windows**：Windows 7 或更高版本（推荐 Windows 10/11）
  - **macOS**：macOS 15（Sequoia）或更高版本，
    - 更低版本应该也能运行，但需要自行编译 ImHex。由于 GitHub 没有 macOS 15 或更低版本的 CI runner，发行版二进制文件将**无法**运行。
    - macOS 构建未签名，需要你在“安全性与隐私”设置中手动允许。
  - **Linux**：“现代” Linux。以下发行版提供官方构建；其他发行版通过 AppImage、Flatpak 和 Snap 支持。
    - Ubuntu and Debian
    - Fedora
    - RHEL/AlmaLinux
    - Arch Linux
    - 从源代码编译 ImHex 时，基本上其他任何发行版也都可用。
  - **FreeBSD**：已在 FreeBSD 14.3 测试
    - 其他版本很可能也可用，但未经测试
- **CPU**：官方支持 x86、AMD64 和 ARM64，不过任何小端 CPU 都应该可用。
- **GPU**：OpenGL 3.0 或更高版本
  - 支持 Intel HD 集成 GPU，但已知某些驱动会造成各种图形伪影，尤其是在 Windows 上。使用风险自负。
  - 如果没有 GPU，可使用适用于 Windows 和 macOS 的软件渲染发行版。
- **内存**：约 50MiB，更复杂的分析需要更多内存
- **存储空间**：约 100MiB

## 安装

安装 ImHex 的说明见[安装指南](https://github.com/WerWolv/ImHex/blob/34483cb90527cd71c92049ef3b1b2cc27936a1d4/INSTALL.md)。

## 编译

在任何平台编译 ImHex，都需要支持 C++23 或更高版本的 GCC（或 Clang）。
Windows 和 Linux 发行版使用最新可用 GCC 构建。
macOS 发行版使用最新可用 LLVM Clang 构建。

请注意，MSVC 和 AppleClang 编译器均**不受支持**，因为它们通常严重过时，缺少 GCC 和 LLVM Clang 具备的功能。

> [!NOTE]
> 许多依赖通过子模块捆绑在仓库中，请确保使用 `--recurse-submodules` 选项克隆。
> 未捆绑的依赖可通过 `/dist` 目录中的依赖安装脚本安装。

更多信息请参阅[编译指南](https://github.com/WerWolv/ImHex/tree/34483cb90527cd71c92049ef3b1b2cc27936a1d4/dist/compiling)。

## 贡献
参见[贡献指南](https://github.com/WerWolv/ImHex/blob/34483cb90527cd71c92049ef3b1b2cc27936a1d4/CONTRIBUTING.md)。


## 插件开发
开发 ImHex 插件时，可使用以下模板项目入门。随后你可以访问完整的 libimhex、ImHex API 和内容注册表，以便与 ImHex 交互或添加新内容。
构建插件需要使用我们的 SDK。

### 在本地获取 SDK
可以通过如下方式编译 ImHex 来构建 SDK：
- `cmake -G Ninja -DIMHEX_BUNDLE_PLUGIN_SDK=ON -B build`
- `cd build`
- `DESTDIR=install ninja install`
随后 SDK 位于 `install/usr/local/share/imhex/sdk`。需要将 `IMHEX_SDK_PATH` 变量设置为该绝对路径。

### 在 GitHub Actions CI 中获取 SDK
可以使用[此 action](https://github.com/WerWolv/imhex-download-sdk) 自动将 SDK 下载到 GitHub Runner。
- [ImHex Plugin Template](https://github.com/WerWolv/ImHex-Plugin-Template)


## 致谢

### 贡献者

- [AxCut](https://github.com/paxcut)：为模式文本编辑器和 ImHex 其他大量部分做出了巨大贡献
- [iTrooz](https://github.com/iTrooz)：让 ImHex 登上 Web，并为项目各处做出数百项贡献
- [jumanji144](https://github.com/jumanji144)：为模式语言和 ImHex 基础设施做出巨大贡献
- [Mary](https://github.com/marysaka)：在将 ImHex 移植到 macOS 和开发过程中给予极大帮助
- [Roblabla](https://github.com/Roblabla)：为 ImHex 添加 MSI 安装程序支持
- [Mailaender](https://github.com/Mailaender)：让 ImHex 登上 Flathub
- 所有在 Discord 或 GitHub 报告问题并与我愉快交流过的其他人 :)

### 依赖

- 非常感谢 ocornut 提供出色的 [Dear ImGui](https://github.com/ocornut/imgui)，用于构建整个界面
  - 感谢 epezent 提供 [ImPlot](https://github.com/epezent/implot)，用于各处的数据绘图
  - 感谢 Nelarius 提供 [ImNodes](https://github.com/Nelarius/imnodes)，作为数据处理器基础
  - 感谢 BalazsJako 提供 [ImGuiColorTextEdit](https://github.com/BalazsJako/ImGuiColorTextEdit)，用于模式语言语法高亮
- 感谢 nlohmann 提供用于配置文件的 [json](https://github.com/nlohmann/json) 库
- 感谢 vitaut 提供 [libfmt](https://github.com/fmtlib/fmt) 库，使格式化和日志记录更加方便
- 感谢 btzy 提供 [nativefiledialog-extended](https://github.com/btzy/nativefiledialog-extended) 和出色支持，用于处理所有平台的文件对话框
- 感谢 danyspin97 提供 [xdgpp](https://sr.ht/~danyspin97/xdgpp)，用于处理 Linux 上的文件夹路径
- 感谢 aquynh 提供 [capstone](https://github.com/aquynh/capstone)，它是反汇编窗口的基础
- 感谢 rxi 提供 [microtar](https://github.com/rxi/microtar)，用于提取下载的商店资源
- 感谢 VirusTotal 提供 [Yara](https://github.com/VirusTotal/yara)，供 Yara 插件使用
- 感谢 Martinsos 提供 [edlib](https://github.com/Martinsos/edlib)，用于差异视图中的序列搜索
- 感谢 ron4fun 提供 [HashLibPlus](https://github.com/ron4fun/HashLibPlus)，实现了各种哈希算法
- 感谢 mackron 提供 [miniaudio](https://github.com/mackron/miniaudio)，用于播放音频文件
- 感谢所有其他其库被 ImHex 使用的团体和组织

### 许可证

ImHex 的大部分代码采用 GPLv2-only 许可。
值得注意的例外是以下部分采用 LGPLv2.1 许可：
- **/lib/libimhex**：允许插件与 ImHex 交互的库。
- **/plugins/ui**：包含可供其他插件使用的常用界面元素的 UI 插件库。

这样做是为了允许为 ImHex 开发专有插件。

### 代码签名政策

免费代码签名由 [SignPath.io](https://about.signpath.io/) 提供，
证书由 [SignPath Foundation](https://signpath.org/) 提供。

除非用户、安装者或操作者明确要求，
本程序不会向其他联网系统传输任何信息。


#### 拥有直接推送权限的人
- [WerWolv](https://github.com/WerWolv)
- [iTrooz](https://github.com/iTrooz)
- [jumanji144](https://github.com/jumanji144)
- [AxCut](https://github.com/paxcut)
