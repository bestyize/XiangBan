> **上游维护提示：**核验固定版本的 102 个同仓库相对目标时，发现其中 78 个不存在，包括部分核心 SaaS 自动化技能目录，因此这些技能入口当前无法访问。以下保留原路径并固定到所选提交；请勿据此认为所有技能都可直接使用。上游通用 [connect 技能](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/connect)可访问，但并非这些单独目录的等价替代。

<h1 align="center">Awesome Claude Skills</h1>

<p align="center">
<a href="https://dashboard.composio.dev/login?utm_source=Github&utm_medium=Youtube&utm_campaign=2025-11&utm_content=AwesomeSkills">
  <img width="1280" height="640" alt="Composio banner" src="https://github.com/user-attachments/assets/e91255af-e4ba-4d71-b1a8-bd081e8a234a">
</a>


</p>

<p align="center">
  <a href="https://awesome.re">
    <img src="https://awesome.re/badge.svg" alt="Awesome" />
  </a>
  <a href="https://makeapullrequest.com">
    <img src="https://img.shields.io/badge/PRs-welcome-brightgreen.svg?style=flat-square" alt="PRs Welcome" />
  </a>
  <a href="https://www.apache.org/licenses/LICENSE-2.0">
    <img src="https://img.shields.io/badge/License-Apache_2.0-blue.svg?style=flat-square" alt="License: Apache-2.0" />
  </a>
</p>
<div>
<p align="center">
  <a href="https://twitter.com/composio">
    <img src="https://img.shields.io/badge/Follow%20on%20X-000000?style=for-the-badge&logo=x&logoColor=white" alt="Follow on X" />
  </a>
  <a href="https://www.linkedin.com/company/composiohq/">
    <img src="https://img.shields.io/badge/Follow%20on%20LinkedIn-0077B5?style=for-the-badge&logo=linkedin&logoColor=white" alt="Follow on LinkedIn" />
  </a>
  <a href="https://discord.com/invite/composio">
    <img src="https://img.shields.io/badge/Join%20our%20Discord-5865F2?style=for-the-badge&logo=discord&logoColor=white" alt="Join our Discord" />
  </a>
  </p>
</div>

这是一份全面精选的 1000 多个可用于生产且实用的 Claude 技能与插件清单，可提升 Claude.ai、Claude Code，以及 Codex、Cursor、Gemini CLI、Antigravity 等编码智能体在各类场景中的工作效率。

<a id="give-your-skills-real-world-actions"></a>
## 让技能执行真实世界中的操作

技能告诉智能体**如何**工作；MCP 网关为其提供所需工具的安全访问。

Composio [MCP Gateway](https://composio.dev/mcp-gateway) 为 1,000 多种集成提供单一 MCP 端点，内置身份验证、基于团队的访问控制、审计日志和适用于生产的可靠性。

---

<a id="quickstart-connect-claude-to-1000-apps"></a>
## 快速开始：将 Claude 连接到 1000 多个应用

**connect-apps** 插件让 Claude 执行真实操作，例如发送邮件、创建 issue、发布到 Slack。它处理认证，并在底层通过 Composio 连接 1000 多个应用。

<a id="1-install-the-plugin"></a>
### 1. 安装插件

```bash
claude --plugin-dir ./connect-apps-plugin
```

<a id="2-run-setup"></a>
### 2. 运行设置

```
/connect-apps:setup
```

出现提示时粘贴 API 密钥。（可在 [dashboard.composio.dev](https://dashboard.composio.dev/login?utm_source=Github&utm_content=AwesomeSkills) 免费获取密钥。）

<a id="3-restart--try-it"></a>
### 3. 重启并尝试

```bash
exit
claude
```

> **希望技能不止生成文本？** Claude 可以发送邮件、创建 issue、发布到 Slack，并在 1000 多个应用中执行操作。[了解方法 →](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/connect)

如果收到邮件，Claude 现在已连接到 1000 多个应用。

**[查看全部支持的应用 →](https://composio.dev/toolkits)**

---

<a id="contents"></a>
## 目录

- [Claude Skills 是什么？](#what-are-claude-skills)
- [技能](#skills)
  - [文档处理](#document-processing)
  - [开发与代码工具](#development--code-tools)
  - [数据与分析](#data--analysis)
  - [商业与营销](#business--marketing)
  - [沟通与写作](#communication--writing)
  - [创意与媒体](#creative--media)
  - [生产力与组织](#productivity--organization)
  - [协作与项目管理](#collaboration--project-management)
  - [安全与系统](#security--systems)
  - [通过 Composio 自动操作应用](#app-automation-via-composio)
- [入门](#getting-started)
- [创建技能](#creating-skills)
- [贡献](#contributing)
- [资源](#resources)
- [许可证](#license)

<a id="what-are-claude-skills"></a>
## Claude Skills 是什么？

Claude Skills 是可复用的指令包，用来教 AI 智能体处理特定类别的任务。每个技能都是一个文件夹，包含带 YAML 前置元数据（名称、描述）和 Markdown 指令的 `SKILL.md`，还可附带脚本、参考资料和素材。Anthropic 于 2025 年 10 月引入此格式，并于 2025 年 12 月将其发布为[开放标准](https://github.com/anthropics/skills)；现在 Claude Code、Claude.ai、Claude API、OpenAI Codex、Cursor、Gemini CLI、Antigravity 和 Windsurf 均支持它。

技能采用渐进加载。会话开始时，智能体只能看到每个技能的名称和描述，每项约 100 个 token；只有判断该技能与当前任务相关时，才加载完整 SKILL.md 正文（通常少于 5,000 个 token）。`scripts/` 和 `references/` 中的辅助文件按需加载，因此单个智能体可以承载数百个技能，而不会使上下文窗口膨胀。

技能不是 MCP 服务器，也不是工具。MCP 定义智能体如何连接外部系统，包括认证、传输和工具发现；工具是智能体调用的单个函数；技能在具备所需连接与工具之后定义工作流程，即做什么、按什么顺序做以及采用什么防护措施。在生产中，这三层共同运行：MCP 负责访问、工具负责操作、技能负责行为。

<a id="skills"></a>
## 技能

<a id="document-processing"></a>
### 文档处理

- [docx](https://github.com/anthropics/skills/tree/main/skills/docx) - 创建、编辑和分析 Word 文档，支持修订、批注和格式。
- [pdf](https://github.com/anthropics/skills/tree/main/skills/pdf) - 提取文本、表格及元数据，合并并批注 PDF。
- [pptx](https://github.com/anthropics/skills/tree/main/skills/pptx) - 读取、生成和调整幻灯片、布局及模板。
- [xlsx](https://github.com/anthropics/skills/tree/main/skills/xlsx) - 操作电子表格，包括公式、图表和数据转换。
- [Markdown to EPUB Converter](https://github.com/smerchek/claude-epub-skill) - 将 Markdown 文档及聊天摘要转换为专业 EPUB 电子书文件。 *作者：[@smerchek](https://github.com/smerchek)*
- [Master Claude for Legal](https://github.com/sboghossian/master-claude-for-legal) - 面向法律团队的技能包，支持 NDA 初筛、多方版本差异比较、引文核验、会议简报和周五通讯状态汇总模式。包含 10 份参考文档（特权、核验、长文档、执业领域）及 3 个律所模板，基于 Anthropic 公开的 Claude for Legal Teams 网络研讨会数据集。 *作者：[@sboghossian](https://github.com/sboghossian)*

<a id="development--code-tools"></a>
### 开发与代码工具

- [artifacts-builder](https://github.com/anthropics/skills/tree/main/skills/web-artifacts-builder) - 使用现代前端技术（React、Tailwind CSS、shadcn/ui）创建复杂多组件 claude.ai HTML 成品的工具套件。
- [aws-skills](https://github.com/zxkane/aws-skills) - 用于 AWS 开发，涵盖 CDK 最佳实践、成本优化 MCP 服务器及无服务器／事件驱动架构模式。
- [building-blog](https://github.com/BuildShipGrowRepeat/nextjs-sanity-blog-skill) - 通过 40 个问题的需求收集、一页计划和 20 节规格，为 Next.js + Sanity 网站添加注重 SEO、支持国际化的博客；包含通过 Gemini 3 Pro Image（Nano Banana Pro）生成 AI 主视觉图的工具。 *作者：[@BuildShipGrowRepeat](https://github.com/BuildShipGrowRepeat)*
- [Changelog Generator](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/changelog-generator) - 分析 Git 历史，将技术性提交转换成用户易读的发行说明，自动生成面向用户的更新日志。
- [Chrome Relay](https://chrome-relay.kushalsm.com/) - 通过本地 CLI 桥接控制用户已打开的 Chrome 会话，包括 cookie、单点登录、扩展和 localhost。它是 Playwright Browser Automation 使用真实 Chrome 的对应工具；通过 `npx skills add chrome-relay` 与 [Chrome 网上应用店扩展](https://chromewebstore.google.com/detail/chrome-relay/cpdiapbifblhlcpnmlmfpgfjlacebokb)安装，无需远程中继、Playwright fixture 或 MCP 服务器。
- [Claude Code Terminal Title](https://github.com/bluzername/claude-code-terminal-title) - 为每个 Claude Code 终端窗口设置描述当前工作的动态标题，便于区分窗口。
- [Connect](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/connect) - 将 Claude 连接到应用，在 Gmail、Slack、GitHub、Notion 等 1000 多项服务中发送邮件、创建 issue、发布消息和更新数据库。
- [D3.js Visualization](https://github.com/chrisvoncsefalvay/claude-d3js-skill) - 教 Claude 制作 D3 图表与交互式数据可视化。 *作者：[@chrisvoncsefalvay](https://github.com/chrisvoncsefalvay)*
- [FFUF Web Fuzzing](https://github.com/jthack/ffuf_claude_skill) - 集成 ffuf Web 模糊测试工具，使 Claude 能执行测试并分析漏洞结果。 *作者：[@jthack](https://github.com/jthack)*
- [finishing-a-development-branch](https://github.com/obra/superpowers/tree/main/skills/finishing-a-development-branch) - 通过提供清晰选项并处理所选流程，指导开发工作的收尾。
- [Full-Page Screenshot](https://github.com/LewisLiu007/full-page-screenshot) - 通过 Chrome DevTools Protocol 截取网页全页截图，零依赖。 *作者：[@LewisLiu007](https://github.com/LewisLiu007)*
- [great_cto](https://github.com/avelikiy/great_cto) - Claude Code 插件：由 7 个专门的子智能体（tech-lead、senior-dev、qa-engineer、security-officer、devops、l3-support、project-auditor）编排完整软件开发生命周期，涵盖架构、TDD、12 个角度的代码审查、QA、安全审计和部署；自动识别 11 种项目类型，支持 13 个合规框架（GDPR/PCI-DSS/HIPAA/SOC2/ISO 27001），并通过自我改进的知识层从每次事件中学习。 *作者：[@avelikiy](https://github.com/avelikiy)*
- [iOS Simulator](https://github.com/conorluddy/ios-simulator-skill) - 让 Claude 与 iOS 模拟器交互，以测试和调试 iOS 应用。 *作者：[@conorluddy](https://github.com/conorluddy)*
- [jules](https://github.com/sanjay3290/ai-skills/tree/main/skills/jules) - 将编码任务委派给 Google Jules AI 智能体，异步完成 GitHub 仓库中的缺陷修复、文档、测试和功能实现。 *作者：[@sanjay3290](https://github.com/sanjay3290)*
- [LangSmith Fetch](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/langsmith-fetch) - 通过自动获取和分析 LangSmith Studio 执行轨迹调试 LangChain 与 LangGraph 智能体；面向 Claude Code 的 AI 可观测性技能。 *作者：[@OthmanAdi](https://github.com/OthmanAdi)*
- [lean-ctx](https://github.com/yvgude/lean-ctx) - 面向 AI 编码智能体的 MCP 服务器和上下文运行时，通过会话缓存、AST 感知压缩及 90 多种 shell 模式减少 token 消耗。支持 Claude Code、Cursor、Copilot 等；使用 `lean-ctx init --agent claude-code` 安装 Claude Code 技能，文档见 [leanctx.com](https://leanctx.com)。 *作者：[@yvgude](https://github.com/yvgude)*
- [MCP Builder](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/mcp-builder) - 指导使用 Python 或 TypeScript 创建高质量 MCP（模型上下文协议）服务器，以将外部 API 和服务与大语言模型集成。
- [move-code-quality-skill](https://github.com/1NickPappas/move-code-quality-skill) - 依据官方 Move Book 代码质量清单分析 Move 语言包，检查其是否符合 Move 2024 Edition 和最佳实践。
- [OpenWeb](https://github.com/openweb-org/openweb) - 供智能体访问网站：调用网站自身使用的 API（JSON 输入与输出），按请求自动处理 cookie、JWT、CSRF 和签名等认证；内置 90 多个网站。 *作者：[@openweb-org](https://github.com/openweb-org)*
- [overkill](https://github.com/santiago-vargas-de-kruijf/claude-overkill) - 针对讨论中的方案提出先进且复杂的替代选择，包括高级数据结构、分布式系统算法、小众框架、设计模式和前沿工具；每项按校准的复杂度分级，并附学习链接及适用场景。 *作者：[@santiago-vargas-de-kruijf](https://github.com/santiago-vargas-de-kruijf)*
- [Playwright Browser Automation](https://github.com/lackeyjb/playwright-skill) - 由模型调用 Playwright 自动化，测试和验证 Web 应用。 *作者：[@lackeyjb](https://github.com/lackeyjb)*
- [prompt-engineering](https://github.com/NeoLabHQ/context-engineering-kit/tree/master/plugins/customaize-agent/skills/prompt-engineering) - 讲解常见提示工程技术和模式，包括 Anthropic 最佳实践和智能体说服原则。
- [pypict-claude-skill](https://github.com/omkamal/pypict-claude-skill) - 使用 PICT（成对独立组合测试）为需求或代码设计全面测试用例，生成具备成对覆盖的优化测试套件。
- [reddit-fetch](https://github.com/ykdojo/claude-code-tips/tree/main/skills/reddit-fetch) - 在 WebFetch 被阻止或返回 403 时，通过 Gemini CLI 获取 Reddit 内容。
- [Septim Agents Pack](https://septimlabs.com/tools/agents?utm_source=awesome-claude-skills&utm_medium=awesome-list&utm_campaign=oss-backlink) - 包含 10 个具名 Claude Code 子智能体（Atlas、Luca、Canon、Ember、Tally、Nova、Ward、Mira、Juno、Pip），覆盖规划、架构、品牌、营销、财务、设计、法律、客户、研究和协调；放入 `.claude/agents/` 即可。 *作者：[@septimlabs-code](https://github.com/septimlabs-code)*
- [Skill Creator](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/skill-creator) - 提供创建高效 Claude 技能的指导，以专业知识、工作流和工具集成扩展能力。
- [Skill Seekers](https://github.com/yusufkaraaslan/Skill_Seekers) - 数分钟内自动将任意文档网站转换为 Claude AI 技能。 *作者：[@yusufkaraaslan](https://github.com/yusufkaraaslan)*
- [software-architecture](https://github.com/NeoLabHQ/context-engineering-kit/tree/master/plugins/ddd/skills/software-architecture) - 实现 Clean Architecture、SOLID 等设计模式与全面的软件设计最佳实践。
- [subagent-driven-development](https://github.com/NeoLabHQ/context-engineering-kit/tree/master/plugins/sadd/skills/subagent-driven-development) - 为各项任务派遣独立子智能体，并在迭代之间设置代码审查检查点，以实现快速而受控的开发。
- [test-driven-development](https://github.com/obra/superpowers/tree/main/skills/test-driven-development) - 实现任何功能或修复缺陷时，在编写实现代码前使用。
- [using-git-worktrees](https://github.com/obra/superpowers/blob/main/skills/using-git-worktrees/) - 通过智能目录选择和安全验证创建隔离的 Git 工作树。
- [Webapp Testing](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/webapp-testing) - 使用 Playwright 测试本地 Web 应用，验证前端功能、调试界面行为并截图。

<a id="data--analysis"></a>
### 数据与分析

- [CSV Data Summarizer](https://github.com/coffeefuelbump/csv-data-summarizer-claude-skill) - 自动分析 CSV 文件并生成包含可视化的全面洞察，无需用户提示。 *作者：[@coffeefuelbump](https://github.com/coffeefuelbump)*
- [deep-research](https://github.com/sanjay3290/ai-skills/tree/main/skills/deep-research) - 使用 Gemini Deep Research Agent 执行自主多步研究，涵盖市场分析、竞争格局和文献综述。 *作者：[@sanjay3290](https://github.com/sanjay3290)*
- [postgres](https://github.com/sanjay3290/ai-skills/tree/main/skills/postgres) - 对 PostgreSQL 数据库执行安全的只读 SQL 查询，支持多连接和纵深防御。 *作者：[@sanjay3290](https://github.com/sanjay3290)*
- [recursive-research](https://github.com/Anjos2/recursive-research) - 在科学、技术、商业、艺术、人文等任何领域开展最高博士级的递归研究，支持来源分级、用于自主决策的 WDM 与芒格逆向思维，以及通过磁盘检查点应对上下文压缩。 *作者：[@Anjos2](https://github.com/Anjos2)*
- [root-cause-tracing](https://github.com/obra/superpowers/tree/main/skills/root-cause-tracing) - 当错误发生在执行深处时，向上追踪以找到最初触发原因。

<a id="business--marketing"></a>
### 商业与营销

- [Brand Build Skills](https://github.com/rampstackco/claude-skills) - 包含 59 项技能的库，覆盖网站全生命周期：品牌、设计、内容、SEO、开发、运维、增长和研究。与技术栈无关，带有由 Ahrefs MCP 驱动的 SEO 审计套件及用于编写自己技能的元技能。 *作者：[@rampstackco](https://github.com/rampstackco)*
- [Brand Guidelines](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/brand-guidelines) - 将 Anthropic 官方品牌颜色和字体应用到成品，保持视觉识别与专业设计标准一致。
- [Competitive Ads Extractor](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/competitive-ads-extractor) - 从广告库提取并分析竞争对手广告，了解有效的信息传达与创意方式。
- [Domain Name Brainstormer](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/domain-name-brainstormer) - 生成创意域名并检查 .com、.io、.dev、.ai 等多个顶级域名的可用性。
- [Internal Comms](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/internal-comms) - 按公司特定格式协助撰写内部沟通内容，包括 3P 更新、公司通讯、常见问题、状态报告及项目更新。
- [Lead Research Assistant](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/lead-research-assistant) - 分析产品、搜索目标公司并提供可执行的外联策略，以识别和筛选高质量潜在客户。

<a id="communication--writing"></a>
### 沟通与写作

- [article-extractor](https://github.com/michalparkola/tapestry-skills-for-claude-code/tree/main/article-extractor) - 从网页提取完整文章正文及元数据。
- [brainstorming](https://github.com/obra/superpowers/tree/main/skills/brainstorming) - 通过结构化提问和探索备选方案，将初步想法转化为完整设计。
- [Content Research Writer](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/content-research-writer) - 通过研究、添加引用、改进开头吸引力和逐节反馈，协助撰写高质量内容。
- [family-history-research](https://github.com/emaynard/claude-family-history-research-skill) - 协助规划家族历史和族谱研究项目。
- [Meeting Insights Analyzer](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/meeting-insights-analyzer) - 分析会议记录，发现回避冲突、发言比例、口头禅及领导风格等行为模式。
- [NotebookLM Integration](https://github.com/PleasePrompto/notebooklm-skill) - 让 Claude Code 直接与 NotebookLM 对话，获得完全基于已上传文档和来源的答案。 *作者：[@PleasePrompto](https://github.com/PleasePrompto)*
- [Twitter Algorithm Optimizer](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/twitter-algorithm-optimizer) - 利用 Twitter 开源算法相关信息分析和优化推文，以扩大覆盖范围；重写和编辑推文，提升互动和可见度。

<a id="creative--media"></a>
### 创意与媒体

- [anydesign](https://github.com/uxKero/anydesign) - 分析图片、URL 或 Figma 文件，生成结构化 `design.md`，包含完整设计系统、组件清单和重建说明，可用于 v0、Lovable、Cursor 等 AI 构建工具。 *作者：[@uxKero](https://github.com/uxKero)*
- [Canvas Design](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/canvas-design) - 运用设计理念和美学原则，创建用于海报、设计和静态作品的 PNG、PDF 视觉艺术。
- [imagen](https://github.com/sanjay3290/ai-skills/tree/main/skills/imagen) - 使用 Google Gemini 图像生成 API 生成界面模型、图标、插图及视觉素材。 *作者：[@sanjay3290](https://github.com/sanjay3290)*
- [Image Enhancer](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/image-enhancer) - 通过提升分辨率、锐度和清晰度改善图片与截图质量，用于专业演示和文档。
- [Slack GIF Creator](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/slack-gif-creator) - 创建针对 Slack 优化的动态 GIF，包含尺寸约束验证器和可组合动画基础元素。
- [Theme Factory](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/theme-factory) - 为幻灯片、文档、报告和 HTML 落地页等成品应用专业字体及配色主题，内置 10 套主题。
- [Video Downloader](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/video-downloader) - 从 YouTube 和其他平台下载视频供离线观看、编辑或归档，支持多种格式和质量选项。
- [youtube-transcript](https://github.com/michalparkola/tapestry-skills-for-claude-code/tree/main/youtube-transcript) - 获取 YouTube 视频字幕并准备摘要。
- [swiftui-design-skill](https://github.com/wholiver/swiftui-design-skill) - SwiftUI 前端设计 skill — 反 AI Slop 六条铁律、设计方向顾问、品牌资产协议、五维评审。支持 Claude Code / Cursor / Codex / OpenCode 等全部 AI agent 平台。 *作者：[@wholiver](https://github.com/wholiver)*
- [Pixelbin-Media-Generation](https://github.com/anandpareek-hub/pixelbin-claude-skill) - 通过 85 多项 API 生成及编辑图片、视频，并构建美观的网站页面。

<a id="productivity--organization"></a>
### 生产力与组织

- [File Organizer](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/file-organizer) - 理解上下文、查找重复文件并建议更好的组织结构，智能整理文件与文件夹。
- [Invoice Organizer](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/invoice-organizer) - 读取文件、提取信息并统一重命名，自动整理发票及收据以便报税。
- [kaizen](https://github.com/NeoLabHQ/context-engineering-kit/tree/master/plugins/kaizen/skills/kaizen) - 基于日本改善哲学与精益方法，使用多种分析方式实施持续改进。
- [n8n-skills](https://github.com/haunchen/n8n-skills) - 使 AI 助手能够直接理解和操作 n8n 工作流。
- [Raffle Winner Picker](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/raffle-winner-picker) - 以密码学安全的随机性，从名单、电子表格或 Google Sheets 中随机抽取赠品及竞赛获奖者。
- [solo-skills](https://github.com/rockscy/solo-skills) - 面向独立创始人与独立开发者的 7 个英中双语技能，涵盖发布推文、客户邮件、决策框架和复盘；每项均明确列出“不适用时机”。
- [Tailored Resume Generator](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/tailored-resume-generator) - 分析职位描述并生成定制简历，突出相关经历、技能和成果，以提高获得面试机会的可能性。
- [ship-learn-next](https://github.com/michalparkola/tapestry-skills-for-claude-code/tree/main/ship-learn-next) - 基于反馈循环，帮助迭代决定接下来要构建或学习什么。
- [tapestry](https://github.com/michalparkola/tapestry-skills-for-claude-code/tree/main/tapestry) - 将相关文档相互链接并汇总成知识网络。

<a id="collaboration--project-management"></a>
### 协作与项目管理

- [git-pushing](https://github.com/mhattingpete/claude-skills-marketplace/tree/main/engineering-workflow-plugin/skills/git-pushing) - 自动执行 Git 操作及仓库交互。
- [google-workspace-skills](https://github.com/sanjay3290/ai-skills/tree/main/skills) - Google Workspace 集成套件，覆盖 Gmail、Calendar、Chat、Docs、Sheets、Slides 和 Drive，支持跨平台 OAuth。 *作者：[@sanjay3290](https://github.com/sanjay3290)*
- [mercury-mcp](https://www.teamoffsite.ai/proton/docs/skill) - Mercury（Proton）MCP 工具速查表；可在协作智能体团队中向队友发送消息、管理线程、创建任务和安排自动化。 *作者：[Mercury](https://mercury.build)*
- [outline](https://github.com/sanjay3290/ai-skills/tree/main/skills/outline) - 搜索、阅读、创建和管理 Outline Wiki 实例中的文档，支持云端或自托管。 *作者：[@sanjay3290](https://github.com/sanjay3290)*
- [review-implementing](https://github.com/mhattingpete/claude-skills-marketplace/tree/main/engineering-workflow-plugin/skills/review-implementing) - 评估代码实施计划并使其与规格保持一致。
- [test-fixing](https://github.com/mhattingpete/claude-skills-marketplace/tree/main/engineering-workflow-plugin/skills/test-fixing) - 检测失败的测试并提出补丁或修复方案。

<a id="security--systems"></a>
### 安全与系统

- [computer-forensics](https://github.com/mhattingpete/claude-skills-marketplace/tree/main/computer-forensics-skills/skills/computer-forensics) - 数字取证分析与调查技术。
- [file-deletion](https://github.com/mhattingpete/claude-skills-marketplace/tree/main/computer-forensics-skills/skills/file-deletion) - 安全删除文件和清理数据的方法。
- [metadata-extraction](https://github.com/mhattingpete/claude-skills-marketplace/tree/main/computer-forensics-skills/skills/metadata-extraction) - 为取证目的提取和分析文件元数据。
- [threat-hunting-with-sigma-rules](https://github.com/jthack/threat-hunting-with-sigma-rules-skill) - 使用 Sigma 检测规则进行威胁狩猎与安全事件分析。

<a id="assistive-technology"></a>
### 辅助技术

- [ASD-AuDHD-PAI-Skills](https://github.com/emory/ASD-AuDHD-PAI-Skills) - 新技能集：首个技能 [pda-reframing](https://github.com/emory/ASD-AuDHD-PAI-Skills/blob/main/Skills/pda-reframing/SKILL.md) 可重新表述请求或决定，帮助面临持续性需求回避的自闭症谱系人士或难以开始任务的 ADHD 人士与任务建立联系。

<a id="app-automation-via-composio"></a>
### 通过 Composio 自动操作应用

通过 [Rube MCP（Composio）](https://composio.dev)为 78 个 SaaS 应用预构建的工作流技能。每项技能均包含工具调用顺序、参数指导、已知问题和速查表，使用从 Composio API 发现的真实工具标识。

**CRM 与销售**
- [Close Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/close-automation) - 自动操作 Close CRM：潜在客户、联系人、商机、活动及销售管道。
- [HubSpot Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/hubspot-automation) - 自动操作 HubSpot CRM：联系人、交易、公司、工单及邮件互动。
- [Pipedrive Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/pipedrive-automation) - 自动操作 Pipedrive：交易、联系人、组织、活动及销售管道。
- [Salesforce Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/salesforce-automation) - 自动操作 Salesforce：对象、记录、SOQL 查询及批量操作。
- [Zoho CRM Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/zoho-crm-automation) - 自动操作 Zoho CRM：潜在客户、联系人、交易、账户及模块。

**项目管理**
- [Asana Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/asana-automation) - 自动操作 Asana：任务、项目、分区、分配及工作区。
- [Basecamp Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/basecamp-automation) - 自动操作 Basecamp：待办清单、消息、人员、群组及项目。
- [ClickUp Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/clickup-automation) - 自动操作 ClickUp：任务、列表、空间、目标及时间跟踪。
- [Jira Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/jira-automation) - 自动操作 Jira：问题、项目、看板、冲刺及 JQL 查询。
- [Linear Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/linear-automation) - 自动操作 Linear：问题、项目、周期、团队及工作流。
- [Monday Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/monday-automation) - 自动操作 Monday.com：看板、条目、列、组及工作区。
- [Notion Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/notion-automation) - 自动操作 Notion：页面、数据库、块、评论及搜索。
- [Todoist Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/todoist-automation) - 自动操作 Todoist：任务、项目、分区、标签及筛选器。
- [Trello Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/trello-automation) - 自动操作 Trello：看板、卡片、列表、成员及检查清单。
- [Wrike Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/wrike-automation) - 自动操作 Wrike：任务、文件夹、项目、评论及工作流。

**沟通**
- [Discord Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/discord-automation) - 自动操作 Discord：消息、频道、服务器、角色及回应。
- [Intercom Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/intercom-automation) - 自动操作 Intercom：会话、联系人、公司、工单及文章。
- [Microsoft Teams Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/microsoft-teams-automation) - 自动操作 Teams：消息、频道、团队、聊天及会议。
- [Slack Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/slack-automation) - 自动操作 Slack：消息、频道、搜索、回应、线程及定时发送。
- [Telegram Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/telegram-automation) - 自动操作 Telegram：消息、聊天、媒体、群组及机器人。
- [WhatsApp Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/whatsapp-automation) - 自动操作 WhatsApp：消息、媒体、模板、群组及企业资料。

**邮件**
- [Gmail Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/gmail-automation) - 自动操作 Gmail：发送／回复、搜索、标签、草稿及附件。
- [Outlook Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/outlook-automation) - 自动操作 Outlook：邮件、文件夹、联系人及日历集成。
- [Postmark Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/postmark-automation) - 自动操作 Postmark：事务性邮件、模板、服务器及投递统计。
- [SendGrid Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/sendgrid-automation) - 自动操作 SendGrid：邮件、模板、联系人、列表及营销活动统计。

**代码与 DevOps**
- [Bitbucket Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/bitbucket-automation) - 自动操作 Bitbucket：仓库、PR、分支、问题及工作区。
- [CircleCI Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/circleci-automation) - 自动操作 CircleCI：流水线、工作流、作业及项目配置。
- [Datadog Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/datadog-automation) - 自动操作 Datadog：监控器、仪表盘、指标、事件及警报。
- [GitHub Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/github-automation) - 自动操作 GitHub：问题、PR、仓库、分支、Actions 及代码搜索。
- [GitLab Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/gitlab-automation) - 自动操作 GitLab：问题、MR、项目、流水线及分支。
- [PagerDuty Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/pagerduty-automation) - 自动操作 PagerDuty：事件、服务、排班、升级策略及值班。
- [Render Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/render-automation) - 自动操作 Render：服务、部署及项目管理。
- [Sentry Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/sentry-automation) - 自动操作 Sentry：问题、事件、项目、版本及警报。
- [Supabase Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/supabase-automation) - 自动操作 Supabase：SQL 查询、表结构、边缘函数及存储。
- [Vercel Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/vercel-automation) - 自动操作 Vercel：部署、项目、域名、环境变量及日志。

**存储与文件**
- [Box Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/box-automation) - 自动操作 Box：文件、文件夹、搜索、分享、协作及签名请求。
- [Dropbox Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/dropbox-automation) - 自动操作 Dropbox：文件、文件夹、搜索、分享及批量操作。
- [Google Drive Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/google-drive-automation) - 自动操作 Google Drive：上传、下载、搜索、分享和整理文件。
- [OneDrive Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/one-drive-automation) - 自动操作 OneDrive：文件、文件夹、搜索、分享、权限及版本管理。

**电子表格与数据库**
- [Airtable Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/airtable-automation) - 自动操作 Airtable：记录、表、数据基地、视图及字段管理。
- [Coda Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/coda-automation) - 自动操作 Coda：文档、表、行、公式及自动化。
- [Google Sheets Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/googlesheets-automation) - 自动操作 Google Sheets：读写单元格、格式、公式及批量操作。

**日历与排程**
- [Cal.com Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/cal-com-automation) - 自动操作 Cal.com：事件类型、预约、可用时间及排程。
- [Calendly Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/calendly-automation) - 自动操作 Calendly：事件、受邀者、事件类型、排程链接及可用时间。
- [Google Calendar Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/google-calendar-automation) - 自动操作 Google Calendar：事件、参与者、忙闲状态及重复日程。
- [Outlook Calendar Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/outlook-calendar-automation) - 自动操作 Outlook Calendar：事件、参与者、提醒及重复日程。

**社交媒体**
- [Instagram Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/instagram-automation) - 自动操作 Instagram：帖子、动态、评论、媒体及商业洞察。
- [LinkedIn Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/linkedin-automation) - 自动操作 LinkedIn：帖子、个人资料、公司、图片及评论。
- [Reddit Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/reddit-automation) - 自动操作 Reddit：帖子、评论、子版块、投票及版务管理。
- [TikTok Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/tiktok-automation) - 自动操作 TikTok：视频上传、查询及创作者管理。
- [Twitter Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/twitter-automation) - 自动操作 Twitter/X：推文、搜索、用户、列表及互动。
- [YouTube Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/youtube-automation) - 自动操作 YouTube：视频、频道、播放列表、评论及订阅。

**营销与邮件营销**
- [ActiveCampaign Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/activecampaign-automation) - 自动操作 ActiveCampaign：联系人、交易、营销活动、列表及自动化。
- [Brevo Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/brevo-automation) - 自动操作 Brevo：联系人、邮件营销活动、事务性邮件及列表。
- [ConvertKit Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/convertkit-automation) - 自动操作 ConvertKit（Kit）：订阅者、标签、序列、广播及表单。
- [Klaviyo Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/klaviyo-automation) - 自动操作 Klaviyo：资料、列表、分组、营销活动及事件。
- [Mailchimp Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/mailchimp-automation) - 自动操作 Mailchimp：受众、营销活动、模板、分组及报告。

**支持与服务台**
- [Freshdesk Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/freshdesk-automation) - 自动操作 Freshdesk：工单、联系人、客服人员、群组及预设回复。
- [Freshservice Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/freshservice-automation) - 自动操作 Freshservice：工单、资产、变更、问题及服务目录。
- [Help Scout Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/helpdesk-automation) - 自动操作 Help Scout：会话、客户、邮箱及标签。
- [Zendesk Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/zendesk-automation) - 自动操作 Zendesk：工单、用户、组织、搜索及宏。

**电子商务与支付**
- [Shopify Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/shopify-automation) - 自动操作 Shopify：产品、订单、客户、库存及 GraphQL 查询。
- [Square Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/square-automation) - 自动操作 Square：支付、客户、商品目录、订单及地点。
- [Stripe Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/stripe-automation) - 自动操作 Stripe：收费、客户、产品、订阅及退款。

**设计与协作**
- [Canva Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/canva-automation) - 自动操作 Canva：设计、模板、素材、文件夹及品牌套件。
- [Confluence Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/confluence-automation) - 自动操作 Confluence：页面、空间、搜索、CQL、标签及版本。
- [DocuSign Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/docusign-automation) - 自动操作 DocuSign：信封、模板、签署及文档管理。
- [Figma Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/figma-automation) - 自动操作 Figma：文件、组件、评论、项目及团队管理。
- [Miro Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/miro-automation) - 自动操作 Miro：画板、便利贴、形状、连接线及条目。
- [Webflow Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/webflow-automation) - 自动操作 Webflow：CMS 集合、条目、网站、发布及素材。

**分析与数据**
- [Amplitude Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/amplitude-automation) - 自动操作 Amplitude：事件、人群、用户属性及分析查询。
- [Google Analytics Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/google-analytics-automation) - 自动操作 Google Analytics：报告、维度、指标及属性管理。
- [Mixpanel Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/mixpanel-automation) - 自动操作 Mixpanel：事件、漏斗、人群、注释及 JQL 查询。
- [PostHog Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/posthog-automation) - 自动操作 PostHog：事件、用户、功能开关、洞察及注释。
- [Segment Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/segment-automation) - 自动操作 Segment：来源、目标、跟踪及数据仓库连接。

**人力资源与人员**
- [BambooHR Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/bamboohr-automation) - 自动操作 BambooHR：员工、休假、报告及目录管理。

**自动化平台**
- [Make Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/make-automation) - 自动操作 Make（Integromat）：场景、连接及执行管理。

**Zoom 与会议**
- [Zoom Automation](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2/zoom-automation) - 自动操作 Zoom：会议、录制、参与者、网络研讨会及报告。

<a id="getting-started"></a>
## 入门

<a id="using-skills-in-claudeai"></a>
### 在 Claude.ai 中使用技能

1. 点击聊天界面中的技能图标（🧩）。
2. 从市场添加技能或上传自定义技能。
3. Claude 根据任务自动激活相关技能。

<a id="using-skills-in-claude-code"></a>
### 在 Claude Code 中使用技能

1. 将技能放入 `~/.config/claude-code/skills/`：
   ```bash
   mkdir -p ~/.config/claude-code/skills/
   cp -r skill-name ~/.config/claude-code/skills/
   ```

2. 验证技能元数据：
   ```bash
   head ~/.config/claude-code/skills/skill-name/SKILL.md
   ```

3. 启动 Claude Code：
   ```bash
   claude
   ```

4. 技能会自动加载，并在相关时激活。

<a id="using-skills-via-api"></a>
### 通过 API 使用技能

使用 Claude Skills API 以编程方式加载和管理技能：

```python
import anthropic

client = anthropic.Anthropic(api_key="your-api-key")

response = client.messages.create(
    model="claude-3-5-sonnet-20241022",
    skills=["skill-id-here"],
    messages=[{"role": "user", "content": "Your prompt"}]
)
```

详情参见 [Skills API 文档](https://docs.claude.com/en/api/skills-guide)。

<a id="creating-skills"></a>
## 创建技能

<a id="skill-structure"></a>
### 技能结构

每个技能都是包含带 YAML 前置元数据的 `SKILL.md` 文件的文件夹：

```
skill-name/
├── SKILL.md          # Required: Skill instructions and metadata
├── scripts/          # Optional: Helper scripts
├── templates/        # Optional: Document templates
└── resources/        # Optional: Reference files
```

<a id="basic-skill-template"></a>
### 基本技能模板

```markdown
---
name: my-skill-name
description: A clear description of what this skill does and when to use it.
---

# My Skill Name

Detailed description of the skill's purpose and capabilities.

## When to Use This Skill

- Use case 1
- Use case 2
- Use case 3

## Instructions

[Detailed instructions for Claude on how to execute this skill]

## Examples

[Real-world examples showing the skill in action]
```

<a id="skill-best-practices"></a>
### 技能最佳实践

- 专注于具体、可重复的任务
- 包含清晰的示例和边界情况
- 为 Claude 编写指令，而不是为最终用户编写
- 在 Claude.ai、Claude Code 和 API 中测试
- 记录前提条件与依赖
- 包含错误处理指导

<a id="contributing"></a>
## 贡献

欢迎贡献！以下事项请参阅我们的[贡献指南](https://github.com/ComposioHQ/awesome-claude-skills/blob/be2a406907dbc61b73e6827ded415c96139d13a2/CONTRIBUTING.md)：

- 如何提交新技能
- 技能质量标准
- 拉取请求流程
- 行为准则

<a id="quick-contribution-steps"></a>
### 快速贡献步骤

1. 确保技能基于真实使用场景
2. 检查现有技能中是否已有重复内容
3. 遵循技能结构模板
4. 在各平台测试技能
5. 提交包含清晰文档的拉取请求

<a id="resources"></a>
## 资源

<a id="official-documentation"></a>
### 官方文档

- [Claude Skills Overview](https://www.anthropic.com/news/skills) - 官方公告与功能介绍
- [Skills User Guide](https://support.claude.com/en/articles/12512180-using-skills-in-claude) - 如何在 Claude 中使用技能
- [Creating Custom Skills](https://support.claude.com/en/articles/12512198-creating-custom-skills) - 技能开发指南
- [Skills API Documentation](https://docs.claude.com/en/api/skills-guide) - API 集成指南
- [Agent Skills Blog Post](https://anthropic.com/engineering/equipping-agents-for-the-real-world-with-agent-skills) - 工程深入解析

<a id="community-resources"></a>
### 社区资源

- [Anthropic Skills Repository](https://github.com/anthropics/skills) - 官方示例技能
- [Claude Community](https://community.anthropic.com) - 与其他用户讨论技能
- [Skills Marketplace](https://claude.ai/marketplace) - 发现及分享技能

<a id="inspiration--use-cases"></a>
### 灵感与使用场景

- [Lenny's Newsletter](https://www.lennysnewsletter.com/p/everyone-should-be-using-claude-code) - 人们使用 Claude Code 的 50 种方式
- [Notion Skills](https://www.notion.so/notiondevs/Notion-Skills-for-Claude-28da4445d27180c7af1df7d8615723d0) - Notion 集成技能
- [精选 Claude 技能](https://composio.dev/content/top-claude-skills)


<a id="join-the-community"></a>
## 加入社区

- [Join our Discord](https://discord.com/invite/composio) - 与其他 Claude Skills 开发者交流
- [Follow on Twitter/X](https://x.com/composio) - 及时了解新技能与功能
- 有疑问？[support@composio.dev](mailto:support@composio.dev)

---

<p align="center">
  <b>加入 20,000 多名构建并交付智能体的开发者</b>
</p>

<p align="center">
  <a href="https://platform.composio.dev/?utm_source=Github&utm_content=AwesomeSkills">
    <img src="https://img.shields.io/badge/Get_Started_Free-4F46E5?style=for-the-badge" alt="Get Started"/>
  </a>
</p>

<a id="license"></a>
## 许可证

本仓库采用 Apache License 2.0 许可。

单独的技能可能采用不同许可证，请检查各技能文件夹中的具体许可信息。

---

**注意**：Claude Skills 可在 Claude.ai、Claude Code 和 Claude API 中使用。创建技能后，它可在所有平台之间移植，使工作流在各处保持一致。
