> 根据 [Ollama 固定版本 README](https://github.com/ollama/ollama/blob/9a3e8177767245acc62b735eba7d5d8a553606d0/README.md) 翻译。隐私、功能和第三方集成等描述为上游原文表述，未逐项独立验证，使用云服务前请核对其隐私政策。

<p align="center">
  <a href="https://ollama.com">
    <img src="https://github.com/ollama/ollama/assets/3325447/0d0b44e2-8f4a-4e99-9b52-a5c1c741c8f7" alt="ollama" width="200"/>
  </a>
</p>

# Ollama

开始使用开放模型构建。在本地或云端访问最新发行版，并享有完整隐私保护。

## 下载

### macOS

```shell
curl -fsSL https://ollama.com/install.sh | sh
```

或[手动下载](https://ollama.com/download/Ollama.dmg)

### Windows

```shell
irm https://ollama.com/install.ps1 | iex
```

或[手动下载](https://ollama.com/download/OllamaSetup.exe)

### Linux

```shell
curl -fsSL https://ollama.com/install.sh | sh
```

[手动安装说明](https://docs.ollama.com/linux#manual-install)

### Docker

官方 [Ollama Docker 镜像](https://hub.docker.com/r/ollama/ollama) `ollama/ollama` 可在 Docker Hub 获取。

### 客户端库

- [ollama-python](https://github.com/ollama/ollama-python)
- [ollama-js](https://github.com/ollama/ollama-js)

### 社区

- [Discord](https://discord.gg/ollama)
- [𝕏 (Twitter)](https://x.com/ollama)
- [Reddit](https://reddit.com/r/ollama)

## 开始使用

```
ollama
```

程序会提示你运行模型，或将 Ollama 连接到现有代理或应用，例如 `Claude Code`、`OpenClaw`、`OpenCode`、`Codex`、`Copilot` 等。

### 编程

启动特定集成：

```
ollama launch claude
```

支持的集成包括 [Claude Code](https://docs.ollama.com/integrations/claude-code)、[Codex](https://docs.ollama.com/integrations/codex)、[Copilot CLI](https://docs.ollama.com/integrations/copilot-cli)、[DeepSeek Harness](https://docs.ollama.com/integrations/deepseek-harness)、[Droid](https://docs.ollama.com/integrations/droid) 和 [OpenCode](https://docs.ollama.com/integrations/opencode)。

### AI 助手

使用 [OpenClaw](https://docs.ollama.com/integrations/openclaw)，让 Ollama 成为可通过 WhatsApp、Telegram、Slack、Discord 等平台使用的个人 AI 助手：

```
ollama launch openclaw
```

### 与模型聊天

运行并与 [Gemma 4](https://ollama.com/library/gemma4) 聊天：

```
ollama run gemma4
```

完整列表参见 [ollama.com/library](https://ollama.com/library)。

更多细节参见[快速入门指南](https://docs.ollama.com/quickstart)。

## REST API

Ollama 提供 REST API 来运行和管理模型。

```
curl http://localhost:11434/api/chat -d '{
  "model": "gemma4",
  "messages": [{
    "role": "user",
    "content": "Why is the sky blue?"
  }],
  "stream": false
}'
```

所有端点参见 [API 文档](https://docs.ollama.com/api)。

### Python

```
pip install ollama
```

```python
from ollama import chat

response = chat(model='gemma4', messages=[
  {
    'role': 'user',
    'content': 'Why is the sky blue?',
  },
])
print(response.message.content)
```

### JavaScript

```
npm i ollama
```

```javascript
import ollama from "ollama";

const response = await ollama.chat({
  model: "gemma4",
  messages: [{ role: "user", content: "Why is the sky blue?" }],
});
console.log(response.message.content);
```

## 支持的后端

- [llama.cpp](https://github.com/ggml-org/llama.cpp) 项目，由 Georgi Gerganov 创立。

## 文档

- [命令行参考](https://docs.ollama.com/cli)
- [REST API 参考](https://docs.ollama.com/api)
- [导入模型](https://docs.ollama.com/import)
- [Modelfile 参考](https://docs.ollama.com/modelfile)
- [从源代码构建](https://github.com/ollama/ollama/blob/main/docs/development.md)

## 社区集成

> 想添加你的项目？请提交拉取请求。

### 聊天界面

#### 网页端

- [Open WebUI](https://github.com/open-webui/open-webui) - 可扩展的自托管 AI 界面
- [Onyx](https://github.com/onyx-dot-app/onyx) - 互联的 AI 工作空间
- [LibreChat](https://github.com/danny-avila/LibreChat) - 支持多个服务商的增强版 ChatGPT 克隆
- [Lobe Chat](https://github.com/lobehub/lobe-chat) - 具有插件生态的现代聊天框架 ([文档](https://lobehub.com/docs/self-hosting/examples/ollama))
- [NextChat](https://github.com/ChatGPTNextWeb/ChatGPT-Next-Web) - 跨平台 ChatGPT 界面 ([文档](https://docs.nextchat.dev/models/ollama))
- [Perplexica](https://github.com/ItzCrazyKns/Perplexica) - AI 搜索引擎，Perplexity 的开源替代品
- [big-AGI](https://github.com/enricoros/big-AGI) - 面向专业人士的 AI 套件
- [Lollms WebUI](https://github.com/ParisNeo/lollms-webui) - 多模型网页界面
- [ChatOllama](https://github.com/sugarforever/chat-ollama) - 支持知识库的聊天机器人
- [Bionic GPT](https://github.com/bionic-gpt/bionic-gpt) - 本地部署的 AI 平台
- [Chatbot UI](https://github.com/ivanfioravanti/chatbot-ollama) - ChatGPT 风格网页界面
- [Hollama](https://github.com/fmaclen/hollama) - 简洁的网页界面
- [Chatbox](https://github.com/Bin-Huang/Chatbox) - 桌面和网页 AI 客户端
- [chat](https://github.com/swuecho/chat) - 面向团队的网页聊天应用
- [Ollama RAG Chatbot](https://github.com/datvodinh/rag-chatbot.git) - 通过 RAG 与多个 PDF 对话
- [Tkinter-based client](https://github.com/chyok/ollama-gui) - Python 桌面客户端

#### 桌面端

- [Dify.AI](https://github.com/langgenius/dify) - 大语言模型应用开发平台
- [AnythingLLM](https://github.com/Mintplex-Labs/anything-llm) - 适用于 Mac、Windows 和 Linux 的一体化 AI 应用
- [Maid](https://github.com/Mobile-Artificial-Intelligence/maid) - 跨平台移动及桌面客户端
- [Witsy](https://github.com/nbonamy/witsy) - 适用于 Mac、Windows 和 Linux 的 AI 桌面应用
- [Cherry Studio](https://github.com/kangfenmao/cherry-studio) - 支持多个服务商的桌面客户端
- [Ollama App](https://github.com/JHubi1/ollama-app) - 适用于桌面和移动端的多平台客户端
- [PyGPT](https://github.com/szczyglis-dev/py-gpt) - 适用于 Linux、Windows 和 Mac 的 AI 桌面助手
- [Alpaca](https://github.com/Jeffser/Alpaca) - 适用于 Linux 和 macOS 的 GTK4 客户端
- [SwiftChat](https://github.com/aws-samples/swift-chat) - 跨平台，支持 iOS、Android 和 Apple Vision Pro
- [Enchanted](https://github.com/AugustDev/enchanted) - 原生 macOS 和 iOS 客户端
- [RWKV-Runner](https://github.com/josStorer/RWKV-Runner) - 多模型桌面运行工具
- [Ollama Grid Search](https://github.com/dezoito/ollama-grid-search) - 评估并比较模型
- [macai](https://github.com/Renset/macai) - 适用于 Ollama 和 ChatGPT 的 macOS 客户端
- [AI Studio](https://github.com/MindWorkAI/AI-Studio) - 支持多个服务商的桌面 IDE
- [Reins](https://github.com/ibrahimcetin/reins) - 参数调优和推理模型支持
- [ConfiChat](https://github.com/1runeberg/confichat) - 注重隐私并提供可选加密
- [LLocal.in](https://github.com/kartikm7/llocal) - Electron 桌面客户端
- [MindMac](https://mindmac.app) - Mac AI 聊天客户端
- [Msty](https://msty.app) - 多模型桌面客户端
- [BoltAI for Mac](https://boltai.com) - Mac AI 聊天客户端
- [IntelliBar](https://intellibar.app/) - macOS AI 助手
- [Kerlig AI](https://www.kerlig.com/) - macOS AI 写作助手
- [Hillnote](https://hillnote.com) - 以 Markdown 为核心的 AI 工作空间
- [Perfect Memory AI](https://www.perfectmemory.ai/) - 根据屏幕和会议历史个性化的生产力 AI

#### 移动端

- [Ollama Android Chat](https://github.com/sunshine0523/OllamaServer) - 在 Android 上一键使用 Ollama

> 上面列出的 SwiftChat、Enchanted、Maid、Ollama App、Reins 和 ConfiChat 也支持移动平台。

### 代码编辑器与开发

- [Cline](https://github.com/cline/cline) - 支持多文件或整个仓库编码的 VS Code 扩展
- [Continue](https://github.com/continuedev/continue) - 适用于任何 IDE 的开源 AI 编程助手
- [Void](https://github.com/voideditor/void) - 开源 AI 代码编辑器，Cursor 的替代品
- [Copilot for Obsidian](https://github.com/logancyang/obsidian-copilot) - Obsidian AI 助手
- [twinny](https://github.com/rjmacarthy/twinny) - Copilot 及 Copilot Chat 的替代品
- [gptel Emacs client](https://github.com/karthink/gptel) - Emacs 大语言模型客户端
- [Ollama Copilot](https://github.com/bernardo-bruning/ollama-copilot) - 将 Ollama 用作 GitHub Copilot
- [Obsidian Local GPT](https://github.com/pfrankov/obsidian-local-gpt) - Obsidian 本地 AI
- [Ellama Emacs client](https://github.com/s-kostyaev/ellama) - Emacs 大语言模型工具
- [orbiton](https://github.com/xyproto/orbiton) - 无需配置并支持 Ollama Tab 补全的文本编辑器
- [AI ST Completion](https://github.com/yaroslavyaroslav/OpenAI-sublime-text) - Sublime Text 4 AI 助手
- [VT Code](https://github.com/vinhnx/vtcode) - 基于 Rust、支持 Tree-sitter 的终端编码代理
- [QodeAssist](https://github.com/Palm1r/QodeAssist) - Qt Creator AI 编码助手
- [AI Toolkit for VS Code](https://aka.ms/ai-tooklit/ollama-docs) - 微软官方 VS Code 扩展
- [Open Interpreter](https://docs.openinterpreter.com/language-model-setup/local-models/ollama) - 计算机的自然语言界面
- [oxi](https://github.com/maziluiosif/oxi) - 原生 Rust 桌面编码代理，包含编辑器、Git 面板和终端

### 库和 SDK

- [LiteLLM](https://github.com/BerriAI/litellm) - 面向 100 多家大语言模型服务商的统一 API
- [Semantic Kernel](https://github.com/microsoft/semantic-kernel/tree/main/python/semantic_kernel/connectors/ai/ollama) - 微软 AI 编排 SDK
- [LangChain4j](https://github.com/langchain4j/langchain4j) - Java LangChain ([示例](https://github.com/langchain4j/langchain4j-examples/tree/main/ollama-examples/src/main/java))
- [LangChainGo](https://github.com/tmc/langchaingo/) - Go LangChain ([示例](https://github.com/tmc/langchaingo/tree/main/examples/ollama-completion-example))
- [Spring AI](https://github.com/spring-projects/spring-ai) - Spring 框架 AI 支持 ([文档](https://docs.spring.io/spring-ai/reference/api/chat/ollama-chat.html))
- [LangChain](https://python.langchain.com/docs/integrations/chat/ollama/) 和 [LangChain.js](https://js.langchain.com/docs/integrations/chat/ollama/)，附[示例](https://js.langchain.com/docs/tutorials/local_rag/)
- [Ollama for Ruby](https://github.com/crmne/ruby_llm) - Ruby 大语言模型库
- [any-llm](https://github.com/mozilla-ai/any-llm) - Mozilla 的统一大语言模型接口
- [OllamaSharp for .NET](https://github.com/awaescher/OllamaSharp) - .NET SDK
- [LangChainRust](https://github.com/Abraxas-365/langchain-rust) - Rust LangChain ([示例](https://github.com/Abraxas-365/langchain-rust/blob/main/examples/llm_ollama.rs))
- [Agents-Flex for Java](https://github.com/agents-flex/agents-flex) - Java 代理框架 ([示例](https://github.com/agents-flex/agents-flex/tree/main/agents-flex-chat/agents-flex-chat-ollama))
- [Elixir LangChain](https://github.com/brainlid/langchain) - Elixir LangChain
- [Ollama-rs for Rust](https://github.com/pepperoni21/ollama-rs) - Rust SDK
- [LangChain for .NET](https://github.com/tryAGI/LangChain) - .NET LangChain ([示例](https://github.com/tryAGI/LangChain/blob/main/examples/LangChain.Samples.OpenAI/Program.cs))
- [chromem-go](https://github.com/philippgille/chromem-go) - 支持 Ollama 嵌入的 Go 向量数据库 ([示例](https://github.com/philippgille/chromem-go/tree/v0.5.0/examples/rag-wikipedia-ollama))
- [LangChainDart](https://github.com/davidmigloz/langchain_dart) - Dart LangChain
- [LlmTornado](https://github.com/lofcz/llmtornado) - 面向多种推理 API 的统一 C# 接口
- [Ollama4j for Java](https://github.com/ollama4j/ollama4j) - Java SDK
- [Ollama for Laravel](https://github.com/cloudstudio/ollama-laravel) - Laravel 集成
- [Ollama for Swift](https://github.com/mattt/ollama-swift) - Swift SDK
- [LlamaIndex](https://docs.llamaindex.ai/en/stable/examples/llm/ollama/) and [LlamaIndexTS](https://developers.llamaindex.ai/typescript/framework/modules/models/llms/ollama/) - 用于大语言模型应用的数据框架
- [Haystack](https://github.com/deepset-ai/haystack-integrations/blob/main/integrations/ollama.md) - AI 流水线框架
- [Firebase Genkit](https://firebase.google.com/docs/genkit/plugins/ollama) - Google AI 框架
- [Ollama-hpp for C++](https://github.com/jmont-dev/ollama-hpp) - C++ SDK
- [PromptingTools.jl](https://github.com/svilupp/PromptingTools.jl) - Julia 大语言模型工具包 ([示例](https://svilupp.github.io/PromptingTools.jl/dev/examples/working_with_ollama))
- [Ollama for R - rollama](https://github.com/JBGruber/rollama) - R SDK
- [Portkey](https://portkey.ai/docs/welcome/integration-guides/ollama) - AI 网关
- [Testcontainers](https://testcontainers.com/modules/ollama/) - 基于容器的测试
- [LLPhant](https://github.com/theodo-group/LLPhant?tab=readme-ov-file#ollama) - PHP AI 框架

### 框架与代理

- [AutoGPT](https://github.com/Significant-Gravitas/AutoGPT/blob/master/docs/platform/ollama.md) - 自主 AI 代理平台
- [crewAI](https://github.com/crewAIInc/crewAI) - 多代理编排框架
- [Strands Agents](https://github.com/strands-agents/sdk-python) - AWS 提供的模型驱动代理构建工具
- [Cheshire Cat](https://github.com/cheshire-cat-ai/core) - AI 助手框架
- [any-agent](https://github.com/mozilla-ai/any-agent) - Mozilla 的统一代理框架接口
- [Stakpak](https://github.com/stakpak/agent) - 开源 DevOps 代理
- [Hexabot](https://github.com/hexastack/hexabot) - 对话 AI 构建工具
- [Neuro SAN](https://github.com/cognizant-ai-lab/neuro-san-studio) - 多代理编排 ([文档](https://github.com/cognizant-ai-lab/neuro-san-studio/blob/main/docs/user_guide.md#ollama))

### RAG 与知识库

- [RAGFlow](https://github.com/infiniflow/ragflow) - 基于深度文档理解的 RAG 引擎
- [R2R](https://github.com/SciPhi-AI/R2R) - 开源 RAG 引擎
- [MaxKB](https://github.com/1Panel-dev/MaxKB/) - 开箱即用的 RAG 聊天机器人
- [Minima](https://github.com/dmayboroda/minima) - 本地部署或完全本地的 RAG
- [Chipper](https://github.com/TilmanGriesel/chipper) - 集成 Haystack RAG 的 AI 界面
- [ARGO](https://github.com/xark-argo/argo) - 适用于 Mac、Windows 和 Linux 的 RAG 与深度研究工具
- [Archyve](https://github.com/nickthecook/archyve) - 支持 RAG 的文档库
- [Casibase](https://casibase.org) - 集成 RAG 和单点登录的 AI 知识库
- [BrainSoup](https://www.nurgo-software.com/products/brainsoup) - 支持 RAG 和多代理自动化的原生客户端

### 机器人与消息

- [LangBot](https://github.com/RockChinQ/LangBot) - 支持代理与 RAG 的多平台消息机器人
- [AstrBot](https://github.com/AstrBotDevs/AstrBot/) - 支持 RAG 和插件的多平台聊天机器人
- [Discord-Ollama Chat Bot](https://github.com/kevinthedang/discord-ollama) - TypeScript Discord 机器人
- [Ollama Telegram Bot](https://github.com/ruecat/ollama-telegram) - Telegram 机器人
- [LLM Telegram Bot](https://github.com/innightwolfsleep/llm_telegram_bot) - 角色扮演 Telegram 机器人

### 终端与命令行

- [aichat](https://github.com/sigoden/aichat) - 包含 Shell 助手、RAG 和 AI 工具的一体化大语言模型命令行工具
- [oterm](https://github.com/ggozad/oterm) - Ollama 终端客户端
- [gollama](https://github.com/sammcj/gollama) - 基于 Go 的 Ollama 模型管理器
- [tlm](https://github.com/yusufcanb/tlm) - 本地 Shell 助手
- [tenere](https://github.com/pythops/tenere) - 大语言模型终端用户界面
- [ParLlama](https://github.com/paulrobello/parllama) - Ollama 终端用户界面
- [llm-ollama](https://github.com/taketwo/llm-ollama) - [Datasette 的大语言模型命令行工具](https://llm.datasette.io/en/stable/)插件
- [ShellOracle](https://github.com/djcopley/ShellOracle) - Shell 命令建议
- [LLM-X](https://github.com/mrdjohnson/llm-x) - 大语言模型渐进式网页应用
- [cmdh](https://github.com/pgibler/cmdh) - 将自然语言转换为 Shell 命令
- [VT](https://github.com/vinhnx/vt.ai) - 简洁的多模态 AI 聊天应用

### 生产力与应用

- [AppFlowy](https://github.com/AppFlowy-IO/AppFlowy) - AI 协作工作空间，可自托管的 Notion 替代品
- [Screenpipe](https://github.com/mediar-ai/screenpipe) - 全天候屏幕和麦克风录制及 AI 搜索
- [Vibe](https://github.com/thewh1teagle/vibe) - 转录和分析会议
- [Page Assist](https://github.com/n4ze3m/page-assist) - 用于 AI 辅助浏览的 Chrome 扩展
- [NativeMind](https://github.com/NativeMindBrowser/NativeMindExtension) - 注重隐私、在设备本地运行的浏览器 AI 助手
- [Ollama Fortress](https://github.com/ParisNeo/ollama_proxy_server) - Ollama 安全代理
- [1Panel](https://github.com/1Panel-dev/1Panel/) - 基于网页的 Linux 服务器管理
- [Writeopia](https://github.com/Writeopia/Writeopia) - 集成 Ollama 的文本编辑器
- [QA-Pilot](https://github.com/reid41/QA-Pilot) - 理解 GitHub 代码仓库
- [Raycast extension](https://github.com/MassimilianoPasquini97/raycast_ollama) - 在 Raycast 中使用 Ollama
- [Painting Droid](https://github.com/mateuszmigas/painting-droid) - 集成 AI 的绘画应用
- [Serene Pub](https://github.com/doolijb/serene-pub) - AI 角色扮演应用
- [Mayan EDMS](https://gitlab.com/mayan-edms/mayan-edms) - 带 Ollama 工作流的文档管理
- [TagSpaces](https://www.tagspaces.org) - 支持 [AI 标签](https://docs.tagspaces.org/ai/)的文件管理

### 可观测性与监控

- [Opik](https://www.comet.com/docs/opik/cookbook/ollama) - 调试、评估和监控大语言模型应用
- [OpenLIT](https://github.com/openlit/openlit) - 基于 OpenTelemetry 的 Ollama 和 GPU 监控
- [Lunary](https://lunary.ai/docs/integrations/ollama) - 具备分析和个人身份信息脱敏功能的大语言模型可观测性
- [Langfuse](https://langfuse.com/docs/integrations/ollama) - 开源大语言模型可观测性
- [HoneyHive](https://docs.honeyhive.ai/integrations/ollama) - 面向代理的 AI 可观测性和评估
- [MLflow Tracing](https://mlflow.org/docs/latest/llms/tracing/index.html#automatic-tracing) - 开源大语言模型可观测性

### 数据库与嵌入

- [pgai](https://github.com/timescale/pgai) - 将 PostgreSQL 用作向量数据库 ([指南](https://github.com/timescale/pgai/blob/main/docs/vectorizer-quick-start.md))
- [MindsDB](https://docs.mindsdb.com/integrations/ai-engines/ollama) - 将 Ollama 连接至 200 多个数据平台
- [chromem-go](https://github.com/philippgille/chromem-go/blob/v0.5.0/embed_ollama.go) - 可嵌入的 Go 向量数据库 ([示例](https://github.com/philippgille/chromem-go/tree/v0.5.0/examples/rag-wikipedia-ollama))
- [Kangaroo](https://github.com/dbkangaroo/kangaroo) - AI 辅助 SQL 客户端

### 基础设施与部署

#### 云

- [Google Cloud](https://cloud.google.com/run/docs/tutorials/gpu-gemma2-with-ollama)
- [Fly.io](https://fly.io/docs/python/do-more/add-ollama/)
- [Koyeb](https://www.koyeb.com/deploy/ollama)
- [Harbor](https://github.com/av/harbor) - 以 Ollama 为默认后端的容器化大语言模型工具包

#### 包管理器

- [Pacman](https://archlinux.org/packages/extra/x86_64/ollama/)
- [Homebrew](https://formulae.brew.sh/formula/ollama)
- [Nix package](https://search.nixos.org/packages?show=ollama&from=0&size=50&sort=relevance&type=packages&query=ollama)
- [Helm Chart](https://artifacthub.io/packages/helm/ollama-helm/ollama)
- [Gentoo](https://github.com/gentoo/gentoo/tree/master/sci-ml/ollama)
- [Flox](https://flox.dev/blog/ollama-part-one)
- [Guix channel](https://codeberg.org/tusharhero/ollama-guix)
