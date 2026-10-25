# U：桌面前端（connector-ui）

分支：`codex/connector-ui`。改动限于 `client/xiaoban-desktop-wallpaper/src/ui` 及本汇报；没有修改、构建 `src-tauri`，没有运行 cargo、推送或修改其他分支。

## 提交阻塞

功能代码及下述验证已完成，**尚未完成 git commit**。执行 `git add` / `git commit` 均失败：

```text
fatal: Unable to create 'C:/Users/sdjz0/Documents/Claude/XiangBan/.git/worktrees/ui/index.lock': Permission denied
```

只读检查发现 worktree Git 元数据目录的 ACL 有显式写入拒绝；当前沙箱不能提权，未改 ACL 或绕过权限。文件仍保存在当前 worktree，尚未暂存。需要在有写入权限的环境中暂存本包文件及本汇报，再执行 `git commit -m "feat(desktop): add connector plugin center and structured mentions"`；无需重跑已通过的检查，也不要推送。

## 已完成

- 新增 `ConnectorCenter.vue` 和接口类型：AI 插件页连接器标签内展示已安装连接器的中文状态灯、版本、认证方式、工具数；管理弹窗提供工具说明、随包技能、示例提问、启用/停用、二次确认断开并删除凭据、OAuth 重新授权。示例切换至新任务并填入输入框，不自动发送。保留旧 `RegistryCatalog`，旧 MCP 配置仍使用 `ConnectorConfig.vue`。
- 新增 `ConnectorTokenForm.vue`：严格使用 `auth.schema`，支持密码/文本、必填、占位及说明；已保存字段只标记“已保存”，输入始终为空；用户勾选确认后调用 `ai_connector_configure`，编辑字段会撤销确认。提交后立即清除输入值，失败也不缓存凭据。
- OAuth 复用 `ai_store_connector_auth_status/auth_cancel` 和 `open_download_url`；支持轮询、手动检查、重新打开页面、取消，以及卸载组件时停止计时并尝试取消授权。
- 新增 `ChatMentionMenu.vue`、`chat/mentions.ts`：输入末尾 `@` 搜索 `ai_mentions`，支持中文、鼠标、上下键、Enter/Tab、Escape 和输入法保护。专家通过既有 `useExpert` 召唤，技能写为 `/技能id`；连接器显示可移除、去重的优先使用标签。`＋` 中旧连接器选择也改为结构化标签。
- `chat.send` 保留原有前三个参数，第四参数为 `mentions`；仅把 `{kind,id}` 放入 `chat_send` 参数，不往正文加连接器提示词。发送失败恢复正文及标签，切换会话清除标签。
- 新增 `LocalPluginInstaller.vue`：页面顶部安装入口、ZIP 选择、原生拖入路径及完整路径输入；审查展示类型、名称、版本、技能/专家数、连接器信息和警告，经勾选确认才安装。保留当前 Rust 实现要求的清单摘要锁定；安装后刷新列表。
- `ChatComposer.vue` / `AiStorePage.vue` 仅增加入口及必要发送、回调接线；主要实现集中在新文件，便于合并专家团修改。

## 保守解释与限制

1. **原生文件选择器还需接线**：文档建议复用附件的 `<input type="file">`，但 WebView 的标准 File API 不提供绝对路径，而现有审查命令只接受 `path`。当前 capability 也未开放 dialog 前端调用，因此没有假造路径、修改权限或另造 Rust 命令。已提供相同 HTML ZIP 选择器；宿主若提供 `File.path` 则直接审查，否则明确提示拖入 ZIP 或填写完整路径。原生 `tauri://drag-drop` 可提供路径，已 mock 测试；完整“点选即审查”仍需要 Claude 补原生路径选择入口。
2. **本地安装参数差异**：架构文档简写为 `(path, confirmed)`，当前 `chat/plugins.rs` 实际要求 `(path, sha256)`；前端发送 `{path, sha256: review.manifestSha256, confirmed:true}`，保留摘要校验。缺少摘要时阻止安装，不降级绕过验证。
3. **审查计数**：当前审查响应仅有 `skills[]`、`agents[]` 和 `hasMcp`，没有实际 MCP 服务数。优先展示可选的 `skillCount/agentCount/connectorCount`；缺连接器计数时展示“包含（数量待连接时确认）”，没有将 `hasMcp` 猜成 1 个服务。Claude 可补 `connectorCount` 使精确数量自动生效。技能/专家缺精确计数时按现有数组计数。
4. **已存字段更新**：留空的已存字段不发送，意味着保留原值；不把空串解释为删除凭据。删除使用单独的 unbind。没有语言设置，schema 使用中文字段。
5. **专家 ID**：`ai_mentions` 的专家 id 按已安装专家的 `slug` 或 `plugin` 精确匹配，再调用现有 `useExpert`；无法匹配时报错，避免构造错误的专家身份。后端应保证 id 与其一一致。
6. **缺表单与不支持项**：token/env 缺 schema 不猜 JSON 占位符，提示更新插件；`unsupported` 禁止连接/启用。CLI 仅提示先登录再启用，未实现服务专属登录。旧市场 MCP 与插件安装中心并列展示，分类只作用于旧市场目录，中心按关键字筛选。

## 验证

在 `client/xiaoban-desktop-wallpaper` 执行：

```powershell
npx.cmd vitest run --configLoader runner src/ui/ai-store/ConnectorCenter.test.ts src/ui/ai-store/LocalPluginInstaller.test.ts src/ui/components/ChatConnectorMentions.test.ts src/ui/chat/chat-store.test.ts src/ui/pages/AiStorePage.test.ts src/ui/pages/AiStoreMarket.test.ts src/ui/pages/ChatPage.test.ts src/ui/components/ChatWorkbenchTools.test.ts src/ui/ai-store/RegistryCatalog.test.ts
npx.cmd vue-tsc --noEmit
```

结果：9 个测试文件、190 个测试通过；类型检查通过。新增 22 个测试覆盖中心、表单、OAuth、安装审查与摘要、拖入及路径限制、`@` 三类选择、结构化发送与恢复。所有新增命令均 mock `invoke`，没有调用真实服务或凭据。

PowerShell 禁止执行 `npx.ps1`，故使用同安装目录的 `npx.cmd`。默认 Vitest 配置加载会尝试写链接到主目录的 `node_modules/.vite-temp`，被沙箱拒绝；使用 Vite 已有的 `--configLoader runner` 避免该写入，不改依赖和测试配置。

## Claude 范围外接线

- 注册并实现文档的 `ai_connector_*`、`ai_mentions`，对接 L 包保险箱/状态/策略；接口返回值遵循 `connector-plugins.ts`。`ai_connector_configure` 对省略的已存字段保留原值，对用户确认及未知字段仍由原生端校验。
- `chat_send` 接收 `mentions:[{kind,id}]` 并交给内核运行时，不应通过修改可见消息正文实现。重试请求目前保持既有 `retry:true` 协议，引用的持久化/重试恢复由 Rust 保存原始请求信息。
- 补原生 ZIP 路径选择入口及审查的精确计数字段，确认本地安装继续按 `sha256` 锁定版本。
- 合并时注意 `ChatComposer.vue` 的专家类型/`useExpert` 和 `chat-store.ts` 第四参数与专家团新增参数的兼容性；本工作包未改专家团协议。
- 真实 WebView、文件对话框、OAuth 浏览器跳转、保险箱落盘和 SSE 连接需合并后联调；本包没有启动桌面程序，也没有进行这些实机验证。
