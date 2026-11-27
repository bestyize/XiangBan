import type { TestDefinition } from '../engine/types.ts';

export const mentalCheckupTest: TestDefinition = {
  id: 'checkup',
  title: '精神电量体检单',
  shortTitle: '电量体检',
  subtitle: '测精神年龄、班味浓度与内耗来源',
  heroDesc: '非医疗趣味状态体检：一键测出你的实测精神年龄、心理剩余电量、班味浓度与专属电子处方！',
  category: 'viral',
  badge: '打工人戳',
  estimatedMinutes: 2,
  cardBg: '#8DD3FF',
  posterBg: '#8DD3FF',
  tagColor: '#8DD3FF',
  accentColor: '#2364AA',
  scoringMode: 'nearest-profile',
  dimensions: [
    { id: 'drain', label: '电量损耗', hotWhenHigh: true },
    { id: 'banwei', label: '班味浓度', hotWhenHigh: true },
    { id: 'retire', label: '提前退休欲', hotWhenHigh: true },
    { id: 'heal', label: '自愈韧性', hotWhenHigh: false }
  ],
  microHints: [
    '✦ 正在连接你的精神心电图…',
    '✦ 检测到轻微班味与电子榨菜依赖',
    '✦ 正在估算你的实测精神年龄…',
    '✦ 检验报告单正在打印，附赠充电处方'
  ],
  disclaimer: '本测评为生活化情绪与压力趣味自查，不构成任何临床医疗诊断。',
  questions: [
    {
      id: 'mc1',
      tag: '早起开机',
      title: '工作日早上闹钟响第一声时，你重启开机需要多久？',
      sub: '开机耗电监测',
      options: [
        { key: 'A', text: '每隔 5 分钟设一个闹钟，躺到最后 1 秒垂死病中惊坐起', sub: '开机即掉电 30%', weights: { drain: 3, banwei: 2, retire: 2 } },
        { key: 'B', text: '在床边坐着眼神放空发呆 10 分钟，灵魂比肉体晚到半小时', sub: '精神已率先提交退休申请', weights: { retire: 3, drain: 2 } },
        { key: 'C', text: '一响就起，刷牙听播客，效率极高像上了发条', sub: '电量管理稳健', weights: { heal: 3, drain: -2 } },
        { key: 'D', text: '看一眼手机群消息，还没出门班味就已经漫出来了', sub: '重度工作信息反射', weights: { banwei: 3, drain: 2 } }
      ]
    },
    {
      id: 'mc2',
      tag: '吃饭仪式',
      title: '一个人点好外卖准备开吃时，如果突然找不到下饭视频（电子榨菜），你会？',
      sub: '电子榨菜依赖检测',
      options: [
        { key: 'A', text: '宁可饭凉掉也要先翻 15 分钟找到完美视频才肯动第一口筷', sub: '重度电子榨菜阳性', weights: { drain: 2, retire: 2 } },
        { key: 'B', text: '随手点开看过十遍的老剧背景音，熟悉才安心', sub: '大脑拒绝接收任何新剧情耗电', weights: { drain: 3, retire: 3 } },
        { key: 'C', text: '顺手回几条工作消息或刷行业资讯把饭吃完', sub: '班味渗入进食系统', weights: { banwei: 3 } },
        { key: 'D', text: '不看屏幕专心品尝食物，顺便放空挺舒服的', sub: '高专注自愈力', weights: { heal: 3, drain: -1 } }
      ]
    },
    {
      id: 'mc3',
      tag: '语言习惯',
      title: '最近和朋友私下聊天时，你有没有不小心冒出过“对齐、闭环、辛苦了、收到”？',
      sub: '班味渗透率测试',
      options: [
        { key: 'A', text: '高频触发！甚至跟外卖小哥和家里长辈都差点说“好的收到”', sub: '班味浓度严重超标', weights: { banwei: 3, drain: 1 } },
        { key: 'B', text: '偶尔嘴瓢，说完自己嫌弃得浑身起鸡皮疙瘩', sub: '中度班味感染，尚有自救意识', weights: { banwei: 2, heal: 1 } },
        { key: 'C', text: '下班一秒彻底切换语言系统，绝不让工作污染私生活', sub: '物理级防班味结界', weights: { heal: 3, banwei: -2 } },
        { key: 'D', text: '我最近连打字都嫌累，基本只发表情包和“嗯嗯”', sub: '极度省电休眠模式', weights: { drain: 3, retire: 2 } }
      ]
    },
    {
      id: 'mc4',
      tag: '休假幻想',
      title: '刷到别人在大理晒太阳、海边发呆或提前养老的视频，你的心动值是？',
      sub: '精神退休指数',
      options: [
        { key: 'A', text: '1000%！已经认真算过存够多少钱可以立刻去县城躺平', sub: '精神年龄直逼 70 岁', weights: { retire: 3, drain: 2 } },
        { key: 'B', text: '羡慕三秒钟，然后默默收藏进“等我有空”吃灰夹', sub: '理性打工人日常望梅止渴', weights: { banwei: 2, retire: 2 } },
        { key: 'C', text: '觉得旅游几天挺好，真让我天天闲着反而不习惯', sub: '能量饱满的实干派', weights: { heal: 2, retire: -2 } },
        { key: 'D', text: '立刻开始查机票和周末短途攻略，这周就去吹吹风', sub: '超强行动自愈力', weights: { heal: 3, drain: -1 } }
      ]
    },
    {
      id: 'mc5',
      tag: '社交耗电',
      title: '连上五天班/课后，周五晚上有人约你去热闹的聚会喝酒，你会？',
      sub: '剩余电量评估',
      options: [
        { key: 'A', text: '光是想象要化妆出门寒暄，电量就已经直接变红报警', sub: '急需独自静默快充', weights: { drain: 3, retire: 2 } },
        { key: 'B', text: '推掉大局，只愿意和 1 个不用装的朋友安安静静吃顿夜宵', sub: '精准省电，只见充电型人类', weights: { heal: 2, retire: 1 } },
        { key: 'C', text: '冲！正需要一场酣畅淋漓的热闹把一周郁气全散掉', sub: '越热闹越回血的满电体质', weights: { heal: 3, drain: -2, retire: -2 } },
        { key: 'D', text: '看对方是不是潜在人脉或工作相关，权衡后再决定去不去', sub: '随时在线的职业雷达', weights: { banwei: 3 } }
      ]
    },
    {
      id: 'mc6',
      tag: '通知应激',
      title: '周末下午手机突然响起类似工作软件提示音的“叮咚”声，你会？',
      sub: '条件反射检测',
      options: [
        { key: 'A', text: '心头猛地一紧，哪怕发现只是垃圾短信也缓了半分钟', sub: '重度工作应激反应', weights: { banwei: 3, drain: 3 } },
        { key: 'B', text: '早把工作通知全静音了，周末休想找到我', sub: '高境界防御大师', weights: { heal: 3, banwei: -2 } },
        { key: 'C', text: '习以为常看一眼，不急就划走继续躺平', sub: '情绪稳定不内耗', weights: { heal: 2, retire: 1 } },
        { key: 'D', text: '叹口气感叹一句“这破班什么时候是个头啊”', sub: '持续性低气压吐槽', weights: { drain: 2, retire: 3 } }
      ]
    },
    {
      id: 'mc7',
      tag: '记忆力检测',
      title: '最近你有没有出现过“拿起手机解锁，却完全忘了刚才要干嘛”的瞬间？',
      sub: '大脑内存占用率',
      options: [
        { key: 'A', text: '每天上演好几次，感觉大脑后台开了 99 个没关的标签页', sub: '内存 98% 过载预警', weights: { drain: 3, banwei: 2 } },
        { key: 'B', text: '偶尔累的时候会，睡个好觉第二天就恢复清晰了', sub: '良性恢复曲线', weights: { heal: 2, drain: 1 } },
        { key: 'C', text: '记不住琐事，但一到点饭和放假日期记得比谁都清', sub: '选择性节能大脑', weights: { retire: 2, heal: 2 } },
        { key: 'D', text: '很少，我习惯随时写进待办清单清空大脑缓存', sub: '清爽秩序型选手', weights: { heal: 3, drain: -2 } }
      ]
    },
    {
      id: 'mc8',
      tag: '快乐阈值',
      title: '最近一次让你打从心底“哇，好开心啊”的小事，通常是什么级别？',
      sub: '多巴胺敏感度',
      options: [
        { key: 'A', text: '抢到满减券、快递提前一天到、或者今晚居然不用加班', sub: '快乐阈值极低，一点甜就能哄好', weights: { retire: 2, heal: 2, banwei: 1 } },
        { key: 'B', text: '好像已经很久没特别兴奋过了，大多是淡淡的“还行”', sub: '情绪钝化低电预警', weights: { drain: 3, retire: 3 } },
        { key: 'C', text: '搞定一个难题、运动流汗或和朋友笑到肚子痛', sub: '鲜活高能生命力', weights: { heal: 3, drain: -2 } },
        { key: 'D', text: '看着账户存款数字又涨了一小截', sub: '务实安全感优先', weights: { banwei: 2, heal: 1 } }
      ]
    },
    {
      id: 'mc9',
      tag: '养生觉醒',
      title: '下面哪个“提前步入老年”的行为，你最近已经偷偷解锁了？',
      sub: '精神年龄实锤',
      options: [
        { key: 'A', text: '保温杯泡枸杞/陈皮、爱上逛公园晒太阳和八段锦', sub: '血脉觉醒型赛博养生', weights: { retire: 3, heal: 2 } },
        { key: 'B', text: '听不得太吵的音乐和网红打卡地，只想找安静茶馆坐着', sub: '精神年龄直接 +30 岁', weights: { retire: 3, drain: 1 } },
        { key: 'C', text: '一边熬到凌晨两点一边狂吃护肝片和维生素', sub: '朋克养生主力军', weights: { drain: 3, banwei: 2 } },
        { key: 'D', text: '我依然精力充沛爱折腾，觉得自己永远 18 岁', sub: '满电少年感拉满', weights: { heal: 3, retire: -3 } }
      ]
    },
    {
      id: 'mc10',
      tag: '情绪内耗',
      title: '当一天忙碌结束后躺回床上，你脑子里的主要背景音是？',
      sub: '睡前内耗检测',
      options: [
        { key: 'A', text: '反复复盘白天哪句话没说好、明天那件事会不会搞砸', sub: '高强度夜间后台耗电', weights: { drain: 3, banwei: 2 } },
        { key: 'B', text: '什么也不想，疯狂刷短视频直到手机砸脸上', sub: '报复性放松麻痹疲惫', weights: { drain: 2, retire: 2 } },
        { key: 'C', text: '复盘完一件今天做得不错的小事，安心入睡', sub: '高韧性自愈模式', weights: { heal: 3, drain: -2 } },
        { key: 'D', text: '盘算周末去哪吃顿好的，嘴角带着微笑睡着', sub: '自带幸福感转换器', weights: { heal: 3, retire: 1 } }
      ]
    },
    {
      id: 'mc11',
      tag: '辞职/退学冲动',
      title: '过去一个月里，你心里冒出“要不干脆不干了去卖烤肠”的次数大概是？',
      sub: '真实心理承压值',
      options: [
        { key: 'A', text: '几乎每周三次，烤肠摊选址和招牌名字都想好了', sub: '随时准备提桶跑路', weights: { retire: 3, drain: 3, banwei: 2 } },
        { key: 'B', text: '偶尔特别累的时候闪过一秒，发完工资立刻又好了', sub: '经典打工人周期性自愈', weights: { banwei: 2, heal: 2 } },
        { key: 'C', text: '很少，目前节奏还在掌控范围内', sub: '低耗能平稳巡航', weights: { heal: 3, drain: -2, retire: -1 } },
        { key: 'D', text: '不是想摆烂，是真心想换个更有意思的新赛道折腾', sub: '探索欲大于疲惫感', weights: { heal: 2, retire: -1 } }
      ]
    },
    {
      id: 'mc12',
      tag: '专注续航',
      title: '现在让你不看手机、静下心连续看 30 分钟长文章或书，你的难度是？',
      sub: '注意力磨损度',
      options: [
        { key: 'A', text: '极难！超过 3 分钟手指就不自觉想切出去刷点短平快的', sub: '多巴胺碎片化严重预警', weights: { drain: 3 } },
        { key: 'B', text: '开头有点浮躁，进入状态后反而能安静看很久', sub: '稍加引导即可回血', weights: { heal: 2, drain: 1 } },
        { key: 'C', text: '很轻松，沉浸阅读反而是我最舒服的充电方式', sub: '深水专注力极佳', weights: { heal: 3, drain: -2 } },
        { key: 'D', text: '看题材：如果是八卦或搞钱攻略我能一动不动看两小时', sub: '兴趣驱动型专注', weights: { retire: 1, heal: 1 } }
      ]
    },
    {
      id: 'mc13',
      tag: '周末回血',
      title: '过完一个双休日后的周日晚上，你的心理电量通常会恢复到多少？',
      sub: '充电效率测评',
      options: [
        { key: 'A', text: '刚充到 35%，想到明天周一瞬间又跌回 15%', sub: '周日晚间综合征重度患者', weights: { drain: 3, banwei: 3 } },
        { key: 'B', text: '能回到 70% 左右，如果周末没被打扰就更稳', sub: '正常充电速率', weights: { heal: 2, drain: 1 } },
        { key: 'C', text: '90%+ 满血！周末出门晒太阳/运动/见朋友太治愈了', sub: '顶级快充体质', weights: { heal: 3, drain: -2 } },
        { key: 'D', text: '其实周末两天我都在补觉和发呆，觉得一天只过了两小时', sub: '深度休眠省电模式', weights: { retire: 3, drain: 2 } }
      ]
    },
    {
      id: 'mc14',
      tag: '人情消耗',
      title: '面对必须应付的客套、人情世故和多余寒暄，你现在的耐受度是？',
      sub: '情绪耐磨损检测',
      options: [
        { key: 'A', text: '耐受度趋近于零，现在只想和舒服的人待在一起', sub: '阅尽千帆，拒绝无效社交', weights: { retire: 3, drain: 2 } },
        { key: 'B', text: '虽然累，但该走的体面流程还是会咬牙走完', sub: '高配合度耗电体', weights: { banwei: 2, drain: 2 } },
        { key: 'C', text: '游刃有余，当成轻度社交游戏一点不费劲', sub: '社交护甲拉满', weights: { heal: 3, drain: -1 } },
        { key: 'D', text: '能线上发文字解决的绝不打电话，能不去的局一律谢绝', sub: '极致节能生活方式', weights: { retire: 2, heal: 1 } }
      ]
    },
    {
      id: 'mc15',
      tag: '自我关怀',
      title: '如果现在允许你立刻对自己做一件“毫无用处但超级开心”的事，你会选？',
      sub: '寻找你的快充插头',
      options: [
        { key: 'A', text: '去公园长椅晒半小时太阳，看大爷下棋看小狗奔跑', sub: '自然光 + 慢节奏疗愈', weights: { retire: 2, heal: 3 } },
        { key: 'B', text: '关掉所有通讯软件，拉上窗帘一口气睡够 12 小时', sub: '严重缺觉缺安静，急需深度重启', weights: { drain: 3, retire: 2 } },
        { key: 'C', text: '去海边或山顶大喊一声，再炫一顿高热量碳水', sub: '释放积压压力', weights: { banwei: 2, heal: 2 } },
        { key: 'D', text: '约最铁的朋友逛街看展吃大餐，从头笑到尾', sub: '情感连接高能快充', weights: { heal: 3, drain: -1 } }
      ]
    },
    {
      id: 'mc16',
      tag: '终检签字',
      title: '用一句话形容你此刻最真实的精神状态，最贴近的是？',
      sub: '最后一题 · 立刻打印你的体检报告单',
      options: [
        { key: 'A', text: '“生理正值当年，精神已经在江南水乡提前退休”', sub: '提前退休圣体', weights: { retire: 3, drain: 2 } },
        { key: 'B', text: '“表面情绪稳定，实际电量只剩 20% 靠冰美式硬撑”', sub: '残血硬撑打工人', weights: { drain: 3, banwei: 3 } },
        { key: 'C', text: '“偶尔有点小累，但总体内核很稳、随时能把自己哄好”', sub: '自愈系健康样本', weights: { heal: 3, drain: -2 } },
        { key: 'D', text: '“半睡半醒之间，主打一个随遇而安顺其自然”', sub: '低耗能佛系巡航', weights: { retire: 2, heal: 2 } }
      ]
    }
  ],
  profiles: [
    {
      code: '68岁',
      title: '低电量提前退休体',
      subtitle: '生理正值当年 · 精神已赴江南摇椅',
      rarity: '仅 9.2% 测出',
      quote: '生理年龄风华正茂，精神状态只想找个没人的海边小院晒太阳看云。',
      tags: ['精神 68 岁', '急需充电', '电子榨菜阳性'],
      buddy: { name: '带薪发呆大师', desc: '共享工位续命奶茶清单，谁也不卷谁' },
      rival: { name: '满电早八战神', desc: '对方周一早上八点元气满满，让你呼吸微滞' },
      avatarEmoji: '🪫',
      targetVector: { drain: 84, banwei: 72, retire: 92, heal: 36 },
      extra: {
        mentalAge: 68,
        batteryPct: 27,
        batteryLabel: '27% · 残血警告',
        rows: [
          { name: '班味浓度指标', value: '偏高 ↑', status: 'flag' },
          { name: '提前退休渴望', value: '94 / 100', status: 'flag' },
          { name: '电子榨菜依赖', value: '阳性 (+)', status: 'flag' },
          { name: '独处自愈潜力', value: '良好 ✓', status: 'ok' }
        ],
        prescription: '医嘱：本周至少安排半天手机免打扰，去公园晒 20 分钟太阳，禁止睡前复盘工作。'
      }
    },
    {
      code: '54岁',
      title: '班味超标硬撑体',
      subtitle: '靠冰美式续命 · 随时准备对齐闭环',
      rarity: '仅 8.1% 测出',
      quote: '嘴上说着“好的收到马上改”，心里其实已经给烤肠摊选好了三个黄金地段。',
      tags: ['重度班味', '后台 99 标签', '咖啡续命'],
      buddy: { name: '低电量退休体', desc: '互相吐槽发疯，是工位上最硬核的战友情' },
      rival: { name: '满电早八战神', desc: '对方在群里狂发 200 字正能量复盘，让你眼前一黑' },
      avatarEmoji: '☕',
      targetVector: { drain: 88, banwei: 94, retire: 64, heal: 34 },
      extra: {
        mentalAge: 54,
        batteryPct: 19,
        batteryLabel: '19% · 严重透支',
        rows: [
          { name: '班味浓度指标', value: '严重超标 ↑↑', status: 'flag' },
          { name: '通知应激反应', value: '高敏 (+)', status: 'flag' },
          { name: '大脑内存占用', value: '96%', status: 'flag' },
          { name: '责任心指标', value: '过载 ✓', status: 'ok' }
        ],
        prescription: '医嘱：今晚 20:00 起强制关闭工作群提醒，吃一顿不看屏幕的热乎饭。'
      }
    },
    {
      code: '75岁',
      title: '公园晒太阳大师',
      subtitle: '看淡内卷 · 血脉觉醒型养生青年',
      rarity: '仅 7.3% 测出',
      quote: '不是没精力，是把精力精准留给了好天气、热茶、散步和真正喜欢的人。',
      tags: ['血脉养生', '反卷护盾', '稳态回血'],
      buddy: { name: '满电自愈样本', desc: '一起逛公园喝茶，把周末过成神仙画卷' },
      rival: { name: '班味超标硬撑体', desc: '你想慢慢走看树叶，对方边走边接电话改 PPT' },
      avatarEmoji: '🍵',
      targetVector: { drain: 38, banwei: 32, retire: 88, heal: 84 },
      extra: {
        mentalAge: 75,
        batteryPct: 74,
        batteryLabel: '74% · 节能稳态',
        rows: [
          { name: '班味浓度指标', value: '极低 ✓', status: 'ok' },
          { name: '提前退休气质', value: '已达标 ✓', status: 'ok' },
          { name: '养生血脉觉醒', value: '强阳性 (+)', status: 'flag' },
          { name: '情绪自愈能力', value: '优秀 ✓', status: 'ok' }
        ],
        prescription: '医嘱：你的松弛感已是全网前 10% 模范生，继续保持！'
      }
    },
    {
      code: '22岁',
      title: '满电自愈模范生',
      subtitle: '内核稳健 · 自带快充头的稀缺人类',
      rarity: '仅 5.6% 测出',
      quote: '偶尔也会累，但睡一觉、晒会儿太阳或和朋友笑一场，第二天又是满血好汉！',
      tags: ['快充体质', '情绪稳健', '低内耗'],
      buddy: { name: '公园晒太阳大师', desc: '双向奔赴的高质量健康生活搭档' },
      rival: { name: '深夜断电emo体', desc: '你劝对方早点睡，对方凌晨三点还在写伤感小作文' },
      avatarEmoji: '🔋',
      targetVector: { drain: 20, banwei: 28, retire: 24, heal: 92 },
      extra: {
        mentalAge: 22,
        batteryPct: 91,
        batteryLabel: '91% · 能量满格',
        rows: [
          { name: '班味浓度指标', value: '正常范围 ✓', status: 'ok' },
          { name: '周末回血速率', value: '极快 ✓', status: 'ok' },
          { name: '夜间内耗残留', value: '极低 ✓', status: 'ok' },
          { name: '心理韧性评分', value: '95 / 100', status: 'ok' }
        ],
        prescription: '医嘱：状态极佳！多把你的松弛与阳光分一点给身边残血的搭子。'
      }
    },
    {
      code: '42岁',
      title: '半梦半醒省电体',
      subtitle: '白天体面运行 · 晚上报复性刷手机',
      rarity: '仅 8.7% 测出',
      quote: '并没有真的崩溃，只是开启了长期低功耗后台模式，能躺着就绝不站着。',
      tags: ['低功耗运行', '报复性熬夜', '间歇自愈'],
      buddy: { name: '低电量退休体', desc: '一起点外卖追老剧，彼此零压力陪伴' },
      rival: { name: '满电自愈模范生', desc: '对方周六早上 7 点约你爬山，直接触发你的防卫机制' },
      avatarEmoji: '😴',
      targetVector: { drain: 68, banwei: 54, retire: 66, heal: 56 },
      extra: {
        mentalAge: 42,
        batteryPct: 48,
        batteryLabel: '48% · 低功耗模式',
        rows: [
          { name: '班味浓度指标', value: '轻度偏高', status: 'flag' },
          { name: '报复性熬夜率', value: '偏高 ↑', status: 'flag' },
          { name: '选择性节能', value: '已开启 ✓', status: 'ok' },
          { name: '情绪恢复底盘', value: '稳定 ✓', status: 'ok' }
        ],
        prescription: '医嘱：今晚试着提前 30 分钟放下手机，给大脑留一段空白散热期。'
      }
    },
    {
      code: '31岁',
      title: '高压高效运转体',
      subtitle: '边喊累边封神 · 抗压型实干选手',
      rarity: '仅 6.4% 测出',
      quote: '压力再大也能保持清醒节奏，发完牢骚转手就把事情漂漂亮亮搞定。',
      tags: ['抗压战神', '清醒务实', '效率在线'],
      buddy: { name: '满电自愈模范生', desc: '互相带动正反馈，生活工作两不误' },
      rival: { name: '低电量退休体', desc: '你想周末约着做复盘，对方已经躺在公园草坪睡着了' },
      avatarEmoji: '⚙️',
      targetVector: { drain: 46, banwei: 78, retire: 32, heal: 78 },
      extra: {
        mentalAge: 31,
        batteryPct: 67,
        batteryLabel: '67% · 高效巡航',
        rows: [
          { name: '班味浓度指标', value: '偏高 ↑', status: 'flag' },
          { name: '高压抗摔指数', value: '优秀 ✓', status: 'ok' },
          { name: '专注力续航', value: '良好 ✓', status: 'ok' },
          { name: '下班切换能力', value: '需加强', status: 'flag' }
        ],
        prescription: '医嘱：你的引擎很强，但也要定期进站保养——周末给自己安排一场纯玩无目的的放风。'
      }
    }
  ]
};
