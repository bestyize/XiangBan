---
id: community-20261031-tech-productivity-05
title: 同时让 Claude Code 与 Codex 做事：用 worktree 隔离文件，再合并结果
category: tech-productivity
tags:
- claude-code
- codex
- 工作流
---

两个编程会话同时修改同一个目录，即使任务看起来不同，也可能共同碰到依赖文件、路由或生成产物。分支隔离提交历史，独立 worktree 还能隔离实际工作文件。

先划清任务归属，例如一个会话负责只读接口，另一个负责独立页面；共享入口由一个人接入。用 Git 的 worktree 为任务建立独立目录与分支，再分别让 Claude Code 或 Codex 在对应目录工作。工具提供托管 worktree 时，也要确认当前目录和分支。

可以先运行 `git worktree list` 查看已有工作区。创建、删除和合并按 Git 官方说明操作；删除前检查是否有未提交文件。不要把“临时工作区”理解为可以随意丢弃的缓存。

两个工作区各自通过相关验证后，按依赖顺序整合提交。共享配置和数据库迁移需要人工检查，即使 Git 没有文本冲突，也可能存在行为冲突。

密钥、依赖目录和本地配置未必自动复制到新 worktree，缺少时应按项目约定准备，不能把秘密提交进仓库来解决。最终交接说明带上提交号、验证命令和需要根分支接入的位置。

资料：[Git worktree 官方文档](https://git-scm.com/docs/git-worktree)；[Codex worktree 文档](https://learn.chatgpt.com/docs/environments/git-worktrees)；[Claude Code 常用工作流](https://code.claude.com/docs/en/common-workflows)。
