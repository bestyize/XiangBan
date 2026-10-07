import { describe, expect, it } from 'vitest'
import { daysTogether } from './MeStats.tsx'

describe('daysTogether', () => {
  it('counts the day you met as day 1, by calendar day', () => {
    const met = new Date(2026, 9, 1, 23, 50).getTime()
    expect(daysTogether(met, new Date(2026, 9, 1, 23, 59).getTime())).toBe(1)
    expect(daysTogether(met, new Date(2026, 9, 2, 0, 5).getTime())).toBe(2)
    expect(daysTogether(met, new Date(2026, 9, 6, 12).getTime())).toBe(6)
  })

  it('falls back to day 1 for a missing date', () => {
    expect(daysTogether(0)).toBe(1)
    expect(daysTogether(Number.NaN)).toBe(1)
  })
})
