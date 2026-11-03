&nbsp;
<p align="center">
  <a href="https://ghost.org/#gh-light-mode-only" target="_blank">
    <img src="https://user-images.githubusercontent.com/65487235/157884383-1b75feb1-45d8-4430-b636-3f7e06577347.png" alt="Ghost" width="200px">
  </a>
  <a href="https://ghost.org/#gh-dark-mode-only" target="_blank">
    <img src="https://user-images.githubusercontent.com/65487235/157849205-aa24152c-4610-4d7d-b752-3a8c4f9319e6.png" alt="Ghost" width="200px">
  </a>
</p>
&nbsp;

<p align="center">
    <a href="https://ghost.org/">Ghost.org</a> •
    <a href="https://forum.ghost.org">Forum</a> •
    <a href="https://docs.ghost.org">Docs</a> •
    <a href="https://github.com/TryGhost/Ghost/blob/230457a41d9efe3826bb7d285e1b79a6b10d8647/docs/README.md">Contributing</a> •
    <a href="https://twitter.com/ghost">Twitter</a>
    <br /><br />
    <a href="https://ghost.org/">
        <img src="https://img.shields.io/badge/downloads-100M+-brightgreen.svg" alt="Downloads" />
    </a>
    <a href="https://github.com/TryGhost/Ghost/releases/">
        <img src="https://img.shields.io/github/release/TryGhost/Ghost.svg" alt="Latest release" />
    </a>
    <a href="https://github.com/TryGhost/Ghost/actions">
        <img src="https://github.com/TryGhost/Ghost/actions/workflows/ci.yml/badge.svg?branch=main" alt="Build status" />
    </a>
    <a href="https://github.com/TryGhost/Ghost/contributors/">
        <img src="https://img.shields.io/github/contributors/TryGhost/Ghost.svg" alt="Contributors" />
    </a>
    <a href="https://digitalpublicgoods.net/r/ghost"><img src="https://img.shields.io/badge/Verified-DPG-3333AB?logo=data:image/svg%2bxml;base64,PHN2ZyB3aWR0aD0iMzEiIGhlaWdodD0iMzMiIHZpZXdCb3g9IjAgMCAzMSAzMyIgZmlsbD0ibm9uZSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj4KPHBhdGggZD0iTTE0LjIwMDggMjEuMzY3OEwxMC4xNzM2IDE4LjAxMjRMMTEuNTIxOSAxNi40MDAzTDEzLjk5MjggMTguNDU5TDE5LjYyNjkgMTIuMjExMUwyMS4xOTA5IDEzLjYxNkwxNC4yMDA4IDIxLjM2NzhaTTI0LjYyNDEgOS4zNTEyN0wyNC44MDcxIDMuMDcyOTdMMTguODgxIDUuMTg2NjJMMTUuMzMxNCAtMi4zMzA4MmUtMDVMMTEuNzgyMSA1LjE4NjYyTDUuODU2MDEgMy4wNzI5N0w2LjAzOTA2IDkuMzUxMjdMMCAxMS4xMTc3TDMuODQ1MjEgMTYuMDg5NUwwIDIxLjA2MTJMNi4wMzkwNiAyMi44Mjc3TDUuODU2MDEgMjkuMTA2TDExLjc4MjEgMjYuOTkyM0wxNS4zMzE0IDMyLjE3OUwxOC44ODEgMjYuOTkyM0wyNC44MDcxIDI5LjEwNkwyNC42MjQxIDIyLjgyNzdMMzAuNjYzMSAyMS4wNjEyTDI2LjgxNzYgMTYuMDg5NUwzMC42NjMxIDExLjExNzdMMjQuNjI0MSA5LjM1MTI3WiIgZmlsbD0id2hpdGUiLz4KPC9zdmc+Cg==" alt="DPG Badge"/></a>
</p>

&nbsp;

> [!NOTE]
> 热爱开源？我们正在招聘！Ghost 正在寻找资深工程师[加入团队](https://careers.ghost.org)，与我们全职共事

<a href="https://ghost.org/"><img src="https://user-images.githubusercontent.com/353959/169805900-66be5b89-0859-4816-8da9-528ed7534704.png" alt="Fiercely independent, professional publishing. Ghost is the most popular open source, headless Node.js CMS which already works with all the tools you know and love." /></a>

&nbsp;

<a href="https://ghost.org/pricing/#gh-light-mode-only" target="_blank"><img src="https://user-images.githubusercontent.com/65487235/157849437-9b8fcc48-1920-4b26-a1e8-5806db0e6bb9.png" alt="Ghost(Pro)" width="165px" /></a>
<a href="https://ghost.org/pricing/#gh-dark-mode-only" target="_blank"><img src="https://user-images.githubusercontent.com/65487235/157849438-79889b04-b7b6-4ba7-8de6-4c1e4b4e16a5.png" alt="Ghost(Pro)" width="165px" /></a>

部署生产实例最简单的方法是使用官方托管服务 **[Ghost(Pro)](https://ghost.org/pricing/)**。创建新站点大约需要 2 分钟，全球 CDN、备份、安全和维护均由服务提供。

对大多数人来说，考虑到[节省的时间](https://docs.ghost.org/hosting/)，这最终是最划算的选择——并且 100% 的收入都归 Ghost 基金会，用于资助项目本身的维护和进一步开发。因此你既在支持开源软件，_又_获得了优质服务！

&nbsp;

# 快速安装

如果想运行自己的 Ghost 实例，大多数情况下最好的方式是使用我们的 **CLI 工具**。

```
npm install ghost-cli -g
```

&nbsp;

如果在本地安装，请添加 `local` 参数，一分钟内即可运行——[本地安装文档](https://docs.ghost.org/install/local/)

```
ghost install local
```

&nbsp;

或者在服务器上运行完整安装，包括通过 LetsEncrypt 自动配置 SSL——[生产环境安装文档](https://docs.ghost.org/install/ubuntu/)

```
ghost install
```

&nbsp;

请参阅我们的[官方文档](https://docs.ghost.org/)，进一步了解[推荐的托管技术栈](https://docs.ghost.org/hosting/)与如何正确[升级 Ghost](https://docs.ghost.org/update/),，以及开发自己的 Ghost [主题](https://docs.ghost.org/themes/)或使用[我们的 API](https://docs.ghost.org/content-api/).

### 贡献者与高级开发者

要为 Ghost 做贡献，请先阅读
[贡献指南](https://github.com/TryGhost/Ghost/blob/230457a41d9efe3826bb7d285e1b79a6b10d8647/.github/CONTRIBUTING.md). 要参与单体仓库开发，请参阅
[代码库文档](https://github.com/TryGhost/Ghost/blob/230457a41d9efe3826bb7d285e1b79a6b10d8647/docs/README.md).

&nbsp;

# Ghost 赞助商

非常感谢让 Ghost 成为可能的赞助商和合作伙伴。如果你有意赞助 Ghost 并支持项目，请访问我们的 [GitHub Sponsors](https://github.com/sponsors/TryGhost) :heart:

**[DigitalOcean](https://m.do.co/c/9ff29836d717)** • **[Fastly](https://www.fastly.com/)** • **[Tinybird](https://tbrd.co/ghost)** • **[BairesDev](https://www.bairesdev.com)**

&nbsp;

# 获取帮助

任何人都可以在 [Ghost 论坛](https://forum.ghost.org/)向庞大的开发者社区寻求帮助和支持。 **Ghost(Pro)** 客户可获得全天候电子邮件支持。

如需了解最新新闻和产品更新，请[订阅更新日志邮件](https://ghost.org/changelog/)，或在 [Twitter](https://twitter.com/Ghost) 关注我们，获取简短轻松的更新。 :saxophone::turtle:

&nbsp;

# 许可与商标

版权所有 (c) 2013-2026 Ghost Foundation——依据 [MIT 许可证](https://github.com/TryGhost/Ghost/blob/230457a41d9efe3826bb7d285e1b79a6b10d8647/LICENSE)发布。
Ghost 和 Ghost 标志是 Ghost Foundation Ltd. 的商标。有关可接受的使用方式，请参阅我们的[商标政策](https://ghost.org/trademark/)。
