export const experts=[
{id:'research',name:'深度研究专家',role:'研究分析',desc:'从资料检索到观点对比，帮你完成系统化调研。',cat:'研究',skills:['网页总结','数据分析'],connectors:['Google Drive'],subtitle:'适合研究与信息整合',tags:['研究','报告'],intro:'你好，我是深度研究专家。告诉我研究主题、目标和你最在意的结论，我会帮你规划调研框架。'},
{id:'code',name:'编程开发专家',role:'工程开发',desc:'阅读代码、分析问题并协助设计与实现软件。',cat:'开发',skills:['代码审查','自动化工作流'],connectors:['GitHub'],subtitle:'适合开发任务与代码审查',tags:['编程','技术'],intro:'你好，我是编程开发专家。请告诉我项目结构、技术栈或遇到的具体问题。'},
{id:'design',name:'视觉设计专家',role:'设计创作',desc:'把抽象想法转为清晰的产品视觉与交互方案。',cat:'创作',skills:['海报设计','图片去背景'],connectors:['Figma'],subtitle:'适合界面与视觉设计',tags:['UI','设计'],intro:'你好，我是视觉设计专家。我们可以从目标用户、使用场景和视觉方向开始。'},
{id:'writer',name:'内容创作专家',role:'文字与传播',desc:'辅助文案写作、内容策划和表达优化。',cat:'创作',skills:['视频转文案','网页总结'],connectors:['Notion'],subtitle:'适合内容创作与传播',tags:['文案','创作'],intro:'你好，我是内容创作专家。告诉我平台、受众、风格和想传达的关键信息。'},
{id:'coach',name:'职场成长教练',role:'职场发展',desc:'围绕职业目标与个人成长提供行动建议。',cat:'生活',skills:['会议纪要'],connectors:['飞书'],subtitle:'适合职业规划与复盘',tags:['职场','成长'],intro:'你好，我是职场成长教练。你目前最希望解决的是职业选择、沟通协作还是能力提升？'},
{id:'learning',name:'学习规划顾问',role:'学习提升',desc:'制定学习路线，拆分知识重点并跟踪学习节奏。',cat:'生活',skills:['网页总结'],connectors:['Notion'],subtitle:'适合知识学习与复习',tags:['学习','规划'],intro:'你好，我是学习规划顾问。告诉我你希望掌握的主题以及每周可投入的时间。'}
];

export const connectors=[
{id:'github',name:'GitHub',cat:'开发',desc:'读取仓库、Issue 与 Pull Request，辅助开发协作',kind:'代码协作',scopes:['读取仓库与代码','读取 Issue / Pull Request','按授权写入代码变更']},
{id:'feishu',name:'飞书',cat:'办公',desc:'连接文档、知识库与团队协作信息',kind:'团队协作',scopes:['读取文档','查看共享知识库','按授权创建内容']},
{id:'notion',name:'Notion',cat:'办公',desc:'使用页面和数据库，管理个人及团队知识',kind:'知识管理',scopes:['读取选定页面','搜索知识库','写入用户授权页面']},
{id:'drive',name:'Google Drive',cat:'办公',desc:'读取与整理云端文件，让智能体理解你的资料',kind:'云端文件',scopes:['查看授权范围内文件','读取选中的文件','按授权创建文件']},
{id:'figma',name:'Figma',cat:'创作',desc:'连接设计文件，协助评审界面与设计交付',kind:'设计协作',scopes:['查看设计文件','读取节点数据','按授权发表评论']},
{id:'slack',name:'Slack',cat:'办公',desc:'检索消息和频道，提高团队信息获取效率',kind:'沟通协作',scopes:['查看已授权频道','检索消息','按授权发送消息']},
{id:'gmail',name:'Gmail',cat:'办公',desc:'协助检索邮件、整理沟通及准备回复草稿',kind:'邮件',scopes:['查看授权邮件','创建邮件草稿','按授权发送邮件']},
{id:'linear',name:'Linear',cat:'开发',desc:'读取项目事项、更新任务和跟进研发进度',kind:'项目管理',scopes:['查看项目和 Issue','读取任务详情','按授权更新任务']}
];

export type Expert = typeof experts[number];
export type Connector = typeof connectors[number];
export interface Skill { slug: string; name: string; description: string; version: string; source?: string; removable?: boolean }
export interface SkillPage { results: Skill[]; page: number; total: number | null; hasMore: boolean }
export interface SkillReview {
  slug: string; sha256: string; content: string;
  manifest: { version: string; fileCount: number; totalBytes: number; files: { path: string; size: number; sha256: string; executableScript: boolean }[] };
  scripts: { path: string; sha256: string; preview: string | null; previewTruncated: boolean }[];
}

