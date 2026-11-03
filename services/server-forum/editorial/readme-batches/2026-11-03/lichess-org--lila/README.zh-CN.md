# [lichess.org](https://lichess.org)

[![Crowdin](https://d322cqt584bo4o.cloudfront.net/lichess/localized.svg)](https://crowdin.com/project/lichess)
[![Mastodon](https://img.shields.io/mastodon/follow/109298525492334687?domain=mastodon.online)](https://mastodon.online/@lichess)
[![Bluesky](https://img.shields.io/badge/Bluesky-0285FF?logo=bluesky&logoColor=fff)](https://bsky.app/profile/lichess.org)
[![Discord](https://img.shields.io/discord/280713822073913354?label=Discord&logo=discord&style=flat)](https://discord.gg/lichess)
[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/lichess-org/lila)

<img src="https://raw.githubusercontent.com/lichess-org/lila/2e3e641c32b361c113e696cee34f9fddefef6727/public/images/home-bicolor.png" alt="Lichess 首页" title="Lichess 支持亮色和暗色主题，截图展示了两者。" />

Lila（li[chess in sca]la）是免费的在线国际象棋游戏服务器，侧重[实时](https://lichess.org/games)对局和易用性。

它提供[搜索引擎](https://lichess.org/games/search)、通过 [fishnet](https://github.com/lichess-org/fishnet) 分布式运行的[计算机分析](https://lichess.org/ief49lif)、[锦标赛](https://lichess.org/tournament)、[车轮战](https://lichess.org/simul)、[论坛](https://lichess.org/forum)、[团队](https://lichess.org/team)、[战术训练](https://lichess.org/training)、[移动应用](https://lichess.org/app)和[共享分析棋盘](https://lichess.org/study)。得益于社区，界面提供超过 [140 种语言](https://crowdin.com/project/lichess)。

Lichess 使用 [Scala 3](https://www.scala-lang.org/) 编写，并依赖经过修改的 [Play 2.8](https://www.playframework.com/) 框架。
使用 [scalatags](https://com-lihaoyi.github.io/scalatags/) 构建模板。
纯粹的国际象棋逻辑包含在 [scalachess](https://github.com/lichess-org/scalachess) 子模块中。
服务器完全异步，大量使用 Scala Futures 和 [Pekko streams](https://pekko.apache.org)。
WebSocket 连接由[独立服务器](https://github.com/lichess-org/lila-ws)处理，服务器之间使用 [redis](https://redis.io/) 通信。
Lichess 与部署在捐赠服务器组成的 [AI 集群](https://github.com/lichess-org/fishnet)中的 [Stockfish](https://stockfishchess.org/) 通信。
它使用 [MongoDB](https://www.mongodb.com) 存储超过 120 亿局对弈，并由 [elasticsearch](https://github.com/elastic/elasticsearch) 建立索引。
HTTP 请求和 WebSocket 连接可以由 [nginx](https://nginx.org) 代理。
Web 客户端使用 [TypeScript](https://www.typescriptlang.org/) 和 [snabbdom](https://github.com/snabbdom/snabbdom) 编写，并使用 [Sass](https://sass-lang.com/) 生成 CSS。
所有等级分对局都发布在[免费 PGN 数据库](https://database.lichess.org)中。
浏览器测试使用 [Browserstack](https://www.browserstack.com)。
代理检测使用 [IP2Proxy 数据库](https://www.ip2location.com/database/ip2proxy)。
软件构建和 CI/CD 使用 [Depot](https://depot.dev/)。
请帮助我们[使用 Crowdin 翻译 Lichess](https://crowdin.com/project/lichess)。

仓库列表请见 [lichess.org/source](https://lichess.org/source)。

[加入我们的 Discord](https://discord.gg/lichess) 了解更多信息。
请使用 [GitHub issues](https://github.com/lichess-org/lila/issues) 报告错误和提出功能请求。

## 安装

```
./lila.sh # thin wrapper around sbt
run
```

Wiki 说明了[如何设置开发环境](https://github.com/lichess-org/lila/wiki/Lichess-Development-Onboarding)。

## HTTP API

欢迎在应用和网站中使用 [Lichess API](https://lichess.org/api)。

## 支持的浏览器

| 名称 | 版本 | 说明 |
| ----------------- | ------- | -------------------------- |
| Firefox | 100+ | 完全支持（推荐） |
| Chromium / Chrome | 100+ | 完全支持 |
| Edge | 100+ | 完全支持 |
| Opera | 97+ | 基本支持 |
| Safari | 14+ | 基本支持 |

旧版浏览器无法使用。为了你自己，请升级。想想安全和性能吧！

## 许可证

Lila 采用 GNU Affero 通用公共许可证第 3 版或你选择的任何后续版本许可。详情请参阅 [copying](https://github.com/lichess-org/lila/blob/2e3e641c32b361c113e696cee34f9fddefef6727/COPYING.md)。

## 生产架构（截至 2022 年 7 月）

![Lichess 生产服务器架构图](https://raw.githubusercontent.com/lichess-org/lila/2e3e641c32b361c113e696cee34f9fddefef6727/public/images/architecture.png)

## 致谢

请参阅 [lichess.org/thanks](https://lichess.org/thanks) 以及此处的贡献者：

[![GitHub 贡献者](https://contrib.rocks/image?repo=lichess-org/lila)](https://github.com/lichess-org/lila/graphs/contributors)

## 能力发展计划

Lichess 希望通过承担相关培训材料和活动的费用，支持贡献者发展能力。这是对付出时间的贡献者提供进一步支持的一种小方式，也有助于他们将来继续贡献或提升贡献质量。有关如何申请等更多信息，请查看 [Lichess 贡献者能力发展](https://lichess.org/page/competence-development)。
