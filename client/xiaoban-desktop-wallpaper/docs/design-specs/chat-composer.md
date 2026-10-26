# 聊天输入框：技能 / 专家 / 连接器

新建任务和会话页共用同一个输入框（`src/ui/components/ChatComposer.vue`）。使用技能、专家、连接器时，三类入口各有固定位置，工具栏不再放它们：

```text
┌──────────────────────────────────────────────┐
│ [⚡ PPT 演示文稿 ×]      [软 软件开发团队 ×] [飞 G 历] │  ← 顶部一行
│ 文字……                                          │
│ ＋  工作区  权限            ○  模型  🎙  ∿  ↑   │  ← 工具栏
└──────────────────────────────────────────────┘
```

| 能力 | 位置 | 作用范围 | 入口 | 代码 |
| --- | --- | --- | --- | --- |
| 技能 | 顶部左侧 chip，可多选（最多 3 个） | 一次性：发送后清除 | 输入 `/`，或 ＋ › 技能，或 `@` | `ChatComposer.vue`、`chat/slash.ts` |
| 专家 / 专家团 | 顶部右侧 chip，圆形头像是专家、圆角方块是专家团 | 整个对话；新对话开始前可点 × 取消，开始后只显示 | ＋ › 专家，或 `@` | `ChatExpertChip.vue` |
| 连接器 | 顶部右侧重叠图标，最多 3 个，其余折成 +N；点开逐个开关 | 整个对话；换一个对话才清空 | ＋ › 连接器，或 `@` | `ChatConnectorCluster.vue` |

- 技能发送时写成消息开头的 `/名称 `，后端 `skill_invocation` 照旧展开，对话记录里也留着用了什么。技能 chip 单独也能发送。
- 连接器的开关只决定这个对话里「优先使用」哪些（随每条消息作为结构化 `mentions` 发送），连接和授权仍在 AI 插件里完成；没连上的连接器只给「去连接」。
- 顶部一行没有任何内容时不占位；技能放不下会自己换行，右侧的专家名先用省略号截断。
- 连接器弹层在新建任务页往下展开，在会话页往上展开。
- 专家是对话的身份，所以不能中途更换；要换人，在新建任务里重新召唤。

验证：`npx vitest run src/ui/components/ChatConnectorCluster.test.ts src/ui/components/ChatConnectorMentions.test.ts src/ui/chat/slash.test.ts src/ui/pages/ChatPage.test.ts src/ui/pages/NewTaskPage.test.ts`。
