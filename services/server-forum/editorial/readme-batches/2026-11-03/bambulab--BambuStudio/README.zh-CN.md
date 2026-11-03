![图片](https://user-images.githubusercontent.com/106916061/179006347-497d24c0-9bd6-45b7-8c49-d5cc8ecfe5d7.png)
# BambuStudio
Bambu Studio 是一款功能丰富的切片软件。  
它包含基于项目的工作流、经过系统优化的切片算法和易用的图形界面，旨在为用户带来流畅的打印体验。

可通过 [GitHub 发布页面](https://github.com/bambulab/BambuStudio/releases/)获取预构建的 Windows、macOS 64 位和 Linux 版本。

Bambu Studio 基于 Prusa Research 的 [PrusaSlicer](https://github.com/prusa3d/PrusaSlicer)，后者源自 Alessandro Ranellucci 和 RepRap 社区的 [Slic3r](https://github.com/Slic3r/Slic3r)。

更多信息请参阅 [Wiki](https://github.com/bambulab/BambuStudio/wiki) 和[文档目录](https://github.com/bambulab/BambuStudio/tree/79df7ae309be398ed30c9f397c13c9911c2edc57/doc)。

# Bambu Studio 的主要功能是什么？
主要功能包括：
- 基本切片功能和 GCode 查看器
- 多打印盘管理
- 远程控制和监控
- 自动排列物体
- 自动调整物体方向
- 混合、树状和普通支撑类型，以及自定义支撑
- 多材料打印和丰富的涂绘工具
- 支持多个平台（Windows、Mac、Linux）
- 全局、物体和零件级别的切片参数

其他主要功能包括：
- 控制风扇转速和动态打印速度的高级冷却逻辑
- 根据力学分析自动生成裙边
- 支持圆弧路径（G2/G3）
- 支持 STEP 格式
- 装配视图与爆炸视图
- 换料时将过渡耗材冲刷到填充或物体中

# 如何编译
目前支持以下编译平台：
- Windows 64 位，[编译指南](https://github.com/bambulab/BambuStudio/wiki/Windows-Compile-Guide)
- Mac 64 位，[编译指南](https://github.com/bambulab/BambuStudio/wiki/Mac-Compile-Guide)
- Linux，[编译指南](https://github.com/bambulab/BambuStudio/wiki/Linux-Compile-Guide)
  - 目前我们在 [GitHub 发布页面](https://github.com/bambulab/BambuStudio/releases)仅提供适用于 Ubuntu/Fedora 的 Linux AppImage，也可在所有 Linux 平台使用 [Flathub 版本](https://flathub.org/apps/com.bambulab.BambuStudio)

# 报告问题
如果问题**尚未存在**，可以在 [GitHub 问题跟踪器](https://github.com/bambulab/BambuStudio/issues)中提交。

# 许可证
Bambu Studio 采用 GNU Affero 通用公共许可证第 3 版。Bambu Studio 基于 Prusa Research 的 PrusaSlicer。

PrusaSlicer 采用 GNU Affero 通用公共许可证第 3 版，由 Prusa Research 拥有。PrusaSlicer 最初基于 Alessandro Ranellucci 的 Slic3r。

Slic3r 采用 GNU Affero 通用公共许可证第 3 版，由 Alessandro Ranellucci 在众多其他贡献者的帮助下创建。

项目说明称，GNU Affero 通用公共许可证第 3 版规定，如果以任何方式使用本软件的任何部分（即使在 Web 服务器后面），你的软件也必须采用相同许可证发布。

Bambu 网络插件基于非自由库。它是 Bambu Studio 的可选组件，为用户提供扩展网络功能。
默认情况下，安装不含网络插件的 Bambu Studio 后，切片完成后可以通过 SD 卡发起打印。
