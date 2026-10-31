# 回复作品卡与附件

扩展既有回复 API，普通用户授权、发言/每日次数、帖子阅读权限和采纳锁定规则保持原约束。

`POST /api/topics/{id}/posts`、`PATCH /api/posts/{id}` 在 `body` / `reply_to` 外支持：

```json
{
  "work_refs": [{"kind":"pack","id":123,"version":1},{"kind":"scheme","id":12}],
  "attachment_ids": ["UUID"],
  "attachment_prices": [{"id":"UUID","price":0}]
}
```

- `work_refs` 最多 3 项，种类只允许 pack/scheme，不能重复引用同一对象。作品省略 version 时在发表时固化真实存在的当前版本；方案不接受 version。只保存引用，不能提交标题、HTML、manifest 或任意 JSON。新引用必须公开可读；编辑可以原样保留后来失效的既有引用。
- 附件沿用 `POST /api/attachments` 上传，最多 5 个，只能使用本人未绑定或本回复已绑定的附件，不能抢用别人的、其他回复或主题正文附件。
- 价格使用既有 0–10 积分范围，已有购买流水继续生效。操作整个事务提交，任何非法附件、引用或定价整体回滚。
- 编辑省略任一可选字段时保留原值；`work_refs:[]` 清空作品引用，`attachment_ids:[]` 清空附件并在提交后删除文件。旧客户端只改正文不会丢失内容。

`GET /api/topics/{id}/posts` 的每个 PostItem 增加：

```json
{
  "work_refs":[{"kind":"pack","id":123,"version":1,"available":true,"content":{}}],
  "attachments":[]
}
```

`content` 是既有安全公开 `PackItem` 或方案详情，失效引用只保留 `kind/id/version/available:false`，没有旧标题、正文、链接或 manifest。被隐藏、删除、读者屏蔽或封禁作者的回复两数组均为空。附件列表和主题正文附件相互独立。附件下载与购买均检查回复状态、作者封禁和读者屏蔽，再沿用帖子阅读等级、购买凭证与原积分账本；不可通过已有文件直链绕过。

`GET /api/posts/{id}/source`（仅本人）增加 `work_refs:[{kind,id,version:null|number}]` 与 `attachments:Attachment[]`，便于编辑。`PATCH` 返回 `body_html/work_refs/attachments`，引用为安全解析后的卡片。

接入：根模块声明 `mod reply_content;`，数据库 migrate 在基础 posts/attachments 建表后调用 `reply_content::schema::init(&pool)`。不增加路由，窄改已有 posts/attachments handler。迁移给 attachments 增加可空 post_id，topic_id 保留以复用原阅读权限/付费，旧主题附件不变。管理与作者隐藏回复会立即隐藏相关附件。

验证：隔离 PG `cargo test --locked reply_content`，每测试独立 schema，覆盖跨用户附件、非法版本、事务回滚、既有编辑兼容、正文与回复附件互不删除/挪用、下架卡片隐去、附件隐藏/屏蔽/封禁/阅读等级以及购买入口拦截。Web 执行 `npm run build`。
