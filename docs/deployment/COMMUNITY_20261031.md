# 社区生产部署与真实来源接入（2026-10-31）

本记录取代社区验收记录中“尚未生产部署、缺真实密钥”的历史状态；真机、资源机与七日观察限制仍适用。

## 部署

- 生产：`https://chat.xiangban.xyz`，香港现有 systemd `server-forum`，目录 `/home/ubuntu/xiangban-forum`，监听 `127.0.0.1:55010`。没有迁移到 Docker，也没有更新账号中心、AI、桌面或手机客户端。
- 二进制提交：`ca7fd4421cb2e433bc6b89550d8b5850e4a3d578`。其中 `e3af0da8` 修复真实 Codex Release 暴露的辅助程序误识别：带平台前缀的 sandbox/helper 不得挤占主程序清单。11项针对性测试通过，1项真实网络 smoke 未重复运行。
- macOS 本地通过 cargo-zigbuild 交叉编译 `x86_64-unknown-linux-gnu` release，前端沿用已验证的社区构建产物。没有在服务器编译，也没有触发 GitHub Actions。
- 生产二进制 SHA-256：`1102ba0b2e47f05b0143ee82abec974f876ea83cf9856d0bbb94998bcb42522c`。上传后逐文件核对首批71项产物，修复版二进制再次核对摘要。生产 `DEPLOYED_COMMIT` 保存提交号。
- 数据库自动迁移成功，保留394个原帖子与2条原回复。内网/公网健康检查正常，systemd active，检查期间 error 级服务日志为空。
- 生成并保存独立采集加密主密钥，显式接受 Android 和桌面的现有账号客户端 ID。密钥未写入仓库、部署记录或前端。

## 真实来源

| 来源 | 已发布或已采集 | 计划 |
| --- | --- | --- |
| Artificial Analysis | 真实719个模型；来源、指数版本、价格、性能和缺失值按官方数据保留 | 每日北京时间12:00采集到待审 |
| OpenRouter | 2026-10-28、29、30三个完整UTC日；每日Top50及other，精确字符串整数；公开最新日合计30,374,878,828,444 Token | 每日北京时间12:00采最近三个完整UTC日到待审 |
| 官方日报 | 发布10条，来自OpenAI与Google DeepMind；三个固定RSS来源均已选择，保留来源发表时间，不抓正文 | 每6小时；北京时间每日最多10个新候选 |
| GitHub仓库 | 8个真实仓库、8条首日观测；首日不生成净增榜单 | 每日北京时间12:00；至少跨两个观测日才有部分榜，连续8个日边界才有完整七日净增 |
| GitHub工具 | `openai/codex` 正式版本 `rust-v0.166.0`，6个主程序包覆盖macOS/Windows/Linux与aarch64/x86_64，摘要来自GitHub官方digest | 每6小时采集到待审；没有下载或运行这些二进制 |

仓库名单：openai/codex、ollama/ollama、langgenius/dify、open-webui/open-webui、comfyanonymous/ComfyUI、microsoft/autogen、cline/cline、lobehub/lobe-chat。GitHub来源凭据经过官方只读验证并加密保存；AA/OpenRouter由用户在生产后台配置后成功采集，没有在聊天传递密钥。

全部五个来源 enabled，验收时 last_error 为空。本次首批内容经核对后走正式整批终审API发布；后续定时采集只生成待审内容，不自动终审。旧Codex批次保留运行历史，因新版本草稿修订号变化而无法按旧版本发布。

36条显式 `sample=true` 的内容通过管理员API转为草稿，保留数据和审计：横幅1、日报5、热榜1、模型12、厂商6、额度1、上新3、限免2、工具5。未删除；原始记录另有服务器备份。验收时公开示例数为0。没有来源密钥的占位榜、未积累的周增长和未验证的限免内容不继续公开。

## 验证

- 生产公开 `/api/discover/models/v2` 返回719项且sample=false。
- `/api/discover/model-usage` 返回50项；用整数重新核对Top50之和加other等于total；前一日可用、历史日期3项。
- `/api/discover/daily` 总计10项、sample全部false；`/api/tools/github-tool-965415649` 返回6个主程序，无sandbox/helper。
- 首日没有已发布热榜，`/api/discover/repos` 返回404，表示尚无已发布期数；这是积累观测中的实际状态，不能视为已有完整周榜。
- 浏览器核对生产 `/models` 的能力榜与调用量榜，日期、精确Token、排名变化与缺失指标均可见。截图本机 `/tmp/community-production-capability.png`、`/tmp/community-production-usage.png`。
- 后台入口 `/admin`、采集入口 `/admin/collection`，模型榜 `/models`。

## 备份与回滚

部署前服务器备份：`backups/community-20b8fe41-20261031T134247Z/`，含旧程序、前端、原环境与数据库custom dump。将数据库真实恢复到独立临时库 `forum_restore_20b8fe41`，核对394/2/57（帖子/回复/发现条目）后删除临时库；没有覆盖生产库。

接入真实来源后的备份：`backups/community-ca7fd442-live/`，含数据库、当前运行环境（含采集主密钥）与uploads归档。数据库与环境另存本机受限目录 `/Users/read/.codex/deployments/xiangban-forum/20261031-ca7fd442/`；备份包含敏感配置，不提交。原上传目录43MiB，本次部署不移动或替换上传文件。

回滚优先停止论坛，使用备份旧程序（`install -m 755`）和旧dist替换，再启动并验证健康；保留当前环境与采集主密钥。迁移为追加结构，普通程序回滚无需覆盖数据库。若需数据恢复，先保全故障后的新增数据，再按明确恢复点单独执行，禁止直接用旧dump覆盖用户新数据。

临时运维令牌有效期30分钟，服务器临时脚本/令牌在验收后删除。没有修改管理员密码或长期登录状态。

## 保留限制

按用户明确选择，本次不部署资源服务。上传、镜像与真实资源机闭环仍待后续部署；公开工具目前走官方来源。没有以本次服务器部署代替Android/iOS真机、桌面打包应用或完整跨设备方案迁移验收。

## 客观榜单每日自动发布（db4a87b8）

- 能力榜、调用量榜、开源热榜继续每天北京时间 12:00 采集（生产均为 `interval_minutes=1440`），公开读接口只查询数据库已发布快照，不触发上游请求。
- 本次改为校验与发布同一事务，三类客观榜单无需人工审核。失败保留上一版；人工修改和定时稿冲突时停止覆盖。资讯与工具版本保留人工审核。
- 本地隔离 PostgreSQL 的采集回归 4 项通过，包含自动上线、批次回滚保留旧值、重复采集幂等、定时稿保护及资讯不自动发布；管理台构建通过。
- 本地交叉编译后上传并重启 `server-forum`，服务 active；生产程序 SHA256 为 `575167febfdf1cea4bb3ffcef36d03142f2f45aa8b338cba153396b241d97705`。管理台同时更新发布方式说明。
- 生产执行一次能力榜与调用量榜采集验证：分别成功 1 / 3 项，`reviewed_by` 分别为 `collector:artificial_analysis` / `collector:openrouter`，无人工审核请求。下次采集均为 2026-11-01 12:00（北京时间）。
- 部署前备份在服务器 `backups/community-auto-publish-20261031/`，含旧程序、dist、数据库 custom dump；本次不涉及数据库结构变更。运维短时令牌只在内存使用，没有修改管理员密码或持久化额外令牌。
