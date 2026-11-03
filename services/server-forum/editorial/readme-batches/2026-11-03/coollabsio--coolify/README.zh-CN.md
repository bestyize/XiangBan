<div align="center">

<img src="https://raw.githubusercontent.com/coollabsio/coolify/82d89589a2b29233d2eb3ec47dc2b2ed4289bc9e/public/coolify-logo.svg" alt="Coolify logo" width="120" />

# Coolify

**用于将应用、数据库和服务部署到自己服务器的开源平台。**

永久开源免费，遵循我们的[理念](https://coolify.io/philosophy).

![Latest Release Version](https://img.shields.io/badge/dynamic/json?labelColor=grey&color=6366f1&label=Latest%20released%20version&url=https%3A%2F%2Fcdn.coollabs.io%2Fcoolify%2Fversions.json&query=stable.coolify.version&style=for-the-badge
)

[官网](https://coolify.io) · [文档](https://coolify.io/docs) · [Cloud](https://app.coolify.io) · [Discord](https://coollabs.io/discord) · [社区](https://github.com/coollabsio/coolify/discussions)

</div>

## Coolify 是什么？

Coolify 是 Heroku、Netlify 和 Vercel 的开源、自托管替代方案。它帮助你在自己的硬件上管理服务器、应用和数据库，只需 SSH 连接。

你可以使用 VPS、裸机服务器、树莓派或其他接受 SSH 连接的服务器。Coolify 提供云平台的便利，同时让你保有对基础设施的控制。

配置保留在你的服务器上。即使停止使用 Coolify，正在运行的资源仍会继续工作并可继续管理。

## Coolify 能做什么？

- **部署应用：** 从 GitHub、GitLab、Bitbucket 或 Gitea 获取代码，使用 Nixpacks、Railpack、Dockerfile、Docker Compose 或预构建 Docker 镜像进行构建。
- **运行数据库和服务：** 部署带持久存储及生成凭据的托管数据库和 360 多种一键服务。
- **自动部署：** 每次 Git 推送时部署、创建 Pull Request 预览、调用部署 webhook，并回滚到保留的应用镜像。
- **管理网络：** 配置自定义域名、自动 HTTPS 证书、反向代理、健康检查和容器网络。
- **运维基础设施：** 管理多台服务器、检查部署和运行日志、打开容器终端并监控资源状态。
- **保护工作负载：** 配置数据库与存储备份、计划任务、环境变量、密钥和通知。
- **集成工作流：** 通过仪表盘、API、CLI、MCP 和基于团队的访问控制管理资源。

## 快速开始

在受支持的服务器上使用一条命令安装 Coolify：

```bash
curl -fsSL https://cdn.coollabs.io/coolify/install.sh | bash
```

有关要求和详细说明，请阅读[安装指南](https://coolify.io/docs/installation)。运行前也可以审阅[安装脚本](https://github.com/coollabsio/coolify/blob/82d89589a2b29233d2eb3ec47dc2b2ed4289bc9e/scripts/install.sh)。

## 自托管或云服务

| 自托管 | Coolify Cloud |
| --- | --- |
| 在自己的服务器上安装 Coolify。 | 使用由我们维护的 Coolify 实例。 |
| 控制并维护完整平台。 | 获得高可用性并减少维护工作。 |
| 免费且开源。 | 提供邮件通知及额外支持的付费服务。 |

推荐的自托管配置是使用一台服务器运行 Coolify，并使用一台或多台服务器运行部署的资源。如果不想维护 Coolify 服务器，可使用 [Coolify Cloud](https://app.coolify.io)。当前价格请查看 [coolify.io](https://coolify.io)。

## 社区与支持

- 阅读[文档](https://coolify.io/docs)。
- 在 [Discord](https://coollabs.io/discord) 加入社区。
- 在 [GitHub Discussions](https://github.com/coollabsio/coolify/discussions) 提问。
- 提交更改前请阅读[贡献指南](https://github.com/coollabsio/coolify/blob/82d89589a2b29233d2eb3ec47dc2b2ed4289bc9e/CONTRIBUTING.md)。
- 查阅[行为准则](https://github.com/coollabsio/coolify/blob/82d89589a2b29233d2eb3ec47dc2b2ed4289bc9e/CODE_OF_CONDUCT.md).
- 通过[支持页面](https://coolify.io/docs/contact)联系团队。

## 捐赠
为了保持完全免费与开源、不将功能置于付费墙后并持续发展项目，我们需要你的帮助。如果你喜欢 Coolify，请考虑捐赠，支持项目未来的开发。

[coolify.io/sponsorships](https://coolify.io/sponsorships)

非常感谢！

### 大额赞助商

* [Ginernet](https://ginernet.com/) - 在西班牙托管高性能服务器
* [SerpAPI](https://serpapi.com) - Google Search API——通过其快速、易用、完整的 API 抓取 Google 和其他搜索引擎。
* [MVPS](https://www.mvps.net) - 尽可能高品质的低价 VPS 服务器
* [ScreenshotOne](https://screenshotone.com) - 面向开发者的截图 API
* [PrivateAlps](https://privatealps.net) - 面向关注隐私和控制权用户的云服务、VPS 和服务器基础设施提供商
* [Seibert Group](https://seibert.link/coolifysoftware) - 使用 Claude Code 等 AI 智能体提升全公司生产力
* [Contabo](https://contabo.com/en/coolify-vps/) - 价格无与伦比的云 VPS 和独立服务器

### 主要赞助商

* [Vanaways](https://www.vanaways.co.uk) - 在英国销售和租赁新货车
* [Cloudways](https://www.cloudways.com/en/?id=2125302) - DigitalOcean 的托管云主机平台
* [ByteBase](https://www.bytebase.com) - 大规模数据库 CI/CD 与安全
* [Ramnode](https://ramnode.com/) - 高性能云 VPS 主机
* [23M](https://23m.com) - 高可用主机解决方案
* [Macarne](https://macarne.com) - 面向简化网络连接的最佳 IP Transit 与运营商以太网解决方案
* [Hetzner](http://htznr.li/CoolifyXHetzner) - 服务器、云、主机和数据中心解决方案
* [Logto](https://logto.io) - 面向开发者的更好身份基础设施
* [Supadata](https://supadata.ai/) - 抓取 YouTube、网页和文件，为项目获取可供 AI 使用的整洁数据。 
* [Tolgee](https://tolgee.io) - 开源本地化平台
* [Best Consultant](https://bc.direct) - 值得信赖的技术咨询合作伙伴
* [ArcJet](https://arcjet.com) - 先进的 Web 安全与性能解决方案
* [SupaGuide](https://supa.guide) - Supabase 综合指南
* [CodeRabbit](https://coderabbit.ai) - 将代码审查时间和错误减少一半
* [Convex](https://convex.link/coolify.io) - 面向 Web 应用开发者的开源响应式数据库。
* [GoldenVM](https://billing.goldenvm.com) - 高端虚拟机托管解决方案
* [Comit International](https://comit.international) - 获得《纽约时报》奖项的承包商
* [Compai](https://www.trycomp.ai) - 开源合规自动化平台，帮助快速完成合规工作，是 Drata 和 Vanta 的开源替代方案。
* [Tigris](https://www.tigrisdata.com) - 现代 S3 替代方案
* [Blacksmith](https://blacksmith.sh) - 基础设施自动化平台
* [JobsCollider](https://jobscollider.com/remote-jobs) - 面向开发者的 30,000 多个远程职位
* [Darweb](https://darweb.nl/?ref=coolify.io&utm_source=coolify.io) - 设计、开发和交付，专注于电商 3D CPQ 解决方案。
* [Hostinger](https://www.hostinger.com/vps/coolify-hosting) - 网站托管和 VPS 解决方案
* [Mobb](https://vibe.mobb.ai/) - 保护 AI 生成的代码，提升开发效率
* [Ubicloud](https://www.ubicloud.com) - 开源云基础设施平台
* [PFGLabs](https://pfglabs.com) - 使用 Golang 构建真实项目
* [JuxtDigital](https://juxtdigital.com) - 数字公关与 AI 权威建设机构
* [SaasyKit](https://saasykit.com) - 面向开发者的完整 SaaS 入门工具包
* [American Cloud](https://americancloud.com) - 美国云基础设施服务
* [Greptile](https://www.greptile.com) - AI 代码审查工具
* [VPSDime](https://vpsdime.com/) - 低价 VPS 主机——4GB 内存每月 5 美元
* [dataforest Cloud](https://cloud.dataforest.net/en) - 数秒内以种子形式独立部署云服务器。企业级硬件、优质网络，100% 德国制造。
* [ISHosting](https://ishosting.com/) - 主机托管和 VPS 解决方案
* [NodePlus.ai](https://nodeplus.ai/) - 读取企业所使用各个系统的 AI

### 其他赞助商

译注：本节 45 张赞助商图片改为名称链接；原图墙见[固定版本上游原文](https://github.com/coollabsio/coolify/blob/82d89589a2b29233d2eb3ec47dc2b2ed4289bc9e/README.md#small-sponsors)。

[Serverside](https://serverside.com/?utm_source=coolify.io)
[DarkVPS](https://darkvps.pro)
[Open Source Alternatives](https://www.opensourcealternatives.to)
[Onserva](https://onserva.com/)
[Movavi](https://www.movavi.com/mac.html?utm_source=coolify.io)
[ABXY](https://usefoil.com/)
[LaunchFast Boilerplates](https://www.launchfa.st/?utm_source=coolify.io)
[Netrouting](https://www.netrouting.com/?utm_source=coolify.io)
[MindEd Tech](https://github.com/mindedtech)
[YouStable](https://youstable.com/?utm_source=coolify.io)
[Transcript LOL](https://transcript.lol/?utm_source=coolify.io)
[Autom](https://www.autom.dev/?utm_source=coolify.io)
[HuntAPI](https://www.huntapi.com/?utm_source=coolify.io)
[ULTRASERVERS](https://ultraservers.com/?utm_source=coolify.io)
[VibeTone](https://vibetone.com/?utm_source=coolify.io)
[Piloterr](https://www.piloterr.com/?utm_source=coolify.io)
[Alexey Panteleev](https://yoxel.com/?utm_source=coolify.io)
[SummYT - YouTube Summarizer](https://summyt.app?utm_source=coolify.io)
[Xaman](https://xaman.app/?utm_source=coolify.io)
[Monadical](https://monadical.com/?utm_source=coolify.io)
[Magic as a Service](https://maas.engineering/?utm_source=coolify.io)
[FiveManage](https://fivemanage.com?utm_source=coolify.io)
[Crypto Jobs List](https://cryptojobslist.com/?utm_source=coolify.io)
[SerpAPI](https://serpapi.com/?utm_source=coolify.io)
[typebot](https://typebot.io/?utm_source=coolify.io)
[360Creators](https://360creators.com/?utm_source=coolify.io)
[Cap-go](https://capgo.app/?utm_source=coolify.io)
[Cirun](https://cirun.io/?utm_source=coolify.io)
[Puls Digital Group](https://github.com/puls-digital-group)
[Jonathan Pereira](https://github.com/jonathanprl)
[OutboundGateway](https://outboundgateway.com/)
[T4DT GmbH](https://github.com/t4dt)
[Internet Garden](https://internetgarden.co/?utm_source=coolify.io)
[Evercam](https://evercam.io/?utm_source=coolify.io)
[Web3 Jobs](https://web3.career/?utm_source=coolify.io)
[LinkDr](https://linkdr.com?utm_source=coolify.io)
[Arvensis Systems](https://arvensis.systems/?utm_source=coolify.io)
[RunPod](https://www.runpod.io/?utm_source=coolify.io)
[Gravity Wiz](http://gravitywiz.com/?utm_source=coolify.io)
[UXWizz](https://www.uxwizz.com/?utm_source=coolify.io)
[Codext](https://codext.link/coolify-io?utm_source=coolify.io)
[InterviewPal](https://interviewpal.com)
[Decidable](https://decidable.no?utm_source=coolify.io)
[Host Havoc](https://hosthavoc.com)
[martiinii](https://martiinii.dev)

……以及 [GitHub Sponsors](https://github.com/sponsors/coollabsio)

## Star 历史

[![Star History Chart](https://api.star-history.com/svg?repos=coollabsio/coolify&type=Date)](https://star-history.com/#coollabsio/coolify&Date)
