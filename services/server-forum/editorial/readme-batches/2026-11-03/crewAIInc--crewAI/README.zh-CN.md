> 译注：中文全文由 Terra 翻译并对照[固定版本英文 README](https://github.com/crewAIInc/crewAI/blob/86dca29b536d700b69a97a3592857d5f9d4a8873/README.md)审校；上游采用 [MIT 许可证](https://github.com/crewAIInc/crewAI/blob/86dca29b536d700b69a97a3592857d5f9d4a8873/LICENSE)。商业服务、性能和隐私表述均为上游说明；代码示例保留原文。

<p align="center">
  <a href="https://github.com/crewAIInc/crewAI">
    <img src="https://raw.githubusercontent.com/crewAIInc/crewAI/86dca29b536d700b69a97a3592857d5f9d4a8873/docs/images/crewai_logo.png" width="600px" alt="Open source Multi-AI Agent orchestration framework">
  </a>
</p>
<p align="center" style="display: flex; justify-content: center; gap: 20px; align-items: center;">
  <a href="https://trendshift.io/repositories/11239" target="_blank">
    <img src="https://trendshift.io/api/badge/repositories/11239" alt="crewAIInc%2FcrewAI | Trendshift" style="width: 250px; height: 55px;" width="250" height="55"/>
  </a>
</p>

<p align="center">
  <a href="https://crewai.com">主页</a>
  ·
  <a href="https://crewai.com/open-source">开源</a>
  ·
  <a href="https://docs.crewai.com">文档</a>
  ·
  <a href="https://app.crewai.com">开始云端试用</a>
  ·
  <a href="https://blog.crewai.com">博客</a>
  ·
  <a href="https://community.crewai.com">论坛</a>
</p>

<p align="center">
  <a href="https://github.com/crewAIInc/crewAI">
    <img src="https://img.shields.io/github/stars/crewAIInc/crewAI" alt="GitHub Repo stars">
  </a>
  <a href="https://github.com/crewAIInc/crewAI/network/members">
    <img src="https://img.shields.io/github/forks/crewAIInc/crewAI" alt="GitHub forks">
  </a>
  <a href="https://github.com/crewAIInc/crewAI/issues">
    <img src="https://img.shields.io/github/issues/crewAIInc/crewAI" alt="GitHub issues">
  </a>
  <a href="https://github.com/crewAIInc/crewAI/pulls">
    <img src="https://img.shields.io/github/issues-pr/crewAIInc/crewAI" alt="GitHub pull requests">
  </a>
  <a href="https://opensource.org/licenses/MIT">
    <img src="https://img.shields.io/badge/License-MIT-green.svg" alt="License: MIT">
  </a>
</p>

<p align="center">
  <a href="https://pypi.org/project/crewai/">
    <img src="https://img.shields.io/pypi/v/crewai" alt="PyPI version">
  </a>
  <a href="https://pypi.org/project/crewai/">
    <img src="https://img.shields.io/pypi/dm/crewai" alt="PyPI downloads">
  </a>
  <a href="https://twitter.com/crewAIInc">
    <img src="https://img.shields.io/twitter/follow/crewAIInc?style=social" alt="Twitter Follow">
  </a>
</p>

### 快速灵活的多智能体自动化框架

> CrewAI 是开源 Python 框架，提供高层抽象和底层 API，用于构建可投入生产的多智能体工作流。
> 开发者可通过 Crews 实现智能体自主协作，通过 Flows 实现精确的事件驱动控制。

- **CrewAI Crews**：通过基于角色的 AI 智能体，优化自主性和协作智能。
- **CrewAI Flows**：构建事件驱动自动化，结合精确的工作流控制、单次 LLM 调用和对 Crews 的原生支持。

据项目介绍，已有超过 100,000 名开发者通过 [learn.crewai.com](https://learn.crewai.com) 社区课程获得认证；CrewAI 正迅速成为可用于生产的智能体自动化标准。

# CrewAI AMP Suite

对需要围绕 CrewAI 使用商业控制平面的组织，[CrewAI AMP Suite](https://crewai.com/amp) 提供托管部署、可观测性、治理、安全和企业支持。

你可以免费试用套件中的 [Crew Control Plane](https://app.crewai.com)。

## Crew Control Plane 主要功能

- **追踪与可观测性**：实时监控和追踪 AI 智能体及工作流，包括指标、日志和追踪记录。
- **统一控制平面**：集中管理、监控和扩展 AI 智能体与工作流的平台。
- **无缝集成**：连接现有企业系统、数据源和云基础设施。
- **高级安全**：内置安全与合规措施，支持安全部署和管理。
- **可操作的洞察**：通过实时分析与报告优化性能和决策。
- **24/7 全天候支持**：专门的企业支持，协助保持运行并快速解决问题。
- **本地和云端部署选项**：可根据安全与合规要求，在本地或云端部署 CrewAI AMP。

CrewAI AMP 面向希望将复杂业务流程转化为高效智能自动化的企业。

## 目录

- [使用 AI 构建](#build-with-ai)
- [为什么选择 CrewAI？](#why-crewai)
- [入门](#getting-started)
  - [学习资源](#learning-resources)
  - [理解 Flows 和 Crews](#understanding-flows-and-crews)
  - [安装](#1-installation)
  - [设置 Crew](#2-setting-up-your-crew)
  - [运行 Crew](#3-running-your-crew)
- [主要功能](#key-features)
- [示例](#examples)
  - [快速教程](#quick-tutorial)
  - [编写职位描述](#write-job-descriptions)
  - [旅行规划器](#trip-planner)
  - [股票分析](#stock-analysis)
  - [结合使用 Crews 和 Flows](#using-crews-and-flows-together)
- [将 Crew 连接到模型](#connecting-your-crew-to-a-model)
- [何时使用 CrewAI](#when-to-use-crewai)
- [贡献](#contribution)
- [遥测](#telemetry)
- [许可](#license)
- [常见问题（FAQ）](#frequently-asked-questions-faq)

<a id="build-with-ai"></a>
## 使用 AI 构建

使用 AI 编码智能体？一条命令即可让它学习 CrewAI 最佳实践：

**Claude Code:**
```shell
/plugin marketplace add crewAIInc/skills
/plugin install crewai-skills@crewai-plugins
/reload-plugins
```
当你提出相关 CrewAI 问题时，以下四项技能会自动启用：

| 技能 | 运行场景 |
|-------|--------------|
| `getting-started` | 创建项目框架、选择 `LLM.call()` / `Agent` / `Crew` / `Flow`、配置 `crew.jsonc` / `main.py` |
| `design-agent` | 配置智能体：角色、目标、背景、工具、LLM、记忆和防护措施 |
| `design-task` | 编写任务描述、依赖、结构化输出（`output_pydantic`、`output_json`）和人工审核 |
| `ask-docs` | 查询实时的 [CrewAI 文档 MCP 服务器](https://docs.crewai.com/mcp)，获取最新 API 详情 |

**Cursor、Codex、Windsurf 等（[skills.sh](https://skills.sh/crewaiinc/skills)）：**
```shell
npx skills add crewaiinc/skills
```

此命令安装官方 [CrewAI Skills](https://github.com/crewAIInc/skills)：一组结构化指令，帮助编码智能体创建 Flow 框架、配置 Crew、设计智能体和任务，并遵循 CrewAI 模式。

<a id="why-crewai"></a>
## 为什么选择 CrewAI？

<div align="center" style="margin-bottom: 30px;">
  <img src="https://raw.githubusercontent.com/crewAIInc/crewAI/86dca29b536d700b69a97a3592857d5f9d4a8873/docs/images/asset.png" alt="CrewAI Logo" width="100%">
</div>

CrewAI 通过 AI 智能体组成的 Crews 和事件驱动的 Flows，提供多智能体自动化所需的速度、灵活性和控制：

- **专用架构**：专为智能体编排设计，具有轻量 Python 核心和适用于实际自动化的基础组件。
- **高性能**：针对速度和低资源占用优化，加快执行。
- **灵活的底层定制**：可自由定制工作流、系统架构、智能体行为、内部提示词和执行逻辑。
- **适用于多种场景**：项目称其已在简单任务、复杂工作流及生产级自动化中证明有效。
- **强大的社区**：由快速增长的社区支持，其中有超过 **100,000 名获得认证**的开发者，提供支持与资源。

CrewAI 帮助开发者和团队构建兼顾简洁、灵活与生产级控制的智能自动化。

<a id="getting-started"></a>
## 入门

按照本教程设置并运行你的第一个 CrewAI 智能体。

[![CrewAI Getting Started Tutorial](https://img.youtube.com/vi/-kSOTtYzgEw/hqdefault.jpg)](https://www.youtube.com/watch?v=-kSOTtYzgEw "CrewAI Getting Started Tutorial")

<a id="learning-resources"></a>
### 学习资源

通过以下课程学习 CrewAI：

- [Multi AI Agent Systems with CrewAI](https://www.deeplearning.ai/short-courses/multi-ai-agent-systems-with-crewai/) - 掌握多智能体系统基础
- [Practical Multi AI Agents and Advanced Use Cases](https://www.deeplearning.ai/short-courses/practical-multi-ai-agents-and-advanced-use-cases-with-crewai/) - 深入了解高级实现

<a id="understanding-flows-and-crews"></a>
### 理解 Flows 和 Crews

CrewAI 提供两种互补方式，可共同用于构建复杂 AI 应用：

1. **Crews**：由具有自主能力的 AI 智能体组成团队，通过基于角色的协作完成复杂任务。Crews 支持：

   - 智能体之间自然、自主地决策
   - 动态任务委派与协作
   - 具有明确目标与专业知识的专门角色
   - 灵活的问题解决方式

2. **Flows**：可用于生产的事件驱动工作流，精确控制复杂自动化。Flows 提供：

   - 对实际场景执行路径的细粒度控制
   - 任务之间安全、一致的状态管理
   - 将 AI 智能体清晰集成到生产 Python 代码
   - 用条件分支处理复杂业务逻辑

将 Crews 和 Flows 结合可以：

- 构建复杂的生产级应用
- 在自主性和精确控制之间取得平衡
- 处理复杂的实际场景
- 保持清晰、易维护的代码结构

### 开始安装

按照以下步骤开始使用 CrewAI。完整过程见[安装指南](https://docs.crewai.com/en/installation)。

<a id="1-installation"></a>
### 1. 安装

CrewAI 要求 `Python >=3.10 and <3.14`。使用以下命令检查版本：

```bash
python3 --version
```

CrewAI 使用 [UV](https://docs.astral.sh/uv/) 管理依赖和软件包。如果尚未安装 `uv`，请先安装。

**macOS/Linux:**

```shell
curl -LsSf https://astral.sh/uv/install.sh | sh
```

如果系统没有 `curl`，可使用 `wget`：

```shell
wget -qO- https://astral.sh/uv/install.sh | sh
```

**Windows:**

```shell
powershell -ExecutionPolicy ByPass -c "irm https://astral.sh/uv/install.ps1 | iex"
```

如遇问题，请参阅 [UV 安装指南](https://docs.astral.sh/uv/getting-started/installation/)。

然后安装 CrewAI CLI：

```shell
uv tool install crewai
```

如果出现 `PATH` 警告，运行：

```shell
uv tool update-shell
```

如果在 Windows 遇到 `chroma-hnswlib==0.7.6` 构建错误（`fatal error C1083: Cannot open include file: 'float.h'`），请安装 [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/) 并选择 *Desktop development with C++*。

验证安装：

```shell
uv tool list
```

应看到类似输出：

```shell
crewai v0.102.0
- crewai
```

日后升级全局 CLI：

```shell
uv tool install crewai --upgrade
```

这只会升级**全局 `crewai` CLI 工具**。要升级项目虚拟环境内的 `crewai` 版本，请参阅[在项目中升级 CrewAI](https://docs.crewai.com/en/guides/migration/upgrading-crewai)。

<a id="2-setting-up-your-crew"></a>
### 2. 设置 Crew

`crewai create crew` 创建以 JSON 为核心的 Crew 项目。智能体位于 `agents/*.jsonc`，任务和 Crew 级设置位于 `crew.jsonc`，`crewai run` 直接加载该 JSON 定义。

```shell
crewai create crew <project_name>
```

此命令创建具有以下结构的新项目文件夹：

```
my_project/
├── .gitignore
├── .env
├── agents/
│   └── researcher.jsonc
├── crew.jsonc
├── knowledge/
├── pyproject.toml
├── README.md
├── skills/
└── tools/
```

如果需要包含 `crew.py`、`config/agents.yaml` 和 `config/tasks.yaml` 的旧版 Python/YAML 项目框架，运行：

```shell
crewai create crew <project_name> --classic
```

经典模式请参阅[使用注解](https://docs.crewai.com/en/learn/using-annotations)。

#### 要自定义项目，可以：

- 修改 `agents/*.jsonc`，定义每个智能体的角色、目标、背景、LLM、工具和行为。
- 修改 `crew.jsonc`，定义任务、流程和输入默认值。
- 在 `tools/` 中添加自定义工具，并以 `"custom:<name>"` 引用。
- 在 `knowledge/` 中添加可选知识文件，在 `skills/` 中添加技能文件。
- 在 `.env` 文件中添加环境变量。

在智能体和任务文本中使用 `{placeholder}` 值，然后在 `crew.jsonc` 的 `inputs` 下设置默认值。运行 `crewai run` 时，CLI 会提示输入缺失的值。

#### 顺序流程的简单 Crew 示例：

```shell
crewai create crew latest-ai-development
cd latest_ai_development
```

然后编辑生成的文件：

**agents/researcher.jsonc**

```jsonc
{
  "role": "{topic} Senior Data Researcher",
  "goal": "Uncover cutting-edge developments in {topic}",
  "backstory": "You're a seasoned researcher who finds relevant information and presents it clearly.",
  "llm": "openai/gpt-4o",
  "tools": ["SerperDevTool"],
  "settings": {
    "verbose": true
  }
}
```

**agents/reporting_analyst.jsonc**

```jsonc
{
  "role": "{topic} Reporting Analyst",
  "goal": "Create detailed reports based on {topic} data analysis and research findings",
  "backstory": "You're a meticulous analyst who turns complex data into clear, concise reports.",
  "llm": "openai/gpt-4o",
  "settings": {
    "verbose": true
  }
}
```

**crew.jsonc**

```jsonc
{
  "name": "Latest AI Development",
  "agents": ["researcher", "reporting_analyst"],
  "tasks": [
    {
      "name": "research_task",
      "description": "Conduct thorough research about {topic}. Find recent, relevant information.",
      "expected_output": "A list with 10 bullet points of the most relevant information about {topic}.",
      "agent": "researcher"
    },
    {
      "name": "reporting_task",
      "description": "Review the research and expand each topic into a full section for a report.",
      "expected_output": "A markdown report with the main topics, each with a full section of information. No fenced code blocks around the whole document.",
      "agent": "reporting_analyst",
      "context": ["research_task"],
      "output_file": "output/report.md",
      "markdown": true
    }
  ],
  "process": "sequential",
  "verbose": true,
  "inputs": {
    "topic": "AI Agents"
  }
}
```

<a id="3-running-your-crew"></a>
### 3. 运行 Crew

运行 Crew 前，在 `.env` 文件中设置所需密钥：

- 模型提供商的 API 密钥——参阅 [LLM 设置](https://docs.crewai.com/en/concepts/llms#setting-up-your-llm)
- 使用网页搜索时需提供 [Serper.dev](https://serper.dev/) API 密钥：`SERPER_API_KEY=YOUR_KEY_HERE`

然后在项目目录安装依赖并运行：

```shell
crewai install
crewai run
```

如果需要其他软件包，使用 `uv add <package-name>`。

控制台应显示输出，项目根目录应生成 `output/report.md`。

除了顺序流程，也可使用层级流程：它自动为指定 Crew 分配管理者，通过委派和结果验证协调任务规划与执行。[了解更多流程信息](https://docs.crewai.com/en/concepts/processes)。

以 Flow 为先的教程请参阅[快速入门](https://docs.crewai.com/en/quickstart)。

<a id="key-features"></a>
## 主要功能

CrewAI 为开发者提供从原型走向生产的智能体系统基础：在有帮助的地方自主协作，在关键环节明确控制工作流，并全程支持原生 Python 定制。

- **用 Crews 实现自主性**：通过角色、目标、工具和任务建立专业 AI 智能体团队。
- **用 Flows 实现控制**：构建包含状态、分支、路由及生产逻辑的事件驱动工作流。
- **无缝集成**：结合 Crews 和 Flows 创建复杂的实际自动化。
- **原生 Python 定制**：定制提示词、工具、执行路径、状态和集成。
- **智能体能力**：利用工具、记忆、知识、检查点、异步执行及 MCP/A2A 支持，构建能力更强的生产智能体。
- **生产就绪模式**：随着系统发展，添加确定性步骤、人工输入、结构化输出和检查点。
- **活跃社区**：有完善文档和超过 100,000 名认证开发者，提供支持与指导。

使用 CrewAI 构建强大、灵活且可用于生产的 AI 自动化。

<a id="examples"></a>
## 示例

可以在 [CrewAI-examples 仓库](https://github.com/crewAIInc/crewAI-examples?tab=readme-ov-file)测试不同的真实 AI Crew 示例：

- [落地页生成器](https://github.com/crewAIInc/crewAI-examples/tree/main/crews/landing_page_generator)
- [执行中接收人工输入](https://docs.crewai.com/en/learn/human-input-on-execution)
- [旅行规划器](https://github.com/crewAIInc/crewAI-examples/tree/main/crews/trip_planner)
- [股票分析](https://github.com/crewAIInc/crewAI-examples/tree/main/crews/stock_analysis)

<a id="quick-tutorial"></a>
### 快速教程

[![CrewAI Tutorial](https://img.youtube.com/vi/tnejrr-0a94/maxresdefault.jpg)](https://www.youtube.com/watch?v=tnejrr-0a94 "CrewAI Tutorial")

<a id="write-job-descriptions"></a>
### 编写职位描述

[查看此示例的代码](https://github.com/crewAIInc/crewAI-examples/tree/main/crews/job-posting)，或观看下方视频：

[![Jobs postings](https://img.youtube.com/vi/u98wEMz-9to/maxresdefault.jpg)](https://www.youtube.com/watch?v=u98wEMz-9to "Jobs postings")

<a id="trip-planner"></a>
### 旅行规划器

[查看此示例的代码](https://github.com/crewAIInc/crewAI-examples/tree/main/crews/trip_planner)，或观看下方视频：

[![Trip Planner](https://img.youtube.com/vi/xis7rWp-hjs/maxresdefault.jpg)](https://www.youtube.com/watch?v=xis7rWp-hjs "Trip Planner")

<a id="stock-analysis"></a>
### 股票分析

[查看此示例的代码](https://github.com/crewAIInc/crewAI-examples/tree/main/crews/stock_analysis)，或观看下方视频：

[![Stock Analysis](https://img.youtube.com/vi/e0Uj4yWdaAg/maxresdefault.jpg)](https://www.youtube.com/watch?v=e0Uj4yWdaAg "Stock Analysis")

<a id="using-crews-and-flows-together"></a>
### 结合使用 Crews 和 Flows

将 Crews 和 Flows 结合可创建复杂的自动化流水线。
CrewAI Flow 支持 `or_` 和 `and_` 等逻辑运算符来组合多个条件。可与 `@start`、`@listen` 或 `@router` 装饰器配合，创建复杂触发条件。

- `or_`：满足任一指定条件时触发。
- `and_`：满足所有指定条件时触发。

以下示例展示如何在一个 Flow 中编排多个 Crew：

```python
from crewai.flow.flow import Flow, listen, start, router, or_
from crewai import Crew, Agent, Task, Process
from pydantic import BaseModel

# Define structured state for precise control
class MarketState(BaseModel):
    sentiment: str = "neutral"
    confidence: float = 0.0
    recommendations: list = []

class AdvancedAnalysisFlow(Flow[MarketState]):
    @start()
    def fetch_market_data(self):
        # Demonstrate low-level control with structured state
        self.state.sentiment = "analyzing"
        return {"sector": "tech", "timeframe": "1W"}  # These parameters match the task description template

    @listen(fetch_market_data)
    def analyze_with_crew(self, market_data):
        # Show crew agency through specialized roles
        analyst = Agent(
            role="Senior Market Analyst",
            goal="Conduct deep market analysis with expert insight",
            backstory="You're a veteran analyst known for identifying subtle market patterns"
        )
        researcher = Agent(
            role="Data Researcher",
            goal="Gather and validate supporting market data",
            backstory="You excel at finding and correlating multiple data sources"
        )

        analysis_task = Task(
            description="Analyze {sector} sector data for the past {timeframe}",
            expected_output="Detailed market analysis with confidence score",
            agent=analyst
        )
        research_task = Task(
            description="Find supporting data to validate the analysis",
            expected_output="Corroborating evidence and potential contradictions",
            agent=researcher
        )

        # Demonstrate crew autonomy
        analysis_crew = Crew(
            agents=[analyst, researcher],
            tasks=[analysis_task, research_task],
            process=Process.sequential,
            verbose=True
        )
        return analysis_crew.kickoff(inputs=market_data)  # Pass market_data as named inputs

    @router(analyze_with_crew)
    def determine_next_steps(self):
        # Show flow control with conditional routing
        if self.state.confidence > 0.8:
            return "high_confidence"
        elif self.state.confidence > 0.5:
            return "medium_confidence"
        return "low_confidence"

    @listen("high_confidence")
    def execute_strategy(self):
        # Demonstrate complex decision making
        strategy_crew = Crew(
            agents=[
                Agent(role="Strategy Expert",
                      goal="Develop optimal market strategy")
            ],
            tasks=[
                Task(description="Create detailed strategy based on analysis",
                     expected_output="Step-by-step action plan")
            ]
        )
        return strategy_crew.kickoff()

    @listen(or_("medium_confidence", "low_confidence"))
    def request_additional_analysis(self):
        self.state.recommendations.append("Gather more data")
        return "Additional analysis required"
```

此示例展示如何：

1. 使用 Python 代码进行基本数据操作
2. 创建并执行 Crew，作为工作流中的步骤
3. 使用 Flow 装饰器管理操作顺序
4. 根据 Crew 结果实现条件分支

<a id="connecting-your-crew-to-a-model"></a>
## 将 Crew 连接到模型

CrewAI 支持通过多种连接方式使用各类 LLM。默认情况下，智能体查询模型时使用 OpenAI API；也有其他连接方式，例如通过 Ollama 工具配置智能体使用本地模型。

配置智能体与模型的连接，详见[将 CrewAI 连接到 LLM](https://docs.crewai.com/en/learn/llm-connections)。

<a id="when-to-use-crewai"></a>
## 何时使用 CrewAI

如果需求超出单次提示或聊天机器人，例如多步骤工作、专业智能体、工具调用、结构化输出、人工审核，或将自主推理与明确业务逻辑结合的工作流，可以使用 CrewAI。

CrewAI 特别适合以下需求：

- 协调具有明确角色和任务的多个智能体。
- 将智能体工作放入确定性的事件驱动工作流。
- 使用常规 Python 编写应用逻辑。
- 无需更换框架即可从实验转向生产。
- 随系统发展添加工具、记忆、检查点和异步执行。

<a id="contribution"></a>
## 贡献

CrewAI 是开源项目，欢迎贡献。完整的设置指南、分支约定和 PR 检查清单请参阅
[`.github/CONTRIBUTING.md`](https://github.com/crewAIInc/crewAI/blob/86dca29b536d700b69a97a3592857d5f9d4a8873/.github/CONTRIBUTING.md)。

快速开始：

```bash
git clone https://github.com/crewAIInc/crewAI.git
cd crewAI
uv sync --all-groups --all-extras
uv run pre-commit install
```

```bash
# Tests
uv run pytest lib/crewai/tests/ -x -q

# Type checks
uv run mypy lib/
```

### 为文档作贡献

[docs.crewai.com](https://docs.crewai.com) 网站由 [Mintlify](https://www.mintlify.com/) 从 `docs/` 发布。文档使用基于目录的版本管理：对 `docs/edge/<lang>/...`（例如 `docs/edge/en/concepts/agents.mdx`）的修改会立即出现在 **Edge** 版本选择项中，并在下次发布时冻结到 `docs/v<X.Y.Z>/` 的新版本快照。冻结快照不可变；没有 `[docs-freeze]` 标题前缀而修改快照的 PR 会被 CI 拒绝。发布 CLI（`devtools release`）自动处理冻结；贡献指南见
[`.github/CONTRIBUTING.md`](https://github.com/crewAIInc/crewAI/blob/86dca29b536d700b69a97a3592857d5f9d4a8873/.github/CONTRIBUTING.md)，发布工具见
[`lib/devtools/README.md`](https://github.com/crewAIInc/crewAI/blob/86dca29b536d700b69a97a3592857d5f9d4a8873/lib/devtools/README.md)。

<a id="telemetry"></a>
## 遥测

CrewAI 使用匿名遥测收集使用数据，主要用于了解最常用的功能、集成和工具，以帮助改进该库。

除所述例外情况外，项目称**不会收集**提示词、任务描述、智能体背景或目标、工具使用、API 调用与响应、智能体处理的任何数据，以及秘密和环境变量。启用 `share_crew` 功能时，会收集包括任务描述、智能体背景或目标及其他特定属性在内的详细数据，以提供更深入的洞察。用户可将环境变量 OTEL_SDK_DISABLED 设置为 true 来禁用遥测。

收集的数据包括：

- CrewAI 版本
  - 用于了解有多少用户在使用最新版本
- Python 版本
  - 用于决定重点支持哪些版本
- 一般操作系统信息（如 CPU 数量、macOS/Windows/Linux）
  - 用于确定重点支持的系统，以及是否开发特定系统功能
- Crew 中智能体和任务的数量
  - 用于确保内部测试覆盖类似场景，并向用户介绍最佳实践
- 使用的 Crew 流程
  - 用于了解应重点投入的方向
- 智能体是否使用记忆或允许委派
  - 用于了解是否需要改进或移除这些功能
- 任务是并行还是顺序执行
  - 用于了解是否应更关注并行执行
- 使用的语言模型
  - 改善对最常用语言的支持
- Crew 中智能体的角色
  - 了解总体使用场景，以构建更好的相关工具、集成和示例
- 可用工具的名称
  - 了解公开工具中哪些最常用，以便改进

用户可选择启用进一步遥测：将 Crew 的 `share_crew` 属性设为 `True`，共享完整遥测数据。启用 `share_crew` 后会收集详细的 Crew 和任务执行数据，包括任务的 `goal`、`backstory`、`context` 和 `output`，以便更深入了解使用模式；是否共享由用户选择。

<a id="license"></a>
## 许可

CrewAI 基于 [MIT 许可证](https://github.com/crewAIInc/crewAI/blob/main/LICENSE).

<a id="frequently-asked-questions-faq"></a>
## 常见问题（FAQ）

### 一般问题

- [CrewAI 究竟是什么？](#q-what-exactly-is-crewai)
- [如何安装 CrewAI？](#q-how-do-i-install-crewai)
- [CrewAI 是独立框架吗？](#q-is-crewai-a-standalone-framework)
- [CrewAI 是开源的吗？](#q-is-crewai-open-source)
- [CrewAI 会收集用户数据吗？](#q-does-crewai-collect-data-from-users)

### 功能与能力

- [CrewAI 能处理复杂场景吗？](#q-can-crewai-handle-complex-use-cases)
- [能将 CrewAI 与本地 AI 模型一起使用吗？](#q-can-i-use-crewai-with-local-ai-models)
- [Crews 与 Flows 有何不同？](#q-what-makes-crews-different-from-flows)
- [CrewAI 支持微调或训练自定义模型吗？](#q-does-crewai-support-fine-tuning-or-training-custom-models)

### 资源与社区

- [哪里可以找到 CrewAI 的实际示例？](#q-where-can-i-find-real-world-crewai-examples)
- [如何为 CrewAI 作贡献？](#q-how-can-i-contribute-to-crewai)

### 企业功能

- [CrewAI AMP 提供哪些额外功能？](#q-what-additional-features-does-crewai-amp-offer)
- [CrewAI AMP 支持云端和本地部署吗？](#q-is-crewai-amp-available-for-cloud-and-on-premise-deployments)
- [可以免费试用 CrewAI AMP 吗？](#q-can-i-try-crewai-amp-for-free)

<a id="q-what-exactly-is-crewai"></a>
### 问：CrewAI 究竟是什么？

答：CrewAI 是轻量、快速的 Python 框架，专为编排自主 AI 智能体和可用于生产的智能体工作流而构建。

<a id="q-how-do-i-install-crewai"></a>
### 问：如何安装 CrewAI？

答：使用 [UV](https://docs.astral.sh/uv/) 安装 CrewAI CLI：

```shell
uv tool install crewai
```

然后用 `crewai create crew <project_name>` 创建项目，运行 `crewai install`，再用 `crewai run` 启动。详情参阅[安装指南](https://docs.crewai.com/en/installation)。

<a id="q-is-crewai-a-standalone-framework"></a>
### 问：CrewAI 是独立框架吗？

答：是。CrewAI 是独立的 Python 框架，提供自己的智能体、任务、Crew、Flow、工具和编排基础组件。

<a id="q-can-crewai-handle-complex-use-cases"></a>
### 问：CrewAI 能处理复杂场景吗？

答：是。项目称 CrewAI 适用于简单和高度复杂的实际场景，提供高层和底层深度定制，包括内部提示词及复杂工作流编排。

<a id="q-can-i-use-crewai-with-local-ai-models"></a>
### 问：能将 CrewAI 与本地 AI 模型一起使用吗？

答：可以。CrewAI 支持包括本地模型在内的多种语言模型，可通过 Ollama 和 LM Studio 等工具集成。详情参阅 [LLM 连接文档](https://docs.crewai.com/en/learn/llm-connections)。

<a id="q-what-makes-crews-different-from-flows"></a>
### 问：Crews 与 Flows 有何不同？

答：Crews 提供自主智能体协作，适合需要灵活决策和动态交互的任务。Flows 提供精确的事件驱动控制，适合管理详细执行路径和安全状态。两者可以结合使用。

<a id="q-is-crewai-open-source"></a>
### 问：CrewAI 是开源的吗？

答：是，CrewAI 开源，并积极鼓励社区贡献与协作。

<a id="q-does-crewai-collect-data-from-users"></a>
### 问：CrewAI 会收集用户数据吗？

答：项目称 CrewAI 收集匿名遥测数据用于改进，除非用户明确启用，否则不会收集提示词、任务或 API 响应等敏感数据。

<a id="q-where-can-i-find-real-world-crewai-examples"></a>
### 问：哪里可以找到 CrewAI 的实际示例？

答：可在 [CrewAI-examples 仓库](https://github.com/crewAIInc/crewAI-examples)查看旅行规划、股票分析和职位发布等实际示例。

<a id="q-how-can-i-contribute-to-crewai"></a>
### 问：如何为 CrewAI 作贡献？

答：欢迎贡献！Fork 仓库、创建分支、完成修改并提交 pull request。详细指南参阅 [`.github/CONTRIBUTING.md`](https://github.com/crewAIInc/crewAI/blob/86dca29b536d700b69a97a3592857d5f9d4a8873/.github/CONTRIBUTING.md)。

<a id="q-what-additional-features-does-crewai-amp-offer"></a>
### 问：CrewAI AMP 提供哪些额外功能？

答：CrewAI AMP 提供统一控制平面、实时可观测性、安全集成、高级安全、可操作的洞察和 24/7 全天候专属企业支持。

<a id="q-is-crewai-amp-available-for-cloud-and-on-premise-deployments"></a>
### 问：CrewAI AMP 支持云端和本地部署吗？

答：是，CrewAI AMP 支持云端和本地部署，以满足企业的特定安全与合规要求。

<a id="q-can-i-try-crewai-amp-for-free"></a>
### 问：可以免费试用 CrewAI AMP 吗？

答：可以。你可免费访问 [Crew Control Plane](https://app.crewai.com)，体验 CrewAI AMP Suite 的一部分。

<a id="q-does-crewai-support-fine-tuning-or-training-custom-models"></a>
### 问：CrewAI 支持微调或训练自定义模型吗？

答：可以。CrewAI 可集成自训练或微调模型，为智能体提供领域知识并提高准确性。

### 问：CrewAI 智能体可以与外部工具和 API 交互吗？

答：可以。CrewAI 智能体可集成外部工具、API 和数据库，使用实际数据与资源。

### 问：CrewAI 适合生产环境吗？

答：是。项目称 CrewAI 采用生产级模式设计，支持可靠、稳定、可扩展的智能体工作流。

### 问：CrewAI 的可扩展性如何？

答：项目称 CrewAI 具有很高的可扩展性，既支持简单自动化，也支持同时涉及多个智能体和复杂任务的大规模工作流。

### 问：CrewAI 提供调试和监控工具吗？

答：是。CrewAI AMP 包含高级调试、追踪和实时可观测性功能，帮助管理自动化并排除故障。

### 问：CrewAI 支持哪些编程语言？

答：CrewAI 主要基于 Python，也可通过 API 集成用其他编程语言编写的服务和 API。

### 问：CrewAI 为初学者提供学习资源吗？

答：是。CrewAI 通过 learn.crewai.com 提供适合初学者的教程、课程和文档，支持不同水平的开发者。

### 问：CrewAI 能自动化需要人工参与的工作流吗？

答：是。CrewAI 支持人工参与的工作流，让人类专家与 AI 智能体协作辅助决策。
