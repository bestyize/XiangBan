# 功能地图与页面导航

让小伴 AI 知道自己有哪些功能、在软件的哪个页面，并能把用户带过去。

- `app_guide(topic?)`：返回功能清单。不填 topic 是完整目录；填功能名、页面名、工具名或关键词（如“置顶”“代理”“资料库”）返回详情，含：能做什么、在控制台哪里、对应工具、哪些要用户自己在页面里操作。搜不到时返回完整目录，不会直接得出“不支持”。
- `open_page(page, tab?)`：把控制台带到前台并打开页面。`page`：`new-task`、`ai-store`、`library`、`studio`、`games`、`care`、`settings`；`studio` 的 `tab`：`wallpapers`、`effects`、`widgets`、`pets`；`settings` 的 `tab`：`general`、`appearance`、`ai`、`assistant`、`voice`、`network`、`safety`、`about`。AI 插件和资料库的内部标签（专家/技能/连接器、我的文档/云文档/收藏/记忆）暂不能直接跳转，打开页面后由用户点标签。
- 系统提示词要求：回答“不支持”之前先 `app_guide`；对话里做不了的事（代理、API 密钥、权限、插件安装等）用 `open_page` 带用户去。

## 维护

功能清单的唯一来源是 `src-tauri/src/chat/app_guide.rs` 的 `FEATURES`。新增工具、控制台页面、装扮标签或设置分类时，同一次改动里补一行。下列测试会在遗漏时失败：

- `desktop_tools::app_guide_covers_every_tool_so_the_model_can_find_each_one`：每个桌面工具都要出现在清单里。
- `app_guide::every_console_page_and_settings_category_is_in_the_guide`：读取 `src/ui/control-types.ts`，页面、装扮标签、设置分类缺一不可（`chat` 与无侧栏入口的 `interactive` 除外）。

前端由 `ControlShell.vue` 监听 `control-navigate` 事件完成跳转。

## 按请求自动提示（不再靠模型自己想起来查）

`chat/feature_hints.rs` 在每次调用模型前，用用户最新一句话去匹配功能地图（`app_guide.rs` 里的 `ALIASES`：每项功能的常见说法），命中的功能（最多 3 项）会作为一条临时提示紧跟在这句话后面：能做什么、对应工具、怎么打开页面、哪些要用户自己操作。提示只加在本次请求里，不写进聊天记录。普通闲聊不命中，不增加开销。

**以后小伴又对某件事说“不支持”时的处理办法**：把用户的原话加进 `app_guide.rs` 测试 `requests_reach_the_features_they_are_about` 的表里，让它失败，再给对应功能的 `ALIASES` 补上缺的词，测试变绿即可。不要再去改提示词。

## 提醒到点是怎么提醒的（`reminders.rs`）

- 到点弹出「小伴关怀」气泡：屏幕右上角（前台窗口或鼠标所在的屏幕），始终置顶、不抢焦点。有「关闭提醒」和「稍后提醒」（10 分钟后再弹）两个按钮。
- **一次性提醒**（`repeat=once` 的普通提醒）：气泡一直留着，直到点掉或稍后提醒；到点后 12 小时内都会补发（屏幕锁定、应用刚打开等）。后面又有提醒到期时，会把它们合并进同一个气泡，不会被挡住。
- 重复提醒、下班提醒等：气泡约 10 秒（开了语音播报是 30 秒），补发窗口 30 分钟。
- **一次性提醒默认带提示音**（「提醒」页的「一次性提醒带提示音」，默认开，即使总的「提示音」关着；Windows 是系统哔声，macOS 是 Pop 音）。其他提醒的提示音和语音播报默认关，在「提醒」页打开；没有系统通知。
- 总开关关闭、安静时段（默认关）、屏幕锁定、玩游戏、专注模式时先压住，解除后补弹。小伴需要在运行。
- `care_set` 创建或修改提醒后，返回内容会按实际设置写明时间和提醒方式（`how_it_reminds`），小伴据此如实告诉用户。
