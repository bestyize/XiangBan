import { getToken } from '../api/client.ts'
import { storage } from '../lib/storage.ts'
import { isComplete, validAnswers } from './scoring.ts'
import type { QuizAttempt, QuizDefinition } from './types.ts'

export function quizAccountId(): number | null {
  try {
    const part = getToken()?.split('.')[1]
    if (!part) return null
    const claims = JSON.parse(atob(part.replace(/-/g, '+').replace(/_/g, '/')))
    const id = Number(claims.sub)
    return claims.account_kind === 'user' && Number.isSafeInteger(id) && id > 0 ? id : null
  } catch { return null }
}
export const attemptKey = (quiz: QuizDefinition, owner = quizAccountId()) => owner ? `xb_quiz_user_${owner}_${quiz.id}` : `xb_quiz_${quiz.id}`
export interface SavedAttempt extends QuizAttempt { updatedAt: number }
const listeners = new Set<(quiz: QuizDefinition, attempt: SavedAttempt, owner: number | null) => void>()
export function onAttemptSaved(listener: (quiz: QuizDefinition, attempt: SavedAttempt, owner: number | null) => void) {
  listeners.add(listener); return () => { listeners.delete(listener) }
}
export function readSavedAttempt(quiz: QuizDefinition, owner = quizAccountId()): SavedAttempt | null {
  try {
    const value = JSON.parse(storage.get(attemptKey(quiz, owner)) ?? 'null')
    if (!value || value.version !== quiz.version || !validAnswers(quiz, value.answers)) return null
    const legacyTime = Number.isSafeInteger(value.completedAt) && value.completedAt > 0 ? value.completedAt : 1
    return { ...value, updatedAt: Number.isSafeInteger(value.updatedAt) && value.updatedAt > 0 ? value.updatedAt : legacyTime }
  } catch { return null }
}
export function applyRemoteAttempt(quiz: QuizDefinition, attempt: SavedAttempt, owner: number) {
  const local = readSavedAttempt(quiz, owner)
  if (!local || attempt.updatedAt > local.updatedAt) {
    storage.set(attemptKey(quiz, owner), JSON.stringify(attempt)); return true
  }
  return false
}
export function importGuestAttempts(quizzes: readonly QuizDefinition[], owner: number) {
  for (const quiz of quizzes) {
    const guest = readSavedAttempt(quiz, null)
    if (guest && applyRemoteAttempt(quiz, guest, owner)) {
      listeners.forEach((listener) => listener(quiz, guest, owner))
    }
  }
}

export function newAttempt(quiz: QuizDefinition): QuizAttempt {
  return { version: quiz.version, answers: quiz.questions.map(() => null), cursor: 0, completedAt: null }
}

export function loadAttempt(quiz: QuizDefinition): QuizAttempt | null {
  try {
    const raw: unknown = JSON.parse(storage.get(attemptKey(quiz)) ?? 'null')
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
  const owner = quizAccountId()
  const previous = readSavedAttempt(quiz, owner)
  const saved = { ...attempt, updatedAt: Math.max(Date.now(), (previous?.updatedAt ?? 0) + 1) }
  storage.set(attemptKey(quiz, owner), JSON.stringify(saved))
  listeners.forEach((listener) => listener(quiz, saved, owner))
}

export function clearAttempt(quiz: QuizDefinition) {
  saveAttempt(quiz, newAttempt(quiz))
  if (quizAccountId() === null) storage.remove(attemptKey(quiz))
}
