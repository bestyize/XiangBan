<img src="https://raw.githubusercontent.com/NationalSecurityAgency/ghidra/251f3603f89c0620bdc503ff9a19382bf469ffec/Ghidra/Features/Base/src/main/resources/images/GHIDRA_3.png" width="400">

# Ghidra 软件逆向工程框架
Ghidra 是由[美国国家安全局][nsa]研究局创建和维护的软件逆向工程（SRE）框架。该框架包含一套功能全面的软件分析工具，使用户能够在 Windows、macOS 和 Linux 等多种平台上分析已编译代码。功能包括反汇编、汇编、反编译、图形展示和脚本，以及数百项其他功能。Ghidra 支持多种处理器指令集和可执行文件格式，既可交互使用，也可自动运行。用户还可以使用 Java 或 Python 开发自己的 Ghidra 扩展组件和脚本。

为支持美国国家安全局的网络安全任务，Ghidra 旨在解决复杂软件逆向工程工作中的规模扩展与团队协作问题，并提供可定制、可扩展的研究平台。美国国家安全局已将 Ghidra 的软件逆向工程能力用于多种问题，包括分析恶意代码，帮助分析人员深入了解网络和系统中的潜在漏洞。

如果你是对这类项目感兴趣的美国公民，希望为美国国家安全局开发 Ghidra 和其他网络安全工具，以帮助保护美国及其盟友，可考虑[申请相关职位][career]。

## 安全警告
**警告：** Ghidra 的某些版本存在已知安全漏洞。继续之前，请阅读 Ghidra 的[安全公告][security]，以更好地了解可能受到的影响。

## 安装
安装官方预构建的跨平台 Ghidra 发行版：  
* 安装 [JDK 25 64 位版本][jdk]
* 下载 Ghidra [发行文件][releases]
  - **注意：** 官方跨平台发行文件名为 `ghidra_<version>_<release>_<date>.zip`，可在“Assets”下拉列表中找到。此步骤不应下载名为“Source Code”的任一文件。
* 解压 Ghidra 发行文件
  - **注意：** 不要解压覆盖现有安装
* 启动 Ghidra： `./ghidraRun`（Windows 使用 `ghidraRun.bat`）
  - 或启动 [PyGhidra][pyghidra]： `./support/pyghidraRun`（Windows 使用 `support\pyghidraRun.bat`）

有关安装和运行 Ghidra 发行版的更多信息和故障排查提示，请参阅 Ghidra 安装目录根部的[入门文档][gettingstarted]。 

## 构建
[![Build Ghidra][build-ghidra-badge]][build-ghidra-action]

从此源代码仓库为你的平台创建最新开发构建：

##### 安装构建工具：
* [JDK 25 64 位版本][jdk]
* [Gradle 9.1.0+][gradle]（有互联网连接时也可使用提供的 Gradle wrapper）
* 带有 pip 的 [Python3][python3]（版本 3.9 至 3.15）
* GCC 或 Clang，以及 make（仅 Linux/macOS）
* [Microsoft Visual Studio][vs] 2017+ 或 [Microsoft C++ Build Tools][vcbuildtools]，并安装以下组件（仅 Windows）：
  - MSVC
  - Windows SDK
  - C++ ATL

##### 下载并解压源代码：
[从 GitHub 下载][master]
```
unzip ghidra-master
cd ghidra-master
```
**注意：** 除了下载压缩源代码，你也可以克隆 GitHub 仓库： `git clone https://github.com/NationalSecurityAgency/ghidra.git`

##### 将额外构建依赖下载到源代码仓库：
**注意：** 如果有互联网连接且尚未安装 Gradle，以下说明中的 `gradle` 命令可替换为 `./gradlew`（或 `gradlew.bat`）。

```
gradle -I gradle/support/fetchDependencies.gradle
```

##### 创建开发构建： 
```
gradle buildGhidra
```
压缩后的开发构建将位于 `build/dist/`。

有关构建 Ghidra 的详细信息，请阅读[开发者指南][devguide]。

如果构建遇到问题，请查看[已知问题][known-issues]部分，寻找可能的解决办法。

## 开发

### 用户脚本和扩展
Ghidra 支持用户通过 Eclipse 的 *GhidraDev* 插件编写自定义脚本和扩展。该插件及其说明可在 Ghidra 发行版的 `Extensions/Eclipse/GhidraDev/` 中找到，也可通过[此链接][ghidradev]访问。另外，可在脚本管理器中点击 Visual Studio Code 图标，使用 Visual Studio Code 编辑脚本。可以从 Ghidra CodeBrowser 窗口的 _Tools -> Create VSCode Module project_ 创建功能完整的 Visual Studio Code 项目。

**注意：** Eclipse 的 *GhidraDev* 插件和 Visual Studio Code 集成都只支持针对完整构建的 Ghidra 安装进行开发；这些安装可从[发行版][releases]页面下载。

### 高级开发
开发 Ghidra 工具本身时，强烈建议使用 Eclipse；Ghidra 开发流程已针对 Eclipse 进行了大量定制。

##### 安装构建和开发工具：
* 按照上述[构建说明](#构建)操作，使构建无错误地完成
* 安装 [Eclipse IDE for Java Developers][eclipse]

##### 准备开发环境：
``` 
gradle prepdev eclipse buildNatives
```

##### 将 Ghidra 项目导入 Eclipse：
* *File* -> *Import...*
* *General* | *Existing Projects into Workspace*
* 将根目录选择为下载或克隆的 Ghidra 源代码仓库
* 勾选 *Search for nested projects*
* 点击 *Finish*

当 Eclipse 构建项目完成后，可使用提供的 **Ghidra** Eclipse *run configuration* 启动和调试 Ghidra。

有关开发 Ghidra 的详细信息，请阅读[开发者指南][devguide]。

## 参与贡献
如果你希望为 Ghidra 贡献错误修复、改进或新功能，请查阅[贡献者指南][contrib]，了解如何参与这一开源项目。

[build-ghidra-action]: https://github.com/NationalSecurityAgency/ghidra/actions/workflows/build-ghidra.yml
[build-ghidra-badge]: https://github.com/NationalSecurityAgency/ghidra/actions/workflows/build-ghidra.yml/badge.svg
[nsa]: https://www.nsa.gov
[contrib]: https://github.com/NationalSecurityAgency/ghidra/blob/251f3603f89c0620bdc503ff9a19382bf469ffec/CONTRIBUTING.md
[devguide]: https://github.com/NationalSecurityAgency/ghidra/blob/251f3603f89c0620bdc503ff9a19382bf469ffec/DevGuide.md
[gettingstarted]: https://github.com/NationalSecurityAgency/ghidra/blob/251f3603f89c0620bdc503ff9a19382bf469ffec/GhidraDocs/GettingStarted.md
[known-issues]: https://github.com/NationalSecurityAgency/ghidra/blob/251f3603f89c0620bdc503ff9a19382bf469ffec/DevGuide.md#known-issues
[career]: https://www.intelligencecareers.gov/nsa
[releases]: https://github.com/NationalSecurityAgency/ghidra/releases
[jdk]: https://adoptium.net/temurin/releases
[gradle]: https://gradle.org/releases/
[python3]: https://www.python.org/downloads/
[vs]: https://visualstudio.microsoft.com/vs/community/
[vcbuildtools]: https://visualstudio.microsoft.com/visual-cpp-build-tools/
[eclipse]: https://www.eclipse.org/downloads/packages/
[master]: https://github.com/NationalSecurityAgency/ghidra/archive/refs/heads/master.zip
[security]: https://github.com/NationalSecurityAgency/ghidra/security/advisories
[ghidradev]: https://github.com/NationalSecurityAgency/ghidra/blob/251f3603f89c0620bdc503ff9a19382bf469ffec/GhidraBuild/EclipsePlugins/GhidraDev/GhidraDevPlugin/README.md
[pyghidra]: https://github.com/NationalSecurityAgency/ghidra/blob/251f3603f89c0620bdc503ff9a19382bf469ffec/Ghidra/Features/PyGhidra/README.md
