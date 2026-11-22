# 超级小伴社区设计预览

## 全站 UI 重设计稿（活力社区风 · PC + 移动 H5）

[社区全站 UI 重设计稿](prototypes/community-redesign/index.html) 采用 **现代活力视觉社区风**，同时覆盖 **PC 端网页**（左导轨 + 精简顶栏）与 **移动端 H5**（390px 交互样机与真机适配）：顶部搜索与提醒纯图标收进右上角不带底色、「首页 / 讨论 / 资源」严格居中，创意页卡片统一 `3:2` 视觉封面与轻盈小红心互动；包含首页五栏目（创意、资讯、精选、排行榜、导航）、讨论、资源、应用、小伴 AI、我的以及作品/帖子详情与评论互动，支持 6 套客户端同款主题色与日间/夜间模式。

从仓库根目录启动本机预览：

```sh
python3 -m http.server 1449 --bind 127.0.0.1 --directory services/server-forum/docs/design-specs/prototypes/community-redesign
```

- [PC + H5 同屏预览](http://127.0.0.1:1449/) · [PC 端网页预览](http://127.0.0.1:1449/?device=desktop) · [移动端 H5 预览](http://127.0.0.1:1449/?device=mobile)

---

## 既有单页与卡片参考稿

[发帖页设计稿](../../../../client/xiaoban-multiplatform/docs/design-specs/prototypes/post-composer.html) 覆盖网页版 PC 820px 弹窗（级联板块菜单、编辑/预览切换、单作品入口支持 V2 ZIP / HTML·HTM 转 V2 与关联创作、内联附件定价、上拉阅读权限与弹窗内关闭确认）、H5 窄屏 16px 底部抽屉，以及手机端 3 画面交互稿，支持明暗模式与 6 套主题色。[网页预览](http://127.0.0.1:1421/prototypes/post-composer.html?view=web) · [手机预览](http://127.0.0.1:1421/prototypes/post-composer.html)。

[更换头像设计稿](../../../../client/xiaoban-multiplatform/docs/design-specs/prototypes/avatar-editor.html) 覆盖「我的设置 → 资料」点头像后的弹窗：PC 820 居中弹窗（左侧裁剪、右侧四种尺寸与帖子评论预览），窄屏与手机 App 完全一致（底部选图弹层 + 全屏裁剪页）。[本机预览](http://127.0.0.1:1421/prototypes/avatar-editor.html)。

[讨论卡片设计稿](../../../../client/xiaoban-multiplatform/docs/design-specs/prototypes/discussion-cards.html) 使用单列列表，标题单行。底部左侧依次显示标签、头像、发布时间，右侧为回复和点赞。可切换桌面、移动和深色预览。[本机在线预览](http://127.0.0.1:1421/prototypes/discussion-cards.html)。

[创意页作品卡片](prototypes/ideas-cards/index.html) 展示微观展厅、《小王子：星辰之旅》和「运输船：Saltline」。卡片使用 16:9 封面、右上角可选类型标签、标题、说明、作者及收录日期。示例日期用于展示。

从仓库根目录启动本机预览：

```sh
python3 -m http.server 1448 --bind 127.0.0.1 --directory services/server-forum/docs/design-specs/prototypes/ideas-cards
```

[本机在线预览](http://127.0.0.1:1448/) 也可直接打开 HTML 文件。首张「微观展厅」卡片直接打开现有社区展厅，《小王子》和 Saltline 点击后进入各自的作品详情。

[创意作品详情页](prototypes/ideas-cards/work-detail.html) 以「碎花垂耳兔」展示完整作品图、短介绍、作者与来源。两端通栏，电脑端将作品图和介绍并排，手机端纵向呈现。封面取自 RealWorldBuilder 的微观展厅，「体验 3D 模型」使用灰底深色字，通往现有作品运行页。评论列表、评论入口和编辑弹层沿用全站评论样式。留言、回复和点赞可在预览中体验，刷新后恢复示例评论。[全站评论组件](../../web/src/components/comments/CommentSection.vue) · [电脑预览](http://127.0.0.1:1448/work-detail.html) · [手机预览](http://127.0.0.1:1448/work-detail.html?view=mobile)

微观展厅使用社区展品的水族箱封面，来源为 RealWorldBuilder。《小王子》的封面使用 [Sac 的作品官网](https://littleprince.dev/)主视觉，Saltline 的封面使用 [作品展示页](https://saltline.games/)中的游戏画面，作者为 Felix Rieseberg。设计稿独立于生产页面。
