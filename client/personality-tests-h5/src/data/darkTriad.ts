import type { QuestionSpec, TestDefinition } from '../engine/types.ts';

function makeDarkQuestion(
  id: string,
  tag: string,
  title: string,
  sub: string,
  dim: 'mach' | 'narc' | 'cold',
  reverse = false
): QuestionSpec {
  const s = reverse ? -1 : 1;
  return {
    id,
    tag,
    title,
    sub,
    options: [
      { key: 'A', text: '完全赞同，现实本来就是这样', sub: '高认同', weights: { [dim]: 3 * s } },
      { key: 'B', text: '多数时候确实如此', sub: '偏认同', weights: { [dim]: 1 * s } },
      { key: 'C', text: '不太同意，我更看重真诚和感性', sub: '偏不认同', weights: { [dim]: -1 * s } },
      { key: 'D', text: '完全不认同，我绝不这么做事', sub: '强烈反对', weights: { [dim]: -3 * s } }
    ]
  };
}

const questions: QuestionSpec[] = [
  makeDarkQuestion('dt1', '战略谋略 · 1/18', '与人博弈或谈判时，亮出全部底牌是天真的，永远要给自己留一张后手', '战略思维维度', 'mach'),
  makeDarkQuestion('dt2', '主体气场 · 2/18', '站在人群里，我内心其实相信自己只要发力就能做到比大多数人更耀眼', '自我笃定维度', 'narc'),
  makeDarkQuestion('dt3', '冷静果决 · 3/18', '遇到危机或必须止损的时刻，我能迅速切断情绪牵绊，做出最清醒的决定', '情绪隔离维度', 'cold'),

  makeDarkQuestion('dt4', '战略谋略 · 4/18', '看懂局势里的利益链条和人性弱点，往往比闷头苦干更容易拿到结果', '局势洞察', 'mach'),
  makeDarkQuestion('dt5', '主体气场 · 5/18', '我很享受成为某个领域的主角，被人欣赏和仰望会让我充满掌控感', '聚光灯张力', 'narc'),
  makeDarkQuestion('dt6', '冷静果决 · 6/18', '有人企图用卖惨或道德绑架来拿捏我时，我内心毫无波澜甚至有点想笑', '反道德绑架护盾', 'cold'),

  makeDarkQuestion('dt7', '战略谋略 · 7/18', '遇到不公平的坑时，我倾向先不动声色收集筹码，等时机成熟再优雅反击', '延迟反杀策略', 'mach'),
  makeDarkQuestion('dt8', '主体气场 · 8/18', '我很难忍受平庸无聊的人生，骨子里总想留下点别人替代不了的狠作品', '主体野心', 'narc'),
  makeDarkQuestion('dt9', '冷静果决 · 9/18', '如果一段关系或合作已经明显持续内耗，我可以非常干脆地一键清空止损', '断舍离决断', 'cold'),

  makeDarkQuestion('dt10', '战略谋略 · 10/18', '我习惯把所有想法毫无保留地摊开给刚认识的人，不爱费心琢磨策略', '反向计分题', 'mach', true),
  makeDarkQuestion('dt11', '主体气场 · 11/18', '我其实更喜欢低调隐身在幕后，被大家盯着夸奖反而会觉得浑身不自在', '反向计分题', 'narc', true),
  makeDarkQuestion('dt12', '冷静果决 · 12/18', '哪怕对方之前冒犯过我，只要他来道个歉，我心一软很快就会原谅', '反向计分题', 'cold', true),

  makeDarkQuestion('dt13', '战略谋略 · 13/18', '在复杂职场或社交场里，说真话的时机和包装方式比真话本身更重要', '沟通谋略', 'mach'),
  makeDarkQuestion('dt14', '主体气场 · 14/18', '只要是我认准的标准，别人再怎么指手画脚也很难动摇我的自我评价', '高主体性', 'narc'),
  makeDarkQuestion('dt15', '冷静果决 · 15/18', '压力爆棚或别人慌成一团时，我的心跳和头脑反而会奇异地变得无比冷静', '高压冷血稳态', 'cold'),

  makeDarkQuestion('dt16', '战略谋略 · 16/18', '真正聪明的棋手从不硬碰硬，而是顺着对手的力道把局面引向自己有利的方向', '借力打力', 'mach'),
  makeDarkQuestion('dt17', '主体气场 · 17/18', '我值得拥有最好的资源、最舒服的边界和最高质量的同频圈层', '高配得感', 'narc'),
  makeDarkQuestion('dt18', '冷静果决 · 18/18', '善良必须带点锋芒——没有牙的温柔，在现实里只会被当成好欺负', '最后一题 · 解锁你的暗面锋芒', 'cold')
];

export const darkTriadTest: TestDefinition = {
  id: 'darktriad',
  title: '人格暗面 · 三联征',
  shortTitle: '人格暗面',
  subtitle: '策略谋略 × 主体气场 × 冷静果决',
  heroDesc: '基于经典黑暗三联征（SD3）维度去污名化重构：测测你骨子里的战略谋略、主体野心与反情绪勒索锋芒！',
  category: 'classic',
  badge: '高燃反差',
  estimatedMinutes: 2,
  cardBg: '#FF7885',
  posterBg: '#FF7885',
  tagColor: '#FF7885',
  accentColor: '#FF4D5E',
  scoringMode: 'nearest-profile',
  dimensions: [
    { id: 'mach', label: '战略谋略', hotWhenHigh: true },
    { id: 'narc', label: '主体气场', hotWhenHigh: true },
    { id: 'cold', label: '冷静果决', hotWhenHigh: true }
  ],
  microHints: [
    '✦ 正在扫描你礼貌外表下的暗面锋芒…',
    '✦ 已测出你的战略思维与反拿捏系数',
    '✦ 正在校准你的主体配得感与止损速度…',
    '✦ 你的暗黑人格代号即将解锁'
  ],
  disclaimer: '本测评基于心理特质研究（Dark Triad）进行中性化趣味解读，高分代表高边界、高主体与强策略感，非负面标签。',
  questions,
  profiles: [
    {
      code: 'DARK-S',
      title: '清醒黑桃棋手',
      subtitle: '高策略 × 高果决 · 优雅看透局势的幕后操盘手',
      rarity: '仅 4.3% 测出',
      quote: '并非冷血无情，只是你比大多数人更早看透规则，并且绝不允许任何人拿捏你的底线。',
      tags: ['看透不说透', '优雅反杀', '反道德绑架'],
      buddy: { name: '铁血破局统帅', desc: '强强联手直奔结果，彼此欣赏绝不矫情内耗' },
      rival: { name: '情感绑架大师', desc: '企图用卖惨控制你，只会被你一眼看穿并拉黑' },
      avatarEmoji: '♠️',
      targetVector: { mach: 90, narc: 64, cold: 84 },
      extra: { darkBadge: 'BLACK SPADE' }
    },
    {
      code: 'DARK-K',
      title: '天生红桃王座',
      subtitle: '高主体 × 高野心 · 自带聚光灯的顶级配得感',
      rarity: '仅 5.1% 测出',
      quote: '你从不羞于承认自己的野心与光芒，高配得感让你自带把资源和焦点吸引到身边的磁场。',
      tags: ['顶级配得感', '主角磁场', '不甘平庸'],
      buddy: { name: '清醒黑桃棋手', desc: '一个站在台前光芒万丈，一个在幕后布好天罗地网' },
      rival: { name: '打压式说教者', desc: '想通过贬低你来建立权威？你只会礼貌请他出局' },
      avatarEmoji: '👑',
      targetVector: { mach: 64, narc: 94, cold: 58 },
      extra: { darkBadge: 'CROWN AURA' }
    },
    {
      code: 'DARK-I',
      title: '冰刃止损猎手',
      subtitle: '超高冷静 × 极致断舍离 · 越乱越清醒的狠角色',
      rarity: '仅 4.8% 测出',
      quote: '你最可怕的超能力不是进攻，而是随时能面不改色地切断一切消耗你的人与事。',
      tags: ['一键止损', '高压冷血', '免疫内耗'],
      buddy: { name: '清醒黑桃棋手', desc: '理性至上、边界清晰，合作起来丝滑得像精密齿轮' },
      rival: { name: '拉扯反复戏精', desc: '对方刚演到第二幕，你已经默默把联系通道封死' },
      avatarEmoji: '🗡️',
      targetVector: { mach: 62, narc: 52, cold: 94 },
      extra: { darkBadge: 'ICE BLADE' }
    },
    {
      code: 'DARK-X',
      title: '暗夜三冠魔王',
      subtitle: '三联征全线高能 · 兼具谋略、气场与杀伐果决',
      rarity: '仅 2.9% 测出',
      quote: '你拥有完整的强者底层系统：看得懂人心、扛得住大局、狠得下心止损——只是平时懒得亮爪。',
      tags: ['稀有高能', '自带锋芒', '大女主/大男主'],
      buddy: { name: '纯白治愈天使', desc: '只有彻底真诚不玩套路的人，才配让你卸下所有铠甲' },
      rival: { name: '另一位暗夜魔王', desc: '一山难容二虎，除非你们找到共同要拿下的更大蛋糕' },
      avatarEmoji: '🖤',
      targetVector: { mach: 88, narc: 88, cold: 88 },
      extra: { darkBadge: 'TRIPLE CROWN' }
    },
    {
      code: 'LIGHT-G',
      title: '带锋芒的守夜人',
      subtitle: '中度锋芒 · 心里有光，手里也有防身短刃',
      rarity: '仅 14.6% 测出',
      quote: '你本质善良温和，但经历过世事后长出了恰到好处的边界感：不主动害人，也绝不任人欺负。',
      tags: ['柔中带刚', '清醒善良', '平衡大师'],
      buddy: { name: '天生红桃王座', desc: '既欣赏对方的野心，又能帮对方守住人情温度' },
      rival: { name: '双标吸血鬼', desc: '平时好说话不代表没脾气，踩过红线立刻变脸' },
      avatarEmoji: '🛡️',
      targetVector: { mach: 54, narc: 54, cold: 54 },
      extra: { darkBadge: 'BALANCED EDGE' }
    },
    {
      code: 'LIGHT-W',
      title: '纯白稀缺白月光',
      subtitle: '极低暗面 · 拒绝套路的绝对真诚派',
      rarity: '仅 9.8% 测出',
      quote: '在人人都研究城府和套路的时代，你依然选择用直白与真心面对世界——这本身就是最顶级的勇敢。',
      tags: ['零套路', '绝对真诚', '稀缺净土'],
      buddy: { name: '带锋芒的守夜人', desc: '对方会像骑士一样，替你挡掉外界那些弯弯绕绕' },
      rival: { name: '高阶套路玩家', desc: '记住偶尔多留一个心眼，别把真心浪费在错的人身上' },
      avatarEmoji: '🤍',
      targetVector: { mach: 20, narc: 26, cold: 24 },
      extra: { darkBadge: 'PURE LIGHT' }
    }
  ]
};
