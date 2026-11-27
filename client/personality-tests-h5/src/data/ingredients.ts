import type { TestDefinition } from '../engine/types.ts';

export const ingredientsTest: TestDefinition = {
  id: 'ingredients',
  title: '性格成分配料表',
  shortTitle: '成分配料表',
  subtitle: '仿超市营养成分表 · 测你的核心配方',
  heroDesc: '把你的灵魂拆成一张超市零食配料表：嘴硬心软占几成？纯爱原浆还剩多少？储存条件是什么？',
  category: 'viral',
  badge: '二期上新',
  estimatedMinutes: 2,
  cardBg: '#FFE066',
  posterBg: '#FFE066',
  tagColor: '#FFE066',
  accentColor: '#FF914D',
  scoringMode: 'nearest-profile',
  dimensions: [
    { id: 'sweet', label: '纯爱甜度' },
    { id: 'tsun', label: '嘴硬酸爽', hotWhenHigh: true },
    { id: 'night', label: '深夜浓度', hotWhenHigh: true },
    { id: 'grind', label: '自律含金' }
  ],
  microHints: [
    '✦ 正在称量你的灵魂净含量…',
    '✦ 检测到高纯度嘴硬与间歇性自律成分',
    '✦ 正在计算保质期与储存温度…',
    '✦ 你的专属人类配料表即将打印完成'
  ],
  questions: [
    {
      id: 'ig1', tag: '核心配料', title: '深夜 11 点朋友发来“出来吃烧烤”，你此刻的真实成分是？', sub: '选最像你的第一反应',
      options: [
        { key: 'A', text: '嘴上回“太油腻了不去”，五分钟后穿好鞋问定位', sub: '嘴硬酸爽成分拉满', weights: { tsun: 3, sweet: 1 } },
        { key: 'B', text: '立刻出发！只要是喜欢的人叫我，几点都有空', sub: '高甜纯爱原浆', weights: { sweet: 3 } },
        { key: 'C', text: '礼貌婉拒，明天早起健身/搬砖不能破戒', sub: '自律含金量爆表', weights: { grind: 3 } },
        { key: 'D', text: '不想出门，但在床上发呆看天花板到凌晨两点', sub: '深夜哲学浓缩液', weights: { night: 3 } }
      ]
    },
    {
      id: 'ig2', tag: '甜度测定', title: '看到路边卖花的小推车或夕阳下的小狗，你会？', sub: '测试纯爱浪漫浓度',
      options: [
        { key: 'A', text: '买一束送给自己或在乎的人，平凡日子要有甜味', sub: '满格浪漫甜度', weights: { sweet: 3 } },
        { key: 'B', text: '偷偷拍一张留着，嘴上嫌弃“没必要花这钱”', sub: '口嫌体正直配方', weights: { tsun: 3, sweet: 1 } },
        { key: 'C', text: '站着发呆想一串人生感悟，顺手记进备忘录', sub: '文艺深夜底味', weights: { night: 3 } },
        { key: 'D', text: '欣赏两秒继续赶路，心里装着今天要完成的清单', sub: '清醒干练基调', weights: { grind: 3 } }
      ]
    },
    {
      id: 'ig3', tag: '保质期测试', title: '你买回来的运动装备、电子书或网课，通常的命运是？', sub: '间歇性自律检测',
      options: [
        { key: 'A', text: '真的每周打卡坚持下来了，成就感拉满', sub: '稀缺纯自律成分', weights: { grind: 3 } },
        { key: 'B', text: '热血三天、吃灰三月，偶尔半夜突然燃起来续一次', sub: '经典间歇性自律因子', weights: { night: 2, tsun: 2 } },
        { key: 'C', text: '买完那一刻就已经获得 90% 的精神满足感了', sub: '快乐消费不内耗', weights: { sweet: 2, tsun: 1 } },
        { key: 'D', text: '拉着朋友互相打卡，有人陪就能坚持很久', sub: '陪伴型甜度驱动', weights: { sweet: 3, grind: 1 } }
      ]
    },
    {
      id: 'ig4', tag: '嘴硬浓度', title: '其实特别期待对方给你带杯奶茶，对方问“要喝吗”时你会？', sub: '诚实面对你的傲娇值',
      options: [
        { key: 'A', text: '“不用麻烦啦随便吧”，心里疯狂默念三分糖去冰', sub: '一级嘴硬提取物', weights: { tsun: 3, sweet: 1 } },
        { key: 'B', text: '“要要要！我要霸气芝士葡萄，谢谢老板！”', sub: '直球高甜毫不扭捏', weights: { sweet: 3, tsun: -2 } },
        { key: 'C', text: '“喝美式吧，控糖提神下午还要干活”', sub: '硬核自律清醒风', weights: { grind: 3 } },
        { key: 'D', text: '“晚上喝睡不着，不过来半杯我也行”', sub: '纠结细品派', weights: { night: 2, tsun: 1 } }
      ]
    },
    {
      id: 'ig5', tag: '夜间活性', title: '一天当中，你思维最清晰、灵感或emo最汹涌的时段是？', sub: '生物钟配方检测',
      options: [
        { key: 'A', text: '晚上 11 点到凌晨 2 点，世界安静了我才真正开机', sub: '高浓度夜行性灵魂', weights: { night: 3 } },
        { key: 'B', text: '上午 9 点到 11 点，效率拉满像精密仪器', sub: '晨型自律成分', weights: { grind: 3, night: -2 } },
        { key: 'C', text: '下午三四点晒着太阳喝着奶茶，心情最好', sub: '温柔下午茶甜味', weights: { sweet: 3 } },
        { key: 'D', text: '看当天谁惹我了，情绪上来随时爆发高光', sub: '带刺碳酸口感', weights: { tsun: 3 } }
      ]
    },
    {
      id: 'ig6', tag: '防伪标识', title: '如果朋友偷偷夸你“你其实特别温柔”，你的第一反应？', sub: '表面反应 vs 内心真实',
      options: [
        { key: 'A', text: '立刻反驳“你眼瞎了吧我凶得很”，耳朵偷偷红了', sub: '特浓嘴硬心软原液', weights: { tsun: 3, sweet: 2 } },
        { key: 'B', text: '开心接住并顺手回夸对方一通', sub: '双倍甜度反馈', weights: { sweet: 3 } },
        { key: 'C', text: '平静笑笑，觉得靠谱体面本就是成年人修养', sub: '沉稳自律底色', weights: { grind: 2, night: 1 } },
        { key: 'D', text: '回家路上反复回味这句话，觉得被真正看见了', sub: '深水共鸣型', weights: { night: 3, sweet: 1 } }
      ]
    },
    {
      id: 'ig7', tag: '抗氧化力', title: '遇到工作或生活里突然甩过来的无理要求，你会怎么“防氧化”？', sub: '边界感配比',
      options: [
        { key: 'A', text: '礼貌但极其专业地拿规则和排期挡回去', sub: '理性护盾成分', weights: { grind: 3 } },
        { key: 'B', text: '表面冷静怼两句，私下跟熟人疯狂吐槽半小时', sub: '气泡微辣口感', weights: { tsun: 3 } },
        { key: 'C', text: '先默默消化，晚上躲被窝里写段小作文释放', sub: '深夜消化型', weights: { night: 3 } },
        { key: 'D', text: '能顺手帮就帮，帮不了笑嘻嘻撒娇推掉', sub: '柔滑高甜化劲', weights: { sweet: 3 } }
      ]
    },
    {
      id: 'ig8', tag: '口味偏好', title: '如果用一杯饮品比喻你和人熟络后的真实味道，更像？', sub: '口感自评',
      options: [
        { key: 'A', text: '微苦回甘的桂花冰美式：初尝高冷，越品越上头', sub: '清醒回甘型', weights: { grind: 2, tsun: 2 } },
        { key: 'B', text: '白桃乌龙气泡水：甜甜的但偶尔冒点小尖刺', sub: '甜中带刺气泡感', weights: { sweet: 2, tsun: 2 } },
        { key: 'C', text: '海盐焦糖热拿铁：暖呼呼让人一秒放松', sub: '治愈高甜担当', weights: { sweet: 3 } },
        { key: 'D', text: '深夜威士忌加冰球：故事感满满、越夜越迷人', sub: '深邃夜色基酒', weights: { night: 3 } }
      ]
    },
    {
      id: 'ig9', tag: '情绪热量', title: '刷到一个特别好笑或特别戳心的帖子，你通常会？', sub: '情绪外泄率',
      options: [
        { key: 'A', text: '第一时间转发给 3 个不同搭子共享快乐', sub: '高糖分享欲', weights: { sweet: 3 } },
        { key: 'B', text: '默默点个赞收藏进文件夹，独自细品', sub: '安静深水收藏家', weights: { night: 3 } },
        { key: 'C', text: '在评论区精准补一刀神吐槽，收割一堆点赞', sub: '犀利气泡感', weights: { tsun: 3 } },
        { key: 'D', text: '划过记下有用的知识点，继续干正事', sub: '克制干货派', weights: { grind: 3 } }
      ]
    },
    {
      id: 'ig10', tag: '添加剂检测', title: '以下哪一种“当代添加剂”，在你体内含量最高？', sub: '诚实勾选',
      options: [
        { key: 'A', text: '“嘴上说算了，心里还在替对方找借口”', sub: '心软纯爱超标', weights: { tsun: 2, sweet: 3 } },
        { key: 'B', text: '“白天装没事人，半夜突然思考宇宙与人生”', sub: '深夜哲学家超标', weights: { night: 3 } },
        { key: 'C', text: '“不把事情做到 90 分以上浑身不舒服”', sub: '完美主义自律超标', weights: { grind: 3 } },
        { key: 'D', text: '“明明很在意，偏要装出一副无所谓的拽样”', sub: '傲娇碳酸超标', weights: { tsun: 3 } }
      ]
    },
    {
      id: 'ig11', tag: '储存温度', title: '什么环境下待着，你的状态最保鲜、完全不会变质？', sub: '最佳储存条件',
      options: [
        { key: 'A', text: '安静独处、灯光昏暗、手机免打扰的深夜房间', sub: '阴凉避光保存', weights: { night: 3 } },
        { key: 'B', text: '和懂我的三两好友吃着美食大笑的暖场空间', sub: '常温暖心保存', weights: { sweet: 3 } },
        { key: 'C', text: '节奏清晰、目标明确、没有废话拉扯的高效场域', sub: '干爽秩序保存', weights: { grind: 3 } },
        { key: 'D', text: '没人管我、允许我随时傲娇发疯的小领地', sub: '独立密封保存', weights: { tsun: 3 } }
      ]
    },
    {
      id: 'ig12', tag: '能量来源', title: '当你彻底没电、快要蔫掉时，投喂哪样东西最能让你一秒复活？', sub: '特效充电剂',
      options: [
        { key: 'A', text: '在意的人一句具体的偏爱或暖心拥抱', sub: '纯爱直充', weights: { sweet: 3 } },
        { key: 'B', text: '干完一件硬核大事后看到进度条 100%', sub: '成就感直充', weights: { grind: 3 } },
        { key: 'C', text: '一首直击灵魂的歌 + 凌晨吹一段无人晚风', sub: '氛围感直充', weights: { night: 3 } },
        { key: 'D', text: '买一件种草很久的酷玩意 + 吃顿狠的', sub: '爽感直充', weights: { tsun: 2, sweet: 1 } }
      ]
    },
    {
      id: 'ig13', tag: '社交酸度', title: '面对不太熟但硬要套近乎的人，你的礼貌度会？', sub: '表面酸碱度',
      options: [
        { key: 'A', text: '笑容标准但回复滴水不漏，三句话就能优雅结束', sub: '清爽微酸气泡', weights: { tsun: 2, grind: 2 } },
        { key: 'B', text: '还是会友善接话，怕冷场让对方尴尬', sub: '高甜温和体质', weights: { sweet: 3 } },
        { key: 'C', text: '眼神放空灵魂出窍，回家吐槽对方半小时', sub: '暗暗冒泡', weights: { night: 2, tsun: 2 } },
        { key: 'D', text: '只保留必要礼节，绝不浪费多余时间', sub: '高效清醒风', weights: { grind: 3 } }
      ]
    },
    {
      id: 'ig14', tag: '纯爱残留', title: '哪怕见识过很多现实，你心里对“纯粹真心”还信几分？', sub: '底味校准',
      options: [
        { key: 'A', text: '依然信 90%！只是现在学会了只给值得的人', sub: '清醒的纯爱战神', weights: { sweet: 3, grind: 1 } },
        { key: 'B', text: '嘴上说“爱情都是荷尔蒙”，真动心比谁都认真', sub: '傲娇纯爱特调', weights: { tsun: 3, sweet: 2 } },
        { key: 'C', text: '把浪漫藏在心里，用理性保护柔软', sub: '深海微甜配方', weights: { night: 3, grind: 1 } },
        { key: 'D', text: '先把自己过稳过强，真心当作锦上添花', sub: '冰美式独立配方', weights: { grind: 3, sweet: -1 } }
      ]
    },
    {
      id: 'ig15', tag: '使用说明', title: '如果你是一袋摆在货架上的限定零食，最适合贴哪张促销标签？', sub: '自我定位',
      options: [
        { key: 'A', text: '“外脆里甜，越嚼越香（拆封后请珍惜）”', sub: '嘴硬心软款', weights: { tsun: 3, sweet: 2 } },
        { key: 'B', text: '“深夜限定特调，懂的人自然懂”', sub: '夜色哲学款', weights: { night: 3 } },
        { key: 'C', text: '“零添加糊弄，硬核提神靠谱担当”', sub: '清醒自律款', weights: { grind: 3 } },
        { key: 'D', text: '“100% 真诚鲜榨，治愈所有不开心”', sub: '满格元气甜款', weights: { sweet: 3 } }
      ]
    },
    {
      id: 'ig16', tag: '封口压印', title: '最后一步：选一句印在你配料表底部的隐藏彩蛋语！', sub: '选完立刻打印你的营养成分表',
      options: [
        { key: 'A', text: '“本品易心软，请勿用真诚持续加热”', sub: '嘴硬纯爱限定', weights: { tsun: 3, sweet: 2 } },
        { key: 'B', text: '“本品夜间风味最佳，白天处于低功耗状态”', sub: '深夜浓萃限定', weights: { night: 3 } },
        { key: 'C', text: '“本品含高浓度靠谱因子，遇画大饼自动失效”', sub: '冰美式清醒限定', weights: { grind: 3 } },
        { key: 'D', text: '“本品甜度超标，建议与双向奔赴者共享”', sub: '全糖治愈限定', weights: { sweet: 3 } }
      ]
    }
  ],
  profiles: [
    {
      code: '520g',
      title: '嘴硬纯爱特调',
      subtitle: '外冷微刺 · 内里 100% 心软原浆',
      rarity: '仅 6.4% 测出',
      quote: '嘴上嫌弃得比谁都快，真到你淋雨那天，伞永远先歪向你这边。',
      tags: ['嘴硬心软', '外脆里甜', '绝版纯爱'],
      buddy: { name: '冰美式清醒特调', desc: '一眼看穿你的傲娇，既宠你又不跟你拧巴' },
      rival: { name: '高压浓缩鸡血饮', desc: '对方能量太冲不懂留白，同框十分钟就想稀释退场' },
      avatarEmoji: '🥫',
      targetVector: { sweet: 78, tsun: 90, night: 56, grind: 42 },
      extra: {
        netWeight: '520g',
        shelfLife: '遇对的人永久',
        formula: [
          { name: '嘴硬心软提取物', pct: 42, color: '#FF914D' },
          { name: '绝版纯爱原浆', pct: 28, color: '#FF4D5E' },
          { name: '深夜胡思乱想液', pct: 18, color: '#CBB8FF' },
          { name: '间歇自律因子', pct: 12, color: '#8DD3FF' }
        ],
        storageTip: '请置于真诚偏爱环境中常温保存，严禁画大饼与冷暴力暴晒。'
      }
    },
    {
      code: '02:00',
      title: '深夜哲学浓萃',
      subtitle: '白天低功耗 · 凌晨两点风味封神',
      rarity: '仅 5.8% 测出',
      quote: '白天是人群里安静体面的气泡水，凌晨两点自动发酵成满肚故事的威士忌。',
      tags: ['夜行性灵魂', '文艺敏感', '细节收藏家'],
      buddy: { name: '白桃全糖气泡', desc: '对方的明亮甜意，刚好能把你从深夜思绪里拉出来' },
      rival: { name: '晨跑自律燕麦杯', desc: '对方早上六点发打卡图时，你刚准备合眼睡觉' },
      avatarEmoji: '🥃',
      targetVector: { sweet: 48, tsun: 56, night: 94, grind: 44 },
      extra: {
        netWeight: '480ml',
        shelfLife: '越陈越上头',
        formula: [
          { name: '深夜哲学浓缩液', pct: 44, color: '#CBB8FF' },
          { name: '情绪留白蒸馏水', pct: 26, color: '#8DD3FF' },
          { name: '隐性浪漫微粒', pct: 18, color: '#FFB5C8' },
          { name: '嘴硬保护壳', pct: 12, color: '#FF914D' }
        ],
        storageTip: '阴凉避光独处保存，睡前严禁开启三年前社死回忆录。'
      }
    },
    {
      code: '0糖冰咖',
      title: '冰美式清醒特调',
      subtitle: '零添加糊弄 · 清醒自律高质感',
      rarity: '仅 5.2% 测出',
      quote: '不吃画大饼也不搞虚头巴脑，初尝觉得有点冷冽，久处才发现靠谱到无敌。',
      tags: ['人间清醒', '硬核靠谱', '微苦回甘'],
      buddy: { name: '嘴硬纯爱特调', desc: '你懂 TA 的傲娇，TA 欣赏你的清醒笃定' },
      rival: { name: '摸鱼摆烂布丁', desc: '你拉好三张表准备冲刺，对方问能不能明天再说' },
      avatarEmoji: '🧊',
      targetVector: { sweet: 36, tsun: 52, night: 44, grind: 92 },
      extra: {
        netWeight: '500ml',
        shelfLife: '长期主义保鲜',
        formula: [
          { name: '硬核自律原液', pct: 45, color: '#8DD3FF' },
          { name: '清醒边界因子', pct: 27, color: '#6BE5A8' },
          { name: '理性护盾萃取', pct: 18, color: '#CBB8FF' },
          { name: '隐性回甘甜味', pct: 10, color: '#FFE066' }
        ],
        storageTip: '干爽有序环境保存，定期投喂半日放空防止过度浓缩。'
      }
    },
    {
      code: '100%甜',
      title: '白桃全糖气泡',
      subtitle: '真诚鲜榨 · 自带夏日治愈滤镜',
      rarity: '仅 7.6% 测出',
      quote: '不是没见过世面，而是见识过复杂之后依然选择用真诚和甜度温暖世界。',
      tags: ['直球小太阳', '双向奔赴', '零套路'],
      buddy: { name: '深夜哲学浓萃', desc: '你照亮对方的深夜，对方珍藏你的每一份天真' },
      rival: { name: '极寒傲娇苏打', desc: '你满腔热情贴贴，对方高冷端着半天不接球' },
      avatarEmoji: '🍑',
      targetVector: { sweet: 94, tsun: 22, night: 38, grind: 52 },
      extra: {
        netWeight: '550ml',
        shelfLife: '甜度永不过期',
        formula: [
          { name: '100% 真诚鲜榨', pct: 46, color: '#FFB5C8' },
          { name: '快乐多巴胺气泡', pct: 28, color: '#FFE066' },
          { name: '暖心共情果肉', pct: 16, color: '#FF914D' },
          { name: '轻度自律糖霜', pct: 10, color: '#6BE5A8' }
        ],
        storageTip: '请远离冷暴力与敷衍人群，多与高回应搭子共同饮用。'
      }
    },
    {
      code: '微辣气泡',
      title: '青柠带刺苏打',
      subtitle: '犀利清醒 · 专治各种阴阳怪气',
      rarity: '仅 6.1% 测出',
      quote: '自带一针见血的幽默感与高边界护盾，惹到我之前最好先掂量下心理承受力。',
      tags: ['反矫情达人', '带刺幽默', '护短第一'],
      buddy: { name: '冰美式清醒特调', desc: '双清醒合体，吐槽全场毫无压力' },
      rival: { name: '玻璃心草莓奶', desc: '你刚开个无伤大雅的玩笑，对方已经开始写小作文' },
      avatarEmoji: '🍋',
      targetVector: { sweet: 34, tsun: 92, night: 48, grind: 64 },
      extra: {
        netWeight: '330ml',
        shelfLife: '开盖即爽',
        formula: [
          { name: '犀利吐槽气泡', pct: 43, color: '#E4F857' },
          { name: '高边界青柠酸', pct: 27, color: '#6BE5A8' },
          { name: '护短铁血底味', pct: 18, color: '#FF4D5E' },
          { name: '隐藏软心糖粒', pct: 12, color: '#FFB5C8' }
        ],
        storageTip: '切忌摇晃后硬刚，顺毛投喂冰饮即可释放隐藏甜味。'
      }
    },
    {
      code: '全能特饮',
      title: '海盐焦糖拿铁',
      subtitle: '甜咸平衡 · 既能扛事又懂浪漫',
      rarity: '仅 6.9% 测出',
      quote: '工作时能稳稳兜底，生活里又藏着恰到好处的温柔与幽默，堪称人间百搭神仙款。',
      tags: ['高配百搭', '稳重温柔', '甜咸适中'],
      buddy: { name: '任何特调皆可搭', desc: '极强情绪包容度，和谁组队都能调出好风味' },
      rival: { name: '失控极端浓缩液', desc: '过度情绪化内耗会悄悄抽走你的平衡感' },
      avatarEmoji: '🍮',
      targetVector: { sweet: 74, tsun: 42, night: 52, grind: 78 },
      extra: {
        netWeight: '500ml',
        shelfLife: '四季常青款',
        formula: [
          { name: '靠谱自律浓缩', pct: 36, color: '#8DD3FF' },
          { name: '温柔海盐奶盖', pct: 32, color: '#FFD89B' },
          { name: '细腻共情焦糖', pct: 20, color: '#FF914D' },
          { name: '克制夜色余韵', pct: 12, color: '#CBB8FF' }
        ],
        storageTip: '常温常伴最佳，记得也偶尔允许自己什么都不扛。'
      }
    }
  ]
};
