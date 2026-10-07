# 相伴网页 / PWA

状态：已实现。Vite + React + TypeScript + Tailwind；构建为静态站点，Nginx 将 /api 代理到主后端。管理后台仍在本应用内。

## 源码与职责

- src/pages：账户、首页聊天、打卡、游戏、测试、管理后台。
- src/api/types.ts：网页自己的 API 类型，保持后端 JSON 兼容，不导入 Kotlin 源码。
- src/robot 与 src/components/Robot.tsx：网页角色和 React 包装；桌宠维护自己的副本。
- src/poster：分享海报；public/game-packs：沙箱游戏包。
- 修改对应功能前阅读 [小游戏约定](src/games/README.md) 或 [测试约定](src/quizzes/README.md)。

## 开发与验证

在本目录运行 npm ci、npm run dev、npm test、npm run build、npm run lint。开发代理见 vite.config.ts，需要后端时从仓库根运行 gradlew :backend:run。

Dockerfile 的上下文是仓库根，使用 infra/deploy/nginx 配置。不要直接把本目录作为 Docker 构建上下文。

## 边界

网页专属 UI、状态、样式和角色逻辑留在本应用。改 API 时同时核对 services/xiaban-api 的协议和路由；不要根据 Kotlin 包名称改浏览器 URL。角色测试中有一项核对后端解锁目录的集成校验，不是桌宠的依赖。

游戏包遵守宿主消息协议、账户隔离、沙箱与分享校验；账户身份以服务器为准。docs/design-preview 只是设计参考，不直接覆盖运行页面。
