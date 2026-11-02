# 社区采编执行器

独立 Python 3.10+ sidecar，只有标准库依赖。运行时业务属于 server-forum，不依赖 Codex、Claude 或本机开机。通过论坛已有的受限作者令牌和 server-ai 产品调用，不能访问论坛管理接口。不改变 GitHub Actions 的手动触发方式。

`agents.json` 按当前可见入口登记 41 个目标。首批启用有官方来源的 Claude Code、Codex、提示词、工作流、本地部署、AI 硬件和小伴前沿；其他目标保留明确的待接入原因，不能据此宣称所有栏目已上线。作品、方案、工具、资料、日报、前沿与帖子是不同数据类型，执行器拒绝启用尚未实现的发布适配器。排行榜不计数。求助和举报公示依赖真实事件，不能用虚构请求补数量。

流程：来源白名单 → 官方页面资料 → 写作 → 单独事实审校 → 必要时一次修订和复审 → 生成封面 → 待发布稿 → 幂等发布 → 公开详情和封面回读。文章标明公开资料整理、未实测及概念插画。审校绑定稿件哈希；修改后需重新审校。模型审校有局限，需持续抽查并处理更正，不应将其称为人工终审。

状态保存在 `/var/lib/xiaoban-editorial/`：SQLite 队列、调用预留记录、阶段事件、来源证据、稿件、核查意见、封面及公开链接。响应丢失后先按原 `external_id` 查稿件再恢复，不重新发布。单机文件锁避免重叠；这个实现不支持多个服务器共用队列。失败最多尝试三次、间隔一小时；审校拒绝保留给编辑处理，不为了数量强行发表。修改拒绝稿后，应移除旧 `review.json` 并由运营人员显式重新入队。

费用通过独立运营产品余额、每日调用数、单次处理数和每栏目每日篇数约束。按北京时间记账，默认每日最多 240 次模型请求、最多消耗 250 万额度点（当前换算约 5 美元），保留 100 万点。网关对账有延迟，金额是停止阈值，并非上游硬性账单封顶；单次调用还受网关已有额度检查限制。不要另行自动补发额度。

部署文件位于 `deploy/`，使用专用无登录用户 `xiaoban-editorial`，代码放 `/opt/xiaoban-editorial`。上传本地校验后的 Python 源码即可，无云端编译。私密状态目录必须有 `editor-token`（绑定作者且允许发布）及 `ai-session.json`（运营客户端的可刷新账号会话），以及 `discover-token`（仅授予 frontier 草稿及自动发布权限）；均为 600、目录为 700，只由服务用户持有。会话由该进程独占刷新，不在其他主机复制使用；撤销会话或创作令牌可立即停止后续调用。不得上传账号密码。

```sh
python3 -m unittest discover -s services/server-forum/editorial -p 'test_*.py'
python3 services/server-forum/editorial/worker.py --root /private/state audit
python3 services/server-forum/editorial/worker.py --root /private/state run --limit 1
python3 services/server-forum/editorial/worker.py --root /private/state status
```

`audit` 按真实公开接口生成 `coverage.json`，公开总数与本执行器核验过的发布数分列；已有 20 条不代表质量验收通过。`status.json` 是最近一轮状态，命令 `status` 即时查询队列。日志不写凭据、模型原文或上游错误正文。`systemctl stop xiaoban-editorial.timer` 停止后续调度，停止 service 中断当前轮次，队列和稿件保留。定时器每次完成后间隔约 15 分钟再次运行，不会与未完成的轮次重叠。

前沿自动发布走 `/api/discover-editor/drafts/frontier/{key}/publish`，需要单独的 `publish_kinds=["frontier"]` 授权。现有采集令牌默认仍仅写草稿；不能审核其他栏目或访问管理员 API。服务端在同一事务中检查精确 revision、线上基线、1200 字正文、真实来源字段、实际发布时间及本篇封面归属，审计明确记录 `automation:` 操作身份。论坛后台的“采集令牌”列表展示该授权并支持撤销。

初始补稿阶段，每栏每日最多20篇；本执行器核验发布达到20篇后回到每日2篇。全站调用次数与费用停止阈值仍有效，不因补稿自动增发额度。官方 RSS 记录原始发布时间，过滤过旧或未来条目；前沿筛选近30天且有明确日期的来源，站内实际发布时间与来源时间分别保存。
