# Agent Tools / Skill Discovery

小伴桌面通过腾讯 SkillHub Open API 查找、审查、安装 Skill。桌面 Agent 默认注册全部工具。

| 工具 | 用途 | 授权 |
| --- | --- | --- |
| skill_search(keyword) | 关键词检索 | Network，不可信结果 |
| skill_inspect(slug, version, namespace?) | 返回版本锁定的摘要、SHA256、文件/脚本数量及截断标记 | Network，不可信结果 |
| skill_review_read(slug, version, sha256, section?, path?, offset?) | 按摘要固定身份和版本，分页读取完整清单、正文或脚本源码 | Network，不可信结果 |
| skill_install(slug, version, sha256) | 安装已审查的完整多文件包 | 每次需要用户批准 |
| skill_read(skill, path, offset?, limit?) | 分页阅读已安装、整包校验后的文本/引用文件 | Read，不可信结果 |
| skill_run(skill, script, args?) | 运行已安装并校验后的脚本 | 默认权限需确认；完全信任由宿主放行 |

## 生命周期

1. 通过 skill_search 找到目标，再选择固定版本。若未指定发布者，先从详情解析 namespace，再在文件清单、原文和下载请求中固定使用该发布者，校验清单返回的发布者身份。
2. 模型工具 skill_inspect 返回完整可解析的有界 JSON 摘要（`reviewStatus: summary_only`），不把整份长正文和脚本堆进输出后再依赖通用截断。使用 skill_review_read 的 `section: manifest` 分页读清单，`section: file` 分页读 SKILL.md 或清单内的源码；用 `nextOffset` 续读直到 `hasMore: false`。正文页最多 2048 字节，保证 UTF-8 边界；每页重新校验发布者、版本、整包摘要及文件哈希。摘要不代表完成源码审查。原生 UI 的 inspect / inspect_coordinate 继续返回完整审查和脚本预览。
3. skill_install 用返回的 sha256 校验锁定版本，使用 SkillHub 下载 API 取 ZIP。每个 ZIP 路径都与清单比对，限制文件数量、体积、文件类型和哈希。若上游 ZIP 与清单不一致，改从同一发布者、同一版本的文件接口下载，最多并发 4 个请求、首个错误停止其余请求，仍须全部满足原清单的大小与 SHA256；安装阶段绝不执行代码。
4. 全部验证成功后写入临时目录，并原子移动到用户的 chat/skills/<slug>。没有覆盖操作。
5. load_skill 按需加载 SKILL.md；skill_read 读取引用文件；skill_run 按宿主当前权限模式调用本机解释器。运行前复核整包内容与锁文件一致。

搜索、审查、安装分别有 45 / 180 / 300 秒的内部总预算，外层工具另留 5 秒清理时间；单次 HTTP 请求 30 秒、连接 8 秒。请求头和响应体等待均可响应取消，下载结束后及每个暂存文件写入、发布前再次检查取消状态。只在全包校验成功后原子发布，取消的暂存目录会清理。相同发布者、版本、摘要且文件仍完整的重复安装可以确认已有结果；不同来源、版本或被修改的目录仍拒绝覆盖。

桌面端的普通 `read_file` 也可按绝对路径分页读取已安装技能的清单文件：宿主使用 `authorize_installed_file` 逐次核对目录边界、安装清单和整包哈希。未列入清单、被修改或逃逸技能目录的文件仍会被拒绝；这不会开放整个用户数据目录或写权限。

SkillHub API 文档：https://github.com/Tencent/skillhub/blob/main/docs/api/README.md

## 当前支持

- 多文件 SKILL.md + scripts/ + references/ + assets/，及其他一般文件，包括二进制素材。
- 解释器格式：Python (.py)、Node.js (.js/.mjs/.cjs)、Shell (.sh)、PowerShell (.ps1)、Ruby (.rb)。
- 脚本参数直接传给解释器，不通过 shell 插值。进程环境清理后只透传基本系统路径；限制输出和运行时间。
- ZIP 可以包含单个顶层包装目录。完整包最多 512 个文件、解压后 12 MiB，单文件 4 MiB，ZIP 12 MiB，SKILL.md 100 KiB。读取文件清单时核对 SkillHub 返回的 `count` 和实际文件数；空清单、数量不一致和超过上限会明确报错，不会安装不完整的包。
- 拒绝路径穿越、绝对路径、符号链接、Windows 保留路径、大小写重复路径、哈希不匹配和未经 API 文件清单列出的文件。
- 小伴的「完全信任」下，已安装且校验通过的脚本可直接运行；新安装或更新仍须确认来源和摘要。其他宿主可继续用 always_ask 保持逐次确认。

## 不保证的能力

**不是沙箱。** 脚本作为用户账户下的本地进程运行，可能访问宿主系统的文件和网络。SHA256 保障内容与审查清单一致，不代表内容安全。用户应亲自评估第三方脚本和依赖。

解释器依赖本机已经安装；不自动安装 Python/Node、pip/npm 包或额外的操作系统依赖。不支持需要独立后端、容器或复杂环境的脚本一键运行。

当前搜索与安装只连接 SkillHub；尚未支持 skills.sh、GitHub 直接导入、版本自动升级、可信签名验签和真正隔离的沙箱。文件大小超限或未经支持的归档明确拒绝。

## 验证

在可运行 Rust 的环境中分别执行：

    cargo test --manifest-path libs/agent-tools/skill-discovery/Cargo.toml
    cargo test --manifest-path client/xiaoban-desktop/src-tauri/Cargo.toml --locked

## 技能目录检索

AI 商店的技能页直接调用 SkillHub 公开的 `/api/skills`，使用独立的 `keyword`、`category`、`source`、`sortBy`、`page` 参数。可选分类由 `/api/v1/categories` 动态加载，中文显示名称不会拼入关键词，也不会写死分类 key。界面显示服务端的匹配总数与分页，提供综合评分、下载量、收藏量、安装量、更新时间排序及官方/社区来源筛选。分类接口失败时仍允许按「全部」搜索，可重试分类字典。

Agent 的 `skill_search` 单次最多返回 20 条，含 `total`/`hasMore`，接受 `page`（1–5）与 `sortBy`；策略仍限制每轮任务至多 3 次不同的搜索或页面，不允许重复请求相同关键词+页码。这是检索改进，不自动安装、执行第三方技能，也不声称覆盖 WorkBuddy 私有市场或其他聚合站。

## 主动发现策略

用户明确请求时，可以直接搜索 Skill。否则模型必须给出缺少的能力和检查过的现有工具/技能，再使用 `skill_search` 的 `reason=capability_gap` 参数。对于 `reason=repeated_failure`，同一个工具必须在本轮发生至少两次不同调用的实际失败；权限、网络、配额、参数错误不属于能力缺口。

策略还包括：单任务最多三个不同搜索词、相同词不重复搜索、离线要求下禁止在线检索。模型在重复失败后宣布无法完成时，最多收到一次重新考虑 Skill 的提示。发现候选不等于批准安装；安装与脚本运行依然需要用户分别确认。

当前策略仅对单轮任务生效；没有后台搜索，也没有跨会话长期能力需求学习。


## 专家包与 MCP 目录

AI 商店专家页读取 `/api/v1/skillsets`，连接器页读取 `/api/v1/mcp/servers`。这两组接口已对照 SkillHub 网站实现及线上响应验证，目前尚未列入公开 Open API 总览；返回结构变化会显示错误，不回退到示例卡片。

专家包审查固定工作流正文、依赖发布者命名空间、版本、文件清单与脚本预览，生成整包摘要。安装时再次审查，摘要不一致则拒绝；所有依赖验证完成后发布工作流，失败回滚本次新增目录，不覆盖已有不同来源或版本。移除工作流保留共享依赖。依赖缺失或内容改变时标记未就绪。

部分历史技能没有 YAML frontmatter，另有技能的包内 name 与市场 slug 不同（包括中文名称）。注册中心发布者 + slug 决定安装身份，别名兼容通过锁文件 `metadata.registryIdentity` 显式标记；SkillHub 的元数据绑定同样纳入审查摘要，原始 SKILL.md 字节及文件 SHA256 保持不变。加载层核对锁文件与原文哈希，以市场 slug 加载，同时保留原文的描述、展示名和 user/model invocation 限制。损坏 YAML、缺失描述和错误哈希仍拒绝，普通手写技能仍要求名称与目录一致。安装、读取和运行统一支持最多 512 个文件；较大的文本引用通过 skill_read 分页读取，不再误用 100 文件/100 KiB 的旧限制。

MCP 目录读取详情与 README，并在平台提供摘要时校验 README SHA256；JSON 配置模板只用于展示，用户填写凭据并确认后才交给连接器管理器启动。WorkBuddy 专用注册脚本或路径不会自动在小伴执行。

只读线上验证（可选专家包标识）：

    cargo run --manifest-path libs/agent-tools/skill-discovery/Cargo.toml --example catalog-smoke -- tech-test-automation

## 小伴市场

`market` 模块对接自建的小伴市场（`services/server-xiaoban-market`），用于专家、技能、连接器的浏览，以及技能 / 专家的审查、安装、更新。

- **版本锁定**：版本以清单摘要锁定，算法为对按路径排序的每行 `"<文件 sha256>  <路径>\n"` 求 SHA-256（与服务端相同，两边的测试断言同一个固定向量）。客户端从文件清单重新计算摘要，与市场记录和用户审查时看到的摘要逐一比对；下载的 ZIP 还要核对包哈希，并按清单逐文件校验大小与哈希。任何一项不一致都拒绝安装。
- **专家**与技能一样是带 `SKILL.md` 的安装包，装进同一个技能目录；锁文件记录 `origin: {service: "xiaoban-market", kind}`，`provider` 仍为 `skillhub`，所以现有的加载、卸载逻辑照常工作。
- **专家团 / 技能套件**：先逐个校验全部成员，再统一写入；任何一个失败，撤销本次新增的成员；已按相同版本和摘要安装的成员跳过。整组摘要由各成员的坐标和摘要计算。
- **更新**：只替换由市场安装的包；新版本完整校验后才替换，旧目录移到 `chat/skill-backups/` 作为备份，替换失败时自动还原。更新只提示，不自动执行。
- **网页链接**：`parse_link` 严格解析 `xiaoban://market/{open|install|connect}/{skills|experts|connectors}/{namespace}/{slug}?version=&sha256=`，拒绝多余的路径、端口、用户信息和非法参数；链接只能请求展示，审查与确认仍由用户完成。
- 市场地址必须是 `https` 站点根地址（本机回环可用 `http`，仅用于开发和测试），不跟随重定向。

验证：`cargo test --manifest-path libs/agent-tools/skill-discovery/Cargo.toml market`（用进程内的模拟市场覆盖浏览、审查、安装、被篡改的清单、更新备份、整组回滚）。
