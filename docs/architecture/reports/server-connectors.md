# S：server-connectors

## 3.1 已完成

- 专家／连接器按包结构分流，连接器复用内容寻址包存储、版本冲突规则和清单摘要；仅技能插件明确返回暂不支持。
- 校验 MCP 类型、非空名称版本、token schema 与 headers／占位符映射；hooks 与疑似明文密钥给警告，警告不含值。
- `metadata.connector` 和公开列表／详情的 `connector` 包含双语展示、示例、认证、字段摘要（只投影 server/key/label/type/required）、传输、技能与禁用工具计数。
- 图标复用 assets 内容寻址存储；SVG 用 XML 解析加被动绘图元素白名单，拒绝脚本、事件、foreignObject、外链、DTD／ENTITY、CSS 转义及活动元素。资源响应带 CSP 与 nosniff。
- 分类只选现有固定分类；迁移 0004 补固定“其他”，导入不会创建分类。网页卡片／详情显示连接器信息，后台连接器可上传插件包。

## 保守解释与限制

- 未声明 auth 的远端 MCP 展示为 OAuth（自动发现），不声称已确认需授权；静态包无法区分免认证与运行时 401。stdio 的 preAuth=cli 归 env，原始 MCP 模板不放入公开摘要。
- SVG 采取拒绝而非删除危险节点；除文档要求外也拒绝 style 元素、动画和 image 等活动／嵌入内容。安全的本地 # 引用可用；部分复杂 SVG 需作者转成 PNG。
- “禁止真实密钥”按约定实现为警告，包原样保存；管理员必须在发布前处理警告。
- 版本沿用专家规则：同版本不同内容返回 409，必须上游提高版本号，不擅自生成版本。

## 验证（3.1）

- `cargo test --lib --offline --manifest-path services/server-xiaoban-market/Cargo.toml`：37 项通过，包括本机 WorkBuddy 真实连接器 oracle（只读，目录不存在则跳过）。
- `cargo test --test api --no-run --offline --manifest-path services/server-xiaoban-market/Cargo.toml`：API 测试编译通过；新增上传、重复／冲突版本、公开展示、SVG 响应与恶意 SVG、下载用例。
- `npm.cmd --prefix services/server-xiaoban-market/web test -- --no-cache`：22 项通过，包含卡片、详情和后台连接器上传。web 下 `node_modules/.bin/vue-tsc.cmd --noEmit` 通过。共享 node_modules 指向主工作区，默认 Vitest 缓存无写权限，故关闭缓存。生产 build 的 vue-tsc 通过，但 Vite 写入共享 node_modules/.vite-temp 被沙箱拒绝；本任务要求的类型检查独立通过。

## 需要 Claude 接手

- 无本地 PostgreSQL，端到端仅写好并编译；合并后在有数据库环境运行 tests/api.rs（包括既有专家回归）。部署时运行迁移 0004。
- 桌面 Rust／连接器运行时、深链接实际安装与授权联调归 Claude／L 包；本工作包没有修改或构建 src-tauri。

## 3.2

尚未开始：用户要求先提交 3.1，再开展 3.2；第一笔提交被环境权限阻挡。

## 提交阻挡

- 当前分支 `codex/server-connectors`，所有改动仍在当前 worktree，未暂存、未提交、未推送，没有切换或修改其他分支。
- `git add services/server-xiaoban-market docs/architecture/reports/server-connectors.md` 和 `git commit` 都报：`Unable to create 'C:/Users/sdjz0/Documents/Claude/XiangBan/.git/worktrees/server/index.lock': Permission denied`。
- 确认 `.git` 指向上述 worktree 元数据目录，目录 ACL 存在显式写入 Deny，且不存在残留 index.lock。当前会话禁止提权，无法完成提交；没有修改 ACL 或绕开沙箱。
- 需要在允许写入该 Git 元数据目录的会话中先提交当前 3.1 改动，再继续 3.2（草稿／来源／署名／许可接口及筛选、确定性 ZIP 批量导入二进制与测试），最后第二次提交。本工作包整体尚未达到交付标准。

## 3.2 官方市场导入工具（2026-10-25）

本节是第二阶段的最新汇报；上面的“尚未开始／提交阻挡”保留为上次会话记录。用户已告知 Claude 提交了 3.1，本次基于该提交继续，未重做 3.1，未执行 `git add`、`git commit` 或推送。

### 已实现

- 新增二进制 `import-marketplace`（`src/bin/import_marketplace.rs`），打包、扫描、HTTP 和汇总逻辑在 `src/marketplace.rs`，可由库单元测试直接覆盖。支持 `--dir`、`--server`、`--token`、`--dry-run`、逗号分隔的 `--only`；保留服务原来的默认 `cargo run` 入口。
- 优先按市场根目录 `.codebuddy-plugin/marketplace.json` 的 `plugins[].source` 找插件，同时支持根目录 `marketplace.json`、`connectors/<name>/` 和直接传入 connectors 目录。专家和连接器复用 3.1 校验，仅技能插件计为“跳过（仅技能插件）”；单项失败继续处理，汇总新增／更新／跳过／失败，dry-run 另列校验数。有失败时退出码为 1，参数错误为 2。
- ZIP 按文件路径排序，固定时间戳、压缩方式和权限；不写出 ZIP 文件。排除 WorkBuddy 本机下载时间标记 `.downloaded_at`，避免它改变重复导入的内容摘要。限制文件数量、文件及总大小、目录深度，拒绝 source 越界、符号链接和 Windows 重解析点，包内路径仍用既有校验。
- HTTP 上传固定发送 `status=draft`、`origin=workbuddy-official`、`source=official`、`lenient=true` 和许可／署名。服务端对 `workbuddy-official` 来源强制草稿，即使请求传 published，或重导已上架的条目，也回到草稿。其他来源未传 status 时保持原有行为。
- 读取插件清单的 `license`、`author`、`homepage`、`repository`，优先于市场条目信息；缺少许可时使用市场条目的 license，再没有则写 `unknown`。署名按 author／homepage／repository 保存到现有 `items.metadata.attribution` JSONB，缺失字段为 null；origin 和 license 使用现有列，无需新增迁移。
- 接口返回 `skipped`，相同版本和包摘要不新增版本；同名新版本写版本并更新 latest_version；同版本不同内容仍返回 409，事务回滚条目和来源信息的修改，不自动改版本号。
- 后台列表返回来源、许可和署名；页面增加许可列、来源与“许可未知”组合筛选，并显示原作者／主页／仓库和逐个审核提示。查询示例：`GET /api/v1/admin/items?origin=workbuddy-official&license=unknown`。没有新增批量发布入口。
- 令牌不写日志、错误或 dry-run 输出。参数错误不回显实参，所有输出按令牌值脱敏；HTTP 失败只报告状态码或固定错误，不回显服务端响应体。HTTP 禁止重定向，远程服务要求 HTTPS，本机假服务允许 HTTP。

使用示例（在本服务目录运行；server 填服务根地址，不带 `/api/v1`）：

```powershell
cargo run --offline --bin import-marketplace -- --dir <市场目录> --server https://<市场服务> --token <管理员令牌> --dry-run
# 审核 dry-run 结果后去掉 --dry-run；可加 --only name-one,name-two 选择条目。
```

### 验证结果

- `cargo test --lib --offline --manifest-path services/server-xiaoban-market/Cargo.toml`：43 项通过。新增 6 项测试使用临时小市场，覆盖目录布局、专家打包、确定性 ZIP、清单字段优先级与 unknown 回退、越界／无效包、dry-run／only／仅技能跳过、令牌脱敏和 URL 校验；本机 axum 假服务验证真实 multipart 请求及新增／更新／相同内容跳过／HTTP 冲突／无效响应的汇总与继续处理。临时目录测试结束后删除；既有真实连接器 oracle 也通过。
- `cargo test --test api --no-run --offline --manifest-path services/server-xiaoban-market/Cargo.toml`：编译通过，也构建了导入二进制。`tests/api.rs` 新增专家／连接器的草稿强制、来源／署名／许可落库、插件优先与市场许可回退、重复导入、版本升级、冲突回滚、公开目录不可见和后台组合筛选用例。
- `npm.cmd --prefix services/server-xiaoban-market/web test -- --no-cache`：23 项通过。新增页面测试验证来源／许可展示、组合筛选参数、原作者展示和没有批量发布入口。关闭缓存以适应共享 node_modules 的写权限。
- web 目录 `node_modules/.bin/vue-tsc.cmd --noEmit`：通过。
- 用编译出的二进制对两个本机市场做了只读 dry-run（未上传、未写 ZIP、未向样本目录写入任何内容，也未将样本文件复制进仓库）：专家市场 2 个通过、0 失败；官方连接器市场 303 个通过、57 个仅技能跳过、1 个失败，总数 361。

### 未完成的真实环境验证与已知限制

- 本机没有 PostgreSQL，未运行真实服务端端到端测试，也未实际向市场服务批量导入或部署；Claude 需在有数据库的环境运行 `tests/api.rs`。HTTP 上传和汇总已由不依赖数据库的本机假服务测试。
- 实测 361 个条目中，`tencent-map` 既没有 mcp.json／.mcp.json，也没有 skills/，仅有 `extensions["ai.workbuddy"].urlTemplatedMcpServers`。这不是仅技能插件，沿用 3.1 的类型判断明确计为失败，未将它误报成“仅技能跳过”，也未在本阶段扩展 3.1 的 MCP 格式。其余无 MCP 的 57 个确实按仅技能插件跳过；因此全量连接器 dry-run 返回退出码 1。后续支持该额外声明格式后可单独重试。
- “跳过（内容相同）”表示没有新增版本；来源／署名／许可仍会按本次导入刷新，且官方条目保持草稿。重新导入已发布条目会使其回到草稿，需要管理员再次逐个发布。
- 本次改动全部留在当前工作区，待 Claude 提交。
