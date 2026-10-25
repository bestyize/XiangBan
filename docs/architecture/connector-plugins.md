# 连接器插件（第 4、5 期）：格式、接口约定和分工

依据：本机 WorkBuddy 的官方连接器市场（`~/.workbuddy/plugins/marketplaces/workbuddy-connector-plugins-official/connectors/`，361 个插件）。两边冲突时以 WorkBuddy 为准。参考物只读，**不要把它的文件复制进仓库**，测试读取该目录，不存在就跳过。

## 1. 实测的包格式

一个连接器就是一个插件目录：

```
<name>/
  .codebuddy-plugin/plugin.json      清单（name、version、description、keywords、extensions["ai.workbuddy"]）
  mcp.json  或  .mcp.json            { "mcpServers": { "<server>": {...} } }
  ai.workbuddy/token-schema.json     令牌表单（只有需要令牌的才有）
  ai.workbuddy/assets/icon.png|svg   图标
  skills/<skill>/SKILL.md            随连接器带的使用技能
```

统计（361 个）：无 mcp 的 58 个（只带技能的插件）；`streamable-http` 278、`streamableHttp` 2、`stdio` 16、`sse` 8；认证声明：`token` 67、无 `setup`（免认证或走 OAuth 发现）226、有 `setup` 但没有 `auth.type`（`preAuth: "cli"`、`runtime`、`npmRegistry`）10。**没有任何一个声明 `license`。**

`extensions["ai.workbuddy"]`：

- `display`：`name`/`name_en`/`name_zh`、`description_en`/`description_zh`、`examples_en`/`examples_zh`（示例提问数组）。
- `setup.mcpServers.<server>.auth`：`{ "type": "token", "schema": "./ai.workbuddy/token-schema.json", "headers": { "<Header>": { "field": "<KEY>", "prefix": "Bearer " } } }`；stdio 的令牌写进 `env`（mcp.json 的 `env` 值是 `"${KEY}"` 占位）。另见 `preAuth`（`cli`）、`runtime`（`{type:"node",version}`）、`npmRegistry`。
- `policy.mcpServers.<server>`：`disabledTools[]`（要隐藏的工具）、`timeout`（毫秒）；`minWorkbuddyVersion`。

`token-schema.json`：`{ title, title_en, description, description_en, fields: [{ key, label, label_en, type: "password"|"text"|…, required, placeholder, placeholder_en, description, description_en }] }`。

mcp.json 的服务条目：`{ "type": "streamable-http"|"streamableHttp"|"sse"|"stdio", "url" | "command"+"args"+"env", "headers"? }`。

认证三类：无需认证；令牌（表单字段写入 headers 或 env，**本机加密保存**）；OAuth（没声明 `auth`、远端服务返回 401 时按 MCP 授权规范发现，小伴已有 `authMode: auto/oauth`）。

## 2. 小伴现状（改动的基础）

- `libs/agent-tools/connectors`：`ConnectorManager`（`connect(id, name, config, confirmed)`、`disconnect`、`reconnect`、`statuses`、`tools`、OAuth 授权流程 `begin_connect`/`authorization_status`），记录存在 `root` 目录里；`normalized_config` 接受单个服务的 `{url, headers, authMode, clientId, scope}`（远端）或 `{command, args, env}`（stdio），**拒绝 `type: sse`、拒绝含 `YOUR_`/`${` 的占位值**。
- 桌面端：`ai_store_connector_*` 命令（`chat/store.rs`）；`ConnectorConfig.vue` 用启发式从 JSON 里认出占位符生成表单。
- 插件安装：`chat/plugins.rs` + `libs/agent-tools/skill-discovery/src/plugin.rs` 已能安装含 `mcp.json` 的插件到 `<存储>/chat/plugins/<name>/`，并解析 `token-schema`（`parse_token_schema`）。

## 3. 工作包

### 3.1 服务端（`services/server-xiaoban-market`，工作包 S）

1. **连接器插件入库**：`POST /admin/plugins/import` 现在只收专家。扩展为按清单判断类型：有 `agents/` 且 `expertType` 的是专家；有 mcp 服务的是连接器；只有 `skills/` 的归为技能包（可先返回明确的"暂不支持"错误，由下面第 5 期工具决定是否入库）。连接器的条目 `kind = 'connector'`，包存入既有的内容寻址包存储，版本、清单摘要沿用专家的做法。
2. **校验**（宽松模式，以实测为准）：`plugin.rs` 已有 `parse_mcp`/`parse_token_schema`；补连接器校验：名字、版本非空、至少一个 mcp 服务、`type` 取值、`setup.auth.type=token` 时 schema 文件存在且字段 key 与 `headers.*.field` / `${KEY}` 占位对得上、禁止把真实密钥写进包（headers/env 里不是 `${…}` 占位的疑似密钥值给出警告）。`hooks` 只提示。
3. **展示数据**：条目的 `presentation`（已有专家版）增加连接器版：中英名称/描述、示例提问、认证类型（`none|token|oauth|env`）、令牌字段摘要（不含任何值）、传输类型、随包技能数、被 `policy` 禁用的工具数；公开列表/详情接口返回 `connector` 对象；图标走头像同款内容寻址存储，**允许 SVG 但必须清洗**（拒绝 `<script`、`on*=` 事件属性、`<foreignObject`、外部 `href`/`xlink:href`/`url(` 引用、`<!ENTITY`），响应带 `Content-Security-Policy: default-src 'none'; style-src 'unsafe-inline'`、`X-Content-Type-Options: nosniff`。
4. **类别**：连接器分类沿用现有的（开发协作/办公文档/…），导入时用 `keywords`/名称/描述无法判断就放 `其他`，**不要自动新建分类**。
5. **网页**：`ItemCard.vue`/`DetailModal.vue` 显示连接器的认证方式、示例提问、随包技能；后台「内容库」的"上传插件包"按钮对连接器同样可用。
6. **测试**：单元测试 + `tests/api.rs` 的端到端（真实 PostgreSQL，**你没有数据库时不用运行，写好即可，由我在服务器上跑**）。用 `~/.workbuddy` 的真实连接器做 oracle（存在才跑）。

### 3.2 官方市场导入工具（第 5 期，工作包 S 的第二阶段）

服务端 crate 里加一个二进制 `import-marketplace`（`src/bin/import_marketplace.rs`）：

```
import-marketplace --dir <市场目录> --server <https://…> --token <管理员令牌> [--dry-run] [--only <name>,<name>]
```

- 市场目录可以是 `connectors/<name>/` 形式（连接器），也可以是带 `marketplace.json` 的（专家等，按该文件的 `plugins[]`/`source` 找到插件目录）。逐个把插件目录打成**确定性的 zip**（固定时间戳，排序，重复导入不产生新版本），经 `POST /admin/plugins/import` 上传。
- 导入一律是**草稿**，不上架：接口增加字段 `status=draft`（默认仍按现有行为）、`origin`、`attribution`。条目要记录 `origin = "workbuddy-official"`、原作者（`author`/`homepage`/`repository`，没有就空）、`license`（取 `plugin.json` 的 `license`，没声明写 `unknown`）。后台列表要能看到"来源"和"许可"，并能按"来源=WorkBuddy 官方市场 + 许可未知"筛出来，由管理员逐个确认后发布。
- 已存在且内容相同的跳过；同名不同内容按版本规则新增版本；失败的逐个报告，不中断。输出汇总（新增/更新/跳过/失败）。
- `--dry-run` 只校验打包，不上传。

### 3.3 连接器库（`libs/agent-tools/connectors`，工作包 L）

在 `agent-connector-tools` 里新增（不要改变现有公开 API 的行为，只增加）：

- `plugin` 模块：`ConnectorPlugin::load(dir: &Path) -> Result<ConnectorPlugin, String>` 读取上面的目录结构（不要依赖桌面端 crate；可依赖 `serde_json`；若需要 zip 解析等已有能力，库内自己实现，不要引入 skill-discovery）。字段：`name`、`version`、`display`（中英名称、描述、示例提问）、`servers: Vec<ServerSpec>`（`name`、`transport`、`url`/`command`/`args`/`env`/`headers` 原样模板、`auth: AuthSpec`、`disabled_tools`、`timeout_ms`）、`skills_dir`、`icon`。`AuthSpec`：`None | Token { schema: TokenSchema, headers: Vec<(header, field, prefix)> } | OAuth | Cli`。传输 `streamable-http|streamableHttp` 统一成一种，`sse` 要**真正支持**（8 个官方连接器用它）——在 `mcp` 的 HTTP 客户端旁补上旧式 SSE 传输，或在 `normalized_config` 放行并实现；做不到就明确标出并让这类连接器显示"暂不支持"。
- `build_config(&self, server: &str, values: &BTreeMap<String, String>) -> Result<Value, String>`：用表单值替换 `${KEY}` 占位并拼出 headers（含前缀），产出可直接交给 `ConnectorManager::connect` 的配置；缺必填字段、值含控制字符、字段 key 不在 schema 里都要报错。
- `vault` 模块：凭据保险箱。令牌值**不进 `ConnectorManager` 的明文记录和任何日志**。AES-256-GCM，密钥用 HKDF-SHA256 从本机主密钥派生（每个连接器、每个字段单独的 info 和随机 nonce），主密钥存在应用数据目录里的权限受限文件（Windows 用 DPAPI 或至少用户私有 ACL，macOS/Linux 0600；有系统钥匙串可用就优先，但不要新增重依赖时就用文件）。API：`Vault::open(dir)`、`set(connector, key, value)`、`get(connector, key)`、`remove_connector(connector)`、`has(connector) -> Vec<key>`。损坏/被篡改的密文报错，不 panic。
- 状态：`enabled`、`user_disabled`、`header_overrides`、`env_overrides`（WorkBuddy 的 `connector-states` 同义字段），持久化在 root 下的单独 JSON，原子写。
- 接入 `ConnectorManager`：`connect_plugin(plugin: &ConnectorPlugin, server: &str, values, confirmed)`——从保险箱取值 → `build_config` → 现有 `connect`；`policy.disabledTools` 在 `tools()` 里过滤；`timeout` 传给 MCP 调用。重启恢复（`restore`）时同样从保险箱取值而不是读明文。
- 测试：单元测试 + 对 `~/.workbuddy` 真实 361 个连接器的 oracle（存在才跑）：全部能 `load`；67 个 token 连接器的 `build_config` 在填满字段后产出合法配置；保险箱加解密/篡改/换主密钥的负面用例；SSE 传输用本地 mock 服务端测。

### 3.4 桌面前端（`client/xiaoban-desktop-wallpaper/src/ui`，工作包 U）

只改前端，对着下面的 Tauri 命令约定写，测试里 mock `invoke`；**不要改 Rust**，命令由我接。

命令约定（均返回 JSON，错误是字符串）：

- `ai_connector_installed()` → `[{ id, name, nameEn, description, version, source: "market"|"local", iconUrl: string|null /* data: URL */, auth: { type: "none"|"token"|"oauth"|"env"|"cli", schema?: { title, titleEn, description, descriptionEn, fields: [{ key, label, labelEn, type: "password"|"text", required, placeholder, description }] }, configured: string[] /* 已保存的字段 key，永远不含值 */ }, status: "disconnected"|"connecting"|"connected"|"needs_auth"|"error"|"disabled", enabled: boolean, error?: string, tools: [{ name, description }], examples: string[], skills: string[], unsupported?: string }]`
- `ai_connector_configure(id, values: Record<string,string>, confirmed: true)` → 同上一项（保存令牌并连接）。
- `ai_connector_set_enabled(id, enabled)` → 同上一项。
- `ai_connector_unbind(id)` → `null`（断开并清除凭据）。
- `ai_connector_reauthorize(id)` → OAuth 流程信息（与现有 `ai_store_connector_reconnect` 的返回一致，用现有 `ai_store_connector_auth_status/auth_cancel` 轮询）。
- `ai_mentions()` → `[{ kind: "connector"|"expert"|"skill", id, name, description, iconUrl?: string|null }]`，输入框 `@` 菜单的数据源。
- `ai_plugin_inspect_file(path)` / `ai_plugin_install_file(path, confirmed)` 已有（第 0 期），用于本地插件包安装。选文件用现有的文件选择方式（看 `ChatComposer.vue` 里附件是怎么选的）。

页面/组件：

1. **连接器中心**：AI 插件页「连接器」标签下，已安装的连接器卡片/列表显示状态灯、名称、版本、认证方式、工具数；详情含工具列表、示例提问（点一下把它放进新任务输入框）、随包技能；操作：启用/停用、断开并清除凭据、重新授权（OAuth）、配置令牌。状态文案用中文。
2. **令牌表单**：按 `auth.schema` 渲染（`password` 用密码框、必填校验、占位符、字段说明），标题和说明用 `title/description`（有 `titleEn` 时按界面语言选，没有语言设置就用中文）；已保存的字段显示"已保存"，不回显值；提交前有"确认连接"步骤（沿用现有的确认语气）。**不要再用 `ConnectorConfig.vue` 的启发式去猜占位符**来处理插件连接器（旧的市场 MCP 条目仍用它）。
3. **`@` 引入**：输入框里输入 `@` 弹出与 `/` 菜单同款的菜单（`ChatSlashMenu.vue` 可复用），数据来自 `ai_mentions()`；选中专家 = 召唤（走现有 `useExpert`）、选中技能 = 与 `/` 相同的写入、选中连接器 = 在输入框放一个"本次优先使用该连接器"的标签。**不要往消息文本里塞前置提示词**；标签随消息作为结构化字段 `mentions: [{kind,id}]` 传给 `chat_send`（Rust 端我来接，前端先把字段加到发送参数里）。
4. **本地插件包安装**：AI 插件页顶部加"从本地插件包安装"按钮：选 `.zip` → 调用 `ai_plugin_inspect_file` 展示审查结果（类型、名称、版本、技能/连接器/专家数量、警告，连接器要提示"安装后需要你确认并配置才会连接"）→ 确认后 `ai_plugin_install_file`。
5. 每一项都要有 vitest 测试（参照同目录现有测试的写法：`ChatPage.test.ts`、`ChatWorkbenchTools.test.ts`、`AiStorePage.test.ts`）。UI 规范见 `client/xiaoban-desktop-wallpaper/小伴桌面_桌面端UI改造规范.md` 和现有组件的样式变量，**沿用现有风格，不要引入新的样式体系**。

## 4. 分工和约束

| 工作包 | 谁 | 工作区 | 分支 |
|---|---|---|---|
| S 服务端 + 导入工具 | Codex | `../xb-wt/server` | `codex/server-connectors` |
| L 连接器库 | Codex | `../xb-wt/lib` | `codex/connector-lib` |
| U 桌面前端 | Codex | `../xb-wt/ui` | `codex/connector-ui` |
| 专家团内核 + 桌面接入、命令接线、合并联调 | Claude | 主目录 | `main` |

- 桌面 Rust 工程（`src-tauri`，target 目录 33 GB）**只在主目录由 Claude 改**，Codex 不要编译它。
- 每个工作包只改自己范围内的文件；需要改范围外的文件时，停下来在汇报里说明。
- 仓库规则见 `AGENTS.md`（先读）。不要提交 `target/`、`node_modules`；提交信息用中文或英文都行，写清做了什么；**不要 push**。
- GitHub Actions 只允许 `workflow_dispatch`，不要动工作流。
- 做完在 worktree 里提交并写一份简短的汇报（做了什么、没做/做不到的、怎么验证、范围外需要我接的地方）到 `docs/architecture/reports/<包名>.md`。
