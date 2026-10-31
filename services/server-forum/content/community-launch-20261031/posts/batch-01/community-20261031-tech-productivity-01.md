---
id: community-20261031-tech-productivity-01
title: 给 Claude Code 和 Codex 写项目说明：把长期规则与本次任务分开
category: tech-productivity
tags:
- claude-code
- codex
- 提示词
- 工作流
---

每次打开项目都重讲测试命令，很容易遗漏；把一次性需求永久写入项目规则，又容易让未来任务跑偏。项目说明适合保存长期有效、能够验证的约定。

Claude Code 官方文档介绍了项目记忆文件，Codex 官方文档说明了 AGENTS.md 的发现和层级规则。两种工具的加载机制应分别核对，不要因为都能读 Markdown，就假设覆盖顺序完全相同。

一个实用的最小内容是：项目入口、允许修改的职责范围、常用验证命令、生成文件的位置和必须保留的协议约束。例如“修改接口字段时同步调用方；只运行相关测试”，比“写出高质量代码”更容易执行。

本次要实现什么、截止时间和验收样例放在任务消息里。临时排障得到的猜测，不要未经验证就升级成永久规则。涉及账号或密钥，只写获取途径，不写真实值。

写完后用一个小任务检查工具实际读到了哪些说明，并观察是否按正确目录执行命令。如果同目录存在互相矛盾的文件，先整理冲突，不要继续叠加更长的提示词。项目规则的价值在于降低歧义，而不是把仓库变成第二份聊天记录。

资料：[Claude Code 项目记忆说明](https://code.claude.com/docs/en/memory)；[Codex 项目规则文档](https://learn.chatgpt.com/docs/agent-configuration/agents-md.md)。
