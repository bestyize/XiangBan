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
