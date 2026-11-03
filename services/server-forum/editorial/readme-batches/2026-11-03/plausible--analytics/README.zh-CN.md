> 译注：隐私、法规合规、性能和竞品商业模式等表述来自上游项目，尚未独立验证。原反馈看板本次返回 HTTP 403，原链保留；也可通过 [GitHub Issues](https://github.com/plausible/analytics/issues)报告问题。

# Plausible Analytics

<p align="center">
  <a href="https://plausible.io/">
    <img src="https://raw.githubusercontent.com/plausible/docs/master/static/img/plausible-analytics-icon-top.png" width="140px" alt="Plausible Analytics" />
  </a>
</p>
<p align="center">
    <a href="https://plausible.io/plausible.io">在线演示</a> |
    <a href="https://plausible.io/open-source-website-analytics">开源</a> |
    <a href="https://plausible.io/docs">文档</a> |
    <a href="https://github.com/plausible/analytics/blob/3644844d81058c5b26923d33177fe1c3de7f9fb3/CONTRIBUTING.md">贡献</a>
    <br /><br />
</p>

[Plausible Analytics](https://plausible.io/) 是一款隐私优先的开源网站分析工具，是轻量、无需 Cookie 的 [Google Analytics 替代方案](https://plausible.io/blog/remove-google-analytics)。提供托管云服务或自托管社区版。

- 🔐 [注重隐私](https://plausible.io/privacy-focused-web-analytics)
- 🛡️ [符合 GDPR、CCPA、PECR](https://plausible.io/data-policy)
- 📊 [简洁快速的仪表板](https://plausible.io/simple-web-analytics)
- 🪶 [轻量脚本](https://plausible.io/lightweight-web-analytics)
- 🇪🇺 [在欧盟开发与托管](https://plausible.io/eu-hosted-web-analytics)

![Plausible Analytics](https://raw.githubusercontent.com/plausible/analytics/3644844d81058c5b26923d33177fe1c3de7f9fb3/.github/plausible-analytics-dashboard.webp)

## 为什么选择 Plausible？

以下是项目列出的 Plausible 作为 Google Analytics 替代方案的优势，以及数千名付费订阅者信任其提供网站和业务洞察的原因：

- **简洁无杂乱**  
  Plausible 提供[简单的网站分析](https://plausible.io/simple-web-analytics)，减少冗余信息。不需要层层菜单或创建自定义报告；所有关键洞察都在一个页面上，无需培训。

- **隐私优先且符合规定**  
  衡量流量而非个人。不存储个人数据或 IP 地址，也不使用 Cookie 或持久标识符。项目称其完全符合 GDPR、CCPA 和 PECR。  
  [进一步了解数据政策](https://plausible.io/data-policy)

- **轻量**  
  我们的脚本[体积很小](https://plausible.io/lightweight-web-analytics)，有助于网站更快加载。也可以直接通过[事件 API](https://plausible.io/docs/events-api)发送事件.

- **为现代工作流构建**  
  兼容现代框架，开箱即支持 SPA，包括 pushState 和基于 hash 的路由。参阅[详情](https://plausible.io/docs/hash-based-routing).

- **定义关键目标并跟踪转化**  
  使用自定义事件和维度跟踪目标、转化、收入归因及漏斗。可为外部链接点击、表单完成、文件下载和 404 错误页面启用无代码跟踪。

- **便捷报告与分享**  
  通过电子邮件或 Slack 获取每周或每月报告，包括流量激增／下降通知。[邀请团队成员](https://plausible.io/docs/users-roles)并按角色控制访问权限；可公开分享仪表板，或通过链接与任何人分享，使分析默认透明。

- **包含搜索洞察**  
  与 Google Search Console 集成，直接在仪表板中获取关键词数据。

- **API 与集成**  
  通过 API 直接发送事件，使用统计 API 或 CSV 导出数据，并基于数据构建自定义工作流与集成。

- **实时洞察**  
  监测实时流量，及时了解网站正在发生的情况。

- **从 Google Analytics 平稳迁移**  
  提供熟悉的指标、活动跟踪和 Search Console 集成。可导入历史 Google Analytics 统计数据并继续使用。

项目称数千个团队使用 Plausible 了解流量，同时兼顾隐私与简单性。了解如何[充分使用 Plausible](https://plausible.io/docs/your-plausible-experience).

我们致力于让网站分析更加尊重隐私。我们的使命是提供不来自广告技术领域的替代分析工具，以减少企业监视。我们完全独立，仅由订阅者资助。  

想了解更多？请[访问官网](https://plausible.io),在[关于页面](https://plausible.io/about)了解团队与目标或查看[文档](https://plausible.io/docs). 

## 为什么 Plausible Analytics Cloud 不像 Google Analytics 一样免费？

Plausible 是完全由用户资助的独立开源项目。我们收取订阅费，以便长期、持续地开发、维护和改进产品。

项目表示，Google Analytics 免费使用是因为 Google 将用户数据用于广告变现；这种模式伴随数据收集、复杂性以及合规和同意方面的额外成本。

Plausible 的商业模式如下：

- 除汇总、匿名化统计外不收集其他数据  
- 不与第三方分享数据  
- 没有广告或跟踪生态  
- 你完全拥有并控制自己的数据  

我们认为分析服务的付费方式应当直接透明：通过订阅为产品付费，而不是以用户数据付费。[了解更多](https://plausible.io/paid-analytics-vs-free-ga).

## 开始使用 Plausible

开始使用 Plausible Analytics 最简单的方式是[官方托管云服务](https://plausible.io/register).

- 配置约需 2 分钟  
- 包括全球 CDN、高可用性、备份和安全保障  
- 无需自行维护

项目称，为符合 GDPR 和 Schrems II 裁决，其托管云服务的全部访客数据仅在欧洲提供商拥有和运营的服务器及云基础设施上处理，网站数据不会离开欧盟。

项目表示，其托管服务可节省大量开发时间和资源，对多数网站是高性价比选择；收入用于资助 Plausible 的维护与进一步开发，从而支持开源软件并获得服务。

### Plausible 可以自托管吗？

Plausible 是[开源网站分析工具](https://plausible.io/open-source-website-analytics)，并提供名为 [Plausible Community Edition（CE）](https://plausible.io/self-hosted-web-analytics)的免费自托管方案. 以下是 Plausible Analytics 托管云服务与 Plausible CE 的区别：

|  | Plausible Analytics Cloud  | Plausible Community Edition |
| ------------- | ------------- | ------------- |
| **基础设施管理** | 方便快捷，约 2 分钟开始统计；全球 CDN、高可用性、备份、安全和维护均由我们负责，你可以专注统计数据。 | 需要自己准备服务器并管理基础设施，负责安装、维护、升级、服务器容量、运行时间、备份、安全、稳定性、一致性和加载时间等。|
| **发布周期** | 持续开发改进，每周多次发布新功能和更新。 | [长期发行版](https://plausible.io/blog/building-open-source)每年发布两次，因此最新功能和改进不会立即提供。|
| **高级功能** | 提供[定价方案](https://plausible.io/#pricing)中列出的所有功能. | 不提供高级功能（营销漏斗、电商收入目标、SSO 和站点 API），以支持[项目长期可持续发展](https://plausible.io/blog/community-edition).|
| **机器人过滤** | 高级机器人过滤以提高统计准确性：算法检测并排除非人类流量模式，通过 User-Agent 排除已知机器人，并过滤数据中心及引荐垃圾域名流量。默认排除约 3.2 万个数据中心 IP 范围。 | 基础机器人过滤，根据 User-Agent 和引荐垃圾域名针对最常见的非人类流量。|
| **服务器位置** | 所有访客数据仅在欧盟拥有的云基础设施上处理；网站数据保存在德国的安全、加密且由绿色能源供电的服务器。项目称这使数据受到严格的欧盟隐私法律保护并符合 GDPR，数据不会离开欧盟。 | 你可以完全控制实例，并在任何国家／地区的服务器上托管，可以使用家中的服务器或任意云提供商，包括不符合 GDPR 的提供商。|
| **数据可移植性** | 通过仪表板查看网站统计和指标，仪表板仅显示汇总统计。可以通过 [CSV 导出](https://plausible.io/docs/export-stats), [统计 API](https://plausible.io/docs/stats-api)或 [Data Studio 连接器](https://plausible.io/docs/looker-studio)下载统计数据。 | 需要访问原始数据？自托管可直接从 ClickHouse 数据库获取数据，但不提供 Data Studio 连接器。 |
| **高级支持** | 由构建和维护 Plausible 的人员提供支持。 | 不包含高级支持；CE 仅由社区提供支持。|
| **费用** | 提供分析服务需要成本，因此我们收取订阅费。我们选择订阅模式，而非监视资本主义的商业模式；费用用于进一步开发 Plausible。 | 你需要支付服务器、CDN、备份和其他基础设施运行费用，无需向我们支付费用；费用支付给与我们无关的第三方公司。|

想在服务器上自托管 Plausible CE？请查看 [Plausible CE 安装说明](https://github.com/plausible/community-edition/).

Plausible CE 由社区支持，不保证 Plausible 创建者会帮助排查自托管问题。[社区支持论坛](https://github.com/plausible/analytics/discussions/categories/self-hosted-support)可供你寻求帮助。

我们的唯一资金来源是收费的 Plausible 云托管服务。
 
## 技术

Plausible 使用现代、可扩展的技术栈：

- **后端**： Elixir + Phoenix  
- **数据库**： PostgreSQL （一般数据），ClickHouse（分析）  
- **前端**： React + [TailwindCSS](https://tailwindcss.com/)  

项目称其架构可高效处理大量流量，同时保持仪表板快速响应。

## 贡献者

希望为 Plausible 做贡献的人，请参阅[贡献者指南](https://github.com/plausible/analytics/blob/3644844d81058c5b26923d33177fe1c3de7f9fb3/CONTRIBUTING.md).

<a href="https://github.com/plausible/analytics/graphs/contributors"><img src="https://opencollective.com/plausible/contributors.svg?width=800&button=false" /></a>

## 反馈与路线图

我们欢迎社区反馈。公开路线图由社区成员提出的功能建议推动，请查看[反馈看板](https://plausible.io/feedback). 如有需求请告诉我们，并为开放问题投票，以帮助我们确定优先级。

如需了解最新新闻和产品更新，请关注 [X (formerly Twitter)](https://twitter.com/plausiblehq), [Bluesky](https://bsky.app/profile/plausible.io), [LinkedIn](https://www.linkedin.com/company/plausible-analytics/) 和 [Mastodon](https://fosstodon.org/@plausible).

## 许可证与商标

Plausible CE 根据 GNU Affero General Public License 第 3 版（AGPLv3）或更高版本开源，许可证可[在此查看](https://github.com/plausible/analytics/blob/3644844d81058c5b26923d33177fe1c3de7f9fb3/LICENSE.md).

为了避免 AGPL 传播性带来的问题，项目将嵌入网站的 JavaScript 跟踪器以 MIT 许可发布，许可证可[在此查看](https://github.com/plausible/analytics/blob/3644844d81058c5b26923d33177fe1c3de7f9fb3/tracker/npm_package/LICENSE.md).

Copyright (c) 2018-present Plausible Insights OÜ。Plausible Analytics 名称和标志是 Plausible Insights OÜ 的商标。请参阅[商标指南](https://plausible.io/trademark)了解可接受的使用方式。
