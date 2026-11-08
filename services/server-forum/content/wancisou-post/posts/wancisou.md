---
id: wancisou-app-001
title: 万磁搜：在小伴里跨站搜索公开资源
category: create-app
tags:
  - 小伴应用
  - 工具
  - 搜索
---

万磁搜是一款在小伴里运行的独立工具。输入关键词后，它会调用 [TorrentSpiderRust](https://github.com/bestyize/TorrentSpiderRust) 的公开搜索服务，汇集多个磁力索引站的结果，并按相关性显示。

![万磁搜搜索结果界面示意图](images/wancisou-preview.png)

打开「应用广场 → 精选工具」里的万磁搜，输入关键词，选择全部来源或单个站点即可搜索。点结果卡片可以查看磁力链接，并在弹窗里复制。万磁搜只负责搜索与复制链接，不下载 BT 文件。

关键词会发送到 `search.xiangban.xyz`。索引站的内容和可用状态会变化，部分来源可能超时或无法返回结果。使用链接前，请自行核对资源来源与授权。

配图是设计稿示意，实际结果取决于搜索服务和所选关键词。
