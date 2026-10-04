import type { MetricPoint, MetricSeries, MetricsResponse } from './types.ts'

/** Pure helpers for the dashboard, kept here so the numbers can be unit-tested. */

export function byKey(groups: MetricSeries[][], key: string): MetricSeries | undefined {
  for (const group of groups) {
    const found = group.find((s) => s.key === key)
    if (found) return found
  }
  return undefined
}

export function todayOf(series: MetricSeries | undefined): MetricPoint | undefined {
  return series?.points[series.points.length - 1]
}

/** Today's total minus yesterday's; 0 when there is no yesterday to compare against. */
export function deltaOf(series: MetricSeries | undefined): number {
  const today = todayOf(series)
  const yesterday = series?.points[series.points.length - 2]
  if (!today || !yesterday) return 0
  return today.total - yesterday.total
}

export function formatDelta(delta: number): string {
  return delta > 0 ? `+${delta}` : `${delta}`
}

/** Bar heights for a sparkline, normalized to 0–1; a non-zero value never shrinks to an invisible bar. */
export function sparkHeights(values: number[]): number[] {
  const max = Math.max(0, ...values)
  return values.map((v) => {
    if (v <= 0) return 0
    return max <= 0 ? 1 : Math.max(v / max, 0.08)
  })
}

export function hasAnyData(data: MetricsResponse): boolean {
  return [...data.visits, ...data.growth, ...data.product, ...data.games, ...data.quizzes]
    .some((s) => s.points.some((p) => p.total > 0))
}

export function rangeLabel(days: string[]): string {
  return days.length ? `${days[0]} ~ ${days[days.length - 1]}` : ''
}
