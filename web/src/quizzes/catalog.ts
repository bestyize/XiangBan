import { mbtiQuestions } from './mbti.ts'
import { legacyMbantiQuestions, mbantiQuestions } from './mbanti.ts'
import { animalPersona, animalQuestions } from './animal.ts'
import { afterhoursPersona, afterhoursQuestions } from './afterhours.ts'
import type { QuizDefinition } from './types.ts'

export const quizCatalog: readonly QuizDefinition[] = [
  {
    id: 'mbti', version: 1, name: 'MBTI', subtitle: '看看你的性格偏好',
    description: '从日常里的选择，认识你的能量、思考方式和相处节奏。',
    duration: '约 6–8 分钟',
    instructions: '按平时最自然的状态选择，不用选“理想中的自己”。没有好坏答案；拿不准时可以选“一半一半”。',
    caveat: '这是 MBTI 风格的自我探索，采用部分 IPIP 公共领域条目改写及原创题，并非官方 MBTI 量表，未做心理测量验证。结果仅作参考，不用于诊断、招聘或能力判断。',
    category: '认识自己', eyebrow: '性格小记', reportLabel: '性格偏好报告',
    metricsTitle: '你的四组性格偏好', metricsIntro: '偏好没有高低。两边接近时，把更多空间留给情境。',
    shortCaveat: 'MBTI 风格探索 · 非官方量表 · 偏好没有高低',
    questions: mbtiQuestions,
  },
  {
    id: 'mbanti', version: 2, name: 'M班TI', subtitle: '你的打工人精神状态',
    description: '给职场离谱时刻配一句内心弹幕，看看你靠什么续航。',
    duration: '约 3–4 分钟',
    instructions: '每题选最像你的那一句。内心弹幕也算！这份测试不考绩效，只负责给紧绷的一天松个绑。',
    caveat: '20 道原创职场趣味题，结果是娱乐画像，不评价工作表现，也不代表心理健康状况。笑完记得把自己接回生活。',
    category: '职场松绑', eyebrow: '打工续航笔记', reportLabel: '职场续航报告',
    metricsTitle: '你的职场续航配方', metricsIntro: '每一种生存技能都有用，比例来自你这次的 20 个选择。',
    shortCaveat: '原创职场娱乐画像 · 不评价工作表现或心理状态',
    questions: mbantiQuestions,
  },
  {
    id: 'animal', version: 1, name: '精神小动物', subtitle: '你的内心住着哪只小动物？',
    description: '从相处距离、生活步速和好奇心，找到一只和你同频的小动物。',
    duration: '约 1–2 分钟',
    instructions: '每题选最像平时自己的状态。不必一直可爱、一直活力满格，也不用选别人期待你成为的样子。',
    caveat: '12 道原创生活情境题，小动物是趣味比喻，不是科学人格分类或心理测评。比例只表示本次选择呈现的偏好。',
    category: '生活共鸣', eyebrow: '住在心里的小动物', reportLabel: '小动物名片',
    metricsTitle: '你的小动物生活配方', metricsIntro: '没有更好的小动物。三个小偏好，拼出此刻舒服的生活方式。',
    shortCaveat: '原创生活趣味测试 · 小动物是比喻，结果可随情境变化',
    persona: animalPersona, questions: animalQuestions,
  },
  {
    id: 'afterhours', version: 1, name: '下班第二人格', subtitle: '关掉工作，你会切换成谁？',
    description: '白天有工作角色，晚上有自己的频道。看看你怎样把时间还给生活。',
    duration: '约 1–2 分钟',
    instructions: '想象一个不用加班、没有必须完成任务的晚上，选最让你自在的答案。学习或其他忙碌结束后，也可以这样想。',
    caveat: '12 道原创下班情境题，“第二人格”是生活状态的趣味说法，不是临床概念或心理诊断。偏好会随时间和情境变化。',
    category: '下班回血', eyebrow: '下班以后，听你的', reportLabel: '下班生活名片',
    metricsTitle: '你的晚间充电配方', metricsIntro: '独处、相聚、放空、动手都可以是休息。今晚按自己的节奏来。',
    shortCaveat: '原创下班趣味测试 · 描述生活偏好，不是心理诊断',
    persona: afterhoursPersona, questions: afterhoursQuestions,
  },
]

// Previously shared 25-question reports keep their original interpretation; new attempts use v2.
const legacyMbanti: QuizDefinition = {
  ...quizCatalog[1], version: 1, questions: legacyMbantiQuestions,
  caveat: '25 道原创职场趣味题的旧版报告，结果仅用于娱乐，不评价工作表现或心理状态。',
  metricsIntro: '这是 25 题旧版结果，比例来自当时的 25 个选择；新测试已精简到 20 题。',
}

export function findSharedQuiz(id: string, version: string | null): QuizDefinition | undefined {
  if (id === 'mbanti' && version === '1') return legacyMbanti
  const quiz = findQuiz(id)
  return quiz && version === String(quiz.version) ? quiz : undefined
}

export function findQuiz(id: string): QuizDefinition | undefined {
  return quizCatalog.find((quiz) => quiz.id === id)
}
