import { useEffect, useState } from 'react'
import type { Checkin, Tip } from '../../api/types.ts'
import { moodFace, moodLabel } from '../../cat/moods.ts'
import { useLocation } from 'wouter'
import { CatWindow, FieldError, PrimaryButton, QuietButton } from '../../components/ui.tsx'
import { useProfile } from '../../lib/session.tsx'
import { countdown } from '../../lib/time.ts'
import { titleTheme } from '../../lib/titleTheme.ts'
import { GoHome } from './GoHome.tsx'
import { ReplyBubble } from './ReplyBubble.tsx'
import { TonightInfo } from './TonightInfo.tsx'

interface ResultViewProps {
  checkin: Checkin
  tips: Tip[] | null
  fresh: boolean
  busy: boolean
  error: string | null
  onUndo: () => void
}

function useUndoLeft(until: number | null) {
  const [left, setLeft] = useState(() => (until ? until - Date.now() : 0))
  useEffect(() => {
    if (!until) return
    const id = setInterval(() => setLeft(until - Date.now()), 1000)
    return () => clearInterval(id)
  }, [until])
  return until ? left : 0
}

export function ResultView({ checkin, tips, fresh, busy, error, onUndo }: ResultViewProps) {
  const profile = useProfile()
  const theme = titleTheme[checkin.title.code]
  const undoLeft = useUndoLeft(checkin.undoUntil)
  const [, navigate] = useLocation()

  return (
    <div className="flex flex-1 flex-col">
      <section className="mt-6" aria-label="今日称号">
        {checkin.title.rare && (
          <span className="mb-2 inline-block rounded-full border border-current px-2.5 py-0.5 text-xs" style={{ color: theme.accent }}>稀有称号</span>
        )}
        <h1
          className={`origin-left -rotate-2 font-display leading-none whitespace-nowrap ${fresh ? 'animate-stamp' : ''}`}
          style={{ color: theme.accent, fontSize: `min(76px, calc((min(100vw, 480px) - 52px) / ${[...checkin.title.name].length}))` }}
        >
          {checkin.title.name}
        </h1>
        <div className="mt-5 flex items-end gap-4">
          <CatWindow spec={moodFace[checkin.mood]} label={`${profile.companionName}${moodLabel[checkin.mood]}`} size={120} centered={false} />
          <p className="pb-5 text-window/80">
            <span className="font-display text-4xl text-window">{checkin.localTime}</span>
            <span className="ml-1.5">下班</span>
          </p>
        </div>
      </section>

      <ReplyBubble checkin={checkin} />
      <TonightInfo checkin={checkin} tips={tips} />
      <GoHome checkin={checkin} />

      <div className="mt-auto pt-8 text-center">
        <PrimaryButton onClick={() => navigate('/card')}>生成今日下班卡</PrimaryButton>
        {undoLeft > 0 && (
          <QuietButton disabled={busy} onClick={onUndo}>
            按错了？撤销打卡（还剩 {countdown(undoLeft)}）
          </QuietButton>
        )}
        <FieldError message={error} />
      </div>
    </div>
  )
}
