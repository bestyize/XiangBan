<!-- markdownlint-disable MD001 MD041 -->
<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/vllm-project/vllm/190df0b90007eead441d97d3abf6270f84f2c63e/docs/assets/logos/vllm-logo-text-dark.png">
    <img alt="vLLM" src="https://raw.githubusercontent.com/vllm-project/vllm/190df0b90007eead441d97d3abf6270f84f2c63e/docs/assets/logos/vllm-logo-text-light.png" width=55%>
  </picture>
</p>

<h3 align="center">
让每个人都能轻松、快速、低成本地提供大语言模型服务
</h3>

<p align="center">
| <a href="https://docs.vllm.ai"><b>文档</b></a> | <a href="https://blog.vllm.ai/"><b>博客</b></a> | <a href="https://arxiv.org/abs/2309.06180"><b>论文</b></a> | <a href="https://x.com/vllm_project"><b>Twitter/X</b></a> | <a href="https://discuss.vllm.ai"><b>用户论坛</b></a> | <a href="https://slack.vllm.ai"><b>开发者 Slack</b></a> |
</p>

🔥 我们建设了 vLLM 网站，帮助你开始使用 vLLM。请访问 [vllm.ai](https://vllm.ai) 了解更多。
有关活动，请访问 [vllm.ai/events](https://vllm.ai/events) 参与。

---

## 简介

vLLM 是一个快速、易用的大语言模型推理和服务库。

vLLM 最初由加州大学伯克利分校的 [Sky Computing Lab](https://sky.cs.berkeley.edu) 开发，现已发展为活跃的开源 AI 项目之一，由来自数十所学术机构和公司的多元社区与超过 2000 名贡献者共同构建和维护。

vLLM 的高速度来自：

- 先进的服务吞吐量
- 通过以下技术高效管理注意力键和值的内存： [**PagedAttention**](https://blog.vllm.ai/2023/06/20/vllm.html)
- 对传入请求进行连续批处理、分块预填充和前缀缓存
- 使用分段和完整 CUDA/HIP 图实现快速灵活的模型执行
- 量化： FP8, MXFP8/MXFP4, NVFP4, INT8, INT4, GPTQ/AWQ, GGUF, compressed-tensors, ModelOpt, TorchAO, 及[更多](https://docs.vllm.ai/en/latest/features/quantization/index.html)
- 优化的注意力内核，包括 FlashAttention, FlashInfer, TRTLLM-GEN, FlashMLA, and Triton
- 针对不同精度优化的 GEMM/MoE 内核，使用 CUTLASS, TRTLLM-GEN, CuTeDSL
- 推测解码，包括 n-gram, suffix, EAGLE, DFlash
- 使用 torch.compile 自动生成内核并执行图级变换
- 分离式预填充、解码和编码

vLLM 灵活且易于使用，支持：

- 与流行的 Hugging Face 模型无缝集成
- 使用多种解码算法提供高吞吐量服务，包括*并行采样*、*束搜索*等
- 用于分布式推理的张量、流水线、数据、专家和上下文并行
- 流式输出
- 使用 xgrammar 或 guidance 生成结构化输出
- 工具调用和推理解析器
- 兼容 OpenAI 的 API 服务器，并支持 Anthropic Messages API 和 gRPC
- 对密集层和 MoE 层提供高效的多 LoRA 支持
- 支持 NVIDIA GPU、AMD GPU、Intel GPU 和 x86/ARM/PowerPC CPU。此外，还支持多种硬件插件，例如 Google TPU、Intel Gaudi、IBM Spyre、华为昇腾、Rebellions NPU、Apple Silicon、MetaX GPU 等。

vLLM 无缝支持 Hugging Face 上的 200 多种模型架构，包括：

- 仅解码器的大语言模型（例如 Llama, Qwen, Gemma)
- 混合专家大语言模型（例如 Mixtral, DeepSeek-V3, Qwen-MoE, GPT-OSS)
- 混合注意力和状态空间模型（例如 Mamba, Qwen3.5)
- 多模态模型（例如 LLaVA, Qwen-VL, Pixtral)
- 嵌入和检索模型（例如 E5-Mistral, GTE, ColBERT)
- 奖励和分类模型（例如 Qwen-Math)

完整的受支持模型列表见[此处](https://docs.vllm.ai/en/latest/models/supported_models.html).

## 开始使用

使用 [`uv`](https://docs.astral.sh/uv/)（推荐）或 `pip` 安装 vLLM：

```bash
uv pip install vllm
vllm download-kernels  # precompiled kernels for faster startup
```

或者为开发目的[从源代码构建](https://docs.vllm.ai/en/latest/getting_started/installation/gpu/index.html#build-wheel-from-source)。

访问我们的[文档](https://docs.vllm.ai/en/latest/)了解更多。

- [安装](https://docs.vllm.ai/en/latest/getting_started/installation.html)
- [快速开始](https://docs.vllm.ai/en/latest/getting_started/quickstart.html)
- [受支持模型列表](https://docs.vllm.ai/en/latest/models/supported_models.html)

## 参与贡献

我们欢迎并珍视各种贡献与合作。
请参阅[参与 vLLM 贡献](https://docs.vllm.ai/en/latest/contributing/index.html)，了解参与方式。

## 引用

如果你在研究中使用 vLLM，请引用我们的[论文](https://arxiv.org/abs/2309.06180)：

```bibtex
@inproceedings{kwon2023efficient,
  title={Efficient Memory Management for Large Language Model Serving with PagedAttention},
  author={Woosuk Kwon and Zhuohan Li and Siyuan Zhuang and Ying Sheng and Lianmin Zheng and Cody Hao Yu and Joseph E. Gonzalez and Hao Zhang and Ion Stoica},
  booktitle={Proceedings of the ACM SIGOPS 29th Symposium on Operating Systems Principles},
  year={2023}
}
```

## 联系我们

<!-- --8<-- [start:contact-us] -->
- 技术问题和功能请求，请使用 GitHub [Issues](https://github.com/vllm-project/vllm/issues)
- 如需与其他用户讨论，请使用 [vLLM 论坛](https://discuss.vllm.ai)
- 协调贡献和开发，请使用 [Slack](https://slack.vllm.ai)
- 披露安全问题，请使用 GitHub 的 [Security Advisories](https://github.com/vllm-project/vllm/security/advisories)功能
- 有关合作与伙伴关系，请通过以下地址联系我们： [collaboration@vllm.ai](mailto:collaboration@vllm.ai)
<!-- --8<-- [end:contact-us] -->

## 媒体素材

- 如果你希望使用 vLLM 标志，请参阅[我们的媒体素材仓库](https://github.com/vllm-project/media-kit)
