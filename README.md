# 相伴 / 小伴大仓

按产品和部署职责管理客户端、主 API 与规划中的能力服务。各产品独立维护代码、构建与依赖，允许重复实现；不以统一语言或强制共享 UI 为目标。

## 项目导航

| 项目 | 状态 | 说明 |
| --- | --- | --- |
| [client/xiaban-web](client/xiaban-web/README.md) | 已实现 | 相伴网页/PWA，包含管理后台 |
| [client/xiaoban-desktop-wallpaper](client/xiaoban-desktop-wallpaper/README.md) | 1.0.0 | 小伴壁纸桌面版：Windows/macOS 桌宠、动态壁纸、十三个桌面特效（数据驱动的特效包）、游戏空间与关怀提醒；接入壁纸网站的在线壁纸、版本与公告 |
| [client/xiaoban-pot-garden](client/xiaoban-pot-garden/README.md) | 可运行原型 | 表情盆栽的 3D 质感原型（Vite + three.js），花盆养成游戏的起点，目前只有表情和质感，没有玩法 |
| [client/xiaoban-mobile-wallpaper](client/xiaoban-mobile-wallpaper/README.md) | 1.0.0 | 小伴壁纸 Android 版：浏览与下载壁纸网站的图片和视频，Compose 瀑布流，可登录统一账号，收藏只存本机；release 只打 arm64-v8a |
| [client/xiaoban-multiplatform](client/xiaoban-multiplatform/README.md) | 可运行原型 | 把 xiaban.xyz 套进原生壳：Android/iOS 用 KMP + Compose，桌面用 Rust + Tauri（`desktopApp/`）；Android 与 macOS 已运行验证，iOS 仅编译 |
| [client/xiaoban-realbot](client/xiaoban-realbot/README.md) | 规划 | 实体机器人客户端，平台与硬件待定 |
| [services/xiaban-api/backend](services/xiaban-api/backend/README.md) | 已实现 | Ktor 后端、业务、数据与后台任务 |
| [services/xiaban-api/api-contracts](services/xiaban-api/api-contracts/README.md) | 已实现 | 原 shared，按业务分包的 KMP 接口库 |
| [services/server-pan](services/server-pan/README.md) | 已实现 | PanDown 自托管网盘，Rust Axum + SQLite 后端与 React 前端，独立于主 API |
| [services/server-xiangban-wallpaper](services/server-xiangban-wallpaper/README.md) | 已实现 | 正式壁纸网站 wallpaper.xiangban.xyz，图片/视频采集与上传、小伴桌面下载、版本与公告；桌面客户端接入此服务 |
| [services/server-xiaoban-appstore](services/server-xiaoban-appstore/README.md) | 已实现 | 小伴优质应用商店，Rust Axum + PostgreSQL 后端与 Vue 3 + Vite 前端，腾讯柠檬精选美学与管理后台 |
| [services/server-forum](services/server-forum/README.md) | 一期已实现，已部署 | 超级小伴论坛（chat.xiangban.xyz）：技术交流、资源分享、福利羊毛与闲聊的社区，Rust Axum + PostgreSQL + Vue 3，登录走统一账号中心，带管理后台和 SEO 页面 |
| [services/server-account](services/server-account/README.md) | 已实现 | 统一账号中心与管理后台，邮箱/手机号验证码注册登录，Rust Axum + PostgreSQL + Vue 3，签发 EdDSA JWT；壁纸网站与相伴网页（xiaban.xyz，经主后端换票）已接入 |
| [services/server-asr](services/server-asr/README.md) | 规划 | 语音识别服务 |
| [services/server-edgetts](services/server-edgetts/README.md) | 规划 | Edge TTS 服务，现有语音功能仍在主后端 |
| [services/server-websearch](services/server-websearch/README.md) | 规划 | 搜索能力服务 |

```text
client/                         面向用户的独立应用
services/xiaban-api/backend/     Gradle :backend
services/xiaban-api/api-contracts/ Gradle :api-contracts，JVM + iOS
services/server-pan/             独立网盘服务：backend（Cargo）+ frontend（npm）
services/server-*/               其余规划中的独立能力服务
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
npm --prefix client/xiaoban-desktop-wallpaper run desktop:macos:build
npm --prefix client/xiaoban-pot-garden ci
npm --prefix client/xiaoban-pot-garden test
npm --prefix client/xiaoban-pot-garden run build
(cd client/xiaoban-mobile-wallpaper && ./gradlew :app:testDebugUnitTest :app:assembleDebug)
(cd client/xiaoban-mobile-wallpaper && ./gradlew :app:assembleRelease)
./gradlew :api-contracts:jvmTest :backend:test
./gradlew :backend:run
./gradlew :backend:installDist
npm --prefix services/server-pan/frontend ci
npm --prefix services/server-pan/frontend run build
cargo build --release --manifest-path services/server-pan/backend/Cargo.toml
python -X utf8 tools/check-layout.py
npm --prefix services/server-xiangban-wallpaper/web ci
npm --prefix services/server-xiangban-wallpaper/web run build
cargo test --locked --manifest-path services/server-xiangban-wallpaper/Cargo.toml
npm --prefix services/server-xiaoban-appstore/web run build
cargo check --manifest-path services/server-xiaoban-appstore/Cargo.toml
npm --prefix services/server-account/web ci
npm --prefix services/server-account/web run build
cargo test --locked --manifest-path services/server-account/Cargo.toml
npm --prefix services/server-forum/web ci
npm --prefix services/server-forum/web run build
cargo test --manifest-path services/server-forum/Cargo.toml
```

Windows 使用 gradlew.bat；Kotlin 服务需要 JDK 21。Android 壁纸需要 JDK 17+ 与 Android SDK，并自带 Gradle wrapper。网盘服务需要 Rust 1.80+ 与 Node.js 18+，默认端口 55005，运行方式与数据目录要求见其 README；它不接入根 Gradle，也不依赖主 API 的账户与数据库。应用商店服务位于 `services/server-xiaoban-appstore`，使用 Rust Axum、PostgreSQL 与 Vue 3 + Vite，默认端口 55008，包含腾讯柠檬精选美学前台与经典管理后台（支持一键上下架）。后端分发输出在 services/xiaban-api/backend/build/install/backend。集成测试使用独立测试数据库和 Redis 第 15 号库；默认在依赖不可用时跳过，XIABAN_REQUIRE_INTEGRATION=true 可要求必须执行，详见后端 README。

小伴壁纸三端安装包由 `.github/workflows/build-xiaoban-wallpaper.yml`（BuildXiaoBanWallpaper）打包：Windows x64 便携 EXE、macOS aarch64 的 dmg 与 `.app`（ad-hoc 签名，未公证）、Android arm64-v8a release APK。产物上传到 GitHub Release `xiaoban-wallpaper-v<版本号>`（没有就自动创建，重跑时替换同名文件），不占用 Actions artifact 存储额度。在 Actions 页手动运行，或推送 `xiaoban-wallpaper-v*` 标签触发。Android 签名读取仓库 secret `XIAOBAN_ANDROID_KEYSTORE_BASE64`、`XIAOBAN_ANDROID_KEYSTORE_PASSWORD`、`XIAOBAN_ANDROID_KEY_ALIAS`、`XIAOBAN_ANDROID_KEY_PASSWORD`，缺少时产出未签名 APK。本地 Android release 需要 JDK 17+，GraalVM 在 jlink 步骤会失败，可改用 Android Studio 自带的 JBR。

正式壁纸网站位于 `services/server-xiangban-wallpaper`，使用 Node.js 24、Rust stable 和独立 PostgreSQL，默认端口 8080。桌面客户端默认连接 `https://wallpaper.xiangban.xyz`；它的数据库集成测试使用 `TEST_DATABASE_URL`，每个测试创建并清理独立 schema，不更改已有数据。

统一账号中心位于 `services/server-account`，使用独立 PostgreSQL 库，默认端口 55009，管理后台在 `/admin/`；集成测试使用 `TEST_DATABASE_URL`，未设置时跳过。设计见 [docs/architecture/account-center.md](docs/architecture/account-center.md)。 独立部署栈在 `infra/account`，打包与部署工作流为 `.github/workflows/account-center.yml`，流程见 [docs/deployment/ACCOUNT_CENTER_DEPLOYMENT.md](docs/deployment/ACCOUNT_CENTER_DEPLOYMENT.md)。

部署配置在 [infra](infra/README.md)，说明在 [docs/deployment](docs/deployment/README.md)。容器命令显式指定文件：

```sh
docker compose -p xiaban --env-file /absolute/path/to/.env -f infra/compose.yaml config
docker compose -p xiaban --env-file /absolute/path/to/.env -f infra/compose.yaml up -d --build
```

## 实现建议

新建项目时的默认技术选型；已有项目不因此强制迁移，特殊场景可按任务偏离并在子项目 README 说明原因。

- 后端服务：优先使用 Rust + PostgreSQL（参考 `server-xiaoban-appstore`、`server-account`）。
- 网页与管理后台：UI 优先使用 Vue 3 + Vite。
- 移动端应用：优先使用 Jetpack Compose（Compose Multiplatform 视跨平台需要选用）。

## AI 开发边界

- 修改前阅读目标子项目 README，核实已实现/规划状态和真实入口。
- 按任务选择代码归属；允许重复代码，不自动提取共享包或跨目录导入另一个应用的源码。
- 接口库只描述 HTTP 路径、DTO、序列化枚举；鉴权、事件允许策略、称号配置、业务和后台任务归后端。
- Kotlin 包更名不等于改变 HTTP URL、JSON 字段、枚举编码或兼容语义。跨项目变化核对调用方、构建、测试、Docker、CI 和文档。
- 不建立含义模糊的全局 shared/common/utils 容器；规划工程开始实现时补充入口、环境与验证命令。
- 凭据、数据库、构建目录、安装包与日志不提交。部署与生产数据操作按当次任务范围执行。
