# Agent Tools / Connectors

用户配置的 MCP 连接管理，供小伴桌面 AI 商店与 Agent 共用。仅依赖 agent-kernel，不依赖 Tauri。

- 支持本地 stdio 和远程 Streamable HTTP（JSON / 响应 SSE），暂不支持旧式 HTTP+SSE。
- 接收单个服务配置或只含一个服务的标准 mcpServers JSON；配置模板来自 SkillHub README，读取模板不会启动服务。
- 用户确认后才连接或启动本地命令。Node/Python 等依赖需本机提供；npx 自身的下载行为由用户确认的命令决定。
- 完成 initialize、initialized 和 tools/list 后才报告连接成功。配置落盘失败会关闭刚创建的连接。
- 凭据只保存在连接器配置中，不出现在状态列表、工具描述或系统提示词。Unix 配置目录权限 0700、文件权限 0600；Windows 使用用户数据目录的继承 ACL。
- 工具名称经过隔离和长度规范化，保留原工具调用映射。默认 Exec、结果不可信、逐次确认；宿主 Auto danger_check 也必须识别 mcp_ext_ 前缀。
- 断开连接会关闭客户端并撤销旧工具的调用门禁；进行中的远程操作不能回滚。重连时新任务取得新的工具列表。
- 应用退出调用 shutdown；保留用户已启用的配置，下次启动恢复连接。普通聊天不因连接器配置损坏而无法运行。

## 验证

    cargo test --manifest-path libs/agent-tools/connectors/Cargo.toml

测试使用本机 Python stdio 服务与 Rust TCP 模拟 HTTP 服务，不访问真实第三方账户，覆盖连接、分页、SSE 返回、凭据隐藏、持久化和断开后的旧工具撤销。
