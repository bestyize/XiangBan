<picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/flipperdevices/flipperzero-firmware/9981d9d4aa5ec7b14bb46cf15763dd293a7dec6a/.github/assets/dark_theme_banner.png">
    <source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/flipperdevices/flipperzero-firmware/9981d9d4aa5ec7b14bb46cf15763dd293a7dec6a/.github/assets/light_theme_banner.png">
    <img
        alt="A pixel art of a Dophin with text: Flipper Zero Official Repo"
        src="https://raw.githubusercontent.com/flipperdevices/flipperzero-firmware/9981d9d4aa5ec7b14bb46cf15763dd293a7dec6a/.github/assets/light_theme_banner.png">
</picture>

# Flipper Zero 固件

- [Flipper Zero 官方网站](https://flipper.net) - 向朋友简单介绍 Flipper Zero 能做什么。
- [Flipper Zero 固件更新](https://flipper.net/pages/downloads) - 为你的小海豚带来改进：最新固件版本，以及 PC 和移动设备升级工具。
- [用户文档](https://docs.flipper.net/zero) - 更多了解你的小海豚：规格、使用指南和各种问题。
- [开发者文档](https://developer.flipper.net/flipperzero/doxygen) - 深入 Flipper Zero 固件源代码：构建系统、固件结构等。

# 参与贡献

我们的主要目标是在 Flipper 周围建立健康、可持续的社区，因此欢迎各种新想法和贡献。这里也有一些规则和禁忌，请仔细阅读本页及我们的[行为准则](https://github.com/flipperdevices/flipperzero-firmware/blob/9981d9d4aa5ec7b14bb46cf15763dd293a7dec6a/CODE_OF_CONDUCT.md)。

## 我需要帮助

寻找答案的最佳位置是[用户文档](https://docs.flipper.net/zero)。如果找不到答案，请查看我们的 [Discord 服务器](https://flipp.dev/discord)。如果你想参与固件开发或根据需要修改固件，也可以查看[开发者文档](https://developer.flipper.net/flipperzero/doxygen)。

## 我想报告问题

如果发现问题并想报告，请查看 [Issues](https://github.com/flipperdevices/flipperzero-firmware/issues) 页面。请确保描述中包含所用固件版本、平台和清晰的复现步骤。

## 我想提出新功能

如果有功能请求或想支持现有请求，请使用 [Discussions](https://github.com/flipperdevices/flipperzero-firmware/discussions)，并遵循[讨论指南](https://github.com/flipperdevices/flipperzero-firmware/discussions/4395)。

## 我想贡献代码

提交 PR 前，请确认你的更改确实需要包含在固件中。许多想法可以轻松实现为外部应用，并发布在 [Flipper 应用目录](https://github.com/flipperdevices/flipper-application-catalog)中。如果不确定，请通过 [Discord 服务器](https://flipp.dev/discord)或 [Issues](https://github.com/flipperdevices/flipperzero-firmware/issues) 页面联系我们，我们会帮你找到适合代码的位置。

还请阅读[贡献指南](https://github.com/flipperdevices/flipperzero-firmware/blob/9981d9d4aa5ec7b14bb46cf15763dd293a7dec6a/CONTRIBUTING.md)和[代码风格](https://github.com/flipperdevices/flipperzero-firmware/blob/9981d9d4aa5ec7b14bb46cf15763dd293a7dec6a/CODING_STYLE.md)，并确保你的代码与[项目许可证](https://github.com/flipperdevices/flipperzero-firmware/blob/9981d9d4aa5ec7b14bb46cf15763dd293a7dec6a/LICENSE)兼容。

最后，提交 [Pull Request](https://github.com/flipperdevices/flipperzero-firmware/pulls)，并确保 CI/CD 状态全部为绿色。

# 开发

Flipper Zero 固件使用 C 编写，部分使用 C++ 和 armv7m 汇编语言。建议具备中等水平的 C 语言知识，以便更顺畅地编程。Flipper 应用支持 C、C++ 和 armv7m 汇编语言。

## 要求

支持的开发平台：

- Windows 10+，带 PowerShell 和 Git（x86_64）
- macOS 12+，带命令行工具（x86_64、arm64）
- Ubuntu 20.04+，带 build-essential 和 Git（x86_64）

支持的在线调试器（可选但强烈推荐）：

- [Flipper Zero Wi-Fi 开发板](https://flipper.net/products/wifi-devboard)
- 兼容 CMSIS-DAP 的设备：Raspberry Pi Debug Probe 等
- ST-Link（v2、v3、v3mods）
- J-Link

Flipper 构建系统会处理其他所有依赖。

## 克隆源代码

确保有足够空间，然后克隆源代码：

```shell
git clone --recursive https://github.com/flipperdevices/flipperzero-firmware.git
```

## 构建

使用 Flipper Build Tool 构建固件：

```shell
./fbt
```

## 使用在线调试器烧写固件

将在线调试器连接到 Flipper，然后使用 Flipper Build Tool 烧写固件：

```shell
./fbt flash
```

## 使用 USB 烧写固件

确保 Flipper 已开机且固件正常运行。使用 USB 线连接 Flipper，然后使用 Flipper Build Tool 烧写固件：

```shell
./fbt flash_usb
```

## 文档

- [Flipper Build Tool](https://github.com/flipperdevices/flipperzero-firmware/blob/9981d9d4aa5ec7b14bb46cf15763dd293a7dec6a/documentation/fbt.md) - 构建、烧写和调试 Flipper 软件
- [应用](https://github.com/flipperdevices/flipperzero-firmware/blob/9981d9d4aa5ec7b14bb46cf15763dd293a7dec6a/documentation/AppsOnSDCard.md)、[应用清单](https://github.com/flipperdevices/flipperzero-firmware/blob/9981d9d4aa5ec7b14bb46cf15763dd293a7dec6a/documentation/AppManifests.md) - 开发、构建、部署和调试 Flipper 应用
- [硬件组合与解除变砖](https://github.com/flipperdevices/flipperzero-firmware/blob/9981d9d4aa5ec7b14bb46cf15763dd293a7dec6a/documentation/KeyCombo.md) - 从严重故障中恢复 Flipper
- [Flipper 文件格式](https://github.com/flipperdevices/flipperzero-firmware/tree/9981d9d4aa5ec7b14bb46cf15763dd293a7dec6a/documentation/file_formats) - Flipper 如何存储数据以及如何处理数据
- [通用遥控器](https://github.com/flipperdevices/flipperzero-firmware/blob/9981d9d4aa5ec7b14bb46cf15763dd293a7dec6a/documentation/UniversalRemotes.md) - 为通用遥控器数据库贡献红外遥控器
- [开发者文档](https://developer.flipper.net/flipperzero/doxygen)中还有更多内容

# 项目结构

- `applications`        - 固件使用的应用和服务
- `applications_users`  - 放置额外应用和服务的位置
- `assets`              - 应用和服务使用的素材
- `documentation`       - 文档生成系统配置和输入文件
- `furi`                - Furi Core：操作系统级原语和辅助工具
- `lib`                 - 项目自身及第三方的库、驱动、工具等
- `site_scons`          - 构建系统配置和模块
- `scripts`             - 辅助脚本及各种 Python 库
- `targets`             - 固件目标：平台专用代码

更多细节也请参阅这些目录中的 `ReadMe.md` 文件。

# 链接

- Discord：[flipp.dev/discord](https://flipp.dev/discord)
- Reddit：[reddit.com/r/flipperzero](https://www.reddit.com/r/flipperzero/)
- 网站：[flipper.net](https://flipper.net)
- Kickstarter：[kickstarter.com](https://www.kickstarter.com/projects/flipper-devices/flipper-zero-tamagochi-for-hackers)

## 静态应用安全测试工具

- [PVS-Studio](https://pvs-studio.com/pvs-studio/?utm_source=website&utm_medium=github&utm_campaign=open_source) - 用于 C、C++、C# 和 Java 代码的静态分析器。
