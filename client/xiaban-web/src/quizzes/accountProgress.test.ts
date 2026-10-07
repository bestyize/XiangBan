import { beforeEach, describe, expect, it, vi } from 'vitest'
import { setToken } from '../api/client.ts'
import { findQuiz } from './catalog.ts'
import { applyRemoteAttempt, clearAttempt, importGuestAttempts, loadAttempt, newAttempt, readSavedAttempt, saveAttempt } from './progress.ts'

const quiz = findQuiz('animal')!
const token = (id: number) => `header.${btoa(JSON.stringify({ sub: String(id), account_kind: 'user' }))}.signature`

describe('account progress isolation and reconciliation', () => {
  beforeEach(() => {
    const values = new Map<string, string>()
    vi.stubGlobal('localStorage', { getItem: (key: string) => values.get(key) ?? null, setItem: (key: string, value: string) => values.set(key, value), removeItem: (key: string) => values.delete(key) })
    setToken(null)
  })
  it('keeps guest and different account answers separate', () => {
    const guest = newAttempt(quiz); guest.answers[0] = 2; saveAttempt(quiz, guest)
    setToken(token(10)); expect(loadAttempt(quiz)).toBeNull()
    const registered = newAttempt(quiz); registered.answers[0] = -2; saveAttempt(quiz, registered)
    setToken(token(11)); expect(loadAttempt(quiz)).toBeNull()
    setToken(token(10)); expect(loadAttempt(quiz)?.answers[0]).toBe(-2)
    setToken(null); expect(loadAttempt(quiz)?.answers[0]).toBe(2)
  })
  it('imports selected guest progress without replacing a newer account record', () => {
    saveAttempt(quiz, { ...newAttempt(quiz), answers: Array(12).fill(2), completedAt: 1 })
    importGuestAttempts([quiz], 10)
    setToken(token(10)); expect(loadAttempt(quiz)?.completedAt).toBe(1)
    const newer = { ...newAttempt(quiz), updatedAt: Date.now() + 1000 }
    applyRemoteAttempt(quiz, newer, 10)
    importGuestAttempts([quiz], 10)
    expect(loadAttempt(quiz)?.completedAt).toBeNull()
    expect(applyRemoteAttempt(quiz, { ...newer, updatedAt: 1 }, 10)).toBe(false)
  })
  it('stores a timestamped reset so restarting cannot restore an older cloud report', () => {
    setToken(token(10))
    saveAttempt(quiz, { ...newAttempt(quiz), answers: Array(12).fill(2), completedAt: 1 })
    const before = readSavedAttempt(quiz)!
    clearAttempt(quiz)
    expect(loadAttempt(quiz)?.answers).toEqual(Array(12).fill(null))
    expect(readSavedAttempt(quiz)!.updatedAt).toBeGreaterThan(before.updatedAt)
    expect(applyRemoteAttempt(quiz, before, 10)).toBe(false)
  })
  it('retains a valid save timestamp when importing a completed legacy report', () => {
    localStorage.setItem('xb_quiz_animal', JSON.stringify({ ...newAttempt(quiz), answers: Array(12).fill(2), completedAt: 123 }))
    importGuestAttempts([quiz], 10)
    expect(readSavedAttempt(quiz, 10)?.updatedAt).toBe(123)
  })
})
