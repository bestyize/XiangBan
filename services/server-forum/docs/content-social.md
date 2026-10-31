# 内容包与方案收藏、举报接口

所有 `/api/content-social`、`/api/me/content-bookmarks` 请求使用普通用户 Bearer；封禁用户拒绝访问。`kind` 只能为 `pack` 或 `scheme`，编号为正整数。

- `GET /api/content-social/{kind}/{id}` → `{bookmarked:boolean}`。仅公开可读内容，未公开或不可读返回 404。
- `PUT /api/content-social/{kind}/{id}/bookmark` → `{bookmarked:true}`，幂等。
- `DELETE /api/content-social/{kind}/{id}/bookmark` → `{bookmarked:false}`，幂等；失效内容仍可取消，只操作当前用户记录。
- `GET /api/me/content-bookmarks?kind=pack|scheme&page=1` → `{items:[{kind,id,created_at,content}],total,page,page_size:20}`。`kind` 可省略。`content` 为既有 `PackItem` 或安全解析引用后的方案详情 DTO；不含内容包 manifest、文件地址和版本列表。下架、作者封禁、关联讨论非公开的内容动态隐藏；方案中的失效引用仅保留 `available:false` 占位。
- `POST /api/content-social/{kind}/{id}/report`，正文 `{reason,detail?:string}` → `{ok:true}`。`reason` 沿用论坛的 `spam/abuse/illegal/copyright/account_trade/politics/other`，`detail` 最多 500 字。同用户同目标重复待审举报更新原记录与 revision；每日新举报含原论坛举报合计最多 30 次。

普通用户没有举报详情读取接口。不会返回举报者、举报理由或管理员内部处理说明给内容作者。作者仅接收固定通用通知 `kind:content_removed`，`pack_id` 可选；客户端无 pack_id 时跳转我的方案。

管理员使用独立管理员 Bearer：

- `GET /api/admin/content-social/reports?status=open|resolved|dismissed|all&page=1` → `{items:[{id,kind,target_id,reason,detail,status,revision,created_at,handled_at,handled_by,result,reporter:{uid,nickname},target:{title,status,exists}}],total,page,page_size:20}`。
- `POST /api/admin/content-social/reports/{id}/handle`，正文 `{expected_revision,action:'dismiss'|'remove',note:string}` → `{ok:true,target_removed:boolean,reports_handled:number}`。内部说明必填 1–500 字。所读 revision 已过期或举报已处理返回 409，重新读取后再操作。驳回仅处理本条；下架原子修改内容状态并处理同目标所有待审举报。两者写入既有 admin_audit。

接入：声明 `mod content_social` 与 `handlers::content_social`；迁移调用 `content_social::schema::init(&pool)`；API 合并 `handlers::content_social::routes()`，管理员嵌套 `/content-social` 下的 `admin_routes()`。管理菜单加入 `ContentReportsPanel.vue`。不增加新点赞或积分奖励规则。

针对性验证：设置隔离 `TEST_DATABASE_URL` 后执行 `cargo test --locked content_social`；每测试创建独立 schema，验证幂等、跨用户隔离、下架/封禁/非公开隐藏、管理员鉴权、revision 冲突、批量下架联动、审计和通知隐私。
