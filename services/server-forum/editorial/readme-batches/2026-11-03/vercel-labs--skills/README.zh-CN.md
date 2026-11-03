> 译注：npm 软件包网页在本次检查中返回 HTTP 403；可通过 [npm 官方注册表](https://registry.npmjs.org/skills)查看包信息。

# skills

开放智能体技能生态的命令行工具。

智能体技能是可复用的说明，指导编码智能体完成代码审查、使用库等任务。可在 [skills.sh](https://skills.sh) 发现技能。

<!-- agent-list:start -->
支持 **OpenCode**、**Claude Code**、**Codex**、**Cursor** 和[另外 75 种智能体](https://github.com/vercel-labs/skills/blob/7a5e920bead4b1039a6d6df5d7e1163b7b09423a/docs/agents.md).
<!-- agent-list:end -->

<p>
  <a href="https://vercel.com/labs#labs-products"><img alt="Vercel Labs Product" src="https://img.shields.io/badge/LABS-PRODUCT-0a0a0a.svg?style=for-the-badge&amp;logo=Vercel&amp;labelColor=000000" height="28"></a>
  <a href="https://www.npmjs.com/package/skills"><img alt="npm version: skills" src="https://img.shields.io/npm/v/skills.svg?style=for-the-badge&amp;labelColor=000000" height="28"></a>
  <a href="https://github.com/vercel-labs/skills/blob/7a5e920bead4b1039a6d6df5d7e1163b7b09423a/LICENSE"><img alt="License: MIT" src="https://img.shields.io/github/license/vercel-labs/skills.svg?style=for-the-badge&amp;labelColor=000000" height="28"></a>
  <a href="https://www.npmjs.com/package/skills"><img alt="npm downloads per month: skills" src="https://img.shields.io/npm/dm/skills.svg?style=for-the-badge&amp;labelColor=000000&amp;label=npm%20downloads" height="28"></a>
  <a href="https://skills.sh/vercel-labs/skills"><img alt="skills.sh" src="https://skills.sh/b/vercel-labs/skills?style=for-the-badge" height="28"></a>
</p>

## 安装技能

需要 Node.js 22.20 或更高版本。在项目目录运行：

```bash
npx skills add vercel-labs/agent-skills
```

选择要安装的技能和目标编码智能体。技能默认安装到当前项目；添加 `-g` 可使其跨项目可用。

```bash
# Choose one skill and agent
npx skills add vercel-labs/agent-skills --skill web-design-guidelines -a claude-code

# Browse available skills
npx skills find
```

有关来源格式、私有仓库和全部选项，请参阅[安装参考](https://github.com/vercel-labs/skills/blob/7a5e920bead4b1039a6d6df5d7e1163b7b09423a/docs/commands.md#skills-add)。

## 其他命令

| 命令 | 说明 |
| --------------------------------- | -------------------------------- |
| `npx skills find [query]`         | 搜索技能 |
| `npx skills list`                 | 列出已安装技能 |
| `npx skills use <source>@<skill>` | 无需安装即可使用一个技能 |
| `npx skills check`                | 检查可用更新 |
| `npx skills update [skills...]`   | 更新已安装技能 |
| `npx skills remove [skills...]`   | 移除已安装技能 |
| `npx skills init [name]`          | 创建技能模板 |

请阅读 [CLI 参考](https://github.com/vercel-labs/skills/blob/7a5e920bead4b1039a6d6df5d7e1163b7b09423a/docs/commands.md)，或运行 `npx skills --help` 查看示例和选项。实验性命令列于[下方](#实验性功能).

## 无需安装即可使用技能

为技能生成提示词，或使用该技能启动受支持的编码智能体：

```bash
npx skills use vercel-labs/agent-skills@web-design-guidelines --agent claude-code
```

省略 `--agent` 可改为打印提示词。请参阅 [`skills use`](https://github.com/vercel-labs/skills/blob/7a5e920bead4b1039a6d6df5d7e1163b7b09423a/docs/commands.md#skills-use).

## 实验性功能

<details>
<summary>以 <code>experimental_</code> 开头的命令为预览功能，其名称、选项和行为可能在任何版本中更改或删除，恕不另行通知。</summary>

| 命令 | 说明 |
| --------------------------------- | --------------------------------------------------------- |
| `npx skills experimental_sync`    | 将 npm 依赖附带的技能同步到项目 |
| `npx skills experimental_install` | 恢复 `skills-lock.json` 中记录的项目技能 |

一些 npm 包在代码之外还为智能体提供说明。同步会将它们链接到项目，使其随依赖更新，并安装 `package.json` 中声明的技能包：

```bash
npm install -D skills agent-browser
npx skills experimental_sync -y
```

选项、`skills` 字段和背后的 [RFC](https://github.com/vercel-labs/skills/issues/2323)，请参阅[实验命令参考](https://github.com/vercel-labs/skills/blob/7a5e920bead4b1039a6d6df5d7e1163b7b09423a/docs/commands.md#experimental-commands)。

</details>

## 支持的智能体

CLI 会检测已安装的智能体，并让你选择技能的安装位置。请参阅[所有支持的智能体、安装路径和兼容性](https://github.com/vercel-labs/skills/blob/7a5e920bead4b1039a6d6df5d7e1163b7b09423a/docs/agents.md).

## 创建技能

```bash
npx skills init my-skill
```

编辑生成的 `SKILL.md`，写入给智能体的说明。有关格式与发现规则，请参阅[创建与分享技能](https://github.com/vercel-labs/skills/blob/7a5e920bead4b1039a6d6df5d7e1163b7b09423a/docs/creating-skills.md)。

## 文档

- [CLI 参考](https://github.com/vercel-labs/skills/blob/7a5e920bead4b1039a6d6df5d7e1163b7b09423a/docs/commands.md): 安装、命令示例和实验性功能
- [支持的智能体](https://github.com/vercel-labs/skills/blob/7a5e920bead4b1039a6d6df5d7e1163b7b09423a/docs/agents.md): 智能体名称、技能目录和兼容性
- [创建技能](https://github.com/vercel-labs/skills/blob/7a5e920bead4b1039a6d6df5d7e1163b7b09423a/docs/creating-skills.md): frontmatter、仓库结构和插件
- [故障排除与配置](https://github.com/vercel-labs/skills/blob/7a5e920bead4b1039a6d6df5d7e1163b7b09423a/docs/troubleshooting.md): 常见问题、环境变量和遥测

## 遥测

CLI 会收集匿名使用数据。设置 `DISABLE_TELEMETRY=1` 或 `DO_NOT_TRACK=1` 可禁用遥测与安全审计请求。
请参阅[收集的内容](https://github.com/vercel-labs/skills/blob/7a5e920bead4b1039a6d6df5d7e1163b7b09423a/docs/troubleshooting.md#telemetry).

## 相关链接

- [技能目录](https://skills.sh)
- [Agent Skills 规范](https://agentskills.io)
- [Vercel Agent Skills](https://github.com/vercel-labs/agent-skills)

## 许可证

本项目采用 [MIT 许可证](https://github.com/vercel-labs/skills/blob/7a5e920bead4b1039a6d6df5d7e1163b7b09423a/LICENSE).
