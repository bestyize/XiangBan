# MarkItDown

[![PyPI](https://img.shields.io/pypi/v/markitdown.svg)](https://pypi.org/project/markitdown/)
![PyPI - Downloads](https://img.shields.io/pypi/dd/markitdown)

> [!IMPORTANT]
> MarkItDown 使用当前进程的权限执行输入输出。和 open() 或 requests.get() 一样，它可以访问进程本身可访问的资源。在不受信任的环境中应清理输入，并调用满足需求的最窄 `convert_*` 函数（如 `convert_stream()` 或 `convert_local()`）。更多信息参见文档中的[安全注意事项](#security-considerations)。

MarkItDown 是一个轻量级 Python 工具，可将各种文件转换成 Markdown，供大语言模型及相关文本分析流程使用。它与 [textract](https://github.com/deanmalmgren/textract) 较为相似，但侧重以 Markdown 保留重要的文档结构和内容（包括标题、列表、表格、链接等）。输出通常较易阅读，但主要面向文本分析工具；如果需要供人阅读的高保真文档转换，它可能不是最佳选择。

MarkItDown 目前支持转换以下内容：

- PDF
- PowerPoint
- Word
- Excel
- 图片（EXIF 元数据和 OCR）
- 音频（EXIF 元数据和语音转写）
- HTML
- 文本格式（CSV、JSON、XML）
- ZIP 文件（逐项处理内容）
- YouTube 链接
- EPubs
- ……以及更多！

## 为什么使用 Markdown？

Markdown 非常接近纯文本，标记和格式很少，同时仍能表达重要的文档结构。OpenAI 的 GPT-4o 等主流大语言模型原生“会说” Markdown，经常无需提示就在回复中使用它。这表明它们接受过大量 Markdown 格式文本训练，并能很好地理解这种格式。另一个好处是 Markdown 惯例具有很高的 token 效率。

## 前提条件
MarkItDown 需要 Python 3.10 至 3.14。建议使用虚拟环境避免依赖冲突。

使用标准 Python 安装时，可通过以下命令创建并激活虚拟环境：

```bash
python -m venv .venv
source .venv/bin/activate
```

如果使用 `uv`，可通过以下命令创建虚拟环境：

```bash
uv venv --python=3.12 .venv
source .venv/bin/activate
# NOTE: Be sure to use 'uv pip install' rather than just 'pip install' to install packages in this virtual environment
```

如果使用 Anaconda，可通过以下命令创建虚拟环境：

```bash
conda create -n markitdown python=3.12
conda activate markitdown
```

## 安装

使用 pip 安装 MarkItDown：`pip install 'markitdown[all]'`。也可以从源码安装：

```bash
git clone git@github.com:microsoft/markitdown.git
cd markitdown
pip install -e 'packages/markitdown[all]'
```

## 使用

### 命令行

```bash
markitdown path-to-file.pdf > document.md
```

或使用 `-o` 指定输出文件：

```bash
markitdown path-to-file.pdf -o document.md
```

也可以通过管道输入内容：

```bash
cat path-to-file.pdf | markitdown
```

### 可选依赖
MarkItDown 使用可选依赖来启用不同文件格式。上文通过 `[all]` 选项安装了全部可选依赖；也可以单独安装，以便更精细地控制。例如：

```bash
pip install 'markitdown[pdf, docx, pptx]'
```

只会安装 PDF、DOCX 和 PPTX 文件所需的依赖。

目前提供以下可选依赖：

* `[all]` 安装所有可选依赖
* `[pptx]` 安装 PowerPoint 文件依赖
* `[docx]` 安装 Word 文件依赖
* `[xlsx]` 安装 Excel 文件依赖
* `[xls]` 安装旧版 Excel 文件依赖
* `[pdf]` 安装 PDF 文件依赖
* `[outlook]` 安装 Outlook 邮件依赖
* `[az-doc-intel]` 安装 Azure Document Intelligence 依赖
* `[az-content-understanding]` 安装 Azure Content Understanding 依赖
* `[audio-transcription]` 安装 wav 和 mp3 音频转写依赖
* `[youtube-transcription]` 安装获取 YouTube 视频转写内容的依赖

### 插件

MarkItDown 也支持第三方插件。插件默认禁用。列出已安装插件：

```bash
markitdown --list-plugins
```

启用插件：

```bash
markitdown --use-plugins path-to-file.pdf
```

如需查找可用插件，可在 GitHub 搜索标签 `#markitdown-plugin`。如需开发插件，请参阅 `packages/markitdown-sample-plugin`。

#### markitdown-ocr 插件

`markitdown-ocr` 插件为 PDF、DOCX、PPTX 和 XLSX 转换器增加 OCR 支持，使用 LLM Vision 从嵌入图像提取文本；它沿用 MarkItDown 图像描述使用的 `llm_client` / `llm_model` 模式，无需新增机器学习库或二进制依赖。

**安装：**

```bash
pip install markitdown-ocr
pip install openai  # or any OpenAI-compatible client
```

**用法：**

传入与图像描述相同的 `llm_client` 和 `llm_model`：

```python
from markitdown import MarkItDown
from openai import OpenAI

md = MarkItDown(
    enable_plugins=True,
    llm_client=OpenAI(),
    llm_model="gpt-4o",
)
result = md.convert("document_with_images.pdf")
print(result.markdown)
```

如果未提供 `llm_client`，插件仍会加载，但会静默跳过 OCR，转而使用标准内置转换器。

详细文档参见 [`packages/markitdown-ocr/README.md`](https://github.com/microsoft/markitdown/blob/7482c092d9c0074a89803c9fdb63a969e7fe17fc/packages/markitdown-ocr/README.md)。

### Azure Content Understanding

[Azure Content Understanding](https://learn.microsoft.com/azure/ai-services/content-understanding/) 提供质量更高的转换，支持结构化字段提取（YAML front matter）、多模态（文档、图片、音频、视频）和可配置分析器。

安装：`pip install 'markitdown[az-content-understanding]'`

#### 何时使用 Content Understanding

当你需要超出内置转换器或 Document Intelligence 转换器的功能时，Content Understanding 是合适的选择：

- **音频和视频文件**——CU 是唯一的视频选项，也是质量较高的云端音频选项。内置转换器不支持视频，仅支持基本音频转写。
- **结构化字段提取**——[预构建](https://learn.microsoft.com/azure/ai-services/content-understanding/concepts/prebuilt-analyzers)或[自定义](https://learn.microsoft.com/azure/ai-services/content-understanding/how-to/customize-analyzer-content-understanding-studio?tabs=portal)分析器提取特定领域字段（如发票金额、收据日期、合同条款），并序列化为 YAML front matter。内置转换器和 Doc Intel 集成都不会提供字段。
- **更高质量的文档提取**——针对扫描 PDF、复杂表格和多页文档的云端布局分析与 OCR。
- **所有模态共用一个 API**——一个 `cu_endpoint` 即可处理文档、图片、音频和视频，并自动路由到分析器。

| 功能 | 内置转换器 | Azure Document Intelligence | Azure Content Understanding |
|------------|---------------------|-----------------------------|-----------------------------|
| 文档转换 | 离线按格式提取 | 云端布局提取 | 云端多模态提取 |
| 结构化字段 | 不支持 | 模型或查询字段生成 YAML front matter | 分析器字段生成 YAML front matter |
| 自定义分析器／模型 | 不支持 | 通过 `docintel_model_id` 支持 | 通过 `cu_analyzer_id` 支持 |
| 音频和视频 | 基础音频，不支持视频 | 不支持 | 音视频分析器 |
| 费用 | 仅本地计算 | 按 Azure API 调用计费 | 按 Azure API 调用计费 |

**CLI:**

```bash
markitdown path-to-file.pdf --use-cu --cu-endpoint "<content_understanding_endpoint>"
```

也可以在环境中一次性设置端点，之后调用时只需 `--use-cu`：

```bash
export MARKITDOWN_CU_ENDPOINT="<content_understanding_endpoint>"
markitdown path-to-file.pdf --use-cu
```

**Python API:**

```python
from markitdown import MarkItDown

# Zero-config — auto-selects analyzer per file type
md = MarkItDown(cu_endpoint="<content_understanding_endpoint>")
result = md.convert("report.pdf")   # documents → prebuilt-documentSearch
result = md.convert("meeting.mp4")  # video → prebuilt-videoSearch
result = md.convert("call.wav")     # audio → prebuilt-audioSearch
print(result.markdown)
```

**使用自定义分析器**（提取特定领域字段）：

```python
md = MarkItDown(
    cu_endpoint="<content_understanding_endpoint>",
    cu_analyzer_id="my-invoice-analyzer",
)
result = md.convert("invoice.pdf")
print(result.markdown)
# Output includes YAML front matter with extracted fields:
# ---
# contentType: document
# fields:
#   VendorName: CONTOSO LTD.
#   InvoiceDate: '2019-11-15'
# ---
# <!-- page 1 -->
# ...
```

设置 `cu_analyzer_id` 后，转换器会根据分析器的模态自动将其限定于兼容文件类型。不兼容的类型（例如文档分析器遇到音频文件）会自动路由至默认预构建分析器。

**费用提示：**每次对路由至 CU 的格式调用 `convert()` 都是一次计费的 Azure API 调用。可使用 `cu_file_types` 限制路由至 CU 的格式：

```python
from markitdown.converters import ContentUnderstandingFileType

md = MarkItDown(
    cu_endpoint="<content_understanding_endpoint>",
    cu_file_types=[ContentUnderstandingFileType.PDF],  # only PDFs use CU
)
```

更多 Azure Content Understanding 信息见[此处](https://learn.microsoft.com/azure/ai-services/content-understanding/)。

### Azure Document Intelligence

使用 Microsoft Document Intelligence 进行转换：

```bash
export AZURE_API_KEY="<document_intelligence_api_key>"
markitdown path-to-file.pdf -o document.md -d -e "<document_intelligence_endpoint>"
```

也可以在环境中一次性设置端点，之后调用时只需 `-d`：

```bash
export MARKITDOWN_DOCINTEL_ENDPOINT="<document_intelligence_endpoint>"
markitdown path-to-file.pdf -o document.md -d
```

如果未设置 `AZURE_API_KEY`，MarkItDown 将使用 `DefaultAzureCredential`。

通过命令行选择模型、配置分析附加功能或请求字段：

```bash
markitdown document.pdf --use-docintel \
  --endpoint "<document_intelligence_endpoint>" \
  --docintel-model-id prebuilt-layout \
  --docintel-features "ocrHighResolution,styleFont" \
  --docintel-query-fields "Title,Author"
```

默认不启用任何分析附加功能。使用 `--docintel-features` 启用，或使用 `--docintel-features none` 明确指定不使用附加功能。分析附加功能和查询字段仅适用于 PDF、图片等受 OCR 支持的输入。

可用功能值为 `barcodes`、`formulas`、`keyValuePairs`、`languages`、`ocrHighResolution` 和 `styleFont`。`formulas`、`ocrHighResolution` 和 `styleFont` 会产生额外费用；`queryFields` 附加功能也会产生费用，其定价与其他附加功能不同。详情见 [Document Intelligence 附加功能文档](https://learn.microsoft.com/azure/ai-services/document-intelligence/concept/add-on-capabilities?view=doc-intel-4.0.0)和[定价](https://azure.microsoft.com/pricing/details/document-intelligence/)。

关于如何设置 Azure Document Intelligence 资源，更多信息见[此处](https://learn.microsoft.com/en-us/azure/ai-services/document-intelligence/how-to-guides/create-document-intelligence-resource?view=doc-intel-4.0.0)。

### Python API

Python 基本用法：

```python
from markitdown import MarkItDown

md = MarkItDown(enable_plugins=False) # Set to True to enable plugins
result = md.convert("test.xlsx")
print(result.markdown)
```

在 Python 中使用 API 密钥通过 Document Intelligence 转换：

```python
from azure.core.credentials import AzureKeyCredential
from markitdown import MarkItDown

md = MarkItDown(
    docintel_endpoint="<document_intelligence_endpoint>",
    docintel_credential=AzureKeyCredential("<document_intelligence_api_key>"),
)
result = md.convert("test.pdf")
print(result.markdown)
```

省略 `docintel_credential` 后，若设置了 `AZURE_API_KEY` 则使用该密钥，否则使用 `DefaultAzureCredential`。

配置模型、分析附加功能和结构化查询字段：

```python
from azure.ai.documentintelligence.models import DocumentAnalysisFeature
from markitdown import MarkItDown

md = MarkItDown(
    docintel_endpoint="<document_intelligence_endpoint>",
    docintel_model_id="prebuilt-layout",
    docintel_features=[
        DocumentAnalysisFeature.OCR_HIGH_RESOLUTION,
        DocumentAnalysisFeature.STYLE_FONT,
    ],
    docintel_query_fields=["Title", "Author"],
)
result = md.convert("test.pdf")
print(result.markdown)
```

省略 `docintel_features` 或设置 `docintel_features=[]` 可请求不使用分析附加功能。提供查询字段时，MarkItDown 会自动启用所需的 `queryFields` 附加功能。返回结构化字段的模型会将字段作为 YAML front matter 加到 Markdown 开头。单个文档使用顶层 `fields` 映射；服务返回多个文档时，front matter 使用按服务返回顺序排列的 `documents` 列表，每个文档有独立的 `fields` 映射。未提取到字段的文档在列表中保留空映射；如果所有文档字段均为空，则不输出 front matter。选择组表示为所选值的列表。

如需使用大语言模型生成图像描述（目前仅支持 pptx 和图像文件），请提供 `llm_client` 和 `llm_model`：

```python
from markitdown import MarkItDown
from openai import OpenAI

client = OpenAI(max_retries=5)
md = MarkItDown(llm_client=client, llm_model="gpt-4o", llm_prompt="optional custom prompt")
result = md.convert("example.jpg")
print(result.markdown)
```

`max_retries` 控制 OpenAI 客户端对可重试错误的自动重试次数（默认 2）；设为 `5` 最多可尝试六次，并使用退避策略。参见 [OpenAI SDK 重试文档](https://github.com/openai/openai-python#retries)。

任一尝试成功后，图像转换正常继续。如果客户端用尽重试仍报错或遇到不可重试的错误，MarkItDown 会尝试其他适用转换器；只有全部失败时才抛出 `FileConversionException`。

### Docker

```sh
docker build -t markitdown:latest .
docker run --rm -i markitdown:latest < ~/your-file.pdf > output.md
```

## 贡献

开始重大工作前，请阅读[贡献范围](#what-to-contribute)，了解哪些内容属于或不属于本仓库范围。

本项目欢迎贡献和建议。大多数贡献需要你同意贡献者许可协议（CLA），声明你有权并实际授予项目使用你的贡献的权利。详情请访问 https://cla.opensource.microsoft.com。

提交拉取请求时，CLA 机器人会自动判断你是否需要提供 CLA，并在 PR 上做相应标记（如状态检查、评论）。按照机器人的指示操作即可。对于使用该 CLA 的所有仓库，你只需完成一次。

本项目采用 [Microsoft 开源行为准则](https://opensource.microsoft.com/codeofconduct/)。更多信息请参见[行为准则常见问题](https://opensource.microsoft.com/codeofconduct/faq/)；其他问题或意见可联系 [opencode@microsoft.com](mailto:opencode@microsoft.com)。

<a id="what-to-contribute"></a>
### 贡献范围

MarkItDown 是用于将文件转换为 Markdown、供大语言模型及相关文本分析流程使用的 Python 工具。本仓库旨在提供可集成到其他系统的 Python 库，而不是构建在这些库之上的终端用户应用。

#### 范围内

- 改进现有转换器的保真度（新增格式会十分谨慎，尤其是会引入新依赖时。在大多数情况下，通过[第三方插件](#extending-markitdown-without-changing-this-repository)支持新格式更合适。）
- 修复缺陷、改进性能和修复安全问题
- `markitdown` 命令行界面
- `markitdown-mcp` 包
- 测试、文档和开发工具

#### 范围外

项目不能接受额外的应用、服务或服务器，包括：

- Web 服务器、REST 或 HTTP API 及托管转换服务
- Web 前端和基于浏览器的用户界面
- 桌面及移动应用（PyQt、PySide、Tkinter、Electron、Flutter 等）

此类项目很有用，项目方希望它们蓬勃发展，而不是被拒之门外。如果你希望为 MarkItDown 提供 Web 服务、API 或图形应用，请将其作为依赖 [PyPI 上的 `markitdown`](https://pypi.org/project/markitdown/) 的独立包或项目维护。

<a id="extending-markitdown-without-changing-this-repository"></a>
### 不修改本仓库扩展 MarkItDown

MarkItDown 支持第三方插件，因此可以独立于本仓库发布和安装对新格式的支持：

```sh
markitdown --list-plugins
markitdown --use-plugins path-to-file.pdf
```

入门请参阅 `packages/markitdown-sample-plugin`，并为仓库添加 `#markitdown-plugin` 标签以便他人找到。

### 如何贡献

你可以查看 issue 或帮助审查 PR。为方便社区贡献，项目还将部分 issue 标记为 “open for contribution”，将部分 PR 标记为 “open for reviewing”。这些标签只是建议；欢迎上述范围内的贡献。

<div align="center">

|            | 全部                                                          | 特别需要社区帮助                                                                                                      |
| ---------- | ------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------- |
| **Issues** | [所有 Issue](https://github.com/microsoft/markitdown/issues) | [欢迎贡献的 Issue](https://github.com/microsoft/markitdown/issues?q=is%3Aissue+is%3Aopen+label%3A%22open+for+contribution%22) |
| **PRs**    | [所有 PR](https://github.com/microsoft/markitdown/pulls)     | [欢迎审查的 PR](https://github.com/microsoft/markitdown/pulls?q=is%3Apr+is%3Aopen+label%3A%22open+for+reviewing%22)              |

</div>

### 运行测试与检查

- 进入 MarkItDown 包目录：

  ```sh
  cd packages/markitdown
  ```

- 在环境中安装 `hatch` 并运行测试：

  ```sh
  pip install hatch  # Other ways of installing hatch: https://hatch.pypa.io/dev/install/
  hatch shell
  hatch test
  ```

  （替代方式）使用已安装全部依赖的 Devcontainer：

  ```sh
  # Reopen the project in Devcontainer and run:
  hatch test
  ```

- 提交 PR 前运行 pre-commit 检查：`pre-commit run --all-files`

<a id="security-considerations"></a>
### 安全注意事项

MarkItDown 使用当前进程权限执行输入输出。和 `open()` 或 `requests.get()` 一样，它可以访问进程本身可访问的资源。

**清理输入：**不要将不受信任的输入直接传给 MarkItDown。如果输入的任何部分可能由不受信任的用户或系统控制（例如托管或服务器端应用），调用 MarkItDown 之前必须进行验证和限制。根据环境，这可能包括限制文件路径、URI 方案和网络目的地，以及阻止访问私有、环回、链路本地或元数据服务地址。

**仅调用需要的转换方法：**优先使用满足需求的最窄转换 API。MarkItDown 的 `convert()` 方法有意设计得较宽松，可以处理本地文件、远程 URI 和字节流。如果应用只需读取本地文件，请改用 `convert_local()`。如果需要更细致地控制 URI 获取，可自行调用 `requests.get()` 并将响应对象传给 `convert_response()`。如需最大程度的控制，可打开待转换输入的流并调用 `convert_stream()`。

## 商标

本项目可能包含项目、产品或服务的商标或标志。获授权使用 Microsoft 商标或标志时，必须遵守 [Microsoft 商标与品牌指南](https://www.microsoft.com/en-us/legal/intellectualproperty/trademarks/usage/general)。在本项目修改版本中使用 Microsoft 商标或标志不得造成混淆或暗示 Microsoft 提供赞助。任何第三方商标或标志的使用均受其各自政策约束。
