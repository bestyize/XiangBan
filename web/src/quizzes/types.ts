export type QuizId = 'mbti' | 'mbanti' | 'animal' | 'afterhours'
export type Axis = 0 | 1 | 2 | 3

export interface QuizOption {
  value: number
  label: string
}

export interface QuizQuestion {
  id: string
  prompt: string
  options: readonly QuizOption[]
  axis?: Axis
  direction?: 1 | -1
  source: 'ipip-adapted' | 'original'
}

export interface QuizDefinition {
  id: QuizId
  version: number
  name: string
  subtitle: string
  description: string
  duration: string
  instructions: string
  caveat: string
  category: string
  eyebrow: string
  reportLabel: string
  metricsTitle: string
  metricsIntro: string
  shortCaveat: string
  persona?: PersonaDefinition
  questions: readonly QuizQuestion[]
}

export interface PersonaProfile {
  pattern: string
  code: string
  title: string
  quote: string
  description: string
  tags: readonly string[]
  strength: string
  drain: string
  connection: string
  recharge: string
}

export interface PersonaDefinition {
  axes: readonly { label: string; left: string; right: string }[]
  profiles: readonly PersonaProfile[]
  mixed: { title: string; quote: string; description: string }
}

export interface QuizAttempt {
  version: number
  answers: (number | null)[]
  cursor: number
  completedAt: number | null
}

export interface ReportMetric {
  label: string
  left: string
  right?: string
  percent: number
  note: string
}

export interface QuizReport {
  quizId: QuizId
  code: string
  title: string
  quote: string
  description: string
  tags: readonly string[]
  metrics: ReportMetric[]
  sections: { title: string; text: string }[]
  recharge: { title: string; text: string }
  caveat: string
}
