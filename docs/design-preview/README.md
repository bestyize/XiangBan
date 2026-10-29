# 设计资源与预览

状态：设计参考，静态文件用于讨论外观交互，不是客户端运行入口，不参与生产 Docker 构建。

实现设计时修改对应 client 工程并验证；不要直接用预览覆盖已有应用、鉴权或平台接入逻辑。

## 小伴桌面

| 设计稿 | 内容 |
| --- | --- |
| [工作空间检索与列表折叠](../../client/xiaoban-desktop/docs/design-specs/prototypes/workspace-search.html) | 标题、正文与工作区检索；限高滚动结果；项目和项目内会话默认各展示 6 项；点击结果自动展开并定位会话 |
| [长期目标弹窗](../../client/xiaoban-desktop/docs/design-specs/prototypes/long-term-goal.html) | 新建、执行中与待验收状态；折叠设置；浅色、深色及窄屏预览。遮罩仅用于演示内容，桌面端窗口形式按独立窗口规范 |

以上 HTML 均为自包含交互稿，可直接用浏览器打开，展示数据为模拟内容。设计稿源文件统一保留在客户端设计目录，避免多份副本产生差异。

完整设计规范与其他设计稿见[小伴桌面设计资源索引](../../client/xiaoban-desktop/docs/design-specs/README.md)。
