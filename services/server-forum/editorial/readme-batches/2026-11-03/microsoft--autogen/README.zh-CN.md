> 本文由 Microsoft 和 AutoGen 项目贡献者的 [README 原文](https://github.com/microsoft/autogen/blob/d07c667a3239ea4ab7af62a53b0f4ff205a1f23d/README.md)翻译，依据 [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/) 使用。本页为非官方中文译文，已将相对链接改为对应版本的绝对链接。

> 译注：上游 README 前文称项目已归档且停止修复，后文又称接受维护性修复；采集时 GitHub API 的 archived 标志为 false。原文两处均予保留，请结合来源核实。原文 LinkedIn 入口本次返回 HTTP 403，可通过 [GitHub Discussions](https://github.com/microsoft/autogen/discussions)联系社区。

<a name="readme-top"></a>

<div align="center">
<img src="https://microsoft.github.io/autogen/0.2/img/ag.svg" alt="AutoGen Logo" width="100">

[![Twitter](https://img.shields.io/twitter/url/https/twitter.com/cloudposse.svg?style=social&label=Follow%20%40pyautogen)](https://twitter.com/pyautogen)
[![LinkedIn](https://img.shields.io/badge/LinkedIn-Company?style=flat&logo=linkedin&logoColor=white)](https://www.linkedin.com/company/105812540)
[![Discord](https://img.shields.io/badge/discord-chat-green?logo=discord)](https://aka.ms/autogen-discord)
[![Documentation](https://img.shields.io/badge/Documentation-AutoGen-blue?logo=read-the-docs)](https://microsoft.github.io/autogen/)
[![Blog](https://img.shields.io/badge/Blog-AutoGen-blue?logo=blogger)](https://devblogs.microsoft.com/autogen/)

</div>

# AutoGen [![Archived](https://img.shields.io/badge/status-archived-red)](https://github.com/microsoft/agent-framework)

**AutoGen** 是一个用于创建多智能体 AI 应用的框架，这些应用可以自主行动或与人协作。

> [!CAUTION]
> **⚠️ 项目已归档且不再维护**
>
> 本项目已归档，Microsoft 不再积极维护。
>
> Microsoft 不再为本项目提供错误修复、安全修复、新功能或支持。仓库仍可用于历史资料和参考。
>
> **谨慎使用已归档的仓库。** 缺少持续维护或安全修复时，本项目及其依赖中的漏洞可能得不到处理，从而增加持续使用的安全风险。使用 AutoGen 前请仔细评估这些风险，尤其是在处理敏感数据时。AutoGen 是为研究和实验开发的；随着项目转入归档状态，我们仍然建议不要在生产环境中使用。
>
> 感谢 AutoGen 社区提供的代码、文档、错误报告、想法和讨论，它们共同塑造了这个项目。
>
> 新用户应从 [Microsoft Agent Framework](https://github.com/microsoft/agent-framework) 开始。建议现有用户参考 [AutoGen → Microsoft Agent Framework 迁移指南](https://learn.microsoft.com/en-us/agent-framework/migration-guide/from-autogen/)进行迁移。
>
> Microsoft Agent Framework（MAF）是面向企业的 AutoGen 后继项目。Microsoft Agent Framework 现已推出可用于生产的版本，提供稳定 API 和长期支持承诺。无论构建单个助手还是编排一组专业智能体，Microsoft Agent Framework 1.0 都提供企业级多智能体编排、多提供商模型支持，以及通过 A2A 和 MCP 实现的跨运行时互操作性。

## 安装

AutoGen 需要 **Python 3.10 或更高版本**。

```bash
# Install AgentChat and OpenAI client from Extensions
pip install -U "autogen-agentchat" "autogen-ext[openai]"
```

当前稳定版本可在 [releases](https://github.com/microsoft/autogen/releases) 中找到。 如果从 AutoGen v0.2 升级，请参阅[迁移指南](https://microsoft.github.io/autogen/stable/user-guide/agentchat-user-guide/migration-guide.html)，了解更新代码和配置的详细说明。

```bash
# Install AutoGen Studio for no-code GUI
pip install -U "autogenstudio"
```

## 快速开始

以下示例调用 OpenAI API，因此需要先创建账户并通过 `export OPENAI_API_KEY="sk-..."` 导出密钥。

### Hello World

创建一个使用 OpenAI GPT-4o 模型的助手智能体。请参阅[其他受支持模型](https://microsoft.github.io/autogen/stable/user-guide/agentchat-user-guide/tutorial/models.html).

```python
import asyncio
from autogen_agentchat.agents import AssistantAgent
from autogen_ext.models.openai import OpenAIChatCompletionClient

async def main() -> None:
    model_client = OpenAIChatCompletionClient(model="gpt-4.1")
    agent = AssistantAgent("assistant", model_client=model_client)
    print(await agent.run(task="Say 'Hello World!'"))
    await model_client.close()

asyncio.run(main())
```

### MCP Server

创建一个使用 Playwright MCP 服务器的网页浏览助手智能体。

```python
# First run `npm install -g @playwright/mcp@latest` to install the MCP server.
import asyncio
from autogen_agentchat.agents import AssistantAgent
from autogen_agentchat.ui import Console
from autogen_ext.models.openai import OpenAIChatCompletionClient
from autogen_ext.tools.mcp import McpWorkbench, StdioServerParams


async def main() -> None:
    model_client = OpenAIChatCompletionClient(model="gpt-4.1")
    server_params = StdioServerParams(
        command="npx",
        args=[
            "@playwright/mcp@latest",
            "--headless",
        ],
    )
    async with McpWorkbench(server_params) as mcp:
        agent = AssistantAgent(
            "web_browsing_assistant",
            model_client=model_client,
            workbench=mcp, # For multiple MCP servers, put them in a list.
            model_client_stream=True,
            max_tool_iterations=10,
        )
        await Console(agent.run_stream(task="Find out how many contributors for the microsoft/autogen repository"))


asyncio.run(main())
```

> **警告**：仅连接受信任的 MCP 服务器，因为它们可能在本地环境执行命令或暴露敏感信息。

### 多智能体编排

可以使用 `AgentTool` 创建基本的多智能体编排配置。

```python
import asyncio

from autogen_agentchat.agents import AssistantAgent
from autogen_agentchat.tools import AgentTool
from autogen_agentchat.ui import Console
from autogen_ext.models.openai import OpenAIChatCompletionClient


async def main() -> None:
    model_client = OpenAIChatCompletionClient(model="gpt-4.1")

    math_agent = AssistantAgent(
        "math_expert",
        model_client=model_client,
        system_message="You are a math expert.",
        description="A math expert assistant.",
        model_client_stream=True,
    )
    math_agent_tool = AgentTool(math_agent, return_value_as_last_message=True)

    chemistry_agent = AssistantAgent(
        "chemistry_expert",
        model_client=model_client,
        system_message="You are a chemistry expert.",
        description="A chemistry expert assistant.",
        model_client_stream=True,
    )
    chemistry_agent_tool = AgentTool(chemistry_agent, return_value_as_last_message=True)

    agent = AssistantAgent(
        "assistant",
        system_message="You are a general assistant. Use expert tools when needed.",
        model_client=model_client,
        model_client_stream=True,
        tools=[math_agent_tool, chemistry_agent_tool],
        max_tool_iterations=10,
    )
    await Console(agent.run_stream(task="What is the integral of x^2?"))
    await Console(agent.run_stream(task="What is the molecular weight of water?"))


asyncio.run(main())
```

有关更高级的多智能体编排和工作流，请阅读
[AgentChat 文档](https://microsoft.github.io/autogen/stable/user-guide/agentchat-user-guide/index.html).

### AutoGen Studio

使用 AutoGen Studio 无需编写代码即可构建和运行多智能体工作流原型。

> **注意**：AutoGen Studio 旨在帮助你快速制作多智能体工作流原型，并展示使用 AutoGen 构建的终端用户界面示例。它**不是可直接用于生产的应用**。建议开发者使用 AutoGen 框架构建自己的应用，并实现已部署应用所需的身份认证、安全等功能。详情请参阅[安全说明](https://microsoft.github.io/autogen/dev/user-guide/autogenstudio-user-guide/index.html#a-note-on-security)。

```bash
# Run AutoGen Studio on http://localhost:8080
autogenstudio ui --port 8080 --appdir ./my-app
```

## 为什么选择 AutoGen？

<div align="center">
  <img src="https://raw.githubusercontent.com/microsoft/autogen/d07c667a3239ea4ab7af62a53b0f4ff205a1f23d/autogen-landing.jpg" alt="AutoGen Landing" width="500">
</div>

AutoGen 起源于 Microsoft Research，为实验性多智能体编排模式打开了大门，并启发了社区。AutoGen 现已归档，Microsoft 不再维护；以下架构说明仅供历史资料和参考。**新项目建议使用 [Microsoft Agent Framework](https://github.com/microsoft/agent-framework)**,，它在 AutoGen 经验的基础上提供企业级支持。

AutoGen _框架_采用分层、可扩展设计。各层职责明确，并建立在下层之上。这种设计使你可以在从高级 API 到底层组件的不同抽象层级使用框架。

- [Core API](https://github.com/microsoft/autogen/tree/d07c667a3239ea4ab7af62a53b0f4ff205a1f23d/python/packages/autogen-core/) 实现消息传递、事件驱动智能体以及本地与分布式运行时，提供灵活性和能力；还支持 .NET 与 Python 跨语言互操作。
- [AgentChat API](https://github.com/microsoft/autogen/tree/d07c667a3239ea4ab7af62a53b0f4ff205a1f23d/python/packages/autogen-agentchat/) 实现较简单但带有设计约定的 API，便于快速制作原型。该 API 基于 Core API，最接近 v0.2 用户熟悉的方式，并支持双智能体聊天或群聊等常见多智能体模式。
- [Extensions API](https://github.com/microsoft/autogen/tree/d07c667a3239ea4ab7af62a53b0f4ff205a1f23d/python/packages/autogen-ext/) 支持第一方和第三方扩展，持续扩充框架能力，包括 LLM 客户端（如 OpenAI、AzureOpenAI）的具体实现及代码执行等能力。

生态还提供两个重要的_开发工具_：

<div align="center">
  <img src="https://media.githubusercontent.com/media/microsoft/autogen/d07c667a3239ea4ab7af62a53b0f4ff205a1f23d/python/packages/autogen-studio/docs/ags_screen.png" alt="AutoGen Studio Screenshot" width="500">
</div>

- [AutoGen Studio](https://github.com/microsoft/autogen/tree/d07c667a3239ea4ab7af62a53b0f4ff205a1f23d/python/packages/autogen-studio/) 提供用于构建多智能体应用的无代码图形界面。
- [AutoGen Bench](https://github.com/microsoft/autogen/tree/d07c667a3239ea4ab7af62a53b0f4ff205a1f23d/python/packages/agbench/) 提供评估智能体性能的基准测试套件。

你可以使用 AutoGen 框架和开发工具为自己的领域创建应用。例如，[Magentic-One](https://github.com/microsoft/autogen/tree/d07c667a3239ea4ab7af62a53b0f4ff205a1f23d/python/packages/magentic-one-cli/) 是使用 AgentChat API 和 Extensions API 构建的先进多智能体团队，可以处理需要网页浏览、代码执行和文件处理的多种任务。

如需社区支持，请访问我们的 [Discord 服务器](https://aka.ms/autogen-discord) 或 [GitHub Discussions](https://github.com/microsoft/autogen/discussions)。请注意，AutoGen 现在由社区管理，回复可能有限。

## 下一步

> **开始新项目？** 前往 [Microsoft Agent Framework](https://github.com/microsoft/agent-framework)，获取具有长期支持的最新多智能体能力。
>
> **现有 AutoGen 用户？** 使用[迁移指南](https://learn.microsoft.com/en-us/agent-framework/migration-guide/from-autogen/)进行迁移，或参考下方资源了解当前 AutoGen 文档。

<div align="center">

|               | [![Python](https://img.shields.io/badge/AutoGen-Python-blue?logo=python&logoColor=white)](https://github.com/microsoft/autogen/tree/d07c667a3239ea4ab7af62a53b0f4ff205a1f23d/python)                                                                                                                                                                                                                                                                                                                | [![.NET](https://img.shields.io/badge/AutoGen-.NET-green?logo=.net&logoColor=white)](https://github.com/microsoft/autogen/tree/d07c667a3239ea4ab7af62a53b0f4ff205a1f23d/dotnet)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    | [![Studio](https://img.shields.io/badge/AutoGen-Studio-purple?logo=visual-studio&logoColor=white)](https://github.com/microsoft/autogen/tree/d07c667a3239ea4ab7af62a53b0f4ff205a1f23d/python/packages/autogen-studio)                        |
| ------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 安装          | [![Installation](https://img.shields.io/badge/Install-blue)](https://microsoft.github.io/autogen/stable/user-guide/agentchat-user-guide/installation.html)                                                                                                                                                                                                                                                         | [![Install](https://img.shields.io/badge/Install-green)](https://microsoft.github.io/autogen/dotnet/dev/core/installation.html)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   | [![Install](https://img.shields.io/badge/Install-purple)](https://microsoft.github.io/autogen/stable/user-guide/autogenstudio-user-guide/installation.html) |
| 快速开始      | [![Quickstart](https://img.shields.io/badge/Quickstart-blue)](https://microsoft.github.io/autogen/stable/user-guide/agentchat-user-guide/quickstart.html#)                                                                                                                                                                                                                                                         | [![Quickstart](https://img.shields.io/badge/Quickstart-green)](https://microsoft.github.io/autogen/dotnet/dev/core/index.html)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    | [![Usage](https://img.shields.io/badge/Quickstart-purple)](https://microsoft.github.io/autogen/stable/user-guide/autogenstudio-user-guide/usage.html#)      |
| 教程          | [![Tutorial](https://img.shields.io/badge/Tutorial-blue)](https://microsoft.github.io/autogen/stable/user-guide/agentchat-user-guide/tutorial/index.html)                                                                                                                                                                                                                                                          | [![Tutorial](https://img.shields.io/badge/Tutorial-green)](https://microsoft.github.io/autogen/dotnet/dev/core/tutorial.html)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     | [![Usage](https://img.shields.io/badge/Tutorial-purple)](https://microsoft.github.io/autogen/stable/user-guide/autogenstudio-user-guide/usage.html#)        |
| API 参考      | [![API](https://img.shields.io/badge/Docs-blue)](https://microsoft.github.io/autogen/stable/reference/index.html#)                                                                                                                                                                                                                                                                                                 | [![API](https://img.shields.io/badge/Docs-green)](https://microsoft.github.io/autogen/dotnet/dev/api/Microsoft.AutoGen.Contracts.html)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            | [![API](https://img.shields.io/badge/Docs-purple)](https://microsoft.github.io/autogen/stable/user-guide/autogenstudio-user-guide/usage.html)               |
| 软件包        | [![PyPi autogen-core](https://img.shields.io/badge/PyPi-autogen--core-blue?logo=pypi)](https://pypi.org/project/autogen-core/) <br> [![PyPi autogen-agentchat](https://img.shields.io/badge/PyPi-autogen--agentchat-blue?logo=pypi)](https://pypi.org/project/autogen-agentchat/) <br> [![PyPi autogen-ext](https://img.shields.io/badge/PyPi-autogen--ext-blue?logo=pypi)](https://pypi.org/project/autogen-ext/) | [![NuGet Contracts](https://img.shields.io/badge/NuGet-Contracts-green?logo=nuget)](https://www.nuget.org/packages/Microsoft.AutoGen.Contracts/) <br> [![NuGet Core](https://img.shields.io/badge/NuGet-Core-green?logo=nuget)](https://www.nuget.org/packages/Microsoft.AutoGen.Core/) <br> [![NuGet Core.Grpc](https://img.shields.io/badge/NuGet-Core.Grpc-green?logo=nuget)](https://www.nuget.org/packages/Microsoft.AutoGen.Core.Grpc/) <br> [![NuGet RuntimeGateway.Grpc](https://img.shields.io/badge/NuGet-RuntimeGateway.Grpc-green?logo=nuget)](https://www.nuget.org/packages/Microsoft.AutoGen.RuntimeGateway.Grpc/) | [![PyPi autogenstudio](https://img.shields.io/badge/PyPi-autogenstudio-purple?logo=pypi)](https://pypi.org/project/autogenstudio/)                          |

</div>

有兴趣贡献？请参阅 [CONTRIBUTING.md](https://github.com/microsoft/autogen/blob/d07c667a3239ea4ab7af62a53b0f4ff205a1f23d/CONTRIBUTING.md)了解指南。由于 AutoGen 处于维护模式，贡献仅限错误修复、安全补丁和文档改进。对于功能开发，可考虑为 [Microsoft Agent Framework](https://github.com/microsoft/agent-framework) 贡献。

有问题？请查看[常见问题（FAQ）](https://github.com/microsoft/autogen/blob/d07c667a3239ea4ab7af62a53b0f4ff205a1f23d/FAQ.md)获取常见问题的答案。社区支持可通过 [GitHub Discussions](https://github.com/microsoft/autogen/discussions) 和 [Discord 服务器](https://aka.ms/autogen-discord) 获得；由于 AutoGen 现在由社区管理，回复时间可能有所不同。如需持续获得支持的工具，请参阅 [Microsoft Agent Framework](https://github.com/microsoft/agent-framework)。

## 法律声明

Microsoft 和所有贡献者依据 [Creative Commons Attribution 4.0 国际公共许可证](https://creativecommons.org/licenses/by/4.0/legalcode)，
授予本仓库中 Microsoft 文档和其他内容的许可，参见 [LICENSE](https://github.com/microsoft/autogen/blob/d07c667a3239ea4ab7af62a53b0f4ff205a1f23d/LICENSE)文件；并依据 [MIT 许可证](https://opensource.org/licenses/MIT)，授予本仓库中代码的许可，参见
[LICENSE-CODE](https://github.com/microsoft/autogen/blob/d07c667a3239ea4ab7af62a53b0f4ff205a1f23d/LICENSE-CODE)文件。

文档中提及的 Microsoft、Windows、Microsoft Azure 和／或其他 Microsoft 产品及服务可能是 Microsoft 在美国和／或其他国家或地区的商标或注册商标。本项目许可证不授予使用任何 Microsoft 名称、标志或商标的权利。Microsoft 的一般商标准则见 <http://go.microsoft.com/fwlink/?LinkID=254653>.

隐私信息见 <https://go.microsoft.com/fwlink/?LinkId=521839>

Microsoft 和所有贡献者保留其他一切权利，无论这些权利来自各自的版权、专利或商标，也无论是通过默示、禁止反言或其他方式产生。

<p align="right" style="font-size: 14px; color: #555; margin-top: 20px;">
  <a href="#readme-top" style="text-decoration: none; color: blue; font-weight: bold;">
    ↑ 返回顶部 ↑
  </a>
</p>
