# 桌面模型供应商配置

管理入口：`/admin/ai-providers`。配置保存在已有 PostgreSQL `app_settings` 的 `ai.providers` 项，无需新增数据库迁移。未配置时沿用四家第三方供应商作为默认目录。

- `GET /api/v1/ai/providers`：完整目录，支持 ETag / 304；未登录仅返回 third，登录后增加 free，paid 仅返回后台明确授权给该 UID 的供应商和令牌。
- `GET /api/v1/admin/ai/providers`：管理员读取目录，返回 hasKey 标记，不返回凭证明文。
- `PUT /api/v1/admin/ai/providers`：管理员更新整个目录，携带当前 revision；过期修订被拒绝。apikey 留空保持原值，clearKey 清除凭证。模型/供应商停用后不再下发。

目录字段：version、revision、refreshAfterSeconds、xiaobanAi、providers。`xiaobanAi.enabled`（缺省为 true）控制桌面端设置里内置的“小伴 AI”入口（账号登录加额度的官方服务，由 server-ai 提供，不在 providers 列表里）：关闭后桌面端在下次刷新时隐藏该卡片、不能再启用，正在用它聊天的会提示改选其他服务商；管理入口在 `/admin/ai-providers` 页顶部的开关。供应商字段：id、provider、description、endPoint、protocol、type、defaultModel、models；third 不下发 apikey。模型字段：id、name、description。后台额外包含 enabled、clearKey、userAccess；这些字段不下发给桌面。

type 为 third / free / paid；custom 为客户端本地类型。协议为 OpenAI Responses、OpenAI Chat Completions、Anthropic Messages。可用 endpoints 配置不同协议的基础地址；consoleUrl / consoleText 用于获取第三方 Key 的入口。

免费凭证必须是专用受限凭证；付费 userAccess 为 {uid, apikey} 列表，每个用户配置其自己的网关令牌。调用权限、额度和计费由所配置的网关执行，本服务没有新增收费、订单或令牌签发功能。

桌面原生层在启动时刷新，默认每 6 小时刷新，轮询到期时间同时覆盖休眠恢复；账号变化立即刷新。请求失败保留缓存，重试有间隔，并发请求合并；凭证存入原有私有 ai.json，不进入网页界面。账号变化清除上个账号的官方凭证。

已有用户配置按手动选择迁移；新供应商默认跟随推荐。仅 recommended 选择随 defaultModel 更新，manual 保留。third 允许手填非推荐模型 ID，保留用户自己的 Key 和已有代理地址。custom 不参与同步。服务器删除供应商后本地保留不可用标记，不用离线默认目录重新启用。

聊天错误按原因展示，提供切换面板和显式重试。重试使用原记录与图片，且只允许尾部为未获得回复的用户消息；已持久化回复或工具动作不能直接重放。

针对性验证：桌面 vue-tsc；相关 AI 配置、聊天和错误分类 Vitest；后台 vue-tsc；Rust 环境运行服务端 ai_providers 测试、桌面 ai_catalogue 测试及 agent-kernel 的 retry_last 测试。
