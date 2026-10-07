const WEEKDAYS = '日一二三四五六'

export function formatDay(date: Date): string {
  return `${date.getMonth() + 1}月${date.getDate()}日 周${WEEKDAYS[date.getDay()]}`
}

export function formatDuration(minutes: number): string {
  const m = Math.max(0, Math.round(minutes))
  const h = Math.floor(m / 60)
  const rest = m % 60
  if (h === 0) return `${rest} 分钟`
  return rest === 0 ? `${h} 小时` : `${h} 小时 ${rest} 分`
}

function minutesOf(hhmm: string): number {
  const [h, m] = hhmm.split(':').map(Number)
  return h * 60 + m
}

/** Minutes on the 下班日 timeline (05:00 to 05:00), matching the server's rule. */
function workDayMinute(date: Date): number {
  const m = date.getHours() * 60 + date.getMinutes()
  return (m - 300 + 1440) % 1440
}

/** Positive once past the configured off-work time. */
export function minutesPastWorkEnd(now: Date, workEnd: string): number {
  const end = (minutesOf(workEnd) - 300 + 1440) % 1440
  return workDayMinute(now) - end
}

export function countdown(ms: number): string {
  const total = Math.max(0, Math.ceil(ms / 1000))
  return `${Math.floor(total / 60)}:${String(total % 60).padStart(2, '0')}`
}
