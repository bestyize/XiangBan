> 译注：原文 Slack 社区入口本次检查返回 HTTP 403；可使用原文提供的 [GitHub Issues](https://github.com/sgl-project/sglang/issues)联系项目。原链接保留。

# SGLang：面向大语言模型和多模态模型的快速推理

<p align="center" id="sglangtop">
<img src="https://raw.githubusercontent.com/sgl-project/sglang/62f9f3f785f889822ab4e78d65de9997d0b58441/assets/logo.png" alt="SGLang" width="400">
</p>

<p align="center">
  <a href="https://pypi.org/project/sglang/"><img src="https://img.shields.io/pypi/v/sglang?style=flat&amp;label=PyPI&amp;labelColor=555555&amp;color=orange" alt="PyPI version"></a>
  <a href="https://github.com/sgl-project/sglang/blob/62f9f3f785f889822ab4e78d65de9997d0b58441/LICENSE"><img src="https://img.shields.io/badge/License-Apache%202.0-green?style=flat&amp;labelColor=555555" alt="License: Apache 2.0"></a>
  <a href="https://pypistats.org/packages/sglang"><img src="https://img.shields.io/pypi/dm/sglang?style=flat&amp;label=Downloads&amp;labelColor=555555&amp;color=blue" alt="PyPI downloads per month"></a>
</p>

<p align="center">
  <a href="https://docs.sglang.io/">Docs</a> |
  <a href="https://cookbook.sglang.io/">Cookbook</a> |
  <a href="https://www.sglang.io/">Website</a> |
  <a href="https://lmsys.org/blog/">Blog</a> |
  <a href="https://slack.sglang.io/">Slack</a>
</p>

SGLang 是面向大语言模型、视觉语言模型和扩散模型的开源推理框架，针对智能体工作负载、大规模服务和强化学习 rollout 进行了优化。[SGLang Diffusion](https://docs.sglang.io/docs/sglang-diffusion) 是其内置图像与视频生成引擎，包含在本仓库和 `sglang` Python 软件包中。

👋 可从下方开始使用，也可在 [SGLang Events](https://www.sglang.io/events) 与社区交流，包括线下聚会、开发者会议、工作坊和答疑时间。

> **[SGLang Summit 2026：与开放 AI 的构建者见面](https://www.sglang.io/summit)**<br>
> **年度旗舰聚会 · 11 月 12–13 日 · 旧金山**
>
> 通过技术演讲、现场演示和实践工作坊深入了解推理、训练和 AI 硬件。带上你的部署难题，与构建相关技术栈的工程师和研究人员交流。

## 开始使用

拉取包含 SGLang 及其依赖的 Docker 镜像：

```bash
docker pull lmsysorg/sglang:latest
```

也可以在已激活的 Python 环境中使用 uv 安装 SGLang：

```bash
uv pip install --prerelease=allow sglang
```

接下来启动模型：

- [快速入门](https://docs.sglang.io/docs/get-started/quickstart):运行第一个模型并发送请求。
- [Cookbook](https://cookbook.sglang.io/):选择模型和硬件，获取可直接运行的启动命令。

## 支持的硬件

SGLang 支持多种 GPU、TPU、NPU、CPU 和 Apple Silicon 平台。

| 平台 | 代表性硬件 |
| --- | --- |
| [NVIDIA](https://docs.sglang.io/docs/hardware-platforms/nvidia-gpus) | B200/B300/GB200/GB300; H100/H200/H800/H20; A100; RTX 30/40/50 系列, RTX 6000 Ada / PRO 6000; DGX Spark, Jetson Orin |
| [AMD](https://docs.sglang.io/docs/hardware-platforms/amd_gpu) | Instinct MI300X, MI325X, MI350X, MI355X |
| [Google TPU](https://docs.sglang.io/docs/hardware-platforms/tpu) | v6e, v7; [SGLang-TorchTPU](https://github.com/sgl-project/sglang-torchtpu) / [SGL-JAX](https://github.com/sgl-project/sglang-jax) |
| Intel ([GPU](https://docs.sglang.io/docs/hardware-platforms/xpu) / [CPU](https://docs.sglang.io/docs/hardware-platforms/cpu_server)) | Arc / Arc Pro B 系列 GPU、Xeon CPU |
| [Apple Silicon](https://docs.sglang.io/docs/hardware-platforms/apple_metal) | 通过 Metal / MLX 使用 Mac |
| [Huawei Ascend](https://docs.sglang.io/docs/hardware-platforms/ascend-npus/getting-started/installation) | A2, A3, 950PR/DT NPUs |
| [Moore Threads](https://docs.sglang.io/docs/hardware-platforms/mthreads_gpu) | MTT S5000 GPUs |

正在进行集成： AWS Trainium, [Alibaba T-Head PPU](https://github.com/sgl-project/sglang/issues/37519), [Cambricon MLU](https://github.com/sgl-project/sglang/pull/26898), Qualcomm QAIC, MetaX, Hygon HCU/DCU, Iluvatar CoreX 等。

SGLang-TorchTPU 通过 PyTorch、torch_tpu 和 Pallas 内核将 SGLang 服务引擎带到 Google TPU。它支持在 TPU7x（Ironwood）上提供大语言模型和视觉语言模型服务，包括跨主机张量并行和专家并行。SGL-JAX 提供基于 JAX 的 TPU 服务路径。

请参阅 [Cookbook](https://cookbook.sglang.io/)及各平台指南，了解模型兼容性和配置方式。

## SGL 生态

| 领域 | 项目 | 用途 |
| --- | --- | --- |
| 扩散 | [SGLang Diffusion](https://docs.sglang.io/docs/sglang-diffusion/installation) | 内置于 SGLang，使用扩散模型生成图像和视频。 |
| 音频 | [SGLang Omni](https://github.com/sgl-project/sglang-omni) | 用于文本转语音（TTS）和自动语音识别（ASR）的音频模型服务。 |
| 推测解码 | [SpecForge](https://github.com/sgl-project/SpecForge) | 训练用于推测解码的草稿模型，并使用 SGLang 部署。 |
| 强化学习与后训练 | [Miles](https://github.com/radixark/miles), [slime](https://github.com/THUDM/slime), [AReaL](https://github.com/inclusionAI/AReaL), [Tunix](https://github.com/google/tunix), [verl](https://github.com/volcengine/verl) | 集成 SGLang 以生成 rollout 的训练框架。 |
| 部署与编排 | [SMG](https://github.com/smg-project/smg), [RBG](https://github.com/sgl-project/rbg), [llm-d](https://llm-d.ai/docs/dev/operations/disaggregation/sglang), [Ray Serve](https://docs.ray.io/en/latest/serve/llm/user-guides/sglang.html), [NVIDIA Dynamo](https://docs.nvidia.com/dynamo/backends/sg-lang/reference-guide) | 通过路由、负载均衡和集群编排部署与扩展 SGLang 推理服务。 |
| 教育 | [Mini-SGLang](https://github.com/sgl-project/mini-sglang), [zero-to-sglang](https://github.com/datawhalechina/zero-to-sglang), [DeepLearning.AI course](https://www.deeplearning.ai/short-courses/efficient-inference-with-sglang-text-and-image-generation/) | 通过代码和实践课程学习推理引擎设计及高效文本、图像生成。 |

## 开发与贡献

欢迎贡献，包括错误修复、文档、模型支持和性能改进。

### 开发环境配置

从 `lmsysorg/sglang:dev` Docker 镜像开始，该镜像提供开发工具和大部分依赖。在容器中克隆或挂载 SGLang 检出目录，然后从仓库根目录以可编辑模式安装，使测试使用本地 Python 修改：

```bash
pip install -e "python"
```

在已激活的虚拟环境中，也可以使用 `uv pip install --prerelease=allow -e "python"`。请参阅[开发指南](https://docs.sglang.io/docs/developer_guide/development_guide_using_docker)了解容器配置和测试方法。

### 贡献

1. Fork 仓库并为修改创建分支。较大的更改请在 [GitHub issue](https://github.com/sgl-project/sglang/issues)或 [Slack](https://slack.sglang.io/)上讨论提议。
2. 完成修改、运行相关测试，并为修复或新行为添加回归覆盖。提交前运行 `pre-commit run --all-files`。
3. 创建 pull request，说明修改及测试方法；适用时附上基准测试或准确性评估。

格式、测试和 pull request 说明请参阅[贡献者指南](https://docs.sglang.io/docs/developer_guide/contribution_guide)。文档贡献者可以从[文档指南](https://github.com/sgl-project/sglang/blob/62f9f3f785f889822ab4e78d65de9997d0b58441/docs/README.md)开始。

## 社区与赞助

SGLang 由 [LMSYS](https://lmsys.org/about/) 托管，LMSYS 是一个非营利开源组织。

- **社区讨论：** 加入 [Slack](https://slack.sglang.io/)讨论技术问题和开发。
- **活动：** 在 [SGLang Events](https://www.sglang.io/events) 查找聚会、工作坊和答疑时间。
- **更新：** 关注 [X](https://x.com/lmsysorg)和 [LinkedIn](https://www.linkedin.com/company/sgl-project/)了解项目更新，并通过 [LMSYS Blog](https://lmsys.org/blog/)查看版本公告和技术文章。
- **项目资源：** 查看[文档](https://docs.sglang.io/), [Cookbook](https://cookbook.sglang.io/), [路线图](https://roadmap.sglang.io/), [发行说明](https://github.com/sgl-project/sglang/releases), [问题跟踪器](https://github.com/sgl-project/sglang/issues),和[贡献者指南](https://docs.sglang.io/docs/developer_guide/contribution_guide).
- **联系我们：** 有关企业采用和部署、技术咨询、赞助或合作，请联系 [sglang@lmsys.org](mailto:sglang@lmsys.org).
- **贡献者赞助：** 长期活跃的 SGLang 贡献者有资格获得编码智能体赞助，包括 Cursor、Claude Code 或 OpenAI Codex。申请时请发送邮件至 [sglang@lmsys.org](mailto:sglang@lmsys.org)，并附上关键提交或 pull request 的链接。

## 受到工业界与研究界信赖

项目称 SGLang 为 AI 实验室、云平台、企业和大学的生产工作负载提供服务。

<img src="https://raw.githubusercontent.com/sgl-project/sgl-learning-materials/refs/heads/main/slides/adoption.png" alt="Organizations adopting SGLang" width="800">

## 致谢
我们借鉴了以下项目的设计并复用了其代码： [Guidance](https://github.com/guidance-ai/guidance), [vLLM](https://github.com/vllm-project/vllm), [LightLLM](https://github.com/ModelTC/lightllm), [FlashInfer](https://github.com/flashinfer-ai/flashinfer), [Outlines](https://github.com/outlines-dev/outlines)，以及 [LMQL](https://github.com/eth-sri/lmql).
