# 超级小伴社区设计预览

全端统一浏览入口见[小伴设计稿总导航](../../../../docs/design-preview/index.html)（仓库根运行 `python3 docs/design-preview/serve.py --open`）。

## 详情页分享面板重设计稿（PC + 移动 H5）

[详情页分享面板重设计稿](prototypes/web-detail-share.html) 覆盖社区全站详情页（社区帖子 / 创意作品 / 开源精选 / AI 日报），**分享面板纯黑白灰中性配色、仅分享图内保留品牌主题色**：**手机 H5 1:1 对齐手机 App 分享面板**（单行预览条 + 单行可横滑图标轨 + 通栏「取消」+ 沉浸式磨砂分享卡片页），**PC 端采用 716px 居中双栏弹窗**（左栏实时预览卡片与纸色/摘录/二维码控制，右栏集成链接复制、直接复制卡片图片粘贴进微信/飞书、保存 PNG、复制推荐语/全文与问问小伴；主态作者删除收在右栏底行并走单步内联确认）。支持 PC + H5 同屏、主态/客态、深浅色与分享图 6 套主题色切换（与个人中心共用 1450 端口）。

- [PC + H5 同屏预览](http://127.0.0.1:1450/prototypes/web-detail-share.html) · [PC 端预览](http://127.0.0.1:1450/prototypes/web-detail-share.html?device=desktop) · [手机 H5 预览](http://127.0.0.1:1450/prototypes/web-detail-share.html?device=mobile)

- 在可运行作品的详情页打开分享面板，或从作品主按钮进入 V2 运行页。桌面和 Android 使用同一作品标识发起打开。[作品分享预览](http://127.0.0.1:1450/prototypes/web-detail-share.html?scene=work&work=whale-girl) · [V2 运行页预览](http://127.0.0.1:1450/prototypes/web-detail-share.html?scene=work&work=whale-girl&view=run)

---

## 全站主站 UI 重设计稿（活力社区风 · PC + 移动 H5）

[社区全站 UI 重设计稿](prototypes/community-redesign/index.html) 采用 **现代活力视觉社区风**，覆盖 **PC 端网页**（左导轨 + 精简顶栏）与 **移动端 H5**（390px 交互样机与真机适配）：包含首页五栏目（创意、资讯、精选、排行榜、导航）、讨论（左文右图讨论卡）、资源、应用、小伴 AI 及作品/帖子详情与评论互动，详情页核心体验/访问/下载操作提升为品牌实心胶囊主按钮，支持 6 套客户端同款主题色与日间/夜间模式。

```sh
# 跨稿跳转到分享与运行页时，同时启动上方原稿使用的 1450 服务。
python3 -m http.server 1449 --bind 127.0.0.1 --directory services/server-forum/docs/design-specs/prototypes/community-redesign
```

- [PC + H5 同屏预览](http://127.0.0.1:1449/) · [PC 端网页预览](http://127.0.0.1:1449/?device=desktop) · [移动端 H5 预览](http://127.0.0.1:1449/?device=mobile) · [作品详情页直达](http://127.0.0.1:1449/?detail=work:whale-girl)

---

## 我的 / 个人主页 / 我的作品 / 通知重设计稿（PC + 移动 H5）

[个人中心四页套件重设计稿](prototypes/web-profile-suite.html) 1:1 对齐手机 App 方案 1 通栏精修规范与 `community-redesign` PC 端左导轨壳层，支持 **PC + 手机 H5 同屏对照** 切换查看四个页面：**我的**（隐藏外露积分、一键复制 ID、每日签到与收藏/作品/通知三宫格）、**个人主页**（本人/他人视角、四列等宽数据条、帖子/回复/关注/粉丝四栏，他人屏蔽入口收进 `···` 菜单）、**我的作品**（内容包/方案切换 + 全部/草稿/已上线/审核中/已下架状态筛选与计数、草稿虚线轻描边、全状态删除与 PC/H5 居中二次确认弹窗区分下架与永久删除）与**通知**（顶栏右上角全部已读 + 全部/未读 Tab + 发送者头像与未读红点 + 居中空态）。

```sh
python3 -m http.server 1450 --bind 127.0.0.1 --directory services/server-forum/docs/design-specs
```

- [PC + H5 同屏预览](http://127.0.0.1:1450/prototypes/web-profile-suite.html) · [PC 端预览](http://127.0.0.1:1450/prototypes/web-profile-suite.html?device=desktop) · [手机 H5 预览](http://127.0.0.1:1450/prototypes/web-profile-suite.html?device=mobile)

---

## 跨端共享单页参考稿（手机 + 网页双端）

- [基础组件库与通用弹层规范](../../../../client/xiaoban-multiplatform/docs/design-specs/prototypes/ui-kit-overlays.html)：覆盖手机 `:common:ui` 与社区站前台 `src/ui/` 的分层规范（弹层体系、通用控件、PC↔H5 响应式对照与模块边界；管理后台不动）。[弹层预览](http://127.0.0.1:1421/prototypes/ui-kit-overlays.html?tab=overlays) · [PC ↔ H5 对照](http://127.0.0.1:1421/prototypes/ui-kit-overlays.html?tab=responsive)。
- [发帖页设计稿](../../../../client/xiaoban-multiplatform/docs/design-specs/prototypes/post-composer.html)：覆盖网页版 PC 820px 级联弹窗、H5 窄屏 16px 底部抽屉与手机端三态交互。[网页预览](http://127.0.0.1:1421/prototypes/post-composer.html?view=web) · [手机预览](http://127.0.0.1:1421/prototypes/post-composer.html)。
- [更换头像设计稿](../../../../client/xiaoban-multiplatform/docs/design-specs/prototypes/avatar-editor.html)：覆盖「我的设置 → 资料」PC 820px 居中裁剪预览弹窗与手机/H5 选图裁剪流。[本机预览](http://127.0.0.1:1421/prototypes/avatar-editor.html)。
