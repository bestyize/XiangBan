# 项目名称与术语

本文件统一仓库中的常用名称，帮助开发者和 AI 确定讨论对象与代码归属。功能状态和验证范围以各子项目 README、代码及实际环境为准；本表不代表服务已经部署或功能已验收。

## 产品与项目

| 名称 | 指代 | 代码位置与边界 |
| --- | --- | --- |
| 相伴 / 小伴项目 | 本仓库及相关产品的统称；“小伴”也可能指桌面伙伴或 AI 助手，按上下文区分。 | 仓库根目录；不等于某一个客户端或服务器。 |
| 小伴桌面 / 桌面客户端 | Windows、macOS 上的桌面应用，包含桌面伙伴、壁纸、特效、组件、游戏、聊天和超级小伴。 | [client/xiaoban-desktop](client/xiaoban-desktop/README.md)。当前项目名是 `xiaoban-desktop`，不再称为“小伴壁纸桌面版”。 |
| 小伴手机 App / 移动端 | 小伴的移动端应用，包含原生社区、AI 与手机桌面互联；平台支持与实现范围见项目说明。 | [client/xiaoban-multiplatform](client/xiaoban-multiplatform/README.md) 的 Android/iOS 工程。该目录中的 `desktopApp/` 是旧网页壳，不是小伴桌面客户端。 |
| 桌面伙伴 / 桌宠 | 显示在桌面上的角色实例，支持主题、外观与互动。一个主题可以有多只伙伴。 | 属于小伴桌面，不等于超级小伴或手机 App。 |
| 超级小伴 | 小伴桌面的悬浮助手，以小球、胶囊和回答卡片提供快捷对话、语音与截图入口。 | 属于小伴桌面；设计见 [超级小伴规范](client/xiaoban-desktop/docs/design-specs/super-assistant.md)。 |
| 小伴 AI | 面向用户的 AI 能力与模型服务入口，可用于桌面或手机对话。 | [server-ai](services/server-ai/README.md) 提供网关与额度；客户端也可配置其他模型服务。不要把“小伴 AI”直接等同于单一模型或某个上游供应商。 |
| 小伴官网 / 壁纸网站 | 提供小伴桌面介绍、下载、版本公告和壁纸图片/视频的网站。 | [services/server-xiaoban](services/server-xiaoban/README.md)，现有域名 `wallpaper.xiangban.xyz`。网站包含壁纸功能，并不意味着桌面应用仍叫“小伴壁纸”。 |
| 小伴市场 | 专家、技能、连接器的自建目录、下载与更新服务。 | [services/server-xiaoban-market](services/server-xiaoban-market/README.md)。桌面“AI 插件”是客户端入口，市场是内容来源；两者不是同一个项目。 |
| 超级小伴论坛 / 社区 | 用户交流、帖子和社区内容的网站及相关功能。 | [services/server-forum](services/server-forum/README.md)，现有域名 `chat.xiangban.xyz`。论坛与超级小伴悬浮助手是不同对象。 |
| 统一账号中心 | 注册、登录、身份验证和账号令牌服务，供各产品使用。 | [services/server-account](services/server-account/README.md)；不等同于 AI 额度系统。 |
| AI 网关 | 管理模型调用、产品额度、兑换码及上游服务的服务端。 | [services/server-ai](services/server-ai/README.md)；身份来自账号中心。 |
| 手机桌面互联 / 中转服务 | 同账号手机和桌面设备发现与消息转发。 | [services/server-link](services/server-link/README.md)；不等同于账号中心或 AI 网关。 |
| 资源服务 | 社区内容包及封面等资源的上传校验与下载服务。 | [services/server-resource](services/server-resource/README.md)；与桌面内置游戏资源基准目录职责不同。 |

## 桌面功能名称

| 名称 | 含义 |
| --- | --- |
| 专家 | 为一场对话提供特定身份或工作流的能力；工作流型专家以已验证的工作流说明参与任务。 |
| 技能 / Skill | 一组供 Agent 按需读取和使用的任务说明及配套资源；输入框中的技能通常作用于当前消息。 |
| 连接器 / MCP 连接器 | 连接外部服务和工具的配置或插件；安装、认证、连接与工具调用各有自己的边界。 |
| 工作区 | 与任务关联的本机工作文件夹；侧栏可按工作区组织对话，不等同于 Git worktree。 |
| 会话 / 对话 | 一组持续保存的聊天记录及其绑定状态；在小伴桌面内，“新建任务”是创建或开始对话的入口。 |
| 资料库 | 桌面端管理文档、收藏及记忆等内容的入口；其具体分区和存储方式见 [资料库规范](client/xiaoban-desktop/docs/design-specs/library.md)。 |
| 资源包 | 可导入、导出的作品文件，如壁纸、特效、组件、游戏或伙伴主题；格式与支持范围见 [资源包格式](client/xiaoban-desktop/docs/resource-pack-format.md)。 |
| 游戏资源基准包 | [assets/game-packs](assets/game-packs/README.md) 中用于构建校验的资源；桌面应用运行的是 `client/xiaoban-desktop/public/game-packs/` 内的副本。 |

## 旧名称与兼容标识

- `client/xiaoban-desktop-wallpaper` 是小伴桌面的旧目录，现为 `client/xiaoban-desktop`；`services/server-xiangban-wallpaper` 是官网服务的旧目录，现为 `services/server-xiaoban`。
- 部分稳定标识保留旧名称以兼容现有账号、安装和数据：桌面账号客户端 ID `xiaoban-desktop-wallpaper`、应用标识 `xyz.xiaban.desktop-wallpaper`、数据目录 `~/.xiaobanwallpaper`，以及现有 Release 标签 `xiaoban-wallpaper-v<版本号>`。它们不是旧项目目录，也不能因项目改名而直接替换。
- 原相伴网页 `client/xiaban-web` 和主后端 `services/xiaban-api` 已从仓库移除。历史文档中出现的名称不代表项目仍存在或线上状态仍相同。
- Git 分支、worktree、Codex 聊天和小伴桌面内的会话是不同概念；讨论“任务”“项目”时，应结合当前操作对象，避免把 Codex 工作区当成小伴应用的工作区。
