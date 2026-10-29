export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / 1024 / 1024).toFixed(1)} MB`
  return `${(bytes / 1024 / 1024 / 1024).toFixed(2)} GB`
}

/** "45 秒", "3 分 20 秒", "2 小时 5 分", "3 天". */
export function formatDuration(totalSecs: number): string {
  const secs = Math.max(0, Math.round(totalSecs))
  if (secs < 60) return `${secs} 秒`
  if (secs < 3600) {
    const m = Math.floor(secs / 60)
    const s = secs % 60
    return s ? `${m} 分 ${s} 秒` : `${m} 分`
  }
  if (secs < 86400) {
    const h = Math.floor(secs / 3600)
    const m = Math.floor((secs % 3600) / 60)
    return m ? `${h} 小时 ${m} 分` : `${h} 小时`
  }
  const d = Math.floor(secs / 86400)
  const h = Math.floor((secs % 86400) / 3600)
  return h ? `${d} 天 ${h} 小时` : `${d} 天`
}

export function formatPercent(ratio: number | null): string {
  return ratio === null ? '—' : `${(ratio * 100).toFixed(ratio === 1 || ratio === 0 ? 0 : 1)}%`
}

export const TTL_UNITS = [
  { label: '秒', secs: 1 },
  { label: '分钟', secs: 60 },
  { label: '小时', secs: 3600 },
  { label: '天', secs: 86400 },
] as const

/** Splits a lifetime into the largest unit that divides it evenly, for showing in a number + unit field. */
export function splitTtl(totalSecs: number): { value: number; unit: number } {
  for (const unit of [...TTL_UNITS].reverse()) {
    if (totalSecs % unit.secs === 0) return { value: totalSecs / unit.secs, unit: unit.secs }
  }
  return { value: totalSecs, unit: 1 }
}
