import { useEffect, useMemo, useState } from 'react'
import { adminRequest } from './api.ts'
import type { MetricsResponse, MetricSeries } from './types.ts'
import { Alerts, panel, TextButton } from './ui.tsx'

const RANGES = [7, 30] as const
type Range = (typeof RANGES)[number]

/** Horizontal bar chart of the days in one series: total, and within it registered vs guest people. */
function Bars({ series }: { series: MetricSeries }) {
  const max = Math.max(1, ...series.points.map((p) => p.total))
  return (
    <ul className="mt-3 space-y-1.5">
      {series.points.map((p) => (
        <li key={p.day} className="flex items-center gap-2">
          <span className="w-12 shrink-0 text-xs tabular-nums text-slate-500">{p.day.slice(5)}</span>
          <span className="relative h-4 flex-1 rounded bg-slate-100">
            <span className="absolute inset-y-0 left-0 rounded bg-slate-500" style={{ width: `${(p.total / max) * 100}%` }} />
            <span className="absolute inset-y-0 left-0 rounded bg-emerald-500" style={{ width: `${(p.registered / max) * 100}%` }} />
          </span>
          <span className="w-12 shrink-0 text-right text-xs tabular-nums text-slate-600">{p.total.toLocaleString()}</span>
        </li>
      ))}
    </ul>
  )
}

function SeriesBlock({ title, hint, series }: { title: string; hint?: string; series: MetricSeries[] }) {
  return (
    <section className={panel}>
      <h2 className="font-semibold">{title}</h2>
      {hint && <p className="mt-1 text-xs text-slate-500">{hint}</p>}
      {series.map((s) => (
        <div key={s.key} className="mt-4">
          <div className="flex items-baseline justify-between">
            <p className="text-sm text-slate-600">{s.label}</p>
            <p className="text-lg font-semibold text-slate-900">{(s.points[s.points.length - 1]?.total ?? 0).toLocaleString()}<span className="ml-1 text-xs font-normal text-slate-400">今日</span></p>
          </div>
          <Bars series={s} />
        </div>
      ))}
    </section>
  )
}

export default function AdminMetrics() {
  const [range, setRange] = useState<Range>(7)
  const [data, setData] = useState<MetricsResponse | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [revision, setRevision] = useState(0)

  useEffect(() => {
    const controller = new AbortController()
    setLoading(true); setError(null)
    adminRequest<MetricsResponse>(`/metrics?days=${range}`, 'GET', undefined, controller.signal)
      .then(setData)
      .catch((e: Error) => { if (e.name !== 'AbortError') setError(e.message) })
      .finally(() => { if (!controller.signal.aborted) setLoading(false) })
    return () => controller.abort()
  }, [range, revision])

  const days = data?.days ?? []
  const daysLabel = useMemo(() => (days.length ? `${days[0]} ~ ${days[days.length - 1]}` : ''), [days])

  return <div className="space-y-6"><Alerts error={error} />
    <div className="flex flex-wrap items-center justify-between gap-3">
      <div className="flex items-center gap-2">
        {RANGES.map((n) => (
          <button key={n} type="button" onClick={() => setRange(n)}
            className={`rounded-full px-4 py-1.5 text-sm ${range === n ? 'bg-slate-900 font-medium text-white' : 'border border-slate-300 text-slate-600 hover:bg-slate-100'}`}>
            近 {n} 天
          </button>
        ))}
        <span className="ml-2 text-xs text-slate-500">{daysLabel}</span>
      </div>
      <TextButton disabled={loading} onClick={() => setRevision((n) => n + 1)}>{loading ? '正在刷新…' : '刷新数据'}</TextButton>
    </div>
    <div className="rounded-lg bg-slate-50 p-4 text-xs leading-relaxed text-slate-500">
      {data?.dayNote ?? '按北京时间统计，每天从凌晨 05:00 开始，与打卡的下班日一致'}。人数按匿名设备标识去重，同一人当天多次操作只算一次。
      <span className="ml-2 inline-flex items-center gap-1"><span className="inline-block size-2 rounded-full bg-slate-500" />合计</span>
      <span className="ml-2 inline-flex items-center gap-1"><span className="inline-block size-2 rounded-full bg-emerald-500" />注册用户</span>
    </div>

    <div className="grid gap-6 lg:grid-cols-2">
      <SeriesBlock title="网站访问" hint="页面访问次数与去重访客数" series={data?.visits ?? []} />
      <SeriesBlock title="新增注册" hint="当天完成邮箱验证的用户数" series={data?.growth ?? []} />
      <SeriesBlock title="产品使用" hint="打卡与分享下班卡的人数" series={data?.product ?? []} />
      <SeriesBlock title="游戏体验人数" hint="每个游戏当天有开始对局的人数" series={data?.games ?? []} />
      <SeriesBlock title="测试体验人数" hint="每个测试当天有开始作答的人数" series={data?.quizzes ?? []} />
    </div>
    {!loading && !data && <p className="text-slate-500">暂无数据。</p>}
  </div>
}
