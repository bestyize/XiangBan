> 译注：性能与安全等主张来自上游项目，尚未独立验证。代码和代码注释保留原文；上游失效的 LlamaParse 章节锚点已在中英文阅读稿中修正，原文快照保留。

# 🗂️ LlamaIndex Framework 🦙

[![PyPI - Downloads](https://img.shields.io/pypi/dm/llama-index)](https://pypi.org/project/llama-index/)
[![Build](https://github.com/run-llama/llama_index/actions/workflows/build_package.yml/badge.svg)](https://github.com/run-llama/llama_index/actions/workflows/build_package.yml)
[![GitHub contributors](https://img.shields.io/github/contributors/jerryjliu/llama_index)](https://github.com/jerryjliu/llama_index/graphs/contributors)
[![Discord](https://img.shields.io/discord/1059199217496772688)](https://discord.gg/dGcwcsnxhU)
[![Twitter](https://img.shields.io/twitter/follow/llama_index)](https://x.com/llama_index)
[![Reddit](https://img.shields.io/reddit/subreddit-subscribers/LlamaIndex?style=plastic&logo=reddit&label=r%2FLlamaIndex&labelColor=white)](https://www.reddit.com/r/LlamaIndex/)

> [!NOTE]
> LlamaIndex 当前重点是构建由 AI 驱动的文档解析与提取引擎。 **[LlamaParse](https://cloud.llamaindex.ai?utm_medium=li_github&utm_source=github&utm_campaign=2026--)**是我们的企业平台，提供智能体 OCR、解析、提取、索引等功能。 [LiteParse](https://github.com/run-llama/liteparse)代表我们构建免费、快速、低成本文本解析器的努力。 [ParseBench](https://www.parsebench.ai/)和 [ExtractBench](https://www.extractbench.ai/)体现了我们对解析与提取开放基准测试的投入。
>
> 自这个开源框架于三年前（2023 年）首次推出以来，公司本身也经历了演变。从早期开始，框架就包含一系列编排工具，使开发者能够构建各种 RAG 和智能体应用。
>
> 虽然开源框架仍作为开放工具包供大家使用，我们的主要重心已转向 LlamaParse、liteparse 和基准测试工作。我们认为智能体是文档的新消费者，它们需要合适的工具，才能大规模、准确且低成本地从最复杂的文档中提取上下文。无论你是处理文档的 AI 初创公司，还是希望自动化文档工作流的企业，都欢迎[联系我们](https://www.llamaindex.ai/contact)。

LlamaIndex Framework（由 [LlamaIndex](https://llamaindex.ai?utm_medium=li_github&utm_source=github&utm_campaign=2026--)，即 LlamaParse 背后的公司开发）是一个构建智能体应用的开源框架。LlamaParse 可配合此框架或单独使用；请参阅下方 [LlamaParse](#llamaparse文档智能体平台)了解注册和产品链接。

> ### 📚 **文档：**
>
> - [LlamaParse](https://developers.llamaindex.ai/llamaparse/?utm_medium=li_github&utm_source=github&utm_campaign=2026--)
> - [LlamaIndex Framework](https://developers.llamaindex.ai/python/framework/?utm_medium=li_github&utm_source=github&utm_campaign=2026--)
> - [LlamaAgents](https://developers.llamaindex.ai/python/llamaagents/overview/?utm_medium=li_github&utm_source=github&utm_campaign=2026--)

使用 LlamaIndex 构建通常需要 LlamaIndex 核心及一组选定的集成（或插件）。在 Python 中有两种入门方式：

1. **入门包**: [`llama-index`](https://pypi.org/project/llama-index/).一个入门 Python 软件包，包含 LlamaIndex 核心及部分集成。

2. **自定义**: [`llama-index-core`](https://pypi.org/project/llama-index-core/).安装 LlamaIndex 核心，并从[集成页面](https://developers.llamaindex.ai/python/framework/community/integrations)添加应用需要的集成包。超过 300 个 LlamaIndex 集成包可与核心协同工作，让你使用偏好的 LLM、嵌入模型和向量存储提供方。

LlamaIndex Python 库使用命名空间：包含 `core` 的导入语句表示使用核心包，不包含 `core` 的语句则表示使用集成包。

```python
# typical pattern
from llama_index.core.xxx import ClassABC  # core submodule xxx
from llama_index.xxx.yyy import (
    SubclassABC,
)  # integration yyy for submodule xxx

# concrete example
from llama_index.core.llms import LLM
from llama_index.llms.openai import OpenAI
```

### LlamaParse（文档智能体平台）

**LlamaParse** 是独立的平台，专注文档智能体和智能体 OCR。它包括 **Parse**（解析）、**LlamaAgents**（已部署的文档智能体）、**Extract**（结构化提取）和 **Index**（摄入与 RAG）。可与 LlamaIndex 框架配合或单独使用。

- **[注册 LlamaParse](https://cloud.llamaindex.ai?utm_medium=li_github&utm_source=github&utm_campaign=2026--)**——创建账户并获取 API 密钥。
- **Parse**——智能体 OCR 和文档解析（130 多种格式）。 [文档](https://developers.llamaindex.ai/python/cloud/llamaparse/?utm_medium=li_github&utm_source=github&utm_campaign=2026--)
- **Extract**——从文档中提取结构化数据。 [文档](https://developers.llamaindex.ai/python/cloud/llamaextract/?utm_medium=li_github&utm_source=github&utm_campaign=2026--)
- **Index**——摄入、索引和 RAG 流水线。 [文档](https://developers.llamaindex.ai/llamaparse/cloud-index-v2/getting_started/?utm_medium=li_github&utm_source=github&utm_campaign=2026--)
- **Split**——将大型文档拆分为子类别。 [文档](https://developers.llamaindex.ai/python/cloud/split/getting_started/?utm_medium=li_github&utm_source=github&utm_campaign=2026--)
- **Agents**——使用 `Workflows` 和 Agent Builder 构建端到端文档智能体。 [文档](https://developers.llamaindex.ai/python/llamaagents/overview/?utm_medium=li_github&utm_source=github&utm_campaign=2026--)

### 重要链接

[文档](https://developers.llamaindex.ai/python/framework/?utm_medium=li_github&utm_source=github&utm_campaign=2026--)

[X (formerly Twitter)](https://x.com/llama_index)

[LinkedIn](https://www.linkedin.com/company/llamaindex/)

[Reddit](https://www.reddit.com/r/LlamaIndex/)

[Discord](https://discord.gg/dGcwcsnxhU)

## 🚀 概览

**注意**：此 README 的更新频率不如文档。有关最新信息，请查看上方文档！

### 背景

- 大语言模型是一项用于知识生成和推理的强大技术，在大量公开数据上预训练。
- 如何使用自己的私有数据更好地增强大语言模型？

我们需要一套完整工具来帮助为大语言模型进行数据增强。

### 提议的方案

这就是 **LlamaIndex** 的用武之地。LlamaIndex 是帮助构建 LLM 应用的“数据框架”，提供以下工具：

- 提供**数据连接器**，摄入现有数据源和格式（API、PDF、文档、SQL 等）。
- 提供**组织数据结构**的方式（索引、图），使数据可方便地供大语言模型使用。
- 提供针对数据的**高级检索／查询接口**：输入 LLM 提示词，返回检索到的上下文和知识增强输出。
- 方便与外部应用框架集成，例如 LangChain、Flask、Docker、ChatGPT 等。

LlamaIndex 同时为初学者和高级用户提供工具。高层 API 让初学者使用 5 行代码摄入并查询数据；底层 API 允许高级用户自定义和扩展任意模块（数据连接器、索引、检索器、查询引擎和重排模块），以适应需求。

## 💡 贡献

有兴趣贡献？我们接受并鼓励对 LlamaIndex 核心及基于核心的集成做贡献！更多详情请参阅[贡献指南](https://github.com/run-llama/llama_index/blob/619d74664e8404612038bc15a1a40c686037277c/CONTRIBUTING.md)。

新集成应与现有 LlamaIndex 框架组件进行有意义的集成。LlamaIndex 维护者可自行决定拒绝某些集成。

## 📄 文档

完整文档可[在此查看](https://developers.llamaindex.ai/python/framework/?utm_medium=li_github&utm_source=github&utm_campaign=2026--)

请查看最新教程、操作指南、参考和其他资源！

## 💻 使用示例

```sh
# custom selection of integrations to work with core
pip install llama-index-core
pip install llama-index-llms-openai
pip install llama-index-llms-ollama
pip install llama-index-embeddings-huggingface
```

示例位于 `docs/examples` 文件夹，索引位于 `indices` 文件夹（参见下方索引列表）。

使用 OpenAI 构建简单的向量存储索引：

```python
import os

os.environ["OPENAI_API_KEY"] = "YOUR_OPENAI_API_KEY"

from llama_index.core import VectorStoreIndex, SimpleDirectoryReader

documents = SimpleDirectoryReader("YOUR_DATA_DIRECTORY").load_data()
index = VectorStoreIndex.from_documents(documents)
```

使用非 OpenAI 的 LLM（例如通过 Ollama 托管的模型）构建简单向量存储索引：

```python
from llama_index.core import Settings, VectorStoreIndex, SimpleDirectoryReader
from llama_index.embeddings.huggingface import HuggingFaceEmbedding
from llama_index.llms.ollama import Ollama
from transformers import AutoTokenizer

# set the LLM
Settings.llm = Ollama(
    model="llama-3.1:latest",
    request_timeout=360.0,
)

# set tokenizer to match LLM
Settings.tokenizer = AutoTokenizer.from_pretrained(
    "meta-llama/Llama-3.1-8B-Instruct"
)

# set the embed model
Settings.embed_model = HuggingFaceEmbedding(
    model_name="BAAI/bge-small-en-v1.5"
)

documents = SimpleDirectoryReader("YOUR_DATA_DIRECTORY").load_data()
index = VectorStoreIndex.from_documents(
    documents,
)
```

查询：

```python
query_engine = index.as_query_engine()
query_engine.query("YOUR_QUESTION")
```

默认情况下数据保存在内存中。要持久化到磁盘（位于 `./storage`）：

```python
index.storage_context.persist()
```

从磁盘重新加载：

```python
from llama_index.core import StorageContext, load_index_from_storage

# rebuild storage context
storage_context = StorageContext.from_defaults(persist_dir="./storage")
# load index
index = load_index_from_storage(storage_context)
```

## 关于构建资源验证的说明

默认情况下，`llama-index-core` 包含 `_static` 文件夹，存放随软件包安装的 nltk 和 tiktoken 缓存。这使你可以在运行时磁盘访问权限受限的环境中轻松运行 `llama-index`。

为验证这些文件安全有效，项目使用 GitHub `attest-build-provenance` action 检查 `_static` 文件夹中的文件是否与 `llama-index-core/llama_index/core/_static` 中的文件相同。

可以运行以下脚本验证（指向已安装的软件包）：

```bash
#!/bin/bash
STATIC_DIR="venv/lib/python3.13/site-packages/llama_index/core/_static"
REPO="run-llama/llama_index"

find "$STATIC_DIR" -type f | while read -r file; do
    echo "Verifying: $file"
    gh attestation verify "$file" -R "$REPO" || echo "Failed to verify: $file"
done
```

## 📖 引用

如果在论文中使用 LlamaIndex，请引用以下文献：

```
@software{Liu_LlamaIndex_2022,
author = {Liu, Jerry},
doi = {10.5281/zenodo.1234},
month = {11},
title = {{LlamaIndex}},
url = {https://github.com/jerryjliu/llama_index},
year = {2022}
}
```
