> 译注：原文 Reddit 社区链接在本次检查中返回 HTTP 403；如无法访问，可使用原文提供的 FreeCAD 官方论坛。

<a href="https://freecad.org"><img src="https://raw.githubusercontent.com/FreeCAD/FreeCAD/f69b1eb064874749739b87bf3d951985ce51bfe3/src/Gui/Icons/freecad.svg" height="100px" width="100px"></a>

### 属于你的三维参数化建模工具

[官网](https://www.freecad.org) •
[文档](https://wiki.freecad.org) •
[论坛](https://forum.freecad.org/) •
[错误追踪](https://github.com/FreeCAD/FreeCAD/issues) •
[Git 仓库](https://github.com/FreeCAD/FreeCAD) •
[博客](https://blog.freecad.org)


[![Release](https://img.shields.io/github/release/freecad/freecad.svg)](https://github.com/freecad/freecad/releases/latest) [![Crowdin](https://d322cqt584bo4o.cloudfront.net/freecad/localized.svg)](https://crowdin.com/project/freecad)

<img src="https://raw.githubusercontent.com/FreeCAD/FreeCAD/f69b1eb064874749739b87bf3d951985ce51bfe3/.github/images/partdesign.jpg" width="800"/>

概览
--------

* **自由构建你想要的内容** FreeCAD 是一款开源的三维参数化建模工具，可用于设计任意尺寸的现实物体。参数化建模让你通过修改模型历史中的参数来更改设计。

* **在二维与三维之间转换** FreeCAD 支持绘制受几何约束的二维图形，并以其为基础构建其他物体。它还提供调整尺寸以及从三维模型创建高质量生产图纸的工具。

* **满足不同需求** 无论你是爱好者、程序员、有经验的 CAD 用户、学生还是教师，FreeCAD 都旨在适应产品设计、机械工程和建筑等广泛用途。

* **跨平台** FreeCAD 可在 Windows、macOS 和 Linux 操作系统上运行。

* **底层技术**
    * **OpenCASCADE** 强大的几何内核，是 FreeCAD 最重要的组件
    * **Coin3D 库** 符合 Open Inventor 标准的三维场景表示模型
    * **Python** FreeCAD 提供广泛的 Python API
    * **Qt** 使用 Qt 构建的图形用户界面


安装
----------

Windows、macOS 和 Linux 稳定版的预编译软件包可在[最新发布页面](https://github.com/FreeCAD/FreeCAD/releases/latest)获取。

在大多数 Linux 发行版中，也可以直接从软件中心安装 FreeCAD。

每周开发版本请访问[发布页面](https://github.com/FreeCAD/FreeCAD/releases/)。

其他选项请参阅 Wiki 的[下载页面](https://wiki.freecad.org/Download)。

编译
---------

构建说明请参阅[开发者手册——入门](https://freecad.github.io/DevelopersHandbook/gettingstarted/)。


报告问题
---------

报告问题时请：

- 考虑先在[论坛](https://forum.freecad.org)、[Discord](https://discord.com/invite/w2cTKGzccC)频道或 [Reddit](https://www.reddit.com/r/FreeCAD)发帖，确认问题； 
- 搜索已有的 [issues](https://github.com/FreeCAD/FreeCAD/issues)，检查是否有重复； 
- 使用最新稳定版或[开发版本](https://github.com/FreeCAD/FreeCAD/releases/)； 
- 提供从 `Help > About FreeCAD > Copy to clipboard` 复制的版本信息； 
- 通过 `Help > Restart in safe mode` 以安全模式重启 FreeCAD，然后再次尝试复现问题。如果问题解决，可以通过删除 FreeCAD 配置文件修复。
- 开始录制宏 `Macro > Macro recording...` 并重复所有步骤。问题出现后停止录制，并在 issue 中上传保存的宏或复制宏代码； 
- 逐步说明如何复现问题； 
- 上传示例文件（将 FCStd 作为 ZIP 文件）以展示问题； 

更多详情请参阅：

- [错误追踪](https://github.com/FreeCAD/FreeCAD/issues)
- [报告问题与请求功能](https://github.com/FreeCAD/FreeCAD/issues/new/choose)
- [贡献](https://github.com/FreeCAD/FreeCAD/blob/f69b1eb064874749739b87bf3d951985ce51bfe3/CONTRIBUTING.md)
- [帮助论坛](https://forum.freecad.org/viewforum.php?f=3)

> [!NOTE]
[FPA](https://fpa.freecad.org)为开发者提供申请资助、从事自选项目的机会。查看[职位与资助](https://blog.freecad.org/jobs/)了解更多。


使用与获取帮助
--------------------

FreeCAD Wiki 包含一般使用和 Python 脚本文档。有关详情请查看以下页面：

- [开始使用](https://wiki.freecad.org/Getting_started)
- [功能列表](https://wiki.freecad.org/Feature_list)
- [常见问题](https://wiki.freecad.org/FAQ/en)
- [工作台](https://wiki.freecad.org/Workbenches)
- [脚本](https://wiki.freecad.org/Power_users_hub)
- [开发者手册](https://freecad.github.io/DevelopersHandbook/)

[FreeCAD 论坛](https://forum.freecad.org)是学习 FreeCAD 时寻求帮助和解决具体问题的好地方。

---

<p>本项目获得了以下机构慷慨的基础设施支持：
  <a href="https://www.digitalocean.com/">
    <img src="https://opensource.nyc3.cdn.digitaloceanspaces.com/attribution/assets/SVG/DO_Logo_horizontal_blue.svg" width="91px">
  </a> 和 <a href="https://www.kipro-pcb.com/">KiCad Services Corp.</a>
</p>
