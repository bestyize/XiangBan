# 社区真实内容补齐（2026-11-01 发布）

本目录保存可审查的原稿、作品源文件、来源记录与发布清单。目录名沿用任务开始日期；正式发布时间以生产数据库和发布报告为准。

## 内容与来源

- 11 种作品各 5 个，共 55 个原创离线 ZIP，附真实渲染封面或明确标注的模板插画。35 个能力类作品是手动模板，包含步骤、提示词、工作页和示例，不承诺自动运行。
- 5 套方案固定引用真实作品的版本；8 个首页精选引用已发布作品。
- Claude、Codex、腾讯 SkillHub 资料各 5 条；AI 工具共 6 个；海外、国产导航各 5 个；项目导读和历史周回顾各 5 条。
- 5 条发布记录保留 2025 年原事件日期并明确“历史回顾”。5 项优惠与试用为官方当前展示的长期权益，保留资格条件和来源，不编造统一开始或结束时间。
- 开源榜从数据库内 8 个真实首日快照展示总星数。缺失的涨星量为未知；不把 GitHub 事件流中的新增 Star 次数当作净增。历史调查过程见 `research/repo-history-investigation.json`。
- 模型榜继续读取已采集快照，维持每天更新一次的采集配置；内容浏览不向原厂实时请求榜单。

## 文件

- `posts/`：论坛 Markdown 原稿，按稳定 external id 去重。每个批次由负责的作者流程单独导入、发布。
- `packs/`：55 个作品源目录；`PACKS_README.md` 与 `scripts/README-resources.md` 说明包格式、构建和浏览器验证。
- `pack-catalog.json`：版本、对象键、字节数、SHA-256、封面与使用说明。
- `curated-discover.json`、`projects.json`、`weekly.json`、`tracker.json`：经核实的公开内容及管理接口请求体。
- `packs-register.json`、`schemes-register.json`：本批次发布请求。**POST 不是通用重放脚本**；重试先核对正式 ID 与已有版本，不能重复建方案。
- `featured.json`：带预期 revision 的精选清单，不能盲目覆盖后续运营修改。
- `research/`：公开来源、原事件日期、采集时间和历史口径说明。
- `reports/`：导入、发布、公开数据与下载验收记录。报告不含凭据或短期签名下载 URL。

## 公开验收

2026-11-01 生产公开接口验收 **70/70 项通过**，所有受检分类、标签、分组和榜单指标均至少 5 条。完整计数见 `reports/public-audit.json`。

| 范围 | 已上线结果 |
| --- | --- |
| 论坛 | 新增 125 篇；27 个叶分类全部至少 5 篇，保留原有内容 |
| 学习标签 | Claude Code 5、Codex 5、提示词 9、工作流 11、本地部署 10、AI 硬件 6 |
| 创作 | 11 类各 5 个作品，共 55 个；方案 5 套 |
| 资料与导航 | 3 组资料、2 组导航，每组 5 条 |
| 工具、项目、周刊 | AI 工具 6 个，项目 5 个，历史周回顾 5 期 |
| 追踪 | 历史发布记录 5 条，附资格条件的优惠与试用 5 项 |
| 榜单 | 开源快照 8 个；模型 10 项指标均超过 5 条，调用量榜 50 个 |

105 篇内容帖通过逐帖公开 API 比对，另 20 篇站务帖导入和发布全部成功；两部分原稿及报告分别留存。55 个资源包均完成实际公开下载及哈希验证，小米 14 已安装支持新版接口的 Android 包。构建、部署与验证范围见 `reports/release-validation.json`。

```sh
python3 services/server-forum/content/community-launch-20261031/scripts/audit.py \
  --output /tmp/community-public-audit.json
python3 services/server-forum/content/community-launch-20261031/scripts/verify-downloads.py \
  --catalog services/server-forum/content/community-launch-20261031/pack-catalog.json \
  --output /tmp/community-download-audit.json
```

计数按真实客户端筛选：27 个论坛叶分类、11 个作品类型、6 个学习标签、资料与导航分组，以及方案、工具、项目、周刊、榜单各指标。个人收藏、通知、账号额度不生成虚构数据。

首日榜单与无统一截止日期的权益使用 `home/v2`、`repos/v2`、`repos/issues/v2`、`tracker/v2`。旧接口保留数值字段约定，过滤无法准确表示的新条目，不用补零冒充测量结果。

资源服务部署见 `infra/resource/README.md`。上线前已备份数据库及配置，二进制和网页均在本地构建后上传；55 个作品通过公开入口逐包检查下载 SHA、封面 SHA/MIME、Range 和未签名请求拒绝。
