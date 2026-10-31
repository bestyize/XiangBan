---
id: community-20261031-share-learn-03
title: 补工具链基础：用 Missing Semester 与 Git 文档整理一个练习仓库
category: share-learn
tags:
- 学习资料
- 工作流
- Git
---

会写代码却不敢碰终端和 Git，并不罕见。把工具链当作单独的学习对象，比在真实项目出问题时临时搜索危险命令更稳妥。

MIT 的 Missing Semester 课程围绕计算机工具提供学习入口，Git 官方文档则适合核对具体命令语义。建议新建一个只放练习文件的仓库，先体验查看状态、查看差异、提交与读取历史，不在重要项目里练习删除和重置。

每次操作前先写出期待结果，再执行命令验证。比如改两行文本后，用 `git diff` 观察未暂存差异；暂存后再理解暂存区与工作区并不是同一个概念。不要为了让状态变干净就删掉自己还没理解的文件。

把常用命令记成“场景—命令—预期输出”，比一整页没有背景的命令清单更容易复用。涉及 worktree、忽略规则和冲突处理时，回到官方对应章节，不把不同目的的命令互相替代。

练习结束保留仓库历史与说明文件，记录一次操作为什么产生那个结果。工具熟练度的目标是知道当前状态和下一步影响，而不是能背出多少短命令。

资料：[MIT Missing Semester 课程](https://missing.csail.mit.edu/)；[Git diff 官方文档](https://git-scm.com/docs/git-diff)；[Git 忽略规则文档](https://git-scm.com/docs/gitignore)。
