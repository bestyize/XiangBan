# Claude Code 仓库规则

修改代码前请阅读根目录 `AGENTS.md`，并遵守其中的仓库级开发规范。

## GitHub Actions 手动触发强制规范

- **所有 GitHub Actions Workflow 必须只能手动触发**：在 `.github/workflows/*.yml` 和 `*.yaml` 中，`on` 只能配置 `workflow_dispatch`。
- **禁止任何自动触发**，包括 `push`、`pull_request`、`pull_request_target`、`schedule`、`release`、`workflow_run`、`repository_dispatch`、`workflow_call` 等。
- 新增或修改 Workflow 时必须移除其他触发器，不得因 CI、构建、测试、打包或部署擅自启用自动执行。
- 所有平台和子项目均适用；需要构建时由用户在 GitHub Actions 页面手动运行。
