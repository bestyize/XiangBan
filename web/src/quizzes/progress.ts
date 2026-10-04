import { storage } from '../lib/storage.ts'
import { isComplete, validAnswers } from './scoring.ts'
import type { QuizAttempt, QuizDefinition } from './types.ts'

const key = (quiz: QuizDefinition) => `xb_quiz_${quiz.id}`

export function newAttempt(quiz: QuizDefinition): QuizAttempt {
  return { version: quiz.version, answers: quiz.questions.map(() => null), cursor: 0, completedAt: null }
}

export function loadAttempt(quiz: QuizDefinition): QuizAttempt | null {
  try {
    const raw: unknown = JSON.parse(storage.get(key(quiz)) ?? 'null')
    if (!raw || typeof raw !== 'object') return null
    const value = raw as Record<string, unknown>
    if (value.version !== quiz.version || !validAnswers(quiz, value.answers)) return null
    const cursor = typeof value.cursor === 'number' && Number.isInteger(value.cursor) && value.cursor >= 0 && value.cursor < quiz.questions.length ? value.cursor : 0
    const completedAt = typeof value.completedAt === 'number' && Number.isFinite(value.completedAt) && value.completedAt > 0 && isComplete(quiz, value.answers) ? value.completedAt : null
    return { version: quiz.version, answers: value.answers, cursor, completedAt }
  } catch {
    return null
  }
}

export function saveAttempt(quiz: QuizDefinition, attempt: QuizAttempt) {
  storage.set(key(quiz), JSON.stringify(attempt))
}

export function clearAttempt(quiz: QuizDefinition) {
  storage.remove(key(quiz))
}
