# Claude Code 仓库规则

修改代码前请阅读根目录 `AGENTS.md`，并遵守其中的仓库级开发规范。

## GitHub Actions 手动触发强制规范

- **所有 GitHub Actions Workflow 必须只能手动触发**：在 `.github/workflows/*.yml` 和 `*.yaml` 中，`on` 只能配置 `workflow_dispatch`。
- **禁止任何自动触发**，包括 `push`、`pull_request`、`pull_request_target`、`schedule`、`release`、`workflow_run`、`repository_dispatch`、`workflow_call` 等。
- 新增或修改 Workflow 时必须移除其他触发器，不得因 CI、构建、测试、打包或部署擅自启用自动执行。
- 所有平台和子项目均适用；需要构建时由用户在 GitHub Actions 页面手动运行。

## npm 镜像源（沙箱里装依赖）

- `package-lock.json` 的下载地址固定为 `registry.npmmirror.com`，云端沙箱代理通常不放行，直接 `npm ci` 会卡住。
- 沙箱里用：`npm ci --replace-registry-host=always --registry=https://registry.npmjs.org/ --no-audit --no-fund`，**不要改动 `package-lock.json`**。
- 详细说明见 `AGENTS.md` 的“前端依赖安装（npm 镜像源）”。
