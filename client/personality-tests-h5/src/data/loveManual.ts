import type { TestDefinition } from '../engine/types.ts';

export const loveManualTest: TestDefinition = {
  id: 'manual',
  title: '相处避雷说明书',
  shortTitle: '避雷说明书',
  subtitle: '哄好耗时、吃醋隐蔽度与双人合盘',
  heroDesc: '测完直接生成一份《和我相处的出厂说明书》：嘴硬指数、哄好耗时、雷区警告，发给对象或闺蜜对号入座！',
  category: 'relation',
  badge: '情侣必测',
  estimatedMinutes: 2,
  cardBg: '#FFB5C8',
  posterBg: '#FFB5C8',
  tagColor: '#FFB5C8',
  accentColor: '#FF4D5E',
  scoringMode: 'nearest-profile',
  dimensions: [
    { id: 'direct', label: '直球坦率', lowLabel: '隐晦傲娇', highLabel: '直球明示' },
    { id: 'space', label: '空间需求', lowLabel: '高糖贴贴', highLabel: '独立呼吸' },
    { id: 'detail', label: '细节敏感', lowLabel: '大条洒脱', highLabel: '细节雷达', hotWhenHigh: true },
    { id: 'soft', label: '好哄指数', lowLabel: '需讲诚意', highLabel: '一秒顺毛' }
  ],
  microHints: [
    '✦ 正在读取你的出厂相处参数…',
    '✦ 已测出你的吃醋隐蔽度与嘴硬阈值',
    '✦ 正在整理你的绝对禁令与充电方式',
    '✦ 你的专属相处说明书正在排版中…'
  ],
  questions: [
    {
      id: 'lm1',
      tag: '回复速度',
      title: '给很在意的人发消息，对方隔了 3 小时才回一句“刚忙完”，你第一反应是？',
      sub: '诚实评估你的内心小剧场',
      options: [
        { key: 'A', text: '秒懂对方真的在忙，继续开心接上刚才的话题', sub: '安全感足，不靠秒回衡量爱意', weights: { space: 3, soft: 2, detail: -2 } },
        { key: 'B', text: '嘴上回“辛苦啦”，心里偷偷瞄了一眼对方刚才有没有发动态', sub: '超高敏细节雷达自动启动', weights: { detail: 3, direct: -2 } },
        { key: 'C', text: '直接撒娇抱怨：“你消失三小时我好想你，快补偿我！”', sub: '直球表达诉求，绝不憋成内伤', weights: { direct: 3, space: -2, soft: 2 } },
        { key: 'D', text: '那我也礼貌晾你一小时再回，保持对等呼吸感', sub: '势均力敌的傲娇拉扯', weights: { space: 2, direct: -2, soft: -2 } }
      ]
    },
    {
      id: 'lm2',
      tag: '生闷气信号',
      title: '你有点不开心但还没到吵架程度时，最明显的“危险信号”是？',
      sub: '让 TA 提前学会看懂你',
      options: [
        { key: 'A', text: '回复字数断崖式下跌，从“哈哈哈”变成“嗯/好的”', sub: '礼貌降温就是最高级警报', weights: { direct: -3, detail: 2, space: 2 } },
        { key: 'B', text: '憋不住三分钟，直接摊牌：“你刚才那句话让我不太舒服”', sub: '问题不过夜的直球选手', weights: { direct: 3, soft: 1 } },
        { key: 'C', text: '瞬间安静躲进自己壳里，想先独自冷静两小时', sub: '需要退回安全洞穴自我消化', weights: { space: 3, soft: -1 } },
        { key: 'D', text: '鼓着腮帮子疯狂暗示，只要对方来抱一下立刻破功笑场', sub: '超好哄的软萌警报', weights: { soft: 3, direct: 1, space: -2 } }
      ]
    },
    {
      id: 'lm3',
      tag: '吃醋隐蔽度',
      title: '看到伴侣或挚友在群里和某个异性/新朋友聊得很开心，你会？',
      sub: '醋坛子打翻实测',
      options: [
        { key: 'A', text: '脸上笑眯眯装大度，心里已经把酸味腌成了陈年老醋', sub: '吃醋隐蔽度 95%，绝不先认输', weights: { direct: -3, detail: 3 } },
        { key: 'B', text: '当面直接问：“哎哟你们聊啥这么开心，我吃醋了快哄我”', sub: '大大方方吃醋，反而超可爱', weights: { direct: 3, soft: 2, space: -1 } },
        { key: 'C', text: '完全不介意，每个人本来就该有自己的社交圈', sub: '内核超稳的松弛派', weights: { space: 3, detail: -2 } },
        { key: 'D', text: '不动声色观察对方有没有分寸感，越界才会一次性算总账', sub: '底线清晰的冷静观察者', weights: { detail: 3, soft: -2, space: 1 } }
      ]
    },
    {
      id: 'lm4',
      tag: '哄人秘籍',
      title: '闹别扭之后，对方怎么做最能让你一秒消气？',
      sub: '写进你的说明书核心章节',
      options: [
        { key: 'A', text: '别讲大道理！先抱住我认真说一句“刚才忽略你感受了”', sub: '态度优先，情绪接住就赢了 90%', weights: { soft: 3, detail: 2 } },
        { key: 'B', text: '拎着我最爱喝的奶茶/小零食出现，顺便卖个萌', sub: '美食与诚意双杀，三分钟顺毛', weights: { soft: 3, direct: 1 } },
        { key: 'C', text: '先给我半小时冷静空间，再来心平气和复盘下次怎么改进', sub: '重逻辑和实际改变，讨厌敷衍哄骗', weights: { space: 3, soft: -2, direct: 1 } },
        { key: 'D', text: '别装没事发生，主动把刚才没解释清的细节讲明白', sub: '需要清晰确定感才能真正翻篇', weights: { detail: 3, direct: 2 } }
      ]
    },
    {
      id: 'lm5',
      tag: '礼物偏好',
      title: '过纪念日或生日，哪种心意最能直接戳进你心坎里？',
      sub: '爱的接收通道',
      options: [
        { key: 'A', text: '对方居然记住了我三个月前随口提过的一件小东西', sub: '“被放进心上”比价格贵一万倍', weights: { detail: 3, soft: 2 } },
        { key: 'B', text: '提前问我想要啥直接买单，或者给我一场超爽双人旅行', sub: '实用又痛快的偏爱', weights: { direct: 3, detail: -1 } },
        { key: 'C', text: '一封认认真真手写的小卡片，写满相处里的具体瞬间', sub: '文字与肯定永远最动人', weights: { detail: 2, space: -1, soft: 2 } },
        { key: 'D', text: '不用搞花里胡哨的仪式，安静吃顿好吃的待在一起就很幸福', sub: '低压力细水长流派', weights: { space: 2, soft: 2, detail: -1 } }
      ]
    },
    {
      id: 'lm6',
      tag: '黏人曲线',
      title: '周末两天全待在一起，你对“亲密距离”的真实感受是？',
      sub: '测量你的社交与恋爱电池',
      options: [
        { key: 'A', text: '超级幸福！恨不得连去楼下买瓶水都要牵手一起去', sub: '高糖贴贴无限续航', weights: { space: -3, direct: 2 } },
        { key: 'B', text: '在同一间屋里各做各的事、偶尔抬头笑一下，最舒服', sub: '有呼吸感的陪伴才是顶配', weights: { space: 2, soft: 2 } },
        { key: 'C', text: '周六尽情玩一天，周日需要半天独处回血', sub: '精准平衡亲密与自我', weights: { space: 3, direct: 1 } },
        { key: 'D', text: '看当下心情：想黏的时候超黏，想静的时候请别打扰', sub: '猫系波动式亲密曲线', weights: { space: 2, direct: -1, detail: 2 } }
      ]
    },
    {
      id: 'lm7',
      tag: '绝对雷区',
      title: '以下四种行为里，哪一种最容易让你瞬间下头、好感清零？',
      sub: '你的红色严禁条款',
      options: [
        { key: 'A', text: '画大饼、说话不算数，答应好的小事转头全忘光', sub: '信任与靠谱是绝对底线', weights: { detail: 3, soft: -2 } },
        { key: 'B', text: '遇到矛盾就冷暴力、玩失踪、拒不沟通', sub: '冷处理是头号杀伤武器', weights: { direct: 3, space: -2 } },
        { key: 'C', text: '过度查岗、控制欲爆棚、不尊重我的独立空间', sub: '窒息感会让你一秒想逃跑', weights: { space: 3, soft: -1 } },
        { key: 'D', text: '当着外人面拆我台、拿我的短处开玩笑', sub: '体面与偏爱必须给足', weights: { detail: 2, direct: -1, soft: -2 } }
      ]
    },
    {
      id: 'lm8',
      tag: '分享欲测试',
      title: '你在路上拍到一朵形状像小狗的云，你会？',
      sub: '分享欲 = 喜欢程度',
      options: [
        { key: 'A', text: '一秒甩给对方加十个感叹号：“快看快看像不像你！”', sub: '把所有细碎可爱都第一时间递给你', weights: { direct: 3, space: -2, soft: 2 } },
        { key: 'B', text: '发过去配一句简短的“你看”，其实心里超期待对方夸我眼尖', sub: '含蓄型分享欲，需要你接住暗号', weights: { direct: -2, detail: 2 } },
        { key: 'C', text: '存进相册，晚上见面聊天时顺手翻出来一起看', sub: '偏爱面对面的真实温度', weights: { space: 2, soft: 1 } },
        { key: 'D', text: '发完如果对方回得很敷衍，下次就会悄悄收起分享欲', sub: '分享欲很珍贵，只给有回应的人', weights: { detail: 3, soft: -2 } }
      ]
    },
    {
      id: 'lm9',
      tag: '称赞需求',
      title: '当你换了新发型或认真做了一顿饭，最期待对方怎么夸？',
      sub: '夸夸指南',
      options: [
        { key: 'A', text: '越具体越好！“这个刘海衬得你眉眼好温柔/这菜火候绝了”', sub: '细节控一眼分辨真心夸还是敷衍夸', weights: { detail: 3, soft: 2 } },
        { key: 'B', text: '直接大声吹爆彩虹屁，情绪价值给满！', sub: '高能量直球赞美最受用', weights: { direct: 3, soft: 3 } },
        { key: 'C', text: '哪怕淡淡一句“很好看/很香”，只要眼神真诚我就超满足', sub: '看重真实感，不爱浮夸表演', weights: { space: 2, direct: -1 } },
        { key: 'D', text: '表面说“哪有啦”，但希望你再多夸两遍', sub: '傲娇型求夸体质', weights: { direct: -2, soft: 2 } }
      ]
    },
    {
      id: 'lm10',
      tag: '吵架风格',
      title: '真起争执的时候，你最接近哪种战斗状态？',
      sub: '提前打好预防针',
      options: [
        { key: 'A', text: '逻辑条理拉满，摆事实讲道理，最怕对方情绪化胡搅蛮缠', sub: '冷静辩手型，对事不对人', weights: { space: 2, direct: 2, soft: -2 } },
        { key: 'B', text: '眼眶比嘴先委屈，其实只是想要一句你在乎我', sub: '软心肠，凶不起来三秒就心软', weights: { soft: 3, space: -2, detail: 2 } },
        { key: 'C', text: '嘴上放狠话“随便你”，说完转身自己又后悔得要命', sub: '口是心非选手，千万别信狠话', weights: { direct: -3, soft: 2 } },
        { key: 'D', text: '先喊停各自冷静半小时，绝不在气头上说伤人的话', sub: '高情商克制派，保护关系底线', weights: { space: 3, detail: 1 } }
      ]
    },
    {
      id: 'lm11',
      tag: '安全感来源',
      title: '在一段关系里，什么最能让你彻底卸下防备、踏实安心？',
      sub: '核心安全感开关',
      options: [
        { key: 'A', text: '事事有回应、件件有着落，说到的永远做得到', sub: '确定感与靠谱是最高级浪漫', weights: { detail: 3, soft: 1 } },
        { key: 'B', text: '每天明目张胆的偏爱、拥抱和“我超喜欢你”', sub: '高浓度可见爱意', weights: { direct: 3, space: -3 } },
        { key: 'C', text: '彼此信任不猜忌，给足自由空间却又永远彼此兜底', sub: '成熟独立且稳固的锚点', weights: { space: 3, detail: -1 } },
        { key: 'D', text: '哪怕我露出脆弱、狼狈或不完美的一面，你依然坚定选我', sub: '深度接纳才敢交出真心', weights: { detail: 2, direct: -2, soft: 1 } }
      ]
    },
    {
      id: 'lm12',
      tag: '主动权',
      title: '遇到非常有好感的人，你会主动迈出第一步吗？',
      sub: '追爱勇敢度',
      options: [
        { key: 'A', text: '会！喜欢就要创造机会，绝不留遗憾', sub: '勇敢直球出击型', weights: { direct: 3, space: -1 } },
        { key: 'B', text: '会疯狂制造偶遇和暗示，但最后那句表白希望对方先说', sub: '高端钓系暗示选手', weights: { direct: -2, detail: 2 } },
        { key: 'C', text: '先慢慢观察对方人品和三观，确认双向奔赴才加速', sub: '慢热稳健派，绝不盲目上头', weights: { space: 2, detail: 3, soft: -1 } },
        { key: 'D', text: '太容易害羞，常常在心里演完一生，面上稳如老狗', sub: '极致慢热纯爱战士', weights: { direct: -3, space: 2 } }
      ]
    },
    {
      id: 'lm13',
      tag: '财务与分工',
      title: '日常约会开销或生活琐事分工，你最舒服的模式是？',
      sub: '长期相处契合点',
      options: [
        { key: 'A', text: '你请顿大餐我买杯奶茶和电影票，有来有往不斤斤计较', sub: '默契流动的舒服平衡', weights: { soft: 2, space: 1 } },
        { key: 'B', text: '提前把规则说清（比如共同小金库），透明省心没后患', sub: '清晰坦诚拒绝隐性委屈', weights: { direct: 3, space: 2 } },
        { key: 'C', text: '嘴上客气算清，但只要对方需要我，我永远最舍得为 TA 花钱', sub: '外客气内护短的大方派', weights: { direct: -1, detail: 2, soft: 1 } },
        { key: 'D', text: '谁擅长啥就做啥，互相照顾才像一家人', sub: '温暖互补型搭档', weights: { space: -2, soft: 3 } }
      ]
    },
    {
      id: 'lm14',
      tag: '情绪低谷',
      title: '工作或学习受了大委屈回到家，你希望对方怎么陪你？',
      sub: '标准情绪急救流程',
      options: [
        { key: 'A', text: '什么都别分析，先抱住我一起骂两句奇葩让我出气', sub: '无条件站我这边就对了', weights: { soft: 3, space: -2 } },
        { key: 'B', text: '安静给我倒杯热水，等我缓过神想说的时候再听我说', sub: '不逼问的温柔留白最治愈', weights: { space: 3, detail: 2 } },
        { key: 'C', text: '帮我一起梳理局势想出反杀办法，带我走出困境', sub: '靠谱军师型伴侣最安心', weights: { direct: 2, space: 1, soft: -1 } },
        { key: 'D', text: '直接拉我出门吃顿热辣火锅或兜风，快速切换好心情', sub: '行动带飞式解压', weights: { direct: 2, detail: -2, soft: 2 } }
      ]
    },
    {
      id: 'lm15',
      tag: '称呼与仪式',
      title: '对于专属昵称、情侣头像、纪念日打卡这些仪式感，你的态度是？',
      sub: '浪漫浓度测定',
      options: [
        { key: 'A', text: '超爱！专属称呼和仪式感就是普通日子里的小烟花', sub: '高糖仪式感星人', weights: { space: -3, detail: 2, soft: 2 } },
        { key: 'B', text: '私下怎么腻歪都行，但朋友圈想保持一点低调神秘感', sub: '私密高糖，对外克制', weights: { direct: -1, space: 2, detail: 2 } },
        { key: 'C', text: '顺其自然就好，真正的好感情不在朋友圈秀出来', sub: '务实松弛不搞形式主义', weights: { space: 3, detail: -2 } },
        { key: 'D', text: '嘴上嫌肉麻，但你真准备了我会偷偷开心一整周', sub: '嘴硬心甜傲娇代表', weights: { direct: -3, soft: 3 } }
      ]
    },
    {
      id: 'lm16',
      tag: '坦诚度',
      title: '如果发现对方善意隐瞒了一件小事（比如怕你担心没说自己加班太累），你会？',
      sub: '透明边界测试',
      options: [
        { key: 'A', text: '心疼之余认真告诉 TA：“以后不管好事坏事都直接告诉我”', sub: '偏好高透明度的并肩感', weights: { direct: 3, detail: 2 } },
        { key: 'B', text: '理解对方的体贴，悄悄把对方照顾得更周到', sub: '双向心疼的温柔派', weights: { soft: 3, space: 1 } },
        { key: 'C', text: '会有点敏感，忍不住想“还有没有别的事也瞒着我”', sub: '高细节敏感，需要坦诚兜底', weights: { detail: 3, soft: -2 } },
        { key: 'D', text: '无所谓，每个人都有自己消化小麻烦的习惯', sub: '高松弛边界感', weights: { space: 3, detail: -2 } }
      ]
    },
    {
      id: 'lm17',
      tag: '合拍信号',
      title: '哪一个瞬间最会让你笃定“就是这个人了，我想和 TA 处很久”？',
      sub: '心动定格瞬间',
      options: [
        { key: 'A', text: '哪怕沉默着各自做事，空气里也全是安心而不是尴尬', sub: '低耗能高默契的松弛感', weights: { space: 3, soft: 2 } },
        { key: 'B', text: '我还没开口，对方已经递过来我正需要的那样东西', sub: '被稳稳接住的细节偏爱', weights: { detail: 3, direct: -1 } },
        { key: 'C', text: '发生分歧时没有指责，而是拉着我的手说“我们一起解决”', sub: '成熟直球、情绪稳定', weights: { direct: 3, soft: 1 } },
        { key: 'D', text: '在对方面前我可以肆无忌惮当回幼稚小朋友', sub: '毫无防备的纯粹快乐', weights: { soft: 3, space: -2 } }
      ]
    },
    {
      id: 'lm18',
      tag: '出厂铭牌',
      title: '如果要把一句话印在你说明书封面的最显眼位置，你会印？',
      sub: '最后一题 · 封装你的相处说明书',
      options: [
        { key: 'A', text: '“请用真诚和细节唤醒我，嘴硬只是我的保护色”', sub: '高边界慢热型专属铭牌', weights: { direct: -2, detail: 3 } },
        { key: 'B', text: '“直球偏爱我，我会把全宇宙的甜双倍还给你”', sub: '高糖直球型专属铭牌', weights: { direct: 3, space: -2, soft: 2 } },
        { key: 'C', text: '“保留彼此自由呼吸的空间，我们就能走得特别远”', sub: '独立松弛型专属铭牌', weights: { space: 3, direct: 1 } },
        { key: 'D', text: '“别讲大道理，抱紧再递杯奶茶我就能立刻顺毛”', sub: '治愈软萌型专属铭牌', weights: { soft: 3, detail: 1 } }
      ]
    }
  ],
  profiles: [
    {
      code: 'XB-09',
      title: '高边界慢热型',
      subtitle: '嘴硬心软 · 细节控专属护短体',
      rarity: '仅 5.9% 测出',
      quote: '嘴上说着“随便都行”，心里其实早给你悄悄打好了 100 分细节表。',
      tags: ['嘴硬心软', '细节雷达', '极度护短'],
      buddy: { name: 'XB-01 直球治愈系', desc: '不玩猜心游戏，一句明确偏爱就能让你瞬间回血' },
      rival: { name: 'XB-05 冰山冷静派', desc: '一旦开始比谁更沉默，两个人能礼貌冷到下个世纪' },
      avatarEmoji: '📋',
      targetVector: { direct: 28, space: 64, detail: 88, soft: 56 },
      extra: {
        modelNo: 'MODEL XB-2026-09',
        specs: [
          { label: '平均哄好耗时', value: '15 分钟' },
          { label: '吃醋隐蔽度', value: '92%' },
          { label: '社交续航电量', value: '2.5 小时' },
          { label: '直球偏爱加成', value: '+200%' }
        ],
        ingredients: '45% 嘴硬心软 + 35% 细节雷达 + 20% 极度护短',
        chargeMethod: '明确且具体的偏爱、不敷衍的回应、偶尔安静待在一起。',
        warning: '严禁画大饼、严禁已读不回后装作无事发生！'
      }
    },
    {
      code: 'XB-01',
      title: '直球高糖治愈系',
      subtitle: '明目张胆偏爱 · 情绪不过夜小太阳',
      rarity: '仅 7.4% 测出',
      quote: '喜欢就大声说，委屈就当面哄，和你谈恋爱完全不用做阅读理解！',
      tags: ['直球偏爱', '一秒顺毛', '高糖分享欲'],
      buddy: { name: 'XB-09 高边界慢热型', desc: '你的真诚直球刚好能稳稳化开对方的傲娇外壳' },
      rival: { name: 'XB-05 冰山冷静派', desc: '你热情发十条语音，对方两小时后回个“嗯”' },
      avatarEmoji: '🍓',
      targetVector: { direct: 90, space: 22, detail: 58, soft: 90 },
      extra: {
        modelNo: 'MODEL XB-2026-01',
        specs: [
          { label: '平均哄好耗时', value: '3 分钟' },
          { label: '吃醋隐蔽度', value: '12%' },
          { label: '每日贴贴需求', value: 'MAX' },
          { label: '情绪隔夜概率', value: '0.1%' }
        ],
        ingredients: '50% 直球热情 + 30% 软萌好哄 + 20% 分享欲爆棚',
        chargeMethod: '秒回接梗、大声夸夸、牵手散步与热腾腾的拥抱。',
        warning: '严禁冷暴力、严禁敷衍式敷敷了事！'
      }
    },
    {
      code: 'XB-03',
      title: '松弛呼吸伴侣型',
      subtitle: '不捆绑不猜忌 · 细水长流顶配',
      rarity: '仅 6.8% 测出',
      quote: '各自发光又彼此兜底，不靠查岗维系忠诚，舒服自由才是最长情的告白。',
      tags: ['独立呼吸感', '内核极稳', '零内耗'],
      buddy: { name: 'XB-03 松弛呼吸型', desc: '双松弛组合，吵架率低于全网 98% 的情侣' },
      rival: { name: 'XB-07 高敏查岗型', desc: '高频追问行踪会让你瞬间触发逃跑警报' },
      avatarEmoji: '🍃',
      targetVector: { direct: 66, space: 92, detail: 34, soft: 72 },
      extra: {
        modelNo: 'MODEL XB-2026-03',
        specs: [
          { label: '平均哄好耗时', value: '10 分钟' },
          { label: '吃醋隐蔽度', value: '40%' },
          { label: '独处回血需求', value: '每周 1 天' },
          { label: '情绪稳定度', value: '95%' }
        ],
        ingredients: '50% 高级边界感 + 30% 松弛信任 + 20% 靠谱兜底',
        chargeMethod: '给足独立空间、并肩安静看电影、真诚不拧巴的沟通。',
        warning: '严禁过度控制、严禁查岗猜忌与情绪勒索！'
      }
    },
    {
      code: 'XB-05',
      title: '清醒理智军师型',
      subtitle: '先稳局面再谈爱 · 用行动兑现承诺',
      rarity: '仅 5.2% 测出',
      quote: '嘴上不会说天花乱坠的甜话，但你遇事第一个站出来替你摆平的永远是我。',
      tags: ['行动派偏爱', '对事不对人', '靠谱天花板'],
      buddy: { name: 'XB-03 松弛呼吸型', desc: '情绪稳定不翻旧账，遇事商量着解决超高效' },
      rival: { name: 'XB-07 高敏细节控', desc: '你想讲逻辑解决问题，对方其实只想要一个拥抱' },
      avatarEmoji: '🧭',
      targetVector: { direct: 74, space: 76, detail: 62, soft: 26 },
      extra: {
        modelNo: 'MODEL XB-2026-05',
        specs: [
          { label: '平均哄好耗时', value: '30 分钟' },
          { label: '吃醋隐蔽度', value: '82%' },
          { label: '承诺兑现率', value: '99%' },
          { label: '画大饼容忍度', value: '0%' }
        ],
        ingredients: '45% 清醒理智 + 35% 靠谱执行 + 20% 隐性深情',
        chargeMethod: '坦诚沟通不绕弯子、说到做到、尊重彼此节奏。',
        warning: '严禁反复食言、严禁在气头上口不择言翻旧账！'
      }
    },
    {
      code: 'XB-07',
      title: '高敏纯爱细节控',
      subtitle: '眼神敏锐 · 把你放进每一格小事里',
      rarity: '仅 6.1% 测出',
      quote: '我记住你随口提过的每件小事，也敏锐察觉你语气里 0.5 度的降温。',
      tags: ['高纯度深情', '情绪雷达', '确定感刚需'],
      buddy: { name: 'XB-01 直球高糖系', desc: '对方的透明和热烈，能把你所有的不安瞬间清零' },
      rival: { name: 'XB-03 松弛呼吸型', desc: '对方太佛系不报备，会让你疯狂脑补十集悬疑剧' },
      avatarEmoji: '💌',
      targetVector: { direct: 46, space: 26, detail: 94, soft: 46 },
      extra: {
        modelNo: 'MODEL XB-2026-07',
        specs: [
          { label: '平均哄好耗时', value: '20 分钟' },
          { label: '吃醋隐蔽度', value: '76%' },
          { label: '细节记忆力', value: 'MAX' },
          { label: '事事有回应需求', value: '100%' }
        ],
        ingredients: '45% 细腻深情 + 35% 确定感雷达 + 20% 极度专一',
        chargeMethod: '主动报备、记得纪念日细节、第一时间坚定站在 TA 这边。',
        warning: '严禁忽冷忽热、严禁模糊边界和与其他人不清不楚！'
      }
    },
    {
      code: 'XB-08',
      title: '傲娇奶凶反差萌',
      subtitle: '表面不好惹 · 实则一颗糖就能哄笑',
      rarity: '仅 6.5% 测出',
      quote: '吵架时放狠话天下第一，其实只要你递来一杯冰美式顺顺毛，三秒就破防。',
      tags: ['口嫌体正直', '奶凶护短', '超快破功'],
      buddy: { name: 'XB-05 清醒理智军师', desc: '对方稳重宠溺一眼看穿你的小傲娇，绝不跟你较真' },
      rival: { name: 'XB-09 高边界慢热型', desc: '两个死鸭子嘴硬凑一起，谁也不肯先发第一条消息' },
      avatarEmoji: '🎀',
      targetVector: { direct: 24, space: 42, detail: 66, soft: 88 },
      extra: {
        modelNo: 'MODEL XB-2026-08',
        specs: [
          { label: '平均哄好耗时', value: '5 分钟' },
          { label: '吃醋隐蔽度', value: '86%' },
          { label: '嘴硬心软率', value: '98%' },
          { label: '美食投喂加成', value: '+300%' }
        ],
        ingredients: '40% 傲娇嘴硬 + 40% 软萌好哄 + 20% 护短小虎牙',
        chargeMethod: '别信 TA 的气话！先给抱抱、投喂爱吃的再顺毛夸两句。',
        warning: '严禁顺着气话说“那就这样吧”，真的会偷偷难过！'
      }
    }
  ]
};
