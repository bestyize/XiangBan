> 译注：本次检查中 SourceForge 下载链接返回 403，可使用原文提供的 [GitHub Releases](https://github.com/x64dbg/x64dbg/releases)；DeepWiki 限流、tr4ceflow 网站及部分历史致谢链接不可读，原链接均保留，详情见审校记录。

# x64dbg

<img width="100" src="https://raw.githubusercontent.com/x64dbg/x64dbg/79d57163d07432d24f0bff00298bfd2acd98f341/src/bug_black.png"/>

[![Crowdin](https://d322cqt584bo4o.cloudfront.net/x64dbg/localized.svg)](https://translate.x64dbg.com) [![Download x64dbg](https://img.shields.io/sourceforge/dm/x64dbg.svg)](https://sourceforge.net/projects/x64dbg/files/latest/download) [![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/x64dbg/x64dbg)

[![Discord](https://img.shields.io/badge/chat-on%20Discord-green.svg)](https://discord.x64dbg.com) [![Slack](https://img.shields.io/badge/chat-on%20Slack-red.svg)](https://slack.x64dbg.com) [![Gitter](https://img.shields.io/badge/chat-on%20Gitter-lightseagreen.svg)](https://gitter.im/x64dbg/x64dbg) [![Matrix](https://img.shields.io/badge/chat-on%20Matrix-yellowgreen.svg)](https://matrix.to/#/#x64dbg:matrix.org) [![IRC](https://img.shields.io/badge/chat-on%20IRC-purple.svg)](https://web.libera.chat/#x64dbg)

一款用于 Windows 的开源二进制调试器，面向恶意软件分析和没有源代码的可执行文件逆向工程。它提供许多功能，以及完整的[插件系统](https://plugins.x64dbg.com)，供你添加自己的功能。更多信息请参阅[博客](https://x64dbg.com/blog)!

## 截图

![主界面（浅色）](https://raw.githubusercontent.com/x64dbg/x64dbg/79d57163d07432d24f0bff00298bfd2acd98f341/.github/screenshots/cpu-light.png)

![主界面（深色）](https://raw.githubusercontent.com/x64dbg/x64dbg/79d57163d07432d24f0bff00298bfd2acd98f341/.github/screenshots/cpu-dark.png)

| ![graph](https://raw.githubusercontent.com/x64dbg/x64dbg/79d57163d07432d24f0bff00298bfd2acd98f341/.github/screenshots/graph-light.png) | ![内存映射](https://raw.githubusercontent.com/x64dbg/x64dbg/79d57163d07432d24f0bff00298bfd2acd98f341/.github/screenshots/memory-map-light.png) |
| :--: | :--: |

## 安装与使用

1. 从 [GitHub](https://github.com/x64dbg/x64dbg/releases)或 [SourceForge](https://sourceforge.net/projects/x64dbg/files/snapshots)下载快照并解压到当前用户有写入权限的位置。
2. _可选_：使用 `x96dbg.exe` 注册 shell 扩展并在桌面添加快捷方式。
3. 调试 32 位可执行文件可运行 `x32\x32dbg.exe`，调试 64 位可执行文件可运行 `x64\x64dbg.exe`。如果不确定，也可以运行 `x96dbg.exe` 并在其中选择架构。

你也可以[自行编译](https://github.com/x64dbg/x64dbg/wiki/Compiling-the-whole-project) x64dbg。

## 赞助商

<div align="center" markdown="1">

  <a href="https://sponsors.x64dbg.com/warp" target="_blank">
    <img alt="Warp sponsorship" width="400" src="https://raw.githubusercontent.com/warpdotdev/brand-assets/main/Github/Sponsor/Warp-Github-LG-02.png">
  </a>

  [**Warp，面向多个 AI 智能体协作编码而构建**](https://sponsors.x64dbg.com/warp)

<br>

[![](https://raw.githubusercontent.com/x64dbg/x64dbg/79d57163d07432d24f0bff00298bfd2acd98f341/.github/sponsors/telekom.svg)](https://sponsors.x64dbg.com/telekom)

</div>

## 贡献

这是一个社区项目，我们接受 pull request！更多信息请参阅 [CONTRIBUTING](https://github.com/x64dbg/x64dbg/blob/79d57163d07432d24f0bff00298bfd2acd98f341/.github/CONTRIBUTING.md)文档。如有问题，可以[联系我们](https://x64dbg.com/#contact)或创建 [issue](https://github.com/x64dbg/x64dbg/issues). 可以查看[适合新手的问题](https://easy.x64dbg.com/)开始参与。

## 致谢

- 调试器核心由 [TitanEngine Community Edition](https://github.com/x64dbg/TitanEngine) 提供
- 反汇编由 [Zydis](https://zydis.re) 提供
- 汇编由 [XEDParse](https://github.com/x64dbg/XEDParse) and [asmjit](https://github.com/asmjit) 提供
- 导入重建由 [Scylla](https://github.com/NtQuery/Scylla) 提供
- JSON 功能由 [Jansson](https://www.digip.org/jansson) 提供
- 数据库压缩由 [lz4](https://bitbucket.org/mrexodia/lz4) 提供
- 虫子图标由 [VisualPharm](https://www.visualpharm.com) 提供
- 界面图标由 [Fugue](https://p.yusukekamiyamane.com) 提供
- 网站由 [tr4ceflow](https://tr4ceflow.com) 提供


## 开发者

- [mrexodia](https://mrexodia.github.io)
- Sigma
- [tr4ceflow](https://blog.tr4ceflow.com)
- [Dreg](https://www.fr33project.org)
- [Nukem](https://github.com/Nukem9)
- [Herz3h](https://github.com/Herz3h)
- [torusrxxx](https://github.com/torusrxxx)

## 代码贡献

GitHub 贡献者完整列表请见[这里](https://github.com/x64dbg/x64dbg/graphs/contributors).

## 特别感谢

- Sigma 开发最初的图形界面
- 所有捐赠者！
- 所有提交 issue 的人！
- 我忘记加入名单的人
- [博客作者](https://x64dbg.com/blog/2016/07/09/Looking-for-writers.html)!
- [EXETools 社区](https://forum.exetools.com)
- [Tuts4You 社区](https://forum.tuts4you.com)
- [ReSharper](https://www.jetbrains.com/resharper)
- [Coverity](https://www.coverity.com)
- acidflash
- cyberbob
- cypher
- Teddy Rogers
- TEAM DVT
- DMichael
- Artic
- ahmadmansoor
- \_pusher\_
- firelegend
- [kao](https://lifeinhex.com)
- sstrato
- [kobalicek](https://github.com/kobalicek)
- [athre0z](https://github.com/athre0z)
- [ZehMatt](https://github.com/ZehMatt)
- [mrfearless](https://twitter.com/fearless0)
- [JustMagic](https://github.com/JustasMasiulis)

没有众多人和其他开源项目的帮助，x64dbg 就不可能成为今天的样子，谢谢大家！

## 历史捐赠者

在完全转向 [GitHub Sponsors](https://github.com/sponsors/mrexodia),之前，项目通过 BountySource 接收捐赠。原捐赠条款允许捐赠者在捐赠时请求附上网站链接；下方标注的链接反映了这些请求。BountySource 此后已经关闭，因此这些记录由人工重建。如果你在这段时期捐过款而用户名或金额缺失、不正确，请联系我们。

|用户名|金额|日期||用户名|金额|日期|
|-|-|-|-|-|-|-|
|sghctoma|$50|2015-04-19||dfrunza|$20|2017-01-30|
|overflow|$50|2015-04-25||ham3di|$100|2017-02-01|
|jl2id|$15|2015-04-29||johnny5|$5|2017-02-19|
|cypherpunk|$50|2015-05-02||David-Reguera-Garcia-Dreg|$90|2017-02-26|
|Aciid|$50|2015-05-05||[Alexandro Sanchez Bach](https://phi.nz)|-|2017-03-02|
|PI32|$15|2015-05-09||（未知）|$6|2017-03-11|
|darkvapeur|$8|2015-05-21||fred26|$50|2017-04-08|
|fearless|$5|2015-05-24||gatesbillou|$20|2017-04-15|
|0x90|$10|2015-05-31||David-Reguera-Garcia-Dreg|$10|2017-04-24|
|acidflash|$50|2015-06-03||Adir|$20|2017-05-03|
|VackerSimon|$10|2015-06-14||ferbeb|$10|2017-05-17|
|Artic|$10|2015-06-29||（未知）|$16|2017-06-04|
|[crystalidea](https://www.crystalidea.com/uninstall-tool)|$24|2015-07-10||androsa|$20|2017-06-11|
|jl2id|$10|2015-08-13||robersor|$25|2017-07-05|
|[PELock](https://pelock.com)|$115|2015-08-26||DDSTrainers|$10|2017-07-15|
|[tslater2006](https://github.com/tslater2006)|$20|2015-09-04||blaquee|$20|2017-08-27|
|Exidous|$20|2015-09-04||SmilingWolf|$15|2017-09-26|
|lupier|$40|2015-09-08||Alexander H.|$150|2017-10-11|
|Stef|$10|2015-09-15||gatesbillou|$25|2017-10-14|
|[d3v1l401](https://d3vsite.org)|$5|2015-10-06||t4rmo|$5|2017-10-18|
|Artur|$20|2015-10-24||joelcornu|$5|2017-10-27|
|RoBa|$100|2015-11-18||Adir|$35|2017-11-02|
|mr.tuna7331|-|2015-12-15||（未知）|$10|2017-11-11|
|lupier|$90|2016-01-12||xdeng|$10|2018-01-04|
|fvrmatteo|$10|2016-01-21||v-p-b|$50|2018-03-21|
|willi.neu9|$10|2016-01-30||EmptyBrain|$50|2018-03-30|
|rithien|$100|2016-02-19||[mentebinaria](https://www.mentebinaria.com.br/)|$19|2018-04-12|
|ey|$20|2016-02-26||Mauro Bollini|$50|2018-06-07|
|clockwork|$10|2016-03-06||gatesbillou|$15|2018-06-17|
|codespy|$5|2016-03-23||Kirbiflint|$3|2018-06-22|
|test|$100|2016-03-28||Chisato Rokumiya|$20|2018-09-20|
|RomanGol|$10|2016-03-28||pengchang|$100|2018-10-24|
|fearless|$10|2016-04-24||younsunmin|$5|2018-11-18|
|Jack|$5|2016-05-17||EmptyBrain|$50|2018-12-27|
|willi.neu9|$30|2016-05-26||Yim|$10|2019-01-13|
|gatesbillou|$20|2016-06-02||[OALabs](https://www.youtube.com/c/OALabs)|-|2019-01-27|
|AGI|$155|2016-06-16||Lixinist|$10|2019-04-15|
|lupier|$100|2016-06-24||bloodmc|$50|2019-04-29|
|0x90|$50|2016-07-19||（未知）|$20|2019-05-24|
|tr4nc3|$15|2016-07-31||masacate|$10|2019-07-10|
|MikeGuidry|$2500|2016-07-31||User Manuals|$5|2020-01-20|
|Alexander H.|$150|2016-09-09||Jim Conyngham|$50|2020-03-23|
|darkvapeur|$10|2016-10-04||Danya|$5|2020-06-08|
|h907308901|$5|2016-10-06||RooT|$160|2020-07-22|
|Adir|$20|2016-10-24||jadakiss9018|$2|2020-09-11|
|NicoG|$100|2016-10-27||samsonpianofingers|$15|2020-09-14|
|Angie|$150|2016-11-03||tpericin|$1337|2020-10-09|
|hulucc|$20|2016-12-02||nikkej|$100|2020-10-22|
|napcode|$10|2016-12-05||kha1ifaa|$10|2021-01-04|
|TechLord|-|2016-12-26||rikaardhosein|$20|2021-09-08|
|ayylmao5|$5|2017-01-01||Lukas21|$50|2021-09-15|
|affelwafro|$10|2017-01-03||Flavio Nardiello|$30|2021-12-01|
|FS|$40|2017-01-15||stevemk14ebr|$1000|2021-12-03|
|EmptyBrain|$50|2017-01-27||[ethical.blue](https://ethical.blue)|$19|2022-05-14|

_致所有早期支持者：感谢你们在项目成为今天的样子之前就相信它！_
