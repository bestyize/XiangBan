# 部署说明

- [DEPLOYMENT.md](DEPLOYMENT.md)：主站部署配置；现有工作流仍在远端构建，使用前须按文中限制处理。
- [ACCOUNT_DATA_MODEL.md](ACCOUNT_DATA_MODEL.md)：账户和数据模型。
- [ACCOUNT_CENTER_DEPLOYMENT.md](ACCOUNT_CENTER_DEPLOYMENT.md)：统一账号中心的本地打包、独立部署与回滚；当前没有账号中心专用工作流。
- [MOBILE_LINK_ROLLOUT.md](MOBILE_LINK_ROLLOUT.md)：手机互联上线清单——账号中心、server-ai、server-forum、server-link 的顺序与检查方法。
- [LINK_DEPLOYMENT.md](LINK_DEPLOYMENT.md)：互联中转 server-link 的独立部署栈、验证与备份。
- [MARKET_DEPLOYMENT.md](MARKET_DEPLOYMENT.md)：小伴市场的手动测试、打包、部署与备份。

模板位于 [infra](../../infra/README.md)。云服务器使用本地或构建机构建好的产物，不在远端执行大型编译。现存 GitHub Actions 均仅手动触发；推送代码或标签不会启动构建、部署。旧 VMware 机器地址与会话交接提示词已移除；预构建配置仍保留在 `infra/deploy/compose.vmware-prebuilt.yaml`。
