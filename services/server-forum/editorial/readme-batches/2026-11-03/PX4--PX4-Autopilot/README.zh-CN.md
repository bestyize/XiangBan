<p align="center">
  <a href="https://px4.io">
    <img src="https://raw.githubusercontent.com/PX4/PX4-Autopilot/649929706fb34bf0b06dbd2df24e04d5d452d616/docs/assets/site/px4_logo.svg" alt="PX4 Autopilot" width="240">
  </a>
</p>

<p align="center">
  <em>业界赖以构建的自动驾驶技术栈。</em>
</p>

<p align="center">
  <a href="https://github.com/PX4/PX4-Autopilot/releases"><img src="https://img.shields.io/github/release/PX4/PX4-Autopilot.svg" alt="Release"></a>
  <a href="https://zenodo.org/badge/latestdoi/22634/PX4/PX4-Autopilot"><img src="https://zenodo.org/badge/22634/PX4/PX4-Autopilot.svg" alt="DOI"></a>
  <a href="https://discord.gg/dronecode"><img src="https://img.shields.io/discord/1022170275984457759?label=discord&logo=discord&logoColor=white&color=5865F2" alt="Discord"></a>
</p>

<p align="center">
  <a href="https://www.bestpractices.dev/projects/6520"><img src="https://www.bestpractices.dev/projects/6520/badge" alt="OpenSSF Best Practices"></a>
  <a href="https://insights.linuxfoundation.org/project/px4"><img src="https://insights.linuxfoundation.org/api/badge/health-score?project=px4" alt="LFX Health Score"></a>
  <a href="https://insights.linuxfoundation.org/project/px4"><img src="https://insights.linuxfoundation.org/api/badge/contributors?project=px4" alt="LFX Contributors"></a>
  <a href="https://insights.linuxfoundation.org/project/px4"><img src="https://insights.linuxfoundation.org/api/badge/active-contributors?project=px4" alt="LFX Active Contributors"></a>
</p>

---

## 关于

PX4 是用于无人机和无人驾驶车辆的开源自动驾驶技术栈。它支持多旋翼、固定翼、垂直起降、地面车，以及从竞速四旋翼到工业测绘飞机等更多实验平台。它可运行于 [NuttX](https://nuttx.apache.org/)、Linux 和 macOS。采用 [BSD 3-Clause](https://github.com/PX4/PX4-Autopilot/blob/649929706fb34bf0b06dbd2df24e04d5d452d616/LICENSE) 许可。

## 为什么选择 PX4

**模块化架构。** PX4 围绕兼容 [DDS](https://docs.px4.io/main/en/middleware/uxrce_dds.html) 的发布/订阅中间件 [uORB](https://docs.px4.io/main/en/middleware/uorb.html) 构建。模块完全并行且线程安全。你可以构建自定义配置并裁剪不需要的部分。

**广泛的硬件支持。** PX4 运行在多种[自动驾驶仪开发板](https://docs.px4.io/main/en/flight_controller/)上，并通过 [Pixhawk](https://pixhawk.org/) 生态支持大量传感器、遥测无线电和执行器。

**对开发者友好。** 一流支持 [MAVLink](https://mavlink.io/) 和 [DDS / ROS 2](https://docs.px4.io/main/en/ros2/) 集成。提供全面的 [SITL 仿真](https://docs.px4.io/main/en/simulation/)、硬件在环测试和[日志分析](https://docs.px4.io/main/en/log/flight_log_analysis.html)工具。开发者社区通过 [Discord](https://discord.gg/dronecode) 和[每周开发者会议](https://docs.px4.io/main/en/contribute/)保持活跃。

**厂商中立的治理。** PX4 由 Linux Foundation 旗下的 [Dronecode Foundation](https://www.dronecode.org/) 托管，采用对商业友好的 BSD-3 许可证。没有单一厂商控制路线图。

## 支持的载具

<table>
  <tr>
    <td align="center">
      <a href="https://docs.px4.io/main/en/frames_multicopter/">
        <img src="https://raw.githubusercontent.com/PX4/PX4-Autopilot/649929706fb34bf0b06dbd2df24e04d5d452d616/docs/assets/airframes/types/QuadRotorX.svg" width="50" alt="多旋翼"><br>
        <sub>多旋翼</sub>
      </a>
    </td>
    <td align="center">
      <a href="https://docs.px4.io/main/en/frames_plane/">
        <img src="https://raw.githubusercontent.com/PX4/PX4-Autopilot/649929706fb34bf0b06dbd2df24e04d5d452d616/docs/assets/airframes/types/Plane.svg" width="50" alt="固定翼"><br>
        <sub>固定翼</sub>
      </a>
    </td>
    <td align="center">
      <a href="https://docs.px4.io/main/en/frames_vtol/">
        <img src="https://raw.githubusercontent.com/PX4/PX4-Autopilot/649929706fb34bf0b06dbd2df24e04d5d452d616/docs/assets/airframes/types/VTOLPlane.svg" width="50" alt="VTOL"><br>
        <sub>VTOL</sub>
      </a>
    </td>
    <td align="center">
      <a href="https://docs.px4.io/main/en/frames_rover/">
        <img src="https://raw.githubusercontent.com/PX4/PX4-Autopilot/649929706fb34bf0b06dbd2df24e04d5d452d616/docs/assets/airframes/types/Rover.svg" width="50" alt="地面车"><br>
        <sub>地面车</sub>
      </a>
    </td>
  </tr>
</table>

<sub>……以及更多载具：直升机、自转旋翼机、飞艇、潜艇、船只和其他实验平台。这些机型获得基本支持，但不属于常规飞行测试计划。请参阅<a href="https://docs.px4.io/main/en/airframes/airframe_reference.html">完整机型参考</a>。</sub>

## 试用 PX4

只需一条命令即可在仿真中运行 PX4。无需构建工具，除了 Docker 之外没有其他依赖：

```bash
docker run --rm -it -p 14550:14550/udp px4io/px4-sitl:latest
```

打开 [QGroundControl](https://qgroundcontrol.com) 开始飞行。更多选项请参阅 [PX4 仿真快速入门](https://docs.px4.io/main/en/simulation/px4_simulation_quickstart)。

## 从源代码构建

```bash
git clone https://github.com/PX4/PX4-Autopilot.git --recursive
cd PX4-Autopilot
make px4_sitl
```

> [!NOTE]
> 有关工具链设置和构建选项，请参阅[开发指南](https://docs.px4.io/main/en/development/development.html)。

## 文档与资源

| 资源 | 说明 |
| --- | --- |
| [用户指南](https://docs.px4.io/main/en/) | 构建、配置并使用 PX4 飞行 |
| [开发者指南](https://docs.px4.io/main/en/development/development.html) | 修改飞行技术栈、添加外设、移植到新硬件 |
| [机型参考](https://docs.px4.io/main/en/airframes/airframe_reference.html) | 支持的机型完整列表 |
| [自动驾驶仪硬件](https://docs.px4.io/main/en/flight_controller/) | 兼容的飞行控制器 |
| [发行说明](https://docs.px4.io/main/en/releases/) | 各版本的新增内容 |
| [贡献指南](https://docs.px4.io/main/en/contribute/) | 如何为 PX4 作贡献 |

## 社区

- **每周开发者会议** — 向所有开发者开放（[Dronecode 日历](https://www.dronecode.org/calendar/)）
- **Discord** — [加入 Dronecode 服务器](https://discord.gg/dronecode)
- **讨论论坛** — [PX4 Discuss](https://discuss.px4.io/)
- **维护者** — 参见 [`MAINTAINERS.md`](https://github.com/PX4/PX4-Autopilot/blob/649929706fb34bf0b06dbd2df24e04d5d452d616/MAINTAINERS.md)
- **贡献者统计** — [LFX Insights](https://insights.lfx.linuxfoundation.org/foundation/dronecode)

## 参与贡献

我们欢迎各种贡献，包括错误报告、文档、新功能和代码审查。请先阅读[贡献指南](https://docs.px4.io/main/en/contribute/)。

## 引用

如果在学术工作中使用 PX4，请引用它。BibTeX：

```bibtex
@software{px4_autopilot,
  author    = {Meier, Lorenz and {The PX4 Contributors}},
  title     = {{PX4 Autopilot}},
  publisher = {Zenodo},
  doi       = {10.5281/zenodo.595432},
  url       = {https://px4.io}
}
```

上述 DOI 是 Zenodo 概念 DOI，始终指向最新版本。如需固定版本的引用，请参阅 [Zenodo 记录](https://doi.org/10.5281/zenodo.595432)或 [`CITATION.cff`](https://github.com/PX4/PX4-Autopilot/blob/649929706fb34bf0b06dbd2df24e04d5d452d616/CITATION.cff)。

## 治理

PX4 Autopilot 项目由 [Linux Foundation](https://www.linuxfoundation.org/) 的协作项目 [Dronecode Foundation](https://www.dronecode.org/) 托管。Dronecode 持有全部 PX4 商标并担任项目的法律守护者，确保厂商中立的治理：没有任何一家公司拥有该名称或控制路线图。源代码采用 [BSD 3-Clause](https://github.com/PX4/PX4-Autopilot/blob/649929706fb34bf0b06dbd2df24e04d5d452d616/LICENSE) 许可，因此你可以在自己的项目中自由使用、修改和分发。

<p align="center">
  <a href="https://www.dronecode.org/">
    <img src="https://raw.githubusercontent.com/PX4/PX4-Autopilot/649929706fb34bf0b06dbd2df24e04d5d452d616/docs/assets/site/dronecode_logo.svg" alt="Dronecode Logo" width="180">
  </a>
</p>
