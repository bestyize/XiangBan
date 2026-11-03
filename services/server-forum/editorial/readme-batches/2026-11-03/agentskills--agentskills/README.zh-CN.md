> 本文由 Agent Skills 项目的 [README 原文](https://github.com/agentskills/agentskills/blob/69ef37e9424c0a7ea9dd2293b559e43ec8176379/README.md)翻译，原文作者为 Agent Skills 项目贡献者，依据 [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/) 使用。本页为非官方中文译文，并将相对链接改为对应版本的绝对链接。

# Agent Skills

[![Discord](https://img.shields.io/badge/Discord-Join-5865F2?logo=discord&logoColor=white)](https://discord.gg/MKPE9g8aUy)

为 AI 智能体提供新能力和专业知识的一种标准化方式。

## 什么是 Agent Skills？

Agent Skills 是一种轻量、开放的格式，用于通过专业知识和工作流程扩展 AI 智能体的能力。

技能的核心是一个包含 `SKILL.md` 文件的文件夹。此文件包含元数据（至少包括 `name` 和 `description`）以及指导智能体执行特定任务的说明。技能还可以打包脚本、参考资料、模板和其他资源。

```
my-skill/
├── SKILL.md          # Required: metadata + instructions
├── scripts/          # Optional: executable code
├── references/       # Optional: documentation
├── assets/           # Optional: templates, resources
└── ...               # Any additional files or directories
```

## 为什么使用 Agent Skills？

智能体的能力不断增强，但往往缺少可靠完成实际工作所需的上下文。技能通过把流程知识以及公司、团队和用户特定的上下文打包成可移植、受版本控制的文件夹来解决这一问题，智能体可按需加载这些文件夹。这为智能体提供：

- **领域专业知识**：将法律审查流程、数据分析管道、演示文稿格式等专业知识整理成可复用的说明和资源。
- **可重复的工作流程**：将多步骤任务转化为一致、可审计的流程。
- **跨产品复用**：创建一次技能，即可在任何兼容技能的智能体中使用。

## Agent Skills 如何工作？

智能体通过**渐进式披露**分三个阶段加载技能：

1. **发现**：启动时，智能体只加载每个可用技能的名称和描述，信息足以判断何时可能相关。

2. **激活**：当任务与某个技能的描述匹配时，智能体将完整的 `SKILL.md` 说明读入上下文。

3. **执行**：智能体遵循说明，并可按需执行随附代码或加载引用的文件。

完整说明仅在任务需要时加载，因此智能体能够保留许多可用技能，同时只占用少量上下文。

## 可以在哪里使用 Agent Skills？

许多 AI 工具和智能体客户端都支持 Agent Skills，参阅[客户端展示](https://agentskills.io/clients)了解其中的一部分！

## 开始使用

- **[文档](https://agentskills.io)** — 指南和教程
- **[规范](https://agentskills.io/specification)** — 格式详情
- **[技能示例](https://github.com/anthropics/skills)** — 看看可以实现什么
- **[Discord](https://discord.gg/MKPE9g8aUy)** — 分享你正在构建的东西！

## 开放开发

Agent Skills 格式最初由 [Anthropic](https://www.anthropic.com/) 开发，作为开放标准发布，并已被越来越多的智能体产品采用。该标准欢迎更广泛生态的贡献，参与方式请参阅 [`CONTRIBUTING.md`](https://github.com/agentskills/agentskills/blob/69ef37e9424c0a7ea9dd2293b559e43ec8176379/CONTRIBUTING.md)。

## 许可证

本仓库的代码采用 [Apache 2.0](https://github.com/agentskills/agentskills/blob/69ef37e9424c0a7ea9dd2293b559e43ec8176379/LICENSE)。文档采用 [CC-BY-4.0](https://creativecommons.org/licenses/by/4.0/)。详情请参阅各个目录。
