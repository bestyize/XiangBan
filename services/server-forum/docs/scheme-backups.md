# 私有方案备份

`scheme_backups` 与公开 `schemes` 独立。所有接口要求本人论坛账号；查询带 owner_uid，其他账号详情返回 404。创建/更新只接受 schema_version=1、title（1–60字）、summary（最多500字）、1–20个唯一公开 pack_id 与确实存在的固定 version、expected_revision。拒绝额外字段，不能存入任意桌面设置、文件或密钥。

- GET `/api/me/scheme-backups?page=&page_size=&status=active|removed`，分页最多50条。
- GET `/api/me/scheme-backups/{uuid}`：本人详情，引用依照当前公开状态解析。下架、封禁、受限引用仅显示不可用占位。
- PUT 同一路径：expected_revision=0创建，其后必须与当前版本相同，恢复回收站也会递增版本；冲突409，禁止静默覆盖。
- POST `/{uuid}/remove {expected_revision}`：进入回收站。每人最多100份活动方案、200份总记录，用户行锁串行配额。

网页入口“我的作品 → 我的方案 → 账号方案备份”，本次访问默认关闭。开启只读备份，保存所选方案才写入。发布公开方案有单独确认，不会自动发布备份。手机提供默认关闭的本人备份/回收站只读视图。桌面独立“我的桌面方案”窗口先保存在本地，按账号明确开启云端后才可备份所选作品引用。

桌面本地 `config/community-schemes.json` 保存受限布局：壁纸编号、特效、伙伴主题和现存组件的比例位置。云端只上传用户明确选择的公开作品编号/固定版本、方案名称和固定说明，不上传本地布局或资源文件。恢复云端引用不会下载或执行资源；本地还原需确认，缺失资源跳过，组件正文保留。它目前不提供跨设备完整桌面资源迁移。冲突保留本地版本，由用户查看云端后处理。

论坛 `FORUM_ACCOUNT_EXTRA_CLIENT_IDS` 若显式覆盖默认值，需要同时包含 `xiaoban-android,xiaoban-desktop-wallpaper`，供手机和桌面账号令牌调用；默认值已包含两者。生产仅接受账户服务已签名且 issuer 合法的令牌。

验证：显式 `TEST_DATABASE_URL` 后运行 `cargo test --locked --manifest-path services/server-forum/Cargo.toml scheme_backup`；覆盖本人隔离、未知字段、失效引用、不存在版本、极端分页、并发更新与删除。桌面针对性检查见桌面 README。没有把本地模拟界面当成打包原生应用验收。
