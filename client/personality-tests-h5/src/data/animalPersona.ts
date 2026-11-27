import type { TestDefinition } from '../engine/types.ts';

export const animalPersonaTest: TestDefinition = {
  id: 'animal',
  title: '动物塑深度版',
  shortTitle: '动物塑',
  subtitle: '8 种反差萌动物原型 + 专属饲养须知',
  heroDesc: '比四个字母更戳人的灵魂化身：你是情绪稳如泰山的松弛水豚、外冷内热的嘴硬藏狐，还是战略雪豹？',
  category: 'viral',
  badge: '全民热测',
  estimatedMinutes: 2,
  cardBg: '#FFD89B',
  posterBg: '#FFD89B',
  tagColor: '#FFD89B',
  accentColor: '#C8732C',
  scoringMode: 'nearest-profile',
  dimensions: [
    { id: 'zen', label: '松弛钝感', lowLabel: '警觉敏感', highLabel: '水豚稳态' },
    { id: 'bond', label: '黏人热忱', lowLabel: '独行自由', highLabel: '贴贴小狗' },
    { id: 'sharp', label: '野心锐度', lowLabel: '随遇而安', highLabel: '精准狩猎' },
    { id: 'tsundere', label: '反差嘴硬', lowLabel: '直球透明', highLabel: '外冷内热' }
  ],
  microHints: [
    '✦ 正在扫描你的毛茸茸灵魂轮廓…',
    '✦ 检测到明显的反差萌信号',
    '✦ 正在匹配你的专属栖息地与天敌',
    '✦ 你的动物盲盒正在拆封中…'
  ],
  questions: [
    {
      id: 'an1',
      tag: '突袭事件',
      title: '原本平静的下午突然天降一个紧急大麻烦，你第一秒的反应更像？',
      sub: '看你的本能应激模式',
      options: [
        { key: 'A', text: '先喝口水深呼吸：“急也没用，慢慢拆解”', sub: '情绪极其稳定，天塌先稳住橘子', weights: { zen: 3, sharp: 1 } },
        { key: 'B', text: '瞬间眼神锐利，5 分钟拉出行动方案直接上场', sub: '越乱越冷静的顶级猎手', weights: { sharp: 3, zen: 1 } },
        { key: 'C', text: '嘴上吐槽“烦死了”，手上已经麻利地帮忙兜底了', sub: '典型嘴硬心软实干派', weights: { tsundere: 3, sharp: 2 } },
        { key: 'D', text: '立刻摇朋友/队友一起商量，人多力量大心里踏实', sub: '群居协作瞬间满血', weights: { bond: 3 } }
      ]
    },
    {
      id: 'an2',
      tag: '领地意识',
      title: '如果有人未经同意随便翻你手机或动你桌面上的私人物品，你会？',
      sub: '测试你的领地红线',
      options: [
        { key: 'A', text: '瞬间炸毛警报拉响，礼貌但极其冰冷地拿回来', sub: '领地意识极强，不可冒犯', weights: { sharp: 2, tsundere: 3, zen: -2 } },
        { key: 'B', text: '表面波澜不惊，暗暗把对方移出核心信任名单', sub: '安静划清界限的独行者', weights: { zen: 2, tsundere: 2, bond: -2 } },
        { key: 'C', text: '如果是特别铁的好朋友其实无所谓，随便看', sub: '对亲近的人几乎不设防', weights: { bond: 3, zen: 2 } },
        { key: 'D', text: '直接开口提醒：“下次先跟我说一声哈”', sub: '直球定规矩，不藏着憋着', weights: { sharp: 2, tsundere: -2 } }
      ]
    },
    {
      id: 'an3',
      tag: '被夸反应',
      title: '当喜欢的人或很欣赏的前辈认真夸你“你真的好厉害”，你会？',
      sub: '嘴硬指数实测',
      options: [
        { key: 'A', text: '表面淡淡回“还好吧常规操作”，转身嘴角咧到耳根', sub: '教科书级外冷内热反差萌', weights: { tsundere: 3, sharp: 1 } },
        { key: 'B', text: '眼睛发亮开心接下：“嘿嘿谢谢！我也觉得！”', sub: '情绪全写在脸上的小太阳', weights: { bond: 3, tsundere: -2 } },
        { key: 'C', text: '笑眯眯点头“承让承让”，顺手回夸对方一句', sub: '从容大方毫不拧巴', weights: { zen: 3, bond: 1 } },
        { key: 'D', text: '表面微笑，心里立刻盘算“下次要做得更绝”', sub: '高标准自我驱动型', weights: { sharp: 3, tsundere: 1 } }
      ]
    },
    {
      id: 'an4',
      tag: '周末栖息',
      title: '连续忙碌五天后的周六上午，最理想的充电姿势是？',
      sub: '你的天然栖息地',
      options: [
        { key: 'A', text: '窝在沙发里晒太阳发呆，连手机都不想拿', sub: '温泉水豚式极致省电', weights: { zen: 3, sharp: -1 } },
        { key: 'B', text: '约搭子逛市集吃火锅，把一周的委屈聊完', sub: '互动即回血的快乐生物', weights: { bond: 3, tsundere: -1 } },
        { key: 'C', text: '去健身房挥汗或研究副业/新技能，停不下来', sub: '自带高燃引擎的行动派', weights: { sharp: 3 } },
        { key: 'D', text: '独自躲进安静角落看剧画画，只回 1 个人的消息', sub: '精致独处 + 选择性贴贴', weights: { tsundere: 2, zen: 2, bond: -1 } }
      ]
    },
    {
      id: 'an5',
      tag: '社交破冰',
      title: '走进一个全是陌生人的局，你通常需要多久才能放开？',
      sub: '慢热 vs 自来熟',
      options: [
        { key: 'A', text: '三分钟！很快就能和旁边的人聊成一片', sub: '天然摇尾巴社交悍匪', weights: { bond: 3, tsundere: -2 } },
        { key: 'B', text: '先在高处默默观察半小时，摸清气氛再开口', sub: '冷静审视环境的猫科直觉', weights: { sharp: 2, tsundere: 2, bond: -1 } },
        { key: 'C', text: '全程微笑点头当安静背景板，吃好喝好就撤', sub: '绝不勉强营业的松弛生物', weights: { zen: 3, bond: -1 } },
        { key: 'D', text: '除非遇到聊得来的同频人，否则自动省电', sub: '精准社交，拒绝广泛撒网', weights: { tsundere: 3, zen: 1 } }
      ]
    },
    {
      id: 'an6',
      tag: '护短测试',
      title: '你在意的好朋友被人阴阳怪气欺负了，你的反应是？',
      sub: '反差战斗力检测',
      options: [
        { key: 'A', text: '平时超佛系，但动我的人瞬间切换高冷怼人模式', sub: '温柔外表下的隐藏护短大佬', weights: { sharp: 3, tsundere: 2 } },
        { key: 'B', text: '第一时间冲上去护在前面，绝不让朋友受委屈', sub: '忠诚直球的热血骑士', weights: { bond: 3, sharp: 2, tsundere: -1 } },
        { key: 'C', text: '先带朋友离开现场吃顿好的安抚情绪，别跟烂人纠缠', sub: '情绪价值拉满的稳重后盾', weights: { zen: 3, bond: 2 } },
        { key: 'D', text: '嘴上说“你怎么这么笨”，转头替对方把场子找回来', sub: '刀子嘴豆腐心代表', weights: { tsundere: 3, bond: 1 } }
      ]
    },
    {
      id: 'an7',
      tag: '目标感',
      title: '面对一个你非常想拿下的机会（好岗位/心动对象/稀缺名额），你会？',
      sub: '狩猎风格',
      options: [
        { key: 'A', text: '提前做足功课潜伏观察，选准时机一击即中', sub: '雪豹式精准狙击', weights: { sharp: 3, tsundere: 1 } },
        { key: 'B', text: '大大方方表达诚意和优势，用热情打动对方', sub: '坦荡直球更容易赢', weights: { bond: 3, sharp: 1, tsundere: -2 } },
        { key: 'C', text: '尽人事听天命，得之我幸失之我命，心态稳得很', sub: '不拧巴的长期赢家心态', weights: { zen: 3 } },
        { key: 'D', text: '明明超想要，表面还装出一副云淡风轻的样子', sub: '死要面子的傲娇选手', weights: { tsundere: 3 } }
      ]
    },
    {
      id: 'an8',
      tag: '面对批评',
      title: '有人当众指出你刚才犯的一个小失误，你心里的真实波动是？',
      sub: '抗压与自愈',
      options: [
        { key: 'A', text: '“哦确实错了，改了就行”，情绪几乎零波动', sub: '防弹级稳定内核', weights: { zen: 3, tsundere: -1 } },
        { key: 'B', text: '冷静记下问题复盘优化，绝不在同一个地方跌两次', sub: '高速进化的硬核选手', weights: { sharp: 3, zen: 1 } },
        { key: 'C', text: '表面淡定说谢谢，私下其实有点小在意会琢磨一会儿', sub: '外壳坚硬内里柔软', weights: { tsundere: 3, zen: -1 } },
        { key: 'D', text: '马上找信任的人求一句安慰，抱抱完就满血复活', sub: '需要情绪呼应的暖萌生物', weights: { bond: 3, zen: -1 } }
      ]
    },
    {
      id: 'an9',
      tag: '消费快乐',
      title: '发了一笔意外小奖金，你最想怎么花掉它？',
      sub: '生活享乐倾向',
      options: [
        { key: 'A', text: '请家人和最好的朋友搓一顿硬核大餐', sub: '独乐乐不如众乐乐', weights: { bond: 3 } },
        { key: 'B', text: '升级一件能大幅提升独处幸福感的装备（耳机/床品）', sub: '把舒适度拉到极致', weights: { zen: 3, tsundere: 1 } },
        { key: 'C', text: '存一半做底气，另一半投资自己或买高质感单品', sub: '清醒稳健的长期主义', weights: { sharp: 3 } },
        { key: 'D', text: '悄悄给在意的人买份小礼物，但嘴上说“顺路买的”', sub: '藏不住温柔的傲娇鬼', weights: { tsundere: 3, bond: 1 } }
      ]
    },
    {
      id: 'an10',
      tag: '微信头像',
      title: '你的社交头像或表情包风格通常更偏向？',
      sub: '互联网皮肤映射',
      options: [
        { key: 'A', text: '呆萌佛系、淡定吃瓜或看起来很困的动物表情包', sub: '自带松弛笑点', weights: { zen: 3 } },
        { key: 'B', text: '可可爱爱、圆滚滚或热情贴贴的搞笑梗图', sub: '亲和力一眼拉满', weights: { bond: 3, tsundere: -1 } },
        { key: 'C', text: '极简酷感、风景暗调或带点小冷幽默的图', sub: '保持神秘高级感', weights: { sharp: 2, tsundere: 2 } },
        { key: 'D', text: '表面高冷拽姐/拽哥，聊熟后全是沙雕抽象表情包', sub: '熟人专属反差惊喜', weights: { tsundere: 3, bond: 1 } }
      ]
    },
    {
      id: 'an11',
      tag: '被放鸽子',
      title: '期待了一周的约会突然因故取消，你的情绪续航会？',
      sub: '适应力检测',
      options: [
        { key: 'A', text: '秒切 B 计划：正好窝家里吃瓜睡觉，血赚！', sub: '自带无限松弛缓冲垫', weights: { zen: 3, bond: -1 } },
        { key: 'B', text: '确实会失落一会儿，期待对方下次好好补回来', sub: '重承诺且真心期待见面', weights: { bond: 3 } },
        { key: 'C', text: '立刻利用空出来的 3 小时推进自己的事，一点不浪费', sub: '高效率自驱达人', weights: { sharp: 3 } },
        { key: 'D', text: '嘴上回“完全没事呀”，其实默默失落了半小时', sub: '习惯体谅别人的小傲娇', weights: { tsundere: 3 } }
      ]
    },
    {
      id: 'an12',
      tag: '团队角色',
      title: '小组合作赶项目，大家卡住吵起来了，你通常会？',
      sub: '群居协作角色',
      options: [
        { key: 'A', text: '淡定递上奶茶：“先喘口气，吵架也出不了方案”', sub: '全队情绪定海神针', weights: { zen: 3, bond: 1 } },
        { key: 'B', text: '直接敲黑板切回核心目标，三分钟拍板分工', sub: '天生带队破局的头狼/雪豹', weights: { sharp: 3 } },
        { key: 'C', text: '积极打圆场夸夸两边，让氛围重新热络起来', sub: '最强粘合剂小天使', weights: { bond: 3, tsundere: -1 } },
        { key: 'D', text: '懒得参与口水战，直接把最难的那块默默做完甩群里', sub: '用实力说话的酷猫', weights: { tsundere: 3, sharp: 2 } }
      ]
    },
    {
      id: 'an13',
      tag: '亲密节奏',
      title: '在亲密关系里，你最渴望的相处状态是？',
      sub: '情感温度计',
      options: [
        { key: 'A', text: '各自有独立空间，但需要时永远第一时间在场', sub: '舒服自由的高质量陪伴', weights: { zen: 2, tsundere: 2 } },
        { key: 'B', text: '每天分享琐碎日常，牵手贴贴、把喜欢说满一万遍', sub: '高糖直球双倍治愈', weights: { bond: 3, tsundere: -2 } },
        { key: 'C', text: '势均力敌共同成长，彼此是恋人更是最佳战友', sub: '慕强且互相成就', weights: { sharp: 3, bond: 1 } },
        { key: 'D', text: '不用说太多甜话，但对方能一眼看穿我的小别扭', sub: '被偏爱和读懂的安心感', weights: { tsundere: 3, bond: 1 } }
      ]
    },
    {
      id: 'an14',
      tag: '生活小习惯',
      title: '下面哪个细节最像你日常的真实写照？',
      sub: '习惯照见本性',
      options: [
        { key: 'A', text: '出门前能坐着绝不站着，能躺着绝不坐着', sub: '节能环保一级保护动物', weights: { zen: 3 } },
        { key: 'B', text: '看到路边可爱的小猫小狗一定要停下来打招呼', sub: '柔软心肠藏不住', weights: { bond: 3 } },
        { key: 'C', text: '手机桌面和日程规划井井有条，心里有张清晰地图', sub: '高度自律的掌控感', weights: { sharp: 3 } },
        { key: 'D', text: '外人面前高冷寡言，回到家对着熟人秒变话痨', sub: '只对认证家人解锁粘人皮肤', weights: { tsundere: 3, bond: 1 } }
      ]
    },
    {
      id: 'an15',
      tag: '面对竞争',
      title: '身边突然出现一个实力很强的竞争对手，你会？',
      sub: '胜负欲测试',
      options: [
        { key: 'A', text: '反而燃起来了！遇强则强正是激发潜力的好机会', sub: '顶级狩猎者的兴奋感', weights: { sharp: 3 } },
        { key: 'B', text: '祝对方发财，我走我的独木桥绝不跟着焦虑内卷', sub: '松弛感无敌，卷不到我', weights: { zen: 3, sharp: -2 } },
        { key: 'C', text: '主动去交个朋友取取经，说不定还能合作共赢', sub: '开放真诚的社交智慧', weights: { bond: 3, sharp: 1 } },
        { key: 'D', text: '嘴上说“随便啦我不在乎”，背地里偷偷加练两小时', sub: '暗中较劲的傲娇优等生', weights: { tsundere: 3, sharp: 2 } }
      ]
    },
    {
      id: 'an16',
      tag: '情绪信号',
      title: '当你真的累了、不想营业了，你会发出什么信号？',
      sub: '饲养员必看预警',
      options: [
        { key: 'A', text: '眼神放空、回复变慢，只想找个温暖角落缩起来', sub: '自动进入休眠充能态', weights: { zen: 2, tsundere: 2 } },
        { key: 'B', text: '直接跟熟人撒娇喊累：“今天电量 0% 求投喂”', sub: '坦率求助，很快就能回血', weights: { bond: 3, tsundere: -2 } },
        { key: 'C', text: '干脆利落推掉无关事务，高冷关门独处 24 小时', sub: '果断止损保护核心精力', weights: { sharp: 2, bond: -2 } },
        { key: 'D', text: '表面一切照常，但谁凑太近说话语气会突然变淡', sub: '隐形警报，需要被温柔察觉', weights: { tsundere: 3 } }
      ]
    },
    {
      id: 'an17',
      tag: '美食诱惑',
      title: '深夜 11 点刷到超诱人的夜宵视频，你的自控力像哪种动物？',
      sub: '本能与克制',
      options: [
        { key: 'A', text: '立刻点单！快乐比明天称体重重要一万倍', sub: '及时行乐松弛主义', weights: { zen: 2, bond: 2 } },
        { key: 'B', text: '看两眼果断划走喝口白水，绝不破坏自己的计划', sub: '钢铁自控力', weights: { sharp: 3 } },
        { key: 'C', text: '转发给搭子：“别让我一个人受折磨，你也看！”', sub: '共享罪恶感的快乐小狗', weights: { bond: 3 } },
        { key: 'D', text: '嘴上说“太油腻了不吃”，五分钟后默默煮了碗泡面', sub: '口嫌体正直冠军', weights: { tsundere: 3, zen: 1 } }
      ]
    },
    {
      id: 'an18',
      tag: '灵魂栖息',
      title: '如果下辈子可以任选一种动物自由活一次，你最想当？',
      sub: '最后一题 · 揭晓你的动物塑盲盒',
      options: [
        { key: 'A', text: '泡在暖呼呼温泉里顶着橘子看云的水豚', sub: '世间喧嚣皆与我无关', weights: { zen: 3 } },
        { key: 'B', text: '在雪山之巅独来独往、优雅强大的雪豹', sub: '自由、锋利、自带气场', weights: { sharp: 3, tsundere: 1 } },
        { key: 'C', text: '被全世界偏爱、每天甩着尾巴迎接主人的小金毛', sub: '双向奔赴的满格爱意', weights: { bond: 3, tsundere: -1 } },
        { key: 'D', text: '在阳台晒太阳、高兴才赏你摸一下的高冷小黑猫', sub: '独立又迷人的反差萌', weights: { tsundere: 3, zen: 1 } }
      ]
    }
  ],
  profiles: [
    {
      code: 'CAPYBARA',
      title: '松弛水豚',
      subtitle: '情绪稳定天花板 · 顶橘看淡世事',
      rarity: '仅 7.4% 测出',
      quote: '天塌下来先看看头顶的橘子掉没掉，情绪稳定是你最顶级的防御武器。',
      tags: ['情绪稳定天花板', '拒绝内耗', '隐性宅家'],
      buddy: { name: '嘴硬藏狐', desc: '外冷内热超合拍，各自安静刷手机也能待一天' },
      rival: { name: '卷王边牧', desc: '对方每小时都在排计划，你只想原地泡温泉' },
      avatarEmoji: '🍊',
      targetVector: { zen: 92, bond: 52, sharp: 32, tsundere: 38 },
      extra: {
        animalKey: 'capybara',
        animalEn: 'ZEN CAPYBARA',
        animalPills: ['情绪稳定天花板', '拒绝内耗', '隐性宅家'],
        feedingGuide: '投喂冰美式、安静独处时间与不催促的陪伴即可满格回血，切忌高分贝焦虑轰炸。'
      }
    },
    {
      code: 'FOX',
      title: '嘴硬藏狐',
      subtitle: '外冷内热 · 眼神淡定内心弹幕满屏',
      rarity: '仅 6.2% 测出',
      quote: '看起来看透红尘有点不好惹，其实只要被真诚偏爱一次，尾巴早就偷偷摇成螺旋桨了。',
      tags: ['刀子嘴豆腐心', '高边界慢热', '暗中护短'],
      buddy: { name: '松弛水豚', desc: '对方稳得住你的小别扭，不催不逼舒服到极致' },
      rival: { name: '社牛萨摩耶', desc: '对方自来熟过头，会让你瞬间拉起礼貌警报' },
      avatarEmoji: '🦊',
      targetVector: { zen: 56, bond: 42, sharp: 64, tsundere: 92 },
      extra: {
        animalKey: 'fox',
        animalEn: 'TSUNDERE FOX',
        animalPills: ['外冷内热', '极度护短', '反差萌天花板'],
        feedingGuide: '别信 TA 嘴上说的“随便/没事”，多给一点明确细节和坚定偏爱，立刻乖乖贴贴。'
      }
    },
    {
      code: 'LEOPARD',
      title: '战略雪豹',
      subtitle: '高冷猎手 · 优雅克制的顶级破局者',
      rarity: '仅 4.6% 测出',
      quote: '从不把野心挂在嘴边吵闹，只在认准目标后悄无声息地做到全场惊艳。',
      tags: ['慕强清醒', '独立气场', '精准破局'],
      buddy: { name: '卷王边牧', desc: '双强同频聊效率，办事干脆利落绝不拖泥带水' },
      rival: { name: '摸鱼考拉', desc: '你急着冲山顶，对方在树杈上问午饭点啥' },
      avatarEmoji: '🐆',
      targetVector: { zen: 52, bond: 28, sharp: 94, tsundere: 66 },
      extra: {
        animalKey: 'leopard',
        animalEn: 'SNOW LEOPARD',
        animalPills: ['顶级破局力', '高冷护短', '自律成瘾'],
        feedingGuide: '给足尊重、空间与智性共鸣，别试图用情绪绑架控制 TA，否则一秒消失在雪线之上。'
      }
    },
    {
      code: 'SAMOYED',
      title: '快乐萨摩耶',
      subtitle: '微笑天使 · 走到哪都能热场的小太阳',
      rarity: '仅 7.8% 测出',
      quote: '真诚是你最无敌的必杀技，只要给一点点善意，就能回馈一整片晴朗夏天。',
      tags: ['直球小太阳', '满格能量', '超好哄'],
      buddy: { name: '高冷黑猫', desc: '你的滚烫直球刚好能融化黑猫的高冷防线' },
      rival: { name: '战略雪豹', desc: '你热情分享十件事，对方只冷静回了句“挺好”' },
      avatarEmoji: '🐶',
      targetVector: { zen: 58, bond: 94, sharp: 44, tsundere: 16 },
      extra: {
        animalKey: 'samoyed',
        animalEn: 'HAPPY SAMOYED',
        animalPills: ['直球小太阳', '气氛担当', '双向奔赴'],
        feedingGuide: '多夸奖、多回应、多约好吃的，绝不要冷暴力——冷落 2 小时小狗耳朵就会耷拉下来。'
      }
    },
    {
      code: 'BLACKCAT',
      title: '高冷黏人黑猫',
      subtitle: '选择性贴贴 · 只对认证人类撒娇',
      rarity: '仅 5.9% 测出',
      quote: '在外人眼里自带结界难以靠近，但在认定的人面前其实超级喜欢赖着贴贴。',
      tags: ['领地意识', '熟人黏人', '精致挑剔'],
      buddy: { name: '快乐萨摩耶', desc: '对方永远热情不计较，给你满满的安全感' },
      rival: { name: '卷王边牧', desc: '对方总想替你规划人生，猫科动物最讨厌被管束' },
      avatarEmoji: '🐈‍⬛',
      targetVector: { zen: 44, bond: 64, sharp: 56, tsundere: 88 },
      extra: {
        animalKey: 'blackcat',
        animalEn: 'VELVET BLACK CAT',
        animalPills: ['选择性黏人', '领地意识', '细腻敏锐'],
        feedingGuide: '保持耐心与稳定情绪，等 TA 自己蹭过来的时候用力抱住就好，千万别硬撸。'
      }
    },
    {
      code: 'COLLIE',
      title: '卷王边牧',
      subtitle: '智商担当 · 清单打钩重度强迫症',
      rarity: '仅 5.4% 测出',
      quote: '脑子转得比谁都快，嘴上喊着想摆烂，手里却顺手把全队最难的活全干完了。',
      tags: ['高能大脑', '责任感爆棚', '嘴上喊累'],
      buddy: { name: '战略雪豹', desc: '同频高执行力，强强联手能把任何事办漂亮' },
      rival: { name: '松弛水豚', desc: '看着对方天塌不惊慢悠悠喝茶，你急得直转圈' },
      avatarEmoji: '🐕',
      targetVector: { zen: 24, bond: 72, sharp: 90, tsundere: 42 },
      extra: {
        animalKey: 'collie',
        animalEn: 'GENIUS COLLIE',
        animalPills: ['最强大脑', '行动派', '靠谱天花板'],
        feedingGuide: '给 TA 清晰反馈和足够认可，定期强制没收待办清单拉去放风休息。'
      }
    },
    {
      code: 'RED_PANDA',
      title: '炸毛小熊猫',
      subtitle: '奶凶可爱 · 举起双爪装凶但毫无威慑力',
      rarity: '仅 6.6% 测出',
      quote: '遇到不公第一个举爪奶凶护短，但只要递上一口好吃的，三秒钟立刻消气。',
      tags: ['奶凶护短', '心软好哄', '鲜活情绪'],
      buddy: { name: '松弛水豚', desc: '你负责鲜活可爱，水豚负责兜底所有小情绪' },
      rival: { name: '战略雪豹', desc: '对方太冷静理性，吵架都吵不起来气得跺脚' },
      avatarEmoji: '🦝',
      targetVector: { zen: 34, bond: 82, sharp: 48, tsundere: 68 },
      extra: {
        animalKey: 'redpanda',
        animalEn: 'FLUFFY RED PANDA',
        animalPills: ['奶凶护短', '情绪鲜活', '一口哄好'],
        feedingGuide: '炸毛时千万别讲大道理，先递小零食 + 顺毛夸两句，立刻解锁软萌形态。'
      }
    },
    {
      code: 'KOALA',
      title: '抱树考拉',
      subtitle: '慢频治愈 · 拒绝一切无意义消耗',
      rarity: '仅 6.1% 测出',
      quote: '世界这么吵，我只想找一棵舒服的树抱着，用自己的慢节拍把日子过暖。',
      tags: ['慢频生活', '高纯度宅', '温柔慢热'],
      buddy: { name: '嘴硬藏狐', desc: '都不爱瞎折腾，窝在一起追剧吃零食就是天堂' },
      rival: { name: '战略雪豹', desc: '对方气场太紧绷，让你只想往树顶再爬高两米' },
      avatarEmoji: '🐨',
      targetVector: { zen: 86, bond: 62, sharp: 18, tsundere: 52 },
      extra: {
        animalKey: 'koala',
        animalEn: 'COZY KOALA',
        animalPills: ['慢频治愈', '高纯度宅', '温柔无害'],
        feedingGuide: '别催促、别突然搞突袭社交，提前约好并留足午睡时间就是最高礼遇。'
      }
    }
  ]
};
