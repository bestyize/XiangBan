<!-- markdownlint-disable MD030 -->

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/langflow-ai/langflow/b5c54a64e685b7e50fd3bc76e965ad718fc2dea3/docs/static/img/langflow-logo-color-blue-bg.svg">
  <img src="https://raw.githubusercontent.com/langflow-ai/langflow/b5c54a64e685b7e50fd3bc76e965ad718fc2dea3/docs/static/img/langflow-logo-color-black-solid.svg" alt="Langflow logo">
</picture>

[![Release Notes](https://img.shields.io/github/release/langflow-ai/langflow?style=flat-square)](https://github.com/langflow-ai/langflow/releases)
[![PyPI - License](https://img.shields.io/badge/license-MIT-orange)](https://opensource.org/licenses/MIT)
[![PyPI - Downloads](https://img.shields.io/pypi/dm/langflow?style=flat-square)](https://pypistats.org/packages/langflow)
[![Twitter](https://img.shields.io/twitter/url/https/twitter.com/langflow-ai.svg?style=social&label=Follow%20%40Langflow)](https://twitter.com/langflow_ai)
[![YouTube Channel](https://img.shields.io/badge/Subscribe-red?logo=youtube&logoColor=white)](https://www.youtube.com/@Langflow)
[![Discord Server](https://img.shields.io/discord/1116803230643527710?logo=discord&style=social&label=Join)](https://discord.gg/EqksyE2EX9)
[Ask DeepWiki](https://deepwiki.com/langflow-ai/langflow)

[Langflow](https://langflow.org) 是一个用于构建和部署 AI 智能体与工作流的平台。它为开发者提供可视化编辑体验，并内置 API 和 MCP 服务器，可将每个工作流转化为工具，集成到使用各种框架或技术栈构建的应用中。Langflow 自带多种功能，支持主流大语言模型、向量数据库以及不断扩充的 AI 工具库。

## ✨ 功能亮点

- **可视化构建界面**，方便快速上手和迭代。
- **访问源代码**，可使用 Python 自定义任意组件。
- **交互式测试环境**，可以逐步控制，即时测试和完善流程。
- **多智能体编排**，支持对话管理与检索。
- **部署为 API**，或导出为 JSON 供 Python 应用使用。
- **部署为 MCP 服务器**，将流程转为 MCP 客户端可用的工具。
- 通过 LangSmith、LangFuse 等集成提供**可观测性**。
- 面向**企业使用**的安全性和可扩展性。

## 🖥️  Langflow Desktop

Langflow Desktop 是上手 Langflow 的便捷方式。它包含所有依赖，因此无需手动管理 Python 环境或安装软件包。
适用于 Windows 和 macOS。

[📥 下载 Langflow Desktop](https://www.langflow.org/desktop)

## ⚡️ 快速开始

### 本地安装（推荐）

需要 Python 3.10–3.14 和 [uv](https://docs.astral.sh/uv/getting-started/installation/)（推荐的包管理器）。

#### 安装

在一个新目录中运行：
```shell
uv pip install langflow -U
```

这会安装最新的 Langflow 包。
更多信息参见[安装并运行 Langflow OSS Python 包](https://docs.langflow.org/get-started-installation#install-and-run-the-langflow-oss-python-package)。

#### 运行

官方发布包和带标签的源代码归档默认禁用自动登录。启动 Langflow 前请设置 `LANGFLOW_SUPERUSER_PASSWORD`，然后使用用户名 `langflow`（或你配置的 `LANGFLOW_SUPERUSER`）登录。开发分支默认仍启用自动登录。参见[身份验证配置](https://docs.langflow.org/api-keys-and-authentication#start-a-langflow-server-with-authentication-enabled)。

启动 Langflow，运行：
```shell
uv run langflow run
```

Langflow 将在 http://127.0.0.1:7860 启动。

完成！现在可以使用 Langflow 开始构建了！🎉

## 📦 其他安装方式

### 从源代码运行
如果你已克隆本仓库并希望参与贡献，请在仓库根目录运行以下命令：
```shell
make run_cli
```
更多信息参见 [DEVELOPMENT.md](https://github.com/langflow-ai/langflow/blob/b5c54a64e685b7e50fd3bc76e965ad718fc2dea3/DEVELOPMENT.md).

### Docker
启动容器前，请设置并导出一个强密码（Bash）：
```shell
read -rsp "Langflow administrator password: " LANGFLOW_SUPERUSER_PASSWORD; echo
export LANGFLOW_SUPERUSER_PASSWORD
docker run -p 7860:7860 -e LANGFLOW_SUPERUSER_PASSWORD langflowai/langflow:latest
```
可通过 http://localhost:7860/ 访问 Langflow。使用用户名 `langflow` 和输入的密码登录。官方 Docker 镜像默认要求身份验证。
如果现有安装启用了自动登录，请在切换默认设置前按照[升级说明](https://docs.langflow.org/api-keys-and-authentication#upgrade-from-auto-login)操作。
配置选项参见 [Docker 部署指南](https://docs.langflow.org/deployment-docker).

## 🛡️ 安全

安全信息参见我们的[安全策略](https://github.com/langflow-ai/langflow/blob/b5c54a64e685b7e50fd3bc76e965ad718fc2dea3/SECURITY.md).

## 🚀 部署

Langflow 完全开源，你可以将它部署到主流云平台。有关部署方法，请参见 [Langflow 部署指南](https://docs.langflow.org/deployment-overview).

## ⭐ 保持关注

在 GitHub 上为 Langflow 点星，以便及时获知新版本。

![Star Langflow](https://github.com/user-attachments/assets/03168b17-a11d-4b2a-b0f7-c1cce69e5a2c)

## 👋 参与贡献

我们欢迎不同经验水平的开发者贡献。如果你想参与，请阅读我们的[贡献指南](https://github.com/langflow-ai/langflow/blob/b5c54a64e685b7e50fd3bc76e965ad718fc2dea3/CONTRIBUTING.md)，帮助更多人方便地使用 Langflow。

---

[![Star History Chart](https://api.star-history.com/svg?repos=langflow-ai/langflow&type=Timeline)](https://star-history.com/#langflow-ai/langflow&Date)

## ❤️ 贡献者

[![langflow contributors](https://contrib.rocks/image?repo=langflow-ai/langflow)](https://github.com/langflow-ai/langflow/graphs/contributors)
