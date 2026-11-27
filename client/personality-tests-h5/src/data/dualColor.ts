import type { TestDefinition } from '../engine/types.ts';

export const dualColorTest: TestDefinition = {
  id: 'pantone',
  title: '人格双色主导色',
  shortTitle: '双色主导色',
  subtitle: '表层社交色 × 底层灵魂色渐变色卡',
  heroDesc: '每个人都不止一种颜色：测出别人第一眼看见的「表层社交色」，与只有深夜才显影的「底层灵魂色」。',
  category: 'aesthetic',
  badge: '小红书热',
  estimatedMinutes: 2,
  cardBg: 'linear-gradient(135deg,#FFB085 0%,#F78FA7 48%,#6C63AC 100%)',
  posterBg: '#F6B89E',
  tagColor: '#FFB5C8',
  accentColor: '#5468A8',
  scoringMode: 'nearest-profile',
  dimensions: [
    { id: 'warmth', label: '表层暖度', lowLabel: '清冷疏离', highLabel: '明媚亲和' },
    { id: 'depth', label: '灵魂深潜', lowLabel: '轻盈自在', highLabel: '深邃思辨' },
    { id: 'romance', label: '理想浪漫', lowLabel: '清醒务实', highLabel: '诗意造梦' },
    { id: 'spark', label: '能量张力', lowLabel: '静水微澜', highLabel: '热烈耀眼' }
  ],
  microHints: [
    '✦ 正在调制你的第一层光晕…',
    '✦ 已捕捉到你的社交表层色温',
    '✦ 正在向更深的情绪底色潜入…',
    '✦ 你的专属双色编号即将显影'
  ],
  questions: [
    {
      id: 'pc1',
      tag: '黄昏时刻',
      title: '下班或放学路上撞见一场绝美粉橘色晚霞，你第一件想做的事是？',
      sub: '凭第一直觉选择',
      options: [
        { key: 'A', text: '停下脚步拍一张发给某个特定的人', sub: '想把温柔瞬间分享给在意的人', weights: { warmth: 3, romance: 3 } },
        { key: 'B', text: '戴着耳机静静多走两站路，什么也不拍', sub: '把浪漫留在自己心里慢慢发酵', weights: { depth: 3, romance: 2 } },
        { key: 'C', text: '立刻发小红书或朋友圈配一句绝美文案', sub: '好心情值得被更多人看见', weights: { warmth: 2, spark: 3 } },
        { key: 'D', text: '赞叹两秒，顺便盘算晚上吃什么最实在', sub: '短暂欣赏后回归轻盈日常', weights: { warmth: 1, romance: -2 } }
      ]
    },
    {
      id: 'pc2',
      tag: '雨天氛围',
      title: '一场暴雨把你困在街角咖啡馆，窗外水雾朦胧，你会怎么度过这 1 小时？',
      sub: '感受你最舒服的独处温度',
      options: [
        { key: 'A', text: '点一杯热拿铁，看路人撑伞走过脑补他们的故事', sub: '温柔观察者视角', weights: { warmth: 2, romance: 3, depth: 2 } },
        { key: 'B', text: '翻开电子书或写一段备忘录，享受不被打扰的深水区', sub: '雨声是最好的专注白噪音', weights: { depth: 3, spark: -1 } },
        { key: 'C', text: '给朋友发定位：“快来陪我躲雨，我请喝咖啡！”', sub: '把偶发状况变成小惊喜', weights: { warmth: 3, spark: 3 } },
        { key: 'D', text: '看一眼雷达云图算好雨停时间，顺手把工作收个尾', sub: '冷静有序不慌乱', weights: { depth: 2, romance: -2 } }
      ]
    },
    {
      id: 'pc3',
      tag: '第一印象',
      title: '陌生人刚认识你时，最常对你产生哪种“美丽误会”？',
      sub: '别人眼里的你 vs 真实的你',
      options: [
        { key: 'A', text: '觉得我超好说话超温和，熟了才发现我底线硬得很', sub: '暖杏外表 × 深靛内核', weights: { warmth: 3, depth: 3 } },
        { key: 'B', text: '以为我很高冷难接近，熟了才发现我其实超搞笑', sub: '冰雾外表 × 炽热内核', weights: { warmth: -2, spark: 2, depth: 2 } },
        { key: 'C', text: '觉得我每天都无忧无虑，其实我深夜也很会多想', sub: '明亮外表 × 细腻情绪', weights: { warmth: 3, spark: 2, romance: 2 } },
        { key: 'D', text: '觉得我很理性稳重，其实我为了热爱的事超敢冲', sub: '克制外表 × 滚烫理想', weights: { depth: 2, spark: 3, romance: 2 } }
      ]
    },
    {
      id: 'pc4',
      tag: '空间美学',
      title: '如果可以随心布置一间只属于你的房间，哪种光线最让你安心？',
      sub: '你的情绪充电灯光',
      options: [
        { key: 'A', text: '2700K 暖黄落地灯 + 一盏晃动的日落投影', sub: '柔软包裹的微醺暖意', weights: { warmth: 3, romance: 3 } },
        { key: 'B', text: '大落地窗配清透自然光，干净极简的留白空间', sub: '通透清醒不被杂念干扰', weights: { depth: 2, romance: -1, warmth: 1 } },
        { key: 'C', text: '拉上窗帘只留一盏幽蓝小夜灯与桌面微光', sub: '深海般的静谧安全感', weights: { depth: 3, spark: -2 } },
        { key: 'D', text: '明亮撞色装饰画 + 唱片机，处处是鲜活生命力', sub: '高饱和能量补给站', weights: { spark: 3, warmth: 2 } }
      ]
    },
    {
      id: 'pc5',
      tag: '音乐偏好',
      title: '深夜戴上耳机循环一首歌，最容易戳中你的往往是？',
      sub: '声音里的灵魂色',
      options: [
        { key: 'A', text: '带点忧郁电影感的弦乐或独立民谣', sub: '在克制的旋律里安放情绪', weights: { depth: 3, romance: 3 } },
        { key: 'B', text: '松弛律动的 CityPop 或爵士轻音乐', sub: '微风拂面的自由呼吸感', weights: { warmth: 2, spark: 1, romance: 2 } },
        { key: 'C', text: '鼓点炸裂的摇滚、电子或燃系热单', sub: '把积压的能量瞬间点燃', weights: { spark: 3, depth: 1 } },
        { key: 'D', text: '温柔治愈的流行老歌，让人想起安心的夏天', sub: '熟悉与温暖是最好的归属', weights: { warmth: 3, romance: 1 } }
      ]
    },
    {
      id: 'pc6',
      tag: '情绪表达',
      title: '当你非常欣赏或喜欢一个人时，你的表达方式更接近？',
      sub: '爱意的温度与显色度',
      options: [
        { key: 'A', text: '嘴上克制，但会悄悄记住对方提过的每件小事', sub: '无声渗透的深水温柔', weights: { depth: 3, warmth: 2, romance: 2 } },
        { key: 'B', text: '大方夸奖、主动分享日常，藏不住眼里的小星星', sub: '明目张胆的炽热偏爱', weights: { warmth: 3, spark: 3, romance: 2 } },
        { key: 'C', text: '用靠谱的实际行动兜底，比甜言蜜语更让人踏实', sub: '稳重低饱和的长久陪伴', weights: { depth: 2, romance: -2, warmth: 2 } },
        { key: 'D', text: '保持恰到好处的呼吸感，互不捆绑但精神共振', sub: '清透独立的高级默契', weights: { depth: 3, warmth: -1 } }
      ]
    },
    {
      id: 'pc7',
      tag: '理想周末',
      title: '没有预算限制的完美周日，你更想拥有哪种画面？',
      sub: '选择最让你心跳加速的场景',
      options: [
        { key: 'A', text: '海边公路自驾，开着敞篷听风看橘子海日落', sub: '自由、明媚、毫无束缚', weights: { spark: 3, romance: 3, warmth: 2 } },
        { key: 'B', text: '躲进安静的美术馆或旧书店，一整个下午不看手机', sub: '在审美与思考中慢慢回血', weights: { depth: 3, romance: 2 } },
        { key: 'C', text: '在家里做一顿丰盛 brunch，和三两挚友聊天撸猫', sub: '烟火气里的踏实幸福', weights: { warmth: 3, depth: 1 } },
        { key: 'D', text: '去尝试一件从未玩过的新体验：冲浪、陶艺或 Livehouse', sub: '好奇心拉满的能量探索', weights: { spark: 3, romance: 1 } }
      ]
    },
    {
      id: 'pc8',
      tag: '面对低谷',
      title: '人生偶尔卡壳不顺时，你内心深处的自愈底色是什么？',
      sub: '看你如何穿越阴影',
      options: [
        { key: 'A', text: '允许自己先难过一晚，第二天擦干眼泪继续温柔生活', sub: '柔软却有韧性的疗愈力', weights: { warmth: 2, romance: 2, depth: 2 } },
        { key: 'B', text: '彻底冷静下来拆解问题，把情绪和事实分开处理', sub: '冰山般的清醒自持', weights: { depth: 3, romance: -2 } },
        { key: 'C', text: '换个发型、买束花或去跑五公里，用行动重启气场', sub: '靠行动力迅速拉升频段', weights: { spark: 3, warmth: 1 } },
        { key: 'D', text: '把低谷当成一部电影的转折章节，相信后面自有彩蛋', sub: '浪漫主义者的终极护盾', weights: { romance: 3, spark: 2 } }
      ]
    },
    {
      id: 'pc9',
      tag: '社交距离',
      title: '在热闹的聚会里，你最自在的位置通常是？',
      sub: '人群中的光谱站位',
      options: [
        { key: 'A', text: '靠近中心但不抢戏，照顾到角落里落单的朋友', sub: '温柔发光的小太阳', weights: { warmth: 3, spark: 2 } },
        { key: 'B', text: '靠窗边和一两个投缘的人深聊到忘记时间', sub: '偏爱小范围高浓度对话', weights: { depth: 3, warmth: 1 } },
        { key: 'C', text: '天然聚光灯位，带头抛梗拍照把气氛推向高潮', sub: '高饱和鲜活存在感', weights: { spark: 3, warmth: 2 } },
        { key: 'D', text: '安安静静品尝美食观察全场，像自带玻璃罩的猫', sub: '礼貌疏离的清醒看客', weights: { depth: 2, warmth: -2 } }
      ]
    },
    {
      id: 'pc10',
      tag: '气味记忆',
      title: '如果给自己的性格挑一款专属气味，你会选？',
      sub: '感官通感测试',
      options: [
        { key: 'A', text: '阳光晒过的白棉被 + 淡淡白桃乌龙', sub: '干净温柔毫无攻击性', weights: { warmth: 3, romance: 2 } },
        { key: 'B', text: '雨后松林雪松 + 一缕微苦黑茶', sub: '冷冽克制却越闻越上头', weights: { depth: 3, warmth: -1 } },
        { key: 'C', text: '气泡西柚柑橘 + 盛夏海盐微风', sub: '明快通透一秒提神', weights: { spark: 3, warmth: 2 } },
        { key: 'D', text: '老图书馆木质香 + 晚间玫瑰琥珀', sub: '神秘复古的故事感', weights: { depth: 3, romance: 3 } }
      ]
    },
    {
      id: 'pc11',
      tag: '决策罗盘',
      title: '面临一个重要选择（比如换城市或换方向），最后让你拍板的通常是？',
      sub: '头脑与心跳的拔河',
      options: [
        { key: 'A', text: '“我能不能在这件事里看到让我心动的未来画面”', sub: '理想与审美驱动', weights: { romance: 3, spark: 2 } },
        { key: 'B', text: '“冷静评估完风险和底线后，它是否符合我长期原则”', sub: '深度理性导航', weights: { depth: 3, romance: -2 } },
        { key: 'C', text: '“身边在意的人和生活氛围能不能让我感到温暖踏实”', sub: '人与温度的权重最高', weights: { warmth: 3, depth: 1 } },
        { key: 'D', text: '“先干了再说！不试试怎么知道不会撞见奇迹”', sub: '直觉勇敢的破局力', weights: { spark: 3, romance: 1 } }
      ]
    },
    {
      id: 'pc12',
      tag: '拍照审美',
      title: '翻看你的手机相册，占比例最高的是哪一类照片？',
      sub: '镜头出卖了你的内心世界',
      options: [
        { key: 'A', text: '天空、云朵、树影、街角路灯等氛围空镜', sub: '对光影与情绪极其敏感', weights: { romance: 3, depth: 2 } },
        { key: 'B', text: '和朋友的鬼脸合照、热气腾腾的美食与打卡瞬间', sub: '鲜活记录每一份快乐', weights: { warmth: 3, spark: 3 } },
        { key: 'C', text: '收藏的干货截图、灵感拼图与极简构图建筑', sub: '秩序感与思考痕迹', weights: { depth: 3, romance: -1 } },
        { key: 'D', text: '相册很精简，只留少数真正珍贵的纪念照', sub: '克制而念旧', weights: { depth: 2, warmth: 1 } }
      ]
    },
    {
      id: 'pc13',
      tag: '边界风格',
      title: '遇到不合拍的人一直试图对你指手画脚，你通常会？',
      sub: '柔和还是锋利',
      options: [
        { key: 'A', text: '礼貌微笑听完，转身照旧按自己的节奏走', sub: '柔中带刚，谁也别想操控我', weights: { warmth: 2, depth: 3 } },
        { key: 'B', text: '眼神一冷直接划清界限，用沉默降温', sub: '自带高冷结界', weights: { warmth: -2, depth: 3 } },
        { key: 'C', text: '带点幽默感巧妙怼回去，既不失风度又守住场子', sub: '机智爽快的能量反击', weights: { spark: 3, warmth: 1 } },
        { key: 'D', text: '嘴上不争，但事后心里会默默消化一会儿', sub: '外柔内敏的共情者', weights: { romance: 2, warmth: 2, spark: -1 } }
      ]
    },
    {
      id: 'pc14',
      tag: '电影口味',
      title: '哪一类电影或剧集最容易让你看完后久久走不出来？',
      sub: '灵魂共振频段',
      options: [
        { key: 'A', text: '细腻克制的文艺片，留白里藏着千言万语', sub: '偏爱余韵悠长的深色故事', weights: { depth: 3, romance: 3 } },
        { key: 'B', text: '治愈温暖的群像生活剧，像冬天里的一杯热可可', sub: '渴望真诚连接与烟火气', weights: { warmth: 3, romance: 1 } },
        { key: 'C', text: '脑洞大开的科幻/悬疑烧脑片，反转越狠越过瘾', sub: '理性智性与探索欲拉满', weights: { depth: 3, spark: 2, romance: -1 } },
        { key: 'D', text: '热血浪漫的公路冒险片，看完立刻想出发', sub: '热烈自由的生命张力', weights: { spark: 3, romance: 2 } }
      ]
    },
    {
      id: 'pc15',
      tag: '深夜对话',
      title: '如果凌晨两点有位知己发来“睡了吗，聊聊”，你最想聊的话题是？',
      sub: '卸下白天角色之后',
      options: [
        { key: 'A', text: '关于人生意义、宇宙尽头或某个藏了很久的心事', sub: '深夜专属的灵魂深潜', weights: { depth: 3, romance: 3 } },
        { key: 'B', text: '吐槽白天的奇葩事顺便互相安利好吃好玩的', sub: '轻松解压的快乐夜谈', weights: { warmth: 3, spark: 2 } },
        { key: 'C', text: '聊彼此未来的野心计划与搞钱灵感，越聊越燃', sub: '思想碰撞的能量场', weights: { spark: 3, depth: 2 } },
        { key: 'D', text: '不用聊多深，只要互相发几首歌安静陪着就很好', sub: '无言默契的高级温柔', weights: { warmth: 2, depth: 2, spark: -1 } }
      ]
    },
    {
      id: 'pc16',
      tag: '显色终章',
      title: '你最希望别人提起你时，脑海里浮现出怎样的画面？',
      sub: '最后一题 · 锁定你的双色色卡',
      options: [
        { key: 'A', text: '“外表温柔好相处，骨子里却极其清醒独立”', sub: '暖雾与深海的双重魅力', weights: { warmth: 2, depth: 3, romance: 2 } },
        { key: 'B', text: '“只要有 TA 在，整片空气都变得明亮滚烫”', sub: '无可替代的鲜活光芒', weights: { warmth: 3, spark: 3 } },
        { key: 'C', text: '“像一阵干净清冽的山风，自由、高级、从不将就”', sub: '低饱和冷调美学', weights: { depth: 3, warmth: -2 } },
        { key: 'D', text: '“永远保有一点天真和浪漫，把平凡日子过成诗”', sub: '柔软而坚定的造梦者', weights: { romance: 3, warmth: 2 } }
      ]
    }
  ],
  profiles: [
    {
      code: 'NO.047',
      title: '暖雾杏 × 暮海靛',
      subtitle: '温柔克制的深水区',
      rarity: '仅 6.8% 测出',
      quote: '别人只看见你恰到好处的礼貌暖意，只有深夜才露出深水区的清醒与克制。',
      tags: ['外暖内冷', '情绪深潜', '高级分寸'],
      buddy: { name: '薄荷烟灰 × 松林绿', desc: '低饱和高级感同频，聊天像一起看慢电影' },
      rival: { name: '电光柠 × 烈焰橙', desc: '高分贝能量场太耀眼，相处半小时电量掉 60%' },
      avatarEmoji: '🌅',
      targetVector: { warmth: 66, depth: 86, romance: 68, spark: 34 },
      extra: {
        colorNo: 'NO.047',
        swatchGradient: 'linear-gradient(145deg,#FFB085 0%,#E89AA4 45%,#5A6B9C 80%,#2B3A67 100%)',
        surfaceColor: { name: '暖雾杏', hex: '#FFB085', pct: 62 },
        soulColor: { name: '暮海靛', hex: '#2B3A67', pct: 38 }
      }
    },
    {
      code: 'NO.019',
      title: '日落橘 × 桃粉霓虹',
      subtitle: '热烈滚烫的多巴胺光源',
      rarity: '仅 7.1% 测出',
      quote: '你天生带着把阴天照亮的能力，爱恨鲜明，喜欢的事绝不藏着掖着。',
      tags: ['明媚直球', '高饱和快乐', '浪漫行动派'],
      buddy: { name: '奶油黄 × 蜂蜜茶', desc: '双向奔赴的温暖组合，每天都像在过盛夏' },
      rival: { name: '冷雾灰 × 曜石黑', desc: '你热情抛出十个梗，对方冷静回了句“还好”' },
      avatarEmoji: '🍑',
      targetVector: { warmth: 88, depth: 36, romance: 76, spark: 92 },
      extra: {
        colorNo: 'NO.019',
        swatchGradient: 'linear-gradient(145deg,#FF914D 0%,#FF6B8B 58%,#D9468F 100%)',
        surfaceColor: { name: '日落橘', hex: '#FF914D', pct: 58 },
        soulColor: { name: '桃粉霓虹', hex: '#E04D8E', pct: 42 }
      }
    },
    {
      code: 'NO.082',
      title: '冰雾蓝 × 曜石夜',
      subtitle: '清冽自持的理性观察者',
      rarity: '仅 5.2% 测出',
      quote: '看似疏离难接近，实则只是把滚烫和信任留给了极少数通过筛选的同频者。',
      tags: ['清醒独立', '高冷反差', '逻辑之美'],
      buddy: { name: '薰衣草雾 × 午夜紫', desc: '精神世界旗鼓相当，懂彼此的沉默与边界' },
      rival: { name: '日落橘 × 桃粉霓虹', desc: '节奏太快太满，让你迫切需要找个角落静一静' },
      avatarEmoji: '❄️',
      targetVector: { warmth: 24, depth: 90, romance: 32, spark: 36 },
      extra: {
        colorNo: 'NO.082',
        swatchGradient: 'linear-gradient(145deg,#9AD4F5 0%,#4F7AA6 52%,#1D2436 100%)',
        surfaceColor: { name: '冰雾蓝', hex: '#9AD4F5', pct: 54 },
        soulColor: { name: '曜石夜', hex: '#1D2436', pct: 46 }
      }
    },
    {
      code: 'NO.063',
      title: '薰衣草雾 × 午夜紫',
      subtitle: '月光下的诗意造梦者',
      rarity: '仅 4.9% 测出',
      quote: '你在现实里体面奔波，却在精神宇宙里养了一座永不打烊的浪漫花园。',
      tags: ['审美敏锐', '灵魂深潜', '电影感'],
      buddy: { name: '冰雾蓝 × 曜石夜', desc: '一个懂浪漫一个守底线，精神契合度爆表' },
      rival: { name: '焦糖麦 × 咖啡褐', desc: '聊诗和远方时，对方突然开始算房贷利率' },
      avatarEmoji: '🔮',
      targetVector: { warmth: 48, depth: 88, romance: 94, spark: 48 },
      extra: {
        colorNo: 'NO.063',
        swatchGradient: 'linear-gradient(145deg,#D5C2FF 0%,#8E74D8 55%,#352766 100%)',
        surfaceColor: { name: '薰衣草雾', hex: '#D5C2FF', pct: 50 },
        soulColor: { name: '午夜紫', hex: '#352766', pct: 50 }
      }
    },
    {
      code: 'NO.031',
      title: '薄荷烟灰 × 松林绿',
      subtitle: '自由通透的森林呼吸感',
      rarity: '仅 6.3% 测出',
      quote: '不争不抢也不迎合，像雨后森林里的清风，靠近你的人会不自觉放下焦虑。',
      tags: ['松弛治愈', '天然边界', '慢生活'],
      buddy: { name: '暖雾杏 × 暮海靛', desc: '彼此舒服不消耗，是能安静并肩走很久的人' },
      rival: { name: '电光青 × 极光紫', desc: '对方天天想着颠覆世界，你只想晒太阳喝茶' },
      avatarEmoji: '🌲',
      targetVector: { warmth: 56, depth: 62, romance: 52, spark: 28 },
      extra: {
        colorNo: 'NO.031',
        swatchGradient: 'linear-gradient(145deg,#A9E5C8 0%,#5CA684 55%,#244F3E 100%)',
        surfaceColor: { name: '薄荷烟灰', hex: '#A9E5C8', pct: 64 },
        soulColor: { name: '松林绿', hex: '#244F3E', pct: 36 }
      }
    },
    {
      code: 'NO.095',
      title: '电光青 × 极光紫',
      subtitle: '锋利滚烫的破局灵感家',
      rarity: '仅 4.4% 测出',
      quote: '厌倦一切按部就班，你的大脑永远比现实快半步，冷静外表下藏着一团蓝焰。',
      tags: ['反骨灵感', '高能破局', '酷感十足'],
      buddy: { name: '日落橘 × 桃粉霓虹', desc: '两个高能星体相撞，聊着聊着就能干出件大事' },
      rival: { name: '奶油黄 × 蜂蜜茶', desc: '你爱冒险变化，对方更偏爱稳妥安逸的日常' },
      avatarEmoji: '⚡',
      targetVector: { warmth: 38, depth: 74, romance: 58, spark: 92 },
      extra: {
        colorNo: 'NO.095',
        swatchGradient: 'linear-gradient(145deg,#5CE1E6 0%,#5C6CFF 56%,#6B2FD9 100%)',
        surfaceColor: { name: '电光青', hex: '#5CE1E6', pct: 55 },
        soulColor: { name: '极光紫', hex: '#6B2FD9', pct: 45 }
      }
    },
    {
      code: 'NO.012',
      title: '奶油黄 × 蜂蜜茶',
      subtitle: '自带烟火气的温柔治愈师',
      rarity: '仅 7.9% 测出',
      quote: '你擅长把细碎日常经营得热气腾腾，和你待在一起总让人觉得“生活还不错”。',
      tags: ['暖心烟火', '可靠踏实', '温柔兜底'],
      buddy: { name: '日落橘 × 桃粉霓虹', desc: '一个管快乐一个管踏实，顶级神仙日常搭子' },
      rival: { name: '冰雾蓝 × 曜石夜', desc: '对方太爱讲大道理，偶尔忘了先抱抱情绪' },
      avatarEmoji: '🍯',
      targetVector: { warmth: 90, depth: 44, romance: 64, spark: 48 },
      extra: {
        colorNo: 'NO.012',
        swatchGradient: 'linear-gradient(145deg,#FFE699 0%,#F2B866 56%,#9C6635 100%)',
        surfaceColor: { name: '奶油黄', hex: '#FFE699', pct: 66 },
        soulColor: { name: '蜂蜜茶', hex: '#9C6635', pct: 34 }
      }
    },
    {
      code: 'NO.058',
      title: '焦糖麦 × 咖啡褐',
      subtitle: '清醒笃定的长期主义者',
      rarity: '仅 5.8% 测出',
      quote: '不被一阵风吹乱阵脚，你在浮躁世界里保有难得的质感、耐心与掌控力。',
      tags: ['稳重质感', '务实清醒', '越处越香'],
      buddy: { name: '薄荷烟灰 × 松林绿', desc: '不焦躁不内耗，细水长流的顶级默契' },
      rival: { name: '薰衣草雾 × 午夜紫', desc: '一个在算落地路径，一个在聊宇宙星辰' },
      avatarEmoji: '☕',
      targetVector: { warmth: 58, depth: 72, romance: 24, spark: 56 },
      extra: {
        colorNo: 'NO.058',
        swatchGradient: 'linear-gradient(145deg,#E3B786 0%,#A67247 58%,#4A301F 100%)',
        surfaceColor: { name: '焦糖麦', hex: '#E3B786', pct: 60 },
        soulColor: { name: '咖啡褐', hex: '#4A301F', pct: 40 }
      }
    }
  ]
};
