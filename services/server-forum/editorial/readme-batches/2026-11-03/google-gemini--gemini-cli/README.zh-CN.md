# Gemini CLI

[![Gemini CLI CI](https://github.com/google-gemini/gemini-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/google-gemini/gemini-cli/actions/workflows/ci.yml)
[![Gemini CLI E2E (Chained)](https://github.com/google-gemini/gemini-cli/actions/workflows/chained_e2e.yml/badge.svg)](https://github.com/google-gemini/gemini-cli/actions/workflows/chained_e2e.yml)
[![Version](https://img.shields.io/npm/v/@google/gemini-cli)](https://www.npmjs.com/package/@google/gemini-cli)
[![License](https://img.shields.io/github/license/google-gemini/gemini-cli)](https://github.com/google-gemini/gemini-cli/blob/main/LICENSE)
[![View Code Wiki](https://assets.codewiki.google/readme-badge/static.svg)](https://codewiki.google/github.com/google-gemini/gemini-cli?utm_source=badge&utm_medium=github&utm_campaign=github.com/google-gemini/gemini-cli)

<!-- prettier-ignore -->
> [!IMPORTANT]
> **安装前请阅读——Gemini CLI 状态与工具指南**
>
> - **仅限企业 Code Assist：** 面向 Google Cloud 上的 **Gemini Code Assist Standard / Enterprise** 许可证和 **Code Assist for GitHub** 设计（你的访问权限保持不变）。
> - **不提供免费层级或 Google AI 计划：** 免费个人访问和 **Google AI Pro / Ultra**（Google One）已于 2026 年 6 月 18 日迁移至 [Antigravity CLI](https://antigravity.google/product/antigravity-cli)（[迁移公告](https://developers.googleblog.com/an-important-update-transitioning-gemini-cli-to-antigravity-cli/)）。
> - **未针对 Gemini 4 及更高版本优化：** 未针对 **Gemini 4** 系列（例如 [Gemini 4 Argon](https://blog.google/innovation-and-ai/models-and-research/gemini-models/gemini-4-argon/)）或本代、后续世代的未来版本进行调优。
> - **不适用于模型基准测试：** 不要使用 Gemini CLI 对 Gemini 模型进行基准测试——其运行框架未针对 Gemini 4 及更高版本调优，所得分数不具代表性。
>
> **你需要哪种 Google CLI？**
>
> | 工具 | 适用场景 | 链接 |
> | :--- | :--- | :--- |
> | **[Antigravity CLI](https://antigravity.google/product/antigravity-cli)** (`agy`) | 交互式多智能体编码终端。适用于 **Gemini 4 Argon** 及更高版本、免费／**Google AI Pro 与 Ultra** 账号或 Google Cloud。 | [下载](https://antigravity.google/download) · [迁移](https://antigravity.google/docs/cli/gcli-migration) · [Argon 公告](https://antigravity.google/blog/gemini-4-argon-in-google-antigravity) |
> | **[Gemini API CLI](https://github.com/google-gemini/gemini-api-cli)** (`gemini-api`) | 无状态 API CLI（*不是编码运行框架*）。为智能体（`agy`、Claude Code、Codex）、脚本和 CI 提供直接命令，用于 Gemini 模型、媒体生成、文件分析及托管智能体。 | [GitHub](https://github.com/google-gemini/gemini-api-cli) · [发行版本](https://github.com/google-gemini/gemini-api-cli/releases) · [API 文档](https://ai.google.dev/gemini-api/docs) |
> | **Gemini CLI** (`gemini`, *本仓库*) | 使用 Google Cloud 上 **Gemini Code Assist Standard / Enterprise** 或 **Code Assist for GitHub** 的组织（*以及旧版付费 API / Vertex 密钥*）。 | [文档](https://geminicli.com/docs/) |

Gemini CLI（`gemini`）是一个开源终端智能体，为 Google Cloud 上的企业 Gemini Code Assist 工作流维护。更多信息请参阅 [Gemini CLI 文档](https://geminicli.com/docs/)。

## 📦 安装

有关推荐系统规格及详细安装指南，请参阅 [Gemini CLI 安装、运行与发行版本](https://www.geminicli.com/docs/get-started/installation)。

### 快速安装

#### 使用 npx 立即运行

```bash
# Using npx (no installation required)
npx @google/gemini-cli
```

#### 使用 npm 全局安装

```bash
npm install -g @google/gemini-cli
```

#### 使用 Homebrew 全局安装（macOS/Linux）

```bash
brew install gemini-cli
```

#### 使用 MacPorts 全局安装（macOS）

```bash
sudo port install gemini-cli
```

#### 使用 Anaconda 安装（受限环境）

```bash
# Create and activate a new environment
conda create -y -n gemini_env -c conda-forge nodejs
conda activate gemini_env

# Install Gemini CLI globally via npm (inside the environment)
npm install -g @google/gemini-cli
```

## 发行渠道

更多信息请参阅[发行版本](https://www.geminicli.com/docs/changelogs)。

### 预览版

新的预览版每周二 UTC 23:59 发布。这些版本尚未经过完整验证，可能包含回归或其他未解决的问题。欢迎使用 `preview` 标签安装并帮助测试。

```bash
npm install -g @google/gemini-cli@preview
```

### 稳定版

- 新稳定版每周二 UTC 20:00 发布，包含上周 `preview` 版本的完整晋升以及错误修复和验证。使用 `latest` 标签。

```bash
npm install -g @google/gemini-cli@latest
```

### 每日构建

- 新版本每天 UTC 00:00 发布，包含发布时主分支上的所有变更。应假定其中仍有待验证内容和问题。使用 `nightly` 标签。

```bash
npm install -g @google/gemini-cli@nightly
```

## 📋 主要功能

### 代码理解与生成

- 查询和编辑大型代码库
- 利用多模态能力根据 PDF、图片或草图生成新应用
- 使用自然语言调试问题并排查故障

### 自动化与集成

- 自动执行查询 Pull Request 或处理复杂 rebase 等操作任务
- 使用 MCP 服务器连接新能力，包括[使用 Imagen、Veo 或 Lyria 生成媒体](https://github.com/GoogleCloudPlatform/vertex-ai-creative-studio/tree/main/experiments/mcp-genmedia)
- 在脚本中以非交互方式运行，实现工作流自动化

## 🔐 身份验证选项

选择最适合你需求的身份验证方式：

### 选项 1：使用 Google 登录（通过 Google 账号进行 OAuth 登录）

#### 使用组织提供的付费 Code Assist 许可证时，请记得设置 Google Cloud 项目

```bash
# Set your Google Cloud Project
export GOOGLE_CLOUD_PROJECT="YOUR_PROJECT_ID"
gemini
```

### 选项 2：Gemini API 密钥

**✨ 适用：** 需要指定模型或访问付费层级的开发者

**优势：**

- **模型选择**：选择特定 Gemini 模型
- **按用量计费**：需要时升级以获得更高限额

```bash
# Get your key from https://aistudio.google.com/apikey
export GEMINI_API_KEY="YOUR_API_KEY"
gemini
```

### 选项 3：Vertex AI

**✨ 适用：** 企业团队和生产工作负载

**优势：**

- **企业功能**：高级安全与合规
- **可扩展**：使用结算账号获得更高的速率限制
- **集成**：可配合现有 Google Cloud 基础设施使用

```bash
# Get your key from Google Cloud Console
export GOOGLE_API_KEY="YOUR_API_KEY"
export GOOGLE_GENAI_USE_VERTEXAI=true
gemini
```

有关 Google Workspace 账号和其他验证方式，请参阅[身份验证指南](https://www.geminicli.com/docs/get-started/authentication)。

## 🚀 开始使用

### 基本用法

#### 在当前目录启动

```bash
gemini
```

#### 包含多个目录

```bash
gemini --include-directories ../lib,../docs
```

#### 使用指定模型

```bash
gemini -m gemini-2.5-flash
```

#### 脚本的非交互模式

获取简单文本回复：

```bash
gemini -p "Explain the architecture of this codebase"
```

对于更高级的脚本，包括解析 JSON 和处理错误，可使用 `--output-format json` 参数获取结构化输出：

```bash
gemini -p "Explain the architecture of this codebase" --output-format json
```

对于实时事件流（可用于监控长时间运行的操作），使用 `--output-format stream-json` 获取以换行分隔的 JSON 事件：

```bash
gemini -p "Run tests and deploy" --output-format stream-json
```

### 快速示例

#### 开始新项目

```bash
cd new-project/
gemini
> Write me a Discord bot that answers questions using a FAQ.md file I will provide
```

#### 分析现有代码

```bash
git clone https://github.com/google-gemini/gemini-cli
cd gemini-cli
gemini
> Give me a summary of all of the changes that went in yesterday
```

## 📚 文档

### 入门

- [**快速入门指南**](https://www.geminicli.com/docs/get-started) - 快速开始运行。
- [**身份验证设置**](https://www.geminicli.com/docs/get-started/authentication) -
  详细的身份验证配置。
- [**配置指南**](https://www.geminicli.com/docs/reference/configuration) -
  设置与自定义。
- [**键盘快捷键**](https://www.geminicli.com/docs/reference/keyboard-shortcuts) -
  提高效率的提示。

### 核心功能

- [**命令参考**](https://www.geminicli.com/docs/reference/commands) -
  所有斜杠命令（`/help`、`/chat` 等）。
- [**自定义命令**](https://www.geminicli.com/docs/cli/custom-commands) -
  创建自己的可复用命令。
- [**上下文文件（GEMINI.md）**](https://www.geminicli.com/docs/cli/gemini-md) -
  为 Gemini CLI 提供持久上下文。
- [**检查点**](https://www.geminicli.com/docs/cli/checkpointing) - 保存和恢复对话。
- [**Token 缓存**](https://www.geminicli.com/docs/cli/token-caching) -
  优化 token 使用量。

### 工具与扩展

- [**内置工具概览**](https://www.geminicli.com/docs/reference/tools)
  - [文件系统操作](https://www.geminicli.com/docs/tools/file-system)
  - [Shell 命令](https://www.geminicli.com/docs/tools/shell)
  - [网页获取与搜索](https://www.geminicli.com/docs/tools/web-fetch)
- [**MCP 服务器集成**](https://www.geminicli.com/docs/tools/mcp-server) -
  使用自定义工具扩展。
- [**自定义扩展**](https://geminicli.com/docs/extensions/writing-extensions) -
  构建并分享自己的命令。

### 高级主题

- [**无头模式（脚本）**](https://www.geminicli.com/docs/cli/headless) -
  在自动化工作流中使用 Gemini CLI。
- [**IDE 集成**](https://www.geminicli.com/docs/ide-integration) - VS Code 配套工具。
- [**沙箱与安全**](https://www.geminicli.com/docs/cli/sandbox) - 安全执行环境。
- [**受信任文件夹**](https://www.geminicli.com/docs/cli/trusted-folders) -
  按文件夹控制执行策略。
- [**企业指南**](https://www.geminicli.com/docs/cli/enterprise) - 在企业环境中部署和管理。
- [**遥测与监控**](https://www.geminicli.com/docs/cli/telemetry) -
  使用情况跟踪。
- [**工具参考**](https://www.geminicli.com/docs/reference/tools) -
  内置工具概览。
- [**本地开发**](https://www.geminicli.com/docs/local-development) -
  本地开发工具。

### 故障排查与支持

- [**故障排查指南**](https://www.geminicli.com/docs/resources/troubleshooting) -
  常见问题及解决方案。
- [**FAQ**](https://www.geminicli.com/docs/resources/faq) - 常见问题。
- 使用 `/bug` 命令直接从 CLI 报告问题。

### 使用 MCP 服务器

在 `~/.gemini/settings.json` 中配置 MCP 服务器，使用自定义工具扩展 Gemini CLI：

```text
> @github List my open pull requests
> @slack Send a summary of today's commits to #dev channel
> @database Run a query to find inactive users
```

设置说明请参阅 [MCP 服务器集成指南](https://www.geminicli.com/docs/tools/mcp-server)。

## 🤝 参与贡献

欢迎贡献！Gemini CLI 完全开源（Apache 2.0），我们鼓励社区：

- 报告错误和提出功能建议。
- 改进文档。
- 提交代码改进。
- 分享你的 MCP 服务器和扩展。

请参阅[贡献指南](https://github.com/google-gemini/gemini-cli/blob/0ac57c029b848d109eef940afe0db4b5a0c3dad0/CONTRIBUTING.md)，了解开发环境设置、编码标准以及如何提交 Pull Request。

请查看[官方路线图](https://github.com/orgs/google-gemini/projects/11)，了解计划中的功能和优先事项。

## 📖 资源

- **[免费课程](https://learn.deeplearning.ai/courses/gemini-cli-code-and-create-with-an-open-source-agent/information)** -
  学习基础知识。
- **[官方路线图](https://github.com/google-gemini/gemini-cli/blob/0ac57c029b848d109eef940afe0db4b5a0c3dad0/ROADMAP.md)** - 查看接下来的计划。
- **[更新日志](https://www.geminicli.com/docs/changelogs)** - 查看近期重要更新。
- **[NPM 包](https://www.npmjs.com/package/@google/gemini-cli)** - 软件包注册表。
- **[GitHub Issues](https://github.com/google-gemini/gemini-cli/issues)** -
  报告错误或请求功能。
- **[安全公告](https://github.com/google-gemini/gemini-cli/security/advisories)** -
  安全更新。

### 卸载

卸载说明请参阅[卸载指南](https://www.geminicli.com/docs/resources/uninstall)。

## 📄 法律信息

- **许可证**： [Apache License 2.0](https://github.com/google-gemini/gemini-cli/blob/0ac57c029b848d109eef940afe0db4b5a0c3dad0/LICENSE)
- **服务条款**：
  [条款与隐私](https://www.geminicli.com/docs/resources/tos-privacy)
- **安全**： [安全政策](https://github.com/google-gemini/gemini-cli/blob/0ac57c029b848d109eef940afe0db4b5a0c3dad0/SECURITY.md)

<p align="left">
 <a href="https://www.star-history.com/google-gemini/gemini-cli">
  <picture>
   <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/badge?repo=google-gemini/gemini-cli&theme=dark" />
   <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/badge?repo=google-gemini/gemini-cli" />
   <img alt="Star History Rank" src="https://api.star-history.com/badge?repo=google-gemini/gemini-cli" />
  </picture>
 </a>
</p>

---

<p align="center">
  由 Google 和开源社区用 ❤️ 构建
</p>
