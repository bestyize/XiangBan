> 中文译文由 Terra 翻译并审校，基于上游固定版本的 [README 原文](https://github.com/huggingface/lerobot/blob/f5c9bc02137c331b009eccd44ece6f11b62187a8/README.md)；原文许可与版权声明见 [上游许可证](https://github.com/huggingface/lerobot/blob/f5c9bc02137c331b009eccd44ece6f11b62187a8/LICENSE)。

> 译注：上游列出的飞书中文教程入口检查时发生重定向循环，原链接保留；可先使用 [LeRobot 官方文档](https://huggingface.co/docs/lerobot)。代码示例与引文保留原文。

<p align="center">
  <img alt="LeRobot, Hugging Face Robotics Library" src="https://raw.githubusercontent.com/huggingface/lerobot/f5c9bc02137c331b009eccd44ece6f11b62187a8/media/readme/lerobot-logo-thumbnail.png" width="100%">
</p>

<div align="center">

[![Tests](https://github.com/huggingface/lerobot/actions/workflows/latest_deps_tests.yml/badge.svg?branch=main)](https://github.com/huggingface/lerobot/actions/workflows/latest_deps_tests.yml?query=branch%3Amain)
[![Tests](https://github.com/huggingface/lerobot/actions/workflows/docker_publish.yml/badge.svg?branch=main)](https://github.com/huggingface/lerobot/actions/workflows/docker_publish.yml?query=branch%3Amain)
[![Python versions](https://img.shields.io/pypi/pyversions/lerobot)](https://www.python.org/downloads/)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://github.com/huggingface/lerobot/blob/main/LICENSE)
[![Status](https://img.shields.io/pypi/status/lerobot)](https://pypi.org/project/lerobot/)
[![Version](https://img.shields.io/pypi/v/lerobot)](https://pypi.org/project/lerobot/)
[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-v2.1-ff69b4.svg)](https://github.com/huggingface/lerobot/blob/main/CODE_OF_CONDUCT.md)
[![Discord](https://img.shields.io/badge/Discord-Join_Us-5865F2?style=flat&logo=discord&logoColor=white)](https://discord.gg/s3KuuzsPFb)

</div>

**LeRobot** 致力于提供基于 PyTorch 的真实世界机器人模型、数据集和工具。其目标是降低入门门槛，让每个人都能为共享数据集和预训练模型作出贡献并从中受益。

🤗 与硬件无关的原生 Python 接口，统一从低成本机械臂（SO-100）到人形机器人的各类平台控制。

🤗 托管在 Hugging Face Hub 上的标准化、可扩展 LeRobotDataset 格式（Parquet + MP4 或图像），支持高效存储、流式传输和可视化海量机器人数据集。

🤗 已展示能够迁移到真实世界的先进策略，可用于训练和部署。

🤗 全面支持开源生态，推动物理 AI 普及。

## 快速入门

LeRobot 可直接从 PyPI 安装。

```bash
pip install lerobot
lerobot-info
```

基础安装较轻量。按需添加额外组件，例如用 `pip install 'lerobot[dataset]'` 加载数据集、用 `'lerobot[training]'` 训练策略，或用 `'lerobot[core_scripts,feetech]'` 通过 SO-101 机械臂录制。

> [!IMPORTANT]
> 详细安装指南请参阅[安装文档](https://huggingface.co/docs/lerobot/installation).

## 机器人与控制

<div align="center">
  <img src="https://raw.githubusercontent.com/huggingface/lerobot/f5c9bc02137c331b009eccd44ece6f11b62187a8/media/readme/robots_control_video.webp" width="640px" alt="Reachy 2 Demo">
</div>

LeRobot 提供统一的 `Robot` 类接口，将控制逻辑与具体硬件解耦。它支持多种机器人和遥操作设备。

```python
from lerobot.robots.myrobot import MyRobot

# Connect to a robot
robot = MyRobot(config=...)
robot.connect()

# Read observation and send action
obs = robot.get_observation()
action = model.select_action(obs)
robot.send_action(action)
```

**支持的硬件：** SO101、OpenARM、LeKiwi、Unitree G1、reBot B601、Koch、HopeJR、OMX、EarthRover、Reachy2、游戏手柄、键盘、手机等。

这些设备原生集成在 LeRobot 代码库中，而该库也设计为可扩展。你可以实现 Robot 接口，为自定义机器人使用 LeRobot 的数据采集、训练和可视化工具。

硬件设置详情请参阅[硬件文档](https://huggingface.co/docs/lerobot/integrate_hardware).

## LeRobot 数据集

为解决机器人领域的数据碎片化问题，我们使用 **LeRobotDataset** 格式。

- **结构：** 同步的 MP4 视频（或图像）用于视觉，Parquet 文件用于状态和动作数据。
- **HF Hub 集成：** 在 [Hugging Face Hub](https://huggingface.co/lerobot) 探索数千个机器人数据集。
- **工具：** 可以删除 episode、按索引或比例拆分、增删特征以及合并多个数据集。

```python
from lerobot.datasets.lerobot_dataset import LeRobotDataset

# Load a dataset from the Hub
dataset = LeRobotDataset("lerobot/aloha_mobile_cabinet")

# Access data (automatically handles video decoding)
episode_index=0
print(f"{dataset[episode_index]['action'].shape=}\n")
```

详情请参阅 [LeRobotDataset 文档](https://huggingface.co/docs/lerobot/lerobot-dataset-v3).

## 先进模型

LeRobot 使用纯 PyTorch 实现先进策略，涵盖模仿学习、强化学习、视觉语言动作（VLA）模型、世界模型和奖励模型，未来还会增加更多模型。它还提供工具以监测和检查训练过程。

<p align="center">
  <img alt="Gr00t Architecture" src="https://raw.githubusercontent.com/huggingface/lerobot/f5c9bc02137c331b009eccd44ece6f11b62187a8/media/readme/VLA_architecture.jpg" width="640px">
</p>

训练策略只需运行脚本配置：

```bash
lerobot-train \
  --policy.type=act \
  --dataset.repo_id=lerobot/aloha_mobile_cabinet \
  --policy.push_to_hub=false
```

| 类别                       | 模型                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| -------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **模仿学习**     | [ACT](https://github.com/huggingface/lerobot/blob/f5c9bc02137c331b009eccd44ece6f11b62187a8/docs/source/policy_act_README.md), [Diffusion](https://github.com/huggingface/lerobot/blob/f5c9bc02137c331b009eccd44ece6f11b62187a8/docs/source/policy_diffusion_README.md), [VQ-BeT](https://github.com/huggingface/lerobot/blob/f5c9bc02137c331b009eccd44ece6f11b62187a8/docs/source/policy_vqbet_README.md), [Multitask DiT Policy](https://github.com/huggingface/lerobot/blob/f5c9bc02137c331b009eccd44ece6f11b62187a8/docs/source/policy_multi_task_dit_README.md)                                                                                                                                                                                                                                                                                                                                                      |
| **强化学习** | [HIL-SERL](https://github.com/huggingface/lerobot/blob/f5c9bc02137c331b009eccd44ece6f11b62187a8/docs/source/hilserl.mdx), [TDMPC](https://github.com/huggingface/lerobot/blob/f5c9bc02137c331b009eccd44ece6f11b62187a8/docs/source/policy_tdmpc_README.md) 和 QC-FQL（即将推出）                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| **VLA 模型**            | [Pi0](https://github.com/huggingface/lerobot/blob/f5c9bc02137c331b009eccd44ece6f11b62187a8/docs/source/pi0.mdx), [Pi0Fast](https://github.com/huggingface/lerobot/blob/f5c9bc02137c331b009eccd44ece6f11b62187a8/docs/source/pi0fast.mdx), [Pi0.5](https://github.com/huggingface/lerobot/blob/f5c9bc02137c331b009eccd44ece6f11b62187a8/docs/source/pi05.mdx), [FineART-VLA](https://github.com/huggingface/lerobot/blob/f5c9bc02137c331b009eccd44ece6f11b62187a8/docs/source/fineart_vla.mdx), [GR00T N1.7](https://github.com/huggingface/lerobot/blob/f5c9bc02137c331b009eccd44ece6f11b62187a8/docs/source/policy_groot_README.md), [SmolVLA](https://github.com/huggingface/lerobot/blob/f5c9bc02137c331b009eccd44ece6f11b62187a8/docs/source/policy_smolvla_README.md), [XVLA](https://github.com/huggingface/lerobot/blob/f5c9bc02137c331b009eccd44ece6f11b62187a8/docs/source/xvla.mdx), [EO-1](https://github.com/huggingface/lerobot/blob/f5c9bc02137c331b009eccd44ece6f11b62187a8/docs/source/eo1.mdx), [MolmoAct2](https://github.com/huggingface/lerobot/blob/f5c9bc02137c331b009eccd44ece6f11b62187a8/docs/source/molmoact2.mdx), [LingBot-VLA 2.0](https://github.com/huggingface/lerobot/blob/f5c9bc02137c331b009eccd44ece6f11b62187a8/docs/source/lingbot_vla_v2.mdx), [WALL-OSS](https://github.com/huggingface/lerobot/blob/f5c9bc02137c331b009eccd44ece6f11b62187a8/docs/source/walloss.mdx), [Wall-OSS-0.5](https://github.com/huggingface/lerobot/blob/f5c9bc02137c331b009eccd44ece6f11b62187a8/docs/source/wall_oss_05.mdx), [EVO1](https://github.com/huggingface/lerobot/blob/f5c9bc02137c331b009eccd44ece6f11b62187a8/docs/source/evo1.mdx), [DM05](https://github.com/huggingface/lerobot/blob/f5c9bc02137c331b009eccd44ece6f11b62187a8/docs/source/dm05.mdx) |
| **世界模型**           | [VLA-JEPA](https://github.com/huggingface/lerobot/blob/f5c9bc02137c331b009eccd44ece6f11b62187a8/docs/source/vla_jepa.mdx), [LingBot-VA](https://github.com/huggingface/lerobot/blob/f5c9bc02137c331b009eccd44ece6f11b62187a8/docs/source/lingbot_va.mdx), [FastWAM](https://github.com/huggingface/lerobot/blob/f5c9bc02137c331b009eccd44ece6f11b62187a8/docs/source/fastwam.mdx), [Long-WAM](https://github.com/huggingface/lerobot/blob/f5c9bc02137c331b009eccd44ece6f11b62187a8/docs/source/longwam.mdx), [LaWAM](https://github.com/huggingface/lerobot/blob/f5c9bc02137c331b009eccd44ece6f11b62187a8/docs/source/lawam.mdx), [FLUX 3 Action](https://github.com/huggingface/lerobot/blob/f5c9bc02137c331b009eccd44ece6f11b62187a8/docs/source/flux3.mdx)                                                                                                                                                                                                                                                                                                                                  |
| **奖励模型**          | [SARM](https://github.com/huggingface/lerobot/blob/f5c9bc02137c331b009eccd44ece6f11b62187a8/docs/source/sarm.mdx), [TOPReward](https://github.com/huggingface/lerobot/blob/f5c9bc02137c331b009eccd44ece6f11b62187a8/docs/source/topreward.mdx), [Robometer](https://github.com/huggingface/lerobot/blob/f5c9bc02137c331b009eccd44ece6f11b62187a8/docs/source/robometer.mdx)                                                                                                                                                                                                                                                                                                                                                                                                                                                           |

与硬件类似，你可以实现自己的策略，使用 LeRobot 的数据采集、训练和可视化工具，并将模型分享到 HF Hub。

策略设置详情请参阅[策略文档](https://huggingface.co/docs/lerobot/bring_your_own_policies). 各策略的 GPU／内存要求及预计训练时间请参阅[计算硬件指南](https://huggingface.co/docs/lerobot/hardware_guide).

## 推理与评估

使用 `lerobot-eval` 在仿真中评估策略。LeRobot 支持 **LIBERO**、**MetaWorld** 等标准基准，未来还将增加更多。要在真实机器人上运行训练好的策略，请使用 `lerobot-rollout`（参阅[策略部署文档](https://huggingface.co/docs/lerobot/inference)）。

```bash
# Evaluate a policy on the LIBERO benchmark
lerobot-eval \
  --policy.path=lerobot/pi0_libero_finetuned \
  --env.type=libero \
  --env.task=libero_object \
  --eval.n_episodes=10
```

请按照 [EnvHub 文档](https://huggingface.co/docs/lerobot/envhub)学习如何实现自己的仿真环境或基准，并通过 HF Hub 分发。

### 第三方硬件

除了原生支持的硬件，社区还维护不断扩展的插件生态，支持其他机器人、遥操作器、摄像头和传感器，包括 UFACTORY xArm、Universal Robots UR5e、Franka、AgileX Piper、Trossen WidowX、ARX5、I2RT YAM、GELLO、SpaceMouse、Meta Quest、ROS 2 桥接、触觉和深度摄像头等。

插件按包名自动发现：LeRobot 会导入所有已安装且前缀为 `lerobot_robot_`、`lerobot_teleoperator_` 或 `lerobot_camera_` 的包。安装插件后即可在命令行直接使用它注册的 `type`：

```bash
pip install lerobot_robot_<name> lerobot_teleoperator_<name>

lerobot-record \
  --robot.type=<robot_name> \
  --teleop.type=<teleoperator_name> \
  --dataset.repo_id=${HF_USER}/my-dataset
```

完整列表请参阅[第三方机器人与遥操作器](https://huggingface.co/docs/lerobot/main/third_party_robots)和[第三方摄像头与传感器](https://huggingface.co/docs/lerobot/main/third_party_sensors)文档。

## 资源

- **[文档](https://huggingface.co/docs/lerobot/index):** 完整教程和 API 指南。
- **[中文教程：LeRobot+SO-ARM101中文教程-同济子豪兄](https://zihao-ai.feishu.cn/wiki/space/7589642043471924447)** 涵盖组装、遥操作、数据集、训练和部署的详细文档。经 Seed Studio 和 5 位全球黑客松参与者验证。
- **[Discord](https://discord.gg/s3KuuzsPFb):** 加入 `LeRobot` 服务器与社区讨论。
- **[X](https://x.com/LeRobotHF):** 在 X 关注我们，了解最新进展。
- **[机器人学习教程](https://huggingface.co/spaces/lerobot/robot-learning-tutorial):** 免费实践课程，使用 LeRobot 学习机器人学习。
- **[T 恤折叠实验](https://huggingface.co/spaces/lerobot/robot-folding):** 使用 LeRobot 折叠 T 恤的端到端演示。
- **[LeLab](https://github.com/huggingface/leLab):** LeRobot 的网页界面——在浏览器中遥操作、校准、录制数据集、回放及训练 SO 机械臂，无需命令行。

## 引用

如果在项目中使用 LeRobot，请引用 GitHub 仓库，以认可持续开发工作和贡献者：

```bibtex
@misc{cadene2024lerobot,
    author = {Cadene, Remi and Alibert, Simon and Soare, Alexander and Gallouedec, Quentin and Zouitine, Adil and Palma, Steven and Kooijmans, Pepijn and Aractingi, Michel and Shukor, Mustafa and Aubakirova, Dana and Russi, Martino and Capuano, Francesco and Pascal, Caroline and Choghari, Jade and Meftah, Khalil and Ellerbach, Maxime and Moss, Jess and Wolf, Thomas},
    title = {LeRobot: State-of-the-art Machine Learning for Real-World Robotics in Pytorch},
    howpublished = "\url{https://github.com/huggingface/lerobot}",
    year = {2024}
}
```

如果引用我们的研究或学术论文，也请引用我们的 ICLR 论文：

<details>
<summary><b>ICLR 2026 论文</b></summary>

```bibtex
@inproceedings{cadenelerobot,
  title={LeRobot: An Open-Source Library for End-to-End Robot Learning},
  author={Cadene, Remi and Alibert, Simon and Capuano, Francesco and Aractingi, Michel and Zouitine, Adil and Kooijmans, Pepijn and Choghari, Jade and Russi, Martino and Pascal, Caroline and Palma, Steven and Shukor, Mustafa and Moss, Jess and Soare, Alexander and Aubakirova, Dana and Lhoest, Quentin and Gallou\'edec, Quentin and Wolf, Thomas},
  booktitle={The Fourteenth International Conference on Learning Representations},
  year={2026},
  url={https://arxiv.org/abs/2602.22818}
}
```

</details>

## 贡献

欢迎社区中每个人贡献！请先阅读 [CONTRIBUTING.md](https://github.com/huggingface/lerobot/blob/main/CONTRIBUTING.md) 指南。无论是添加功能、改进文档还是修复错误，你的帮助与反馈都非常宝贵。我们对开源机器人技术的未来充满期待，期待与你共同推进下一步——感谢你的支持！

<p align="center">
  <img alt="SO101 Video" src="https://raw.githubusercontent.com/huggingface/lerobot/f5c9bc02137c331b009eccd44ece6f11b62187a8/media/readme/so100_video.webp" width="640px">
</p>

<div align="center">
<sub>由 <a href="https://huggingface.co">Hugging Face</a> 的 <a href="https://huggingface.co/lerobot">LeRobot</a> 团队用 ❤️ 构建</sub>
</div>
