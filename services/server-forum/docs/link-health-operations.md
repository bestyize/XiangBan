# 外链健康检查与栏目运营

本模块只写独立检查记录与运行计划，不修改、删除、审核或发布发现页内容。访问受限、限流和连接失败分别记录，不能把一次自动访问失败直接解释为内容失效。

## 接入

根入口由集成分支统一修改：

1. `main.rs` 声明 `mod link_health;`。
2. 初始化数据库后调用 `link_health::schema::init(&pool).await?`；需要已有 `discover_items`、`discover_drafts` 和 `collectors::schema`。迁移可重复执行。
3. 管理员路由合并 `link_health::admin_routes()`，父路径 `/api/admin`。
4. 后台任务启动处调用 `link_health::spawn(state.clone())`。只在数据库显式启用后执行，默认关闭；不使用 GitHub Actions。内部每五分钟检查到期计划。
5. 管理后台注册 `CommunityOperationsPanel.vue`，建议路径 `/admin/community-operations`，菜单“栏目运营”；面板自身提供“运营概览 / 外链检查”。
6. 操作日志可选映射：`link_health.run` → “启动外链检查”、`link_health.settings` → “修改外链检查计划”，target `link_health` → “外链检查”。

## 管理 API

全部复用独立管理员认证。令牌失效时前端沿用现有 401 清理机制。

- `GET /community-operations`：`{as_of,columns,sources,links:{total,statuses},freshness_rule,candidate_scope}`。
  - columns：`kind,published,future,unlisted,pending,scheduled,sample,last_live_update`。
  - sources：`id,title,enabled,interval_minutes,last_success_at,last_error,next_run_at,active,failed_24h,succeeded_24h,pending,freshness`。
  - 已发布只计 `status='published' AND published_at<=now()`；future 单列未来日期，unlisted 单列非 published；pending 与 scheduled 来自编辑副本。
  - 已发布条目的未审编辑副本仍计待审；来源归属只从 `collection_owned_items` 与实际草稿关联统计。样例仅计显式 `data.sample=true` 的线上条目，不推测未标记数据。
  - freshness：`disabled / never / stale / fresh`；启用来源最后成功时间超过自身周期两倍则 stale。最近错误与新鲜度分别显示；没有用户数、访问量等合成指标。
- `GET /link-health?page=1&status=`：`{items,total,page,page_size:30,settings,runs,max_links_per_run:60,max_run_seconds:120,candidate_scope}`。
  - items：`id,url,host,refs,status,http_status,final_url,error,duration_ms,last_checked_at,last_success_at,failures`。
  - refs：`{kind,key,field,scope}`；scope 为 `published` 或 `editor_source`。
  - settings：`{enabled,interval_hours,next_run_at,active_run,lease_until}`。
  - runs 最近十次：`{id,status,started_at,finished_at,total_candidates,selected_count,checked_count,healthy_count,attention_count,blocked_count,error}`。
- `POST /link-health/run`，请求必须为 `{}`，未知字段（包括 URL、域名、凭据）拒绝。返回 `202 {run_id,total_candidates,selected_count}`；有效租约内重复请求 409。
- `PATCH /link-health/settings {enabled,interval_hours}`，周期为 24–720 小时；响应 `{ok:true}`。默认 `enabled=false`。修改计划不自动发布或立即采集。

运行状态 `succeeded` 只表示本轮检查结束，不表示所有链接可访问。`partial` 保留未检查链接的旧记录。前端仅在管理员启动后短暂刷新十次，之后及重新进入页面均需手动刷新，不会持续轮询或重复启动。

## 来源范围与限额

候选仅从数据库维护字段提取：

- 按 kind/key 排序的前 1000 条已发布且到期的 nav.url、doc.url、tool.homepage_url，以及这些条目的 data.source_url。
- 最近维护的 1000 条 nav/doc/tool/news/model_board/model_usage/repo_issue 编辑副本的独立 source_url；该字段包括待审来源，UI 明确显示“编辑来源”。
- 同一规范化 URL 去重，最多展示 50 处维护位置。API/UI 明确展示窗口范围，不把窗口统计称为全库完整链接审计。未在窗口内的历史链接不会由本轮主动访问；历史检查记录仍保留。
- 每轮最多 60 条，优先未检查、其次最早检查；并发 4，任务最多 120 秒，单链接最多 10 秒。DNS 最多 3 秒，每次请求连接最多 2 秒、请求最多 4 秒。到限不补造成功结果。
- 租约有效期五分钟，数据库行锁防多进程重复运行；新任务回收过期租约时把旧运行记为中断；旧任务不得覆盖新租约结果。

## 网络边界

固定精确域名白名单在 `link_health/network.rs::HOSTS`，由受维护的官方 AI 服务、文档和已有来源组成；不是任意 URL 代理，也不接受 API 扩展域名或子域通配符。未列入白名单的现有链接仍显示“安全策略阻止”，供编辑人工核对。变更白名单需代码审查。

仅 HTTPS 默认 443 端口，无用户名/密码；URL 最长 2048 字符，禁止控制字符。查询参数仅允许简单的 hl/lang/locale/tab/view/utm_source/utm_medium/utm_campaign 展示参数，其余阻止。显示被拒链接时移除凭据、查询串和片段；无效或非 HTTP(S) 链接显示占位符，不回显任意原文。检查 API 不回传原始 URL。

每次重定向重新验证 URL 和 DNS；拒绝任一非公网地址、空结果或超过 32 个地址，IPv4/IPv6 内网、环回、链路本地、文档地址、特殊映射与保留范围均保守拒绝。DNS 地址通过 reqwest `resolve_to_addrs` 固定到独立新建客户端，校验实际 peer 地址与固定集合一致，避免二次解析和 DNS rebinding。

自动跳转关闭，最多两次显式跳转；不继承共享 HTTP client 的 Cookie、Authorization、Referer 或代理配置，禁压缩、禁连接池复用。先 HEAD，只有 405/501 回退一次带 `Range: bytes=0-0` 的 GET。只读取响应头，不消费响应体，不抓全文，服务器忽略 Range 也不会继续读取正文。HTTP/1 解析器有默认有界头缓冲，超大响应头作为连接异常；测试覆盖 1 MiB 响应头拒绝。

状态：`unchecked,healthy,restricted,broken,rate_limited,unavailable,blocked,dns_error,network_error,timeout,attention`。HTTP 401/403 为受限；404/410 为不存在；429 为限流且本轮不重试。失败保留最近成功时间，不删除线上内容。

## 验证

临时声明模块后可独立执行：

```sh
TEST_DATABASE_URL='postgres://xiaban@%2Ftmp%2Fxb-pgsock:55439/postgres' \
CARGO_TARGET_DIR=/Users/read/StudioProjects/XiangBan/services/server-forum/target \
cargo test --manifest-path services/server-forum/Cargo.toml link_health
npm run build --prefix services/server-forum/web
```

测试在随机独立 schema 中运行真实论坛迁移、采集迁移和本模块迁移，结束时删除 schema，断言失败也清理。不操作 public；网络测试使用模拟传输或 localhost 专用私有原始传输，不对外发出探测。
