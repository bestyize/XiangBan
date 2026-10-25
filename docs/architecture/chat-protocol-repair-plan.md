# 聊天协议可靠性修复方案

> 历史修复记录：四阶段已实施；下列测试结果仅对应记录时基线，当前实现与验证入口见 [内核 README](../../libs/agent-kernel/README.md) 和 [桌面 README](../../client/xiaoban-desktop-wallpaper/README.md)。

基线：e3a81c83d667c14ad71003779997d106e3d9faa4。

| 阶段 | 实施内容 | 验收条件 |
| --- | --- | --- |
| 1 消息一致性 | 带 runId/sequence 的会话快照，缓冲并去重加载期间事件；恢复内容在会话锁内持久化，发送前补恢复；损坏日志降级 | 活跃会话历史不为空；快照期间增量不丢、不重复；停止/崩溃重开与继续使用同一份已保存内容 |
| 2 能力配置 | 摘要能力、思考模式、预算随模型草稿完整往返；旧配置兼容 | 打开、保存、重开以及拉取模型后字段不丢失；更换端点不泄漏旧能力 |
| 3 协议续接 | refusal、失败终态输出补取、phase、原始输出块有序回传；结构化推理与工具扩展状态合并；输出上限恢复与网关结构化错误 | 思考→工具→结果→下一轮可续接；失败输出只展示，不执行未完成工具；预算合法；错误可分类恢复 |
| 4 持久化与验证 | 思考增量与正文采用同样刷盘策略，终态/工具边界保持同步；针对性回归用例 | 不逐思考分片同步刷盘；覆盖完成/截断/失败/取消/损坏日志/重开/配置往返 |

## 实现约束

- 不自动重放工具、不伪造签名、不把加密状态作为思考文本展示。
- 对不一致快照、未知输出类型保留原始状态并明确边界；不把未支持扩展宣称为完整协议支持。
- 本轮不扩展原生音视频、服务端内置工具、自定义工具协议，也不改悬浮聊天框布局。
- 各阶段独立提交；只做相关检查，不新增自动 CI，不合并主分支。

## 验证

前端：会话快照与事件游标、模型配置往返的针对性测试。
内核：拒绝/终态/有序状态往返/预算修正/日志持久化测试。
服务端：输出上限错误映射测试。
真实供应商验证需要对应凭证；本地回归不等同于线上联调。

## 实施结果

四阶段代码已实现。新增会话快照协议（旧命令兼容），终态优先于运行表的短暂滞后；研究子运行不会替换主运行的快照。恢复内容在租约内写入，并在新运行开始前补恢复。

验证结果：内核单元与选定集成测试 107 项通过；产品逻辑隔离测试 13 项通过；聊天状态、展示与能力字段前端测试 93 项通过。模型设置表单通过 Vue script/template 编译检查。

可复现命令（仓库根目录）：

```sh
cargo test --locked --manifest-path libs/agent-kernel/Cargo.toml --lib --test responses --test anthropic --test wire --test middleware_retry --test retry_last --test run_journal --features openai,anthropic,journal
cargo test --locked --manifest-path tools/protocol-reliability-tests/Cargo.toml
cd client/xiaoban-desktop-wallpaper
npx vitest run src/ui/chat/chat-store.test.ts src/ui/chat/view.test.ts src/ui/model-capabilities.test.ts
```

验证边界：没有执行完整 Tauri 打包、数据库网关集成或真实供应商凭证联调。隔离测试直接编译产品源文件，不使用复制的模拟实现。
