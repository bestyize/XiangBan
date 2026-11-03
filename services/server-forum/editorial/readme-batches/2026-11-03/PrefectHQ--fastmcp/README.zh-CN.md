<div align="center">

<!-- omit in toc -->

<picture>
  <source width="550" media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/PrefectHQ/fastmcp/901575873111515457755df6872d7b0ccdf9407b/docs/assets/brand/f-watercolor-waves-4-dark.png">
  <source width="550" media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/PrefectHQ/fastmcp/901575873111515457755df6872d7b0ccdf9407b/docs/assets/brand/f-watercolor-waves-4.png">
  <img width="550" alt="FastMCP Logo" src="https://raw.githubusercontent.com/PrefectHQ/fastmcp/901575873111515457755df6872d7b0ccdf9407b/docs/assets/brand/f-watercolor-waves-2.png">
</picture>

# FastMCP 🚀

<strong>快速行动，创造事物。</strong>

*由 [Prefect](https://www.prefect.io/) 用 💙 制作*

[![Docs](https://img.shields.io/badge/docs-gofastmcp.com-blue)](https://gofastmcp.com)
[![Discord](https://img.shields.io/badge/community-discord-5865F2?logo=discord&logoColor=white)](https://discord.gg/uu8dJCgttd)
[![PyPI - Version](https://img.shields.io/pypi/v/fastmcp.svg)](https://pypi.org/project/fastmcp)
[![TypeScript](https://img.shields.io/npm/v/%40prefecthq%2Ffastmcp-ts?label=typescript&color=3178c6)](https://github.com/PrefectHQ/fastmcp-ts)
[![Tests](https://github.com/PrefectHQ/fastmcp/actions/workflows/run-tests.yml/badge.svg)](https://github.com/PrefectHQ/fastmcp/actions/workflows/run-tests.yml)
[![License](https://img.shields.io/github/license/PrefectHQ/fastmcp.svg)](https://github.com/PrefectHQ/fastmcp/blob/901575873111515457755df6872d7b0ccdf9407b/LICENSE)

<a href="https://trendshift.io/repositories/21461" target="_blank"><img src="https://trendshift.io/api/badge/repositories/21461" alt="prefecthq%2Ffastmcp | Trendshift" style="width: 250px; height: 55px;" width="250" height="55"/></a>
</div>

---

[Model Context Protocol](https://modelcontextprotocol.io/)（MCP）将大语言模型连接到工具和数据。FastMCP 是面向服务器、客户端和交互式应用的完整 MCP 应用框架。服务器从普通 Python 代码开始：

```python
from fastmcp import FastMCP

mcp = FastMCP("Demo 🚀")

@mcp.tool
def add(a: int, b: int) -> int:
    """Add two numbers"""
    return a + b

if __name__ == "__main__":
    mcp.run()
```

## 为什么选择 FastMCP

构建有效的 MCP 应用比看起来更难。FastMCP 负责处理这些工作：通过 Python 函数声明工具，自动生成模式、验证和文档；通过 URL 连接服务器，由框架管理传输协商、认证和协议生命周期。你可以专注于自己的逻辑：**FastMCP 内置了最佳实践。**

**项目称 FastMCP 是使用 MCP 的标准框架。** FastMCP 1.0 于 2024 年被纳入官方 MCP Python SDK。项目称，如今持续维护的独立项目每天被下载一百万次，各种版本的 FastMCP 支撑了跨语言 70% 的 MCP 服务器。

FastMCP 有三个支柱：

<table>
<tr>
<td align="center" valign="top" width="33%">
<a href="https://gofastmcp.com/servers/server">
<img src="https://raw.githubusercontent.com/PrefectHQ/fastmcp/901575873111515457755df6872d7b0ccdf9407b/docs/assets/images/servers-card.png" alt="Servers" />
<br /><strong>服务器</strong>
</a>
<br />向大语言模型提供工具、资源和提示词。
</td>
<td align="center" valign="top" width="33%">
<a href="https://gofastmcp.com/apps/overview">
<img src="https://raw.githubusercontent.com/PrefectHQ/fastmcp/901575873111515457755df6872d7b0ccdf9407b/docs/assets/images/apps-card.png" alt="Apps" />
<br /><strong>应用</strong>
</a>
<br />为工具提供直接在对话中渲染的交互式界面。
</td>
<td align="center" valign="top" width="33%">
<a href="https://gofastmcp.com/clients/client">
<img src="https://raw.githubusercontent.com/PrefectHQ/fastmcp/901575873111515457755df6872d7b0ccdf9407b/docs/assets/images/clients-card.png" alt="Clients" />
<br /><strong>客户端</strong>
</a>
<br />连接任何 MCP 服务器——本地或远程、通过程序或命令行。
</td>
</tr>
</table>

**[服务器](https://gofastmcp.com/servers/server)**将 Python 函数封装为符合 MCP 的工具、资源和提示词。 **[客户端](https://gofastmcp.com/clients/client)**以完整协议支持连接任何服务器。 **[应用](https://gofastmcp.com/apps/overview)**为工具提供直接在对话中渲染的交互式界面。

**使用 TypeScript 开发？** [FastMCP for TypeScript](https://github.com/PrefectHQ/fastmcp-ts)是由同一团队构建和维护的官方对应版本，采用相同的支柱和理念，安装方式为 `npm install @prefecthq/fastmcp-ts`。

准备开始构建？请从[安装指南](https://gofastmcp.com/getting-started/installation)开始，或直接查看[快速入门](https://gofastmcp.com/getting-started/quickstart).

<a id="scale-mcp-with-horizon"></a>

## 使用 FastMCP Gateway 扩展 MCP

FastMCP 负责 MCP 应用层。**[FastMCP Gateway](https://www.prefect.io/horizon?utm_source=github&utm_medium=readme&utm_campaign=readme_horizon&utm_content=readme_body)**是企业级 MCP 网关，用于跨团队扩展服务器和工具，并集中治理其部署、发现、安全和使用方式。

FastMCP 和 FastMCP Gateway 由 [Prefect](https://www.prefect.io/) 的同一团队构建。

从 GitHub 部署 FastMCP 服务器，支持分支预览和即时回滚；为公司使用的 MCP 创建私有注册表；通过 SSO 和工具级 RBAC 保护访问；为 MCP 技术栈提供审计日志、可观测性和治理；将已批准的工具重新组合为面向团队和智能体的专用端点。

从 FastMCP 开始。[使用 FastMCP Gateway 扩展 →](https://www.prefect.io/horizon?utm_source=github&utm_medium=readme&utm_campaign=readme_horizon&utm_content=readme_cta)

## 安装

我们建议使用 [uv](https://docs.astral.sh/uv/) 将 FastMCP 添加到项目：

```bash
uv add fastmcp
```

完整安装说明，包括验证和升级，请参阅[**安装指南**](https://gofastmcp.com/getting-started/installation).

**正在升级？** 我们提供以下指南：
- [从 FastMCP 3 升级](https://gofastmcp.com/getting-started/upgrading/from-fastmcp-3)
- [从 FastMCP 2 升级](https://gofastmcp.com/getting-started/upgrading/from-fastmcp-2)
- [从 MCP SDK v1 升级](https://gofastmcp.com/getting-started/upgrading/from-mcp-sdk-v1)或 [v2](https://gofastmcp.com/getting-started/upgrading/from-mcp-sdk-v2)
- [从底层 SDK v1 升级](https://gofastmcp.com/getting-started/upgrading/from-low-level-sdk-v1)或 [v2](https://gofastmcp.com/getting-started/upgrading/from-low-level-sdk-v2)

## 📚 文档

FastMCP 的完整文档位于 **[gofastmcp.com](https://gofastmcp.com)**，包括详细指南、API 参考和高级模式。

文档还提供 [llms.txt 格式](https://llmstxt.org/)，这是一种大语言模型可以轻松读取的简单 Markdown 标准：

- [`llms.txt`](https://gofastmcp.com/llms.txt)本质上是一份站点地图，列出文档中的所有页面。
- [`llms-full.txt`](https://gofastmcp.com/llms-full.txt)包含完整文档。请注意，这可能超出你的大语言模型的上下文窗口。

为编码智能体提供 [`llms.txt`](https://gofastmcp.com/llms.txt)作为起点。有关版本匹配的文档及可选的服务器编写技能，请参阅 [Coding Agents](https://gofastmcp.com/getting-started/ai-assisted-development)。

**社区：** 加入我们的 [Discord 服务器](https://discord.gg/uu8dJCgttd)，与其他 FastMCP 开发者交流并分享你正在构建的内容。

## 贡献

请先阅读 [CONTRIBUTING.md](https://github.com/PrefectHQ/fastmcp/blob/901575873111515457755df6872d7b0ccdf9407b/CONTRIBUTING.md)了解接受的贡献与 issue 分配流程，再按照[贡献指南](https://gofastmcp.com/development/contributing)进行配置和验证。智能体应阅读 [AGENTS.md](https://github.com/PrefectHQ/fastmcp/blob/901575873111515457755df6872d7b0ccdf9407b/AGENTS.md).
