# 四个站务分类的取材与核实

范围：site-notice/site-rules/site-feedback/site-reports，各5篇。稿件固定ID见reports/site-post-manifest.json；仅posts/batch-site为本组导入目录。所有帖子配原创流程图，不伪造截图、人物或处罚案例。

## 已核实依据

- 根README、server-forum README、server-resource README、桌面resource-pack-format.md/effects.md。
- web StaticView.vue及content/rules.ts、terms.ts、privacy.ts：正式规则与support@xiangban.xyz联系/申诉渠道；本文不新增处罚档位或时限。
- src/handlers/interact.rs、admin.rs、content_social模块：举报理由、补充材料、管理员处理与权限；普通用户无他人举报详情接口。
- src/handlers/posts.rs、reply_content模块、attachments.rs：最多3引用/5附件、作品版本固定、附件阅读与积分权限。
- collectors/model_sources.rs、collectors/mod.rs与公开model_rankings接口：完整UTC日用量、缺失字段、相邻日比较及定时采集失败保留既有快照。
- scheme_items与public_content_card：方案安全引用、下架/封禁/非公开动态隐藏。

## 公开核实

reports/site-public-evidence.json记录当次只读访问结果：55个作品、5方案、Claude/Codex/SkillHub各5资料、模型719、OpenRouter用量50（date=2026-10-30）。集成方另确认开源新协议为/api/discover/repos/v2，8项真实首日baseline；旧/repos返回404，不能当作“没有快照”，更不能推断已拥有七天历史。

site-context及四分类recent均已通过技能CLI实时读取，发布权限为true，四分类原均无公开帖。网页搜索工具无法读取本站时，按已配置CLI和公开API核实；不将工具失败当作站点不可用。

## 创作约束

公告只说明当前真实能力，不承诺每次定时任务都成功，不把Token用量当请求数；手动模板不宣传自动运行。规则帖是现有规则的使用解释，不自行创设规则。反馈帖提供可执行的信息收集方法；示例明确为写法而非实际缺陷。举报帖只解释机制、隐私、材料与邮箱申诉，不虚构已处罚案例、处理记录或新服务时限。
