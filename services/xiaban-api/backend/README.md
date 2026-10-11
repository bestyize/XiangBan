# 相伴主后端

状态：已实现。Kotlin/Ktor 单体服务，入口 src/main/kotlin/xyz/xiaban/server/Application.kt，JDK 21、PostgreSQL、Redis；Gradle 名称 :backend。

## 代码归属

- users/account/auth：会话、账户、资料、身份与权限。统一账号中心登录见 account/AccountCenter*.kt：`POST /api/account/center/login` 验签账号中心令牌并换成本站令牌，配置 `ACCOUNT_ISSUER`（默认 https://account.xiangban.xyz）、`ACCOUNT_BASE_URL`（默认同签发者，取 JWKS 与 /api/me）、`ACCOUNT_AUDIENCES`（默认 xiaban-web,xiaban-api）；迁移 V19 增加 `users.account_uid`，V20 去掉 `user_profiles.nickname_key`：网页和 App 里的「昵称」就是账号中心里唯一且不可修改的用户名，登录时镜像过来，用户自己不能改。旧的本地邮箱验证码登录接口（`/api/account/login*`、`bind*`）已删除，登录只走账号中心。
- checkin/stats/unlocks：打卡、称号、统计、解锁规则。
- chat/memory/llm/tts：聊天、记忆、模型与语音，当前仍在本服务。
- guardian/mail/events/admin：守护、通知、埋点、管理。
- src/main/resources/db/migration：Flyway 迁移；已有迁移不改，追加新迁移。
- checkin/TitleCatalog.kt、companion/PersonalityCatalog.kt：服务端显示配置。
- events/EventNames.kt：允许事件与访客策略，属于后端业务规则。
- games/SnakeWallet.kt：贪吃蛇糖果币与道具策略，V18 追加钱包、每局状态和幂等请求记录。钱包初始 40 币；保护卡 20 币、续命 30 币，每局只能续命一次。结束的本人贪吃蛇记录可结算，每日最多 300 币；分享参与奖励每日一次赠送下一局 5 秒开局保护，不代表验证了真实转发。每日边界统一为 Asia/Shanghai。`GET /api/games/snake-arena/wallet` 与 `POST /api/games/snake-arena/wallet/actions` 使用账户鉴权，事务锁保证余额和道具扣减，成绩与结束状态读取服务器记录。

贪吃蛇钱包随服务器游客账号的数据合并转移：只有一方有钱包时原样保留，双方都有时余额相加后扣除一份 40 币新手赠送，最低为零；不追溯已消耗道具，也不产生欠额。保护卡库存相加，待领取开局保护取较长值；当日结算额度相加封顶 300，分享已领取状态保留。每局消费标记与游戏记录保持原 session ID，已结算局不可二次领奖。同名幂等请求保留目标账号记录，其余随源账号转移。账号导出包含钱包、每局道具状态与操作记录；删除账号时由外键级联清除。

## 开发与验证

从仓库根运行 gradlew :backend:run、gradlew :backend:test、gradlew :backend:installDist；Windows 使用 gradlew.bat。分发在 services/xiaban-api/backend/build/install/backend，启动脚本 bin/backend。

配置见 config/AppConfig.kt，环境模板见 infra/deploy/.env.production.example。集成测试使用 xiaban_test 数据库与 Redis 第 15 号库。XIABAN_TEST_DATABASE_URL / XIABAN_TEST_REDIS_URL 可指定独立环境；XIABAN_REQUIRE_INTEGRATION=true 时依赖不可用会失败。不要用生产库执行测试，跳过的测试不能算完成集成验证。

## 边界

业务在这里，传输模型在 ../api-contracts。不把数据库、Ktor、事件白名单、称号规则或后台任务放入接口库。API 变动核对兼容与调用方；权限不以客户端 UI 状态替代。

两个 Dockerfile 均使用仓库根上下文。Dockerfile 构建分发；Dockerfile.runtime 用预构建分发。容器逻辑服务名仍为 server、镜像名 xiaban-server，以兼容现有部署。
