# 2026-11-03 开源精选 README 整理批次

本批按线上 revision 2 冻结：168 个唯一项目、2 个专题。`inventory.json` 保留原 key、分类、专题引用及旧目录；项目不会因整理正文重建身份，评论也不迁移。

- `manifest.json`：本批完整状态，区分 ready_for_review / blocked / pending；状态更新由分片审校结果汇总。
- `card-manifest.json`：规范化的项目卡片和来源说明；未知信息留空。
- `ready-snapshot.json`：候选稿及其源文、双语文稿、元数据、审校和许可文件的 SHA256；候选不自动等于上线。
- 项目目录：`README.source.md` 保存上游原始字节，扩展名不代表源文一定是 Markdown，真实路径/格式见 metadata；`README.zh-CN.md` 和 `README.en.md` 是可阅读的完整文稿，许可文件保持原文。
- `SPOTCHECK-2026-11-03.md`：分片交叉抽查与许可待核事项。
- `tools/`：从文件重新汇总清单的工具。`drafts/`、分块译稿和未核实正文不用于生产发布。

源文、许可和快照文件通过上级 `.gitattributes` 禁用换行归一化；否则包含 CRLF 的上游文件在 Git 入库后会失去原始 blob SHA。

已发布范围与版本以 [正式网页静态索引](../../../web/public/open-source-readmes/index.json) 及 [发布记录](../../releases/2026-11-03-readme.md) 为准；独立快照校验和发布步骤见 [采编规范](../../OPEN_SOURCE_README_WORKFLOW.md)。译文不声称实际运行过项目，不保证上游外链未来可用。上游文档自身的维护、迁移和链接缺陷在正文和项目卡片中明确保留。
