# 下班啦（xiaban.xyz）

每天下班打一次卡，小伴（一只嘴有点毒、心很软的猫）给你一个今日称号、一句调侃、几条今晚用得上的信息，再生成一张能晒的下班卡。产品需求见 [PRD_下班啦.md](PRD_下班啦.md)。

## 技术栈

| 层 | 选型 |
|---|---|
| 前端 `web/` | Vite 8 + React 19 + TypeScript 6 + Tailwind CSS 4，wouter 路由，vite-plugin-pwa；分享卡在浏览器里用 Canvas 生成 |
| 后端 `server/` | Kotlin 2.4 + Ktor 3（Netty），单体、无状态；JWT 匿名会话 |
| 共享模块 `shared/` | Kotlin Multiplatform（JVM + iOS），放 API 路径和 DTO，相伴 App 可以直接依赖 |
| 数据库 | PostgreSQL 17，HikariCP + 原生 SQL，Flyway 管理迁移 |
| 缓存与队列 | Redis（Lettuce）：去重、百分比统计、平安守护到期队列；大模型润色和邮件走 Redis Streams |
| 部署 | Docker Compose + Nginx，配置全部走环境变量 |

## 目录结构

```
shared/   Kotlin Multiplatform：API 路径、请求和响应模型
server/   Ktor 后端：业务规则、接口、定时任务、数据库迁移
web/      React 前端（构建成纯静态资源）
```

## 本地开发

需要 JDK 21、Node 22.12+、PostgreSQL 17、Redis 7+。macOS 上可以用 Homebrew：

```bash
brew install postgresql@17 redis
```

创建开发库和测试库（只需一次）：

```bash
createuser -s xiaban && psql -d postgres -c "alter user xiaban password 'xiaban'" && createdb -O xiaban xiaban && createdb -O xiaban xiaban_test
```

启动后端（默认 8080 端口，启动时自动执行数据库迁移）：

```bash
./gradlew :server:run
```

启动前端（默认 5173 端口，`/api` 代理到 8080）：

```bash
cd web && npm install && npm run dev
```

## 测试

```bash
./gradlew build
```

```bash
cd web && npm test && npm run build
```

后端集成测试连接本机的 `xiaban_test` 库和 Redis 第 15 号库，连不上时自动跳过。设置 `XIABAN_REQUIRE_INTEGRATION=true` 可以强制要求执行（CI 里使用）。
