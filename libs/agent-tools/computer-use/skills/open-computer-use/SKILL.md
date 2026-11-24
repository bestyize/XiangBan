---
name: open-computer-use
user-invocable: false
description: 操作电脑上其他应用（看窗口、点击、输入、滚动、拖拽）前必读。说明 list_apps、get_app_state、click、type_text、set_value、query 等电脑控制工具的正确用法、取不到文字或点不中时的处理，以及哪些操作要先问用户。
---

# 电脑控制（Open Computer Use）

改编自开源项目 iFurySt/open-codex-computer-use 的官方 skill（MIT 许可），按小伴的工具调用方式重写。

这些工具通过系统无障碍接口读取其他应用的控件树，再按控件编号操作。它们直接以工具的形式出现在你的工具列表里，不要用 shell 去运行 `open-computer-use` 命令。

## 工具

所有平台提供 `list_apps`、`get_app_state`、`click`、`perform_secondary_action`、`scroll`、`drag`、`type_text`、`press_key`、`set_value`。macOS 另有 `query` 和 `select_text`，可以按文字或角色定位控件、在可编辑控件中选中指定文字。参数以本轮工具列表为准。

## 标准流程

0. 普通读取和动作会自动检查权限。权限错误或用户说“开关已经开了仍不能用”时，先调用 `computer_use_diagnose`。MCP 已连接不代表系统授权有效。macOS 看 `engine.permissions` 的真实布尔状态；缺权限先用 `request_accessibility_permission` 启动原生引导，用户授权后复查。`engine.authorizationMismatch.services` 有对应项，表示系统日志确认旧签名冲突；先说明影响，再用 `computer_use_repair_permissions(permission="accessibility" 或 "screenRecording")`，由应用逐次确认后仅清除引擎所选旧授权，并打开当前 App 和系统设置。系统最后的授权点击由用户完成。日志不可用或没匹配时不能声称已确认签名问题。诊断可在 MCP 未启动时使用。
1. 已知应用名或 bundle id 可直接读取窗口；需要发现应用时用 `list_apps` 看有哪些应用在运行，最简参数是 `{}`；不要加 schema 没声明的 `reason`、`command` 等字段。用它返回的应用名或 bundle id 作为 `app` 参数。目标应用不在列表里时，如实告诉用户，不要悄悄换成别的应用。
2. `get_app_state` 取目标应用当前的控件树。要用 `element_index` 的操作，都必须紧接在最近一次 `get_app_state` 之后；导航、弹窗出现、页面刷新、操作失败后都要重新取，不要凭记忆沿用旧编号，也不要跨对话猜编号。
3. 优先用元素编号操作：可编辑的输入框、文本框先试 `set_value`，其次才是 `type_text`；按钮、菜单项用 `click` 带 `element_index`；右键菜单、展开等次要动作用 `perform_secondary_action`，并且只用状态结果里列出的动作。
4. 只有控件树里找不到目标时，才改用坐标的 `click`、`scroll`、`drag`。
5. 打开菜单、选择钢笔、出现弹窗等会改变界面，先看返回状态。已确定同一画布的一笔路径时，连续落点后一起验收。

## 在 Mac 画布上连续落点

查看一次带截图的 `get_app_state`，从返回末尾取 `window_id` 和截图像素尺寸。若当前 `click` 提供 `points`，把同一笔已确定的 1–32 个点放进一次调用。例如在已选好钢笔的无边记白板上，传 `{"app":"com.apple.freeform","window_id":42,"points":[{"x":555,"y":340},{"x":735,"y":340,"click_count":2}]}`，示例编号 42 和坐标均须换成当前窗口实际值。若需先选已知绘图工具，可在同一调用传 `prepare_buttons:["Insert Shape","Draw with Pen"]`，工具逐个匹配窗口中的唯一按钮并确认回到画布；按钮名不确定时先观察。无边记新画独立一笔时，把 `freeform_new_stroke:true` 与钢笔按钮名同传，按双击终点收笔。小伴会在选择工具前及落点后分别按 Escape。仅凭水平线出现无法判断是否仍在旧路径中；核对整笔及周围留白，钢笔尚在编辑时点击空白处可能增加斜线。已知各点时不要逐点请求模型。最后一张截图用于核对成形效果。
窗口或尺寸变化、工具返回中途停止时，重新观察已留下的笔画，从尚未完成处继续。跨窗口操作、未知菜单及须单独确认的动作各自执行。

只需读文字、查控件或核对字段时，`get_app_state` 传 `include_screenshot: false`。需要看画布、图标或按坐标操作时保留截图；只要画面时可传 `include_ax: false`。两项至少保留一项。

## 文字被截断、内容不全

- 状态里的文字默认只保留 500 个字符，以 `...` 结尾就是被截断了，不是页面没内容。需要读聊天记录、邮件正文、文档段落时，调用 `get_app_state` 并带 `text_limit: 1000`，要完整内容用 `text_limit: "max"`。
- 长网页、长列表看起来不完整（截图里有、控件树里没有）可能是触到了默认上限：1200 个节点、64 层深。带上 `max_tree_nodes: 3000` 和 `max_tree_depth: 96` 重取。`text_limit: "max"` 不会放宽这两个上限。
- 动作工具返回的刷新状态一律用默认限制。动作之后还要读长文字，要再单独调一次 `get_app_state`。

## macOS 专用：query

已知控件的文字或角色时，用 `query` 直接定位，比读整棵树快得多，也更不容易漏。结果里 `truncated` 为 false 时，空结果才代表“没有找到这样的控件”。查询编号只在同一次会话内保留约 120 秒，动作会重新校验位置，失败就重新 `query`。

同一应用中有多个同名窗口时，已确认目标 `window_id` 就用它限定 `query`，再用返回的控件编号点击或写入。无法区分窗口时先让用户指明目标；按键和文本输入还依赖目标窗口中当前的焦点。

修改字段里的一段文字时，可用 `select_text` 按当前控件编号和完整子串选中，再输入替换文字并回读整个字段。子串重复时用 `prefix` 或 `suffix` 消除歧义。整栏替换优先用 `set_value`；用全选快捷键后要核实选区，避免意外追加。

## 点击和键盘输入的方式

- `click` 不写 `click_method` 就是 `auto`：先走无障碍语义，不行再退到事件。显式指定的方式不会回退。
- `accessibility`：只调用控件的无障碍动作，必须带 `element_index`。
- `app_post`：不经过无障碍，直接给目标窗口发鼠标事件，不会移动系统鼠标。适合必须点在空白处、或不能被重定向到子控件的点击。
- `sky_click`（仅 macOS）：目标窗口被别的窗口盖住、或 Chromium 忽略 `app_post` 时使用，不会移动鼠标、也不会把窗口提到前面。窗口移动、关闭、切换桌面、隐藏或最小化之后，要先重新 `get_app_state`。
- `global`：走全局鼠标，会真的移动鼠标、改变前台焦点，需要运行环境显式开启，除非用户明确要求，不要用。
- `type_text` 和 `press_key` 的 `key_method` 同理。`sky_key`（仅 macOS）在窗口被盖住或在别的桌面时使用：先点击或聚焦输入框，因为它输入到目标窗口当前聚焦的位置；`cmd+a`、`cmd+c`、`cmd+v` 这类组合键会走目标应用自己的菜单。

## 窗口被盖住、在别的桌面

`get_app_state` 不会主动把离屏窗口提到前面。窗口被遮挡时先读取控件并尝试无障碍操作。即使后台事件点击返回了新状态，也要核对目标控件是否生效。虚拟显示器默认关闭。仅在当前工具实际提供 `window_placement` 且用户需要临时移窗时使用 `agent_display`，并在本轮用 `restore` 还原。

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
