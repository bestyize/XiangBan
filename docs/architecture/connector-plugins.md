# 连接器插件：格式与接入边界

依据：本机 WorkBuddy 的官方连接器市场（`~/.workbuddy/plugins/marketplaces/workbuddy-connector-plugins-official/connectors/`，361 个插件）。这是包格式兼容性的参照；当前支持范围以下文和代码为准。参考物只读，**不要把它的文件复制进仓库**，测试读取该目录，不存在就跳过。

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

历史样本统计（361 个，非实时目录）：58 个没有独立 mcp 文件，其中 57 个仅技能，`tencent-map` 仅在 `extensions["ai.workbuddy"].urlTemplatedMcpServers` 声明服务，当前导入器不支持；`streamable-http` 278、`streamableHttp` 2、`stdio` 16、`sse` 8；认证声明：`token` 67、无 `setup`（免认证或走 OAuth 发现）226、有 `setup` 但没有 `auth.type`（`preAuth: "cli"`、`runtime`、`npmRegistry`）10。**没有任何一个声明 `license`。**

`extensions["ai.workbuddy"]`：

- `display`：`name`/`name_en`/`name_zh`、`description_en`/`description_zh`、`examples_en`/`examples_zh`（示例提问数组）。
- `setup.mcpServers.<server>.auth`：`{ "type": "token", "schema": "./ai.workbuddy/token-schema.json", "headers": { "<Header>": { "field": "<KEY>", "prefix": "Bearer " } } }`；stdio 的令牌写进 `env`（mcp.json 的 `env` 值是 `"${KEY}"` 占位）。另见 `preAuth`（`cli`）、`runtime`（`{type:"node",version}`）、`npmRegistry`。
- `policy.mcpServers.<server>`：`disabledTools[]`（要隐藏的工具）、`timeout`（毫秒）；`minWorkbuddyVersion`。

`token-schema.json`：`{ title, title_en, description, description_en, fields: [{ key, label, label_en, type: "password"|"text"|…, required, placeholder, placeholder_en, description, description_en }] }`。

mcp.json 的服务条目：`{ "type": "streamable-http"|"streamableHttp"|"sse"|"stdio", "url" | "command"+"args"+"env", "headers"? }`。

认证三类：无需认证；令牌（表单字段写入 headers 或 env，**本机加密保存**）；OAuth（没声明 `auth`、远端服务返回 401 时按 MCP 授权规范发现，小伴已有 `authMode: auto/oauth`）。

## 2. 实现入口与边界

| 层 | 实现入口 | 当前状态 |
| --- | --- | --- |
| 连接器库 | [connectors README](../../libs/agent-tools/connectors/README.md) | 插件解析、表单、加密保险箱、策略过滤、OAuth；支持 stdio、Streamable HTTP 和旧式 HTTP+SSE |
| 市场服务 | [市场 README](../../services/server-xiaoban-market/README.md) | 连接器/专家插件上传、公开摘要、图标校验、版本与清单摘要；官方市场批量导入默认草稿 |
| 桌面宿主 | `client/xiaoban-desktop-wallpaper/src-tauri/src/chat/connectors.rs`、`plugins.rs` | 连接器管理命令、结构化引用、本地包选择与摘要锁定安装 |
| 桌面界面 | `client/xiaoban-desktop-wallpaper/src/ui/ai-store/` | 连接器中心、令牌表单、安装审查；旧式 SSE 包与其他传输使用同一套审查、摘要锁定和确认流程 |

旧式 SSE 与其他传输共用市场安装及连接流程。用户明确选择“安装并连接”后，远程服务自动试连，令牌表单和 OAuth 浏览器授权在同一安装窗口完成；本地命令仍需单独点击运行。断流、超时或取消后不自动重放远程动作；结果未知时需核查，关闭本机连接不能撤回远端写操作。标准 MCP OAuth 可复用，WorkBuddy 私有身份与服务专属 CLI 登录仍需独立接入。

## 3. 桌面接口约定

以下命令的实际参数与返回字段以宿主实现和 `src/ui/ai-store/connector-plugins.ts` 为准：

- `ai_connector_installed` 返回安装信息、市场 namespace、传输方式、启动命令、认证字段 schema、已保存字段名、连接状态和工具；不返回凭据值。远程未声明认证时使用 `auth.type: auto`，不预判为免认证或 OAuth。
- `ai_connector_configure(id, values, confirmed)` 在用户确认后保存字段并连接；成功返回连接器信息，自动发现 OAuth 时返回 `status: authorization_required`、`flowId` 和 `authorizationUrl`，沿用授权状态/取消命令。已保存字段留空表示保留原值，清除使用 `ai_connector_unbind`。
- `ai_connector_set_enabled`、`ai_connector_unbind`、`ai_connector_reauthorize` 分别管理启停、解绑与 OAuth 重新授权；授权状态/取消沿用 `ai_store_connector_auth_status/auth_cancel`。
- `ai_mentions` 提供专家、技能、连接器引用。连接器以 `mentions: [{kind,id}]` 传给 `chat_send`，不往可见消息正文塞提示词。
- `ai_plugin_pick_file` 取得原生 ZIP 路径，`ai_plugin_inspect_file(path)` 审查，`ai_plugin_install_file(path, sha256)` 锁定已审查摘要后安装。仅安装不启动服务；市场的“安装并连接”明确包含远程连接意图，本地命令仍需单独点击运行。

对话工具 `connector_search` 返回最多三个候选与 reference；`connector_request` 使用已有的 pending question broker 展示连接卡片、支持快照恢复并等待用户操作，结束时核实真实连接状态及市场 namespace。成功后返回 `tools_for(ids)` 的工具 schema，`connector_call` 动态调用当前工具，保留结果中的媒体及结构化数据；该入口与普通 MCP 工具一样逐次确认、输出不可信，断开后的调用会被拒绝。模型不能通过这两个发现工具静默安装插件或代填凭据。

多服务插件逐服务连接、隔离凭据。插件 API 使用保险箱保存表单与覆盖项；旧 `connect` API 的明文配置兼容行为仍在，宿主插件接入应使用 `connect_plugin` / `begin_connect_plugin`，不要退回旧接口。

## 4. 市场导入与验证

批量导入命令、草稿规则、来源与许可审查见 [市场 README](../../services/server-xiaoban-market/README.md#workbuddy-插件与官方市场导入)。包中 SVG 使用被动绘图元素白名单，危险内容直接拒绝；公开摘要不包含令牌值。疑似密钥警告不替代发布前人工审查。

库测试使用本机 stdio / HTTP mock；真实 WorkBuddy 样本只读且不存在时跳过。市场端到端测试需要独立 PostgreSQL，桌面交互和真实 SaaS 授权需实机验证。不要把历史工作包测试数量当作当前全量验收结果；各项目 README 提供可复现命令。
