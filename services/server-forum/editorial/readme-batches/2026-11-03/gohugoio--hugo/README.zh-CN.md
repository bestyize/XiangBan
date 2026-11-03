[Git]: https://git-scm.com/book/en/v2/Getting-Started-Installing-Git
[Go]: https://go.dev/doc/install
[bep]: https://github.com/bep
[bugs]: https://github.com/gohugoio/hugo/issues?q=is%3Aopen+is%3Aissue+label%3ABug
[contributing]: https://github.com/gohugoio/hugo/blob/185a609ec26020ac97b0a3f78c9ca299e20e7477/CONTRIBUTING.md
[create a proposal]: https://github.com/gohugoio/hugo/issues/new?labels=Proposal%2C+NeedsTriage&template=feature_request.md
[details]: https://gohugo.io/host-and-deploy/deploy-with-hugo-deploy/
[documentation repository]: https://github.com/gohugoio/hugoDocs
[documentation]: https://gohugo.io/documentation
[dragonfly bsd, freebsd, netbsd, and openbsd]: https://gohugo.io/installation/bsd
[features]: https://gohugo.io/about/features/
[forum]: https://discourse.gohugo.io
[friends]: https://github.com/gohugoio/hugo/graphs/contributors
[hugo modules]: https://gohugo.io/hugo-modules/
[installation]: https://gohugo.io/installation
[issue queue]: https://github.com/gohugoio/hugo/issues
[linux]: https://gohugo.io/installation/linux
[macos]: https://gohugo.io/installation/macos
[prebuilt binary]: https://github.com/gohugoio/hugo/releases/latest
[requesting help]: https://discourse.gohugo.io/t/requesting-help/9132
[spf13]: https://github.com/spf13
[static site generator]: https://en.wikipedia.org/wiki/Static_site_generator
[support]: https://discourse.gohugo.io
[themes]: https://themes.gohugo.io/
[website]: https://gohugo.io
[windows]: https://gohugo.io/installation/windows

<a href="https://gohugo.io/"><img src="https://raw.githubusercontent.com/gohugoio/gohugoioTheme/master/static/images/hugo-logo-wide.svg?sanitize=true" alt="Hugo" width="565"></a>

由 [bep][]、[spf13][] 和[朋友们][friends]用 Go 用心构建的快速、灵活的静态网站生成器。

---

[![GoDoc](https://godoc.org/github.com/gohugoio/hugo?status.svg)](https://godoc.org/github.com/gohugoio/hugo)
[![Tests on Linux, MacOS and Windows](https://github.com/gohugoio/hugo/workflows/Test/badge.svg)](https://github.com/gohugoio/hugo/actions?query=workflow%3ATest)

[官网][website] | [安装][installation] | [文档][documentation] | [支持][support] | [参与贡献][contributing] | <a rel="me" href="https://fosstodon.org/@gohugoio">Mastodon</a>

## 概览

Hugo 是一个用 Go 编写的[静态网站生成器][static site generator]，针对速度进行优化，并以灵活性为设计目标。借助先进的模板系统和快速的资源处理流水线，Hugo 可以在数秒甚至更短时间内渲染完整网站。

凭借灵活的框架、多语言支持和强大的分类系统，Hugo 被广泛用于创建：

- 企业、政府、非营利组织、教育、新闻、活动和项目网站
- 文档网站
- 图片作品集
- 落地页
- 商业、专业及个人博客
- 简历

开发时可使用 Hugo 内置的 Web 服务器，即时查看内容、结构、行为和展示效果的变化。之后可以将网站部署到主机，或将更改推送到 Git 服务以自动构建和部署。

Hugo 的快速资源处理流水线包括：

- CSS 处理 &ndash; 打包、转换、压缩、创建 source map、计算 SRI 哈希并与 PostCSS 集成。
- 图片处理 &ndash; 转换、调整大小、裁剪、旋转、调整颜色、应用滤镜、叠加文字和图片，以及提取元数据
- JavaScript 打包 &ndash; 将 TypeScript 和 JSX 转译为 JavaScript，打包、tree shake、压缩、创建 source map 并计算 SRI 哈希。
- Sass 处理 &ndash; 将 Sass 转译为 CSS，打包、tree shake、压缩、创建 source map、计算 SRI 哈希并与 PostCSS 集成
- Tailwind CSS 处理 &ndash; 将 Tailwind CSS 工具类编译为标准 CSS，打包、tree shake、优化、压缩、计算 SRI 哈希并与 PostCSS 集成

通过 [Hugo Modules][]，你还可以通过公开或私有 Git 仓库与其他项目共享内容、资源、数据、翻译、主题、模板和配置。

有关 Hugo 能力的完整概述，请参阅文档中的[功能][features]部分。

## 赞助商

<p>&nbsp;</p>
<p float="left">
  <a href="https://www.jetbrains.com/go/?utm_source=OSS&utm_medium=referral&utm_campaign=hugo" target="_blank"><img src="https://raw.githubusercontent.com/gohugoio/hugoDocs/master/assets/images/sponsors/goland.svg" width="200" alt="The complete IDE crafted for professional Go developers."></a>
  &nbsp;&nbsp;&nbsp;
    <a href="https://cloudcannon.com/hugo-cms/?utm_campaign=HugoSponsorship&utm_source=sponsor&utm_content=gohugo" target="_blank"><img src="https://raw.githubusercontent.com/gohugoio/hugoDocs/master/assets/images/sponsors/cloudcannon-cms-logo.svg" width="200" alt="CloudCannon"></a>
</p>

## 版本

Hugo 有两个版本。除非需要额外功能，否则请使用标准版。

功能|standard|deploy
:--|:-:|:-:
核心功能|:heavy_check_mark:|:heavy_check_mark:
直接云部署 (1)|:x:|:heavy_check_mark:

(1) 将网站直接部署到 Google Cloud Storage 存储桶、AWS S3 存储桶或 Azure Storage 容器。参见&nbsp;[详情][details]。

## 安装

从[预构建二进制文件][prebuilt binary]、包管理器或软件包仓库安装 Hugo。请查看适合你的操作系统的安装说明：

- [macOS][]
- [Linux][]
- [Windows][]
- [DragonFly BSD, FreeBSD, NetBSD, and OpenBSD][]

## 从源代码构建

从源代码构建 Hugo 必须安装：

1. [Git][]
1. [Go][] 1.27.0 或更高版本

### 标准版

构建并安装标准版：

```sh
CGO_ENABLED=0 go install github.com/gohugoio/hugo@latest
```

### Deploy 版

构建并安装 Deploy 版：

```sh
CGO_ENABLED=0 go install -tags withdeploy github.com/gohugoio/hugo@latest
```

## 文档

Hugo 的[文档][documentation]包含安装说明、快速入门指南、概念说明、参考信息和示例。

请将文档问题和 Pull Request 提交到[文档仓库][documentation repository]。

## 支持

请**不要使用问题列表**提问或排查故障。除非你确定问题是软件缺陷，否则请使用[论坛][forum]。

Hugo 的[论坛][forum]是一个活跃的用户和开发者社区，大家在那里解答问题、分享知识并提供示例。快速搜索超过 20,000 个主题通常就能找到答案。第一次提问前，请务必阅读[如何请求帮助][requesting help]。

## 参与贡献

你可以通过以下方式为 Hugo 项目贡献：

- 在[论坛][forum]回答问题
- 改进[文档][documentation]
- 关注[问题列表][issue queue]
- 创建或改进[主题][themes]
- 修复[错误][bugs]

请将文档问题和 Pull Request 提交到[文档仓库][documentation repository]。

如果你有改进或新功能的想法，请在[论坛][forum]的“Feature”分类中创建新主题。这有助于：

- 确认该功能是否已经存在
- 衡量关注度
- 完善构想

如果有足够的关注，请[创建提案][create a proposal]。在项目负责人接受提案之前，请勿提交 Pull Request。

有关参与 Hugo 的完整指南，请参阅[贡献指南](https://github.com/gohugoio/hugo/blob/185a609ec26020ac97b0a3f78c9ca299e20e7477/CONTRIBUTING.md).

## 许可证

Hugo 源代码的许可请参阅 [LICENSE](https://github.com/gohugoio/hugo/blob/185a609ec26020ac97b0a3f78c9ca299e20e7477/LICENSE).

我们还以二进制/WASM 形式捆绑了一些库：

- [libwebp](https://github.com/webmproject/libwebp), [BSD-3-Clause 许可证](https://github.com/webmproject/libwebp?tab=BSD-3-Clause-1-ov-file#readme)
- [Katex](https://github.com/KaTeX/KaTeX), [MIT 许可证](https://github.com/KaTeX/KaTeX?tab=MIT-1-ov-file#readme)
- [QuickJS](https://github.com/bellard/quickjs?tab=License-1-ov-file#readme), [License](https://github.com/bellard/quickjs?tab=License-1-ov-file#readme)
