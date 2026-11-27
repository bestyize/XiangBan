import type { QuestionSpec, TestDefinition } from '../engine/types.ts';

function makeAttachQuestion(
  id: string,
  tag: string,
  title: string,
  sub: string,
  dim: 'anxiety' | 'avoidance',
  reverse = false
): QuestionSpec {
  const s = reverse ? -1 : 1;
  return {
    id,
    tag,
    title,
    sub,
    options: [
      { key: 'A', text: '太戳了，完全就是我的真实写照', sub: '高度符合', weights: { [dim]: 3 * s } },
      { key: 'B', text: '有点像，亲密关系里经常这样', sub: '比较符合', weights: { [dim]: 1 * s } },
      { key: 'C', text: '偶尔闪过，但大多时候还好', sub: '不太符合', weights: { [dim]: -1 * s } },
      { key: 'D', text: '完全不像，我几乎不会这样', sub: '极不符合', weights: { [dim]: -3 * s } }
    ]
  };
}

const questions: QuestionSpec[] = [
  makeAttachQuestion('at1', '安全感 · 1/16', '当在意的人回复消息突然变慢或语气冷了一点，我很容易担心是不是自己做错了什么', 'ECR 焦虑维度', 'anxiety'),
  makeAttachQuestion('at2', '边界感 · 2/16', '当一段关系迅速升温变得太亲密时，我反而会下意识想往后退半步透口气', 'ECR 回避维度', 'avoidance'),
  makeAttachQuestion('at3', '确认欲 · 3/16', '我经常需要对方用明确的言语或行动反复确认“你依然坚定选我”，心里才踏实', 'ECR 焦虑维度', 'anxiety'),
  makeAttachQuestion('at4', '自我暴露 · 4/16', '哪怕对方是我很喜欢的人，我也很难把内心最脆弱无助的一面彻底摊开给 TA 看', 'ECR 回避维度', 'avoidance'),
  makeAttachQuestion('at5', '分离焦虑 · 5/16', '如果吵完架对方提出“各自冷静一下”，等待的那几个小时对我来说格外煎熬', 'ECR 焦虑维度', 'anxiety'),
  makeAttachQuestion('at6', '独立偏好 · 6/16', '遇到困难或难过时，我本能想自己消化扛过去，不太习惯依赖伴侣安慰', 'ECR 回避维度', 'avoidance'),
  makeAttachQuestion('at7', '稳态信任 · 7/16', '我内心笃定自己是值得被好好爱着的，不太会因为对方一时忙碌就怀疑感情变质', '反向计分题', 'anxiety', true),
  makeAttachQuestion('at8', '舒适贴近 · 8/16', '向伴侣撒娇求助或彼此深度依赖，会让我觉得温暖自然而不是被束缚', '反向计分题', 'avoidance', true),
  makeAttachQuestion('at9', '情绪雷达 · 9/16', '我极其敏锐于伴侣的一丝情绪波动，常常在脑内预演“万一我们要走散了怎么办”', 'ECR 焦虑维度', 'anxiety'),
  makeAttachQuestion('at10', '抽离防御 · 10/16', '一旦察觉对方情绪太浓烈或期待太高，我会本能开启冷静抽离模式保护自己', 'ECR 回避维度', 'avoidance'),
  makeAttachQuestion('at11', '被爱期待 · 11/16', '我偶尔会觉得自己付出的投入度比对方高，害怕对方没有我想象中那么在乎我', 'ECR 焦虑维度', 'anxiety'),
  makeAttachQuestion('at12', '空间底线 · 12/16', '比起整天黏在一起，我更看重关系里绝对不受打扰的个人自由领地', 'ECR 回避维度', 'avoidance'),
  makeAttachQuestion('at13', '坦然表达 · 13/16', '有委屈或需求时，我能平和清晰地告诉对方，而不必靠试探或猜心来验证爱意', '反向计分题', 'anxiety', true),
  makeAttachQuestion('at14', '信任托付 · 14/16', '我愿意在亲密关系里放下防备，让对方真正走进我生活和情绪的核心区域', '反向计分题', 'avoidance', true),
  makeAttachQuestion('at15', '失衡敏锐 · 15/16', '如果对方临时取消约定，即便知道理由合理，我心里还是会泛起明显的失落与不安', 'ECR 焦虑维度', 'anxiety'),
  makeAttachQuestion('at16', '退守习惯 · 16/16', '与其在关系里承担被伤害的风险，我潜意识里觉得还是保持三分清醒留白更安全', '最后一题 · 生成依恋四象限坐标', 'avoidance')
];

export const attachmentTest: TestDefinition = {
  id: 'attachment',
  title: '亲密关系依恋类型',
  shortTitle: '依恋象限',
  subtitle: 'ECR 四象限坐标图 · 深度情感合盘',
  heroDesc: '基于经典成人依恋理论（ECR）：从「焦虑维度 × 回避维度」精准定位你在亲密关系中的真实落点，极适合情侣互测合盘！',
  category: 'relation',
  badge: '情感顶流',
  estimatedMinutes: 2,
  cardBg: '#FFC6D9',
  posterBg: '#FFC6D9',
  tagColor: '#FFC6D9',
  accentColor: '#E0243C',
  scoringMode: 'axis-code',
  dimensions: [
    { id: 'anxiety', label: '依恋焦虑', highLabel: '高敏感确认', lowLabel: '稳态信任', highCode: 'A', lowCode: 'S', hotWhenHigh: true },
    { id: 'avoidance', label: '亲密回避', highLabel: '高独立防御', lowLabel: '舒适靠近', highCode: 'V', lowCode: 'C', hotWhenHigh: true }
  ],
  microHints: [
    '✦ 正在测绘你的亲密关系安全感坐标…',
    '✦ 已定位你的焦虑敏感度轴线',
    '✦ 正在扫描你的亲密距离舒适区…',
    '✦ 即将点亮你的 ECR 四象限落点'
  ],
  disclaimer: '本测评参考成人亲密关系依恋理论（ECR）独立编写，用于自我觉察与关系沟通。',
  questions,
  profiles: [
    {
      code: 'SC',
      title: '稳态安全型依恋',
      subtitle: '低焦虑 × 低回避 · 亲密关系里的定海神针',
      rarity: '仅 26% 测出',
      quote: '既能舒展地爱与被爱，也能坦然独处不拧巴；和你谈恋爱是一件低内耗且极具治愈感的事。',
      tags: ['内核稳固', '直球信任', '顶级治愈力'],
      buddy: { name: '任何依恋皆可治愈', desc: '你的情绪稳定与坦诚沟通，能让所有不安全感慢慢融化' },
      rival: { name: '极端冷战操控者', desc: '你愿意认真沟通解决问题，最受不了蓄意拿捏和冷暴力' },
      avatarEmoji: '🕊️',
      extra: { quadrant: 'secure' }
    },
    {
      code: 'AC',
      title: '高敏焦虑型依恋',
      subtitle: '高焦虑 × 低回避 · 深情滚烫却怕走散的纯爱雷达',
      rarity: '仅 24% 测出',
      quote: '你把爱意给得毫无保留，之所以偶尔敏感多想，只是因为你太珍惜、太害怕被突然丢下。',
      tags: ['高纯深情', '秒回刚需', '渴望确定感'],
      buddy: { name: '稳态安全型依恋', desc: '对方的事事有回应和稳定偏爱，能让你彻底放下不安' },
      rival: { name: '独立回避型依恋', desc: '你越追问对方越后退，经典“追逃循环”极度耗电' },
      avatarEmoji: '💓',
      extra: { quadrant: 'anxious' }
    },
    {
      code: 'SV',
      title: '独立回避型依恋',
      subtitle: '低焦虑 × 高回避 · 清醒克制的高边界独行者',
      rarity: '仅 22% 测出',
      quote: '不是不懂爱，而是你太习惯靠自己兜底——太近会窒息，给足呼吸感你反而更长情靠谱。',
      tags: ['独立自主', '慢热防备', '空间至上'],
      buddy: { name: '稳态安全型依恋', desc: '不催逼不情绪勒索，给足尊重让你自愿一步步敞开心门' },
      rival: { name: '高敏焦虑型依恋', desc: '高密度查岗与情绪追问会瞬间触发你的防御退守开关' },
      avatarEmoji: '🏔️',
      extra: { quadrant: 'avoidant' }
    },
    {
      code: 'AV',
      title: '高敏矛盾型依恋',
      subtitle: '高焦虑 × 高回避 · 既渴望靠近又悄悄筑墙',
      rarity: '仅 18% 测出',
      quote: '一边无比渴望被坚定偏爱，一边又在靠近前悄悄多筑半堵安全墙——只为护住那颗极其柔软的心。',
      tags: ['外冷内热', '渴望偏爱', '自我保护壳'],
      buddy: { name: '稳态安全型依恋', desc: '对方温柔坚定、不因你试探就转身离开，是最好的解药' },
      rival: { name: '忽冷忽热玩家', desc: '模糊不确定的态度会让你的内心雷达瞬间过载警报' },
      avatarEmoji: '🦔',
      extra: { quadrant: 'fearful' }
    }
  ]
};
