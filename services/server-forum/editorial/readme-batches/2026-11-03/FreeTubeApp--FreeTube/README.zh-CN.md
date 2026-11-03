<p align="center">
 <img alt="FreeTube 标志" src="https://raw.githubusercontent.com/FreeTubeApp/FreeTube/a0dd9bd6c19715f06bd5ae6fe3c1087addf30b31/_icons/logoColor.svg" width=500 align="center">
</p>

FreeTube 是一款注重隐私的开源桌面 YouTube 播放器。项目称它可以让你不受广告干扰地使用 YouTube，并避免 Google 通过 Cookie 和 JavaScript 跟踪你。借助 Electron，它支持 Windows（10 及更高版本）、Mac（macOS 13 及更高版本）和 Linux。

<p align="center"><a href="https://github.com/FreeTubeApp/FreeTube/releases">下载 FreeTube</a></p>
<p align="center">
  <a href="https://github.com/FreeTubeApp/FreeTube/actions/workflows/build.yml"><img alt="构建状态" src="https://github.com/FreeTubeApp/FreeTube/actions/workflows/build.yml/badge.svg?branch=development" /></a>
  <a href="https://hosted.weblate.org/engage/free-tube/"><img src="https://hosted.weblate.org/widgets/free-tube/-/svg-badge.svg" alt="翻译状态" /></a>
</p>

<hr>
<p align="center"><a href="#截图">截图</a> &bull; <a href="#它如何工作">工作原理</a> &bull; <a href="#功能">功能</a> &bull; <a href="#下载链接">下载链接</a> &bull; <a href="#参与贡献">参与贡献</a> &bull; <a href="#本地化">本地化</a> &bull; <a href="#联系">联系</a> &bull; <a href="#捐赠">捐赠</a> &bull; <a href="#许可证">许可证</a></p>
<p align="center"><a href="https://freetubeapp.io/">网站</a> &bull; <a href="https://docs.freetubeapp.io/">文档</a> &bull; <a href="https://docs.freetubeapp.io/faq/">常见问题</a> &bull; <a href="https://github.com/FreeTubeApp/FreeTube/discussions">讨论</a></p>
<hr>

> [!NOTE]
> FreeTube 目前处于 Beta 阶段。虽然对大多数用户而言应该能够正常工作，但仍有错误和缺失功能需要解决。
>
> 如果你有想法或发现错误，请提交 [GitHub issue](https://github.com/FreeTubeApp/FreeTube/issues/new/choose)以便我们跟踪。提交前请[搜索现有问题](https://github.com/FreeTubeApp/FreeTube/issues?q=is%3Aissue+sort%3Arelevance-desc)，避免重复。

## 截图
| FreeTube 主窗口 |
|---|
| ![](https://raw.githubusercontent.com/FreeTubeApp/FreeTubeApp.io/master/src/images/FreeTube1.png) |

| 观看视频 |
|---|
| ![](https://raw.githubusercontent.com/FreeTubeApp/FreeTubeApp.io/master/src/images/FreeTube2.png) |

| 设置 |
|---|
| ![](https://raw.githubusercontent.com/FreeTubeApp/FreeTubeApp.io/master/src/images/FreeTube3.png) |

## 它如何工作？

FreeTube 使用内置提取器获取并提供数据和视频，也可以选择使用 [Invidious API](https://github.com/iv-org/invidious)。FreeTube 不使用任何官方 API 获取数据。项目说明称，虽然 YouTube 仍能看到你的视频请求，但不能再使用 Cookie 或 JavaScript 跟踪你。你的订阅、播放列表和历史记录存储在本地计算机上，永远不会被发送出去。

> [!IMPORTANT]
> 项目强烈建议使用 VPN 或 Tor，以在使用 FreeTube 时隐藏 IP。

## 功能
* 无广告观看视频
* 使用 YouTube 时避免 Google 使用 Cookie 和 JavaScript 跟踪
* 两种提取器 API 可选（内置或 Invidious）
* 无需账号订阅频道
* 连接到外部配置的代理，例如 Tor
* 查看和搜索本地订阅、播放列表及历史记录
* 将订阅整理为“个人资料”，创建更聚焦的信息流
* 导出和导入订阅
* YouTube 热门趋势
* YouTube 章节
* 基于所设置 Invidious 实例的最热视频页面
* SponsorBlock
* DeArrow
* 通过扩展从浏览器直接在 FreeTube 中打开视频
* 使用外部播放器观看视频
* 完整的主题支持
* 截取视频画面
* 多窗口
* 迷你播放器（画中画）
* 键盘快捷键
* 可选择仅显示适合家庭观看的内容
* 使用免干扰设置显示或隐藏应用内功能或元素
* 查看频道帖子

### 浏览器扩展
以下扩展可直接在 FreeTube 中打开 YouTube 链接：

- [LibRedirect](https://libredirect.manerakai.com/)
- [RedirectTube](https://github.com/MStankiewiczOfficial/RedirectTube)

LibRedirect 会自动将 YouTube 链接重定向到 FreeTube。
> [!IMPORTANT]
> 为确保正常运行，请在扩展的 Services 设置中选择 FreeTube 作为前端。

RedirectTube 不会自动在 FreeTube 中打开 YouTube 链接（尽管可以在设置中启用该功能）。它会在工具栏和右键菜单中添加按钮，供你点击后手动在 FreeTube 中打开视频。

- 从 [Mozilla Add-ons](https://addons.mozilla.org/firefox/addon/libredirect/)（基于 Firefox 的浏览器）或[开发者网站](https://libredirect.manerakai.com/download_chromium.html)（Chrome 及基于 Chromium 的浏览器）下载 LibRedirect。
- 从 [Mozilla Add-ons](https://addons.mozilla.org/firefox/addon/redirecttube/)（基于 Firefox 的浏览器）或 [Chrome Web Store](https://chromewebstore.google.com/detail/redirecttube/jpbaggklodpddjcadlebabhiopjkjfjh)（Chrome 及基于 Chromium 的浏览器）下载 RedirectTube。

> [!NOTE]
> 这些扩展不能用于 Linux 便携版！
>
> 如果扩展与 FreeTube 配合使用时出现问题，请在本仓库而非扩展仓库中创建 issue。

## 下载链接
### 官方下载

> [!CAUTION]
> FreeTube 仅支持 Windows 10 及更高版本、macOS 13 及更高版本以及多种 Linux 发行版。在不支持的系统上安装可能导致意外问题。

* [GitHub Releases](https://github.com/FreeTubeApp/FreeTube/releases)
* [FreeTube 网站](https://freetubeapp.io/#download)
* Flathub 上的 Flatpak：[下载](https://flathub.org/apps/details/io.freetubeapp.FreeTube)和[源代码](https://github.com/flathub/io.freetubeapp.FreeTube)

#### 自动构建（每夜 / 每周）
> [!WARNING]
> 使用这些构建的风险由使用者承担。它们是预发布版本，仅供希望提前测试更改并愿意接受不同构建之间可能出现故障的人使用。

这些构建通过 [GitHub Actions](https://github.com/FreeTubeApp/FreeTube/actions?query=workflow%3ABuild) 根据开发分支的变更自动创建。

第一个带绿色勾号的构建即最新构建。

> [!IMPORTANT]
> 下载这些构建需要 GitHub 账号。

### FreeTube 团队个人成员维护的项目
* Homebrew FreeTube（仅限 Apple Silicon）：[安装](https://github.com/PikachuEXE/homebrew-FreeTube)
* FreeTubeAndroid（FreeTube 的 Android 和 PWA 移植版）：[下载](https://github.com/MarmadileManteater/FreeTubeAndroid/releases)和[源代码](https://github.com/MarmadileManteater/FreeTubeAndroid)

> [!IMPORTANT]
> 这些项目与 FreeTube 有关，由 FreeTube 团队的个人成员维护。虽然它们不属于主要的 FreeTube 项目，但可能对用户有用。与官方构建相比，使用这些项目时可能出现问题。此类构建特有的问题应报告给各自的维护者。向我们报告问题前，请务必先尝试[官方下载](https://github.com/freetubeapp/freetube/#official-downloads)！

## 参与贡献

非常感谢使 FreeTube 成为可能的[人员和项目](https://docs.freetubeapp.io/credits/)！

如果你喜欢动手并希望作贡献，我们非常欢迎你的帮助。提交 pull request 后，会有人审查你的代码。

> [!IMPORTANT]
> 提交 pull request 前，请遵循[贡献指南](https://github.com/FreeTubeApp/FreeTube/blob/a0dd9bd6c19715f06bd5ae6fe3c1087addf30b31/CONTRIBUTING.md)。

## 本地化
<a href="https://hosted.weblate.org/engage/free-tube/">
<img src="https://hosted.weblate.org/widgets/free-tube/-/287x66-grey.png" alt="翻译状态" />
</a>

我们一直在积极寻找译者！我们使用 [Weblate](https://hosted.weblate.org/engage/free-tube/)方便译者参与。点击上方徽章了解如何参与。

对于 Linux Flatpak，桌面条目注释字符串可以在我们的 [Flatpak 仓库](https://github.com/flathub/io.freetubeapp.FreeTube/blob/master/io.freetubeapp.FreeTube.desktop)翻译。

## 联系

如果有任何问题，欢迎在我们的[讨论](https://github.com/FreeTubeApp/FreeTube/discussions)页面提问。也可以加入我们的 [Matrix Space](https://matrix.to/#/#freetubeapp:matrix.org)。

> [!IMPORTANT]
> 加入前别忘了查看[规则](https://docs.freetubeapp.io/community/matrix/)。

## 捐赠

如果你喜欢 FreeTube，欢迎通过以下方式捐赠。

* 比特币地址：`1Lih7Ho5gnxb1CwPD4o59ss78pwo2T91eS`

虽然我们非常感谢捐赠，但请只在你真正愿意时捐赠。捐款用于维持网站运行以及可能产生的代码签名费用。

> [!TIP]
> 如果使用 Invidious API，我们建议向你使用的实例捐赠。也可以向 [Invidious 团队](https://invidious.io/donate/)或[本地 API 开发者](https://github.com/sponsors/LuanRT)捐赠。

## 许可证
[![GNU AGPLv3 图片](https://www.gnu.org/graphics/agplv3-155x51.png)](https://www.gnu.org/licenses/agpl-3.0.html)

FreeTube 是自由软件：你可以按自己的意愿使用、研究、分享和改进它。具体而言，你可以根据自由软件基金会发布的 [GNU Affero 通用公共许可证](https://www.gnu.org/licenses/agpl-3.0.html)第 3 版，或由你选择的任何更高版本的条款，重新分发和/或修改它。
