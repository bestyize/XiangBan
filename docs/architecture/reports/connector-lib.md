# L：连接器库（connector-lib）

范围：只修改 `libs/agent-tools/connectors` 与本汇报；分支 `codex/connector-lib`。未修改 agent-kernel、桌面 src-tauri、前端、工作流；未 push。

## 已完成

- 新增 `plugin` 模块：清单、双 MCP 文件名、双 Streamable HTTP 拼写、display、令牌 schema、headers/env/args/url 原样模板、CLI/OAuth/Token/None 认证、disabledTools、毫秒 timeout、技能目录与图标路径。资源路径 canonicalize 后限制在插件目录内。
- `build_config` 校验必填、未知字段、控制字符与 header 格式，替换 `${KEY}` 并拼接认证 header 前缀。同名 header 按大小写不敏感覆盖。
- 新增 `vault`：AES-256-GCM、HKDF-SHA256，每个连接器/字段独立 info、随机 96-bit nonce、身份绑定的 AAD；支持 set/get/has/remove_connector，密文错误返回错误。Windows 主密钥用当前用户 DPAPI；Unix 0700/0600。密文与状态单文件原子写；同进程多个 Vault 实例对同一目录的读写互斥。
- 新增插件连接 API：`connect_plugin` 合并 vault 中已有字段后调用原有连接流程，记录只保存模板快照和 OAuth 会话引用。`restore` / `reconnect` 从 vault 重新取值；`tools()` 与 toolCount 过滤策略禁用工具，timeout 传入两种既有客户端。
- `begin_connect_plugin` 复用原有 OAuth 浏览器流程，插件 OAuth 初次保存和 refresh token 轮换同样加密；替换授权会话及解绑会清除旧凭据。旧配置 API 的明文配置/旧 OAuth 存储行为保持兼容，不自动迁移旧数据。
- `connector-states.json` 持久化 enabled、user_disabled、header_overrides、env_overrides；覆盖值进 vault，JSON 只存引用。增加字段名查询、启停、覆盖配置、解绑 API。断开/停用撤销旧工具；解绑也取消待完成授权。
- 原有 stdio 测试在 Windows 使用 `python`，其他系统使用 `python3`；原 OAuth mock 在 accept 后显式恢复 blocking，修复 Windows 继承 nonblocking 导致的 WouldBlock 与测试析构二次 panic。

## 未做及保守选择

- **旧式 HTTP+SSE 尚未实现。** 按 3.3 节的允许降级路径，保留可加载的 `Transport::Sse`，`ServerSpec::unsupported()` 返回“旧式 SSE 暂不支持”；`build_config` / `connect_plugin` 明确拒绝，不会误用 HTTP POST，不会存凭据或访问网络。8 个官方 SSE 服务受影响（其中 3 个是 token）。MCP HTTP 客户端位于范围外的 agent-kernel，此包没有修改它或新增旧式 SSE 传输。新增本地监听 mock 验证拒绝路径，不声称完成 SSE 握手测试。
- 全部 361 个官方目录中 58 个只有 skills。按 oracle “全部能 load”要求允许 `servers=[]`；宿主应按技能包处理，不能当作可连接服务。
- 未声明认证的远端标为 `AuthSpec::None`，构造 `authMode:auto`，保留遇到 401 时的 OAuth 发现；不凭空推断平台私有授权。
- `preAuth:cli` 仅表达认证类型，实际依赖用户本机已安装/登录的 CLI；不实现 WorkBuddy 私有代授权或自动安装 runtime。
- 多服务插件按 `connection_id(server)` 分别管理、隔离凭据。配置时持久化确认过的模板快照；升级后需用新插件对象重新确认连接，恢复时不会静默重读已改变的包。
- 覆盖项磁盘值采用 vault 引用，保证 token 不落入明文 JSON；运行时应用实际覆盖值。`Vault::has` 使用 `Result<Vec<String>, String>`，避免把密文损坏伪装成“未配置”。
- 为保持旧公开 API 兼容，直接调用旧 `connect(build_config(...))` 仍沿用旧明文保存方式；插件接线必须使用新插件 API。首次网络连接失败后，已通过表单校验的凭据保留在 vault，允许重试或 `unbind_plugin(id)` 清除。
- 本机验证 Windows 分支；未在 macOS/Linux 执行权限测试，未连接真实第三方账号。

## 验证

命令：`cargo test --offline --manifest-path libs/agent-tools/connectors/Cargo.toml`。

最终结果：**17 passed / 0 failed**（5 单元测试、6 个 `tests/oauth.rs` 用例、6 个 `tests/plugins.rs` 用例；doc-tests 0）。包含全部原有测试。

- 本机真实 WorkBuddy oracle：361/361 目录 load 成功，58 个 skill-only；67 个 token 服务中 64 个 build_config 成功，3 个 SSE 明确 unsupported；全部 8 个 SSE 服务都可解析并可查询 unsupported。参考目录只读，未复制其文件。
- 本地 stdio：模板 env 注入、策略过滤、旧工具撤销、重启恢复、加密覆盖配置、启停持久化与解绑。
- 本地 HTTP：Bearer header 注入、策略过滤、120ms 策略导致慢调用超时；保留原 Streamable HTTP 响应 SSE/分页测试。
- 本地 OAuth：发现、PKCE/state、刷新、资源绑定、取消、代际撤销；插件加密存储、轮换、重启恢复、重新授权、解绑；递归检查存储文件不含明文 access/refresh token。
- Vault：往返、随机 nonce、无效 nonce/base64、损坏密文、字段/连接器之间调包、更换 DPAPI 主密钥、丢失主密钥等负面用例。

测试仅有依赖 agent-kernel 已存在的 sandbox dead-code warnings。本次未运行桌面 cargo 构建。

## Claude 需要接线的范围外事项

1. 桌面安装插件后调用 `ConnectorPlugin::load`，按 server 展示；`unsupported()` 非空时传入前端 `unsupported`，显示“暂不支持”，禁用连接操作。图标/技能的 UI、data URL、安全展示仍由桌面层负责。
2. configure 使用 `connect_plugin`；需要 OAuth 自动发现或重新授权时使用 `Arc<ConnectorManager>::begin_connect_plugin`，继续使用现有授权状态/取消 API。不要把 build_config 的含凭据结果传给旧 connect。
3. UI ID 与 `connection_id(server)` 对齐，configured 从 `configured_plugin_fields` 取字段名；启停用 `set_plugin_enabled`、解绑用 `unbind_plugin`。状态/覆盖接口见本 crate README；不要将 vault 引用当作凭据回显。
4. 新增 ring 与 Windows DPAPI 依赖已更新此 crate 的 Cargo.lock；桌面依赖图由 Claude 在主目录离线构建时更新对应锁文件。
5. 如需解除 8 个旧式 SSE 连接器的限制，在 agent-kernel 范围内补旧式 GET event-stream + endpoint POST 传输及完整 mock 测试，再接入此库；现有 Streamable HTTP 客户端不能当作旧式 SSE 使用。

## 提交阻塞

实现、17 个测试和汇报已完成，`git diff --check` 通过，但 **尚未生成 commit**。执行 `git add` 与 `git commit` 均被本机权限拒绝：

```
fatal: Unable to create 'C:/Users/sdjz0/Documents/Claude/XiangBan/.git/worktrees/lib/index.lock': Permission denied
```

只读检查确认不存在遗留 index.lock；实际 worktree Git 目录 ACL 含显式 Deny Write。当前执行环境不允许提权，未修改 ACL、未切分支或绕过限制。需要修复该 Git 目录对当前执行身份的写权限后，在本 worktree 暂存上述文件并提交；目标提交信息：`feat(connectors): add WorkBuddy plugin loading and encrypted credential vault`。没有 push。
