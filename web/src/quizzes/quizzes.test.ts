import { beforeEach, describe, expect, it, vi } from 'vitest'
import { findQuiz, findSharedQuiz, quizCatalog } from './catalog.ts'
import { personalities } from './mbti.ts'
import { workModes } from './mbanti.ts'
import { clearAttempt, loadAttempt, newAttempt, saveAttempt } from './progress.ts'
import { isComplete, makeReport, scoreAnswers, validScores } from './scoring.ts'
import { readSharedScores, shareLink } from './share.ts'

const mbti = findQuiz('mbti')!
const mbanti = findQuiz('mbanti')!
const legacyMbanti = findSharedQuiz('mbanti', '1')!
const personaQuizzes = quizCatalog.filter((quiz) => quiz.persona)

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
  it('has exactly 20 four-choice items and equally exposes all five modes', () => {
    expect(mbanti.questions).toHaveLength(20)
    expect(mbanti.version).toBe(2)
    expect(new Set(mbanti.questions.map((question) => question.id)).size).toBe(20)
    for (const question of mbanti.questions) {
      expect(question.options).toHaveLength(4)
      expect(new Set(question.options.map((option) => option.value)).size).toBe(4)
    }
    for (const value of workModes.keys()) {
      expect(mbanti.questions.filter((question) => question.options.some((option) => option.value === value))).toHaveLength(16)
    }
  })

  it('counts only the chosen option and preserves ties', () => {
    const answers = mbanti.questions.map((question) => question.options[0].value)
    const scores = scoreAnswers(mbanti, answers)
    expect(scores.reduce((a, b) => a + b, 0)).toBe(20)
    expect(makeReport(mbanti, [8, 8, 2, 1, 1]).code).toBe('OFF / LOW')
    expect(makeReport(mbanti, [4, 4, 4, 4, 4]).code).toBe('MIX')
    expect(makeReport(mbanti, [4, 4, 4, 4, 4]).tags).toHaveLength(5)
    const report = makeReport(mbanti, [1, 2, 14, 1, 2])
    expect(report.title).toBe('快乐补给官')
    expect(report.metrics[2].percent).toBe(70)
    expect(report.metrics[2].note).toContain('14 / 20')
  })

  it('rejects an answer not offered on that question', () => {
    const answers = mbanti.questions.map((question) => question.options[0].value)
    answers[0] = 0 // the first item intentionally has no OFF option
    expect(() => scoreAnswers(mbanti, answers)).toThrow()
  })
})

describe.each(personaQuizzes)('$name', (quiz) => {
  it('has twelve original items balanced across three axes, with rotated answer positions', () => {
    expect(quiz.questions).toHaveLength(12)
    expect(new Set(quiz.questions.map((question) => question.id)).size).toBe(12)
    for (const axis of [0, 1, 2]) expect(quiz.questions.filter((question) => question.axis === axis)).toHaveLength(4)
    for (const question of quiz.questions) {
      expect(question.source).toBe('original')
      expect(question.options.map((option) => option.value).sort((a, b) => a - b)).toEqual([-2, -1, 1, 2])
    }
    expect(quiz.questions[0].options[0].value).not.toBe(quiz.questions[1].options[0].value)
  })

  it.each(quiz.persona!.profiles)('produces $code with a complete illustrated persona report', (profile) => {
    const answers = quiz.questions.map((question) => profile.pattern[question.axis!] === '1' ? 2 : -2)
    const scores = scoreAnswers(quiz, answers)
    const report = makeReport(quiz, scores)
    expect(scores).toEqual(Array.from(profile.pattern, (bit) => bit === '1' ? 8 : -8))
    expect(report.code).toBe(profile.code)
    expect(report.title).toBe(profile.title)
    expect(report.tags).toHaveLength(3)
    expect(report.sections.every((section) => section.text.length > 20)).toBe(true)
    expect(report.recharge.text.length).toBeGreaterThan(20)
    expect(report.metrics.map((metric) => metric.percent)).toEqual(Array.from(profile.pattern, (bit) => bit === '1' ? 100 : 0))
  })

  it('keeps balanced axes open and explains weak preferences', () => {
    const answers = quiz.questions.map((_, i) => Math.floor(i / 3) % 2 ? 2 : -2)
    const scores = scoreAnswers(quiz, answers)
    expect(scores).toEqual([0, 0, 0])
    expect(makeReport(quiz, scores).code).toBe('MIX')
    expect(makeReport(quiz, [0, 8, -8]).title).toBe(quiz.persona!.mixed.title)
    expect(makeReport(quiz, scores).metrics.every((metric) => metric.percent === 50)).toBe(true)
    expect(makeReport(quiz, [1, -1, 2]).metrics.every((metric) => metric.note.includes('两边较接近'))).toBe(true)
  })

  it('rejects incomplete answers and impossible aggregate scores', () => {
    for (const answers of [Array(11).fill(2), Array(12).fill(0), Array(12).fill(null), Array(12), Array(12).fill('2')]) {
      expect(isComplete(quiz, answers)).toBe(false)
      expect(() => scoreAnswers(quiz, answers)).toThrow()
    }
    for (const scores of [[9, 0, 0], [-9, 0, 0], [1.5, 0, 0], [0, 0], [0, 0, 0, 0], Array(3)]) {
      expect(validScores(quiz, scores)).toBe(false)
      expect(() => makeReport(quiz, scores)).toThrow()
    }
    expect(readSharedScores(quiz, '#v=1&s=9,0,0')).toBeNull()
    expect(readSharedScores(quiz, '#v=2&s=0,0,0')).toBeNull()
  })
})

describe('shared report data', () => {
  it.each([
    { quiz: mbti, scores: [20, -12, 0, 4] }, { quiz: mbanti, scores: [6, 6, 3, 2, 3] },
    ...personaQuizzes.flatMap((quiz) => [{ quiz, scores: [8, -4, 0] }, { quiz, scores: [0, 0, 0] }]),
  ])('round-trips aggregate scores for $quiz.id', ({ quiz, scores }) => {
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
    expect(validScores(mbanti, [17, 1, 1, 1, 0])).toBe(false)
    expect(() => shareLink(mbanti, [0, 0, 0, 0, 0], 'https://xiaban.xyz')).toThrow()
    expect(() => makeReport(mbanti, [0, 0, 0, 0, 0])).toThrow()
  })

  it('preserves the interpretation of previously shared 25-question reports', () => {
    const scores = [10, 10, 3, 1, 1]
    expect(legacyMbanti.questions).toHaveLength(25)
    const report = makeReport(legacyMbanti, scores)
    expect(report.code).toBe('OFF / LOW')
    expect(report.metrics[0].percent).toBe(40)
    expect(report.metrics[0].note).toContain('10 / 25')
    const fragment = new URL(shareLink(legacyMbanti, scores, 'https://xiaban.xyz')).hash
    expect(readSharedScores(legacyMbanti, fragment)).toEqual(scores)
    expect(readSharedScores(mbanti, fragment)).toBeNull()
    expect(findSharedQuiz('mbanti', '2')).toBe(mbanti)
    expect(findSharedQuiz('mbanti', '3')).toBeUndefined()
    expect(findSharedQuiz('animal', '2')).toBeUndefined()
    expect(findSharedQuiz('missing', '1')).toBeUndefined()
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

  it('starts a fresh 20-question attempt instead of reinterpreting old workplace answers', () => {
    const oldAttempt = { ...newAttempt(legacyMbanti), answers: legacyMbanti.questions.map((question) => question.options[0].value), completedAt: 123 }
    saveAttempt(legacyMbanti, oldAttempt)
    saveAttempt(mbti, newAttempt(mbti))
    expect(loadAttempt(mbanti)).toBeNull()
    expect(newAttempt(mbanti).answers).toHaveLength(20)
    expect(loadAttempt(mbti)).not.toBeNull()
  })

  it.each(personaQuizzes)('resumes $name independently from other tests', (quiz) => {
    const attempt = newAttempt(quiz)
    attempt.answers[0] = 2
    attempt.cursor = 1
    saveAttempt(quiz, attempt)
    saveAttempt(mbti, newAttempt(mbti))
    expect(loadAttempt(quiz)).toEqual(attempt)
    clearAttempt(mbti)
    expect(loadAttempt(quiz)).toEqual(attempt)
  })
})
