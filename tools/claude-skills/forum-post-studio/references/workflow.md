# 格式与调用

## 安装与配置

把整个 `forum-post-studio` 文件夹复制到 `~/.claude/skills/`，或项目的 `.claude/skills/`。

```sh
python -m pip install -r <skill>/scripts/requirements.txt
```

在论坛 `/admin/editor-accounts` 创建或绑定编辑账号，生成专用创作令牌。默认只有导入权限；需要让 Claude 发布时勾选“允许本地直接发布”。将令牌通过本地环境变量管理，禁止写进 Skill、帖子或版本库。`FORUM_URL` 默认为 `https://chat.xiangban.xyz`，`FORUM_EDITOR_TOKEN` 必填。修改站点时使用最终 HTTPS 根地址。令牌不会发到重定向地址。

## 文件格式

每篇 Markdown 顶部使用以下元信息；`category` 必须使用 `context` 中的实际 slug。只支持这四个字段，避免正文元信息被静默忽略。

```markdown
---
id: ai-workflow-001
title: 把重复工作整理成可复用的 AI 工作流
category: "实际板块 slug"
tags:
  - AI提效
  - 工作流
---

这里是正文。

## 操作步骤

![工作流示意](images/workflow.png)
```

`id` 为固定且稳定的稿件编号，1–100 位字母、数字、点、下划线或连字符，同作者下唯一。不要在重试时换 id。标题 4–80 字，普通正文 10–20000 字；最多 5 个标签，每个最多 20 字。

配图相对于该 Markdown 定位，可用行内或引用式图片语法。文件名含空格时用 `![图](<images/my image.png>)`；不要给路径做 URL 编码，不支持 `../`。图片必须是 PNG/JPEG/GIF/WebP，默认每张最多 5 MB，每篇最多 20 张、合计 20 MB。每批 1–50 篇；ZIP 最大 100 MB，解压后总大小最大 80 MB，最多 500 个内容文件。目录只放 `.md` 与配图，选题表、导入报告和来源笔记放在外面。来源链接可直接放在文章末尾。

## 常用命令

```sh
python <skill>/scripts/forum_posts.py context
python <skill>/scripts/forum_posts.py recent --category <slug>
python <skill>/scripts/forum_posts.py validate ./posts
python <skill>/scripts/forum_posts.py pack ./posts --output ./batch-01.zip
python <skill>/scripts/forum_posts.py import ./posts --report ./batch-01-report.json
python <skill>/scripts/forum_posts.py show <稿件UUID> --output ./preview.html
python <skill>/scripts/forum_posts.py publish --report ./batch-01-report.json
python <skill>/scripts/forum_posts.py publish --id <稿件UUID>
```

`validate` 是离线格式检查，缺图和图片真实类型等由导入接口再次校验。`import` 对每篇独立报告结果，部分失败时退出码为 1，成功项保留。`publish` 重试原 ID 不会多发一篇。`--overwrite` 只更新待发布稿，不能修改已发布帖子；已发布内容通过论坛原来的编辑入口维护。

未发布稿件和配图由受保护的接口返回，公开列表和搜索不会出现。后台的同一套渲染器用于预览和正式帖子。编辑正文后保存才能更新预览；发布的是保存的版本。

## 示例指令

- “查看论坛板块，给帖子较少的 AI 板块准备 5 篇教程，整理图片并导入后台待发布列表。”
- “创作 3 篇具体的软件使用技巧，输出 Markdown 和配图，打成 ZIP，先不要发布。”
- “发布这批已审核的稿件，失败项保留，返回帖子链接。”
