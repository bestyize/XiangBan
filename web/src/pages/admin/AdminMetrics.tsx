import { useEffect, useMemo, useRef, useState } from 'react'
import { adminRequest } from './api.ts'
import type { MetricsResponse, MetricSeries } from './types.ts'
import { byKey, deltaOf, formatDelta, rangeLabel, todayOf } from './metrics.ts'
import { Alerts, TextButton } from './ui.tsx'

const RANGES = [7, 30] as const
type Range = (typeof RANGES)[number]

const GREEN = '#07c160'

/** The three lines of the trend chart, in the WeChat-console manner. */
const TREND = [
  { key: 'uv', color: GREEN },
  { key: 'checkin', color: '#576b95' },
  { key: 'share', color: '#10aeff' },
] as const

/* ---------- 关键指标卡片 ---------- */

function KpiCard({ label, series }: { label: string; series: MetricSeries | undefined }) {
  const today = todayOf(series)
  const delta = deltaOf(series)
  return (
    <div className="rounded-lg border border-[#ededed] bg-white px-5 py-4 shadow-[0_1px_2px_rgba(0,0,0,0.03)]">
      <p className="text-xs text-[#999]">{label}</p>
      <p className="mt-1.5 text-[28px] font-bold leading-none tabular-nums text-[#353535]">{today?.total.toLocaleString() ?? 0}</p>
      <p className="mt-2 text-xs text-[#999]">
        较昨日
        <span className={`ml-1.5 font-medium tabular-nums ${delta > 0 ? 'text-[#07c160]' : delta < 0 ? 'text-[#fa5151]' : 'text-[#b2b2b2]'}`}>
          {formatDelta(delta)}{delta > 0 ? ' ↑' : delta < 0 ? ' ↓' : ''}
        </span>
      </p>
      {today && (today.registered > 0 || today.guest > 0) && (
        <p className="mt-1 text-xs tabular-nums text-[#b2b2b2]">注册 {today.registered} · 游客 {today.guest}</p>
      )}
    </div>
  )
}

/* ---------- 趋势折线图 ---------- */

const VIEW_W = 720
const VIEW_H = 240
const PAD = { l: 40, r: 12, t: 14, b: 30 }

/** Catmull-Rom → cubic bezier, so the line reads like the WeChat console charts. */
function smoothPath(pts: { x: number; y: number }[]): string {
  if (pts.length === 0) return ''
  let d = `M${pts[0].x},${pts[0].y}`
  for (let i = 0; i < pts.length - 1; i++) {
    const p0 = pts[Math.max(0, i - 1)]
    const p1 = pts[i]
    const p2 = pts[i + 1]
    const p3 = pts[Math.min(pts.length - 1, i + 2)]
    d += ` C${p1.x + (p2.x - p0.x) / 6},${p1.y + (p2.y - p0.y) / 6} ${p2.x - (p3.x - p1.x) / 6},${p2.y - (p3.y - p1.y) / 6} ${p2.x},${p2.y}`
  }
  return d
}

function TrendChart({ days, series }: { days: string[]; series: (MetricSeries | undefined)[] }) {
  const [hidden, setHidden] = useState<Record<string, boolean>>({})
  const [hover, setHover] = useState<number | null>(null)
  const wrap = useRef<HTMLDivElement>(null)

  const shown = series.filter((s) => s && !hidden[s.key])
  const max = Math.max(1, ...shown.flatMap((s) => (s?.points ?? []).map((p) => p.total)))
  const step = days.length > 1 ? (VIEW_W - PAD.l - PAD.r) / (days.length - 1) : 0
  const xOf = (i: number) => PAD.l + i * step
  const yOf = (v: number) => PAD.t + (1 - v / max) * (VIEW_H - PAD.t - PAD.b)

  return (
    <div>
      <div className="flex flex-wrap items-center gap-5 pb-3 text-xs text-[#7f7f7f]">
        {series.map((s) => !s ? null : (
          <button key={s.key} type="button" onClick={() => setHidden((h) => ({ ...h, [s.key]: !h[s.key] }))}
            className={`flex items-center gap-1.5 ${hidden[s.key] ? 'opacity-40' : ''}`}>
            <span className="inline-block size-2 rounded-full" style={{ background: TREND.find((t) => t.key === s.key)?.color }} />
            {s.label}
          </button>
        ))}
      </div>
      <div ref={wrap} className="relative" onMouseLeave={() => setHover(null)}
        onMouseMove={(e) => {
          const rect = e.currentTarget.getBoundingClientRect()
          const x = ((e.clientX - rect.left) / rect.width) * VIEW_W
          const idx = days.length > 1 ? Math.round((x - PAD.l) / step) : 0
          setHover(Math.min(Math.max(idx, 0), days.length - 1))
        }}>
        <svg viewBox={`0 0 ${VIEW_W} ${VIEW_H}`} className="h-[240px] w-full">
          {[0, 0.25, 0.5, 0.75, 1].map((p) => (
            <g key={p}>
              <line x1={PAD.l} x2={VIEW_W - PAD.r} y1={yOf(max * p)} y2={yOf(max * p)} stroke={p === 0 ? '#e5e5e5' : '#f2f2f2'} strokeDasharray={p === 0 ? undefined : '3 3'} />
              <text x={PAD.l - 8} y={yOf(max * p) + 4} textAnchor="end" fontSize="11" fill="#b2b2b2">{Math.round(max * p)}</text>
            </g>
          ))}
          {days.map((day, i) => (days.length <= 7 || i % Math.ceil(days.length / 7) === 0 || i === days.length - 1) && (
            <text key={day} x={xOf(i)} y={VIEW_H - 8} textAnchor="middle" fontSize="11" fill="#b2b2b2">{day.slice(5)}</text>
          ))}
          {shown.map((s) => {
            const color = TREND.find((t) => t.key === s!.key)?.color ?? GREEN
            const pts = s!.points.map((p, i) => ({ x: xOf(i), y: yOf(p.total) }))
            return (
              <g key={s!.key}>
                <path d={`${smoothPath(pts)} L${xOf(pts.length - 1)},${yOf(0)} L${xOf(0)},${yOf(0)} Z`} fill={color} opacity="0.08" />
                <path d={smoothPath(pts)} fill="none" stroke={color} strokeWidth="2.5" strokeLinecap="round" />
                {hover !== null && <circle cx={pts[hover]?.x} cy={pts[hover]?.y} r="4.5" fill="#fff" stroke={color} strokeWidth="2.5" />}
              </g>
            )
          })}
          {hover !== null && (
            <line x1={xOf(hover)} x2={xOf(hover)} y1={PAD.t} y2={yOf(0)} stroke="#e5e5e5" />
          )}
        </svg>
        {hover !== null && (
          <div className="pointer-events-none absolute top-2 rounded-md border border-[#ededed] bg-white px-3 py-2 text-xs shadow-[0_2px_8px_rgba(0,0,0,0.08)]"
            style={{ left: `${(xOf(hover) / VIEW_W) * 100}%`, transform: `translateX(${xOf(hover) > VIEW_W * 0.65 ? '-105%' : '10px'})` }}>
            <p className="font-medium text-[#353535]">{days[hover].slice(5)}</p>
            {series.map((s, i) => s && (
              <p key={s.key} className="mt-0.5 tabular-nums text-[#7f7f7f]">
                <span className="mr-1.5 inline-block size-2 rounded-full align-middle" style={{ background: TREND[i].color }} />
                {s.label} {s.points[hover]?.total ?? 0}
              </p>
            ))}
          </div>
        )}
      </div>
    </div>
  )
}

/* ---------- 明细表格 ---------- */

function DataTable({ title, rows }: { title: string; rows: (MetricSeries | undefined)[] }) {
  return (
    <section>
      <h3 className="flex items-center gap-2 pb-3 pt-1 text-sm font-bold text-[#353535]">
        <span className="inline-block h-3.5 w-[3px] rounded-full bg-[#07c160]" />{title}
      </h3>
      <div className="overflow-x-auto rounded-lg border border-[#ededed]">
        <table className="w-full min-w-[520px] border-collapse bg-white text-sm">
          <thead>
            <tr className="bg-[#fafafa] text-xs text-[#999]">
              <th className="border-b border-[#f2f2f2] px-4 py-2.5 text-left font-normal">日期</th>
              {rows.map((s) => <th key={s?.key} className="border-b border-[#f2f2f2] px-4 py-2.5 text-right font-normal">{s?.label}</th>)}
            </tr>
          </thead>
          <tbody className="tabular-nums text-[#353535]">
            {(rows[0]?.points ?? []).map((p, i) => (
              <tr key={p.day} className="transition-colors hover:bg-[#f7f8fa]">
                <td className="border-b border-[#f2f2f2] px-4 py-2.5 text-[#7f7f7f]">{p.day.slice(5)}</td>
                {rows.map((s) => <td key={s?.key} className="border-b border-[#f2f2f2] px-4 py-2.5 text-right">{s?.points[i]?.total ?? 0}</td>)}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </section>
  )
}

/* ---------- 页面 ---------- */

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

  const groups = useMemo(
    () => [data?.visits ?? [], data?.growth ?? [], data?.product ?? [], data?.games ?? [], data?.quizzes ?? []],
    [data],
  )
  const pick = (key: string) => byKey(groups, key)

  return <div className="space-y-7"><Alerts error={error} />
    <div className="flex flex-wrap items-end justify-between gap-3">
      <div>
        <h2 className="flex items-center gap-2 text-base font-bold text-[#353535]">
          <span className="inline-block h-4 w-[3px] rounded-full bg-[#07c160]" />数据概览
        </h2>
        <p className="mt-1 text-xs text-[#b2b2b2]">{rangeLabel(data?.days ?? [])} · 按北京时间 05:00 起算，同一人当天多次操作只算一次</p>
      </div>
      <div className="flex items-center gap-5">
        <div className="flex items-center gap-4 text-sm">
          {RANGES.map((n) => (
            <button key={n} type="button" onClick={() => setRange(n)}
              className={`border-b-2 pb-1 ${range === n ? 'border-[#07c160] font-medium text-[#07c160]' : 'border-transparent text-[#7f7f7f] hover:text-[#353535]'}`}>
              近 {n} 天
            </button>
          ))}
        </div>
        <TextButton disabled={loading} onClick={() => setRevision((n) => n + 1)}>{loading ? '正在刷新…' : '刷新数据'}</TextButton>
      </div>
    </div>

    <div className="grid grid-cols-2 gap-4 lg:grid-cols-4">
      <KpiCard label="访客数" series={pick('uv')} />
      <KpiCard label="打卡人数" series={pick('checkin')} />
      <KpiCard label="分享人数" series={pick('share')} />
      <KpiCard label="新增注册" series={pick('signup')} />
    </div>

    <section>
      <h2 className="flex items-center gap-2 pb-1 text-sm font-bold text-[#353535]">
        <span className="inline-block h-3.5 w-[3px] rounded-full bg-[#07c160]" />趋势分析
      </h2>
      <div className="rounded-lg border border-[#ededed] bg-white px-5 py-4 shadow-[0_1px_2px_rgba(0,0,0,0.03)]">
        <TrendChart days={data?.days ?? []} series={TREND.map((t) => pick(t.key))} />
      </div>
    </section>

    <div className="space-y-6">
      <DataTable title="详细数据" rows={[pick('uv'), pick('pv'), pick('checkin'), pick('share'), pick('signup')]} />
      <DataTable title="游戏体验人数" rows={data?.games ?? []} />
      <DataTable title="测试体验人数" rows={data?.quizzes ?? []} />
    </div>
  </div>
}
