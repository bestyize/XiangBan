# 大仓边界与迁移

按产品与部署职责组织，不强制共享应用实现。

| 旧位置 | 新位置 |
| --- | --- |
| web | client/xiaban-web |
| server / :server | services/xiaban-api/backend / :backend |
| shared / :shared | services/xiaban-api/api-contracts / :api-contracts |
| deploy 配置 | infra/deploy |
| compose.yaml | infra/compose.yaml |
| deploy 说明 | docs/deployment |
| 需求与商业规划 | docs/product |
| design-preview | docs/design-preview |

客户端项目清单见 [client/README.md](../../client/README.md)。规划项目用 README 声明状态，开始开发时才建立工程。

## 依赖与归属

backend → api-contracts。接口库按业务包组织，仍为一个 KMP 构建模块；包清单见 [接口库 README](../../services/xiaban-api/api-contracts/README.md)。后端维护称号配置、性格展示标签和事件策略。

网页维护自己的 React 角色与 TS API 类型。桌宠 src/companion 初始复制原网页纯角色核心，SVG/弹簧/状态机参数不变，测试在桌宠内部执行。需要同步行为时明确比较与验证，不隐式跨应用导入。

client/xiaoban-mobile-wallpaper 是 Android 壁纸浏览与下载 App（Compose），把壁纸网站的图片和视频保存到手机相册，收藏只存本机；自带 Kotlin 版小伴角色（从桌宠 src/companion 移植，不导入其源码），访问壁纸网站接口，并可接入统一账号中心，不连接主 API。未来 App 可使用 KMP 接口库或独立模型，实体机器人自行确定设备协议。services 下的规划目录不表示主后端已经拆成微服务。

## 构建与发布

客户端独立 npm 锁文件，桌宠和独立 Rust 服务各自维护 Cargo 工程。根 Gradle 当前只包含 api-contracts 与 backend；Android App 有自己的 Gradle 工程与 wrapper，不接入根 Gradle，多端 App 的独立 KMP 工程和 Tauri 桌面壳同样未接入。

services/server-pan 是独立的网盘服务：backend 为 Cargo 工程，frontend 为独立 npm 工程，构建产物由后端单端口托管。它自带账号、JWT 与 SQLite，不依赖 api-contracts，也不接入主 API 的鉴权与数据库；运行时数据目录 pandown_data 不入库。目前没有 Docker、Compose 与 CI 配置，接入部署时再补充。镜像、服务、数据卷和 URL 不因物理路径迁移更名。

正式壁纸网站使用 `services/server-xiangban-wallpaper`。网站域名为 `wallpaper.xiangban.xyz`，拥有独立采集、上架与媒体上传功能，同时管理桌面下载版本与公告。桌面客户端以此网站的 `/api/v1` 接口浏览公开图片/视频，并通过单项交付清单下载经 SHA-256 校验的不可变资源；已选资源缓存后可离线播放。版本发布、公告展示策略和内容上架策略均归网站后端，客户端维护自己的协议类型与缓存实现，无跨项目源码导入。配置复用网站现有 `app_settings`，不增加历史数据迁移。

Docker 与工作流使用当前目录路径。所有 GitHub Actions 仅由 `workflow_dispatch` 手动运行，代码、标签和网页改动均不会自动触发构建。

移动目录时检查 README 链接、Gradle projectDir、DTO 导入、Docker COPY/ENTRYPOINT、Compose 上下文、CI 路径和测试资产位置。
