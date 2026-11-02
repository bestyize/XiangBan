# 模型厂商图标

图标随 App 打包为 Compose ImageVector，运行时不请求远程图片。原始 SVG/图片、来源、版本、别名记录在 manifest.json，Lobe Icons 的 MIT 授权见 LICENSE-lobe-icons。商标仅用于标识来源。

- 主体来自 https://github.com/lobehub/lobe-icons 的 `@lobehub/icons-static-svg@1.101.0` 单色图标。
- InclusionAI 使用其 Ling 模型标志（https://github.com/inclusionAI/Ling）；Alibaba 使用企业标志，Qwen 保留原模型标志。
- Poolside 保留原始 SVG，`poolside-mark.svg` 取其前景路径用于单色小图标，省略透明度渐变遮罩。
- Celeris 和 Thinking Machines 使用官网 favicon；Reflection 使用 https://huggingface.co/reflection 组织头像。原始图片保留，单色 SVG 保留小图标尺度的覆盖度。Reflection 的两圆标志以品牌绿色显示。
- 既有 Claude、Google、DeepSeek、Qwen、Kimi、MiniMax、Doubao、智谱路径继续复用。未知厂商保留首字母兜底。

更新本地源文件和 manifest.json 后，在仓库根目录运行：

```sh
python3 client/xiaoban-multiplatform/business/community-native/tools/generate-vendor-logos.py
```

生成器不访问网络，也不依赖第三方 Python 库。新增复杂 SVG 需先人工归一化；不接受忽略变换或遮罩后直接生成。

网页附带的 `legacy-*.svg` 从 `client/xiaoban-multiplatform/business/community-native/src/commonMain/kotlin/xyz/xiangban/xiaoban/business/communitynative/BrandLogos.kt` 的原始画布 SVG 路径生成；`legacy.json` 记录别名与配色。商标仅用于标识来源。

2026-11-03：网页导航补充 `baidu.svg` 与 `spark.svg`，来自同版本 `@lobehub/icons-static-svg@1.101.0` 的原始单色 SVG（MIT），别名与来源登记在网页 manifest.json。

导航四分类新增的 OpenClaw、OpenCode、Cursor、TRAE、Qoder、WorkBuddy、ModelScope、Vercel、OpenRouter、SiliconCloud、Together、Fireworks、Groq 图标继续取自同版 Lobe Icons（MIT）。Hermes Agent 使用其官网声明的 favicon，原始来源：https://web-assets.nousresearch.com/marketing/1a978bb815e9d8b6e75e2320c4cf5e1313d7d684/assets/hermes-agent/icon.png；仅用于识别对应官方项目。
