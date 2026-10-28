# P0 验证记录表

Windows 11 本机实测记录（2026-10-28）。

## 基本信息

| 项 | 值 |
| --- | --- |
| 系统与版本 | Windows 11 Home China (win32 10.0.26300 x64) |
| 机型 / 芯片 | x86_64 |
| Node 版本 | v24.14.0 |
| Claude Code 版本 | 2.1.285 (Claude Code CLI) |
| Codex 版本 | codex-cli 0.159.2 |
| Claude 桌面版版本 | Claude.exe (运行中，MainWindowTitle: Claude) |
| ChatGPT 桌面版版本 | ChatGPT.exe (运行中，MainWindowTitle: ChatGPT) |
| 是否中国大陆网络 / 是否用了代理 | 中国大陆网络 / 本机代理 |
| 登录方式（订阅 / API 密钥 / 第三方接口） | Codex: 官方订阅登录；Claude: 桌面端已登录，CLI 尚未登录 (`claude auth status` 退出码 1) |

## A. 脚本结果

| 脚本 | 结果（✅ / ❌ / 部分） | 备注、报错 |
| --- | --- | --- |
| 01 环境检查 | ✅ | 修复了 Windows 上 `where` 匹配 extensionless 文件导致 spawn 报错的问题；识别出 Node v24.14.0、git 2.53.0、claude 2.1.285、codex 0.159.2，检测到 Claude 与 ChatGPT 桌面版进程 |
| 02 Codex 握手与列表 | ✅ | `initialize` 与 `thread/list` 正常，支持 `data,nextCursor,backwardsCursor` 分页 |
| 02 Codex `--chat` 一轮对话（是否收到 `turn/completed`） | ✅ | `thread/start` 成功建立，`turn/start` 发送，11492 ms 后完整收到 `turn/completed` 以及 `item/agentMessage/delta` 等全部流式通知 |
| 03 Claude hooks：http 类型收到的事件 | 部分 | 收到 `UserPromptSubmit: 1, SessionEnd: 1`；确认了 http 类型收不到 `SessionStart`。因 CLI 未登录，模型请求未流出 |
| 03 Claude hooks：command 类型收到的事件 | 部分 | 收到 `SessionStart: 1, UserPromptSubmit: 1, SessionEnd: 1`。命令类型能稳定捕获 `SessionStart` |
| 03 PermissionRequest 返回“允许”后工具是否执行 | ❌ | 因 CLI 报 `Not logged in` 未进入工具调用步骤 |
| 04 无界面恢复后记得内容 | 部分 | 建立会话与同一 `session_id` 恢复均成功，但因未登录回答 `Not logged in` 未能复述暗号 |
| 04 会话文件是否在预期位置 | ✅ | 文件位于 `~/.claude/projects/C--Users-sdjz0-AppData-Local-Temp-p0-resume-<id>/<uuid>.jsonl`，确认了 Windows 盘符与路径转短划线的映射规范 |
| 04 单进程多轮输入 | ✅ | `--input-format stream-json` 多轮输入正常消费，第二轮响应仅 624 ms |
| 04 手动：终端 `claude --resume` 能否看到完整历史并接着聊 | 不确定 | CLI 需先完成 `claude /login` 后方可继续对话 |

## B. 桌面版（每个应用一行；快捷键等要手动试）

| 应用 | 有没有全局快捷键 / 快速输入（键位） | 无障碍找到输入框？（05 tree） | 激活并粘贴用时（05 paste） | 文字真的进了输入框？ | 备注 |
| --- | --- | --- | --- | --- | --- |
| Claude 桌面版 | 默认快捷键呼出 | 0 个（Electron 默认无障碍树在后台未完全暴露给 UIA 查询） | 250 ms | 是（通过窗口置顶激活并键入 Ctrl+V 注入） | UIA 树递归深度受限，但 `ShowWindow(9)` + `SetForegroundWindow` + `SendKeys("^v")` 极快生效 |
| ChatGPT 桌面版 | Alt+Space（依系统设置） | 找到 2 个（`ControlType.Document` + `ControlType.Edit`，聚焦的 `ProseMirror` 输入框） | 265 ms | 是 | UIA 能直接定位到 `ProseMirror-focused` 类名的输入框 |
| Codex 桌面版（如已安装） | 暂无独立 GUI 窗口 | ❌（无 MainWindowHandle 进程） | 无 | 无 | 本机主要使用 `codex` CLI 与 `codex app-server`，无独立带窗体桌面 App |

补充观察：
- 应用在后台 / 最小化 / 另一个桌面时，激活是否成功：`ShowWindow(handle, 9)`（RESTORE）加 `SetForegroundWindow` 在 Windows 11 下均能成功唤醒并置顶窗口（耗时 250–265 ms）。
- 输入框里已有未发送文字时，粘贴是追加还是覆盖：`Ctrl+V` 在聚焦光标位置为追加粘贴，不会覆盖已有文字。
- 中文输入法开启时粘贴有无异常：使用 Windows 剪贴板 `Set-Clipboard` 加 `^v` 粘贴绕过了输入法编码拦截，不会出现拼音乱码。
- 是否出现权限弹窗（macOS 辅助功能 / 屏幕录制）：Windows 下管理员进程操作同权限窗口不需要额外 UAC 弹窗，普通权限进程即可操作同权限窗口。

## C. 想到的任何问题

1. **Windows 下 npm 命令探测兼容性**：Windows 上 `where` 命令会优先返回无后缀的 shell 脚本（如 `.../npm/claude`），直接用 Node 的 `spawn` 执行会报 `%1 is not a valid Win32 application`。`lib.mjs` 中的 `which` 已修正为优先查找 `.exe`、`.cmd`、`.bat`。
2. **Claude Code hooks 的 http 与 command 差异**：Windows 上同样验证了 `http` 类型的 hook 不会收到 `SessionStart` 事件，而 `command` 类型能收到 `SessionStart`。
3. **输入框注入路线验证**：Windows 下 `ShowWindow(9)` + `SetForegroundWindow` + 剪贴板 `Ctrl+V` 注入总耗时仅 ~250ms，比任何看图识别方式快得多，证实了“先上屏后发送”界面层路线在 Windows 上的极高可行性。
