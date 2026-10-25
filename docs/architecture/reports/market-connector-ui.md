# U2：市场连接器插件安装

改动仅在 `client/xiaoban-desktop-wallpaper/src/ui` 和本汇报。未改 src-tauri、未运行 cargo、未执行 git add / git commit / push；文件留在当前 worktree，交给 Claude 提交。

## 已完成

- 市场连接器按 `connector` 对象分流。新卡片优先显示中文名称/描述，认证文案复用 `connectorAuth`，展示随包技能数；无 `connector` 的旧条目保留 `ConnectorConfig.vue` 填表、确认连接和 OAuth 流程。
- 新增 `MarketConnectorDetail.vue`：展示中英文回退的说明、示例提问、令牌字段名称及必填/选填、传输、策略禁用工具数和随包技能。字段摘要不渲染值、占位符或配置输入框。
- “安装/更新”先调用 `ai_market_inspect`，沿用现有市场弹窗、文件清单、脚本预览、确认勾选和版本摘要锁定风格；审查展示 MCP 名称、传输、地址、将运行的命令、环境变量名/请求头名、两处 warnings 去重、技能、文件和脚本（含截断/无预览文案）。勾选后调用 `ai_market_install`，使用审查版本与 sha256；缺摘要、版本不一致、状态未读完或读取失败时不会安装。
- 列表或审查结果出现 `sse` 均展示“旧式 SSE 暂不支持，暂时无法连接”，禁用安装。安装本身不调用任何配置/连接命令。
- 通过 `ai_connector_installed` 的 `plugin === slug` 判断已安装，多 MCP 服务按同一插件匹配。更新复用技能页现有 `ai_market_updates` 结果，通过 AiStorePage 传入，匹配 kind、namespace、slug、hasUpdate 后显示“更新”；没有更新信息时只显示“已安装”，未新增本地版本比较规则。
- 安装成功保留详情，提示“已安装，下一步到连接器中心填写令牌并连接”，提供“去连接器中心”。AiStorePage 清除搜索/分类筛选，ConnectorCenter 根据 plugin 打开对应服务管理弹窗和空令牌表单；中心刷新未完成时会等待结果再展开。安装后刷新插件、随包技能和更新信息。网页唤起连接器详情时也保留 connector 和固定版本数据。
- 审查失败/安装失败可重试；关闭弹窗后的迟到结果不会复用；更新目标版本变化会撤销旧审查及确认。卡片与弹窗使用现有样式，插件卡片元数据允许换行。

## 验证

在 `client/xiaoban-desktop-wallpaper` 执行：

```powershell
npx.cmd vitest run --configLoader runner src/ui/ai-store/RegistryMarketConnector.test.ts src/ui/ai-store/RegistryCatalog.test.ts src/ui/pages/AiStoreMarket.test.ts src/ui/pages/AiStorePage.test.ts src/ui/ai-store/ConnectorCenter.test.ts
npx.cmd vue-tsc --noEmit
```

结果：5 个测试文件、59 个测试通过；类型检查通过。新增 `RegistryMarketConnector.test.ts` 的 19 个测试覆盖插件摘要与值隐藏、英文回退、完整审查与确认安装、同插件多服务识别和更新、SSE 双层拦截、失败重试、缺摘要/错版本/缺版本拒绝、安装状态等待/重试、旧市场连接回归、关闭后的迟到审查、更新撤销确认、跳转清除搜索并打开表单、中心加载中的展开请求。invoke 全部 mock，无真实服务和凭据调用。`git diff --check` 通过。

## 未做与 Claude 范围外接线

- 本包前端需求已完成；没有运行真实 Tauri/WebView，也没有真实下载、落盘、升级或第三方账号连接联调。旧式 SSE 仍按库的现状禁用，没有补传输实现。
- 按本次约定调用已有 `ai_market_detail/inspect/install`，无需新增命令。Claude 合并时确认 connectors 的审查结果提供 `manifest.version/connectorCount/servers/skills`、sha256、warnings 和 scripts；servers 的 command 应包含完整待运行命令（也支持单独 args），仅返回 envKeys/headerKeys 名称供审查。
- `ai_connector_installed` 必须为每个服务提供对应市场 slug 的 plugin 字段，以及现有 ConnectorCenter 所需的 auth/schema/status 等字段。新增的 TypeScript plugin 字段保留可选，以兼容既有测试/旧数据；没有 plugin 时不会猜测 id 来匹配。
- “更新”依赖现有 `ai_market_updates` 返回连接器的 `hasUpdate/latest/kind/namespace/slug`；后端若尚未提供连接器更新结果，界面按要求只显示“已安装”。本包按授权命令统一使用 `ai_market_install` 安装审查过的新版本，实际替换、备份和原生端策略仍由既有实现负责，需在主目录联调确认。
- 去连接器中心会展开同插件返回的第一个服务；多服务仍由现有中心各自管理。表单提交、凭据加密、OAuth、连接及状态刷新继续使用上一轮中心与原生命令，没有在市场详情收集令牌。
