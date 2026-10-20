# agent-computer-use-tools

可复用的 Open Computer Use 接入包，依赖 agent-kernel 的 MCP 客户端，不依赖 Tauri。

- `config()`：优先检测本机 `open-computer-use -v`，可用时启动 `open-computer-use mcp`，否则沿用 `npx -y open-computer-use mcp`（Windows 使用 `npx.cmd`）。
- `spawn()`：通过 Kernel 的 McpClient 启动服务并发现工具，工具风险仍为 Exec，调用超时仍为 60 秒。
- `SKILL`、`SKILL_NAME`、`SKILL_ORIGIN_MARKER`：提供随包 Skill 及其已有安装副本识别标记。Skill 内容原样迁移。
- 宿主继续负责启用开关、系统权限、Skill 副本同步、工具注册和界面提示。
- 实际电脑控制实现仍在外部 open-computer-use 服务中，Kernel MCP 通信实现不迁移。

```rust,ignore
let client = agent_computer_use_tools::spawn()?;
for tool in client.tools() {
    builder = builder.tool_arc(tool);
}
```

验证：`cargo test --manifest-path libs/agent-tools/computer-use/Cargo.toml --locked`。测试不会启动真实电脑控制服务或通过 npx 安装依赖。

