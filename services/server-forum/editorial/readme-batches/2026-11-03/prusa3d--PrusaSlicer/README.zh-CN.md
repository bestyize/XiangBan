![PrusaSlicer 标志](https://raw.githubusercontent.com/prusa3d/PrusaSlicer/ff3d6677cfccfc7fbad72acc63f01c9abec6f5d9/resources/icons/PrusaSlicer_128px.png)

# PrusaSlicer

PrusaSlicer 可以将你的 3D 模型生成 3D 打印指令并发送给 3D 打印机。它同时支持 FDM 和 mSLA 3D 打印机。该软件由 [Prusa Research](https://www.prusa3d.com/) 开发，除 Prusa 打印机外，还支持多家其他厂商的设备。

PrusaSlicer 最初基于 Alessandro Ranellucci 和 RepRap 社区的 [Slic3r](https://github.com/Slic3r/Slic3r)。

## 安装

**推荐的安装方法**是访问 [PrusaSlicer 项目页面](https://www.prusa3d.com/prusaslicer/)，按照页面说明下载安装软件。

**另一种方式**是：Windows 和 macOS 用户可以直接从 [GitHub 发布页面](https://github.com/prusa3d/PrusaSlicer/releases)下载软件。Linux 版 PrusaSlicer 目前仅通过 [Flathub](https://flathub.org/en/apps/com.prusa3d.PrusaSlicer) 分发。

如果愿意，也可以随时自行从源代码构建 PrusaSlicer。请参阅[文档](https://github.com/prusa3d/PrusaSlicer/blob/ff3d6677cfccfc7fbad72acc63f01c9abec6f5d9/doc/Build.md)了解方法。

### 主要功能

* 精确设置**打印参数**，从影响**整个打印任务**的设置到**单层调整**。
* 在打印前修改物体以满足需求：可以直接在 PrusaSlicer 中进行**涂绘**、**切割**、**排列**及**许多其他操作**。
* 同时打开**多个项目**，每个项目包含**多个打印床**（每个打印床可能有**不同设置**）。
* 充分利用**多材料打印**。
* 使用你**偏好的方式将打印任务发送给打印机**，PrusaSlicer 支持多种方式。
* 在高级 **3D 预览**中**查看生成的打印指令**。
* 通过**命令行界面**，在自动化环境中**无需 GUI 使用 PrusaSlicer**。
* 如有需要，可以利用与 [PrusaConnect](https://connect.prusa3d.com/) 和 [Printables](https://www.printables.com) 的**集成**，大幅简化工作流。

### 报告错误

发现错误了吗？可以在 [GitHub 问题跟踪器](https://github.com/prusa3d/PrusaSlicer/issues)中报告，但**首先请确保报告符合[问题跟踪器政策](https://github.com/prusa3d/PrusaSlicer?tab=contributing-ov-file#issue-tracker-policy)**。**不确定是否符合？**在问题跟踪器提交错误报告之前，请先通过 [GitHub Discussions](https://www.github.com/prusa3d/discussions) 联系我们。等更有把握时，仍可以提交错误报告。

### 如何联系

我们在此仓库维护了一个 [GitHub Discussions](https://www.github.com/prusa3d/discussions) 页面，用于**一般讨论**、**提问**和**功能请求**。此外还有一个公告分类，我们通过它直接与你沟通。

### Pull request

更多信息请阅读[贡献指南](https://github.com/prusa3d/PrusaSlicer/blob/ff3d6677cfccfc7fbad72acc63f01c9abec6f5d9/.github/CONTRIBUTING.md)。

### 技术栈

PrusaSlicer 全部使用 C++ 编写，以 CMake 作为构建系统。代码假定编译器支持 C++20。

切片后端大量使用 Angus Johnson 的 [Clipper](https://www.angusj.com/clipper2)，处理多边形布尔运算、偏移等操作。[Eigen](https://libeigen.gitlab.io/) 库用于基本类型和线性代数计算。

[wxWidgets](https://wxwidgets.org) 库用于跨平台处理 GUI 窗口和事件。大部分界面使用基于 [Yoga 布局引擎](https://github.com/facebook/yoga)和 [Dear ImGui](https://github.com/ocornut/imgui) 的自定义 OpenGL 界面工具包实现，因此不依赖特定平台。

应用还使用了许多其他库。可查看源代码树中的 [deps/](https://github.com/prusa3d/PrusaSlicer/tree/ff3d6677cfccfc7fbad72acc63f01c9abec6f5d9/deps/) 和 [bundled_deps/](https://github.com/prusa3d/PrusaSlicer/tree/ff3d6677cfccfc7fbad72acc63f01c9abec6f5d9/bundled_deps/) 文件夹，了解完整列表。感谢这些库的作者和维护者将成果开源。
