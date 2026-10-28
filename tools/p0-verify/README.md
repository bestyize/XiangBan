# P0 验证脚本（语音编程）

用来在你的 Windows 与 macOS 电脑上核实外部 Agent 的接口行为，对应 [目标与实施计划_语音编程](../../docs/product/目标与实施计划_语音编程.md) 的 P0 和 [外部Agent接口核实_P0](../../docs/agent/外部Agent接口核实_P0.md)。**每台机器都跑一遍**，结果填进 [RECORD.md](RECORD.md)。

## 准备

- Node.js 18 以上（装 Claude Code / Codex 通常已经有）。无需 `npm install`，脚本没有第三方依赖。
- 已安装并登录 Claude Code 和 Codex（`npm i -g @anthropic-ai/claude-code @openai/codex`，或各自的官方安装方式）。只想验证其中一个也可以，另一个的脚本会提示后退出。
- 做桌面版探测前，先打开 Claude / ChatGPT 桌面版，进入一个对话页，把光标放在输入框里。macOS 要在“系统设置 → 隐私与安全性 → 辅助功能”里给运行脚本的终端授权。

## 顺序

| 脚本 | 做什么 | 会不会调用模型 |
| --- | --- | --- |
| `01-env-check.mjs` | 系统、Node、git、Claude Code、Codex 的版本与路径，桌面版是否在运行 | 否 |
| `02-codex-appserver.mjs` | 启动 `codex app-server`，握手、列出会话；加 `--chat` 再发一轮真实对话 | 仅 `--chat` 时（一轮，极短） |
| `03-claude-hooks.mjs` | 在临时目录里配 hooks（http 与 command 两种），跑一轮需要权限确认的对话，看收到哪些事件、hook 返回“允许”是否生效 | 是（两轮，极短） |
| `04-claude-resume.mjs` | 无界面建立会话 → 按编号恢复 → 确认会话文件位置 → 单进程多轮输入；最后提示你在真实终端里手动 `claude --resume` | 是（四轮，极短） |
| `06-claude-stop-inject.mjs` | 验证 `Stop` hook 返回“继续并附上指令”后会话是否带着新指令接着做（语音指令注入运行中会话的机制） | 是（两轮，极短） |
| `05-desktop-probe.mjs <应用名>` | 用系统无障碍接口找桌面版的输入框；试“激活并粘贴”（不按回车）并计时 | 否 |

```sh
node 01-env-check.mjs
node 02-codex-appserver.mjs --chat
node 03-claude-hooks.mjs both
node 04-claude-resume.mjs
node 05-desktop-probe.mjs Claude both      # 再换 ChatGPT、Codex 各跑一次
```

Windows 用 PowerShell 或 cmd 运行即可，命令相同。

## 安全说明

- 脚本**不读取、不保存任何登录凭据**；`claude auth status`、`codex login status` 只看退出码。
- hooks 只写在**临时目录**的 `.claude/settings.local.json`，不改你的全局设置；临时目录用完即删。
- 会调用模型的几步都是几句话的小请求，消耗很小，但用的是你已登录的账号。
- `05` 会把一行测试文字粘贴进你打开的应用输入框，**不会按回车**；脚本会恢复你原来的剪贴板，但粘贴期间请不要复制别的东西。
- 结果写在 `out/`（已被 git 忽略），其中的家目录路径已替换成 `~`。发给我之前自己打开看一眼。

## 已在 Linux 沙箱里验证过的部分

`samples/` 里是在云端 Linux 沙箱用真实登录跑出来的样例（Claude Code 2.1.312、Codex 0.165.0）：Codex app-server 握手、Claude hooks 事件载荷、stream-json 事件类型。结论见 P0 文档的“实测结果”一节。Windows、macOS 上的行为和桌面版的探测**没有**验证过，`probe-mac.js` 和 `probe-win.ps1` 是第一次写出来的探测脚本，出错时把完整报错带回来，我来改。
