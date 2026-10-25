# Agent Tools / Connectors

用户配置的 MCP 连接管理，供小伴桌面 AI 商店与 Agent 共用。基于 agent-kernel，不依赖 Tauri。

- 支持本地 stdio、远程 Streamable HTTP（JSON / 响应 SSE）和旧式 HTTP+SSE（`transport: "sse"`）。旧式 SSE 先 GET 事件流，再向同源 endpoint POST 请求；断流、取消或超时后关闭连接，需显式重连。
- 接收单个服务配置或只含一个服务的标准 mcpServers JSON；配置模板来自 SkillHub README，读取模板不会启动服务。
- 用户确认后才连接或启动本地命令。Node/Python 等依赖需本机提供；npx 自身的下载行为由用户确认的命令决定。
- 完成 initialize、initialized 和 tools/list 后才报告连接成功。配置落盘失败会关闭刚创建的连接。
- 旧配置 API 的 API Key / 环境变量保存在本机连接配置，OAuth access/refresh token 按授权会话单独保存；此兼容行为保持不变。插件 API 的表单凭据、覆盖值和 OAuth 凭据使用下述加密保险箱。凭据不出现在状态列表、工具描述或系统提示词。
- 工具名称经过隔离和长度规范化，保留原工具调用映射。默认 Exec、结果不可信、逐次确认；宿主 Auto danger_check 也必须识别 mcp_ext_ 前缀。
- 断开连接会关闭客户端并撤销旧工具的调用门禁；进行中的远程操作不能回滚。`tools_for(ids)` 返回指定连接当前可用的工具，宿主可在同一任务完成用户授权后重新获取工具；名称隔离、策略过滤及逐次确认规则与 `tools()` 相同。
- 应用退出调用 shutdown；保留用户已启用的配置，下次启动恢复连接。普通聊天不因连接器配置损坏而无法运行。

## 认证与桌面表单

桌面连接弹窗默认展示服务地址、认证方式与凭据输入框，JSON 收入“高级配置”。远程服务可选择自动识别、网页登录授权、填写密钥、无需授权；本机命令的环境变量和参数占位符以表单填写。修改配置后需重新确认。

收到 HTTP 401 时，自动识别模式可进入标准 MCP OAuth 流程；明确填写 Authorization 头时优先使用该凭据。网页登录支持保护资源元数据发现、授权服务器发现、动态公共客户端注册或手填客户端 ID、PKCE S256、state 校验与本机随机端口回调。授权与令牌请求携带 resource；仅请求手填或服务挑战要求的 scope，不自动申请所有支持的权限。远程授权端点要求 HTTPS；本机模拟服务允许 loopback HTTP。授权超时为 5 分钟，可取消或重新打开页面；完成 MCP 初始化及工具发现后才保存为已连接。

OAuth 令牌提前 30 秒刷新，收到 401 时再刷新并重试一次；刷新令牌轮换保存到原授权会话。取消或失败会清理未完成会话，更换配置会废弃旧授权。断开连接关闭工具并保留本机凭据供手动重连；此操作不等同于服务商撤销授权。应用重启只尝试恢复连接，不主动打开浏览器；需要重新登录时由用户点击连接。

服务需提供标准 MCP OAuth 元数据，并支持公共客户端的本机回调；不支持动态注册时需服务商预先分配客户端 ID。WorkBuddy 的私有身份、云端代授权与服务专属 CLI 登录不能直接复用，仍需服务商独立接入。旧式 SSE 的 GET 或 POST 返回 401 均可进入授权发现流程。

## 验证

    cargo test --offline --manifest-path libs/agent-tools/connectors/Cargo.toml

测试使用本机 Python stdio 服务与 Rust TCP 模拟 HTTP 服务，不访问真实第三方账户，覆盖连接、分页、SSE 返回、凭据隐藏、持久化和断开后的旧工具撤销，以及 OAuth 发现、注册、PKCE 回调、错误 state、取消、令牌刷新、刷新失败后的重新授权与资源绑定校验。

## WorkBuddy 插件 API

`plugin::ConnectorPlugin::load(&dir)` 读取插件清单、`mcp.json` / `.mcp.json`、令牌 schema、策略及技能/图标路径，不启动服务。只带技能的目录可以加载，其 `servers` 为空。`ServerSpec::unsupported()` 对已知传输（含 SSE）返回 `None`；SSE 的 `build_config` 保留 `transport: "sse"`。`streamableHttp` 和 `streamable-http` 统一为 `Transport::StreamableHttp`。

`build_config(server, &values)` 校验字段并替换模板，返回单服务配置；插件实际连接请使用 `ConnectorManager::connect_plugin(&plugin, server, &values, confirmed)`，让管理器从 vault 合并已存字段并加密保存新字段。需要 OAuth 时用 `Arc<ConnectorManager>::begin_connect_plugin(...)`，复用 `authorization_status` / `cancel_authorization`；显式重新授权也可再次调用此入口。

`connection_id(server)` 返回稳定的插件/服务 ID。多服务插件逐服务连接；凭据按此 ID 隔离。`configured_plugin_fields(id)` 只返回已保存字段名；`plugin_state(id)` 返回状态；`set_plugin_enabled` 启停；`unbind_plugin` 断开并删除表单凭据、覆盖凭据和当前 OAuth 会话。`set_plugin_overrides(id, &headers, &env)` 加密保存覆盖项并断开，需再显式重连。状态文件 `connector-states.json` 的 `header_overrides` / `env_overrides` 保存 vault 引用，使用时再取实际值，不能把引用字符串当作真实凭据展示。

插件记录保存确认过的模板快照与会话引用，恢复时重新从 vault 构造配置，不保存表单填写后的配置。工具策略在名称隔离前过滤，`toolCount` 也排除被禁用工具；策略 timeout 按毫秒传给 stdio / HTTP 客户端。

`vault::Vault` 使用 ring 的 AES-256-GCM 与 HKDF-SHA256，按连接器和字段派生密钥、每次写入生成随机 nonce，并认证连接器/字段身份。Windows 主密钥用当前用户 DPAPI 保护；Unix 目录 0700、文件 0600。`has` 返回 `Result<Vec<String>, String>`，以便将密文损坏报告为错误。不要删除或替换主密钥，否则已有凭据无法解密。

新增测试直接读取本机 WorkBuddy 官方市场（不存在时跳过），不会复制其文件到仓库。Unix 权限代码需要在对应平台验证；本机 Windows 测试覆盖 DPAPI、密文篡改和更换主密钥。
