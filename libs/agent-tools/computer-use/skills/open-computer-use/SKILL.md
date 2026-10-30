---
name: open-computer-use
user-invocable: false
description: 操作电脑上其他应用（看窗口、点击、输入、滚动、拖拽）前必读。说明 list_apps、get_app_state、click、type_text、set_value、query 等电脑控制工具的正确用法、取不到文字或点不中时的处理，以及哪些操作要先问用户。
---

# 电脑控制（Open Computer Use）

改编自开源项目 iFurySt/open-codex-computer-use 的官方 skill（MIT 许可），按小伴的工具调用方式重写。

这些工具通过系统无障碍接口读取其他应用的控件树，再按控件编号操作。它们直接以工具的形式出现在你的工具列表里，不要用 shell 去运行 `open-computer-use` 命令。

## 工具

所有平台都有这九个：`list_apps`、`get_app_state`、`click`、`perform_secondary_action`、`scroll`、`drag`、`type_text`、`press_key`、`set_value`。macOS 另有 `query`：按文字或角色直接查找控件，不需要截图，也不需要渲染整棵树。以工具列表里实际出现的为准，每个工具的参数说明以它自己的描述为准。

## 标准流程

0. 首次操作、权限错误或用户说“开关已经开了仍不能用”时，先调用 `computer_use_diagnose`。MCP 已连接不代表系统授权有效。macOS 看 `engine.permissions` 的真实布尔状态；缺权限先用 `request_accessibility_permission` 启动原生引导，用户授权后复查。`engine.authorizationMismatch.services` 有对应项，表示系统日志确认旧签名冲突；先说明影响，再用 `computer_use_repair_permissions(permission="accessibility" 或 "screenRecording")`，由应用逐次确认后仅清除引擎所选旧授权，并打开当前 App 和系统设置。系统最后的授权点击由用户完成。日志不可用或没匹配时不能声称已确认签名问题。诊断可在 MCP 未启动时使用。
1. `list_apps` 看有哪些应用在运行，最简参数是 `{}`；不要加 schema 没声明的 `reason`、`command` 等字段。用它返回的应用名或 bundle id 作为 `app` 参数。目标应用不在列表里时，如实告诉用户，不要悄悄换成别的应用。
2. `get_app_state` 取目标应用当前的控件树。要用 `element_index` 的操作，都必须紧接在最近一次 `get_app_state` 之后；导航、弹窗出现、页面刷新、操作失败后都要重新取，不要凭记忆沿用旧编号，也不要跨对话猜编号。
3. 优先用元素编号操作：可编辑的输入框、文本框先试 `set_value`，其次才是 `type_text`；按钮、菜单项用 `click` 带 `element_index`；右键菜单、展开等次要动作用 `perform_secondary_action`，并且只用状态结果里列出的动作。
4. 只有控件树里找不到目标时，才改用坐标的 `click`、`scroll`、`drag`。
5. 每次动作之后看返回的新状态，确认真的生效了再做下一步。

## 文字被截断、内容不全

- 状态里的文字默认只保留 500 个字符，以 `...` 结尾就是被截断了，不是页面没内容。需要读聊天记录、邮件正文、文档段落时，调用 `get_app_state` 并带 `text_limit: 1000`，要完整内容用 `text_limit: "max"`。
- 长网页、长列表看起来不完整（截图里有、控件树里没有）可能是触到了默认上限：1200 个节点、64 层深。带上 `max_tree_nodes: 3000` 和 `max_tree_depth: 96` 重取。`text_limit: "max"` 不会放宽这两个上限。
- 动作工具返回的刷新状态一律用默认限制。动作之后还要读长文字，要再单独调一次 `get_app_state`。

## macOS 专用：query

已知控件的文字或角色时，用 `query` 直接定位，比读整棵树快得多，也更不容易漏。结果里 `truncated` 为 false 时，空结果才代表“没有找到这样的控件”。查询编号只在同一次会话内保留约 120 秒，动作会重新校验位置，失败就重新 `query`。

## 点击和键盘输入的方式

- `click` 不写 `click_method` 就是 `auto`：先走无障碍语义，不行再退到事件。显式指定的方式不会回退。
- `accessibility`：只调用控件的无障碍动作，必须带 `element_index`。
- `app_post`：不经过无障碍，直接给目标窗口发鼠标事件，不会移动系统鼠标。适合必须点在空白处、或不能被重定向到子控件的点击。
- `sky_click`（仅 macOS）：目标窗口被别的窗口盖住、或 Chromium 忽略 `app_post` 时使用，不会移动鼠标、也不会把窗口提到前面。窗口移动、关闭、切换桌面、隐藏或最小化之后，要先重新 `get_app_state`。
- `global`：走全局鼠标，会真的移动鼠标、改变前台焦点，需要运行环境显式开启，除非用户明确要求，不要用。
- `type_text` 和 `press_key` 的 `key_method` 同理。`sky_key`（仅 macOS）在窗口被盖住或在别的桌面时使用：先点击或聚焦输入框，因为它输入到目标窗口当前聚焦的位置；`cmd+a`、`cmd+c`、`cmd+v` 这类组合键会走目标应用自己的菜单。

## 窗口被盖住、在别的桌面

`get_app_state` 不会主动把离屏窗口提到前面。结果末尾提示窗口被盖住时，让用户把窗口切到可见位置再取一次。macOS 上也可以用 `window_placement: "agent_display"` 把窗口临时挪到一块用户看不见的虚拟显示器上操作，操作完必须用 `window_placement: "restore"` 还原；这期间窗口会从用户桌面上消失，所以只在确有必要时用，并把挪走、操作、还原放在同一轮里完成。

## 拖拽

默认的 `drag` 只是把鼠标事件发给目标进程，窗口移动、拖选文字、访达里拖放文件这三类操作不会有任何效果，却不会报错，只会在结果里写 `Drag delivered via app_post`。看到这句就要知道没做成。换个办法达到同样的结果：移动文件用 shell 命令，选中文字用 `set_value` 或键盘选择，移动窗口用应用自己的窗口控制。

## 出问题时

- `invalid arguments`、`Additional properties are not allowed`：本次工具没有执行；对照当前 schema 删除多余字段或补齐必填字段，再重试原工具。参数错误不构成权限或网站不支持的证据，不要因此重置授权，不要转用 shell/Python/AppKit/AppleScript。`python3 -c "import AppKit..."` 不能替代 `list_apps`。
- `computer_use_permission_required`：运行诊断、启动授权引导，说明系统实际需要授权的条目；等待后复查。旧签名冲突可由 `computer_use_repair_permissions` 经用户确认后定向修复。不能在 shell 自行重置 TCC，更不能 reset All、直接写权限数据库或脚本绕过授权。有效权限受保护，修复工具不会清除已生效的所选权限。同 Bundle ID 的独立安装版也受所选权限重置影响，确认前要说明。
- `stableSigningIdentity: false` 表示签名身份可能随构建变化，向用户指出当前引擎路径和签名问题；用户机器不需要导入构建证书，也不要让用户安装根证书。
- 状态为空、没有截图：目标应用没有可见窗口，或窗口被最小化、隐藏；macOS 还可能缺屏幕录制权限。缺无障碍或屏幕录制权限时，请用户在系统设置 → 隐私与安全性中为系统显示的应用条目授权；内置 Open Computer Use 引擎可能单独列出。只有当前工具列表存在权限工具时才能调用，不要编造 `request_accessibility_permission`。
- 元素动作失败：重新 `get_app_state`，确认编号还指向原来的控件；能 `set_value` 就 `set_value`；语义路线都不通了才用坐标。
- macOS 低于 14.0 运行不了，这种情况权限设置帮不上忙，如实告诉用户需要升级系统。
- Windows 和 Linux 必须在已登录的图形桌面会话里才看得到窗口。

## 浏览器

页面内部元素没有展开时，不要反复盲猜坐标。先确认权限有效，再用快捷键（macOS 的 ⌘L 定位地址栏、Tab 切换焦点）配合 `press_key`。浏览器交互使用电脑控制工具，不要用 System Events/AppleScript 额外申请主程序无障碍权限。打开网址只是准备步骤，不代表完成用户要求的浏览器任务。

## 安全

这是用户正在使用的真实桌面。

- 发送消息、提交表单、删除、购买、批准授权、上传文件，以及任何会对外产生影响的动作，先问用户。
- 密码管理器和其他敏感应用，除非用户明确要求，不要查看。
- 不要绕过系统的权限弹窗，也不要为了方便开启全局鼠标。
