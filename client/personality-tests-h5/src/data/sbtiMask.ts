import type { TestDefinition } from '../engine/types.ts';

export const sbtiMaskTest: TestDefinition = {
  id: 'sbti',
  title: '人类真实面具',
  shortTitle: '真实面具',
  subtitle: 'SBTI 毒舌解构 · 含 2 个隐藏彩蛋',
  heroDesc: '撕掉正经人格滤镜！测出你是「体面伪人」「摸鱼圣体」「赛博苦行僧」还是 1% 隐藏彩蛋款？',
  category: 'viral',
  badge: '本周爆款',
  estimatedMinutes: 2,
  cardBg: '#E4F857',
  posterBg: '#E4F857',
  tagColor: '#E4F857',
  accentColor: '#FF4D5E',
  scoringMode: 'nearest-profile',
  dimensions: [
    { id: 'mask', label: '体面伪装', hotWhenHigh: true },
    { id: 'chill', label: '班味防御', hotWhenHigh: false },
    { id: 'ghost', label: '已读不回', hotWhenHigh: true },
    { id: 'social', label: '社交电量', hotWhenHigh: false },
    { id: 'overthink', label: '深夜内耗', hotWhenHigh: true },
    { id: 'control', label: '暗中控场', hotWhenHigh: false }
  ],
  microHints: [
    '✦ 刚开始热身，放心说真话',
    '✦ 已扫描到你的礼貌外壳厚度…',
    '✦ 已看穿你 50% 的社交体面伪装',
    '✦ 检测到高频深夜弹幕与已读不回倾向',
    '✦ 马上撕开滤镜，校验 1% 隐藏彩蛋…'
  ],
  questions: [
    {
      id: 'sb1',
      tag: '职场防线',
      title: '周五 17:50 工作群突然 @ 你“方便拉个小会吗”，你内心第一反应是？',
      sub: '选最贴近你真实弹幕的一项，不必维持体面',
      options: [
        { key: 'A', text: '嘴上秒回“好的马上”，心里已写好三版离职信', sub: '极厚礼貌外壳 + 高压弹幕', weights: { mask: 3, overthink: 2, chill: 1 } },
        { key: 'B', text: '假装没看见，拖到 17:59 问“下周一聊行吗”', sub: '精准卡点的时间管理大师', weights: { ghost: 3, chill: 3 } },
        { key: 'C', text: '直接反问议题和预计时长，速战速决', sub: '高边界控场型，拒绝模糊拉扯', weights: { control: 3, social: 1 } },
        { key: 'D', text: '有点好奇是什么瓜，顺手点开听听顺便凑热闹', sub: '电量充足自带吃瓜雷达', weights: { social: 3, chill: 1 } }
      ]
    },
    {
      id: 'sb2',
      tag: '社交电量',
      title: '原本约好周末出门聚餐，当天早上朋友发来“有点累要不改天”，你会？',
      sub: '想象自己刚睁开眼躺在被窝里',
      options: [
        { key: 'A', text: '内心放烟花，回“太理解了快好好休息！”', sub: '被放鸽子是最顶级多巴胺', weights: { mask: 2, chill: 3, ghost: 1 } },
        { key: 'B', text: '长舒一口气，立刻下单外卖切回追剧模式', sub: '独处充电比热闹聚会更回血', weights: { chill: 3, social: -2 } },
        { key: 'C', text: '已经画好妆了有点失落，顺手摇另一位搭子', sub: '高电量现充，绝不浪费好天气', weights: { social: 3, control: 1 } },
        { key: 'D', text: '嘴上说没事，心里因为计划被打乱默默扣 5 分', sub: '嘴硬计划控，隐性记小本本', weights: { control: 2, overthink: 2, mask: 2 } }
      ]
    },
    {
      id: 'sb3',
      tag: '手机红点',
      title: '收到一条不太想立刻回的微信消息，你最真实的处理方式是？',
      sub: '诚实面对你的通知栏',
      options: [
        { key: 'A', text: '下拉通知栏偷看完，假装自己没上线', sub: '意念回复十级选手', weights: { ghost: 3, mask: 1 } },
        { key: 'B', text: '心里排练了 5 版措辞，最后拖到三天后才回', sub: '过度思考导致回复瘫痪', weights: { overthink: 3, ghost: 2 } },
        { key: 'C', text: '立刻回个表情包结束话题，绝不留红点', sub: '强迫症式高效清零', weights: { control: 2, mask: 2 } },
        { key: 'D', text: '秒回加语音轰炸，顺便展开聊半小时', sub: '随时满格在线的聊天狂魔', weights: { social: 3, ghost: -2 } }
      ]
    },
    {
      id: 'sb4',
      tag: '团建噩梦',
      title: '公司或社团宣布周末搞“破冰素拓 + 真心话大冒险”，你的灵魂状态是？',
      sub: '此刻你想按下哪个按钮',
      options: [
        { key: 'A', text: '笑着报名并积极鼓掌，回家累到两天不想说话', sub: '白天 E 人演技拉满，夜晚电量归零', weights: { mask: 3, overthink: 1, social: -1 } },
        { key: 'B', text: '瞬间查好请假话术：家里的猫要去做心理咨询', sub: '班味防御满级，谁也别想占用周末', weights: { chill: 3, ghost: 2 } },
        { key: 'C', text: '如果实在逃不掉，干脆主动当主持掌控节奏', sub: '与其被尴尬支配，不如自己定规则', weights: { control: 3, social: 2 } },
        { key: 'D', text: '居然有点期待，能蹭顿饭还能认识新朋友', sub: '天生社牛免疫一切尴尬', weights: { social: 3, chill: 1 } }
      ]
    },
    {
      id: 'sb5',
      tag: '深夜剧场',
      title: '凌晨 1 点准备睡觉，大脑突然自动播放你三年前的一次社死瞬间，你会？',
      sub: '高频精神内耗时刻',
      options: [
        { key: 'A', text: '脚趾抠地翻身三圈，顺便把人生复盘到天亮', sub: '细节记忆力过剩型内耗选手', weights: { overthink: 3, mask: 1 } },
        { key: 'B', text: '自我安慰“反正没人记得”，五分钟后香甜入睡', sub: '顶级钝感力，自带一键清空缓存', weights: { chill: 3, overthink: -2 } },
        { key: 'C', text: '在脑内重拍一遍，写出当时最帅气的反击台词', sub: '迟到三年的完美剧本导演', weights: { control: 2, overthink: 2 } },
        { key: 'D', text: '立刻发给闺蜜/铁子一起笑，社死只要分享就减半', sub: '情绪外置处理器', weights: { social: 3, chill: 1 } }
      ]
    },
    {
      id: 'sb6',
      tag: '消费观',
      title: '面对购物车里躺了一个月的心动好物，你最后怎么决定买不买？',
      sub: '看你的理性与冲动博弈',
      options: [
        { key: 'A', text: '货比五家做 Excel 评测，最后错过满减干脆不买', sub: '研究过程已经爽完了', weights: { control: 2, overthink: 3 } },
        { key: 'B', text: '受了点委屈立刻全款拿下：“这是我的精神损失费”', sub: '情绪消费正当化大师', weights: { chill: 2, overthink: 1, mask: 1 } },
        { key: 'C', text: '看一眼余额心如止水，悟出“物欲皆空”', sub: '低欲望赛博修行者', weights: { chill: 3, social: -1 } },
        { key: 'D', text: '拉着朋友一起凑单，顺便互相安利种草', sub: '社交型购物快乐翻倍', weights: { social: 3, control: 1 } }
      ]
    },
    {
      id: 'sb7',
      tag: '情绪伪装',
      title: '其实今天心情糟透了，但同事/同学随口问一句“最近挺顺利吧”，你会？',
      sub: '观察你的第一层保护色',
      options: [
        { key: 'A', text: '挂上标准职业微笑：“挺好的呀，老样子哈哈”', sub: '情绪稳定得像预装了礼貌插件', weights: { mask: 3, ghost: 1 } },
        { key: 'B', text: '直接开摆吐槽：“别提了，魂都快飘出工位了”', sub: '真实不装，主打一个共沉沦', weights: { chill: 2, social: 2, mask: -2 } },
        { key: 'C', text: '只挑能解决的部分简短同步，情绪自己咽下去', sub: '问题导向，不习惯暴露脆弱', weights: { control: 3, mask: 2 } },
        { key: 'D', text: '反问对方最近怎样，丝滑把聚光灯转走', sub: '顶级话题转移避险术', weights: { mask: 2, ghost: 2, overthink: 1 } }
      ]
    },
    {
      id: 'sb8',
      tag: '电梯尴尬',
      title: '电梯里偶遇半熟不熟的领导或远房熟人，剩余 15 层你会怎么度过？',
      sub: '空气突然安静的 30 秒',
      options: [
        { key: 'A', text: '死盯手机屏幕假装在批阅千万级合同', sub: '低头隐身术全国统考满分', weights: { ghost: 3, mask: 2 } },
        { key: 'B', text: '搜肠刮肚夸一句“今天天气真不错”，说完脚趾抠地', sub: '被迫营业后的严重内耗', weights: { mask: 3, overthink: 2 } },
        { key: 'C', text: '自然寒暄两句顺便同步下最近进展，毫不怯场', sub: '向上社交游刃有余', weights: { social: 3, control: 2 } },
        { key: 'D', text: '礼貌点个头就放空发呆，尴尬的从来不是我', sub: '内核超稳的无所谓选手', weights: { chill: 3, overthink: -1 } }
      ]
    },
    {
      id: 'sb9',
      tag: '旅行分工',
      title: '和三四个朋友约好去外地玩 3 天，你通常会变成哪个角色？',
      sub: '集体出行照妖镜',
      options: [
        { key: 'A', text: '连哪家店要排几分钟都做进表格的铁血总领队', sub: '不掌控全局心里发慌', weights: { control: 3, overthink: 1 } },
        { key: 'B', text: '自带“我都行、听你们的、我负责付钱和拍照”的省心挂件', sub: '情绪价值拉满的躺平搭子', weights: { chill: 3, mask: 1 } },
        { key: 'C', text: '气氛组 + 探店雷达，走到哪都能和老板唠上嗑', sub: '自带热闹场域的快乐小狗', weights: { social: 3 } },
        { key: 'D', text: '走到第二天下午就想独自回酒店静静躺两小时', sub: '急需独处回血的脆皮电池', weights: { ghost: 2, chill: 2, social: -2 } }
      ]
    },
    {
      id: 'sb10',
      tag: '朋友圈心理',
      title: '你发一条朋友圈（或小红书）后的真实心理活动是？',
      sub: '点击发布后的半小时',
      options: [
        { key: 'A', text: '反复重读 8 遍，隔天觉得矫情默默设成仅自己可见', sub: '间歇性表达欲 + 极速后悔', weights: { overthink: 3, ghost: 2 } },
        { key: 'B', text: '分组屏蔽 6 个列表，精准投放不同人设版本', sub: '精细化社交矩阵运营大师', weights: { mask: 3, control: 2 } },
        { key: 'C', text: '发完直接关软件，点赞评论随缘看', sub: '松弛记录生活，不被数据绑架', weights: { chill: 3 } },
        { key: 'D', text: '每条评论都认真玩梗回复，顺便在评论区聊成群聊', sub: '高互动社交达人', weights: { social: 3 } }
      ]
    },
    {
      id: 'sb11',
      tag: '熬夜真相',
      title: '明明困得睁不开眼，为什么半夜 1 点半你还不肯放下手机？',
      sub: '报复性熬夜诊断',
      options: [
        { key: 'A', text: '白天都在扮演情绪稳定的大人，只有深夜时间才属于我', sub: '在凌晨偷回一点人生主权', weights: { mask: 3, overthink: 2 } },
        { key: 'B', text: '刷搞笑视频和沙雕评论停不下来，纯粹快乐', sub: '多巴胺续命选手', weights: { chill: 2, social: 1 } },
        { key: 'C', text: '总觉得今天少做了点啥，刷点干货假装自己在努力', sub: '焦虑驱动型伪自律', weights: { control: 2, overthink: 3 } },
        { key: 'D', text: '其实我 11 点就睡了，熬夜是什么我不熟', sub: '罕见的碳基生物作息模范', weights: { control: 2, chill: 2, overthink: -2 } }
      ]
    },
    {
      id: 'sb12',
      tag: '面对画饼',
      title: '有人给你画了一个听起来很宏大但明显不靠谱的大饼，你会？',
      sub: '你的反画饼防御值',
      options: [
        { key: 'A', text: '眼神真挚频频点头：“太有启发了”，转头打开招聘软件', sub: '嘴上满分配合，行动极其清醒', weights: { mask: 3, control: 2 } },
        { key: 'B', text: '面无表情微笑：“那这笔预算什么时候到账呢？”', sub: '一针见血的现实主义终结者', weights: { control: 3, mask: -1 } },
        { key: 'C', text: '左耳进右耳出，脑子里已经在想中午吃麻辣烫还是螺蛳粉', sub: '自动开启精神离线护盾', weights: { chill: 3, ghost: 2 } },
        { key: 'D', text: '居然短暂燃了一下，晚上冷静下来才发现被忽悠了', sub: '容易被热情点燃的理想派', weights: { social: 2, overthink: 2 } }
      ]
    },
    {
      id: 'sb13',
      tag: '边界感',
      title: '不太熟的人突然找你借钱或让你免费帮忙做一件挺费力的事，你会？',
      sub: '拒绝别人的难易程度',
      options: [
        { key: 'A', text: '纠结半小时不好意思直接拒绝，最后编了个超长理由', sub: '温和派，拒绝别人比自己干活还累', weights: { overthink: 3, mask: 2 } },
        { key: 'B', text: '直接已读不回，让沉默替我回答一切', sub: '只要我不尴尬，尴尬的就是对方', weights: { ghost: 3, chill: 1 } },
        { key: 'C', text: '礼貌而坚定地回“最近排期满了，实在帮不上忙”', sub: '干脆利落的成年人边界感', weights: { control: 3, mask: 1 } },
        { key: 'D', text: '如果对方态度好且顺手，就帮一把交个朋友', sub: '热心仗义容易心软', weights: { social: 3, overthink: 1 } }
      ]
    },
    {
      id: 'sb14',
      tag: '外卖选择',
      title: '中午点外卖时，打开 App 划了 20 分钟还没下单，通常是因为？',
      sub: '当代选择困难实录',
      options: [
        { key: 'A', text: '算满减、看差评、比配送费，像在审批重大采购', sub: '极致性价比与细节控', weights: { control: 2, overthink: 3 } },
        { key: 'B', text: '什么都想吃又什么都不想吃，最后点了昨天吃过的那家', sub: '安全感优先的懒人策略', weights: { chill: 3 } },
        { key: 'C', text: '等搭子先选，然后一句“那我和你点一家拼配送费”', sub: '决策外包省心第一', weights: { social: 2, chill: 2 } },
        { key: 'D', text: '其实我只用 10 秒就选完了，效率拉满', sub: '果断不内耗的行动派', weights: { control: 3, overthink: -2 } }
      ]
    },
    {
      id: 'sb15',
      tag: '情绪发泄',
      title: '压力大到快溢出来的时候，你最常选择的“偷偷回血方式”是？',
      sub: '你的秘密解压出口',
      options: [
        { key: 'A', text: '关灯戴耳机听歌发呆，谁也别跟我说话', sub: '彻底断联的静默重启', weights: { ghost: 3, overthink: 1, social: -2 } },
        { key: 'B', text: '炫一顿碳水炸弹或微醺小酌一杯，今晚先放过自己', sub: '碳水与微醺治愈一切不服', weights: { chill: 3, overthink: -1 } },
        { key: 'C', text: '疯狂整理房间、列计划表，把失控感重新抓回手里', sub: '通过秩序感恢复心理掌控', weights: { control: 3 } },
        { key: 'D', text: '找铁子开启两小时高密度语音吐槽大会', sub: '把情绪说出来就赢了一半', weights: { social: 3 } }
      ]
    },
    {
      id: 'sb16',
      tag: '人际直觉',
      title: '刚认识一个新朋友，对方嘴上客气但眼神微妙，你的雷达会？',
      sub: '察言观色灵敏度',
      options: [
        { key: 'A', text: '0.5 秒就察觉到不对劲，但面上依然笑得像春风', sub: '高敏雷达 + 顶级伪装', weights: { mask: 3, overthink: 2, control: 1 } },
        { key: 'B', text: '直接悄悄拉开距离，懒得费神猜对方心思', sub: '能量守恒，远离复杂人类', weights: { ghost: 2, chill: 2 } },
        { key: 'C', text: '完全没注意到，甚至觉得对方还挺友善的', sub: '大心脏钝感福星', weights: { chill: 3, overthink: -2, social: 1 } },
        { key: 'D', text: '主动开个玩笑试探破冰，看能不能处成自己人', sub: '主动出击掌控氛围', weights: { social: 3, control: 2 } }
      ]
    },
    {
      id: 'sb17',
      tag: '工作节奏',
      title: '离一个重要任务截止还有 5 天，你的真实推进曲线更像？',
      sub: 'DDL 驱动力检测',
      options: [
        { key: 'A', text: '前 4 天疯狂精神内耗但没动笔，最后 6 小时爆种交出精品', sub: '肾上腺素型压线战神', weights: { overthink: 3, chill: 1 } },
        { key: 'B', text: '第一天就拆完步骤，提前一天交差然后安心摸鱼', sub: '先苦后甜的掌控感玩家', weights: { control: 3, chill: 1 } },
        { key: 'C', text: '能 60 分飘过就绝不卷到 100 分，留点命享受生活', sub: '反内卷性价比哲学家', weights: { chill: 3, mask: 1 } },
        { key: 'D', text: '一边拉着搭子连麦互相监督，一边聊天一边写完', sub: '陪伴型高效产出', weights: { social: 3 } }
      ]
    },
    {
      id: 'sb18',
      tag: '群聊生存',
      title: '在一个 200 人的兴趣群或同事群里，你通常的活跃形态是？',
      sub: '群聊生态位鉴定',
      options: [
        { key: 'A', text: '常年隐身潜水，只在发红包或发重要通知时冒个泡', sub: '幽灵观察者，从不留痕', weights: { ghost: 3, social: -1 } },
        { key: 'B', text: '礼貌复制上一句“收到/恭喜/哈哈哈”，完美融入人群', sub: '群聊气氛礼貌复读机', weights: { mask: 3 } },
        { key: 'C', text: '金句输出机 + 表情包大户，经常带飞全群节奏', sub: '群聊灵魂段子手', weights: { social: 3, control: 1 } },
        { key: 'D', text: '直接设置免打扰折叠进群助手，眼不见心不烦', sub: '信息降噪清静派', weights: { chill: 3, ghost: 2 } }
      ]
    },
    {
      id: 'sb19',
      tag: '自我评价',
      title: '如果让最了解你的 1 位死党用一句话吐槽你，最可能说？',
      sub: '第三方视角校准',
      options: [
        { key: 'A', text: '“你对外人客气得像客服，对自己人嘴硬得像石头”', sub: '外圆内方的反差伪人', weights: { mask: 3, overthink: 2 } },
        { key: 'B', text: '“天塌下来你都要先问一句今天外卖满减多少”', sub: '情绪稳定到令人发指', weights: { chill: 3, overthink: -1 } },
        { key: 'C', text: '“嘴上说着随便，其实心里早把每个人安排明白了”', sub: '不动声色的幕后军师', weights: { control: 3, mask: 1 } },
        { key: 'D', text: '“手机长在手上却经常失联，但一见面又能聊到凌晨”', sub: '间歇性失联高质量搭子', weights: { ghost: 2, social: 2 } }
      ]
    },
    {
      id: 'sb20',
      tag: '彩蛋终极题',
      title: '如果现在白送你 48 小时“全宇宙免打扰通行证”，你最想怎么挥霍？',
      sub: '最后一题 · 选完立刻撕开真实面具',
      options: [
        { key: 'A', text: '手机开飞行模式，睡到自然醒 + 喝点小酒吹晚风放空', sub: '彻底离线，把世界关在门外', weights: { chill: 3, ghost: 3 } },
        { key: 'B', text: '把拖了半年的个人心愿清单一口气爽快清零', sub: '高掌控心流爽感', weights: { control: 3, overthink: -1 } },
        { key: 'C', text: '买张随心飞去陌生城市独自乱逛，谁也不认识我最好', sub: '短暂脱下所有社会角色', weights: { mask: 2, ghost: 2, chill: 2 } },
        { key: 'D', text: '叫上最对味的 2 个铁子吃火锅唱 K 聊到天亮', sub: '高浓度快乐充电', weights: { social: 3, chill: 1 } }
      ]
    }
  ],
  profiles: [
    {
      code: 'DRUNK',
      title: '绝版微醺酒鬼',
      subtitle: '1% 隐藏彩蛋 · 清醒摆烂天花板',
      rarity: '仅 1.1% 触发',
      quote: '不是真的嗜酒，是只有微醺和离线的时候，才觉得这个荒诞的世界顺眼了一点。',
      tags: ['1% 隐藏彩蛋', '高阶反内卷', '随时离线'],
      buddy: { name: 'MONK · 赛博苦行僧', desc: '一个端杯看戏一个盘腿看空，绝佳避世搭子' },
      rival: { name: 'CTRL · 隐形拿捏者', desc: '对方越催着定 KPI，你越想立刻关机失联' },
      avatarEmoji: '🍸',
      isHiddenEgg: true,
      eggConditions: [
        { dim: 'chill', op: 'gte', value: 78 },
        { dim: 'ghost', op: 'gte', value: 72 }
      ],
      extra: { stampText: 'HIDDEN EGG', eggHint: '恭喜解锁 1% 稀有彩蛋款！全站极少人能同时拉满松弛与离线值' }
    },
    {
      code: 'NORM',
      title: '绝版正常人',
      subtitle: '1% 隐藏彩蛋 · 当代碳基稀缺物种',
      rarity: '仅 1.4% 触发',
      quote: '既不精神内耗也不演戏装累，作息相对正常、情绪自带稳压器——在当代互联网简直像活化石。',
      tags: ['1% 隐藏彩蛋', '情绪稳压器', '碳基奇迹'],
      buddy: { name: 'SUN · 快乐发电站', desc: '两个健康灵魂凑一块，能把日子过出阳光滤镜' },
      rival: { name: 'LOOP · 深夜复盘怪', desc: '你实在理解不了为什么凌晨三点还在想三年前的话' },
      avatarEmoji: '🌿',
      isHiddenEgg: true,
      eggConditions: [
        { dim: 'overthink', op: 'lte', value: 26 },
        { dim: 'mask', op: 'lte', value: 38 }
      ],
      extra: { stampText: 'RARE 1%', eggHint: '恭喜解锁隐藏彩蛋！低内耗 + 低伪装的绝版正常人' }
    },
    {
      code: 'FAKE',
      title: '体面伪人',
      subtitle: '外礼内疯 · 高级情绪表演艺术家',
      rarity: '仅 4.1% 测出',
      quote: '白天情绪稳定到像装了礼貌 AI 插件，凌晨两点在脑内演完三季甄嬛传。',
      tags: ['嘴硬心软', '班味防御', '深夜剧场'],
      buddy: { name: 'MONK · 赛博苦行僧', desc: '同样低欲望高边界，互不打扰就是最高级陪伴' },
      rival: { name: 'CTRL · 隐形拿捏者', desc: '对方越想控场施压，你越想立刻原地礼貌失联' },
      avatarEmoji: '🎭',
      targetVector: { mask: 88, chill: 52, ghost: 66, social: 34, overthink: 78, control: 56 },
      extra: { stampText: '高危真实', eggHint: '彩蛋提示：把班味防御和已读不回同时拉满，可解锁 1% 彩蛋' }
    },
    {
      code: 'MOYU',
      title: '天选摸鱼圣体',
      subtitle: '能量守恒 · 反内卷性价比哲学家',
      rarity: '仅 6.8% 测出',
      quote: '工位是肉身的牢笼，但没人能阻止我的灵魂在下午三点飞去海边吹风。',
      tags: ['带薪发呆', '拒绝内耗', '松弛大师'],
      buddy: { name: 'SUN · 快乐发电站', desc: '一个找乐子一个搭台子，工位摸鱼快乐翻倍' },
      rival: { name: 'BOSS · 铁血排期王', desc: '对方每小时问一次进度，直接触犯你的松弛天条' },
      avatarEmoji: '🐟',
      targetVector: { mask: 44, chill: 88, ghost: 58, social: 52, overthink: 28, control: 36 },
      extra: { stampText: '松弛天花板', eggHint: '距离触发隐藏款「绝版酒鬼」只差一点已读不回值' }
    },
    {
      code: 'MONK',
      title: '赛博苦行僧',
      subtitle: '低欲断联 · 高边界精神独行者',
      rarity: '仅 5.3% 测出',
      quote: '物欲很低，社交很淡，被放鸽子反而是周末最开心的瞬间。',
      tags: ['已读不回', '独处满血', '情绪省电'],
      buddy: { name: 'FAKE · 体面伪人', desc: '彼此懂得礼貌留白，半年不联系一见面照样熟' },
      rival: { name: 'SUN · 快乐发电站', desc: '对方连续三天拉你组局，你的社交电池会原地冒烟' },
      avatarEmoji: '🪷',
      targetVector: { mask: 46, chill: 76, ghost: 88, social: 18, overthink: 48, control: 46 },
      extra: { stampText: '省电模式', eggHint: '你已极其接近隐藏彩蛋，保持高边界就是你的超能力' }
    },
    {
      code: 'CTRL',
      title: '隐形拿捏者',
      subtitle: '冷静军师 · 嘴上随便心里有谱',
      rarity: '仅 4.9% 测出',
      quote: '从不跟人扯嗓门吵架，但早就用三步棋把局势和退路悄悄算明白了。',
      tags: ['幕后军师', '高边界感', '效率拉满'],
      buddy: { name: 'BOSS · 铁血排期王', desc: '强强联手办事零废话，一天能干完别人一周的活' },
      rival: { name: 'MOYU · 天选摸鱼圣体', desc: '你想推进方案，对方只想讨论晚饭吃什么' },
      avatarEmoji: '♟️',
      targetVector: { mask: 68, chill: 46, ghost: 42, social: 58, overthink: 54, control: 90 },
      extra: { stampText: '控场专家', eggHint: '偶尔把掌控权交出去一次，会发现世界也没塌' }
    },
    {
      code: 'LOOP',
      title: '深夜复盘怪',
      subtitle: '高敏雷达 · 细节记忆力过剩选手',
      rarity: '仅 7.2% 测出',
      quote: '别人随口说的一句话，我能在凌晨两点的脑内法庭里开庭审理三个小时。',
      tags: ['高敏共情', '迟到反击', '脑内导演'],
      buddy: { name: 'SUN · 快乐发电站', desc: '对方的直球和大心脏，能一秒吹散你的深夜乌云' },
      rival: { name: 'MONK · 赛博苦行僧', desc: '对方回个“嗯”，你能脑补出八集分手/绝交大戏' },
      avatarEmoji: '🌀',
      targetVector: { mask: 66, chill: 30, ghost: 58, social: 36, overthink: 92, control: 48 },
      extra: { stampText: '内耗预警', eggHint: '记得给大脑设置凌晨 12 点自动下班闹钟' }
    },
    {
      code: 'SUN',
      title: '快乐发电站',
      subtitle: '天然社牛 · 走到哪都能热场的 E 人光环',
      rarity: '仅 6.4% 测出',
      quote: '手机电量可以只剩 5%，但只要对面坐着对味的人，我的聊天电量永远满格。',
      tags: ['气氛组长', '直球热心', '吃瓜雷达'],
      buddy: { name: 'LOOP · 深夜复盘怪', desc: '你负责把人拉出胡思乱想，对方负责记住你所有小喜好' },
      rival: { name: 'MONK · 赛博苦行僧', desc: '你发了十条 60 秒语音，对方三天后回了一个句号' },
      avatarEmoji: '☀️',
      targetVector: { mask: 36, chill: 62, ghost: 18, social: 92, overthink: 32, control: 54 },
      extra: { stampText: '满格电量', eggHint: '高电量稀有生物，全群冷场时全靠你救火' }
    },
    {
      code: 'BOSS',
      title: '铁血排期王',
      subtitle: '秩序强迫 · 清单打钩重度成瘾者',
      rarity: '仅 5.7% 测出',
      quote: '人生最大的多巴胺不是放假，是把待办清单上最后一项用力划掉的那一秒。',
      tags: ['计划控', '拒绝模糊', '行动派'],
      buddy: { name: 'CTRL · 隐形拿捏者', desc: '两个清醒大佬凑一起，效率高到让周围人汗流浃背' },
      rival: { name: 'MOYU · 天选摸鱼圣体', desc: '你连旅游都排到分钟，对方睡到中午说随缘逛逛' },
      avatarEmoji: '⚡',
      targetVector: { mask: 42, chill: 34, ghost: 26, social: 62, overthink: 44, control: 86 },
      extra: { stampText: '效率满级', eggHint: '偶尔允许自己留白半天，灵感反而会自己找上门' }
    },
    {
      code: 'CAT',
      title: '间歇性失联猫',
      subtitle: '忽冷忽热 · 只在想贴贴时出现的神秘物种',
      rarity: '仅 6.1% 测出',
      quote: '消息不是不想回，是当下电量只够发呆；但只要我在状态，就能陪你聊到天亮。',
      tags: ['间歇活跃', '慢热护短', '氛围挑剔'],
      buddy: { name: 'FAKE · 体面伪人', desc: '都懂社交耗电的痛，不黏人却永远彼此兜底' },
      rival: { name: 'BOSS · 铁血排期王', desc: '被催着必须 5 分钟内给明确回复，会瞬间炸毛逃跑' },
      avatarEmoji: '🐈',
      targetVector: { mask: 56, chill: 68, ghost: 76, social: 54, overthink: 60, control: 38 },
      extra: { stampText: '猫系节奏', eggHint: '对的人从不会因为你回得慢而怀疑你的真心' }
    }
  ]
};
