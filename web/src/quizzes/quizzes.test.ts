import { beforeEach, describe, expect, it, vi } from 'vitest'
import { findQuiz } from './catalog.ts'
import { personalities } from './mbti.ts'
import { workModes } from './mbanti.ts'
import { clearAttempt, loadAttempt, newAttempt, saveAttempt } from './progress.ts'
import { isComplete, makeReport, scoreAnswers, validScores } from './scoring.ts'
import { readSharedScores, shareLink } from './share.ts'

const mbti = findQuiz('mbti')!
const mbanti = findQuiz('mbanti')!

describe('personality scoring', () => {
  it('offers forty questions with ten balanced items per axis', () => {
    expect(mbti.questions).toHaveLength(40)
    for (const axis of [0, 1, 2, 3]) {
      const items = mbti.questions.filter((question) => question.axis === axis)
      expect(items).toHaveLength(10)
      expect(items.filter((question) => question.direction === 1)).toHaveLength(5)
      expect(items.filter((question) => question.direction === -1)).toHaveLength(5)
    }
  })

  it.each(Object.keys(personalities))('produces %s from consistent preferences, including reversed items', (code) => {
    const positive = ['E', 'N', 'T', 'J']
    const answers = mbti.questions.map((question) => {
      const preferred = code[question.axis!] === positive[question.axis!] ? 1 : -1
      return preferred * question.direction! > 0 ? 5 : 1
    })
    const scores = scoreAnswers(mbti, answers)
    expect(makeReport(mbti, scores).code).toBe(code)
    const metrics = makeReport(mbti, scores).metrics
    expect(metrics[1].percent).toBe(code[1] === 'S' ? 100 : 0)
  })

  it('does not force neutral answers into a personality type', () => {
    const scores = scoreAnswers(mbti, Array(40).fill(3))
    expect(scores).toEqual([0, 0, 0, 0])
    expect(makeReport(mbti, scores).code).toBe('XXXX')
    expect(makeReport(mbti, [20, 0, -20, 0]).code).toBe('EXFX')
    expect(makeReport(mbti, scores).metrics.every((metric) => metric.percent === 50)).toBe(true)
  })

  it('rejects skipped, out-of-range or wrong-length answers', () => {
    expect(isComplete(mbti, Array(40).fill(null))).toBe(false)
    expect(() => scoreAnswers(mbti, Array(40).fill(null))).toThrow()
    expect(() => scoreAnswers(mbti, Array(40).fill(6))).toThrow()
    expect(() => scoreAnswers(mbti, Array(39).fill(3))).toThrow()
    expect(() => scoreAnswers(mbti, Array(40).fill('3'))).toThrow()
    expect(() => scoreAnswers(mbti, Array(40))).toThrow()
    expect(validScores(mbti, Array(4))).toBe(false)
  })
})

describe('M班TI', () => {
  it('has exactly 25 four-choice items and equally exposes all five modes', () => {
    expect(mbanti.questions).toHaveLength(25)
    for (const question of mbanti.questions) {
      expect(question.options).toHaveLength(4)
      expect(new Set(question.options.map((option) => option.value)).size).toBe(4)
    }
    for (const value of workModes.keys()) {
      expect(mbanti.questions.filter((question) => question.options.some((option) => option.value === value))).toHaveLength(20)
    }
  })

  it('counts only the chosen option and preserves ties', () => {
    const answers = mbanti.questions.map((question) => question.options[0].value)
    const scores = scoreAnswers(mbanti, answers)
    expect(scores.reduce((a, b) => a + b, 0)).toBe(25)
    expect(makeReport(mbanti, [10, 10, 3, 1, 1]).code).toBe('OFF / LOW')
    expect(makeReport(mbanti, [5, 5, 5, 5, 5]).code).toBe('MIX')
    expect(makeReport(mbanti, [5, 5, 5, 5, 5]).tags).toHaveLength(5)
    expect(makeReport(mbanti, [2, 3, 15, 1, 4]).title).toBe('快乐补给官')
  })

  it('rejects an answer not offered on that question', () => {
    const answers = mbanti.questions.map((question) => question.options[0].value)
    answers[0] = 0 // the first item intentionally has no OFF option
    expect(() => scoreAnswers(mbanti, answers)).toThrow()
  })
})

describe('shared report data', () => {
  it.each([{ quiz: mbti, scores: [20, -12, 0, 4] }, { quiz: mbanti, scores: [7, 7, 4, 3, 4] }])('round-trips aggregate scores for $quiz.id', ({ quiz, scores }) => {
    const url = new URL(shareLink(quiz, scores, 'https://xiaban.xyz'))
    expect(url.pathname).toBe(`/tests/${quiz.id}/report`)
    expect(url.search).toBe('')
    expect(readSharedScores(quiz, url.hash)).toEqual(scores)
    expect([...new URLSearchParams(url.hash.slice(1)).keys()]).toEqual(['v', 's'])
  })

  it.each(['', '#v=2&s=0,0,0,0', '#v=1&s=21,0,0,0', '#v=1&s=NaN,0,0,0', '#v=1&s=1.5,0,0,0', '#v=1&s=0,0', '#v=1&v=1&s=0,0,0,0', '#v=1&s=0,0,0,0&answers=1'])('rejects malformed personality fragments: %s', (fragment) => {
    expect(readSharedScores(mbti, fragment)).toBeNull()
  })

  it('rejects impossible workplace counts', () => {
    expect(validScores(mbanti, [5, 5, 5, 5, 4])).toBe(false)
    expect(validScores(mbanti, [-1, 10, 10, 3, 3])).toBe(false)
    expect(validScores(mbanti, [5.5, 4.5, 5, 5, 5])).toBe(false)
    expect(validScores(mbanti, [25, 0, 0, 0, 0])).toBe(false)
    expect(() => shareLink(mbanti, [0, 0, 0, 0, 0], 'https://xiaban.xyz')).toThrow()
    expect(() => makeReport(mbanti, [0, 0, 0, 0, 0])).toThrow()
  })
})

describe('local progress', () => {
  const values = new Map<string, string>()
  beforeEach(() => {
    values.clear()
    vi.stubGlobal('localStorage', {
      getItem: (key: string) => values.get(key) ?? null,
      setItem: (key: string, value: string) => values.set(key, value),
      removeItem: (key: string) => values.delete(key),
    })
  })

  it('resumes progress and clears only the selected quiz', () => {
    const attempt = newAttempt(mbti)
    attempt.answers[0] = 5
    attempt.cursor = 1
    saveAttempt(mbti, attempt)
    saveAttempt(mbanti, newAttempt(mbanti))
    expect(loadAttempt(mbti)).toEqual(attempt)
    clearAttempt(mbti)
    expect(loadAttempt(mbti)).toBeNull()
    expect(loadAttempt(mbanti)).not.toBeNull()
  })

  it('accepts completed results only with complete valid answers', () => {
    const complete = { ...newAttempt(mbti), answers: Array(40).fill(3), completedAt: 123 }
    saveAttempt(mbti, complete)
    expect(loadAttempt(mbti)?.completedAt).toBe(123)
    complete.answers[4] = null
    saveAttempt(mbti, complete)
    expect(loadAttempt(mbti)?.completedAt).toBeNull()
  })

  it('ignores outdated or corrupted progress', () => {
    values.set('xb_quiz_mbti', '{broken')
    expect(loadAttempt(mbti)).toBeNull()
    saveAttempt(mbti, { ...newAttempt(mbti), version: 0 })
    expect(loadAttempt(mbti)).toBeNull()
    saveAttempt(mbti, { ...newAttempt(mbti), answers: [5] })
    expect(loadAttempt(mbti)).toBeNull()
  })
})
