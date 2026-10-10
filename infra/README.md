# 基础设施

compose.yaml 管理 PostgreSQL、Redis、主后端和网页；deploy/nginx 是网页镜像代理配置；deploy/.env.production.example 是环境模板；deploy/compose.vmware-prebuilt.yaml 用于预构建后端的部署叠加。

Compose 的上下文 .. 指向仓库根，Dockerfile 路径以仓库根为准。服务/镜像/卷名保持原值，源码迁移不更换生产数据卷。

account 目录是统一账号中心的独立部署栈（自带 PostgreSQL，不属于上面的主站 compose）：compose.yaml、环境模板与 Nginx 反代示例，部署说明见 [docs/deployment/ACCOUNT_CENTER_DEPLOYMENT.md](../docs/deployment/ACCOUNT_CENTER_DEPLOYMENT.md)。

从根目录显式指定 -f infra/compose.yaml，并按需要附加 -f infra/deploy/compose.vmware-prebuilt.yaml。操作说明见 [docs/deployment](../docs/deployment/README.md)。保存模板与配置，不存真实凭据、数据库、包体和临时日志；实际部署以当次任务为范围。
