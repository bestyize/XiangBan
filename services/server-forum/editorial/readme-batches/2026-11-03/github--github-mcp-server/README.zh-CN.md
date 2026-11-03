[![Go Report Card](https://goreportcard.com/badge/github.com/github/github-mcp-server/v2)](https://goreportcard.com/report/github.com/github/github-mcp-server/v2)

# GitHub MCP 服务器

GitHub MCP 服务器将 AI 工具直接连接到 GitHub 平台，让 AI 代理、助手和聊天机器人能够读取仓库和代码文件、管理议题与 PR、分析代码并自动化工作流，所有操作均通过自然语言交互完成。

### 使用场景

- 仓库管理：浏览和查询代码、搜索文件、分析提交，并了解你有权访问的任意仓库的项目结构。
- 议题与 PR 自动化：创建、更新和管理议题及拉取请求。让 AI 协助分类缺陷、审查代码变更并维护项目看板。
- CI/CD 与工作流洞察：监控 GitHub Actions 工作流运行、分析构建失败、管理发布并了解开发流水线。
- 代码分析：检查安全发现、审查 Dependabot 警报、理解代码模式，并全面了解代码库。
- 团队协作：访问讨论、管理通知、分析团队活动并简化团队流程。

本项目面向希望将 AI 工具连接到 GitHub 上下文和能力的开发者，支持从简单自然语言查询到复杂多步骤代理工作流的各种场景。

### 使用 Go 模块

Go 模块路径为 `github.com/github/github-mcp-server/v2`。库的使用者
必须在导入时使用此前缀，例如
`github.com/github/github-mcp-server/v2/pkg/github`，在 `go get` 命令中也要使用该前缀。
从仓库检出构建时不需要 `/v2` 子目录。

现有 `v2.0.0` 标签早于模块路径修正，不能作为 Go 模块使用。
合并此更改后，必须从包含修正后模块路径的提交创建新发布，
例如标记 `v2.0.1`。在此之前，
使用者可以使用包含此更改的提交对应的伪版本。

---

## 远程 GitHub MCP 服务器

[![Install in VS Code](https://img.shields.io/badge/VS_Code-Install_Server-0098FF?style=flat-square&logo=visualstudiocode&logoColor=white)](https://insiders.vscode.dev/redirect/mcp/install?name=github&config=%7B%22type%22%3A%20%22http%22%2C%22url%22%3A%20%22https%3A%2F%2Fapi.githubcopilot.com%2Fmcp%2F%22%7D) [![Install in VS Code Insiders](https://img.shields.io/badge/VS_Code_Insiders-Install_Server-24bfa5?style=flat-square&logo=visualstudiocode&logoColor=white)](https://insiders.vscode.dev/redirect/mcp/install?name=github&config=%7B%22type%22%3A%20%22http%22%2C%22url%22%3A%20%22https%3A%2F%2Fapi.githubcopilot.com%2Fmcp%2F%22%7D&quality=insiders) [![Install in Visual Studio](https://img.shields.io/badge/Visual_Studio-Install_Server-C16FDE?style=flat-square&logo=visualstudio&logoColor=white)](https://aka.ms/vs/mcp-install?%7B%22name%22%3A%22github%22%2C%22gallery%22%3Atrue%2C%22url%22%3A%22https%3A%2F%2Fapi.githubcopilot.com%2Fmcp%2F%22%7D)

远程 GitHub MCP 服务器由 GitHub 托管，提供最简单的入门方式。如果你的 MCP 主机不支持远程 MCP 服务器，可以改用 [GitHub MCP 服务器的本地版本](https://github.com/github/github-mcp-server?tab=readme-ov-file#local-github-mcp-server)。

### 前提条件

1. 支持远程服务器的兼容 MCP 主机（VS Code 1.101+、Claude Desktop、Cursor、Windsurf 等）
2. 启用任何适用的[策略](https://github.com/github/github-mcp-server/blob/main/docs/policies-and-governance.md)

### 在 VS Code 中安装

要快速安装，请使用上方的一键安装按钮之一。完成流程后切换代理模式（位于 Copilot Chat 文本输入框旁），服务器便会启动。请确保使用 [VS Code 1.101](https://code.visualstudio.com/updates/v1_101) 或[更新版本](https://code.visualstudio.com/updates)，以支持远程 MCP 和 OAuth。

或者，要手动配置 VS Code，请从下方示例中选择适当的 JSON 块并添加到主机配置：

<table>
<tr><th>使用 OAuth</th><th>使用 GitHub PAT</th></tr>
<tr><th align=left colspan=2>VS Code（版本 1.101 或更新）</th></tr>
<tr valign=top>
<td>

```json
{
  "servers": {
    "github": {
      "type": "http",
      "url": "https://api.githubcopilot.com/mcp/"
    }
  }
}
```

</td>
<td>

```json
{
  "servers": {
    "github": {
      "type": "http",
      "url": "https://api.githubcopilot.com/mcp/",
      "headers": {
        "Authorization": "Bearer ${input:github_mcp_pat}"
      }
    }
  },
  "inputs": [
    {
      "type": "promptString",
      "id": "github_mcp_pat",
      "description": "GitHub Personal Access Token",
      "password": true
    }
  ]
}
```

</td>
</tr>
</table>

### 在其他 MCP 主机中安装

- **[Copilot CLI](https://github.com/github/github-mcp-server/blob/3366807bbb8473841894de040d4abc89193e5d51/docs/installation-guides/install-copilot-cli.md)** - GitHub Copilot CLI 安装指南
- **[其他 IDE 中的 GitHub Copilot](https://github.com/github/github-mcp-server/blob/3366807bbb8473841894de040d4abc89193e5d51/docs/installation-guides/install-other-copilot-ides.md)** - 在 JetBrains、Visual Studio、Eclipse 和 Xcode 中安装 GitHub Copilot
- **[Claude 应用](https://github.com/github/github-mcp-server/blob/3366807bbb8473841894de040d4abc89193e5d51/docs/installation-guides/install-claude.md)** - Claude Desktop 和 Claude Code CLI 安装指南
- **[Codex](https://github.com/github/github-mcp-server/blob/3366807bbb8473841894de040d4abc89193e5d51/docs/installation-guides/install-codex.md)** - OpenAI Codex 安装指南
- **[Cursor](https://github.com/github/github-mcp-server/blob/3366807bbb8473841894de040d4abc89193e5d51/docs/installation-guides/install-cursor.md)** - Cursor IDE 安装指南
- **[OpenCode](https://github.com/github/github-mcp-server/blob/3366807bbb8473841894de040d4abc89193e5d51/docs/installation-guides/install-opencode.md)** - OpenCode 终端代理安装指南
- **[Windsurf](https://github.com/github/github-mcp-server/blob/3366807bbb8473841894de040d4abc89193e5d51/docs/installation-guides/install-windsurf.md)** - Windsurf IDE 安装指南
- **[Zed](https://github.com/github/github-mcp-server/blob/3366807bbb8473841894de040d4abc89193e5d51/docs/installation-guides/install-zed.md)** - Zed 编辑器安装指南
- **[Rovo Dev CLI](https://github.com/github/github-mcp-server/blob/3366807bbb8473841894de040d4abc89193e5d51/docs/installation-guides/install-rovo-dev-cli.md)** - Rovo Dev CLI 安装指南

> **注意：** 每个 MCP 主机应用都需要配置 GitHub App 或 OAuth App，才能通过 OAuth 支持远程访问。任何支持远程 MCP 服务器的主机应用都应支持使用 PAT 认证的远程 GitHub 服务器。配置细节和支持程度因主机而异，请参阅主机应用文档了解更多信息。

### 配置

#### 工具集配置

远程服务器配置、工具集、标头和高级用法的完整细节参见[远程服务器文档](https://github.com/github/github-mcp-server/blob/3366807bbb8473841894de040d4abc89193e5d51/docs/remote-server.md)。该文件详细说明并举例介绍如何在 VS Code 和其他 MCP 主机中连接、自定义及安装远程 GitHub MCP 服务器。

未指定工具集时，使用[默认工具集](https://github.com/github/github-mcp-server/blob/3366807bbb8473841894de040d4abc89193e5d51/README.md#default-toolset)。

#### Insiders 模式

> **抢先体验新功能！** 远程服务器提供 insiders 版本，可提前使用新功能和实验性工具。

<table>
<tr><th>使用 URL 路径</th><th>使用标头</th></tr>
<tr valign=top>
<td>

```json
{
  "servers": {
    "github": {
      "type": "http",
      "url": "https://api.githubcopilot.com/mcp/insiders"
    }
  }
}
```

</td>
<td>

```json
{
  "servers": {
    "github": {
      "type": "http",
      "url": "https://api.githubcopilot.com/mcp/",
      "headers": {
        "X-MCP-Insiders": "true"
      }
    }
  }
}
```

</td>
</tr>
</table>

更多细节和示例参见[远程服务器文档](https://github.com/github/github-mcp-server/blob/3366807bbb8473841894de040d4abc89193e5d51/docs/remote-server.md#insiders-mode)，完整可用功能列表参见 [Insiders 功能](https://github.com/github/github-mcp-server/blob/3366807bbb8473841894de040d4abc89193e5d51/docs/insiders-features.md)。

#### GitHub Enterprise

##### 带数据驻留的 GitHub Enterprise Cloud（ghe.com）

GitHub Enterprise Cloud 也可以使用远程服务器。

对 `https://octocorp.ghe.com` 使用 GitHub PAT 令牌的示例：

```
{
    ...
    "github-octocorp": {
      "type": "http",
      "url": "https://copilot-api.octocorp.ghe.com/mcp",
      "headers": {
        "Authorization": "Bearer ${input:github_mcp_pat}"
      }
    },
    ...
}
```

> **注意：** 在 VS Code 和 GitHub Copilot 中通过 OAuth 使用 GitHub Enterprise 时，还需要将 VS Code 设置指向你的 GitHub Enterprise 实例，参见[从 VS Code 进行认证](https://docs.github.com/en/enterprise-cloud@latest/copilot/how-tos/configure-personal-settings/authenticate-to-ghecom)。

##### GitHub Enterprise Server

GitHub Enterprise Server 不支持托管远程服务器。请参阅本地服务器配置中的 [GitHub Enterprise Server 和带数据驻留的 Enterprise Cloud（ghe.com）](https://github.com/github/github-mcp-server/blob/3366807bbb8473841894de040d4abc89193e5d51/README.md#github-enterprise-server-and-enterprise-cloud-with-data-residency-ghecom)。

---

## 本地 GitHub MCP 服务器

[![Install with Docker in VS Code](https://img.shields.io/badge/VS_Code-Install_Server-0098FF?style=flat-square&logo=visualstudiocode&logoColor=white)](https://insiders.vscode.dev/redirect/mcp/install?name=github&config=%7B%22command%22%3A%22docker%22%2C%22args%22%3A%5B%22run%22%2C%22-i%22%2C%22--rm%22%2C%22-p%22%2C%22127.0.0.1%3A8085%3A8085%22%2C%22-e%22%2C%22GITHUB_OAUTH_CALLBACK_PORT%22%2C%22ghcr.io%2Fgithub%2Fgithub-mcp-server%22%5D%2C%22env%22%3A%7B%22GITHUB_OAUTH_CALLBACK_PORT%22%3A%228085%22%7D%7D) [![Install with Docker in VS Code Insiders](https://img.shields.io/badge/VS_Code_Insiders-Install_Server-24bfa5?style=flat-square&logo=visualstudiocode&logoColor=white)](https://insiders.vscode.dev/redirect/mcp/install?name=github&config=%7B%22command%22%3A%22docker%22%2C%22args%22%3A%5B%22run%22%2C%22-i%22%2C%22--rm%22%2C%22-p%22%2C%22127.0.0.1%3A8085%3A8085%22%2C%22-e%22%2C%22GITHUB_OAUTH_CALLBACK_PORT%22%2C%22ghcr.io%2Fgithub%2Fgithub-mcp-server%22%5D%2C%22env%22%3A%7B%22GITHUB_OAUTH_CALLBACK_PORT%22%3A%228085%22%7D%7D&quality=insiders) [![Install with Docker in Visual Studio](https://img.shields.io/badge/Visual_Studio-Install_Server-C16FDE?style=flat-square&logo=visualstudio&logoColor=white)](https://aka.ms/vs/mcp-install?%7B%22name%22%3A%22github%22%2C%22command%22%3A%22docker%22%2C%22args%22%3A%5B%22run%22%2C%22-i%22%2C%22--rm%22%2C%22-p%22%2C%22127.0.0.1%3A8085%3A8085%22%2C%22-e%22%2C%22GITHUB_OAUTH_CALLBACK_PORT%3D8085%22%2C%22ghcr.io%2Fgithub%2Fgithub-mcp-server%22%5D%7D)

### 前提条件

1. 若要在容器中运行服务器，需要安装 [Docker](https://www.docker.com/)。
2. 安装 Docker 后，还需确保 Docker 正在运行。Docker 镜像位于 `ghcr.io/github/github-mcp-server`。该镜像是公开的；如果拉取时出错，令牌可能已过期，需要执行 `docker logout ghcr.io`。
3. **认证。** 在 github.com 上无需预先创建任何内容：上方的一键按钮在首次使用时通过 OAuth 登录（基于浏览器的流程；令牌仅保存在内存中）。Docker 按钮发布固定回调端口（`127.0.0.1:8085`），使容器的登录回调可访问。工作方式、无头/设备代码回退方案以及自带 OAuth 或 GitHub App（GitHub Enterprise Server 和 `ghe.com` 必需）参见**[本地服务器 OAuth 登录](https://github.com/github/github-mcp-server/blob/3366807bbb8473841894de040d4abc89193e5d51/docs/oauth-login.md)**。

   更喜欢令牌？仍可通过设置 `GITHUB_PERSONAL_ACCESS_TOKEN` 使用 [GitHub 个人访问令牌](https://github.com/settings/personal-access-tokens/new) 认证（其优先级高于 OAuth）。MCP 服务器可以使用许多 GitHub API，因此请启用你愿意授予 AI 工具的权限（有关访问令牌的更多信息，请参阅[文档](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens)）。

<details><summary><b>安全处理 PAT</b></summary>

### 环境变量（推荐）

为了让 GitHub PAT 保持安全，并能在不同 MCP 主机间复用：

1. **将 PAT 存储在环境变量中**

   ```bash
   export GITHUB_PAT=your_token_here
   ```

   或创建 `.env` 文件：

   ```env
   GITHUB_PAT=your_token_here
   ```

2. **保护你的 `.env` 文件**

   ```bash
   # 添加到 .gitignore 以防意外提交
   echo ".env" >> .gitignore
   ```

3. **在配置中引用令牌**

   ```bash
   # CLI 用法
   claude mcp add github -e GITHUB_PERSONAL_ACCESS_TOKEN=$GITHUB_PAT -- docker run -i --rm -e GITHUB_PERSONAL_ACCESS_TOKEN ghcr.io/github/github-mcp-server

   # 配置文件中（支持时）
   "env": {
     "GITHUB_PERSONAL_ACCESS_TOKEN": "$GITHUB_PAT"
   }
   ```

> **注意**：环境变量支持情况因主机应用和 IDE 而异。一些应用（如 Windsurf）要求在配置文件中硬编码令牌。

### 令牌安全最佳实践

- **最小权限范围**：只授予必要权限
  - `repo` - 仓库操作
  - `read:packages` - Docker 镜像访问
  - `read:org` - 组织团队访问
- **分开使用令牌**：不同项目/环境使用不同 PAT
- **定期轮换**：定期更新令牌
- **绝不提交**：不要将令牌提交到版本控制
- **文件权限**：限制对含令牌配置文件的访问

  ```bash
  chmod 600 ~/.your-app/config.json
  ```

</details>

### GitHub Enterprise Server 和带数据驻留的 Enterprise Cloud（ghe.com）

可以使用 `--gh-host` 标志和 `GITHUB_HOST` 环境变量设置
GitHub Enterprise Server 或带数据驻留的 GitHub Enterprise Cloud 的主机名。

- 对于 GitHub Enterprise Server，请在主机名前加上 `https://` URI 方案。HTTPS 是强制要求：非 HTTPS 主机会被拒绝，以免凭据通过明文传输（唯一例外是用于本地开发的 `http://localhost` 等环回主机）。
- 对于带数据驻留的 GitHub Enterprise Cloud，使用 `https://YOURSUBDOMAIN.ghe.com` 作为主机名。

``` json
"github": {
    "command": "docker",
    "args": [
    "run",
    "-i",
    "--rm",
    "-e",
    "GITHUB_PERSONAL_ACCESS_TOKEN",
    "-e",
    "GITHUB_HOST",
    "ghcr.io/github/github-mcp-server"
    ],
    "env": {
        "GITHUB_PERSONAL_ACCESS_TOKEN": "${input:github_token}",
        "GITHUB_HOST": "https://<your GHES or ghe.com domain name>"
    }
}
```

## 安装

### 在 VS Code 的 GitHub Copilot 中安装

要快速安装，请使用上方的一键安装按钮之一。完成流程后切换代理模式（位于 Copilot Chat 文本输入框旁），服务器便会启动。

有关在 VS Code 中使用 MCP 服务器工具的更多信息，参见[代理模式文档](https://code.visualstudio.com/docs/copilot/chat/mcp-servers)。

在其他 IDE（JetBrains、Visual Studio、Eclipse 等）的 GitHub Copilot 中安装

将下列 JSON 块之一添加到 IDE 的 MCP 设置中。

**使用 OAuth 登录（无需创建或存储令牌）。** 在 github.com 上，官方镜像已包含应用凭据，因此无需自行提供：首次使用时会运行基于浏览器的登录，并将生成的令牌**仅保存在内存中**。在 Docker 中，这需要向环回地址发布固定回调端口，以便访问容器的登录回调：

```json
{
  "mcp": {
    "servers": {
      "github": {
        "command": "docker",
        "args": [
          "run",
          "-i",
          "--rm",
          "-p",
          "127.0.0.1:8085:8085",
          "-e",
          "GITHUB_OAUTH_CALLBACK_PORT",
          "ghcr.io/github/github-mcp-server"
        ],
        "env": {
          "GITHUB_OAUTH_CALLBACK_PORT": "8085"
        }
      }
    }
  }
}
```

原生二进制流程（无需固定端口）、无头/设备代码回退方案、GitHub Enterprise Server / `ghe.com` 及自带 OAuth 或 GitHub App 的说明，参见**[本地服务器 OAuth 登录](https://github.com/github/github-mcp-server/blob/3366807bbb8473841894de040d4abc89193e5d51/docs/oauth-login.md)**。

非交互式 stdio 部署参见 **[GitHub App 认证](https://github.com/github/github-mcp-server/blob/3366807bbb8473841894de040d4abc89193e5d51/docs/github-app-auth.md)**。

**或使用个人访问令牌认证。** 改为设置 `GITHUB_PERSONAL_ACCESS_TOKEN`（其优先级高于 OAuth）：

```json
{
  "mcp": {
    "inputs": [
      {
        "type": "promptString",
        "id": "github_token",
        "description": "GitHub Personal Access Token",
        "password": true
      }
    ],
    "servers": {
      "github": {
        "command": "docker",
        "args": [
          "run",
          "-i",
          "--rm",
          "-e",
          "GITHUB_PERSONAL_ACCESS_TOKEN",
          "ghcr.io/github/github-mcp-server"
        ],
        "env": {
          "GITHUB_PERSONAL_ACCESS_TOKEN": "${input:github_token}"
        }
      }
    }
  }
}
```

也可以在工作区中名为 `.vscode/mcp.json` 的文件中添加类似示例（即不带 mcp 键）。这样可以与接受相同格式的其他主机应用共享配置。

<details>
<summary><b>不含 MCP 键的 JSON 示例</b></summary>
<br>

```json
{
  "inputs": [
    {
      "type": "promptString",
      "id": "github_token",
      "description": "GitHub Personal Access Token",
      "password": true
    }
  ],
  "servers": {
    "github": {
      "command": "docker",
      "args": [
        "run",
        "-i",
        "--rm",
        "-e",
        "GITHUB_PERSONAL_ACCESS_TOKEN",
        "ghcr.io/github/github-mcp-server"
      ],
      "env": {
        "GITHUB_PERSONAL_ACCESS_TOKEN": "${input:github_token}"
      }
    }
  }
}
```

</details>

### 在其他 MCP 主机中安装

其他 MCP 主机应用请参阅我们的安装指南：

- **[Copilot CLI](https://github.com/github/github-mcp-server/blob/3366807bbb8473841894de040d4abc89193e5d51/docs/installation-guides/install-copilot-cli.md)** - GitHub Copilot CLI 安装指南
- **[其他 IDE 中的 GitHub Copilot](https://github.com/github/github-mcp-server/blob/3366807bbb8473841894de040d4abc89193e5d51/docs/installation-guides/install-other-copilot-ides.md)** - 在 JetBrains、Visual Studio、Eclipse 和 Xcode 中安装 GitHub Copilot
- **[Claude Code 与 Claude Desktop](https://github.com/github/github-mcp-server/blob/3366807bbb8473841894de040d4abc89193e5d51/docs/installation-guides/install-claude.md)** - Claude Code 和 Claude Desktop 安装指南
- **[Cursor](https://github.com/github/github-mcp-server/blob/3366807bbb8473841894de040d4abc89193e5d51/docs/installation-guides/install-cursor.md)** - Cursor IDE 安装指南
- **[Google Gemini CLI](https://github.com/github/github-mcp-server/blob/3366807bbb8473841894de040d4abc89193e5d51/docs/installation-guides/install-gemini-cli.md)** - Google Gemini CLI 安装指南
- **[OpenCode](https://github.com/github/github-mcp-server/blob/3366807bbb8473841894de040d4abc89193e5d51/docs/installation-guides/install-opencode.md)** - OpenCode 终端代理安装指南
- **[Windsurf](https://github.com/github/github-mcp-server/blob/3366807bbb8473841894de040d4abc89193e5d51/docs/installation-guides/install-windsurf.md)** - Windsurf IDE 安装指南
- **[Zed](https://github.com/github/github-mcp-server/blob/3366807bbb8473841894de040d4abc89193e5d51/docs/installation-guides/install-zed.md)** - Zed 编辑器安装指南

所有安装选项的完整概览参见**[安装指南索引](https://github.com/github/github-mcp-server/blob/3366807bbb8473841894de040d4abc89193e5d51/docs/installation-guides)**。

> **注意：** 任何支持本地 MCP 服务器的主机应用都应能访问本地 GitHub MCP 服务器。不过，具体配置过程、语法和集成稳定性因主机应用而异。虽然许多应用可能采用与上述示例类似的格式，但并不保证如此。正确的 MCP 配置语法和设置过程请参阅主机应用文档。

### 从源码构建

如果没有 Docker，可以在 `cmd/github-mcp-server` 目录中使用 `go build`
构建二进制文件，然后使用 `github-mcp-server stdio` 命令，并将环境变量 `GITHUB_PERSONAL_ACCESS_TOKEN` 设置为你的令牌。使用 `-o` 标志指定构建输出位置。应将服务器配置中的 `command` 设为构建出的可执行文件。

STDIO API 请求将服务器标识为 `github-mcp-server/<version>`，并在可用时将上游 MCP 客户端名称/版本保留在括号中。客户端元数据中的控制字符、引号、反斜杠和括号会被转义，以保持 HTTP 标头有效；普通名称、版本、空格和可打印 Unicode 保持不变。发布构建保留其发布版本。带修订元数据的源码构建和默认 Docker 构建使用 `vcs-<full-commit-sha>`。Docker 发布工作流保留标签版本（包括预发布版），并让分支、拉取请求及夜间构建使用关联的源码修订。`-dirty` 标记仅在修订和修改状态均来自嵌入式 VCS 元数据时适用，绝不用于有效且显式提供的 `main.commit`。这是 VCS 构建标识，而非发布版本号。

在 Git 检出目录运行 `go build -o github-mcp-server ./cmd/github-mcp-server` 构建完整包，以嵌入 VCS 修订。对于没有 VCS 元数据的构建，使用 `-ldflags '-X main.version=<release>'` 提供实际发布版本，或使用 `-ldflags '-X main.commit=<sha>'` 提供完整源码修订。有效元数据按以下顺序选择：显式发布版本、显式源码修订、嵌入式 VCS 修订、已安装的主模块版本。即使构建上下文过滤改变了嵌入式 VCS 元数据，有效的显式修订仍为准。缺失或格式错误的候选值会依次回退到下一个可用来源。若均不可用，STDIO 仍以开发标签 `dev` 启动并向 stderr 输出警告，stdout 则留给 MCP 协议。需要按版本归因时请提供真实发布或修订元数据。

例如：

```JSON
{
  "mcp": {
    "servers": {
      "github": {
        "command": "/path/to/github-mcp-server",
        "args": ["stdio"],
        "env": {
          "GITHUB_PERSONAL_ACCESS_TOKEN": "<YOUR_TOKEN>"
        }
      }
    }
  }
}
```

## 工具配置

GitHub MCP 服务器支持通过 `--toolsets` 标志启用或禁用特定功能组，从而控制 AI 工具可使用哪些 GitHub API 能力。只启用所需工具集有助于大语言模型选择工具并减小上下文。

_工具集不限于工具；适用时也包含相关 MCP 资源和提示词。_

未指定工具集时，使用[默认工具集](https://github.com/github/github-mcp-server/blob/3366807bbb8473841894de040d4abc89193e5d51/README.md#default-toolset)。

> **想看示例？** 常见配置方法，如最小设置、只读模式以及将工具与工具集组合使用，请参见[服务器配置指南](https://github.com/github/github-mcp-server/blob/3366807bbb8473841894de040d4abc89193e5d51/docs/server-configuration.md)。

#### 指定工具集

可以通过两种方式传入允许列表，指定供大语言模型使用的工具集：

1. **使用命令行参数**：

   ```bash
   github-mcp-server --toolsets repos,issues,pull_requests,actions,code_security
   ```

2. **使用环境变量**：

   ```bash
   GITHUB_TOOLSETS="repos,issues,pull_requests,actions,code_security" ./github-mcp-server
   ```

如果同时提供环境变量 `GITHUB_TOOLSETS` 和命令行参数，环境变量优先。

#### 指定单个工具

也可以使用 `--tools` 标志配置特定工具。工具可单独使用，也可与工具集组合，以实现细粒度控制。

1. **使用命令行参数**：

   ```bash
   github-mcp-server --tools get_file_contents,issue_read,create_pull_request
   ```

2. **使用环境变量**：

   ```bash
   GITHUB_TOOLS="get_file_contents,issue_read,create_pull_request" ./github-mcp-server
   ```

3. **与工具集组合**（累加）：

   ```bash
   github-mcp-server --toolsets repos,issues --tools get_gist
   ```

   这会注册 `repos` 和 `issues` 工具集中的所有工具，以及 `get_gist`。

**重要说明：**

- 可以同时使用工具和工具集
- 只读模式优先：如果设置了 `--read-only`，即使通过 `--tools` 明确请求也会跳过写入工具
- 工具名称必须完全匹配（例如 `get_file_contents`，而非 `getFileContents`）。无效名称会导致服务器启动失败并报错
- 工具重命名后，旧名称会保留为别名以兼容旧版本。详见[工具重命名](https://github.com/github/github-mcp-server/blob/3366807bbb8473841894de040d4abc89193e5d51/docs/tool-renaming.md)。

### 在 Docker 中使用工具集

使用 Docker 时，可以通过环境变量传入工具集：

```bash
docker run -i --rm \
  -e GITHUB_PERSONAL_ACCESS_TOKEN=<your-token> \
  -e GITHUB_TOOLSETS="repos,issues,pull_requests,actions,code_security" \
  ghcr.io/github/github-mcp-server
```

### 在 Docker 中使用工具

使用 Docker 时，可以通过环境变量传入特定工具，也可以将工具与工具集组合：

```bash
# 仅工具
docker run -i --rm \
  -e GITHUB_PERSONAL_ACCESS_TOKEN=<your-token> \
  -e GITHUB_TOOLS="get_file_contents,issue_read,create_pull_request" \
  ghcr.io/github/github-mcp-server

# 工具与工具集组合（累加）
docker run -i --rm \
  -e GITHUB_PERSONAL_ACCESS_TOKEN=<your-token> \
  -e GITHUB_TOOLSETS="repos,issues" \
  -e GITHUB_TOOLS="get_gist" \
  ghcr.io/github/github-mcp-server
```

### 特殊工具集

#### "all" 工具集

可以提供特殊工具集 `all`，无论其他配置如何都启用所有可用工具集：

```bash
./github-mcp-server --toolsets all
```

或使用环境变量：

```bash
GITHUB_TOOLSETS="all" ./github-mcp-server
```

#### "default" 工具集

默认工具集 `default` 是未指定工具集时传给服务器的配置。

默认配置为：

- context
- repos
- issues
- pull_requests
- users

要保留默认配置并添加其他工具集：

```bash
GITHUB_TOOLSETS="default,stargazers" ./github-mcp-server
```

### Insiders 模式

本地 GitHub MCP 服务器提供 insiders 版本，可提前使用新功能和实验性工具。

1. **使用命令行参数**：

   ```bash
   ./github-mcp-server --insiders
   ```

2. **使用环境变量**：

   ```bash
   GITHUB_INSIDERS=true ./github-mcp-server
   ```

使用 Docker 时：

```bash
docker run -i --rm \
  -e GITHUB_PERSONAL_ACCESS_TOKEN=<your-token> \
  -e GITHUB_INSIDERS=true \
  ghcr.io/github/github-mcp-server
```

### 可用工具集

提供以下工具集：

<!-- START AUTOMATED TOOLSETS -->
|     | 工具集                 | 说明                                                   |
| --- | ----------------------- | ------------------------------------------------------------- |
| <picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/person-dark.png"><source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/person-light.png"><img src="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/person-light.png" width="20" height="20" alt="person"></picture> | `context`               | **强烈推荐**：提供当前用户及操作所在 GitHub 上下文的工具 |
| <picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/workflow-dark.png"><source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/workflow-light.png"><img src="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/workflow-light.png" width="20" height="20" alt="workflow"></picture> | `actions` | GitHub Actions 工作流和 CI/CD 操作 |
| <picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/code-square-dark.png"><source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/code-square-light.png"><img src="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/code-square-light.png" width="20" height="20" alt="code-square"></picture> | `code_quality` | GitHub Code Quality 相关工具 |
| <picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/codescan-dark.png"><source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/codescan-light.png"><img src="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/codescan-light.png" width="20" height="20" alt="codescan"></picture> | `code_security` | 代码安全相关工具，如 GitHub Code Scanning |
| <picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/copilot-dark.png"><source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/copilot-light.png"><img src="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/copilot-light.png" width="20" height="20" alt="copilot"></picture> | `copilot` | Copilot 相关工具 |
| <picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/copilot-dark.png"><source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/copilot-light.png"><img src="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/copilot-light.png" width="20" height="20" alt="copilot"></picture> | `copilot_issue_intents` | 选择启用的 Copilot 议题指派工具，携带意图元数据（理由、置信度、建议） |
| <picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/dependabot-dark.png"><source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/dependabot-light.png"><img src="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/dependabot-light.png" width="20" height="20" alt="dependabot"></picture> | `dependabot` | Dependabot 工具 |
| <picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/comment-discussion-dark.png"><source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/comment-discussion-light.png"><img src="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/comment-discussion-light.png" width="20" height="20" alt="comment-discussion"></picture> | `discussions` | GitHub Discussions 相关工具 |
| <picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/logo-gist-dark.png"><source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/logo-gist-light.png"><img src="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/logo-gist-light.png" width="20" height="20" alt="logo-gist"></picture> | `gists` | GitHub Gist 相关工具 |
| <picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/git-branch-dark.png"><source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/git-branch-light.png"><img src="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/git-branch-light.png" width="20" height="20" alt="git-branch"></picture> | `git` | 用于底层 Git 操作的 GitHub Git API 相关工具 |
| <picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/law-dark.png"><source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/law-light.png"><img src="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/law-light.png" width="20" height="20" alt="law"></picture> | `governance` | 仓库治理工具，用于管理仓库、组织和企业级规则集与自定义属性 |
| <picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/issue-opened-dark.png"><source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/issue-opened-light.png"><img src="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/issue-opened-light.png" width="20" height="20" alt="issue-opened"></picture> | `issues` | GitHub Issues 相关工具 |
| <picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/tag-dark.png"><source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/tag-light.png"><img src="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/tag-light.png" width="20" height="20" alt="tag"></picture> | `labels` | GitHub Labels 相关工具 |
| <picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/bell-dark.png"><source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/bell-light.png"><img src="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/bell-light.png" width="20" height="20" alt="bell"></picture> | `notifications` | GitHub Notifications 相关工具 |
| <picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/organization-dark.png"><source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/organization-light.png"><img src="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/organization-light.png" width="20" height="20" alt="organization"></picture> | `orgs` | GitHub Organization 相关工具 |
| <picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/project-dark.png"><source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/project-light.png"><img src="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/project-light.png" width="20" height="20" alt="project"></picture> | `projects` | GitHub Projects 相关工具 |
| <picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/git-pull-request-dark.png"><source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/git-pull-request-light.png"><img src="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/git-pull-request-light.png" width="20" height="20" alt="git-pull-request"></picture> | `pull_requests` | GitHub Pull Request 相关工具 |
| <picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/repo-dark.png"><source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/repo-light.png"><img src="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/repo-light.png" width="20" height="20" alt="repo"></picture> | `repos` | GitHub Repository 相关工具 |
| <picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/shield-lock-dark.png"><source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/shield-lock-light.png"><img src="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/shield-lock-light.png" width="20" height="20" alt="shield-lock"></picture> | `secret_protection` | 密钥保护相关工具，如 GitHub Secret Scanning |
| <picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/shield-dark.png"><source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/shield-light.png"><img src="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/shield-light.png" width="20" height="20" alt="shield"></picture> | `security_advisories` | 安全公告相关工具 |
| <picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/star-dark.png"><source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/star-light.png"><img src="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/star-light.png" width="20" height="20" alt="star"></picture> | `stargazers` | GitHub Stargazers 相关工具 |
| <picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/people-dark.png"><source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/people-light.png"><img src="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/people-light.png" width="20" height="20" alt="people"></picture> | `users` | GitHub User 相关工具 |
<!-- END AUTOMATED TOOLSETS -->

### 远程 GitHub MCP 服务器中的附加工具集

| 工具集                 | 说明                                                   |
| ----------------------- | ------------------------------------------------------------- |
| `copilot` | Copilot 相关工具（如 Copilot Coding Agent） |
| `copilot_spaces` | Copilot Spaces 相关工具 |
| `github_support_docs_search` | 搜索文档以回答 GitHub 产品与支持问题 |

## 工具

当协商的 MCP 协议版本为受支持的 `2026-07-28` 或更新版本时，仓库工具会公开输出模式和有类型的 `structuredContent`。较旧客户端、协议版本未知的客户端及省略版本的客户端会收到现有内容，不含输出模式或结构化输出。未知版本不会仅因排序晚于 `2026-07-28` 就被视为受支持。

对于 `get_file_contents`，目录结果（包括字段投影）是结构化数组。文件下载和仅元数据响应使用由有类型文本、嵌入资源或资源链接块组成的 `content` 数组，保留状态消息、文本、base64 数据块、空文件和下载链接。加星、删除等纯文本仓库变更也返回结构化 `message`；fork 响应区分仓库引用和进行中消息。仓库删除在返回完成结果前仍需经过原有确认流程。

结构化仓库结果使用精简 DTO，而非原始 REST 对象。发布版本查询与发布列表使用相同的精简发布结构；Git 引用保留 ref、对象类型和 SHA；附注标签保留标签/消息/标记者/目标信息；删除提交保留 SHA、消息、作者、树 SHA、父提交 SHA 以及可用时供人使用的 `html_url`。目录条目保留文件元数据和有用的 HTML/下载链接，但即使投影请求这些字段也省略 API 和超媒体路由 URL。旧版及未知协议版本逐字节保留原始文本；新版纯文本 JSON 响应将与 `structuredContent` 相同的精简 DTO 序列化。非文本资源块和工具错误保留现有内容行为。结构化投影使用与旧版格式化器相同的经过净化和筛选的来源。

对于 `get_repository_tree`、`list_gists`、`get_gist`、`create_gist` 和 `update_gist`，gist 结果保留文件名、内容、大小、语言/类型信息、时间戳、可见性和精简所有者资料，同时省略 API/node 和 Git 传输 URL。树条目省略可推导的 API URL。创建/更新 gist 的结果包含 gist ID 和 HTML URL。新版 JSON 文本与结构化内容序列化同一精简结果。较旧或未知协议版本逐字节保留旧版文本，不含输出模式或结构化内容。

议题元数据（`list_issue_types`、`list_issue_fields`）、评论（`add_issue_comment`、`update_issue_comment`）、依赖（`issue_dependency_read`、`issue_dependency_write`）和重复检测（`find_duplicate`）工具在协商到受支持的 `2026-07-28` 或更新协议版本时提供有类型的 `outputSchema` 和 `structuredContent`。较旧或未知协议版本保留不含这些字段的原有文本响应。工具错误不返回结构化内容。依赖和重复检测工具保留现有功能开关。

当协商到受支持的 `2026-07-28` 或更新协议版本时，合并后的 Actions 工具（`actions_list`、`actions_get`、`actions_run_trigger`）返回包含 `method` 判别字段和特定方法的有类型字段的结构化对象；JSON 文本包含相同的精简投影。工作流、任务和产物投影省略 API/超媒体 URL，但保留浏览器链接及有用的标识符、状态、时间戳和使用时长。运行用量按确定性的名称顺序列出计费运行器环境。旧版和未知协议的文本响应保持不变；`get_job_logs` 保留其有类型内容、URL 或失败任务集合等变体。

<!-- START AUTOMATED TOOLS -->
<details>

<summary><picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/workflow-dark.png"><source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/workflow-light.png"><img src="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/workflow-light.png" width="20" height="20" alt="workflow"></picture> Actions</summary>

- **actions_get** - 获取 GitHub Actions 资源（工作流、工作流运行、任务和产物）的详情
  - **OAuth Challenge Scopes**: `repo`
  - `method`：要执行的方法（字符串，必填）
  - `owner`: 仓库所有者（字符串，必填）
  - `repo`: 仓库名称（字符串，必填）
  - `resource_id`：资源的唯一标识符。该值随所提供的 "method" 而异，请确保提供正确的 ID：
    - 对 'get_workflow' 方法提供工作流 ID 或工作流文件名（例如 ci.yaml）。
    - 对 'get_workflow_run'、'get_workflow_run_usage' 和 'get_workflow_run_logs_url' 方法提供工作流运行 ID。
    - 对 'download_workflow_run_artifact' 方法提供产物 ID。
    - 对 'get_workflow_job' 方法提供任务 ID。
     （字符串，必填）

- **actions_list** - 列出仓库中的 GitHub Actions 工作流
  - **OAuth Challenge Scopes**: `repo`
  - `method`: 要执行的操作（字符串，必填）
  - `owner`: 仓库所有者（字符串，必填）
  - `page`：分页页码（默认：1）（数字，可选）
  - `perPage`：分页时每页结果数（默认：30，最大：100）（数字，可选）
  - `repo`: 仓库名称（字符串，必填）
  - `resource_id`：资源的唯一标识符。该值随所提供的 "method" 而异，请确保提供正确的 ID：
    - 对 'list_workflows' 方法不要提供资源 ID。
    - 对 'list_workflow_runs' 方法提供工作流 ID 或工作流文件名（例如 ci.yaml），或省略以列出仓库中的所有工作流运行。
    - 对 'list_workflow_jobs' 和 'list_workflow_run_artifacts' 方法提供工作流运行 ID。
     （字符串，可选）
  - `workflow_jobs_filter`：工作流任务筛选条件。**仅**在 method 为 'list_workflow_jobs' 时使用（对象，可选）
  - `workflow_runs_filter`：工作流运行筛选条件。**仅**在 method 为 'list_workflow_runs' 时使用（对象，可选）

- **actions_run_trigger** - 触发 GitHub Actions 工作流操作
  - **OAuth Challenge Scopes**: `repo`
  - `inputs`：工作流接受的输入。仅用于 'run_workflow' 方法。（对象，可选）
  - `method`: 要执行的方法（字符串，必填）
  - `owner`: 仓库所有者（字符串，必填）
  - `ref`：工作流的 git 引用。可以是分支或标签名称。'run_workflow' 方法必填。（字符串，可选）
  - `repo`: 仓库名称（字符串，必填）
  - `run_id`：工作流运行 ID。除 'run_workflow' 外所有方法均必填。（数字，可选）
  - `workflow_id`：工作流 ID（数字）或工作流文件名（例如 main.yml、ci.yaml）。'run_workflow' 方法必填。（字符串，可选）

- **get_job_logs** - 获取 GitHub Actions 工作流任务日志
  - **OAuth Challenge Scopes**: `repo`
  - `failed_only`：为 true 时，获取 run_id 指定的工作流运行中所有失败任务的日志。必须提供 run_id。（布尔值，可选）
  - `job_id`：工作流任务的唯一标识符。获取单个任务的日志时必填。（数字，可选）
  - `owner`: 仓库所有者（字符串，必填）
  - `repo`: 仓库名称（字符串，必填）
  - `return_content`：返回实际日志内容而非 URL（布尔值，可选）
  - `run_id`：工作流运行的唯一标识符。当 failed_only 为 true 以获取该运行中所有失败任务的日志时必填。（数字，可选）
  - `tail_lines`：从日志末尾返回的行数（数字，可选）

</details>

<details>

<summary><picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/code-square-dark.png"><source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/code-square-light.png"><img src="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/code-square-light.png" width="20" height="20" alt="code-square"></picture> 代码质量</summary>

- **get_code_quality_finding** - 获取代码质量发现
  - **OAuth Challenge Scopes**: `repo`
  - `findingNumber`：发现的编号。（数字，必填）
  - `owner`: 仓库所有者。（字符串，必填）
  - `repo`: 仓库名称。（字符串，必填）

</details>

<details>

<summary><picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/codescan-dark.png"><source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/codescan-light.png"><img src="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/codescan-light.png" width="20" height="20" alt="codescan"></picture> 代码安全</summary>

- **get_code_scanning_alert** - 获取代码扫描警报
  - **OAuth Challenge Scopes**: `security_events`
  - `alertNumber`: 警报编号。（数字，必填）
  - `owner`: 仓库所有者。（字符串，必填）
  - `repo`: 仓库名称。（字符串，必填）

- **list_code_scanning_alerts** - 列出代码扫描警报
  - **OAuth Challenge Scopes**: `security_events`
  - `owner`: 仓库所有者。（字符串，必填）
  - `page`：分页页码（最小 1）（数字，可选）
  - `perPage`：分页时每页结果数（最小 1，最大 100）（数字，可选）
  - `ref`：要列出的结果对应的 Git 引用。（字符串，可选）
  - `repo`: 仓库名称。（字符串，必填）
  - `severity`：按严重程度筛选代码扫描警报（字符串，可选）
  - `state`：按状态筛选代码扫描警报。默认为 open（字符串，可选）
  - `tool_name`：用于代码扫描的工具名称。（字符串，可选）

</details>

<details>

<summary><picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/person-dark.png"><source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/person-light.png"><img src="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/person-light.png" width="20" height="20" alt="person"></picture> 上下文</summary>

- **get_me** - 获取当前用户资料
  - **MCP App UI**: `ui://github-mcp-server/get-me`
  - 无需参数

- **get_team_members** - 获取团队成员
  - **OAuth Challenge Scopes**: `read:org`
  - `org`：包含该团队的组织登录名（所有者）。（字符串，必填）
  - `team_slug`：团队 slug（字符串，必填）

- **get_teams** - 获取团队
  - **OAuth Challenge Scopes**: `read:org`
  - `user`：要获取团队的用户名。未提供时使用已认证用户。（字符串，可选）

- **ui_get** - 获取界面数据
  - **OAuth Challenge Scopes**: `repo`, `read:org`
  - `method`：要获取的数据类型（字符串，必填）
  - `owner`：仓库所有者（所有方法均必填）（字符串，必填）
  - `repo`：仓库名称（labels、assignees、milestones、branches、issue fields、reviewers 方法必填）（字符串，可选）

</details>

<details>

<summary><picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/copilot-dark.png"><source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/copilot-light.png"><img src="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/copilot-light.png" width="20" height="20" alt="copilot"></picture> Copilot</summary>

- **assign_copilot_to_issue** - 将 Copilot 指派给议题
  - **OAuth Challenge Scopes**: `repo`
  - `base_ref`：代理开始工作的 Git 引用（如分支）。未指定时默认为仓库默认分支（字符串，可选）
  - `custom_instructions`：议题正文之外用于指导代理的可选自定义说明。用于提供议题说明中未包含的额外上下文、约束或指导（字符串，可选）
  - `issue_number`：议题编号（数字，必填）
  - `owner`: 仓库所有者（字符串，必填）
  - `repo`: 仓库名称（字符串，必填）

- **request_copilot_review** - 请求 Copilot 审查
  - **OAuth Challenge Scopes**: `repo`
  - `owner`: 仓库所有者（字符串，必填）
  - `pullNumber`：拉取请求编号（数字，必填）
  - `repo`: 仓库名称（字符串，必填）

</details>

<details>

<summary><picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/copilot-dark.png"><source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/copilot-light.png"><img src="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/copilot-light.png" width="20" height="20" alt="copilot"></picture> Copilot 议题意图</summary>

- **assign_copilot_to_issue_with_intent** - 带意图将 Copilot 指派给议题
  - **OAuth Challenge Scopes**: `repo`
  - `base_ref`：代理开始工作的 Git 引用（如分支）。未指定时默认为仓库默认分支。is_suggestion 为 true 时忽略（字符串，可选）
  - `confidence`：对该选择的置信程度。'HIGH' 用于明确信号或用户明确请求，'MEDIUM' 用于存在一定歧义的合理推断，'LOW' 用于信号有限时的最佳猜测。（字符串，必填）
  - `custom_instructions`：议题正文之外用于指导代理的可选自定义说明。is_suggestion 为 true 时忽略（字符串，可选）
  - `is_suggestion`：为 true 时记录待处理的 Copilot 指派意图，而不是启动代理。稍后批准时提供启动上下文；这种情况下忽略 base_ref 和 custom_instructions。（布尔值，必填）
  - `issue_number`：议题编号（数字，必填）
  - `owner`: 仓库所有者（字符串，必填）
  - `rationale`：用简洁的一句话解释议题中的哪些具体因素促使选择 Copilot。说明具体信号（例如 '范围明确且验收标准清晰的任务'）。（字符串，必填）
  - `repo`: 仓库名称（字符串，必填）

</details>

<details>

<summary><picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/dependabot-dark.png"><source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/dependabot-light.png"><img src="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/dependabot-light.png" width="20" height="20" alt="dependabot"></picture> Dependabot</summary>

- **get_dependabot_alert** - 获取 Dependabot 警报
  - **OAuth Challenge Scopes**: `security_events`
  - `alertNumber`: 警报编号。（数字，必填）
  - `owner`: 仓库所有者。（字符串，必填）
  - `repo`: 仓库名称。（字符串，必填）

- **list_dependabot_alerts** - 列出 Dependabot 警报
  - **OAuth Challenge Scopes**: `security_events`
  - `after`：分页游标。使用上一次响应中的游标。（字符串，可选）
  - `owner`: 仓库所有者。（字符串，必填）
  - `perPage`: 分页时每页结果数（最小 1，最大 100）（数字，可选）
  - `repo`: 仓库名称。（字符串，必填）
  - `severity`：按严重程度筛选 Dependabot 警报（字符串，可选）
  - `state`：按状态筛选 Dependabot 警报。默认为 open（字符串，可选）

</details>

<details>

<summary><picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/comment-discussion-dark.png"><source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/comment-discussion-light.png"><img src="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/comment-discussion-light.png" width="20" height="20" alt="comment-discussion"></picture> 讨论</summary>

- **discussion_comment_write** - 管理讨论评论
  - **OAuth Challenge Scopes**: `repo`
  - `body`：评论内容（'add'、'reply' 和 'update' 方法必填）（字符串，可选）
  - `commentNodeID`：讨论评论的节点 ID（'reply'、'update'、'delete'、'mark_answer' 和 'unmark_answer' 方法必填）。对于 'reply'，这是要回复的顶级评论；GitHub Discussions 仅支持一级嵌套。（字符串，可选）
  - `discussionNumber`：讨论编号（'add' 和 'reply' 方法必填）（数字，可选）
  - `method`：对讨论评论执行的写操作。
    可选值：
    - 'add' - 为讨论添加新的顶级评论。
    - 'reply' - 回复顶级讨论评论（GitHub Discussions 仅支持一级嵌套）。
    - 'update' - 更新现有讨论评论。
    - 'delete' - 删除讨论评论。
    - 'mark_answer' - 将讨论评论标记为答案（仅限问答）。
    - 'unmark_answer' - 取消讨论评论的答案标记（仅限问答）。
     （字符串，必填）
  - `owner`：仓库所有者（'add' 和 'reply' 方法必填）（字符串，可选）
  - `repo`：仓库名称（'add' 和 'reply' 方法必填）（字符串，可选）

- **get_discussion** - 获取讨论
  - **OAuth Challenge Scopes**: `repo`
  - `discussionNumber`: 讨论编号（数字，必填）
  - `owner`: 仓库所有者（字符串，必填）
  - `repo`: 仓库名称（字符串，必填）

- **get_discussion_comments** - 获取讨论评论
  - **OAuth Challenge Scopes**: `repo`
  - `after`: 分页游标。使用上一次响应中的游标。（字符串，可选）
  - `discussionNumber`: 讨论编号（数字，必填）
  - `includeReplies`：为 true 时，每条顶级评论会嵌套包含其回复（每条评论最多 100 条回复，为 GitHub API 的上限）。默认为 false。（布尔值，可选）
  - `owner`: 仓库所有者（字符串，必填）
  - `perPage`: 分页时每页结果数（最小 1，最大 100）（数字，可选）
  - `repo`: 仓库名称（字符串，必填）

- **list_discussion_categories** - 列出讨论分类
  - **OAuth Challenge Scopes**: `repo`
  - `owner`: 仓库所有者（字符串，必填）
  - `repo`：仓库名称。未提供时将在组织级别查询讨论分类。（字符串，可选）

- **list_discussions** - 列出讨论
  - **OAuth Challenge Scopes**: `repo`
  - `after`: 分页游标。使用上一次响应中的游标。（字符串，可选）
  - `category`：可选，按讨论分类 ID 筛选。提供时仅列出该分类中的讨论。（字符串，可选）
  - `direction`：排序方向。（字符串，可选）
  - `orderBy`：按字段排序讨论。提供时还需提供 'direction'。（字符串，可选）
  - `owner`: 仓库所有者（字符串，必填）
  - `perPage`: 分页时每页结果数（最小 1，最大 100）（数字，可选）
  - `repo`：仓库名称。未提供时将在组织级别查询讨论。（字符串，可选）

</details>

<details>

<summary><picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/logo-gist-dark.png"><source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/logo-gist-light.png"><img src="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/logo-gist-light.png" width="20" height="20" alt="logo-gist"></picture> Gist</summary>

- **create_gist** - 创建 Gist
  - **OAuth Challenge Scopes**: `gist`
  - `content`：创建简单单文件 gist 的内容（字符串，必填）
  - `description`：gist 的说明（字符串，可选）
  - `filename`：创建简单单文件 gist 的文件名（字符串，必填）
  - `public`：gist 是否公开（布尔值，可选）

- **get_gist** - 获取 Gist 内容
  - `gist_id`：gist 的 ID（字符串，必填）

- **list_gists** - 列出 Gist
  - `page`: 分页页码（最小 1）（数字，可选）
  - `perPage`: 分页时每页结果数（最小 1，最大 100）（数字，可选）
  - `since`：仅返回此时间后更新的 gist（ISO 8601 时间戳）（字符串，可选）
  - `username`：GitHub 用户名（省略则返回已认证用户的 gist）（字符串，可选）

- **update_gist** - 更新 Gist
  - **OAuth Challenge Scopes**: `gist`
  - `content`：文件内容（字符串，必填）
  - `description`：更新后的 gist 说明（字符串，可选）
  - `filename`：要更新或创建的文件名（字符串，必填）
  - `gist_id`：要更新的 gist ID（字符串，必填）

</details>

<details>

<summary><picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/git-branch-dark.png"><source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/git-branch-light.png"><img src="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/git-branch-light.png" width="20" height="20" alt="git-branch"></picture> Git</summary>

- **get_repository_tree** - 获取仓库树
  - **OAuth Challenge Scopes**: `repo`
  - `owner`: 仓库所有者（用户名或组织）（字符串，必填）
  - `path_filter`：可选路径前缀，用于筛选树结果（例如 'src/' 仅显示 src 目录中的文件）（字符串，可选）
  - `recursive`：将此参数设为 true 时返回树所引用的对象或子树。默认为 false（布尔值，可选）
  - `repo`: 仓库名称（字符串，必填）
  - `tree_sha`：树的 SHA1 值或引用（分支或标签）名称。默认为仓库默认分支（字符串，可选）

</details>

<details>

<summary><picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/law-dark.png"><source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/law-light.png"><img src="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/law-light.png" width="20" height="20" alt="law"></picture> 治理</summary>

- **create_repository_ruleset** - 创建仓库规则集
  - **OAuth Challenge Scopes**: `repo`, `admin:org`, `admin:enterprise`
  - `bypass_actors`：可以绕过此规则集中规则的参与者（对象数组，可选）
  - `conditions`：规则集适用的条件，例如 {"ref_name": {"include": ["refs/heads/main"], "exclude": []}}（对象，可选）
  - `enforcement`：规则集的强制级别。'evaluate' 允许管理员在强制执行之前测试规则（字符串，必填）
  - `enterprise`: 企业 slug。当 level 为 'enterprise' 时必填。（字符串，可选）
  - `level`：配置规则集的级别：
    - 'repository'：单个仓库的规则集（需要 'owner' 和 'repo'）。
    - 'organization'：覆盖组织中各仓库的规则集（需要 'org'）。
    - 'enterprise'：覆盖企业中各仓库的规则集（需要 'enterprise'）。（字符串，必填）
  - `name`：规则集名称（字符串，必填）
  - `org`: 组织名称。当 level 为 'organization' 时必填。（字符串，可选）
  - `owner`: 仓库所有者。当 level 为 'repository' 时必填。（字符串，可选）
  - `repo`: 仓库名称。当 level 为 'repository' 时必填。（字符串，可选）
  - `rules`：规则集中的规则数组。每条规则是带有 'type'（如 'creation'、'deletion'、'non_fast_forward'、'required_signatures'、'pull_request'、'required_status_checks'）的对象；需要配置的规则还包含 'parameters' 对象（对象数组，必填）
  - `target`：规则集的目标。默认为 'branch'。'repository' 仅对 'organization' 和 'enterprise' 级规则集有效。（字符串，可选）

- **custom_properties_read** - 读取自定义属性
  - **OAuth Challenge Scopes**: `repo`, `read:org`, `read:enterprise`
  - `enterprise`: 企业 slug。当 level 为 'enterprise' 时必填。（字符串，可选）
  - `level`：管理自定义属性的级别：
    - 'repository'：分配给仓库的自定义属性值（需要 'owner' 和 'repo'）。
    - 'organization'：组织的自定义属性定义（模式）（需要 'org'）。
    - 'enterprise'：企业的自定义属性定义（模式）（需要 'enterprise'）。（字符串，必填）
  - `org`: 组织名称。当 level 为 'organization' 时必填。（字符串，可选）
  - `owner`: 仓库所有者。当 level 为 'repository' 时必填。（字符串，可选）
  - `repo`: 仓库名称。当 level 为 'repository' 时必填。（字符串，可选）

- **custom_properties_write** - 设置自定义属性
  - **OAuth Challenge Scopes**: `repo`, `admin:org`, `admin:enterprise`
  - `enterprise`: 企业 slug。当 level 为 'enterprise' 时必填。（字符串，可选）
  - `level`：管理自定义属性的级别：
    - 'repository'：分配给仓库的自定义属性值（需要 'owner' 和 'repo'）。
    - 'organization'：组织的自定义属性定义（模式）（需要 'org'）。
    - 'enterprise'：企业的自定义属性定义（模式）（需要 'enterprise'）。（字符串，必填）
  - `org`: 组织名称。当 level 为 'organization' 时必填。（字符串，可选）
  - `owner`: 仓库所有者。当 level 为 'repository' 时必填。（字符串，可选）
  - `properties`：要创建或更新的自定义属性。在仓库级别，每项分配一个值（'property_name' 和 'value'）；在组织和企业级别，每项定义模式（'property_name' 和 'value_type'，以及可选定义字段）。（对象数组，必填）
  - `repo`: 仓库名称。当 level 为 'repository' 时必填。（字符串，可选）

- **repository_ruleset_read** - 读取仓库规则集
  - **OAuth Challenge Scopes**: `repo`, `read:org`, `read:enterprise`
  - `actor_name`：用于筛选规则套件的 GitHub 用户账户名。'list_rule_suites' 方法使用。（字符串，可选）
  - `branch`：分支名称。'get_rules_for_branch' 方法必填。（字符串，可选）
  - `enterprise`: 企业 slug。当 level 为 'enterprise' 时必填。（字符串，可选）
  - `evaluate_status`：按规则集评估模式筛选规则套件。'list_rule_suites' 方法使用。（字符串，可选）
  - `includes_parents`：包括在更高级别配置且同样适用的规则集。默认为 true。仓库级别的 'get' 和 'list' 方法使用。（布尔值，可选）
  - `level`：配置规则集的级别：
    - 'repository'：单个仓库的规则集（需要 'owner' 和 'repo'）。
    - 'organization'：覆盖组织中各仓库的规则集（需要 'org'）。
    - 'enterprise'：覆盖企业中各仓库的规则集（需要 'enterprise'）。（字符串，必填）
  - `method`：要执行的操作：
    - 'get'：按 ID 获取指定规则集（需要 'ruleset_id'）。所有级别均支持。
    - 'list'：列出所有规则集。所有级别均支持。
    - 'get_rules_for_branch'：获取适用于某分支的所有规则（需要 'branch'）。仅限仓库级别。
    - 'list_rule_suites'：列出规则套件，即针对推送的规则评估。仅限仓库和组织级别。
    - 'get_rule_suite'：按 ID 获取指定规则套件（需要 'rule_suite_id'）。仅限仓库和组织级别。（字符串，必填）
  - `org`: 组织名称。当 level 为 'organization' 时必填。（字符串，可选）
  - `owner`: 仓库所有者。当 level 为 'repository' 时必填。（字符串，可选）
  - `page`: 分页页码（最小 1）（数字，可选）
  - `perPage`: 分页时每页结果数（最小 1，最大 100）（数字，可选）
  - `ref`：用于筛选规则套件的引用名称（分支、标签等）。'list_rule_suites' 方法使用。（字符串，可选）
  - `repo`: 仓库名称。当 level 为 'repository' 时必填。（字符串，可选）
  - `repository_name`：用于筛选规则套件的仓库名称。组织级别的 'list_rule_suites' 方法使用。（字符串，可选）
  - `rule_suite_id`：规则套件 ID。'get_rule_suite' 方法必填。（数字，可选）
  - `rule_suite_result`：用于筛选的规则套件结果。'list_rule_suites' 方法使用。（字符串，可选）
  - `ruleset_id`：规则集 ID。'get' 方法必填。（数字，可选）
  - `time_period`：用于筛选规则套件的时间段。'list_rule_suites' 方法使用。（字符串，可选）

</details>

<details>

<summary><picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/issue-opened-dark.png"><source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/issue-opened-light.png"><img src="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/issue-opened-light.png" width="20" height="20" alt="issue-opened"></picture> 议题</summary>

- **add_issue_comment** - 为议题或拉取请求添加评论
  - **OAuth Challenge Scopes**: `repo`
  - `body`：评论内容。未提供 reaction 时必填。（字符串，可选）
  - `comment_id`：要添加回应的议题或拉取请求评论的数字 ID。用于对评论回应；省略则对议题或拉取请求本身回应。不能与 body 同时使用。（整数，可选）
  - `issue_number`：要评论或回应的议题或拉取请求编号。（数字，必填）
  - `owner`: 仓库所有者（字符串，必填）
  - `reaction`：要添加的表情回应。未提供 body 时必填。（字符串，可选）
  - `repo`: 仓库名称（字符串，必填）

- **get_label** - 从仓库获取指定标签
  - **OAuth Challenge Scopes**: `repo`
  - `name`: 标签名称。（字符串，必填）
  - `owner`: 仓库所有者（用户名或组织名称）（字符串，必填）
  - `repo`: 仓库名称（字符串，必填）

- **issue_read** - 获取议题详情
  - **OAuth Challenge Scopes**: `repo`
  - `issue_number`：议题编号（数字，必填）
  - `method`：要对单个议题执行的读取操作。
    可选值：
    1. get - 获取议题详情。还会尽力返回层级标志（`has_parent`、`has_children`）；`parent` 和 `sub_issues_summary` 为可选关系摘要，`closed_by_pull_requests` 以 `total_count` 和最多 5 个 `references` 汇总配置为关闭该议题的拉取请求。
    2. get_comments - 获取议题评论。
    3. get_sub_issues - 获取议题的子议题。
    4. get_parent - 如果本议题是另一个议题的子议题，获取其父议题。
    5. get_labels - 获取分配给议题的标签。
     （字符串，必填）
  - `owner`：仓库所有者（字符串，必填）
  - `page`: 分页页码（最小 1）（数字，可选）
  - `perPage`: 分页时每页结果数（最小 1，最大 100）（数字，可选）
  - `repo`：仓库名称（字符串，必填）

- **issue_write** - 创建或更新议题/拉取请求
  - **OAuth Challenge Scopes**: `repo`
  - **MCP App UI**: `ui://github-mcp-server/issue-write`
  - `assignees`：要指派给此议题的用户名（字符串数组，可选）
  - `body`：议题正文内容（字符串，可选）
  - `duplicate_of`：本议题所重复的议题编号。state_reason 为 'duplicate' 时必填。（数字，可选）
  - `issue_fields`：要设置或清除的议题字段值。每项需要 'field_name'，以及 'value'、'field_option_name' 或 'delete: true' 中恰好一项。（对象数组，可选）
  - `issue_number`：要更新的议题编号（数字，可选）
  - `labels`：要应用于此议题的标签（字符串数组，可选）
  - `method`：要对单个议题执行的写操作。
    可选值：
    - 'create' - 创建新议题。
    - 'update' - 更新现有议题。
     （字符串，必填）
  - `milestone`：里程碑编号（数字，可选）
  - `owner`: 仓库所有者（字符串，必填）
  - `parent_issue_number`：父议题编号。仅在 method 为 'create' 时使用，且不能与 issue_fields 组合。新议题会在同一操作中创建并附加到该父议题。（数字，可选）
  - `parent_owner`：父议题的仓库所有者。必须与 parent_repo 一同提供。两者均省略时使用 owner 和 repo。仅在 method 为 'create' 且提供 parent_issue_number 时使用。（字符串，可选）
  - `parent_repo`：父议题的仓库名称。必须与 parent_owner 一同提供。两者均省略时使用 owner 和 repo。仅在 method 为 'create' 且提供 parent_issue_number 时使用。（字符串，可选）
  - `repo`: 仓库名称（字符串，必填）
  - `state`: 新状态（字符串，可选）
  - `state_reason`：状态变更原因。未变更 state 时忽略。（字符串，可选）
  - `title`：议题标题（字符串，可选）
  - `type`：本议题的类型。更新时传入 null 可移除当前类型。仅当此仓库启用议题类型时使用。使用 list_issue_types 获取该仓库或其所属组织的有效类型值。如果仓库不支持议题类型，请省略此参数。（字符串 | null，可选）

- **list_issue_fields** - 列出议题字段
  - **OAuth Challenge Scopes**: `repo`, `read:org`
  - `owner`：仓库或组织的账户所有者。名称不区分大小写。（字符串，必填）
  - `repo`：仓库名称。提供时返回此特定仓库的字段（继承自其组织）；省略时直接返回组织级字段。（字符串，可选）

- **list_issue_types** - 列出可用议题类型
  - **OAuth Challenge Scopes**: `repo`, `read:org`
  - `owner`：仓库或组织的账户所有者。（字符串，必填）
  - `repo`：仓库名称。提供时返回此特定仓库的议题类型；省略时直接返回组织级议题类型。（字符串，可选）

- **list_issues** - 列出议题
  - **OAuth Challenge Scopes**: `repo`
  - `after`: 分页游标。使用上一次响应中的游标。（字符串，可选）
  - `direction`：排序方向。提供时还需提供 'orderBy'。（字符串，可选）
  - `field_filters`：按自定义议题字段值筛选。每项包含 field_name 和 value；服务器查找字段并将值转换为其类型（单选选项名称、文本、数字或 YYYY-MM-DD 日期）。（对象数组，可选）
  - `fields`：每个议题要返回的字段子集。省略时返回所有字段。仅需特定字段时可用它减小响应大小；尤其省略 'body' 和 'field_values' 可去掉每条结果中最大的数据。（字符串数组，可选）
  - `labels`：按标签筛选（字符串数组，可选）
  - `orderBy`：按字段排序议题。提供时还需提供 'direction'。（字符串，可选）
  - `owner`: 仓库所有者（字符串，必填）
  - `perPage`: 分页时每页结果数（最小 1，最大 100）（数字，可选）
  - `repo`: 仓库名称（字符串，必填）
  - `since`：按日期筛选（ISO 8601 时间戳）（字符串，可选）
  - `state`：按状态筛选，未提供时默认同时返回打开和关闭的议题（字符串，可选）

- **search_issues** - 搜索议题
  - **OAuth Challenge Scopes**: `repo`
  - `fields`：每个议题结果要返回的字段子集。省略时返回所有字段。仅需特定字段时可用它减小响应大小；尤其省略 'body'、'reactions' 和 'labels' 可去掉每条结果中最大的数据。（字符串数组，可选）
  - `order`: 排序顺序（字符串，可选）
  - `owner`：可选仓库所有者。与 repo 一同提供时，仅列出该仓库的议题。（字符串，可选）
  - `page`: 分页页码（最小 1）（数字，可选）
  - `perPage`: 分页时每页结果数（最小 1，最大 100）（数字，可选）
  - `query`：自然语言搜索查询。当用户给出不同措辞时，将其作为普通词语纳入，不要用 OR 连接。（字符串，必填）
  - `repo`：可选仓库名称。与 owner 一同提供时，仅列出该仓库的议题。（字符串，可选）
  - `sort`：按分类匹配数量排序的字段，默认最佳匹配（字符串，可选）

- **sub_issue_write** - 更改子议题
  - **OAuth Challenge Scopes**: `repo`
  - `after_id`：要排在其后的子议题 ID（应指定 after_id 或 before_id 之一）（数字，可选）
  - `before_id`：要排在其前的子议题 ID（应指定 after_id 或 before_id 之一）（数字，可选）
  - `issue_number`：父议题编号（数字，必填）
  - `method`：要对单个子议题执行的操作
    可选值：
    - 'add' - 向 GitHub 仓库中的父议题添加子议题。
    - 'remove' - 从 GitHub 仓库中的父议题移除子议题。
    - 'reprioritize' - 更改 GitHub 仓库中父议题内子议题的顺序。使用 'after_id' 或 'before_id' 指定新位置。
    写入议题层级。要将子议题移到新的父议题，使用 `add` 并设置 `replace_parent=true`；没有可写的父字段。
     （字符串，必填）
  - `owner`: 仓库所有者（字符串，必填）
  - `replace_parent`：为 true 时替换子议题当前的父议题。仅与 'add' 方法一同使用。（布尔值，可选）
  - `repo`: 仓库名称（字符串，必填）
  - `sub_issue_id`：要添加的子议题 ID。ID 不同于议题编号（数字，必填）

- **update_issue_comment** - 更新议题评论
  - **OAuth Challenge Scopes**: `repo`
  - `body`：新的评论内容（字符串，必填）
  - `comment_id`：要更新的议题或拉取请求会话评论的数字 ID。不要使用拉取请求审查评论 ID。（整数，必填）
  - `owner`: 仓库所有者（字符串，必填）
  - `repo`: 仓库名称（字符串，必填）

</details>

<details>

<summary><picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/tag-dark.png"><source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/tag-light.png"><img src="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/tag-light.png" width="20" height="20" alt="tag"></picture> 标签</summary>

- **get_label** - 从仓库获取指定标签
  - **OAuth Challenge Scopes**: `repo`
  - `name`: 标签名称。（字符串，必填）
  - `owner`: 仓库所有者（用户名或组织名称）（字符串，必填）
  - `repo`: 仓库名称（字符串，必填）

- **label_write** - 对仓库标签执行写操作
  - **OAuth Challenge Scopes**: `repo`
  - `color`：不带 '#' 前缀的 6 位十六进制标签颜色（例如 'f29513'）。'create' 必填，'update' 可选。（字符串，可选）
  - `description`：标签描述文本。'create' 和 'update' 均可选。（字符串，可选）
  - `method`：要执行的操作：'create'、'update' 或 'delete'（字符串，必填）
  - `name`：标签名称，所有操作均必填（字符串，必填）
  - `new_name`：标签新名称（仅在 'update' 方法中用于重命名）（字符串，可选）
  - `owner`: 仓库所有者（用户名或组织名称）（字符串，必填）
  - `repo`: 仓库名称（字符串，必填）

- **list_label** - 列出仓库标签
  - **OAuth Challenge Scopes**: `repo`
  - `after`: 分页游标。使用上一次响应中的游标。（字符串，可选）
  - `owner`：仓库所有者（用户名或组织名称），所有操作均必填（字符串，必填）
  - `perPage`: 分页时每页结果数（最小 1，最大 100）（数字，可选）
  - `repo`：仓库名称，所有操作均必填（字符串，必填）

</details>

<details>

<summary><picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/bell-dark.png"><source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/bell-light.png"><img src="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/bell-light.png" width="20" height="20" alt="bell"></picture> 通知</summary>

- **dismiss_notification** - 处理通知
  - **OAuth Challenge Scopes**: `notifications`
  - `state`：通知的新状态（read/done）（字符串，必填）
  - `threadID`：通知线程 ID（字符串，必填）

- **get_notification_details** - 获取通知详情
  - **OAuth Challenge Scopes**: `notifications`
  - `notificationID`：通知 ID（字符串，必填）

- **list_notifications** - 列出通知
  - **OAuth Challenge Scopes**: `notifications`
  - `before`：仅显示给定时间之前更新的通知（ISO 8601 格式）（字符串，可选）
  - `filter`：通知筛选条件，未指定时使用默认值。已读通知是用户已确认的通知。参与通知是用户直接参与的通知，如用户评论或创建的议题或拉取请求。（字符串，可选）
  - `owner`：可选仓库所有者。与 repo 一同提供时，仅列出该仓库的通知。（字符串，可选）
  - `page`: 分页页码（最小 1）（数字，可选）
  - `perPage`: 分页时每页结果数（最小 1，最大 100）（数字，可选）
  - `repo`：可选仓库名称。与 owner 一同提供时，仅列出该仓库的通知。（字符串，可选）
  - `since`：仅显示给定时间之后更新的通知（ISO 8601 格式）（字符串，可选）

- **manage_notification_subscription** - 管理通知订阅
  - **OAuth Challenge Scopes**: `notifications`
  - `action`：要执行的操作：ignore、watch 或 delete 通知订阅。（字符串，必填）
  - `notificationID`：通知线程 ID。（字符串，必填）

- **manage_repository_notification_subscription** - 管理仓库通知订阅
  - **OAuth Challenge Scopes**: `notifications`
  - `action`：要执行的操作：ignore、watch 或 delete 仓库通知订阅。（字符串，必填）
  - `owner`：仓库账户所有者。（字符串，必填）
  - `repo`: 仓库名称。（字符串，必填）

- **mark_all_notifications_read** - 将所有通知标记为已读
  - **OAuth Challenge Scopes**: `notifications`
  - `lastReadAt`：上次检查通知的时间点（可选）。默认：现在（字符串，可选）
  - `owner`：可选仓库所有者。与 repo 一同提供时，仅将该仓库通知标记为已读。（字符串，可选）
  - `repo`：可选仓库名称。与 owner 一同提供时，仅将该仓库通知标记为已读。（字符串，可选）

</details>

<details>

<summary><picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/organization-dark.png"><source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/organization-light.png"><img src="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/organization-light.png" width="20" height="20" alt="organization"></picture> 组织</summary>

- **search_orgs** - 搜索组织
  - **OAuth Challenge Scopes**: `read:org`
  - `order`: 排序顺序（字符串，可选）
  - `page`: 分页页码（最小 1）（数字，可选）
  - `perPage`: 分页时每页结果数（最小 1，最大 100）（数字，可选）
  - `query`：组织搜索查询。例如：'microsoft'、'location:california'、'created:>=2025-01-01'。搜索会自动限定为 type:org。（字符串，必填）
  - `sort`：按分类排序的字段（字符串，可选）

</details>

<details>

<summary><picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/project-dark.png"><source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/project-light.png"><img src="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/project-light.png" width="20" height="20" alt="project"></picture> 项目</summary>

- **projects_get** - 获取 GitHub Projects 资源详情
  - **OAuth Challenge Scopes**: `read:project`
  - `field_id`：字段 ID。'get_project_field' 方法必填。（数字，可选）
  - `field_names`：获取项目条目时响应中包含的特定字段名称列表（例如 ["Status", "Priority"]）。服务器端会将名称解析为字段 ID；只知道人类可读名称时使用它代替 'fields'。与 'fields' 互斥，只能提供其一。仅用于 'get_project_item' 方法。（字符串数组，可选）
  - `fields`：获取项目条目时响应中包含的特定字段 ID 列表（例如 ["102589", "985201", "169875"]）。若 'fields' 和 'field_names' 均未提供，则只包含标题字段。与 'field_names' 互斥，只能提供其一。仅用于 'get_project_item' 方法。（字符串数组，可选）
  - `item_id`：条目 ID。'get_project_item' 方法必填。（数字，可选）
  - `method`: 要执行的方法（字符串，必填）
  - `owner`：所有者（用户或组织登录名）。名称不区分大小写。（字符串，可选）
  - `owner_type`：所有者类型（user 或 org）。未提供时自动检测。（字符串，可选）
  - `project_number`：项目编号。（数字，可选）
  - `status_update_id`：项目状态更新的节点 ID。'get_project_status_update' 方法必填。（字符串，可选）
  - `view_id`：项目视图的节点 ID。'get_project_view' 方法必填。（字符串，可选）

- **projects_list** - 列出 GitHub Projects 资源
  - **OAuth Challenge Scopes**: `read:project`
  - `after`：上一页 pageInfo.nextCursor 的向前分页游标。（字符串，可选）
  - `before`：上一页 pageInfo.prevCursor 的向后分页游标（少见）。（字符串，可选）
  - `field_names`：列出项目条目时包含的字段名称（例如 ["Status", "Priority"]）。服务器端会将名称解析为字段 ID；只知道人类可读名称时使用它代替 'fields'。无法解析的名称返回结构化错误。与 'fields' 互斥，只能提供其一。仅用于 'list_project_items' 方法。（字符串数组，可选）
  - `fields`：列出项目条目时包含的字段 ID（例如 ["102589", "985201"]）。关键：始终提供此参数以获取字段值。没有它（也没有 'field_names'）时只返回标题。与 'field_names' 互斥，只能提供其一。仅用于 'list_project_items' 方法。（字符串数组，可选）
  - `method`: 要执行的操作（字符串，必填）
  - `owner`：所有者（用户或组织登录名）。名称不区分大小写。（字符串，必填）
  - `owner_type`：所有者类型（user 或 org）。未提供时自动尝试两者。（字符串，可选）
  - `perPage`：每页结果数（最大 50）（数字，可选）
  - `project_number`：项目编号。'list_project_fields'、'list_project_items'、'list_project_views' 和 'list_project_status_updates' 方法必填。（数字，可选）
  - `query`：筛选/查询字符串。对于 list_projects：按标题文本和状态筛选（例如 "roadmap is:open"）。对于 list_project_items：使用 GitHub 项目筛选语法进行高级筛选。（字符串，可选）

- **projects_write** - 管理 GitHub Projects
  - **OAuth Challenge Scopes**: `project`
  - `body`：状态更新的正文（markdown）。用于 'create_project_status_update' 方法。（字符串，可选）
  - `field_name`：迭代字段名称（例如 'Sprint'）。'create_iteration_field' 方法必填。（字符串，可选）
  - `filter`：保存视图的筛选条件；更新时省略以保留，或传入 null 清除。（字符串 | null，可选）
  - `issue_number`：议题编号。当 item_type 为 'issue' 时，'add_project_item' 必填。'update_project_item' 也接受此参数以按议题编号解析条目（与 item_owner 和 item_repo 组合）。（数字，可选）
  - `item_id`：项目条目 ID。'delete_project_item' 必填。对于 'update_project_item'，提供 item_id 或（item_owner + item_repo + issue_number）以按议题解析条目。（数字，可选）
  - `item_owner`：包含议题或拉取请求的仓库所有者（用户或组织）。'add_project_item' 方法必填。'update_project_item' 按议题编号解析条目时也接受此参数。（字符串，可选）
  - `item_repo`：包含议题或拉取请求的仓库名称。'add_project_item' 方法必填。'update_project_item' 按议题编号解析条目时也接受此参数。（字符串，可选）
  - `item_type`：条目类型，issue 或 pull_request。'add_project_item' 方法必填。（字符串，可选）
  - `items`：使用顶层 'updated_field' 更新的条目。'update_project_items' 必填；优先使用它，而不是循环调用 'update_project_item'。每项必须恰好匹配一种引用形式：'node_id'、数字 'item_id' 或 'item_owner' + 'item_repo' + 'issue_number'。每次调用最多 50 项。（对象数组，可选）
  - `iteration_duration`：该字段各迭代的持续天数（例如每周为 7、每两周为 14）。'create_iteration_field' 方法必填。（数字，可选）
  - `iterations`：'create_iteration_field' 方法的自定义迭代。仅在需要不同时长、迭代间隔或特定标题时设置。否则省略：GitHub 会从 'start_date' 开始自动创建三个持续 'iteration_duration' 天的迭代，这适合大多数情况。（对象数组，可选）
  - `layout`：视图布局；创建视图时必填。（字符串，可选）
  - `method`: 要执行的方法（字符串，必填）
  - `name`：视图名称；创建视图时必填。（字符串，可选）
  - `owner`：项目所有者（用户或组织登录名）。名称不区分大小写。（字符串，必填）
  - `owner_type`：所有者类型（user 或 org）。'create_project' 方法必填；其他方法未提供时自动检测。（字符串，可选）
  - `project_number`：项目编号。除 'create_project' 外所有方法必填。（数字，可选）
  - `pull_request_number`：拉取请求编号（'add_project_item' 方法中 item_type 为 'pull_request' 时使用）。提供 issue_number 或 pull_request_number 之一。（数字，可选）
  - `start_date`：YYYY-MM-DD 格式的开始日期。用于 'create_project_status_update' 和 'create_iteration_field' 方法。（字符串，可选）
  - `status`：项目状态。用于 'create_project_status_update' 方法。（字符串，可选）
  - `target_date`：状态更新的目标日期，格式为 YYYY-MM-DD。用于 'create_project_status_update' 方法。（字符串，可选）
  - `title`：项目标题。'create_project' 方法必填。（字符串，可选）
  - `updated_field`：要应用的字段/值，使用 {"id": 123, "value": ...} 或 {"name": "Status", "value": ...}；null 清除字段。'update_project_item' 和 'update_project_items' 必填，其中一个顶层字段/值适用于批次中的每个条目。对于 'update_project_item' 的 SINGLE_SELECT 字段，名称形式接受选项名称；ID 形式要求选项 ID。（对象，可选）
  - `view_id`：用于更新或删除的项目视图节点 ID；必须属于 owner/project_number。（字符串，可选）
  - `visible_field_names`：创建时显示或更新时替换的有序项目字段名称；更新时省略以保留，或传入 [] 重置。与 visible_fields 互斥。Roadmap 仅接受 []。（字符串数组，可选）
  - `visible_fields`：创建时显示或更新时替换的有序项目字段数据库 ID；更新时省略以保留，或传入 [] 重置。与 visible_field_names 互斥。Roadmap 仅接受 []。（字符串数组，可选）

</details>

<details>

<summary><picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/git-pull-request-dark.png"><source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/git-pull-request-light.png"><img src="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/git-pull-request-light.png" width="20" height="20" alt="git-pull-request"></picture> 拉取请求</summary>

- **add_comment_to_pending_review** - 为请求者最新的待处理拉取请求审查添加评论
  - **OAuth Challenge Scopes**: `repo`
  - `body`：审查评论文本（字符串，必填）
  - `line`：拉取请求差异中评论所针对的 blob 行号。多行评论时为范围的最后一行（数字，可选）
  - `owner`: 仓库所有者（字符串，必填）
  - `path`：需要评论的文件相对路径（字符串，必填）
  - `pullNumber`: 拉取请求编号（数字，必填）
  - `repo`: 仓库名称（字符串，必填）
  - `side`：评论所在的差异侧。LEFT 表示之前状态，RIGHT 表示新状态（字符串，可选）
  - `startLine`：多行评论中评论范围的第一行（数字，可选）
  - `startSide`：多行评论中评论范围的起始差异侧。LEFT 表示之前状态，RIGHT 表示新状态（字符串，可选）
  - `subjectType`：评论针对的级别（字符串，必填）

- **add_reply_to_pull_request_comment** - 回复拉取请求评论
  - **OAuth Challenge Scopes**: `repo`
  - `body`：回复文本。未提供 reaction 时必填。（字符串，可选）
  - `commentId`：要回复或回应的拉取请求审查评论数字 ID。使用 #discussion_r... 锚点中的数字，而非 GraphQL 线程节点 ID（PRRT_...）。（数字，必填）
  - `owner`: 仓库所有者（字符串，必填）
  - `pullNumber`：拉取请求编号。提供 body 时必填。（数字，可选）
  - `reaction`：要添加的表情回应。未提供 body 时必填。（字符串，可选）
  - `repo`: 仓库名称（字符串，必填）

- **create_pull_request** - 打开新的拉取请求
  - **OAuth Challenge Scopes**: `repo`
  - **MCP App UI**: `ui://github-mcp-server/pr-write`
  - `base`：要合并到的分支（字符串，必填）
  - `body`：PR 说明（字符串，可选）
  - `draft`：创建为草稿 PR（布尔值，可选）
  - `head`：包含更改的分支（字符串，必填）
  - `maintainer_can_modify`: 允许维护者编辑（布尔值，可选）
  - `owner`: 仓库所有者（字符串，必填）
  - `repo`: 仓库名称（字符串，必填）
  - `reviewers`：要请求审查的 GitHub 用户名或 ORG/team-slug 团队审查者（字符串数组，可选）
  - `title`：PR 标题（字符串，必填）

- **list_pull_requests** - 列出拉取请求
  - **OAuth Challenge Scopes**: `repo`
  - `base`：按目标分支筛选（字符串，可选）
  - `direction`：排序方向（字符串，可选）
  - `fields`：每个拉取请求要返回的字段子集。省略时返回所有字段。仅需特定字段时可用它减小响应大小；尤其省略 'body' 可去掉每条结果中最大的数据。（字符串数组，可选）
  - `head`：按来源用户/组织和分支筛选（字符串，可选）
  - `owner`: 仓库所有者（字符串，必填）
  - `page`: 分页页码（最小 1）（数字，可选）
  - `perPage`: 分页时每页结果数（最小 1，最大 100）（数字，可选）
  - `repo`: 仓库名称（字符串，必填）
  - `sort`：排序依据（字符串，可选）
  - `state`: 按状态筛选（字符串，可选）

- **merge_pull_request** - 合并拉取请求
  - **OAuth Challenge Scopes**: `repo`
  - `commit_message`：合并提交的额外详情（字符串，可选）
  - `commit_title`：合并提交标题（字符串，可选）
  - `expectedHeadSha`: 拉取请求 HEAD 引用的预期 SHA（字符串，可选）
  - `merge_method`：合并方法（字符串，可选）
  - `owner`: 仓库所有者（字符串，必填）
  - `pullNumber`: 拉取请求编号（数字，必填）
  - `repo`: 仓库名称（字符串，必填）

- **pull_request_read** - 获取单个拉取请求的详情
  - **OAuth Challenge Scopes**: `repo`
  - `after`：分页游标，仅供 get_review_comments 方法使用。传入上一页 PageInfo 中的 endCursor 以获取下一页。（字符串，可选）
  - `method`：指定需要从 GitHub 获取哪些拉取请求数据的操作。
    可选项：
     1. get - 获取指定拉取请求的详情。
     2. get_diff - 获取拉取请求的差异。
     3. get_status - 获取拉取请求中 head 提交的合并提交状态。
     4. get_files - 获取拉取请求中变更的文件列表。可使用分页参数控制返回结果数量。
     5. get_commits - 获取拉取请求中的提交列表。可使用分页参数控制返回结果数量。
     6. get_review_comments - 获取拉取请求上的审查线程。每个线程包含拉取请求审查期间针对同一代码位置发表并按逻辑分组的审查评论。返回线程元数据和带可空当前及原始行范围坐标（line、start_line、original_line、original_start_line）的评论。当前坐标不可用时会省略，例如过时评论。使用基于游标的分页（perPage、after）控制结果。
     7. get_reviews - 获取拉取请求上的审查。若请求审查评论，请使用 get_review_comments 方法。可使用分页参数控制返回结果数量。
     8. get_comments - 获取拉取请求上的评论。用户没有明确要求审查评论时使用。可使用分页参数控制返回结果数量。
     9. get_check_runs - 获取拉取请求 head 提交的检查运行。检查运行是 PR 上运行的各个 CI/CD 任务和检查。
     （字符串，必填）
  - `owner`: 仓库所有者（字符串，必填）
  - `page`: 分页页码（最小 1）（数字，可选）
  - `perPage`: 分页时每页结果数（最小 1，最大 100）（数字，可选）
  - `pullNumber`: 拉取请求编号（数字，必填）
  - `repo`: 仓库名称（字符串，必填）

- **pull_request_review_write** - 对拉取请求审查执行写操作（创建、提交、删除）
  - **OAuth Challenge Scopes**: `repo`
  - `body`：审查评论文本（字符串，可选）
  - `commitID`：要审查的提交 SHA（字符串，可选）
  - `event`：要执行的审查操作。（字符串，可选）
  - `method`：要对拉取请求审查执行的写操作。（字符串，必填）
  - `owner`: 仓库所有者（字符串，必填）
  - `pullNumber`: 拉取请求编号（数字，必填）
  - `repo`: 仓库名称（字符串，必填）
  - `threadId`：审查线程的节点 ID（例如 PRRT_kwDOxxx）。resolve_thread 和 unresolve_thread 方法必填。通过 pull_request_read 的 get_review_comments 方法获取线程 ID。（字符串，可选）

- **search_pull_requests** - 搜索拉取请求
  - **OAuth Challenge Scopes**: `repo`
  - `fields`：每个拉取请求结果要返回的字段子集。省略时返回所有字段。仅需特定字段时可用它减小响应大小；尤其省略 'body'、'reactions' 和 'labels' 可去掉每条结果中最大的数据。（字符串数组，可选）
  - `order`: 排序顺序（字符串，可选）
  - `owner`：可选仓库所有者。与 repo 一同提供时，仅列出该仓库的拉取请求。（字符串，可选）
  - `page`: 分页页码（最小 1）（数字，可选）
  - `perPage`: 分页时每页结果数（最小 1，最大 100）（数字，可选）
  - `query`：使用 GitHub 拉取请求搜索语法的查询（字符串，必填）
  - `repo`：可选仓库名称。与 owner 一同提供时，仅列出该仓库的拉取请求。（字符串，可选）
  - `sort`：按分类匹配数量排序的字段，默认最佳匹配（字符串，可选）

- **update_pull_request** - 编辑拉取请求
  - **OAuth Challenge Scopes**: `repo`
  - **MCP App UI**: `ui://github-mcp-server/pr-edit`
  - `base`：新的目标分支名称（字符串，可选）
  - `body`：新的说明（字符串，可选）
  - `draft`：将拉取请求标记为草稿（true）或准备审查（false）（布尔值，可选）
  - `maintainer_can_modify`: 允许维护者编辑（布尔值，可选）
  - `owner`: 仓库所有者（字符串，必填）
  - `pullNumber`：要更新的拉取请求编号（数字，必填）
  - `repo`: 仓库名称（字符串，必填）
  - `reviewers`：要请求审查的 GitHub 用户名或 ORG/team-slug 团队审查者（字符串数组，可选）
  - `state`: 新状态（字符串，可选）
  - `title`：新标题（字符串，可选）

- **update_pull_request_branch** - 更新拉取请求分支
  - **OAuth Challenge Scopes**: `repo`
  - `expectedHeadSha`: 拉取请求 HEAD 引用的预期 SHA（字符串，可选）
  - `owner`: 仓库所有者（字符串，必填）
  - `pullNumber`: 拉取请求编号（数字，必填）
  - `repo`: 仓库名称（字符串，必填）

</details>

<details>

<summary><picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/repo-dark.png"><source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/repo-light.png"><img src="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/repo-light.png" width="20" height="20" alt="repo"></picture> 仓库</summary>

- **create_branch** - 创建分支
  - **OAuth Challenge Scopes**: `repo`
  - `branch`：新分支名称（字符串，必填）
  - `from_branch`：来源分支（默认为仓库默认分支）（字符串，可选）
  - `owner`: 仓库所有者（字符串，必填）
  - `repo`: 仓库名称（字符串，必填）

- **create_or_update_file** - 创建或更新文件
  - **OAuth Challenge Scopes**: `repo`, `workflow`
  - `allow_symlink_write`：设为 true 可更新符号链接本身；content 必须是其新的目标路径。（布尔值，可选）
  - `branch`：创建/更新文件所在的分支（字符串，必填）
  - `content`：文件内容，与写入后的预期内容完全一致。不要进行 base64 编码；服务器会在调用 REST API 前进行编码。（字符串，必填）
  - `message`: 提交消息（字符串，必填）
  - `owner`: 仓库所有者（用户名或组织）（字符串，必填）
  - `path`：创建/更新文件的路径（字符串，必填）
  - `repo`: 仓库名称（字符串，必填）
  - `sha`：被替换文件的 blob SHA。如果文件已存在则必填。使用相同 owner、repo 和 path 调用 get_file_contents，并将 ref 设置为此工具的 branch 值，以获取该 SHA。（字符串，可选）

- **create_repository** - 创建仓库
  - **OAuth Challenge Scopes**: `repo`
  - `autoInit`：使用 README 初始化（布尔值，可选）
  - `description`：仓库说明（字符串，可选）
  - `name`: 仓库名称（字符串，必填）
  - `organization`：创建仓库所在的组织（省略则创建在个人账户中）（字符串，可选）
  - `private`：仓库是否私有。省略时默认为 true（私有）。（布尔值，可选）

- **delete_file** - 删除文件
  - **OAuth Challenge Scopes**: `repo`, `workflow`
  - `branch`：删除文件所在的分支（字符串，必填）
  - `message`: 提交消息（字符串，必填）
  - `owner`: 仓库所有者（用户名或组织）（字符串，必填）
  - `path`：要删除的文件路径（字符串，必填）
  - `repo`: 仓库名称（字符串，必填）

- **delete_repository** - 删除仓库
  - **OAuth Challenge Scopes**: `delete_repo`, `repo`
  - `owner`: 仓库所有者（用户名或组织）（字符串，必填）
  - `repo`: 仓库名称（字符串，必填）

- **fork_repository** - 派生仓库
  - **OAuth Challenge Scopes**: `repo`
  - `organization`：要派生到的组织（字符串，可选）
  - `owner`: 仓库所有者（字符串，必填）
  - `repo`: 仓库名称（字符串，必填）

- **get_commit** - 获取提交详情
  - **OAuth Challenge Scopes**: `repo`
  - `detail`：变更文件包含的详细程度。"none" 完全省略统计和文件；"stats"（默认）包括每个文件的元数据：文件名、状态及代码行数（新增、删除、变更），不含补丁内容；"full_patch" 还包括每个文件的统一差异内容，可能非常大。（字符串，可选）
  - `owner`: 仓库所有者（字符串，必填）
  - `page`: 分页页码（最小 1）（数字，可选）
  - `perPage`: 分页时每页结果数（最小 1，最大 100）（数字，可选）
  - `repo`: 仓库名称（字符串，必填）
  - `sha`：提交 SHA、分支名称或标签名称（字符串，必填）

- **get_file_contents** - 获取文件或目录内容
  - **OAuth Challenge Scopes**: `repo`
  - `fields`：路径为目录时每个条目要返回的字段子集。省略时返回所有字段；路径为单个文件时忽略。列出目录且仅需特定字段（如 'name' 和 'type'）时，可用它减小响应大小。（字符串数组，可选）
  - `owner`: 仓库所有者（用户名或组织）（字符串，必填）
  - `path`：文件/目录路径（字符串，可选）
  - `ref`：接受可选 git 引用，例如 `refs/tags/{tag}`、`refs/heads/{branch}` 或 `refs/pull/{pr_number}/head`（字符串，可选）
  - `repo`: 仓库名称（字符串，必填）
  - `sha`：接受可选提交 SHA。指定时将替代 ref（字符串，可选）

- **get_latest_release** - 获取最新发布
  - **OAuth Challenge Scopes**: `repo`
  - `owner`: 仓库所有者（字符串，必填）
  - `repo`: 仓库名称（字符串，必填）

- **get_release_by_tag** - 按标签名称获取发布
  - **OAuth Challenge Scopes**: `repo`
  - `owner`: 仓库所有者（字符串，必填）
  - `repo`: 仓库名称（字符串，必填）
  - `tag`：标签名称（例如 'v1.0.0'）（字符串，必填）

- **get_tag** - 获取标签详情
  - **OAuth Challenge Scopes**: `repo`
  - `owner`: 仓库所有者（字符串，必填）
  - `repo`: 仓库名称（字符串，必填）
  - `tag`：标签名称（字符串，必填）

- **list_branches** - 列出分支
  - **OAuth Challenge Scopes**: `repo`
  - `owner`: 仓库所有者（字符串，必填）
  - `page`: 分页页码（最小 1）（数字，可选）
  - `perPage`: 分页时每页结果数（最小 1，最大 100）（数字，可选）
  - `repo`: 仓库名称（字符串，必填）

- **list_commits** - 列出提交
  - **OAuth Challenge Scopes**: `repo`
  - `author`：用于筛选提交的作者用户名或电子邮件地址（字符串，可选）
  - `fields`：每次提交要返回的字段子集。省略时返回所有字段。仅需特定字段（如 'sha' 和 'html_url'）时，可用它减小响应大小。（字符串数组，可选）
  - `owner`: 仓库所有者（字符串，必填）
  - `page`: 分页页码（最小 1）（数字，可选）
  - `path`：仅返回包含此文件路径的提交（字符串，可选）
  - `perPage`: 分页时每页结果数（最小 1，最大 100）（数字，可选）
  - `repo`: 仓库名称（字符串，必填）
  - `sha`：要列出其提交的提交 SHA、分支或标签名称。未提供时使用仓库默认分支。提供提交 SHA 时列出直至该 SHA 的提交。（字符串，可选）
  - `since`：仅返回此日期之后的提交（ISO 8601 格式：YYYY-MM-DDTHH:MM:SSZ 或 YYYY-MM-DD）（字符串，可选）
  - `until`：仅返回此日期之前的提交（ISO 8601 格式：YYYY-MM-DDTHH:MM:SSZ 或 YYYY-MM-DD）（字符串，可选）

- **list_releases** - 列出发布
  - **OAuth Challenge Scopes**: `repo`
  - `fields`：每个发布要返回的字段子集。省略时返回所有字段。仅需特定字段时可用它减小响应大小；尤其省略 'body' 可去掉每个发布中最大的数据。（字符串数组，可选）
  - `owner`: 仓库所有者（字符串，必填）
  - `page`: 分页页码（最小 1）（数字，可选）
  - `perPage`: 分页时每页结果数（最小 1，最大 100）（数字，可选）
  - `repo`: 仓库名称（字符串，必填）

- **list_repository_collaborators** - 列出仓库协作者
  - **OAuth Challenge Scopes**: `repo`
  - `affiliation`：按关联关系筛选。可为 'outside'（外部协作者）、'direct'（拥有权限的所有用户，无论组织成员身份）、'all'（所有协作者）。默认：'all'（字符串，可选）
  - `owner`: 仓库所有者（字符串，必填）
  - `page`：分页页码（默认 1，最小 1）（数字，可选）
  - `perPage`：分页时每页结果数（默认 30，最小 1，最大 100）（数字，可选）
  - `repo`: 仓库名称（字符串，必填）

- **list_tags** - 列出标签
  - **OAuth Challenge Scopes**: `repo`
  - `owner`: 仓库所有者（字符串，必填）
  - `page`: 分页页码（最小 1）（数字，可选）
  - `perPage`: 分页时每页结果数（最小 1，最大 100）（数字，可选）
  - `repo`: 仓库名称（字符串，必填）

- **push_files** - 向仓库推送文件
  - **OAuth Challenge Scopes**: `repo`, `workflow`
  - `branch`：要推送到的分支（字符串，必填）
  - `files`：要推送的文件对象数组，每个对象含 path（字符串）和 content（字符串）（对象数组，必填）
  - `message`: 提交消息（字符串，必填）
  - `owner`: 仓库所有者（字符串，必填）
  - `repo`: 仓库名称（字符串，必填）

- **search_code** - 搜索代码
  - **OAuth Challenge Scopes**: `repo`
  - `fields`：每条代码搜索结果要返回的字段子集。省略时返回所有字段。仅需特定字段时可用它减小响应大小；尤其省略 'repository' 和 'text_matches' 可去掉每条结果中最大的数据。（字符串数组，可选）
  - `order`：结果排序顺序（字符串，可选）
  - `page`: 分页页码（最小 1）（数字，可选）
  - `perPage`: 分页时每页结果数（最小 1，最大 100）（数字，可选）
  - `query`：搜索查询（GitHub 代码搜索 REST）。词项之间隐含 AND；支持 `OR`、`NOT` 和用于精确匹配的 `"quoted phrase"`。限定符：`repo:owner/repo`、`org:`、`user:`、`language:`、`path:dir`（前缀匹配）、`filename:exact.ext`、`extension:`、`in:file`、`in:path`、`size:`、`is:archived`、`is:fork`。最多 256 个字符。示例：`WithContext language:go org:github`；`"package main" repo:o/r`；`func extension:go path:cmd repo:o/r`；`NOT TODO language:go repo:o/r`。（字符串，必填）
  - `sort`：排序字段（仅 'indexed'）（字符串，可选）

- **search_commits** - 搜索提交
  - **OAuth Challenge Scopes**: `repo`
  - `order`: 排序顺序（字符串，可选）
  - `page`: 分页页码（最小 1）（数字，可选）
  - `perPage`: 分页时每页结果数（最小 1，最大 100）（数字，可选）
  - `query`：提交搜索查询（GitHub 提交搜索 REST）。仅搜索默认分支上的提交消息。使用 `repo:owner/repo`、`org:` 或 `user:` 限定搜索范围（没有范围限定符的查询会匹配整个 GitHub，通常不是所需结果）。其他限定符：`author:`、`committer:`、`author-name:`、`committer-name:`、`author-email:`、`committer-email:`、`author-date:`、`committer-date:`（支持 `>`、`<`、`>=`、`<=` 和 `YYYY-MM-DD..YYYY-MM-DD` 范围）、`merge:true|false`、`hash:`、`tree:`、`parent:`、`is:public`。示例：`repo:owner/repo fix panic`；`org:github author:defunkt committer-date:>=2024-01-01`；`"refactor cache" repo:o/r`；`hash:abc1234 repo:o/r`。（字符串，必填）
  - `sort`：按作者或提交者日期排序（默认最佳匹配）（字符串，可选）

- **search_repositories** - 搜索仓库
  - **OAuth Challenge Scopes**: `repo`
  - `minimal_output`：返回最少仓库信息（默认：true）。为 false 时，新版客户端收到额外精选的仓库详情，而非完整 GitHub API 对象。旧版客户端保留完整 GitHub API 仓库对象。（布尔值，可选）
  - `order`: 排序顺序（字符串，可选）
  - `page`: 分页页码（最小 1）（数字，可选）
  - `perPage`: 分页时每页结果数（最小 1，最大 100）（数字，可选）
  - `query`：仓库搜索查询。例如：'machine learning in:name stars:>1000 language:python'、'topic:react'、'user:facebook'。支持高级搜索语法进行精确筛选。（字符串，必填）
  - `sort`：仓库排序字段，默认最佳匹配（字符串，可选）

</details>

<details>

<summary><picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/shield-lock-dark.png"><source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/shield-lock-light.png"><img src="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/shield-lock-light.png" width="20" height="20" alt="shield-lock"></picture> 密钥保护</summary>

- **get_secret_scanning_alert** - 获取密钥扫描警报
  - **OAuth Challenge Scopes**: `security_events`
  - `alertNumber`: 警报编号。（数字，必填）
  - `owner`: 仓库所有者。（字符串，必填）
  - `repo`: 仓库名称。（字符串，必填）

- **list_secret_scanning_alerts** - 列出密钥扫描警报
  - **OAuth Challenge Scopes**: `security_events`
  - `owner`: 仓库所有者。（字符串，必填）
  - `page`: 分页页码（最小 1）（数字，可选）
  - `perPage`: 分页时每页结果数（最小 1，最大 100）（数字，可选）
  - `repo`: 仓库名称。（字符串，必填）
  - `resolution`：按处理结果筛选（字符串，可选）
  - `secret_type`：要返回的密钥类型列表，以逗号分隔。默认返回所有默认密钥模式。要返回通用模式，请在参数中传入令牌名称。（字符串，可选）
  - `state`: 按状态筛选（字符串，可选）

</details>

<details>

<summary><picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/shield-dark.png"><source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/shield-light.png"><img src="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/shield-light.png" width="20" height="20" alt="shield"></picture> 安全公告</summary>

- **get_global_security_advisory** - 获取全局安全公告
  - **OAuth Challenge Scopes**: `security_events`
  - `ghsaId`：GitHub 安全公告 ID（格式：GHSA-xxxx-xxxx-xxxx）。（字符串，必填）

- **list_global_security_advisories** - 列出全局安全公告
  - **OAuth Challenge Scopes**: `security_events`
  - `affects`：按受影响的包或版本筛选公告（例如 "package1,package2@1.0.0"）。（字符串，可选）
  - `cveId`：按 CVE ID 筛选。（字符串，可选）
  - `cwes`：按通用弱点枚举 ID 筛选（例如 ["79", "284", "22"]）。（字符串数组，可选）
  - `ecosystem`：按包生态系统筛选。（字符串，可选）
  - `ghsaId`：按 GitHub 安全公告 ID 筛选（格式：GHSA-xxxx-xxxx-xxxx）。（字符串，可选）
  - `isWithdrawn`：是否只返回已撤回的公告。（布尔值，可选）
  - `modified`：按发布或更新日期或日期范围筛选（ISO 8601 日期或范围）。（字符串，可选）
  - `published`：按发布日期或日期范围筛选（ISO 8601 日期或范围）。（字符串，可选）
  - `severity`：按严重程度筛选。（字符串，可选）
  - `type`：公告类型。（字符串，可选）
  - `updated`：按更新日期或日期范围筛选（ISO 8601 日期或范围）。（字符串，可选）

- **list_org_repository_security_advisories** - 列出组织仓库安全公告
  - **OAuth Challenge Scopes**: `security_events`
  - `direction`: 排序方向。（字符串，可选）
  - `org`：组织登录名。（字符串，必填）
  - `sort`: 排序字段。（字符串，可选）
  - `state`：按公告状态筛选。（字符串，可选）

- **list_repository_security_advisories** - 列出仓库安全公告
  - **OAuth Challenge Scopes**: `security_events`
  - `direction`: 排序方向。（字符串，可选）
  - `owner`: 仓库所有者。（字符串，必填）
  - `repo`: 仓库名称。（字符串，必填）
  - `sort`: 排序字段。（字符串，可选）
  - `state`：按公告状态筛选。（字符串，可选）

</details>

<details>

<summary><picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/star-dark.png"><source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/star-light.png"><img src="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/star-light.png" width="20" height="20" alt="star"></picture> 加星用户</summary>

- **list_starred_repositories** - 列出已加星仓库
  - **OAuth Challenge Scopes**: `repo`
  - `direction`：结果排序方向。（字符串，可选）
  - `page`: 分页页码（最小 1）（数字，可选）
  - `perPage`: 分页时每页结果数（最小 1，最大 100）（数字，可选）
  - `sort`：结果排序方式。可为 'created'（仓库被加星的时间）或 'updated'（仓库上次推送的时间）。（字符串，可选）
  - `username`：要列出其已加星仓库的用户名。默认为已认证用户。（字符串，可选）

- **star_repository** - 给仓库加星
  - **OAuth Challenge Scopes**: `repo`
  - `owner`: 仓库所有者（字符串，必填）
  - `repo`: 仓库名称（字符串，必填）

- **unstar_repository** - 取消仓库加星
  - **OAuth Challenge Scopes**: `repo`
  - `owner`: 仓库所有者（字符串，必填）
  - `repo`: 仓库名称（字符串，必填）

</details>

<details>

<summary><picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/people-dark.png"><source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/people-light.png"><img src="https://raw.githubusercontent.com/github/github-mcp-server/3366807bbb8473841894de040d4abc89193e5d51/pkg/octicons/icons/people-light.png" width="20" height="20" alt="people"></picture> 用户</summary>

- **search_users** - 搜索用户
  - **OAuth Challenge Scopes**: `repo`
  - `order`: 排序顺序（字符串，可选）
  - `page`: 分页页码（最小 1）（数字，可选）
  - `perPage`: 分页时每页结果数（最小 1，最大 100）（数字，可选）
  - `query`：用户搜索查询。例如：'john smith'、'location:seattle'、'followers:>100'。搜索会自动限定为 type:user。（字符串，必填）
  - `sort`：按关注者数、仓库数或加入 GitHub 的时间排序用户。（字符串，可选）

</details>
<!-- END AUTOMATED TOOLS -->

### 远程 GitHub MCP 服务器中的附加工具

<details>

<summary>Copilot</summary>

- **create_pull_request_with_copilot** - 使用 GitHub Copilot 编程代理执行任务
  - `owner`：仓库所有者。你可以猜测所有者，但继续前应与用户确认。（字符串，必填）
  - `repo`：仓库名称。你可以猜测仓库名称，但继续前应与用户确认。（字符串，必填）
  - `problem_statement`：要执行任务的详细说明（例如 '实现功能 X'、'修复缺陷 Y' 等）（字符串，必填）
  - `title`：将要创建的拉取请求标题（字符串，必填）
  - `base_ref`：代理开始工作的 Git 引用（如分支）。未指定时默认为仓库默认分支（字符串，可选）

</details>

<details>

<summary>Copilot Spaces</summary>

- **认证说明**
  - 细粒度 PAT 不会被经典 PAT 权限范围筛选隐藏，因此即使令牌无法使用这些工具，它们仍可能显示。
  - 对于组织拥有的空间，细粒度 PAT 必须安装在所属组织上，且包含 `organization_copilot_spaces: read`。
  - 如果组织拥有的空间包含基于仓库的资源，令牌还必须有权访问所有被引用的仓库，否则空间可能被视为不存在。

- **get_copilot_space** - 获取 Copilot Space
  - `owner`：空间所有者。（字符串，必填）
  - `name`：空间名称。（字符串，必填）

- **list_copilot_spaces** - 列出 Copilot Space

</details>

<details>

<summary>GitHub 支持文档搜索</summary>

- **github_support_docs_search** - 检索与回答 GitHub 产品和支持问题相关的文档。支持主题包括：GitHub Actions Workflows、认证、GitHub 支持咨询、拉取请求实践、仓库维护、GitHub Pages、GitHub Packages、GitHub Discussions、Copilot Spaces
  - `query`：用户提出的问题输入。这是用户最新的原始未编辑消息。应始终保留用户消息原样，绝不要修改。（字符串，必填）

</details>

## 只读模式

要以只读模式运行服务器，可以使用 `--read-only` 标志。这样只提供只读工具，防止修改仓库、议题、拉取请求等。

```bash
./github-mcp-server --read-only
```

使用 Docker 时，可以通过环境变量传入只读模式：

```bash
docker run -i --rm \
  -e GITHUB_PERSONAL_ACCESS_TOKEN=<your-token> \
  -e GITHUB_READ_ONLY=1 \
  ghcr.io/github/github-mcp-server
```

## 锁定模式

锁定模式限制服务器从公开仓库展示的内容。启用后，服务器检查每项内容的作者是否对仓库拥有推送权限。私有仓库不受影响，协作者仍可完整访问自己的内容。

锁定模式是尽力而为的内容过滤器，旨在降低来自不受信任仓库内容（议题、拉取请求、评论、提交等）的提示注入风险。它**不是**授权边界：不会改变底层 GitHub 凭据可读写的内容，被过滤工具响应隐去的内容仍可能通过其他工具或使用相同凭据直接访问 GitHub API 获取。

作为有意设置的例外，少数受信任机器人账户（目前为 `github-actions[bot]` 和 `copilot`）编写的内容始终被视为安全，无论其是否拥有推送权限。这可以避免锁定模式过滤常规自动化输出（如 CI 生成的提交或评论）。

```bash
./github-mcp-server --lockdown-mode
```

使用 Docker 运行时，设置相应环境变量：

```bash
docker run -i --rm \
  -e GITHUB_PERSONAL_ACCESS_TOKEN=<your-token> \
  -e GITHUB_LOCKDOWN_MODE=1 \
  ghcr.io/github/github-mcp-server
```

在 HTTP 模式下，此标志（或 `GITHUB_LOCKDOWN_MODE`）是上限：操作员未启用时，`X-MCP-Lockdown` 请求标头可以启用锁定模式，但不能禁用操作员已启用的锁定模式。详见[服务器配置指南](https://github.com/github/github-mcp-server/blob/3366807bbb8473841894de040d4abc89193e5d51/docs/server-configuration.md#lockdown-mode)。

锁定模式的行为取决于调用的工具。

当作者没有推送权限时，以下工具会返回错误：

- `issue_read:get`
- `pull_request_read:get`
- `pull_request_read:get_diff`
- `pull_request_read:get_files`
- `pull_request_read:get_commits`

以下工具会过滤掉没有推送权限的用户发布的内容：

- `issue_read:get_comments`
- `issue_read:get_sub_issues`
- `pull_request_read:get_comments`
- `pull_request_read:get_review_comments`
- `pull_request_read:get_reviews`

## 国际化 / 覆盖说明

可以在与二进制文件相同的目录中创建
`github-mcp-server-config.json` 文件，以覆盖工具说明。

文件应包含一个 JSON 对象，以工具名称作为键，新的
说明作为值。例如：

```json
{
  "TOOL_ADD_ISSUE_COMMENT_DESCRIPTION": "an alternative description",
  "TOOL_CREATE_BRANCH_DESCRIPTION": "Create a new branch in a GitHub repository"
}
```

可以运行二进制文件并使用 `--export-translations` 标志，
导出当前翻译。

此标志会保留你所做的翻译/覆盖，同时添加
自上次导出以来添加到二进制文件中的
新翻译。

```sh
./github-mcp-server --export-translations
cat github-mcp-server-config.json
```

也可以使用环境变量覆盖说明。环境变量
名称与 JSON 文件中的键相同，前面加上
`GITHUB_MCP_` 前缀并全部大写。

例如，要覆盖 `TOOL_ADD_ISSUE_COMMENT_DESCRIPTION` 工具，可以
设置以下环境变量：

```sh
export GITHUB_MCP_TOOL_ADD_ISSUE_COMMENT_DESCRIPTION="an alternative description"
```

### 覆盖服务器名称和标题

相同的覆盖机制可用于自定义初始化响应中 MCP 服务器的 `name`
和 `title` 字段。运行多个 GitHub MCP Server 实例时（例如一个用于 github.com，
另一个用于 GitHub Enterprise Server），此功能很有用，
使代理能够区分它们。

| 键 | 环境变量 | 默认值 |
|-----|---------------------|---------|
| `SERVER_NAME` | `GITHUB_MCP_SERVER_NAME` | `github-mcp-server` |
| `SERVER_TITLE` | `GITHUB_MCP_SERVER_TITLE` | `GitHub MCP Server` |

例如，为 GitHub Enterprise Server 配置服务器实例：

```json
{
  "SERVER_NAME": "ghes-mcp-server",
  "SERVER_TITLE": "GHES MCP Server"
}
```

或使用环境变量：

```sh
export GITHUB_MCP_SERVER_NAME="ghes-mcp-server"
export GITHUB_MCP_SERVER_TITLE="GHES MCP Server"
```

## 库的使用

目前本模块导出的 Go API 应视为不稳定，可能发生破坏性变更。未来我们可能提供稳定性保证；如果你的用例需要这一点，请提交议题。

## 贡献

欢迎贡献。提交拉取请求前，请阅读[贡献指南](https://github.com/github/github-mcp-server/blob/3366807bbb8473841894de040d4abc89193e5d51/CONTRIBUTING.md)，了解设置、测试、lint 和文档生成说明。

## 支持

使用 GitHub MCP 服务器需要帮助时，请参阅[支持指南](https://github.com/github/github-mcp-server/blob/3366807bbb8473841894de040d4abc89193e5d51/SUPPORT.md)。如果发现缺陷或希望请求新功能，请在创建新议题前搜索现有议题。

## 安全

请勿通过公开议题报告安全漏洞。请按照[安全策略](https://github.com/github/github-mcp-server/blob/3366807bbb8473841894de040d4abc89193e5d51/SECURITY.md)中的说明负责任地报告漏洞。

## 许可证

本项目根据 MIT 开源许可证条款授权。完整条款请参见 [MIT](https://github.com/github/github-mcp-server/blob/3366807bbb8473841894de040d4abc89193e5d51/LICENSE)。
