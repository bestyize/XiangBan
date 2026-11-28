# 小伴游戏厅（`game.xiaban.xyz`）移动端 H5

面向移动端优先、零广告、免登录即玩的精品小游戏合集站（Vue 3 + Vite + TypeScript + PWA 离线缓存），收录小伴现役 **8 款官方游戏**（总包体约 3.8 MB），由香港云服务器宿主机 Nginx 纯静态托管。

- 评审设计稿：[xiaoban-games-h5.html](../xiaoban-multiplatform/docs/design-specs/prototypes/xiaoban-games-h5.html)
- 香港机部署说明：[infra/game/README.md](../../infra/game/README.md)

## 收录作品（8 款）

| 游戏 | 目录 ID | 类型 | 方向 |
| --- | --- | --- | --- |
| 水墨剑客 | `ink-blade` | 动作 | 横屏 |
| 倒杯快乐 | `water-sort` | 解压 | 竖横皆可 |
| 暮色滑翔 | `dusk-glider` | 解压 | 横屏 |
| 方块爆破 | `block-puzzle` | 益智 | 竖屏 |
| 2048 | `merge-2048` | 益智 | 竖屏 |
| 小伴贪吃蛇 | `snake-arena` | 动作 | 横屏 |
| 3D全阶魔方 | `pro-cube` | 益智 | 横屏 |
| 雅韵五子棋 | `jade-gomoku` | 棋牌 | 横屏 |

运行 `dev` / `build` 前会自动执行 `scripts/sync-games.mjs`，从 `client/xiaoban-desktop/public/game-packs/` 无侵入同步 8 款官方游戏包并注入 `public/xb-bridge.js`（Web 协议 2 运行时桥接），播放器同时兼容协议 1 握手与协议 2 `window.xb` 接口。

## 开发与验证

```sh
npm --prefix client/xiaoban-games-h5 install
npm --prefix client/xiaoban-games-h5 test
npm --prefix client/xiaoban-games-h5 run build
npm --prefix client/xiaoban-games-h5 run dev
```

可选环境变量（登录注册弹窗默认对接相伴统一账号中心）：
- `VITE_ACCOUNT_API_BASE`：默认 `https://account.xiangban.xyz`
- `VITE_ACCOUNT_CLIENT_ID`：默认 `xiaban-web`
