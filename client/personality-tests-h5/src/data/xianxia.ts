import type { TestDefinition } from '../engine/types.ts';

export const xianxiaTest: TestDefinition = {
  id: 'xianxia',
  title: '中式修仙灵根录',
  shortTitle: '修仙灵根',
  subtitle: '测宗门定位、本命法宝与渡劫心魔',
  heroDesc: '一朝踏入仙途：你是洞府喝茶的「摆烂炼丹长老」、寅时挥剑的「卷王孤高剑修」，还是表面正道的「魔教军师」？',
  category: 'aesthetic',
  badge: '国风爆款',
  estimatedMinutes: 2,
  cardBg: '#F5DDB3',
  posterBg: '#F5DDB3',
  tagColor: '#F5DDB3',
  accentColor: '#B5241C',
  scoringMode: 'nearest-profile',
  dimensions: [
    { id: 'sword', label: '剑道卷度' },
    { id: 'dao', label: '佛系道心' },
    { id: 'scheme', label: '智谋城府', hotWhenHigh: true },
    { id: 'gossip', label: '红尘烟火' }
  ],
  microHints: [
    '✦ 正在探测你的先天灵根纯度…',
    '✦ 仙门测灵碑亮起奇异光晕',
    '✦ 正在推演你的本命法宝与渡劫心魔…',
    '✦ 你的专属修仙命格符箓即将落印'
  ],
  questions: [
    {
      id: 'xx1', tag: '入宗大典', title: '仙门收徒大典上，四大长老同时向你抛出橄榄枝，你选哪一峰？', sub: '决定你的第一重道心',
      options: [
        { key: 'A', text: '剑峰：一剑破万法，酷就完事了', sub: '慕强极简，直指大道', weights: { sword: 3 } },
        { key: 'B', text: '丹峰：不用出外勤打架，在暖和洞府炼丹喝茶', sub: '顶级洞府养生派', weights: { dao: 3 } },
        { key: 'C', text: '阵法/藏经峰：掌管全宗情报与护山大阵，暗中定乾坤', sub: '幕后操盘智囊', weights: { scheme: 3 } },
        { key: 'D', text: '外门百事峰：天天逛山下集市吃美食顺便听八卦', sub: '人间烟火乐子人', weights: { gossip: 3 } }
      ]
    },
    {
      id: 'xx2', tag: '早课试炼', title: '宗门规定每日卯时（早 5 点）集合吐纳练功，你的真实状态是？', sub: '修仙自律度检测',
      options: [
        { key: 'A', text: '寅时就已经在后山挥完三千剑，比掌门起得还早', sub: '卷王剑修本色', weights: { sword: 3 } },
        { key: 'B', text: '捏个纸人替身去点卯，本尊继续在洞府补回笼觉', sub: '摸鱼老祖亲传弟子', weights: { dao: 3, scheme: 1 } },
        { key: 'C', text: '准时到场但站在最后一排，悄悄跟旁边师兄唠昨晚的瓜', sub: '宗门情报交流早会', weights: { gossip: 3 } },
        { key: 'D', text: '仔细研究门规漏洞，成功申请成为“夜间守阵免早课长老”', sub: '规则利用大师', weights: { scheme: 3, dao: 1 } }
      ]
    },
    {
      id: 'xx3', tag: '秘境奇遇', title: '在千年古秘境里撞见一株绝世灵草，但旁边盘着一头打盹的妖兽，你会？', sub: '风险决策风格',
      options: [
        { key: 'A', text: '亮剑正面硬刚，顺便检验一下最近突破的剑意', sub: '遇强则强铁血战神', weights: { sword: 3 } },
        { key: 'B', text: '设个迷阵引走妖兽，零伤亡把灵草和妖兽老巢一起端了', sub: '零成本收益最大化', weights: { scheme: 3 } },
        { key: 'C', text: '算了命比灵草贵，拔两片边上的普通灵叶泡茶走人', sub: '保命第一稳健道心', weights: { dao: 3 } },
        { key: 'D', text: '丢两块灵肉过去套近乎，试图把妖兽拐回宗门当坐骑', sub: '万物皆可交朋友', weights: { gossip: 3, dao: 1 } }
      ]
    },
    {
      id: 'xx4', tag: '宗门大比', title: '五年一度的宗门大比即将开打，头名奖励是一颗极品破境丹，你会？', sub: '胜负欲鉴定',
      options: [
        { key: 'A', text: '闭关三月狂卷功法，势必要把头名刻在石碑上', sub: '纯粹强者执念', weights: { sword: 3 } },
        { key: 'B', text: '首轮体面落败，然后买把瓜子坐观战席点评全场', sub: '看戏比打架快乐一百倍', weights: { dao: 2, gossip: 3 } },
        { key: 'C', text: '提前摸清每位种子选手的弱点，专挑性价比最高的方式赢', sub: '精准算计稳拿奖励', weights: { scheme: 3, sword: 1 } },
        { key: 'D', text: '干脆报名当裁判或后勤炼丹师，旱涝保收还能收灵石', sub: '清醒务实不内卷', weights: { dao: 3, scheme: 1 } }
      ]
    },
    {
      id: 'xx5', tag: '洞府装修', title: '修为提升分到一座独立灵山洞府，你第一件装修大事是？', sub: '你的仙界生活方式',
      options: [
        { key: 'A', text: '布下九重防打扰聚灵阵，非请勿入、清静至上', sub: '高边界社恐仙尊', weights: { dao: 3, scheme: 1 } },
        { key: 'B', text: '辟出一片超大演武场和兵器架，极简冷冽', sub: '纯粹修行狂人', weights: { sword: 3 } },
        { key: 'C', text: '建个温泉茶亭 + 烧烤灵石炉，随时招待三界好友', sub: '洞府轰趴馆主', weights: { gossip: 3, dao: 1 } },
        { key: 'D', text: '留好三条隐秘逃生传送阵 + 密室藏宝库，狡兔三窟', sub: '安全感拉满谋略家', weights: { scheme: 3 } }
      ]
    },
    {
      id: 'xx6', tag: '灵石理财', title: '意外捡到一万块上品灵石，你最先怎么花？', sub: '修仙消费观',
      options: [
        { key: 'A', text: '全砸进顶级飞剑锻造材料，战力就是硬道理', sub: '重度自我投资', weights: { sword: 3 } },
        { key: 'B', text: '一半存进灵石庄吃利息，一半买顶级仙茶灵果享受人生', sub: '提前实现修仙退休', weights: { dao: 3 } },
        { key: 'C', text: '在仙门坊市盘两家铺子倒卖稀缺丹药，钱生钱', sub: '修真界商业奇才', weights: { scheme: 3, gossip: 1 } },
        { key: 'D', text: '请全峰师兄弟去醉仙楼搓一顿，顺便打听三界秘闻', sub: '豪爽仗义人脉王', weights: { gossip: 3 } }
      ]
    },
    {
      id: 'xx7', tag: '同门挑衅', title: '隔壁峰有个嚣张师兄当众嘲讽你修为进展慢，你会？', sub: '仙门反击姿势',
      options: [
        { key: 'A', text: '话不多说直接约生死台，三招教他重新做人', sub: '能动手绝不多废话', weights: { sword: 3 } },
        { key: 'B', text: '笑眯眯拱手：“师兄说得对，我确实爱摸鱼”——气死他', sub: '顶级松弛化骨绵掌', weights: { dao: 3 } },
        { key: 'C', text: '表面客气，半个月后他最想拿的秘境名额神秘落到了我手里', sub: '君子报仇优雅无声', weights: { scheme: 3 } },
        { key: 'D', text: '立刻把他的黑历史编成顺口溜，三天内传遍全宗门', sub: '舆论战绝对王者', weights: { gossip: 3, scheme: 1 } }
      ]
    },
    {
      id: 'xx8', tag: '本命法宝', title: '炼器师允许你定制一件专属本命法宝，你最心动哪件？', sub: '法宝即性格',
      options: [
        { key: 'A', text: '霜寒绝尘剑：出鞘即霜雪满天，一击必杀', sub: '锋利果决', weights: { sword: 3 } },
        { key: 'B', text: '自动恒温炼丹茶壶：能续命回血还能随时泡冰镇灵茶', sub: '松弛享乐主义', weights: { dao: 3 } },
        { key: 'C', text: '天机八卦罗盘：算尽吉凶人心，自带全图视野', sub: '掌控全局军师', weights: { scheme: 3 } },
        { key: 'D', text: '千里吃瓜留影镜：实时转播三界热搜绝不漏瓜', sub: '三界首席记者', weights: { gossip: 3 } }
      ]
    },
    {
      id: 'xx9', tag: '魔教诱惑', title: '魔教圣使半夜潜入洞府拉你跳槽，开出双倍灵石 + 无需早课，你会？', sub: '忠诚与务实博弈',
      options: [
        { key: 'A', text: '正邪不两立！一剑把对方轰出护山大阵', sub: '原则如铁', weights: { sword: 3 } },
        { key: 'B', text: '淡定倒杯茶：“先说说你们魔教的五险一金和年假怎么算”', sub: '清醒打工人本色', weights: { scheme: 2, dao: 2 } },
        { key: 'C', text: '顺势打入内部当双面间谍，两边情报和灵石全都要', sub: '高段位操盘手', weights: { scheme: 3 } },
        { key: 'D', text: '先八卦一小时魔教教主最近的情感纠葛，再婉拒对方', sub: '吃瓜刻进 DNA', weights: { gossip: 3 } }
      ]
    },
    {
      id: 'xx10', tag: '飞升执念', title: '修仙千年，你心里真正追求的终极境界是？', sub: '道心核心',
      options: [
        { key: 'A', text: '证道飞升、立于三界之巅，看遍最高处的风景', sub: '野心与锋芒', weights: { sword: 3 } },
        { key: 'B', text: '寿元管够、洞府有网、谁也管不着我的永恒松弛', sub: '逍遥长生派', weights: { dao: 3 } },
        { key: 'C', text: '不用站台前受累，但三界大事都得按我的棋局走', sub: '无冕之王', weights: { scheme: 3 } },
        { key: 'D', text: '和三五道友游历九州尝遍人间百味，快乐比飞升重要', sub: '红尘自在仙', weights: { gossip: 3, dao: 1 } }
      ]
    },
    {
      id: 'xx11', tag: '收徒风格', title: '掌门硬塞给你一个天赋平平但很听话的小徒弟，你会怎么带？', sub: '带队风格投影',
      options: [
        { key: 'A', text: '严师出高徒！制定魔鬼训练表，硬把废柴练成剑圣', sub: '高标准卷王师尊', weights: { sword: 3 } },
        { key: 'B', text: '放养式教学：“想练就练，不想练帮为师烧壶水烤红薯”', sub: '神仙松弛师尊', weights: { dao: 3 } },
        { key: 'C', text: '发现他口才不错，专门教他搞宗门人脉和资源整合', sub: '因材施教策略家', weights: { scheme: 3 } },
        { key: 'D', text: '带着徒弟满山抓灵宠逛夜市，处成忘年交铁哥们', sub: '快乐搭子师尊', weights: { gossip: 3 } }
      ]
    },
    {
      id: 'xx12', tag: '情劫考验', title: '天道突然降下一场“情劫”考验你的道心，你的过关方式是？', sub: '情感态度映射',
      options: [
        { key: 'A', text: '心中无爱拔剑自然神！直接一剑斩断情丝继续修仙', sub: '事业心拉满无情道', weights: { sword: 3 } },
        { key: 'B', text: '顺其自然：合拍就结为道侣双修，不合拍礼貌挥手再见', sub: '松弛清醒不内耗', weights: { dao: 3 } },
        { key: 'C', text: '先暗中考察对方三观背景，确认是双向奔赴才动真心', sub: '理智克制派', weights: { scheme: 3 } },
        { key: 'D', text: '轰轰烈烈谈一场再说！人生嘛体验感最重要', sub: '热烈体验派', weights: { gossip: 3 } }
      ]
    },
    {
      id: 'xx13', tag: '雷劫将至', title: '渡劫天雷即将劈下，劫云滚滚压顶，你此刻脑子里的念头是？', sub: '抗压心态检测',
      options: [
        { key: 'A', text: '“来得好！正好拿天雷淬炼我的肉身和剑骨”', sub: '越高压越兴奋', weights: { sword: 3 } },
        { key: 'B', text: '早买好了避雷阵 + 九层防御符，躺阵里喝茶等雷劈完', sub: '准备万全稳如老狗', weights: { scheme: 3, dao: 1 } },
        { key: 'C', text: '“能不能打个商量分期劈，我最近电量不太够”', sub: '自带幽默松弛感', weights: { dao: 3 } },
        { key: 'D', text: '赶紧叫道友拿留影石录下来，渡过去发修真界朋友圈！', sub: '生死看淡出片第一', weights: { gossip: 3 } }
      ]
    },
    {
      id: 'xx14', tag: '灵宠选择', title: '御兽阁送你一只先天灵宠作伴，你最想抱走哪一只？', sub: '气场偏好',
      options: [
        { key: 'A', text: '银瞳白虎：高冷威武，打架时一爪子秒杀全场', sub: '强者气场', weights: { sword: 3 } },
        { key: 'B', text: '嗜睡灵龟：千年不挪窝，陪我一起晒太阳发呆', sub: '佛系长寿', weights: { dao: 3 } },
        { key: 'C', text: '九尾灵狐：通晓人心变化，帮你参谋局势无往不利', sub: '智谋搭档', weights: { scheme: 3 } },
        { key: 'D', text: '话痨灵鹦鹉：能模仿掌门说话，每天给你播报宗门趣闻', sub: '快乐源泉', weights: { gossip: 3 } }
      ]
    },
    {
      id: 'xx15', tag: '隐藏人设', title: '三界流传着关于你的一个江湖传说，最可能是什么？', sub: '外界口碑',
      options: [
        { key: 'A', text: '“那位剑尊十年不出山，一出山就劈碎了排行榜”', sub: '低调硬核战神', weights: { sword: 3 } },
        { key: 'B', text: '“那位长老天天看着像在摸鱼，可谁也看不透他真实境界”', sub: '扫地僧式松弛大佬', weights: { dao: 3, scheme: 1 } },
        { key: 'C', text: '“千万别惹那位笑面军师，你刚抬手他就猜到了你后三步”', sub: '顶级谋士威名', weights: { scheme: 3 } },
        { key: 'D', text: '“全修真界没有他不认识的人，也没有他不知道的瓜”', sub: '三界百晓生', weights: { gossip: 3 } }
      ]
    },
    {
      id: 'xx16', tag: '命格落印', title: '若要为自己选一枚刻在玉简上的道号，你会pick？', sub: '最后一题 · 揭晓你的修仙命格',
      options: [
        { key: 'A', text: '「霜刃破云」——孤高锋利，一往无前', sub: '剑道独尊', weights: { sword: 3 } },
        { key: 'B', text: '「清风煮茶」——自在随心，长生无忧', sub: '逍遥散仙', weights: { dao: 3 } },
        { key: 'C', text: '「藏锋执棋」——运筹帷幄，笑看风云', sub: '幕后天机', weights: { scheme: 3 } },
        { key: 'D', text: '「醉月寻欢」——红尘作伴，快意潇洒', sub: '烟火剑客', weights: { gossip: 3 } }
      ]
    }
  ],
  profiles: [
    {
      code: '炼丹长老',
      title: '摆烂炼丹长老',
      subtitle: '木火双灵根 · 洞府养生天花板',
      rarity: '仅 5.5% 测出',
      quote: '宗门大比能逃则逃，唯独炼出好茶好丹时肯赏脸出山晃悠半日。',
      tags: ['洞府养生', '反卷老祖', '扫地僧实力'],
      buddy: { name: '吃瓜外门百晓生', desc: '对方负责带来三界热搜，你负责泡茶投喂灵丹' },
      rival: { name: '卷王孤高剑修', desc: '对方寅时练剑三千次，你还在洞府睡回笼觉' },
      avatarEmoji: '🫖',
      targetVector: { sword: 28, dao: 92, scheme: 52, gossip: 56 },
      extra: {
        sealText: '天品\n灵根',
        linggen: '木火双灵根（极品）',
        artifact: '冰美式九转丹炉',
        realm: '化神摸鱼后期',
        tribulation: '周一卯时早会',
        cave: '靠窗有网清静峰'
      }
    },
    {
      code: '孤高剑修',
      title: '卷王孤高剑修',
      subtitle: '变异雷剑灵根 · 一剑破万法',
      rarity: '仅 4.9% 测出',
      quote: '话不多、效率极高，别人还在纠结站队时，你已经一剑劈穿了天花板。',
      tags: ['慕强自律', '干脆利落', '战力担当'],
      buddy: { name: '表面正道魔教军师', desc: '对方负责布阵算计，你负责正面一剑封神' },
      rival: { name: '摆烂炼丹长老', desc: '你满腔热血邀约切磋，对方问要不要改天喝茶' },
      avatarEmoji: '⚔️',
      targetVector: { sword: 94, dao: 26, scheme: 54, gossip: 30 },
      extra: {
        sealText: '极品\n剑骨',
        linggen: '变异雷金单灵根',
        artifact: '霜寒绝尘本命剑',
        realm: '大乘冲刺巅峰',
        tribulation: '队友拖后腿磨叽',
        cave: '极简冷冽剑意崖'
      }
    },
    {
      code: '魔教军师',
      title: '表面正道魔教军师',
      subtitle: '暗影天机灵根 · 笑看三界棋局',
      rarity: '仅 4.4% 测出',
      quote: '表面温文尔雅最守门规，暗地里早把三界各方利益算到了三年之后。',
      tags: ['人间清醒', '规则大师', '优雅腹黑'],
      buddy: { name: '卷王孤高剑修', desc: '最强脑子配最强输出，横扫三界毫无悬念' },
      rival: { name: '直球热血小师弟', desc: '你苦心布好三层局，对方直接一嗓子喊穿底牌' },
      avatarEmoji: '🪭',
      targetVector: { sword: 48, dao: 54, scheme: 94, gossip: 46 },
      extra: {
        sealText: '天机\n命格',
        linggen: '暗影风双灵根',
        artifact: '九算八卦天机扇',
        realm: '合体藏锋境',
        tribulation: '遇不按套路出牌的猪队友',
        cave: '三出口秘境玄机阁'
      }
    },
    {
      code: '百晓生',
      title: '吃瓜外门情报头子',
      subtitle: '风火烟火灵根 · 三界热搜永动机',
      rarity: '仅 7.2% 测出',
      quote: '飞升哪有看三界八卦好玩？只要有你在，再冷清的宗门都能聊成大集市。',
      tags: ['三界百晓生', '社牛仗义', '烟火仙人'],
      buddy: { name: '摆烂炼丹长老', desc: '一个带瓜一个泡茶，洞府下午茶永不散场' },
      rival: { name: '社恐闭关符箓师', desc: '你激动敲门分享惊天大瓜，对方贴了三层静音符' },
      avatarEmoji: '🏮',
      targetVector: { sword: 36, dao: 56, scheme: 48, gossip: 94 },
      extra: {
        sealText: '红尘\n仙缘',
        linggen: '风火双灵根',
        artifact: '千里吃瓜留影镜',
        realm: '元婴乐子圆满',
        tribulation: '手机/玉简没信号',
        cave: '山下夜市醉仙居'
      }
    },
    {
      code: '符箓大师',
      title: '社恐隐世符箓大师',
      subtitle: '冰水静默灵根 · 护山结界建筑师',
      rarity: '仅 5.9% 测出',
      quote: '能贴张静音符解决的事绝不开麦，看似与世无争，实则身家丰厚稳如泰山。',
      tags: ['静音结界', '闷声发财', '高边界感'],
      buddy: { name: '表面正道魔教军师', desc: '彼此点到为止不废话，合作效率奇高' },
      rival: { name: '吃瓜外门情报头子', desc: '对方每天上门串门三小时，结界都快被聊穿了' },
      avatarEmoji: '📜',
      targetVector: { sword: 44, dao: 82, scheme: 78, gossip: 18 },
      extra: {
        sealText: '玄清\n道体',
        linggen: '冰水变异灵根',
        artifact: '九重免打扰护山符',
        realm: '炼虚静默境',
        tribulation: '被迫参加宗门团建',
        cave: '云深不知处静心谷'
      }
    },
    {
      code: '逍遥侠客',
      title: '红尘自在逍遥侠',
      subtitle: '全系混沌灵根 · 既能拔剑也能喝酒',
      rarity: '仅 6.6% 测出',
      quote: '有事时一剑顶在最前面，无事时提壶逛遍人间夜市，活得最洒脱自在。',
      tags: ['快意恩仇', '仗义热血', '能文能武'],
      buddy: { name: '吃瓜外门情报头子', desc: '结伴闯荡九州，每一站都是高能奇遇' },
      rival: { name: '表面正道魔教军师', desc: '讨厌弯弯绕绕算计，有话直说才痛快' },
      avatarEmoji: '🍶',
      targetVector: { sword: 78, dao: 62, scheme: 34, gossip: 84 },
      extra: {
        sealText: '逍遥\n剑仙',
        linggen: '混沌全系灵根',
        artifact: '青莲醉月飞剑',
        realm: '化神自在境',
        tribulation: '被繁琐规矩死死捆住',
        cave: '四海为家随缘客栈'
      }
    }
  ]
};
