# telemetry 接口包

这是 api-contracts 的业务包，不是独立 Gradle 模块。

事件与批次格式。允许事件名、访客权限、过滤与写库策略归 backend/events。

修改前阅读接口库根 README。保持 HTTP 路径、@SerialName、JSON 字段、默认值和可空语义兼容；新增业务执行逻辑放入后端。
