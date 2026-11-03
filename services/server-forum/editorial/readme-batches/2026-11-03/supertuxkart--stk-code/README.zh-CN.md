# SuperTuxKart
[![Linux build status](https://github.com/supertuxkart/stk-code/actions/workflows/linux.yml/badge.svg)](https://github.com/supertuxkart/stk-code/actions/workflows/linux.yml)
[![Apple build status](https://github.com/supertuxkart/stk-code/actions/workflows/apple.yml/badge.svg)](https://github.com/supertuxkart/stk-code/actions/workflows/apple.yml)
[![Windows build status](https://github.com/supertuxkart/stk-code/actions/workflows/windows.yml/badge.svg)](https://github.com/supertuxkart/stk-code/actions/workflows/windows.yml)
[![Switch build status](https://github.com/supertuxkart/stk-code/actions/workflows/switch.yml/badge.svg)](https://github.com/supertuxkart/stk-code/actions/workflows/switch.yml)
[![#supertuxkart on the libera IRC network](https://img.shields.io/badge/libera-%23supertuxkart-brightgreen.svg)](https://web.libera.chat/?channels=#supertuxkart)

SuperTuxKart 是一款免费卡丁车竞速游戏，注重乐趣而非逼真的卡丁车物理效果。使用说明可以在游戏内的帮助页面找到。

SuperTuxKart 主页：<https://supertuxkart.net/>。我们还提供[常见问题](https://supertuxkart.net/FAQ)及如何联系[社区](https://supertuxkart.net/Community)的信息。

最新正式版二进制文件可从[这里](https://github.com/supertuxkart/stk-code/releases/latest)获取，预览版可从[这里](https://github.com/supertuxkart/stk-code/releases/preview)获取。

## 硬件要求

任何支持 OpenGL >= 3.3 或 OpenGL ES >= 3.0 的硬件都应该能够运行 SuperTuxKart。Android 需要 Android 5.0 或更高版本。

这包括 2010 年后发布的 Nvidia 或 AMD 独立显卡、2012 年后发布的集成显卡，以及 2014 年后发布的 Android 手机。

游戏可在一些性能很弱的设备上运行，但更好的 GPU 有助于在更高画质和分辨率下流畅运行，更好的 CPU 则有助于在线多人游戏保持流畅体验。

更多信息请查看[常见问题](https://supertuxkart.net/FAQ)中的“What are the hardware requirements?”问题。

## 许可证

本软件按照 GNU 通用公共许可证（GPL）发布，许可证可在与本文件同目录的 [`COPYING`](https://github.com/supertuxkart/stk-code/blob/7ed766d35d7f4a265df79c74f9d3f90522ae19a3/COPYING) 中找到。

---

## 从源代码构建

构建说明见 [`INSTALL.md`](https://github.com/supertuxkart/stk-code/blob/7ed766d35d7f4a265df79c74f9d3f90522ae19a3/INSTALL.md)。

## 贡献代码

**如要向官方 STK 仓库贡献代码，请阅读[“如何贡献代码”](https://supertuxkart.net/How_to_contribute_code)指南。**其中包含重要准则，有助于让流程更顺畅并提高贡献被接受的机会。

## 3D 坐标

给查看代码和 3D 模型的人的提醒：

SuperTuxKart：X 向右，Y 向上，Z 向前

Blender：X 向右，Y 向前，Z 向上

导出工具会执行所需的转换，因此在 Blender 中只需将 XY 平面作为地面，内容在 STK 中就会正常显示（代码中显然使用 XZ 作为地面）。
