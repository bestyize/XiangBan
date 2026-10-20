# Agent Tools

Kernel 外的通用工具扩展，按能力放在独立子目录中。各工具包独立管理依赖和测试，由宿主选择接入；不建立默认启用全部工具的聚合 crate。

| 目录 | crate | 能力 |
| --- | --- | --- |
| [files](files/README.md) | agent-file-tools | 文件读取、文档提取、会话附件、分析 Skill 与脚本 |

新增类型时在本目录新增独立工具包，复用 agent-kernel 的 Tool/ToolResult 接口。解析器、平台依赖及运行环境留在对应工具包，agent-kernel 不反向依赖这些扩展。

文件工具验证：`cargo check --manifest-path libs/agent-tools/files/Cargo.toml --locked`。
