[![GitHub build status](https://github.com/git/git/workflows/CI/badge.svg)](https://github.com/git/git/actions?query=branch%3Amaster+event%3Apush)
[![GitLab build status](https://gitlab.com/git-scm/git/badges/master/pipeline.svg)](https://gitlab.com/git-scm/git/-/pipelines?ref=master)

Git——快速、可扩展的分布式版本控制系统
========================================

Git 是一个快速、可扩展的分布式版本控制系统，拥有异常丰富的命令集，既提供高级操作，也允许完整访问内部机制。

Git 是一个依据 GNU General Public License 第 2 版授权的开源项目（部分内容采用与 GPLv2 兼容的其他许可证）。它最初由 Linus Torvalds 在一群网络黑客的帮助下编写。

安装说明请阅读 [INSTALL][] 文件。

许多 Git 在线资源可从 <https://git-scm.com/> 获取，包括完整文档和 Git 相关工具。

请从 [Documentation/gittutorial.adoc][] 开始，然后阅读 [Documentation/giteveryday.adoc][] 了解实用的最小命令集，阅读 `Documentation/git-<commandname>.adoc` 了解各命令的文档。如果 Git 已正确安装，也可使用 `man gittutorial` 或 `git help tutorial` 阅读教程，并使用 `man git-<commandname>` 或 `git help
<commandname>` 阅读各命令文档。

CVS 用户还可以阅读 [Documentation/gitcvs-migration.adoc][]（如果已安装 Git，可运行 `man gitcvs-migration` 或 `git help cvs-migration`）。

Git 的用户讨论和开发在 Git 邮件列表中进行——欢迎任何人向 git@vger.kernel.org 提交错误报告、功能请求、评论和补丁（提交补丁的说明请参阅 [Documentation/SubmittingPatches][]，另请阅读 [Documentation/CodingGuidelines][]）。

希望帮助翻译错误消息、用法及信息字符串（本地化 l10）的人应参阅 [po/README.md][]（`po` 文件是保存译文的 Portable Object 文件）。

要订阅邮件列表，请发送电子邮件至 <git+subscribe@vger.kernel.org>（详情见 <https://subspace.kernel.org/subscribing.html>）。邮件列表存档可在 <https://lore.kernel.org/git/>、<https://marc.info/?l=git> 等站点查看。

与安全有关的问题应私下向 Git 安全邮件列表 <git-security@googlegroups.com> 披露。

维护者经常向邮件列表发送“What’s cooking”报告，列出各种开发主题的当前状态。随后的讨论可作为了解项目状态、开发方向和剩余任务的参考。

“git”这个名字是 Linus Torvalds 编写最初版本时取的。他将该工具描述为“愚蠢的内容跟踪器”，并称这个名字可以按心情解释为：

 - 一个可以发音且没有被常用 UNIX 命令使用的随机三字母组合。它恰好是“get”的错误发音，也许有关系，也许没有。
 - 愚蠢、卑鄙、可鄙、简单；你可以从俚语词典中任选一种解释。
 - “global information tracker”（全球信息跟踪器）：当你心情不错，而且它确实对你管用时，天使歌唱，房间里突然充满光明。
 - “goddamn idiotic truckload of sh*t”（该死的一大堆蠢东西）：当它出问题时。

[INSTALL]: https://github.com/git/git/blob/ebdc087dc99d68bb340c282b2ba5f722def28d19/INSTALL
[Documentation/gittutorial.adoc]: https://github.com/git/git/blob/ebdc087dc99d68bb340c282b2ba5f722def28d19/Documentation/gittutorial.adoc
[Documentation/giteveryday.adoc]: https://github.com/git/git/blob/ebdc087dc99d68bb340c282b2ba5f722def28d19/Documentation/giteveryday.adoc
[Documentation/gitcvs-migration.adoc]: https://github.com/git/git/blob/ebdc087dc99d68bb340c282b2ba5f722def28d19/Documentation/gitcvs-migration.adoc
[Documentation/SubmittingPatches]: https://github.com/git/git/blob/ebdc087dc99d68bb340c282b2ba5f722def28d19/Documentation/SubmittingPatches
[Documentation/CodingGuidelines]: https://github.com/git/git/blob/ebdc087dc99d68bb340c282b2ba5f722def28d19/Documentation/CodingGuidelines
[po/README.md]: https://github.com/git/git/blob/ebdc087dc99d68bb340c282b2ba5f722def28d19/po/README.md
