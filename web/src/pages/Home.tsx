import { PrimaryButton, Screen } from '../components/ui.tsx'
import { useProfile } from '../lib/session.tsx'
import { formatDay } from '../lib/time.ts'
import { IdleView } from './home/IdleView.tsx'
import { ResultView } from './home/ResultView.tsx'
import { useToday } from './home/useToday.ts'

export default function Home() {
  const profile = useProfile()
  const { status, today, error, busy, fresh, checkIn, undo, reload } = useToday()
  const name = profile.nickname ?? '你'

  return (
    <Screen>
      <header className="flex h-10 items-center justify-between">
        <span className="font-display text-2xl text-lamp">下班了么</span>
        <span className="text-sm text-window/70">{formatDay(new Date())}</span>
      </header>

      {status === 'loading' && <p className="mt-24 text-center text-window/60">小伴在翻今天的记录…</p>}
      {status === 'error' && (
        <div className="mt-24 text-center">
          <p className="text-window/80">{error}</p>
          <PrimaryButton className="mt-6 max-w-60" onClick={reload}>再试一次</PrimaryButton>
        </div>
      )}
      {status === 'ready' && today && (today.checkin ? (
        <ResultView checkin={today.checkin} fresh={fresh} busy={busy} error={error} onUndo={undo} />
      ) : (
        <IdleView greeting={`${name}，今天也辛苦了。下班了就按一下。`} busy={busy} error={error} onCheckIn={checkIn} />
      ))}
    </Screen>
  )
}
