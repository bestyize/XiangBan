# 基础设施

原主后端已停用；compose.yaml 及其环境模板仅供参考，不能再从源码构建。deploy/.env.production.example 是环境模板；deploy/compose.vmware-prebuilt.yaml 用于预构建后端的部署叠加。

Compose 的上下文 .. 指向仓库根，Dockerfile 路径以仓库根为准。服务/镜像/卷名保持原值，源码迁移不更换生产数据卷。

account 目录是统一账号中心的独立部署栈（自带 PostgreSQL，不属于上面的主站 compose）：compose.yaml、环境模板与 Nginx 反代示例，部署说明见 [docs/deployment/ACCOUNT_CENTER_DEPLOYMENT.md](../docs/deployment/ACCOUNT_CENTER_DEPLOYMENT.md)。

market 目录是小伴市场（专家 / 技能 / 连接器目录与采集工作台）的独立部署栈：compose.yaml、环境模板与 Nginx 反代示例，部署说明见 [docs/deployment/MARKET_DEPLOYMENT.md](../docs/deployment/MARKET_DEPLOYMENT.md)。

people 目录是人格贩卖机 H5（`people.xiaban.xyz`）的香港机纯静态部署模板与本地构建同步脚本，见 [infra/people/README.md](people/README.md)。

game 目录是小伴游戏厅 H5（`game.xiaban.xyz`）的香港机纯静态部署模板与本地构建同步脚本，见 [infra/game/README.md](game/README.md)。

portal 目录是小伴产品矩阵总站与双端独立下载页（`xiaban.xyz`、`/desktop`、`/app`）的香港机纯静态部署模板、Nginx `/api/v1/desktop/` 反代配置与一键构建同步脚本，见 [infra/portal/README.md](portal/README.md)。

aireset 目录是 AI订阅测评 QuotaReset（`aireset.xiaban.xyz`）的香港机纯静态部署模板（HTTP / HTTPS 两阶段 Nginx 配置）与本地构建同步脚本，见 [infra/aireset/README.md](aireset/README.md)。

从根目录显式指定 -f infra/compose.yaml，并按需要附加 -f infra/deploy/compose.vmware-prebuilt.yaml。其他服务操作说明见 [docs/deployment](../docs/deployment/README.md)。保存模板与配置，不存真实凭据、数据库、包体和临时日志；实际部署以当次任务为范围。

images 目录是服务端镜像的统一打包：services.json 是清单，gen-dockerfiles.py 生成各服务的运行时 Dockerfile，build-image.sh 编译并导出镜像，GitHub Actions 的 BuildXiaoBanWallpaper 工作流用同一套。流程见 [docs/deployment/RELEASE_AND_DEPLOY.md](../docs/deployment/RELEASE_AND_DEPLOY.md)。
