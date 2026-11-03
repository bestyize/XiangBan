> 本文依据 [LosslessCut 固定版本 README](https://github.com/mifi/lossless-cut/blob/6fc47dd4cc81b72a8f1b3132c47a99880cd9bbdb/README.md) 翻译；项目采用 [GPL-2.0](https://github.com/mifi/lossless-cut/blob/6fc47dd4cc81b72a8f1b3132c47a99880cd9bbdb/LICENSE)，图标另有署名许可，详见文末。

<div align="center">
<br>
<p><a href="https://losslesscut.app/"><img src="https://raw.githubusercontent.com/mifi/lossless-cut/6fc47dd4cc81b72a8f1b3132c47a99880cd9bbdb/src/renderer/src/icon.svg" width="120" alt="LosslessCut" /></a></p>
<p><a href="https://losslesscut.app/"><b>LosslessCut</b></a></p>
无损视频与音频编辑的瑞士军刀
<br>
<img src="https://github.com/mifi/lossless-cut/workflows/Build/release/badge.svg" />
<a href="https://paypal.me/mifino/usd"><img src="https://img.shields.io/badge/Donate-PayPal-green.svg" /></a> <a href="https://github.com/mifi/lossless-cut#download"><img src="https://img.shields.io/github/v/release/mifi/lossless-cut" /></a> <a href="https://discord.gg/fhnEREfUJ3"><img src="https://img.shields.io/discord/986051448385183804" /></a> <a href="https://twitter.com/losslesscut"><img src="https://img.shields.io/twitter/follow/losslesscut?label=Twitter&style=social" alt="Twitter"></a> <a href="https://bsky.app/profile/losslesscut.app"><img src="https://img.shields.io/badge/Bluesky-losslesscut.app-0085ff?logo=bluesky&logoColor=white" alt="Bluesky"></a> <a href="https://mastodon.social/@losslesscut"><img src="https://img.shields.io/badge/Mastodon-@losslesscut-6364ff?logo=mastodon&logoColor=white" alt="Mastodon"></a> <a href="https://www.instagram.com/losslesscut.app/"><img src="https://img.shields.io/badge/Instagram-losslesscut.app-e4405f?logo=instagram&logoColor=white" alt="Instagram"></a>
<br><br>
感谢[支持者](https://mifi.no/thanks/)和所有购买 LosslessCut 的人！
<br><br>
<p align="center"><img src="https://static.mifi.no/media/InHeghy_4DwVeujK/2000.jpg" width="600" alt="截图" /></p>
<br><br>
</div>

LosslessCut 致力于成为跨平台的 FFmpeg 图形界面，让视频、音频、字幕和其他相关媒体文件的操作极快且无损。主要功能是无损裁剪音视频文件，非常适合粗剪摄像机、GoPro 或无人机等拍摄的大型视频以节省空间。它可让你快速提取视频中的精彩部分并丢弃数 GB 数据，无须缓慢地重新编码并因此损失画质。它还有许多其他[用途](https://losslesscut.app/docs/features#example-use-cases)。由于几乎直接复制数据，所有操作都很快；繁重工作由出色的 FFmpeg 完成。

## 目录

- [功能](#features)
- [下载](#download)
- [支持的格式](#supported-formats)
- [文档、使用与入门](#documentation)
- [视频演示](#video-demos)
- [相关报道](#featured)
- [署名](#attributions)

<a id="features"></a>
## 功能

<!-- 以下仅为亮点，完整列表见 docs/features.md；请与网站首页同步。 -->
- 在几秒内无损裁剪视频和音频
- 无损合并剪切片段或编解码器相同的完整文件
- 添加、删除、替换或提取音轨、字幕等轨道
- 无须重新编码即可更换格式（重新封装），例如从 MKV 转为 MP4
- 片段、标记和标签；支持导入或导出章节、CSV、EDL 等
- 检测场景、黑帧及静音，并据此拆分
- 编辑元数据和旋转角度，创建章节
- 打开几乎任何格式，包括播放器无法播放的格式
- 截图和帧导出、键盘快捷键、[命令行界面](https://losslesscut.app/docs/cli)以及 [HTTP API](https://losslesscut.app/docs/api)

**[所有功能和使用示例 →](https://losslesscut.app/docs/features)**

另见[操作方法集](https://losslesscut.app/docs/recipes)。

<a id="download"></a>
## 下载

如果你想支持我持续开发 LosslessCut，同时希望通过安全、简单的安装流程获得自动且稳定的更新，可以考虑从你喜欢的商店获取：

<a href="https://apps.apple.com/app/id1505323402"><img src="https://raw.githubusercontent.com/mifi/lossless-cut/6fc47dd4cc81b72a8f1b3132c47a99880cd9bbdb/mac-app-store-badge.svg" alt="Mac App Store" height="50"/></a> <a href="https://www.microsoft.com/store/apps/9P30LSR4705L?cid=storebadge&ocid=badge"><img src="https://raw.githubusercontent.com/mifi/lossless-cut/6fc47dd4cc81b72a8f1b3132c47a99880cd9bbdb/ms-store-badge.svg" alt="MS badge" height="50"/></a>

Linux 版本发布在 Snap Store：

<a href="https://snapcraft.io/losslesscut"><img src="https://github.com/mifi/lossless-cut/raw/master/snap-store-black.svg?sanitize=true" alt="Snapcraft" height="50"/></a>

如果更喜欢手动下载可执行文件，这当然始终免费（另见[支持的操作系统](https://losslesscut.app/docs/requirements)）：

- macOS：[Intel](https://github.com/mifi/lossless-cut/releases/latest/download/LosslessCut-mac-x64.dmg) / [Apple Silicon](https://github.com/mifi/lossless-cut/releases/latest/download/LosslessCut-mac-arm64.dmg) DMG（注意 PKG **不可用**）
- Windows：[7zip](https://github.com/mifi/lossless-cut/releases/latest/download/LosslessCut-win-x64.7z)（Windows 7、8 和 8.1 在 [v3.50.0 后不再受支持](https://losslesscut.app/docs/requirements)）
- Linux：[x64 tar.bz2](https://github.com/mifi/lossless-cut/releases/latest/download/LosslessCut-linux-x64.tar.bz2) / [x64 AppImage](https://github.com/mifi/lossless-cut/releases/latest/download/LosslessCut-linux-x86_64.AppImage) / [arm64 tar.bz2](https://github.com/mifi/lossless-cut/releases/latest/download/LosslessCut-linux-arm64.tar.bz2)
- [更多发行版](https://github.com/mifi/lossless-cut/releases)——注意 APPX（Windows）和 PKG（macOS）**不可用**
- [最新 nightly 构建 🧪](https://mifi.no/llc/nightly/)

请注意以上是[唯一的官方下载渠道](https://losslesscut.app/docs/faq#can-i-download-losslesscut-from-other-websites-or-package-managers)。也有非官方发行版，例如 [Flathub](https://flathub.org/apps/details/no.mifi.losslesscut)（并非由我维护）。应用商店版本和 GitHub 下载版有何区别？[请参阅 FAQ](https://losslesscut.app/docs/faq#is-losslesscut-free-whats-the-difference-between-the-app-store-and-github-versions)。

[![](https://raw.githubusercontent.com/mifi/lossless-cut/6fc47dd4cc81b72a8f1b3132c47a99880cd9bbdb/donate.svg)](https://mifi.no/thanks)

LosslessCut 由我一人维护，并将始终免费开源。如果它对你有用，请考虑支持我的工作。或者[捐赠 FFmpeg 团队](https://www.ffmpeg.org/donations.html)，因为他们为世界做出了巨大贡献。

[Paypal](https://paypal.me/mifino/usd) | [加密货币](https://mifi.no/thanks) | [更多](https://mifi.no/thanks)

<a id="supported-formats"></a>
## 支持的格式

LosslessCut 使用 Chromium 浏览器的 HTML5 视频播放器，并非所有格式和编解码器都[受到原生支持](https://www.chromium.org/audio-video)。通常可用的文件格式包括 `MP4`、`MOV`、`WebM`、`Matroska`、`OGG` 和 `WAV`；音频编解码器包括 `FLAC`、`MP3`、`Opus`、`PCM`、`Vorbis` 和 `AAC`；视频编解码器包括 `H264`、`AV1`、`Theora`、`VP8`、`VP9` 和 `H265`（需要硬件解码器）。了解[编解码器和格式的区别](https://losslesscut.app/docs#primer-videoaudio-codecs-vs-formats)。其他编解码器和格式的文件改由 FFmpeg 播放（*FFmpeg 辅助播放*）：LosslessCut 知道不支持时自动启用，也可从 `File` → *Convert to supported format* 启用。导出仍然使用原文件，因此仍是无损的。这意味着你可以打开 FFmpeg 能解码的几乎任何文件。

<a id="documentation"></a>
## 文档

- 官方网站：[LosslessCut.app](https://losslesscut.app)
- **[入门](https://losslesscut.app/docs/getting-started)**
- [常见问题](https://losslesscut.app/docs/faq)
- [使用文档](https://losslesscut.app/docs)
- [故障排查、已知问题与限制](https://losslesscut.app/docs/troubleshooting)
- [参与贡献](https://github.com/mifi/lossless-cut/blob/6fc47dd4cc81b72a8f1b3132c47a99880cd9bbdb/CONTRIBUTING.md)

<a id="video-demos"></a>
### 视频演示

- [常用功能](https://www.youtube.com/watch?v=pYHMxXy05Jg)
- [如何为 MP4 添加缩略图或封面](https://www.youtube.com/watch?v=4pYJ93cn80E)
- [如何为视频添加多语言音轨](https://www.youtube.com/watch?v=MRBGDsuw_WU)
- 你的视频也可以放在这里？

<a id="featured"></a>
## 相关报道

- [Console newsletter](https://console.substack.com/p/console-93)
- Hacker News：[2024](https://news.ycombinator.com/item?id=40829494) [2022](https://news.ycombinator.com/item?id=33969490) [2020-10](https://news.ycombinator.com/item?id=24883030) [2020-01](https://news.ycombinator.com/item?id=22026412) [2016](https://news.ycombinator.com/item?id=12885585)
- [维基百科](https://en.m.wikipedia.org/wiki/LosslessCut)
- 你的链接也可以放在这里？
- theo（YouTube）：[1](https://youtu.be/FI5ba4RRE8U?t=246) [2](https://youtu.be/uaCypXEJjes?t=381)

<img src="https://api.star-history.com/svg?repos=mifi/lossless-cut&type=Date" alt="Star 历史图" width="400px" />

<a id="attributions"></a>
## 署名

- 应用图标由 [Dimi Kazak](http://www.flaticon.com/authors/dimi-kazak "Dimi Kazak") 制作，来自 [www.flaticon.com](http://www.flaticon.com "Flaticon")，采用 [CC BY 3.0](http://creativecommons.org/licenses/by/3.0/ "Creative Commons BY 3.0") 许可。
- [Chris Gannon 制作的 Lottie 动画](https://lottiefiles.com/7077-magic-flow)。
- 感谢 Adi Abinun 和 [@abdul-alhasany](https://github.com/mifi/lossless-cut/issues/2561) 参与界面工作。
- 感谢帮助翻译应用的译者。[你也可以参与！](https://losslesscut.app/docs/translation)
- [感谢所有支持](https://mifi.no/thanks/)我开源工作的人 🙌

---

在 [🇳🇴](https://www.youtube.com/watch?v=uQIv8Vo9_Jc) 用 ❤️ 制作。

[mifi.no 的更多应用](https://mifi.no/)

在 [GitHub](https://github.com/mifi/)、[YouTube](https://www.youtube.com/channel/UC6XlvVH63g0H54HSJubURQA)、[IG](https://www.instagram.com/mifi.no/)、[Twitter](https://twitter.com/mifi_no) 关注我，获取更多精彩内容！
