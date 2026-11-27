import type { QuestionSpec, TestDefinition } from '../engine/types.ts';

function makeLikertQuestion(
  id: string,
  tag: string,
  title: string,
  sub: string,
  dim: 'open' | 'consc' | 'extra' | 'agree' | 'calm',
  reverse = false
): QuestionSpec {
  const s = reverse ? -1 : 1;
  return {
    id,
    tag,
    title,
    sub,
    options: [
      { key: 'A', text: '非常符合，简直就是我本人', sub: '高度认同该描述', weights: { [dim]: 3 * s } },
      { key: 'B', text: '比较符合，多数时候是这样', sub: '偏向符合', weights: { [dim]: 1 * s } },
      { key: 'C', text: '不太符合，偶尔才会这样', sub: '偏向不符合', weights: { [dim]: -1 * s } },
      { key: 'D', text: '完全不符合，我刚好相反', sub: '高度不符合', weights: { [dim]: -3 * s } }
    ]
  };
}

const questions: QuestionSpec[] = [
  makeLikertQuestion('bf1', '开放性 · O', '我对陌生领域的知识、小众审美或新奇脑洞总是充满好奇心', '基于 OCEAN 开放性维度', 'open'),
  makeLikertQuestion('bf2', '尽责性 · C', '答应下来的事情，哪怕再琐碎我也会有条理地按时完成', '基于 OCEAN 尽责性维度', 'consc'),
  makeLikertQuestion('bf3', '外向性 · E', '在聚会或讨论中，我通常能很快打开话匣子并享受热闹氛围', '基于 OCEAN 外向性维度', 'extra'),
  makeLikertQuestion('bf4', '宜人性 · A', '别人遇到麻烦或心情低落时，我很容易共情并愿意伸手帮忙', '基于 OCEAN 宜人性维度', 'agree'),
  makeLikertQuestion('bf5', '情绪稳定 · S', '遇到突发变故或压力时，我通常能保持冷静而不太容易慌乱崩溃', '情绪稳态维度（神经质反向）', 'calm'),

  makeLikertQuestion('bf6', '开放性 · O', '相比按部就班走老路，我更喜欢尝试从未体验过的新玩法和新思路', '探索欲与创造力', 'open'),
  makeLikertQuestion('bf7', '尽责性 · C', '做重要决定或出发旅行前，我习惯提前做好规划和备选方案', '秩序与自律度', 'consc'),
  makeLikertQuestion('bf8', '外向性 · E', '周末如果一整天都在见朋友、逛街或聊天，我反而会觉得精力更充沛', '社交充电倾向', 'extra'),
  makeLikertQuestion('bf9', '宜人性 · A', '在团队协作中，我更看重彼此尊重和双赢，而不是非要争个输赢高下', '温和协作度', 'agree'),
  makeLikertQuestion('bf10', '情绪稳定 · S', '一点小失误或别人的无心评价，往往会让我反复纠结很久', '反向计分题', 'calm', true),

  makeLikertQuestion('bf11', '开放性 · O', '看电影、听歌或看展时，我经常会被作品里的隐喻、构图或氛围深深打动', '审美敏锐度', 'open'),
  makeLikertQuestion('bf12', '尽责性 · C', '我经常把该做的事拖到最后一刻才仓促动工', '反向计分题', 'consc', true),
  makeLikertQuestion('bf13', '外向性 · E', '走进陌生场合时，我倾向于主动认识新朋友而不是独自安静待着', '主动破冰倾向', 'extra'),
  makeLikertQuestion('bf14', '宜人性 · A', '我对他人通常先抱有善意预设，相信大多数人本性并不坏', '信任与善意度', 'agree'),
  makeLikertQuestion('bf15', '情绪稳定 · S', '就算白天遇到烦心事，我通常也能很快调节情绪不把坏心情带进第二天', '情绪复原力', 'calm'),

  makeLikertQuestion('bf16', '开放性 · O', '我更偏爱熟悉确定的常规体验，对太抽象或标新立异的东西兴趣不大', '反向计分题', 'open', true),
  makeLikertQuestion('bf17', '尽责性 · C', '我的工作文件、待办事项或常用物品大多保持着清晰的分类秩序', '条理与执行力', 'consc'),
  makeLikertQuestion('bf18', '外向性 · E', '长时间待在热闹人群里会让我迅速耗尽电量，急需躲回安静空间', '反向计分题', 'extra', true),
  makeLikertQuestion('bf19', '宜人性 · A', '如果别人观点明显不合理，我会直截了当地反驳，不太顾忌场面温和', '反向计分题', 'agree', true),
  makeLikertQuestion('bf20', '情绪稳定 · S', '面对未来不确定性时，我很容易提前陷入焦虑和胡思乱想', '反向计分题', 'calm', true),

  makeLikertQuestion('bf21', '开放性 · O', '我喜欢和观点完全不同的人交流，因为那能帮我打开全新的视角', '认知弹性', 'open'),
  makeLikertQuestion('bf22', '尽责性 · C', '一旦定下长期目标，即使过程枯燥我也能靠自律一步步坚持下来', '长期毅力', 'consc'),
  makeLikertQuestion('bf23', '外向性 · E', '朋友们普遍觉得我表达欲强、有感染力，是容易带动气氛的人', '外放感染力', 'extra'),
  makeLikertQuestion('bf24', '宜人性 · A', '只要不触及原则底线，我愿意体谅别人的难处并做出适度让步', '同理与宽容度', 'agree'),
  makeLikertQuestion('bf25', '情绪稳定 · S', '整体而言，我的情绪底盘像稳压器一样平和，很少大起大落', '最后一题 · 生成五维雷达图', 'calm')
];

export const bigFiveTest: TestDefinition = {
  id: 'bigfive',
  title: '大五人格科学画像',
  shortTitle: '大五雷达',
  subtitle: 'OCEAN 五维雷达图 · 心理学界金标准',
  heroDesc: '心理学界公认信效度最高的人格模型！不贴刻板标签，用 25 道精选情景题绘出你的 OCEAN 五维科学雷达图。',
  category: 'classic',
  badge: '科学权威',
  estimatedMinutes: 3,
  cardBg: '#A9F0CA',
  posterBg: '#A9F0CA',
  tagColor: '#A9F0CA',
  accentColor: '#158F53',
  scoringMode: 'nearest-profile',
  dimensions: [
    { id: 'open', label: '开放性' },
    { id: 'consc', label: '尽责性' },
    { id: 'extra', label: '外向性' },
    { id: 'agree', label: '宜人性' },
    { id: 'calm', label: '情绪稳定' }
  ],
  microHints: [
    '✦ 正在采样你的 OCEAN 五大核心特质…',
    '✦ 已建立你的开放性与自律度坐标',
    '✦ 正在校准你的社交能量与共情倾角…',
    '✦ 五维科学雷达图即将绘制完成'
  ],
  disclaimer: '本测评参考公有领域 Big Five（OCEAN）五因素模型独立编制，用于科学自我探索。',
  questions,
  profiles: [
    {
      code: 'OCEAN-S',
      title: '高开放沉思学者',
      subtitle: '高开放 × 高稳定 · 拥有独立宇宙的思想者',
      rarity: '仅 6.1% 测出',
      quote: '好奇心与审美维度拉满，兼顾独立思考与温和边界，是越深入了解越迷人的耐看型灵魂。',
      tags: ['高开放好奇', '独立清醒', '情绪稳态'],
      buddy: { name: '温暖高宜人协作者', desc: '你的奇思妙想总能被对方温柔接住并温暖落地' },
      rival: { name: '强硬教条执行官', desc: '拒绝讨论任何可能性的死板沟通会迅速让你沉默' },
      avatarEmoji: '🔭',
      targetVector: { open: 88, consc: 66, extra: 38, agree: 70, calm: 76 }
    },
    {
      code: 'OCEAN-L',
      title: '全景驱动领航员',
      subtitle: '高尽责 × 高外向 × 高稳定 · 稀缺六边形支柱',
      rarity: '仅 4.7% 测出',
      quote: '行动力、感染力与抗压能力全线在线，既看得见远方也落得下细节，是团队天然定海神针。',
      tags: ['高尽责执行', '稳定抗压', '天生领航'],
      buddy: { name: '灵动创意造梦师', desc: '对方源源不断的脑洞，在你手里能变成真正的作品' },
      rival: { name: '低尽责拖延怪', desc: '面对反复放鸽子和毫无计划的协作会让你血压微升' },
      avatarEmoji: '🧭',
      targetVector: { open: 72, consc: 90, extra: 84, agree: 66, calm: 86 }
    },
    {
      code: 'OCEAN-W',
      title: '温暖高宜人治愈者',
      subtitle: '高宜人 × 高共情 · 自带安全场域的温柔力量',
      rarity: '仅 7.4% 测出',
      quote: '你拥有极高的情感雷达与善意底色，待在你身边的人总能不自觉卸下防备、感到被接纳。',
      tags: ['高宜人共情', '值得托付', '柔和坚韧'],
      buddy: { name: '高开放沉思学者', desc: '彼此尊重边界又精神相通，舒服得毫无消耗' },
      rival: { name: '锋利竞争辩手', desc: '把每场聊天都变成辩论赛的人最容易让你心累' },
      avatarEmoji: '🌱',
      targetVector: { open: 64, consc: 68, extra: 62, agree: 92, calm: 64 }
    },
    {
      code: 'OCEAN-C',
      title: '灵动创意造梦师',
      subtitle: '超高开放 × 高外向 · 永远在探索旷野的灵感源',
      rarity: '仅 6.8% 测出',
      quote: '你厌恶一成不变的轨道，大脑像装了万花筒，总能把平淡日子过出意想不到的彩蛋。',
      tags: ['创意爆棚', '拥抱变化', '鲜活感染力'],
      buddy: { name: '全景驱动领航员', desc: '你负责想象一万种可能，对方负责帮你搭起通天梯' },
      rival: { name: '极简严谨建筑师', desc: '被要求按分钟打卡填表会瞬间封印你的创造力' },
      avatarEmoji: '🎨',
      targetVector: { open: 94, consc: 38, extra: 86, agree: 72, calm: 54 }
    },
    {
      code: 'OCEAN-A',
      title: '极简严谨建筑师',
      subtitle: '超高尽责 × 高稳定 · 秩序与靠谱的极致化身',
      rarity: '仅 5.9% 测出',
      quote: '不靠情绪和口号做事，你用清晰边界、稳定产出与高度自律为自己构建出稳固的安全感。',
      tags: ['秩序大师', '言出必行', '防弹稳态'],
      buddy: { name: '温暖高宜人治愈者', desc: '对方给你的严谨日常注入恰到好处的柔软烟火气' },
      rival: { name: '灵动创意造梦师', desc: '对方临出发前五分钟改主意，让你心跳漏掉一拍' },
      avatarEmoji: '📐',
      targetVector: { open: 44, consc: 92, extra: 42, agree: 52, calm: 84 }
    },
    {
      code: 'OCEAN-M',
      title: '高敏细腻感受家',
      subtitle: '高开放 × 高敏感 · 捕捉世间微光的艺术雷达',
      rarity: '仅 7.1% 测出',
      quote: '你的情绪颗粒度远比常人精细，容易为小事起涟漪，但也因此拥有常人难以企及的洞察与共情深度。',
      tags: ['细腻感知', '审美共鸣', '深情敏锐'],
      buddy: { name: '全景驱动领航员', desc: '对方稳如泰山的情绪底盘，能让你安心舒展所有天赋' },
      rival: { name: '粗线条硬核钢铁侠', desc: '对方一句无心直男吐槽，可能让你深夜琢磨半小时' },
      avatarEmoji: '🫧',
      targetVector: { open: 84, consc: 54, extra: 36, agree: 76, calm: 24 }
    }
  ]
};
