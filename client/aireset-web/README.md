# AI额度监控 QuotaReset（aireset-web）

状态：已实现首版（静态站），域名 `aireset.xiangban.xyz`，部署到香港云服务器，见 [infra/aireset/README.md](../../infra/aireset/README.md)。

御三家（ChatGPT/Codex、反重力 Antigravity、Claude）额度重置雷达与订阅额度实测对比站。视觉与交互按设计稿 [ai-quota-monitor.html](../../docs/design-preview/prototypes/ai-quota-monitor.html) 落地，PC 与移动端共用同一份响应式页面（容器查询）。

## 页面

| 路径 | 内容 |
| --- | --- |
| `/` | 重置雷达：三平台距上次全局重置计时与状态、30 天三轨时间轴、统一历史重置记录（点平台卡片可筛选） |
| `/benchmark/` | 订阅额度实测：10 档订阅的平均月 / 每周 / 五小时 Token 与 API 等值、月费倍率、每 $1 Token，同套餐多模型拆解 |

顶部「交流群」跳转小伴社区联系我们页（`chat.xiangban.xyz/contact`），「小伴产品门户」跳转 `xiaban.xyz`。

## 数据现状（重要）

- **重置时间轴与历史记录目前是演示数据**（页面上有声明），官方信源自动接入与人工核验流程尚未实现；在此之前不要把它当作真实重置时间。
- 订阅额度数据整理自 ModelDial 订阅额度实测公开页面（快照 2026-11-25），页面已标注来源并外链。ModelDial 的 CC BY 4.0 许可只明确覆盖其「公开雷达」数据集，订阅额度页面的转载授权未确认，正式长期使用前建议联系对方授权。
- 数据目前直接写在 `src/index.html` 的脚本里（`PLATFORMS`、`TIMELINE_EVENTS`、`BENCHMARKS`），后续接入真实采集时再抽成独立数据源。

## 开发与验证

零依赖，需要 Node.js 18+ 与一个 Chromium 内核浏览器（Edge / Chrome / Chromium）用于构建期预渲染：

```sh
cd client/aireset-web
npm test            # 构建 dist/、预渲染、自检（SEO 标签、预渲染行数、无预览残留）
python3 -m http.server 8090 -d dist   # 本地预览 http://127.0.0.1:8090/
```

`scripts/build.mjs` 为每个入口页生成独立 `title / description / canonical / og`，用无头浏览器把脚本生成的表格与卡片写进静态 HTML（爬虫无需执行 JS 就能读到内容，页面脚本保留，浏览器端接管交互），并生成 `favicon.svg`、`robots.txt`、`sitemap.xml`、`404.html`。未找到浏览器时可设 `BROWSER_BIN`，或加 `--no-prerender` 仅作调试（不利于 SEO）。

## 边界

- 纯前端静态站，没有后端、账号或数据库；不依赖其他客户端源码。
- Nginx 不做 SPA 全量回退，未知路径返回真实 404。
- 不要把真实凭据、服务器密钥写进本目录。
