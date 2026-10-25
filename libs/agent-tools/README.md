# Agent Tools

Kernel 外的通用工具扩展，按能力放在独立子目录中。各工具包独立管理依赖和测试，由宿主选择接入；不建立默认启用全部工具的聚合 crate。

| 目录 | crate | 能力 |
| --- | --- | --- |
| [computer-use](computer-use/README.md) | agent-computer-use-tools | Open Computer Use 原生引擎分端打包、启动与配套 Skill |
| [files](files/README.md) | agent-file-tools | 文件读取、文档提取、会话附件、分析 Skill 与脚本 |
| [skill-discovery](skill-discovery/README.md) | agent-skill-discovery | 技能检索、审查、安装与 WorkBuddy 插件解析 |
| [connectors](connectors/README.md) | agent-connector-tools | MCP 连接、OAuth、插件配置与加密凭据保险箱 |

新增类型时在本目录新增独立工具包，复用 agent-kernel 的 Tool/ToolResult 接口。解析器、平台依赖及运行环境留在对应工具包，agent-kernel 不反向依赖这些扩展。

文件工具验证：`cargo check --manifest-path libs/agent-tools/files/Cargo.toml --locked`。
