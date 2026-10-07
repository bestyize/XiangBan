# 主后端接口协议库

原 shared，现名 api-contracts，由主后端维护。Gradle :api-contracts，目标 JVM、iOS arm64、iOS 模拟器 arm64；当前 backend 直接依赖，未来 App 可自行选择。

## 业务包

源码根 src/commonMain/kotlin/xyz/xiaban/contracts：

| 包 | 内容 |
| --- | --- |
| common | 错误、成功、健康检查、启动公共开关 |
| account/auth | 登录、绑定邮箱、退出、账户生命周期路径 |
| account/session | 会话 |
| account/profile | 资料、城市、元信息、订阅 |
| account/guardian | 平安守护传输数据 |
| account/consent | 条款与授权 |
| companion/chat | 历史、流式请求、语音请求 |
| companion/personality | 性格、心情的协议枚举和展示响应 |
| afterwork/checkin | 打卡、称号编码、回复、问候 |
| afterwork/stats | 统计、百分比、提示 |
| afterwork/unlocks | 解锁记录 |
| admin/settings | 管理与展示配置 DTO |
| admin/reports | 举报传输数据，权限仍由路由决定 |
| telemetry | 事件格式与批次 |

三级包当前不是独立 Gradle 模块。各领域维护 *ApiPaths 对象，原 HTTP 字符串保留。

## 兼容与边界

这里只描述传输格式，不执行业务。@SerialName、JSON 字段、默认值、可空语义和 URL 是外部协议，改 Kotlin 包不应改变这些值。称号标签/稀有度/心情配置、性格标签与事件白名单已归后端。

网页自行维护 TS 类型，桌宠当前离线。允许客户端独立模型，不自动扩展成全产品的 shared 包。

从根目录运行 gradlew :api-contracts:jvmTest 与 gradlew :backend:test。commonTest 覆盖旧请求、JSON 和路径兼容；iOS 编译链接需要 Mac。对外变动明确版本、调用方和兼容验证。
