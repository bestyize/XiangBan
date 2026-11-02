---
name: xiaoban-frontier
description: 为小伴前沿采集并核实科技选题，使用 server-ai 撰稿和生成封面，保存本地图文并导入社区发现页待审稿。适用于小伴前沿运营，不用于普通论坛帖子。
---

# 小伴前沿运营

先读仓库 `docs/product/产品与技术方案_小伴前沿.md` 和 `services/server-forum/README.md`。严格区分事实、推测、官方声明和实际体验；优先核实原始资料，记录原文链接与时间，不能访问或无法核实的材料不当成事实。先检查近期日报、已发布文章及待审稿，避免重复；资料不足就换题或保留待核实项。生成稿须由编辑审核后发布，不自动发布。

使用本目录 `scripts/frontier.py`：提供 UTF-8 JSON 资料文件，字段为 `key`（稳定标识）、`title`、`source`、`url`（原始 HTTPS 来源）、`evidence`（核实后的文字与链接）、`image_prompt` 和可选 `illustration_prompts` 数组（最多四张正文配图）。运行 `python3 scripts/frontier.py generate 资料.json 输出目录`；脚本用 `XIAOBAN_AI_TOKEN` 访问 `XIAOBAN_AI_URL`（默认 https://ai.xiangban.xyz），调用 server-ai 的文字和图片接口。变量 `XIAOBAN_AI_TIER` 可指定档位；未设置时用默认档。密钥不得写进文件、日志或对话。真实生成可能计费；用户要求生成时已授权合理范围内的调用。

审校输出的 `article.json`、`image-1.png` 等图片 和来源，修订错误并保留证据。生图目前只支持 server-ai 的 base64 图片结果，返回 URL 时明确失败而不下载。待审图片通过站内接口保存，公开接口在文章发布并引用图片之前不可访问。不得使用第三方无权转发的图片或临时链接。

栏目更新部署且有 `frontier` 类型的发现页采集令牌后，可设置 `FORUM_DISCOVER_TOKEN`，运行 `python3 scripts/frontier.py submit 输出目录/article.json`，先上传图片再导入待审稿。`FORUM_URL` 默认 https://chat.xiangban.xyz。脚本要求新稿件版本为 0；同 key 已存在时提示人工检查，不自动覆盖。`--revision` 只在明确需要更新且已读取现有版本时使用。提交成功不代表已审核或发布。发布须在论坛管理台对具体版本人工终审；报告实际结果，不编造成功。

本机运营账号的短期 AI 令牌和受限论坛令牌分别存放在 `~/.config/xiaoban-frontier/ai-token` 与 `forum-token`，文件权限应为 600。运行脚本前将文件读入对应环境变量；访问令牌过期时运行 `python3 scripts/login.py` 重新登录。不得输出令牌内容。
