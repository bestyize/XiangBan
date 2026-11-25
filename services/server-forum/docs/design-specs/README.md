# 超级小伴社区设计预览

全端统一浏览入口见[小伴设计稿总导航](../../../../docs/design-preview/index.html)（仓库根运行 `python3 docs/design-preview/serve.py --open`）。

## 全站 UI 重设计稿（活力社区风 · PC + 移动 H5）

[社区全站 UI 重设计稿](prototypes/community-redesign/index.html) 采用 **现代活力视觉社区风**，统一覆盖 **PC 端网页**（左导轨 + 精简顶栏）与 **移动端 H5**（390px 交互样机），并已整合原独立「个人中心四页套件」与「详情页分享面板」：

- **社区主站**：首页五栏目（创意、资讯、精选、排行榜、导航）、讨论（左文右图讨论卡）、资源、应用广场、小伴 AI 与作品/帖子详情及评论互动，支持 6 套客户端主题色与日夜切换。
- **个人中心四页**：**我的**（隐藏外露积分、复制 ID、每日签到、收藏/作品/通知三宫格与未登录态）、**个人主页**（本人/他人/空态视角、四列数据条、帖子/回复/关注/粉丝与拉黑空态）、**我的作品**（内容包/方案、全部/草稿/已上线/审核中/已下架计数筛选、草稿虚线描边与居中二次确认区分下架/永久删除）及**通知**（全部/未读 Tab 与顶栏右上角全部已读）。
- **详情页分享面板**：分享面板纯黑白灰中性配色、仅分享图内保留品牌主题色；PC 端为 716px 居中双栏弹窗（左栏实时卡片预览与极简白/暖纸笺/墨夜纸色及摘录/二维码开关，右栏复制链接/图片/保存 PNG/推荐语/问问小伴与作者内联删除确认）；H5 端 1:1 对齐手机 App（单行预览条 + 横滑图标轨 + 沉浸式磨砂分享卡片舞台）。

从仓库根目录通过总导航启动，或单独启动本目录静态服务：

```sh
python3 -m http.server 1449 --bind 127.0.0.1 --directory services/server-forum/docs/design-specs/prototypes/community-redesign
```

- [PC + H5 同屏预览](http://127.0.0.1:1449/) · [PC 端网页预览](http://127.0.0.1:1449/?device=desktop) · [移动端 H5 预览](http://127.0.0.1:1449/?device=mobile) · [个人中心](http://127.0.0.1:1449/?page=me) · [我的作品](http://127.0.0.1:1449/?page=works) · [作品详情与分享](http://127.0.0.1:1449/?detail=work:whale-girl&share=1)

---

## 跨端共享单页参考稿（手机 + 网页双端）

- [发帖页设计稿](../../../../client/xiaoban-multiplatform/docs/design-specs/prototypes/post-composer.html)：覆盖网页版 PC 820px 级联弹窗、H5 窄屏 16px 底部抽屉与手机端三态交互。[网页预览](http://127.0.0.1:1421/prototypes/post-composer.html?view=web) · [手机预览](http://127.0.0.1:1421/prototypes/post-composer.html)。
- [更换头像设计稿](../../../../client/xiaoban-multiplatform/docs/design-specs/prototypes/avatar-editor.html)：覆盖「我的设置 → 资料」PC 820px 居中裁剪预览弹窗与手机/H5 选图裁剪流。[本机预览](http://127.0.0.1:1421/prototypes/avatar-editor.html)。
