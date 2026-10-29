# 大仓边界与迁移

按产品与部署职责组织，不强制共享应用实现。

| 旧位置 | 新位置 |
| --- | --- |
| deploy 配置 | infra/deploy |
| compose.yaml | infra/compose.yaml |
| deploy 说明 | docs/deployment |
| 需求与商业规划 | docs/product |
| design-preview | docs/design-preview |

客户端项目清单见 [client/README.md](../../client/README.md)。规划项目用 README 声明状态，开始开发时才建立工程。

## 依赖与归属




## 构建与发布

客户端独立 npm 锁文件，桌宠和独立 Rust 服务各自维护 Cargo 工程。多端 App 的 KMP 工程与 Gradle wrapper 位于其项目目录内，Tauri 桌面壳是独立工程。


正式壁纸网站使用 `services/server-xiaoban`。网站域名为 `wallpaper.xiangban.xyz`，拥有独立采集、上架与媒体上传功能，同时管理桌面下载版本与公告。桌面客户端以此网站的 `/api/v1` 接口浏览公开图片/视频，并通过单项交付清单下载经 SHA-256 校验的不可变资源；已选资源缓存后可离线播放。版本发布、公告展示策略和内容上架策略均归网站后端，客户端维护自己的协议类型与缓存实现，无跨项目源码导入。配置复用网站现有 `app_settings`，不增加历史数据迁移。

Docker 与工作流使用当前目录路径。所有 GitHub Actions 仅由 `workflow_dispatch` 手动运行，代码、标签和网页改动均不会自动触发构建。

移动目录时检查 README 链接、Gradle projectDir、DTO 导入、Docker COPY/ENTRYPOINT、Compose 上下文、CI 路径和测试资产位置。
