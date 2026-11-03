## *hashcat* ##

hashcat 是一个高度优化的密码恢复平台，适用于 GPU、CPU 和大型分布式系统。

### 功能 ###

> 译注：以下速度等性能称谓为项目作者表述，尚未独立验证。

* 世界上最快的密码破解工具
* 世界首个也是唯一的内核规则引擎，可用于每种攻击模式
* 免费开源，采用 MIT 许可证
* 多操作系统：支持 Linux、Windows 和 macOS，并可在 BSD 和 Android 上从源代码构建
* 多后端：CUDA、HIP、Metal 和 OpenCL
* 多设备：一台系统中可使用多个设备及混合设备类型
* 多哈希：一次破解大量哈希
* 超过 630 种哈希模式，每一种都以性能为设计考虑
* 哈希配方：将哈希写为表达式，例如 `pbkdf2_sha256(md5(pass), salt, iter, 32)`，完全使用 GPU 加速
* 提供 10 种生成候选密码的攻击模式
* Assimilation bridge：使用 C、Python 或 Rust 添加哈希模式，无需编写内核
* Brain：跳过先前会话已尝试的候选项
* 使用覆盖网络的分布式破解网络
* 从词表、标准输入或其他程序读取候选项
* 读取使用 gzip、xz 或 zstd 压缩的词表
* 支持可定位读取的词表
* 使用马尔可夫链排列密钥空间，将更可能的候选项排在前面
* 按设备自动调整性能
* 支持交互式暂停、恢复、回退和前进
* 支持命名会话以及中断后恢复
* 内置基准测试
* 集成温度监控

### 许可证 ###

**hashcat** 采用 MIT 许可证。参阅 [docs/license.txt](https://github.com/hashcat/hashcat/blob/4ff3224b6794000cad4df605fd9fc4b8812db28f/docs/license.txt).

### 安装 ###

下载[最新版本](https://hashcat.net/hashcat/)并解压到所需位置。从命令行解压时使用 `7z x`，以保留完整文件路径。

你的平台可能还提供[软件包](https://github.com/hashcat/hashcat/blob/4ff3224b6794000cad4df605fd9fc4b8812db28f/docs/packages.md). hashcat 所需的驱动程序和运行时列于 [docs/hashcat-requirements.md](https://github.com/hashcat/hashcat/blob/4ff3224b6794000cad4df605fd9fc4b8812db28f/docs/hashcat-requirements.md).

### 构建 ###

从源代码构建是可选的。发行包是相同的程序，自己构建的二进制文件不会破解得更快。如果需要自己的修改、master 中尚未发布的修复，或我们没有提供二进制文件的平台版本，可以自行构建。

请参阅 [BUILD.md](https://github.com/hashcat/hashcat/blob/4ff3224b6794000cad4df605fd9fc4b8812db28f/BUILD.md)了解方法。

测试：

Build | BSD | Rust | Test | Fuzz
----- | --- | ---- | ---- | ----
[![Build](https://github.com/hashcat/hashcat/actions/workflows/build.yml/badge.svg?branch=master&event=push)](https://github.com/hashcat/hashcat/actions/workflows/build.yml) | [![BSD](https://github.com/hashcat/hashcat/actions/workflows/bsd.yml/badge.svg?branch=master&event=schedule)](https://github.com/hashcat/hashcat/actions/workflows/bsd.yml) | [![Rust tests](https://github.com/hashcat/hashcat/actions/workflows/rust.yml/badge.svg?branch=master&event=push)](https://github.com/hashcat/hashcat/actions/workflows/rust.yml) | [![test](https://github.com/hashcat/hashcat/actions/workflows/test.yml/badge.svg?branch=master&event=schedule)](https://github.com/hashcat/hashcat/actions/workflows/test.yml) | [![fuzz](https://github.com/hashcat/hashcat/actions/workflows/fuzz.yml/badge.svg?branch=master&event=schedule)](https://github.com/hashcat/hashcat/actions/workflows/fuzz.yml)

### 使用与帮助 ###

从 `--help` 开始，仓库中也保存有 [docs/hashcat-help.md](https://github.com/hashcat/hashcat/blob/4ff3224b6794000cad4df605fd9fc4b8812db28f/docs/hashcat-help.md). 每种模式的示例哈希见 [docs/hashcat-example-hashes.md](https://github.com/hashcat/hashcat/blob/4ff3224b6794000cad4df605fd9fc4b8812db28f/docs/hashcat-example-hashes.md).

[Wiki](https://hashcat.net/wiki/)和[常见问题](https://hashcat.net/wiki/doku.php?id=frequently_asked_questions)提供更多信息。[论坛](https://hashcat.net/forum/)保存了多年已经回答的问题。如果仍需要人工帮助，请前往 [Discord](https://discord.gg/HFS523HGBT).

### 文档 ###

[docs](https://github.com/hashcat/hashcat/tree/4ff3224b6794000cad4df605fd9fc4b8812db28f/docs/)目录涵盖各项功能。最常被问到的包括：

* [哈希配方](https://github.com/hashcat/hashcat/blob/4ff3224b6794000cad4df605fd9fc4b8812db28f/docs/hashcat-hash-recipe.md)：无需编写哈希模式即可破解自定义哈希方案
* [插件开发指南](https://github.com/hashcat/hashcat/blob/4ff3224b6794000cad4df605fd9fc4b8812db28f/docs/hashcat-plugin-development-guide.md)：用于添加哈希模式
* [Assimilation bridge](https://github.com/hashcat/hashcat/blob/4ff3224b6794000cad4df605fd9fc4b8812db28f/docs/hashcat-assimilation-bridge.md)及其 [Python](https://github.com/hashcat/hashcat/blob/4ff3224b6794000cad4df605fd9fc4b8812db28f/docs/hashcat-python-plugin-quickstart.md)和 [Rust](https://github.com/hashcat/hashcat/blob/4ff3224b6794000cad4df605fd9fc4b8812db28f/docs/hashcat-rust-plugin-quickstart.md)快速入门
* [FPGA 配置](https://github.com/hashcat/hashcat/blob/4ff3224b6794000cad4df605fd9fc4b8812db28f/docs/hashcat-fpga-setup.md)：使用模式 75000 在 FPGA 板卡上运行 bcrypt
* [通用攻击模式](https://github.com/hashcat/hashcat/blob/4ff3224b6794000cad4df605fd9fc4b8812db28f/docs/hashcat-generic-attack-mode.md), [PCFG](https://github.com/hashcat/hashcat/blob/4ff3224b6794000cad4df605fd9fc4b8812db28f/docs/hashcat-pcfg.md), [表格](https://github.com/hashcat/hashcat/blob/4ff3224b6794000cad4df605fd9fc4b8812db28f/docs/hashcat-table.md)和[关联](https://github.com/hashcat/hashcat/blob/4ff3224b6794000cad4df605fd9fc4b8812db28f/docs/hashcat-association.md)攻击
* [Brain](https://github.com/hashcat/hashcat/blob/4ff3224b6794000cad4df605fd9fc4b8812db28f/docs/hashcat-brain.md)：跳过先前会话已尝试的候选项
* [v8.0.0 发行说明](https://github.com/hashcat/hashcat/blob/4ff3224b6794000cad4df605fd9fc4b8812db28f/docs/releases_notes_v8.0.0.md)及完整的[变更日志](https://github.com/hashcat/hashcat/blob/4ff3224b6794000cad4df605fd9fc4b8812db28f/docs/changes.txt)

### 贡献 ###

欢迎贡献。[CONTRIBUTING.md](https://github.com/hashcat/hashcat/blob/4ff3224b6794000cad4df605fd9fc4b8812db28f/CONTRIBUTING.md)介绍代码风格、pull request 的要求以及提交前如何测试修改。

### 安全 ###

[SECURITY.md](https://github.com/hashcat/hashcat/blob/4ff3224b6794000cad4df605fd9fc4b8812db28f/SECURITY.md)说明如何报告漏洞以及什么情况构成漏洞。

### 祝破解愉快！
