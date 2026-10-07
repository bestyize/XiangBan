# 相伴主 API

状态：已实现。两个模块由仓库根 Gradle wrapper 管理：

| 子项目 | Gradle 名称 | 职责 |
| --- | --- | --- |
| [backend](backend/README.md) | :backend | Ktor 运行服务、业务、鉴权、存储、后台任务 |
| [api-contracts](api-contracts/README.md) | :api-contracts | HTTP 路径、DTO、序列化枚举，JVM/iOS KMP 库 |

依赖方向为 backend → api-contracts，接口库不依赖后端实现。客户端自行选择模型来源。HTTP/JSON 是对外协议，Kotlin 包层次只是源码组织。

实现改动先读 backend README；协议改动同时读 api-contracts README 并核对调用方。部署说明在根 docs/deployment，配置在 infra。
