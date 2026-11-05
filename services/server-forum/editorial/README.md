# 社区采编执行器

开源精选采用独立的 [README 采编规范](OPEN_SOURCE_README_WORKFLOW.md)，默认中文并支持英文原文；其批量稿件、来源和发布边界不沿用下述普通图文文章流程。

独立 Python 3.10+ sidecar，图片编解码依赖 Pillow（需支持 WebP）。Ubuntu 安装 `sudo apt-get install --no-install-recommends python3-pil`；本地虚拟环境安装 `python -m pip install Pillow`。运行时业务属于 server-forum，不依赖 Codex、Claude 或本机开机。通过论坛已有的受限作者令牌和 server-ai 产品调用，不能访问论坛管理接口。不改变 GitHub Actions 的手动触发方式。

新封面先读取网关 `/api/ai/tiers` 公布的输出格式，支持 WebP 时请求 `output_format=webp`，否则请求 PNG。网关返回 PNG 或 JPEG 时，执行器在上传前使用 Pillow 转成 WebP（quality=82、method=4），保留原尺寸和透明通道；原生 WebP 在上传限制内直接保留字节，避免重复有损编码。转换不触发额外生图调用。生成响应可在7 MiB内，转换后的实际上传文件仍不超过5 MiB；拒绝异常尺寸、损坏或动画图片，并完整解码校验。普通帖子使用 `images/cover.webp`，日报和前沿资产接口按实际字节识别 MIME。常规采编不修改已经导入的历史配图；恢复与验收兼容 PNG 和 WebP，不会为了换格式重新发表文章。

已有线上图片的显式迁移使用 `migrate_images_webp.py`，只在运营者授权后以 root 执行。依次运行 `prepare --backup /var/lib/xiaoban-editorial/webp-migration-日期`、`apply --backup 同目录`、`verify --backup 同目录`。准备阶段读取公开图片记录与现有文件，保留原文件和原元数据，将静态 PNG/JPEG 转成 WebP；原生 WebP、GIF、未导入的待发布图及未引用的发现页资产不修改。应用阶段停止定时领取、等待共享锁，短暂停论坛后原子替换文件并在单个数据库事务内更新 MIME/大小或资产字节；每行有并发冲突校验，失败时恢复原图。启动论坛并等待分类接口恢复后重启原定时器。全部文章、草稿及发现页对象的字段摘要前后必须一致，权限、状态、URL和正文均不修改。迁移记录和备份为私有文件，不写凭据。

采编的历史 `cover.png`、原始导入 JSON 和审校记录保留；迁移另存 `cover.webp` 及 `cover-webp.json`，后者同时绑定原图与新图 SHA256。验收按该记录读取实际线上配图，并拒绝摘要不符；原来的正文配图路径仍按历史导入记录核对。`verify` 逐图 GET 检查实际响应 MIME、字节数与 SHA256，并核对采编的本地转换记录，不运行文章重发或模型调用。

`agents.json` 启用 41 个文章目标。日报、前沿使用发现页文章通道，其余栏目使用对应分类的图文通道。创作、方案、工具、资料页面同时保留真实作品包或目录；导读文章不计为可安装作品、目录条目、实际悬赏或处罚公示。排行榜不计数。求助与举报栏目可发布明确标识的站方指南，不能虚构请求、经历或处理事件。

流程：来源白名单 → 官方页面资料 → 写作 → 单独事实审校 → 必要时一次修订和复审 → 生成封面 → 待发布稿 → 幂等发布 → 公开详情和封面回读。文章标明公开资料整理、未实测及概念插画。审校绑定稿件哈希；修改后需重新审校。模型审校有局限，需持续抽查并处理更正，不应将其称为人工终审。

状态保存在 `/var/lib/xiaoban-editorial/`：SQLite 队列、调用预留记录、阶段事件、来源证据、稿件、核查意见、封面及公开链接。响应丢失后先按原 `external_id` 查稿件再恢复，不重新发布。单机文件锁避免重叠；这个实现不支持多个服务器共用队列。失败最多尝试三次、间隔一小时；审校拒绝保留给编辑处理，不为了数量强行发表。修改拒绝稿后，应移除旧 `review.json` 并由运营人员显式重新入队。

费用通过独立运营产品余额、每日调用数、单次处理数和每栏目每日篇数约束。按北京时间记账，初始补稿配置每日最多 6000 次模型请求、每日与本次补稿各最多消耗 4500 万额度点（当前换算约 90 美元），保留 250 万点（约 5 美元）；只使用已有运营余额。网关对账有延迟，金额是停止阈值，并非上游硬性账单封顶；单次调用还受网关已有额度检查限制。不要另行自动补发额度。

部署文件位于 `deploy/`，使用专用无登录用户 `xiaoban-editorial`，代码放 `/opt/xiaoban-editorial`。上传本地校验后的 Python 源码即可，无云端编译。私密状态目录必须有 `editor-token`（绑定作者且允许发布）及 `ai-session.json`（运营客户端的可刷新账号会话），以及 `discover-token`（仅授予 frontier、news 草稿及自动发布权限）；均为 600、目录为 700，只由服务用户持有。会话由该进程独占刷新，不在其他主机复制使用；撤销会话或创作令牌可立即停止后续调用。不得上传账号密码。

```sh
python3 -m unittest discover -s services/server-forum/editorial -p 'test_*.py'
python3 services/server-forum/editorial/worker.py --root /private/state audit
python3 services/server-forum/editorial/worker.py --root /private/state run --limit 1
python3 services/server-forum/editorial/worker.py --root /private/state fill --parallel 8 --target 20
python3 services/server-forum/editorial/worker.py --root /private/state status
```

`audit` 按真实公开接口生成 `coverage.json`，公开总数与本执行器核验过的发布数分列；已有 20 条不代表质量验收通过。`status.json` 是最近一轮状态，命令 `status` 即时查询队列。日志不写凭据、模型原文或上游错误正文。`systemctl stop xiaoban-editorial.timer` 停止后续调度，停止 service 中断当前轮次，队列和稿件保留。定时器每次完成后间隔约 15 分钟再次运行，不会与未完成的轮次重叠。

前沿与日报自动发布走 `/api/discover-editor/drafts/{frontier|news}/{key}/publish`，需要单独的 `publish_kinds=["frontier","news"]` 授权。现有采集令牌默认仍仅写草稿；不能审核其他栏目或访问管理员 API。服务端在同一事务中检查精确 revision、线上基线、1200 字正文、真实来源字段、实际发布时间及本篇封面归属，审计明确记录 `automation:` 操作身份。论坛后台的“采集令牌”列表展示该授权并支持撤销。

初始补稿阶段，每栏每日最多20篇；本执行器核验发布达到20篇后回到每日2篇。全站调用次数与费用停止阈值仍有效，不因补稿自动增发额度。官方 RSS 记录原始发布时间，过滤过旧或未来条目；前沿筛选近90天且有明确日期的来源，站内实际发布时间与来源时间分别保存。

`fill` 是初始补稿任务，最多八个并发、同栏最多一个在途稿件，达到每栏20篇即停止。独立 `xiaoban-editorial-fill.service` 持有同一文件锁，常规定时器不会与之重叠；`campaign.json.complete` 只有全部41栏的核验发表数均达20时才为真。额度停止或暂无可用选题时保留未完成状态。

`channels.json` 是分类的共同清单。服务端一次性迁移旧行并保留 ID、外键和已有权限；旧 slug 查询仍兼容，旧分类页面308到新地址。`handbook.json` 由 `build_handbook.py` 从真实仓库产品说明和规则生成，540个问题各有明确来源及内容边界；公开 `/community/sources/{id}` 供读者核查。服务器可通过 `FORUM_EDITORIAL_HANDBOOK` 指定外部同格式公开资料文件，更新后重启论坛。不能把选题清单当成已经发表或已经验收的文章。

官方页面、RSS 和 GitHub 文档索引均只从白名单采集。索引翻页只跟随实际页面链接，Notebook 仅提取文字和代码源，不采信执行输出。正文存在跨标题重复检测，独立审校同时参考本栏最近20篇的标题与摘要。生成封面明确注明概念插画；每篇发布后检查公开详情和站内图片响应。

Nginx 的 `/api/discover-editor/articles/` 与旧 `/api/discover-editor/frontier/` 上传入口均需 `client_max_body_size 7m`，普通编辑稿导入需31m；不要把全站请求上限一并放大。

初始补稿收到 SIGTERM/SIGINT 时先停止领取，等待在途稿件结束（服务停止超时300秒），避免打断已付费的生成请求。时间字段只允许等价的 RFC3339 表示差异，标题、正文和其他待发布字段仍精确比较。

经运营者检查确需修订的退稿，可显式运行 `python3 repair.py --agent create-wallpaper --limit 4`。每篇只增加一次修订机会，仅处理从未导入的事实审校退稿，保留旧稿、旧证据和核查意见。该命令不会批准或发布：新版本清除旧审校记录，仍须进入正常执行器的独立审校；再次不通过便保留退稿，不循环尝试凑数。每天调用数和运营余额阈值同样有效。`--limit` 约束实际尝试次数，失败也占一次。

壁纸补充Krita官方图像制作手册，组件补充GNOME官方人机界面指南，梦想补充GitLab公开学习发展手册。选题均先验证服务器可读取的实际正文；外部工具的操作和组织规则不能套成小伴能力或普适承诺。这些资料是相应话题的导读来源，不是已经导入的小伴作品。

初始补稿在229篇时累计1517次调用，按实际审校与修订比例将每日请求次数阈值校准为6000次；每日及整轮4500万点的费用阈值、250万点保留额度继续约束所有调用。小游戏补充MDN官方游戏开发资料，标准Web能力不等于小伴已支持的沙箱能力。

技能补充Anthropic官方公开技能示例，按具体SKILL文档说明结构与依赖；公开可读不等于统一开源或可商用，也不代表已经安装在小伴或能够直接导入。

目标按实际浏览到的Atlassian Team Playbook规划条目补充，任务接入GitHub Projects官方文档，提醒接入Google Calendar/Tasks官方帮助，例行接入Apple快捷指令用户指南。外部产品和小伴的能力始终分开。Apple索引只选当前iOS路径，Google帮助去除ref_topic导航上下文后再计算来源身份，避免把版本或导航参数当新选题。
