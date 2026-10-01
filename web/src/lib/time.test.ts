import { describe, expect, it } from 'vitest'
import { countdown, formatDay, formatDuration, minutesPastWorkEnd } from './time.ts'

describe('time helpers', () => {
  it('formats the header date', () => {
    expect(formatDay(new Date(2026, 9, 2))).toBe('10月2日 周五')
  })

  it('formats durations in Chinese', () => {
    expect(formatDuration(8)).toBe('8 分钟')
    expect(formatDuration(120)).toBe('2 小时')
    expect(formatDuration(133)).toBe('2 小时 13 分')
  })

  it('measures overtime on the 5am-to-5am timeline', () => {
    expect(minutesPastWorkEnd(new Date(2026, 9, 2, 20, 13), '18:00')).toBe(133)
    expect(minutesPastWorkEnd(new Date(2026, 9, 2, 17, 30), '18:00')).toBe(-30)
    expect(minutesPastWorkEnd(new Date(2026, 9, 3, 1, 0), '18:00')).toBe(420)
  })

  it('renders an undo countdown', () => {
    expect(countdown(8 * 60_000 + 21_000)).toBe('8:21')
    expect(countdown(-5)).toBe('0:00')
  })
})
