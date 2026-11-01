# 资讯订阅（开发中，未部署）

同一账号保存任意主题的资讯订阅与私有历史期刊，入口 `/me/subscriptions`。桌面社区/资料库复用独立社区窗口；手机「我的」使用现有浏览器入口（网页独立登录同一账号，不注入客户端令牌）。并非原生移动编辑器。自然语言关注描述可识别简单中文每日/每周时间，始终以可编辑表单确认后保存；不支持的时间语义要求手动选择，不静默猜测。

服务先托管在 server-forum，复用账号中心 JWT 和数据库迁移；队列/执行/投递/额度边界分离，后续可迁独立 worker。agent-proactivity 是本机注意力策略，桌面提醒依赖客户端运行，不适合托管云端周期任务；不改变其权限或本地提醒。云端期刊为独立账号资产，桌面资料库提供入口，不上传本地资料和记忆。

## 安全与调度

- 用户 API 只接受 AuthUser，所有读写按 owner 隔离，修订号防覆盖。邮箱修改撤销验证；确认码仅存摘要，30分钟有效，最多5次尝试。每账号20条有效订阅、每日30次创建/5封确认邮件，每订阅5分钟间隔。默认关闭发信与 worker。
- 每日/每周按 IANA 时区计算；DST 不存在的时间跳过、重复时间只取首次。宕机不补发大量往期，每条最多排一期到期任务。一次性任务接受含偏移的 RFC3339 时间。
- PostgreSQL SKIP LOCKED、唯一 occurrence、带随机租约的生成任务；生成最多3次，重试间隔5分钟。编辑/暂停/删除取消未发送版本。邮件提交与暂停共用订阅锁，暂停成功返回后旧任务不会开始新发送。
- SMTP 没有幂等接口。先持久化 sending，再提交 SMTP；超时或崩溃进入 uncertain，不自动重发，不宣称 exactly-once。需人工与供应商核对后处理账务；尚无管理员核对界面。SMTP 接受不代表收件箱投递成功，未实现退信 webhook。
- HTML 转义所有内容，引用网址来自检索结果，不接受模型生成链接；历史以纯文本展示，避免执行邮件 HTML。没有可靠来源时不发送。正文按来源摘要，不保证模型事实判断完全正确。
- 客户端不提交密钥。所有搜索、模型和 SMTP 凭据为平台凭据，不存入订阅、URL 或历史。

## 配置（不含真实凭据，需独立部署配置）

| 变量 | 用途 |
|---|---|
| INFORMATION_WORKER_ENABLED | 仅 `true` 开启独立后台循环，默认关闭 |
| INFORMATION_MAIL_ENABLED | 仅 `true` 开启邮件，默认关闭 |
| INFORMATION_SMTP_HOST / PORT / USER / PASSWORD | 平台 SMTP，587 STARTTLS / 465 TLS |
| INFORMATION_MAIL_FROM | 已验证的发件人邮箱，遵守供应商发信域认证 |
| INFORMATION_SEARCH_URL / SEARCH_KEY | 独立检索端点及平台凭据；兼容现有 server-ai 规范 |
| INFORMATION_MODEL_URL / MODEL_KEY / MODEL | 平台模型端点、凭据、管理员选择的模型；无默认模型 |
| INFORMATION_ACCOUNTING_URL | server-ai `/api/internal/information-jobs` 完整 URL |
| INFORMATION_SERVICE_TOKEN | 与 AI_INFORMATION_SERVICE_TOKEN 相同的专用服务凭据 |
| AI_INFORMATION_SERVICE_TOKEN | server-ai 专用服务凭据，至少32字符；不使用账号令牌 |
| AI_INFORMATION_POLICY | server-ai JSON 策略，未设置时拒绝预约；不提供生产价格 |

Policy 字段：`product_id`（现有统一产品）、`reserve_units`（单任务上限）、`model`、`input_per_million`、`output_per_million`、`search_units`、`email_units`。以上均是现有额度单位，不是人民币价格。模型/搜索分别计量；原始 usage 保留供核对。月卡与资源包沿用现有 quota_buckets，不建立新钱包、不实现新支付。预约按现有最早到期顺序冻结桶额度、写共同流水，成功后按快照策略结算、释放差额，终止失败释放；同一期重复调用幂等。过期或被撤销的原桶不会因退款复活。超出预约上限或缺少计量保持待核对，不偷偷追加扣款。现有 New API 对账有延迟，仍需上线前验证跨交互调用的限额边界。

**未确定的商业规则**：价格、额度换算、月费包含量、缓存 token 规则、扣费时点和退信/不确定投递处理。代码的“SMTP 接受后结算”是待审阅实现策略，不代表已确认商业条款。不得在这些规则确认前开启生产计费。未配置服务时保存订阅并展示不可投递状态，不伪装已经运行。

DeepSeek 检索端点核对（2026-11-01）：https://api-docs.deepseek.com/guides/search/ ，Search 为 `/v1/tools/search`，Fetch 为 `/v1/tools/fetch`，并非从聊天端点假设出的工具。自然语言关注内容不局限 AI。

## 验证

- `cargo test --locked --manifest-path services/server-forum/Cargo.toml subscriptions`
- `TEST_DATABASE_URL=隔离测试库 ...`：验证真实 PostgreSQL 迁移、账号隔离、确认、并发 tick/claim、更新取消、删除保留历史。
- `cargo test --locked --manifest-path services/server-ai/Cargo.toml information_jobs`
- `npm --prefix services/server-forum/web run build`

不发真实测试邮件、不运行生产迁移、不部署。发布前还需原生设备入口与登录、真实供应商沙箱投递、退信/投诉与退订运营、邮件域名信誉、队列/数据库备份监控验收。
