import { describe, expect, it } from 'vitest'
import type { MetricsResponse, MetricSeries } from './types.ts'
import { byKey, deltaOf, formatDelta, hasAnyData, rangeLabel, sparkHeights, todayOf } from './metrics.ts'

const series = (totals: number[]): MetricSeries => ({
  key: 'k',
  label: 'label',
  points: totals.map((total, i) => ({ day: `2026-10-0${i + 1}`, total, registered: 0, guest: 0 })),
})

const empty: MetricsResponse = {
  days: [], dayNote: '', isLatest: true, visits: [], growth: [], product: [], games: [], quizzes: [],
}

describe('metrics helpers', () => {
  it('finds a series across groups and reads today', () => {
    const groups = [[series([1, 2])], [series([3, 4])]]
    expect(byKey(groups, 'k')?.points).toHaveLength(2)
    expect(todayOf(byKey([[series([5, 7])]], 'k'))?.total).toBe(7)
    expect(todayOf(undefined)).toBeUndefined()
  })

  it('computes the change against yesterday', () => {
    expect(deltaOf(series([4, 7]))).toBe(3)
    expect(deltaOf(series([4, 2]))).toBe(-2)
    expect(deltaOf(series([5]))).toBe(0)
    expect(deltaOf(undefined)).toBe(0)
  })

  it('formats deltas with a sign only when positive', () => {
    expect(formatDelta(3)).toBe('+3')
    expect(formatDelta(-1)).toBe('-1')
    expect(formatDelta(0)).toBe('0')
  })

  it('normalizes sparkline heights and keeps small bars visible', () => {
    expect(sparkHeights([0, 2, 8, 4])).toEqual([0, 0.25, 1, 0.5])
    expect(sparkHeights([0, 1, 2])[1]).toBeCloseTo(0.5)
    expect(sparkHeights([0, 0])).toEqual([0, 0])
  })

  it('detects an empty window and formats the date range', () => {
    expect(hasAnyData(empty)).toBe(false)
    expect(hasAnyData({ ...empty, visits: [series([0, 1])] })).toBe(true)
    expect(rangeLabel(['2026-09-28', '2026-10-04'])).toBe('2026-09-28 ~ 2026-10-04')
  })
})
