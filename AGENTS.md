# 开发导航

## GitHub Actions 手动触发强制规范

- **所有 GitHub Actions Workflow 必须只能手动触发**：在 `.github/workflows/*.yml` 和 `*.yaml` 中，`on` 只能配置 `workflow_dispatch`。
- **禁止所有自动触发**，包括但不限于 `push`、`pull_request`、`pull_request_target`、`schedule`、`release`、`workflow_run`、`repository_dispatch`、`workflow_call`。
- 新增或修改 Workflow 时，应删除其他触发器；禁止通过推送代码、标签、PR、定时事件或其他 Workflow 自动发起 CI、构建、测试、打包和部署。
- 本规范覆盖所有子项目与平台，任何 AI 编码助手均不得擅自添加自动触发 Workflow。需要构建时，由用户在 GitHub Actions 页面手动运行。
- **禁止新建自动触发的 Workflow**：包括测试、lint、CI、构建、部署、发布、定时任务等任何用途。即使只是「顺手加个 PR 检查」也不允许，PR 检查一律由用户手动触发。
- 新增 Workflow 必须是 `on: workflow_dispatch:` 单一触发器；若复用模板或从别处复制，必须先删掉全部 `push` / `pull_request` / `schedule` 等自动触发器再提交。
- 发现现有 Workflow 含自动触发器时，应作为缺陷修正为手动触发，不要以「保持原样」为由保留。

先阅读根目录 README.md，再阅读本次修改涉及的子项目 README.md，确认职责、实现状态、依赖边界和验证命令。

客户端允许重复实现，按任务决定是否复用。接口库保持协议兼容，业务策略归后端。目录迁移时同时检查构建、测试、部署和文档路径。

**云服务器部署规范**：云服务器性能较差、资源有限，编译耗时极长。凡遇到需要部署到云服务器上的服务时，**一律优先在本地编译/构建好产物**（如二进制执行文件、前端静态资源包或 Docker 镜像），再上传或同步至云服务器，严禁在远端直接执行大型编译任务。

---

# Git 提交与工具调用红线规范

为防止模型过度思考、频繁自省以及调用高开销工具导致额度被快速消耗，必须严格遵守以下规则：

### 1. 严格禁止滥用 GitHub MCP 工具
- **严禁调用任何 GitHub API / MCP 工具**（包括但不限于 `github.fetch_pr`、`github.fetch_workflow_job_logs`、`github.create_pull_request`、`github.merge_pull_request` 等）。
- **严禁将海量 Diff、PR 详情或远端 CI 构建日志载入对话上下文**，避免上下文膨胀导致巨额 Token 消耗。

### 2. 纯本地 Git 同步（禁止擅自提 PR 与监控 CI）
- 除非用户明确说“提 PR”或“创建 Pull Request”，否则“合并”、“推送”、“提交代码”一律视为**本地确定性操作**：
  - 本地合并或切换分支后，直接执行 `git commit` 和 `git push origin <branch>`。
- **严禁轮询或等待远程 GitHub Actions / CI 构建状态**，代码推送完成即立即结束任务并汇报。
- **严禁编写临时归档/解压打包验证脚本**（如 `git archive` 打包 tar 再解压测试），不做多余的患得患失式检验。

### 3. 按需思考与精准执行
- **轻量确定性操作（Git 提交、命令行执行、简单文件检索）**：
  - 必须最小化思考时间，直接执行命令，禁止多轮元规划（Meta-planning）和重复自检。
- **外科手术式修改**：
  - 只修改用户指定的文件和功能范围，严禁在提交前自作主张对全工程做全量编译、格式化或耗时测试。

### 4. 简单任务的默认执行范围
- 提交、推送、合并默认只做：检查当前分支与改动范围 → 提交 → 按需同步远端并推送 → 确认命令成功后立即汇报。
- 已通过的测试不因提交或推送而重复执行；不顺手修改文档、清理无关文件或扩大任务范围。
- 简单任务不启动多 agent；复杂开发仅在用户明确要求或适用仓库规则要求时使用。
- 只有合并冲突、权限问题或远端明确要求的检查阻挡操作时，才做针对性处理；不要主动展开 CI 排查。
- 工具输出只保留与当前决策有关的摘要，避免重复读取说明和输出大段日志。优先节省用户时间与额度。

---

# 界面约定

- 修改小伴桌面控制台（`client/xiaoban-desktop-wallpaper/src/ui`）里的菜单、弹窗、卡片前，先读 [`client/xiaoban-desktop-wallpaper/docs/design-specs/ui-conventions.md`](client/xiaoban-desktop-wallpaper/docs/design-specs/ui-conventions.md)：菜单一项一行、说明放悬停提示、优先复用 `xb-menu` 共享类、颜色走 `--ui-*` 主题变量。

---

# 大型设置与配置窗口规范

- **桌面版设置、模型/服务商配置以及后续新增的复杂编辑、审查、预览界面，默认使用原生独立窗口**；禁止用内嵌遮罩、`NModal` 或全局焦点锁替代。
- 保留系统标题栏，可移动、缩放、独立关闭，不阻塞主窗口。窗口外框必须适配当前显示器工作区并校正恢复位置；主题和保存结果跨窗口同步。
- 小型重命名、删除确认可保留内嵌；浏览器预览可降级。实现遵守 `client/xiaoban-desktop-wallpaper/docs/design-specs/ui-conventions.md` 的“独立窗口”章节，优先复用 `configuration_window` 基础设施。

---

# 前端依赖安装（npm 镜像源）

- 各前端子项目的 `package-lock.json` 里 `resolved` 地址固定为 `https://registry.npmmirror.com`（npmmirror 镜像）。**云端沙箱 / 受限网络的代理通常不放行 npmmirror**，直接 `npm ci` 会长时间卡住或失败（表现为 `connect_rejected`、`MaxListenersExceededWarning` 后无进展）。
- 这种环境里改走官方源安装，**不要修改或重新生成 `package-lock.json`**：

      npm ci --replace-registry-host=always --registry=https://registry.npmjs.org/ --no-audit --no-fund

  `--replace-registry-host=always` 只替换下载主机，`integrity` 校验仍然有效。
- 能访问 npmmirror 的本机环境照常使用 `npm ci`。
