(() => {
  'use strict';

  /* ==========================================================================
     真实仓库内容与本地资源（均来自 services/server-forum）
     ========================================================================== */
  const WORKS = [
    {
      id: 'whale-girl',
      title: '潮汐书房 · 鲸鱼娘',
      category: '3D 互动',
      author: 'RealWorldBuilder',
      avatarBg: '#2F6FDB',
      date: '2026-11-19',
      likes: 186,
      commentsCount: 24,
      cover: './assets/whale-girl.webp',
      tall: true,
      description: '拜访月湾书房的溟月，看她读书、写信，从书页间发现藏品。',
      introduction: '月升时，溟月会坐在窗边读书，也会在贝壳灯下写信。可以陪她在书房散步，请她从书架取一本书，翻看夹页里的藏品，听听它的来历。作品免费体验，角色模型采用 CC BY-NC-SA 4.0 许可。',
      controls: '桌面：点击地板散步，拖动环顾，滚轮缩放 · 手机：轻触地板散步，双指缩放',
      actionLabel: '进入 3D 书房体验'
    },
    {
      id: 'jiangshan-map',
      title: '江山时序',
      category: '工具',
      author: 'DeanChensj',
      avatarBg: '#A8640C',
      date: '2026-11-18',
      likes: 243,
      commentsCount: 31,
      cover: './assets/jiangshan-map.webp',
      tall: false,
      description: '沿时间轴看历代疆域，查古今地名，跟着历史人物走一程。',
      introduction: '从公元前 403 年到 1948 年，拖动时间轴查看历代疆域，点击古城了解沿革，也可以搜索苏轼、李白等人物，沿地图回看他们的行迹。支持离线使用与桌面端导入。',
      controls: '拖动时间轴，滚轮缩放，点击古城查看沿革',
      actionLabel: '打开江山时序'
    },
    {
      id: 'micro-gallery',
      title: '微观展厅 · 龙鱼水族箱',
      category: '展厅',
      author: 'RealWorldBuilder',
      avatarBg: '#07C160',
      date: '2026-11-15',
      likes: 164,
      commentsCount: 19,
      cover: './assets/micro-gallery.webp',
      tall: true,
      description: '看朱红龙鱼与孔雀鱼共游，轻触水面投喂，探索更多 3D 展品。',
      introduction: '社区常驻 3D 微观展厅。水族箱默认展示朱红龙鱼和砂石水草，支持切换蓝色龙鱼与孔雀鱼组合，播放时轻按水面即可投喂。',
      controls: '点按水面投喂，拖动视角环顾，设置内切换鱼群',
      actionLabel: '进入微观展厅'
    },
    {
      id: 'plush-rabbit',
      title: '碎花垂耳兔',
      category: '3D 模型',
      author: 'RealWorldBuilder',
      avatarBg: '#D93A44',
      date: '2026-11-15',
      likes: 129,
      commentsCount: 14,
      cover: './assets/plush-rabbit.webp',
      tall: false,
      description: '转动 3D 模型，近看绒毛、碎花布料与头顶蝴蝶结纹理。',
      introduction: '奶白色的垂耳兔，内耳和头顶的蝴蝶结用了同样的碎花布。打开模型转一圈，可以近看耳朵的弧度和绒毛材质表现。',
      controls: '拖动旋转模型，滚轮 / 双指缩放细节',
      actionLabel: '体验 3D 模型'
    },
    {
      id: 'starry-nightrider',
      title: '星夜骑士：Starry Nightrider',
      category: '游戏',
      author: '@ebbyamir',
      avatarBg: '#7152D8',
      date: '2026-11-15',
      likes: 215,
      commentsCount: 28,
      cover: './assets/starry-nightrider.webp',
      tall: true,
      description: '骑车穿过梵高《星夜》里的村庄，把收集到的光球送进教堂尖塔。',
      introduction: '在油画笔触构成的小镇夜色中穿梭，收集散落星光并冲上山坡，支持键盘方向键与触控操作，内置本地排行榜。',
      controls: '方向键 / WASD 控制骑行与跳跃，触屏支持虚拟按键',
      actionLabel: '试玩星夜骑士'
    },
    {
      id: 'little-prince',
      title: '小王子：星辰之旅',
      category: '游戏',
      author: 'Sac',
      avatarBg: '#2F6FDB',
      date: '2026-11-15',
      likes: 198,
      commentsCount: 22,
      cover: './assets/little-prince.webp',
      tall: false,
      description: '从 B-612 小行星出发，踏上星球间的柔和 3D 旅程。',
      introduction: '在低重力微型星球表面漫步，照料玫瑰花并飞往相邻小行星，体验轻盈治愈的交互叙事。',
      controls: 'WASD 移动，鼠标拖动调整视角',
      actionLabel: '前往作品官网'
    },
    {
      id: 'saltline',
      title: '运输船：Saltline',
      category: '游戏',
      author: 'Felix Rieseberg',
      avatarBg: '#2B2B30',
      date: '2026-11-15',
      likes: 142,
      commentsCount: 16,
      cover: './assets/saltline.webp',
      tall: true,
      description: '潜入雨夜货港，切断信号中继站后乘船撤离。',
      introduction: '冷雨与探照灯交织的潜行小品，利用集装箱阴影躲避巡逻艇，单文件即可运行于小伴桌面与网页沙箱。',
      controls: '方向键潜行移动，空格键互动与切断电源',
      actionLabel: '启动 Saltline'
    },
    {
      id: 'oil-paint',
      title: 'Oil Paint · 实时油画漫步',
      category: '游戏',
      author: '@BlendiByl',
      avatarBg: '#D93A44',
      date: '2026-11-15',
      likes: 176,
      commentsCount: 20,
      cover: './assets/oil-paint.webp',
      tall: false,
      description: '在实时笔触绘成的 3D 油画里漫步，探索秋林与极光下的雪脊。',
      introduction: '基于 WebGL2 与 Rapier 物理引擎构建的后处理笔触世界，每一帧画面都像流动的厚涂油画。',
      controls: 'WASD 漫步，鼠标拖动转动视角',
      actionLabel: '进入画中世界'
    }
  ];

  const FRONTIER_ITEMS = [
    {
      key: 'whale-girl-release',
      title: '《潮汐书房 · 鲸鱼娘》上线：把可散步的 3D 书房装进小伴桌面与社区',
      source: '前沿编辑部',
      date: '11-19',
      cover: './assets/whale-girl.webp',
      summary: '月光书架、可翻阅的藏品夹页与跨端 V2 包同步机制详解，附模型许可与桌面导入指南。'
    },
    {
      key: 'jiangshan-timeline',
      title: '《江山时序》离线版收录：从战国到近现代的交互式历史地理时间轴',
      source: '开源创作观察',
      date: '11-18',
      cover: './assets/jiangshan-map.webp',
      summary: '如何将大型历史疆域地图裁剪为纯离线可读的轻量化互动应用包。'
    },
    {
      key: 'local-first-canvas',
      title: '本地优先白板与知识库工具潮：Excalidraw 与 AFFiNE 的协同设计拆解',
      source: '工具与设计',
      date: '11-17',
      cover: './assets/excalidraw-showcase.webp',
      summary: '为什么越来越多的个人创作者重新选择把文件、画布和记录留在本地。'
    }
  ];

  const DAILY_NEWS = [
    {
      key: 'daily-1119',
      title: 'Claude 与 DeepSeek 编程 Agent 实测对比：长仓库重构谁更稳？',
      summary: '综合 40 个真实开源修复任务，对比多轮工具调用、测试自愈率与单次成本。',
      source: 'AI 日报',
      date: '今天 09:00',
      thumb: './assets/excalidraw-showcase.webp'
    },
    {
      key: 'daily-1118',
      title: '网页与桌面统一采用运行时协议 2：支持 100MB 互动应用与改编溯源',
      summary: '创作者发布应用或游戏时可勾选允许改编，社区自动保留原作者署名与版本谱系。',
      source: '站方动态',
      date: '昨天 18:30',
      thumb: './assets/whale-girl.webp'
    },
    {
      key: 'daily-1117',
      title: 'OpenRouter 周调用量观察：低延迟推理模型份额持续上升',
      summary: '编码辅助与桌面实时对话场景带动轻量快速模型成为日常主力。',
      source: '数据速览',
      date: '11-17',
      thumb: ''
    },
    {
      key: 'daily-1116',
      title: 'Blender 几何节点新工作流：用参数化方式批量生成桌面微景道具',
      summary: '社区 3D 作者分享轻量化烘焙与 glTF 压缩经验，单个场景控制在 6MB 内。',
      source: '3D 与渲染',
      date: '11-16',
      thumb: './assets/blender-screen.webp'
    }
  ];

  const HOT_REPOS_PREVIEW = [
    { rank: 1, name: 'excalidraw/excalidraw', desc: '手绘风格协同白板', stars: '86.4k', gain: '+1,420' },
    { rank: 2, name: 'toeverything/AFFiNE', desc: '文档与白板一体工作区', stars: '44.2k', gain: '+980' },
    { rank: 3, name: 'codecrafters-io/build-your-own-x', desc: '从零亲手实现经典技术栈', stars: '318k', gain: '+890' }
  ];

  const OPEN_SOURCE_PROJECTS = [
    {
      full_name: 'excalidraw/excalidraw',
      name: 'Excalidraw',
      category: '设计创作',
      stars: '86.4k',
      cover: './assets/excalidraw-showcase.webp',
      icon: './assets/excalidraw-icon.webp',
      summary: '手绘风格虚拟白板，支持端到端加密协作、图表绘制与丰富图形库。',
      tags: ['白板协作', '手绘矢量']
    },
    {
      full_name: 'blender/blender',
      name: 'Blender',
      category: '设计创作',
      stars: '13.5k',
      cover: './assets/blender-screen.webp',
      icon: '',
      summary: '自由开源的全流程 3D 创作套件，覆盖建模、雕刻、绑定、动画与实时渲染。',
      tags: ['3D 套件', '渲染']
    },
    {
      full_name: 'toeverything/AFFiNE',
      name: 'AFFiNE',
      category: '办公效率',
      stars: '44.2k',
      cover: './assets/affine-writing.webp',
      icon: '',
      summary: '融合块级文档、无限画布白板与多维表格的新一代本地优先知识工作区。',
      tags: ['本地优先', '知识库']
    },
    {
      full_name: 'AppFlowy-IO/AppFlowy',
      name: 'AppFlowy',
      category: '办公效率',
      stars: '59.8k',
      cover: './assets/appflowy.webp',
      icon: './assets/appflowy-icon.webp',
      summary: '开源且数据完全归属自己的 Notion 替代方案，内置 AI 写作与看板。',
      tags: ['笔记文档', 'Rust']
    },
    {
      full_name: 'FreeCAD/FreeCAD',
      name: 'FreeCAD',
      category: '开发工具',
      stars: '21.7k',
      cover: './assets/freecad-screen.webp',
      icon: './assets/freecad-icon.svg',
      summary: '开源参数化 3D CAD 建模器，适合机械工程、结构零件与开源硬件设计。',
      tags: ['参数化 CAD', '硬件']
    },
    {
      full_name: 'godotengine/godot',
      name: 'Godot Engine',
      category: '开发工具',
      stars: '91.2k',
      cover: '',
      icon: './assets/godot-icon.svg',
      summary: '轻量级跨平台 2D 与 3D 开源游戏引擎，社区不少 Web 互动小品基于它导出。',
      tags: ['游戏引擎', '跨平台']
    },
    {
      full_name: 'withastro/astro',
      name: 'Astro',
      category: '开发工具',
      stars: '47.6k',
      cover: '',
      icon: './assets/astro-icon.svg',
      summary: '以内容加载速度为核心的现代 Web 框架，支持孤岛架构与零默认脚本。',
      tags: ['Web 框架', '内容站']
    },
    {
      full_name: 'coollabsio/coolify',
      name: 'Coolify',
      category: '开发工具',
      stars: '35.1k',
      cover: '',
      icon: './assets/coolify-icon.svg',
      summary: '开源可自托管的 PaaS 部署平台，在自有云服务器上轻松托管应用与数据库。',
      tags: ['自托管', '运维部署']
    },
    {
      full_name: 'codecrafters-io/build-your-own-x',
      name: 'Build Your Own X',
      category: '教程资源',
      stars: '318k',
      cover: '',
      icon: '',
      summary: '按技术领域整理从零编写 Git、数据库、编译器、渲染器等核心项目的实战指南。',
      tags: ['从零实现', '教程目录']
    }
  ];

  const RANK_MODELS = [
    { id: 'claude-opus', name: 'Claude 4.1 Opus', vendor: 'Anthropic', color: '#D97757', intelligence: 74, price: 24.0, speed: 62 },
    { id: 'gemini-pro', name: 'Gemini 2.5 Pro', vendor: 'Google', color: '#4285F4', intelligence: 71, price: 3.1, speed: 118 },
    { id: 'deepseek-r1', name: 'DeepSeek R1', vendor: 'DeepSeek', color: '#4D6BFE', intelligence: 68, price: 0.96, speed: 78 },
    { id: 'qwen-max', name: 'Qwen 3 Max', vendor: 'Qwen', color: '#615CED', intelligence: 66, price: 1.4, speed: 94 },
    { id: 'kimi-k2', name: 'Kimi K2', vendor: 'Moonshot', color: '#2F6BFF', intelligence: 65, price: 1.1, speed: 105 },
    { id: 'gemini-flash', name: 'Gemini 2.5 Flash', vendor: 'Google', color: '#4285F4', intelligence: 61, price: 0.24, speed: 245 },
    { id: 'deepseek-v3', name: 'DeepSeek V3', vendor: 'DeepSeek', color: '#4D6BFE', intelligence: 60, price: 0.48, speed: 142 },
    { id: 'glm-45', name: 'GLM-4.5 Air', vendor: '智谱 AI', color: '#5366D3', intelligence: 57, price: 0.32, speed: 186 }
  ];

  const USAGE_TOP5 = [
    { rank: 1, name: 'Claude Sonnet 4', share: '24.6%', delta: '—' },
    { rank: 2, name: 'Gemini 2.5 Flash', share: '18.2%', delta: '↑ 1' },
    { rank: 3, name: 'DeepSeek V3', share: '15.9%', delta: '↓ 1' },
    { rank: 4, name: 'Qwen 3 Coder', share: '9.4%', delta: '↑ 2' },
    { rank: 5, name: 'Kimi K2', share: '6.8%', delta: '新入榜' }
  ];

  const NAV_ENTRIES = [
    { key: 'claude', name: 'Claude', vendor: 'Anthropic', cat: 'models', filter: '海外', type: '对话与研究', domain: 'claude.ai', icon: './assets/vendor-claude.svg', summary: '擅长长上下文代码阅读、界面原型构思与细致写作。' },
    { key: 'deepseek', name: 'DeepSeek', vendor: '深度求索', cat: 'models', filter: '国产', type: '推理模型', domain: 'chat.deepseek.com', icon: './assets/vendor-deepseek.svg', summary: '开源高性价比推理与编码大模型官方体验入口。' },
    { key: 'gemini', name: 'Gemini', vendor: 'Google', cat: 'models', filter: '海外', type: '多模态', domain: 'gemini.google.com', icon: './assets/vendor-gemini.svg', summary: '原生支持文本、图片、视频理解与长文档分析。' },
    { key: 'qwen', name: '通义千问 Qwen', vendor: '阿里云', cat: 'models', filter: '国产', type: '开源主力', domain: 'chat.qwen.ai', icon: './assets/vendor-qwen.svg', summary: '涵盖通用对话、代码专精与多尺寸开源权重模型。' },
    { key: 'kimi', name: 'Kimi', vendor: '月之暗面', cat: 'models', filter: '国产', type: '长文本', domain: 'kimi.com', icon: './assets/vendor-kimi.svg', summary: '长文件研读、深度搜索与 Agent 模式入口。' },
    { key: 'zhipu', name: '智谱清言 GLM', vendor: '智谱 AI', cat: 'models', filter: '国产', type: '全能助手', domain: 'chatglm.cn', icon: './assets/vendor-zhipu.svg', summary: '国内常用综合助手，支持代码、推理与工具调用。' },
    { key: 'minimax', name: 'MiniMax', vendor: 'MiniMax', cat: 'models', filter: '国产', type: '语音与编码', domain: 'minimaxi.com', icon: './assets/vendor-minimax.svg', summary: '兼具高速编程模型与自然语音合成的多模态平台。' },
    { key: 'doubao', name: '豆包', vendor: '字节跳动', cat: 'models', filter: '国产', type: '日常助手', domain: 'doubao.com', icon: './assets/vendor-doubao.svg', summary: '桌面与移动端轻量协作、图像生成与资料问答。' },
    { key: 'cursor', name: 'Cursor', vendor: 'Anysphere', cat: 'harness', filter: '编程 Agent', type: 'AI IDE', domain: 'cursor.com', icon: './assets/vendor-cursor.svg', summary: '面向全项目上下文的智能编程编辑器与代码代理。' },
    { key: 'skillhub', name: 'SkillHub', vendor: '腾讯', cat: 'skills', filter: '技能市场', type: '技能市场', domain: 'skillhub.cn', icon: '', summary: '面向中文用户的 Skills 社区，收录常用工作流与桌面技能包。' },
    { key: 'modelscope-skills', name: '魔搭 Skills', vendor: 'ModelScope', cat: 'skills', filter: '技能市场', type: '技能市场', domain: 'modelscope.cn/skills', icon: '', summary: '发现与使用社区智能体技能，一键接入本地 Agent。' },
    { key: 'skills-sh', name: 'skills.sh', vendor: 'Vercel', cat: 'skills', filter: '官方技能库', type: '技能目录', domain: 'skills.sh', icon: '', summary: 'Vercel 与开源社区维护的通用 Agent 技能目录与命令行工具。' },
    { key: 'clawhub', name: 'ClawHub', vendor: 'OpenClaw', cat: 'skills', filter: '官方技能库', type: '注册中心', domain: 'clawhub.ai', icon: '', summary: 'OpenClaw 技能与插件注册中心，支持版本追踪与本地同步。' },
    { key: 'groq', name: 'GroqCloud', vendor: 'Groq', cat: 'api', filter: '聚合与推理', type: '超低延迟 API', domain: 'console.groq.com', icon: './assets/vendor-groq.svg', summary: '基于 LPU 的极速开源大模型推理 API 服务。' },
    { key: 'bedrock', name: 'AWS Bedrock', vendor: 'Amazon Web Services', cat: 'api', filter: '云平台', type: '企业云托管', domain: 'aws.amazon.com/bedrock', icon: './assets/vendor-aws.svg', summary: '提供主流基础模型统一托管 API 与安全合规控制。' },
    { key: 'fireworks', name: 'Fireworks AI', vendor: 'Fireworks', cat: 'api', filter: '聚合与推理', type: '高性能推理', domain: 'fireworks.ai', icon: './assets/vendor-fireworks.svg', summary: '支持开源模型快速部署、微调与高吞吐推理。' }
  ];

  const DISCUSS_TOPICS = [
    {
      id: 109,
      title: 'kiro白嫖一个月pro，能用claude opus 5.5！',
      excerpt: 'Free 账号第一次升级 Pro（原价 $20/月），当月剩下的天数直接 $0 免单，还额外多送 500 Trial Credits（一年有效），记得月初 1 号开最划算。',
      category: '羊毛',
      tabs: ['casual-deals'],
      tone: 'deal',
      tag: 'Kiro',
      author: '小伴编辑',
      avatarBg: '#A8640C',
      time: '刚刚',
      replies: 6,
      likes: 18,
      cover: './assets/deal-kiro-cover.jpg',
      images: ['./assets/deal-kiro-cover.jpg', './assets/deal-kiro-real.jpg']
    },
    {
      id: 110,
      title: 'AWS Quick, 免绑卡白用30天opus5.5',
      excerpt: '不用 AWS 云账号也不用绑信用卡，邮箱或 Google 登录就免费用 30 天 Plus，里面能直接选 Claude Opus 5.5；有亚马逊卖家主账号的年底前能领一年。',
      category: '羊毛',
      tabs: ['casual-deals'],
      tone: 'deal',
      tag: 'AWS',
      author: '小伴编辑',
      avatarBg: '#A8640C',
      time: '5 分钟前',
      replies: 4,
      likes: 12,
      cover: './assets/deal-aws-cover.jpg',
      images: ['./assets/deal-aws-cover.jpg', './assets/deal-aws-real.jpg']
    },
    {
      id: 101,
      title: '做小伴桌面小组件时，先搭布局还是先调动画曲线？',
      excerpt: '这周尝试把番茄钟和天气卡片做成 2x2 小组件，发现桌面透明底色下阴影参数在深浅壁纸里表现差异很大，大家一般怎么定基准？',
      category: '教程',
      tabs: ['learn'],
      tone: 'learn',
      tag: '桌面组件',
      author: '阿栗',
      avatarBg: '#07C160',
      time: '2 小时前',
      replies: 18,
      likes: 34
    },
    {
      id: 102,
      title: '《潮汐书房》窗口光影质感是怎么压到这么轻量的？',
      excerpt: '拆开看了一下资源包体积非常克制，木地板和水面反射却很有氛围，很好奇烘焙贴图和实时光源是怎么分工的。',
      category: '3D',
      tabs: ['3d', 'design'],
      tone: 'design',
      tag: '3D 质感',
      author: '临风',
      avatarBg: '#7152D8',
      time: '3 小时前',
      replies: 12,
      likes: 27,
      cover: './assets/micro-gallery.webp',
      images: ['./assets/micro-gallery.webp']
    },
    {
      id: 103,
      title: '分享我用单个 HTML 打包进应用广场的小工具工作流',
      excerpt: '写好纯前端小页面后，在「发布 → 分享已有作品」直接选单个 UTF-8 HTML，网页会自动转成 V2 ZIP 包，桌面和手机都能直接导入。',
      category: '副业',
      tabs: ['casual-indie'],
      tone: 'dev',
      tag: 'V2 应用包',
      author: '半杯茶',
      avatarBg: '#2F6FDB',
      time: '5 小时前',
      replies: 23,
      likes: 49
    },
    {
      id: 104,
      title: '大家在桌面养桌面伙伴时，平时喜欢开几个实例、放什么位置？',
      excerpt: '我习惯右上角放一个绿叶盆栽看视线跟随，左下角留给系统监控小组件，你们有什么不遮挡 IDE 的摆法吗？',
      category: '闲聊灌水',
      tabs: ['casual-chat'],
      tone: 'game',
      tag: '桌面布置',
      author: '木杉',
      avatarBg: '#D93A44',
      time: '昨天',
      replies: 31,
      likes: 41
    },
    {
      id: 105,
      title: '《星夜骑士》第二坡道的加速光球怎么一次连跳吃到？',
      excerpt: '卡在 1,840 分两天了，每次过教堂弯道都会少一颗光球，求大佬分享按键节奏。',
      category: '游戏',
      tabs: ['casual-games'],
      tone: 'game',
      tag: '攻略讨论',
      author: '星图',
      avatarBg: '#A8640C',
      time: '昨天',
      replies: 9,
      likes: 15
    },
    {
      id: 106,
      title: '超级小伴社区板块与发帖规范说明（2026-11 更新）',
      excerpt: '整理了创意作品发布、开源项目转载署名及求助悬赏结算的常见规则说明，欢迎新同学在发帖前查阅并提出改进建议。',
      category: '站务',
      tabs: ['site'],
      tone: 'dev',
      tag: '站方公告',
      author: '站务君',
      avatarBg: '#16181f',
      time: '2 天前',
      replies: 14,
      likes: 56
    },
    {
      id: 107,
      title: 'Mac mini M4 作为桌面小伴常驻工作站的静音与外设方案分享',
      excerpt: '双 4K 显示器配一根雷雳一线通，小伴壁纸和语音助手长开一星期风扇几乎零转速，顺手记录一下功耗与键鼠切换细节。',
      category: '数码装备',
      tabs: ['casual-gear'],
      tone: 'design',
      tag: '桌面硬件',
      author: 'Kite',
      avatarBg: '#2F6FDB',
      time: '3 天前',
      replies: 19,
      likes: 38,
      cover: './assets/blender-screen.webp',
      images: ['./assets/blender-screen.webp']
    },
    {
      id: 108,
      title: '桌面端多显示器切换分辨率后，小组件坐标记忆的一点观察',
      excerpt: '外接显示器插拔瞬间小组件会自动吸附回安全工作区，不会跑出屏幕外丢失窗口，顺便问下大家常用哪几个边距参数？',
      category: '默认',
      tabs: ['discuss-uncategorized'],
      tone: 'learn',
      tag: '使用体验',
      author: '晴窗',
      avatarBg: '#07C160',
      time: '3 天前',
      replies: 6,
      likes: 11
    }
  ];

  const RESOURCE_SOFTWARE = [
    {
      slug: 'openclaw/openclaw',
      name: 'openclaw',
      ver: 'v2026.11.5',
      cat: 'AI',
      lang: 'TypeScript',
      langColor: '#3178c6',
      stars: '394.8K',
      starsNum: 394800,
      starGain: 4820,
      updated: '更新于 1 天前',
      updatedDays: 1,
      platforms: ['macOS', 'Linux'],
      glyph: 'OC',
      iconBg: 'linear-gradient(135deg, #ff5858, #d93a44)',
      iconColor: '#ffffff',
      desc: '真正做事的人工智能助手，覆盖桌面、终端与本地工作区自动化任务。'
    },
    {
      slug: 'anomalyco/opencode',
      name: 'opencode',
      ver: 'v1.19.6',
      cat: '开发',
      lang: 'TypeScript',
      langColor: '#3178c6',
      stars: '227.5K',
      starsNum: 227500,
      starGain: 3410,
      updated: '更新于 2 天前',
      updatedDays: 2,
      platforms: ['Windows', 'macOS', 'Linux'],
      glyph: '>_',
      iconBg: '#16181f',
      iconColor: '#ffffff',
      desc: '一个开源 AI 编程助手，可以在终端里理解代码、修改文件和执行开发任务。'
    },
    {
      slug: 'yt-dlp/yt-dlp',
      name: 'yt-dlp',
      ver: '2026.10.26',
      cat: '影音',
      lang: 'Python',
      langColor: '#3572A5',
      stars: '208.9K',
      starsNum: 208900,
      starGain: 1960,
      updated: '更新于 2 天前',
      updatedDays: 2,
      platforms: ['Windows', 'macOS', 'Linux'],
      glyph: 'YT',
      iconBg: '#fff1f2',
      iconColor: '#e11d48',
      desc: '功能丰富的命令行音频与视频下载器，支持多轨封装与字幕提取。'
    },
    {
      slug: 'ollama/ollama',
      name: 'Ollama',
      ver: 'v0.40.16',
      cat: 'AI',
      lang: 'Go',
      langColor: '#00ADD8',
      stars: '187.7K',
      starsNum: 187700,
      starGain: 2890,
      updated: '今天更新',
      updatedDays: 0,
      platforms: ['Windows', 'macOS', 'Linux'],
      glyph: 'O',
      iconBg: '#f3f4f6',
      iconColor: '#111827',
      desc: '在本地一键运行和管理大语言模型，提供标准化 API 与轻量运行时。'
    },
    {
      slug: 'clash-verge-rev/clash-verge-rev',
      name: 'clash-verge-rev',
      ver: 'v2.5.10',
      cat: '网络',
      lang: 'Rust',
      langColor: '#dea584',
      stars: '159.4K',
      starsNum: 159400,
      starGain: 2350,
      updated: '今天更新',
      updatedDays: 0,
      platforms: ['Windows', 'macOS', 'Linux'],
      glyph: 'CV',
      iconBg: 'linear-gradient(135deg, #c084fc, #7e22ce)',
      iconColor: '#ffffff',
      desc: '基于 Tauri 的现代 GUI 客户端，支持 Windows、macOS 与 Linux 多平台。'
    },
    {
      slug: 'Genymobile/scrcpy',
      name: 'scrcpy',
      ver: 'v5.0.1',
      cat: '工具',
      lang: 'C',
      langColor: '#555555',
      stars: '155.2K',
      starsNum: 155200,
      starGain: 1520,
      updated: '今天更新',
      updatedDays: 0,
      platforms: ['Windows', 'macOS', 'Linux'],
      glyph: 'SC',
      iconBg: 'linear-gradient(135deg, #4ade80, #16a34a)',
      iconColor: '#ffffff',
      desc: '通过 USB 或无线低延迟投屏并控制 Android 移动设备，免手机端安装。'
    },
    {
      slug: 'microsoft/PowerToys',
      name: 'PowerToys',
      ver: 'v0.102.3',
      cat: '其它',
      lang: 'C#',
      langColor: '#178600',
      stars: '141.2K',
      starsNum: 141200,
      starGain: 1680,
      updated: '今天更新',
      updatedDays: 0,
      platforms: ['Windows'],
      glyph: 'PT',
      iconBg: '#eff6ff',
      iconColor: '#0078d4',
      desc: '微软开源桌面效率增强套件，提供窗口分屏布局、批量重命名与快捷取色。'
    },
    {
      slug: 'rustdesk/rustdesk',
      name: 'RustDesk',
      ver: '1.5.3',
      cat: '工具',
      lang: 'Rust',
      langColor: '#dea584',
      stars: '130.3K',
      starsNum: 130300,
      starGain: 1310,
      updated: '更新于 1 天前',
      updatedDays: 1,
      platforms: ['Windows', 'macOS', 'Linux', 'Android'],
      glyph: 'RD',
      iconBg: '#eff6ff',
      iconColor: '#2563eb',
      desc: '开源跨平台远程桌面客户端及自建中继服务套件，开箱即用。'
    },
    {
      slug: 'godotengine/godot',
      name: 'Godot Engine',
      ver: 'v4.4.1',
      cat: '游戏',
      lang: 'C++',
      langColor: '#f34b7d',
      stars: '91.2K',
      starsNum: 91200,
      starGain: 1120,
      updated: '更新于 2 天前',
      updatedDays: 2,
      platforms: ['Windows', 'macOS', 'Linux', 'Android'],
      icon: './assets/godot-icon.svg',
      desc: '轻量级跨平台 2D 与 3D 开源游戏引擎，社区不少 Web 互动作品基于它导出。'
    },
    {
      slug: 'excalidraw/excalidraw',
      name: 'Excalidraw',
      ver: 'v0.18.0',
      cat: '效率',
      lang: 'TypeScript',
      langColor: '#3178c6',
      stars: '86.4K',
      starsNum: 86400,
      starGain: 1420,
      updated: '今天更新',
      updatedDays: 0,
      platforms: ['Windows', 'macOS', 'Linux'],
      icon: './assets/excalidraw-icon.webp',
      desc: '轻量手绘矢量白板，支持端到端加密协作并导出 SVG / PNG 图形。'
    }
  ];

  const HELP_BOUNTIES = [
    { id: 301, title: '求一份适合 macOS 浅色毛玻璃桌面的 4K 极简山丘动态壁纸循环素材', bounty: 80, status: '进行中', expires: '11-26 截止', answers: 4, author: '晴窗' },
    { id: 302, title: '如何把 Three.js 场景导出成支持手机触摸双指缩放的协议 2 应用包？', bounty: 120, status: '进行中', expires: '11-29 截止', answers: 7, author: 'Kite' },
    { id: 303, title: '求推荐免费可商用的中文像素风格点阵字体（需包含常见标点）', bounty: 30, status: '已采纳', expires: '已结束', answers: 5, author: '阿墨' }
  ];

  const RESOURCE_UNCATEGORIZED = [
    { id: 401, title: '小伴桌面常用透明微件参考配色与圆角规范表（浅色 / 深色）', tag: '默认 · 设计素材', time: '今天 10:15', author: '晴窗', replies: 8 },
    { id: 402, title: '开源中文字体授权清单整理：思源黑体、霞鹜文楷与像素点阵字体合集', tag: '默认 · 字体资料', time: '昨天 21:40', author: '阿墨', replies: 15 },
    { id: 403, title: 'Windows 与 macOS 本地 AI 运行环境显存占用速查对照表', tag: '默认 · 工具对照', time: '11-17', author: '半杯茶', replies: 11 }
  ];

  const PLAZA_GAMES = [
    { id: 'dusk-glider', title: '暮色滑翔', genre: '冒险', icon: './assets/app-dusk-glider.svg' },
    { id: 'ink-blade', title: '水墨剑客', genre: '冒险', icon: './assets/app-ink-blade.svg' },
    { id: 'block-puzzle', title: '方块爆破', genre: '益智', icon: './assets/app-block-puzzle.webp' },
    { id: 'pro-cube', title: '魔方空间', genre: '益智', icon: './assets/app-pro-cube.webp' },
    { id: 'jade-gomoku', title: '青玉五子棋', genre: '棋牌', icon: './assets/app-jade-gomoku.webp' },
    { id: 'water-sort', title: '彩虹分水', genre: '益智', icon: './assets/app-water-sort.webp' },
    { id: 'merge-2048', title: '合成 2048', genre: '益智', icon: './assets/app-merge-2048.webp' }
  ];

  const PLAZA_TOOLS = [
    { id: 'mbti', title: 'MBTI 性格测评', genre: '测评', icon: './assets/app-mbti.webp' },
    { id: 'remote-control', title: '遥控器', genre: '工具', icon: './assets/app-remote-control.webp' },
    { id: 'quick-launch', title: '轻启', genre: '效率', icon: './assets/app-quick-launch.webp' },
    { id: 'live-wallpapers', title: '动态壁纸', genre: '桌面', icon: './assets/app-live-wallpapers.webp' },
    { id: 'wancisou', title: '万磁搜', genre: '检索', icon: './assets/app-wancisou.webp' }
  ];

  const PLAZA_WORKS = [
    {
      id: 'wancisou',
      title: '万磁搜',
      kind: '工具',
      category: '精选',
      author: 'read',
      avatarBg: '#D9A726',
      date: '2026-11-19',
      views: 24,
      commentsCount: 1,
      likes: 18,
      featured: true,
      isWancisou: true,
      cover: './assets/app-wancisou.webp',
      introduction: '轻量聚合公开资源检索工具，输入关键词即可按体积与热度筛选结果并一键复制链接。',
      controls: '输入关键词回车检索，点击胶囊切换标签筛选',
      actionLabel: '立即运行工具'
    },
    {
      id: 'starry-nightrider',
      title: '星夜骑士：Starry Nightrider',
      kind: '游戏',
      category: '游戏',
      author: '@ebbyamir',
      avatarBg: '#7152D8',
      date: '2026-11-18',
      views: 186,
      commentsCount: 28,
      likes: 215,
      featured: false,
      cover: './assets/starry-nightrider.webp',
      introduction: '在油画笔触构成的小镇夜色中穿梭，收集散落星光并冲上山坡。',
      controls: '方向键 / 触屏控制骑行与跳跃',
      actionLabel: '试玩星夜骑士'
    },
    {
      id: 'little-prince',
      title: '小王子：星辰之旅',
      kind: '游戏',
      category: '游戏',
      author: 'Sac',
      avatarBg: '#2F6FDB',
      date: '2026-11-17',
      views: 152,
      commentsCount: 22,
      likes: 198,
      featured: false,
      cover: './assets/little-prince.webp',
      introduction: '从 B-612 小行星出发，在微型星球间漫步的柔和 3D 旅程。',
      controls: 'WASD 移动，拖动视角',
      actionLabel: '进入星辰之旅'
    },
    {
      id: 'oil-paint',
      title: '油画水彩画板工坊',
      kind: '工具',
      category: '工具',
      author: '林澈',
      avatarBg: '#07C160',
      date: '2026-11-16',
      views: 94,
      commentsCount: 11,
      likes: 88,
      featured: false,
      cover: './assets/oil-paint.webp',
      introduction: '支持湿画晕染与冷暖调色盘的浏览器轻量绘画工具，单文件即可启动。',
      controls: '画笔大小与湿度滑块调节，支持压感',
      actionLabel: '打开画板工具'
    }
  ];

  const INITIAL_COMMENTS = [
    { id: 1, author: '阿栗', avatarBg: '#07C160', time: '28 分钟前', region: '上海', likes: 42, text: '页面信息层级比之前清晰好多，特别是图文卡片和纯文字讨论终于各自有合适的呼吸感了。' },
    { id: 2, author: '半杯茶', avatarBg: '#2F6FDB', time: '1 小时前', region: '北京', likes: 27, text: '桌面端把底部手机导航拿掉放回左侧以后，宽屏看作品和大图再也不会觉得憋屈了。' },
    { id: 3, author: '临风', avatarBg: '#7152D8', time: '3 小时前', region: '杭州', likes: 19, text: '评论区吸底栏加上弹出写评论面板在手机上特别顺手，点赞和收藏随手就能点。' }
  ];

  /* ==========================================================================
     内联 SVG 图标集（零外部依赖）
     ========================================================================== */
  const ICON_PATHS = {
    home: ['M3 10.5 12 3l9 7.5V20a1 1 0 0 1-1 1h-5v-6H9v6H4a1 1 0 0 1-1-1Z'],
    compass: ['M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20Z', 'm16.2 7.8-2.1 6.3-6.3 2.1 2.1-6.3Z'],
    chat: ['M21 11.5a8.5 8.5 0 0 1-8.5 8.5H4l1.6-4.3A8.5 8.5 0 1 1 21 11.5Z'],
    folder: ['M3 6h6l2 3h10v11H3Z'],
    grid: ['M4 4h6v6H4Z', 'M14 4h6v6h-6Z', 'M4 14h6v6H4Z', 'M14 14h6v6h-6Z'],
    spark: ['m12 3 2.2 6.3L20.5 11.5l-6.3 2.2L12 20l-2.2-6.3L3.5 11.5l6.3-2.2Z'],
    me: ['M16 8a4 4 0 1 1-8 0 4 4 0 0 1 8 0Z', 'M4 21v-2a8 8 0 0 1 16 0v2'],
    plus: ['M12 5v14', 'M5 12h14'],
    search: ['M11 18a7 7 0 1 1 0-14 7 7 0 0 1 0 14Z', 'm16.2 16.2 4.3 4.3'],
    bell: ['M6 17v-6a6 6 0 0 1 12 0v6l2 2H4Z', 'M10 21h4'],
    sun: ['M12 16a4 4 0 1 0 0-8 4 4 0 0 0 0 8Z', 'M12 2v2', 'M12 20v2', 'M2 12h2', 'M20 12h2', 'm4.9 4.9 1.4 1.4', 'm17.7 17.7 1.4 1.4', 'm4.9 19.1 1.4-1.4', 'm17.7 6.3 1.4-1.4'],
    moon: ['M20.5 14.5A8.5 8.5 0 0 1 9.5 3.5a9 9 0 1 0 11 11Z'],
    heart: ['M20.8 4.6a5.5 5.5 0 0 0-7.8 0L12 5.7l-1.1-1.1a5.5 5.5 0 0 0-7.8 7.8L12 21l8.8-8.6a5.5 5.5 0 0 0 0-7.8Z'],
    star: ['m12 3.2 2.7 5.5 6.1.9-4.4 4.3 1 6.1-5.4-2.9-5.4 2.9 1-6.1-4.4-4.3 6.1-.9Z'],
    share: ['M4 12v7a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-7', 'm16 6-4-4-4 4', 'M12 2v13'],
    image: ['M5 5h14a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V7a2 2 0 0 1 2-2Z', 'm3 16 5-5 4 4 3-3 6 6'],
    smile: ['M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18Z', 'M8.5 14.5c1 1.3 2.2 2 3.5 2s2.5-.7 3.5-2', 'M9 9.5h.01', 'M15 9.5h.01'],
    at: ['M12 16a4 4 0 1 0 0-8 4 4 0 0 0 0 8Z', 'M16 8v5a2.5 2.5 0 0 0 5 0v-1a9 9 0 1 0-4 7.5'],
    edit: ['M12 20h9', 'M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4Z'],
    sort: ['M4 7h16', 'M7 12h10', 'M10 17h4'],
    back: ['m15 5-7 7 7 7'],
    arrow: ['m9 5 7 7-7 7'],
    external: ['M15 3h6v6', 'M10 14 21 3', 'M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6'],
    download: ['M12 3v12', 'm7 10 5 5 5-5', 'M4 17v3h16v-3'],
    trash: ['M3 6h18', 'M8 6V4a1 1 0 0 1 1-1h6a1 1 0 0 1 1 1v2', 'M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6', 'M10 11v6', 'M14 11v6'],
    pin: ['M12 17v5', 'M9 10.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24V16a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1v-.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V7a1 1 0 0 1 1-1 2 2 0 0 0 0-4H8a2 2 0 0 0 0 4 1 1 0 0 1 1 1Z'],
    flag: ['M4 22V4', 'M4 15s1-1 4-1 5 2 8 2 4-1 4-1V3s-1 1-4 1-5-2-8-2-4 1-4 1'],
    link: ['M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71', 'M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71'],
    eyeOff: ['M9.88 9.88a3 3 0 1 0 4.24 4.24', 'M10.73 5.08A10.43 10.43 0 0 1 12 5c7 0 10 7 10 7a13.16 13.16 0 0 1-1.67 2.68', 'M6.61 6.61A13.53 13.53 0 0 0 2 12s3 7 10 7a9.74 9.74 0 0 0 5.39-1.61', 'M2 2l20 20']
  };

  function icon(name, size = 18) {
    if (name === 'more') {
      return `<svg width="${size}" height="${size}" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><circle cx="5.5" cy="12" r="1.75"/><circle cx="12" cy="12" r="1.75"/><circle cx="18.5" cy="12" r="1.75"/></svg>`;
    }
    if (name === 'play') {
      return `<svg width="${size}" height="${size}" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M8 5.6v12.8a1 1 0 0 0 1.52.85l10.2-6.4a1 1 0 0 0 0-1.7L9.52 4.75A1 1 0 0 0 8 5.6Z"/></svg>`;
    }
    const paths = ICON_PATHS[name] || ICON_PATHS.spark;
    return `<svg width="${size}" height="${size}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${paths.map(d => `<path d="${d}"/>`).join('')}</svg>`;
  }

  function brandLogoSvg(size = 28) {
    return `<svg width="${size}" height="${size}" viewBox="0 0 64 64" aria-hidden="true"><path fill="var(--brand-fill)" d="M10 8h44a4 4 0 0 1 4 4v13a7 7 0 0 0 0 14v13a4 4 0 0 1-4 4H10a4 4 0 0 1-4-4V39a7 7 0 0 0 0-14V12a4 4 0 0 1 4-4z"/><path fill="none" stroke="var(--brand-ink)" stroke-width="6" stroke-linecap="round" stroke-linejoin="round" d="M21 33l8 8 15-17"/></svg>`;
  }

  function platformIcon(name) {
    switch (name) {
      case 'Windows':
        return `<span title="Windows" aria-label="Windows" style="display:inline-grid;place-items:center"><svg width="13" height="13" viewBox="0 0 16 16" fill="#0078d4" aria-hidden="true"><path d="M1 2.3 7.3 1.4v6.1H1V2.3Zm7.2-1L15 .3v7.2H8.2V1.3ZM1 8.5h6.3v6.1L1 13.7V8.5Zm7.2 0H15v7.2l-6.8-1V8.5Z"/></svg></span>`;
      case 'macOS':
        return `<span title="macOS" aria-label="macOS" style="display:inline-grid;place-items:center"><svg width="13" height="13" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true"><path d="M11.6 8.4c0-1.7 1.4-2.5 1.5-2.6-.8-1.2-2.1-1.4-2.5-1.4-1.1-.1-2.1.6-2.7.6-.5 0-1.4-.6-2.3-.6-1.2 0-2.3.7-2.9 1.7-1.2 2.1-.3 5.2.9 6.9.6.8 1.3 1.8 2.2 1.7.9 0 1.2-.6 2.3-.6s1.4.6 2.3.6c1 0 1.6-.8 2.2-1.7.7-1 1-1.9 1-2-.1 0-2-.8-2-2.6ZM9.9 3.3c.5-.6.8-1.4.7-2.2-.7 0-1.6.5-2.1 1.1-.4.5-.8 1.3-.7 2.1.8.1 1.6-.4 2.1-1Z"/></svg></span>`;
      case 'Linux':
        return `<span title="Linux" aria-label="Linux" style="display:inline-grid;place-items:center"><svg width="13" height="13" viewBox="0 0 16 16" fill="none" aria-hidden="true"><path d="M8 1.5c1.7 0 2.7 1.2 2.7 3 0 .8-.2 1.5.4 2.3l1.7 2.3c.7 1 .4 2.8-.8 3.6-.8.6-2.2.8-4 .8s-3.2-.2-4-.8c-1.2-.8-1.5-2.6-.8-3.6l1.7-2.3c.6-.8.4-1.5.4-2.3 0-1.8 1-3 2.7-3Z" fill="#d97706"/><circle cx="6.9" cy="4.2" r="0.6" fill="#fff"/><circle cx="9.1" cy="4.2" r="0.6" fill="#fff"/><path d="M5.4 13.3c0-.8.8-1.2 2.6-1.2s2.6.4 2.6 1.2" stroke="#111827" stroke-width="1.2" stroke-linecap="round"/></svg></span>`;
      case 'Android':
        return `<span title="Android" aria-label="Android" style="display:inline-grid;place-items:center"><svg width="13" height="13" viewBox="0 0 16 16" fill="#22c55e" aria-hidden="true"><path d="M11.6 4.7 12.8 2.6a.35.35 0 1 0-.6-.35L11 4.4A5.5 5.5 0 0 0 5 4.4L3.8 2.25a.35.35 0 0 0-.6.35l1.2 2.1A4.7 4.7 0 0 0 2 8.7h12a4.7 4.7 0 0 0-2.4-4Zm-5.8 2a.6.6 0 1 1 0-1.2.6.6 0 0 1 0 1.2Zm4.4 0a.6.6 0 1 1 0-1.2.6.6 0 0 1 0 1.2ZM2.8 9.6v3.7a1 1 0 0 0 1 1h.8v1.2a.7.7 0 0 0 1.4 0v-1.2h4v1.2a.7.7 0 0 0 1.4 0v-1.2h.8a1 1 0 0 0 1-1V9.6H2.8Z"/></svg></span>`;
      default:
        return '';
    }
  }

  function esc(str) {
    return String(str ?? '')
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;');
  }

  /* ==========================================================================
     应用状态管理 + URL 同步
     ========================================================================== */
  const VALID_DEVICES = new Set(['desktop', 'mobile', 'both']);
  const VALID_THEMES = new Set(['light', 'dark']);
  const VALID_ACCENTS = new Set(['red', 'green', 'blue', 'purple', 'amber', 'graphite']);
  const VALID_PAGES = new Set(['ideas', 'news', 'opensource', 'rankings', 'nav', 'discuss', 'resources', 'apps', 'ai', 'me']);
  const VALID_ROLES = new Set(['reader', 'author', 'admin']);

  function parseDetailParam(raw) {
    if (!raw) return null;
    const idx = raw.indexOf(':');
    if (idx <= 0) return null;
    const type = raw.slice(0, idx);
    const idRaw = raw.slice(idx + 1);
    if (!idRaw) return null;
    if (type === 'work' || type === 'project') return { type, id: idRaw };
    if (type === 'topic') {
      const num = Number(idRaw);
      return Number.isFinite(num) ? { type, id: num } : null;
    }
    return null;
  }

  const params = new URLSearchParams(window.location.search);
  const state = {
    device: VALID_DEVICES.has(params.get('device')) ? params.get('device') : 'both',
    theme: VALID_THEMES.has(params.get('theme')) ? params.get('theme') : 'light',
    accent: VALID_ACCENTS.has(params.get('accent')) ? params.get('accent') : 'red',
    page: VALID_PAGES.has(params.get('page')) ? params.get('page') : 'ideas',
    role: VALID_ROLES.has(params.get('role')) ? params.get('role') : 'author',
    osCategory: '推荐',
    navCat: 'models',
    discussTab: 'all',
    resourceTab: 'software',
    softwareCat: '全部',
    appsFeedFilter: '精选',
    appsAllGames: false,
    appsAllTools: false,
    softwareSort: 'stars',
    frontierIndex: 0,
    selectedModelId: 'deepseek-r1',
    detail: parseDetailParam(params.get('detail')), // { type: 'work'|'topic'|'project', id: string|number }
    detailSlide: 0,
    pcMoreOpen: false,
    h5MoreOpen: false,
    deleteConfirm: null, // { device: 'desktop'|'mobile', target: 'post'|'comment', key, title, commentId? }
    pinnedKeys: new Set(),
    likedIds: new Set(['whale-girl', 'topic-103']),
    savedIds: new Set(['excalidraw/excalidraw', 'toeverything/AFFiNE']),
    followedAuthors: new Set(),
    likedCommentIds: new Set([1]),
    h5ComposerOpen: false,
    replyTo: '',
    composerDraft: '',
    commentsByItem: new Map(),
    overlay: null, // 'spec' | 'search' | 'publish' | null
    searchQuery: '',
    collapsed: false
  };

  function syncUrl() {
    const next = new URLSearchParams();
    next.set('device', state.device);
    next.set('theme', state.theme);
    next.set('accent', state.accent);
    next.set('page', state.page);
    next.set('role', state.role);
    if (state.detail) next.set('detail', `${state.detail.type}:${state.detail.id}`);
    const url = `${window.location.pathname}?${next.toString()}`;
    window.history.replaceState(null, '', url);
  }

  function showToast(msg) {
    const root = document.getElementById('toast-root');
    if (!root) return;
    const el = document.createElement('div');
    el.className = 'wb-toast';
    el.textContent = msg;
    root.appendChild(el);
    setTimeout(() => el.remove(), 2200);
  }

  /* ==========================================================================
     内容片段渲染（活力视觉社区风 · PC + H5）
     ========================================================================== */
  const HOME_TABS = [
    { key: 'ideas', label: '创意', action: 'go-page' },
    { key: 'news', label: '资讯', action: 'go-page' },
    { key: 'opensource', label: '精选', action: 'go-page' },
    { key: 'rankings', label: '排行榜', action: 'go-page' },
    { key: 'nav', label: '导航', action: 'go-page' }
  ];

  const DISCUSS_TABS = [
    { key: 'all', label: '全部', action: 'discuss-tab' },
    { key: 'discuss-uncategorized', label: '默认', action: 'discuss-tab' },
    { key: 'design', label: '设计', action: 'discuss-tab' },
    { key: 'casual-games', label: '游戏', action: 'discuss-tab' },
    { key: '3d', label: '3D', action: 'discuss-tab' },
    { key: 'learn', label: '教程', action: 'discuss-tab' },
    { key: 'casual-deals', label: '羊毛', action: 'discuss-tab' },
    { key: 'casual-travel', label: '旅行', action: 'discuss-tab' },
    { key: 'casual-screen', label: '影视', action: 'discuss-tab' },
    { key: 'casual-feelings', label: '情感', action: 'discuss-tab' },
    { key: 'casual-food', label: '美食', action: 'discuss-tab' },
    { key: 'casual-work', label: '职场', action: 'discuss-tab' },
    { key: 'site', label: '站务', action: 'discuss-tab' },
    { key: 'casual-gear', label: '数码装备', action: 'discuss-tab' },
    { key: 'casual-indie', label: '副业', action: 'discuss-tab' },
    { key: 'casual-help', label: '互助', action: 'discuss-tab' },
    { key: 'casual-chat', label: '闲聊灌水', action: 'discuss-tab' }
  ];

  const RESOURCE_TABS = [
    { key: 'software', label: '开源软件', action: 'res-tab' },
    { key: 'resources-uncategorized', label: '默认', action: 'res-tab' },
    { key: 'resource-movies', label: '影视', action: 'res-tab' },
    { key: 'resource-help', label: '求助悬赏', action: 'res-tab' }
  ];

  function secondaryTabsOfSection(sec) {
    if (sec === 'home') return { items: HOME_TABS, active: state.page, attr: 'data-page', label: '首页子栏目' };
    if (sec === 'discuss') return { items: DISCUSS_TABS, active: state.discussTab, attr: 'data-val', label: '讨论子栏目' };
    if (sec === 'resources') return { items: RESOURCE_TABS, active: state.resourceTab, attr: 'data-val', label: '资源子栏目' };
    return null;
  }

  function sectionKeyOfPage(page) {
    if (HOME_TABS.some(t => t.key === page)) return 'home';
    if (page === 'discuss') return 'discuss';
    if (page === 'resources') return 'resources';
    return page;
  }

  function isLiked(key) { return state.likedIds.has(key); }
  function likeDelta(key, base) { return base + (isLiked(key) ? 1 : 0); }

  function renderWorkCard(w) {
    return `
      <article class="vb-work-card" data-action="open-work" data-id="${w.id}">
        <button type="button" class="work-cover" style="border:0;padding:0;width:100%" data-action="open-work" data-id="${w.id}">
          <img src="${w.cover}" alt="${esc(w.title)}" loading="lazy">
          <span class="vb-glass-tag">${esc(w.category)}</span>
        </button>
        <div class="vb-work-body">
          <button type="button" class="vb-work-title" data-action="open-work" data-id="${w.id}">${esc(w.title)}</button>
          <div class="vb-work-foot">
            <span class="author-chip">
              <span class="mini-avatar" style="background:${w.avatarBg}">${esc(w.author[0].replace('@', '').toUpperCase())}</span>
              <span>${esc(w.author)}</span>
            </span>
            <span class="work-date">${esc(w.date.slice(5))}</span>
          </div>
        </div>
      </article>`;
  }

  function renderIdeasPage() {
    const hero = WORKS[0];
    const cards = [...WORKS.slice(1), WORKS[0]];

    return `
      <div class="vb-matrix">
        <button type="button" class="vb-spot-main" data-action="open-work" data-id="${hero.id}">
          <img src="${hero.cover}" alt="${esc(hero.title)}">
          <div class="vb-spot-scrim">
            <span class="glass-pill">✨ 本周主推 · ${esc(hero.category)}</span>
            <div class="vb-spot-copy">
              <h3>${esc(hero.title)}</h3>
              <p>${esc(hero.description)}</p>
              <span class="vb-spot-cta">立即体验 ${icon('arrow', 13)}</span>
            </div>
          </div>
        </button>
      </div>
      <div class="vb-ideas-grid">${cards.map(renderWorkCard).join('')}</div>
    `;
  }

  function renderNewsPage() {
    const activeItem = FRONTIER_ITEMS[state.frontierIndex] || FRONTIER_ITEMS[0];
    const sideItems = FRONTIER_ITEMS.filter((_, idx) => idx !== state.frontierIndex).slice(0, 2);

    return `
      <section class="frontier-box">
        <div class="section-head">
          <h2>前沿</h2>
        </div>
        <div class="frontier-grid">
          <div class="carousel-main" data-action="open-frontier" data-idx="${state.frontierIndex}">
            <img src="${activeItem.cover}" alt="${esc(activeItem.title)}">
            <button type="button" class="carousel-caption" data-action="open-frontier" data-idx="${state.frontierIndex}">
              <h3>${esc(activeItem.title)}</h3>
            </button>
            <div class="carousel-dots" role="tablist" aria-label="切换前沿焦点">
              ${FRONTIER_ITEMS.map((item, idx) => `
                <button type="button" class="carousel-dot ${idx === state.frontierIndex ? 'is-active' : ''}"
                  data-action="frontier-dot" data-idx="${idx}" aria-label="第 ${idx + 1} 条：${esc(item.title)}"></button>
              `).join('')}
            </div>
            <span class="carousel-meta">${esc(activeItem.source)} · ${esc(activeItem.date)}</span>
          </div>
          <div class="frontier-side">
            ${sideItems.map(item => `
              <button type="button" class="frontier-subcard" data-action="open-frontier" data-idx="${FRONTIER_ITEMS.indexOf(item)}">
                <img src="${item.cover}" alt="${esc(item.title)}">
                <span>${esc(item.title)}</span>
              </button>
            `).join('')}
          </div>
        </div>
      </section>

      <div class="news-split">
        <section class="surface-card">
          <div class="section-head">
            <h2>AI 日报</h2>
            <span class="section-meta">11 / 19 更新</span>
          </div>
          ${DAILY_NEWS.map(n => `
            <button type="button" class="daily-item" data-action="open-news" data-key="${n.key}">
              <div style="flex:1;min-width:0">
                <h3>${esc(n.title)}</h3>
                <span class="section-meta">${esc(n.source)} · ${esc(n.date)}</span>
              </div>
              ${n.thumb ? `<img class="daily-thumb" src="${n.thumb}" alt="" loading="lazy">` : ''}
            </button>
          `).join('')}
        </section>

        <section class="surface-card">
          <div class="section-head">
            <h2>开源热榜</h2>
            <button type="button" class="section-link" data-action="go-page" data-page="opensource">更多精选 ${icon('arrow', 13)}</button>
          </div>
          ${HOT_REPOS_PREVIEW.map(r => `
            <div class="repo-mini-row" data-action="open-os" data-slug="${esc(r.name)}">
              <span class="rank-num ${r.rank <= 3 ? 'is-top' : ''}">${r.rank}</span>
              <div style="flex:1;min-width:0">
                <div style="font-weight:650;font-size:13px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap">${esc(r.name)}</div>
                <div class="section-meta">${esc(r.desc)}</div>
              </div>
              <div style="text-align:right;flex:none;font-variant-numeric:tabular-nums">
                <div style="font-size:12.5px;font-weight:650">${r.stars}</div>
                <div style="font-size:11px;color:var(--brand)">${r.gain}</div>
              </div>
            </div>
          `).join('')}
        </section>
      </div>
    `;
  }

  function renderOpenSourceCard(p, rank) {
    return `
      <article class="os-card">
        <button type="button" class="os-row-btn" data-action="open-os" data-slug="${esc(p.full_name)}">
          <div class="os-main">
            <div class="os-id-row">
              ${rank ? `<span class="os-rank-badge ${rank <= 3 ? 'is-top' : ''}">${rank}</span>` : ''}
              <div class="os-icon">
                ${p.icon ? `<img src="${p.icon}" alt="">` : `<span>${esc(p.name[0])}</span>`}
              </div>
              <div class="os-name-col">
                <h3>${esc(p.name)}</h3>
                <span class="os-slug">${esc(p.full_name)}</span>
              </div>
            </div>
            <p class="os-desc">${esc(p.summary)}</p>
            <div class="os-foot">
              <div class="os-tags">
                ${p.tags.map(t => `<span class="tag-chip">${esc(t)}</span>`).join('')}
              </div>
              <span class="os-stars">★ ${esc(p.stars)}</span>
            </div>
          </div>
          ${p.cover ? `<span class="os-thumb"><img src="${p.cover}" alt="${esc(p.name)}" loading="lazy"></span>` : ''}
        </button>
      </article>
    `;
  }

  function renderOpenSourcePage() {
    const cats = ['推荐', '热榜', 'AI 与智能体', 'Skill 与 MCP', '办公效率', '设计创作', '影音媒体', '开发工具', '教程资源', '开源硬件', '安全逆向', '趣味探索', '建站应用'];
    const starValue = s => parseFloat(s) * (String(s).toLowerCase().endsWith('k') ? 1000 : 1);
    let listHtml = '';
    if (state.osCategory === '热榜') {
      const ranked = [...OPEN_SOURCE_PROJECTS].sort((a, b) => starValue(b.stars) - starValue(a.stars));
      listHtml = `<div class="os-list">${ranked.map((p, idx) => renderOpenSourceCard(p, idx + 1)).join('')}</div>`;
    } else {
      const items = state.osCategory === '推荐'
        ? OPEN_SOURCE_PROJECTS
        : OPEN_SOURCE_PROJECTS.filter(p => p.category === state.osCategory);
      listHtml = items.length
        ? `<div class="os-list">${items.map(p => renderOpenSourceCard(p)).join('')}</div>`
        : `<div class="empty-state-card"><strong>这个分类的项目正在整理中</strong><p>可以先浏览「推荐」或「热榜」查看已收录开源项目。</p></div>`;
    }

    return `
      <div class="filter-pills">
        ${cats.map(c => `<button type="button" class="filter-pill ${state.osCategory === c ? 'is-active' : ''}" data-action="os-cat" data-val="${c}">${c}</button>`).join('')}
      </div>
      ${listHtml}
    `;
  }

  function renderRankingsPage() {
    const minLog = Math.log10(0.18), maxLog = Math.log10(32);
    const minScore = 52, maxScore = 78;
    const W = 520, H = 260, padL = 42, padR = 20, padT = 18, padB = 34;
    const plotW = W - padL - padR, plotH = H - padT - padB;
    const xOf = price => padL + ((Math.log10(price) - minLog) / (maxLog - minLog)) * plotW;
    const yOf = score => padT + (1 - (score - minScore) / (maxScore - minScore)) * plotH;

    const frontier = [...RANK_MODELS]
      .sort((a, b) => a.price - b.price)
      .filter((m, _, arr) => !arr.some(o => o.price < m.price && o.intelligence >= m.intelligence))
      .map(m => `${xOf(m.price).toFixed(1)},${yOf(m.intelligence).toFixed(1)}`)
      .join(' ');

    const selected = RANK_MODELS.find(m => m.id === state.selectedModelId) || RANK_MODELS[0];
    const topIntelligence = [...RANK_MODELS].sort((a, b) => b.intelligence - a.intelligence).slice(0, 6);
    const topSpeed = [...RANK_MODELS].sort((a, b) => b.speed - a.speed).slice(0, 5);

    return `
      <div class="rank-grid">
        <section class="surface-card">
          <div class="section-head" style="margin-bottom:8px">
            <h2 style="font-size:15px">智能指数 vs 加权价格（对数轴）</h2>
            <span class="section-meta">越靠左上性价比越高</span>
          </div>
          <div class="scatter-wrap">
            <svg class="scatter-svg" viewBox="0 0 ${W} ${H}" role="img" aria-label="智能指数与价格散点图">
              <line x1="${padL}" y1="${padT}" x2="${padL}" y2="${H - padB}" stroke="currentColor" stroke-opacity="0.16" />
              <line x1="${padL}" y1="${H - padB}" x2="${W - padR}" y2="${H - padB}" stroke="currentColor" stroke-opacity="0.16" />
              ${[55, 60, 65, 70, 75].map(s => `
                <line x1="${padL}" y1="${yOf(s)}" x2="${W - padR}" y2="${yOf(s)}" stroke="currentColor" stroke-opacity="0.08" stroke-dasharray="3 3" />
                <text x="${padL - 8}" y="${yOf(s) + 3}" text-anchor="end" font-size="10" fill="currentColor" opacity="0.55">${s}</text>
              `).join('')}
              ${[0.3, 1, 3, 10].map(p => `
                <line x1="${xOf(p)}" y1="${padT}" x2="${xOf(p)}" y2="${H - padB}" stroke="currentColor" stroke-opacity="0.08" stroke-dasharray="3 3" />
                <text x="${xOf(p)}" y="${H - padB + 15}" text-anchor="middle" font-size="10" fill="currentColor" opacity="0.55">$${p}</text>
              `).join('')}
              <polyline fill="none" stroke="var(--brand-fill)" stroke-width="1.8" stroke-dasharray="5 4" opacity="0.65" points="${frontier}" />
              ${RANK_MODELS.map(m => {
                const cx = xOf(m.price), cy = yOf(m.intelligence), active = m.id === selected.id;
                return `
                  <g style="cursor:pointer" data-action="select-model" data-id="${m.id}">
                    ${active ? `<circle cx="${cx}" cy="${cy}" r="13" fill="${m.color}" opacity="0.22" />` : ''}
                    <circle cx="${cx}" cy="${cy}" r="${active ? 6.5 : 5}" fill="${m.color}" stroke="#fff" stroke-width="1.5" />
                    <text x="${cx + 8}" y="${cy + 3}" font-size="10.5" font-weight="${active ? 700 : 500}" fill="currentColor" opacity="${active ? 1 : 0.72}">${esc(m.name.split(' ')[0])}</text>
                  </g>
                `;
              }).join('')}
            </svg>
          </div>
          <div class="scatter-detail">
            <div>
              <strong style="font-size:14px">${esc(selected.name)}</strong>
              <div class="section-meta">${esc(selected.vendor)} · 加权报价 $${selected.price} / M tokens</div>
            </div>
            <div style="display:flex;gap:14px;font-variant-numeric:tabular-nums">
              <div><small class="section-meta">智能</small><div><strong>${selected.intelligence}</strong></div></div>
              <div><small class="section-meta">输出速度</small><div><strong>${selected.speed} t/s</strong></div></div>
            </div>
          </div>
        </section>

        <div style="display:grid;gap:16px">
          <section class="surface-card">
            <div class="section-head" style="margin-bottom:10px">
              <h2 style="font-size:15px">AA 智能指数 Top 6</h2>
              <span class="section-meta">v2 基准</span>
            </div>
            <div class="rank-bars">
              ${topIntelligence.map((m, idx) => `
                <div class="rank-bar-item">
                  <div class="rank-bar-top">
                    <span>${idx + 1}. ${esc(m.name)}</span>
                    <span style="font-variant-numeric:tabular-nums">${m.intelligence}</span>
                  </div>
                  <div class="rank-track"><div class="rank-fill" style="width:${Math.round((m.intelligence / 78) * 100)}%"></div></div>
                </div>
              `).join('')}
            </div>
          </section>

          <section class="surface-card">
            <div class="section-head" style="margin-bottom:10px">
              <h2 style="font-size:15px">输出速度 & OpenRouter 调用量</h2>
            </div>
            <div style="display:grid;gap:8px;font-size:12.5px">
              ${topSpeed.slice(0, 3).map((m, idx) => `
                <div style="display:flex;justify-content:space-between;padding:4px 0;border-bottom:1px solid var(--bd)">
                  <span>⚡ ${idx + 1}. ${esc(m.name)}</span>
                  <strong style="font-variant-numeric:tabular-nums">${m.speed} token/s</strong>
                </div>
              `).join('')}
              ${USAGE_TOP5.slice(0, 3).map(u => `
                <div style="display:flex;justify-content:space-between;padding:4px 0;border-bottom:1px solid var(--bd)">
                  <span>🔥 #${u.rank} ${esc(u.name)}</span>
                  <span style="font-variant-numeric:tabular-nums">${u.share} <small style="color:var(--brand)">${u.delta}</small></span>
                </div>
              `).join('')}
            </div>
          </section>
        </div>
      </div>
    `;
  }

  function renderNavPage() {
    const navCats = [
      { key: 'models', label: '大模型' },
      { key: 'harness', label: 'Harness' },
      { key: 'skills', label: 'Skill' },
      { key: 'api', label: 'API' }
    ];
    const currentCat = navCats.find(c => c.key === state.navCat) || navCats[0];
    const list = NAV_ENTRIES.filter(e => e.cat === currentCat.key);

    return `
      <div class="filter-pills">
        ${navCats.map(c => `<button type="button" class="filter-pill ${state.navCat === c.key ? 'is-active' : ''}" data-action="nav-cat" data-val="${c.key}">${c.label}</button>`).join('')}
      </div>
      <div class="nav-grid">
        ${list.map(item => `
          <a href="#nav-${item.key}" class="nav-card" data-action="toast" data-msg="即将跳转官方入口：${esc(item.domain)}">
            <span class="vendor-tile">${item.icon ? `<img src="${item.icon}" alt="">` : `<span>${esc(item.name[0])}</span>`}</span>
            <span class="nav-card-name">${esc(item.name)}</span>
            <span class="nav-card-vendor">${esc(item.vendor)}</span>
            <p class="nav-card-summary">${esc(item.summary)}</p>
            <div class="nav-card-foot">
              <span style="font-family:var(--font-mono)">${esc(item.domain)}</span>
              <span class="tag-chip">${esc(item.type)}</span>
            </div>
          </a>
        `).join('')}
      </div>
    `;
  }

  function renderDiscussPage() {
    const filtered = state.discussTab === 'all'
      ? DISCUSS_TOPICS
      : DISCUSS_TOPICS.filter(t => (t.tabs || []).includes(state.discussTab));

    const currentLabel = DISCUSS_TABS.find(t => t.key === state.discussTab)?.label || '讨论';
    const rows = filtered.map(t => {
      const liked = isLiked('topic-' + t.id);
      const likes = likeDelta('topic-' + t.id, t.likes || 0);
      return `
        <article class="vb-topic-card" data-action="open-topic" data-id="${t.id}">
          <div class="vb-topic-body">
            <div class="vb-topic-main">
              <button type="button" class="topic-title" data-action="open-topic" data-id="${t.id}">${esc(t.title)}</button>
              <p class="vb-topic-excerpt">${esc(t.excerpt)}</p>
            </div>
            ${t.cover ? `<div class="vb-topic-cover" aria-hidden="true"><img src="${t.cover}" alt="" loading="lazy"></div>` : ''}
          </div>
          <div class="topic-foot">
            <div class="topic-byline">
              <span class="cat-badge" data-tone="${t.tone}">${esc(t.category)}</span>
              <span class="topic-tag"># ${esc(t.tag)}</span>
              <span class="topic-identity">
                <span class="mini-avatar" style="background:${t.avatarBg}">${esc(t.author[0])}</span>
                <span class="topic-author-text">${esc(t.author)} · ${esc(t.time)}</span>
              </span>
            </div>
            <div class="topic-metrics">
              <button type="button" class="metric-btn" data-action="open-topic" data-id="${t.id}">${icon('chat', 13)} ${t.replies}</button>
              <button type="button" class="metric-btn ${liked ? 'is-liked' : ''}" data-action="toggle-like" data-key="topic-${t.id}">${icon('heart', 13)}${likes ? `<span>${likes}</span>` : ''}</button>
            </div>
          </div>
        </article>
      `;
    }).join('');

    return `
      <div class="discuss-wrap">
        <div class="vb-topic-list">
          ${rows || `<div class="empty-state-card"><strong>「${esc(currentLabel)}」暂无示例讨论</strong><p>点击左上角「发布灵感」或在真实上线后发布该板块的首篇讨论。</p></div>`}
        </div>
      </div>
    `;
  }

  function renderResourcesPage() {
    if (state.resourceTab === 'software') {
      const cats = ['全部', 'AI', '影音', '网络', '工具', '效率', '开发', '游戏', '其它'];
      const filtered = state.softwareCat === '全部'
        ? RESOURCE_SOFTWARE
        : RESOURCE_SOFTWARE.filter(s => s.cat === state.softwareCat);
      const sorted = [...filtered].sort((a, b) => {
        if (state.softwareSort === 'updated') return a.updatedDays - b.updatedDays || b.starsNum - a.starsNum;
        if (state.softwareSort === 'hot') return (b.starGain || 0) - (a.starGain || 0) || b.starsNum - a.starsNum;
        return b.starsNum - a.starsNum;
      });
      const toolbarHtml = `
        <div class="sw-toolbar">
          <div class="filter-pills" role="tablist" aria-label="开源软件分类">
            ${cats.map(c => `<button type="button" class="filter-pill ${state.softwareCat === c ? 'is-active' : ''}" data-action="sw-cat" data-val="${c}">${c}</button>`).join('')}
          </div>
          <div class="sw-sort-group" role="group" aria-label="排序方式">
            <button type="button" class="filter-pill ${state.softwareSort === 'stars' ? 'is-active' : ''}" data-action="sw-sort" data-val="stars">收藏数</button>
            <button type="button" class="filter-pill ${state.softwareSort === 'updated' ? 'is-active' : ''}" data-action="sw-sort" data-val="updated">最近更新</button>
            <button type="button" class="filter-pill ${state.softwareSort === 'hot' ? 'is-active' : ''}" data-action="sw-sort" data-val="hot">近期热门</button>
          </div>
        </div>
      `;
      return `
        ${toolbarHtml}
        <div class="software-grid">
          ${sorted.map(s => `
            <article class="software-card" data-action="open-os" data-slug="${esc(s.slug)}">
              <div class="sw-head">
                <div class="sw-icon" style="${!s.icon && s.iconBg ? `background:${s.iconBg};color:${s.iconColor}` : ''}">
                  ${s.icon ? `<img src="${s.icon}" alt="">` : `<span>${esc(s.glyph || s.name[0])}</span>`}
                </div>
                <div class="sw-title-col">
                  <div class="sw-name-row">
                    <strong class="sw-name">${esc(s.name)}</strong>
                    <span class="sw-stars">${esc(s.stars)}</span>
                  </div>
                  <span class="sw-slug">${esc(s.slug)}</span>
                </div>
              </div>
              <p class="sw-desc">${esc(s.desc)}</p>
              <div class="sw-badges">
                <span class="sw-ver">${esc(s.ver)}</span>
                <span class="sw-lang"><i class="sw-lang-dot" style="background:${s.langColor}"></i>${esc(s.lang)}</span>
              </div>
              <div class="sw-foot">
                <div class="sw-platforms">${s.platforms.map(platformIcon).join('')}</div>
                <span class="sw-updated">${esc(s.updated)}</span>
              </div>
            </article>
          `).join('')}
        </div>
      `;
    }

    if (state.resourceTab === 'resources-uncategorized') {
      return `
        <div style="display:grid;gap:12px;max-width:840px">
          ${RESOURCE_UNCATEGORIZED.map(r => `
            <article class="help-card" data-action="toast" data-msg="打开默认资源贴：${esc(r.title)}">
              <div style="display:flex;align-items:center;gap:8px;flex-wrap:wrap">
                <span class="tag-chip">${esc(r.tag)}</span>
                <span class="section-meta">${esc(r.time)}</span>
              </div>
              <button type="button" class="topic-title" data-action="toast" data-msg="打开默认资源贴：${esc(r.title)}">${esc(r.title)}</button>
              <div class="section-meta">${esc(r.author)} 分享 · ${r.replies} 条交流</div>
            </article>
          `).join('')}
        </div>
      `;
    }

    if (state.resourceTab === 'resource-help') {
      return `
        <div style="display:grid;gap:12px;max-width:840px">
          ${HELP_BOUNTIES.map(h => `
            <article class="help-card" data-action="toast" data-msg="查看悬赏详情：${esc(h.title)}">
              <div style="display:flex;align-items:center;gap:8px;flex-wrap:wrap">
                <span class="bounty-pill">悬赏 ${h.bounty} 积分</span>
                <span class="status-pill ${h.status === '已采纳' ? 'is-solved' : ''}">${esc(h.status)}</span>
                <span class="section-meta">${esc(h.expires)}</span>
              </div>
              <button type="button" class="topic-title" data-action="toast" data-msg="查看悬赏详情：${esc(h.title)}">${esc(h.title)}</button>
              <div class="section-meta">${esc(h.author)} 发起 · ${h.answers} 个回答</div>
            </article>
          `).join('')}
        </div>
      `;
    }

    return `
      <div class="vb-ideas-grid">
        ${WORKS.slice(4, 7).map(renderWorkCard).join('')}
      </div>
    `;
  }

  function renderPlazaWorkCover(w) {
    if (w.isWancisou) {
      return `
        <div class="plaza-wancisou-shot">
          <div class="pw-head">万磁搜</div>
          <div class="pw-search-row">
            <span class="pw-input">${icon('search', 10)} <span>搜索公开资源</span></span>
            <span class="pw-search-btn">搜索</span>
          </div>
          <div class="pw-item">
            <div class="pw-item-title">Ubuntu 24.04 LTS Desktop</div>
            <div class="pw-tags"><span>RS256</span><span>4.7 GB</span><span>2026-11-11</span></div>
          </div>
          <div class="plaza-shot-foot">
            <span class="plaza-shot-stats">👁 ${w.views} &nbsp;💬 ${w.commentsCount}</span>
            <span class="plaza-shot-badge">${esc(w.category)}</span>
          </div>
        </div>
      `;
    }
    return `
      <div class="plaza-photo-shot">
        <img src="${w.cover}" alt="${esc(w.title)}" loading="lazy">
        <div class="plaza-shot-foot">
          <span class="plaza-shot-stats">👁 ${w.views} &nbsp;💬 ${w.commentsCount}</span>
          <span class="plaza-shot-badge">${esc(w.category)}</span>
        </div>
      </div>
    `;
  }

  function renderAppsPage() {
    const games = state.appsAllGames ? PLAZA_GAMES : PLAZA_GAMES.slice(0, 4);
    const tools = state.appsAllTools ? PLAZA_TOOLS : PLAZA_TOOLS.slice(0, 4);
    const feedFilters = ['精选', '最新', '游戏', '工具'];
    const filteredWorks = PLAZA_WORKS.filter(w => {
      if (state.appsFeedFilter === '精选') return w.featured;
      if (state.appsFeedFilter === '最新') return true;
      return w.kind === state.appsFeedFilter;
    });

    return `
      <section class="plaza-section">
        <div class="plaza-head">
          <h3>精选游戏</h3>
          <button type="button" class="plaza-more-btn" data-action="apps-toggle-games">${state.appsAllGames ? '收起' : '全部'} ${icon('arrow', 13)}</button>
        </div>
        <div class="plaza-icon-grid">
          ${games.map(g => `
            <button type="button" class="plaza-icon-item" data-action="toast" data-msg="启动游戏：${esc(g.title)}">
              <span class="plaza-app-icon"><img src="${g.icon}" alt="${esc(g.title)}"></span>
              <span class="plaza-app-name">${esc(g.title)}</span>
              <span class="plaza-app-sub">${esc(g.genre)}</span>
            </button>
          `).join('')}
        </div>
      </section>

      <section class="plaza-section">
        <div class="plaza-head">
          <h3>精选工具</h3>
          <button type="button" class="plaza-more-btn" data-action="apps-toggle-tools">${state.appsAllTools ? '收起' : '全部'} ${icon('arrow', 13)}</button>
        </div>
        <div class="plaza-icon-grid">
          ${tools.map(t => `
            <button type="button" class="plaza-icon-item" data-action="toast" data-msg="打开工具：${esc(t.title)}">
              <span class="plaza-app-icon"><img src="${t.icon}" alt="${esc(t.title)}"></span>
              <span class="plaza-app-name">${esc(t.title)}</span>
              ${state.appsAllTools ? `<span class="plaza-app-sub">${esc(t.genre)}</span>` : ''}
            </button>
          `).join('')}
        </div>
      </section>

      <section class="plaza-section">
        <div class="plaza-head">
          <h3>社区作品</h3>
        </div>
        <div class="filter-pills">
          ${feedFilters.map(f => `<button type="button" class="filter-pill ${state.appsFeedFilter === f ? 'is-active' : ''}" data-action="apps-filter" data-val="${f}">${f}</button>`).join('')}
        </div>
        <div class="plaza-work-grid">
          ${filteredWorks.map(w => `
            <article class="plaza-work-card" data-action="open-work" data-id="${w.id}">
              <button type="button" class="plaza-work-cover" data-action="open-work" data-id="${w.id}">
                ${renderPlazaWorkCover(w)}
              </button>
              <div class="plaza-work-body">
                <button type="button" class="plaza-work-title" data-action="open-work" data-id="${w.id}">${esc(w.title)}</button>
                <div class="work-meta-row" style="margin-top:8px">
                  <span class="mini-avatar" style="background:${w.avatarBg}">${esc(w.author[0])}</span>
                  <span>${esc(w.author)}</span>
                </div>
              </div>
            </article>
          `).join('')}
        </div>
        <div class="plaza-end">已看完所有作品</div>
      </section>
    `;
  }

  function renderAiPage() {
    return `
      <div class="surface-card" style="max-width:540px;margin:20px auto;text-align:center;padding:34px 24px">
        <div style="width:58px;height:58px;border-radius:18px;background:var(--brand-soft);color:var(--brand);display:inline-grid;place-items:center;margin-bottom:14px">
          ${icon('spark', 28)}
        </div>
        <h2 style="margin:0 0 8px;font-size:22px">小伴 AI · 全端创作助手</h2>
        <p style="margin:0 0 22px;color:var(--ink-2);font-size:13.5px">在 Windows、macOS 与 Android 端启动本地工作区、生成游戏/壁纸并一键分享至社区。</p>
        <div style="display:flex;gap:10px;justify-content:center;flex-wrap:wrap">
          <button type="button" class="btn-solid" data-action="toast" data-msg="开始下载 Android 安装包">${icon('download', 15)} 下载手机版</button>
          <button type="button" class="btn-outline" data-action="toast" data-msg="前往小伴桌面下载页 wallpaper.xiangban.xyz">下载桌面版</button>
        </div>
      </div>
    `;
  }

  function renderMePage() {
    return `
      <div class="me-card">
        <div class="surface-card" style="display:flex;align-items:center;gap:14px">
          <span class="user-avatar" style="width:52px;height:52px;font-size:19px">伴</span>
          <div style="flex:1;min-width:0">
            <strong style="font-size:18px">阿伴创作者</strong>
            <div class="section-meta">账号 ID 100482 · 240 积分</div>
          </div>
          <button type="button" class="btn-outline" style="height:32px;font-size:12px" data-action="toast" data-msg="已签到，今日积分 +1">每日签到</button>
        </div>
        <div class="surface-card" style="padding:0;overflow:hidden">
          <div class="me-row" data-action="go-page" data-page="apps"><span>我的作品</span><span class="section-meta">3 个已发布 ${icon('arrow', 14)}</span></div>
          <div class="me-row" data-action="go-page" data-page="opensource"><span>我的收藏</span><span class="section-meta">${state.savedIds.size} 个开源项目 ${icon('arrow', 14)}</span></div>
          <div class="me-row" data-action="toast" data-msg="请在小伴桌面或手机客户端查看 AI 额度与兑换记录"><span>AI 额度与兑换</span><span class="section-meta">在桌面/手机查看 ${icon('arrow', 14)}</span></div>
          <div class="me-row" data-action="toast" data-msg="QQ 群 / 微信交流群入口请前往社区公告查看"><span>社区交流群</span><span class="section-meta">QQ 群 / 微信群 ${icon('arrow', 14)}</span></div>
        </div>
      </div>
    `;
  }

  const GALLERY_EXTRAS = ['./assets/micro-gallery.webp', './assets/whale-girl.webp', './assets/jiangshan-map.webp', './assets/excalidraw-showcase.webp'];

  function buildGallery(cover) {
    if (!cover) return [];
    return [cover, ...GALLERY_EXTRAS.filter(src => src !== cover).slice(0, 2)];
  }

  function getDetailModel() {
    const d = state.detail;
    if (!d) return null;
    if (d.type === 'work') {
      const w = WORKS.find(item => item.id === d.id) || PLAZA_WORKS.find(item => item.id === d.id);
      if (!w) return null;
      return {
        key: w.id,
        badge: w.category,
        tags: [w.category, '小伴创意', w.kind || '交互作品'].filter((v, i, a) => v && a.indexOf(v) === i),
        title: w.title,
        author: w.author,
        avatarBg: w.avatarBg,
        date: w.date,
        region: '社区',
        cover: w.cover,
        gallery: buildGallery(w.cover),
        body: w.introduction,
        controls: w.controls,
        actionLabel: w.actionLabel,
        actionKind: w.actionLabel && w.actionLabel.startsWith('前往') ? 'external' : 'play',
        likes: w.likes,
        saves: Math.max(12, Math.round((w.likes || 40) * 0.26))
      };
    }
    if (d.type === 'topic') {
      const t = DISCUSS_TOPICS.find(item => item.id === d.id);
      if (!t) return null;
      return {
        key: 'topic-' + t.id,
        badge: t.category,
        tags: [t.category, t.tag, '社区讨论'].filter((v, i, a) => v && a.indexOf(v) === i),
        title: t.title,
        author: t.author,
        avatarBg: t.avatarBg,
        date: t.time,
        region: '社区',
        cover: t.cover || '',
        gallery: t.images || (t.cover ? [t.cover] : []),
        body: t.excerpt + '\n\n欢迎在下方留言区继续补充你的桌面实拍图或调试经验。',
        controls: '',
        actionLabel: '',
        actionKind: '',
        likes: t.likes,
        saves: Math.max(8, Math.round((t.likes || 20) * 0.26))
      };
    }
    if (d.type === 'project') {
      const p = OPEN_SOURCE_PROJECTS.find(item => item.full_name === d.id);
      if (p) {
        return {
          key: p.full_name,
          badge: p.category,
          tags: [...p.tags, p.category].filter((v, i, a) => v && a.indexOf(v) === i),
          title: `${p.name} (${p.full_name})`,
          author: `GitHub ${p.stars} Stars`,
          avatarBg: '#2B2B30',
          date: '公开资料整理',
          region: '开源精选',
          cover: p.cover,
          gallery: buildGallery(p.cover),
          body: p.summary + ' 本条目用于社区精选导读，点击下方按钮可查看官方仓库发布页或保存收藏。',
          controls: '标签：' + p.tags.join(' · '),
          actionLabel: '访问官方仓库',
          actionKind: 'external',
          likes: 92,
          saves: 34
        };
      }
      const sw = RESOURCE_SOFTWARE.find(item => item.slug === d.id);
      if (!sw) return null;
      return {
        key: sw.slug,
        badge: `${sw.cat} · ${sw.ver}`,
        tags: [sw.cat, sw.lang, ...sw.platforms.slice(0, 2)].filter((v, i, a) => v && a.indexOf(v) === i),
        title: `${sw.name} (${sw.slug})`,
        author: `GitHub ${sw.stars} Stars`,
        avatarBg: '#2B2B30',
        date: sw.updated,
        region: '开源软件',
        cover: '',
        gallery: [],
        body: `${sw.desc}\n\n支持平台：${sw.platforms.join(' / ')}。当前收录版本 ${sw.ver}，主要开发语言为 ${sw.lang}。`,
        controls: `平台支持：${sw.platforms.join(' · ')}`,
        actionLabel: `下载安装包 (${sw.ver})`,
        actionKind: 'download',
        likes: 120,
        saves: 41
      };
    }
    return null;
  }

  function getDetailMoreItems(item, device) {
    const isPinned = state.pinnedKeys.has(item.key);
    const includeShare = device === 'mobile' && state.role !== 'author';
    const items = [];
    if (includeShare) {
      items.push({ icon: 'share', label: '分享给好友', action: 'detail-more-toast', msg: '已复制分享链接' });
    }
    if (state.role === 'author') {
      items.push({ icon: 'edit', label: '编辑内容', action: 'detail-more-toast', msg: '演示打开：编辑当前内容' });
      items.push({ icon: 'link', label: '复制链接', action: 'detail-more-toast', msg: '已复制内容链接' });
      items.push({ divider: true });
      items.push({ icon: 'trash', label: '删除内容', action: 'ask-delete-post', device, danger: true });
    } else if (state.role === 'admin') {
      items.push({ icon: 'pin', label: isPinned ? '取消置顶' : '置顶内容', action: 'toggle-pin-detail', key: item.key });
      items.push({ icon: 'edit', label: '编辑内容', action: 'detail-more-toast', msg: '演示打开：管理员编辑内容' });
      items.push({ icon: 'link', label: '复制链接', action: 'detail-more-toast', msg: '已复制内容链接' });
      items.push({ divider: true });
      items.push({ icon: 'trash', label: '删除内容', action: 'ask-delete-post', device, danger: true });
    } else {
      items.push({ icon: 'link', label: '复制链接', action: 'detail-more-toast', msg: '已复制内容链接' });
      items.push({ icon: 'eyeOff', label: '不感兴趣', action: 'detail-more-toast', msg: '已减少此类内容推荐' });
      items.push({ divider: true });
      items.push({ icon: 'flag', label: '举报内容', action: 'detail-more-toast', msg: '已提交举报，感谢反馈', danger: true });
    }
    return items;
  }

  function renderDeleteConfirmDialog(device) {
    const cfg = state.deleteConfirm;
    if (!cfg || cfg.device !== device) return '';
    const isComment = cfg.target === 'comment';
    const title = isComment ? '确认删除这条评论？' : '确认删除这条内容？';
    const desc = isComment
      ? `删除后无法恢复：${cfg.title}`
      : `删除「${cfg.title}」后将无法恢复，相关评论与互动数据也会一并移除。`;
    const roleHint = !isComment && state.role === 'admin'
      ? '<div class="xhs-confirm-note">管理员操作 · 删除后将自动通知作者</div>'
      : '';
    return `
      <div class="xhs-confirm-backdrop ${device === 'mobile' ? 'is-mobile' : 'is-desktop'}" data-action="close-detail-popups">
        <div class="xhs-confirm-dialog" role="alertdialog" aria-modal="true" aria-label="${esc(title)}">
          <h3>${esc(title)}</h3>
          <p>${esc(desc)}</p>
          ${roleHint}
          <div class="xhs-confirm-actions">
            <button type="button" class="xhs-confirm-btn is-cancel" data-action="close-detail-popups">取消</button>
            <button type="button" class="xhs-confirm-btn is-danger" data-action="confirm-delete">确认删除</button>
          </div>
        </div>
      </div>
    `;
  }

  function renderDetailComments(item, comments, device) {
    const canManageComments = state.role === 'author' || state.role === 'admin';
    return `
      <div class="xhs-comments-head">
        <span>共 ${comments.length} 条评论</span>
        <button type="button" class="xhs-sort-btn" data-action="toast" data-msg="已切换评论排序">
          ${icon('sort', 14)}
        </button>
      </div>
      ${device === 'mobile' ? `
        <div class="xhs-inline-prompt">
          <span class="mini-avatar" style="background:#D93A44;width:30px;height:30px;font-size:12px">伴</span>
          <button type="button" class="xhs-inline-pill" data-action="open-h5-composer">
            <span>留下你的想法吧</span>
            <span class="xhs-inline-icons">${icon('image', 15)}${icon('smile', 15)}</span>
          </button>
        </div>
      ` : ''}
      <div class="xhs-comments-list">
        ${comments.map(c => {
          const cLiked = state.likedCommentIds.has(c.id);
          const cLikes = (c.likes || 0) + (cLiked ? 1 : 0);
          return `
            <div class="xhs-comment-row">
              <span class="mini-avatar" style="background:${c.avatarBg};width:32px;height:32px;font-size:12.5px">${esc(c.author[0])}</span>
              <div class="xhs-comment-body">
                <div class="xhs-comment-author">${esc(c.author)}</div>
                <div class="xhs-comment-text">${esc(c.text)}</div>
                <div class="xhs-comment-foot">
                  <span class="xhs-comment-meta">${esc(c.time)} ${esc(c.region || '社区')}</span>
                  <button type="button" class="xhs-reply-btn" data-action="reply-comment" data-author="${esc(c.author)}">回复</button>
                  ${canManageComments ? `
                    <button type="button" class="xhs-comment-del-btn" data-action="ask-delete-comment" data-cid="${c.id}" data-device="${device}">删除</button>
                  ` : ''}
                  <button type="button" class="xhs-comment-like ${cLiked ? 'is-liked' : ''}" data-action="toggle-comment-like" data-cid="${c.id}">
                    ${icon('heart', 14)}${cLikes ? `<span>${cLikes}</span>` : ''}
                  </button>
                </div>
              </div>
            </div>
          `;
        }).join('')}
      </div>
    `;
  }

  function renderDetailView(device) {
    const item = getDetailModel();
    if (!item) return '';
    const comments = state.commentsByItem.get(item.key) || INITIAL_COMMENTS;
    const liked = isLiked(item.key);
    const saved = state.savedIds.has(item.key);
    const followed = state.followedAuthors.has(item.author);
    const likeCount = likeDelta(item.key, item.likes);
    const saveCount = (item.saves || 18) + (saved ? 1 : 0);
    const gallery = item.gallery || [];
    const slideIdx = gallery.length ? Math.min(state.detailSlide, gallery.length - 1) : 0;
    const activeImg = gallery[slideIdx] || item.cover;
    const dotsHtml = (overlay = false) => gallery.length > 1 ? `
      <div class="xhs-media-dots ${overlay ? 'is-overlay' : ''}" role="tablist" aria-label="切换图片">
        ${gallery.map((_, idx) => `<button type="button" class="xhs-media-dot ${idx === slideIdx ? 'is-active' : ''}" data-action="detail-slide" data-idx="${idx}" aria-label="第 ${idx + 1} 张"></button>`).join('')}
      </div>
    ` : '';
    const tagsHtml = (item.tags || []).map(tag => `<span class="xhs-hashtag">#${esc(tag)}</span>`).join(' ');
    const actionIcon = item.actionKind === 'external' ? 'external' : item.actionKind === 'download' ? 'download' : 'play';
    const primaryCtaHtml = item.actionLabel ? `
      <button type="button" class="xhs-primary-cta" data-action="toast" data-msg="演示打开：${esc(item.actionLabel)}">
        <span class="xhs-primary-cta-icon">${icon(actionIcon, actionIcon === 'play' ? 16 : 15)}</span>
        <span class="xhs-primary-cta-label">${esc(item.actionLabel)}</span>
      </button>
    ` : '';
    const infoStripsHtml = item.controls ? `
      <div class="xhs-info-strip is-static">
        <span class="xhs-strip-tag">说明</span>
        <span class="xhs-strip-text">${esc(item.controls)}</span>
      </div>
    ` : '';

    if (device === 'desktop') {
      const replyPlaceholder = state.replyTo ? `回复 @${state.replyTo}…` : '说点什么…';
      const isAuthor = state.role === 'author';
      const isPinned = state.pinnedKeys.has(item.key);
      const pcMenuItems = getDetailMoreItems(item, 'desktop');
      return `
        <div class="xhs-pc-wrap">
          <div class="xhs-pc-backbar">
            <button type="button" class="btn-outline" style="height:32px;font-size:12.5px" data-action="close-detail">
              ${icon('back', 14)} 返回列表
            </button>
          </div>
          <div class="xhs-pc-card">
            <div class="xhs-author-bar">
              <div class="xhs-head-author">
                <span class="mini-avatar" style="background:${item.avatarBg};width:34px;height:34px;font-size:13px">${esc(item.author[0].replace('@', '').toUpperCase())}</span>
                <strong>${esc(item.author)}</strong>
                ${isAuthor ? '<span class="xhs-role-pill">我</span>' : ''}
                ${isPinned ? '<span class="xhs-pin-pill">置顶</span>' : ''}
              </div>
              <div class="xhs-author-actions">
                ${!isAuthor ? `<button type="button" class="xhs-follow-btn ${followed ? 'is-followed' : ''}" data-action="toggle-follow" data-author="${esc(item.author)}">${followed ? '已关注' : '关注'}</button>` : ''}
                <button type="button" class="h5-icon-btn" aria-label="分享" data-action="toast" data-msg="已复制分享链接">${icon('share', 16)}</button>
                <div class="xhs-more-wrap">
                  <button type="button" class="h5-icon-btn ${state.pcMoreOpen ? 'is-open' : ''}" aria-label="更多操作" aria-expanded="${state.pcMoreOpen}" data-action="toggle-pc-more">${icon('more', 17)}</button>
                  ${state.pcMoreOpen ? `
                    <div class="xhs-pc-menu" role="menu" aria-label="内容操作">
                      ${pcMenuItems.map(entry => entry.divider ? '<div class="xhs-menu-divider" role="separator"></div>' : `
                        <button type="button" class="xhs-menu-item ${entry.danger ? 'is-danger' : ''}" role="menuitem"
                          data-action="${esc(entry.action)}"
                          ${entry.msg ? `data-msg="${esc(entry.msg)}"` : ''}
                          ${entry.key ? `data-key="${esc(entry.key)}"` : ''}
                          ${entry.device ? `data-device="${esc(entry.device)}"` : ''}>
                          ${icon(entry.icon, 15)}<span>${esc(entry.label)}</span>
                        </button>
                      `).join('')}
                    </div>
                  ` : ''}
                </div>
              </div>
            </div>
            ${activeImg ? `
              <div class="xhs-pc-media" ${gallery.length > 1 ? 'data-action="detail-slide-next"' : ''}>
                <img src="${activeImg}" alt="${esc(item.title)}">
                <span class="xhs-media-pill">${esc(item.badge)}</span>
                ${gallery.length > 1 ? `
                  <span class="xhs-media-count">${slideIdx + 1}/${gallery.length}</span>
                  <button type="button" class="xhs-pc-nav is-prev" aria-label="上一张" data-action="detail-slide-prev">${icon('back', 16)}</button>
                  <button type="button" class="xhs-pc-nav is-next" aria-label="下一张" data-action="detail-slide-next">${icon('arrow', 16)}</button>
                ` : ''}
              </div>
              ${dotsHtml(false)}
            ` : ''}
            <div class="xhs-pc-body ${gallery.length > 1 ? 'has-dots' : ''}">
              <div class="xhs-pc-head">
                <h1 class="xhs-title">${esc(item.title)}</h1>
                ${primaryCtaHtml}
              </div>
              <div class="xhs-prose">${esc(item.body).replace(/\n\n/g, '<br><br>')}</div>
              <div class="xhs-hashtags">${tagsHtml}</div>
              ${infoStripsHtml}
              <div class="xhs-meta-row">
                <span>编辑于 ${esc(item.date)} ${esc(item.region || '社区')}</span>
                ${state.role === 'reader' ? '<button type="button" class="xhs-dislike-pill" data-action="toast" data-msg="已减少此类内容推荐">不喜欢</button>' : ''}
              </div>
            </div>
            <div class="xhs-pc-dock">
              <form class="xhs-pc-form" data-action="submit-comment" data-key="${esc(item.key)}">
                <label class="xhs-pc-input-wrap">
                  ${icon('edit', 14)}
                  <input class="comment-input" type="text" placeholder="${esc(replyPlaceholder)}" maxlength="200" required>
                </label>
                <button class="xhs-send-btn is-ready" type="submit">发送</button>
              </form>
              <div class="xhs-pc-stats">
                <button type="button" class="h5-dock-stat ${liked ? 'is-liked' : ''}" data-action="toggle-like" data-key="${esc(item.key)}">
                  ${icon('heart', 19)}<span>${likeCount}</span>
                </button>
                <button type="button" class="h5-dock-stat ${saved ? 'is-saved' : ''}" data-action="toggle-save" data-key="${esc(item.key)}">
                  ${icon('star', 19)}<span>${saveCount}</span>
                </button>
                <button type="button" class="h5-dock-stat" data-action="focus-pc-comment">
                  ${icon('chat', 18)}<span>${comments.length}</span>
                </button>
              </div>
            </div>
            <div class="xhs-pc-comments" id="detail-comments-desktop">
              ${renderDetailComments(item, comments, 'desktop')}
            </div>
          </div>
        </div>
      `;
    }

    return `
      <div class="xhs-h5-detail">
        ${activeImg ? `
          <div class="xhs-h5-media" ${gallery.length > 1 ? 'data-action="detail-slide-next"' : ''}>
            <img src="${activeImg}" alt="${esc(item.title)}">
          </div>
          ${dotsHtml(false)}
        ` : ''}
        <div class="xhs-h5-body ${gallery.length > 1 ? 'has-dots' : ''}">
          <h1 class="xhs-title">${esc(item.title)}</h1>
          ${primaryCtaHtml}
          <div class="xhs-prose">${esc(item.body).replace(/\n\n/g, '<br><br>')}</div>
          <div class="xhs-hashtags">${tagsHtml}</div>
          ${infoStripsHtml}
          <div class="xhs-meta-row">
            <span>编辑于 ${esc(item.date)} ${esc(item.region || '社区')}</span>
            ${state.role === 'reader' ? '<button type="button" class="xhs-dislike-pill" data-action="toast" data-msg="已减少此类内容推荐">不喜欢</button>' : ''}
          </div>
        </div>
        <div class="xhs-divider"></div>
        <div class="xhs-h5-comments" id="detail-comments-mobile">
          ${renderDetailComments(item, comments, 'mobile')}
        </div>
      </div>
    `;
  }

  function renderActiveContent(device) {
    if (state.detail) return renderDetailView(device);
    switch (state.page) {
      case 'ideas': return renderIdeasPage();
      case 'news': return renderNewsPage();
      case 'opensource': return renderOpenSourcePage();
      case 'rankings': return renderRankingsPage();
      case 'nav': return renderNavPage();
      case 'discuss': return renderDiscussPage();
      case 'resources': return renderResourcesPage();
      case 'apps': return renderAppsPage();
      case 'ai': return renderAiPage();
      case 'me': return renderMePage();
      default: return renderIdeasPage();
    }
  }

  /* ==========================================================================
     PC 端与移动端 H5 壳体装配
     ========================================================================== */
  function renderDesktopApp(framed) {
    const sec = sectionKeyOfPage(state.page);

    return `
      <div class="app-shell ${framed ? 'is-framed' : 'is-unframed'}"
        data-scheme="vibrant" data-device="desktop" data-theme="${state.theme}" data-accent="${state.accent}">
        <aside class="pc-rail" aria-label="主导航">
          <div class="rail-brand">
            <span class="rail-brand-mark">${brandLogoSvg(28)}</span>
            <strong>超级小伴</strong>
            <small>社区</small>
          </div>
          <button type="button" class="rail-search-btn" data-action="open-search">
            ${icon('search', 14)}<span>搜索</span><span class="search-kbd">⌘K</span>
          </button>
          <button type="button" class="rail-publish-cta" data-action="open-publish">
            ${icon('plus', 15)} <span>发布灵感</span>
          </button>
          <div class="rail-nav-scroll">
            <div class="rail-group">
              <div class="rail-label">社区</div>
              <button type="button" class="rail-link ${sec === 'home' ? 'is-active' : ''}" data-action="go-page" data-page="ideas">
                <span class="rail-icon">${icon('compass', 15)}</span><span>首页</span><span class="rail-chevron">${icon('arrow', 13)}</span>
              </button>
              ${sec === 'home' ? `
                <div class="rail-sub">
                  ${HOME_TABS.map(t => `
                    <button type="button" class="rail-sub-link ${state.page === t.key && !state.detail ? 'is-active' : ''}"
                      data-action="go-page" data-page="${t.key}">${t.label}</button>
                  `).join('')}
                </div>
              ` : ''}
              <button type="button" class="rail-link ${sec === 'discuss' ? 'is-active' : ''}" data-action="go-page" data-page="discuss">
                <span class="rail-icon">${icon('chat', 15)}</span><span>讨论</span><span class="rail-chevron">${icon('arrow', 13)}</span>
              </button>
              ${sec === 'discuss' ? `
                <div class="rail-sub">
                  ${DISCUSS_TABS.map(t => `
                    <button type="button" class="rail-sub-link ${state.discussTab === t.key && !state.detail ? 'is-active' : ''}"
                      data-action="${t.action}" data-val="${esc(t.key)}">${esc(t.label)}</button>
                  `).join('')}
                </div>
              ` : ''}
              <button type="button" class="rail-link ${sec === 'resources' ? 'is-active' : ''}" data-action="go-page" data-page="resources">
                <span class="rail-icon">${icon('folder', 15)}</span><span>资源</span><span class="rail-chevron">${icon('arrow', 13)}</span>
              </button>
              ${sec === 'resources' ? `
                <div class="rail-sub">
                  ${RESOURCE_TABS.map(t => `
                    <button type="button" class="rail-sub-link ${state.resourceTab === t.key && !state.detail ? 'is-active' : ''}"
                      data-action="${t.action}" data-val="${esc(t.key)}">${esc(t.label)}</button>
                  `).join('')}
                </div>
              ` : ''}
            </div>
            <div class="rail-divider"></div>
            <div class="rail-group">
              <div class="rail-label">产品空间</div>
              <button type="button" class="rail-link ${sec === 'apps' ? 'is-active' : ''}" data-action="go-page" data-page="apps">
                <span class="rail-icon">${icon('grid', 15)}</span><span>应用</span>
              </button>
              <button type="button" class="rail-link ${sec === 'ai' ? 'is-active' : ''}" data-action="go-page" data-page="ai">
                <span class="rail-icon">${icon('spark', 15)}</span><span>小伴 AI</span>
              </button>
              <button type="button" class="rail-link ${sec === 'me' ? 'is-active' : ''}" data-action="go-page" data-page="me">
                <span class="rail-icon">${icon('me', 15)}</span><span>我的</span>
              </button>
            </div>
          </div>
          <div class="rail-bottom">
            <div class="rail-user-bar">
              <button type="button" class="rail-user-btn" data-action="go-page" data-page="me">
                <span class="user-avatar">伴</span>
                <span class="rail-user-meta"><strong>阿伴</strong><small>240 积分</small></span>
              </button>
              <button type="button" class="rail-bell-btn" aria-label="通知" data-action="toast" data-msg="暂无新的未读回复">
                ${icon('bell', 15)}<i class="unread-dot"></i>
              </button>
            </div>
            <div class="rail-util-row">
              <button type="button" class="rail-util-btn" data-action="toggle-theme" title="切换深浅色外观">
                ${icon(state.theme === 'dark' ? 'sun' : 'moon', 14)}
                <span>${state.theme === 'dark' ? '浅色' : '深色'}</span>
              </button>
              <button type="button" class="rail-util-btn" data-action="go-page" data-page="ai" title="下载桌面 / 手机客户端">
                ${icon('download', 14)}<span>客户端</span>
              </button>
            </div>
          </div>
        </aside>

        <div class="pc-main-col">
          <div class="pc-content-scroll" data-scroll-key="pc">
            <div class="content-wrap">
              ${renderActiveContent('desktop')}
            </div>
          </div>
        </div>
        ${renderDeleteConfirmDialog('desktop')}
      </div>
    `;
  }

  function renderMobileApp() {
    const sec = sectionKeyOfPage(state.page);
    const inCommunity = ['home', 'discuss', 'resources'].includes(sec);
    const detailItem = getDetailModel();
    const subTabs = secondaryTabsOfSection(sec);

    let headerHtml = '';
    if (state.detail && detailItem) {
      const followed = state.followedAuthors.has(detailItem.author);
      const isAuthor = state.role === 'author';
      const isPinned = state.pinnedKeys.has(detailItem.key);
      headerHtml = `
        <header class="h5-header">
          <div class="h5-detail-head">
            <button type="button" class="h5-icon-btn" aria-label="返回" data-action="close-detail">${icon('back', 19)}</button>
            <div class="xhs-head-author">
              <span class="mini-avatar" style="background:${detailItem.avatarBg};width:30px;height:30px;font-size:12px">${esc(detailItem.author[0].replace('@', '').toUpperCase())}</span>
              <strong>${esc(detailItem.author)}</strong>
              ${isAuthor ? '<span class="xhs-role-pill">我</span>' : ''}
              ${isPinned ? '<span class="xhs-pin-pill">置顶</span>' : ''}
            </div>
            ${isAuthor
              ? `<button type="button" class="h5-icon-btn" aria-label="分享" data-action="toast" data-msg="已复制作品分享链接">${icon('share', 18)}</button>`
              : `<button type="button" class="xhs-follow-btn ${followed ? 'is-followed' : ''}" data-action="toggle-follow" data-author="${esc(detailItem.author)}">${followed ? '已关注' : '关注'}</button>`}
            <button type="button" class="h5-icon-btn ${state.h5MoreOpen ? 'is-open' : ''}" aria-label="更多操作" data-action="open-h5-more">${icon('more', 18)}</button>
          </div>
        </header>
      `;
    } else if (inCommunity) {
      headerHtml = `
        <header class="h5-header">
          <div class="h5-top-row">
            <button type="button" class="h5-brand-btn" data-action="go-page" data-page="ideas" aria-label="超级小伴">${brandLogoSvg(25)}</button>
            <div class="h5-primary-switch" role="navigation" aria-label="社区主切换">
              <button type="button" class="h5-primary-btn ${sec === 'home' ? 'is-active' : ''}" data-action="go-page" data-page="ideas">首页</button>
              <button type="button" class="h5-primary-btn ${sec === 'discuss' ? 'is-active' : ''}" data-action="go-page" data-page="discuss">讨论</button>
              <button type="button" class="h5-primary-btn ${sec === 'resources' ? 'is-active' : ''}" data-action="go-page" data-page="resources">资源</button>
            </div>
            <div class="h5-top-actions">
              <button type="button" class="h5-icon-btn" aria-label="搜索" data-action="open-search">${icon('search', 17)}</button>
              <button type="button" class="h5-icon-btn" aria-label="通知" data-action="toast" data-msg="有 2 条新回复">${icon('bell', 17)}<i class="unread-dot"></i></button>
            </div>
          </div>
          ${subTabs ? `
            <div class="h5-subtabs" role="tablist" aria-label="${subTabs.label}">
              ${subTabs.items.map(t => `
                <button type="button" class="h5-subtab ${subTabs.active === t.key ? 'is-active' : ''}"
                  data-action="${t.action}" ${subTabs.attr}="${esc(t.key)}">${esc(t.label)}</button>
              `).join('')}
            </div>
          ` : ''}
        </header>
      `;
    } else {
      const simpleTitle = sec === 'apps' ? '应用' : sec === 'ai' ? '小伴 AI' : '我的';
      headerHtml = `
        <header class="h5-header">
          <div class="h5-simple-head">
            <strong>${simpleTitle}</strong>
            ${sec === 'me' ? `<button type="button" class="h5-icon-btn" aria-label="切换浅深色" data-action="toggle-theme">${icon(state.theme === 'dark' ? 'sun' : 'moon', 17)}</button>` : ''}
          </div>
        </header>
      `;
    }

    let bottomBarHtml = '';
    let sheetHtml = '';
    if (state.detail && detailItem) {
      const liked = isLiked(detailItem.key);
      const saved = state.savedIds.has(detailItem.key);
      const comments = state.commentsByItem.get(detailItem.key) || INITIAL_COMMENTS;
      const likeCount = likeDelta(detailItem.key, detailItem.likes);
      const saveCount = (detailItem.saves || 18) + (saved ? 1 : 0);
      bottomBarHtml = `
        <div class="h5-comment-dock">
          <button type="button" class="h5-dock-input" data-action="open-h5-composer">
            ${icon('edit', 14)}<span>说点什么…</span>
          </button>
          <button type="button" class="h5-dock-stat ${liked ? 'is-liked' : ''}" data-action="toggle-like" data-key="${esc(detailItem.key)}">
            ${icon('heart', 19)}<span>${likeCount}</span>
          </button>
          <button type="button" class="h5-dock-stat ${saved ? 'is-saved' : ''}" data-action="toggle-save" data-key="${esc(detailItem.key)}">
            ${icon('star', 19)}<span>${saveCount}</span>
          </button>
          <button type="button" class="h5-dock-stat" data-action="scroll-h5-comments">
            ${icon('chat', 18)}<span>${comments.length}</span>
          </button>
        </div>
      `;
      if (state.h5ComposerOpen) {
        const placeholder = state.replyTo ? `回复 @${state.replyTo}…` : '留下你的想法吧';
        const emojis = ['👍', '❤️', '🔥', '😂', '✨', '🎉', '👏', '🤩'];
        sheetHtml = `
          <div class="h5-sheet-backdrop" data-action="close-h5-composer"></div>
          <form class="h5-composer-sheet" data-action="submit-comment" data-key="${esc(detailItem.key)}">
            <textarea class="h5-composer-textarea" placeholder="${esc(placeholder)}" maxlength="200" rows="3" required>${esc(state.composerDraft)}</textarea>
            <div class="h5-composer-bar">
              <div class="h5-composer-tools">
                <button type="button" class="h5-tool-icon" aria-label="图片" data-action="toast" data-msg="示例交互：选择配图">${icon('image', 19)}</button>
                <button type="button" class="h5-tool-icon" aria-label="提及" data-action="insert-emoji" data-emoji="@">${icon('at', 19)}</button>
                <button type="button" class="h5-tool-icon" aria-label="表情" data-action="insert-emoji" data-emoji="😊">${icon('smile', 19)}</button>
                <button type="button" class="h5-tool-icon" aria-label="更多" data-action="toast" data-msg="示例交互：附带作品引用">${icon('plus', 19)}</button>
              </div>
              <button type="submit" class="h5-send-btn ${state.composerDraft.trim() ? 'is-ready' : ''}">发送</button>
            </div>
            <div class="h5-emoji-bar">
              ${emojis.map(e => `<button type="button" class="h5-emoji-btn" data-action="insert-emoji" data-emoji="${e}">${e}</button>`).join('')}
            </div>
          </form>
        `;
      } else if (state.h5MoreOpen) {
        const h5MenuItems = getDetailMoreItems(detailItem, 'mobile');
        sheetHtml = `
          <div class="h5-sheet-backdrop" data-action="close-detail-popups"></div>
          <div class="h5-action-sheet" role="dialog" aria-modal="true" aria-label="更多操作">
            <div class="h5-sheet-handle" aria-hidden="true"></div>
            <div class="h5-sheet-group">
              ${h5MenuItems.map(entry => entry.divider ? '<div class="h5-sheet-divider" role="separator"></div>' : `
                <button type="button" class="h5-sheet-row ${entry.danger ? 'is-danger' : ''}"
                  data-action="${esc(entry.action)}"
                  ${entry.msg ? `data-msg="${esc(entry.msg)}"` : ''}
                  ${entry.key ? `data-key="${esc(entry.key)}"` : ''}
                  ${entry.device ? `data-device="${esc(entry.device)}"` : ''}>
                  ${icon(entry.icon, 17)}<span>${esc(entry.label)}</span>
                </button>
              `).join('')}
            </div>
            <button type="button" class="h5-sheet-cancel" data-action="close-detail-popups">取消</button>
          </div>
        `;
      }
      sheetHtml += renderDeleteConfirmDialog('mobile');
    } else {
      bottomBarHtml = `
        <nav class="h5-tabbar" aria-label="移动底栏">
          <button type="button" class="h5-tab-btn ${sec === 'ai' ? 'is-active' : ''}" data-action="go-page" data-page="ai">${icon('spark', 19)}<span>小伴AI</span></button>
          <button type="button" class="h5-tab-btn ${sec === 'apps' ? 'is-active' : ''}" data-action="go-page" data-page="apps">${icon('grid', 19)}<span>应用</span></button>
          <button type="button" class="h5-plus-btn" aria-label="发布" data-action="open-publish">${icon('plus', 19)}</button>
          <button type="button" class="h5-tab-btn ${inCommunity ? 'is-active' : ''}" data-action="go-page" data-page="ideas">${icon('compass', 19)}<span>社区</span></button>
          <button type="button" class="h5-tab-btn ${sec === 'me' ? 'is-active' : ''}" data-action="go-page" data-page="me">${icon('me', 19)}<span>我的</span></button>
        </nav>
      `;
    }

    return `
      <div class="app-shell is-framed ${state.detail && detailItem ? 'is-detail' : ''}" data-scheme="vibrant" data-device="mobile" data-theme="${state.theme}" data-accent="${state.accent}">
        ${headerHtml}
        <div class="h5-scroll" data-scroll-key="h5">
          <div class="content-wrap">
            ${renderActiveContent('mobile')}
          </div>
        </div>
        ${bottomBarHtml}
        ${sheetHtml}
      </div>
    `;
  }

  function renderPhoneUnit(showHead = true) {
    return `
      <div class="phone-unit">
        ${showHead ? `
          <div class="pane-head" style="max-width:390px">
            <div class="pane-title"><span>移动 H5 · 活力社区风</span><span class="pane-badge">390px</span></div>
            <button type="button" class="pane-expand-btn" data-set-device="mobile">单机全幅 ↗</button>
          </div>
        ` : ''}
        <div class="phone-frame">
          <div class="phone-screen" style="background:${state.theme === 'dark' ? '#14161b' : '#ffffff'};color:${state.theme === 'dark' ? '#f3f4f6' : '#111827'}">
            <div class="phone-status">
              <span>9:41</span>
              <span class="phone-notch" aria-hidden="true"></span>
              <span>5G</span>
            </div>
            <div style="flex:1;min-height:0;overflow:hidden">
              ${renderMobileApp()}
            </div>
          </div>
        </div>
      </div>
    `;
  }

  /* ==========================================================================
     工作台主渲染与缩放控制
     ========================================================================== */
  const scrollCache = new Map();
  let scaleObserver = null;

  function applyScaleHosts() {
    document.querySelectorAll('.pc-scale-host').forEach(host => {
      const stage = host.querySelector('.pc-scale-stage');
      if (!stage) return;
      const w = host.clientWidth;
      const h = host.clientHeight;
      if (!w || !h) return;
      const scale = Math.min(1, w / 1280);
      const vh = Math.max(600, Math.round(h / scale));
      stage.style.width = '1280px';
      stage.style.height = `${vh}px`;
      stage.style.transform = `scale(${scale})`;
    });
  }

  function captureScroll() {
    document.querySelectorAll('[data-scroll-key]').forEach(el => {
      scrollCache.set(el.getAttribute('data-scroll-key'), el.scrollTop);
    });
  }

  function restoreScroll(resetToTop) {
    document.querySelectorAll('[data-scroll-key]').forEach(el => {
      const key = el.getAttribute('data-scroll-key');
      el.scrollTop = resetToTop ? 0 : (scrollCache.get(key) || 0);
    });
  }

  function renderWorkbenchState() {
    document.body.dataset.wbTheme = state.theme;
    document.body.classList.toggle('wb-collapsed', state.collapsed);
    const bar = document.getElementById('workbench-bar');
    const restore = document.getElementById('wb-restore-btn');
    if (bar) bar.hidden = state.collapsed;
    if (restore) restore.hidden = !state.collapsed;

    document.querySelectorAll('[data-set-device]').forEach(btn => {
      btn.classList.toggle('is-active', btn.getAttribute('data-set-device') === state.device);
    });
    document.querySelectorAll('[data-set-role]').forEach(btn => {
      btn.classList.toggle('is-active', btn.getAttribute('data-set-role') === state.role);
    });
    document.querySelectorAll('[data-set-accent]').forEach(btn => {
      btn.classList.toggle('is-active', btn.getAttribute('data-set-accent') === state.accent);
    });

    const pageSel = document.getElementById('wb-page-select');
    if (pageSel) pageSel.value = state.page;

    const themeBtn = document.getElementById('wb-theme-btn');
    const themeIcon = document.getElementById('wb-theme-icon');
    const themeLabel = document.getElementById('wb-theme-label');
    if (themeBtn) themeBtn.setAttribute('aria-pressed', String(state.theme === 'dark'));
    if (themeIcon) themeIcon.innerHTML = icon(state.theme === 'dark' ? 'moon' : 'sun', 14);
    if (themeLabel) themeLabel.textContent = state.theme === 'dark' ? '夜间' : '日间';
  }

  function renderOverlay() {
    const root = document.getElementById('overlay-root');
    if (!root) return;
    if (!state.overlay) {
      root.innerHTML = '';
      return;
    }

    if (state.overlay === 'spec') {
      root.innerHTML = `
        <div class="wb-modal-backdrop" data-action="backdrop-close">
          <div class="wb-modal" role="dialog" aria-modal="true" aria-label="设计说明">
            <div class="wb-modal-head">
              <h2>超级小伴社区 · 活力社区风设计说明</h2>
              <button type="button" class="wb-close" data-action="close-overlay" aria-label="关闭">×</button>
            </div>
            <p style="margin:0 0 8px;opacity:0.85">本设计稿围绕「现代活力视觉社区风」重构 PC 端网页与移动端 H5 体验，兼顾内容浏览效率与视觉社区质感。</p>
            <table class="spec-table">
              <thead>
                <tr><th>模块 / 维度</th><th>设计要点</th></tr>
              </thead>
              <tbody>
                <tr><td>PC 端布局</td><td>左侧毛玻璃导航轨（社区直达 + 产品空间 + 主题切换）+ 顶部单行栏（频道药丸 Tab、居中搜索触发条、积分与通知）</td></tr>
                <tr><td>移动 H5 顶栏</td><td>Logo 居左、「首页 / 讨论 / 资源」严格居中、搜索与通知纯图标无背景底色收于右上角</td></tr>
                <tr><td>创意首页</td><td>2:1 沉浸式视觉主推矩阵 + 4 列（H5 双列）统一 3:2 封面视觉卡片流与可点击小红心</td></tr>
                <tr><td>讨论与资源</td><td>独立浮起圆角卡片（16px 圆角 + 柔和阴影）、彩色板块胶囊与两行正文摘要；开源精选按有图与紧凑项目分区排布</td></tr>
                <tr><td>详情页体验入口</td><td>将核心体验/访问/下载操作从灰底信息条提升为品牌实心胶囊主按钮（PC 与标题同行居右 42px，H5 标题下通栏 46px 保证首屏可见），「说明」等辅助条回退为中性弱色避免视觉抢镜</td></tr>
                <tr><td>主题与双模式</td><td>完整保留客户端 6 套主题色（小伴红 / 薄荷绿 / 海湾蓝 / 暮紫 / 琥珀 / 石墨）与高对比度日夜双模式</td></tr>
              </tbody>
            </table>
          </div>
        </div>
      `;
      return;
    }

    if (state.overlay === 'publish') {
      root.innerHTML = `
        <div class="wb-modal-backdrop" data-action="backdrop-close">
          <div class="wb-modal" style="max-width:440px" role="dialog" aria-modal="true" aria-label="发布入口">
            <div class="wb-modal-head">
              <h2>分享一点新鲜事</h2>
              <button type="button" class="wb-close" data-action="close-overlay" aria-label="关闭">×</button>
            </div>
            <div style="display:grid;gap:10px">
              <button type="button" class="search-result-btn" style="border:1px solid rgba(128,128,128,0.2);padding:14px" data-action="toast-close" data-msg="已进入发帖编辑器（设计稿演示）">
                <div><strong>发布讨论帖</strong><div style="font-size:12px;opacity:0.7">交流想法、分享桌面经验或发起悬赏求助</div></div>
                <span>→</span>
              </button>
              <button type="button" class="search-result-btn" style="border:1px solid rgba(128,128,128,0.2);padding:14px" data-action="toast-close" data-msg="已进入作品上传弹窗：支持单 HTML 或 V2 ZIP">
                <div><strong>分享已有作品</strong><div style="font-size:12px;opacity:0.7">上传单个 HTML 或小伴桌面导出 V2 ZIP 包</div></div>
                <span>→</span>
              </button>
            </div>
          </div>
        </div>
      `;
      return;
    }

    if (state.overlay === 'search') {
      root.innerHTML = `
        <div class="wb-modal-backdrop" data-action="backdrop-close">
          <div class="wb-modal" style="max-width:560px" role="dialog" aria-modal="true" aria-label="搜索社区">
            <div class="wb-modal-head">
              <h2>搜索社区内容</h2>
              <button type="button" class="wb-close" data-action="close-overlay" aria-label="关闭">×</button>
            </div>
            <input id="wb-search-field" class="wb-search-input" type="search" value="${esc(state.searchQuery)}" placeholder="输入关键词，例如：鲸鱼娘、Excalidraw、小组件…" autocomplete="off">
            <div id="wb-search-results" style="display:grid;gap:4px"></div>
          </div>
        </div>
      `;
      renderSearchResults();
      const input = document.getElementById('wb-search-field');
      if (input) {
        input.focus();
        input.setSelectionRange(input.value.length, input.value.length);
      }
    }
  }

  function renderSearchResults() {
    const container = document.getElementById('wb-search-results');
    if (!container) return;
    const q = state.searchQuery.trim().toLowerCase();
    const hits = [
      ...WORKS.map(w => ({ kind: '作品', title: w.title, action: `open-work:${w.id}` })),
      ...DISCUSS_TOPICS.map(t => ({ kind: '讨论', title: t.title, action: `open-topic:${t.id}` })),
      ...OPEN_SOURCE_PROJECTS.map(p => ({ kind: '开源', title: `${p.name} — ${p.summary}`, action: `open-os:${p.full_name}` }))
    ].filter(item => !q || item.title.toLowerCase().includes(q)).slice(0, 8);

    container.innerHTML = hits.length ? hits.map(h => `
      <button type="button" class="search-result-btn" data-action="search-pick" data-target="${esc(h.action)}">
        <span style="overflow:hidden;text-overflow:ellipsis;white-space:nowrap">${esc(h.title)}</span>
        <span class="wb-tag">${esc(h.kind)}</span>
      </button>
    `).join('') : '<p style="opacity:0.65;text-align:center;padding:18px 0">没有找到匹配的示例内容</p>';
  }

  function render(resetScroll = false) {
    captureScroll();
    renderWorkbenchState();
    syncUrl();

    const stage = document.getElementById('stage');
    if (!stage) return;

    if (state.device === 'desktop') {
      stage.innerHTML = `<div class="single-desktop-wrap">${renderDesktopApp(false)}</div>`;
    } else if (state.device === 'mobile') {
      stage.innerHTML = `<div class="phone-stage is-single-mobile">${renderPhoneUnit(false)}</div>`;
    } else {
      stage.innerHTML = `
        <div class="both-grid">
          <div class="compare-pane">
            <div class="pane-head">
              <div class="pane-title"><span>PC 端网页 · 活力社区风</span></div>
              <button type="button" class="pane-expand-btn" data-set-device="desktop">全幅展开 ↗</button>
            </div>
            <div class="pc-scale-host">
              <div class="pc-scale-stage">${renderDesktopApp(true)}</div>
            </div>
          </div>
          ${renderPhoneUnit(true)}
        </div>
      `;
    }

    if (scaleObserver) scaleObserver.disconnect();
    const hosts = document.querySelectorAll('.pc-scale-host');
    if (hosts.length && typeof ResizeObserver !== 'undefined') {
      scaleObserver = new ResizeObserver(applyScaleHosts);
      hosts.forEach(h => scaleObserver.observe(h));
    }
    applyScaleHosts();
    restoreScroll(resetScroll);
    renderOverlay();
  }

  /* ==========================================================================
     全局交互事件委托
     ========================================================================== */
  document.addEventListener('click', event => {
    if (state.pcMoreOpen && !event.target.closest('.xhs-more-wrap')) {
      state.pcMoreOpen = false;
      if (!event.target.closest('[data-action], [data-set-device], [data-set-accent], [data-set-role]')) {
        render(false);
        return;
      }
    }

    const btn = event.target.closest('[data-action], [data-set-device], [data-set-accent], [data-set-role]');
    if (!btn) return;

    if (btn.hasAttribute('data-set-device')) {
      state.device = btn.getAttribute('data-set-device');
      state.pcMoreOpen = false;
      state.h5MoreOpen = false;
      state.deleteConfirm = null;
      render(false);
      return;
    }
    if (btn.hasAttribute('data-set-role')) {
      const nextRole = btn.getAttribute('data-set-role');
      if (VALID_ROLES.has(nextRole)) {
        state.role = nextRole;
        state.pcMoreOpen = false;
        state.h5MoreOpen = false;
        state.deleteConfirm = null;
        render(false);
        if (!state.detail) {
          const labels = { reader: '读者视角', author: '作者视角', admin: '管理员视角' };
          showToast(`已切换为「${labels[nextRole]}」，点开任意内容可查看详情页菜单`);
        }
      }
      return;
    }
    if (btn.hasAttribute('data-set-accent')) {
      state.accent = btn.getAttribute('data-set-accent');
      render(false);
      return;
    }

    const action = btn.getAttribute('data-action');
    switch (action) {
      case 'go-page':
        state.detail = null;
        state.detailSlide = 0;
        state.pcMoreOpen = false;
        state.h5MoreOpen = false;
        state.deleteConfirm = null;
        state.h5ComposerOpen = false;
        state.replyTo = '';
        state.composerDraft = '';
        state.page = btn.getAttribute('data-page');
        render(true);
        break;
      case 'os-cat':
        state.osCategory = btn.getAttribute('data-val');
        render(false);
        break;
      case 'nav-cat':
        state.navCat = btn.getAttribute('data-val');
        render(false);
        break;
      case 'apps-filter':
        state.appsFeedFilter = btn.getAttribute('data-val');
        render(false);
        break;
      case 'apps-toggle-games':
        state.appsAllGames = !state.appsAllGames;
        render(false);
        break;
      case 'apps-toggle-tools':
        state.appsAllTools = !state.appsAllTools;
        render(false);
        break;
      case 'discuss-tab': {
        const changedPage = state.page !== 'discuss' || state.detail;
        state.detail = null;
        state.page = 'discuss';
        state.discussTab = btn.getAttribute('data-val');
        render(Boolean(changedPage));
        break;
      }
      case 'res-tab': {
        const changedPage = state.page !== 'resources' || state.detail;
        state.detail = null;
        state.page = 'resources';
        state.resourceTab = btn.getAttribute('data-val');
        render(Boolean(changedPage));
        break;
      }
      case 'sw-cat':
        state.softwareCat = btn.getAttribute('data-val');
        render(false);
        break;
      case 'sw-sort':
        state.softwareSort = btn.getAttribute('data-val');
        render(false);
        break;
      case 'frontier-dot':
        state.frontierIndex = Number(btn.getAttribute('data-idx')) || 0;
        render(false);
        break;
      case 'open-frontier': {
        const idx = Number(btn.getAttribute('data-idx')) || 0;
        const f = FRONTIER_ITEMS[idx];
        showToast(`阅读前沿：${f ? f.title : ''}`);
        break;
      }
      case 'open-news': {
        const key = btn.getAttribute('data-key');
        const n = DAILY_NEWS.find(item => item.key === key);
        showToast(`打开 AI 日报：${n ? n.title : ''}`);
        break;
      }
      case 'select-model':
        state.selectedModelId = btn.getAttribute('data-id');
        render(false);
        break;
      case 'open-work':
        state.detail = { type: 'work', id: btn.getAttribute('data-id') };
        state.detailSlide = 0;
        state.pcMoreOpen = false;
        state.h5MoreOpen = false;
        state.deleteConfirm = null;
        state.h5ComposerOpen = false;
        state.replyTo = '';
        state.composerDraft = '';
        render(true);
        break;
      case 'open-topic':
        state.detail = { type: 'topic', id: Number(btn.getAttribute('data-id')) };
        state.detailSlide = 0;
        state.pcMoreOpen = false;
        state.h5MoreOpen = false;
        state.deleteConfirm = null;
        state.h5ComposerOpen = false;
        state.replyTo = '';
        state.composerDraft = '';
        render(true);
        break;
      case 'open-os':
        state.detail = { type: 'project', id: btn.getAttribute('data-slug') };
        state.detailSlide = 0;
        state.pcMoreOpen = false;
        state.h5MoreOpen = false;
        state.deleteConfirm = null;
        state.h5ComposerOpen = false;
        state.replyTo = '';
        state.composerDraft = '';
        render(true);
        break;
      case 'close-detail':
        state.detail = null;
        state.detailSlide = 0;
        state.pcMoreOpen = false;
        state.h5MoreOpen = false;
        state.deleteConfirm = null;
        state.h5ComposerOpen = false;
        state.replyTo = '';
        state.composerDraft = '';
        render(true);
        break;
      case 'toggle-pc-more':
        state.pcMoreOpen = !state.pcMoreOpen;
        state.deleteConfirm = null;
        render(false);
        break;
      case 'open-h5-more':
        state.h5MoreOpen = true;
        state.h5ComposerOpen = false;
        state.deleteConfirm = null;
        render(false);
        break;
      case 'close-detail-popups':
        if (btn.classList.contains('xhs-confirm-backdrop') && event.target !== btn) break;
        state.pcMoreOpen = false;
        state.h5MoreOpen = false;
        state.deleteConfirm = null;
        render(false);
        break;
      case 'detail-more-toast':
        state.pcMoreOpen = false;
        state.h5MoreOpen = false;
        render(false);
        showToast(btn.getAttribute('data-msg') || '演示操作');
        break;
      case 'toggle-pin-detail': {
        const key = btn.getAttribute('data-key');
        state.pcMoreOpen = false;
        state.h5MoreOpen = false;
        if (key) {
          if (state.pinnedKeys.has(key)) {
            state.pinnedKeys.delete(key);
            showToast('已取消置顶');
          } else {
            state.pinnedKeys.add(key);
            showToast('已置顶当前内容');
          }
        }
        render(false);
        break;
      }
      case 'ask-delete-post': {
        const item = getDetailModel();
        if (!item) break;
        const device = btn.getAttribute('data-device') || (btn.closest('[data-device="mobile"]') ? 'mobile' : 'desktop');
        state.pcMoreOpen = false;
        state.h5MoreOpen = false;
        state.deleteConfirm = { device, target: 'post', key: item.key, title: item.title };
        render(false);
        break;
      }
      case 'ask-delete-comment': {
        const item = getDetailModel();
        if (!item) break;
        const rawCid = btn.getAttribute('data-cid');
        const cid = Number(rawCid) || rawCid;
        const list = state.commentsByItem.get(item.key) || INITIAL_COMMENTS;
        const target = list.find(c => String(c.id) === String(cid));
        if (!target) break;
        const device = btn.getAttribute('data-device') || (btn.closest('[data-device="mobile"]') ? 'mobile' : 'desktop');
        state.pcMoreOpen = false;
        state.h5MoreOpen = false;
        state.deleteConfirm = {
          device,
          target: 'comment',
          key: item.key,
          commentId: cid,
          title: `${target.author} 的评论「${target.text.slice(0, 22)}${target.text.length > 22 ? '…' : ''}」`
        };
        render(false);
        break;
      }
      case 'confirm-delete': {
        const cfg = state.deleteConfirm;
        if (!cfg) break;
        state.deleteConfirm = null;
        if (cfg.target === 'comment') {
          const list = [...(state.commentsByItem.get(cfg.key) || INITIAL_COMMENTS)];
          const nextList = list.filter(c => String(c.id) !== String(cfg.commentId));
          state.commentsByItem.set(cfg.key, nextList);
          render(false);
          showToast('已删除该条评论（演示）');
        } else {
          state.detail = null;
          state.detailSlide = 0;
          state.pcMoreOpen = false;
          state.h5MoreOpen = false;
          state.h5ComposerOpen = false;
          state.replyTo = '';
          state.composerDraft = '';
          render(true);
          showToast(`已删除「${cfg.title}」（设计稿演示，内容未真实删除）`);
        }
        break;
      }
      case 'detail-slide':
        state.detailSlide = Number(btn.getAttribute('data-idx')) || 0;
        render(false);
        break;
      case 'detail-slide-prev':
      case 'detail-slide-next': {
        const len = (getDetailModel()?.gallery || []).length;
        if (len > 1) {
          const step = action === 'detail-slide-next' ? 1 : len - 1;
          state.detailSlide = (state.detailSlide + step) % len;
          render(false);
        }
        break;
      }
      case 'toggle-follow': {
        const author = btn.getAttribute('data-author');
        if (!author) break;
        if (state.followedAuthors.has(author)) {
          state.followedAuthors.delete(author);
          showToast(`已取消关注 ${author}`);
        } else {
          state.followedAuthors.add(author);
          showToast(`已关注 ${author}`);
        }
        render(false);
        break;
      }
      case 'toggle-comment-like': {
        const raw = btn.getAttribute('data-cid');
        const cid = Number(raw) || raw;
        if (state.likedCommentIds.has(cid)) state.likedCommentIds.delete(cid);
        else state.likedCommentIds.add(cid);
        render(false);
        break;
      }
      case 'open-h5-composer':
        state.h5ComposerOpen = true;
        state.replyTo = '';
        render(false);
        setTimeout(() => document.querySelector('.h5-composer-textarea')?.focus(), 30);
        break;
      case 'close-h5-composer':
        state.h5ComposerOpen = false;
        state.replyTo = '';
        render(false);
        break;
      case 'reply-comment': {
        const targetAuthor = btn.getAttribute('data-author') || '';
        const inMobile = Boolean(btn.closest('[data-device="mobile"]'));
        state.replyTo = targetAuthor;
        if (inMobile) {
          state.h5ComposerOpen = true;
          render(false);
          setTimeout(() => document.querySelector('.h5-composer-textarea')?.focus(), 30);
        } else {
          render(false);
          setTimeout(() => document.querySelector('[data-device="desktop"] .comment-input')?.focus(), 30);
        }
        break;
      }
      case 'insert-emoji': {
        const em = btn.getAttribute('data-emoji') || '';
        state.composerDraft += em;
        const ta = document.querySelector('.h5-composer-textarea');
        if (ta) {
          ta.value = state.composerDraft;
          ta.focus();
          ta.closest('form')?.querySelector('.h5-send-btn')?.classList.toggle('is-ready', Boolean(state.composerDraft.trim()));
        }
        break;
      }
      case 'scroll-h5-comments': {
        const el = btn.closest('.app-shell')?.querySelector('#detail-comments-mobile');
        el?.scrollIntoView({ behavior: 'smooth', block: 'start' });
        break;
      }
      case 'focus-pc-comment': {
        const input = btn.closest('.app-shell')?.querySelector('.comment-input');
        input?.focus();
        break;
      }
      case 'toggle-like': {
        const key = btn.getAttribute('data-key');
        if (state.likedIds.has(key)) state.likedIds.delete(key);
        else state.likedIds.add(key);
        render(false);
        break;
      }
      case 'toggle-save': {
        const key = btn.getAttribute('data-key');
        if (state.savedIds.has(key)) {
          state.savedIds.delete(key);
          showToast('已取消收藏');
        } else {
          state.savedIds.add(key);
          showToast('已加入收藏');
        }
        render(false);
        break;
      }
      case 'toggle-theme':
        state.theme = state.theme === 'dark' ? 'light' : 'dark';
        render(false);
        break;
      case 'open-search':
        state.overlay = 'search';
        renderOverlay();
        break;
      case 'open-publish':
        state.overlay = 'publish';
        renderOverlay();
        break;
      case 'backdrop-close':
        if (event.target === btn) {
          state.overlay = null;
          renderOverlay();
        }
        break;
      case 'close-overlay':
        state.overlay = null;
        renderOverlay();
        break;
      case 'toast':
        event.preventDefault();
        showToast(btn.getAttribute('data-msg') || '演示操作');
        break;
      case 'toast-close':
        state.overlay = null;
        renderOverlay();
        showToast(btn.getAttribute('data-msg') || '演示操作');
        break;
      case 'search-pick': {
        const [kind, val] = (btn.getAttribute('data-target') || '').split(':');
        state.overlay = null;
        state.detailSlide = 0;
        state.h5ComposerOpen = false;
        state.replyTo = '';
        state.composerDraft = '';
        if (kind === 'open-work') state.detail = { type: 'work', id: val };
        if (kind === 'open-topic') state.detail = { type: 'topic', id: Number(val) };
        if (kind === 'open-os') state.detail = { type: 'project', id: val };
        render(true);
        break;
      }
    }
  });

  document.addEventListener('submit', event => {
    const form = event.target.closest('[data-action="submit-comment"]');
    if (!form) return;
    event.preventDefault();
    const field = form.querySelector('input, textarea');
    const raw = field?.value.trim();
    if (!raw) return;
    const text = state.replyTo ? `回复 @${state.replyTo}：${raw}` : raw;
    const key = form.getAttribute('data-key');
    const list = [...(state.commentsByItem.get(key) || INITIAL_COMMENTS)];
    list.unshift({ id: Date.now(), author: '阿伴创作者（我）', avatarBg: '#D93A44', time: '刚刚', region: '社区', likes: 0, text });
    state.commentsByItem.set(key, list);
    state.h5ComposerOpen = false;
    state.replyTo = '';
    state.composerDraft = '';
    render(false);
    showToast('评论已发布');
  });

  document.addEventListener('input', event => {
    if (event.target.id === 'wb-search-field') {
      state.searchQuery = event.target.value;
      renderSearchResults();
    } else if (event.target.classList?.contains('h5-composer-textarea')) {
      state.composerDraft = event.target.value;
      event.target.closest('form')?.querySelector('.h5-send-btn')?.classList.toggle('is-ready', Boolean(state.composerDraft.trim()));
    }
  });

  document.addEventListener('keydown', event => {
    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
      event.preventDefault();
      state.overlay = state.overlay === 'search' ? null : 'search';
      renderOverlay();
    } else if (event.key === 'Escape') {
      if (state.overlay) {
        state.overlay = null;
        renderOverlay();
      } else if (state.deleteConfirm || state.pcMoreOpen || state.h5MoreOpen || state.h5ComposerOpen) {
        state.deleteConfirm = null;
        state.pcMoreOpen = false;
        state.h5MoreOpen = false;
        state.h5ComposerOpen = false;
        render(false);
      }
    }
  });

  document.getElementById('wb-page-select')?.addEventListener('change', event => {
    state.detail = null;
    state.page = event.target.value;
    render(true);
  });

  document.getElementById('wb-theme-btn')?.addEventListener('click', () => {
    state.theme = state.theme === 'dark' ? 'light' : 'dark';
    render(false);
  });

  document.getElementById('wb-spec-btn')?.addEventListener('click', () => {
    state.overlay = state.overlay === 'spec' ? null : 'spec';
    renderOverlay();
  });

  document.getElementById('wb-collapse-btn')?.addEventListener('click', () => {
    state.collapsed = true;
    render(false);
  });

  document.getElementById('wb-restore-btn')?.addEventListener('click', () => {
    state.collapsed = false;
    render(false);
  });

  window.addEventListener('resize', applyScaleHosts);

  // 资讯页前沿轮播自动切换（每 6 秒）
  setInterval(() => {
    if (state.page === 'news' && !state.detail && !state.overlay && !document.hidden) {
      state.frontierIndex = (state.frontierIndex + 1) % FRONTIER_ITEMS.length;
      render(false);
    }
  }, 6000);

  render(true);
})();
