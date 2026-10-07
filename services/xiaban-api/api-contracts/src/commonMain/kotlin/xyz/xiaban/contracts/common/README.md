# common 接口包

这是 api-contracts 的业务包，不是独立 Gradle 模块。

仅放错误/成功/健康检查和公共启动开关，避免扩展为通用业务或工具容器。

修改前阅读接口库根 README。保持 HTTP 路径、@SerialName、JSON 字段、默认值和可空语义兼容；新增业务执行逻辑放入后端。
