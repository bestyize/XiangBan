import { useEffect, useState } from 'react'
import { NavBar } from '../../components/SubPage.tsx'
import { UnlockOverlay } from '../../components/UnlockOverlay.tsx'
import { PrimaryButton, Screen } from '../../components/ui.tsx'
import { DexSheet } from '../home/DexSheet.tsx'
import { IdleView } from '../home/IdleView.tsx'
import { ResultView } from '../home/ResultView.tsx'
import { useToday } from '../home/useToday.ts'

/**
 * 下班啦: the daily check-in campaign (title, 小伴's teasing, tonight's info, share card).
 * Shared cards land here; going back always ends on the 相伴 home page (see main.tsx).
 */
export default function CheckinActivity() {
  const { status, today, tips, unlocked, dismissUnlocked, error, busy, fresh, checkIn, undo, reload } = useToday(true)
  const [dexOpen, setDexOpen] = useState(false)
  useEffect(() => { document.title = '下班啦｜相伴' }, [])

  return (
    <Screen className="app-subpage">
      <NavBar title="下班啦" action={
        <button type="button" onClick={() => setDexOpen(true)} aria-haspopup="dialog" aria-label="表情图鉴"
          className="grid size-11 place-items-center rounded-xl text-lamp transition active:scale-95 active:bg-lamp/10">
          <svg aria-hidden="true" viewBox="0 0 24 24" className="size-6" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round"><path d="M12 5v15M12 5C9 3 5 3 3 4v15c2-1 6-1 9 1 3-2 7-2 9-1V4c-2-1-6-1-9 1Z" /></svg>
        </button>
      } />
      <div className="flex flex-1 flex-col" style={{ paddingBottom: 'calc(var(--safe-bottom) + 16px)' }}>
        {status === 'loading' && <p className="mt-24 text-center text-window/60">小伴在翻今天的记录…</p>}
        {status === 'error' && (
          <div className="mt-24 text-center">
            <p className="text-window/80">{error}</p>
            <PrimaryButton className="mt-6 max-w-60" onClick={reload}>再试一次</PrimaryButton>
          </div>
        )}
        {status === 'ready' && today && (today.checkin ? (
          <ResultView checkin={today.checkin} tips={tips} fresh={fresh} busy={busy} error={error} onUndo={undo} />
        ) : (
          <IdleView greeting={today.greeting} busy={busy} error={error} onCheckIn={checkIn} />
        ))}
      </div>
      {unlocked.length > 0 && <UnlockOverlay unlocks={unlocked} onDone={dismissUnlocked} />}
      <DexSheet open={dexOpen} onClose={() => setDexOpen(false)} />
    </Screen>
  )
}
