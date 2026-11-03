> 译注：原文 npm 首页和包网页本次检查返回 HTTP 403；可查看 [npm 官方文档](https://docs.npmjs.com/)与 [该包的官方注册表信息](https://registry.npmjs.org/chrome-devtools-mcp)。原链接保留。

# 面向智能体的 Chrome DevTools

[![npm chrome-devtools-mcp package](https://img.shields.io/npm/v/chrome-devtools-mcp.svg)](https://npmjs.org/package/chrome-devtools-mcp)

面向智能体的 Chrome DevTools（`chrome-devtools-mcp`）允许编码智能体（如 Antigravity、Claude、Cursor 或 Copilot）控制并检查正在运行的 Chrome 浏览器。它可作为 Model Context Protocol（MCP）服务器或 [CLI][cli] 使用，两者都让 AI 编码助手利用 Chrome DevTools 的能力进行可靠的自动化、深入调试和性能分析。

[工具参考][tool-reference] | [CLI][cli] | [技能][skills] | [更新日志][changelog] | [贡献][contributing] | [故障排除][troubleshooting] | [设计原则][design-principles]

## 主要功能

- **获取性能洞察**：使用 [Chrome DevTools](https://github.com/ChromeDevTools/devtools-frontend) 记录性能跟踪并提取可操作的性能信息。
- **高级浏览器调试**：分析网络请求、截取屏幕并检查浏览器控制台消息（包括源代码映射后的堆栈跟踪）。
- **可靠的自动化**：使用 [puppeteer](https://github.com/puppeteer/puppeteer) 自动执行 Chrome 操作，并自动等待操作结果。

## 免责声明

`chrome-devtools-mcp` 会向 MCP 客户端暴露浏览器实例的内容，使其能够检查、调试和修改浏览器或 DevTools 中的任何数据。请避免共享不希望 MCP 客户端获取的敏感或个人信息。

`chrome-devtools-mcp` 官方仅支持 Google Chrome 和 [Chrome for Testing](https://developer.chrome.com/blog/chrome-for-testing/)。其他基于 Chromium 的浏览器可能可用，但不作保证，也可能出现意外行为，请自行决定是否使用。项目承诺为最新版 [Extended Stable Chrome](https://chromiumdash.appspot.com/schedule) 提供修复和支持。

性能工具可能将跟踪 URL 发送到 Google CrUX API，以获取真实用户体验数据。通过并列展示现场数据和实验室数据，可以更全面地了解性能。这些数据由 [Chrome 用户体验报告（CrUX）](https://developer.chrome.com/docs/crux)收集。要禁用此功能，请使用 `--no-performance-crux` 参数运行。

## **使用统计**

Google 收集使用统计信息（如工具调用成功率、延迟和环境信息），以改善 Chrome DevTools MCP 的可靠性和性能。

数据收集**默认启用**。启动服务器时传入 `--no-usage-statistics` 参数即可退出：

```json
"args": ["-y", "chrome-devtools-mcp@latest", "--no-usage-statistics"]
```

Google 根据 [Google 隐私政策](https://policies.google.com/privacy)处理这些数据。

Google 对 Chrome DevTools MCP 的使用统计收集独立于 Chrome 浏览器的使用统计。退出 Chrome 指标收集不会自动退出此工具的统计收集，反之亦然。

如果设置了 `CHROME_DEVTOOLS_MCP_NO_USAGE_STATISTICS` 或 `CI` 环境变量，则禁用收集。

## 更新检查

默认情况下，服务器会定期检查 npm 注册表中的更新，并在有新版本时记录通知。设置 `CHROME_DEVTOOLS_MCP_NO_UPDATE_CHECKS` 环境变量可以禁用这些检查。

## 要求

- [Node.js](https://nodejs.org/) [LTS](https://github.com/nodejs/Release#release-schedule) 版本。
- [Chrome](https://www.google.com/chrome/)当前稳定版本或更新版本。
- [npm](https://www.npmjs.com/)

## 开始使用

将以下配置添加到 MCP 客户端：

```json
{
  "mcpServers": {
    "chrome-devtools": {
      "command": "npx",
      "args": ["-y", "chrome-devtools-mcp@latest"]
    }
  }
}
```

> [!NOTE]
> 使用 `chrome-devtools-mcp@latest` 可确保 MCP 客户端始终使用最新版 Chrome DevTools MCP 服务器。

### MCP 客户端配置

有关特定编辑器或智能体（如 Antigravity、Claude Code、Cursor、VS Code）的配置说明，请参阅[客户端配置指南][client-configurations-guide]。

### 第一个提示词

在 MCP 客户端中输入以下提示词，检查是否一切正常：

```
Check the performance of https://developers.chrome.com
```

MCP 客户端应打开浏览器并记录性能跟踪。

> [!NOTE]
> 当 MCP 客户端使用需要运行中浏览器实例的工具时，MCP 服务器会自动启动浏览器。仅连接 Chrome DevTools MCP 服务器本身不会自动启动浏览器。

## CLI

该软件包还包含 `chrome-devtools` CLI，可在不使用 MCP 的情况下从终端控制浏览器：

```sh
npm i chrome-devtools-mcp@latest -g
chrome-devtools navigate_page 1 --url "https://developers.chrome.com"
chrome-devtools take_screenshot 1 --filePath screenshot.png
chrome-devtools stop
```

全部命令和选项请参阅 [CLI 指南][cli]。

## 技能

该软件包包含[技能](https://github.com/ChromeDevTools/chrome-devtools-mcp/tree/aade3be8f65d8afc941c97b498b9752ef562f181/skills)，用于指导智能体如何使用 Chrome DevTools：

| 技能                                                     | 用途                                       |
| --------------------------------------------------------- | ----------------------------------------------- |
| [`chrome-devtools`](https://github.com/ChromeDevTools/chrome-devtools-mcp/tree/aade3be8f65d8afc941c97b498b9752ef562f181/skills/chrome-devtools)             | 使用 MCP 工具调试和自动化页面。    |
| [`chrome-devtools-cli`](https://github.com/ChromeDevTools/chrome-devtools-mcp/tree/aade3be8f65d8afc941c97b498b9752ef562f181/skills/chrome-devtools-cli)     | 运行 CLI 并使用它编写 shell 脚本。    |
| [`a11y-debugging`](https://github.com/ChromeDevTools/chrome-devtools-mcp/tree/aade3be8f65d8afc941c97b498b9752ef562f181/skills/a11y-debugging)               | 审查并修复无障碍问题。             |
| [`cookie-debugging`](https://github.com/ChromeDevTools/chrome-devtools-mcp/tree/aade3be8f65d8afc941c97b498b9752ef562f181/skills/cookie-debugging)           | 调试 Cookie、会话与认证。    |
| [`debug-optimize-lcp`](https://github.com/ChromeDevTools/chrome-devtools-mcp/tree/aade3be8f65d8afc941c97b498b9752ef562f181/skills/debug-optimize-lcp)       | 调试和改进 Largest Contentful Paint。     |
| [`memory-leak-debugging`](https://github.com/ChromeDevTools/chrome-devtools-mcp/tree/aade3be8f65d8afc941c97b498b9752ef562f181/skills/memory-leak-debugging) | 查找并修复内存泄漏。                      |
| [`troubleshooting`](https://github.com/ChromeDevTools/chrome-devtools-mcp/tree/aade3be8f65d8afc941c97b498b9752ef562f181/skills/troubleshooting)             | 修复 MCP 与 Chrome 之间的连接问题。 |

### 将技能作为插件安装

插件会安装 MCP 服务器和全部技能，包括 CLI 技能：

- **Claude Code：** 运行 `/plugin marketplace add ChromeDevTools/chrome-devtools-mcp`，然后运行 `/plugin install chrome-devtools-mcp@chrome-devtools-plugins`.
- **Copilot / VS Code：** 运行 **Chat: Install Plugin From Source** 并输入 `ChromeDevTools/chrome-devtools-mcp`.
- **Gemini CLI：** 运行 `gemini extensions install --auto-update https://github.com/ChromeDevTools/chrome-devtools-mcp`.

更多信息请参阅[客户端配置指南][client-configurations-guide]。

### 手动安装技能

如果智能体不支持插件，请将技能文件夹复制到该智能体的技能目录。npm 软件包包含这些技能：

```sh
npm i chrome-devtools-mcp@latest -g
# Example for Claude Code. Use the skills directory of your agent.
mkdir -p ~/.claude/skills
cp -R "$(npm root -g)/chrome-devtools-mcp/skills/chrome-devtools-cli" ~/.claude/skills/
```

> [!IMPORTANT]
> `chrome-devtools-cli` 技能需要 CLI。使用技能前请全局安装该软件包。

### 保持技能更新

技能描述特定版本的工具。每次更新 MCP 服务器或 CLI 时都应更新技能；过时的技能可能指导智能体使用已更改的工具或参数。

- **插件：** 使用智能体插件管理器的更新命令。上面的 Gemini CLI 命令通过 `--auto-update` 启用自动更新。
- **手动安装：** 更新软件包，然后再次复制技能文件夹：

```sh
npm i chrome-devtools-mcp@latest -g
mkdir -p ~/.claude/skills
cp -R "$(npm root -g)/chrome-devtools-mcp/skills/chrome-devtools-cli" ~/.claude/skills/
```

## 工具

如果遇到问题，请查看[故障排除指南][troubleshooting]。
全部支持的 MCP 功能请参阅完整[工具参考][tool-reference]。

## 配置

服务器参数的完整列表（如 `--headless`、`--isolated`、`--slim`）及 WebSocket 连接配置方法，请参阅[配置指南][configuration-guide]。

如果只执行基本浏览器任务，`--slim` 参数会提供精简的工具集。请参阅[精简工具参考][slim-tool-reference]。

## 高级用法

并发会话、持久化用户数据目录、连接已运行的 Chrome 实例而非启动新实例，以及在 Android 上调试等高级功能，请参阅[高级用法指南][advanced-usage-guide]。

## 集成为浏览器子智能体

如果正在开发智能体工具，并希望将集成式浏览器子智能体作为产品的一部分，建议基于面向智能体的 Chrome DevTools 构建。

参考实现请参阅 [Gemini CLI 浏览器智能体文档](https://geminicli.com/docs/core/subagents/#browser-agent).

[advanced-usage-guide]: https://github.com/ChromeDevTools/chrome-devtools-mcp/blob/aade3be8f65d8afc941c97b498b9752ef562f181/docs/advanced-usage.md
[changelog]: https://github.com/ChromeDevTools/chrome-devtools-mcp/blob/aade3be8f65d8afc941c97b498b9752ef562f181/CHANGELOG.md
[cli]: https://github.com/ChromeDevTools/chrome-devtools-mcp/blob/aade3be8f65d8afc941c97b498b9752ef562f181/docs/cli.md
[client-configurations-guide]: https://github.com/ChromeDevTools/chrome-devtools-mcp/blob/aade3be8f65d8afc941c97b498b9752ef562f181/docs/client-configurations.md
[configuration-guide]: https://github.com/ChromeDevTools/chrome-devtools-mcp/blob/aade3be8f65d8afc941c97b498b9752ef562f181/docs/configuration.md
[contributing]: https://github.com/ChromeDevTools/chrome-devtools-mcp/blob/aade3be8f65d8afc941c97b498b9752ef562f181/CONTRIBUTING.md
[design-principles]: https://github.com/ChromeDevTools/chrome-devtools-mcp/blob/aade3be8f65d8afc941c97b498b9752ef562f181/docs/design-principles.md
[skills]: #技能
[slim-tool-reference]: https://github.com/ChromeDevTools/chrome-devtools-mcp/blob/aade3be8f65d8afc941c97b498b9752ef562f181/docs/slim-tool-reference.md
[tool-reference]: https://github.com/ChromeDevTools/chrome-devtools-mcp/blob/aade3be8f65d8afc941c97b498b9752ef562f181/docs/tool-reference.md
[troubleshooting]: https://github.com/ChromeDevTools/chrome-devtools-mcp/blob/aade3be8f65d8afc941c97b498b9752ef562f181/docs/troubleshooting.md
