import type { TestDefinition } from '../engine/types.ts';

export const rpgCardTest: TestDefinition = {
  id: 'rpg',
  title: '人生 RPG 角色卡',
  shortTitle: 'RPG角色卡',
  subtitle: '复古像素属性面板 + 被动技能 & Debuff',
  heroDesc: '如果人生是一场开放世界 RPG，你的初始职业是什么？测出你的 STR/INT/CHA/LUCK 四维数值、神级被动与致命 Debuff！',
  category: 'viral',
  badge: '玩家必测',
  estimatedMinutes: 2,
  cardBg: '#6BE5A8',
  posterBg: '#6BE5A8',
  tagColor: '#6BE5A8',
  accentColor: '#19231E',
  scoringMode: 'nearest-profile',
  dimensions: [
    { id: 'str', label: 'STR 行动抗压' },
    { id: 'int', label: 'INT 洞察脑力' },
    { id: 'cha', label: 'CHA 社交魅力' },
    { id: 'luck', label: 'LUCK 佛系欧气' }
  ],
  microHints: [
    '✦ 正在读取你的角色初始加点…',
    '✦ 已检测到一项隐藏被动技能',
    '✦ 正在扫描你的致命 Debuff 触发条件…',
    '✦ 玩家属性面板即将加载完毕！'
  ],
  questions: [
    {
      id: 'rp1', tag: '新手村加点', title: '开局系统只给你 10 点自由属性，你第一反应会优先砸给？', sub: '直觉暴露你的核心流派',
      options: [
        { key: 'A', text: '全点智力 INT：脑子够快，一切机制都能靠套路通关', sub: '法师/策士流派', weights: { int: 3 } },
        { key: 'B', text: '全点体力抗压 STR：血厚能扛，正面硬推一切 Boss', sub: '战士/骑士流派', weights: { str: 3 } },
        { key: 'C', text: '全点魅力 CHA：嘴甜会组队，靠队友带飞最香', sub: '吟游诗人流派', weights: { cha: 3 } },
        { key: 'D', text: '全点幸运 LUCK：谋事在人成事在天，欧皇无视数值！', sub: '天选欧皇流派', weights: { luck: 3 } }
      ]
    },
    {
      id: 'rp2', tag: '遭遇 Boss', title: '工作或学习里突然刷出一只超难缠的“终极 Boss 级难题”，你会？', sub: '团战输出习惯',
      options: [
        { key: 'A', text: '先绕后观察三分钟摸清攻击规律，一击打中弱点', sub: '高洞察刺客/法师', weights: { int: 3 } },
        { key: 'B', text: '深吸一口气直接顶盾冲上去，边打边抗直到磨死它', sub: '钢铁意志前排', weights: { str: 3 } },
        { key: 'C', text: '公屏摇大佬组 5 人团，分工拉满效率翻倍', sub: '团队协调核心', weights: { cha: 3 } },
        { key: 'D', text: '先回城泡个温泉睡一觉，说不定明天版本更新它自己削弱了', sub: '佛系随缘大师', weights: { luck: 3 } }
      ]
    },
    {
      id: 'rp3', tag: '蓝条续航', title: '连续高强度社交或连开三场大会后，你的角色状态栏会弹出？', sub: 'Debuff 检测',
      options: [
        { key: 'A', text: '【蓝条归零】触发 24 小时强制静音自闭结界', sub: '脆皮法师典型弱点', weights: { int: 2, cha: -2, luck: 1 } },
        { key: 'B', text: '【越战越勇】肾上腺素飙升，甚至还能顺手把复盘写了', sub: '重装战神耐力', weights: { str: 3 } },
        { key: 'C', text: '【魅力充能】跟大家聊得正嗨，顺便约了晚饭续摊', sub: '社交永动机', weights: { cha: 3 } },
        { key: 'D', text: '【原地挂机】人还在工位，灵魂已经切去野外看风景', sub: '自动巡航省电', weights: { luck: 3 } }
      ]
    },
    {
      id: 'rp4', tag: '支线任务', title: '玩开放世界游戏（或度过周末），你的真实游玩路线更像？', sub: '探索倾向',
      options: [
        { key: 'A', text: '主线先放一边，把地图上所有奇葩支线和彩蛋全摸一遍', sub: '好奇心拉满探索者', weights: { luck: 2, int: 2 } },
        { key: 'B', text: '直推主线速通升级，绝不在无关 NPC 身上浪费时间', sub: '效率推进狂魔', weights: { str: 3, int: 1 } },
        { key: 'C', text: '专挑能跟 NPC 聊天、买房换装、拍照打卡的玩法', sub: '氛围感社交玩家', weights: { cha: 3 } },
        { key: 'D', text: '打开攻略研究极限配装公式，把数值调到最完美', sub: '硬核研究型大脑', weights: { int: 3 } }
      ]
    },
    {
      id: 'rp5', tag: '背包管理', title: '看看你现实中的包/电脑桌面，最像哪种 RPG 背包？', sub: '秩序 vs 随性',
      options: [
        { key: 'A', text: '分类强迫症：武器、药水、任务道具码得整整齐齐', sub: '高智商条理控', weights: { int: 2, str: 2 } },
        { key: 'B', text: '塞满各种“万一用得上”的补给品和奇奇怪怪小物件', sub: '仓鼠型安心感', weights: { luck: 2, cha: 1 } },
        { key: 'C', text: '极简轻装上阵，只带最核心的几件神装', sub: '干脆利落行动派', weights: { str: 3 } },
        { key: 'D', text: '表面乱糟糟但总能在关键时刻掏出刚好救命的道具', sub: '自带幸运光环', weights: { luck: 3 } }
      ]
    },
    {
      id: 'rp6', tag: '队伍定位', title: '如果拉你进一个 4 人副本小队，你最自然会站到哪个位置？', sub: '天生职业定位',
      options: [
        { key: 'A', text: '后排军师/法师：看清全场机制，关键时刻放大招定胜负', sub: '智慧输出位', weights: { int: 3 } },
        { key: 'B', text: '前排坦克/骑士：稳稳扛住压力，让队友安心发挥', sub: '靠谱兜底位', weights: { str: 3 } },
        { key: 'C', text: '全能辅助/诗人：给全队加 Buff 顺便调解内部矛盾', sub: '气氛与凝聚核心', weights: { cha: 3 } },
        { key: 'D', text: '自由游侠/摸金客：负责开宝箱触发奇遇，偶尔神来一笔', sub: '自由幸运位', weights: { luck: 3 } }
      ]
    },
    {
      id: 'rp7', tag: '抽卡玄学', title: '面对低概率但回报巨大的机会（抽稀有卡/冲梦中情岗），你会？', sub: '风险偏好',
      options: [
        { key: 'A', text: '先算清数学期望和保底机制，绝不盲目上头', sub: '绝对理性计算', weights: { int: 3 } },
        { key: 'B', text: '管它概率多少，靠肝靠拼硬把保底肝出来！', sub: '卷死系统的执行力', weights: { str: 3 } },
        { key: 'C', text: '找手气最好的朋友帮我按下去，沾沾欧气', sub: '社交互动加成', weights: { cha: 2, luck: 2 } },
        { key: 'D', text: '随手一抽居然单发出奇迹——平时经常莫名好运', sub: '天选幸运属性', weights: { luck: 3 } }
      ]
    },
    {
      id: 'rp8', tag: 'NPC 对话', title: '遇到一个话特别密但信息量很低的人（职场啰嗦 NPC），你会？', sub: '耐心值检测',
      options: [
        { key: 'A', text: '内心狂按“跳过对话”键，面上保持礼貌点头', sub: '高敏高效脑', weights: { int: 2, luck: 1 } },
        { key: 'B', text: '直接打断切回正题：“说重点，需要我做什么？”', sub: '战士直球效率', weights: { str: 3 } },
        { key: 'C', text: '顺着聊两句哄得对方超开心，顺手套出隐藏情报', sub: '高魅力话术大师', weights: { cha: 3 } },
        { key: 'D', text: '放空听个白噪音，正好趁机在脑子里歇会儿', sub: '顶级钝感防护', weights: { luck: 3 } }
      ]
    },
    {
      id: 'rp9', tag: '暴击弱点', title: '以下哪件事最容易对你造成“9999 点真实暴击伤害”？', sub: '致命 Debuff 定位',
      options: [
        { key: 'A', text: '精心思考半天的方案被一句毫无逻辑的“我觉得不行”否掉', sub: '智性尊严受损', weights: { int: 3 } },
        { key: 'B', text: '卡在原地动弹不得、满腔干劲却无处施展', sub: '行动派窒息瞬间', weights: { str: 3 } },
        { key: 'C', text: '被在意的人冷落或误解，空气突然降到冰点', sub: '情感连接掉血', weights: { cha: 3 } },
        { key: 'D', text: '好不容易等来的假期突然被通知加班重开', sub: '松弛感瞬间破防', weights: { luck: 3 } }
      ]
    },
    {
      id: 'rp10', tag: '装备审美', title: '如果只允许保留一件现实中的“神器装备”，你最离不开？', sub: '核心外挂',
      options: [
        { key: 'A', text: '降噪耳机：一戴上立刻与嘈杂世界物理隔离', sub: '专注结界神器', weights: { int: 3 } },
        { key: 'B', text: '高效笔记/全能电脑：随时把脑子里的计划变成现实', sub: '生产力战斧', weights: { str: 3 } },
        { key: 'C', text: '绝美穿搭或氛围感单品：走出去自带主角气场', sub: '魅力光环装备', weights: { cha: 3 } },
        { key: 'D', text: '超舒服大床或懒人沙发：躺上去每秒回血 +50', sub: '终极回血祭坛', weights: { luck: 3 } }
      ]
    },
    {
      id: 'rp11', tag: '经验获取', title: '你现实中成长最快、升级最猛的时候通常发生在？', sub: '升级机制',
      options: [
        { key: 'A', text: '独自啃透一套底层逻辑、突然豁然开朗的瞬间', sub: '悟性顿悟升级', weights: { int: 3 } },
        { key: 'B', text: '被扔进高压实战里真刀真枪干完一个硬项目', sub: '实战刷怪升级', weights: { str: 3 } },
        { key: 'C', text: '和厉害的同频伙伴深度聊天碰撞出新视野', sub: '组队经验翻倍', weights: { cha: 3 } },
        { key: 'D', text: '某次随手尝试的无心插柳，居然打开了全新副本', sub: '奇遇事件升级', weights: { luck: 3 } }
      ]
    },
    {
      id: 'rp12', tag: '金钱观', title: '攒够第一桶金币，你更倾向怎么升级你的账号？', sub: '资源分配',
      options: [
        { key: 'A', text: '买稀缺知识、工具或低风险资产，复利滚雪球', sub: '精算师流派', weights: { int: 3 } },
        { key: 'B', text: '直接投资自己的核心事业或硬技能，把战力拉满', sub: '主 C 养成流派', weights: { str: 3 } },
        { key: 'C', text: '提升生活体验、请朋友吃喝玩乐、去世界各地看风景', sub: '快乐体验流派', weights: { cha: 3 } },
        { key: 'D', text: '先留足应急底气，剩下的随缘买点让自己开心的小确幸', sub: '稳健松弛流派', weights: { luck: 3 } }
      ]
    },
    {
      id: 'rp13', tag: '隐藏被动', title: '身边人最常感叹你自带的“不可思议被动技能”是？', sub: '天赋校准',
      options: [
        { key: 'A', text: '“你总能一眼看出别人没注意到的破绽和关键点”', sub: '鹰眼洞察被动', weights: { int: 3 } },
        { key: 'B', text: '“只要交给你，天大的烂摊子最后都能稳稳落地”', sub: '钢铁兜底被动', weights: { str: 3 } },
        { key: 'C', text: '“跟你待在一起气氛一下就热了，谁都喜欢你”', sub: '万人迷光环被动', weights: { cha: 3 } },
        { key: 'D', text: '“怎么每次踩线交卷或临时改道，你偏偏都能遇上好运”', sub: '锦鲤附体被动', weights: { luck: 3 } }
      ]
    },
    {
      id: 'rp14', tag: '存档习惯', title: '面对人生重要分叉口，如果现实有“即时存档”功能，你会？', sub: '行事胆魄',
      options: [
        { key: 'A', text: '每走三步存一个档，把所有支线结局全试一遍选最优解', sub: '全知全能完美主义', weights: { int: 3 } },
        { key: 'B', text: '不回头！选定一条路就一往无前打出我的专属结局', sub: '硬核一命通关', weights: { str: 3 } },
        { key: 'C', text: '拉着在意的人一起商量存档点，确保大家都不掉队', sub: '羁绊守护者', weights: { cha: 3 } },
        { key: 'D', text: '懒得手动存档，顺着自动存档随缘触发彩蛋才刺激', sub: '自由漫游玩家', weights: { luck: 3 } }
      ]
    },
    {
      id: 'rp15', tag: '反派魅力', title: '如果你在游戏里临时客串一次终极反派，你最像哪种 Boss？', sub: '暗面气场',
      options: [
        { key: 'A', text: '幕后黑手大法师：全程喝茶下棋，主角最后才发现全是我的局', sub: '高智商反派', weights: { int: 3 } },
        { key: 'B', text: '守关孤傲战神：不搞阴谋诡计，堂堂正正用实力压制全场', sub: '硬核武力反派', weights: { str: 3 } },
        { key: 'C', text: '人气超高魅惑反派：嘴太会说，主角团一半人都想倒戈跟我混', sub: '满级魅力反派', weights: { cha: 3 } },
        { key: 'D', text: '隐藏摆烂 Boss：其实我根本不想毁灭世界，只想在城堡睡觉', sub: '反差萌反派', weights: { luck: 3 } }
      ]
    },
    {
      id: 'rp16', tag: '角色出炉', title: '最后一句：选一条印在你游戏载入画面上的专属格言！', sub: '选完立刻生成你的复古像素角色面板',
      options: [
        { key: 'A', text: '“看透底层机制的人，从来不需要硬拼数值。”', sub: '法师/策士格言', weights: { int: 3 } },
        { key: 'B', text: '“没有推不平的副本，如果有就再升两级回来推。”', sub: '骑士/战士格言', weights: { str: 3 } },
        { key: 'C', text: '“一个人走得快，但跟对味的人组队才够好玩。”', sub: '诗人/领航格言', weights: { cha: 3 } },
        { key: 'D', text: '“人生是超大开放世界，躺草坪看云也算通关！”', sub: '欧皇游侠格言', weights: { luck: 3 } }
      ]
    }
  ],
  profiles: [
    {
      code: 'LV.27',
      title: '高敏暗影法师',
      subtitle: 'INT 满格 · 一眼看透底层机制的脆皮大脑',
      rarity: '仅 4.8% 测出',
      quote: '智力与洞察点满，但蓝条极脆——连开三场无效会议直接触发强制休眠 Debuff。',
      tags: ['高敏洞察', '机制解构', '脆皮蓝条'],
      buddy: { name: '松弛圣骑士', desc: '前排稳稳抗伤兜底，让你安心在后排输出神级灵感' },
      rival: { name: '无脑冲锋狂战士', desc: '对方不看机制直接开团，把你吓得原地开启隐身术' },
      avatarEmoji: '🧙',
      targetVector: { str: 40, int: 94, cha: 46, luck: 58 },
      extra: {
        rpgLevel: 'LV.27',
        passiveName: '微表情雷达',
        passiveDesc: '0.5 秒看穿场面客套与画大饼，对逻辑漏洞造成双倍暴击。',
        debuffName: '脆皮蓝条',
        debuffDesc: '高噪音无效社交超过 2 小时，自动进入 24h 强制静音恢复态。'
      }
    },
    {
      code: 'LV.32',
      title: '铁血破局圣骑士',
      subtitle: 'STR 满格 · 烂摊子终结者与团队主心骨',
      rarity: '仅 5.4% 测出',
      quote: '从不搞花里胡哨的借口，关键时刻永远是那个举起盾牌说“交给我”的硬核前排。',
      tags: ['钢铁兜底', '实战战神', '极度靠谱'],
      buddy: { name: '高敏暗影法师', desc: '你扛住正面所有压力，对方为你补齐最关键的策略视野' },
      rival: { name: '佛系挂机游侠', desc: '你正喊着全军冲锋，对方在旁边草丛里捡蘑菇' },
      avatarEmoji: '🛡️',
      targetVector: { str: 94, int: 62, cha: 56, luck: 36 },
      extra: {
        rpgLevel: 'LV.32',
        passiveName: '钢铁兜底光环',
        passiveDesc: '局面越混乱抗压值越高，队友在身边时焦虑值自动 -40%。',
        debuffName: '过度揽责',
        debuffDesc: '看到别人干得磨叽就忍不住亲自上手，容易偷偷耗光耐力槽。'
      }
    },
    {
      code: 'LV.25',
      title: '嘴甜魅惑吟游诗人',
      subtitle: 'CHA 满格 · 自带热场光环的组队核心',
      rarity: '仅 6.8% 测出',
      quote: '打打杀杀太累了，只要我开口聊五分钟，连守关 Boss 都想跟我加微信组局。',
      tags: ['魅力光环', '情绪Buff机', '组队神器'],
      buddy: { name: '铁血破局圣骑士', desc: '你负责凝聚气氛和人脉，对方负责把宏图稳稳落地' },
      rival: { name: '独狼冷面刺客', desc: '你热情抛出十个梗，对方全程沉默只顾自己刷怪' },
      avatarEmoji: '🎸',
      targetVector: { str: 48, int: 52, cha: 94, luck: 62 },
      extra: {
        rpgLevel: 'LV.25',
        passiveName: '全队士气 +200%',
        passiveDesc: '所在队伍尴尬率清零，与陌生 NPC 首次对话好感度直接拉满。',
        debuffName: '冷场过敏',
        debuffDesc: '如果发出去的消息长期没人接梗，快乐值会以肉眼可见速度下降。'
      }
    },
    {
      code: 'LV.??',
      title: '天选欧皇漫游游侠',
      subtitle: 'LUCK 满格 · 无视数值的开放世界松弛玩家',
      rarity: '仅 6.3% 测出',
      quote: '别人辛辛苦苦背攻略卷数值，你随手逛个支线就能顺路踩中隐藏宝箱。',
      tags: ['锦鲤附体', '松弛漫游', '反内卷大师'],
      buddy: { name: '嘴甜魅惑吟游诗人', desc: '两个快乐星人结伴探索世界，每天都有意想不到的奇遇' },
      rival: { name: '铁血破局圣骑士', desc: '对方每十分钟催你推主线，你只想在海边钓鱼看落日' },
      avatarEmoji: '🍀',
      targetVector: { str: 36, int: 50, cha: 58, luck: 94 },
      extra: {
        rpgLevel: 'LV.??',
        passiveName: '隐藏奇遇磁铁',
        passiveDesc: '危机时刻总能莫名触发巧合转机，自带免疫精神内耗护盾。',
        debuffName: '主线失忆症',
        debuffDesc: '极易被路边有趣小事带偏，经常忘记最初出门到底要干嘛。'
      }
    },
    {
      code: 'LV.35',
      title: '全能炼金战略家',
      subtitle: 'STR × INT 双修 · 既懂原理又能杀伐落地',
      rarity: '仅 4.5% 测出',
      quote: '脑子装得下全局蓝图，手上拿得起硬核工具，堪称版本答案级六边形战士。',
      tags: ['六边形战士', '冷静掌控', '效率怪物'],
      buddy: { name: '天选欧皇漫游游侠', desc: '你的严谨规划配对方的自由灵感，能解锁罕见隐藏结局' },
      rival: { name: '拖延症史莱姆', desc: '面对毫无时间观念的沟通对象，耐心槽 3 秒见底' },
      avatarEmoji: '⚗️',
      targetVector: { str: 84, int: 88, cha: 56, luck: 44 },
      extra: {
        rpgLevel: 'LV.35',
        passiveName: '最优解合成',
        passiveDesc: '能把零散资源快速炼成高胜率方案，复盘一次即永久免疫同款坑。',
        debuffName: '完美主义过载',
        debuffDesc: '偶尔因为想一次性做到 100 分，忘记给自己按暂停键歇口气。'
      }
    },
    {
      code: 'LV.29',
      title: '毒舌治愈魔导师',
      subtitle: 'INT × CHA 双修 · 嘴上吐槽手上给你加满血',
      rarity: '仅 5.9% 测出',
      quote: '一边精准吐槽你操作离谱，一边默默给你套上三层护盾加满血条的反差大佬。',
      tags: ['毒舌奶妈', '外冷内暖', '精准护短'],
      buddy: { name: '铁血破局圣骑士', desc: '一个管冲锋一个管毒舌兜底，绝佳黄金双排组合' },
      rival: { name: '玻璃心脆皮输出', desc: '对方经不起半句玩笑，让你憋着槽点难受一整天' },
      avatarEmoji: '✨',
      targetVector: { str: 46, int: 82, cha: 84, luck: 52 },
      extra: {
        rpgLevel: 'LV.29',
        passiveName: '毒舌回血术',
        passiveDesc: '一针见血点破问题同时稳稳接住情绪，治愈力与犀利度双在线。',
        debuffName: '口是心非',
        debuffDesc: '越是关心队友嘴上吐槽越狠，新手玩家初次组队容易误判好感度。'
      }
    }
  ]
};
