# 设计资源与预览

状态：设计参考，静态文件用于讨论外观交互，不是客户端运行入口，不参与生产 Docker 构建。

实现设计时修改对应 client 工程并验证；不要直接用预览覆盖已有应用、鉴权或平台接入逻辑。

## 小伴桌面

| 设计稿 | 内容 |
| --- | --- |
| [小伴桌面 UI 升级设计稿](../../client/xiaoban-desktop/docs/design-specs/ui-upgrade/README.md) | 39 个画板：主界面、设置、超级小伴、资料库、实时对话、小伴市场接入、输入框与各类状态卡片；[在线画布](https://claude.ai/artifact/GzhNp5y7ehkzSrWWExkqzg) |
| [工作空间检索与列表折叠](../../client/xiaoban-desktop/docs/design-specs/prototypes/workspace-search.html) | 标题、正文与工作区检索；限高滚动结果；项目和项目内会话默认各展示 6 项；点击结果自动展开并定位会话 |
| [长期目标弹窗](../../client/xiaoban-desktop/docs/design-specs/prototypes/long-term-goal.html) | 新建、执行中与待验收状态；折叠设置；浅色、深色及窄屏预览。遮罩仅用于演示内容，桌面端窗口形式按独立窗口规范 |

## 小伴手机 App

| 设计稿 | 内容 |
| --- | --- |
| [社区与小伴AI 手机原型](../../client/xiaoban-multiplatform/docs/design-specs/mobile-prototype/README.md) | 70 个画板：社区、小伴 AI、游戏、我的与设置、分享、设备遥控、聊天状态卡片、实时语音与按住说话；[在线画布](https://claude.ai/artifact/BGtDJ7S7vGenqaY5MvyLLJ) |

两份 Claude Design 画布导出的设计稿由多个 `*.dc.html` 画板组成，需在各自目录运行 `python3 -m http.server` 后打开 `index.html` 总览；其余单个 HTML 设计稿均自包含，可直接用浏览器打开，展示数据为模拟内容。设计稿源文件统一保留在客户端设计目录，避免多份副本产生差异。

完整设计规范与其他设计稿见[小伴桌面设计资源索引](../../client/xiaoban-desktop/docs/design-specs/README.md)。
