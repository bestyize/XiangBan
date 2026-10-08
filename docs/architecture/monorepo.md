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

小伴的三个 client 路径保留。规划项目用 README 声明状态，开始开发时才建立工程。

## 依赖与归属

backend → api-contracts。接口库用 14 个业务包组织 52 个定义，仍为一个 KMP 构建模块。后端维护称号配置、性格展示标签和事件策略。

网页维护自己的 React 角色与 TS API 类型。桌宠 src/companion 初始复制原网页纯角色核心，SVG/弹簧/状态机参数不变，测试在桌宠内部执行。需要同步行为时明确比较与验证，不隐式跨应用导入。

未来 App 可使用 KMP 接口库或独立模型，实体机器人自行确定设备协议。services 下的规划目录不表示主后端已经拆成微服务。

## 构建与发布

客户端独立 npm 锁文件，桌宠和独立 Rust 服务各自维护 Cargo 工程。根 Gradle 当前只包含 api-contracts 与 backend，规划 App 未接入。

services/server-pan 是独立的网盘服务：backend 为 Cargo 工程，frontend 为独立 npm 工程，构建产物由后端单端口托管。它自带账号、JWT 与 SQLite，不依赖 api-contracts，也不接入主 API 的鉴权与数据库；运行时数据目录 pandown_data 不入库。目前没有 Docker、Compose 与 CI 配置，接入部署时再补充。镜像、服务、数据卷和 URL 不因物理路径迁移更名。

services/server-xiaoban-wallpaper 是独立的小伴桌面网站与内容管理服务：backend 为 Rust/Axum/SQLx，数据库仅使用独立 PostgreSQL，frontend 为 Vue 3/Vite。服务只存壁纸及发布元数据，外部图片、视频、动画与安装包不上传、不落盘。自带管理员会话、目录 JSON 与桌面更新接口，拥有独立 Docker、Compose 和 CI，不接入根 Gradle或主 API 的腾讯云部署流程。

Docker、手动部署工作流和桌宠 CI 使用新路径。桌宠 CI 只安装桌宠依赖，网页改动不触发桌宠构建。

移动目录时检查 README 链接、Gradle projectDir、DTO 导入、Docker COPY/ENTRYPOINT、Compose 上下文、CI 路径和测试资产位置。
