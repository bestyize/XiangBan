## Playwright MCP

一个使用 [Playwright](https://playwright.dev) 提供浏览器自动化能力的模型上下文协议（MCP）服务器。该服务器让大语言模型通过结构化无障碍快照与网页交互，无需截图或针对视觉调整的模型。

### Playwright MCP 与 Playwright CLI

该软件包为 Playwright 提供 MCP 接口。如果你使用的是**编程代理**，也许改用 [CLI+SKILLS](https://github.com/microsoft/playwright-cli) 会更合适。

- **CLI**：现代**编程代理**越来越倾向于使用以 SKILL 形式提供的 CLI 工作流，而非 MCP，因为 CLI 调用更节省令牌：它避免将大型工具模式和冗长的无障碍树载入模型上下文，让代理通过简洁的专用命令操作。因此，对于需要在有限上下文窗口中平衡浏览器自动化、大型代码库、测试和推理的高吞吐量编程代理，CLI + SKILLs 更合适。<br>**进一步了解 [带 SKILLS 的 Playwright CLI](https://github.com/microsoft/playwright-cli)**。

- **MCP**：MCP 对依赖持久状态、丰富检查能力和对页面结构进行迭代推理的专门代理循环仍有价值，例如探索性自动化、自愈测试或长期运行的自主工作流；在这些场景中，保持连续浏览器上下文的价值高于令牌成本。

### 主要功能

- **快速轻量**。使用 Playwright 的无障碍树，而非基于像素的输入。
- **适合大语言模型**。无需视觉模型，完全基于结构化数据运行。
- **确定性的工具调用**。避免截图方法常见的歧义。

### 要求
- Node.js 18 或更新版本
- VS Code、Cursor、Windsurf、Claude Desktop、Goose、Grok、Junie 或其他 MCP 客户端

<!--
// Generate using:
node utils/generate-links.js
-->

### 入门

首先，使用你的客户端安装 Playwright MCP 服务器。

**标准配置**适用于大多数工具：

```js
{
  "mcpServers": {
    "playwright": {
      "command": "npx",
      "args": [
        "@playwright/mcp@latest"
      ]
    }
  }
}
```

[<img src="https://img.shields.io/badge/VS_Code-VS_Code?style=flat-square&label=Install%20Server&color=0098FF" alt="Install in VS Code">](https://insiders.vscode.dev/redirect?url=vscode%3Amcp%2Finstall%3F%257B%2522name%2522%253A%2522playwright%2522%252C%2522command%2522%253A%2522npx%2522%252C%2522args%2522%253A%255B%2522%2540playwright%252Fmcp%2540latest%2522%255D%257D) [<img alt="Install in VS Code Insiders" src="https://img.shields.io/badge/VS_Code_Insiders-VS_Code_Insiders?style=flat-square&label=Install%20Server&color=24bfa5">](https://insiders.vscode.dev/redirect?url=vscode-insiders%3Amcp%2Finstall%3F%257B%2522name%2522%253A%2522playwright%2522%252C%2522command%2522%253A%2522npx%2522%252C%2522args%2522%253A%255B%2522%2540playwright%252Fmcp%2540latest%2522%255D%257D)

<details>
<summary>Amp</summary>

通过 Amp 的 VS Code 扩展设置界面添加，或更新 settings.json 文件：

```json
"amp.mcpServers": {
  "playwright": {
    "command": "npx",
    "args": [
      "@playwright/mcp@latest"
    ]
  }
}
```

**Amp CLI 设置：**

通过下方 `amp mcp add` 命令添加

```bash
amp mcp add playwright -- npx @playwright/mcp@latest
```

</details>

<details>
<summary>Antigravity</summary>

通过 Antigravity 设置添加，或更新配置文件：

```json
{
  "mcpServers": {
    "playwright": {
      "command": "npx",
      "args": [
        "@playwright/mcp@latest"
      ]
    }
  }
}
```

</details>

<details>
<summary>Claude Code</summary>

使用 Claude Code CLI 添加 Playwright MCP 服务器：

```bash
claude mcp add playwright npx @playwright/mcp@latest
```
</details>

<details>
<summary>Claude Desktop</summary>

按照 MCP 安装[指南](https://modelcontextprotocol.io/quickstart/user)，使用上面的标准配置。

</details>

<details>
<summary>Cline</summary>

按照[配置 MCP 服务器](https://docs.cline.bot/mcp/configuring-mcp-servers)章节中的说明操作。

**示例：本地设置**

将以下内容添加到你的 [`cline_mcp_settings.json`](https://docs.cline.bot/mcp/configuring-mcp-servers#editing-mcp-settings-files) 文件：

```json
{
  "mcpServers": {
    "playwright": {
      "type": "stdio",
      "command": "npx",
      "timeout": 30,
      "args": [
        "-y",
        "@playwright/mcp@latest"
      ],
      "disabled": false
    }
  }
}
```

</details>

<details>
<summary>Codex</summary>

使用 Codex CLI 添加 Playwright MCP 服务器：

```bash
codex mcp add playwright npx "@playwright/mcp@latest"
```

也可以创建或编辑配置文件 `~/.codex/config.toml`，并添加：

```toml
[mcp_servers.playwright]
command = "npx"
args = ["@playwright/mcp@latest"]
```

更多信息参见 [Codex MCP 文档](https://github.com/openai/codex/blob/main/codex-rs/config.md#mcp_servers)。

</details>

<details>
<summary>Copilot</summary>

使用 Copilot CLI 以交互方式添加 Playwright MCP 服务器：

```bash
/mcp add
```

也可以创建或编辑配置文件 `~/.copilot/mcp-config.json`，并添加：

```json
{
  "mcpServers": {
    "playwright": {
      "type": "local",
      "command": "npx",
      "tools": [
        "*"
      ],
      "args": [
        "@playwright/mcp@latest"
      ]
    }
  }
}
```

更多信息参见 [Copilot CLI 文档](https://docs.github.com/en/copilot/concepts/agents/about-copilot-cli)。

</details>

<details>
<summary>Cursor</summary>

#### 点击按钮安装：

[<img src="https://cursor.com/deeplink/mcp-install-dark.svg" alt="Install in Cursor">](https://cursor.com/en/install-mcp?name=Playwright&config=eyJjb21tYW5kIjoibnB4IEBwbGF5d3JpZ2h0L21jcEBsYXRlc3QifQ%3D%3D)

#### 或手动安装：

进入 `Cursor Settings` -> `MCP` -> `Add new MCP Server`。名称可自行选择，使用 `command` 类型并将命令设为 `npx @playwright/mcp@latest`。也可以点击 `Edit` 验证配置或添加命令等参数。

</details>

<details>
<summary>Factory</summary>

使用 Factory CLI 添加 Playwright MCP 服务器：

```bash
droid mcp add playwright "npx @playwright/mcp@latest"
```

也可以在 Factory droid 中输入 `/mcp`，打开用于管理 MCP 服务器的交互式界面。

更多信息参见 [Factory MCP 文档](https://docs.factory.ai/cli/configuration/mcp)。

</details>

<details>
<summary>Gemini CLI</summary>

按照 MCP 安装[指南](https://github.com/google-gemini/gemini-cli/blob/main/docs/tools/mcp-server.md#configure-the-mcp-server-in-settingsjson)，使用上面的标准配置。

</details>

<details>
<summary>Goose</summary>

#### 点击按钮安装：

[![Install in Goose](https://block.github.io/goose/img/extension-install-dark.svg)](https://block.github.io/goose/extension?cmd=npx&arg=%40playwright%2Fmcp%40latest&id=playwright&name=Playwright&description=Interact%20with%20web%20pages%20through%20structured%20accessibility%20snapshots%20using%20Playwright)

#### 或手动安装：

进入 `Advanced settings` -> `Extensions` -> `Add custom extension`。名称可自行选择，类型使用 `STDIO`，并将 `command` 设置为 `npx @playwright/mcp`。点击 “Add Extension”。
</details>

<details>
<summary>Grok</summary>

使用 Grok CLI 添加 Playwright MCP 服务器：

```bash
grok mcp add playwright -- npx @playwright/mcp@latest
```

也可以创建或编辑配置文件 `~/.grok/config.toml`，并添加：

```toml
[mcp_servers.playwright]
command = "npx"
args = ["@playwright/mcp@latest"]
```

更多信息参见 [Grok MCP 文档](https://docs.x.ai/build/features/mcp-servers)。

</details>

<details>
<summary>Junie</summary>

在 Junie CLI 中添加 Playwright MCP 服务器：

1. 输入 `/mcp`
2. 按 `Ctrl+A` 添加新的 MCP 服务器
3. 从列表选择 **Playwright**

也可以添加到 `.junie/mcp/mcp.json`：

```json
{
  "mcpServers": {
    "Playwright": {
      "command": "npx",
      "args": [
        "-y",
        "@playwright/mcp@latest"
      ]
    }
  }
}
```

更多信息参见 [Junie MCP 配置文档](https://junie.jetbrains.com/docs/junie-cli-mcp-configuration.html)。

</details>

<details>
<summary>Kiro</summary>

[![Add to Kiro](https://kiro.dev/images/add-to-kiro.svg)](https://kiro.dev/launch/mcp/add?name=playwright&config=%7B%22command%22%3A%22npx%22%2C%22args%22%3A%5B%22%40playwright%2Fmcp%40latest%22%5D%7D)

按照 MCP 服务器[文档](https://kiro.dev/docs/mcp/)操作。例如在 `.kiro/settings/mcp.json` 中：

```json
{
  "mcpServers": {
    "playwright": {
      "command": "npx",
      "args": [
        "@playwright/mcp@latest"
      ]
    }
  }
}
```
</details>

<details>
<summary>LM Studio</summary>

#### 点击按钮安装：

[![Add MCP Server playwright to LM Studio](https://files.lmstudio.ai/deeplink/mcp-install-light.svg)](https://lmstudio.ai/install-mcp?name=playwright&config=eyJjb21tYW5kIjoibnB4IiwiYXJncyI6WyJAcGxheXdyaWdodC9tY3BAbGF0ZXN0Il19)

#### 或手动安装：

在右侧边栏进入 `Program` -> `Install` -> `Edit mcp.json`。使用上面的标准配置。
</details>

<details>
<summary>opencode</summary>

按照 MCP 服务器[文档](https://opencode.ai/docs/mcp-servers/)操作。例如在 `~/.config/opencode/opencode.json` 中：

```json
{
  "$schema": "https://opencode.ai/config.json",
  "mcp": {
    "playwright": {
      "type": "local",
      "command": [
        "npx",
        "@playwright/mcp@latest"
      ],
      "enabled": true
    }
  }
}

```
</details>

<details>
<summary>Qodo Gen</summary>

打开 VSCode 或 IntelliJ 中的 [Qodo Gen](https://docs.qodo.ai/qodo-documentation/qodo-gen) 聊天面板 → Connect more tools → + Add new MCP → 粘贴上面的标准配置。

点击 <code>Save</code>。
</details>

<details>
<summary>VS Code</summary>

#### 点击按钮安装：

[<img src="https://img.shields.io/badge/VS_Code-VS_Code?style=flat-square&label=Install%20Server&color=0098FF" alt="Install in VS Code">](https://insiders.vscode.dev/redirect?url=vscode%3Amcp%2Finstall%3F%257B%2522name%2522%253A%2522playwright%2522%252C%2522command%2522%253A%2522npx%2522%252C%2522args%2522%253A%255B%2522%2540playwright%252Fmcp%2540latest%2522%255D%257D) [<img alt="Install in VS Code Insiders" src="https://img.shields.io/badge/VS_Code_Insiders-VS_Code_Insiders?style=flat-square&label=Install%20Server&color=24bfa5">](https://insiders.vscode.dev/redirect?url=vscode-insiders%3Amcp%2Finstall%3F%257B%2522name%2522%253A%2522playwright%2522%252C%2522command%2522%253A%2522npx%2522%252C%2522args%2522%253A%255B%2522%2540playwright%252Fmcp%2540latest%2522%255D%257D)

#### 或手动安装：

按照 MCP 安装[指南](https://code.visualstudio.com/docs/copilot/chat/mcp-servers#_add-an-mcp-server)，使用上面的标准配置。也可以使用 VS Code CLI 安装 Playwright MCP 服务器：

```bash
# For VS Code
code --add-mcp '{"name":"playwright","command":"npx","args":["@playwright/mcp@latest"]}'
```

安装后，可在 VS Code 中通过 GitHub Copilot 代理使用 Playwright MCP 服务器。
</details>

<details>
<summary>Warp</summary>

进入 `Settings` -> `AI` -> `Manage MCP Servers` -> `+ Add`，[添加 MCP 服务器](https://docs.warp.dev/knowledge-and-collaboration/mcp#adding-an-mcp-server)。使用上面的标准配置。

也可以在 Warp 提示符中使用斜杠命令 `/add-mcp`，并粘贴上面的标准配置：
```js
{
  "mcpServers": {
    "playwright": {
      "command": "npx",
      "args": [
        "@playwright/mcp@latest"
      ]
    }
  }
}
```

</details>

<details>
<summary>Windsurf</summary>

按照 Windsurf MCP [文档](https://docs.windsurf.com/windsurf/cascade/mcp)操作。使用上面的标准配置。

</details>

### 配置

Playwright MCP 服务器支持以下参数，可作为 `"args"` 列表的一部分放入上面的 JSON 配置中：

<!--- Options generated by update-readme.js -->

| 选项 | 说明 |
|--------|-------------|
| --allowed-hosts <hosts...> | 此服务器允许服务的主机列表，以逗号分隔。默认为 localhost、环回 IP 和通过 --host 传入的主机；其他主机必须明确列出。传入 `*` 可禁用主机检查。<br>*env* `PLAYWRIGHT_MCP_ALLOWED_HOSTS`  |
| --allowed-origins <origins> | 允许浏览器请求的可信源列表，以分号分隔。默认允许所有源。重要：它*不是*安全边界，也*不*影响重定向。<br>*env* `PLAYWRIGHT_MCP_ALLOWED_ORIGINS`  |
| --allow-unrestricted-file-access | 允许访问工作区根目录之外的文件，也允许不受限制地访问 file:// URL。默认情况下，文件系统访问仅限于工作区根目录（未配置根目录时为当前工作目录），且禁止导航至 file:// URL。<br>*env* `PLAYWRIGHT_MCP_ALLOW_UNRESTRICTED_FILE_ACCESS`  |
| --blocked-origins <origins> | 禁止浏览器请求的源列表，以分号分隔。先评估阻止列表，再评估允许列表。如果只使用阻止列表而未使用允许列表，不匹配阻止列表的请求仍会被允许。重要：它*不是*安全边界，也*不*影响重定向。<br>*env* `PLAYWRIGHT_MCP_BLOCKED_ORIGINS`  |
| --block-service-workers | 阻止 service worker。<br>*env* `PLAYWRIGHT_MCP_BLOCK_SERVICE_WORKERS`  |
| --browser <browser> | 要使用的浏览器或 Chrome 通道，可选值：chrome、firefox、webkit、msedge。<br>*env* `PLAYWRIGHT_MCP_BROWSER`  |
| --caps <caps> | 要启用的其他能力列表，以逗号分隔，可选值：vision、pdf、devtools。<br>*env* `PLAYWRIGHT_MCP_CAPS`  |
| --cdp-endpoint <endpoint> | 要连接的 CDP 端点。<br>*env* `PLAYWRIGHT_MCP_CDP_ENDPOINT`  |
| --cdp-header <headers...> | 连接请求中发送的 CDP 标头，可指定多个。<br>*env* `PLAYWRIGHT_MCP_CDP_HEADERS`  |
| --cdp-timeout <timeout> | 连接 CDP 端点的超时毫秒数，默认为 30000ms。<br>*env* `PLAYWRIGHT_MCP_CDP_TIMEOUT`  |
| --codegen <lang> | 指定代码生成语言，可选值："typescript"、"python"、"java"、"csharp"、"none"。默认为 "typescript"。<br>*env* `PLAYWRIGHT_MCP_CODEGEN`  |
| --config <path> | 配置文件路径。<br>*env* `PLAYWRIGHT_MCP_CONFIG`  |
| --console-level <level> | 要返回的控制台消息级别："error"、"warning"、"info"、"debug"。每个级别均包含更严重级别的消息。<br>*env* `PLAYWRIGHT_MCP_CONSOLE_LEVEL`  |
| --device <device> | 要模拟的设备，例如 "iPhone 15"。<br>*env* `PLAYWRIGHT_MCP_DEVICE`  |
| --mobile | 模拟通用移动设备（Chromium 使用 Pixel 10，WebKit 使用 iPhone 17）。移动页面通常更轻量，可节省令牌。不能与 --device 同时使用。<br>*env* `PLAYWRIGHT_MCP_MOBILE`  |
| --executable-path <path> | 浏览器可执行文件路径。<br>*env* `PLAYWRIGHT_MCP_EXECUTABLE_PATH`  |
| --extension | 连接正在运行的浏览器实例（仅限 Edge/Chrome）。需要安装 "Playwright Extension"。<br>*env* `PLAYWRIGHT_MCP_EXTENSION`  |
| --endpoint <endpoint> | 要连接的绑定浏览器端点。<br>*env* `PLAYWRIGHT_MCP_ENDPOINT`  |
| --file-paths <mode> | 工具结果中文件路径的呈现方式：相对工作区根目录的 "relative" 或 "absolute"。默认为 "relative"。<br>*env* `PLAYWRIGHT_MCP_FILE_PATHS`  |
| --grant-permissions <permissions...> | 要授予浏览器上下文的权限列表，例如 "geolocation"、"clipboard-read"、"clipboard-write"。<br>*env* `PLAYWRIGHT_MCP_GRANT_PERMISSIONS`  |
| --headless | 以无头模式运行浏览器，默认有头模式。<br>*env* `PLAYWRIGHT_MCP_HEADLESS`  |
| --host <host> | 服务器绑定的主机。默认为 localhost。使用 0.0.0.0 可绑定所有接口。<br>*env* `PLAYWRIGHT_MCP_HOST`  |
| --idle-timeout <timeout> | 在指定毫秒数内没有完成的工具调用后关闭浏览器，下一次工具调用会重新启动。无头浏览器默认为一小时，有头浏览器默认永不关闭，0 表示禁用。<br>*env* `PLAYWRIGHT_MCP_IDLE_TIMEOUT`  |
| --ignore-https-errors | 忽略 HTTPS 错误。<br>*env* `PLAYWRIGHT_MCP_IGNORE_HTTPS_ERRORS`  |
| --ignore-roots | 不向客户端询问工作区根目录，而是使用当前工作目录。适用于声明支持 roots 但无法响应 roots 请求的客户端。<br>*env* `PLAYWRIGHT_MCP_IGNORE_ROOTS`  |
| --init-page <path...> | 要在 Playwright 页面对象上执行的 TypeScript 文件路径。<br>*env* `PLAYWRIGHT_MCP_INIT_PAGE`  |
| --init-script <path...> | 要作为初始化脚本添加的 JavaScript 文件路径。脚本会在每个页面的所有脚本之前执行。可多次指定。<br>*env* `PLAYWRIGHT_MCP_INIT_SCRIPT`  |
| --isolated | 将浏览器配置文件保留在内存中，不保存到磁盘。<br>*env* `PLAYWRIGHT_MCP_ISOLATED`  |
| --image-responses <mode> | 是否向客户端发送图片响应。可为 "allow"、"omit" 或 "only"。使用 "only" 时，携带图片的响应仅包含图片部分，不含文本部分。默认为 "allow"。<br>*env* `PLAYWRIGHT_MCP_IMAGE_RESPONSES`  |
| --no-sandbox | 对通常启用沙箱的所有进程类型禁用沙箱。<br>*env* `PLAYWRIGHT_MCP_NO_SANDBOX`  |
| --no-webmcp | 不收集或公开页面通过 WebMCP API 注册的工具。<br>*env* `PLAYWRIGHT_MCP_WEBMCP=false`  |
| --output-dir <path> | 自动命名输出文件（例如未明确指定文件名的截图）的目录。明确命名的文件相对于工作区根目录解析，不受此选项影响。<br>*env* `PLAYWRIGHT_MCP_OUTPUT_DIR`  |
| --output-max-size <bytes> | 清理旧输出文件的阈值，单位为字节。<br>*env* `PLAYWRIGHT_MCP_OUTPUT_MAX_SIZE`  |
| --port <port> | SSE 传输监听的端口。<br>*env* `PLAYWRIGHT_MCP_PORT`  |
| --profile-dir-name <name> | 使用 --extension 连接时，用户数据目录中的配置文件目录名称，例如 "Profile 1"。默认使用最后使用且已安装扩展的配置文件。<br>*env* `PLAYWRIGHT_MCP_PROFILE_DIR_NAME`  |
| --proxy-bypass <bypass> | 绕过代理的域名，以逗号分隔，例如 ".com,chromium.org,.domain.com"。<br>*env* `PLAYWRIGHT_MCP_PROXY_BYPASS`  |
| --proxy-server <proxy> | 指定代理服务器，例如 "http://myproxy:3128"、"http://user:password@myproxy:3128" 或 "socks5://myproxy:8080"。<br>*env* `PLAYWRIGHT_MCP_PROXY_SERVER`  |
| --redact-url-params <names> | URL 查询和片段参数名称列表，以逗号分隔，例如 "access_token,code"；这些参数的值在工具响应中会被替换为 <redacted>。<br>*env* `PLAYWRIGHT_MCP_REDACT_URL_PARAMS`  |
| --sandbox | 对通常不启用沙箱的所有进程类型启用沙箱。<br>*env* `PLAYWRIGHT_MCP_SANDBOX`  |
| --save-session | 是否将 Playwright MCP 会话保存到输出目录。<br>*env* `PLAYWRIGHT_MCP_SAVE_SESSION`  |
| --secrets <path> | 包含 dotenv 格式密钥的文件路径。<br>*env* `PLAYWRIGHT_MCP_SECRETS_FILE`  |
| --shared-browser-context | 在所有已连接的 HTTP 客户端之间复用同一浏览器上下文。<br>*env* `PLAYWRIGHT_MCP_SHARED_BROWSER_CONTEXT`  |
| --snapshot-boxes | 在快照中以 [box=x,y,width,height] 包含每个元素的边界框。坐标相对于视口，单位为 CSS 像素。<br>*env* `PLAYWRIGHT_MCP_SNAPSHOT_BOXES`  |
| --snapshot-mode <mode> | 为响应拍摄快照时使用的模式。可为 "full" 或 "none"。默认为 "full"。<br>*env* `PLAYWRIGHT_MCP_SNAPSHOT_MODE`  |
| --storage-state <path> | 隔离会话的存储状态文件路径。<br>*env* `PLAYWRIGHT_MCP_STORAGE_STATE`  |
| --test-id-attribute <attribute> | 指定测试 ID 使用的属性，默认为 "data-testid"。<br>*env* `PLAYWRIGHT_MCP_TEST_ID_ATTRIBUTE`  |
| --timeout-action <timeout> | 指定操作超时毫秒数，默认为 5000ms。<br>*env* `PLAYWRIGHT_MCP_TIMEOUT_ACTION`  |
| --timeout-navigation <timeout> | 指定导航超时毫秒数，默认为 60000ms。<br>*env* `PLAYWRIGHT_MCP_TIMEOUT_NAVIGATION`  |
| --timeout-settle <timeout> | 每次操作后等待触发工作稳定的时间，单位为毫秒，默认为 500ms。<br>*env* `PLAYWRIGHT_MCP_TIMEOUT_SETTLE`  |
| --user-agent <ua string> | 指定用户代理字符串。<br>*env* `PLAYWRIGHT_MCP_USER_AGENT`  |
| --user-data-dir <path> | 用户数据目录路径。未指定时将创建临时目录。<br>*env* `PLAYWRIGHT_MCP_USER_DATA_DIR`  |
| --viewport-size <size> | 指定浏览器视口大小，单位为像素，例如 "1280x720"。<br>*env* `PLAYWRIGHT_MCP_VIEWPORT_SIZE`  |

<!--- End of options generated section -->

### 用户配置文件

你可以像普通浏览器一样使用持久配置文件运行 Playwright MCP（默认），也可以在隔离上下文中测试会话，或使用浏览器扩展连接现有浏览器。

**持久配置文件**

所有登录信息都会存储在持久配置文件中；如需清除离线状态，可以在会话之间删除该文件。
持久配置文件位于以下位置，你可以使用 `--user-data-dir` 参数覆盖其位置。

```bash
# Windows
%USERPROFILE%\AppData\Local\ms-playwright\mcp-{channel}-{workspace-hash}

# macOS
- ~/Library/Caches/ms-playwright/mcp-{channel}-{workspace-hash}

# Linux
- ~/.cache/ms-playwright/mcp-{channel}-{workspace-hash}
```

`{workspace-hash}` 根据 MCP 客户端的工作区根目录生成，因此不同项目会自动获得独立的配置文件。

> [!IMPORTANT]
> 一个持久配置文件同一时间只能由一个浏览器实例使用，因此共享同一工作区的并发 MCP 客户端会发生冲突。若要并行运行多个客户端，请为每个额外客户端指定 `--isolated`，或指向不同的 `--user-data-dir`。

**隔离模式**

在隔离模式下，每个会话都在隔离的配置文件中启动。每次要求 MCP 关闭浏览器时，
会话都会关闭，该会话的所有存储状态都会丢失。你可以通过配置中的 `contextOptions`
或通过 `--storage-state` 参数向浏览器提供初始存储状态。有关存储
状态的更多信息，请参见[此处](https://playwright.dev/docs/auth)。

```js
{
  "mcpServers": {
    "playwright": {
      "command": "npx",
      "args": [
        "@playwright/mcp@latest",
        "--isolated",
        "--storage-state={path/to/storage.json}"
      ]
    }
  }
}
```

**浏览器扩展**

Playwright MCP Chrome 扩展允许你连接现有浏览器标签页，并使用已登录的会话和浏览器状态。安装与设置说明参见 [microsoft/playwright › packages/extension](https://github.com/microsoft/playwright/tree/main/packages/extension#readme)。

### 初始状态

有多种方式可以向浏览器上下文或页面提供初始状态。

对于存储状态，你可以：
- 使用 `--user-data-dir` 参数从用户数据目录启动。这会在会话间持久保存所有浏览器数据。
- 使用 `--storage-state` 参数从存储状态文件启动。这会将文件中的 Cookie 和本地存储加载到隔离的浏览器上下文。

对于页面状态，你可以使用：

- `--init-page` 指向一个将在 Playwright 页面对象上执行的 TypeScript 文件。这样可以运行任意代码来设置页面。

```ts
// init-page.ts
export default async ({ page }) => {
  await page.context().grantPermissions(['geolocation']);
  await page.context().setGeolocation({ latitude: 37.7749, longitude: -122.4194 });
  await page.setViewportSize({ width: 1280, height: 720 });
};
```

- `--init-script` 指向一个作为初始化脚本添加的 JavaScript 文件。该脚本会在每个页面的所有脚本之前执行。
这适用于覆盖浏览器 API 或设置环境。

```js
// init-script.js
window.isPlaywrightMCP = true;
```

### 配置文件

Playwright MCP 服务器可以使用 JSON 配置文件进行配置。你可以
使用 `--config` 命令行选项指定配置文件：

```bash
npx @playwright/mcp@latest --config path/to/config.json
```

<details>
<summary>Configuration file schema</summary>

<!--- Config generated by update-readme.js -->

```typescript
{
  /**
   * 要使用的浏览器。
   */
  browser?: {
    /**
     * 要使用的浏览器类型。
     */
    browserName?: 'chromium' | 'firefox' | 'webkit';

    /**
     * 将浏览器配置文件保留在内存中，不保存到磁盘。
     */
    isolated?: boolean;

    /**
     * 用于持久保存浏览器配置文件的用户数据目录路径。
     * 默认创建临时目录。
     */
    userDataDir?: string;

    /**
     * 传递给以下方法的启动选项：
     * @see https://playwright.dev/docs/api/class-browsertype#browser-type-launch-persistent-context
     *
     * 可用于设置 `channel`、`headless`、`executablePath` 等选项。
     */
    launchOptions?: playwright.LaunchOptions;

    /**
     * 浏览器上下文的上下文选项。
     *
     * 可用于设置 `viewport` 等选项。
     */
    contextOptions?: playwright.BrowserContextOptions;

    /**
     * Chromium 系列浏览器用于连接现有浏览器实例的 Chrome DevTools Protocol 端点。
     */
    cdpEndpoint?: string;

    /**
     * 连接请求中发送的 CDP 标头。
     */
    cdpHeaders?: Record<string, string>;

    /**
     * 连接 CDP 端点的超时毫秒数。默认为 30000（30 秒）。传入 0 禁用超时。
     */
    cdpTimeout?: number;

    /**
     * 连接现有 Playwright 服务器的远程端点。可以是
     * WebSocket URL 字符串，或与测试运行器使用的 `connectOptions`
     * 形状一致的 [ConnectOptions] 对象。以对象传入时，
     * `exposeNetwork`、`headers`、`slowMo` 和 `timeout` 会传递给
     * 底层连接调用。
     */
    remoteEndpoint?: string | playwright.ConnectOptions & { endpoint: string };

    /**
     * 要作为 Playwright 页面初始化脚本添加的 TypeScript 文件路径。
     */
    initPage?: string[];

    /**
     * 要作为初始化脚本添加的 JavaScript 文件路径。
     * 这些脚本会在每个页面的所有脚本之前执行。
     */
    initScript?: string[];
  },

  /**
   * 连接正在运行的浏览器实例（仅限 Edge/Chrome）。指定后会忽略 `browser`
   * 配置。
   * 需要安装 "Playwright Extension"。
   */
  extension?: boolean;

  server?: {
    /**
     * SSE 或 MCP 传输监听的端口。
     */
    port?: number;

    /**
     * 服务器绑定的主机。默认为 localhost。使用 0.0.0.0 可绑定所有接口。
     */
    host?: string;

    /**
     * 此服务器允许服务的主机。默认为 localhost、环回 IP 和通过 --host 传入的主机；其他主机必须明确列出。
     * 此设置用于 DNS 重绑定保护，而非 CORS。
     */
    allowedHosts?: string[];
  },

  /**
   * 已启用的工具能力列表。可选值：
   *   - 'core'：核心浏览器自动化功能。
   *   - 'pdf'：PDF 生成和处理。
   *   - 'vision'：基于坐标的交互。
   *   - 'devtools'：开发者工具功能。
   */
  capabilities?: ToolCapability[];

  /**
   * 是否将 Playwright 会话保存到输出目录。
   */
  saveSession?: boolean;

  /**
   * 是否收集并公开页面通过实验性 WebMCP API 注册的工具。
   * 默认启用。
   */
  webmcp?: boolean;

  /**
   * 在所有已连接的 HTTP 客户端之间复用同一浏览器上下文。
   */
  sharedBrowserContext?: boolean;

  /**
   * 密钥用于替换工具响应中匹配的明文，以防止大语言模型
   * 意外获取敏感数据。这是一项便利功能而非安全功能；
   * 请务必检查客户端进出工具的信息。
   */
  secrets?: Record<string, string>;

  /**
   * URL 查询和片段参数名称，例如 `access_token` 或 `code`；其值
   * 在工具响应中会被替换为 `<redacted>`。用于运行时生成且无法列入
   * `secrets` 的会话和授权值。与 `secrets` 一样，这是一项便利功能，
   * 而非安全功能。
   */
  redactUrlParams?: string[];

  /**
   * 自动命名输出文件（例如未明确指定文件名的截图）的目录。
   * 明确命名的文件相对于工作区根目录解析，
   * 不受此选项影响。
   */
  outputDir?: string;

  /**
   * 清理旧输出文件的阈值，单位为字节。
   */
  outputMaxSize?: number;

  console?: {
    /**
     * 要返回的控制台消息级别。每个级别均包含更严重级别的消息。默认为 "info"。
     */
    level?: 'error' | 'warning' | 'info' | 'debug';
  },

  network?: {
    /**
     * 允许浏览器请求的源列表。默认允许所有源。同时匹配 `allowedOrigins` 和 `blockedOrigins` 的源将被阻止。
     *
     * 支持的格式：
     * - 完整源：`https://example.com:8080` - 仅匹配该源
     * - 通配端口：`http://localhost:*` - 匹配使用 http 协议的 localhost 任意端口
     */
    allowedOrigins?: string[];

    /**
     * 禁止浏览器请求的源列表。同时匹配 `allowedOrigins` 和 `blockedOrigins` 的源将被阻止。
     *
     * 支持的格式：
     * - 完整源：`https://example.com:8080` - 仅匹配该源
     * - 通配端口：`http://localhost:*` - 匹配使用 http 协议的 localhost 任意端口
     */
    blockedOrigins?: string[];
  };

  /**
   * 指定测试 ID 使用的属性，默认为 "data-testid"。
   */
  testIdAttribute?: string;

  timeouts?: {
    /*
     * 配置默认操作超时：https://playwright.dev/docs/api/class-page#page-set-default-timeout。默认为 5000ms。
     */
    action?: number;

    /*
     * 配置默认导航超时：https://playwright.dev/docs/api/class-page#page-set-default-navigation-timeout。默认为 60000ms。
     */
    navigation?: number;

    /**
     * 配置默认期望超时：https://playwright.dev/docs/test-timeouts#expect-timeout。默认为 5000ms。
     */
    expect?: number;

    /**
     * 响应前，每次操作后等待触发的工作（导航、请求）稳定的时间。默认为 500ms。
     */
    settle?: number;

    /**
     * 在指定毫秒数内没有工具调用后关闭浏览器，并在下一次调用时重新启动。
     * 对 Playwright 启动的无头浏览器默认为一小时，有头或附加浏览器默认无超时。传入 0 禁用。
     * CLI 会关闭整个会话，而不是重新启动。
     */
    idle?: number;
  };

  /**
   * 是否向客户端发送图片响应。可为 "allow"、"omit" 或 "only"。默认为 "allow"。
   * 使用 "only" 时，携带图片的响应仅包含图片部分，不含文本部分。
   */
  imageResponses?: 'allow' | 'omit' | 'only';

  /**
   * 工具结果中文件路径的呈现方式：相对工作区根目录的 "relative" 或 "absolute"。默认为 "relative"。
   */
  filePaths?: 'relative' | 'absolute';

  snapshot?: {
    /**
     * 为响应拍摄快照时使用的模式。
     */
    mode?: 'full' | 'none';

    /**
     * 是否在快照中以 [box=x,y,width,height] 包含每个元素的边界框。
     * 坐标相对于视口，单位为 CSS 像素（Element.getBoundingClientRect）。
     */
    boxes?: boolean;
  };

  /**
   * allowUnrestrictedFileAccess 是一道保护措施，旨在防止大语言模型意外
   * 离开预期工作区。它是一种用于捕获非预期文件访问的便利防护，
   * 并非安全边界；蓄意访问其他目录很容易绕过该限制，
   * 因此真正的安全性始终应依靠客户端级权限。
   */
  allowUnrestrictedFileAccess?: boolean;

  /**
   * 不向客户端询问工作区根目录，而是使用当前工作目录
   * 作为工作区根目录。适用于声明支持 roots 但
   * 无法响应 roots 请求的客户端。
   */
  ignoreRoots?: boolean;

  /**
   * 指定代码生成使用的语言。
   */
  codegen?: 'typescript' | 'python' | 'java' | 'csharp' | 'none';
}
```

<!--- End of config generated section -->

</details>

### 独立 MCP 服务器

在没有显示设备的系统上运行有头浏览器或从 IDE 工作进程运行时，
请从具有 DISPLAY 的环境运行 MCP 服务器，并传入 `--port` 标志以启用 HTTP 传输。

```bash
npx @playwright/mcp@latest --port 8931
```

然后在 MCP 客户端配置中将 `url` 设置为 HTTP 端点：

```js
{
  "mcpServers": {
    "playwright": {
      "url": "http://localhost:8931/mcp"
    }
  }
}
```

## 安全

Playwright MCP **不是**安全边界。有关保护部署的指导，请参阅 [MCP 安全最佳实践](https://modelcontextprotocol.io/docs/tutorials/security/security_best_practices)。

<details>
<summary><b>Docker</b></summary>

**注意：** Docker 实现目前只支持无头 chromium。

```js
{
  "mcpServers": {
    "playwright": {
      "command": "docker",
      "args": ["run", "-i", "--rm", "--init", "--pull=always", "mcr.microsoft.com/playwright/mcp"]
    }
  }
}
```

或者，如果你更希望将容器作为长期运行的服务运行，而不是让 MCP 客户端启动它，请使用：

```
docker run -d -i --rm --init --pull=always \
  --entrypoint node \
  --name playwright \
  -p 8931:8931 \
  mcr.microsoft.com/playwright/mcp \
  /app/cli.js --headless --browser chromium --no-sandbox --port 8931 --host 0.0.0.0
```

服务器将在主机端口 **8931** 上监听，任何 MCP 客户端都可以访问。  

你可以自行构建 Docker 镜像。

```
docker build -t mcr.microsoft.com/playwright/mcp .
```
</details>

<details>
<summary><b>Programmatic usage</b></summary>

```js
import http from 'http';

import { createConnection } from '@playwright/mcp';
import { SSEServerTransport } from '@modelcontextprotocol/sdk/server/sse.js';

http.createServer(async (req, res) => {
  // ...

  // 创建使用 SSE 传输的无头 Playwright MCP 服务器
  const connection = await createConnection({ browser: { launchOptions: { headless: true } } });
  const transport = new SSEServerTransport('/messages', res);
  await connection.connect(transport);

  // ...
});
```
</details>

### 工具

<!--- Tools generated by update-readme.js -->

<details>
<summary><b>核心自动化</b></summary>

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_click**
  - 标题：点击
  - 说明：在网页上执行点击
  - 参数：
    - `element`（字符串，可选）：用于获取与元素交互权限的人类可读元素描述
    - `target`（字符串）：页面快照中的精确目标元素引用，或唯一元素选择器
    - `doubleClick`（布尔值，可选）：是否执行双击而非单击
    - `button`（字符串，可选）：点击按钮，默认为左键
    - `modifiers`（数组，可选）：要按下的修饰键
  - 只读：**false**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_close**
  - 标题：关闭浏览器
  - 说明：关闭页面
  - 参数：无
  - 只读：**false**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_console_messages**
  - 标题：获取控制台消息
  - 说明：返回所有控制台消息
  - 参数：
    - `level`（字符串）：要返回的控制台消息级别。每个级别均包含更严重级别的消息。默认为 "info"。
    - `all`（布尔值，可选）：返回自会话开始以来的所有控制台消息，而非仅返回上次导航以来的消息。默认为 false。
    - `filename`（字符串，可选）：保存控制台消息的文件名。相对文件名相对于工作区根目录解析。未提供时以文本返回消息。
  - 只读：**true**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_drag**
  - 标题：拖动鼠标
  - 说明：在两个元素之间执行拖放
  - 参数：
    - `startElement`（字符串，可选）：用于获取与源元素交互权限的人类可读源元素描述
    - `startTarget`（字符串）：页面快照中的精确目标元素引用，或唯一元素选择器
    - `endElement`（字符串，可选）：用于获取与目标元素交互权限的人类可读目标元素描述
    - `endTarget`（字符串）：页面快照中的精确目标元素引用，或唯一元素选择器
  - 只读：**false**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_drop**
  - 标题：将文件或数据拖放到元素上
  - 说明：将文件或带 MIME 类型的数据拖放到元素上，如同从页面外部拖入。必须至少提供 "paths" 或 "data" 之一。
  - 参数：
    - `element`（字符串，可选）：用于获取与元素交互权限的人类可读元素描述
    - `target`（字符串）：页面快照中的精确目标元素引用，或唯一元素选择器
    - `paths`（数组，可选）：要拖放的文件绝对路径。
    - `data`（对象，可选）：要拖放的数据，为 MIME 类型到字符串值的映射（例如 {"text/plain": "hello", "text/uri-list": "https://example.com"}）。
  - 只读：**false**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_emulate_media**
  - 标题：模拟媒体功能
  - 说明：为页面模拟 CSS 媒体功能，例如在浅色和深色配色方案之间切换。省略的参数保持不变；null 清除覆盖。
  - 参数：
    - `colorScheme`（可选）：模拟 prefers-color-scheme 媒体功能
    - `reducedMotion`（可选）：模拟 prefers-reduced-motion 媒体功能
    - `forcedColors`（可选）：模拟 forced-colors 媒体功能
    - `contrast`（可选）：模拟 prefers-contrast 媒体功能
    - `media`（可选）：更改页面的 CSS 媒体类型
  - 只读：**false**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_evaluate**
  - 标题：执行 JavaScript
  - 说明：在页面或元素上执行 JavaScript 表达式
  - 参数：
    - `element`（字符串，可选）：用于获取与元素交互权限的人类可读元素描述
    - `target`（字符串，可选）：页面快照中的精确目标元素引用，或唯一元素选择器
    - `function`（字符串）：提供 element 时为 () => { /* code */ } 或 (element) => { /* code */ }
    - `filename`（字符串，可选）：保存结果的文件名。相对文件名相对于工作区根目录解析。未提供时以文本返回结果。
  - 只读：**false**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_file_upload**
  - 标题：上传文件
  - 说明：上传一个或多个文件
  - 参数：
    - `paths`（数组，可选）：要上传的文件的绝对路径。可以是单个或多个文件。省略时取消文件选择器。
  - 只读：**false**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_fill_form**
  - 标题：填写表单
  - 说明：填写多个表单字段
  - 参数：
    - `fields`（数组）：要填写的字段
  - 只读：**false**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_find**
  - 标题：在页面快照中查找
  - 说明：在当前页面的无障碍快照中搜索文本或正则表达式。返回匹配的快照节点及周围几行上下文（类似搜索摘要），每项显示在其从树根开始的路径下；当你只需定位元素及其引用时，这比获取整个快照更省资源。
  - 参数：
    - `text`（字符串，可选）：要在页面快照中搜索的纯文本（不区分大小写的子串匹配）。提供 text 或 regex 之一，不能同时提供。
    - `regex`（字符串，可选）：要在页面快照中搜索的正则表达式。默认区分大小写；用斜杠包围模式以添加标志，例如 "/error/i" 表示不区分大小写。提供 text 或 regex 之一，不能同时提供。
    - `maxResults`（数字，可选）：返回的匹配项最大数量。默认返回所有匹配项。
    - `filename`（字符串，可选）：将结果保存到文件，而非在响应中返回。相对文件名相对于工作区根目录解析。
  - 只读：**true**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_handle_dialog**
  - 标题：处理对话框
  - 说明：处理对话框
  - 参数：
    - `accept`（布尔值）：是否接受对话框。
    - `promptText`（字符串，可选）：提示对话框的输入文本。
  - 只读：**false**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_hover**
  - 标题：悬停鼠标
  - 说明：将鼠标悬停在页面元素上
  - 参数：
    - `element`（字符串，可选）：用于获取与元素交互权限的人类可读元素描述
    - `target`（字符串）：页面快照中的精确目标元素引用，或唯一元素选择器
  - 只读：**false**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_navigate**
  - 标题：导航至 URL
  - 说明：导航至 URL
  - 参数：
    - `url`（字符串）：要导航到的 URL
  - 只读：**false**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_navigate_back**
  - 标题：返回
  - 说明：返回历史记录中的上一页面
  - 参数：无
  - 只读：**false**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_network_request**
  - 标题：显示网络请求详情
  - 说明：返回单个网络请求的完整详情（标头和正文）；如果设置了 `part`，则只返回指定部分。使用 browser_network_requests 给出的编号。
  - 参数：
    - `index`（整数）：browser_network_requests 输出的请求编号，从 1 开始。
    - `part`（字符串，可选）：只返回请求的此部分。省略时返回完整详情。
    - `filename`（字符串，可选）：保存结果的文件名。相对文件名相对于工作区根目录解析。未提供时以文本返回输出。
  - 只读：**true**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_network_requests**
  - 标题：列出网络请求
  - 说明：返回自页面加载以来的网络请求编号列表。使用编号调用 browser_network_request 可获取完整详情。
  - 参数：
    - `static`（布尔值）：是否包括成功加载的静态资源，如图片、字体、脚本等。默认为 false。
    - `filter`（字符串，可选）：只返回 URL 与此正则表达式匹配的请求（例如 "/api/.*user"）。
    - `all`（布尔值，可选）：返回自会话开始以来的所有网络请求，而非仅返回上次导航以来的请求。默认为 false。
    - `filename`（字符串，可选）：保存网络请求的文件名。相对文件名相对于工作区根目录解析。未提供时以文本返回请求。
  - 只读：**true**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_press_key**
  - 标题：按键
  - 说明：按下键盘上的某个键
  - 参数：
    - `key`（字符串）：要按下的键名或要生成的字符，例如 `ArrowLeft` 或 `a`
  - 只读：**false**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_resize**
  - 标题：调整浏览器窗口大小
  - 说明：调整浏览器窗口大小
  - 参数：
    - `width`（数字）：浏览器窗口宽度
    - `height`（数字）：浏览器窗口高度
  - 只读：**false**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_run_code_unsafe**
  - 标题：运行 Playwright 代码（不安全）
  - 说明：运行 Playwright 代码片段。不安全：在 Playwright 服务器进程中执行任意 JavaScript，等同于远程代码执行（RCE）。
  - 参数：
    - `code`（字符串，可选）：包含要执行的 Playwright 代码的 JavaScript 函数。调用时会传入单个参数 page，可用于任意页面交互。例如：`async (page) => { await page.getByRole('button', { name: 'Submit' }).click(); return await page.title(); }`
    - `filename`（字符串，可选）：从指定文件加载代码。相对文件名相对于工作区根目录解析。如果同时提供 code 和 filename，将忽略 code。
  - 只读：**false**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_select_option**
  - 标题：选择选项
  - 说明：在下拉列表中选择选项
  - 参数：
    - `element`（字符串，可选）：用于获取与元素交互权限的人类可读元素描述
    - `target`（字符串）：页面快照中的精确目标元素引用，或唯一元素选择器
    - `values`（数组）：要在下拉列表中选择的值数组，可以是一个或多个值。
  - 只读：**false**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_snapshot**
  - 标题：页面快照
  - 说明：获取当前页面的无障碍快照，这比截图更好
  - 参数：
    - `target`（字符串，可选）：页面快照中的精确目标元素引用，或唯一元素选择器
    - `filename`（字符串，可选）：将快照保存到文件，而非在响应中返回。相对文件名相对于工作区根目录解析。
    - `depth`（数字，可选）：限制快照树的深度
    - `boxes`（布尔值，可选）：在快照中以 [box=x,y,width,height] 包含每个元素的边界框。坐标相对于视口，单位为 CSS 像素（Element.getBoundingClientRect）
  - 只读：**true**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_take_screenshot**
  - 标题：截图
  - 说明：截取当前页面。无法根据截图执行操作，执行操作请使用 browser_snapshot。
  - 参数：
    - `element`（字符串，可选）：用于获取与元素交互权限的人类可读元素描述
    - `target`（字符串，可选）：页面快照中的精确目标元素引用，或唯一元素选择器
    - `type`（字符串，可选）：截图的图片格式。未设置时根据文件扩展名推断，否则使用 png。
    - `filename`（字符串，可选）：保存截图的文件名。相对文件名相对于工作区根目录解析。未指定时，截图以 `page-{timestamp}.{png|jpeg|webp}` 保存至输出目录。
    - `fullPage`（布尔值，可选）：为 true 时截取整个可滚动页面，而非当前可见视口。不能与元素截图同时使用。
    - `scale`（字符串）：图像分辨率缩放比例。"css" 生成以 CSS 像素计大小的截图（较小，在各设备上一致）；"device" 使用设备像素生成高分辨率截图（较大，考虑设备像素比）。默认为 css。
  - 只读：**true**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_type**
  - 标题：输入文本
  - 说明：向可编辑元素输入文本
  - 参数：
    - `element`（字符串，可选）：用于获取与元素交互权限的人类可读元素描述
    - `target`（字符串）：页面快照中的精确目标元素引用，或唯一元素选择器
    - `text`（字符串）：要输入元素的文本
    - `submit`（布尔值，可选）：是否提交输入的文本（输入后按 Enter）
    - `slowly`（布尔值，可选）：是否每次输入一个字符。有助于触发页面按键处理程序。默认一次填入全部文本。
  - 只读：**false**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_wait_for**
  - 标题：等待
  - 说明：等待文本出现或消失，或等待指定时间过去。若同时提供 text 和 textGone，则等待最先发生的事件。
  - 参数：
    - `time`（数字，可选）：等待秒数，最多 30。与 text 或 textGone 同用时，作为它们的超时值，替代默认操作超时。
    - `text`（字符串，可选）：等待其出现的文本
    - `textGone`（字符串，可选）：等待其消失的文本
  - 只读：**false**

</details>

<details>
<summary><b>标签页管理</b></summary>

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_tabs**
  - 标题：管理标签页
  - 说明：列出、创建、关闭或选择浏览器标签页。
  - 参数：
    - `action`（字符串）：要执行的操作
    - `index`（数字，可选）：标签页索引，用于关闭/选择。关闭时省略则关闭当前标签页。
    - `url`（字符串，可选）：新标签页要导航到的 URL，用于新建操作。
    - `isolatedContext`（字符串，可选）：创建新标签页所用隔离浏览器上下文的名称，用于新建操作。同一隔离上下文中的标签页共享 Cookie 和存储，不同上下文完全隔离。省略或为 "default" 时在默认上下文中创建。
  - 只读：**false**

</details>

<details>
<summary><b>浏览器安装</b></summary>

</details>

<details>
<summary><b>配置（通过 --caps=config 选择启用）</b></summary>

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_get_config**
  - 标题：获取配置
  - 说明：获取合并 CLI 选项、环境变量和配置文件后最终解析出的配置。
  - 参数：无
  - 只读：**true**

</details>

<details>
<summary><b>网络（通过 --caps=network 选择启用）</b></summary>

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_network_state_set**
  - 标题：设置网络状态
  - 说明：将浏览器网络状态设为在线或离线。离线时所有网络请求都会失败。
  - 参数：
    - `state`（字符串）：设为 "offline" 模拟离线模式，设为 "online" 恢复网络连接
  - 只读：**false**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_route**
  - 标题：模拟网络请求
  - 说明：设置路由以模拟与 URL 模式匹配的网络请求
  - 参数：
    - `pattern`（字符串）：要匹配的 URL 模式（例如 "**/api/users"、"**/*.{png,jpg}"）
    - `status`（数字，可选）：要返回的 HTTP 状态码（默认：200）
    - `body`（字符串，可选）：响应正文（文本或 JSON 字符串）
    - `contentType`（字符串，可选）：Content-Type 标头（例如 "application/json"、"text/html"）
    - `headers`（数组，可选）：以 "Name: Value" 格式添加的标头
    - `removeHeaders`（字符串，可选）：要从请求中删除的标头名称，以逗号分隔
  - 只读：**false**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_route_list**
  - 标题：列出网络路由
  - 说明：列出所有活动网络路由
  - 参数：无
  - 只读：**true**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_unroute**
  - 标题：删除网络路由
  - 说明：删除匹配模式的网络路由（未指定模式时删除全部路由）
  - 参数：
    - `pattern`（字符串，可选）：要取消路由的 URL 模式（省略则删除全部路由）
  - 只读：**false**

</details>

<details>
<summary><b>存储（通过 --caps=storage 选择启用）</b></summary>

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_cookie_clear**
  - 标题：清除 Cookie
  - 说明：清除所有 Cookie
  - 参数：无
  - 只读：**false**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_cookie_delete**
  - 标题：删除 Cookie
  - 说明：删除指定 Cookie（可选择仅限指定域名/路径）
  - 参数：
    - `name`（字符串）：要删除的 Cookie 名称
    - `domain`（字符串，可选）：仅删除此域名上的 Cookie
    - `path`（字符串，可选）：仅删除此路径上的 Cookie
  - 只读：**false**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_cookie_get**
  - 标题：获取 Cookie
  - 说明：按名称获取指定 Cookie
  - 参数：
    - `name`（字符串）：要获取的 Cookie 名称
  - 只读：**true**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_cookie_list**
  - 标题：列出 Cookie
  - 说明：列出所有 Cookie（可按域名/路径筛选）
  - 参数：
    - `domain`（字符串，可选）：按域名筛选 Cookie
    - `path`（字符串，可选）：按路径筛选 Cookie
  - 只读：**true**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_cookie_set**
  - 标题：设置 Cookie
  - 说明：设置 Cookie，可选指定标志（domain、path、expires、httpOnly、secure、sameSite）
  - 参数：
    - `name`（字符串）：Cookie 名称
    - `value`（字符串）：Cookie 值
    - `domain`（字符串，可选）：Cookie 域名
    - `path`（字符串，可选）：Cookie 路径
    - `expires`（数字，可选）：Cookie 过期时间，以 Unix 时间戳表示
    - `httpOnly`（布尔值，可选）：Cookie 是否仅限 HTTP
    - `secure`（布尔值，可选）：Cookie 是否安全
    - `sameSite`（字符串，可选）：Cookie 的 SameSite 属性
  - 只读：**false**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_localstorage_clear**
  - 标题：清除 localStorage
  - 说明：清除所有 localStorage
  - 参数：无
  - 只读：**false**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_localstorage_delete**
  - 标题：删除 localStorage 项
  - 说明：删除一个 localStorage 项
  - 参数：
    - `key`（字符串）：要删除的键
  - 只读：**false**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_localstorage_get**
  - 标题：获取 localStorage 项
  - 说明：按键获取 localStorage 项
  - 参数：
    - `key`（字符串）：要获取的键
  - 只读：**true**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_localstorage_list**
  - 标题：列出 localStorage
  - 说明：列出所有 localStorage 键值对
  - 参数：无
  - 只读：**true**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_localstorage_set**
  - 标题：设置 localStorage 项
  - 说明：设置一个 localStorage 项
  - 参数：
    - `key`（字符串）：要设置的键
    - `value`（字符串）：要设置的值
  - 只读：**false**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_sessionstorage_clear**
  - 标题：清除 sessionStorage
  - 说明：清除所有 sessionStorage
  - 参数：无
  - 只读：**false**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_sessionstorage_delete**
  - 标题：删除 sessionStorage 项
  - 说明：删除一个 sessionStorage 项
  - 参数：
    - `key`（字符串）：要删除的键
  - 只读：**false**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_sessionstorage_get**
  - 标题：获取 sessionStorage 项
  - 说明：按键获取 sessionStorage 项
  - 参数：
    - `key`（字符串）：要获取的键
  - 只读：**true**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_sessionstorage_list**
  - 标题：列出 sessionStorage
  - 说明：列出所有 sessionStorage 键值对
  - 参数：无
  - 只读：**true**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_sessionstorage_set**
  - 标题：设置 sessionStorage 项
  - 说明：设置一个 sessionStorage 项
  - 参数：
    - `key`（字符串）：要设置的键
    - `value`（字符串）：要设置的值
  - 只读：**false**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_set_storage_state**
  - 标题：恢复存储状态
  - 说明：从文件恢复存储状态（Cookie、本地存储）。恢复前会清除现有 Cookie 和本地存储。
  - 参数：
    - `filename`（字符串）：要从中恢复的存储状态文件路径。相对文件名相对于工作区根目录解析。
  - 只读：**false**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_storage_state**
  - 标题：保存存储状态
  - 说明：将存储状态（Cookie、本地存储）保存到文件以供日后复用
  - 参数：
    - `filename`（字符串，可选）：保存存储状态的文件名。相对文件名相对于工作区根目录解析。未指定时，存储状态以 `storage-state-{timestamp}.json` 保存到输出目录。
  - 只读：**true**

</details>

<details>
<summary><b>开发者工具（通过 --caps=devtools 选择启用）</b></summary>

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_annotate**
  - 标题：标注当前页面
  - 说明：以标注模式为当前页面打开 Playwright Dashboard，并等待用户绘制标注。返回带标注的截图、ARIA 快照及标注列表。
  - 参数：无
  - 只读：**true**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_hide_highlight**
  - 标题：隐藏元素高亮
  - 说明：删除之前为元素添加的高亮覆盖层。
  - 参数：
    - `element`（字符串，可选）：添加高亮时使用的人类可读元素描述；必须与传给 browser_highlight 的值一致。
    - `target`（字符串，可选）：页面快照中的精确目标元素引用，或唯一元素选择器
  - 只读：**true**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_highlight**
  - 标题：高亮元素
  - 说明：在页面元素周围显示持久高亮覆盖层。
  - 参数：
    - `element`（字符串，可选）：用于获取与元素交互权限的人类可读元素描述
    - `target`（字符串）：页面快照中的精确目标元素引用，或唯一元素选择器
    - `style`（字符串，可选）：应用于高亮覆盖层的附加行内 CSS，例如 "outline: 2px dashed red"。
  - 只读：**true**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_resume**
  - 标题：恢复暂停的脚本执行
  - 说明：脚本暂停后恢复执行。当调用时将 step 设为 true，执行将在下一操作前再次暂停。
  - 参数：
    - `step`（布尔值，可选）：为 true 时，执行将在下一操作前再次暂停，以便逐步调试。
    - `location`（字符串，可选）：在指定的 <file>:<line> 处暂停执行，例如 "example.spec.ts:42"。
  - 只读：**false**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_start_recording**
  - 标题：开始录制用户操作
  - 说明：开始将用户在浏览器中执行的操作录制为 Playwright 代码。当用户希望手动演示流程时使用。用户表示完成后调用 browser_stop_recording 取回录制操作。
  - 参数：无
  - 只读：**true**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_start_tracing**
  - 标题：开始跟踪
  - 说明：开始录制跟踪
  - 参数：无
  - 只读：**true**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_start_video**
  - 标题：开始录像
  - 说明：开始录制视频
  - 参数：
    - `filename`（字符串，可选）：保存视频的文件名。相对文件名相对于工作区根目录解析。未指定时，视频以 `video-{timestamp}.webm` 保存到输出目录。
    - `size`（对象，可选）：视频大小
    - `fps`（数字，可选）：视频帧率，单位为每秒帧数，默认为 25
    - `cursor`（布尔值，可选）：渲染一个移动到各操作点的动画鼠标光标。操作间隔 800ms，让光标有时间移动。
  - 只读：**true**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_stop_recording**
  - 标题：停止录制用户操作
  - 说明：停止由 browser_start_recording 开始的录制，并将录制操作作为 Playwright 代码返回。
  - 参数：无
  - 只读：**true**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_stop_tracing**
  - 标题：停止跟踪
  - 说明：停止录制跟踪
  - 参数：无
  - 只读：**true**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_stop_video**
  - 标题：停止录像
  - 说明：停止录制视频
  - 参数：无
  - 只读：**true**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_video_chapter**
  - 标题：视频章节
  - 说明：向录像添加章节标记。显示带模糊背景的全屏章节卡片。
  - 参数：
    - `title`（字符串）：章节标题
    - `description`（字符串，可选）：章节说明
    - `duration`（数字，可选）：章节卡片的显示时间，单位为毫秒
  - 只读：**true**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_video_hide_actions**
  - 标题：隐藏操作覆盖层
  - 说明：停止标注在页面上执行的操作。
  - 参数：无
  - 只读：**true**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_video_show_actions**
  - 标题：显示操作覆盖层
  - 说明：为后续在页面上执行的操作添加标注，注明操作名称，并在设置样式时标记操作点及高亮目标元素。适用于录像或屏幕录制。
  - 参数：
    - `duration`（数字，可选）：每条操作标注在屏幕上的停留时间，单位为毫秒。默认为 500。
    - `position`（字符串，可选）：操作标题相对页面的位置。默认为右上角。
    - `cursor`（字符串，可选）：指针操作的光标装饰。"pointer"（默认）将鼠标指针从上一个操作点移动到下一个操作点；"none" 禁用光标装饰。
    - `style`（对象，可选）：操作装饰的样式。
  - 只读：**true**

</details>

<details>
<summary><b>基于坐标（通过 --caps=vision 选择启用）</b></summary>

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_mouse_click_xy**
  - 标题：点击
  - 说明：在指定位置点击鼠标按钮
  - 参数：
    - `x`（数字）：X 坐标
    - `y`（数字）：Y 坐标
    - `button`（字符串，可选）：点击按钮，默认为左键
    - `clickCount`（数字，可选）：点击次数，默认为 1
    - `delay`（数字，可选）：鼠标按下与松开之间的等待时间，单位为毫秒，默认为 0
  - 只读：**false**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_mouse_down**
  - 标题：按下鼠标
  - 说明：按下鼠标
  - 参数：
    - `button`（字符串，可选）：要按下的按钮，默认为左键
  - 只读：**false**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_mouse_drag_xy**
  - 标题：拖动鼠标
  - 说明：按住鼠标左键拖动至指定位置
  - 参数：
    - `startX`（数字）：起始 X 坐标
    - `startY`（数字）：起始 Y 坐标
    - `endX`（数字）：结束 X 坐标
    - `endY`（数字）：结束 Y 坐标
  - 只读：**false**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_mouse_move_xy**
  - 标题：移动鼠标
  - 说明：将鼠标移动到指定位置
  - 参数：
    - `x`（数字）：X 坐标
    - `y`（数字）：Y 坐标
  - 只读：**false**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_mouse_up**
  - 标题：松开鼠标
  - 说明：松开鼠标
  - 参数：
    - `button`（字符串，可选）：要按下的按钮，默认为左键
  - 只读：**false**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_mouse_wheel**
  - 标题：滚动鼠标滚轮
  - 说明：滚动鼠标滚轮
  - 参数：
    - `deltaX`（数字）：X 方向增量
    - `deltaY`（数字）：Y 方向增量
  - 只读：**false**

</details>

<details>
<summary><b>生成 PDF（通过 --caps=pdf 选择启用）</b></summary>

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_pdf_save**
  - 标题：保存为 PDF
  - 说明：将页面保存为 PDF
  - 参数：
    - `filename`（字符串，可选）：保存 PDF 的文件名。相对文件名相对于工作区根目录解析。未指定时，PDF 以 `page-{timestamp}.pdf` 保存到输出目录。
  - 只读：**true**

</details>

<details>
<summary><b>测试断言（通过 --caps=testing 选择启用）</b></summary>

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_generate_locator**
  - 标题：为元素创建定位器
  - 说明：为给定元素生成供测试使用的定位器
  - 参数：
    - `element`（字符串，可选）：用于获取与元素交互权限的人类可读元素描述
    - `target`（字符串）：页面快照中的精确目标元素引用，或唯一元素选择器
  - 只读：**true**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_verify_element_visible**
  - 标题：验证元素可见
  - 说明：验证元素在页面上可见
  - 参数：
    - `role`（字符串）：元素的 ROLE。可在快照中找到，例如：`- {ROLE} "Accessible Name":`
    - `accessibleName`（字符串）：元素的 ACCESSIBLE_NAME。可在快照中找到，例如：`- role "{ACCESSIBLE_NAME}"`
  - 只读：**false**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_verify_list_visible**
  - 标题：验证列表可见
  - 说明：验证列表在页面上可见
  - 参数：
    - `element`（字符串）：人类可读列表描述
    - `target`（字符串）：指向列表的精确目标元素引用
    - `items`（数组）：要验证的项目
  - 只读：**false**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_verify_text_visible**
  - 标题：验证文本可见
  - 说明：验证文本在页面上可见。如有可能，优先使用 browser_verify_element_visible。
  - 参数：
    - `text`（字符串）：要验证的 TEXT。可在快照中找到，例如：`- role "Accessible Name": {TEXT}` 或 `- text: {TEXT}`
  - 只读：**false**

<!-- NOTE: This has been generated via update-readme.js -->

- **browser_verify_value**
  - 标题：验证值
  - 说明：验证元素值
  - 参数：
    - `type`（字符串）：元素类型
    - `element`（字符串）：人类可读元素描述
    - `target`（字符串）：页面快照中的精确目标元素引用
    - `value`（字符串）：要验证的值。复选框使用 "true" 或 "false"。
  - 只读：**false**

</details>


<!--- End of tools generated section -->
