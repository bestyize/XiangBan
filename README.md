# 相伴 / 小伴大仓

按产品和部署职责管理客户端与能力服务。各产品独立维护代码、构建与依赖，允许重复实现；不以统一语言或强制共享 UI 为目标。

## 项目导航

| 项目 | 状态 | 说明 |
| --- | --- | --- |
| [client/xiaoban-desktop](client/xiaoban-desktop/README.md) | 3.0.0 | 小伴桌面：Windows/macOS 桌宠、动态壁纸、十二个桌面特效、游戏空间、关怀提醒与通用 Agent 聊天；接入自研 agent-kernel 内核与壁纸网站 |
| [client/xiaoban-multiplatform](client/xiaoban-multiplatform/README.md) | 可运行原型 | Android/iOS 使用 KMP + Compose；包含社区、小伴 AI 与手机桌面互联，桌面壳使用 Rust + Tauri |
| [services/server-xiaoban](services/server-xiaoban/README.md) | 已实现 | 正式壁纸网站 wallpaper.xiangban.xyz，图片/视频采集与上传、小伴桌面下载、版本与公告；桌面客户端接入此服务 |
| [services/server-xiaoban-market](services/server-xiaoban-market/README.md) | 已实现 | 小伴市场：专家、技能、连接器的自建目录服务，公开网页、下载 / 更新检测 API，以及 SkillHub、魔搭社区的采集工作台；Rust Axum + PostgreSQL + Vue 3 + Vite |
| [services/server-forum](services/server-forum/README.md) | 一期已实现，已部署 | 超级小伴论坛（chat.xiangban.xyz）：技术交流、资源分享、福利羊毛与闲聊的社区，Rust Axum + PostgreSQL + Vue 3，登录走统一账号中心，带管理后台和 SEO 页面 |
| [services/server-ai](services/server-ai/README.md) | 一期已实现，部署状态待核实 | AI 网关与额度系统：账号中心验签，经 New API 调用上游模型；按「账号 × 产品」隔离的试用、月卡与加油包额度，档位对用户隐藏真实模型；Rust Axum + PostgreSQL + Vue 3 管理后台。设计见 [docs/architecture/ai-gateway.md](docs/architecture/ai-gateway.md) |
| [services/server-account](services/server-account/README.md) | 已实现 | 统一账号中心与管理后台，邮箱/手机号验证码验证身份、用户名与密码登录，Rust Axum + PostgreSQL + Vue 3，签发 EdDSA JWT；壁纸网站等客户端已接入 |
| [libs/agent-kernel](libs/agent-kernel/README.md) | 0.3.0，已接入小伴桌面 | 跨产品复用的 Rust Agent 内核：宿主注入模型、工具与存储，支持三种模型协议、权限、文件、网页、Shell、skills、子 Agent、MCP 与 stdio；可选运行日志与受管进程，不含产品逻辑 |
| [libs/agent-proactivity](libs/agent-proactivity/README.md) | 0.1.0，基础设施已接入桌面 | 独立 Rust 主动感知策略引擎：事件聚合、授权/免打扰、统一注意力预算、去重、反馈；默认关闭主动展示，暂不新增悬停气泡或 Agent 后台调用 |

```text
client/                         面向用户的独立应用
assets/game-packs/              游戏资源基准包，用于桌面构建校验
services/server-*/               独立业务服务与规划中的能力服务
libs/                           跨产品复用的 Agent 内核、学习、工具与主动感知库
infra/                          Compose、Nginx 与环境模板
tools/                          仓库开发辅助工具
docs/                           产品、架构、部署与设计资料
```

各客户端自行维护代码、协议模型和测试。架构和迁移映射见 [docs/architecture/monorepo.md](docs/architecture/monorepo.md)。

产品名称、项目归属与历史兼容标识见根目录 [项目名称与术语](TERMINOLOGY.md)。

各目录导航见 [client](client/README.md)、[services](services/README.md)、[libs](libs/README.md)；文档入口见 [docs](docs/README.md)。

## 开发与验证

以下命令在仓库根执行；客户端各自保留 npm 锁文件，当前没有统一 npm workspace。

```sh
npm --prefix client/xiaoban-desktop ci
npm --prefix client/xiaoban-desktop test
npm --prefix client/xiaoban-desktop run desktop
npm --prefix client/xiaoban-desktop run desktop:macos:build
python -X utf8 tools/check-layout.py
npm --prefix services/server-xiaoban/web ci
npm --prefix services/server-xiaoban/web run build
cargo test --locked --manifest-path services/server-xiaoban/Cargo.toml
npm --prefix services/server-xiaoban-market/web ci
npm --prefix services/server-xiaoban-market/web test
npm --prefix services/server-xiaoban-market/web run build
cargo test --manifest-path services/server-xiaoban-market/Cargo.toml
npm --prefix services/server-account/web ci
npm --prefix services/server-account/web run build
cargo test --locked --manifest-path services/server-account/Cargo.toml
npm --prefix services/server-forum/web ci
npm --prefix services/server-forum/web run build
cargo test --manifest-path services/server-forum/Cargo.toml
```


小伴三端安装包由 `.github/workflows/build-xiaoban-wallpaper.yml`（BuildXiaoBanWallpaper）打包：Windows x64 NSIS 一键安装包（当前用户目录、无管理员权限、不再发布便携版）、macOS aarch64 的 dmg 与 `.app`（GitHub Release 使用固定自签名证书，未公证；本地默认 ad-hoc）、小伴手机 App 的 Android APK。产物上传到 GitHub Release `xiaoban-wallpaper-v<版本号>`（没有就自动创建，重跑时替换同名文件），不占用 Actions artifact 存储额度。仅在 Actions 页手动运行；推送代码或 `xiaoban-wallpaper-v*` 标签不会触发构建。Android App 使用 `client/xiaoban-multiplatform`，签名读取仓库 secret `XIAOBAN_ANDROID_KEYSTORE_BASE64`、`XIAOBAN_ANDROID_KEYSTORE_PASSWORD`、`XIAOBAN_ANDROID_KEY_ALIAS`、`XIAOBAN_ANDROID_KEY_PASSWORD`，缺少时产出调试 APK。本地 Android release 需要 JDK 17+，GraalVM 在 jlink 步骤会失败，可改用 Android Studio 自带的 JBR。

正式壁纸网站位于 `services/server-xiaoban`，使用 Node.js 24、Rust stable 和独立 PostgreSQL，默认端口 8080。桌面客户端默认连接 `https://wallpaper.xiangban.xyz`；它的数据库集成测试使用 `TEST_DATABASE_URL`，每个测试创建并清理独立 schema，不更改已有数据。

统一账号中心位于 `services/server-account`，使用独立 PostgreSQL 库，默认端口 55009，管理后台在 `/admin/`；集成测试使用 `TEST_DATABASE_URL`，未设置时跳过。设计见 [docs/architecture/account-center.md](docs/architecture/account-center.md)。 独立部署栈在 `infra/account`，当前没有账号中心专用工作流，本地打包与上传流程见 [docs/deployment/ACCOUNT_CENTER_DEPLOYMENT.md](docs/deployment/ACCOUNT_CENTER_DEPLOYMENT.md)。

部署配置在 [infra](infra/README.md)，说明在 [docs/deployment](docs/deployment/README.md)。云端应上传预构建产物。

## 实现建议

新建项目时的默认技术选型；已有项目不因此强制迁移，特殊场景可按任务偏离并在子项目 README 说明原因。

- 后端服务：优先使用 Rust + PostgreSQL（参考 `server-account`）。
- 网页与管理后台：UI 优先使用 Vue 3 + Vite。
- 移动端应用：优先使用 Jetpack Compose（Compose Multiplatform 视跨平台需要选用）。

## AI 开发边界

- 修改前阅读目标子项目 README，核实已实现/规划状态和真实入口。
- 按任务选择代码归属；允许重复代码，不自动提取共享包或跨目录导入另一个应用的源码。
- 接口库只描述 HTTP 路径、DTO、序列化枚举；鉴权、事件允许策略、称号配置、业务和后台任务归后端。
- Kotlin 包更名不等于改变 HTTP URL、JSON 字段、枚举编码或兼容语义。跨项目变化核对调用方、构建、测试、Docker、CI 和文档。
- 不建立含义模糊的全局 shared/common/utils 容器；规划工程开始实现时补充入口、环境与验证命令。
- 凭据、数据库、构建目录、安装包与日志不提交。部署与生产数据操作按当次任务范围执行。
