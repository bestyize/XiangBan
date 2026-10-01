import type { Checkin, Tip } from '../../api/types.ts'
import { TipIcon } from '../../components/TipIcon.tsx'

interface TonightInfoProps {
  checkin: Checkin
  tips: Tip[] | null
}

function percentileLine(checkin: Checkin): string | null {
  const p = checkin.stats.percentile
  if (!p) return null
  const where = p.scope === 'city' && p.cityName ? p.cityName : ''
  return `你比${where} ${p.percent}% 的下班了么用户下班早`
}

/** PRD 5.5 practical info plus the day's numbers; tips arrive a moment after the title. */
export function TonightInfo({ checkin, tips }: TonightInfoProps) {
  const { streak, totalDays } = checkin.stats
  const percentile = percentileLine(checkin)

  return (
    <section className="mt-4" aria-label="今晚用得上的信息">
      {tips === null ? (
        <p className="py-3 text-sm text-window/55">小伴在看今晚的天气…</p>
      ) : tips.length > 0 ? (
        <ul className="divide-y divide-window/10 rounded-2xl bg-dusk-soft/35 px-4">
          {tips.map((tip) => (
            <li key={tip.kind} className="flex items-center gap-3 py-3 text-[15px] text-window/90">
              <span className="text-lamp"><TipIcon kind={tip.kind} /></span>
              {tip.text}
            </li>
          ))}
        </ul>
      ) : null}

      <div className="mt-4 flex flex-wrap items-baseline gap-x-4 gap-y-1 text-sm text-window/70">
        {percentile && <p className="w-full text-base text-window">{percentile}</p>}
        {streak > 1 && <p>已经连续打卡 <span className="font-display text-lg text-lamp">{streak}</span> 天</p>}
        <p>相伴第 <span className="font-display text-lg text-lamp">{totalDays}</span> 天</p>
      </div>
    </section>
  )
}
