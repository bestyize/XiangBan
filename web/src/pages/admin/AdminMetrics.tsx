import { useEffect, useMemo, useState } from 'react'
import { adminRequest } from './api.ts'
import type { MetricsResponse, MetricSeries } from './types.ts'
import { byKey, deltaOf, formatDelta, hasAnyData, rangeLabel, sparkHeights, todayOf } from './metrics.ts'
import { Alerts, TextButton } from './ui.tsx'

const RANGES = [7, 30] as const
type Range = (typeof RANGES)[number]

/** 访客 / 打卡 / 分享 — the three lines of the trend chart, on the dark hero. */
const TREND = [
  { key: 'uv', color: '#edbd76' },
  { key: 'checkin', color: '#ff9d7a' },
  { key: 'share', color: '#7fd4e8' },
] as const

function Kpi({ label, series, unit }: { label: string; series: MetricSeries | undefined; unit?: string }) {
  const today = todayOf(series)
  const delta = deltaOf(series)
  return (
    <div className="min-w-0">
      <p className="text-[11px] font-medium tracking-[0.14em] text-white/50">{label}</p>
      <p className="mt-2 font-display text-[42px] leading-none tabular-nums text-[#fff3d6]">
        {today?.total.toLocaleString() ?? 0}
        {unit && <span className="ml-1 font-sans text-sm text-white/45">{unit}</span>}
      </p>
      <p className="mt-2 text-xs">
        <span className={`tabular-nums ${delta > 0 ? 'text-emerald-300' : delta < 0 ? 'text-[#ff9d7a]' : 'text-white/40'}`}>
          {formatDelta(delta)}
        </span>
        <span className="ml-1 text-white/40">比昨天</span>
      </p>
      {today && (today.registered > 0 || today.guest > 0) && (
        <p className="mt-1 text-[11px] tabular-nums text-white/40">注册 {today.registered} · 游客 {today.guest}</p>
      )}
    </div>
  )
}

function TrendChart({ days, series }: { days: string[]; series: (MetricSeries | undefined)[] }) {
  const max = Math.max(1, ...series.flatMap((s) => (s?.points ?? []).map((p) => p.total)))
  const empty = series.every((s) => (s?.points ?? []).every((p) => p.total === 0))
  const showLabel = (i: number) => days.length <= 7 || i % 5 === 0 || i === days.length - 1

  return (
    <div className="mt-3">
      <div className="relative h-44">
        {[1, 0.75, 0.5, 0.25, 0].map((p) => (
          <div key={p} className="absolute inset-x-0 flex items-center gap-3" style={{ bottom: `${p * 100}%` }}>
            <span className="w-7 shrink-0 text-right text-[10px] tabular-nums text-white/35">{Math.round(max * p)}</span>
            <span className={`h-px flex-1 ${p === 0 ? 'bg-white/25' : 'bg-white/10'}`} />
          </div>
        ))}
        <div className="absolute inset-0 left-10 flex items-end gap-1.5">
          {days.map((day, i) => {
            const last = i === days.length - 1
            const tip = `${day.slice(5)}　${series.map((s) => `${s?.label} ${s?.points[i]?.total ?? 0}`).join(' · ')}`
            return (
              <div key={day} title={tip} className="flex h-full flex-1 items-end justify-center gap-[2px]">
                {series.map((s, si) => (
                  <div
                    key={s?.key ?? si}
                    className="w-full max-w-[11px] rounded-t-[3px]"
                    style={{ height: `${((s?.points[i]?.total ?? 0) / max) * 100}%`, background: TREND[si].color, opacity: last ? 1 : 0.5 }}
                  />
                ))}
              </div>
            )
          })}
        </div>
      </div>
      <div className="ml-10 mt-2 flex gap-1.5">
        {days.map((day, i) => (
          <div key={day} className="flex-1 text-center text-[10px] tabular-nums text-white/40">
            {showLabel(i) ? day.slice(5) : ''}
          </div>
        ))}
      </div>
      {empty && <p className="mt-4 text-center text-sm text-white/50">还没有数据，等有人来打卡就有了。</p>}
    </div>
  )
}

function Spark({ values }: { values: number[] }) {
  const heights = sparkHeights(values)
  return (
    <div className="flex h-7 flex-1 items-end gap-[2px]">
      {heights.map((h, i) => (
        <div
          key={i}
          className="flex-1 rounded-t-[2px]"
          style={{ height: `${Math.max(h * 100, h > 0 ? 10 : 2)}%`, background: i === heights.length - 1 ? '#182338' : '#cbd5e1' }}
        />
      ))}
    </div>
  )
}

function DetailGroup({ title, rows }: { title: string; rows: (MetricSeries | undefined)[] }) {
  return (
    <div>
      <h3 className="text-[11px] font-semibold tracking-[0.16em] text-slate-500">{title}</h3>
      <ul className="mt-1">
        {rows.map((s) => {
          const today = todayOf(s)
          return (
            <li key={s?.key} className="flex items-center gap-3 border-t border-slate-100 py-2.5 first:border-t-0">
              <span className="w-28 shrink-0 truncate text-sm text-slate-600">{s?.label}</span>
              <Spark values={(s?.points ?? []).map((p) => p.total)} />
              <span className="w-12 text-right text-sm font-semibold tabular-nums text-slate-900">{today?.total ?? 0}</span>
              <span className="hidden w-24 text-right text-[11px] tabular-nums text-slate-400 sm:block">
                注册 {today?.registered ?? 0} · 游客 {today?.guest ?? 0}
              </span>
            </li>
          )
        })}
      </ul>
    </div>
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
  const groups = useMemo(
    () => [data?.visits ?? [], data?.growth ?? [], data?.product ?? [], data?.games ?? [], data?.quizzes ?? []],
    [data],
  )
  const trend = TREND.map((t) => byKey(groups, t.key))

  return <div className="space-y-6"><Alerts error={error} />
    <div className="flex flex-wrap items-center justify-between gap-3">
      <div>
        <p className="text-sm font-semibold tabular-nums text-slate-900">{rangeLabel(days)}</p>
        <p className="mt-0.5 text-xs text-slate-500">{data?.dayNote ?? '按北京时间统计，每天从凌晨 05:00 开始，与打卡的下班日一致'}。人数按设备去重，同一人当天多次操作只算一次。</p>
      </div>
      <div className="flex items-center gap-2">
        <div className="flex rounded-full border border-slate-200 bg-slate-100 p-1">
          {RANGES.map((n) => (
            <button key={n} type="button" onClick={() => setRange(n)}
              className={`rounded-full px-4 py-1.5 text-sm transition ${range === n ? 'bg-[#182338] font-medium text-white shadow-sm' : 'text-slate-500 hover:text-slate-800'}`}>
              近 {n} 天
            </button>
          ))}
        </div>
        <TextButton disabled={loading} onClick={() => setRevision((n) => n + 1)}>{loading ? '正在刷新…' : '刷新数据'}</TextButton>
      </div>
    </div>

    {/* 今日概览：深色主视觉，大数字 + 每日趋势 */}
    <section className="relative overflow-hidden rounded-[28px] bg-[#182338] px-6 py-7 sm:px-8">
      <div aria-hidden="true" className="pointer-events-none absolute -right-20 -top-24 size-72 rounded-full bg-[#edbd76] opacity-15 blur-3xl" />
      <div aria-hidden="true" className="pointer-events-none absolute inset-x-0 bottom-0 h-1/2 bg-gradient-to-t from-black/35 to-transparent" />
      <div className="relative">
        <p className="text-[11px] font-semibold tracking-[0.22em] text-[#edbd76]">今日概览</p>
        <div className="mt-5 grid grid-cols-2 gap-x-6 gap-y-7 sm:grid-cols-3 lg:grid-cols-5">
          <Kpi label="访客数" series={byKey(groups, 'uv')} />
          <Kpi label="页面访问" series={byKey(groups, 'pv')} unit="次" />
          <Kpi label="打卡人数" series={byKey(groups, 'checkin')} />
          <Kpi label="分享人数" series={byKey(groups, 'share')} />
          <Kpi label="新增注册" series={byKey(groups, 'signup')} />
        </div>

        <div className="relative mt-8 border-t border-white/10 pt-6">
          <div className="flex flex-wrap items-center justify-between gap-3">
            <h3 className="text-sm font-semibold text-white/90">每日趋势</h3>
            <div className="flex items-center gap-4 text-[11px] text-white/60">
              {TREND.map((t, i) => (
                <span key={t.key} className="flex items-center gap-1.5">
                  <span className="size-2 rounded-full" style={{ background: t.color }} />{trend[i]?.label}
                </span>
              ))}
            </div>
          </div>
          <TrendChart days={days} series={trend} />
        </div>
      </div>
    </section>

    {/* 明细：一整块白底，分组 + 迷你趋势 */}
    <section className="rounded-2xl border border-slate-200 bg-white px-6 py-6 sm:px-8">
      <div className="grid gap-x-12 gap-y-8 lg:grid-cols-2">
        <DetailGroup title="下班卡" rows={[
          byKey(groups, 'share'), byKey(groups, 'card_save'), byKey(groups, 'card_share'), byKey(groups, 'card_generate'),
        ]} />
        <DetailGroup title="游戏体验人数" rows={data?.games ?? []} />
        <DetailGroup title="测试体验人数" rows={data?.quizzes ?? []} />
        <DetailGroup title="访问与注册" rows={[byKey(groups, 'pv'), byKey(groups, 'signup')]} />
      </div>
    </section>
    {!loading && data && !hasAnyData(data) && (
      <p className="text-center text-sm text-slate-500">统计从上线这天开始累积，先去打个卡吧。</p>
    )}
  </div>
}
