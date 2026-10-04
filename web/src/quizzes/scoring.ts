import { personalities } from './mbti.ts'
import { workModes, workProfiles } from './mbanti.ts'
import type { QuizDefinition, QuizReport, ReportMetric } from './types.ts'

export function validAnswers(quiz: QuizDefinition, answers: unknown): answers is (number | null)[] {
  return Array.isArray(answers) && answers.length === quiz.questions.length &&
    quiz.questions.every((question, i) => answers[i] === null || question.options.some((option) => option.value === answers[i]))
}

export function isComplete(quiz: QuizDefinition, answers: unknown): answers is number[] {
  return validAnswers(quiz, answers) && answers.every((answer) => answer !== null)
}

export function scoreAnswers(quiz: QuizDefinition, answers: unknown): number[] {
  if (!isComplete(quiz, answers)) throw new Error('请先完成所有题目，再查看报告。')
  if (quiz.id === 'mbanti') {
    const counts = [0, 0, 0, 0, 0]
    answers.forEach((answer) => { counts[answer] += 1 })
    return counts
  }
  const scores = [0, 0, 0, 0]
  quiz.questions.forEach((question, i) => {
    if (question.axis === undefined || question.direction === undefined) throw new Error('题目配置不完整。')
    scores[question.axis] += (answers[i] - 3) * question.direction
  })
  return scores
}

export function validScores(quiz: QuizDefinition, scores: unknown): scores is number[] {
  if (!Array.isArray(scores) || !Array.from(scores).every(Number.isInteger)) return false
  if (quiz.id === 'mbti') return scores.length === 4 && scores.every((score) => score >= -20 && score <= 20)
  return scores.length === 5 && scores.every((score) => score >= 0 && score <= 20) && scores.reduce((a, b) => a + b, 0) === 25
}

const axes = [
  { label: '能量来源', positive: 'E', negative: 'I', left: 'E · 外向', right: 'I · 内向',
    positiveText: '与人互动更容易让你获得能量', negativeText: '安静独处更容易让你恢复能量' },
  { label: '信息关注', positive: 'N', negative: 'S', left: 'S · 实感', right: 'N · 直觉',
    positiveText: '你更容易关注联系、原理和新的可能', negativeText: '你更容易关注实际经验和具体细节' },
  { label: '决定方式', positive: 'T', negative: 'F', left: 'T · 思考', right: 'F · 情感',
    positiveText: '做决定时，你更常先比较逻辑和一致的标准', negativeText: '做决定时，你更常先考虑价值和人的感受' },
  { label: '生活节奏', positive: 'J', negative: 'P', left: 'J · 判断', right: 'P · 知觉',
    positiveText: '明确的计划和结论更容易让你安心', negativeText: '保留选择、灵活探索更容易让你自在' },
] as const

export function personalityCode(scores: readonly number[]): string {
  return axes.map((axis, i) => scores[i] === 0 ? 'X' : scores[i] > 0 ? axis.positive : axis.negative).join('')
}

function personalityReport(quiz: QuizDefinition, scores: number[]): QuizReport {
  const code = personalityCode(scores)
  const profile = personalities[code]
  const metrics: ReportMetric[] = axes.map((axis, i) => {
    const score = scores[i]
    const tendency = score > 0 ? axis.positiveText : axis.negativeText
    const strength = Math.abs(score) <= 4 ? '两边很接近，情境可能影响你的选择' : Math.abs(score) >= 12 ? '本次偏好较明显' : '本次有一定偏好'
    return {
      label: axis.label, left: axis.left, right: axis.right,
      percent: Math.round(50 + (i === 1 ? -score : score) * 2.5),
      note: score === 0 ? '两边平衡，暂用 X 表示，不强行归类。' : `${tendency}；${strength}。`,
    }
  })
  const near = scores.some((score) => Math.abs(score) <= 4)
  return {
    quizId: quiz.id, code,
    title: profile?.title ?? '随情境切换的多面派',
    quote: profile?.quote ?? '你可以有很多面，不必急着把自己装进四个字母。',
    description: profile?.description ?? '你有一些维度恰好平衡，所以用 X 标记未确定的偏好。你可能会按场景切换，也可能还需要更多生活经验来观察自己。下面的维度比一个固定标签更值得参考。',
    tags: profile?.tags ?? ['保留多面', '随境切换', '继续观察'],
    metrics,
    sections: [
      { title: '你带来的力量', text: profile?.strength ?? '你不需要在每组偏好里选定一个永久位置。能观察情境、接纳不同的做法，也是你了解自己的起点。' },
      { title: '什么会悄悄耗电', text: profile?.drain ?? '试图符合一个固定的人设，可能比事情本身更累。留意真实的疲惫信号，再决定今天需要热闹、安静、计划还是自由。' },
      { title: '让相处更舒服一点', text: profile?.connection ?? '告诉对方你当下需要什么，比让他们根据一个类型猜测更准确。关系里的你，可以有变化。' },
    ],
    recharge: { title: '今晚，按自己的方式充电', text: profile?.recharge ?? '挑一件做完会觉得轻松的小事，散步、听歌、独处或短聚都可以。感受之后的电量，再慢慢发现自己的节奏。' },
    caveat: `${near ? '你有接近平衡的维度，类型可能随情境或再次作答变化。' : ''}百分比是本次答案的归一化偏好分，不是概率、准确率或人群排名。${quiz.caveat}`,
  }
}

function workReport(quiz: QuizDefinition, counts: number[]): QuizReport {
  const max = Math.max(...counts)
  const leaders = workModes.filter((_, i) => counts[i] === max)
  const profiles = leaders.map((mode) => workProfiles[mode])
  const solo = profiles.length === 1 ? profiles[0] : null
  return {
    quizId: quiz.id,
    code: leaders.length > 2 ? 'MIX' : leaders.join(' / '),
    title: solo?.title ?? (leaders.length === 2 ? '职场双修选手' : '职场多面选手'),
    quote: solo?.quote ?? '你不止一种生存技能，生活也不止一种打开方式。',
    description: solo?.description ?? `${profiles.map((profile) => profile.title).join('、')}并列第一。你会按不同场景切换续航方式，这次不强行选一个冠军。`,
    tags: solo?.tags ?? profiles.map((profile) => profile.title),
    metrics: workModes.map((mode, i) => ({
      label: workProfiles[mode].title, left: mode, percent: counts[i] * 4,
      note: `${counts[i]} / 25 次选择 · ${workProfiles[mode].tags[0]}`,
    })),
    sections: [
      { title: '你的职场生存技能', text: solo?.strength ?? profiles.map((profile) => `${profile.title}：${profile.strength}`).join('\n\n') },
      { title: '最容易掉电的时刻', text: solo?.drain ?? profiles.map((profile) => `${profile.title}：${profile.drain}`).join('\n\n') },
      { title: '给明天留一句好用的话', text: solo?.connection ?? '“我们先确认最重要的一件，我把它处理好，再继续下一件。”不用一次用上所有技能，给今天选一个够用的就好。' },
    ],
    recharge: { title: '今晚的下班处方签', text: solo?.recharge ?? '从今天的多种技能里选一个：关掉通知、安静十分钟、吃点喜欢的、笑完换个话题，或放下优化冲动。只做一件就好。' },
    caveat: `比例只表示你在这 25 题中选择各类答案的次数，不是人格强度或压力诊断。${quiz.caveat}`,
  }
}

export function makeReport(quiz: QuizDefinition, scores: unknown): QuizReport {
  if (!validScores(quiz, scores)) throw new Error('报告数据不完整，请重新测试。')
  return quiz.id === 'mbti' ? personalityReport(quiz, scores) : workReport(quiz, scores)
}
