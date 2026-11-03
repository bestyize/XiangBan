<div align="center">

![logo](https://raw.githubusercontent.com/googleapis/mcp-toolbox/ea2d71e7c2636f00158d2e89b46bdaf36182ceb6/logo.png)

# MCP Toolbox for Databases

<a href="https://trendshift.io/repositories/25495" target="_blank"><img src="https://trendshift.io/api/badge/repositories/25495" alt="googleapis%2Fmcp-toolbox | Trendshift" style="width: 250px; height: 55px;" width="250" height="55"/></a>

[![License: Apache
2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Docs](https://img.shields.io/badge/Docs-MCP_Toolbox-blue)](https://mcp-toolbox.dev/)
[![Discord](https://img.shields.io/badge/Discord-%235865F2.svg?style=flat&logo=discord&logoColor=white)](https://discord.gg/Dmm69peqjh)
[![Medium](https://img.shields.io/badge/Medium-12100E?style=flat&logo=medium&logoColor=white)](https://medium.com/@mcp_toolbox)

[![Python SDK](https://img.shields.io/pypi/v/toolbox-core?logo=python&logoColor=white&label=Python%20SDK)](https://pypi.org/project/toolbox-core/)
[![JS/TS SDK](https://img.shields.io/npm/v/@toolbox-sdk/core?logo=javascript&logoColor=white&label=JS%20SDK)](https://www.npmjs.com/package/@toolbox-sdk/core)
[![Go SDK](https://img.shields.io/github/v/release/googleapis/mcp-toolbox-sdk-go?logo=go&logoColor=white&label=Go%20SDK)](https://pkg.go.dev/github.com/googleapis/mcp-toolbox-sdk-go)
[![Java SDK](https://img.shields.io/maven-central/v/com.google.cloud.mcp/mcp-toolbox-sdk-java?logo=apache-maven&logoColor=white&label=Java%20SDK)](https://mvnrepository.com/artifact/com.google.cloud.mcp/mcp-toolbox-sdk-java)
</div>

MCP Toolbox for Databases 是开源的模型上下文协议（MCP）服务器，可将 AI 智能体、IDE 和应用直接连接到企业数据库。

<p align="center">
<img src="https://raw.githubusercontent.com/googleapis/mcp-toolbox/ea2d71e7c2636f00158d2e89b46bdaf36182ceb6/docs/en/documentation/introduction/architecture.png" alt="架构" width="50%"/>
</p>

它有**两种用途**：
1. **即用型 MCP 服务器（构建时）：**使用*预构建通用工具*，立即将 Gemini CLI、Google Antigravity、Claude Code、Codex 或其他 MCP 客户端连接到数据库。无需编写样板代码即可与数据对话、探索模式并生成代码。
2. **自定义工具框架（运行时）：**项目描述为可为生产智能体构建专用、高安全性 AI 工具的框架，可安全、便捷地定义结构化查询、语义搜索和自然语言转 SQL 能力。

本 README 提供简要概览。完整细节请参阅[完整文档](https://mcp-toolbox.dev/)。

> [!IMPORTANT]
> **仓库名称更新：**`genai-toolbox` 仓库已正式更名为 `mcp-toolbox`。为使本地环境反映新名称，可以更新远程地址：
> `git remote set-url origin https://github.com/googleapis/mcp-toolbox.git`

> [!NOTE]
> 此方案最初名为“Gen AI Toolbox for Databases”（github.com/googleapis/genai-toolbox），因为它最初的开发早于 MCP；后来为体现 MCP 兼容性而更名。

<!-- TOC ignore:true -->
## 目录

- [为什么使用 MCP Toolbox？](#为什么使用-mcp-toolbox)
- [快速入门：预构建工具](#快速入门预构建工具)
- [快速入门：自定义工具](#快速入门自定义工具)
- [安装和运行 Toolbox 服务器](#安装和运行-toolbox-服务器)
- [连接 Toolbox](#连接-toolbox)
  - [MCP 客户端](#mcp-客户端)
  - [Toolbox SDK：集成到应用](#toolbox-sdk集成到应用)
- [其他功能](#其他功能)
- [版本管理](#版本管理)
- [参与贡献](#参与贡献)
- [社区](#社区)

---

## 为什么使用 MCP Toolbox？

- **开箱即用的数据库访问：**从 IDE 或 CLI 直接使用预构建通用工具（如 `list_tables`、`execute_sql`）即时探索数据。
- **自定义工具框架：**使用自定义预定义逻辑构建可用于生产的工具，项目称可通过受限访问、结构化查询和语义搜索保障安全。
- **简化开发：**使用不到 10 行代码，将工具集成到 Agent Development Kit（ADK）、LangChain、LlamaIndex 或自定义智能体。
- **更好的性能：**开箱即用地处理连接池、集成身份验证（IAM）和端到端可观测性（OpenTelemetry）。
- **增强安全性：**通过集成身份验证更安全地访问数据。
- **端到端可观测性：**内置对 OpenTelemetry 的支持，开箱即用地提供指标与追踪。

---

## 快速入门：预构建工具

无需频繁切换上下文，让 AI 助手成为真正的协同开发者。通过 MCP Toolbox 将 IDE 连接到数据库，可以用自然语言查询数据、自动发现和管理模式，并生成了解数据库的代码。

通过配置 MCP 服务器，你可以在任何兼容 MCP 的 IDE 或客户端中使用 Toolbox（如 Gemini CLI、Google Antigravity、Claude Code、Codex 等）。

**预构建工具也可以通过 [Google Antigravity MCP Store](https://antigravity.google/docs/mcp) 一键安装。**

1. 将以下内容添加到客户端的 MCP 配置文件（通常是 `mcp.json` 或 `claude_desktop_config.json`）：

    ```json
    {
      "mcpServers": {
        "toolbox-postgres": {
          "command": "npx",
          "args": [
            "-y",
            "@toolbox-sdk/server",
            "--prebuilt=postgres",
            "--stdio"
          ]
        }
      }
    }
    ```

2. 设置适当的连接环境变量，参阅[预构建工具参考](https://mcp-toolbox.dev/documentation/configuration/prebuilt-configs/)。

使用 `--prebuilt=<database>` 标志运行 Toolbox 时，可以立即使用与该数据库交互的标准工具。也可以通过 `--prebuilt=<database>/<toolset>` 指定特定工具集（例如 `--prebuilt=postgres/data` 只加载 SQL 工具）。

目前支持的数据库包括：
- **Google Cloud：**AlloyDB、BigQuery、Cloud SQL（PostgreSQL、MySQL、SQL Server）、Spanner、Firestore、Knowledge Catalog（原名 Dataplex）。
- **其他数据库：**PostgreSQL、MySQL、[MariaDB](https://mcp-toolbox.dev/integrations/mariadb/source/)、SQL Server、Oracle、MongoDB、Redis、Elasticsearch、CockroachDB、ClickHouse、Couchbase、Neo4j、Snowflake、Trino 等。

所有受支持数据库的可用工具与能力完整列表，参见[预构建工具参考](https://mcp-toolbox.dev/documentation/configuration/prebuilt-configs/)。

*有关 Docker、二进制等不同执行方式，请参阅[安装和运行 Toolbox 服务器](#安装和运行-toolbox-服务器)。*

> [!TIP]
> 如果需要托管方案，[Google Cloud MCP Servers](https://cloud.google.com/blog/products/databases/managed-mcp-servers-for-google-cloud-databases) 提供带预构建工具的托管 MCP 体验；可[在此了解差异](https://mcp-toolbox.dev/dev/reference/faq/)。

---

<a id="configuration"></a>
## 快速入门：自定义工具

Toolbox 也可用作自定义工具框架。配置 Toolbox 的主要方式是 `tools.yaml` 文件。如果有多个文件，可用 `--config tools.yaml` 标志指定加载哪一个。

所有资源类型的详细参考文档见[资源](https://mcp-toolbox.dev/documentation/configuration/)。

### 数据源

`tools.yaml` 的 `sources` 部分定义 Toolbox 可访问的数据源。大多数工具至少需要一个执行所依赖的数据源。

```yaml
kind: source
name: my-pg-source
type: postgres
host: 127.0.0.1
port: 5432
database: toolbox_db
user: toolbox_user
password: my-password
```

有关配置不同类型数据源的更多信息，请参阅[数据源](https://mcp-toolbox.dev/documentation/configuration/sources/)。

### 工具

`tools.yaml` 的 `tools` 部分定义智能体可以执行的操作：工具类型、影响的数据源、使用的参数等。

```yaml
kind: tool
name: search-hotels-by-name
type: postgres-sql
source: my-pg-source
description: Search for hotels based on name.
parameters:
  - name: name
    type: string
    description: The name of the hotel.
statement: SELECT * FROM hotels WHERE name ILIKE '%' || $1 || '%';
```

有关配置不同类型工具的更多信息，请参阅[工具](https://mcp-toolbox.dev/documentation/configuration/tools/)。

### 工具集

`tools.yaml` 的 `toolsets` 部分可定义希望一起加载的工具组，可用于根据智能体或应用定义不同分组。

```yaml
kind: toolset
name: my_first_toolset
tools:
    - my_first_tool
    - my_second_tool
---
kind: toolset
name: my_second_toolset
tools:
    - my_second_tool
    - my_third_tool
```

### 提示词

`tools.yaml` 的 `prompts` 部分定义可用于与大语言模型交互的提示词。

```yaml
kind: prompt
name: code_review
description: "Asks the LLM to analyze code quality and suggest improvements."
messages:
  - content: >
         Please review the following code for quality, correctness,
         and potential improvements: \n\n{{.code}}
arguments:
  - name: "code"
    description: "The code to review"
```

有关配置提示词的更多信息，请参阅[提示词](https://mcp-toolbox.dev/documentation/configuration/prompts/)。

### 资源

`tools.yaml` 的 `resources` 和 `resourceTemplates` 部分定义 MCP 客户端可发现和检索的只读内容、文件或参数化目录树：

```yaml
kind: resource
name: database_schema_ddl
type: text
description: "Core table definitions and constraints."
mimeType: text/x-sql
text: |
  CREATE TABLE customers (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL
  );
---
kind: resource
name: database_schema
type: file
description: "PostgreSQL schema definition."
path: "./schema.sql"
---
kind: resourceTemplate
name: server_logs
type: file
description: "Application log files."
uriTemplate: "file:///var/log/{path}"
allowedPaths:
  - "/var/log"
```

有关配置资源与资源模板的更多信息，请参阅[资源](https://mcp-toolbox.dev/documentation/configuration/resources/)。

---

## 安装和运行 Toolbox 服务器

可以直接使用[配置文件](#快速入门自定义工具)运行 Toolbox：

```sh
npx @toolbox-sdk/server --config tools.yaml
```

这会使用你的配置文件运行最新版本的 Toolbox 服务器。

> [!NOTE]
> 此方法针对便利性而非性能优化。更标准可靠的安装方式，请使用[安装和运行 Toolbox 服务器](#安装和运行-toolbox-服务器)中所述的二进制文件或容器镜像。

### 安装 Toolbox

最新版本请查看[发布页面][releases]，并按照操作系统和 CPU 架构使用以下说明。

[releases]: https://github.com/googleapis/mcp-toolbox/releases

<details open>
<summary>二进制文件</summary>

以二进制文件安装 Toolbox：

<!-- {x-release-please-start-version} -->
> <details>
> <summary>Linux (AMD64)</summary>
>
> 在 Linux (AMD64) 上以二进制文件安装 Toolbox：
>
> ```sh
> # see releases page for other versions
> export VERSION=1.15.0
> curl -L -o toolbox https://storage.googleapis.com/mcp-toolbox-for-databases/v$VERSION/linux/amd64/toolbox
> chmod +x toolbox
> ```
>
> </details>
> <details>
> <summary>macOS (Apple Silicon)</summary>
>
> 在 macOS (Apple Silicon) 上以二进制文件安装 Toolbox：
>
> ```sh
> # see releases page for other versions
> export VERSION=1.15.0
> curl -L -o toolbox https://storage.googleapis.com/mcp-toolbox-for-databases/v$VERSION/darwin/arm64/toolbox
> chmod +x toolbox
> ```
>
> </details>
> <details>
> <summary>macOS (Intel)</summary>
>
> 在 macOS (Intel) 上以二进制文件安装 Toolbox：
>
> ```sh
> # see releases page for other versions
> export VERSION=1.15.0
> curl -L -o toolbox https://storage.googleapis.com/mcp-toolbox-for-databases/v$VERSION/darwin/amd64/toolbox
> chmod +x toolbox
> ```
>
> </details>
> <details>
> <summary>Windows (Command Prompt)</summary>
>
> 在 Windows (Command Prompt) 上以二进制文件安装 Toolbox：
>
> ```cmd
> :: see releases page for other versions
> set VERSION=1.15.0
> curl -o toolbox.exe "https://storage.googleapis.com/mcp-toolbox-for-databases/v%VERSION%/windows/amd64/toolbox.exe"
> ```
>
> </details>
> <details>
> <summary>Windows (PowerShell)</summary>
>
> 在 Windows (PowerShell) 上以二进制文件安装 Toolbox：
>
> ```powershell
> # see releases page for other versions
> $VERSION = "1.15.0"
> curl.exe -o toolbox.exe "https://storage.googleapis.com/mcp-toolbox-for-databases/v$VERSION/windows/amd64/toolbox.exe"
> ```
>
> </details>
> <details>
> <summary>Windows ARM64 (Command Prompt)</summary>
>
> 在 Windows ARM64 (Command Prompt) 上以二进制文件安装 Toolbox：
>
> ```cmd
> :: see releases page for other versions
> set VERSION=1.15.0
> curl -o toolbox.exe "https://storage.googleapis.com/mcp-toolbox-for-databases/v%VERSION%/windows/arm64/toolbox.exe"
> ```
>
> </details>
> <details>
> <summary>Windows ARM64 (PowerShell)</summary>
>
> 在 Windows ARM64 (PowerShell) 上以二进制文件安装 Toolbox：
>
> ```powershell
> # see releases page for other versions
> $VERSION = "1.15.0"
> curl.exe -o toolbox.exe "https://storage.googleapis.com/mcp-toolbox-for-databases/v$VERSION/windows/arm64/toolbox.exe"
> ```
>
> </details>
</details>

<details>
<summary>容器镜像</summary>
也可以将 Toolbox 安装为容器：

```sh
# see releases page for other versions
export VERSION=1.15.0
docker pull us-central1-docker.pkg.dev/database-toolbox/toolbox/toolbox:$VERSION
```

</details>

<details>
<summary>Homebrew</summary>

在 macOS 或 Linux 上通过 Homebrew 安装 Toolbox：

```sh
brew install mcp-toolbox
```

</details>

<details>
<summary>从源代码编译</summary>

要从源代码安装，请确保已[安装](https://go.dev/doc/install)最新版 Go，然后运行以下命令：

```sh
go install github.com/googleapis/mcp-toolbox@v1.15.0
```
<!-- {x-release-please-end} -->

</details>
<details>
<summary>Gemini CLI</summary>
查看 [Gemini CLI 扩展](https://geminicli.com/extensions/)，将适用于 AlloyDB、BigQuery 和 Cloud SQL 等特定数据库的预构建工具直接安装到 Gemini CLI。

```sh
# Install Gemini CLI
npm install -g @google/gemini-cli
# Install the extension
gemini extensions install https://github.com/gemini-cli-extensions/cloud-sql-postgres
# Run Gemini CLI
gemini
```

通过 Gemini CLI 使用自然语言与你的自定义工具交互。

```sh
# Install the extension
gemini extensions install https://github.com/gemini-cli-extensions/mcp-toolbox
```
</details>

### 运行 Toolbox

[配置](#快速入门自定义工具) `tools.yaml` 以定义工具，然后执行 `toolbox` 启动服务器：

<details open>
<summary>二进制文件</summary>

从二进制文件运行 Toolbox：

```sh
./toolbox --config "tools.yaml"
```

> ⓘ 注意
> Toolbox 默认启用动态重载。要禁用它，请使用 `--disable-reload` 标志。

</details>

<details>

<summary>容器镜像</summary>

拉取[容器镜像](#安装-toolbox)后运行服务器：

```sh
export VERSION=0.24.0 # Use the version you pulled
docker run -p 5000:5000 \
-v $(pwd)/tools.yaml:/app/tools.yaml \
us-central1-docker.pkg.dev/database-toolbox/toolbox/toolbox:$VERSION \
--config "/app/tools.yaml"
```

> ⓘ 注意
> `-v` 标志将本地 `tools.yaml` 挂载到容器，`-p` 将容器端口 `5000` 映射到主机端口 `5000`。

</details>

<details>

<summary>源代码</summary>

要直接从源代码运行服务器，请进入项目根目录并运行：

```sh
go run .
```

> ⓘ 注意
> 此命令从源代码运行项目，更适合开发和测试。它**不会**将二进制文件编译到 `$GOPATH`。如果想编译二进制文件，请参阅[开发者文档](https://github.com/googleapis/mcp-toolbox/blob/ea2d71e7c2636f00158d2e89b46bdaf36182ceb6/DEVELOPER.md#building-the-binary)。

</details>

<details>

<summary>Homebrew</summary>

如果通过 [Homebrew](https://brew.sh/) 安装 Toolbox，`toolbox` 二进制文件位于系统路径中，可以使用相同命令启动服务器：

```sh
toolbox --config "tools.yaml"
```

</details>

<details>
<summary>NPM</summary>

无需手动下载二进制文件即可直接运行 Toolbox（需要 Node.js）：

```sh
npx @toolbox-sdk/server --config tools.yaml
```

</details>
<details>
<summary>Gemini CLI</summary>
安装 [Gemini CLI 扩展](https://geminicli.com/extensions/)后，预构建工具在使用时即可用。

```sh
# Run Gemini CLI
gemini

# List extensions
/extensions list
# List MCP servers
/mcp list
```

</details>

使用 `toolbox help` 可以查看完整标志列表！要停止服务器，请发送终止信号（大多数平台为 `ctrl+c`）。

有关部署到不同环境的详细文档，请查看[部署 Toolbox 部分](https://mcp-toolbox.dev/documentation/deploy-to/)的资源。

---

## 连接 Toolbox

Toolbox 服务器启动并运行后，可以将工具加载到兼容 MCP 的客户端或应用中。

### MCP 客户端

将以下配置添加到 MCP 客户端配置中：

```json
{
  "mcpServers": {
    "toolbox": {
      "type": "http",
      "url": "http://127.0.0.1:5000/mcp",
    }
  }
}
```

如果希望连接到特定工具集，将 url 替换为 "http://127.0.0.1:5000/mcp/{toolset_name}"。

### Toolbox SDK：集成到应用

Toolbox 客户端 SDK 提供易用的构建组件和高级功能，将自定义应用连接到 MCP Toolbox 服务器。下面列出用于各种框架的客户端 SDK：

<details open>
  <summary>Python (<a href="https://github.com/googleapis/mcp-toolbox-sdk-python">Github</a>)</summary>
  <br>
  <blockquote>

  <details open>
    <summary>Core</summary>

1. 安装 [Toolbox Core SDK][toolbox-core]：

    ```bash
    pip install toolbox-core
    ```

1. 加载工具：

    ```python
    from toolbox_core import ToolboxClient

    # update the url to point to your server
    async with ToolboxClient("http://127.0.0.1:5000") as client:

        # these tools can be passed to your application!
        tools = await client.load_toolset("toolset_name")
    ```

有关使用 Toolbox Core SDK 的详细说明，请参阅[项目 README][toolbox-core-readme]。

[toolbox-core]: https://pypi.org/project/toolbox-core/
[toolbox-core-readme]: https://github.com/googleapis/mcp-toolbox-sdk-python/tree/main/packages/toolbox-core/README.md

  </details>
  <details>
    <summary>LangChain / LangGraph</summary>

1. 安装 [Toolbox LangChain SDK][toolbox-langchain]：

    ```bash
    pip install toolbox-langchain
    ```

1. 加载工具：

    ```python
    from toolbox_langchain import ToolboxClient

    # update the url to point to your server
    async with ToolboxClient("http://127.0.0.1:5000") as client:

        # these tools can be passed to your application!
        tools = client.load_toolset()
    ```

    有关使用 Toolbox LangChain SDK 的详细说明，请参阅[项目 README][toolbox-langchain-readme]。

    [toolbox-langchain]: https://pypi.org/project/toolbox-langchain/
    [toolbox-langchain-readme]: https://github.com/googleapis/mcp-toolbox-sdk-python/blob/main/packages/toolbox-langchain/README.md

  </details>
  <details>
    <summary>LlamaIndex</summary>

1. 安装 [Toolbox Llamaindex SDK][toolbox-llamaindex]：

    ```bash
    pip install toolbox-llamaindex
    ```

1. 加载工具：

    ```python
    from toolbox_llamaindex import ToolboxClient

    # update the url to point to your server
    async with ToolboxClient("http://127.0.0.1:5000") as client:

        # these tools can be passed to your application!
        tools = client.load_toolset()
    ```

    有关使用 Toolbox Llamaindex SDK 的详细说明，请参阅[项目 README][toolbox-llamaindex-readme]。

    [toolbox-llamaindex]: https://pypi.org/project/toolbox-llamaindex/
    [toolbox-llamaindex-readme]: https://github.com/googleapis/genai-toolbox-llamaindex-python/blob/main/README.md

  </details>
</details>
</blockquote>
<details>
  <summary>Javascript/Typescript (<a href="https://github.com/googleapis/mcp-toolbox-sdk-js">Github</a>)</summary>
  <br>
  <blockquote>

  <details open>
    <summary>Core</summary>

1. 安装 [Toolbox Core SDK][toolbox-core-js]：

    ```bash
    npm install @toolbox-sdk/core
    ```

1. 加载工具：

    ```javascript
    import { ToolboxClient } from '@toolbox-sdk/core';

    // update the url to point to your server
    const URL = 'http://127.0.0.1:5000';
    let client = new ToolboxClient(URL);

    // these tools can be passed to your application!
    const tools = await client.loadToolset('toolsetName');
    ```

    有关使用 Toolbox Core SDK 的详细说明，请参阅[项目 README][toolbox-core-js-readme]。

    [toolbox-core-js]: https://www.npmjs.com/package/@toolbox-sdk/core
    [toolbox-core-js-readme]: https://github.com/googleapis/mcp-toolbox-sdk-js/blob/main/packages/toolbox-core/README.md

  </details>
  <details>
    <summary>LangChain / LangGraph</summary>

1. 安装 [Toolbox Core SDK][toolbox-core-js]：

    ```bash
    npm install @toolbox-sdk/core
    ```

2. 加载工具：

    ```javascript
    import { ToolboxClient } from '@toolbox-sdk/core';

    // update the url to point to your server
    const URL = 'http://127.0.0.1:5000';
    let client = new ToolboxClient(URL);

    // these tools can be passed to your application!
    const toolboxTools = await client.loadToolset('toolsetName');

    // Define the basics of the tool: name, description, schema and core logic
    const getTool = (toolboxTool) => tool(currTool, {
        name: toolboxTool.getName(),
        description: toolboxTool.getDescription(),
        schema: toolboxTool.getParamSchema()
    });

    // Use these tools in your Langchain/Langraph applications
    const tools = toolboxTools.map(getTool);
    ```

  </details>
  <details>
    <summary>Genkit</summary>

1. 安装 [Toolbox Core SDK][toolbox-core-js]：

    ```bash
    npm install @toolbox-sdk/core
    ```

2. 加载工具：

    ```javascript
    import { ToolboxClient } from '@toolbox-sdk/core';
    import { genkit } from 'genkit';

    // Initialise genkit
    const ai = genkit({
        plugins: [
            googleAI({
                apiKey: process.env.GEMINI_API_KEY || process.env.GOOGLE_API_KEY
            })
        ],
        model: googleAI.model('gemini-2.0-flash'),
    });

    // update the url to point to your server
    const URL = 'http://127.0.0.1:5000';
    let client = new ToolboxClient(URL);

    // these tools can be passed to your application!
    const toolboxTools = await client.loadToolset('toolsetName');

    // Define the basics of the tool: name, description, schema and core logic
    const getTool = (toolboxTool) => ai.defineTool({
        name: toolboxTool.getName(),
        description: toolboxTool.getDescription(),
        schema: toolboxTool.getParamSchema()
    }, toolboxTool)

    // Use these tools in your Genkit applications
    const tools = toolboxTools.map(getTool);
    ```

  </details>
  <details>
    <summary>ADK</summary>

1. 安装 [Toolbox ADK SDK][toolbox-adk-js]：

    ```bash
    npm install @toolbox-sdk/adk
    ```

2. 加载工具：

    ```javascript
    import { ToolboxClient } from '@toolbox-sdk/adk';

    // update the url to point to your server
    const URL = 'http://127.0.0.1:5000';
    let client = new ToolboxClient(URL);

    // these tools can be passed to your application!
    const tools = await client.loadToolset('toolsetName');
    ```

    有关使用 Toolbox ADK SDK 的详细说明，请参阅[项目 README][toolbox-adk-js-readme]。

    [toolbox-adk-js]: https://www.npmjs.com/package/@toolbox-sdk/adk
    [toolbox-adk-js-readme]:
       https://github.com/googleapis/mcp-toolbox-sdk-js/blob/main/packages/toolbox-adk/README.md

  </details>
</details>
</blockquote>
<details>
  <summary>Go (<a href="https://github.com/googleapis/mcp-toolbox-sdk-go">Github</a>)</summary>
  <br>
  <blockquote>

  <details>
    <summary>Core</summary>

1. 安装 [Toolbox Go SDK][toolbox-go]：

    ```bash
    go get github.com/googleapis/mcp-toolbox-sdk-go
    ```

2. 加载工具：

    ```go
    package main

    import (
      "github.com/googleapis/mcp-toolbox-sdk-go/core"
      "context"
    )

    func main() {
      // Make sure to add the error checks
      // update the url to point to your server
      URL := "http://127.0.0.1:5000";
      ctx := context.Background()

      client, err := core.NewToolboxClient(URL)

      // Framework agnostic tools
      tools, err := client.LoadToolset("toolsetName", ctx)
    }
    ```

    有关使用 Toolbox Go SDK 的详细说明，请参阅[项目 README][toolbox-core-go-readme]。

    [toolbox-go]: https://pkg.go.dev/github.com/googleapis/mcp-toolbox-sdk-go/core
    [toolbox-core-go-readme]: https://github.com/googleapis/mcp-toolbox-sdk-go/blob/main/core/README.md

  </details>
  <details>
    <summary>LangChain Go</summary>

1. 安装 [Toolbox Go SDK][toolbox-go]：

    ```bash
    go get github.com/googleapis/mcp-toolbox-sdk-go
    ```

2. 加载工具：

    ```go
    package main

    import (
      "context"
      "encoding/json"

      "github.com/googleapis/mcp-toolbox-sdk-go/core"
      "github.com/tmc/langchaingo/llms"
    )

    func main() {
      // Make sure to add the error checks
      // update the url to point to your server
      URL := "http://127.0.0.1:5000"
      ctx := context.Background()

      client, err := core.NewToolboxClient(URL)

      // Framework agnostic tool
      tool, err := client.LoadTool("toolName", ctx)

      // Fetch the tool's input schema
      inputschema, err := tool.InputSchema()

      var paramsSchema map[string]any
      _ = json.Unmarshal(inputschema, &paramsSchema)

      // Use this tool with LangChainGo
      langChainTool := llms.Tool{
        Type: "function",
        Function: &llms.FunctionDefinition{
          Name:        tool.Name(),
          Description: tool.Description(),
          Parameters:  paramsSchema,
        },
      }
    }

    ```

  </details>
  <details>
    <summary>Genkit</summary>

1. 安装 [Toolbox Go SDK][toolbox-go]：

    ```bash
    go get github.com/googleapis/mcp-toolbox-sdk-go
    ```

2. 加载工具：

    ```go
    package main
    import (
      "context"
      "log"

      "github.com/firebase/genkit/go/genkit"
      "github.com/googleapis/mcp-toolbox-sdk-go/core"
      "github.com/googleapis/mcp-toolbox-sdk-go/tbgenkit"
    )

    func main() {
      // Make sure to add the error checks
      // Update the url to point to your server
      URL := "http://127.0.0.1:5000"
      ctx := context.Background()
      g := genkit.Init(ctx)

      client, err := core.NewToolboxClient(URL)

      // Framework agnostic tool
      tool, err := client.LoadTool("toolName", ctx)

      // Convert the tool using the tbgenkit package
      // Use this tool with Genkit Go
      genkitTool, err := tbgenkit.ToGenkitTool(tool, g)
      if err != nil {
        log.Fatalf("Failed to convert tool: %v\n", err)
      }
      log.Printf("Successfully converted tool: %s", genkitTool.Name())
    }
    ```

  </details>
  <details>
    <summary>Go GenAI</summary>

1. 安装 [Toolbox Go SDK][toolbox-go]：

    ```bash
    go get github.com/googleapis/mcp-toolbox-sdk-go
    ```

2. 加载工具：

    ```go
    package main

    import (
      "context"
      "encoding/json"

      "github.com/googleapis/mcp-toolbox-sdk-go/core"
      "google.golang.org/genai"
    )

    func main() {
      // Make sure to add the error checks
      // Update the url to point to your server
      URL := "http://127.0.0.1:5000"
      ctx := context.Background()

      client, err := core.NewToolboxClient(URL)

      // Framework agnostic tool
      tool, err := client.LoadTool("toolName", ctx)

      // Fetch the tool's input schema
      inputschema, err := tool.InputSchema()

      var schema *genai.Schema
      _ = json.Unmarshal(inputschema, &schema)

      funcDeclaration := &genai.FunctionDeclaration{
        Name:        tool.Name(),
        Description: tool.Description(),
        Parameters:  schema,
      }

      // Use this tool with Go GenAI
      genAITool := &genai.Tool{
        FunctionDeclarations: []*genai.FunctionDeclaration{funcDeclaration},
      }
    }
    ```

  </details>
  <details>
    <summary>OpenAI Go</summary>

1. 安装 [Toolbox Go SDK][toolbox-go]：

    ```bash
    go get github.com/googleapis/mcp-toolbox-sdk-go
    ```

2. 加载工具：

    ```go
    package main

    import (
      "context"
      "encoding/json"

      "github.com/googleapis/mcp-toolbox-sdk-go/core"
      openai "github.com/openai/openai-go"
    )

    func main() {
      // Make sure to add the error checks
      // Update the url to point to your server
      URL := "http://127.0.0.1:5000"
      ctx := context.Background()

      client, err := core.NewToolboxClient(URL)

      // Framework agnostic tool
      tool, err := client.LoadTool("toolName", ctx)

      // Fetch the tool's input schema
      inputschema, err := tool.InputSchema()

      var paramsSchema openai.FunctionParameters
      _ = json.Unmarshal(inputschema, &paramsSchema)

      // Use this tool with OpenAI Go
      openAITool := openai.ChatCompletionToolParam{
        Function: openai.FunctionDefinitionParam{
          Name:        tool.Name(),
          Description: openai.String(tool.Description()),
          Parameters:  paramsSchema,
        },
      }

    }
    ```

  </details>
  <details open>
    <summary>ADK Go</summary>

1. 安装 [Toolbox Go SDK][toolbox-go]：

    ```bash
    go get github.com/googleapis/mcp-toolbox-sdk-go
    ```

1. 加载工具：

    ```go
    package main

    import (
      "github.com/googleapis/mcp-toolbox-sdk-go/tbadk"
      "context"
    )

    func main() {
      // Make sure to add the error checks
      // Update the url to point to your server
      URL := "http://127.0.0.1:5000"
      ctx := context.Background()
      client, err := tbadk.NewToolboxClient(URL)
      if err != nil {
        return fmt.Sprintln("Could not start Toolbox Client", err)
      }

      // Use this tool with ADK Go
      tool, err := client.LoadTool("toolName", ctx)
      if err != nil {
        return fmt.Sprintln("Could not load Toolbox Tool", err)
      }
    }
    ```

    有关使用 Toolbox Go SDK 的详细说明，请参阅[项目 README][toolbox-core-go-readme]。


  </details>
</details>
</blockquote>
</details>

---

## 其他功能

### 使用 Toolbox UI 测试工具

使用 `--ui` 标志启动 Toolbox 的交互界面。它可以使用授权参数等功能测试工具和工具集。更多信息请访问 [Toolbox UI](https://mcp-toolbox.dev/documentation/configuration/toolbox-ui/)。

```sh
./toolbox --ui
```

### 遥测

Toolbox 通过 OpenTelemetry 发出追踪和指标。使用 `--telemetry-otlp=<endpoint>` 将其导出到任何兼容 OTLP 的后端，例如 Google Cloud Monitoring、Agnost AI 等。详情请参阅[遥测文档](https://mcp-toolbox.dev/documentation/monitoring/export_telemetry/)。

### 生成 Agent Skills

`skills-generate` 命令可将**工具集**转换为兼容 [Agent Skill 规范](https://agentskills.io/specification)的 **Agent Skill**，便于将工具作为可移植技能包分发。

```bash
toolbox --config tools.yaml skills-generate \
  --name "my-skill" \
  --toolset "my_toolset" \
  --description "A skill containing multiple tools"
```

生成后，可将技能安装到 Gemini CLI：

```bash
gemini skills install ./skills/my-skill
```

更多信息请参阅[生成技能指南](https://mcp-toolbox.dev/documentation/configuration/skills/generating/)。要通过 MCP 提供已有技能，请参阅[通过 MCP 提供技能](https://mcp-toolbox.dev/documentation/configuration/skills/serving/)。

仓库也附带可直接使用的 Agent Skills（例如维护者工作流）。目录和安装说明见 [`skills/`](https://github.com/googleapis/mcp-toolbox/blob/ea2d71e7c2636f00158d2e89b46bdaf36182ceb6/skills/README.md)。

---

## 版本管理

MCP Toolbox for Databases 遵循[语义化版本](https://semver.org/)。

公共 API 包括 Toolbox Server（CLI、配置清单和预构建工具集）及客户端 SDK。

- **主版本号**用于不兼容的更改，例如不兼容的 CLI 或清单更改。
- **次版本号**用于新功能，包括预构建工具集的修改或 beta 功能。
- **补丁版本号**用于向后兼容的错误修复。

更多信息（包括[容器标签政策](https://mcp-toolbox.dev/reference/versioning/#container-tagging-policy)）请参阅[完整版本政策](https://mcp-toolbox.dev/reference/versioning/)。

---

## 参与贡献

欢迎贡献。请参阅 [CONTRIBUTING](https://github.com/googleapis/mcp-toolbox/blob/ea2d71e7c2636f00158d2e89b46bdaf36182ceb6/CONTRIBUTING.md) 指南开始。

有关设置 Toolbox 自身开发环境的技术细节，请参阅 [DEVELOPER](https://github.com/googleapis/mcp-toolbox/blob/ea2d71e7c2636f00158d2e89b46bdaf36182ceb6/DEVELOPER.md) 指南。

请注意，本项目发布时附有贡献者行为准则。参与本项目即表示同意遵守其条款。更多信息请参阅[贡献者行为准则](https://github.com/googleapis/mcp-toolbox/blob/ea2d71e7c2636f00158d2e89b46bdaf36182ceb6/CODE_OF_CONDUCT.md)。

---

## 社区

加入我们的 [Discord 社区](https://discord.gg/GQrFB3Ec3W)，与开发者交流！
