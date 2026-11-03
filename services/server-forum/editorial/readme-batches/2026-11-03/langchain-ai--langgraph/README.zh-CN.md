<div align="center">
  <a href="https://www.langchain.com/langgraph">
    <picture>
      <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/langchain-ai/langgraph/3c916d750575b22c464790e9774494d810ccd322/.github/images/logo-dark.svg">
      <source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/langchain-ai/langgraph/3c916d750575b22c464790e9774494d810ccd322/.github/images/logo-light.svg">
      <img alt="LangGraph Logo" src="https://raw.githubusercontent.com/langchain-ai/langgraph/3c916d750575b22c464790e9774494d810ccd322/.github/images/logo-dark.svg" width="50%">
    </picture>
  </a>
</div>

<div align="center">
  <h3>用于构建有状态智能体的底层编排框架。</h3>
</div>

<div align="center">
  <a href="https://opensource.org/licenses/MIT" target="_blank"><img src="https://img.shields.io/pypi/l/langgraph" alt="PyPI - License"></a>
  <a href="https://pypistats.org/packages/langgraph" target="_blank"><img src="https://img.shields.io/pepy/dt/langgraph" alt="PyPI - Downloads"></a>
  <a href="https://pypi.org/project/langgraph/" target="_blank"><img src="https://img.shields.io/pypi/v/langgraph.svg?label=%20" alt="Version"></a>
  <a href="https://x.com/langchain_oss" target="_blank"><img src="https://img.shields.io/twitter/url/https/twitter.com/langchain_oss.svg?style=social&label=Follow%20%40LangChain" alt="Twitter / X"></a>
</div>

<br>

LangGraph 是一个用于构建、管理和部署长时间运行的有状态智能体的底层编排框架。项目称它受到 Klarna、Replit、Elastic 等致力于智能体发展的公司的信赖。

```bash
pip install -U langgraph
```

> [!TIP]
> 如果你想快速构建智能体，请查看 **[Deep Agents](https://docs.langchain.com/oss/python/deepagents/overview)**——这是构建在 LangGraph 之上的更高层软件包，可用于构建能规划、使用子智能体并利用文件系统处理复杂任务的智能体。

如需相应的 JS/TS 库，请查看 [LangGraph.js](https://github.com/langchain-ai/langgraphjs)以及 [JS 文档](https://docs.langchain.com/oss/javascript/langgraph/overview).

## 为什么使用 LangGraph？

LangGraph 为*任何*长时间运行的有状态工作流或智能体提供底层基础设施：

- **[持久执行](https://docs.langchain.com/oss/python/langgraph/durable-execution)**——构建能够经受故障、长时间运行并从中断位置自动恢复的智能体。
- **[人在回路中](https://docs.langchain.com/oss/python/langgraph/interrupts)**——通过在执行过程中的任意时刻检查和修改智能体状态，引入人工监督。
- **[全面记忆](https://docs.langchain.com/oss/python/langgraph/memory)**——创建真正有状态的智能体，同时具备用于持续推理的短期工作记忆与跨会话的长期持久记忆。
- **[使用 LangSmith 调试](https://www.langchain.com/langsmith)**——使用可追踪执行路径、捕获状态转换并提供详细运行指标的可视化工具，深入了解复杂的智能体行为。
- **[面向生产的部署](https://docs.langchain.com/langsmith/deployments)**——使用为有状态、长时间运行的工作流所面临的独特挑战设计的可扩展基础设施，部署复杂的智能体系统。

> [!TIP]
> 有关开发、调试和部署 AI 智能体及 LLM 应用，请参阅 [LangSmith](https://docs.langchain.com/langsmith/home).

## LangGraph 生态

LangGraph 可独立使用，也可与 LangChain 产品集成，为开发者提供构建智能体所需的一整套工具。

可以将 LangGraph 与以下产品配合使用，改进 LLM 应用开发：

- [Deep Agents](https://docs.langchain.com/oss/python/deepagents/overview)——构建能够规划、使用子智能体并利用文件系统处理复杂任务的智能体。
- [LangChain](https://docs.langchain.com/oss/python/langchain/overview)——提供集成和可组合组件，简化 LLM 应用开发。
- [LangSmith](https://www.langchain.com/langsmith)——帮助进行智能体评估与可观测性分析。可调试表现不佳的 LLM 应用运行、评估智能体轨迹、观察生产环境并持续改进性能。
- [LangSmith Deployment](https://docs.langchain.com/langsmith/deployments)——使用专为长时间运行的有状态工作流打造的部署平台，部署和扩展智能体。在团队间发现、复用、配置和共享智能体，并在 [LangSmith Studio](https://docs.langchain.com/langsmith/studio) 中通过可视化原型快速迭代。

---

## 文档

- [docs.langchain.com](https://docs.langchain.com/oss/python/langgraph/overview)——完整文档，包括概念概览和指南
- [reference.langchain.com/python/langgraph](https://reference.langchain.com/python/langgraph)——LangGraph 软件包的 API 参考文档
- [LangGraph 快速入门](https://docs.langchain.com/oss/python/langgraph/quickstart)——开始使用 LangGraph 构建应用
- [Chat LangChain](https://chat.langchain.com/)——与 LangChain 文档对话并获取问题的答案

**讨论**：访问 [LangChain 论坛](https://forum.langchain.com)，与社区交流并分享技术问题、想法和反馈。

## 其他资源

- **[指南](https://docs.langchain.com/oss/python/learn)**——针对流式处理、添加记忆和持久化以及设计模式（如分支、子图等）等主题提供快速、可操作的代码片段。
- **[LangChain Academy](https://academy.langchain.com/courses/intro-to-langgraph)**——通过免费、系统的课程学习 LangGraph 基础知识。
- **[案例研究](https://www.langchain.com/built-with-langgraph)**——了解行业领军者如何使用 LangGraph 大规模交付 AI 应用。
- [贡献指南](https://docs.langchain.com/oss/python/contributing/overview)——了解如何为 LangChain 项目做贡献并寻找适合新手的问题。
- [行为准则](https://github.com/langchain-ai/langchain/?tab=coc-ov-file)——社区指南及参与标准。

---

## 致谢

LangGraph 受到 [Pregel](https://research.google/pubs/pub37252/) 和 [Apache Beam](https://beam.apache.org/) 的启发。 公共接口借鉴了 [NetworkX](https://networkx.org/documentation/latest/)。 LangGraph 由 LangChain 的创建者 LangChain Inc 构建，但可以不使用 LangChain 单独使用。
