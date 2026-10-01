import { useEffect, useState } from 'react'
import { api } from '../../api/client.ts'
import type { Stats, TitleCode } from '../../api/types.ts'
import { formatDuration } from '../../lib/time.ts'

const TITLE_NAMES: Record<TitleCode, string> = {
  exact_on_time: '准点之神', weekend_work: '周末还上班？？', borrowed_weekend: '被借走的周末', holiday_countdown: '放假倒计时',
  early_leaver: '早退王者', on_time: '准点战神', normal_human: '正常人类', overtime_hero: '加班侠', night_owl: '夜行动物', immortal: '修仙者',
}

function Figure({ value, label }: { value: string; label: string }) {
  return (
    <div>
      <p className="font-display text-3xl text-lamp">{value}</p>
      <p className="mt-0.5 text-xs text-window/60">{label}</p>
    </div>
  )
}

/** PRD 5.5/5.9 numbers: 相伴天数 never resets, a broken streak shows as "上次连续" without blame. */
export function MeStats({ companionName }: { companionName: string }) {
  const [stats, setStats] = useState<Stats | null>(null)
  useEffect(() => {
    api<Stats>('/api/stats').then(setStats).catch(() => setStats(null))
  }, [])
  if (!stats) return <p className="text-sm text-window/50">还没有数据，下班打个卡就有了。</p>

  const streak = stats.currentStreak > 0 ? `${stats.currentStreak}` : '0'
  const topTitles = Object.entries(stats.titleCounts)
    .sort((a, b) => (b[1] ?? 0) - (a[1] ?? 0))
    .slice(0, 3)
    .map(([code, n]) => `${TITLE_NAMES[code as TitleCode]} ${n} 次`)

  return (
    <div>
      <div className="grid grid-cols-3 gap-3">
        <Figure value={`${stats.totalDays}`} label={`和${companionName}相伴的天数`} />
        <Figure value={streak} label={stats.currentStreak > 0 ? '连续打卡天数' : stats.lastStreak ? `上次连续 ${stats.lastStreak} 天` : '连续打卡天数'} />
        <Figure value={`${stats.bestStreak}`} label="最长连续" />
      </div>
      <dl className="mt-4 space-y-1.5 text-sm text-window/80">
        {stats.weekAvgOffTime && <div className="flex justify-between"><dt>本周平均下班</dt><dd>{stats.weekAvgOffTime}</dd></div>}
        <div className="flex justify-between"><dt>本月累计加班</dt><dd>{formatDuration(stats.monthOvertimeMinutes)}</dd></div>
        {topTitles.length > 0 && <div className="flex justify-between gap-4"><dt className="shrink-0">最常拿到</dt><dd className="text-right">{topTitles.join('，')}</dd></div>}
      </dl>
    </div>
  )
}
