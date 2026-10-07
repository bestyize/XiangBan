# 部署资源

保存 Nginx、环境模板和 VMware 预构建叠加配置。基础 Compose 在 ../compose.yaml；步骤和历史交接在 docs/deployment。

Docker 上下文始终为仓库根；Nginx 代理到 server 服务，网页资源来自 client/xiaban-web。改路径时同步 Dockerfile、Compose 和工作流。
