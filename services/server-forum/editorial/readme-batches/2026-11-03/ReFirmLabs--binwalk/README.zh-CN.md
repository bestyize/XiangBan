# Binwalk v3

这是固件分析工具 Binwalk 的更新版本，使用 Rust 重写，以提高速度和准确性。

![binwalk v3](https://raw.githubusercontent.com/ReFirmLabs/binwalk/26713972e3f9f52dc37d4a421c4554b0bd9e82ef/images/binwalk_animated.svg)

## 它能做什么？

Binwalk 可以识别嵌入其他文件中的文件和数据，并可选择将其提取出来。

虽然它主要用于固件分析，但也支持[多种](https://github.com/ReFirmLabs/binwalk/wiki/Supported-Signatures)文件和数据类型。

通过[熵分析](https://github.com/ReFirmLabs/binwalk/wiki/Generating-Entropy-Graphs)，它甚至可以帮助识别未知的压缩或加密！

Binwalk 可以自定义，也可以[集成](https://github.com/ReFirmLabs/binwalk/wiki/Using-the-Rust-Library)到你自己的 Rust 项目中。

## 如何获取？

安装 Binwalk 及其所有依赖最简单的方法是[构建 Docker 镜像](https://github.com/ReFirmLabs/binwalk/wiki/Building-A-Binwalk-Docker-Image)。

也可以通过 Rust 包管理器[安装](https://github.com/ReFirmLabs/binwalk/wiki/Cargo-Installation) Binwalk。

或者，你可以[从源代码编译](https://github.com/ReFirmLabs/binwalk/wiki/Compile-From-Source)！

## 如何使用？

使用_**简单**_，分析_**快速**_，结果_**详尽**_：

```
binwalk DIR-890L_AxFW110b07.bin
```
![输出示例](https://raw.githubusercontent.com/ReFirmLabs/binwalk/26713972e3f9f52dc37d4a421c4554b0bd9e82ef/images/output.png)

使用 `--help`，或查看 [Wiki](https://github.com/ReFirmLabs/binwalk/wiki#usage)，了解更多高级选项！
