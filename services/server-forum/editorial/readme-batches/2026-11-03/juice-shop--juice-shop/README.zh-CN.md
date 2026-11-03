# ![Juice Shop 标志](https://raw.githubusercontent.com/juice-shop/juice-shop/717f5efa886b0417eeecb52378b03cf10567b7e2/frontend/src/assets/public/images/JuiceShop_Logo_100px.png) OWASP Juice Shop

[![OWASP Flagship](https://img.shields.io/badge/owasp-flagship%20project-48A646.svg)](https://owasp.org/projects/#sec-flagships)
[![GitHub release](https://img.shields.io/github/release/juice-shop/juice-shop.svg)](https://github.com/juice-shop/juice-shop/releases/latest)
[![Twitter Follow](https://img.shields.io/twitter/follow/owasp_juiceshop.svg?style=social&label=Follow)](https://twitter.com/owasp_juiceshop)
[![Subreddit subscribers](https://img.shields.io/reddit/subreddit-subscribers/owasp_juiceshop?style=social)](https://reddit.com/r/owasp_juiceshop)

[![CI/CD Pipeline](https://github.com/juice-shop/juice-shop/actions/workflows/ci.yml/badge.svg?branch=develop)](https://github.com/juice-shop/juice-shop/actions/workflows/ci.yml)
[![Release Pipeline](https://github.com/juice-shop/juice-shop/actions/workflows/release.yml/badge.svg)](https://github.com/juice-shop/juice-shop/actions/workflows/release.yml)
[![Coverage Status](https://coveralls.io/repos/github/juice-shop/juice-shop/badge.svg?branch=develop)](https://coveralls.io/github/juice-shop/juice-shop?branch=develop)
[![Cypress tests](https://img.shields.io/endpoint?url=https://dashboard.cypress.io/badge/simple/3hrkhu/develop&style=flat&logo=cypress)](https://dashboard.cypress.io/projects/3hrkhu/runs)
[![OpenSSF Best Practices](https://www.bestpractices.dev/projects/223/badge)](https://www.bestpractices.dev/projects/223)
![GitHub stars](https://img.shields.io/github/stars/juice-shop/juice-shop.svg?label=GitHub%20%E2%98%85&style=flat)
[![Static Badge](https://img.shields.io/badge/OWASP-Code_of_Conduct-blue)](https://github.com/juice-shop/juice-shop/blob/717f5efa886b0417eeecb52378b03cf10567b7e2/CODE_OF_CONDUCT.md)

> [网上最值得信赖的商店。](https://twitter.com/dschadow/status/706781693504589824)（[@dschadow](https://github.com/dschadow)）—
> [整个互联网最好的果汁店！](https://twitter.com/shehackspurple/status/907335357775085568)（[@shehackspurple](https://twitter.com/shehackspurple)）—
> [实际上是现存漏洞最少的漏洞应用！](https://youtu.be/TXAztSpYpvE?t=26m35s)（[@vanderaj](https://twitter.com/vanderaj)）—
> [一开始你 😂😂，然后你 😢](https://twitter.com/kramse/status/1073168529405472768)（[@kramse](https://twitter.com/kramse)）—
> [但这和果汁完全没有关系。](https://twitter.com/coderPatros/status/1199268774626488320)（[@coderPatros 的妻子](https://twitter.com/coderPatros)）

OWASP Juice Shop 是一款刻意不安全的 Web 应用。项目称它可能是最现代、最复杂的不安全 Web 应用！它可用于安全培训、意识演示、CTF 以及作为安全工具的测试对象。Juice Shop 涵盖了整个 [OWASP Top Ten](https://owasp.org/www-project-top-ten) 中的漏洞，以及真实应用中的许多其他安全缺陷。

![Juice Shop 截图轮播](https://raw.githubusercontent.com/juice-shop/juice-shop/717f5efa886b0417eeecb52378b03cf10567b7e2/screenshots/slideshow.gif)

详细介绍、完整功能列表及架构概览，请访问官方项目页面：<https://owasp-juice.shop>

## 目录

- [设置](#设置)
    - [从源代码](#从源代码)
    - [打包发行版](#打包发行版)
    - [Docker 容器](#docker-容器)
    - [Vagrant](#vagrant)
- [演示](#演示)
- [文档](#文档)
    - [Node.js 版本兼容性](#nodejs-版本兼容性)
    - [故障排查](#故障排查)
    - [官方配套指南](#官方配套指南)
- [参与贡献](#参与贡献)
- [参考资料](#参考资料)
- [周边商品](#周边商品)
- [捐赠](#捐赠)
- [贡献者](#贡献者)
- [许可](#许可)

## 设置

> 有关一些不常见的安装方式及在多种云计算服务商上运行 Juice Shop 的说明，请参阅[《运行 OWASP Juice Shop》文档](https://pwning.owasp-juice.shop/companion-guide/latest/part1/running.html)。

> 某些挑战需要 AI/LLM 服务提供商才能正常工作。有关配置本地或云端 AI 提供商的说明，请参阅[《设置外部依赖》文档](https://pwning.owasp-juice.shop/companion-guide/snapshot/part1/running.html#_setting_up_external_dependencies)。

### 从源代码

![GitHub repo size](https://img.shields.io/github/repo-size/juice-shop/juice-shop.svg)

1. 安装 [node.js](#nodejs-版本兼容性)
2. 运行 `git clone https://github.com/juice-shop/juice-shop.git --depth 1`（或克隆[自己的 fork](https://github.com/juice-shop/juice-shop/fork)）
3. 使用 `cd juice-shop` 进入克隆的文件夹
4. 运行 `npm install`（只需在首次启动前或修改源代码后执行）
5. 运行 `npm start`
6. 在浏览器访问 <http://localhost:3000>

### 打包发行版

[![GitHub release](https://img.shields.io/github/downloads/juice-shop/juice-shop/total.svg)](https://github.com/juice-shop/juice-shop/releases/latest)
[![SourceForge](https://img.shields.io/sourceforge/dm/juice-shop?label=sourceforge%20downloads)](https://sourceforge.net/projects/juice-shop/)
[![SourceForge](https://img.shields.io/sourceforge/dt/juice-shop?label=sourceforge%20downloads)](https://sourceforge.net/projects/juice-shop/)

1. 在 Windows、MacOS 或 Linux 机器上安装 64 位 [node.js](#nodejs-版本兼容性)
2. 下载[最新发布版](https://github.com/juice-shop/juice-shop/releases/latest)附带的 `juice-shop-<version>_<node-version>_<os>_x64.zip`（或 `.tgz`）
3. 解压并使用 `cd` 进入解压后的文件夹
4. 运行 `npm start`
5. 在浏览器访问 <http://localhost:3000>

> 每个打包发行版都包含针对运行 `npm install` 时的操作系统和 node.js 版本构建的 `sqlite3` 与 `libxmljs2` 二进制文件。

### Docker 容器

[![Docker Pulls](https://img.shields.io/docker/pulls/bkimminich/juice-shop.svg)](https://hub.docker.com/r/bkimminich/juice-shop)
![Docker Stars](https://img.shields.io/docker/stars/bkimminich/juice-shop.svg)
[![](https://images.microbadger.com/badges/image/bkimminich/juice-shop.svg)](https://microbadger.com/images/bkimminich/juice-shop
"Get your own image badge on microbadger.com")
[![](https://images.microbadger.com/badges/version/bkimminich/juice-shop.svg)](https://microbadger.com/images/bkimminich/juice-shop
"Get your own version badge on microbadger.com")

1. 安装 [Docker](https://www.docker.com)
2. 运行 `docker pull bkimminich/juice-shop`
3. 运行 `docker run --rm -p 127.0.0.1:3000:3000 bkimminich/juice-shop`
4. 在浏览器访问 <http://localhost:3000>（在 macOS 和 Windows 上，如果使用 docker-machine 而不是原生 Docker，请访问 <http://192.168.99.100:3000>）

### Vagrant

1. 安装 [Vagrant](https://www.vagrantup.com/downloads.html) 和 [Virtualbox](https://www.virtualbox.org/wiki/Downloads)
2. 运行 `git clone https://github.com/juice-shop/juice-shop.git`（或克隆[自己的 fork](https://github.com/juice-shop/juice-shop/fork)）
3. 运行 `cd vagrant && vagrant up`
4. 访问 [192.168.56.110](http://192.168.56.110)

## 演示

欢迎查看最新版 OWASP Juice Shop：<http://demo.owasp-juice.shop>

> 这只是部署测试和预览实例！你__不应__使用此实例开展自己的黑客活动！不保证在线时间！如果把它弄坏，肯定会收到严肃的眼神！

## 文档

### Node.js 版本兼容性

![GitHub package.json dynamic](https://img.shields.io/github/package-json/cpu/juice-shop/juice-shop)
![GitHub package.json dynamic](https://img.shields.io/github/package-json/os/juice-shop/juice-shop)

OWASP Juice Shop 尽可能按照官方 [node.js LTS 时间表](https://github.com/nodejs/LTS)支持以下 [node.js](http://nodejs.org) 版本，并相应提供 Docker 镜像和打包发行版。

| node.js | 支持 | 测试 | [打包发行版](#打包发行版) | `master` 的 [Docker 镜像](#docker-容器) | `develop` 的 [Docker 镜像](#docker-容器) |
|:--------|:-----------------------|:-------------------|:--------------------------------------------------|:-------------------------------------------------|:--------------------------------------------------|
| 26.x    | :heavy_check_mark:     | :heavy_check_mark: |                                                   |                                                  |                                                   |
| 25.x    | ( :heavy_check_mark: ) | :x:                |                                                   |                                                  |                                                   |
| 24.x    | :heavy_check_mark:     | :heavy_check_mark: | Windows (`x64`), MacOS (`x64`), Linux (`x64`)     | `latest` (`linux/amd64`, `linux/arm64`)          | `snapshot` (`linux/amd64`, `linux/arm64`)         |
| 23.x    | :x:                    | :x:                |                                                   |                                                  |                                                   |
| 22.x    | :heavy_check_mark:     | :heavy_check_mark: |                                                   |                                                  |                                                   |
| <22.x   | :x:                    | :x:                |                                                   |                                                  |                                                   |

Juice Shop 只针对上述每个 node.js 版本中_最新的 `.x` 次版本_进行自动测试！不保证较旧的 node.js 次版本始终能与 Juice Shop 一起工作。请确保所选版本保持最新。

### 故障排查

[![Gitter](http://img.shields.io/badge/gitter-join%20chat-1dce73.svg)](https://gitter.im/bkimminich/juice-shop)

如果需要应用设置方面的帮助，请先查看[现有故障排查指南](https://pwning.owasp-juice.shop/companion-guide/latest/part4/troubleshooting.html)。如果仍不能解决，请在 [Gitter 聊天](https://gitter.im/bkimminich/juice-shop)中提出具体问题，社区成员会尽力提供帮助。

:stop_sign: **请避免为支持请求或问题打开 GitHub issue！**

### 官方配套指南

[![Write Goodreads Review](https://img.shields.io/badge/goodreads-write%20review-49557240.svg)](https://www.goodreads.com/review/edit/49557240)

OWASP Juice Shop 提供官方配套电子书，完整介绍应用中的所有漏洞，包括如何发现和利用它们的提示。附录还包含每个挑战的完整分步解答。书中还提供[自定义品牌](https://pwning.owasp-juice.shop/companion-guide/latest/part4/customization.html)、[CTF 支持](https://pwning.owasp-juice.shop/companion-guide/latest/part4/ctf.html)、[培训师指南](https://pwning.owasp-juice.shop/companion-guide/latest/part4/trainers.html)等大量文档。

[Pwning OWASP Juice Shop](https://leanpub.com/juice-shop) 采用 [CC BY-NC-ND 4.0](https://creativecommons.org/licenses/by-nc-nd/4.0/) 许可，可在 LeanPub **免费**获取 PDF、Kindle 和 ePub 格式，也可以[在线浏览全部内容](https://pwning.owasp-juice.shop)！

[<img alt="Pwning OWASP Juice Shop cover" src="https://raw.githubusercontent.com/juice-shop/pwning-juice-shop/master/docs/modules/ROOT/assets/images/cover.jpg" width="200"/>](https://leanpub.com/juice-shop)
[<img alt="Pwning OWASP Juice Shop back cover" src="https://raw.githubusercontent.com/juice-shop/pwning-juice-shop/master/docs/modules/ROOT/assets/images/introduction/back.jpg" width="200"/>](https://leanpub.com/juice-shop)

## 参与贡献

[![GitHub contributors](https://img.shields.io/github/contributors/juice-shop/juice-shop.svg)](https://github.com/juice-shop/juice-shop/graphs/contributors)
[![JavaScript Style Guide](https://img.shields.io/badge/code%20style-standard-brightgreen.svg)](http://standardjs.com/)
[![Crowdin](https://d322cqt584bo4o.cloudfront.net/owasp-juice-shop/localized.svg)](https://crowdin.com/project/owasp-juice-shop)
![GitHub issues by-label](https://img.shields.io/github/issues/juice-shop/juice-shop/help%20wanted.svg)
![GitHub issues by-label](https://img.shields.io/github/issues/juice-shop/juice-shop/good%20first%20issue.svg)

我们一直欢迎新贡献者加入！请查看 [CONTRIBUTING.md](https://github.com/juice-shop/juice-shop/blob/717f5efa886b0417eeecb52378b03cf10567b7e2/CONTRIBUTING.md)，了解如何[为代码库贡献](https://github.com/juice-shop/juice-shop/blob/717f5efa886b0417eeecb52378b03cf10567b7e2/CONTRIBUTING.md#code-contributions)或[翻译成不同语言](https://github.com/juice-shop/juice-shop/blob/717f5efa886b0417eeecb52378b03cf10567b7e2/CONTRIBUTING.md#i18n-contributions)。

## 参考资料

你是否撰写过关于或提及 OWASP Juice Shop 的博客、杂志文章或播客？或者举办或参与过提及本项目的会议演讲、聚会、黑客工作坊或公开培训？

请 fork 仓库并提交 Pull Request，将其加入不断增长的 [REFERENCES.md](https://github.com/juice-shop/juice-shop/blob/717f5efa886b0417eeecb52378b03cf10567b7e2/REFERENCES.md) 列表！

## 周边商品

* 在 [Spreadshirt.com](http://shop.spreadshirt.com/juiceshop) 和 [Spreadshirt.de](http://shop.spreadshirt.de/juiceshop) 可获取印有官方 OWASP Juice Shop 标志的周边（T 恤、连帽衫、马克杯）
* 在 [StickerYou.com](https://www.stickeryou.com/products/owasp-juice-shop/794) 可获取多种 OWASP Juice Shop 标志单张贴纸来装饰笔记本电脑，也可以印制磁贴、熨烫贴、贴纸页和临时纹身。

## 捐赠

[![](https://img.shields.io/badge/support-owasp%20juice%20shop-blue)](https://owasp.org/donate/?reponame=www-project-juice-shop&title=OWASP+Juice+Shop)

OWASP Foundation 通过 Stripe 接受捐赠。Juice Shop 等项目随后可以向基金会申请费用报销。如果想表达对 Juice Shop 项目的支持，请务必勾选捐赠表单上的“Publicly list me as a supporter of OWASP Juice Shop”复选框。有关捐赠及用途的更多信息见：

<https://pwning.owasp-juice.shop/companion-guide/latest/part3/donations.html>

## 贡献者

OWASP Juice Shop 项目负责人：

- [Björn Kimminich](https://github.com/bkimminich) aka `bkimminich` [![Keybase PGP](https://img.shields.io/keybase/pgp/bkimminich)](https://keybase.io/bkimminich)
- [Jannik Hollenbach](https://github.com/J12934) aka `J12934`

完整贡献者列表请参阅 [HALL_OF_FAME.md](https://github.com/juice-shop/juice-shop/blob/717f5efa886b0417eeecb52378b03cf10567b7e2/HALL_OF_FAME.md)。

## 许可

[![license](https://img.shields.io/github/license/juice-shop/juice-shop.svg)](https://github.com/juice-shop/juice-shop/blob/717f5efa886b0417eeecb52378b03cf10567b7e2/LICENSE)

本程序是自由软件：你可以根据 [MIT 许可证](https://github.com/juice-shop/juice-shop/blob/717f5efa886b0417eeecb52378b03cf10567b7e2/LICENSE)条款重新分发和/或修改。OWASP Juice Shop 及所有贡献的版权 © 2014–2026 归 Bjoern Kimminich 和 OWASP Juice Shop 贡献者所有。

![Juice Shop 标志](https://raw.githubusercontent.com/juice-shop/juice-shop/717f5efa886b0417eeecb52378b03cf10567b7e2/frontend/src/assets/public/images/JuiceShop_Logo_400px.png)
