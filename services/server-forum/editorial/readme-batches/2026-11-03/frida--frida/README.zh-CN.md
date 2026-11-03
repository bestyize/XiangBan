# Frida

面向开发者、逆向工程师和安全研究人员的动态插桩工具包。更多信息请见 [frida.re](https://frida.re/)。

两种安装方式
===================

## 1. 从预构建的二进制文件安装

这是推荐的入门方式。只需执行：

    pip install frida-tools # CLI tools
    pip install frida       # Python bindings
    npm install frida       # Node.js bindings

你也可以从 GitHub 上 Frida 的[发布页面](https://github.com/frida/frida/releases)下载适用于不同操作系统的预构建二进制文件。

## 2. 自行构建二进制文件

运行：

    make

如果希望指定 `--prefix` 或其他选项，也可以先运行 `./configure`。

### 命令行工具

要运行 Frida 命令行工具，例如 `frida`、`frida-ls-devices`、`frida-ps`、`frida-kill`、`frida-trace`、`frida-discover` 等，需要安装几个包：

    pip install colorama prompt-toolkit pygments websockets

### Apple 操作系统

首先创建受信任的代码签名证书。如果你以前使用过 Xcode，很可能已经有 Apple 开发证书。可以使用以下命令检查：

    security find-identity -v -p codesigning

命令返回的证书格式如下：

    1) XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX "Apple Development: user@mail.com (YYYYYYYYYY)"

如果没有证书，请参考此指南：
https://help.apple.com/xcode/mac/current/#/dev154b28f09。

接下来，将证书名称导出到相应的环境变量，并运行 `make`：

    export MACOS_CERTID=XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
    export IOS_CERTID=XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
    export WATCHOS_CERTID=XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
    export TVOS_CERTID=XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
    make

## 了解更多

请查看我们的[文档](https://frida.re/docs/home/)。
