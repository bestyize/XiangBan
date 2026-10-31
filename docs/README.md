# 项目文档

- [desktop-user-guide.md](desktop-user-guide.md)：小伴桌面全功能用户手册与 AI 玩法指南。
- [architecture](architecture/README.md)：仓库边界、账号、AI 网关、Agent 运行与连接器规范，以及历史设计记录。
- [agent](agent/README.md)：小伴 Agent 演进规划、实现入口与建议验收清单。
- [product](product/README.md)：需求与商业规划。
- [deployment](deployment/README.md)：现有部署栈、备份与数据模型。
- [交接文档_2026-10-28.md](交接文档_2026-10-28.md)：这一轮云端会话的交接（手机桌面互联、语音编程、哪些没验证过、本地怎么起）。带日期的阶段记录，本地验证完、内容落进各 README 后可以删除。
- [交接文档_2026-10-31_社区.md](交接文档_2026-10-31_社区.md)：手机端社区的交接（界面和读接口已上线，自动采集与发布流程待做，附可并行拆分的任务块）。带日期的阶段记录，采集和发布落地并写进 `services/server-forum/README.md` 后可以删除。
- [design-preview](design-preview/README.md)：设计资源与交互预览，包含小伴桌面已确认设计稿，不是运行入口。

源码和子项目 README 说明实际状态；产品计划、设计稿和历史交接不代表已实现能力。路径、入口、协议和运行方式变化时更新相应文档。

维护时将运行步骤放在子项目 README，跨项目约定放在 architecture，需求放在 product，部署操作放在 deployment。临时交接、失效本地产物路径和已被正式说明覆盖的阶段报告不再单独保留；需要追溯时查看 Git 历史。带日期的测试记录只证明对应基线，不代表当前版本已验收。
