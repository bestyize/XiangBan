import { useEffect, useState } from 'react'
import { api } from '../../api/client.ts'
import type { Stats } from '../../api/types.ts'
import { formatDuration } from '../../lib/time.ts'

function Figure({ value, label }: { value: number; label: string }) {
  return (
    <div className="text-center">
      <p className="font-display text-3xl leading-none text-lamp">{value}</p>
      <p className="mt-1.5 text-xs text-window/60">{label}</p>
    </div>
  )
}

/** PRD 5.5/5.9 numbers: 相伴天数 never resets, a broken streak shows as "上次连续" without blame. */
export function MeStats({ companionName, active }: { companionName: string; active: boolean }) {
  const [stats, setStats] = useState<Stats | null>(null)
  useEffect(() => {
    if (active) api<Stats>('/api/stats').then(setStats).catch(() => undefined)
  }, [active])

  const streakLabel = stats && stats.currentStreak === 0 && stats.lastStreak ? `上次连续 ${stats.lastStreak} 天` : '连续打卡'
  const detail = stats && [
    stats.weekAvgOffTime && `本周平均 ${stats.weekAvgOffTime} 下班`,
    stats.monthOvertimeMinutes > 0 && `本月加班 ${formatDuration(stats.monthOvertimeMinutes)}`,
  ].filter(Boolean).join('，')

  return (
    <section aria-label="统计" className="mt-4 rounded-3xl bg-dusk-soft/35 px-4 py-5">
      <div className="grid grid-cols-3 divide-x divide-window/10">
        <Figure value={stats?.totalDays ?? 0} label={`和${companionName}相伴`} />
        <Figure value={stats?.currentStreak ?? 0} label={streakLabel} />
        <Figure value={stats?.bestStreak ?? 0} label="最长连续" />
      </div>
      {detail && <p className="mt-4 text-center text-xs text-window/55">{detail}</p>}
    </section>
  )
}
