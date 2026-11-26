# 小伴 AI 网关管理后台 · 设计资源索引

本目录存放 `services/server-ai/web` 管理后台的交互设计与重设计原型，源文件随仓库维护，不参与生产构建。

## 设计稿清单

| 设计稿 | 状态 | 说明 | 预览方式 |
| --- | --- | --- | --- |
| [AI 网关管理后台极简重设计原型](prototypes/gateway-admin-redesign.html) | 已合入 | 对齐小伴暖白极简 + 墨黑中性基调（支持浅/深色切换、`$ + 小伴点` 双单位对照、`⌘K` 全局检索），重构全部 8 个页面、双域档位路由、可搜索模型选择器、跨页直达用户抽屉与统一确认弹层；已落地至 `services/server-ai/web` 与 `src/handlers/admin.rs` | 直接打开 HTML 或运行 `python3 docs/design-preview/serve.py --open` |

## 拟同步微调的后端管理接口（落地阶段待确认）

当前原型 90%+ 能力可直接基于现有 `/api/admin/*` 接口实现；以下 4 项小幅增强可按需在落地 Vue 工程时顺带补充：

1. `GET /api/admin/bindings` 增加可选筛选 `filter=funded|empty|anomaly` 与排序 `sort=last_used|balance|created`（当前仅支持 `q` / `product_id` 与固定排序）。
2. `GET /api/admin/audit` 增加可选 `category` / `q` 筛选参数，或沿用前端本地过滤（当前返回最近 100–200 条）。
3. `GET /api/admin/products/{id}/tiers` 与 `PUT` 对齐暴露已有的 `capabilities` 字段到前端表单（后端 `TierReq` 已支持）。
4. `GET /api/admin/batches` 增加可选 `product_id` / `status` 筛选（前端亦可对批次列表本地筛选）。
