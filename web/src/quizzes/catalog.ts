import { mbtiQuestions } from './mbti.ts'
import { mbantiQuestions } from './mbanti.ts'
import type { QuizDefinition } from './types.ts'

export const quizCatalog: readonly QuizDefinition[] = [
  {
    id: 'mbti', version: 1, name: 'MBTI', subtitle: '看看你的性格偏好',
    description: '从日常里的选择，认识你的能量、思考方式和相处节奏。',
    duration: '约 6–8 分钟',
    instructions: '按平时最自然的状态选择，不用选“理想中的自己”。没有好坏答案；拿不准时可以选“一半一半”。',
    caveat: '这是 MBTI 风格的自我探索，采用部分 IPIP 公共领域条目改写及原创题，并非官方 MBTI 量表，未做心理测量验证。结果仅作参考，不用于诊断、招聘或能力判断。',
    questions: mbtiQuestions,
  },
  {
    id: 'mbanti', version: 1, name: 'M班TI', subtitle: '你的打工人精神状态',
    description: '给职场离谱时刻配一句内心弹幕，看看你靠什么续航。',
    duration: '约 4–5 分钟',
    instructions: '每题选最像你的那一句。内心弹幕也算！这份测试不考绩效，只负责给紧绷的一天松个绑。',
    caveat: '25 道原创职场趣味题，结果是娱乐画像，不评价工作表现，也不代表心理健康状况。笑完记得把自己接回生活。',
    questions: mbantiQuestions,
  },
]

export function findQuiz(id: string): QuizDefinition | undefined {
  return quizCatalog.find((quiz) => quiz.id === id)
}
