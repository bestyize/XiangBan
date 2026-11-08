/* Original everyday scenarios. All scoring and save validation are DOM independent. */
(() => {
  'use strict'

  const dimensions = [
    { key: 'EI', first: 'E', second: 'I', firstLabel: '外向', secondLabel: '内向', context: '精力的去向' },
    { key: 'SN', first: 'S', second: 'N', firstLabel: '实感', secondLabel: '直觉', context: '信息的入口' },
    { key: 'TF', first: 'T', second: 'F', firstLabel: '思考', secondLabel: '情感', context: '决定的依据' },
    { key: 'JP', first: 'J', second: 'P', firstLabel: '判断', secondLabel: '感知', context: '生活的节奏' },
  ]

  // Each row gives the prompt, then the first-pole and second-pole choices.
  const scenarios = [
    [
      ['忙碌了一周，周五晚上你终于有了空闲。怎样更容易恢复精神？', '约几位熟悉的朋友吃饭，边聊边放松。', '留一段不被打扰的时间，自己做点喜欢的事。'],
      ['到了一个不太熟悉的聚会，你通常怎样进入状态？', '找个话题，先和身边的人聊起来。', '先听听大家在聊什么，再加入一段谈话。'],
      ['一个工作想法刚冒出来，还没完全成形。你更习惯怎样梳理？', '找个人说一说，在来回交流里理清它。', '先自己写写想想，理顺以后再找人交流。'],
      ['出差时有半天空档，同事提议一起逛逛。你更想怎样安排？', '一起走走，有人交流见闻会更有兴致。', '各自逛一会儿，按自己的节奏看看周围。'],
      ['参加一天讨论后，中场休息的二十分钟你更想做什么？', '继续和几个人随意聊聊，换换话题。', '到安静的地方坐一会儿，暂时少说点话。'],
      ['搬到新社区，周末有一场住户活动。你更可能怎样做？', '去现场认识邻居，看看能聊出些什么。', '先过好自己的周末，遇到邻居再慢慢熟悉。'],
      ['碰到一件让你高兴的小事，你通常先怎样消化这份心情？', '马上和人分享，听听对方的回应。', '自己回味一阵，再挑合适的时候说。'],
      ['两份工作内容相近，其中一种沟通节奏让你更舒服。你会选哪种？', '一天里多次短交流，随时碰一碰想法。', '集中几次交流，其余时间独立投入。'],
      ['周末学一项新手艺，两种学习形式都方便。哪种更吸引你？', '参加小班，在一起练习和交谈中摸索。', '自己跟教程练，卡住了再向别人请教。'],
      ['和熟人连续相处了两天，接下来还有一天假期。你更倾向于？', '再约个轻松活动，继续一起度过。', '留一天给自己，让注意力安静下来。'],
      ['会议里出现一个你关心但还没想透的问题。你更常怎样参与？', '先说出初步想法，借讨论把细节补齐。', '先在心里整理，形成完整想法后再说。'],
      ['午饭时没有别的安排，你更期待哪种休息？', '和同事坐一桌，聊些工作以外的事。', '找个清静位置，独自吃饭或散步。'],
    ],
    [
      ['接到一项从没做过的任务，你想先看到哪类材料？', '一份实际做成的样例，以及具体操作步骤。', '这件事的整体思路，以及可以怎样变化。'],
      ['朋友刚从一座陌生城市回来，你更自然会问什么？', '住在哪里、路好不好走、哪家店值得去。', '这座城市给人的感觉、生活方式有什么不同。'],
      ['同事介绍一个新产品，你首先会留意哪部分？', '现在有哪些功能，用起来具体是什么样。', '它想解决什么问题，以后还能往哪里发展。'],
      ['学做一道没试过的菜，你更喜欢怎样开始？', '先按配方和分量做一遍，熟悉后再调整。', '先弄懂味道怎么搭配，再试自己的组合。'],
      ['读完一篇长文章，要给朋友介绍。你更容易先记起什么？', '里面几个具体的事例和有用的事实。', '贯穿全文的观点，以及它带来的联想。'],
      ['整理一次活动的记录，你更想从哪里下笔？', '按发生的过程，留下时间、做法和细节。', '提炼几条线索，写出它们之间的联系。'],
      ['换了一款常用软件，第一次打开时你更想怎样探索？', '找到熟悉功能的位置，试着完成日常操作。', '先看看整体结构，想想有没有新的用法。'],
      ['散步经过一条旧街，你的注意力更容易停在哪里？', '店铺的陈设、建筑的纹理和沿途的声音。', '这里以前的生活，以及街区未来可能的样子。'],
      ['团队要改进一个已经能用的流程，你更想先讨论什么？', '哪一步最费时，有哪些可核对的实际问题。', '整个流程是否还有另一种组织方式。'],
      ['朋友教你使用一件新工具，哪种讲法更容易听进去？', '拿一件真实任务，从头操作给我看。', '先解释背后的原理，让我自己推演用法。'],
      ['挑选一本周末读物，你此刻更容易被哪种内容吸引？', '细致讲述一种生活或一门手艺的真实记录。', '提出新视角、让人联想到许多可能的随笔。'],
      ['复盘一次成功的尝试，你首先想弄明白什么？', '当时具体做对了哪些步骤，下次怎么复用。', '它反映了什么规律，还能迁移到哪些事情上。'],
    ],
    [
      ['两个方案都可行，但各有代价。你更习惯先比较什么？', '用同一组标准比较成本、效果和风险。', '看看各自会怎样影响相关的人和他们的需要。'],
      ['朋友给你看一份初稿，认真问你哪里能改。你通常先做什么？', '指出最影响效果的问题，并解释修改依据。', '先了解他想表达什么，再从他在意的地方谈起。'],
      ['小组对分工有不同意见，你更想先找什么来推进？', '把任务要求和每个人可承担的范围列清楚。', '听听大家各自的顾虑，找到能共同接受的安排。'],
      ['熟人请你破例调整已经约好的规则。你先会考虑什么？', '这次例外的理由是否也能适用于其他人。', '对方眼下的处境，以及这次调整对他的意义。'],
      ['一位同事最近做事不顺，来找你聊天。你自然的第一反应是？', '和他把问题拆开，看看哪些环节可以改变。', '先听他把感受说完，确认他最难受的是什么。'],
      ['要在两位水平相近的合作方中选一位，你更看重哪点？', '交付标准清楚，判断和做法有一致的依据。', '能理解彼此的在意之处，沟通时有相互照顾。'],
      ['讨论中有人反对你很喜欢的想法，你更容易先关注什么？', '他的论据能否成立，是否指出了真正的漏洞。', '他为什么会有这种感受，双方的关注点差在哪里。'],
      ['给一次小组活动写评价，你更想优先写清什么？', '目标完成了多少，哪些安排有效或无效。', '参与者的体验怎样，哪些地方让人感到被支持。'],
      ['你发现一项安排对多数人方便，却让一位伙伴很为难。你会先想？', '比较整体收益和额外代价，看看能否另加补救措施。', '了解他的具体难处，看看安排能否围绕它调整。'],
      ['和朋友意见不同，但今天需要做出决定。你更愿意怎样寻找共识？', '逐项检验各自的理由，找出更站得住的方案。', '说清彼此最看重的东西，找一个双方都愿意接受的方案。'],
      ['收到一封措辞直接、但指出了真实问题的反馈，你先注意到什么？', '里面哪些判断准确，接下来可以怎样改。', '对方传达的态度，以及这段合作中的感受。'],
      ['团队要取消一个投入过心血的项目，你更倾向先谈哪部分？', '继续投入的依据还在不在，资源怎样重新分配。', '大家如何看待这个结束，怎样安顿付出过的努力。'],
    ],
    [
      ['下个月准备去附近城市待两天，你现在更想做什么？', '先定好交通和住处，列出大致的每日安排。', '先留出日期，具体去哪儿到时再看兴趣和天气。'],
      ['一项任务还有一周截止，你更舒服的推进方式是？', '分好几段时间，尽早做出一个完整版本。', '先摸索不同思路，临近截止再集中收拢成形。'],
      ['周末没有必须处理的事，哪种状态让你更自在？', '早上就知道今天大致会做哪几件事。', '不急着安排，随着当时的心情决定。'],
      ['桌上有几件都想做的小事，你通常怎样开始？', '排一下先后，一件做完再切换到下一件。', '先从此刻最有兴致的一件做起，过程中灵活切换。'],
      ['朋友临时提议改掉你们原定的晚间安排，你更自然的反应是？', '先确认原计划如何调整，心里有数再换。', '看看新提议是否有意思，愿意顺势换个安排。'],
      ['要买一件不急用的生活用品，资料已经看得差不多了。你会？', '定下适合的款式，结束这件待办。', '再留一阵选择空间，也许会碰到更合意的。'],
      ['一个新项目刚启动，你希望接下来怎样配合？', '先约定几个明确节点，再沿着它们推进。', '先试一段，根据新发现随时调整推进方式。'],
      ['下班前还有一点时间，你更愿意怎样结束一天？', '收好未完的事，写下明天从哪一步接着做。', '顺着手头的状态多做一点，明天再决定先做什么。'],
      ['和朋友相约半天出游，你更喜欢哪种约法？', '时间、见面地点和主要去处先讲清楚。', '先碰面，去哪儿可以边走边商量。'],
      ['做一个个人小作品，你更享受哪种节奏？', '给它定个收尾日期，让它成为一件完成的作品。', '允许它持续变化，想到新东西就再添一点。'],
      ['搬家前两周，你更可能怎样准备？', '列好清单，分几次打包并确认各项预约。', '先处理最需要的几件，其余随实际进展安排。'],
      ['已经选定的方案又出现一种新可能，你通常更愿意？', '除非有明显好处，否则沿原方案做完再说。', '留一点余地，愿意重新打开选择比较一下。'],
    ],
  ]

  // Interleave dimensions; within every dimension A/B direction reverses each round.
  const questions = []
  for (let round = 0; round < 12; round++) {
    dimensions.forEach((dimension, index) => {
      const [prompt, first, second] = scenarios[index][round]
      const options = [{ text: first, pole: dimension.first }, { text: second, pole: dimension.second }]
      if ((round + index) % 2) options.reverse()
      questions.push({ id: dimension.key.toLowerCase() + '-' + (round + 1), dimension: index, prompt, options })
    })
  }

  function normalizeAnswers(answers) {
    return questions.map((_, index) => Array.isArray(answers) && (answers[index] === 0 || answers[index] === 1) ? answers[index] : null)
  }

  function score(answers) {
    const clean = normalizeAnswers(answers)
    const counts = dimensions.map(() => ({ first: 0, total: 0 }))
    questions.forEach((question, index) => {
      if (clean[index] === null) return
      const count = counts[question.dimension]
      count.total++
      if (question.options[clean[index]].pole === dimensions[question.dimension].first) count.first++
    })
    const percentages = counts.map(count => count.total ? Math.round(count.first * 100 / count.total) : null)
    const type = counts.every(count => count.total > 0)
      ? dimensions.map((dimension, index) => counts[index].first * 2 >= counts[index].total ? dimension.first : dimension.second).join('')
      : null
    return { type, percentages, totals: counts.map(count => count.total), answered: counts.reduce((sum, count) => sum + count.total, 0) }
  }

  function emptySave() {
    return { version: 1, progress: null, lastResult: null }
  }

  function normalizeSave(value) {
    const save = emptySave()
    if (!value || value.version !== 1) return save
    const progress = value.progress
    if (progress && Array.isArray(progress.answers)) {
      save.progress = {
        current: Number.isInteger(progress.current) ? Math.max(0, Math.min(47, progress.current)) : 0,
        answers: normalizeAnswers(progress.answers),
        mode: progress.mode === 'missing' ? 'missing' : 'all',
      }
    }
    const result = value.lastResult
    if (result && /^[EI][SN][TF][JP]$/.test(result.type) &&
      Array.isArray(result.percentages) && result.percentages.length === 4 &&
      result.percentages.every(percent => Number.isInteger(percent) && percent >= 0 && percent <= 100) &&
      Array.isArray(result.totals) && result.totals.length === 4 &&
      result.totals.every(total => Number.isInteger(total) && total >= 1 && total <= 12) &&
      typeof result.completedAt === 'string' && Number.isFinite(Date.parse(result.completedAt))) {
      const expectedType = dimensions.map((dimension, index) => result.percentages[index] >= 50 ? dimension.first : dimension.second).join('')
      if (expectedType === result.type) save.lastResult = {
        type: result.type, percentages: result.percentages.slice(), totals: result.totals.slice(), completedAt: result.completedAt,
      }
    }
    return save
  }

  globalThis.XiaobanMbtiRules = { dimensions, questions, normalizeAnswers, score, emptySave, normalizeSave }
})()
