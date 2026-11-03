# Superpowers

Superpowers 是一套供编程智能体使用的完整软件开发方法，建立在一组可组合技能之上，并通过初始指令确保智能体使用这些技能。

## 目录

- [工作原理](#how-it-works)
- [商业服务](#commercial-services)
- [安装](#installation)
  - [Claude Code](#claude-code)
  - [Antigravity](#antigravity)
  - [Codex App](#codex-app)
  - [Codex CLI](#codex-cli)
  - [Cursor](#cursor)
  - [Devin CLI](#devin-cli)
  - [Factory Droid](#factory-droid)
  - [Gemini CLI](#gemini-cli)
  - [GitHub Copilot CLI](#github-copilot-cli)
  - [Grok Build CLI](#grok-build-cli)
  - [Kimi Code](#kimi-code)
  - [OpenCode](#opencode)
  - [Pi](#pi)
  - [Qwen Code](#qwen-code)
  - [Hermes Agent](#hermes-agent)
  - [Muse](#muse)
- [基本工作流程](#the-basic-workflow)
- [出现问题时](#when-something-goes-wrong)
- [社区](#community)
- [包含什么](#whats-inside)
- [理念](#philosophy)
- [贡献](#contributing)
- [更新](#updating)
- [许可证](#license)
- [可视化辅助功能的遥测](#visual-companion-telemetry)

<a id="how-it-works"></a>
## 工作原理

从启动编程智能体时便开始生效。当它发现你要构建东西时，*不会*直接动手写代码，而是先停下来询问你真正想做什么。

从对话中梳理出规格后，它会分成便于阅读和理解的小段展示给你。

你确认设计后，智能体会制定足够清晰的实施计划，即使一位热情但品味和判断力欠佳、不了解项目背景且不喜欢测试的初级工程师也能照着执行。计划强调真正的红灯／绿灯测试驱动开发、YAGNI（你不会需要它）和 DRY（不要重复自己）。

接下来，一旦你说“开始”，它就启动 *subagent-driven-development*（子智能体驱动开发）流程，让智能体逐项完成工程任务、检查和审查结果，然后继续推进。智能体自主工作数小时而不偏离共同制定的计划，并不罕见。

还有许多其他细节，但这就是系统的核心。技能会自动触发，因此你无需特别操作：你的编程智能体就拥有了 Superpowers。

<a id="commercial-services"></a>
## 商业服务

如果你在企业中使用 Superpowers，并需要商业支持、额外工具或托管支出服务，欢迎发送邮件至 sales@primeradiant.com。

<a id="installation"></a>
## 安装

不同的运行环境安装方式不同。如果使用多个环境，请分别安装 Superpowers。

### Claude Code

可从 [Claude 官方插件市场](https://claude.com/plugins/superpowers)获取 Superpowers。

#### 官方插件市场

- 从 Anthropic 官方市场安装插件：

  ```bash
  /plugin install superpowers@claude-plugins-official
  ```

#### Superpowers 插件市场

Superpowers 插件市场为 Claude Code 提供 Superpowers 和其他相关插件。

- 注册插件市场：

  ```bash
  /plugin marketplace add obra/superpowers-marketplace
  ```

- 从该市场安装插件：

  ```bash
  /plugin install superpowers@superpowers-marketplace
  ```

### Antigravity

从此仓库将 Superpowers 安装为插件：

```bash
agy plugin install https://github.com/obra/superpowers
```

Antigravity 会列出每个已安装技能的说明；`using-superpowers` 技能的说明会提示模型在对话开始时加载该技能。使用相同命令重新安装即可更新。

### Codex App

可从 [Codex 官方插件市场](https://github.com/openai/plugins)获取 Superpowers。

- 在 Codex 应用中点击侧栏的 Plugins。
- 在 Coding 分类中找到 `Superpowers`。
- 点击 Superpowers 旁的 `+` 并按照提示操作。

### Codex CLI

可从 [Codex 官方插件市场](https://github.com/openai/plugins)获取 Superpowers。

- 打开插件搜索界面：

  ```bash
  /plugins
  ```

- 搜索 Superpowers：

  ```bash
  superpowers
  ```

- 选择 `Install Plugin`。

### Cursor

- 在 Cursor Agent 聊天中从插件市场安装：

  ```text
  /add-plugin superpowers
  ```

- 或在插件市场搜索“superpowers”。

### Devin CLI

- 从此仓库安装插件：

  ```bash
  devin plugins install obra/superpowers
  ```

- 使用以下命令更新到最新版：

  ```bash
  devin plugins update superpowers
  ```

### Factory Droid

- 注册插件市场：

  ```bash
  droid plugin marketplace add https://github.com/obra/superpowers
  ```

- 安装插件：

  ```bash
  droid plugin install superpowers@superpowers
  ```

### Gemini CLI

- 安装扩展：

  ```bash
  gemini extensions install https://github.com/obra/superpowers
  ```

- 后续更新：

  ```bash
  gemini extensions update superpowers
  ```

### GitHub Copilot CLI

- 注册插件市场：

  ```bash
  copilot plugin marketplace add obra/superpowers-marketplace
  ```

- 安装插件：

  ```bash
  copilot plugin install superpowers@superpowers-marketplace
  ```

### Grok Build CLI

可从 [Grok 官方插件市场](https://github.com/xai-org/plugin-marketplace)获取 Superpowers。

- 从 xAI 官方市场安装插件：

  ```bash
  grok plugin install superpowers@xai-official --trust
  ```

- 或在终端界面打开插件市场，搜索并安装 Superpowers：

  ```text
  /marketplace
  ```

### Kimi Code

可在 Kimi Code 插件市场获取 Superpowers。

- 打开 Kimi Code 插件管理器：

  ```text
  /plugins
  ```

- 进入 `Marketplace` > `Superpowers` 并安装。

- 或直接从此仓库安装：

  ```text
  /plugins install https://github.com/obra/superpowers
  ```

- 详细文档： [docs/README.kimi.md](https://github.com/obra/superpowers/blob/2ce8aa15d05bd57213e6f746f953aafb7ded7a8b/docs/README.kimi.md)

### OpenCode

OpenCode 使用自己的插件安装方式；即使你已在其他环境使用 Superpowers，也请为 OpenCode 单独安装。

- 告诉 OpenCode：

  ```
  Fetch and follow instructions from https://raw.githubusercontent.com/obra/superpowers/refs/heads/main/.opencode/INSTALL.md
  ```

- 详细文档： [docs/README.opencode.md](https://github.com/obra/superpowers/blob/2ce8aa15d05bd57213e6f746f953aafb7ded7a8b/docs/README.opencode.md)

### Pi

从此仓库将 Superpowers 安装为 Pi 包：

```bash
pi install git:github.com/obra/superpowers
```

如需本地开发，可将此检出目录作为临时包加载来运行 Pi：

```bash
pi -e /path/to/superpowers
```

Pi 包会加载 Superpowers 技能及一个小型扩展，在会话启动和上下文压缩后注入 `using-superpowers` 引导。Pi 原生支持技能，因此无需兼容性 `Skill` 工具。子智能体和任务列表工具仍是可选的 Pi 配套包。

### Qwen Code

Qwen Code 可直接从 Claude Code 插件市场安装插件。

- 从此仓库安装插件，并在提示时选择 `superpowers`：

  ```bash
  qwen extensions install obra/superpowers
  ```

- 后续更新：

  ```bash
  qwen extensions update superpowers
  ```

### Hermes Agent

从此仓库将 Superpowers 安装为 Hermes 插件：

```bash
hermes plugins install obra/superpowers --enable
```

安装后重启正在运行的 Hermes 会话。注意：Hermes 没有压缩后的钩子，因此长会话如果压缩掉首轮内容，就会丢失引导；技能停止触发时请启动新会话。

### Muse

Superpowers 可作为 Muse 原生插件使用：仓库和技能与其他环境相同。它通过原生 `SessionStart` 钩子注入 `using-superpowers` 引导，与 Claude Code、Codex、Cursor、Gemini、Pi 等环境一样，无需每次会话单独启用。

- 从本地检出目录安装：

  ```bash
  muse plugins install ./
  muse plugins approve superpowers
  ```

  或克隆后安装：

  ```bash
  git clone https://github.com/obra/superpowers.git
  muse plugins install ./superpowers
  muse plugins approve superpowers
  ```

- 后续更新：

  ```bash
  muse plugins update superpowers
  ```

安装后重启正在运行的 Muse 会话以使 `SessionStart` 钩子生效。技能立即生效，钩子首次安装时需要批准。验证时，启动新会话并发送 `Let's make a react todo list`；安装正常时，写代码之前会自动触发 `brainstorming`。版本记录在 `.version-bump.json`，由 `scripts/bump-version.sh` 保持同步。

<a id="the-basic-workflow"></a>
## 基本工作流程

1. **brainstorming（头脑风暴）**——在构建之前启动。通过提问引导你描述真正想做什么及原因，再复述其理解供你纠正。小改动在聊天中达成一致；项目则形成可供开发者制定计划的书面设计文档。

2. **using-git-worktrees**——设计获批准后启动。在新分支上创建隔离工作区、运行项目设置并验证干净的测试基线。

3. **writing-plans**——设计获批准后启动。将工作拆成小任务（每项 2–5 分钟）；每项都包含准确的文件路径、完整代码及验证步骤。

4. **subagent-driven-development** 或 **executing-plans**——计划完成后启动。前者为每个任务派出新的子智能体并在每项后审查（最周全）；后者在当前会话内完成所有任务，最后对整个分支进行一次新的审查（成本最低）。

5. **test-driven-development**——实施时启动。执行“红灯—绿灯—重构”：编写失败的测试、确认失败、编写最少代码、确认通过并提交。删除先于测试编写的代码。

6. **requesting-code-review**——任务之间启动。按计划审查并依严重程度报告问题；严重问题会阻止继续推进。

7. **finishing-a-development-branch**——任务完成后启动。验证测试、提供合并／PR／保留／丢弃等选项并清理工作树。

**智能体在任何任务前都会检查相关技能。**这些流程是强制要求，并非建议。

<a id="when-something-goes-wrong"></a>
## 出现问题时

会话有时会出现异常：技能不该触发时触发、该触发时没有反应，或智能体忽略计划、重复工作、消耗超出预期的 token。可以要求编程智能体“找出本次会话中 superpowers 出了什么问题”，它会调用 **diagnosing-superpowers** 技能。如需检查之前的会话，请指定会话，例如“找出会话 `<id>` 中 superpowers 出了什么问题”。

该技能会读取会话记录，以行级证据报告发生的情况，并可按需打包经过脱敏的资料用于错误报告。

<a id="community"></a>
## 社区

Superpowers 由 [Jesse Vincent](https://blog.fsck.com) 和 [Prime Radiant](https://primeradiant.com) 的其他成员开发。

- **Discord**：[加入我们](https://discord.gg/35wsABTejz)，获取社区支持、提问并分享你使用 Superpowers 构建的作品。
- **问题反馈**： https://github.com/obra/superpowers/issues
- **版本发布公告**：[注册](https://primeradiant.com/superpowers/)以接收新版本通知。

<a id="whats-inside"></a>
## 包含什么

### 技能库

**测试**
- **test-driven-development**——红灯—绿灯—重构循环（包含测试反模式参考）

**调试**
- **systematic-debugging**——四阶段根因分析流程（包含根因追踪、纵深防御、按条件等待等技术）
- **verification-before-completion**——确保问题确实得到修复
- **diagnosing-superpowers**——结合证据找出会话中出现的问题；导出脱敏资料包或提交问题

**协作** 
- **brainstorming**——构建前梳理意图
- **writing-plans**——详细的实施计划
- **executing-plans**——在当前上下文中执行计划，最后统一审查
- **dispatching-parallel-agents**——并行子智能体工作流
- **requesting-code-review**——审查前检查清单
- **receiving-code-review**——回应反馈
- **using-git-worktrees**——并行开发分支
- **finishing-a-development-branch**——合并／PR 决策工作流
- **subagent-driven-development**——通过两阶段审查快速迭代（先检查规格符合性，再检查代码质量）

**元技能**
- **writing-skills**——遵循最佳实践创建新技能（包含测试方法）
- **using-superpowers**——技能系统介绍

<a id="philosophy"></a>
## 理念

- **测试驱动开发**——始终先写测试
- **系统化而非临时应付**——用流程代替猜测
- **降低复杂度**——以简单为首要目标
- **证据重于宣称**——宣布成功前先验证

阅读[最初的发布公告](https://blog.fsck.com/2025/10/09/superpowers/)。

<a id="contributing"></a>
## 贡献

Superpowers 的一般贡献流程如下。请注意，项目通常不接受新技能贡献，任何技能更新都必须适用于项目支持的所有编程智能体。除非维护者提出要求，项目也不接受由智能体编写、用于修复他人所提问题的 PR；即使你遇到相同问题，也请在现有 issue 中补充报告。

1. Fork 仓库
2. 切换到 `dev` 分支
3. 为你的工作创建分支
4. 遵循 `writing-skills` 技能创建和测试新增或修改的技能
5. 提交 PR，填写拉取请求模板的每个部分，包括由人用自己的话撰写的摘要。

技能行为测试使用 [superpowers-evals](https://github.com/prime-radiant-inc/superpowers-evals/) 的 drill 评估框架，需克隆至 `evals/`；设置方法见 `evals/README.md`。插件基础设施测试位于 `tests/`，通过相应的 `run-*.sh` 或 `npm test` 运行。

完整指南见 `skills/writing-skills/SKILL.md`。

<a id="updating"></a>
## 更新

Superpowers 的更新方式在一定程度上取决于编程智能体，但通常是自动的。

<a id="license"></a>
## 许可证

MIT 许可证，详情见 LICENSE 文件。

<a id="visual-companion-telemetry"></a>
## 可视化辅助功能的遥测

由于技能和插件不会向创建者提供反馈，我们无法知道有多少人在使用 Superpowers。默认情况下，brainstorming 可选的可视化辅助功能中的 Prime Radiant 标志从我们的网站加载，请求中包含所用的 Superpowers 版本，但不包含你的项目、提示词或编程智能体的任何详情。我们看不到你的点击或正在构建的内容。这帮助我们粗略了解使用人数及版本。此功能完全可选；如需禁用，请将环境变量 `SUPERPOWERS_DISABLE_TELEMETRY` 设为任意真值。Superpowers 也遵循 Claude Code 的 `DISABLE_TELEMETRY` 和 `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC` 停用设置。
