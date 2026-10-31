# 首发原创资源包（55 个）

11 类各 5 个。源目录为 `packs/<slug>/`，每个目录就是 ZIP 根目录。未发布源不含账号、密钥或用户资料。

## 使用边界

- effect / wallpaper / widget / game 是离线作品，按桌面资源包格式制作，下载与导入不代表自动应用。
- routine / reminder / task / goal / skill / expert / workflow 为 `manualOnly:true` 的可下载手动模板：包含实际步骤、JSON、可填写工作页、具体填写示例与提示词。它们不会自动创建任务或执行工作流。
- 组件全部使用 HTML/CSS 原生交互，不依赖内联脚本，不承诺持久保存。游戏不保存数据、不联网。
- 所有示例记录均为虚构。页面素材、几何图形、代码与文档均为原创，无外部字体、图片或 CDN 依赖。

## 清单

| 类型 | 作品 | 源目录 |
| --- | --- | --- |
| routine | 安静开工七分钟 | `packs/quiet-start/` |
| routine | 下班收尾十分钟 | `packs/clear-shutdown/` |
| routine | 周五桌面归零 | `packs/weekly-reset/` |
| routine | 任务切换接力条 | `packs/context-bridge/` |
| routine | 学习闭环四步卡 | `packs/study-loop/` |
| reminder | 屏幕间歇提醒文案包 | `packs/screen-pause/` |
| reminder | 绿植巡看清单 | `packs/plant-round/` |
| reminder | 订阅到期自查表 | `packs/bill-check/` |
| reminder | 个人备份巡检提醒 | `packs/backup-check/` |
| reminder | 离桌前五件事 | `packs/leave-desk/` |
| task | 小版本发布核对单 | `packs/release-readiness/` |
| task | 缺陷复现证据袋 | `packs/bug-evidence/` |
| task | 会议行动项落地表 | `packs/meeting-followups/` |
| task | 搬家小物盘点单 | `packs/home-inventory/` |
| task | 一篇文章三张卡 | `packs/reading-extract/` |
| goal | 四周主题阅读路线 | `packs/reading-four-weeks/` |
| goal | 四周作品集小冲刺 | `packs/portfolio-sprint/` |
| goal | 两周数字整理计划 | `packs/declutter-two-weeks/` |
| goal | 二十一天输入练习册 | `packs/typing-practice/` |
| goal | 六周小项目航线 | `packs/sideproject-map/` |
| skill | 证据与断言核对技能 | `packs/evidence-audit/` |
| skill | 问题分诊技能 | `packs/issue-triage/` |
| skill | 面向用户的更新说明技能 | `packs/release-notes/` |
| skill | CSV质量检查说明技能 | `packs/csv-quality/` |
| skill | 长文结构提炼技能 | `packs/doc-distill/` |
| expert | 技术文档编辑提示词 | `packs/technical-editor/` |
| expert | 资料整理员提示词 | `packs/research-librarian/` |
| expert | 小组讨论引导员提示词 | `packs/meeting-facilitator/` |
| expert | 中文界面校对员提示词 | `packs/localization-reader/` |
| expert | 产品方案追问者提示词 | `packs/product-questioner/` |
| workflow | 反馈到复现手动工作流 | `packs/bug-to-repro/` |
| workflow | 想法到小实验手动工作流 | `packs/idea-to-test/` |
| workflow | Markdown文章交付工作流 | `packs/markdown-to-release/` |
| workflow | 照片归档手动工作流 | `packs/photo-archive/` |
| workflow | 功能交接手动工作流 | `packs/feature-handover/` |
| effect | 琥珀纸屑 | `packs/amber-paper/` |
| effect | 夜池回环 | `packs/pond-rings/` |
| effect | 浮光微尘 | `packs/drift-motes/` |
| effect | 纸舟轻航 | `packs/sail-clouds/` |
| effect | 蓝调细雨 | `packs/blue-hour-trails/` |
| wallpaper | 潮汐绸带 | `packs/tidal-silk/` |
| wallpaper | 等高线山谷 | `packs/contour-valley/` |
| wallpaper | 轨道工作室 | `packs/orbital-studio/` |
| wallpaper | 晨光玻璃 | `packs/morning-glass/` |
| wallpaper | 灯影港湾 | `packs/lantern-harbor/` |
| widget | 呼吸留白卡 | `packs/breathing-circle/` |
| widget | 今日三件事 | `packs/three-priorities/` |
| widget | 选择罗盘 | `packs/decision-compass/` |
| widget | 工位收尾折页 | `packs/desk-reset-card/` |
| widget | 专注边界门牌 | `packs/focus-boundary/` |
| game | 记忆果园 | `packs/memory-orchard/` |
| game | 信号花园 | `packs/signal-garden/` |
| game | 口袋迷径 | `packs/pocket-labyrinth/` |
| game | 方块归港 | `packs/tile-harbor/` |
| game | 接住星光 | `packs/starlight-catcher/` |

## 构建产物

本次本地发布产物在 `/tmp/community-pack-build`：

- `catalog.json`：55 项后台登记字段、manifest、ZIP/封面本地路径、SHA-256 与大小。
- `data/packs/<kind>/<slug>-1.0.0.zip`：55 个实际下载包。
- `data/covers/<slug>-1.0.0.png`：55 张 PNG 封面。
- `browser-validation.json`：实际浏览器、CSP 组件、游戏交互与获胜路径结果。
- `structure-validation.json`：逐包路径、UTF-8、JSON、大小、SHA-256 与封面格式核对。
- `screenshots/`：实际页面和引擎截图；手动模板封面为代码原生的文档示意，不冒充产品截图。

打包时只把每个 slug 目录内文件写入根路径，排除外层 slug 目录；视觉包严格按允许文件列表。封面独立存放，不往 wallpaper/widget/effect ZIP 塞额外图片。

## 验证范围

已验证全部包的压缩结构、路径、JSON、UTF-8、大小及压缩包 SHA-256。5 个特效使用桌面现有 TS parsePack/ParticleSystem/drawEffects 真实解析渲染；5 个壁纸和 5 个组件用 Chromium headless shell 1248 加载，组件额外在 sandbox allow-scripts、script-src self 下验证 HTML/CSS 交互。5 个游戏均验证实际获胜、重开与触控，迷宫也验证键盘移动，滑块验证尺寸切换，接星光验证暂停。无网络请求或浏览器脚本错误。

桌面 Rust effect_packs::validate 和 custom_widgets::validate 已由集成任务使用原文件轻量 harness 逐一验证；没有宣称完整 Tauri ZIP 导入、安装或所有平台运行已经完成。
