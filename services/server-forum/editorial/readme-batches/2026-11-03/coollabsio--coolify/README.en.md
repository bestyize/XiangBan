<div align="center">

<img src="https://raw.githubusercontent.com/coollabsio/coolify/82d89589a2b29233d2eb3ec47dc2b2ed4289bc9e/public/coolify-logo.svg" alt="Coolify logo" width="120" />

# Coolify

**An open-source platform to deploy applications, databases, and services on your own servers.**

Open source & free forever, backed by our [philosophy](https://coolify.io/philosophy).

![Latest Release Version](https://img.shields.io/badge/dynamic/json?labelColor=grey&color=6366f1&label=Latest%20released%20version&url=https%3A%2F%2Fcdn.coollabs.io%2Fcoolify%2Fversions.json&query=stable.coolify.version&style=for-the-badge
)

[Website](https://coolify.io) · [Documentation](https://coolify.io/docs) · [Cloud](https://app.coolify.io) · [Discord](https://coollabs.io/discord) · [Community](https://github.com/coollabsio/coolify/discussions)

</div>

## What is Coolify?

Coolify is an open-source and self-hostable alternative to Heroku, Netlify, and Vercel. It helps you manage servers, applications, and databases on your own hardware. You only need an SSH connection.

You can use a VPS, a bare-metal server, a Raspberry Pi, or any other server that accepts SSH connections. Coolify gives you the convenience of a cloud platform while you keep control of your infrastructure.

Your configurations stay on your servers. If you stop using Coolify, your running resources continue to work and remain manageable.

## What can Coolify do?

- **Deploy any application:** Build from GitHub, GitLab, Bitbucket, or Gitea with Nixpacks, Railpack, Dockerfile, Docker Compose, or a prebuilt Docker image.
- **Run databases and services:** Deploy managed databases and more than 360 one-click services with persistent storage and generated credentials.
- **Automate deployments:** Deploy on every Git push, create pull-request previews, call deployment webhooks, and roll back to retained application images.
- **Manage networking:** Configure custom domains, automatic HTTPS certificates, reverse proxies, health checks, and container networks.
- **Operate your infrastructure:** Manage multiple servers, inspect deployment and runtime logs, open container terminals, and monitor resource status.
- **Protect your workloads:** Configure database and storage backups, scheduled tasks, environment variables, secrets, and notifications.
- **Integrate with your workflow:** Manage resources through the dashboard, API, CLI, MCP, and team-based access controls.

## Quick start

Install Coolify on a supported server with one command:

```bash
curl -fsSL https://cdn.coollabs.io/coolify/install.sh | bash
```

Read the [installation guide](https://coolify.io/docs/installation) for requirements and detailed instructions. You can also review the [installation script](https://github.com/coollabsio/coolify/blob/82d89589a2b29233d2eb3ec47dc2b2ed4289bc9e/scripts/install.sh) before you run it.

## Self-hosted or Cloud

| Self-hosted | Coolify Cloud |
| --- | --- |
| Install Coolify on your own server. | Use a Coolify instance that we maintain. |
| Control and maintain the complete platform. | Get high availability and less maintenance. |
| Free and open source. | Paid service with email notifications and additional support. |

The recommended self-hosted setup uses one server for Coolify and one or more servers for deployed resources. If you do not want to maintain the Coolify server, use [Coolify Cloud](https://app.coolify.io). See [coolify.io](https://coolify.io) for current pricing.

## Community and support

- Read the [documentation](https://coolify.io/docs).
- Join the community on [Discord](https://coollabs.io/discord).
- Ask questions in [GitHub Discussions](https://github.com/coollabsio/coolify/discussions).
- Read the [contribution guide](https://github.com/coollabsio/coolify/blob/82d89589a2b29233d2eb3ec47dc2b2ed4289bc9e/CONTRIBUTING.md) before you submit a change.
- Review the [code of conduct](https://github.com/coollabsio/coolify/blob/82d89589a2b29233d2eb3ec47dc2b2ed4289bc9e/CODE_OF_CONDUCT.md).
- Contact the team through the [support page](https://coolify.io/docs/contact).

## Donations
To stay completely free and open-source, with no feature behind the paywall and evolve the project, we need your help. If you like Coolify, please consider donating to help us fund the project's future development.

[coolify.io/sponsorships](https://coolify.io/sponsorships)

Thank you so much!

### Huge Sponsors

* [Ginernet](https://ginernet.com/) - Hosting powerful servers in Spain
* [SerpAPI](https://serpapi.com) - Google Search API — Scrape Google and other search engines from our fast, easy, and complete API.
* [MVPS](https://www.mvps.net) - Cheap VPS servers at the highest possible quality
* [ScreenshotOne](https://screenshotone.com) - Screenshot API for devs
* [PrivateAlps](https://privatealps.net) - Cloud Services Provider, VPS, servers infrastructure for people who care about privacy and control
* [Seibert Group](https://seibert.link/coolifysoftware) - Boost productivity company-wide with AI agents like Claude Code
* [Contabo](https://contabo.com/en/coolify-vps/) - Cloud VPS & dedicated servers at unbeatable prices

### Big Sponsors

* [Vanaways](https://www.vanaways.co.uk) - New vans for sale and lease across the UK
* [Cloudways](https://www.cloudways.com/en/?id=2125302) - Managed cloud hosting platform by DigitalOcean
* [ByteBase](https://www.bytebase.com) - Database CI/CD and Security at Scale
* [Ramnode](https://ramnode.com/) - High Performance Cloud VPS Hosting
* [23M](https://23m.com) - Your experts for high-availability hosting solutions!
* [Macarne](https://macarne.com) - Best IP Transit & Carrier Ethernet Solutions for Simplified Network Connectivity
* [Hetzner](http://htznr.li/CoolifyXHetzner) - Server, cloud, hosting, and data center solutions
* [Logto](https://logto.io) - The better identity infrastructure for developers
* [Supadata](https://supadata.ai/) - Scrape YouTube, web, and files. Get AI-ready, clean data for your next project. 
* [Tolgee](https://tolgee.io) - The open source localization platform
* [Best Consultant](https://bc.direct) - Your trusted technology consulting partner
* [ArcJet](https://arcjet.com) - Advanced web security and performance solutions
* [SupaGuide](https://supa.guide) - Your comprehensive guide to Supabase
* [CodeRabbit](https://coderabbit.ai) - Cut Code Review Time & Bugs in Half
* [Convex](https://convex.link/coolify.io) - Convex is the open-source reactive database for web app developers.
* [GoldenVM](https://billing.goldenvm.com) - Premium virtual machine hosting solutions
* [Comit International](https://comit.international) - New York Times award–winning contractor!
* [Compai](https://www.trycomp.ai) - The open source compliance automation platform that does everything you need to get compliant, fast. Open source alternative to Drata & Vanta.
* [Tigris](https://www.tigrisdata.com) - Modern S3 Alternative
* [Blacksmith](https://blacksmith.sh) - Infrastructure automation platform
* [JobsCollider](https://jobscollider.com/remote-jobs) - 30,000+ remote jobs for developers
* [Darweb](https://darweb.nl/?ref=coolify.io&utm_source=coolify.io) - Design. Develop. Deliver. Specialized in 3D CPQ Solutions for eCommerce.
* [Hostinger](https://www.hostinger.com/vps/coolify-hosting) - Web hosting and VPS solutions
* [Mobb](https://vibe.mobb.ai/) - Secure Your AI-Generated Code to Unlock Dev Productivity
* [Ubicloud](https://www.ubicloud.com) - Open source cloud infrastructure platform
* [PFGLabs](https://pfglabs.com) - Build Real Projects with Golang
* [JuxtDigital](https://juxtdigital.com) - Digital PR & AI Authority Building Agency
* [SaasyKit](https://saasykit.com) - Complete SaaS starter kit for developers
* [American Cloud](https://americancloud.com) - US-based cloud infrastructure services
* [Greptile](https://www.greptile.com) - The AI Code Reviewer
* [VPSDime](https://vpsdime.com/) - Cheap VPS Hosting - 4GB for $5/month
* [dataforest Cloud](https://cloud.dataforest.net/en) - Deploy cloud servers as seeds independently in seconds. Enterprise hardware, premium network, 100% made in Germany.
* [ISHosting](https://ishosting.com/) - Hosting and VPS solutions
* [NodePlus.ai](https://nodeplus.ai/) - AI that reads every system your business runs on

### Small Sponsors

Editorial note: The 45 sponsor images are shown as name links below; the original image wall is available in the [captured upstream README](https://github.com/coollabsio/coolify/blob/82d89589a2b29233d2eb3ec47dc2b2ed4289bc9e/README.md#small-sponsors).

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

...and many more at [GitHub Sponsors](https://github.com/sponsors/coollabsio)

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=coollabsio/coolify&type=Date)](https://star-history.com/#coollabsio/coolify&Date)
