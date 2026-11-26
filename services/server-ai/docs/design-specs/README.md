# 小伴 AI 网关管理后台 · 设计资源索引

本目录存放 `services/server-ai/web` 管理后台的交互设计与重设计原型，源文件随仓库维护，不参与生产构建。

## 设计稿清单

| 设计稿 | 状态 | 说明 | 预览方式 |
| --- | --- | --- | --- |
| [AI 网关管理后台 · 官方设计规范版原型](prototypes/gateway-admin-redesign.html) | 已合入 | 严格遵循小伴全端统一设计规范：无色偏纯中性画布（浅 `#FAFAFA/#FFFFFF/#F2F2F2`、深 `#111111/#1C1C1E/#2C2C2E`）、6 套官方主题色（默认小伴红）+ 日夜切换、中性底 + 6px 语义圆点状态标签、236px 纯白左侧栏 + 52px 面包屑顶栏、16px 无阴影静态卡片、24px 弹窗、0 原生下拉/复选框与一项一行悬停提示规范 | 直接打开 HTML 或运行 `python3 docs/design-preview/serve.py --open` |

## 拟同步微调的后端管理接口（落地阶段待确认）

当前原型 90%+ 能力可直接基于现有 `/api/admin/*` 接口实现；以下 4 项小幅增强可按需在落地 Vue 工程时顺带补充：

1. `GET /api/admin/bindings` 增加可选筛选 `filter=funded|empty|anomaly` 与排序 `sort=last_used|balance|created`（当前仅支持 `q` / `product_id` 与固定排序）。
2. `GET /api/admin/audit` 增加可选 `category` / `q` 筛选参数，或沿用前端本地过滤（当前返回最近 100–200 条）。
3. `GET /api/admin/products/{id}/tiers` 与 `PUT` 对齐暴露已有的 `capabilities` 字段到前端表单（后端 `TierReq` 已支持）。
4. `GET /api/admin/batches` 增加可选 `product_id` / `status` 筛选（前端亦可对批次列表本地筛选）。
