> ✨ 特别赞助商：[CodeRabbit — 业界领先的 AI 代码审查](https://coderabbit.link/usememos).

<h1 id="memos">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/usememos/.github/refs/heads/main/assets/brand/lockup-on-dark.svg" />
    <img src="https://raw.githubusercontent.com/usememos/.github/refs/heads/main/assets/brand/lockup.svg" alt="Memos" width="220" />
  </picture>
</h1>

### 简单记录日常想法。

Memos 是一款开源、自托管的笔记应用，可将日常想法汇集到个人时间线中。数据由你掌握，你可以选择哪些内容保密或分享。

[演示](https://demo.usememos.com) · [安装](#快速开始) · [文档](https://usememos.com/docs) · [Discord](https://discord.gg/tfPJa4UmAv)

[![Docker pulls](https://img.shields.io/docker/pulls/neosmemo/memos?style=flat-square&logo=docker)](https://hub.docker.com/r/neosmemo/memos)
[![Latest release](https://img.shields.io/github/v/release/usememos/memos?style=flat-square&label=Release)](https://github.com/usememos/memos/releases)

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/usememos/.github/refs/heads/main/assets/previews/timeline-dark.png" />
  <source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/usememos/.github/refs/heads/main/assets/previews/timeline-light.png" />
  <img src="https://raw.githubusercontent.com/usememos/.github/refs/heads/main/assets/previews/timeline-light.png" alt="Memos 时间线，包含笔记编辑器和最近的笔记" />
</picture>

## 关于 Memos

Memos 是一款用于记录日常想法的开源、自托管笔记应用。

- **日常记录。** 一句话就可以是一条完整的笔记，也可以慢慢写下更长的内容，并使用 Markdown 为文字增加结构。
- **个人时间线。** 每条笔记都保留在写下它的那一天，将想法汇集成一条可供回顾的时间线。标签将不同日期的相关想法联系起来，方便你重新拾起某个话题或找到记得写过的内容。
- **自己的数据。** 记录保存在你自己的服务器上，数据由你掌握。
- **自主控制分享。** 你可以选择哪些内容保密、哪些分享给家人和朋友、哪些发布到网上。在共享空间中，你可以与他人一起记录，同时各自保留自己的笔记。

## 快速开始

安装 Docker 后运行：

```bash
docker run -d \
  --name memos \
  -p 5230:5230 \
  -v ~/.memos:/var/opt/memos \
  neosmemo/memos:stable
```

打开 http://localhost:5230 并创建第一个账户。Memos 保存的所有内容都位于 `~/.memos`，因此请备份这个文件夹。

包括 Docker Compose、Kubernetes 和直接运行二进制文件在内的其他运行方式，请参阅[部署指南](https://usememos.com/docs/deploy)。

## 升级

发布版本的标签格式为 `YY.MM.N`，候选版本为 `YY.MM.N-rc.K`，均无 `v` 前缀。Docker 标签 `stable` 跟随稳定版本，`26.10` 等月份标签跟随当月最新的稳定版本，`canary` 跟随开发构建版本。如果从 v0.31.0 之前的版本升级，请先运行一次 [v0.31.0](https://github.com/usememos/memos/releases/tag/v0.31.0)；请参阅[升级要求](https://github.com/usememos/memos/blob/b40d8a7b573774c52b022c8d6537d6baf2129dc0/store/migration/README.md).

## Memos 周边工具

- [Web Clipper](https://usememos.com/web-clipper) 提供 [Chrome](https://chromewebstore.google.com/detail/memos-web-clipper/nebaoebnljalfegiidibihhkebeiklbl) 和 [Firefox](https://addons.mozilla.org/en-US/firefox/addon/memos-web-clipper/) 版本，可将网页、选中的内容和图片以 Markdown 格式保存到笔记，并附上来源链接。
- [Memogram](https://github.com/usememos/telegram-integration) 是一个 Telegram 机器人，可将消息和图片保存到 Memos。
- 通过 [API](https://usememos.com/docs/api/latest) 和 [MCP 服务器](https://usememos.com/docs/integrations/mcp)，可将 Memos 连接到自己的脚本和 AI 工具。

## 赞助商

<p>
  <a href="https://coderabbit.link/usememos" target="_blank" rel="noopener"><picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/usememos/.github/refs/heads/main/assets/sponsors/coderabbit/white-typemark.svg" /><img src="https://raw.githubusercontent.com/usememos/.github/refs/heads/main/assets/sponsors/coderabbit/orange-typemark.svg" alt="CodeRabbit" height="40" align="middle" /></picture></a>
  &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;
  <a href="https://www.testmuai.com/?utm_medium=sponsor&utm_source=memos" target="_blank" rel="noopener"><picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/usememos/.github/refs/heads/main/assets/sponsors/testmuai/white.png" /><img src="https://raw.githubusercontent.com/usememos/.github/refs/heads/main/assets/sponsors/testmuai/black.png" alt="TestMu AI" height="30" align="middle" /></picture></a>
</p>

通过 [GitHub Sponsors](https://github.com/sponsors/usememos) 赞助 Memos。

## 帮助与贡献

如有问题，请查阅[文档](https://usememos.com/docs)，或前往 [Discord](https://discord.gg/tfPJa4UmAv) 或 [GitHub Discussions](https://github.com/usememos/memos/discussions)。错误报告和想法请提交到 [issues](https://github.com/usememos/memos/issues/new/choose)。参与代码修改前，请先阅读[贡献指南](https://usememos.com/docs/development/contributing)。

## Star 历史

<p>
  <a href="https://www.star-history.com/usememos/memos"><picture><source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/badge?repo=usememos/memos&type=rank&theme=dark" /><source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/badge?repo=usememos/memos&type=rank" /><img alt="Star History Rank" src="https://api.star-history.com/badge?repo=usememos/memos&type=rank" /></picture></a>
  <a href="https://www.star-history.com/usememos/memos"><picture><source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/badge?repo=usememos/memos&type=trending&theme=dark" /><source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/badge?repo=usememos/memos&type=trending" /><img alt="GitHub Trending Repository of the Day" src="https://api.star-history.com/badge?repo=usememos/memos&type=trending" /></picture></a>
</p>

## 许可证

Memos 根据 [MIT 许可证](https://github.com/usememos/memos/blob/b40d8a7b573774c52b022c8d6537d6baf2129dc0/LICENSE) 开源。
