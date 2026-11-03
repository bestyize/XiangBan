# Model Context Protocol 服务器

本仓库收集了 [Model Context Protocol](https://modelcontextprotocol.io/)（MCP）的*参考实现*，以及社区构建的服务器和其他资源的链接。

> [!IMPORTANT]
> 如果你正在查找 MCP 服务器列表，可以在 [MCP Registry](https://registry.modelcontextprotocol.io/) 浏览已发布的服务器。本 README 所属仓库仅用于存放由 MCP 指导小组维护的少量参考服务器。

> [!WARNING]
> 本仓库中的服务器旨在作为**参考实现**，展示 MCP 功能和 SDK 的用法。它们是供开发者构建自己的 MCP 服务器时学习的示例，并非可直接用于生产的解决方案。开发者应根据各自的威胁模型和使用场景评估安全需求，并实施适当的防护措施。

本仓库中的服务器展示了 MCP 的多功能性和可扩展性，说明如何用它为大语言模型（LLM）提供对工具和数据源安全、受控的访问。
通常，每个 MCP 服务器都使用 MCP SDK 实现：

- [C# MCP SDK](https://github.com/modelcontextprotocol/csharp-sdk)
- [Go MCP SDK](https://github.com/modelcontextprotocol/go-sdk)
- [Java MCP SDK](https://github.com/modelcontextprotocol/java-sdk)
- [Kotlin MCP SDK](https://github.com/modelcontextprotocol/kotlin-sdk)
- [PHP MCP SDK](https://github.com/modelcontextprotocol/php-sdk)
- [Python MCP SDK](https://github.com/modelcontextprotocol/python-sdk)
- [Ruby MCP SDK](https://github.com/modelcontextprotocol/ruby-sdk)
- [Rust MCP SDK](https://github.com/modelcontextprotocol/rust-sdk)
- [Swift MCP SDK](https://github.com/modelcontextprotocol/swift-sdk)
- [TypeScript MCP SDK](https://github.com/modelcontextprotocol/typescript-sdk)

## 🌟 参考服务器

这些服务器旨在展示 MCP 功能和官方 SDK。

- **[Everything](https://github.com/modelcontextprotocol/servers/blob/194cc86e3ae6400958555bd4116849d822d69263/src/everything)** - 包含提示、资源和工具的参考／测试服务器。
- **[Fetch](https://github.com/modelcontextprotocol/servers/blob/194cc86e3ae6400958555bd4116849d822d69263/src/fetch)** - 获取并转换网页内容，供大语言模型高效使用。
- **[Filesystem](https://github.com/modelcontextprotocol/servers/blob/194cc86e3ae6400958555bd4116849d822d69263/src/filesystem)** - 通过可配置的访问控制提供安全文件操作。
- **[Git](https://github.com/modelcontextprotocol/servers/blob/194cc86e3ae6400958555bd4116849d822d69263/src/git)** - 用于读取、搜索和操作 Git 仓库的工具。
- **[Memory](https://github.com/modelcontextprotocol/servers/blob/194cc86e3ae6400958555bd4116849d822d69263/src/memory)** - 基于知识图谱的持久记忆系统。
- **[Sequential Thinking](https://github.com/modelcontextprotocol/servers/blob/194cc86e3ae6400958555bd4116849d822d69263/src/sequentialthinking)** - 通过思维序列进行动态、反思式问题求解。
- **[Time](https://github.com/modelcontextprotocol/servers/blob/194cc86e3ae6400958555bd4116849d822d69263/src/time)** - 时间与时区转换功能。

### 已归档

以下参考服务器现已归档，可在 [servers-archived](https://github.com/modelcontextprotocol/servers-archived) 找到。

- **[AWS KB Retrieval](https://github.com/modelcontextprotocol/servers-archived/tree/main/src/aws-kb-retrieval-server)** - 使用 Bedrock Agent Runtime 从 AWS Knowledge Base 检索。
- **[Brave Search](https://github.com/modelcontextprotocol/servers-archived/tree/main/src/brave-search)** - 使用 Brave Search API 搜索网页和本地内容。已被[官方服务器](https://github.com/brave/brave-search-mcp-server)（[`@brave/brave-search-mcp-server`](https://www.npmjs.com/package/@brave/brave-search-mcp-server)）取代。
- **[EverArt](https://github.com/modelcontextprotocol/servers-archived/tree/main/src/everart)** - 使用多种模型生成 AI 图片。
- **[GitHub](https://github.com/modelcontextprotocol/servers-archived/tree/main/src/github)** - 仓库管理、文件操作及 GitHub API 集成。
- **[GitLab](https://github.com/modelcontextprotocol/servers-archived/tree/main/src/gitlab)** - 通过 GitLab API 实现项目管理。
- **[Google Drive](https://github.com/modelcontextprotocol/servers-archived/tree/main/src/gdrive)** - Google Drive 的文件访问和搜索功能。
- **[Google Maps](https://github.com/modelcontextprotocol/servers-archived/tree/main/src/google-maps)** - 位置服务、路线和地点详情。
- **[PostgreSQL](https://github.com/modelcontextprotocol/servers-archived/tree/main/src/postgres)** - 带模式检查的只读数据库访问。
- **[Puppeteer](https://github.com/modelcontextprotocol/servers-archived/tree/main/src/puppeteer)** - 浏览器自动化与网页抓取。
- **[Redis](https://github.com/modelcontextprotocol/servers-archived/tree/main/src/redis)** - 与 Redis 键值存储交互。
- **[Sentry](https://github.com/modelcontextprotocol/servers-archived/tree/main/src/sentry)** - 从 Sentry.io 检索和分析问题。
- **[Slack](https://github.com/modelcontextprotocol/servers-archived/tree/main/src/slack)** - 频道管理和消息功能。现由 [Zencoder](https://github.com/zencoderai/slack-mcp-server) 维护。
- **[SQLite](https://github.com/modelcontextprotocol/servers-archived/tree/main/src/sqlite)** - 数据库交互和商业智能功能。

## 🚀 开始使用

### 使用本仓库中的 MCP 服务器
本仓库中基于 TypeScript 的服务器可以使用 `npx` 启动。

例如，以下命令将启动 [Memory](https://github.com/modelcontextprotocol/servers/blob/194cc86e3ae6400958555bd4116849d822d69263/src/memory)服务器：
```sh
npx -y @modelcontextprotocol/server-memory
```

本仓库中基于 Python 的服务器可以使用 [`uvx`](https://docs.astral.sh/uv/concepts/tools/) 或 [`pip`](https://pypi.org/project/pip/) 启动。为了便于使用和设置，建议使用 `uvx`。

例如，以下命令将启动 [Git](https://github.com/modelcontextprotocol/servers/blob/194cc86e3ae6400958555bd4116849d822d69263/src/git)服务器：
```sh
# With uvx
uvx mcp-server-git

# With pip
pip install mcp-server-git
python -m mcp_server_git
```

按照[此说明](https://docs.astral.sh/uv/getting-started/installation/)安装 `uv` / `uvx`，按照[此说明](https://pip.pypa.io/en/stable/installation/)安装 `pip`。

### 使用 MCP 客户端
不过，单独运行服务器的用途有限，应将其配置到 MCP 客户端中。例如，以下是使用上述服务器的 Claude Desktop 配置：

```json
{
  "mcpServers": {
    "memory": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-memory"]
    }
  }
}
```

在 Windows 上，使用 `cmd /c` 包装 `npx`：

```json
{
  "mcpServers": {
    "memory": {
      "command": "cmd",
      "args": ["/c", "npx", "-y", "@modelcontextprotocol/server-memory"]
    }
  }
}
```

使用 Claude Desktop 作为 MCP 客户端的其他示例如下：

```json
{
  "mcpServers": {
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/path/to/allowed/files"]
    },
    "git": {
      "command": "uvx",
      "args": ["mcp-server-git", "--repository", "path/to/git/repo"]
    },
    "fetch": {
      "command": "uvx",
      "args": ["mcp-server-fetch"]
    }
  }
}
```

在 Windows 上，对上面基于 `npx` 的 `filesystem` 条目使用相同包装：将其 `"command"` 改为 `"cmd"`，并在现有 `args` 前添加 `"/c", "npx"`。基于 `uvx` 的条目保持不变。

### 传输方式：stdio 和 Streamable HTTP

MCP 服务器通过**传输方式**与客户端通信。协议定义了两种标准传输方式，它们的运行方式有很大不同：

| 传输方式 | 服务器如何运行 | 客户端如何连接 |
|---|---|---|
| **stdio** | 客户端将服务器作为子进程启动，通过该进程的 stdin 和 stdout 与其通信。服务器不打开端口。 | 在客户端配置中使用 `command` 和 `args`，如上例所示 |
| **Streamable HTTP** | 服务器独立运行，作为 HTTP 服务，可以服务多个客户端 | URL，例如 `http://localhost:3001/mcp` |

**本仓库中的每个服务器都支持 stdio，只有 [Everything](https://github.com/modelcontextprotocol/servers/blob/194cc86e3ae6400958555bd4116849d822d69263/src/everything)还支持 Streamable HTTP**（其 README 说明如何以此方式启动，也介绍已弃用的 HTTP+SSE 传输）。因此，当你在终端运行 `uvx mcp-server-git` 或 `npx -y @modelcontextprotocol/server-memory` 时，它不会打开端口，最多打印一行启动提示，然后看起来像是挂起。这是预期行为：它正在等待客户端通过 stdin 发送消息。通常你不需要自行启动 stdio 服务器；MCP 客户端会按配置启动它，并在结束后停止它。

要在不配置客户端的情况下试用服务器，请使用 [MCP Inspector](https://github.com/modelcontextprotocol/inspector)。提供 stdio 服务器命令时，它会为你启动服务器，并让你浏览和调用工具；提供 URL 时，它会连接到正在运行的 Streamable HTTP 服务器：

```sh
# A stdio server: the Inspector launches it
npx @modelcontextprotocol/inspector uvx mcp-server-git

# A Streamable HTTP server: start it first, then connect to its URL
npx @modelcontextprotocol/server-everything streamableHttp
npx @modelcontextprotocol/inspector --transport http --server-url http://localhost:3001/mcp
```

更多信息请参阅 MCP 架构概览中的[传输层](https://modelcontextprotocol.io/docs/learn/architecture#transport-layer)，以及规范中的[传输方式](https://modelcontextprotocol.io/specification/2026-07-28/basic/transports)。

## 🛠️ 创建自己的服务器

想创建自己的 MCP 服务器？请访问 [modelcontextprotocol.io](https://modelcontextprotocol.io/introduction) 官方文档，查看实现 MCP 服务器的完整指南、最佳实践和技术细节。

## 📚 了解更多

参阅 [ADDITIONAL.md](https://github.com/modelcontextprotocol/servers/blob/194cc86e3ae6400958555bd4116849d822d69263/ADDITIONAL.md)，查看有助于简化 MCP 服务器和客户端构建的框架与资源列表。

## 🤝 参与贡献

参阅 [CONTRIBUTING.md](https://github.com/modelcontextprotocol/servers/blob/194cc86e3ae6400958555bd4116849d822d69263/CONTRIBUTING.md)，了解如何为本仓库贡献。

## 📦 发布

参阅 [RELEASING.md](https://github.com/modelcontextprotocol/servers/blob/194cc86e3ae6400958555bd4116849d822d69263/RELEASING.md)，了解如何发布包（从 CI 使用 OIDC 可信发布，无需注册表令牌）以及如何重试失败的发布。

## 🔒 安全

参阅 [SECURITY.md](https://github.com/modelcontextprotocol/servers/blob/194cc86e3ae6400958555bd4116849d822d69263/SECURITY.md)，了解如何报告安全漏洞。

## 📜 许可证

本项目的新贡献使用 Apache License, Version 2.0，现有代码使用 MIT 许可——详情请参阅 [LICENSE](https://github.com/modelcontextprotocol/servers/blob/194cc86e3ae6400958555bd4116849d822d69263/LICENSE) 文件。

## 💬 社区

- [GitHub Discussions](https://github.com/orgs/modelcontextprotocol/discussions)

## ⭐ 支持

如果你觉得 MCP 服务器有用，欢迎为仓库点星，并贡献新服务器或改进！

---

由 Anthropic 管理，并与社区共同构建。Model Context Protocol 是开源的，我们鼓励所有人贡献自己的服务器和改进！
