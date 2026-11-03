# 开源精选详情页素材

- `export-dialog.png`、`reopened-edit.png`：来自 open-source-curation 工作树的 Excalidraw 采编样稿，官方白板实操截图，展示 PNG 导出及重新编辑。
- `github-home.png`：2026-11-03 截取 https://github.com/excalidraw/excalidraw ，只作为缺少配图时的静态展示素材，图内数据不代表实时状态。
- 详情页默认展示中文 README，可切换已保存的上游英文正文。旧导读截图仅保留为素材，不再用于当前页面。排版源文件为相邻 `open-source-detail.html`。

预览：http://127.0.0.1:1423/prototypes/open-source-detail.html 。项目链接通往真实官网与仓库；点赞、收藏和评论保存在浏览器本地，仅用于原型演示。未接入生产社区。

## README 中文阅读原型

- 原文：https://github.com/excalidraw/excalidraw/blob/master/README.md
- 获取与翻译日期：2026-11-03；README 内容 blob SHA：`48b9d1de9fbb1c74406407e89e2471a01b1f28de`。这不是提交 SHA 或软件版本号。
- `upstream-README.md` 保存此次原文，`excalidraw-LICENSE.txt` 保留上游 MIT 许可全文和版权声明。
- `upstream-showcase.png` 来自原文引用的 https://excalidraw.nyc3.cdn.digitaloceanspaces.com/github%2Fproduct_showcase.png ，用于中文页的官方产品展示。
- 非官方正文译文直接维护在 HTML 的 `panel-readme`；并列功能适当合并，动态徽章和赞助图墙由原文链接承接，注明原文计划事项。当前原型已移除导读，英文页正文来自保存的原文。
- 中文 / English 切换支持键盘左右键，桌面目录随语言切换，两种语言均支持折叠目录、图片放大和命令复制；评论区共用。

项目卡片数据：2026-11-03 从 `https://api.github.com/repos/excalidraw/excalidraw` 获取，`stargazers_count=135057`，展示为 135.1k；更新日期采用 `pushed_at=2026-11-02T16:26:43Z`，按北京时间展示 2026-11-03。静态原型不实时请求 API，悬停可查看完整数量与日期含义。

- `excalidraw-icon.png`：复用开源精选已采集的项目图标，来源为 https://excalidraw.com/apple-touch-icon.png ，替代卡片内的占位字符。

评论与底栏改为小红书式阅读交互：头像 / 昵称 / 正文 / 右侧点赞、展开回复、底部输入面板，以及贴边输入胶囊与赞 / 收藏 / 评论。包含明确标注的示例评论；新增评论、回复和点赞仅保存在浏览器，兼容旧原型留言，不接生产社区。

底部互动栏进一步按用户提供的 750px 宽参考图比例对齐：约三分之一宽的浅灰输入胶囊、无铅笔、统一空心图标与紧邻数量、三点评论气泡，适配 320–430px 与系统安全区；零互动显示 0，不虚构实际点赞与收藏数量。
