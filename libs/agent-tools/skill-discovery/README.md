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

1. 通过 skill_search 找到目标，再选择固定版本。
2. skill_inspect 展示完整文件清单、各文件 SHA256、SKILL.md 正文和脚本预览；返回 manifest.sha256，即整个版本清单的摘要。
3. skill_install 用该 manifest.sha256 校验锁定版本，使用 SkillHub 下载 API 取 ZIP。每个 ZIP 路径都与清单比对，限制文件数量、体积、文件类型和哈希；安装阶段绝不执行代码。
4. 全部验证成功后写入临时目录，并原子移动到用户的 chat/skills/<slug>。没有覆盖操作。
5. load_skill 按需加载 SKILL.md；skill_read 读取引用文件；只有用户再次批准 skill_run 才调用本机解释器。运行前复核整包内容与锁文件一致。

SkillHub API 文档：https://github.com/Tencent/skillhub/blob/main/docs/api/README.md

## 当前支持

- 多文件 SKILL.md + scripts/ + references/ + assets/，及其他一般文件，包括二进制素材。
- 解释器格式：Python (.py)、Node.js (.js/.mjs/.cjs)、Shell (.sh)、PowerShell (.ps1)、Ruby (.rb)。
- 脚本参数直接传给解释器，不通过 shell 插值。进程环境清理后只透传基本系统路径；限制输出和运行时间。
- ZIP 可以包含单个顶层包装目录。完整包最多 100 个文件、解压后 12 MiB，单文件 4 MiB，ZIP 12 MiB，SKILL.md 100 KiB。
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
