# MBTI 报告与分享卡 · 设计说明

本轮为设计原型，不修改题库、计分和客户端。示例为 INFJ，测试日期 2026.11.08 21:46，48/48 题，用时 08:26；数字和解读用于展示排版。四维分数不表示能力、排名或类型匹配率，不额外添加 A/T 维度。

- [完整报告与分享卡](http://127.0.0.1:1424/prototypes/app-plaza.html?board=b10,b11)
- [16 角色图鉴](http://127.0.0.1:1424/prototypes/app-plaza.html?board=b12)
- 完整报告：概览、四维倾向、优势与盲点、工作方式、人际关系、一周行动、写给此刻的你。报告文案采用具体的性格洞察和生活建议，收尾用总结与邀请，避免反复解释“不能代表什么”。支持章节跳转、展开维度解读、切换优势/盲点、勾选行动。
- 分享卡：3:4，人物、类型、短句、可选维度值、测试日期与时间。两种纸色，保存导出 1080×1440 PNG；不自动发布。
- 朋友圈分享分为「图片海报」和「结果链接」两个页签，两者都有「分享到朋友圈」主操作。图片发送当前海报，保留保存图片入口；链接展示标题与缩略图，接收方看到精简结果和测试日期，保留复制链接入口。发布动作交给微信，由用户确认。
- [链接分享画板](http://127.0.0.1:1424/prototypes/app-plaza.html?board=b13)。当前原型仅演示微信跳转，不调用微信、不创建线上结果链接。正式实现需要客户端接入微信开放平台分享能力及公开结果快照服务；SDK 回调不能替代朋友圈实际发布确认。图片导出已实现，复制链接在本稿中明确提示为预览。

## 视觉参考与取舍

用户提供的本机参考图位于 `C:/Users/sdjz0/Desktop/新建文件夹/`。参考了它们的章节结构、人物场景、双端维度条和优势/盲点组织方式。正文与人物均重新设计，不直接复制原截图。

- [16Personalities 人格类型与四组角色](https://www.16personalities.com/personality-types)：参考分组及主题色惯例。
- [INFJ 人格页](https://www.16personalities.com/infj-personality)：参考“人物场景 + 分章报告”的信息组织。
- [INFJ 优势与盲点](https://www.16personalities.com/infj-strengths-and-weaknesses)：参考分段呈现方式，原型使用重新撰写的简短情境文案。
- [Psychdom 分享卡示例](https://www.psychdom.com/sample-reports/share-cards)：参考完整报告与公开分享卡的内容边界。
- [16Personalities 框架说明](https://www.16personalities.com/articles/our-theory)：其第五维 A/T 属于该站的框架，本项目当前四维测评不直接照搬。

以下色值是本稿配色，不声称为官方色号。NT 紫 `#8050AA`，NF 绿 `#168368`，SJ 蓝 `#2685AD`，SP 金黄 `#C7900B`。人物与主题色有辨识度，阅读区以白底和深色文字为主。

## 原创角色概念

| 分组 | 类型与角色 | 识别物 |
| --- | --- | --- |
| NT 紫 | INTJ 构想师 / INTP 求知者 / ENTJ 领航者 / ENTP 破题者 | 图纸与圆规 / 交织圆环 / 路线旗 / 两种观点的几何块 |
| NF 绿 | INFJ 提灯者 / INFP 拾光者 / ENFJ 筑桥者 / ENFP 追风者 | 提灯与书 / 花与手账 / 小桥 / 纸风筝 |
| SJ 蓝 | ISTJ 守序者 / ISFJ 育苗者 / ESTJ 统筹者 / ESFJ 暖场者 | 书与清单 / 幼苗 / 夹板与积木 / 茶盘 |
| SP 黄 | ISTP 造物者 / ISFP 绘梦者 / ESTP 跃行者 / ESFP 舞光者 | 机械工具 / 调色盘 / 球 / 飘带 |

`mbti-character-atlas.png` 是 16 人格形象概念图，按四行四列排列。各角色有独立的姿态、轮廓和道具；后续正式制作应逐一输出独立素材。`mbti-observer-jade.png` 是当前 INFJ 报告/海报共用的高分辨率场景。图片通过内置 imagegen 生成；未使用付费 API 脚本。

`mbti-work.png` 与 `mbti-relations.png` 是同一主角的工作、交流章节插画。`mbti-share-sample.png` 是通过原型海报绘制器实际导出的 1080×1440 示例，可独立查看；[完整提示词](mbti-prompts.json)记录了人物场景与章节图的生成过程。

验证：390/360px 画板无横向溢出；章节跳转、维度解读、优势/盲点切换、成长勾选、纸色切换、维度显隐与 PNG 生成通过浏览器验证。分享不会自动发送到第三方；当前为 INFJ 完整报告示例，图鉴提供 16 种原创角色概念，不表示其他 15 种完整报告已制作。

## 生成提示词

INFJ final edit prompt: Preserve the character, pose, open book garden composition, all objects and true transparent background. Replace dull olive/sage with saturated jade and emerald green, turquoise highlights and forest shadows; keep the ivory scarf, book and tree, and vivid warm golden lantern. Keep paper-craft texture, bright ivory highlights and tasteful diffuse lighting. Fresh and vivid, not neon. No text, logo or new objects. Preserve alpha transparency.

Atlas prompt: Exactly 16 original full-body people in a 4 by 4 grid, white background, consistent scale, complete silhouettes. Premium dimensional paper craft with faceted clothing and softly modeled friendly faces. Four purple analysts (architect with compass; thinker with interlocking rings; leader with route flag; debater balancing contrasting shapes), four emerald diplomats (observer with lantern and notebook; poet with flower and journal; mentor with small bridge; explorer with paper kite), four blue sentinels (archivist with books; caretaker with plant; coordinator with clipboard; host with tea tray), four gold explorers (artisan with mechanism; painter with palette; athlete with ball; dancer with ribbon). Distinct faces, hairstyles, actions and props; no recolored duplicates. No text, grid borders, logos or existing 16Personalities characters. Reference the original INFJ scene for the paper-craft visual language.

## 版式精修

首屏使用类型与人物并排的封面构图，测试记录与章节导航进入首屏。六章共用 20px 内容边距、26px 编号标记和细分隔线；正文以 13–14px、1.75 倍行高组织，表格与分段选择器统一对齐。390px 和 360px 已检查章节截图，正文与表格无横向溢出。

## 海报扫码入口

海报右下角加入「扫码测一测」二维码，指向 `https://chat.xiangban.xyz/create?kind=game&game=mbti`，对应现有网页的 MBTI 直达入口。二维码不携带用户的答案或私有结果。`mbti-test-qr.png` 和 `mbti-test-qr.svg` 用 qrcode 库按 M 级纠错和四模块静区生成；已用 ZXing 解析最终 1080×1440 PNG，确认解码为该地址。页面预览与图片导出均包含二维码和测试日期。

## 分析过渡页

[分析画板](http://127.0.0.1:1424/prototypes/app-plaza.html?board=b14)展示四维线条与纸片标记、三步整理状态。答题演示最后一题选择后出现“提交结果”，点击后经过约 2.75 秒的过渡再进入完整报告。画板外“播放完整过渡”可重播；退出、重置或重新开始会使旧过渡失效。减少动态效果模式关闭循环动画。文案对应本地计分与报告整理，不出现大模型调用或虚构进度百分比。

## 16 型主题预览

预览顶部可切换全部 16 型，也可用 `?board=b10&type=INTJ` 指定类型。`mbti-type-data.js` 是当前离线报告引擎生成的 16 份示例数据快照，维度多数侧示例为 8/12、8/12、9/12、7/12；文案、角色、分组色和海报数据一起切换。INFJ 使用独立场景插画，其余类型使用各自原创角色。分析过渡页在揭晓前使用中性底色与四组色彩，不提前套用固定绿色。
