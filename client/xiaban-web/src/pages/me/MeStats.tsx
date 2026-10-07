import { useEffect, useState } from 'react'
import { loadGameSummaries } from '../../games/summaries.ts'
import { quizCatalog } from '../../quizzes/catalog.ts'
import { readSavedAttempt } from '../../quizzes/progress.ts'

function Figure({ value, unit, label }: { value: number; unit: string; label: string }) {
  return (
    <div className="text-center">
      <p className="leading-none text-lamp"><span className="font-display text-3xl">{value}</span><span className="ml-0.5 text-xs">{unit}</span></p>
      <p className="mt-1.5 text-xs text-window/60">{label}</p>
    </div>
  )
}

const DAY = 86_400_000

/** Days since you first met 小伴, counting the first day as day 1. */
export function daysTogether(createdAt: number, now = Date.now()) {
  if (!Number.isFinite(createdAt) || createdAt <= 0) return 1
  const start = new Date(createdAt)
  start.setHours(0, 0, 0, 0)
  return Math.max(1, Math.floor((now - start.getTime()) / DAY) + 1)
}

/** What you and 小伴 have done together. Check-in streaks live with the 下班啦 campaign now. */
export function MeStats({ companionName, createdAt, active }: { companionName: string; createdAt: number; active: boolean }) {
  const [rounds, setRounds] = useState(0)
  const [tests, setTests] = useState(0)
  useEffect(() => {
    if (!active) return
    loadGameSummaries().then((list) => setRounds(list.reduce((sum, game) => sum + (game.sessions || 0), 0))).catch(() => undefined)
    setTests(quizCatalog.filter((quiz) => readSavedAttempt(quiz)?.completedAt).length)
  }, [active])

  return (
    <section aria-label="一起做过的事" className="mt-4 rounded-3xl bg-dusk-soft/35 px-4 py-5">
      <div className="grid grid-cols-3 divide-x divide-window/10">
        <Figure value={daysTogether(createdAt)} unit="天" label={`和${companionName}相识`} />
        <Figure value={rounds} unit="局" label="一起玩过" />
        <Figure value={tests} unit="个" label="测过的测试" />
      </div>
    </section>
  )
}
