# 小伴对齐 WorkBuddy：专家 / 专家团 / 技能 / 连接器

决定：**两边不一致时以 WorkBuddy 为准**。依据是本机已安装的 WorkBuddy（`~/.workbuddy`）和它自带的规范（`plugins/cache/workbuddy-builtin/skill-expert-manager/*/references/`），不读取它的账号和令牌。

## 1. WorkBuddy 的做法（实测）

一切都是**插件**：目录 `<name>/<version>/`，清单在 `.codebuddy-plugin/plugin.json`。

| 类型 | 构成 |
|---|---|
| 专家（agent） | `agents/<名>.md`（frontmatter：name、description、displayName{en,zh}、profession{en,zh}、maxTurns；正文是人设提示词）、`avatars/expert.png`（512×512，≤500 KB）、`README.md`、可选 `skills/`。plugin.json：`expertType`、`agentName`、`displayDescription`（中文 40–50 字）、`categoryId`、`defaultInitPrompt`（= 第一条推荐提问）、**固定 3 个** `tags`、**固定 3 条** `quickPrompts`，全部中英双语 |
| 专家团（team） | 同一个插件里有主理人 + 成员各自的 agent md；`teamInfo{leadAgent, memberAgents[]}`、`members[]`（role lead/member，各有头像）；主理人文件名带团队前缀，按 SOP 调度成员，成员产出回传主理人 |
| 技能 | `skills/<slug>/SKILL.md`；列表条目有 skillId、slug、source（userSettings/builtin/plugin）、version、installedAt、iconUrl、userInvocable、disableModelInvocation、allowedTools |
| 连接器 | 插件里带 `mcp.json`（streamable-http / sse / stdio）、`.codebuddy-plugin/plugin.json` 的 `extensions["ai.workbuddy"]`（display：中英名称与说明、示例提问；setup：认证；policy：超时、可见范围）、`ai.workbuddy/token-schema.json`（令牌表单）、自带技能 |
| 认证 | 无需认证 / 令牌（表单字段写入 headers 或 env，本机加密保存：AES-256-GCM + HKDF）/ OAuth（服务端地址即授权入口）/ stdio 的环境变量。状态：enabled、userDisabled、headerOverrides、envOverrides；界面有“连接器中心”，可连接和解绑 |
| 市场 | 若干 marketplace（zip 包，带 etag 和 sha256），每个有 `marketplace.json` 或 `connectors/<id>/` 目录；官方连接器目录共 361 个，其中 252 个无需认证 |
| 对话里的引入 | 工作模式（专家模式等）的片段提示词；技能可由模型调用或用户调用；工具权限由系统统一分配，agent md 不能写 `tools` |

## 2. 小伴现状

- 市场服务：专家、技能、连接器三类条目；专家只有一份 SKILL.md 小包（54 个，依赖技能没有随包）；专家团是“引用别的专家”的列表，没有主理人；头像只能是远程链接；推荐提问是单语的任意条；专家分类是我们自己的 8 类。
- 桌面端：技能按 SKILL.md 安装；专家当作技能装，对话里只是往输入框塞一句“请加载…”；**没有人设、没有“召唤”**；已有 MCP 客户端、连接/断开/重连和 OAuth 授权流程（`ConnectorManager`、`ConnectorConfig.vue`），但清单不是 WorkBuddy 格式。
- 内核：子 Agent 工具 `Subagent`（单参数 task，返回最终文本），没有 TeamCreate / SendMessage；提示词可通过 `prompt_section` 动态追加；`maxTurns` 可由构建器设置。

## 3. 目标模型：以 WorkBuddy 插件为唯一格式

**包格式**：采用 WorkBuddy 插件目录（`.codebuddy-plugin/plugin.json`、`agents/`、`skills/`、`mcp.json`、`avatars/`、`ai.workbuddy/`）。路径规则里**放行** `.codebuddy-plugin/` 和 `ai.workbuddy/` 两个清单目录（服务端 `package.rs` 与客户端 `skill-discovery/package.rs` 同步改），其余点开头的路径、`..`、符号链接、大小限制照旧。校验器按 WorkBuddy 规范：3 个标签、3 条提问、中英双语、40–50 字描述、`defaultInitPrompt` 等于第一条提问。

**行业分类**：专家采用 WorkBuddy 的 12 类（`01-ProductDesign` … `12-IndustryConsultant`），带中英文名。

**权限**：WorkBuddy 的专家不收窄也不另外授予工具，由系统统一分配；小伴按同样原则，人设不增减工具，工具照常受用户当前的权限模式约束（提示词里现有的“专家不能增加工具、权限或记忆”那句改成这个说法，不再暗示专家会被限制）。

## 3.1 第 0 期实测结论（拿本机 WorkBuddy 当参照物验证）

- 本机 362 个 WorkBuddy 插件（专家 2、连接器 302、技能插件 58）都能经我们的校验并安装，测试读取 `~/.workbuddy`，没有就跳过。
- 与规范文档不一致、**以实际发布的插件为准**的地方：
  - 头像可以超过 500 KB（实测 1.4 MB），所以这条只是建议，宽松模式下是警告。
  - 版本号不一定是三段式（实测 `2026.10.23+novice.12`），只要求非空。
  - 插件清单里可以有 `hooks`（会执行命令）：小伴只解析不执行，审查时提示。
  - 连接器可以把 `.mcp.json` 放在根目录（7 个），路径规则放行这一个隐藏文件。
  - 插件可以很大：实测最多 635 个文件、5.5 MB、路径 9 层，所以插件包文件数上限 2048、路径深度 12，总大小仍限 12 MiB。
- 清单目录 `.codebuddy-plugin` / `.workbuddy-plugin` 只在最外层放行；包着一层目录时先去掉再检查。

## 3.2 实施进度

| 期 | 状态 | 说明 |
|---|---|---|
| 0 格式与安装骨架 | 完成 | 解析和校验、路径规则、桌面端安装目录和命令 |
| 1 技能对齐 | 完成（界面显示字段除外） | 头部字段、`/` 调用、模型/用户各自可见；技能详情页还没显示版本、作者等字段 |
| 2 专家 | 完成 | 服务端：上传插件包、12 个行业分类、头像存储、SkillHub 专家转换（子技能随包）。桌面端：安装插件专家、对话绑定专家、人设注入和 `maxTurns`、召唤（输入框 ＋ 菜单和 AI 插件页）、新任务页的专家介绍和 3 条推荐提问、对话头部显示专家 |
| 3 专家团 | 未开始 | 先写内核团队运行时的设计 |
| 4 连接器 | 未开始 | |
| 5 导入互通 | 未开始 | |

上线顺序要点：服务端新格式的专家只有新版桌面端能装；已有的 54 个 SKILL.md 小包专家保持原样可用，等新版桌面端发布后再在后台把它们重新同步成插件专家。

## 4. 分期（每期可单独发布）

**第 0 期 · 格式与安装骨架**
- 新增一个小库解析 WorkBuddy 插件（plugin.json、agent md、mcp.json、token-schema）；服务端和客户端各用一份实现。
- 桌面端安装目录：`<存储>/chat/plugins/<name>/`，带回执；插件里的 `skills/` 作为只读技能根加载。技能保持原名，来源用 `pluginId = 插件名@市场` 标识（WorkBuddy 的实际做法，实测确认）。
- 服务端和客户端放行清单目录 `.codebuddy-plugin/`（以及 WorkBuddy 内置插件用的 `.workbuddy-plugin/`）。`ai.workbuddy/` 本来就不以点开头，无需放行。
- “按 WorkBuddy 插件上传”的后台入口放到第 2 期，和专家入库一起做。

**第 1 期 · 技能对齐**
- 技能元数据：skillId、source、userInvocable、disableModelInvocation、allowedTools、来源市场、安装时间、图标。
- 对话里：输入 `/` 列出可用技能，选中后作为用户调用写入本轮；设置“仅用户调用”的技能不进模型的技能索引。
- 技能详情和已安装页显示这些字段。

**第 2 期 · 专家（agent）**
- 服务端：专家的人设、双语名称与职业、头像（内容寻址存储 + 公开路由）、3 标签、3 提问、12 类分类；SkillHub 专家转成人设专家，并把 `orchestration.children` 的子技能**打进插件的 `skills/`**（解决现在子技能缺失）；现有 54 个专家启动时自动迁移。
- 桌面端：会话记录绑定的专家；`build_agent` 追加人设段落、取 `maxTurns` 的较小值、预加载技能；输入区“召唤专家”选择器（取代现在塞一句话的做法），会话头部显示头像、名字、职业；新对话时展示 3 条推荐提问；侧栏“我的专家”。
- 市场网页和桌面专家页：卡片、详情、预览按 WorkBuddy 的展示字段重做。

**第 3 期 · 专家团**
- 一个插件含主理人和成员，成员各有头像和 agent md。
- 桌面端：主理人会话里，每个成员注册为一个子 Agent 工具（人设作为系统提示，权限与主会话共用），主理人按 SOP 调度并汇总。
- 与 WorkBuddy 的差异需要你拍板，见第 5 节。

**第 4 期 · 连接器**
- 服务端：连接器改为 WorkBuddy 连接器插件（mcp.json + extensions + token-schema + 技能）；后台可按 WorkBuddy 格式上传和编辑。
- 桌面端：安装 → 启用/停用 → 认证（无需认证 / 令牌表单 / OAuth / env）→ 解绑；令牌表单读 token-schema；凭据本机加密保存；示例提问作为快捷入口；连接器随专家和技能一起安装。
- 连接器中心页：状态、工具列表、断开、重新授权。
- 对话里：已连接连接器的工具按需出现（沿用延迟加载），调用时仍走权限确认；`@` 引入指定连接器、专家、技能。

**第 5 期 · 导入互通**
- 桌面端“从本地插件包安装”，市场后台“导入 WorkBuddy 插件包”；两边的插件可互相使用。

## 5. 已决定

1. **内容来源**：除自己制作和转换 SkillHub 外，**导入 WorkBuddy 官方市场的专家和连接器**。导入时保留原作者、来源和各插件自带的 `license`，没有声明许可的条目标记为“来源：WorkBuddy 官方市场”并默认存为草稿，由你在后台逐个确认再上架；导入放在第 5 期，不在此之前执行。
2. **专家团调度**：做**真正的团队运行时**（内核增加 TeamCreate / SendMessage、成员并行和互发消息），和 WorkBuddy 一致。第 3 期先写内核设计，再动手。
3. **期数顺序**：0 → 1 → 2 → 3 → 4 → 5。

## 6. 验证

- 用本机 WorkBuddy 当参照物：解析 `accounting-supervisor`、`finance-data`、连接器目录里的 361 个 `mcp.json` 与 `plugin.json`，结果与 WorkBuddy 自己的字段对得上（测试读取 `~/.workbuddy`，不存在就跳过，不把对方文件放进仓库）。
- 服务端：校验规则、迁移、头像存储、导入转换各有单元测试；端到端测试（真实 PostgreSQL）覆盖创建、上架、下载、清单摘要。
- 桌面端：会话绑定专家后系统提示词、`maxTurns`、技能预加载的单元测试；选择器、连接器表单的前端测试；在预览里走完整流程截图确认。
- 每期上线前备份数据库，迁移幂等。
