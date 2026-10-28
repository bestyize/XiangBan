# 手机互联上线清单（多服务）

手机端和手机互联会用到四个线上服务，不止新增的中转。本页是整体顺序与每一步的检查方法；单个服务的细节看各自文档。让 AI 代做时用 [手机互联整体部署提示词](../agent/手机互联整体部署提示词.md)。

手机端的地址都在 `CommunityConfig`（`common/community-core`）：论坛 `https://chat.xiangban.xyz`、账号 `https://account.xiangban.xyz`、AI 网关 `https://ai.xiangban.xyz`，客户端编号 `xiaoban-android`；中转 `https://link.xiangban.xyz/api/link`（桌面端默认值，手机端经桌面二维码拿到地址）。

## 依赖关系与顺序

```
账号中心（登记 xiaoban-android） ──┬─► server-ai（把 xiaoban-android 加进产品）   手机云端聊天
                                  ├─► server-forum（接受 xiaoban-android 的令牌）  手机社区的登录操作
                                  └─► server-link（新服务，用账号中心 JWKS 验签）   手机 ↔ 桌面互联
```

按这个顺序做，每步验证通过再下一步：账号中心 → server-ai → server-forum → server-link → 真机联调。

## 1. 账号中心

- **要求**：线上版本包含 `xiaoban-android` 客户端（启动时登记，新版本默认内置；旧版本升级，或在 `ACCOUNT_CLIENTS` 里补 `xiaoban-android:小伴手机应用`）。部署流程见 [ACCOUNT_CENTER_DEPLOYMENT.md](ACCOUNT_CENTER_DEPLOYMENT.md)。
- **检查**：`POST /api/auth/password/login`，`client_id` 填 `xiaoban-android`、账号密码随便填一个不存在的：返回 401（账号或密码错误）说明客户端已登记；返回 400 说明没登记。
- **注意**：升级会重启账号中心，先备份数据库和 `.env`，且 `ACCOUNT_ISSUER` 不能变（变了所有已发令牌失效）。

## 2. server-ai（AI 网关，手机云端聊天）

- **要求**：令牌 `aud` 必须属于某个已启用的产品，否则返回 403。这是后台数据，不是代码：在 AI 网关管理后台「产品与档位」里，把 `xiaoban-android` 加进现有小伴产品的 `client_ids`（和 `xiaoban-desktop-wallpaper` 同一个产品，才能沿用同一份试用额度）。**一个客户端只能属于一个产品**，加之前先确认它没有出现在别的产品里。
- **如果还没部署**（`services/server-ai/README.md` 写的是“未部署”，而桌面端默认就连 `https://ai.xiangban.xyz`，先确认线上到底有没有）：它依赖 New API 网关，部署和后台配置另行处理；默认端口 55010 与论坛、市场的默认端口相同，同机部署前先用 `ss -ltnp` 看清占用，改 `AI_PORT`。
- **检查**：用测试账号的令牌（`aud=xiaoban-android`）调用 `GET /api/ai/tiers` 和 `GET /api/ai/balance`（只读，不会触发开通），应为 200 而不是 403。

## 3. server-forum（论坛，手机社区的登录操作）

- **要求**：包含 `FORUM_ACCOUNT_EXTRA_CLIENT_IDS` 的版本（默认值就是 `xiaoban-android`，不设环境变量即可）。旧版本只接受网页的 `forum-web`，手机上发帖、评论、点赞、“我的”会被拒绝。网页的 `FORUM_ACCOUNT_CLIENT_ID` 保持不变。
- **部署方式**（见论坛 README 末尾）：香港机 `/home/ubuntu/xiangban-forum`，systemd 服务 `server-forum`，监听 `127.0.0.1:55010`；本地交叉编译后替换二进制和 `web/dist`，重启服务。库迁移在启动时自动完成，不需要手工改库。
- **检查**：先看启动日志里有 `extra_clients=["xiaoban-android"]`；再用测试账号的令牌（`aud=xiaoban-android`）调用 `GET /api/me/points`（需要登录的只读接口）应为 200；用 `aud=forum-web` 的网页登录仍然正常；其他 `aud` 仍然 401。

## 4. server-link（互联中转，新服务）

按 [LINK_DEPLOYMENT.md](LINK_DEPLOYMENT.md)。独立栈、自带 PostgreSQL、端口 55030、域名 `link.xiangban.xyz`，Nginx 要放行 WebSocket。

## 5. 真机联调

1. GitHub Actions 运行 “Android and Link Core”，下载 `xiaoban-android-debug-apk` 装到手机；运行 “Desktop Agent Platform Tests” 确认桌面端在 Windows、macOS 上通过。
2. 桌面端（新版本）登录账号 → “设置 → 连接手机”出二维码；手机登录同一账号，设备页扫码配对。
3. 手机“小伴”页：切到“云端小伴”聊一句（验证 server-ai）；切到“我的电脑”发一条消息，看流式回复；让桌面触发一次需要批准的操作，在手机上批准一次（验证 server-link）。
4. 手机“社区”页登录后发一条回复或点赞（验证论坛）。
5. 结果记到 [实施计划_手机桌面互联](../product/实施计划_手机桌面互联.md) 的 L3 一节。

## 发版

桌面端要出新版本才有手机互联与本轮修复；Android APK 自己分发（见实施计划“已定事项”）。
