# 小伴产品矩阵总站与双端下载页（`xiaban.xyz`）

面向主域名 `xiaban.xyz` 的轻量纯静态站点（Vue 3 + Vite + TypeScript），集中承载产品矩阵首页与桌面/手机独立下载页：

- **`/`**：小伴产品矩阵总入口，直达全部 6 款产品（小伴手机 App、小伴桌面、小伴社区、小伴游戏厅、人格贩卖机、小伴壁纸站）
- **`/desktop`（别名 `/pc`）**：小伴桌面独立下载页（实时读取 `/api/v1/desktop/product` 的 macOS / Windows 发版数据，支持 OS 自动识别、4 场景实机展示、下载后分平台安装指引与明暗切换）
- **`/app`（别名 `/mobile`）**：小伴手机 App 独立下载页（实时读取 `android-arm64` 发版数据，支持微信/QQ 拦截引导、iOS 网页生态导流、下载后安装指引抽屉、明暗切换与 PC 宽屏扫码直下）
- **`/download`**：按访问设备自动分流跳转至 `/app` 或 `/desktop`

所有展示图与游乐场图标均为 `.webp` 格式并托管于资源机 `https://res.xiangban.xyz/img/xiaban-portal/`（可通过环境变量 `VITE_ASSET_BASE` 覆盖）；香港机 Nginx 反代 `/api/v1/desktop/` 至 `https://wallpaper.xiangban.xyz/api/v1/desktop/`。

- 评审设计稿：[xiaban-matrix-portal.html](../../services/server-xiaoban/docs/design-specs/prototypes/xiaban-matrix-portal.html)
- 部署说明：[infra/portal/README.md](../../infra/portal/README.md)

## 开发与验证

```sh
npm --prefix client/xiaban-portal ci
npm --prefix client/xiaban-portal test
npm --prefix client/xiaban-portal run build
npm --prefix client/xiaban-portal run dev
```
