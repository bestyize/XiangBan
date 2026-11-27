# 人格贩卖机 VEND·ME · 趣味与人格测评 H5

面向移动端 H5 优先的纯前端测评站点（Vue 3 + Vite + TypeScript），定稿视觉为「方案 A · 潮玩盲盒 / 潮流杂志风」。无需后端即可静态部署，预留本地/云端存储适配接口以备后续接入统一账号中心。

- 产品方案：[docs/product/PRD_趣味与人格测评H5.md](../../docs/product/PRD_趣味与人格测评H5.md)
- 评审设计稿：[client/xiaoban-multiplatform/docs/design-specs/prototypes/personality-tests-h5.html](../xiaoban-multiplatform/docs/design-specs/prototypes/personality-tests-h5.html)

## 已实现范围（一期 5+1 首发完整版）

1. **C1 人类真实面具（SBTI 毒舌解构风）**：20 道原创生活场景题，8 种常规面具 + 2 种隐藏彩蛋，复古荧光黄大字报皮肤。
2. **C2 人格双色主导色**：16 道原创氛围场景题，8 套表层社交色 × 底层灵魂色双色渐变色卡。
3. **C3 动物塑深度版**：18 道原创日常反应题，8 种反差萌动物形象、原创矢量插画与饲养须知。
4. **C4 《和我相处的避雷说明书》**：18 道原创相处/情感题，6 款仿工业说明书型号与双人合盘入口。
5. **C5 精神状态 & 心理电量体检单**：16 道原创职场/日常状态题，6 份心理电量与精神年龄体检报告。
6. **P1 16 型人格速测版**：24 道原创中文场景题（每维度 6 题，合规原创，不冒充官方 MBTI），覆盖全部 16 型与荣格主导功能解读。
7. **零门槛双人合盘**：通过 URL Hash 编码发起方结果码、维度向量与昵称，受邀人答完即出双人合拍度、四维同轴对比、默契点与避雷提醒。
8. **高清 PNG 海报导出 & 黑金灵魂集卡档案**：前端 Canvas 实时渲染 `1080×1920`（9:16 朋友圈）与 `1080×1440`（3:4 小红书）高清无水印图片，兼容微信长按保存；本地 `localStorage` 自动断点续答并汇总多维灵魂总卡。

## 开发与验证命令

```sh
npm --prefix client/personality-tests-h5 install
npm --prefix client/personality-tests-h5 test
npm --prefix client/personality-tests-h5 run build
npm --prefix client/personality-tests-h5 run dev
```
