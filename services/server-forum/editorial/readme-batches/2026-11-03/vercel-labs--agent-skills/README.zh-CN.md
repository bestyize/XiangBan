> 译注：代码块内的英文提示词示例为上游原文，未翻译。

# Agent Skills

面向 AI 编码智能体的技能集合。技能是打包的说明和脚本，用于扩展智能体能力。

技能遵循 [Agent Skills](https://agentskills.io/) 格式。

[![skills.sh](https://skills.sh/b/vercel-labs/agent-skills)](https://skills.sh/vercel-labs/agent-skills)

## 可用技能

### vercel-optimize

审查 Vercel 项目的成本、性能、可靠性、缓存、函数使用和计费优化机会。它首先收集 Vercel 指标，然后只检查这些指标指向的路由和文件。

**适用场景：**

- 优化已部署的 Vercel 项目
- 降低 Vercel 成本或函数使用量
- 排查缓慢或成本较高的路由
- 查找缓存、ISR、中间件、图片或构建时长问题
- 生成按优先级排列的成本和性能报告

### react-best-practices

Vercel Engineering 提供的 React 和 Next.js 性能优化指南，包含 8 个类别的 40 多条规则，并按影响程度排序。

**适用场景：**

- 编写新的 React 组件或 Next.js 页面
- 实现客户端或服务端数据获取
- 审查代码中的性能问题
- 优化包体积或加载时间

**涵盖类别：**

- 消除瀑布式请求（关键）
- 包体积优化（关键）
- 服务端性能（高）
- 客户端数据获取（中高）
- 重新渲染优化（中）
- 渲染性能（中）
- JavaScript 微优化（中低）

### web-design-guidelines

审查 UI 代码是否符合 Web 界面最佳实践，依据涵盖无障碍、性能和用户体验的 100 多条规则进行检查。

**适用场景：**

- “审查我的 UI”
- “检查无障碍性”
- “审查设计”
- “审查用户体验”
- “按最佳实践检查我的网站”

**涵盖类别：**

- 无障碍（aria-labels、语义化 HTML、键盘处理）
- 焦点状态（可见焦点、focus-visible 模式）
- 表单（自动补全、验证、错误处理）
- 动画（prefers-reduced-motion、适合合成器的变换）
- 排版（弯引号、省略号、tabular-nums）
- 图片（尺寸、懒加载、替代文本）
- 性能（虚拟化、布局抖动、preconnect）
- 导航与状态（URL 反映状态、深链接）
- 深色模式与主题（color-scheme、theme-color meta）
- 触摸与交互（touch-action、tap-highlight）
- 区域设置与国际化（Intl.DateTimeFormat、Intl.NumberFormat）

### writing-guidelines

按 Vercel 写作手册审查文档与文章，依据覆盖语气、结构、内容类型、代码示例、排版和 AI 工作流的 80 多条规则检查页面。

**适用场景：**

- “审查我的文档”
- “检查写作风格”
- “审查文章”
- “审查文档的表达和语气”
- “按照写作手册检查此页面”

**涵盖类别：**

- 规划（内容计划、内容类型、面向用户的标题）
- 表达与语气（主动语态、直接称呼，禁用 `easy`/`simple`/`quick` 等词）
- 按内容类型调整语气（教程、操作指南、参考、概念、故障排除）
- 标题与结构（句首字母大写、描述性小标题、以 TL;DR 开头）
- 列表（何时使用、粗体／说明格式）
- 代码（语言标签、TypeScript 优先、`<Steps/>`、80 列／25 行限制）
- 占位符、单位与数字（`snake_case` 文本、递增数字、`64 KB`/`200 ms`）
- 排版（不以长破折号作标点、弯引号、省略号字符、不换行空格）
- 源格式（不强制换行、不使用 `---` 分隔线、规范空行）
- 定价与金额页面（表格、完整细节）
- AI 工作流（责任、企业模型、先规划、披露 AI 使用）
- 审查（PR 描述规范、作者责任）

### react-native-guidelines

为 AI 智能体优化的 React Native 最佳实践，包含 7 个章节的 16 条规则，涵盖性能、架构和平台特定模式。

**适用场景：**

- 构建 React Native 或 Expo 应用
- 优化移动端性能
- 实现动画或手势
- 使用原生模块或平台 API

**涵盖类别：**

- 性能（关键）——FlashList、记忆化、密集计算
- 布局（高）——flex 模式、安全区域、键盘处理
- 动画（高）——Reanimated、手势处理
- 图片（中）——expo-image、缓存、懒加载
- 状态管理（中）——Zustand 模式、React Compiler
- 架构（中）——单体仓库结构、导入
- 平台（中）——iOS/Android 特定模式

### react-view-transitions

使用 React 的 View Transition API 实现流畅、具原生感的动画。涵盖 `<ViewTransition>` 组件、`addTransitionType`、过渡类型及 Next.js 集成，包括 `next/link` 的 `transitionTypes` 属性。

**适用场景：**

- 添加页面过渡或路由动画
- 为组件进入／退出添加动画
- 创建共享元素过渡（列表到详情的变形）
- 实现方向性导航动画（前进／后退）
- 在 Next.js App Router 中集成视图过渡
- 为列表重排或 Suspense 后备内容显示添加动画

**涵盖主题：**

- `<ViewTransition>` 组件（进入、退出、更新、共享触发器）
- 使用 `addTransitionType` 实现方向性／上下文特定动画
- View Transition Classes 和 CSS 伪元素
- 使用 `name` 属性实现共享元素过渡
- 通过 Web Animations API 实现 JavaScript 动画
- Next.js `next/link` 上的 `transitionTypes` 属性
- 可直接使用的 CSS 动画配方（淡入淡出、滑动、缩放、翻转）
- 无障碍（`prefers-reduced-motion`）

### composition-patterns

可扩展的 React 组合模式，通过复合组件、状态提升和内部组合帮助避免布尔属性泛滥。

**适用场景：**

- 重构具有大量布尔属性的组件
- 构建可复用组件库
- 设计灵活的 API
- 审查组件架构

**涵盖模式：**

- 提取复合组件
- 提升状态以减少属性
- 组合内部结构以提高灵活性
- 避免逐层传递属性

### deploy-to-vercel

将应用和网站部署到 Vercel。优先对已关联项目使用 git-push 部署，必要时回退到 Vercel CLI；未登录时使用 `vercel deploy --temporary`。临时部署会返回领取 URL，供用户保留部署。

**适用场景：**

- “部署我的应用”
- “将它上线”
- “部署并给我链接”
- “创建预览部署”

## 安装

```bash
npx skills add vercel-labs/agent-skills
```

## 用法

安装后技能会自动可用。检测到相关任务时，智能体会使用它们。

**示例：**

```
Deploy my app
```

```
Review this React component for performance issues
```

```
Help me optimize this Next.js page
```

## 发现索引

`main` 上每次技能变更都会发布不可变的 GitHub 发行版，其中包含 Agent Skills 发现索引及每项技能各一个产物。可在本地构建相同产物：

```bash
npm ci --ignore-scripts
node scripts/build-discovery-index.mjs https://example.com/skills
```

## 技能结构

每项技能包含：

- `SKILL.md` - 给智能体的说明
- `scripts/` - 用于自动化的辅助脚本（可选）
- `references/` - 支持文档（可选）

## 许可证

MIT
