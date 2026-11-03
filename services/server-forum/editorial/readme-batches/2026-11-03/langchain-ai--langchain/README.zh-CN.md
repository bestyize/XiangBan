<div align="center">
  <a href="https://docs.langchain.com/oss/python/langchain/overview">
    <picture>
      <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/langchain-ai/langchain/2775506076baf2cba8126b21b9e63424fcd97ab9/.github/images/logo-dark.svg">
      <source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/langchain-ai/langchain/2775506076baf2cba8126b21b9e63424fcd97ab9/.github/images/logo-light.svg">
      <img alt="LangChain Logo" src="https://raw.githubusercontent.com/langchain-ai/langchain/2775506076baf2cba8126b21b9e63424fcd97ab9/.github/images/logo-dark.svg" width="50%">
    </picture>
  </a>
</div>

<div align="center">
  <h3>智能体工程平台。</h3>
</div>

<div align="center">
  <a href="https://opensource.org/licenses/MIT" target="_blank"><img src="https://img.shields.io/pypi/l/langchain" alt="PyPI - License"></a>
  <a href="https://pypistats.org/packages/langchain" target="_blank"><img src="https://img.shields.io/pepy/dt/langchain" alt="PyPI - Downloads"></a>
  <a href="https://pypi.org/project/langchain/#history" target="_blank"><img src="https://img.shields.io/pypi/v/langchain?label=%20" alt="Version"></a>
  <a href="https://x.com/langchain_oss" target="_blank"><img src="https://img.shields.io/twitter/url/https/twitter.com/langchain_oss.svg?style=social&label=Follow%20%40LangChain" alt="Twitter / X"></a>
</div>

<br>

LangChain 是一个用于构建智能体和大语言模型应用的框架。它帮助你连接可互操作的组件和第三方集成，简化 AI 应用开发，同时在底层技术演进时让你的选择更具适应性。

> [!TIP]
> 刚开始使用？可以了解 **[Deep Agents](https://docs.langchain.com/oss/python/deepagents/)**：这是一个基于 LangChain 的更高层级软件包，为智能体内置了规划、子智能体、文件系统使用等常见模式的能力。

## 快速开始

```bash
uv add langchain
```

```python
from langchain.chat_models import init_chat_model

model = init_chat_model("openai:gpt-5.5")
result = model.invoke("Hello, world!")
```

如果你需要更高级的定制或智能体编排，可以了解 [LangGraph](https://github.com/langchain-ai/langgraph)，这是我们用于构建可控智能体工作流的框架。

如需对应的 JS/TS 库，请了解 [LangChain.js](https://github.com/langchain-ai/langchainjs)。

> [!TIP]
> 有关 AI 智能体和大语言模型应用的开发、调试与部署，请参阅 [LangSmith](https://docs.langchain.com/langsmith/home)。

## LangChain 生态系统

LangChain 框架既可独立使用，也可与 LangChain 产品无缝集成，为构建大语言模型应用的开发者提供完整的工具套件。

- **[Deep Agents](https://docs.langchain.com/oss/python/deepagents/)** —— 构建能够规划、使用子智能体和文件系统来处理复杂任务的智能体
- **[LangGraph](https://docs.langchain.com/oss/python/langgraph/overview)** —— 利用我们的底层智能体编排框架，构建能够可靠处理复杂任务的智能体
- **[Integrations](https://docs.langchain.com/oss/python/integrations/providers/overview)** —— 聊天与嵌入模型、工具与工具包等
- **[LangSmith](https://www.langchain.com/langsmith)** —— 面向大语言模型应用的智能体评估、可观测性和调试
- **[LangSmith Deployment](https://docs.langchain.com/langsmith/deployments)** —— 通过专为长时间运行的有状态工作流构建的平台部署和扩展智能体

## 为什么使用 LangChain？

LangChain 通过模型、嵌入、向量存储等的标准接口，帮助开发者构建大语言模型应用。

- **实时数据增强** —— 利用 LangChain 丰富的模型提供商、工具、向量存储、检索器等集成，轻松将大语言模型连接到多种数据源及外部或内部系统
- **模型互操作性** —— 工程团队可以在尝试寻找最适合应用需求的方案时更换模型。随着行业前沿演进，LangChain 的抽象可以帮助你快速适应并保持进度
- **快速原型开发** —— 使用 LangChain 模块化、基于组件的架构快速构建和迭代大语言模型应用。无需从头重建即可测试不同方法与工作流，加快开发周期
- **面向生产的功能** —— 通过 LangSmith 等集成内置监控、评估和调试支持，部署可靠的应用。使用经过实践检验的模式和最佳实践进行扩展
- **活跃的社区和生态系统** —— 使用丰富的集成、模板和社区贡献组件。通过活跃的开源社区获得持续改进，并了解最新的 AI 发展
- **灵活的抽象层** —— 在适合需求的抽象层工作：从便于快速上手的高级链，到提供细粒度控制的底层组件。LangChain 可以随应用复杂度增长而扩展

---

## 资源

- [文档](https://docs.langchain.com/oss/python/langchain/overview) —— 概念概览和指南
- [LangChain 生态系统概览](https://docs.langchain.com/oss/python/concepts/products) —— LangChain、LangGraph 和 Deep Agents 如何协同
- [API 参考](https://reference.langchain.com/python) —— 所有公开类、函数和类型的完整参考
- [讨论](https://forum.langchain.com/c/oss-product-help-lc-and-lg/langchain/14) —— 用于技术问题、想法和反馈的社区论坛
- [LangChain Academy](https://academy.langchain.com/) —— LangChain 团队制作的有关 LangChain 库和产品的全面免费课程
- [贡献指南](https://docs.langchain.com/oss/python/contributing/overview) —— 如何贡献及寻找适合新手的问题
- [行为准则](https://github.com/langchain-ai/langchain/?tab=coc-ov-file) —— 社区指南和标准
