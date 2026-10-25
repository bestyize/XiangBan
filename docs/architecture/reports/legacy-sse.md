# L2：旧式 HTTP+SSE MCP 传输

已完成，改动留在当前 worktree，未执行 git add / git commit / push。范围仅 `libs/agent-kernel`、`libs/agent-tools/connectors` 和本报告；未修改或编译桌面端 src-tauri，未修改依赖或工作流。

## 实现

- 新增公开 `agent_kernel::mcp::SseMcpClient`，复用 `HttpMcpConfig`、`HttpAuthorization`，提供与 HTTP 客户端一致的 `connect`、`connect_with_authorization`、`connect_cancellable`、`tools`、`is_connected`、`needs_review`、`close`。
- 与 Streamable HTTP 共用 initialize、initialized、工具分页/调用、工具权限与超时、结果解码、未知执行结果的 `needs_review` 门禁。旧式 SSE 发起 MCP 2024-11-05 握手。
- GET 携带配置 headers / 动态授权及 `Accept: text/event-stream`；独立读取线程维持流。endpoint 相对地址按原 SSE URL 解析，拒绝跨源、用户信息、fragment 和重复 endpoint；HTTP 重定向保持禁用。
- 请求先登记 ID，再 POST 至 endpoint，等待 SSE 上的对应响应；支持并发、乱序、忽略未知 ID 和通知。服务端 ping 返回空结果，其他服务端请求返回 Method not supported。
- 解析支持多行 data、注释、retry、CRLF/LF、任意网络分块；每个事件（含未终止行）限制 8 MiB，不对整个长连接累计限额。retry 不触发自动重连。
- 策略毫秒 timeout 限制请求，GET 长流不套用整个响应的 reqwest timeout。断流、关闭、取消、超时使连接失效、关闭流并结束其他等待调用；已经派发的工具保留结果未知门禁，显式断开后清除。
- GET / POST 的 OAuth 401 均支持一次刷新重试。连接器的授权发现按 SSE GET 探测；GET 成功而 POST 需要授权时，保持流存活并向同源 endpoint 探测，复用原元数据发现、注册、PKCE、scope/resource、回调及凭据存储流程。
- `Transport::Sse` 不再 unsupported；`build_config` 输出 `transport: "sse"`，`normalized_config` 保留传输标识，`start` 选择对应客户端。两个库 README 已更新。

## 验证

全部在本机 Windows 离线执行并通过：

| 命令 | 结果 |
| --- | --- |
| `cargo test --offline --manifest-path libs/agent-kernel/Cargo.toml` | 238 passed，0 failed |
| `cargo test --offline --manifest-path libs/agent-kernel/Cargo.toml --features mcp,native-tls` | 278 passed，0 failed |
| `cargo test --offline --manifest-path libs/agent-tools/connectors/Cargo.toml` | 18 passed，0 failed |
| `git diff --check` | 通过 |

新增 Rust 标准库 TCP mock（内核/连接器测试共用）覆盖完整握手与工具调用、两页工具、相对/绝对 endpoint、跨源拒绝、并发乱序、多行/注释/retry/CRLF、服务端 ping、未知 ID、断流、超大未终止事件、启动取消/超时、运行中取消与其他等待者退出、策略超时关闭实际流、GET/POST 401 刷新。

连接器测试覆盖 SSE 插件成功连接、vault token 头注入、disabledTools、毫秒 timeout、凭据不出现在连接记录/状态、断开后的旧工具撤销；OAuth mock 覆盖 GET 与 POST 两种 401 的发现→授权→调用→刷新→重启恢复。

本机 WorkBuddy oracle：361/361 加载，58 个 skill-only；67 个 token 服务全部 build_config 成功，8 个 SSE 服务可用，不再标为 unsupported。未访问真实第三方账户。编译输出仅有原有 sandbox dead-code warnings。

## 边界与交接

不自动重连或重放远程操作；断流/取消/超时后需显式重连。关闭本地连接不保证远程写操作已经停止，因此继续沿用 `needs_review`。未在 macOS/Linux 或真实 SaaS 服务上实测。

库层已接通，无需新增桌面 Rust 接线；Claude 可提交当前工作区改动。宿主若另有硬编码的 SSE unsupported 判断，需要在其负责范围内移除。
