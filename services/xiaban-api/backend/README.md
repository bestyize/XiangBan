# 相伴主后端

状态：已实现。Kotlin/Ktor 单体服务，入口 src/main/kotlin/xyz/xiaban/server/Application.kt，JDK 21、PostgreSQL、Redis；Gradle 名称 :backend。

## 代码归属

- users/account/auth：会话、账户、资料、身份与权限。
- checkin/stats/unlocks：打卡、称号、统计、解锁规则。
- chat/memory/llm/tts：聊天、记忆、模型与语音，当前仍在本服务。
- guardian/mail/events/admin：守护、通知、埋点、管理。
- src/main/resources/db/migration：Flyway 迁移；已有迁移不改，追加新迁移。
- checkin/TitleCatalog.kt、companion/PersonalityCatalog.kt：服务端显示配置。
- events/EventNames.kt：允许事件与访客策略，属于后端业务规则。

## 开发与验证

从仓库根运行 gradlew :backend:run、gradlew :backend:test、gradlew :backend:installDist；Windows 使用 gradlew.bat。分发在 services/xiaban-api/backend/build/install/backend，启动脚本 bin/backend。

配置见 config/AppConfig.kt，环境模板见 infra/deploy/.env.production.example。集成测试使用 xiaban_test 数据库与 Redis 第 15 号库。XIABAN_TEST_DATABASE_URL / XIABAN_TEST_REDIS_URL 可指定独立环境；XIABAN_REQUIRE_INTEGRATION=true 时依赖不可用会失败。不要用生产库执行测试，跳过的测试不能算完成集成验证。

## 边界

业务在这里，传输模型在 ../api-contracts。不把数据库、Ktor、事件白名单、称号规则或后台任务放入接口库。API 变动核对兼容与调用方；权限不以客户端 UI 状态替代。

两个 Dockerfile 均使用仓库根上下文。Dockerfile 构建分发；Dockerfile.runtime 用预构建分发。容器逻辑服务名仍为 server、镜像名 xiaban-server，以兼容现有部署。
