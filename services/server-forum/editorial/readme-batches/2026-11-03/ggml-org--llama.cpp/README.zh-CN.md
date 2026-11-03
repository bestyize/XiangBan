# llama.cpp

![llama](https://raw.githubusercontent.com/ggml-org/llama.brand/0708f2327336589bd4d3eba15a95199c318cd771/cover/llama-cpp/cover-llama-cpp-dark.svg)

<div align="center">

<b>使用 C/C++ 进行大语言模型推理</b>

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Release](https://img.shields.io/github/v/release/ggml-org/llama.cpp?filter=v*&color=brightgreen)](https://github.com/ggml-org/llama.cpp/releases?q=tag:v0)
[![Nightly](https://img.shields.io/github/v/release/ggml-org/llama.cpp?label=nightly&filter=b*&color=orange)](https://github.com/ggml-org/llama.cpp/releases?q=b)
[![Server](https://img.shields.io/github/actions/workflow/status/ggml-org/llama.cpp/server.yml?label=Server)](https://github.com/ggml-org/llama.cpp/actions/workflows/server.yml)
[![Docker](https://img.shields.io/github/actions/workflow/status/ggml-org/llama.cpp/docker.yml?label=Docker)](https://github.com/ggml-org/llama.cpp/actions/workflows/docker.yml)
[![Winget](https://img.shields.io/github/actions/workflow/status/ggml-org/llama.cpp/winget.yml?label=Winget)](https://github.com/ggml-org/llama.cpp/actions/workflows/winget.yml)

[ggml](https://github.com/ggml-org/ggml) / [ops](https://github.com/ggml-org/llama.cpp/blob/master/docs/ops.md) / [维护者 PR](https://github.com/ggml-org/llama.cpp/pulls?q=is%3Apr%20is%3Aopen%20draft%3AFalse%20(author%3Asanmai%20OR%20author%3Argerganov%20OR%20author%3AKitaitiMakoto%20OR%20author%3Adanbev%20OR%20author%3Aaldehir%20OR%20author%3Amax-krasnyansky%20OR%20author%3ACISC%20OR%20author%3Aggerganov%20OR%20author%3Aam17an%20OR%20author%3Ajhen0409%20OR%20author%3Abartowski1182%20OR%20author%3Anikwen%20OR%20author%3Ahipudding%20OR%20author%3Aravi9%20OR%20author%3AServeurpersoCom%20OR%20author%3Athelittlefireman%20OR%20author%3Apwilkin%20OR%20author%3Areeselevine%20OR%20author%3Angxson%20OR%20author%3Ajeffbolznv%20OR%20author%3Amarty1885%20OR%20author%3A0cc4m%20OR%20author%3ATitaniumtown%20OR%20author%3Aangt%20OR%20author%3AIMbackK%20OR%20author%3Aarthw%20OR%20author%3AJohannesGaessler%20OR%20author%3AORippler%20OR%20author%3Aruixiang63%20OR%20author%3Axctan%20OR%20author%3AmasterFoad%20OR%20author%3Aallozaur%20OR%20author%3AAndreas-Krebbel%20OR%20author%3Ayomaytk%20OR%20author%3Aaendk%20OR%20author%3Awine99%20OR%20author%3Agaugarg-nv%20OR%20author%3Ataronaeo%20OR%20author%3Aforforever73%20OR%20author%3Achaxu01%20OR%20author%3Alhez%20OR%20author%3AAlekseiNikiforovIBM%20OR%20author%3Anetrunnereve%20OR%20author%3Afairydreaming%20OR%20author%3Aynankani)%20sort%3Aupdated-desc) / [开发统计](https://github.com/ggml-org/llama.cpp-dev) / [lib llama API](https://github.com/ggml-org/llama.cpp/issues/9289) / [llama-server REST API](https://github.com/ggml-org/llama.cpp/issues/9291)

</div>

## 快速开始

在你的机器上安装 `llama.cpp` 的几种方式：

```bash
# curl
curl -LsSf https://llama.app/install.sh | sh

# powershell
irm https://llama.app/install.ps1 | iex
```

- 访问 https://llama.app 并按说明操作
- 使用 Docker 运行——参见 [Docker 文档](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/docker.md)
- 从[发行页面](https://github.com/ggml-org/llama.cpp/releases)下载预构建二进制文件
- 克隆本仓库并从源代码构建——参见[构建指南](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/build.md)

安装完成后：

```sh
# Download and run a model directly from Hugging Face
llama cli -hf ggml-org/Qwen3.5-0.8B-GGUF

# Launch OpenAI-compatible API server
llama serve -hf ggml-org/Qwen3.5-0.8B-GGUF
```

<table align="center">
    <tr>
        <td align="center" width=50%>
            <img width="1310" height="888" alt="VLM session with `llama cli`" src="https://github.com/user-attachments/assets/88726b48-1713-48aa-a525-95a02e78afc4" />
            <i>使用 <b>llama cli</b> 的 VLM 会话</i>
        </td>
        <td align="center">
            <img width="1392" height="958" alt="Built-in web UI against `llama serve` running Qwen 3.6" src="https://github.com/user-attachments/assets/b402f972-2e32-4def-8771-8d849f08cf2e" />
            <i>连接 <b>llama serve</b> 的内置 Web 界面</i>
        </td>
    </tr>
<table>

## 简介

`llama.cpp` 的主要目标是在本地和云端的多种硬件上，以尽可能少的配置实现大语言模型（LLM）及视觉语言模型（VLM）推理，并提供先进的性能。

- 纯 C/C++ 实现，无任何依赖
- 将 Apple silicon 作为重点支持平台，通过 ARM NEON、Accelerate 和 Metal 框架优化
- 在 x86 架构上支持 AVX、AVX2、AVX512 和 AMX
- 在 RISC-V 架构上支持 RVV、ZVFH、ZFH、ZICBOP 和 ZIHINTPAUSE
- 支持 1.5、2、3、4、5、6 和 8 位整数量化，以加快推理并减少内存占用
- 自定义 CUDA 内核用于在 NVIDIA GPU 上运行大语言模型（通过 HIP 支持 AMD GPU，通过 MUSA 支持摩尔线程 GPU）
- 支持 Vulkan 和 SYCL 后端
- CPU+GPU 混合推理，可为超过显存总容量的模型提供部分加速

`llama.cpp` 项目构建于 [ggml](https://github.com/ggml-org/ggml) 库之上。

## 支持的后端

| 后端 | 目标设备 |
| --- | --- |
| [BLAS](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/build.md#blas-build) | 所有设备 |
| [BLIS](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/backend/BLIS.md) | 所有设备 |
| [CANN](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/build.md#cann) | 华为昇腾 NPU |
| [CUDA](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/build.md#cuda) | Nvidia GPU |
| [HIP](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/build.md#hip) | AMD GPU |
| [Hexagon](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/backend/snapdragon/README.md) | Snapdragon |
| [IBM zDNN](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/backend/zDNN.md) | IBM Z & LinuxONE |
| [MUSA](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/build.md#musa) | Moore Threads GPU |
| [Metal](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/build.md#metal-build) | Apple Silicon |
| [OpenCL](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/backend/OPENCL.md) | Adreno GPU |
| [OpenVINO [进行中]](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/backend/OPENVINO.md) | Intel CPU、GPU 和 NPU |
| [RPC](https://github.com/ggml-org/llama.cpp/tree/master/tools/rpc) | 所有设备 |
| [SYCL](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/backend/SYCL.md) | Intel GPU |
| [VirtGPU](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/backend/VirtGPU.md) | VirtGPU APIR |
| [Vulkan](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/build.md#vulkan) | GPU |
| [WebGPU](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/build.md#webgpu) | 所有设备 |
| [ZenDNN](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/build.md#zendnn) | AMD CPU |

## 文档

#### 工具

- [cli](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/tools/cli/README.md)
- [completion](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/tools/completion/README.md)
- [server](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/tools/server/README.md)
- [GBNF grammars](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/grammars/README.md)

#### 开发

- [如何构建](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/build.md)
- [在 Docker 中运行](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/docker.md)
- [在 Android 上构建](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/android.md)
- [多 GPU 使用](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/multi-gpu.md)
- [性能故障排查](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/development/token_generation_performance_tips.md)
- [GGML 技巧](https://github.com/ggml-org/llama.cpp/wiki/GGML-Tips-&-Tricks)
- [XCFramework](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/xcframework.md)
- [补全](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/completions.md)
- [模型](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/models.md)
- [发布流程](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/release.md)

## 参与贡献

- 贡献者可以提交 PR
- 协作者将根据贡献情况获得邀请
- 维护者可以推送到 `llama.cpp` 仓库的分支，并将 PR 合并到 `master` 分支
- 非常感谢任何协助管理 issue、PR 和项目的帮助！
- 阅读 [CONTRIBUTING.md](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/CONTRIBUTING.md)了解更多信息

## 致谢

- [yhirose/cpp-httplib](https://github.com/yhirose/cpp-httplib) - 单头文件 HTTP 服务器，供 `llama-server` 使用 - MIT 许可
- [nothings/stb](https://github.com/nothings/stb) - 单头文件图像格式解码器，供多模态子系统使用 - 公有领域
- [nlohmann/json](https://github.com/nlohmann/json) - 单头文件 JSON 库，供多种工具和示例使用 - MIT 许可
- [mackron/miniaudio](https://github.com/mackron/miniaudio) - 单头文件音频格式解码器，供多模态子系统使用 - 公有领域
- [sheredom/subprocess.h](https://github.com/sheredom/subprocess.h) - C 和 C++ 的单头文件进程启动方案 - 公有领域
