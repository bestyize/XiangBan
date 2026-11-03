[![build-shotcut-linux](https://github.com/mltframework/shotcut/workflows/build-shotcut-linux/badge.svg)](https://github.com/mltframework/shotcut/actions?query=workflow%3Abuild-shotcut-linux+is%3Acompleted+branch%3Amaster)
[![build-shotcut-macos](https://github.com/mltframework/shotcut/workflows/build-shotcut-macos/badge.svg)](https://github.com/mltframework/shotcut/actions?query=workflow%3Abuild-shotcut-macos+is%3Acompleted+branch%3Amaster)
[![build-shotcut-windows](https://github.com/mltframework/shotcut/workflows/build-shotcut-windows/badge.svg)](https://github.com/mltframework/shotcut/actions?query=workflow%3Abuild-shotcut-windows+is%3Acompleted+branch%3Amaster)


# Shotcut - 免费、开源、跨平台的**视频编辑器**

<div align="center">

<img src="https://www.shotcut.org/assets/img/screenshots/Shotcut-18.11.18.png" alt="截图" />

</div>

- 功能：https://www.shotcut.org/features/
- 路线图：https://www.shotcut.org/roadmap/

## 安装

项目定期构建二进制程序，可从 https://www.shotcut.org/download/ 下载。

## 贡献者

- Dan Dennedy <<http://www.dennedy.org>>：主要作者
- Brian Matherly <<code@brianmatherly.com>>：贡献者

## 依赖

Shotcut 的直接依赖（链接依赖或必需的运行时依赖）包括：

- [MLT](https://www.mltframework.org/)：多媒体创作框架
- [Qt 6（最低 6.4）](https://www.qt.io/)：应用程序和 UI 框架
- [FFTW](https://fftw.org/)
- [FFmpeg](https://www.ffmpeg.org/)：多媒体格式及编解码器库
- [Frei0r](https://www.dyne.org/software/frei0r/)：视频插件
- [SDL](http://www.libsdl.org/)：跨平台音频播放

包括间接依赖和捆绑依赖的更完整列表，请见 https://shotcut.org/credits/。

## 许可证

GPLv3。参见 [COPYING](https://github.com/mltframework/shotcut/blob/27e0c04dfb89a7d2f6d5ea147de6221ff17ed8c1/COPYING)。

## 如何构建

**警告**：构建 Shotcut 仅适合清楚自己在做什么的 beta 测试人员或贡献者。

### Qt Creator

构建和试用 Shotcut 开发版本最快的方法是使用 [Qt Creator](https://www.qt.io/download#qt-creator)。

### 通过命令行

首先检查依赖是否已满足，以及用于查找各种库和头文件（Qt、MLT、frei0r 等）的路径是否设置正确。

#### 配置

在用于构建的新目录中（与源代码目录分开）：

```
cmake -DCMAKE_INSTALL_PREFIX=/usr/local/ /path/to/shotcut
```

我们建议在上述命令行中添加 `-GNinja` 来使用 Ninja 生成器。

#### 构建

```
cmake --build .
```

#### 安装

如果不安装，Shotcut 运行时可能会因为找不到需要读取的 QML 文件而失败。

```
cmake --install .
```

## 翻译

如果你想将 Shotcut 翻译成其他语言，请使用 [Transifex](https://explore.transifex.com/ddennedy/shotcut/)。
