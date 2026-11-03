> [!NOTE]
> 想进一步了解 Mastodon？
> 点击下方视频了解更多。

<p align="center">
  <a style="text-decoration:none" href="https://www.youtube.com/watch?v=IPSbNdBmWKE">
    <img alt="Mastodon hero image" src="https://raw.githubusercontent.com/mastodon/mastodon/5ccbf0e488801f9b7ee5fbe9d915f861081fa3ca/docs/hero-nodes.gif" />
  </a>
</p>

<p align="center">
  <a style="text-decoration:none" href="https://github.com/mastodon/mastodon/releases">
    <img src="https://img.shields.io/github/release/mastodon/mastodon.svg" alt="Release" /></a>
  <a style="text-decoration:none" href="https://github.com/mastodon/mastodon/actions/workflows/test-ruby.yml">
    <img src="https://github.com/mastodon/mastodon/actions/workflows/test-ruby.yml/badge.svg" alt="Ruby Testing" /></a>
  <a style="text-decoration:none" href="https://crowdin.com/project/mastodon">
    <img src="https://d322cqt584bo4o.cloudfront.net/mastodon/localized.svg" alt="Crowdin" /></a>
  <a href="https://digitalpublicgoods.net/r/mastodon"><img src="https://img.shields.io/badge/Verified-DPG-3333AB?logo=data:image/svg%2bxml;base64,PHN2ZyB3aWR0aD0iMzEiIGhlaWdodD0iMzMiIHZpZXdCb3g9IjAgMCAzMSAzMyIgZmlsbD0ibm9uZSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj4KPHBhdGggZD0iTTE0LjIwMDggMjEuMzY3OEwxMC4xNzM2IDE4LjAxMjRMMTEuNTIxOSAxNi40MDAzTDEzLjk5MjggMTguNDU5TDE5LjYyNjkgMTIuMjExMUwyMS4xOTA5IDEzLjYxNkwxNC4yMDA4IDIxLjM2NzhaTTI0LjYyNDEgOS4zNTEyN0wyNC44MDcxIDMuMDcyOTdMMTguODgxIDUuMTg2NjJMMTUuMzMxNCAtMi4zMzA4MmUtMDVMMTEuNzgyMSA1LjE4NjYyTDUuODU2MDEgMy4wNzI5N0w2LjAzOTA2IDkuMzUxMjdMMCAxMS4xMTc3TDMuODQ1MjEgMTYuMDg5NUwwIDIxLjA2MTJMNi4wMzkwNiAyMi44Mjc3TDUuODU2MDEgMjkuMTA2TDExLjc4MjEgMjYuOTkyM0wxNS4zMzE0IDMyLjE3OUwxOC44ODEgMjYuOTkyM0wyNC44MDcxIDI5LjEwNkwyNC42MjQxIDIyLjgyNzdMMzAuNjYzMSAyMS4wNjEyTDI2LjgxNzYgMTYuMDg5NUwzMC42NjMxIDExLjExNzdMMjQuNjI0MSA5LjM1MTI3WiIgZmlsbD0id2hpdGUiLz4KPC9zdmc+Cg==" alt="DPG Badge"/></a>
</p>

Mastodon 是基于 [ActivityPub](https://www.w3.org/TR/activitypub/)的**免费开源社交网络服务器**，用户可以关注朋友并发现新朋友。在 Mastodon 上，用户可以发布链接、图片、文字和视频等内容。所有 Mastodon 服务器以联邦网络形式互通：一台服务器上的用户可以与另一台服务器上的用户交流，包括使用其他实现 ActivityPub 的软件的用户！

## 导航

- [项目主页 🐘](https://joinmastodon.org)
- [捐赠支持开发 🎁](https://joinmastodon.org/sponsors#donate)
  - [查看赞助商](https://joinmastodon.org/sponsors)
- [博客 📰](https://blog.joinmastodon.org)
- [文档 📚](https://docs.joinmastodon.org)
- [官方容器镜像 🚢](https://github.com/mastodon/mastodon/pkgs/container/mastodon)

## 功能

<img src="https://raw.githubusercontent.com/mastodon/mastodon/5ccbf0e488801f9b7ee5fbe9d915f861081fa3ca/app/javascript/images/elephant_ui_working.svg" align="right" width="30%" />

**联邦宇宙的一部分。基于开放标准，不受供应商锁定。**——网络不止包括 Mastodon；任何实现 ActivityPub 的服务都属于更广泛的社交网络，即[联邦宇宙](https://jointhefediverse.net/). 你可以关注其他服务器上的用户并与其互动（包括运行不同软件的服务器），他们也可以关注你。

**实时、按时间顺序更新的时间线**——你关注的人的动态会实时显示在界面上。

**媒体附件**——上传并查看动态附带的图片和视频。没有音轨的视频按动画 GIF 处理；普通视频持续循环播放。

**安全与管理工具**——Mastodon 包含私密帖子、锁定账户、词句过滤、静音、屏蔽等功能，以及举报和管理系统。

**OAuth2 与易用的 REST API**——Mastodon 可作为 OAuth2 提供方，第三方应用可以使用 REST 和 Streaming API，由此形成[丰富的应用生态](https://joinmastodon.org/apps)，提供多种选择！

## 部署

### 技术栈

- [Ruby on Rails](https://github.com/rails/rails)为 REST API 和其他网页提供支持。
- [PostgreSQL](https://www.postgresql.org/)是主要数据库。
- [Redis](https://redis.io/)和 [Sidekiq](https://sidekiq.org/)用于缓存和队列。
- [Node.js](https://nodejs.org/)为 Streaming API 提供支持。
- [React.js](https://reactjs.org/)和 [Redux](https://redux.js.org/)用于界面的动态部分。
- [BrowserStack](https://www.browserstack.com/)支持在真实设备和浏览器上测试。（本项目使用 BrowserStack 测试）
- [Chromatic](https://www.chromatic.com/)提供视觉回归测试。（本项目使用 Chromatic 测试）

### 要求

- **Ruby** 4.0
- **PostgreSQL** 14+
- **Redis** 7.0+
- **Node.js** 24+
- **FFmpeg** 5.1+

此仓库包含 **Docker 和 docker-compose** 以及 Heroku、Scalingo 等其他环境的部署配置。有关 Helm charts，请参阅 [mastodon/chart 仓库](https://github.com/mastodon/chart). 主文档还提供[**独立**安装指南](https://docs.joinmastodon.org/admin/install/)。

## 贡献

Mastodon 是依据 **AGPLv3** 授权的**免费开源软件**。我们欢迎所有希望改进项目的人贡献和提供帮助。

请阅读总体的 [CONTRIBUTING](https://github.com/mastodon/.github/blob/main/CONTRIBUTING.md)指南，其中介绍了我们的开发流程。

也请阅读并理解 [CODE OF CONDUCT](https://github.com/mastodon/.github/blob/main/CODE_OF_CONDUCT.md)，帮助我们维护友好、包容的社区。合作始于相互尊重与理解。

可以在 [DEVELOPMENT](https://github.com/mastodon/mastodon/blob/5ccbf0e488801f9b7ee5fbe9d915f861081fa3ca/docs/DEVELOPMENT.md)文档中了解如何配置开发环境。

如果想帮助翻译 🌐，可以前往 [Crowdin](https://crowdin.com/project/mastodon).

## 许可证

Copyright (c) 2016-2026 Eugen Rochko (+ [`mastodon authors`](https://github.com/mastodon/mastodon/blob/5ccbf0e488801f9b7ee5fbe9d915f861081fa3ca/AUTHORS.md))

依据 [LICENSE](https://github.com/mastodon/mastodon/blob/5ccbf0e488801f9b7ee5fbe9d915f861081fa3ca/LICENSE) 中所述的 GNU Affero General Public License 授权：

```text
Copyright (c) 2016-2026 Eugen Rochko & other Mastodon contributors

This program is free software: you can redistribute it and/or modify it under
the terms of the GNU Affero General Public License as published by the Free
Software Foundation, either version 3 of the License, or (at your option) any
later version.

This program is distributed in the hope that it will be useful, but WITHOUT
ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more
details.

You should have received a copy of the GNU Affero General Public License along
with this program. If not, see https://www.gnu.org/licenses/
```
