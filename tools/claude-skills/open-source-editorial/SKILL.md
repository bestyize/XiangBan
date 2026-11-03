---
name: open-source-editorial
description: 为小伴社区开源精选核实项目资料，整理中文 README 与英文原文、项目卡片和配图，并批量改稿、审校和交付待审成果。
---

# 开源精选采编

本栏目采用 README 阅读页：默认中文，可切英文；不再写故事式导读或标题党。用户明确要求独立评测或专题文章时，再按当次任务组织内容。

在 XiangBan 仓库中先读根 README、`services/server-forum/README.md`，然后读取 `services/server-forum/editorial/OPEN_SOURCE_README_WORKFLOW.md`。该文档是正文、来源、项目卡片、图片、批量清单和接入的规范来源。技能不授予发布或生产部署权限。

- 保留官方原文和版本，优先核对可用的官方中文；无官方中文时忠实翻译全文并标注非官方译文。命令、代码、配置键、版本和原有限制不改，不虚构体验或抹掉来源。
- 项目卡片使用真实 Logo、普通人能看懂的一句话用途、合适的官方入口、GitHub、Stars、代码更新日期、许可和平台；图片优先官方 README，找不到时可用真实 GitHub 首页截图。
- 批量任务先冻结完整线上目录及 revision，再按 owner/repo 处理，保留原 key、分类和专题引用。不能仅处理旧种子目录却宣称全量完成。
- 每项交付原文、中文、英文、元数据、许可和审校记录；全量清单列出完成、阻塞和待处理，未完成不算完成。README 中的命令只展示，不执行。
- 先检查当前接口能否承接双语全文。无法承接就保留文件成果并说明缺口，不压缩为摘要、不直接改库。后台草稿与发布按用户已授权范围操作，不把稿件整理视为发布指令。

交互原型：`client/xiaoban-multiplatform/docs/design-specs/prototypes/open-source-detail.html`。代码与设计、内容整理和生产发布分别报告状态。
