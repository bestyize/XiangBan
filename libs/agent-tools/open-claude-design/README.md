# open-claude-design

小伴内置设计工具的独立模块，依赖 agent-kernel，内核不反向依赖本模块。

## 已实现

- `open-claude-design` 工具：格式说明、会话作品列表、读取、新建、带版本检查的保存。
- 关键词“设计稿”触发的请求上下文；已有作品可以继续修改，无需重复关键词。
- 一份设计稿包含多个 DC HTML 画板；读取 v3 `boards/order` 与旧版 `artboards` 索引，保留未知字段。
- 单个 JSON bundle 原子保存源文件与索引，拒绝过期版本覆盖，并保留修改前的版本。
- 独立实现的 DC HTML 预览运行时子集，详见 FORMAT.md。未使用 Anthropic 的 support.js。

## 桌面宿主接入约定

作品属于「资料库 → 我的文档」：每份项目只有一条文档记录，多个画板不拆成文档。
Store 根目录传入 `<data root>/data-library/files`，每份项目的主文件为 `<id>/project.json`。
聊天画布和资料库必须通过同一个 Store 读取、保存；使用稳定项目 id，不能另存预览副本冒充源文件。
宿主通过 DesignTool 的保存回调更新资料库索引并发送刷新事件。资料库的分组、重命名与删除需要由宿主适配。

本模块目前是核心实现；桌面端的工具注册、资料库索引适配、画布界面与受限预览协议尚未接入。
运行时必须放在无 `allow-same-origin` 的 sandbox iframe 中，并由独立响应设置 CSP；不能在有桌面权限的页面执行。
运行时只实现已记录的语法子集，不保证任意 Claude Design 文件无损往返；外部 React/DC 组件导入尚不支持。

## 验证

```sh
cargo test --manifest-path libs/agent-tools/open-claude-design/Cargo.toml
```

测试覆盖新旧索引、路径与文件校验、会话隔离、版本冲突、重新打开和关键词触发。
