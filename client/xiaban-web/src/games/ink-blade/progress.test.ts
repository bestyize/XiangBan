import { readFileSync } from 'node:fs'
import { runInNewContext } from 'node:vm'
import { describe, expect, it } from 'vitest'

/* eslint-disable @typescript-eslint/no-explicit-any */
type Any = any
const context: Record<string, Any> = {}
for (const file of ['levels.js', 'progress.js']) runInNewContext(readFileSync(new URL(`../../../public/game-packs/ink-blade/${file}`, import.meta.url), 'utf8'), context)
const P = context.XiabanInkProgress
const clear = (level: string, score: number, seals: number, grade: number, seconds = 40) => ({ level, cleared: true, score, seals, grade, seconds })

describe('ink blade progress', () => {
  it('opens 幅 one by one and the 长卷 only after the 终幅', () => {
    let save = P.empty()
    expect(['1-1', '1-2', '1-scroll'].map((id) => P.unlocked(save, id))).toEqual([true, false, false])
    save = P.record(save, clear('1-1', 1800, 1, 1)).save
    expect(P.unlocked(save, '1-2')).toBe(true)
    expect(P.unlocked(save, '1-3')).toBe(false)
    for (const id of ['1-2', '1-3', '1-4', '1-5']) save = P.record(save, clear(id, 3000, 1, 1)).save
    expect(P.unlocked(save, '1-scroll')).toBe(false)
    save = P.record(save, clear('1-6', 5000, 1, 1)).save
    expect(P.unlocked(save, '1-scroll')).toBe(true)
    expect(P.next('1-5')).toBe('1-6')
    expect(P.next('1-6')).toBeNull()
    expect(P.next('1-scroll')).toBeNull()
  })

  it('keeps the best of every field and reports only newly earned seals', () => {
    let outcome = P.record(P.empty(), clear('1-2', 2400, 1 | 2, 2, 60))
    expect(outcome).toMatchObject({ firstClear: true, gained: 3, newBest: true, newGrade: true })
    outcome = P.record(outcome.save, clear('1-2', 2100, 1 | 4, 1, 45))
    expect(outcome).toMatchObject({ firstClear: false, gained: 4, newBest: false, newGrade: false })
    expect(outcome.save.levels['1-2']).toEqual({ clears: 2, best: 2400, grade: 2, seals: 7, time: 45 })
    expect(P.sealCount(outcome.save, 1)).toBe(3)
    expect(P.maxSeals(1)).toBe(21)
  })

  it('ignores failed rounds and unknown levels', () => {
    const save = P.empty()
    expect(P.record(save, { level: '1-1', cleared: false, score: 900, seals: 0, grade: 0, seconds: 30 }).save).toBe(save)
    expect(P.record(save, clear('9-9', 900, 7, 4)).save).toBe(save)
  })

  it('rebuilds a clean save from untrusted data', () => {
    const save = P.normalize({
      v: 1, difficulty: 'easy', prompts: ['move', 'move', 'Bad Key!', 7, 'jump'], extra: { evil: true },
      levels: { '1-1': { clears: 2, best: 1900, grade: 9, seals: 15, time: -3 }, '1-2': { clears: 0, best: 5 }, '1-9': { clears: 1 }, toString: 1 },
    })
    expect(save).toEqual({ v: 1, difficulty: 'easy', prompts: ['move', 'jump'], levels: { '1-1': { clears: 2, best: 1900, grade: 1, seals: 1, time: 0 } } })
    for (const junk of [null, 'save', [], 42]) expect(P.normalize(junk)).toEqual(P.empty())
    expect(P.rememberPrompt(save, 'jump')).toBe(save)
    expect(P.rememberPrompt(save, 'dash').prompts).toEqual(['move', 'jump', 'dash'])
  })
})
