# 2048 · 黑曜

独立 HTML/CSS/JavaScript 游戏。黑曜石环境、石质凹槽棋盘、青灰到琥珀金的数字块、125ms 滑动、合并光粒和得分浮字；可选本地合成音效默认关闭。无网络、字体、渲染库或存储依赖，遵循减少动态效果设置，后台暂停声音。

- 方向键 / WASD、棋盘滑动或四个方向按钮操作。每次有效移动随机增加 2（90%）或 4（10%），一次移动中每块最多合并一次。
- 达到 2048 可继续；无法移动时结算。重新开始保留本次打开期间的最高分，刷新后清空。
- 桌面和 Android 使用协议 2 注入的 `XiaobanBuiltinRuntime`，隐藏包内退出按钮，由宿主提供退出。安全区使用 `--xb-safe-*`，顶栏按 `--xb-capsule-*` 局部避让。兼容旧协议 1 握手。
- `rules.js` 返回规则结果与每个源方块的移动轨迹。动画期间最多缓存一个后续方向，重新开始取消未完成动画，不延迟写回旧棋局。
- 基准入口保留共享结算分享；桌面副本移除分享资源和按钮。旧 `theme.css` 仅保留供基准校验，不再加载。
- 桌面实际资源位于 `client/xiaoban-desktop/public/game-packs/merge-2048/`；Android 构建从桌面目录打包。

验证：`node --test client/xiaoban-desktop/scripts/merge-2048.test.mjs`；资源校验 `node client/xiaoban-desktop/scripts/check-game-assets.mjs`；桌面构建 `npm --prefix client/xiaoban-desktop run build`。浏览器检查桌面、手机竖屏 / 短横屏、减少动态效果、胜负结算和协议 2 胶囊避让。

## 材质来源

`obsidian.webp` 由内置 imagegen 生成原图后经 Sharp 压缩为 WebP（1536 × 1024），随包离线分发。`cover.svg` 为可编辑矢量封面。

最终生成提示词（内置工具模式）：

> Create a production background asset for a cinematic premium 2048 puzzle game, landscape 1536x1024. Photorealistic macro photograph of a vast dark obsidian stone tabletop, very fine charcoal mineral grain and faint hairline bronze-gold veins. A soft warm amber shaft of light enters from upper right through atmospheric haze, subtle cold teal ambient reflection far left. Center 60 percent stays extremely dark, uncluttered and low contrast so a game board can be placed over it. Luxury architectural still life, restrained chiaroscuro, tactile fine stone detail, analog film grain, deep black shadows, physically plausible surface, cinematic color grading. Full bleed material environment only. NO objects, NO tiles, NO board, NO typography, NO letters, NO numbers, NO watermark. Save generated asset for local project use.
