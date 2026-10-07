# 相伴 / 小伴大仓

按产品和部署职责管理客户端、主 API 与规划中的能力服务。各产品独立维护代码、构建与依赖，允许重复实现；不以统一语言或强制共享 UI 为目标。

## 项目导航

| 项目 | 状态 | 说明 |
| --- | --- | --- |
| [client/xiaban-web](client/xiaban-web/README.md) | 已实现 | 相伴网页/PWA，包含管理后台 |
| [client/xiaoban-desktop-wallpaper](client/xiaoban-desktop-wallpaper/README.md) | 可运行原型 | Windows/macOS 桌宠与动态壁纸，无聊天和账户 |
| [client/xiaoban-multiplatform](client/xiaoban-multiplatform/README.md) | 规划 | 未来 App，固定保留此目录 |
| [client/xiaoban-realbot](client/xiaoban-realbot/README.md) | 规划 | 实体机器人客户端，平台与硬件待定 |
| [services/xiaban-api/backend](services/xiaban-api/backend/README.md) | 已实现 | Ktor 后端、业务、数据与后台任务 |
| [services/xiaban-api/api-contracts](services/xiaban-api/api-contracts/README.md) | 已实现 | 原 shared，按业务分包的 KMP 接口库 |
| [services/server-asr](services/server-asr/README.md) | 规划 | 语音识别服务 |
| [services/server-edgetts](services/server-edgetts/README.md) | 规划 | Edge TTS 服务，现有语音功能仍在主后端 |
| [services/server-websearch](services/server-websearch/README.md) | 规划 | 搜索能力服务 |

```text
client/                         面向用户的独立应用
services/xiaban-api/backend/     Gradle :backend
services/xiaban-api/api-contracts/ Gradle :api-contracts，JVM + iOS
services/server-*/               规划中的独立能力服务
infra/                          Compose、Nginx 与环境模板
tools/                          仓库开发辅助工具
docs/                           产品、架构、部署与设计资料
```

后端依赖接口库；网页维护自己的 TS API 类型；桌宠拥有自己的角色代码和测试，不再导入网页源码。未来 App 可引用接口库或维护自己的模型。架构和迁移映射见 [docs/architecture/monorepo.md](docs/architecture/monorepo.md)。

## 开发与验证

以下命令在仓库根执行；客户端各自保留 npm 锁文件，当前没有统一 npm workspace。

```sh
npm --prefix client/xiaban-web ci
npm --prefix client/xiaban-web run dev
npm --prefix client/xiaban-web test
npm --prefix client/xiaban-web run build
npm --prefix client/xiaoban-desktop-wallpaper ci
npm --prefix client/xiaoban-desktop-wallpaper test
npm --prefix client/xiaoban-desktop-wallpaper run desktop
./gradlew :api-contracts:jvmTest :backend:test
./gradlew :backend:run
./gradlew :backend:installDist
python -X utf8 tools/check-layout.py
```

Windows 使用 gradlew.bat；Kotlin 服务需要 JDK 21。后端分发输出在 services/xiaban-api/backend/build/install/backend。集成测试使用独立测试数据库和 Redis 第 15 号库；默认在依赖不可用时跳过，XIABAN_REQUIRE_INTEGRATION=true 可要求必须执行，详见后端 README。

部署配置在 [infra](infra/README.md)，说明在 [docs/deployment](docs/deployment/README.md)。容器命令显式指定文件：

```sh
docker compose -p xiaban --env-file /absolute/path/to/.env -f infra/compose.yaml config
docker compose -p xiaban --env-file /absolute/path/to/.env -f infra/compose.yaml up -d --build
```

## AI 开发边界

- 修改前阅读目标子项目 README，核实已实现/规划状态和真实入口。
- 按任务选择代码归属；允许重复代码，不自动提取共享包或跨目录导入另一个应用的源码。
- 接口库只描述 HTTP 路径、DTO、序列化枚举；鉴权、事件允许策略、称号配置、业务和后台任务归后端。
- Kotlin 包更名不等于改变 HTTP URL、JSON 字段、枚举编码或兼容语义。跨项目变化核对调用方、构建、测试、Docker、CI 和文档。
- 不建立含义模糊的全局 shared/common/utils 容器；规划工程开始实现时补充入口、环境与验证命令。
- 凭据、数据库、构建目录、安装包与日志不提交。部署与生产数据操作按当次任务范围执行。
