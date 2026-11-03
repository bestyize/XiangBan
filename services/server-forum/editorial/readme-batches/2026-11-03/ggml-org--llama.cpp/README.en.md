# llama.cpp

![llama](https://raw.githubusercontent.com/ggml-org/llama.brand/0708f2327336589bd4d3eba15a95199c318cd771/cover/llama-cpp/cover-llama-cpp-dark.svg)

<div align="center">

<b>LLM inference in C/C++</b>

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Release](https://img.shields.io/github/v/release/ggml-org/llama.cpp?filter=v*&color=brightgreen)](https://github.com/ggml-org/llama.cpp/releases?q=tag:v0)
[![Nightly](https://img.shields.io/github/v/release/ggml-org/llama.cpp?label=nightly&filter=b*&color=orange)](https://github.com/ggml-org/llama.cpp/releases?q=b)
[![Server](https://img.shields.io/github/actions/workflow/status/ggml-org/llama.cpp/server.yml?label=Server)](https://github.com/ggml-org/llama.cpp/actions/workflows/server.yml)
[![Docker](https://img.shields.io/github/actions/workflow/status/ggml-org/llama.cpp/docker.yml?label=Docker)](https://github.com/ggml-org/llama.cpp/actions/workflows/docker.yml)
[![Winget](https://img.shields.io/github/actions/workflow/status/ggml-org/llama.cpp/winget.yml?label=Winget)](https://github.com/ggml-org/llama.cpp/actions/workflows/winget.yml)

[ggml](https://github.com/ggml-org/ggml) / [ops](https://github.com/ggml-org/llama.cpp/blob/master/docs/ops.md) / [maintainer PRs](https://github.com/ggml-org/llama.cpp/pulls?q=is%3Apr%20is%3Aopen%20draft%3AFalse%20(author%3Asanmai%20OR%20author%3Argerganov%20OR%20author%3AKitaitiMakoto%20OR%20author%3Adanbev%20OR%20author%3Aaldehir%20OR%20author%3Amax-krasnyansky%20OR%20author%3ACISC%20OR%20author%3Aggerganov%20OR%20author%3Aam17an%20OR%20author%3Ajhen0409%20OR%20author%3Abartowski1182%20OR%20author%3Anikwen%20OR%20author%3Ahipudding%20OR%20author%3Aravi9%20OR%20author%3AServeurpersoCom%20OR%20author%3Athelittlefireman%20OR%20author%3Apwilkin%20OR%20author%3Areeselevine%20OR%20author%3Angxson%20OR%20author%3Ajeffbolznv%20OR%20author%3Amarty1885%20OR%20author%3A0cc4m%20OR%20author%3ATitaniumtown%20OR%20author%3Aangt%20OR%20author%3AIMbackK%20OR%20author%3Aarthw%20OR%20author%3AJohannesGaessler%20OR%20author%3AORippler%20OR%20author%3Aruixiang63%20OR%20author%3Axctan%20OR%20author%3AmasterFoad%20OR%20author%3Aallozaur%20OR%20author%3AAndreas-Krebbel%20OR%20author%3Ayomaytk%20OR%20author%3Aaendk%20OR%20author%3Awine99%20OR%20author%3Agaugarg-nv%20OR%20author%3Ataronaeo%20OR%20author%3Aforforever73%20OR%20author%3Achaxu01%20OR%20author%3Alhez%20OR%20author%3AAlekseiNikiforovIBM%20OR%20author%3Anetrunnereve%20OR%20author%3Afairydreaming%20OR%20author%3Aynankani)%20sort%3Aupdated-desc) / [dev stats](https://github.com/ggml-org/llama.cpp-dev) / [lib llama API](https://github.com/ggml-org/llama.cpp/issues/9289) / [llama-server REST API](https://github.com/ggml-org/llama.cpp/issues/9291)

</div>

## Quick start

A few options to get `llama.cpp` installed on your machine:

```bash
# curl
curl -LsSf https://llama.app/install.sh | sh

# powershell
irm https://llama.app/install.ps1 | iex
```

- Visit https://llama.app and follow the instructions
- Run with Docker - see our [Docker documentation](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/docker.md)
- Download pre-built binaries from the [releases page](https://github.com/ggml-org/llama.cpp/releases)
- Build from source by cloning this repository - check out [our build guide](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/build.md)

Once installed:

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
            <i>VLM session with <b>llama cli</b></i>
        </td>
        <td align="center">
            <img width="1392" height="958" alt="Built-in web UI against `llama serve` running Qwen 3.6" src="https://github.com/user-attachments/assets/b402f972-2e32-4def-8771-8d849f08cf2e" />
            <i>Built-in web UI against <b>llama serve</b></i>
        </td>
    </tr>
<table>

## Description

The main goal of `llama.cpp` is to enable LLM (and VLM) inference with minimal setup and state-of-the-art performance on
a wide range of hardware - locally and in the cloud.

- Plain C/C++ implementation without any dependencies
- Apple silicon is a first-class citizen - optimized via ARM NEON, Accelerate and Metal frameworks
- AVX, AVX2, AVX512 and AMX support for x86 architectures
- RVV, ZVFH, ZFH, ZICBOP and ZIHINTPAUSE support for RISC-V architectures
- 1.5-bit, 2-bit, 3-bit, 4-bit, 5-bit, 6-bit, and 8-bit integer quantization for faster inference and reduced memory use
- Custom CUDA kernels for running LLMs on NVIDIA GPUs (support for AMD GPUs via HIP and Moore Threads GPUs via MUSA)
- Vulkan and SYCL backend support
- CPU+GPU hybrid inference to partially accelerate models larger than the total VRAM capacity

The `llama.cpp` project is build on top of the [ggml](https://github.com/ggml-org/ggml) library.

## Supported backends

| Backend | Target devices |
| --- | --- |
| [BLAS](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/build.md#blas-build) | All |
| [BLIS](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/backend/BLIS.md) | All |
| [CANN](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/build.md#cann) | Huawei Ascend NPU |
| [CUDA](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/build.md#cuda) | Nvidia GPU |
| [HIP](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/build.md#hip) | AMD GPU |
| [Hexagon](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/backend/snapdragon/README.md) | Snapdragon |
| [IBM zDNN](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/backend/zDNN.md) | IBM Z & LinuxONE |
| [MUSA](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/build.md#musa) | Moore Threads GPU |
| [Metal](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/build.md#metal-build) | Apple Silicon |
| [OpenCL](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/backend/OPENCL.md) | Adreno GPU |
| [OpenVINO [In Progress]](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/backend/OPENVINO.md) | Intel CPUs, GPUs, and NPUs |
| [RPC](https://github.com/ggml-org/llama.cpp/tree/master/tools/rpc) | All |
| [SYCL](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/backend/SYCL.md) | Intel GPU |
| [VirtGPU](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/backend/VirtGPU.md) | VirtGPU APIR |
| [Vulkan](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/build.md#vulkan) | GPU |
| [WebGPU](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/build.md#webgpu) | All |
| [ZenDNN](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/build.md#zendnn) | AMD CPU |

## Documentation

#### Tools

- [cli](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/tools/cli/README.md)
- [completion](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/tools/completion/README.md)
- [server](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/tools/server/README.md)
- [GBNF grammars](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/grammars/README.md)

#### Development

- [How to build](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/build.md)
- [Running on Docker](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/docker.md)
- [Build on Android](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/android.md)
- [Multi-GPU usage](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/multi-gpu.md)
- [Performance troubleshooting](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/development/token_generation_performance_tips.md)
- [GGML tips & tricks](https://github.com/ggml-org/llama.cpp/wiki/GGML-Tips-&-Tricks)
- [XCFramework](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/xcframework.md)
- [Completions](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/completions.md)
- [Models](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/models.md)
- [Release process](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/docs/release.md)

## Contributing

- Contributors can open PRs
- Collaborators will be invited based on contributions
- Maintainers can push to branches in the `llama.cpp` repo and merge PRs into the `master` branch
- Any help with managing issues, PRs and projects is very appreciated!
- Read the [CONTRIBUTING.md](https://github.com/ggml-org/llama.cpp/blob/2cbca389f515df455412dafa07e2b1359cc72a6d/CONTRIBUTING.md) for more information

## Acknowledgements

- [yhirose/cpp-httplib](https://github.com/yhirose/cpp-httplib) - Single-header HTTP server, used by `llama-server` - MIT license
- [nothings/stb](https://github.com/nothings/stb) - Single-header image format decoder, used by multimodal subsystem - Public domain
- [nlohmann/json](https://github.com/nlohmann/json) - Single-header JSON library, used by various tools/examples - MIT License
- [mackron/miniaudio](https://github.com/mackron/miniaudio) - Single-header audio format decoder, used by multimodal subsystem - Public domain
- [sheredom/subprocess.h](https://github.com/sheredom/subprocess.h) - Single-header process launching solution for C and C++ - Public domain
