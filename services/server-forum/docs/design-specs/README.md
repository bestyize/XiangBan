# 超级小伴社区设计预览

[讨论卡片设计稿](../../../../client/xiaoban-multiplatform/docs/design-specs/prototypes/discussion-cards.html) 使用单列列表，标题单行。底部左侧依次显示标签、头像、发布时间，右侧为回复和点赞。可切换桌面、移动和深色预览。[本机在线预览](http://127.0.0.1:1421/prototypes/discussion-cards.html)。

[创意页作品卡片](prototypes/ideas-cards/index.html) 展示微观展厅、《小王子：星辰之旅》和「运输船：Saltline」。卡片使用 16:9 封面、右上角可选类型标签、标题、说明、作者及收录日期。示例日期用于展示。

从仓库根目录启动本机预览：

```sh
python3 -m http.server 1448 --bind 127.0.0.1 --directory services/server-forum/docs/design-specs/prototypes/ideas-cards
```

[本机在线预览](http://127.0.0.1:1448/) 也可直接打开 HTML 文件。首张「微观展厅」卡片直接打开现有社区展厅，《小王子》和 Saltline 点击后进入各自的作品详情。

微观展厅使用社区展品的水族箱封面，来源为 RealWorldBuilder。《小王子》的封面使用 [Sac 的作品官网](https://littleprince.dev/)主视觉，Saltline 的封面使用 [作品展示页](https://saltline.games/)中的游戏画面，作者为 Felix Rieseberg。设计稿独立于生产页面。
