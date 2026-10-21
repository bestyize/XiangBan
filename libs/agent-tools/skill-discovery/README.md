# Agent Tools / Skill Discovery

小伴桌面通过腾讯 SkillHub Open API 查找、审查、安装 Skill。桌面 Agent 默认注册全部工具。

| 工具 | 用途 | 授权 |
| --- | --- | --- |
| skill_search(keyword) | 关键词检索 | Network，不可信结果 |
| skill_inspect(slug, version) | 审查版本锁定的 manifest、SKILL.md 和脚本预览 | Network，不可信结果 |
| skill_install(slug, version, sha256) | 安装已审查的完整多文件包 | 每次需要用户批准 |
| skill_read(skill, path) | 阅读已安装、校验后的文本/引用文件 | Read，不可信结果 |
| skill_run(skill, script, args?) | 运行已安装并校验后的脚本 | 每次需要用户批准 |

## 生命周期

1. 通过 skill_search 找到目标，再选择固定版本。若未指定发布者，先从详情解析 namespace，再在文件清单、原文和下载请求中固定使用该发布者，校验清单返回的发布者身份。
2. skill_inspect 展示完整文件清单、各文件 SHA256、SKILL.md 正文和脚本预览；返回 manifest.sha256，即整个版本清单的摘要。
3. skill_install 用该 manifest.sha256 校验锁定版本，使用 SkillHub 下载 API 取 ZIP。每个 ZIP 路径都与清单比对，限制文件数量、体积、文件类型和哈希。若上游 ZIP 与清单不一致，改从同一发布者、同一版本的文件接口逐项下载，仍须全部满足原清单的大小与 SHA256；安装阶段绝不执行代码。
4. 全部验证成功后写入临时目录，并原子移动到用户的 chat/skills/<slug>。没有覆盖操作。
5. load_skill 按需加载 SKILL.md；skill_read 读取引用文件；只有用户再次批准 skill_run 才调用本机解释器。运行前复核整包内容与锁文件一致。

SkillHub API 文档：https://github.com/Tencent/skillhub/blob/main/docs/api/README.md

## 当前支持

- 多文件 SKILL.md + scripts/ + references/ + assets/，及其他一般文件，包括二进制素材。
- 解释器格式：Python (.py)、Node.js (.js/.mjs/.cjs)、Shell (.sh)、PowerShell (.ps1)、Ruby (.rb)。
- 脚本参数直接传给解释器，不通过 shell 插值。进程环境清理后只透传基本系统路径；限制输出和运行时间。
- ZIP 可以包含单个顶层包装目录。完整包最多 512 个文件、解压后 12 MiB，单文件 4 MiB，ZIP 12 MiB，SKILL.md 100 KiB。读取文件清单时核对 SkillHub 返回的 `count` 和实际文件数；空清单、数量不一致和超过上限会明确报错，不会安装不完整的包。
- 拒绝路径穿越、绝对路径、符号链接、Windows 保留路径、大小写重复路径、哈希不匹配和未经 API 文件清单列出的文件。
- 用户在「完全信任」模式下，安装与运行第三方脚本仍须逐次确认。

## 不保证的能力

**不是沙箱。** 脚本作为用户账户下的本地进程运行，可能访问宿主系统的文件和网络。SHA256 保障内容与审查清单一致，不代表内容安全。用户应亲自评估第三方脚本和依赖。

解释器依赖本机已经安装；不自动安装 Python/Node、pip/npm 包或额外的操作系统依赖。不支持需要独立后端、容器或复杂环境的脚本一键运行。

当前搜索与安装只连接 SkillHub；尚未支持 skills.sh、GitHub 直接导入、版本自动升级、可信签名验签和真正隔离的沙箱。文件大小超限或未经支持的归档明确拒绝。

## 验证

在可运行 Rust 的环境中分别执行：

    cargo test --manifest-path libs/agent-tools/skill-discovery/Cargo.toml
    cargo test --manifest-path client/xiaoban-desktop-wallpaper/src-tauri/Cargo.toml --locked

## 主动发现策略

用户明确请求时，可以直接搜索 Skill。否则模型必须给出缺少的能力和检查过的现有工具/技能，再使用 `skill_search` 的 `reason=capability_gap` 参数。对于 `reason=repeated_failure`，同一个工具必须在本轮发生至少两次不同调用的实际失败；权限、网络、配额、参数错误不属于能力缺口。

策略还包括：单任务最多三个不同搜索词、相同词不重复搜索、离线要求下禁止在线检索。模型在重复失败后宣布无法完成时，最多收到一次重新考虑 Skill 的提示。发现候选不等于批准安装；安装与脚本运行依然需要用户分别确认。

当前策略仅对单轮任务生效；没有后台搜索，也没有跨会话长期能力需求学习。


## 专家包与 MCP 目录

AI 商店专家页读取 `/api/v1/skillsets`，连接器页读取 `/api/v1/mcp/servers`。这两组接口已对照 SkillHub 网站实现及线上响应验证，目前尚未列入公开 Open API 总览；返回结构变化会显示错误，不回退到示例卡片。

专家包审查固定工作流正文、依赖发布者命名空间、版本、文件清单与脚本预览，生成整包摘要。安装时再次审查，摘要不一致则拒绝；所有依赖验证完成后发布工作流，失败回滚本次新增目录，不覆盖已有不同来源或版本。移除工作流保留共享依赖。依赖缺失或内容改变时标记未就绪。

部分历史技能没有 YAML frontmatter。仅对注册中心提供了匹配身份及非空描述的技能，允许将元数据写进本机锁文件；元数据同样纳入审查摘要，原始 SKILL.md 字节及文件 SHA256 保持不变，安装与加载共用此约定。有头部的文件仍使用标准 YAML 解析并核对名称，不用注册中心描述掩盖损坏头部。

MCP 目录读取详情与 README，并在平台提供摘要时校验 README SHA256；JSON 配置模板只用于展示，用户填写凭据并确认后才交给连接器管理器启动。WorkBuddy 专用注册脚本或路径不会自动在小伴执行。

只读线上验证（可选专家包标识）：

    cargo run --manifest-path libs/agent-tools/skill-discovery/Cargo.toml --example catalog-smoke -- tech-test-automation
